use iced::{widget::text_editor, Element, Length, Renderer, Theme};
use iced_aw::TabLabel;
use std::path::PathBuf;

use super::{TabMessage, Viewable};

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
    id: usize,
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

    fn id(&self) -> usize {
        self.id
    }

    fn new(id: usize, data: Self::Data) -> Self {
        let EditorTabData { path, data } = data;
        let content = text_editor::Content::with_text(data.as_str());
        Self {
            id,
            is_dirty: false,
            content,
            file_path: path,
        }
    }

    fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    fn tab_label(&self) -> TabLabel {
        let path = self.title();

        TabLabel::IconText('\u{F0F6}', path)
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

    fn view(&self) -> Element<'_, super::TabBarMessage, iced::Theme, iced::Renderer> {
        let content: Element<'_, EditorMessage, Theme, Renderer> = text_editor(&self.content)
            .on_action(EditorMessage::Action)
            .height(Length::Fill)
            .padding([4; 4])
            .into();

        content.map(|msg| super::TabBarMessage::UpdateTab((self.id, TabMessage::Editor(msg))))
    }

    fn modal_msg(&self) -> String {
        if self.is_dirty {
            format!("Do you want to save changes to {}?", self.title())
        } else {
            "Editor Modal msg".into()
        }
    }

    fn refresh(&mut self, data: Self::Data) {
        let EditorTabData { path, data } = data;
        self.file_path = path;
        self.is_dirty = false;
        self.content = text_editor::Content::with_text(data.as_str());
    }

    fn path(&self) -> Option<PathBuf> {
        self.file_path.clone()
    }
}
