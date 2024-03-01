use iced::{
    color,
    event::{self, Event},
    executor, font,
    keyboard::{self, key, Key},
    theme, widget,
    widget::{column, container, horizontal_space, row, text, vertical_rule, Container, Row},
    Application, Command, Font, Length, Settings, Subscription, Theme,
};

use iced_aw::native::menu::Item;

use std::path::PathBuf;

// mod temp;
use styles::*;
use utils::*;
mod views;
use views::{home_view, EditorTabData, Identifier, Refresh, TabBarMessage, Tabs};

fn main() -> Result<(), iced::Error> {
    Modav::run(Settings::default())
}

#[derive(Debug, Clone)]
pub enum TabIden {
    Counter,
    Editor,
}

impl TabIden {
    /// Returns true if this tab requires the contents of a file to be loaded
    fn should_load(&self) -> bool {
        match self {
            Self::Counter => false,
            Self::Editor => true,
        }
    }
}

/// What should be done after a file IO action
#[derive(Debug, Clone)]
pub enum FileIOAction {
    NewTab((TabIden, PathBuf)),
    RefreshTab((TabIden, usize, PathBuf)),
    None,
}

impl FileIOAction {
    fn update_path(self, path: PathBuf) -> Self {
        match self {
            Self::None => Self::None,
            Self::NewTab((tidn, _)) => Self::NewTab((tidn, path)),
            Self::RefreshTab((tidn, id, _)) => Self::RefreshTab((tidn, id, path)),
        }
    }
}

pub struct Modav {
    theme: Theme,
    title: String,
    current_model: String,
    file_path: Option<PathBuf>,
    tabs: Tabs,
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
    Exit,
    OpenTab(TabIden),
    TabsMessage(TabBarMessage),
}

impl Modav {
    fn status_bar(&self) -> Container<'_, Message> {
        let current: Row<'_, Message> = {
            let model = if self.current_model.is_empty() {
                None
            } else {
                Some(text(self.current_model.clone()))
            };

            let path = {
                self.file_path.as_ref().and_then(|path| {
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

            match (path, model) {
                (None, None) => row!(),
                (None, Some(m)) => row!(m),
                (Some(p), None) => row!(p),
                (Some(p), Some(m)) => row!(m, vertical_rule(2), p),
            }
        }
        .spacing(10);

        let error = {
            let color = color!(255, 0, 0);
            let msg = self.error.message();
            if msg.is_empty() {
                row!()
            } else {
                let icon = text("•").style(theme::Text::Color(color));
                row!(icon, text(msg)).spacing(5)
            }
        };

        let row: Row<'_, Message> = row!(error, horizontal_space(), current).height(20);

        let bstyle = default_bordered_container(&self.theme);

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
            self.file_path.clone().unwrap_or(PathBuf::new()),
        ));

