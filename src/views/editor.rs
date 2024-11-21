use iced::{
    highlighter::{self, Highlighter},
    widget::text_editor,
    Element, Font, Length, Renderer, Theme,
};
use std::path::PathBuf;

use super::{TabLabel, Viewable};
use crate::{utils::icons, Message};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct EditorTabData {
    path: Option<PathBuf>,
    data: String,
    read_only: bool,
}

impl EditorTabData {
    pub fn new(path: Option<PathBuf>, data: String) -> Self {
        Self {
            path,
            data,
            read_only: false,
        }
    }

    pub fn path(mut self, path: PathBuf) -> Self {
        self.path = Some(path);
        self
    }

    pub fn data(mut self, data: String) -> Self {
        self.data = data;
        self
    }

    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }
}

#[derive(Debug)]
pub struct EditorTab {
    is_dirty: bool,
    file_path: Option<PathBuf>,
    content: text_editor::Content,
    is_empty: bool,
    read_only: bool,
}

#[derive(Debug, Clone)]
pub enum EditorMessage {
    Action(text_editor::Action),
    Refresh(EditorTabData),
}

impl Viewable for EditorTab {
    type Data = EditorTabData;
    type Event = EditorMessage;

    fn new(data: Self::Data) -> Self {
        let EditorTabData {
            path,
            data,
            read_only,
        } = data;
        let is_empty = data.is_empty();
        let content = text_editor::Content::with_text(data.as_str());
        Self {
            content,
            is_empty,
            read_only,
            is_dirty: false,
            file_path: path,
        }
    }

    fn is_dirty(&self) -> bool {
        if self.read_only {
            false
        } else {
            self.is_dirty
        }
    }

    fn label(&self) -> TabLabel {
        let path = self.title();
        let font = Font::with_name(icons::NAME);

        let icon = if self.is_empty {
            icons::NEW_FILE
        } else {
            icons::FILE
        };

        TabLabel::new(icon, path).icon_font(font)
    }

    fn title(&self) -> String {
        let path = self
            .file_path
            .as_ref()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("");

        if path.is_empty() {
            "Untitled".into()
        } else {
            path.into()
        }
    }

    fn update(&mut self, message: Self::Event) -> Option<Message> {
        match message {
            EditorMessage::Action(text_editor::Action::Edit(edit)) => {
                if self.read_only {
                    return None;
                }

                self.is_dirty = true;
                match &edit {
                    text_editor::Edit::Insert(_) => {
                        self.is_empty = false;
                    }
                    text_editor::Edit::Paste(_) => {
                        self.is_empty = false;
                    }
                    _ => {}
                };

                self.content.perform(text_editor::Action::Edit(edit));
            }
            EditorMessage::Action(act) => {
                self.content.perform(act);
            }
            EditorMessage::Refresh(data) => {
                self.is_dirty = false;
                self.refresh(data);
            }
        }

        None
    }

    fn content(&self) -> Option<String> {
        self.content.text().into()
    }

    fn view<'a, Message, F>(&'a self, map: F) -> Element<'a, Message, iced::Theme, iced::Renderer>
    where
        F: 'a + Fn(Self::Event) -> Message,
        Message: 'a,
    {
        let extension = self
            .path()
            .as_ref()
            .and_then(|path| path.extension()?.to_str())
            .unwrap_or("txt")
            .to_string();
        let highlighter_settings = highlighter::Settings {
            token: extension,
            theme: highlighter::Theme::SolarizedDark,
        };

        let content: Element<EditorMessage, Theme, Renderer> = text_editor(&self.content)
            .on_action(EditorMessage::Action)
            .height(Length::Fill)
            .padding([4, 8])
            .highlight_with::<Highlighter>(highlighter_settings, |hl, _theme| hl.to_format())
            .into();

        content.map(map)
    }

    fn modal_msg(&self) -> String {
        if self.is_dirty {
            format!("Do you want to save changes to {}?", self.title())
        } else {
            "Editor Modal msg".into()
        }
    }

    fn refresh(&mut self, data: Self::Data) {
        self.is_empty = data.data.is_empty();
        self.file_path = data.path;
        self.is_dirty = false;
    }

    fn path(&self) -> Option<PathBuf> {
        self.file_path.clone()
    }

    fn can_save(&self) -> bool {
        !self.read_only
    }
}
