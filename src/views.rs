use std::{path::PathBuf, rc::Rc};

use iced::{
    alignment::Horizontal,
    theme::{self, Theme},
    widget::{button, column, container, horizontal_space, row, text, Button, Container, Row},
    Element, Font, Length, Renderer,
};
use iced_aw::{TabBarPosition, TabBarStyles, TabLabel};

mod editor;
pub use editor::EditorTabData;
mod temp;
use temp::{CounterMessage, CounterTab};

use crate::modal::Modal;
use crate::FileIOAction;

use self::{
    editor::{EditorMessage, EditorTab},
    styles::CustomTabBarStyle,
};

use super::{Message, TabIden};

#[derive(Debug, Clone)]
pub enum TabMessage {
    Counter(CounterMessage),
    Editor(EditorMessage),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Identifier {
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

#[derive(Debug)]
enum TabType {
    Counter(CounterTab),
    Editor(EditorTab),
}

impl TabType {
    fn id(&self) -> usize {
        match self {
            TabType::Counter(tab) => tab.id(),
            TabType::Editor(tab) => tab.id(),
        }
    }

    fn update(&mut self, tsg: TabMessage) {
        match (self, tsg) {
            (TabType::Counter(tab), TabMessage::Counter(tsg)) => tab.update(tsg),
            (TabType::Counter(_), _) => {}
            (TabType::Editor(tab), TabMessage::Editor(tsg)) => tab.update(tsg),
            (TabType::Editor(_), _) => {}
        }
    }

    fn is_dirty(&self) -> bool {
        match self {
            TabType::Counter(tab) => tab.is_dirty(),
            TabType::Editor(tab) => tab.is_dirty(),
        }
    }

    fn tab_label(&self) -> TabLabel {
        match self {
            TabType::Counter(tab) => tab.tab_label(),
            TabType::Editor(tab) => tab.tab_label(),
        }
    }

    fn content(&self) -> Option<String> {
        match self {
            TabType::Editor(tab) => tab.content(),
            TabType::Counter(tab) => tab.content(),
        }
    }

    fn view(&self) -> Element<'_, TabBarMessage> {
        match self {
            TabType::Counter(tab) => tab.view(),
            TabType::Editor(tab) => tab.view(),
        }
    }

    /// Refresh self with a Refresh
    /// Assumes the self.id matches Refresh's id
    fn refresh(&mut self, rsh: Refresh) {
        match (self, rsh) {
            (TabType::Counter(tab), Refresh::Counter) => tab.refresh(()),
            (TabType::Counter(_), _) => {}
            (TabType::Editor(tab), Refresh::Editor(data)) => tab.refresh(data),
            (TabType::Editor(_), _) => {}
        }
    }

    /// Returns true if the identifier matches for self
    fn compare_idr(&self, idr: Identifier) -> bool {
        match (self, idr) {
            (TabType::Counter(_), Identifier::Counter) => true,
            (TabType::Counter(_), _) => false,
            (TabType::Editor(_), Identifier::Editor(_)) => true,
            (TabType::Editor(_), _) => true,
        }
    }

    ///Returns the corresponding TabIden of self
    fn kind(&self) -> TabIden {
        match self {
            TabType::Editor(_) => TabIden::Editor,
            TabType::Counter(_) => TabIden::Counter,
        }
    }

    fn modal_msg(&self) -> String {
        match self {
            TabType::Counter(tab) => tab.modal_msg(),
            TabType::Editor(tab) => tab.modal_msg(),
        }
    }

    fn path(&self) -> Option<PathBuf> {
        match self {
            TabType::Counter(tab) => tab.path(),
            TabType::Editor(tab) => tab.path(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Refresh {
    Editor(EditorTabData),
    Counter,
}

#[derive(Debug, Clone)]
pub enum DirtyTabAction {
    Save,
    DontSave,
}

#[derive(Debug, Clone)]
pub enum TabBarMessage {
    TabSelected(usize),
    AddTab(Identifier),
    CloseTab((usize, bool)),
    CloseModal,
    ModalMessage(DirtyTabAction),
    UpdateTab((usize, TabMessage)),
    OpenFile,
    RefreshTab((usize, Refresh)),
    Exit,
    None,
}

pub struct TabState {
    tabs: Vec<TabType>,
    id_counter: usize,
    active_tab: usize,
    modal_shown: bool,
    /// Whether the main app is closing
    exiting: bool,
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
            modal_shown: false,
            id_counter: 0,
            exiting: false,
            active_tab: 0,
            close_size: Some(30.0),
            height: None,
            width: None,
            icon_size: None,
            icon_font: None,
            tab_label_spacing: None,
            tab_label_padding: None,
            tab_bar_width: None,
            tab_bar_position: None,
            tab_bar_max_height: None,
            tab_bar_height: Some(Length::Shrink),
            text_size: None,
            text_font: None,
        }
    }

    fn push(&mut self, identifier: Identifier) {
        match identifier {
            Identifier::Counter => {
                let tab = CounterTab::new(self.id_counter, ());
                self.tabs.push(TabType::Counter(tab));
                self.active_tab = self.id_counter;
                self.id_counter += 1;
            }
            Identifier::Editor(data) => {
                let tab = EditorTab::new(self.id_counter, data);
                self.tabs.push(TabType::Editor(tab));
                self.active_tab = self.id_counter;
                self.id_counter += 1;
            }
            Identifier::None => {}
        };
    }

    fn force_close_tab(&mut self, id: usize) {
        if let Some((idx, _)) = self.tabs.iter().enumerate().find(|(_, tab)| tab.id() == id) {
            if id == self.active_tab {
                if idx == 0 {
                    if let Some(tt) = self.tabs.get(1) {
                        self.active_tab = tt.id();
                    }
                } else if let Some(tt) = self.tabs.get(idx - 1) {
                    self.active_tab = tt.id();
                }
            }

            self.tabs.retain(|tt| tt.id() != id);
        }
    }

    fn close_tab(&mut self, id: usize, force: bool) {
        if force {
            self.force_close_tab(id);
            return;
        }
        if let Some(tab) = self.tabs.iter().find(|tt| tt.id() == id) {
            if tab.is_dirty() {
                self.active_tab = tab.id();
                self.modal_shown = true;
                return;
            }

            self.force_close_tab(id)
        }
    }

    fn create_tab(&self) -> iced_aw::Tabs<'_, TabBarMessage, usize> {
        let tabs = self
            .tabs
            .iter()
            .map(|tab| (tab.id(), tab.tab_label(), tab.view()))
            .collect();

        let mut tabs = iced_aw::Tabs::new_with_tabs(tabs, TabBarMessage::TabSelected)
            .on_close(|id| TabBarMessage::CloseTab((id, false)))
            .tab_bar_style(TabBarStyles::Custom(Rc::new(CustomTabBarStyle {})))
            .set_active_tab(&self.active_tab);

        if let Some(height) = self.height {
            tabs = tabs.height(height)
        };

        if let Some(close_size) = self.close_size {
            tabs = tabs.close_size(close_size)
        };

        if let Some(icon_font) = self.icon_font {
            tabs = tabs.icon_font(icon_font)
        };

        if let Some(icon_size) = self.icon_size {
            tabs = tabs.icon_size(icon_size)
        };

        if let Some(height) = self.tab_bar_height {
            tabs = tabs.tab_bar_height(height)
        };

        if let Some(max_height) = self.tab_bar_max_height {
            tabs = tabs.tab_bar_max_height(max_height)
        };

        if let Some(position) = &self.tab_bar_position {
            tabs = tabs.tab_bar_position(position.clone())
        };

        if let Some(width) = self.tab_bar_width {
            tabs = tabs.tab_bar_width(width)
        };

        if let Some(padding) = self.tab_label_padding {
            tabs = tabs.tab_label_padding(padding)
        };

        if let Some(spacing) = self.tab_label_spacing {
            tabs = tabs.tab_label_spacing(spacing)
        };

        if let Some(font) = self.text_font {
            tabs = tabs.text_font(font)
        };

        if let Some(size) = self.text_size {
            tabs = tabs.text_size(size)
        };

        if let Some(width) = self.width {
            tabs = tabs.width(width)
        };

        tabs
    }

    pub fn content(&self) -> Element<'_, TabBarMessage, Theme, Renderer> {
        let tabs = self.create_tab();

        let modal = self.modal_content();

        if self.modal_shown {
            Modal::new(tabs, modal)
                .on_blur(TabBarMessage::CloseModal)
                .into()
        } else {
            tabs.into()
        }
    }

    fn modal_content(&self) -> Element<'_, TabBarMessage, Theme, Renderer> {
        let msg = self
            .tabs
            .iter()
            .find(|tab| tab.id() == self.active_tab)
            .map(|tab| tab.modal_msg())
            .unwrap_or(String::default());

        let msg = text(msg);

        let header = text("Close Tab?")
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center);

