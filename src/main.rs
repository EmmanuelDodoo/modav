use iced::{
    executor, theme,
    widget::{column, container, horizontal_space, row, text, vertical_space, Column, Text},
    Application, Command, Length, Renderer, Settings, Theme,
};
use std::path::PathBuf;

fn main() -> Result<(), iced::Error> {
    TApp::run(Settings::default())
}

#[derive(Clone, Debug, PartialEq)]
pub enum TError {
    DialogClosed,
}
#[derive(Debug, Clone, PartialEq, Default)]
pub enum TMessage {
    #[default]
    None,
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
            TMessage::None => Command::none(),
            TMessage::Quit => {
                std::process::exit(0);
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let subject = row!();

        let path: Text<'_, Renderer> = {
            if let Some(s) = self.current_file_path.clone().to_str() {
                text(s)
            } else {
                text("No valid path to show")
            }
        };

        let col: Column<'_, TMessage, Renderer> = column!(
            subject,
            vertical_space(Length::Fill),
            path,
            vertical_space(Length::Fill),
        );

        container(col).into()
    }
}
}
