use iced::{
    color, executor, font, theme,
    widget::{column, container, horizontal_space, row, text, vertical_rule, Container, Row},
    Application, Command, Font, Length, Settings, Theme,
};
use std::path::PathBuf;

// mod temp;
use styles::*;
use utils::*;
mod views;
use views::{home_view, Identifier, TabBarMessage, Tabs};

fn main() -> Result<(), iced::Error> {
    Modav::run(Settings::default())
}

#[derive(Clone)]
pub struct Modav {
    theme: Theme,
    title: String,
    current_model: String,
    file_path: PathBuf,
    tabs: Tabs,
    error: AppError,
}

#[derive(Debug, Clone)]
pub enum Message {
    IconLoaded(Result<(), font::Error>),
    ToggleTheme,
    OpenFile,
    FileOpened(Result<PathBuf, AppError>),
    SaveFile,
    Convert,
    None,
    OpenTab(Identifier),
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
                let path = match self.file_path.file_name() {
                    Some(s) => s.to_str().unwrap_or(""),
                    None => "",
                };
                if path.is_empty() {
                    None
                } else {
                    let t = text(path);
                    let icon = status_icon('\u{F0F6}');
                    Some(row!(icon, t).spacing(5))
                }
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
            menus::file_menu(),
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
        ];
        (
            Modav {
                file_path: PathBuf::new(),
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
            Message::OpenFile => {
                self.error = AppError::None;
                Command::perform(pick_file(), Message::FileOpened)
            }
            Message::FileOpened(Ok(p)) => {
                self.file_path = p;
                self.error = AppError::None;
                Command::none()
            }
            Message::FileOpened(Err(e)) => {
                self.error = e;
                Command::none()
            }
            Message::OpenTab(idr) => {
                if let Some(response) = self.tabs.update(TabBarMessage::AddTab(idr)) {
                    return Command::perform(async { response }, |response| response);
                }
                Command::none()
            }
            Message::TabsMessage(tsg) => {
                if let Some(response) = self.tabs.update(tsg) {
                    Command::perform(async { response }, |response| response)
                } else {
                    Command::none()
                }
            }
            Message::Convert => Command::none(),
            Message::SaveFile => Command::none(),
            Message::None => Command::none(),
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
    use std::path::PathBuf;

    #[derive(Debug, Clone, PartialEq, Default)]
    pub enum AppError {
        FontLoading(font::Error),
        FileDialogClosed,
        #[default]
        None,
    }

    impl AppError {
        pub fn message(&self) -> String {
            match self {
                Self::FontLoading(_) => String::from("Error while loading a font"),
                Self::FileDialogClosed => String::from("File Dialog closed prematurely"),
                Self::None => String::new(),
            }
        }
    }

    impl Display for AppError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let msg = self.message();
            match self {
                Self::FontLoading(e) => e.fmt(f),
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
            views::Identifier,
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

        fn create_children(
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

        fn create_menu<'a>(
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

        fn container_wrap<'a>(
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

        pub fn file_menu<'a>() -> Container<'a, Message> {
            let actions_label = vec![
                ("Open File", Message::OpenFile),
                ("Save File", Message::SaveFile),
            ];
            let children: Vec<Item<'a, Message, Theme, Renderer>> = create_children(actions_label);

            let bar = create_menu("File", '\u{F15C}', children);

            container_wrap(bar)
        }

        pub fn models_menu<'a>() -> Container<'a, Message> {
            let actions_labels = vec![("Line Graph", Message::Convert)];

            let children = create_children(actions_labels);

            let bar = create_menu("Models", '\u{E802}', children);

            container_wrap(bar)
        }

        pub fn views_menu<'a>() -> Container<'a, Message> {
            let action_labels = vec![("Add Counter", Message::OpenTab(Identifier::Counter))];

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