        let actions = {
            let btn1 = button("Save").on_press(TabBarMessage::ModalMessage(DirtyTabAction::Save));
            let btn2 = button("Don't Save")
                .on_press(TabBarMessage::ModalMessage(DirtyTabAction::DontSave));
            let btn3 = button("Cancel").on_press(TabBarMessage::CloseModal);

            row!(btn1, btn2, btn3).spacing(16).width(Length::Fill)
        };

        let col = column!(header, msg, actions)
            .width(Length::Fill)
            .spacing(24);

        container(col).padding(16).width(300).into()
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
            TabBarMessage::CloseTab((id, force)) => {
                self.close_tab(id, force);
                None
            }
            TabBarMessage::UpdateTab((id, tsg)) => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id() == id) {
                    tab.update(tsg);
                };
                None
            }
            TabBarMessage::RefreshTab((id, rsh)) => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| {
                    tab.id() == id && tab.compare_idr(Identifier::Editor(EditorTabData::default()))
                }) {
                    tab.refresh(rsh);
                    None
                } else {
                    None
                }
            }
            TabBarMessage::Exit => {
                self.exiting = true;
                if let Some(unclosed) = self.has_dirty_tabs() {
                    self.active_tab = unclosed;
                    self.modal_shown = true;
                    return None;
                }
                Some(Message::CanExit)
            }
            TabBarMessage::CloseModal => {
                self.modal_shown = false;
                self.exiting = false;
                None
            }
            TabBarMessage::ModalMessage(action) => match action {
                DirtyTabAction::Save => {
                    if let Some(tab) = self.tabs.iter().find(|tab| tab.id() == self.active_tab) {
                        let path = tab.path();
                        let contents = tab.content().unwrap_or(String::default());
                        let action = if self.exiting {
                            FileIOAction::Exiting(tab.id())
                        } else {
                            FileIOAction::CloseTab(self.active_tab)
                        };

                        self.modal_shown = false;

                        Some(Message::SaveFile((path, contents, action)))
                    } else {
                        if self.exiting {
                            Some(Message::CheckExit)
                        } else {
                            None
                        }
                    }
                }
                DirtyTabAction::DontSave => {
                    self.close_tab(self.active_tab, true);
                    self.modal_shown = false;
                    if self.exiting {
                        Some(Message::CheckExit)
                    } else {
                        None
                    }
                }
            },
            TabBarMessage::OpenFile => Some(Message::SelectFile),
            TabBarMessage::None => None,
        }
    }

    /// Returns the content of the active tab as a String
    pub fn active_content(&self) -> Option<String> {
        self.tabs
            .iter()
            .find(|tt| tt.id() == self.active_tab)
            .map(|tt| tt.content())?
    }

    pub fn active_tab(&self) -> usize {
        self.active_tab
    }

    pub fn active_tab_type(&self) -> Option<TabIden> {
        self.tabs
            .iter()
            .find(|tt| tt.id() == self.active_tab)
            .map(|tt| tt.kind())
    }

    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    /// Checks if any open tabs are dirty. Returns the id of the first dirty tab
    pub fn has_dirty_tabs(&self) -> Option<usize> {
        self.tabs
            .iter()
            .find(|tab| tab.is_dirty())
            .and_then(|tab| Some(tab.id()))
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

pub fn home_view<'a>() -> Container<'a, Message, Theme, Renderer> {
    let new_btn: Button<'_, Message, Theme, Renderer> = button("New File")
        .on_press(Message::OpenTab(TabIden::Editor))
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

mod styles {
    use iced::{color, Background, Theme};
    use iced_aw::tab_bar;

    #[derive(Debug, Clone)]
    pub struct CustomTabBarStyle;
    impl tab_bar::StyleSheet for CustomTabBarStyle {
        type Style = Theme;

        fn active(&self, _style: &Self::Style, is_active: bool) -> tab_bar::Appearance {
            let bg_color = if is_active {
                Background::Color(color!(0, 150, 50))
            } else {
                Background::Color(color!(0, 200, 150))
            };

            tab_bar::Appearance {
                tab_label_background: bg_color,
                ..Default::default()
            }
        }

        fn hovered(&self, _style: &Self::Style, is_active: bool) -> tab_bar::Appearance {
            let bg_color = if is_active {
                Background::Color(color!(100, 50, 225))
            } else {
                Background::Color(color!(155, 25, 225))
            };

            tab_bar::Appearance {
                tab_label_background: bg_color,
                ..Default::default()
            }
        }
    }
}

pub type Tabs = TabState;