        Message::SaveFile((save_path, content, action))
    }

    fn file_menu(&self) -> Container<'_, Message> {
        let actions_label = vec![
            ("Open File", Message::SelectFile),
            ("Save File", self.save_helper(self.file_path.clone())),
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

        let menus = column!(
            self.file_menu(),
            menus::models_menu(),
            menus::views_menu(),
            menus::about_menu(),
            menus::settings_menu()
        )
        .spacing(45);

        let content = column!(logo, menus).spacing(80);

        let bstyle = default_bordered_container(&self.theme);
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

    fn update_tabs(&mut self, tsg: TabBarMessage) -> Command<Message> {
        if let Some(response) = self.tabs.update(tsg) {
            Command::perform(async { response }, |response| response)
        } else {
            Command::none()
        }
    }

    fn file_io_action_updater(
        &mut self,
        action: FileIOAction,
        content: String,
    ) -> Command<Message> {
        match action {
            FileIOAction::NewTab((TabIden::Counter, _)) => {
                let idr = Identifier::Counter;

                self.update_tabs(TabBarMessage::AddTab(idr))
            }
            FileIOAction::NewTab((TabIden::Editor, path)) => {
                self.file_path = Some(path.clone());
                let data = EditorTabData::new(path, content);
                let idr = Identifier::Editor(data);
                self.update_tabs(TabBarMessage::AddTab(idr))
            }
            FileIOAction::RefreshTab((TabIden::Counter, tid, _)) => {
                self.update_tabs(TabBarMessage::RefreshTab((tid, Refresh::Counter)))
            }
            FileIOAction::RefreshTab((TabIden::Editor, tid, path)) => {
                let data = EditorTabData::new(path, content);
                let rsh = Refresh::Editor(data);
                self.update_tabs(TabBarMessage::RefreshTab((tid, rsh)))
            }
            FileIOAction::None => {
                let idr = Identifier::None;
                self.update_tabs(TabBarMessage::AddTab(idr))
            }
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
            font::load(iced_aw::BOOTSTRAP_FONT_BYTES).map(Message::IconLoaded),
        ];
        (
            Modav {
                file_path: None,
                title: String::from("Modav"),
                theme: Theme::Nightfly,
                // theme: Theme::Dark,
                current_model: String::new(),
                tabs,
                error: AppError::None,
            },
            Command::batch(commands),
        )
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::IconLoaded(Ok(_)) => Command::none(),
            Message::IconLoaded(Err(e)) => {
                self.error = AppError::FontLoading(e);
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
                self.file_path = Some(p);
                self.error = AppError::None;
                Command::none()
            }
            Message::FileSelected(Err(e)) => {
                self.error = e;
                Command::none()
            }
            Message::LoadFile((path, action)) => {
                self.error = AppError::None;
                Command::perform(load_file(path), move |(res, _)| {
                    Message::FileLoaded((res, action))
                })
            }
            Message::FileLoaded((Ok(res), action)) => self.file_io_action_updater(action, res),
            Message::FileLoaded((Err(err), _)) => {
                self.error = err;
                Command::none()
            }
            Message::OpenTab(tidr) => {
                let path = self.file_path.as_ref().filter(|path| path.is_file());

                match (path, tidr.should_load()) {
                    (Some(path), true) => {
                        Command::perform(load_file(path.clone()), |(res, path)| {
                            Message::FileLoaded((res, FileIOAction::NewTab((tidr, path))))
                        })
                    }
                    (Some(_path), false) => {
                        let idr = match tidr {
                            TabIden::Counter => Identifier::Counter,
                            TabIden::Editor => Identifier::None,
                        };
                        self.update_tabs(TabBarMessage::AddTab(idr))
                    }

                    (None, true) => {
                        let idr = match tidr {
                            TabIden::Counter => Identifier::Counter,
                            TabIden::Editor => {
                                let data = EditorTabData::new(PathBuf::new(), String::new());
                                Identifier::Editor(data)
                            }
                        };
                        self.update_tabs(TabBarMessage::AddTab(idr))
                    }
                    (None, false) => {
                        let idr = match tidr {
                            TabIden::Counter => Identifier::Counter,
                            TabIden::Editor => Identifier::None,
                        };
                        self.update_tabs(TabBarMessage::AddTab(idr))
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

                Command::perform(save_file(path, content), |res| {
                    let action = match &res {
                        Err(_) => action,
                        Ok((path, _)) => action.update_path(path.clone()),
                    };
                    Message::FileSaved((res, action))
                })
            }
            Message::SaveKeyPressed => {
                let save_message = self.save_helper(self.file_path.clone());
                Command::perform(async { save_message }, |msg| msg)
            }
            Message::FileSaved((Ok((path, content)), action)) => {
                self.file_path = Some(path);
                self.file_io_action_updater(action, content)
            }
            Message::FileSaved((Err(e), _)) => {
                self.error = e;
                Command::none()
            }
            Message::Exit => {
                self.tabs.update(TabBarMessage::Exit);
                Command::none()
            }
            Message::Convert => Command::none(),
            Message::None => Command::none(),
            Message::Event(event) => match event {
                Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => match key {
                    Key::Named(key::Named::Save) if modifiers.command() => {
                        let save_message = self.save_helper(self.file_path.clone());
                        Command::perform(async { save_message }, |msg| msg)
                    }
                    Key::Character(s) if s.as_str() == "s" && modifiers.command() => {
                        let save_message = self.save_helper(self.file_path.clone());
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
        let status_bar = self.status_bar();
        let dashboard = self.dashboard();

        let content = if self.tabs.is_empty() {
            home_view().into()
        } else {
            self.tabs.content().map(Message::TabsMessage)
        };

        let cross_axis = row!(dashboard, content).height(Length::Fill);

        let main_axis = column!(cross_axis, status_bar);

        container(main_axis).height(Length::Fill).into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        event::listen().map(Message::Event)
    }
}

mod utils {
    use super::styles::*;
    use super::Message;
    use iced::{
        alignment, font,
        widget::{text, Text},
        Font,
    };
    use std::fmt::{Debug, Display};
    use std::io;
    use std::path::PathBuf;

    #[derive(Debug, Clone, PartialEq, Default)]
    pub enum AppError {
        FontLoading(font::Error),
        FileDialogClosed,
        FileLoading(io::ErrorKind),
        FileSaving(io::ErrorKind),
        #[default]
        None,
    }

    impl AppError {
        pub fn message(&self) -> String {
            match self {
                Self::FontLoading(_) => String::from("Error while loading a font"),
                Self::FileDialogClosed => String::from("File Dialog closed prematurely"),
                Self::FileLoading(_) => String::from("Error while loading file"),
                Self::FileSaving(_) => String::from("Error while saving file"),
                Self::None => String::new(),
            }
        }
    }

    impl Display for AppError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let msg = self.message();
            match self {
                Self::FontLoading(e) => e.fmt(f),
                Self::FileLoading(err) => std::fmt::Display::fmt(err, f),
                Self::FileSaving(err) => std::fmt::Display::fmt(err, f),
                Self::FileDialogClosed => write!(f, "{}", msg),
                Self::None => write!(f, "{}", msg),
            }
        }
    }

    pub fn icon(unicode: char, name: &'static str) -> Text<'static> {
        let fnt: Font = Font::with_name(name);
        text(unicode.to_string())
            .font(fnt)
            .horizontal_alignment(alignment::Horizontal::Center)
    }

    pub fn status_icon(unicode: char) -> Text<'static> {
        icon(unicode, "status-icons")
    }

    pub mod menus {

        use iced_aw::{
            menu::{Item, Menu, MenuBar},
            menu_bar, style,
        };

        use crate::{
            styles::{ColoredContainer, CustomMenuBarStyle},
            TabIden,
        };

        use super::{icon, MenuButtonStyle, Message};

        use iced::{
            color,
            theme::{self, Theme},
            widget::{button, container, row, text, Button, Container, Row, Text},
            Element, Length, Renderer,
        };

        fn dash_icon(unicode: char) -> Text<'static> {
            icon(unicode, "dash-icons")
        }

        /// The last item in a Menu Tree
        fn base_tree(label: &str, msg: Message) -> Item<'_, Message, Theme, Renderer> {
            let btn = button(text(label).width(Length::Shrink).height(Length::Shrink))
                .on_press(msg)
                .style(theme::Button::Custom(Box::new(MenuButtonStyle {})))
                .padding([4, 16])
                .width(Length::Shrink)
                .height(Length::Shrink);

            Item::new(btn)
        }

        pub fn create_children(
            labels: Vec<(&str, Message)>,
        ) -> Vec<Item<'_, Message, Theme, Renderer>> {
            labels
                .into_iter()
                .map(|curr| {
                    let label = curr.0;
                    let msg = curr.1;
                    base_tree(label, msg)
                })
                .collect()
        }

        fn create_label<'a>(
            icon: char,
            label: impl Into<Element<'a, Message, Theme, Renderer>>,
        ) -> Row<'a, Message> {
            let icon = dash_icon(icon);
            row!(icon, label.into()).spacing(8).padding([0, 8])
        }

        pub fn create_menu<'a>(
            label: impl Into<Element<'a, Message, Theme, Renderer>>,
            icon: char,
            children: Vec<Item<'a, Message, Theme, Renderer>>,
        ) -> MenuBar<'a, Message, Theme, Renderer> {
            let label = create_label(icon, label.into());
            let item = container(label).width(Length::Fill);
            let menu = Menu::new(children).offset(5.0).width(Length::Shrink);

            menu_bar!((item, menu))
                .check_bounds_width(30.0)
                .width(Length::Fill)
                .style(style::MenuBarStyle::Custom(Box::new(CustomMenuBarStyle)))
        }

        pub fn container_wrap<'a>(
            item: impl Into<Element<'a, Message, Theme, Renderer>>,
        ) -> Container<'a, Message> {
            container(item)
                .padding([8, 0])
                .width(Length::Fixed(125.0))
                .style(theme::Container::Custom(Box::new(ColoredContainer {
                    color: color!(255, 0, 255, 0.3),
                    radius: 8.0,
                })))
        }

        pub fn models_menu<'a>() -> Container<'a, Message> {
            let actions_labels = vec![("Line Graph", Message::Convert)];

            let children = create_children(actions_labels);

            let bar = create_menu("Models", '\u{E802}', children);

            container_wrap(bar)
        }

        pub fn views_menu<'a>() -> Container<'a, Message> {
            let action_labels = vec![
                ("Add Counter", Message::OpenTab(TabIden::Counter)),
                ("Open Editor", Message::OpenTab(TabIden::Editor)),
            ];

            let children = create_children(action_labels);

            let bar = create_menu("Views", '\u{E800}', children);

            container_wrap(bar)
        }

        pub fn about_menu<'a>() -> Container<'a, Message> {
            let label = create_label('\u{E801}', text("About"));
            let btn: Button<'_, Message, Theme, Renderer> = button(label)
                .style(theme::Button::Text)
                .padding([0, 0])
                .on_press(Message::None);

            container_wrap(btn)
        }

        pub fn settings_menu<'a>() -> Container<'a, Message> {
            let actions_labels = vec![("Toggle Theme", Message::ToggleTheme)];

            let children = create_children(actions_labels);

            let bar = create_menu("Settings", '\u{E800}', children);

            container_wrap(bar)
        }
    }

    pub async fn pick_file() -> Result<PathBuf, AppError> {
        let handle = rfd::AsyncFileDialog::new()
            .pick_file()
            .await
            .ok_or(AppError::FileDialogClosed)?;

        Ok(handle.path().into())
    }

    pub async fn load_file(path: PathBuf) -> (Result<String, AppError>, PathBuf) {
        let res = tokio::fs::read_to_string(path.clone())
            .await
            .map_err(|err| AppError::FileLoading(err.kind()));

        (res, path)
    }

    pub async fn save_file(
        path: Option<PathBuf>,
        content: String,
    ) -> Result<(PathBuf, String), AppError> {
        let path = if let Some(path) = path {
            path
        } else {
            rfd::AsyncFileDialog::new()
                .set_title("Choose File name")
                .save_file()
                .await
                .ok_or(AppError::FileDialogClosed)
                .map(|handle| handle.path().to_owned())?
        };

        tokio::fs::write(&path, content.clone())
            .await
            .map_err(|err| AppError::FileSaving(err.kind()))?;

        Ok((path, content))
    }
}

