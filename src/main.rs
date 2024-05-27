use iced::{
    event::{self, Event},
    executor, font,
    keyboard::{self, key, Key},
    theme, widget,
    widget::{column, container, horizontal_space, row, text, vertical_rule, Container, Row},
    window, Application, Command, Element, Font, Length, Settings, Subscription, Theme,
};

use iced_aw::native::menu::Item;

use std::path::PathBuf;

mod styles;
use styles::*;

mod utils;
use utils::*;

mod views;
use views::{home_view, EditorTabData, LineTabData, Refresh, Tabs, TabsMessage, View, ViewType};

mod widgets;
use widgets::{
    modal::Modal,
    toast::{self, Status, Toast},
    wizard::{LineConfigState, Wizard},
};

fn main() -> Result<(), iced::Error> {
    let window = window::settings::Settings {
        exit_on_close_request: false,
        ..Default::default()
    };
    Modav::run(Settings {
        window,
        antialiasing: true,
        ..Default::default()
    })
}

/// What should be done after a file IO action
#[derive(Debug, Clone)]
pub enum FileIOAction {
    /// Create a new tab
    NewTab((View, PathBuf)),
    /// Refresh an existing tab
    RefreshTab((ViewType, usize, PathBuf)),
    /// Close a specific tab
    CloseTab(usize),
    /// Exiting main app
    Exiting(usize),
    /// Do nothing
    None,
}

impl FileIOAction {
    fn update_path(self, path: PathBuf) -> Self {
        match self {
            Self::None => Self::None,
            Self::NewTab((tidn, _)) => Self::NewTab((tidn, path)),
            Self::RefreshTab((tidn, id, _)) => Self::RefreshTab((tidn, id, path)),
            Self::CloseTab(id) => Self::CloseTab(id),
            Self::Exiting(id) => Self::Exiting(id),
        }
    }
}

pub struct Modav {
    theme: Theme,
    title: String,
    current_view: ViewType,
    file_path: Option<PathBuf>,
    tabs: Tabs,
    toasts: Vec<Toast>,
    wizard_shown: bool,
    error: AppError,
}

#[derive(Debug, Clone)]
pub enum Message {
    IconLoaded(Result<(), font::Error>),
    ToggleTheme,
    SelectFile,
    FileSelected(Result<PathBuf, AppError>),
    LoadFile((PathBuf, FileIOAction)),
    FileLoaded((Result<String, AppError>, FileIOAction)),
    SaveFile((Option<PathBuf>, String, FileIOAction)),
    SaveKeyPressed,
    FileSaved((Result<(PathBuf, String), AppError>, FileIOAction)),
    Convert,
    None,
    Event(Event),
    CheckExit,
    CanExit,
    OpenTab(Option<PathBuf>, View),
    NewActiveTab,
    TabsMessage(TabsMessage),
    Debugging,
    WizardSubmit(PathBuf, View),
    ToggleWizardShown,
    AddToast(Toast),
    CloseToast(usize),
    Error(AppError),
}

impl Modav {
    fn status_bar(&self) -> Container<'_, Message> {
        let current: Row<'_, Message> = {
            let path = {
                self.tabs.active_path().as_ref().and_then(|path| {
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .filter(|name| !name.is_empty())
                        .map(|file| {
                            let text = text(file);
                            let icon = status_icon('\u{F0F6}');
                            row!(icon, text).spacing(5)
                        })
                })
            };

