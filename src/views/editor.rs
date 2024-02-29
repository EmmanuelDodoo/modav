use iced::{widget::text_editor, Element, Length, Renderer, Theme};
use iced_aw::TabLabel;
use std::path::PathBuf;

use super::{TabMessage, Viewable};

#[derive(Debug, Clone, PartialEq)]
pub struct EditorTabData {
    path: PathBuf,
    data: String,
}

impl EditorTabData {
    pub fn new(path: PathBuf, data: String) -> Self {
        Self { path, data }
    }
}

#[derive(Debug)]
pub struct EditorTab {
    id: usize,
    is_dirty: bool,
    file_path: PathBuf,
    content: text_editor::Content,
}

#[derive(Debug, Clone)]
pub enum EditorMessage {
    Action(text_editor::Action),
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
        let path = {
            let path = match self.file_path.file_name() {
                Some(s) => s.to_str().unwrap_or(""),
                None => "",
            };
            if path.is_empty() {
                "Untitled"
            } else {
                path
            }
        };
        // let icon = status_icon('\u{F0F6}');

        TabLabel::Text(path.into())
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
        }
    }

    fn content(&self) -> Element<'_, super::TabBarMessage, iced::Theme, iced::Renderer> {
        let content: Element<'_, EditorMessage, Theme, Renderer> = text_editor(&self.content)
            .on_action(EditorMessage::Action)
            .height(Length::Fill)
            .padding([4; 4])
            .into();

        content.map(|msg| super::TabBarMessage::UpdateTab((self.id, TabMessage::Editor(msg))))
    }
}
