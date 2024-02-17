use std::usize;

use iced::{
    theme,
    widget::{button, column, container, horizontal_space, row, text, Button, Container, Row},
    Element, Font, Length, Renderer,
};
use iced_aw::{TabBarPosition, TabLabel};

use crate::temp::{CounterMessage, CounterTab};

use super::Message;

#[derive(Debug, Clone)]
pub enum TabMessage {
    Counter(CounterMessage),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Identifier {
    Counter,
}

#[derive(Clone, Debug)]
pub enum TabType {
    Counter(CounterTab),
}

impl TabType {
    fn id(&self) -> usize {
        match self {
            TabType::Counter(tab) => tab.id(),
        }
    }

    fn update(&mut self, tsg: TabMessage) {
        match (self, tsg) {
            (TabType::Counter(tab), TabMessage::Counter(tsg)) => tab.update(tsg),
            (TabType::Counter(_), _) => {}
        }
    }

    fn is_dirty(&self) -> bool {
        match self {
            TabType::Counter(tab) => tab.is_dirty(),
        }
    }

    fn tab_label(&self) -> TabLabel {
        match self {
            TabType::Counter(tab) => tab.tab_label(),
        }
    }

    fn content(&self) -> Element<'_, TabBarMessage> {
        match self {
            TabType::Counter(tab) => tab.content(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TabBarMessage {
    TabSelected(usize),
    AddTab(Identifier),
    CloseTab(usize),
    UpdateTab((usize, TabMessage)),
    OpenFile,
}

#[derive(Clone)]
pub struct TabState {
    tabs: Vec<TabType>,
    id_counter: usize,
    active_tab: usize,
    close_size: Option<f32>,
    height: Option<Length>,
    icon_font: Option<Font>,
    icon_size: Option<f32>,
    tab_bar_height: Option<Length>,
    tab_bar_width: Option<Length>,
    tab_bar_max_height: Option<f32>,
    tab_bar_position: Option<TabBarPosition>,
    tab_label_padding: Option<f32>,
    tab_label_spacing: Option<f32>,
    text_font: Option<Font>,
    text_size: Option<f32>,
    width: Option<Length>,
}

#[allow(dead_code)]
impl TabState {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            id_counter: 0,
            active_tab: 0,
            close_size: None,
            height: None,
            width: None,
            icon_size: None,
            icon_font: None,
            tab_label_spacing: None,
            tab_label_padding: None,
            tab_bar_width: None,
            tab_bar_position: None,
            tab_bar_max_height: None,
            tab_bar_height: None,
            text_size: None,
            text_font: None,
        }
    }

    fn push(&mut self, identifier: Identifier) {
        match identifier {
            Identifier::Counter => {
                let tab = CounterTab::new(self.id_counter);
                self.tabs.push(TabType::Counter(tab));
                self.active_tab = self.id_counter;
                self.id_counter += 1;
            }
        };
    }

    fn close_tab(&mut self, id: usize) {
        if let Some((idx, tab)) = self.tabs.iter().enumerate().find(|(_, tt)| tt.id() == id) {
            if tab.is_dirty() {
                return;
            }

            if id == self.active_tab {
                if idx == 0 {
                    if let Some(tt) = self.tabs.get(1) {
                        self.active_tab = tt.id();
                    }
                } else {
                    if let Some(tt) = self.tabs.get(idx - 1) {
                        self.active_tab = tt.id();
                    }
                }
            }

            self.tabs.retain(|tt| tt.id() != id)
        }
    }

    fn create_tab(&self) -> iced_aw::Tabs<'_, TabBarMessage, usize> {
        let tabs = self
            .tabs
            .iter()
            .map(|tab| (tab.id(), tab.tab_label(), tab.content()))
            .collect();

        let mut tabs = iced_aw::Tabs::with_tabs(tabs, TabBarMessage::TabSelected)
            .on_close(TabBarMessage::CloseTab)
            .set_active_tab(&self.active_tab);

        if let Some(height) = &self.height {
            tabs = tabs.height(height.clone())
        };

        if let Some(close_size) = self.close_size {
            tabs = tabs.close_size(close_size.clone())
        };

        if let Some(icon_font) = &self.icon_font {
            tabs = tabs.icon_font(icon_font.clone())
        };

        if let Some(icon_size) = &self.icon_size {
            tabs = tabs.icon_size(icon_size.clone())
        };

        if let Some(height) = &self.tab_bar_height {
            tabs = tabs.tab_bar_height(height.clone())
        };

        if let Some(max_height) = self.tab_bar_max_height {
            tabs = tabs.tab_bar_max_height(max_height.clone())
        };

        if let Some(position) = &self.tab_bar_position {
            tabs = tabs.tab_bar_position(position.clone())
        };

        if let Some(width) = &self.tab_bar_width {
            tabs = tabs.tab_bar_width(width.clone())
        };

        if let Some(padding) = self.tab_label_padding {
            tabs = tabs.tab_label_padding(padding)
        };

        if let Some(spacing) = self.tab_label_spacing {
            tabs = tabs.tab_label_spacing(spacing)
        };

        if let Some(font) = &self.text_font {
            tabs = tabs.text_font(font.clone())
        };

        if let Some(size) = self.text_size {
            tabs = tabs.text_size(size)
        };

        if let Some(width) = &self.width {
            tabs = tabs.width(width.clone())
        };

        tabs
    }

