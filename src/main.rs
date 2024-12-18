use iced::{
    application, font,
    keyboard::{self, key, Key},
    widget::{self, column, container, horizontal_space, row, text, vertical_rule, Container, Row},
    window, Alignment, Element, Font, Length, Subscription, Task, Theme,
};

use tracing::{error, info, span, warn, Level};
use tracing_subscriber::EnvFilter;

use std::{fs::File, path::PathBuf};

mod styles;
use styles::*;

mod utils;
use utils::{icons, load_file, menus, pick_file, save_file, AppError};

mod views;
use views::{
    home_view, BarChartTabData, EditorTabData, LineTabData, Refresh, StackedBarChartTabData, Tabs,
    TabsMessage, View, ViewType,
};

pub mod widgets;
use widgets::{
    dashmenu::{DashMenu, DashMenuOption},
    modal::Modal,
    settings::SettingsDialog,
    style::dialog_container,
    toast::{self, Status, Toast},
    wizard::{BarChartConfigState, LineConfigState, StackedBarChartConfigState, Wizard},
};

fn main() -> Result<(), iced::Error> {
    let fallback_log = "./modav.log";

    let log = if cfg!(target_os = "windows") {
        match directories::UserDirs::new() {
            Some(usr) => usr
                .home_dir()
                .join("AppData")
                .join("Local")
                .join("modav.log"),
            None => PathBuf::from(fallback_log),
        }
    } else if cfg!(target_os = "macos") {
        match directories::UserDirs::new() {
            Some(usr) => usr
                .home_dir()
                .join("Library")
                .join("Logs")
                .join("modav.log"),
            None => PathBuf::from(fallback_log),
        }
    } else if cfg!(target_os = "linux") {
        match directories::UserDirs::new() {
            Some(usr) => usr
                .home_dir()
                .join(".local")
                .join("share")
                .join("modav.log"),
            None => PathBuf::from(fallback_log),
        }
    } else {
        PathBuf::from(fallback_log)
    };

    let span = span!(Level::INFO, "Modav");
    let _guard = span.enter();

    let mut fallback_flag = false;
    let log_file = File::create(log.clone()).unwrap_or_else(|_| {
        fallback_flag = true;
        File::create(fallback_log).unwrap()
    });

    let (non_blocking, _log_writer) = tracing_appender::non_blocking(log_file);

    let filter = EnvFilter::new("error");

    let filter = if cfg!(debug_assertions) {
        filter
            .add_directive("modav=info".parse().unwrap())
            .add_directive("modav_core=info".parse().unwrap())
    } else {
        filter
    };

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_env_filter(filter)
        .init();

    let _flags = Flags::Prod(if fallback_flag {
        PathBuf::from(fallback_log)
    } else {
        log
    });

    let flags = Flags::Stacked;

    application(Modav::title, Modav::update, Modav::view)
        .centered()
        .antialiasing(true)
        .subscription(Modav::subscription)
        .exit_on_close_request(false)
        .theme(Modav::theme)
        .run_with(|| {
            let app = Modav::new(flags);
            let tasks = [
                font::load(include_bytes!("../fonts/util-icons.ttf").as_slice())
                    .map(Message::IconLoaded),
                font::load(include_bytes!("../fonts/legend-icons.ttf").as_slice())
                    .map(Message::IconLoaded),
                font::load(include_bytes!("../fonts/line-type-icons.ttf").as_slice())
                    .map(Message::IconLoaded),
                window::get_oldest().and_then(|id| Task::done(Message::SetMainWindowID(id))),
            ];

            let batch = Task::batch(tasks);

            let status = batch.chain(Task::done(Message::Ready));

            (app, status)
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

#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum DialogView {
    Wizard,
    Settings,
    About,
    #[default]
    None,
}

pub struct Modav {
    theme: Theme,
    theme_shadow: Theme,
    title: String,
    current_view: ViewType,
    file_path: Option<PathBuf>,
    tabs: Tabs<Theme>,
    toasts: Vec<Toast>,
    toast_timeout: u64,
    dialog_view: DialogView,
    error: AppError,
    log_file: PathBuf,
    main_window_id: Option<window::Id>,
    is_ready: bool,
}

#[derive(Debug, Clone)]
pub enum Flags {
    Bar,
    Line,
    Stacked,
    Prod(PathBuf),
}

impl Flags {
    fn create(self) -> Modav {
        let theme = Theme::TokyoNight;
        let toasts = Vec::default();
        let title = String::from("Modav");
        let error = AppError::None;
        let main_window_id = None;
        let is_ready = false;
        let mut tabs = Tabs::new()
            .on_open(Message::SelectFile)
            .on_new_active_tab(Message::NewActiveTab)
            .on_save(|path, content, action| Message::SaveFile((path, content, action)))
            .on_check_exit(Message::CheckExit)
            .can_exit(Message::CanExit)
            .width(Length::FillPortion(5))
            .height(Length::Fill)
            .tab_bar_height(45.0)
            .tab_spacing(2.5)
            .tab_bar_padding(3)
            .tab_padding([5, 7]);
        let toast_timeout = 2;
        let dialog_view = DialogView::default();
        let default_log_file = PathBuf::from("~/.local/share/modav.log");
        match self {
            Self::Prod(log_file) => Modav {
                file_path: None,
                current_view: ViewType::None,
                is_ready,
                theme_shadow: theme.clone(),
                title,
                theme,
                toasts,
                main_window_id,
                toast_timeout,
                error,
                tabs,
                dialog_view,
                log_file,
            },
            Self::Line => {
                let file_path = PathBuf::from("../../../alter.csv");
                let current_view = ViewType::LineGraph;

                {
                    let data = LineTabData::new(file_path.clone(), LineConfigState::default())
                        .expect("Line Graph Panic when running dev flag");
                    let view = View::LineGraph(data);

                    tabs.update(TabsMessage::AddTab(view));
                }

                Modav {
                    file_path: Some(file_path),
                    theme_shadow: theme.clone(),
                    is_ready,
                    current_view,
                    title,
                    theme,
                    main_window_id,
                    toasts,
                    toast_timeout,
                    error,
                    tabs,
                    dialog_view,
                    log_file: default_log_file,
                }
            }
            Self::Bar => {
                use modav_core::repr::sheet::utils::{
                    BarChartAxisLabelStrategy, BarChartBarLabels,
                };

                let file_path = PathBuf::from("../../../bar.csv");
                let current_view = ViewType::BarChart;

                {
                    let config = BarChartConfigState {
                        x_col: 1,
                        y_col: 2,
                        axis_label: BarChartAxisLabelStrategy::Headers,
                        bar_label: BarChartBarLabels::FromColumn(0),
                        //order: true,
                        caption: Some("Caption: This".into()),
                        ..Default::default()
                    };

                    let data = BarChartTabData::new(file_path.clone(), config)
                        .expect("Bar Chart panic with dev flag");
                    let view = View::BarChart(data);
                    tabs.update(TabsMessage::AddTab(view));
                }

                Modav {
                    file_path: Some(file_path),
                    theme_shadow: theme.clone(),
                    is_ready,
                    current_view,
                    title,
                    theme,
                    main_window_id,
                    toasts,
                    toast_timeout,
                    error,
                    tabs,
                    dialog_view,
                    log_file: default_log_file,
                }
            }
            Self::Stacked => {
                use modav_core::repr::sheet::utils::StackedBarChartAxisLabelStrategy;

                let file_path = PathBuf::from("../../../stacked_neg.csv");
                let current_view = ViewType::StackedBarChart;

                {
                    let config = StackedBarChartConfigState {
                        x_col: 0,
                        acc_cols_str: "1:5".into(),
                        axis_label: StackedBarChartAxisLabelStrategy::Header("Total Cost".into()),
                        is_horizontal: false,
                        caption: Some("Caption where?".into()),
                        title: "Stacked Bar Chart".into(),
                        ..Default::default()
                    };
                    let data = StackedBarChartTabData::new(file_path.clone(), config)
                        .expect("Stacked Bar Chart dev flag panic")
                        .theme(theme.clone());
                    let view = View::StackedBarChart(data);
                    tabs.update(TabsMessage::AddTab(view));
                }

                Modav {
                    file_path: Some(file_path),
                    theme_shadow: theme.clone(),
                    is_ready,
                    current_view,
                    title,
                    theme,
                    toasts,
                    toast_timeout,
                    main_window_id,
                    error,
                    tabs,
                    dialog_view,
                    log_file: default_log_file,
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    IconLoaded(Result<(), font::Error>),
    ToggleTheme,
    SelectFile,
    Ready,
    FileSelected(Result<PathBuf, AppError>),
    LoadFile((PathBuf, FileIOAction)),
    FileLoaded((Result<String, AppError>, FileIOAction)),
    SaveFile((Option<PathBuf>, String, FileIOAction)),
    SaveKeyPressed,
    FileSaved((Result<(PathBuf, String), AppError>, FileIOAction)),
    Convert,
    None,
    WindowCloseRequested(window::Id),
    SetMainWindowID(window::Id),
    CloseWindow(window::Id),
    KeyPressed(keyboard::Key, keyboard::Modifiers),
    CheckExit,
    CanExit,
    OpenTab(Option<PathBuf>, View),
    NewActiveTab,
    TabsMessage(TabsMessage),
    Debugging,
    WizardSubmit(PathBuf, View),
    CloseWizard,
    OpenSettingsDialog,
    AbortSettings,
    SaveSettings(Theme, u64),
    OpenAboutDialog,
    CloseAboutDialog,
    OpenLogFile,
    ChangeTheme(Theme),
    AddToast(Toast),
    CloseToast(usize),
    Error(AppError, bool),
    /// Open the editor for the current model
    OpenEditor(Option<PathBuf>),
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
                            let text = text(file.to_owned());
                            row!(text).spacing(5)
                        })
                })
            };

            match (path, &self.current_view) {
                (None, ViewType::None) => row!(),
                (None, vt) => {
                    let txt = text("Untitled");
                    let txt = txt;
                    row!(vt.display(), vertical_rule(10), txt)
                }
                (Some(p), ViewType::None) => row!(p),
                (Some(p), m) => row!(m.display(), vertical_rule(10), p),
            }
        }
        .align_y(Alignment::Center)
        .spacing(10);

        let row: Row<'_, Message> = row!(horizontal_space(), current)
            .height(Length::Fill)
            .align_y(Alignment::Center);

        container(row).padding([0, 10]).style(|theme| {
            <BorderedContainer as container::Catalog>::style(&BorderedContainer::default(), theme)
        })
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
            self.tabs
                .active_tab_idx()
                .expect("Save Helper: Attempted to refresh active tab save without an active tab"),
            self.tabs.active_path().unwrap_or(PathBuf::new()),
        ));

        Message::SaveFile((save_path, content, action))
    }

    fn file_menu(&self) -> DashMenu<Message> {
        let options = vec![
            DashMenuOption::new(
                "New File",
                Some(Message::OpenTab(
                    None,
                    View::Editor(EditorTabData::default()),
                )),
            ),
            DashMenuOption::new("Open File", Some(Message::SelectFile)),
            DashMenuOption::new("Save File", Some(self.save_helper(self.tabs.active_path()))),
            DashMenuOption::new("Save As", Some(self.save_helper(None))),
        ];

        let menu = DashMenu::new(icons::FILE, "File").submenus(options);

        menus::menu_styler(menu)
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
            column!(
                self.file_menu(),
                menus::models_menu(),
                menus::about_menu(),
                menus::settings_menu()
            )
            .spacing(45)
        };

        let content = column!(logo, menus).spacing(80);

        container(content)
            .center_x(Length::FillPortion(1))
            .padding([15, 0])
            .height(Length::Fill)
            .style(|theme| {
                <BorderedContainer as container::Catalog>::style(
                    &BorderedContainer::default(),
                    theme,
                )
            })
    }

    fn toggle_theme(&mut self) {
        match self.theme {
            Theme::Dark => {
                self.theme = Theme::Light;
                self.theme_shadow = self.theme.clone();
            }
            Theme::Light => {
                self.theme = Theme::Dark;
                self.theme_shadow = self.theme.clone();
            }
            Theme::SolarizedLight => {
                self.theme = Theme::SolarizedDark;
                self.theme_shadow = self.theme.clone();
            }
            Theme::SolarizedDark => {
                self.theme = Theme::SolarizedLight;
                self.theme_shadow = self.theme.clone();
            }
            Theme::GruvboxLight => {
                self.theme = Theme::GruvboxDark;
                self.theme_shadow = self.theme.clone();
            }
            Theme::GruvboxDark => {
                self.theme = Theme::GruvboxLight;
                self.theme_shadow = self.theme.clone();
            }
            Theme::TokyoNight => {
                self.theme = Theme::TokyoNightLight;
                self.theme_shadow = self.theme.clone();
            }
            Theme::TokyoNightLight => {
                self.theme = Theme::TokyoNight;
                self.theme_shadow = self.theme.clone();
            }
            _ => {}
        }
    }

    fn update_tabs(&mut self, tsg: TabsMessage) -> Task<Message> {
        if let Some(response) = self.tabs.update(tsg) {
            Task::perform(async { response }, |response| response)
        } else {
            Task::none()
        }
    }

    fn file_io_action_handler(&mut self, action: FileIOAction, content: String) -> Task<Message> {
        match action {
            FileIOAction::NewTab((View::Editor(data), path)) => {
                let data = data.path(path).data(content);
                let idr = View::Editor(data);
                self.update_tabs(TabsMessage::AddTab(idr))
            }
            FileIOAction::NewTab((View::LineGraph(_), path)) => {
                let data = LineTabData::new(path, LineConfigState::default());
                match data {
                    Err(err) => {
                        let msg = Message::Error(err, true);
                        Task::perform(async { msg }, |msg| msg)
                    }

                    Ok(data) => {
                        let data = data.theme(self.theme.clone());
                        let idr = View::LineGraph(data);
                        self.update_tabs(TabsMessage::AddTab(idr))
                    }
                }
            }
            FileIOAction::NewTab((View::BarChart(_), path)) => {
                let data = BarChartTabData::new(path, BarChartConfigState::default());
                match data {
                    Err(err) => {
                        let msg = Message::Error(err, true);
                        Task::perform(async { msg }, |msg| msg)
                    }

                    Ok(data) => {
                        let data = data.theme(self.theme.clone());
                        let idr = View::BarChart(data);
                        self.update_tabs(TabsMessage::AddTab(idr))
                    }
                }
            }
            FileIOAction::NewTab((View::StackedBarChart(_), path)) => {
                let data = StackedBarChartTabData::new(path, StackedBarChartConfigState::default());
                match data {
                    Err(err) => {
                        let msg = Message::Error(err, true);
                        Task::perform(async { msg }, |msg| msg)
                    }

                    Ok(data) => {
                        let data = data.theme(self.theme.clone());
                        let idr = View::StackedBarChart(data);
                        self.update_tabs(TabsMessage::AddTab(idr))
                    }
                }
            }
            FileIOAction::NewTab((View::None, _)) => self.update_tabs(TabsMessage::None),
            FileIOAction::RefreshTab((ViewType::Editor, tidx, path)) => {
                let data = EditorTabData::new(Some(path), content);
                let rsh = Refresh::Editor(data);
                self.update_tabs(TabsMessage::RefreshTab(tidx, rsh))
            }
            FileIOAction::RefreshTab((ViewType::LineGraph, tidx, path)) => {
                let data = LineTabData::new(path, LineConfigState::default());
                match data {
                    Err(err) => {
                        let msg = Message::Error(err, true);
                        Task::perform(async { msg }, |msg| msg)
                    }
                    Ok(data) => {
                        let rsh = Refresh::LineGraph(data);
                        self.update_tabs(TabsMessage::RefreshTab(tidx, rsh))
                    }
                }
            }
            FileIOAction::RefreshTab((ViewType::BarChart, tidx, path)) => {
                let data = BarChartTabData::new(path, BarChartConfigState::default());
                match data {
                    Err(err) => {
                        let msg = Message::Error(err, true);
                        Task::perform(async { msg }, |msg| msg)
                    }
                    Ok(data) => {
                        let rsh = Refresh::BarChart(data);
                        self.update_tabs(TabsMessage::RefreshTab(tidx, rsh))
                    }
                }
            }
            FileIOAction::RefreshTab((ViewType::StackedBarChart, tidx, path)) => {
                let data = StackedBarChartTabData::new(path, StackedBarChartConfigState::default());
                match data {
                    Err(err) => {
                        let msg = Message::Error(err, true);
                        Task::perform(async { msg }, |msg| msg)
                    }
                    Ok(data) => {
                        let rsh = Refresh::StackedBarChart(data);
                        self.update_tabs(TabsMessage::RefreshTab(tidx, rsh))
                    }
                }
            }
            FileIOAction::RefreshTab((ViewType::None, _, _)) => self.update_tabs(TabsMessage::None),
            FileIOAction::CloseTab(idx) => {
                let tsg = TabsMessage::CloseTab(idx, true);
                self.update_tabs(tsg)
            }
            FileIOAction::Exiting(idx) => {
                self.tabs.update(TabsMessage::CloseTab(idx, true));
                let tsg = TabsMessage::Exit;
                self.update_tabs(tsg)
            }
            FileIOAction::None => Task::none(),
        }
    }

    fn push_toast(&mut self, toast: Toast) {
        match toast.status {
            Status::Info => info!(toast.body),
            Status::Warn => warn!(toast.body),
            Status::Success => info!(toast.body),
            Status::Error => error!(toast.body),
        }

        self.toasts.push(toast);
    }

    fn info_log(&mut self, message: impl Into<String>) {
        let message: String = message.into();
        info!(message);
    }

    fn about(&self) -> Element<'_, Message> {
        let text = text(
            "Yet another one of my projects. I started this particular one to grow more familiar with Rust. I hope to be able to continue this is it for now. 

This app is meant to be a MOdern Data Visualisation (MODAV) tool split into 2 parts. This is the Iced GUI frontend. The backend, modav_core, can be found on my Github profile.
            ",
        );

        dialog_container(text).height(Length::Shrink).into()
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn new(flags: Flags) -> Self {
        flags.create()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Error(err, show_toast) => {
                if show_toast {
                    let toast = Toast {
                        status: Status::Error,
                        body: err.message(),
                    };
                    self.push_toast(toast);
                } else {
                    error!("{}", err.message());
                }
                Task::none()
            }
            Message::IconLoaded(Ok(_)) => {
                self.info_log("Icon Loaded Successfully");
                Task::none()
            }
            Message::IconLoaded(Err(e)) => {
                let error = AppError::FontLoading(e);
                Task::perform(async { error }, |error| Message::Error(error, true))
            }
            Message::ToggleTheme => {
                self.info_log("Theme toggled");
                self.toggle_theme();
                Task::none()
            }
            Message::SelectFile => {
                self.error = AppError::None;
                Task::perform(pick_file(), Message::FileSelected)
            }
            Message::FileSelected(Ok(p)) => {
                self.error = AppError::None;
                self.info_log(format!(
                    "{} file selected",
                    p.file_name()
                        .map(|name| name.to_str().unwrap_or("None"))
                        .unwrap_or("None")
                ));
                self.file_path = Some(p);
                self.dialog_view = DialogView::Wizard;
                Task::none()
            }
            Message::FileSelected(Err(e)) => {
                Task::perform(async { e }, |error| Message::Error(error, true))
            }
            Message::LoadFile((path, action)) => {
                self.error = AppError::None;
                Task::perform(
                    async { (load_file(path).await, action) },
                    |((res, _), action)| Message::FileLoaded((res, action)),
                )
            }
            Message::FileLoaded((Ok(res), action)) => {
                self.info_log("File loaded");
                self.file_io_action_handler(action, res)
            }
            Message::OpenLogFile => {
                self.dialog_view = DialogView::None;
                self.theme = self.theme_shadow.clone();
                self.info_log("Opening Log file");

                let path = self.log_file.clone();
                let data =
                    EditorTabData::new(Some(path.clone()), String::default()).read_only(true);
                let action = FileIOAction::NewTab((View::Editor(data), path.clone()));

                Task::perform(
                    async { (load_file(path).await, action) },
                    |((res, _), action)| Message::FileLoaded((res, action)),
                )
            }

            Message::FileLoaded((Err(err), _)) => {
                Task::perform(async { err }, |error| Message::Error(error, true))
            }
            Message::OpenTab(path, tidr) => {
                self.info_log("Tab opened");
                let path = path.filter(|path| path.is_file());

                match (path, tidr.should_load()) {
                    (Some(path), true) => Task::perform(
                        async move { (load_file(path.clone()).await, tidr) },
                        |((res, path), tidr)| {
                            Message::FileLoaded((res, FileIOAction::NewTab((tidr, path))))
                        },
                    ),
                    (Some(_path), false) => {
                        let idr = match tidr {
                            View::Editor(_) => View::None,
                            View::LineGraph(data) => {
                                let data = data.theme(self.theme.clone());
                                View::LineGraph(data)
                            }
                            View::BarChart(data) => {
                                let data = data.theme(self.theme.clone());
                                View::BarChart(data)
                            }
                            View::StackedBarChart(data) => {
                                let data = data.theme(self.theme.clone());
                                View::StackedBarChart(data)
                            }
                            View::None => View::None,
                        };
                        self.update_tabs(TabsMessage::AddTab(idr))
                    }

                    (None, true) => self.update_tabs(TabsMessage::AddTab(tidr)),
                    (None, false) => {
                        let idr = match tidr {
                            View::Editor(_) => View::None,
                            View::LineGraph(_) => View::None,
                            View::BarChart(_) => View::None,
                            View::StackedBarChart(_) => View::None,
                            View::None => View::None,
                        };
                        self.update_tabs(TabsMessage::AddTab(idr))
                    }
                }
            }
            Message::TabsMessage(tsg) => {
                if let Some(response) = self.tabs.update(tsg) {
                    Task::perform(async { response }, |response| response)
                } else {
                    Task::none()
                }
            }
            Message::SaveFile((path, content, action)) => {
                self.error = AppError::None;

                if self.tabs.active_tab_can_save() {
                    Task::perform(
                        async move { (save_file(path, content).await, action) },
                        |(res, action)| {
                            let action = match &res {
                                Err(_) => action,
                                Ok((path, _)) => action.update_path(path.clone()),
                            };
                            Message::FileSaved((res, action))
                        },
                    )
                } else {
                    Task::none()
                }
            }
            Message::SaveKeyPressed => {
                let save_message = self.save_helper(self.tabs.active_path());
                Task::perform(async { save_message }, |msg| msg)
            }
            Message::FileSaved((Ok((_path, content)), action)) => {
                let toast = Toast {
                    status: Status::Success,
                    body: "Save Successful!".into(),
                };
                self.push_toast(toast);
                self.file_io_action_handler(action, content)
            }
            Message::FileSaved((Err(e), _)) => {
                Task::perform(async { e }, |error| Message::Error(error, true))
            }
            Message::CheckExit => self.update_tabs(TabsMessage::Exit),
            Message::SetMainWindowID(id) => {
                self.main_window_id = Some(id);
                Task::none()
            }
            Message::CanExit => {
                self.info_log("Application closing");
                match self.main_window_id {
                    Some(id) => window::close(id),
                    None => Task::none(),
                }
            }
            Message::NewActiveTab => {
                self.info_log("New active tab");
                self.current_view = self.tabs.active_tab_type().unwrap_or(ViewType::None);
                self.file_path = self.tabs.active_path();
                Task::none()
            }
            Message::Convert => Task::none(),
            Message::None => Task::none(),
            Message::Debugging => {
                dbg!("Debugging Message sent!");
                Task::none()
            }
            Message::CloseWizard => {
                self.dialog_view = DialogView::None;
                Task::perform(async {}, |_| Message::NewActiveTab)
            }
            Message::WizardSubmit(path, view) => {
                self.dialog_view = DialogView::None;
                self.info_log("Wizard Submitted");
                Task::perform(async { Message::OpenTab(Some(path), view) }, |msg| msg)
            }
            Message::ChangeTheme(theme) => {
                self.theme = theme;
                self.info_log("Theme Changed");
                Task::none()
            }
            Message::OpenSettingsDialog => {
                self.dialog_view = DialogView::Settings;
                self.info_log("Settings Dialog Opened");
                Task::none()
            }
            Message::AbortSettings => {
                self.theme = self.theme_shadow.clone();
                self.dialog_view = DialogView::None;
                self.info_log("Settings Aborted");
                Task::none()
            }
            Message::SaveSettings(theme, timeout) => {
                self.theme_shadow = theme.clone();
                self.theme = theme;
                self.toast_timeout = timeout;
                self.dialog_view = DialogView::None;

                let toast = Toast {
                    body: "Settings Saved".into(),
                    status: Status::Success,
                };
                self.push_toast(toast);

                Task::none()
            }
            Message::AddToast(toast) => {
                self.push_toast(toast);
                Task::none()
            }
            Message::CloseToast(index) => {
                self.toasts.remove(index);
                Task::none()
            }
            Message::OpenAboutDialog => {
                self.dialog_view = DialogView::About;
                self.info_log("About dialog open");
                Task::none()
            }
            Message::CloseAboutDialog => {
                self.dialog_view = DialogView::None;
                self.info_log("About dialog closed");
                Task::none()
            }
            Message::OpenEditor(path) => match path {
                Some(path) => {
                    let data = EditorTabData::new(Some(path.clone()), String::default());
                    let msg = Message::OpenTab(self.file_path.clone(), View::Editor(data));

                    Task::perform(async { msg }, |msg| msg)
                }
                None => Task::none(),
            },
            Message::CloseWindow(id) => {
                self.info_log(format!("Closing window with Id: {id}"));
                window::close(id)
            }
            Message::WindowCloseRequested(id) => window::get_oldest().and_then(move |oldest| {
                if id == oldest {
                    Task::done(Message::CheckExit)
                } else {
                    Task::done(Message::CloseWindow(id))
                }
            }),
            Message::KeyPressed(key, modifiers) => match key {
                Key::Named(key::Named::Save) if modifiers.command() => {
                    let save_message = self.save_helper(self.tabs.active_path());
                    Task::perform(async { save_message }, |msg| msg)
                }
                Key::Character(s) if s.as_str() == "s" && modifiers.command() => {
                    let save_message = self.save_helper(self.tabs.active_path());
                    Task::perform(async { save_message }, |msg| msg)
                }
                Key::Named(key::Named::Tab) => {
                    if modifiers.shift() {
                        widget::focus_previous()
                    } else {
                        widget::focus_next()
                    }
                }
                _ => Task::none(),
            },
            Message::Ready => {
                self.is_ready = true;
                Task::none()
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Message, Theme, iced::Renderer> {
        if !self.is_ready {
            let temp = text("Loading...").size(20.);
            let content = container(temp).center(Length::Fill);
            return content.into();
        }

        let status_bar = self.status_bar().height(Length::FillPortion(1));
        let dashboard = self.dashboard();

        let content = if self.tabs.is_empty() {
            home_view().into()
        } else {
            self.tabs.view(Message::TabsMessage)
        };

        let cross_axis = row!(dashboard, content).height(Length::FillPortion(30));

        let main_axis = column!(cross_axis, status_bar);

        let content: Element<'_, Message> = match self.dialog_view {
            DialogView::None => main_axis.into(),
            DialogView::Wizard => {
                let file = self
                    .file_path
                    .clone()
                    .expect("File path was empty for Wizard")
                    .clone();
                let wizard = Wizard::new(file, Message::WizardSubmit, |error| {
                    Message::Error(error, false)
                })
                .on_reselect(Message::SelectFile)
                .on_cancel(Message::CloseWizard);
                Modal::new(main_axis, wizard)
                    .on_blur(Message::CloseWizard)
                    .into()
            }
            DialogView::Settings => {
                let settings = SettingsDialog::new(self.theme.clone(), self.toast_timeout)
                    .on_cancel(Message::AbortSettings)
                    .on_submit(Message::SaveSettings)
                    .on_log(Message::OpenLogFile)
                    .on_theme_change(Message::ChangeTheme);
                Modal::new(main_axis, settings)
                    .on_blur(Message::AbortSettings)
                    .into()
            }
            DialogView::About => Modal::new(main_axis, self.about())
                .on_blur(Message::CloseAboutDialog)
                .into(),
        };

        let content = toast::Manager::new(content, &self.toasts, Message::CloseToast, &self.theme)
            .timeout(self.toast_timeout);

        container(content).height(Length::Fill).into()
    }

    fn subscription(&self) -> Subscription<Message> {
        let close_window = window::close_requests().map(Message::WindowCloseRequested);

        let key_press = keyboard::on_key_press(|key, modifiers| match key {
            Key::Named(key::Named::Save) | Key::Named(key::Named::Tab) => {
                Some(Message::KeyPressed(key, modifiers))
            }
            Key::Character(ref s) if s.as_str() == "s" => Some(Message::KeyPressed(key, modifiers)),
            _ => None,
        });

        Subscription::batch(vec![close_window, key_press])
        //event::listen()
        //    .with(self.main_window_id.clone())
        //    .map(|id, event| Message::Event(id, event))
    }
}
