use std::path::PathBuf;

use iced::{
    theme::{self, Theme},
    widget::{button, column, container, horizontal_space, row, text, Button, Container, Row},
    Length, Renderer,
};
use iced_aw::TabLabel;

pub mod tabs;
pub use tabs::Refresh;
use tabs::{TabBarMessage, TabState};

mod editor;
pub use editor::EditorTabData;

mod temp;

use super::Message;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum View {
    Counter,
    Editor(EditorTabData),
    #[default]
    None,
}

pub trait Viewable {
    type Message;
    type Data;

    fn new(id: usize, data: Self::Data) -> Self;

    fn id(&self) -> usize;

    fn is_dirty(&self) -> bool;

    fn update(&mut self, message: Self::Message);

    fn tab_label(&self) -> TabLabel;

    /// Returns the content of this view if any
    fn content(&self) -> Option<String> {
        None
    }

    fn refresh(&mut self, data: Self::Data);

    fn modal_msg(&self) -> String;

    fn title(&self) -> String {
        String::default()
    }

    fn view(&self) -> iced::Element<'_, TabBarMessage, Theme, Renderer>;

    fn path(&self) -> Option<PathBuf> {
        None
    }
}

#[derive(Debug, Clone)]
pub enum ViewType {
    Counter,
    Editor,
}

impl ViewType {
    /// Returns true if this tab requires the contents of a file to be loaded
    pub fn should_load(&self) -> bool {
        match self {
            Self::Counter => false,
            Self::Editor => true,
        }
    }
}

pub fn home_view<'a>() -> Container<'a, Message, Theme, Renderer> {
    let new_btn: Button<'_, Message, Theme, Renderer> = button("New File")
        .on_press(Message::OpenTab(None, ViewType::Editor))
        .style(theme::Button::Text);
    let open_btn: Button<'_, Message, Theme, Renderer> = button("Open File")
        .on_press(Message::SelectFile)
        .style(theme::Button::Text);
    let recents_btn: Button<'_, Message, Theme, Renderer> = button("Recent Files")
        .on_press(Message::None)
        .style(theme::Button::Text);
    let options: Row<'_, Message, Theme, Renderer> = row!(
        horizontal_space(),
        column!(new_btn, open_btn, recents_btn).spacing(8),
        horizontal_space()
    )
    .width(Length::Fill);

    let logo = row!(
        horizontal_space(),
        text("modav logo").size(40),
        horizontal_space(),
    )
    .width(Length::Fill);

    let content = column!(logo, options).spacing(24).width(Length::Fill);

    container(content)
        .width(Length::FillPortion(5))
        .height(Length::Fill)
        .center_y()
}

pub type Tabs = TabState;
pub type TabsMessage = TabBarMessage;
