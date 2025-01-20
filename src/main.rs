use iced::{
    application, font,
    keyboard::{self, key, Key},
    widget::{
        self, button, column, container, container::bordered_box, horizontal_space, pick_list, row,
        text, text_input, vertical_rule, Container, Row, Space,
    },
    window, Alignment, Element, Font, Length, Subscription, Task, Theme,
};

use tracing::{error, info, span, warn, Level};
use tracing_subscriber::EnvFilter;

use std::{fs::File, path::PathBuf};

mod styles;
use styles::*;

mod utils;
use utils::{icons, load_file, pick_file, save_file, AppError};

mod views;
use views::{
    home_view, BarChartTabData, EditorTabData, LineTabData, Refresh, StackedBarChartTabData, Tabs,
    TabsMessage, View, ViewType,
};

pub mod widgets;
use widgets::{
    modal::Modal,
    sidemenu::{Context, Menu, MenuSection, SideMenu},
    style::dialog_container,
    toast::{self, Status, Toast},
    wizard::{BarChartConfigState, LineConfigState, StackedBarChartConfigState, Wizard},
};

const THEMES: [Theme; 7] = [
    Theme::TokyoNight,
    Theme::TokyoNightLight,
    Theme::GruvboxDark,
    Theme::GruvboxLight,
    Theme::SolarizedDark,
    Theme::SolarizedLight,
    Theme::Nightfly,
];

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MenuContext {
    File,
    Settings,
    None,
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
    About,
    #[default]
    None,
}

#[derive(Debug, Clone, PartialEq)]
struct Settings {
    theme: Theme,
    timeout: u64,
    log_file: PathBuf,
}