            match (path, &self.current_view) {
                (None, ViewType::None) => row!(),
                (None, vt) => {
                    let txt = text("Untitled");
                    let icon = status_icon('\u{F0F6}');
                    let txt = row!(icon, txt).spacing(5);
                    row!(vt.display(), vertical_rule(10), txt)
                }
                (Some(p), ViewType::None) => row!(p),
                (Some(p), m) => row!(m.display(), vertical_rule(10), p),
            }
        }
        .spacing(10);

        let row: Row<'_, Message> = row!(horizontal_space(), current);

        let bstyle = BorderedContainer::default();

        container(row)
            .padding([3, 10])
            .style(theme::Container::Custom(Box::new(bstyle)))
    }

    /// Creates a save message for the current tab. Assumes the path is valid
    /// for current tab. Handles empty tabs situation
    /// Refreshes the current tab
    fn save_helper(&self, save_path: Option<PathBuf>) -> Message {
        let iden = self.tabs.active_tab_type();

        if self.tabs.is_empty() && iden.is_none() {
            return Message::None;
        }

        let content = self.tabs.active_content().unwrap_or(String::default());

        let action = FileIOAction::RefreshTab((
            iden.unwrap(),
            self.tabs.active_tab(),
            self.tabs.active_path().unwrap_or(PathBuf::new()),
        ));

        Message::SaveFile((save_path, content, action))
    }

    fn file_menu(&self) -> Container<'_, Message> {
        let actions_label = vec![
            (
                "New File",
                Message::OpenTab(None, View::Editor(EditorTabData::default())),
            ),
            ("Open File", Message::SelectFile),
            ("Save File", self.save_helper(self.tabs.active_path())),
            ("Save As", self.save_helper(None)),
        ];

        let children: Vec<Item<'_, Message, _, _>> = menus::create_children(actions_label);

        let bar = menus::create_menu("File", '\u{F15C}', children);

        menus::container_wrap(bar)
    }

    fn dashboard(&self) -> Container<'_, Message> {
        let font = Font {
            family: font::Family::Cursive,
            weight: font::Weight::Bold,
            ..Default::default()
        };
        let logo = container(text("modav").font(font).size(24))
            .padding([0, 8])
            .width(Length::Fixed(125.0));

        let menus = {
            let views_closure = |vt| match vt {
                ViewType::Counter => Message::OpenTab(None, View::Counter),
                ViewType::Editor => {
                    let path = self.file_path.clone();
                    let data = EditorTabData::new(path, String::default());
                    Message::OpenTab(self.file_path.clone(), View::Editor(data))
                }
                ViewType::LineGraph => Message::None,
                ViewType::None => Message::None,
            };
            column!(
                self.file_menu(),
                menus::models_menu(),
                menus::views_menu(views_closure),
                menus::about_menu(),
                menus::settings_menu()
            )
            .spacing(45)
        };

        let content = column!(logo, menus).spacing(80);

        let bstyle = BorderedContainer::default();
        container(content)
            .center_x()
            .padding([15, 0])
            .width(Length::FillPortion(1))
            .height(Length::Fill)
            .style(theme::Container::Custom(Box::new(bstyle)))
    }

    fn toggle_theme(&mut self) {
        match self.theme {
            Theme::Dark => self.theme = Theme::Light,
            Theme::Light => self.theme = Theme::Dark,
            Theme::SolarizedLight => self.theme = Theme::SolarizedDark,
            Theme::SolarizedDark => self.theme = Theme::SolarizedLight,
            Theme::GruvboxLight => self.theme = Theme::GruvboxDark,
            Theme::GruvboxDark => self.theme = Theme::GruvboxLight,
            Theme::TokyoNight => self.theme = Theme::TokyoNightLight,
            Theme::TokyoNightLight => self.theme = Theme::TokyoNight,
            _ => {}
        }
    }

    fn update_tabs(&mut self, tsg: TabsMessage) -> Command<Message> {
        if let Some(response) = self.tabs.update(tsg) {
            Command::perform(async { response }, |response| response)
        } else {
            Command::none()
        }
    }

    fn file_io_action_handler(
        &mut self,
        action: FileIOAction,
        content: String,
    ) -> Command<Message> {
        match action {
            FileIOAction::NewTab((View::Counter, _)) => {
                let idr = View::Counter;

                self.update_tabs(TabsMessage::AddTab(idr))
            }
            FileIOAction::NewTab((View::Editor(_), path)) => {
                let data = EditorTabData::new(Some(path), content);
                let idr = View::Editor(data);
                self.update_tabs(TabsMessage::AddTab(idr))
            }
            FileIOAction::NewTab((View::LineGraph(_), path)) => {
                let data = LineTabData::new(path, LineConfigState::default());
                match data {
                    Err(err) => {
                        let msg = Message::Error(err);
                        Command::perform(async { msg }, |msg| msg)
                    }

                    Ok(data) => {
                        let data = data.theme(self.theme.clone());
                        let idr = View::LineGraph(data);
                        self.update_tabs(TabsMessage::AddTab(idr))
                    }
                }
            }
            FileIOAction::NewTab((View::None, _)) => self.update_tabs(TabsMessage::None),
            FileIOAction::RefreshTab((ViewType::Counter, tid, _)) => {
                self.update_tabs(TabsMessage::RefreshTab((tid, Refresh::Counter)))
            }
            FileIOAction::RefreshTab((ViewType::Editor, tid, path)) => {
                let data = EditorTabData::new(Some(path), content);
                let rsh = Refresh::Editor(data);
                self.update_tabs(TabsMessage::RefreshTab((tid, rsh)))
            }
            FileIOAction::RefreshTab((ViewType::LineGraph, tid, path)) => {
                let data = LineTabData::new(path, LineConfigState::default());
                match data {
                    Err(err) => {
                        let msg = Message::Error(err);
                        Command::perform(async { msg }, |msg| msg)
                    }
                    Ok(data) => {
                        let rsh = Refresh::Model(data);
                        self.update_tabs(TabsMessage::RefreshTab((tid, rsh)))
                    }
                }
            }
            FileIOAction::RefreshTab((ViewType::None, _, _)) => self.update_tabs(TabsMessage::None),
            FileIOAction::CloseTab(id) => {
                let tsg = TabsMessage::CloseTab((id, true));
                self.update_tabs(tsg)
            }
            FileIOAction::Exiting(id) => {
                self.tabs.update(TabsMessage::CloseTab((id, true)));
                let tsg = TabsMessage::Exit;
                self.update_tabs(tsg)
            }
            FileIOAction::None => Command::none(),
        }
    }
}

