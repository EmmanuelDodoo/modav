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

        let row: Row<'_, Message> = row!(error, horizontal_space(Length::Fill), current).height(20);

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
                theme: Theme::Light,
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
                match self.theme {
                    Theme::Dark => self.theme = Theme::Light,
                    Theme::Light => self.theme = Theme::Dark,
                    Theme::Custom(_) => {}
                }
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

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
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
        use iced_aw::{menu_bar, menu_tree, native::menu_tree, style, MenuBar, MenuTree};

        use crate::{
            styles::{ColoredContainer, CustomMenuBarStyle},
            views::Identifier,
        };

        use super::{icon, MenuButtonStyle, Message};

        use iced::{
            color,
            theme::{self},
            widget::{button, container, row, text, Button, Container, Row, Text},
            Element, Length, Renderer,
        };

        fn dash_icon(unicode: char) -> Text<'static> {
            icon(unicode, "dash-icons")
        }

        /// The last item in a Menu Tree
        fn base_tree(label: &str, msg: Message) -> MenuTree<'_, Message, Renderer> {
            let btn = button(text(label).width(Length::Fill).height(Length::Fill))
                .on_press(msg)
                .style(theme::Button::Custom(Box::new(MenuButtonStyle {})))
                .padding([4, 8])
                .width(Length::Fill)
                .height(Length::Shrink);

            menu_tree!(btn)
        }

        fn create_children(labels: Vec<(&str, Message)>) -> Vec<MenuTree<'_, Message, Renderer>> {
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
            label: impl Into<Element<'a, Message, Renderer>>,
        ) -> Row<'a, Message> {
            let icon = dash_icon(icon);
            row!(icon, label.into())
                .spacing(8)
                .padding([0, 8])
                .width(Length::Fill)
        }

        fn create_menu<'a>(
            label: impl Into<Element<'a, Message, Renderer>>,
            icon: char,
            children: Vec<impl Into<MenuTree<'a, Message, Renderer>>>,
        ) -> MenuBar<'a, Message, Renderer> {
            let label = create_label(icon, label.into());
            let item = container(label).width(Length::Fill);

            menu_bar!(menu_tree(item, children))
                .bounds_expand(30)
                .main_offset(5)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(style::MenuBarStyle::Custom(Box::new(CustomMenuBarStyle)))
        }

        fn container_wrap<'a>(
            item: impl Into<Element<'a, Message, Renderer>>,
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
            let children: Vec<MenuTree<'a, Message, Renderer>> = create_children(actions_label);

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
            let btn: Button<'_, Message, Renderer> =
                button(label).style(theme::Button::Text).padding([0, 0]);

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
        Color, Theme,
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
            container::Appearance {
                border_width: self.width,
                border_color: self.bcolor,
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
            container::Appearance {
                background: Some(self.color.into()),
                border_radius: self.radius.into(),
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
            Theme::Custom(_) => BorderedContainer::default(),
        }
    }

    pub struct MenuButtonStyle;
    impl button::StyleSheet for MenuButtonStyle {
        type Style = iced::Theme;

        fn active(&self, style: &Self::Style) -> button::Appearance {
            button::Appearance {
                text_color: style.extended_palette().background.base.text,
                border_radius: [4.0; 4].into(),
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

        fn appearance(&self, style: &Self::Style) -> iced_aw::menu::Appearance {
            iced_aw::menu::Appearance {
                border_radius: [8.0; 4],
                background: style.palette().background,
                path: Color::TRANSPARENT,
                ..Default::default()
            }
        }
    }
}
