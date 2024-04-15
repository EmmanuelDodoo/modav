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

use modav_core::repr::csv::utils::CSVError;

#[derive(Debug, Default)]
pub enum AppError {
    FontLoading(font::Error),
    FileDialogClosed,
    FileLoading(io::ErrorKind),
    FileSaving(io::ErrorKind),
    CSVError(CSVError),
    Simple(String),
    #[default]
    None,
}

impl Clone for AppError {
    fn clone(&self) -> Self {
        match self {
            Self::FileSaving(err) => Self::FileSaving(err.clone()),
            Self::FileLoading(err) => Self::FileLoading(err.clone()),
            Self::FileDialogClosed => Self::FileDialogClosed,
            Self::FontLoading(err) => Self::FontLoading(err.clone()),
            Self::Simple(s) => Self::Simple(s.clone()),
            Self::CSVError(err) => AppError::Simple(err.to_string()),
            Self::None => Self::None,
        }
    }
}

impl AppError {
    pub fn message(&self) -> String {
        match self {
            Self::FontLoading(_) => String::from("Error while loading a font"),
            Self::FileDialogClosed => String::from("File Dialog closed prematurely"),
            Self::FileLoading(_) => String::from("Error while loading file"),
            Self::FileSaving(_) => String::from("Error while saving file"),
            Self::Simple(s) => s.clone(),
            Self::CSVError(err) => err.to_string(),
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
            Self::CSVError(err) => std::fmt::Display::fmt(err, f),
            Self::Simple(s) => write!(f, "{s}"),
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
        ViewType,
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

    pub fn views_menu<'a, F>(on_select: F) -> Container<'a, Message>
    where
        F: Fn(ViewType) -> Message,
    {
        let action_labels = {
            vec![
                ("Add Counter", (on_select)(ViewType::Counter)),
                ("Open Editor", (on_select)(ViewType::Editor)),
            ]
        };

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