impl Application for Modav {
    type Theme = Theme;
    type Flags = ();
    type Message = Message;
    type Executor = executor::Default;

    fn title(&self) -> String {
        self.title.clone()
    }

    fn theme(&self) -> Self::Theme {
        self.theme.clone()
    }

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let tabs = Tabs::new().width(Length::FillPortion(5));
        let commands = [
            font::load(include_bytes!("../fonts/status-icons.ttf").as_slice())
                .map(Message::IconLoaded),
            font::load(include_bytes!("../fonts/dash-icons.ttf").as_slice())
                .map(Message::IconLoaded),
            font::load(include_bytes!("../fonts/wizard-icons.ttf").as_slice())
                .map(Message::IconLoaded),
            font::load(include_bytes!("../fonts/toast-icons.ttf").as_slice())
                .map(Message::IconLoaded),
            font::load(iced_aw::BOOTSTRAP_FONT_BYTES).map(Message::IconLoaded),
        ];
        (
            Modav {
                file_path: None,
                title: String::from("Modav"),
                // theme: Theme::Nightfly,
                // theme: Theme::Dark,
                // theme: Theme::SolarizedLight,
                theme: Theme::GruvboxDark,
                current_view: ViewType::None,
                tabs,
                toasts: Vec::default(),
                wizard_shown: false,
                error: AppError::None,
            },
            Command::batch(commands),
        )
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Error(err) => {
                println!("{}", err.to_string());
                let toast = Toast {
                    status: Status::Error,
                    body: err.message(),
                };
                self.toasts.push(toast);
                Command::none()
            }
            Message::IconLoaded(Ok(_)) => Command::none(),
            Message::IconLoaded(Err(e)) => {
                let error = AppError::FontLoading(e);
                println!("{}", error.to_string());
                let toast = Toast {
                    status: Status::Error,
                    body: error.message(),
                };
                self.toasts.push(toast);
                Command::none()
            }
            Message::ToggleTheme => {
                self.toggle_theme();
                Command::none()
            }
            Message::SelectFile => {
                self.error = AppError::None;
                Command::perform(pick_file(), Message::FileSelected)
            }
            Message::FileSelected(Ok(p)) => {
                self.error = AppError::None;
                self.file_path = Some(p);
                self.wizard_shown = true;
                Command::none()
            }
            Message::FileSelected(Err(e)) => {
                let toast = Toast {
                    status: Status::Error,
                    body: e.message(),
                };
                self.toasts.push(toast);
                Command::none()
            }
            Message::LoadFile((path, action)) => {
                self.error = AppError::None;
                Command::perform(load_file(path), move |(res, _)| {
                    Message::FileLoaded((res, action))
                })
            }
            Message::FileLoaded((Ok(res), action)) => self.file_io_action_handler(action, res),
            Message::FileLoaded((Err(err), _)) => {
                let toast = Toast {
                    status: Status::Error,
                    body: err.message(),
                };
                self.toasts.push(toast);
                Command::none()
            }
            Message::OpenTab(path, tidr) => {
                let path = path.filter(|path| path.is_file());

                match (path, tidr.should_load()) {
                    (Some(path), true) => {
                        Command::perform(load_file(path.clone()), |(res, path)| {
                            Message::FileLoaded((res, FileIOAction::NewTab((tidr, path))))
                        })
                    }
                    (Some(_path), false) => {
                        let idr = match tidr {
                            View::Counter => View::Counter,
                            View::Editor(_) => View::None,
                            View::LineGraph(data) => {
                                let data = data.theme(self.theme.clone());
                                View::LineGraph(data)
                            }
                            View::None => View::None,
                        };
                        self.update_tabs(TabsMessage::AddTab(idr))
                    }

                    (None, true) => self.update_tabs(TabsMessage::AddTab(tidr)),
                    (None, false) => {
                        let idr = match tidr {
                            View::Counter => View::Counter,
                            View::Editor(_) => View::None,
                            View::LineGraph(_) => View::None,
                            View::None => View::None,
                        };
                        self.update_tabs(TabsMessage::AddTab(idr))
                    }
                }
            }
            Message::TabsMessage(tsg) => {
                if let Some(response) = self.tabs.update(tsg) {
                    Command::perform(async { response }, |response| response)
                } else {
                    Command::none()
                }
            }
            Message::SaveFile((path, content, action)) => {
                self.error = AppError::None;

                if self.tabs.active_tab_can_save() {
                    Command::perform(save_file(path, content), |res| {
                        let action = match &res {
                            Err(_) => action,
                            Ok((path, _)) => action.update_path(path.clone()),
                        };
                        Message::FileSaved((res, action))
                    })
                } else {
                    Command::none()
                }
            }
            Message::SaveKeyPressed => {
                let save_message = self.save_helper(self.tabs.active_path());
                Command::perform(async { save_message }, |msg| msg)
            }
            Message::FileSaved((Ok((_path, content)), action)) => {
                let toast = Toast {
                    status: Status::Success,
                    body: "Save Successful!".into(),
                };
                self.toasts.push(toast);
                self.file_io_action_handler(action, content)
            }
            Message::FileSaved((Err(e), _)) => {
                let toast = Toast {
                    status: Status::Error,
                    body: e.message(),
                };
                self.toasts.push(toast);
                Command::none()
            }
            Message::CheckExit => self.update_tabs(TabsMessage::Exit),
            Message::CanExit => window::close(window::Id::MAIN),
            Message::NewActiveTab => {
                self.current_view = self.tabs.active_tab_type().unwrap_or(ViewType::None);
                self.file_path = self.tabs.active_path();
                Command::none()
            }
            Message::Convert => Command::none(),
            Message::None => Command::none(),
            Message::Debugging => {
                println!("Debugging Message sent!");
                Command::none()
            }
            Message::ToggleWizardShown => {
                if self.wizard_shown {
                    self.wizard_shown = false;
                    Command::perform(async {}, |_| Message::NewActiveTab)
                } else {
                    self.wizard_shown = true;
                    Command::none()
                }
            }
            Message::WizardSubmit(path, view) => {
                self.wizard_shown = false;
                Command::perform(async { Message::OpenTab(Some(path), view) }, |msg| msg)
            }
            Message::AddToast(toast) => {
                self.toasts.push(toast);
                Command::none()
            }
            Message::CloseToast(index) => {
                self.toasts.remove(index);
                Command::none()
            }
            Message::Event(event) => match event {
                Event::Window(window::Id::MAIN, window::Event::CloseRequested) => {
                    self.update_tabs(TabsMessage::Exit)
                }
                Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => match key {
                    Key::Named(key::Named::Save) if modifiers.command() => {
                        let save_message = self.save_helper(self.tabs.active_path());
                        Command::perform(async { save_message }, |msg| msg)
                    }
                    Key::Character(s) if s.as_str() == "s" && modifiers.command() => {
                        let save_message = self.save_helper(self.tabs.active_path());
                        Command::perform(async { save_message }, |msg| msg)
                    }
                    Key::Named(key::Named::Tab) => {
                        if modifiers.shift() {
                            widget::focus_previous()
                        } else {
                            widget::focus_next()
                        }
                    }
                    _ => Command::none(),
                },
                _ => Command::none(),
            },
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message, Self::Theme, iced::Renderer> {
        let status_bar = self.status_bar().height(Length::FillPortion(1));
        let dashboard = self.dashboard();

        let content = if self.tabs.is_empty() {
            home_view().into()
        } else {
            self.tabs.content().map(Message::TabsMessage)
        };

        let cross_axis = row!(dashboard, content).height(Length::FillPortion(25));

        let main_axis = column!(cross_axis, status_bar);

        let content: Element<'_, Message> = if self.wizard_shown {
            let file = self
                .file_path
                .clone()
                .expect("File path was empty for Wizard")
                .clone();
            let wizard = Wizard::new(file, Message::WizardSubmit, Message::Error)
                .on_reselect(Message::SelectFile)
                .on_cancel(Message::ToggleWizardShown);
            Modal::new(main_axis, wizard)
                .on_blur(Message::ToggleWizardShown)
                .into()
        } else {
            main_axis.into()
        };

        let content = toast::Manager::new(content, &self.toasts, Message::CloseToast, &self.theme);

        container(content).height(Length::Fill).into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        event::listen().map(Message::Event)
    }
}
