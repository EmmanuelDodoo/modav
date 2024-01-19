use iced::{
    color, executor, font, theme,
    widget::{
        column, container, horizontal_space, row, text, vertical_rule, vertical_space, Container,
        Row,
    },
    Application, Command, Length, Settings, Theme,
};
use std::path::PathBuf;

mod temp;
use styles::*;
// use temp::TApp;
use utils::*;

fn main() -> Result<(), iced::Error> {
    Modav::run(Settings::default())
    // TApp::run(Settings::default())
}

#[derive(Clone, Debug, PartialEq)]
pub struct Modav {
    theme: Theme,
    title: String,
    current_model: String,
    file_path: PathBuf,
    error: AppError,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    FontLoaded(Result<(), font::Error>),
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

        let bstyle = match self.theme {
            Theme::Dark => BorderedContainer {
                bcolor: color!(0xffffff),
                ..Default::default()
            },
            Theme::Light => BorderedContainer::default(),
            Theme::Custom(_) => BorderedContainer::default(),
        };

        container(row)
            .padding([3, 10])
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
        (
            Modav {
                file_path: PathBuf::new(),
                title: String::from("Modav"),
                theme: Theme::Dark,
                current_model: String::new(),
                error: AppError::None,
            },
            font::load(include_bytes!("../fonts/status-icons.ttf").as_slice())
                .map(Message::FontLoaded),
        )
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::FontLoaded(Ok(_)) => Command::none(),
            Message::FontLoaded(Err(e)) => {
                self.error = AppError::Font(e);
                Command::none()
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let status_bar = self.status_bar();
        let text = text("Some text");

        let main_axis = column!(text, vertical_space(Length::Fill), status_bar);

        container(main_axis).into()
    }
}

mod utils {
    use std::fmt::{Debug, Display};

    use iced::{
        alignment, font,
        widget::{text, Text},
        Font,
    };

    #[derive(Debug, Clone, PartialEq, Default)]
    pub enum AppError {
        Font(font::Error),
        #[default]
        None,
    }

    impl AppError {
        pub fn message(&self) -> String {
            match self {
                Self::Font(_) => String::from("Error while loading a font"),
                Self::None => String::new(),
            }
        }
    }

    impl Display for AppError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Font(e) => e.fmt(f),
                Self::None => write!(f, ""),
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
}

pub mod styles {
    use iced::{color, widget::container, Color, Theme};

    pub struct BorderedContainer {
        pub width: f32,
        pub bcolor: Color,
    }

    impl Default for BorderedContainer {
        fn default() -> Self {
            Self {
                width: 0.5,
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
}