impl Settings {
    fn new(theme: Theme, timeout: u64, log_file: PathBuf) -> Self {
        Self {
            theme,
            timeout,
            log_file,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsMessage {
    ThemeChange(Theme),
    TimeoutChange(String),
    ReselectLog,
    LogReselect(PathBuf),
    Cancel,
    Save,
}

pub struct Modav {
    title: String,
    current_view: ViewType,
    file_path: Option<PathBuf>,
    tabs: Tabs<Theme>,
    toasts: Vec<Toast>,
    dialog_view: DialogView,
    error: AppError,
    settings: Settings,
    new_settings: Option<Settings>,
    main_window_id: Option<window::Id>,
    is_ready: bool,
    context: MenuContext,
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
        let mut tabs = Tabs::new(theme.clone())
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
        let context = MenuContext::None;

        let mut settings = Settings::new(theme.clone(), toast_timeout, default_log_file);

        match self {
            Self::Prod(log_file) => {
                settings.log_file = log_file;

                Modav {
                    file_path: None,
                    current_view: ViewType::None,
                    new_settings: None,
                    is_ready,
                    title,
                    toasts,
                    settings,
                    main_window_id,
                    error,
                    tabs,
                    dialog_view,
                    context,
                }
            }
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
                    is_ready,
                    current_view,
                    title,
                    settings,
                    new_settings: None,
                    main_window_id,
                    toasts,
                    error,
                    tabs,
                    dialog_view,
                    context,
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
                    new_settings: None,
                    is_ready,
                    current_view,
                    title,
                    main_window_id,
                    settings,
                    toasts,
                    error,
                    tabs,
                    dialog_view,
                    context,
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
                        .theme(theme);
                    let view = View::StackedBarChart(data);
                    tabs.update(TabsMessage::AddTab(view));
                }

                Modav {
                    file_path: Some(file_path),
                    new_settings: None,
                    is_ready,
                    current_view,
                    title,
                    toasts,
                    settings,
                    main_window_id,
                    error,
                    tabs,
                    dialog_view,
                    context,
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    IconLoaded(Result<(), font::Error>),
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
    Settings(SettingsMessage),
    OpenAboutDialog,
    CloseAboutDialog,
    OpenLogFile,
    AddToast(Toast),
    CloseToast(usize),
    Error(AppError, bool),
    MenuContext(MenuContext),
    CloseContext,
    Chain(Box<(Message, Message)>),
    /// Open the editor for the current model
    OpenEditor(Option<PathBuf>),
}

#[allow(dead_code)]
impl Message {
    fn chain(self, message: Self) -> Self {
        let boxed = Box::new((self, message));

        Self::Chain(boxed)
    }

    fn close_context(self) -> Self {
        self.chain(Message::CloseContext)
    }
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

    fn handle_context(&self) -> Element<'_, Message> {
        let header_font = Font {
            weight: font::Weight::Semibold,
            ..Default::default()
        };
        let size = 18;

        match self.context {
            MenuContext::None => Space::with_width(0).into(),
            MenuContext::File => {
                let styler = |theme: &Theme, status: iced::widget::button::Status| {
                    use iced::widget::button::{text, Status, Style};
                    let default = text(theme, status);

                    let background = theme.extended_palette().background.strong;
                    let border = iced::Border::default().rounded(8.0);

                    match status {
                        Status::Hovered => Style {
                            background: Some(iced::Background::Color(background.color)),
                            text_color: background.text,
                            border,
                            ..default
                        },
                        _ => default,
                    }
                };

                let header = text("File Menu").font(header_font).size(size);

                let open = button("Open File")
                    .on_press(Message::SelectFile.close_context())
                    .width(Length::Fill)
                    .style(styler);

                let new = button("New File")
                    .on_press(
                        Message::OpenTab(None, View::Editor(EditorTabData::default()))
                            .close_context(),
                    )
                    .width(Length::Fill)
                    .style(styler);

                let save = button("Save File")
                    .on_press_maybe(
                        self.tabs
                            .active_tab_can_save()
                            .then(|| self.save_helper(self.tabs.active_path()).close_context()),
                    )
                    .width(Length::Fill)
                    .style(styler);

                let save_new = button("Save As")
                    .on_press_maybe(
                        self.tabs
                            .active_tab_can_save()
                            .then(|| self.save_helper(self.tabs.active_path()).close_context()),
                    )
                    .width(Length::Fill)
                    .style(styler);

                let context =
                    context!(Space::with_height(0.0), header, Space::with_height(28.0), open, new, save, save_new ; Message::MenuContext(MenuContext::None))
                    .width(Length::Fill)
                    .spacing(20.0)
                .height(Length::Fill);

                container(context).style(bordered_box).into()
            }
            MenuContext::Settings => {
                let header = text("Settings Menu").size(size).font(header_font);

                let theme = {
                    let label = text("Change Theme:");

                    let pick_list = pick_list(THEMES, Some(self.theme()), |theme| {
                        Message::Settings(SettingsMessage::ThemeChange(theme))
                    });

                    row!(label, pick_list)
                        .spacing(10)
                        .align_y(Alignment::Center)
                };

                let timeout = {
                    let value = self.timeout();

                    let label = text("Toast timeout:");

                    let input = text_input("", &value.to_string())
                        .on_input(|timeout| {
                            Message::Settings(SettingsMessage::TimeoutChange(timeout))
                        })
                        .padding([0, 5])
                        .width(44.0);

                    row!(
                        label,
                        row!(input, text("seconds"))
                            .align_y(Alignment::Center)
                            .spacing(5)
                    )
                    .spacing(20.0)
                };

                let log = button(text("Open Log File").size(15.0))
                    .on_press(Message::OpenLogFile.close_context());

                let actions = {
                    let cancel = button(text("Cancel").size(13.0))
                        .on_press(Message::Settings(SettingsMessage::Cancel).close_context());

                    let submit = button(text("Save").size(13.0))
                        .on_press(Message::Settings(SettingsMessage::Save).close_context());

                    row!(cancel, horizontal_space(), submit)
                };

                let msg = Message::Settings(SettingsMessage::Cancel).close_context();

                let context = context!(
                        Space::with_height(0.0),
                        header,
                        Space::with_height(28.0),
                        theme,
                        timeout,
                        log,
                        Space::with_height(Length::Fill),
                        actions;
                        msg)
                .spacing(20.0)
                .height(Length::Fill);

                container(context).style(bordered_box).into()
            }
        }
    }

    fn side_menu(&self) -> Element<'_, Message> {
        let font = Font {
            family: font::Family::Cursive,
            weight: font::Weight::Bold,
            ..Default::default()
        };

        let size = 18.0;
        let icon_size = f32::max(size * 0.9, 18.0);

        let header = Menu::new(
            icons::icon(icons::SETTINGS).size(24),
            text("modav").font(font).size(24),
        );

        let file = Menu::new(
            icons::icon(icons::FILE).size(icon_size),
            text("File").size(size),
        )
        .width(Length::Fill)
        .message(Message::MenuContext(MenuContext::File));

        let models = Menu::new(
            icons::icon(icons::CHART).size(icon_size),
            text("Models").size(size),
        )
        .width(Length::Fill);

        let about = Menu::new(
            icons::icon(icons::INFO).size(icon_size),
            text("Information").size(size),
        )
        .width(Length::Fill)
        .message(Message::OpenAboutDialog);

        let help = Menu::new(
            icons::icon(icons::HELP).size(icon_size),
            text("Help").size(size),
        )
        .width(Length::Fill);

        let settings = Menu::new(
            icons::icon(icons::SETTINGS).size(icon_size),
            text("Settings").size(size),
        )
        .message(Message::MenuContext(MenuContext::Settings))
        .width(Length::Fill);

        let menu = SideMenu::new(
            header,
            section!(file, models).width(Length::Fill).spacing(20.0),
            section!(about, help, settings).width(Length::Fill),
        )
        .height(Length::Fill);

        let content = container(menu).style(bordered_box);

        content.into()
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
                        let data = data.theme(self.theme());
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
                        let data = data.theme(self.theme());
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
                        let data = data.theme(self.theme());
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
        self.new_settings
            .as_ref()
            .map(|settings| settings.theme.clone())
            .unwrap_or(self.settings.theme.clone())
    }

    fn theme_ref(&self) -> &Theme {
        self.new_settings
            .as_ref()
            .map(|settings| &settings.theme)
            .unwrap_or(&self.settings.theme)
    }

    fn timeout(&self) -> u64 {
        self.new_settings
            .as_ref()
            .map(|settings| settings.timeout)
            .unwrap_or(self.settings.timeout)
    }

    fn log_file(&self) -> &PathBuf {
        self.new_settings
            .as_ref()
            .map(|settings| &settings.log_file)
            .unwrap_or(&self.settings.log_file)
    }

    fn handle_settings_message(&mut self, message: SettingsMessage) -> Task<Message> {
        if let Some(settings) = self.new_settings.as_mut() {
            match message {
                SettingsMessage::ThemeChange(theme) => settings.theme = theme,

                SettingsMessage::TimeoutChange(mut timeout) => {
                    if !timeout.is_empty() {
                        if let Some(first) = timeout.chars().next() {
                            if settings.timeout == 0 && first != '0' {
                                timeout.pop();
                            }
                        }
                        settings.timeout = timeout.parse().unwrap_or(self.settings.timeout);

                        let toast = Toast {
                            body: format!("Toast with {timeout} second timeout."),
                            status: Status::Info,
                        };
                        self.push_toast(toast);
                    } else {
                        settings.timeout = 0;
                    }
                }

                SettingsMessage::Cancel => {
                    self.dialog_view = DialogView::None;
                    self.new_settings = None;
                    self.info_log("Settings Aborted");
                }

                SettingsMessage::Save => {
                    if let Some(settings) = self.new_settings.take() {
                        self.tabs.set_theme(settings.theme.clone());
                        self.settings = settings;
                    }
                    self.dialog_view = DialogView::None;

                    let toast = Toast {
                        body: "Settings Saved".into(),
                        status: Status::Success,
                    };
                    self.push_toast(toast);
                }

                SettingsMessage::ReselectLog => {
                    let msg = Message::SelectFile;
                    return Task::done(msg);
                }

                SettingsMessage::LogReselect(log) => {
                    self.error = AppError::None;
                    settings.log_file = log;
                }
            }
        }
        Task::none()
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
            Message::SelectFile => {
                self.error = AppError::None;
                Task::perform(pick_file(), Message::FileSelected)
            }
            Message::FileSelected(Ok(file)) => {
                self.error = AppError::None;
                self.info_log(format!(
                    "{} file selected",
                    file.file_name()
                        .map(|name| name.to_str().unwrap_or("None"))
                        .unwrap_or("None")
                ));
                self.file_path = Some(file);
                self.dialog_view = DialogView::Wizard;
                Task::none()
            }
            Message::FileSelected(Err(error)) => Task::done(Message::Error(error, true)),
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
                self.info_log("Opening Log file");

                let path = self.log_file().clone();
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
                                let data = data.theme(self.theme());
                                View::LineGraph(data)
                            }
                            View::BarChart(data) => {
                                let data = data.theme(self.theme());
                                View::BarChart(data)
                            }
                            View::StackedBarChart(data) => {
                                let data = data.theme(self.theme());
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
                self.tabs.set_theme(self.theme());
                Task::none()
            }
            Message::MenuContext(context) => {
                self.context = context;
                if context == MenuContext::Settings {
                    self.new_settings = Some(self.settings.clone());
                    self.info_log("Settings Dialog Opened");
                }
                Task::none()
            }
            Message::CloseContext => {
                self.context = MenuContext::None;
                Task::none()
            }
            Message::Settings(message) => self.handle_settings_message(message),
            Message::Chain(messages) => {
                let (msg1, msg2) = *messages;

                let tsk1 = Task::done(msg1);
                let tsk2 = Task::done(msg2);

                tsk1.chain(tsk2)
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Message, Theme, iced::Renderer> {
        if !self.is_ready {
            let temp = text("Loading...").size(20.);
            let content = container(temp).center(Length::Fill);
            return content.into();
        }

        let status_bar = self.status_bar().height(30.0);

        let content = container(if self.tabs.is_empty() {
            home_view().into()
        } else {
            self.tabs.view(Message::TabsMessage)
        })
        .height(Length::Fill);

        let content = column!(content, status_bar).height(Length::Fill);

        let cross_axis = row!(self.side_menu(), self.handle_context(), content);

        let main_axis = column!(cross_axis);

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
            DialogView::About => Modal::new(main_axis, self.about())
                .on_blur(Message::CloseAboutDialog)
                .into(),
        };

        let content = toast::Manager::new(
            content,
            &self.toasts,
            Message::CloseToast,
            &self.theme_ref(),
        )
        .timeout(self.timeout());

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