pub mod styles {
    use iced::{
        color,
        widget::{button, container},
        Background, Border, Color, Theme,
    };

    pub struct BorderedContainer {
        pub width: f32,
        pub bcolor: Color,
    }

    impl Default for BorderedContainer {
        fn default() -> Self {
            Self {
                width: 0.75,
                bcolor: color!(0, 0, 0, 1.),
            }
        }
    }

    impl container::StyleSheet for BorderedContainer {
        type Style = Theme;

        fn appearance(&self, _style: &Self::Style) -> container::Appearance {
            let border = Border {
                color: self.bcolor,
                width: self.width,
                ..Default::default()
            };
            container::Appearance {
                border,
                ..Default::default()
            }
        }
    }

    pub struct ColoredContainer {
        pub color: Color,
        pub radius: f32,
    }

    impl Default for ColoredContainer {
        fn default() -> Self {
            Self {
                color: Color::TRANSPARENT,
                radius: 0.0,
            }
        }
    }

    impl container::StyleSheet for ColoredContainer {
        type Style = Theme;

        fn appearance(&self, _style: &Self::Style) -> container::Appearance {
            let border = Border {
                radius: self.radius.into(),
                ..Default::default()
            };
            container::Appearance {
                background: Some(self.color.into()),
                border,
                ..Default::default()
            }
        }
    }

