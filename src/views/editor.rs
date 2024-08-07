use iced::{
    highlighter::{self, Highlighter},
    widget::text_editor,
    Element, Length, Renderer, Theme,
};
use std::path::PathBuf;

use super::{TabLabel, Viewable};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct EditorTabData {
    path: Option<PathBuf>,
    data: String,
}

impl EditorTabData {
    pub fn new(path: Option<PathBuf>, data: String) -> Self {
        Self { path, data }
    }
}

#[derive(Debug)]
pub struct EditorTab {
    is_dirty: bool,
    file_path: Option<PathBuf>,
    content: text_editor::Content,
}

#[derive(Debug, Clone)]
pub enum EditorMessage {
    Action(text_editor::Action),
    Refresh(EditorTabData),
}

impl Viewable for EditorTab {
    type Data = EditorTabData;
    type Message = EditorMessage;

    fn new(data: Self::Data) -> Self {
        let EditorTabData { path, data } = data;
        let content = text_editor::Content::with_text(data.as_str());
        Self {
            is_dirty: false,
            content,
            file_path: path,
        }
    }

    fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    fn label(&self) -> TabLabel {
        let path = self.title();

        TabLabel::new('\u{F0F6}', path)
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

    fn update(&mut self, message: Self::Message) {
        match message {
            EditorMessage::Action(text_editor::Action::Edit(edit)) => {
                self.is_dirty = true;
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
    }

    fn content(&self) -> Option<String> {
        self.content.text().into()
    }

    fn view<'a, Message, F>(&'a self, map: F) -> Element<'a, Message, iced::Theme, iced::Renderer>
    where
        F: 'a + Fn(Self::Message) -> Message,
        Message: 'a,
    {
        let extension = self
            .path()
            .as_ref()
            .and_then(|path| path.extension()?.to_str())
            .unwrap_or("txt")
            .to_string();
        let highlighter_settings = highlighter::Settings {
            extension,
            theme: highlighter::Theme::SolarizedDark,
        };

        let content: Element<EditorMessage, Theme, Renderer> = text_editor(&self.content)
            .on_action(EditorMessage::Action)
            .height(Length::Fill)
            .padding([4, 8])
            .highlight::<Highlighter>(highlighter_settings, |hl, _theme| hl.to_format())
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
        self.file_path = data.path;
        self.is_dirty = false;
    }

    fn path(&self) -> Option<PathBuf> {
        self.file_path.clone()
    }

    fn can_save(&self) -> bool {
        true
    }
}
