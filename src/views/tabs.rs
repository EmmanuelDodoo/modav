//
use iced::{
    alignment::Horizontal,
    widget::{button, column, container, row, text},
    Element, Font, Length, Renderer, Theme,
};

use iced_aw::{TabBarPosition, TabBarStyles, TabLabel};

use std::{path::PathBuf, rc::Rc};

use super::editor::{EditorMessage, EditorTab, EditorTabData};
use super::line::{LineGraphTab, LineTabData, ModelMessage};
use super::temp::{CounterMessage, CounterTab};
use super::{Message, View, ViewType, Viewable};

use styles::CustomTabBarStyle;

use crate::widgets::modal::Modal;
use crate::FileIOAction;

#[derive(Debug)]
enum Tab {
    Counter(CounterTab),
    Editor(EditorTab),
    Model(LineGraphTab),
}

impl Tab {
    fn id(&self) -> usize {
        match self {
            Tab::Counter(tab) => tab.id(),
            Tab::Editor(tab) => tab.id(),
            Tab::Model(tab) => tab.id(),
        }
    }

    fn update(&mut self, tsg: TabMessage) {
        match (self, tsg) {
            (Tab::Counter(tab), TabMessage::Counter(tsg)) => tab.update(tsg),
            (Tab::Counter(_), _) => {}
            (Tab::Editor(tab), TabMessage::Editor(tsg)) => tab.update(tsg),
            (Tab::Editor(_), _) => {}
            (Tab::Model(tab), TabMessage::Model(tsg)) => tab.update(tsg),
            (Tab::Model(_), _) => {}
        }
    }

    fn is_dirty(&self) -> bool {
        match self {
            Tab::Counter(tab) => tab.is_dirty(),
            Tab::Editor(tab) => tab.is_dirty(),
            Tab::Model(tab) => tab.is_dirty(),
        }
    }

    fn tab_label(&self) -> TabLabel {
        match self {
            Tab::Counter(tab) => tab.tab_label(),
            Tab::Editor(tab) => tab.tab_label(),
            Tab::Model(tab) => tab.tab_label(),
        }
    }

    fn content(&self) -> Option<String> {
        match self {
            Tab::Editor(tab) => tab.content(),
            Tab::Counter(tab) => tab.content(),
            Tab::Model(tab) => tab.content(),
        }
    }

    fn view(&self) -> Element<'_, TabBarMessage> {
        match self {
            Tab::Counter(tab) => tab.view(),
            Tab::Editor(tab) => tab.view(),
            Tab::Model(tab) => tab.view(),
        }
    }

    /// Refresh self with a Refresh
    /// Assumes the self.id matches Refresh's id
    fn refresh(&mut self, rsh: Refresh) {
        match (self, rsh) {
            (Tab::Counter(tab), Refresh::Counter) => tab.refresh(()),
            (Tab::Counter(_), _) => {}
            (Tab::Editor(tab), Refresh::Editor(data)) => tab.refresh(data),
            (Tab::Editor(_), _) => {}
            (Tab::Model(tab), Refresh::Model(data)) => tab.refresh(data),
            (Tab::Model(_), _) => {}
        }
    }

    /// Returns true if the identifier matches for self
    fn compare_idr(&self, idr: &View) -> bool {
        match (self, idr) {
            (Tab::Counter(_), View::Counter) => true,
            (Tab::Counter(_), _) => false,
            (Tab::Editor(_), View::Editor(_)) => true,
            (Tab::Editor(_), _) => false,
            (Tab::Model(_), View::LineGraph(_)) => true,
            (Tab::Model(_), _) => false,
        }
    }

    ///Returns the corresponding TabIden of self
    fn kind(&self) -> ViewType {
        match self {
            Tab::Editor(_) => ViewType::Editor,
            Tab::Counter(_) => ViewType::Counter,
            Tab::Model(_) => ViewType::LineGraph,
        }
    }

    fn modal_msg(&self) -> String {
        match self {
            Tab::Counter(tab) => tab.modal_msg(),
            Tab::Editor(tab) => tab.modal_msg(),
            Tab::Model(tab) => tab.modal_msg(),
        }
    }

    fn path(&self) -> Option<PathBuf> {
        match self {
            Tab::Counter(tab) => tab.path(),
            Tab::Editor(tab) => tab.path(),
            Tab::Model(tab) => tab.path(),
        }
    }

    fn can_save(&self) -> bool {
        match self {
            Tab::Editor(tab) => tab.can_save(),
            Tab::Counter(tab) => tab.can_save(),
            Tab::Model(tab) => tab.can_save(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TabMessage {
    Counter(CounterMessage),
    Editor(EditorMessage),
    Model(ModelMessage),
}

#[derive(Debug, Clone)]
pub enum DirtyTabAction {
    Save,
    DontSave,
}

#[derive(Debug, Clone)]
pub enum Refresh {
    Editor(EditorTabData),
    Model(LineTabData),
    Counter,
}

#[derive(Debug, Clone)]
pub enum TabBarMessage {
    TabSelected(usize),
    AddTab(View),
    CloseTab((usize, bool)),
    CloseModal,
    ModalMessage(DirtyTabAction),
    UpdateTab((usize, TabMessage)),
    OpenFile,
    RefreshTab((usize, Refresh)),
    Exit,
    None,
}

pub struct TabBarState {
    tabs: Vec<Tab>,
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
impl TabBarState {
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

    fn push(&mut self, identifier: View) {
        match identifier {
            View::Counter => {
                let tab = CounterTab::new(self.id_counter, ());
                self.tabs.push(Tab::Counter(tab));
                self.active_tab = self.id_counter;
                self.id_counter += 1;
            }
            View::Editor(data) => {
                let tab = EditorTab::new(self.id_counter, data);
                self.tabs.push(Tab::Editor(tab));
                self.active_tab = self.id_counter;
                self.id_counter += 1;
            }
            View::LineGraph(data) => {
                let tab = LineGraphTab::new(self.id_counter, data);
                self.tabs.push(Tab::Model(tab));
                self.active_tab = self.id_counter;
                self.id_counter += 1;
            }
            View::None => {}
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
                Some(Message::NewActiveTab)
            }
            TabBarMessage::AddTab(id) => {
                self.push(id);
                Some(Message::NewActiveTab)
            }
            TabBarMessage::CloseTab((id, force)) => {
                self.close_tab(id, force);
                Some(Message::NewActiveTab)
            }
            TabBarMessage::UpdateTab((id, tsg)) => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id() == id) {
                    tab.update(tsg);
                };
                None
            }
            TabBarMessage::RefreshTab((id, rsh)) => {
                if let Some(tab) = self.tabs.iter_mut().find(|tab| {
                    // Only Editor Tabs can be refreshed atm
                    tab.id() == id && tab.compare_idr(&View::Editor(EditorTabData::default()))
                }) {
                    tab.refresh(rsh);
                    Some(Message::NewActiveTab)
                } else {
                    Some(Message::NewActiveTab)
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

    pub fn active_tab_type(&self) -> Option<ViewType> {
        self.tabs
            .iter()
            .find(|tt| tt.id() == self.active_tab)
            .map(|tt| tt.kind())
    }

    /// Returns the tab of the currently active tab
    pub fn active_path(&self) -> Option<PathBuf> {
        let tab = self.tabs.iter().find(|tab| tab.id() == self.active_tab)?;
        tab.path()
    }

    pub fn active_tab_can_save(&self) -> bool {
        self.tabs
            .iter()
            .find(|tab| tab.id() == self.active_tab)
            .map(|tab| tab.can_save())
            .unwrap_or(false)
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