    pub fn default_bordered_container(theme: &Theme) -> BorderedContainer {
        match theme {
            Theme::Light => BorderedContainer::default(),
            Theme::Dark => BorderedContainer {
                bcolor: color!(255, 255, 255),
                ..Default::default()
            },
            _ => BorderedContainer::default(),
        }
    }

    pub struct MenuButtonStyle;
    impl button::StyleSheet for MenuButtonStyle {
        type Style = iced::Theme;

        fn active(&self, _style: &Self::Style) -> button::Appearance {
            let border = Border {
                radius: [4.0; 4].into(),
                ..Default::default()
            };
            button::Appearance {
                border,
                background: Some(Color::TRANSPARENT.into()),
                ..Default::default()
            }
        }

        fn hovered(&self, style: &Self::Style) -> button::Appearance {
            let plt = style.extended_palette();

            button::Appearance {
                background: Some(plt.primary.weak.color.into()),
                text_color: plt.primary.weak.text,
                ..self.active(style)
            }
        }
    }

    pub struct CustomMenuBarStyle;
    impl iced_aw::menu::StyleSheet for CustomMenuBarStyle {
        type Style = Theme;

        fn appearance(&self, _style: &Self::Style) -> iced_aw::menu::Appearance {
            let border = Border {
                radius: [8.0; 4].into(),
                ..Default::default()
            };
            iced_aw::menu::Appearance {
                bar_border: border,
                bar_background: Background::Color(Color::TRANSPARENT),
                path: Background::Color(Color::TRANSPARENT),
                ..Default::default()
            }
        }
    }
}
