use iced::{
    executor, theme,
    widget::{column, container, horizontal_space, row, text, vertical_space, Column, Row, Text},
    Application, Command, Length, Renderer, Settings, Theme,
};
use std::{fmt, path::PathBuf};

use menu::{create_menu, pick_file};
use styles::BorderedContainer;

fn main() -> Result<(), iced::Error> {
    TApp::run(Settings::default())
}

#[derive(Clone, Debug, PartialEq)]
pub enum TError {
    DialogClosed,
}

impl fmt::Display for TError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DialogClosed => write!(f, "Dialog closed prematurely"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum TMessage {
    #[default]
    None,
    ToggleTheme,
    Open,
    FileOpened(Result<PathBuf, TError>),
    Quit,
}

#[derive(Debug, Clone)]
struct TApp {
    title: String,
    recent_error: Option<TError>,
    current_file_path: PathBuf,
    theme: Theme,
    is_dark: bool,
}

impl Application for TApp {
    type Theme = Theme;
    type Flags = ();
    type Message = TMessage;
    type Executor = executor::Default;

    fn title(&self) -> String {
        self.title.clone()
    }

    fn theme(&self) -> Self::Theme {
        self.theme.clone()
    }

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            TApp {
                current_file_path: PathBuf::new(),
                title: String::from("Playground"),
                recent_error: None,
                theme: Theme::Light,
                is_dark: false,
            },
            Command::none(),
        )
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            TMessage::ToggleTheme => {
                self.is_dark = !self.is_dark;
                self.theme = if self.is_dark {
                    Theme::Dark
                } else {
                    Theme::Light
                };
                self.recent_error = None;
                Command::none()
            }
            TMessage::None => {
                self.recent_error = None;
                Command::none()
            }
            TMessage::Open => {
                self.recent_error = None;
                Command::perform(pick_file(), TMessage::FileOpened)
            }
            TMessage::FileOpened(Ok(p)) => {
                self.current_file_path = p;
                self.recent_error = None;
                Command::none()
            }
            TMessage::FileOpened(Err(err)) => {
                self.recent_error = Some(err);
                Command::none()
            }
            TMessage::Quit => {
                std::process::exit(0);
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let rw = row!(
            horizontal_space(Length::Fixed(5.0)),
            create_menu(),
            horizontal_space(Length::Fill)
        );
        let subject = container(rw).style(theme::Container::Custom(Box::new(BorderedContainer {})));

        let path: Text<'_, Renderer> = {
            if let Some(s) = self.current_file_path.clone().to_str() {
                text(s)
            } else {
                text("No valid path to show")
            }
        };

        let status = {
            let err_text = match self.recent_error.clone() {
                Some(err) => text(format!("{}", err)),
                None => text(""),
            };
            let path = match self.current_file_path.to_str() {
                Some(p) => text(p),
                None => text(""),
            };
            let row: Row<'_, TMessage> =
                row!(err_text, horizontal_space(Length::Fill), path).padding([0, 10]);

            container(row).style(theme::Container::Custom(Box::new(BorderedContainer {})))
        };

        let col: Column<'_, TMessage, Renderer> = column!(
            subject,
            vertical_space(Length::Fill),
            path,
            vertical_space(Length::Fill),
            status
        );

        container(col).into()
    }
}

mod menu {
    use super::{
        styles::MenuButtonStyle,
        {TError, TMessage},
    };
    use iced::widget::{button, text};
    use iced::{theme, Element, Length, Renderer};
    use iced_aw::{menu_tree, MenuBar};
    use iced_aw::{
        native::helpers::{menu_bar, menu_tree},
        MenuTree,
    };
    use std::path::PathBuf;

    pub async fn pick_file() -> Result<PathBuf, TError> {
        let handle = rfd::AsyncFileDialog::new()
            .pick_file()
            .await
            .ok_or(TError::DialogClosed)?;

        Ok(handle.path().into())
    }

    pub fn create_menu<'a>() -> MenuBar<'a, TMessage, Renderer> {
        let menus: Vec<MenuTree<'_, TMessage, Renderer>> =
            vec![create_file_menu(), create_view_menu(), create_help_menu()];
        menu_bar(menus)
            .spacing(15.0)
            .padding([5, 0])
            .main_offset(15)
            .bounds_expand(30)
    }

    fn create_view_menu<'a>() -> MenuTree<'a, TMessage, Renderer> {
        let labels = vec![
            ("Toggle Theme", TMessage::ToggleTheme),
            ("Theme", TMessage::None),
        ];
        let children = create_children(labels);
        let name = create_name("View", None);
        menu_tree(name, children)
    }

    fn create_help_menu<'a>() -> MenuTree<'a, TMessage, Renderer> {
        let name = create_name("Help", None);

        menu_tree!(name)
    }

    fn base_tree<'a>(label: &'a str, msg: TMessage) -> MenuTree<'a, TMessage, Renderer> {
        let btn = button(text(label).width(Length::Fill).height(Length::Fill))
            .on_press(msg)
            .style(theme::Button::Custom(Box::new(MenuButtonStyle {})))
            .padding([4, 8])
            .width(Length::Fill)
            .height(Length::Shrink);

        menu_tree!(btn)
    }

    fn create_children<'a>(
        labels: Vec<(&'a str, TMessage)>,
    ) -> Vec<MenuTree<'a, TMessage, Renderer>> {
        labels
            .into_iter()
            .map(|curr| {
                let label = curr.0;
                let msg = curr.1;
                base_tree(label, msg)
            })
            .collect()
    }

    fn create_name<'a>(
        name: &'a str,
        msg: Option<TMessage>,
    ) -> impl Into<Element<'a, TMessage, Renderer>> {
        let content = text(name).width(Length::Fill).height(Length::Fill);

        let btn = match msg {
            Some(msg) => button(content).on_press(msg),
            None => button(content),
        }
        .style(theme::Button::Custom(Box::new(MenuButtonStyle {})))
        .padding([4, 8]);

        btn
    }

    fn create_file_menu<'a>() -> MenuTree<'a, TMessage, Renderer> {
        let labels = vec![("Open File", TMessage::Open), ("Exit", TMessage::Quit)];
        let children = create_children(labels);
        let name = create_name("File", None);
        menu_tree(name, children)
    }
}

mod styles {
    use iced::{
        widget::{button, container},
        Color, Theme,
    };

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

    pub struct BorderedContainer;
    impl container::StyleSheet for BorderedContainer {
        type Style = Theme;

        fn appearance(&self, _style: &Self::Style) -> container::Appearance {
            let color = Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            };

            container::Appearance {
                border_width: 0.5,
                border_color: color,
                ..Default::default()
            }
        }
    }
}