    pub fn content(&self) -> Element<'_, TabBarMessage, Renderer> {
        self.create_tab().into()
    }

    pub fn update(&mut self, tsg: TabBarMessage) -> Option<Message> {
        match tsg {
            TabBarMessage::TabSelected(id) => {
                self.active_tab = id;
                None
            }
            TabBarMessage::AddTab(id) => {
                self.push(id);
                None
            }
            TabBarMessage::CloseTab(id) => {
                self.close_tab(id);
                None
            }
            TabBarMessage::UpdateTab((id, tsg)) => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id() == id) {
                    tab.update(tsg);
                };
                None
            }
            TabBarMessage::OpenFile => Some(Message::OpenFile),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    pub fn close_size(mut self, close_size: f32) -> Self {
        self.close_size = close_size.into();
        self
    }

    pub fn height(mut self, height: Length) -> Self {
        self.height = height.into();
        self
    }

    pub fn icon_font(mut self, icon_font: Font) -> Self {
        self.icon_font = icon_font.into();
        self
    }

    pub fn icon_size(mut self, icon_size: f32) -> Self {
        self.icon_size = icon_size.into();
        self
    }

    pub fn tab_bar_height(mut self, height: Length) -> Self {
        self.tab_bar_height = height.into();
        self
    }

    pub fn tab_bar_max_height(mut self, max_height: f32) -> Self {
        self.tab_bar_max_height = max_height.into();
        self
    }

    pub fn tab_bar_position(mut self, position: TabBarPosition) -> Self {
        self.tab_bar_position = position.into();
        self
    }

    pub fn tab_bar_width(mut self, width: Length) -> Self {
        self.tab_bar_width = width.into();
        self
    }

    pub fn tab_label_padding(mut self, padding: f32) -> Self {
        self.tab_label_padding = padding.into();
        self
    }

    pub fn tab_label_spacing(mut self, spacing: f32) -> Self {
        self.tab_label_spacing = spacing.into();
        self
    }

    pub fn text_font(mut self, font: Font) -> Self {
        self.text_font = font.into();
        self
    }

    pub fn text_size(mut self, text_size: f32) -> Self {
        self.text_size = text_size.into();
        self
    }

    pub fn width(mut self, width: Length) -> Self {
        self.width = width.into();
        self
    }
}

pub fn home_view<'a>() -> Container<'a, Message, Renderer> {
    let new_btn: Button<'_, Message, Renderer> = button("New File")
        .on_press(Message::OpenTab(Identifier::Counter))
        .style(theme::Button::Text);
    let open_btn: Button<'_, Message, Renderer> = button("Open File")
        .on_press(Message::OpenFile)
        .style(theme::Button::Text);
    let recents_btn: Button<'_, Message, Renderer> = button("Recent Files")
        .on_press(Message::None)
        .style(theme::Button::Text);
    let options: Row<'_, Message, Renderer> = row!(
        horizontal_space(Length::Fill),
        column!(new_btn, open_btn, recents_btn).spacing(8),
        horizontal_space(Length::Fill)
    )
    .width(Length::Fill);

    let logo = row!(
        horizontal_space(Length::FillPortion(1)),
        text("modav logo").size(40),
        horizontal_space(Length::FillPortion(1)),
    )
    .width(Length::Fill);

    let content = column!(logo, options).spacing(24).width(Length::Fill);

    container(content)
        .width(Length::FillPortion(5))
        .height(Length::Fill)
        .center_y()
}

pub type Tabs = TabState;
