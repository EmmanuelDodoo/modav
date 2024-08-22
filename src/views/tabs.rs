// Credit: Both the TabBar and Tab were massively inspired by iced_aw's widget
// with similar name

use iced::{
    alignment, theme,
    widget::{button, column, container, row, text},
    Element, Length, Padding, Renderer, Theme,
};

use std::path::PathBuf;

use super::editor::{EditorMessage, EditorTab, EditorTabData};
use super::line::{LineGraphTab, LineTabData, ModelMessage};
use super::temp::{CounterMessage, CounterTab};
use super::{View, ViewType, Viewable};

use crate::widgets::style::DialogContainer;
use crate::Message;

use bar::TabBar;
#[allow(unused_imports)]
pub use bar::{Appearance, StyleSheet, TabBarStyle, TabLabel};

use crate::widgets::modal::Modal;
use crate::FileIOAction;

#[derive(Debug)]
pub enum Tab {
    Counter(CounterTab),
    Editor(EditorTab),
    Model(LineGraphTab),
}

impl Tab {
    fn update(&mut self, tsg: TabMessage) -> Option<Message> {
        match (self, tsg) {
            (Tab::Counter(tab), TabMessage::Counter(tsg)) => tab.update(tsg),
            (Tab::Counter(_), _) => None,
            (Tab::Editor(tab), TabMessage::Editor(tsg)) => tab.update(tsg),
            (Tab::Editor(_), _) => None,
            (Tab::Model(tab), TabMessage::Model(tsg)) => tab.update(tsg),
            (Tab::Model(_), _) => None,
        }
    }

    fn is_dirty(&self) -> bool {
        match self {
            Tab::Counter(tab) => tab.is_dirty(),
            Tab::Editor(tab) => tab.is_dirty(),
            Tab::Model(tab) => tab.is_dirty(),
        }
    }

    fn label(&self) -> TabLabel {
        match self {
            Tab::Counter(tab) => tab.label(),
            Tab::Editor(tab) => tab.label(),
            Tab::Model(tab) => tab.label(),
        }
    }

    fn content(&self) -> Option<String> {
        match self {
            Tab::Editor(tab) => tab.content(),
            Tab::Counter(tab) => tab.content(),
            Tab::Model(tab) => tab.content(),
        }
    }

    fn view(&self, idx: usize) -> Element<'_, TabBarMessage, Theme, Renderer> {
        match self {
            Tab::Counter(tab) => {
                tab.view(move |msg| TabBarMessage::UpdateTab(idx, TabMessage::Counter(msg)))
            }
            Tab::Editor(tab) => {
                tab.view(move |msg| TabBarMessage::UpdateTab(idx, TabMessage::Editor(msg)))
            }
            Tab::Model(tab) => {
                tab.view(move |msg| TabBarMessage::UpdateTab(idx, TabMessage::Model(msg)))
            }
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

#[derive(Debug, Clone, Copy)]
pub enum DirtyTabModalAction {
    Save,
    DontSave,
    Cancel,
}

#[derive(Debug, Clone)]
pub enum Refresh {
    Editor(EditorTabData),
    Model(LineTabData),
    Counter,
}

#[derive(Debug, Clone, Copy)]
pub enum NewTabModalAction {
    Cancel,
    Open,
    New,
}

#[derive(Debug, Clone)]
pub enum TabBarMessage {
    TabSelected(usize),
    AddTab(View),
    CloseTab(usize, bool),
    DirtyTabModal(DirtyTabModalAction),
    UpdateTab(usize, TabMessage),
    RefreshTab(usize, Refresh),
    NewTabModal,
    NewTabModalAction(NewTabModalAction),
    Exit,
    None,
}

pub struct TabsState<Theme>
where
    Theme: StyleSheet,
{
    width: Length,
    height: Length,
    tab_bar_height: Length,
    tab_height: Length,
    tab_padding: Padding,
    tab_bar_padding: Padding,
    tab_spacing: f32,
    labels: Vec<TabLabel>,
    tabs: Vec<Tab>,
    active_tab: Option<usize>,
    close_size: f32,
    modal_shown: bool,
    new_tab_modal_shown: bool,
    exiting: bool,
    on_open: Option<Message>,
    on_new_active_tab: Option<Message>,
    check_exit: Option<Message>,
    can_exit: Option<Message>,
    on_save: Option<Box<dyn Fn(Option<PathBuf>, String, FileIOAction) -> Message>>,
    style: <Theme as StyleSheet>::Style,
}

#[allow(dead_code)]
impl TabsState<Theme>
where
    Message: Clone + 'static,
{
    pub fn new() -> Self {
        Self::with_tabs(Vec::default().into_iter())
    }

    pub fn with_tabs(tabs: impl Iterator<Item = Tab>) -> Self {
        let mut len = 0;
        let mut labels = Vec::default();
        let mut tabs_list = Vec::default();

        for tab in tabs {
            let label = tab.label();

            labels.push(label);
            tabs_list.push(tab);
            len += 1;
        }

        Self {
            width: Length::Fill,
            height: Length::Shrink,
            tab_bar_height: Length::Shrink,
            tab_height: Length::Shrink,
            tab_padding: Padding::ZERO,
            tab_bar_padding: Padding::ZERO,
            tab_spacing: 0.0,
            active_tab: if len > 0 { Some(len - 1) } else { None },
            tabs: tabs_list,
            on_open: None,
            on_new_active_tab: None,
            on_save: None,
            check_exit: None,
            can_exit: None,
            close_size: 16.0,
            modal_shown: false,
            new_tab_modal_shown: false,
            exiting: false,
            style: <Theme as StyleSheet>::Style::default(),
            labels,
        }
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    pub fn tab_bar_height(mut self, height: impl Into<Length>) -> Self {
        self.tab_bar_height = height.into();
        self
    }

    pub fn tab_height(mut self, height: impl Into<Length>) -> Self {
        self.tab_height = height.into();
        self
    }

    pub fn tab_padding(mut self, padding: impl Into<Padding>) -> Self {
        self.tab_padding = padding.into();
        self
    }

    pub fn tab_bar_padding(mut self, padding: impl Into<Padding>) -> Self {
        self.tab_bar_padding = padding.into();
        self
    }

    pub fn tab_spacing(mut self, spacing: f32) -> Self {
        self.tab_spacing = spacing;
        self
    }

    pub fn close_size(mut self, size: f32) -> Self {
        self.close_size = size;
        self
    }

    pub fn style(mut self, style: impl Into<<Theme as StyleSheet>::Style>) -> Self {
        self.style = style.into();
        self
    }

    pub fn on_open(mut self, on_open: Message) -> Self {
        self.on_open = Some(on_open);
        self
    }

    pub fn on_new_active_tab(mut self, message: Message) -> Self {
        self.on_new_active_tab = Some(message);
        self
    }

    pub fn on_save<F>(mut self, on_save: F) -> Self
    where
        F: 'static + Fn(Option<PathBuf>, String, FileIOAction) -> Message,
    {
        self.on_save = Some(Box::new(on_save));
        self
    }

    pub fn on_check_exit(mut self, check_exit: Message) -> Self {
        self.check_exit = Some(check_exit);
        self
    }

    pub fn can_exit(mut self, can_exit: Message) -> Self {
        self.can_exit = Some(can_exit);
        self
    }

    fn push_tab(&mut self, tab: Tab) {
        let new_active = self.active_tab.map(|idx| idx + 1).unwrap_or(0);

        self.labels.insert(new_active, tab.label());
        self.tabs.insert(new_active, tab);

        self.active_tab = Some(new_active);
    }

    pub fn push_view(&mut self, view: View) {
        match view {
            View::Counter => {
                let counter = CounterTab::new(());
                let tab = Tab::Counter(counter);
                self.push_tab(tab);
            }
            View::Editor(data) => {
                let editor = EditorTab::new(data);
                let tab = Tab::Editor(editor);
                self.push_tab(tab);
            }
            View::LineGraph(data) => {
                let graph = LineGraphTab::new(data);
                let tab = Tab::Model(graph);
                self.push_tab(tab);
            }
            View::None => {}
        }
    }

    pub fn view<'a, F>(&'a self, map: F) -> Element<'a, Message, Theme, Renderer>
    where
        F: Fn(TabBarMessage) -> Message + 'a,
    {
        match self.active_tab {
            Some(idx) => {
                let mut bar = TabBar::new_with_tabs(
                    self.labels.clone(),
                    TabBarMessage::TabSelected,
                    &self.style,
                )
                .height(self.tab_bar_height)
                .width(self.width)
                .spacing(self.tab_spacing)
                .tab_padding(self.tab_padding)
                .bar_padding(self.tab_bar_padding)
                .tab_height(self.tab_height)
                .close_size(self.close_size)
                .on_close(|idx| TabBarMessage::CloseTab(idx, false));

                bar = bar.on_expand(|| TabBarMessage::NewTabModal);

                bar.set_active_tab(idx);

                let tab = self
                    .get_active_tab()
                    .expect("Tab: Has active tab index but no active tab")
                    .view(idx);

                let content: Element<'a, TabBarMessage, Theme, Renderer> = column!(bar, tab)
                    .width(self.width)
                    .height(self.height)
                    .into();

                if self.modal_shown {
                    let modal = self.modal_content();
                    let content: Element<'a, TabBarMessage, Theme, Renderer> =
                        Modal::new(content, modal)
                            .on_blur(TabBarMessage::DirtyTabModal(DirtyTabModalAction::Cancel))
                            .into();
                    content.map(map)
                } else if self.new_tab_modal_shown {
                    let modal = self.new_tab_modal();
                    let content: Element<'a, TabBarMessage, Theme, Renderer> =
                        Modal::new(content, modal)
                            .on_blur(TabBarMessage::NewTabModalAction(NewTabModalAction::Cancel))
                            .into();
                    content.map(map)
                } else {
                    content.map(map)
                }
            }
            None => self.empty_view().map(map),
        }
    }

    pub fn update(&mut self, message: TabBarMessage) -> Option<Message> {
        match message {
            TabBarMessage::None => None,
            TabBarMessage::AddTab(view) => {
                self.push_view(view);
                return self.on_new_active_tab.clone();
            }
            TabBarMessage::TabSelected(idx) => {
                self.tab_selected(idx);
                self.on_new_active_tab.clone()
            }
            TabBarMessage::CloseTab(idx, force) => {
                if self.close_tab(idx, force) {
                    return self.on_new_active_tab.clone();
                } else {
                    return None;
                };
            }
            TabBarMessage::NewTabModal => {
                self.new_tab_modal_shown = true;
                None
            }
            TabBarMessage::NewTabModalAction(action) => match action {
                NewTabModalAction::Cancel => {
                    self.new_tab_modal_shown = false;
                    return None;
                }
                NewTabModalAction::Open => {
                    self.new_tab_modal_shown = false;
                    return self.on_open.clone();
                }
                NewTabModalAction::New => {
                    self.new_tab_modal_shown = false;

                    let data = EditorTabData::default();
                    let tab = EditorTab::new(data);
                    let new = Tab::Editor(tab);
                    self.push_tab(new);

                    return self.on_new_active_tab.clone();
                }
            },

            TabBarMessage::DirtyTabModal(action) => match action {
                DirtyTabModalAction::Cancel => {
                    self.modal_shown = false;
                    self.exiting = false;
                    return None;
                }
                DirtyTabModalAction::Save => {
                    if let Some(tab) = self.get_active_tab() {
                        let idx = self.active_tab.expect(
                            "Tabs: Attempting to modal save a tab without an active tab index ",
                        );

                        let path = tab.path();
                        let contents = tab.content().unwrap_or(String::default());
                        let action = if self.exiting {
                            FileIOAction::Exiting(idx)
                        } else {
                            FileIOAction::CloseTab(idx)
                        };

                        self.modal_shown = false;

                        self.on_save
                            .as_ref()
                            .map(|on_save| (on_save)(path, contents, action))
                    } else {
                        if self.exiting {
                            self.check_exit.clone()
                        } else {
                            None
                        }
                    }
                }
                DirtyTabModalAction::DontSave => {
                    if let Some(idx) = self.active_tab {
                        self.close_tab(idx, true);
                    }
                    self.modal_shown = false;

                    if self.exiting {
                        self.check_exit.clone()
                    } else {
                        None
                    }
                }
            },

            TabBarMessage::UpdateTab(idx, tsg) => {
                self.tabs.get_mut(idx).and_then(|tab| tab.update(tsg))
            }

            TabBarMessage::RefreshTab(idx, rsg) => {
                if let Some(tab) = self.tabs.get_mut(idx) {
                    tab.refresh(rsg);
                    return self.on_new_active_tab.clone();
                } else {
                    None
                }
            }

            TabBarMessage::Exit => {
                self.exiting = true;
                if let Some(unclosed) = self.has_dirty_tab() {
                    self.active_tab = Some(unclosed);
                    self.modal_shown = true;
                    return None;
                }

                self.can_exit.clone()
            }
        }
    }

    fn tab_selected(&mut self, idx: usize) {
        self.active_tab = Some(idx);
    }

    fn empty_view<'a>(&self) -> Element<'a, TabBarMessage, Theme, Renderer> {
        container(button("Add Other tab").on_press(TabBarMessage::NewTabModal))
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .width(self.width)
            .height(self.height)
            .into()
    }

    fn modal_content<'a>(&self) -> Element<'a, TabBarMessage, Theme, Renderer> {
        let msg = self
            .get_active_tab()
            .expect("Tabs: Attempt to show modal with no active tab")
            .modal_msg();

        let msg = text(msg);

        let header = text("Close Tab?")
            .width(Length::Fill)
            .horizontal_alignment(alignment::Horizontal::Center);

        let actions = {
            let btn1 =
                button("Save").on_press(TabBarMessage::DirtyTabModal(DirtyTabModalAction::Save));
            let btn2 = button("Don't Save")
                .on_press(TabBarMessage::DirtyTabModal(DirtyTabModalAction::DontSave));
            let btn3 = button("Cancel")
                .on_press(TabBarMessage::DirtyTabModal(DirtyTabModalAction::Cancel));

            row!(btn1, btn2, btn3).spacing(16).width(Length::Fill)
        };

        let col = column!(header, msg, actions)
            .width(Length::Fill)
            .spacing(24);

        container(col)
            .style(theme::Container::Custom(Box::new(
                DialogContainer::default(),
            )))
            .height(175)
            .padding(16)
            .width(300)
            .into()
    }

    fn new_tab_modal<'a>(&self) -> Element<'a, TabBarMessage, Theme, Renderer> {
        let msg = "Open Existing File or New File?";

        let msg = text(msg);

        let header = text("Open Tab?")
            .width(Length::Fill)
            .horizontal_alignment(alignment::Horizontal::Center);

        let actions = {
            let btn1 = button("Open File")
                .on_press(TabBarMessage::NewTabModalAction(NewTabModalAction::Open));
            let btn2 = button("New File")
                .on_press(TabBarMessage::NewTabModalAction(NewTabModalAction::New));
            let btn3 = button("Cancel")
                .on_press(TabBarMessage::NewTabModalAction(NewTabModalAction::Cancel));

            row!(btn1, btn2, btn3).spacing(16).width(Length::Fill)
        };

        let col = column!(header, msg, actions)
            .width(Length::Fill)
            .spacing(24);

        container(col)
            .style(theme::Container::Custom(Box::new(
                DialogContainer::default(),
            )))
            .height(175)
            .padding(16)
            .width(300)
            .into()
    }

    pub fn active_content(&self) -> Option<String> {
        self.get_active_tab().map(|tab| tab.content())?
    }

    pub fn active_tab_type(&self) -> Option<ViewType> {
        self.get_active_tab().map(|tab| tab.kind())
    }

    pub fn active_path(&self) -> Option<PathBuf> {
        self.get_active_tab().map(|tab| tab.path())?
    }

    pub fn active_tab_can_save(&self) -> bool {
        self.get_active_tab()
            .map(|tab| tab.can_save())
            .unwrap_or(false)
    }

    pub fn active_tab_idx(&self) -> Option<usize> {
        self.active_tab
    }

    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    /// Returns the index of the first dirty tab if any
    pub fn has_dirty_tab(&self) -> Option<usize> {
        self.tabs
            .iter()
            .enumerate()
            .find(|(_, tab)| tab.is_dirty())
            .map(|(idx, _)| idx)
    }

    /// Returns true if a tab was successfully closed
    fn close_tab(&mut self, idx: usize, force: bool) -> bool {
        if force {
            self.force_close_tab(idx);
            return true;
        }

        if let Some(tab) = self.tabs.get(idx) {
            if tab.is_dirty() {
                self.active_tab = Some(idx);
                self.modal_shown = true;
                return false;
            }

            self.force_close_tab(idx);
            return true;
        }

        return false;
    }

    fn force_close_tab(&mut self, idx: usize) {
        if let Some(active_tab) = self.active_tab {
            // Deleteing active tab
            if active_tab == idx {
                if idx == 0 && self.labels.len() > 1 {
                    self.active_tab = Some(0);
                } else if idx == 0 {
                    self.active_tab = None;
                } else if idx == self.labels.len() - 1 {
                    self.active_tab = Some(idx - 1);
                } else {
                    self.active_tab = Some(idx);
                }
            }
            // Deleting other tab on left
            else if idx < active_tab {
                self.active_tab = Some(active_tab - 1);
            }
            // Deleting other tab on right
            else {
                self.active_tab = Some(active_tab)
            }

            self.labels.remove(idx);
            self.tabs.remove(idx);
        }
    }

    fn get_active_tab(&self) -> Option<&Tab> {
        self.active_tab.and_then(|idx| self.tabs.get(idx))
    }

    fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.active_tab.and_then(|idx| self.tabs.get_mut(idx))
    }
}

impl Default for TabsState<Theme>
where
    Message: Clone + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

pub mod bar {
    use iced::{
        advanced::{
            self,
            layout::{Limits, Node},
            mouse, renderer,
            text::{LineHeight, Shaping},
            widget::{tree, Tree},
            Widget,
        },
        alignment,
        event::{Event, Status},
        touch,
        widget::{horizontal_space, text, Row, Text},
        Alignment, Background, Border, Color, Element, Font, Length, Padding, Pixels, Point,
        Rectangle, Shadow, Size, Theme,
    };

    use crate::utils::icons;

    #[derive(Debug, PartialEq, Clone)]
    pub struct TabLabel {
        text: String,
        text_font: Option<Font>,
        text_size: f32,
        icon: char,
        icon_font: Option<Font>,
        icon_size: f32,
    }

    impl TabLabel {
        pub fn new(icon: char, text: impl Into<String>) -> Self {
            Self {
                text_font: None,
                icon_font: None,
                icon_size: 16.0,
                text: text.into(),
                text_size: 16.0,
                icon,
            }
        }

        pub fn empty() -> Self {
            Self::new(char::default(), String::default())
        }

        pub fn text_font(mut self, font: Font) -> Self {
            self.text_font = Some(font);
            self
        }

        pub fn icon_font(mut self, font: Font) -> Self {
            self.icon_font = Some(font);
            self
        }

        pub fn icon_size(mut self, size: f32) -> Self {
            self.icon_size = size;
            self
        }

        pub fn text_size(mut self, size: f32) -> Self {
            self.text_size = size;
            self
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Appearance {
        pub bar_background: Background,
        pub add_button_background_active: Background,
        pub add_button_background_hovered: Background,
        pub add_button_border: Border,
        pub add_button_text_color: Option<Color>,
        pub tab_background_active: Background,
        pub tab_background_hovered: Background,
        pub tab_border: Border,
        pub tab_shadow: Shadow,
        pub tab_text_color: Color,
        pub tab_icon_color: Option<Color>,
        pub close_background: Background,
        pub close_border: Border,
        pub close_text_color: Option<Color>,
    }

    pub trait StyleSheet {
        type Style: Default;

        fn appearance(&self, style: &Self::Style) -> Appearance;
    }

    #[derive(Default)]
    pub enum TabBarStyle {
        #[default]
        Default,
        Custom(Box<dyn StyleSheet<Style = Theme>>),
    }

    impl StyleSheet for iced::Theme {
        type Style = TabBarStyle;

        fn appearance(&self, style: &Self::Style) -> Appearance {
            match style {
                TabBarStyle::Default => {
                    let palette = self.extended_palette();

                    let base_background = palette.primary.weak;
                    let text_color = base_background.text;
                    let base_background = Background::Color(base_background.color);

                    Appearance {
                        bar_background: base_background,

                        add_button_background_active: base_background,
                        add_button_background_hovered: Background::Color(
                            palette.primary.base.color,
                        ),
                        add_button_border: Border::with_radius(5.0),
                        add_button_text_color: Some(palette.primary.base.text),

                        tab_background_active: base_background,
                        tab_background_hovered: Background::Color(palette.primary.strong.color),
                        tab_border: Border::with_radius(5.0),
                        tab_shadow: Shadow::default(),
                        tab_text_color: text_color,
                        tab_icon_color: Some(text_color),

                        close_background: Background::Color(palette.primary.base.color),
                        close_border: Border::with_radius(5.0),
                        close_text_color: Some(palette.primary.strong.text),
                    }
                }

                TabBarStyle::Custom(style) => style.appearance(self),
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct TabBarState {
        tab_width: f32,
        add_tabs_width: f32,
        min_tab_width: f32,
        max_tab_width: f32,
        tabs_spacing: f32,
    }

    impl TabBarState {
        fn new() -> Self {
            Self {
                tab_width: 250.0,
                min_tab_width: 50.0,
                max_tab_width: 250.0,
                add_tabs_width: 32.0,
                tabs_spacing: 2.0,
            }
        }
    }

    impl Default for TabBarState {
        fn default() -> Self {
            Self::new()
        }
    }

    pub struct TabBar<'a, Message, Theme>
    where
        Theme: StyleSheet,
    {
        active_tab: usize,
        labels: Vec<TabLabel>,
        on_close: Option<Box<dyn Fn(usize) -> Message>>,
        on_expand: Option<Box<dyn Fn() -> Message>>,
        on_select: Box<dyn Fn(usize) -> Message>,
        close_size: f32,
        close_width: f32,
        close_height: f32,
        spacing: f32,
        tab_padding: Padding,
        bar_padding: Padding,
        width: Length,
        height: Length,
        tab_height: Length,
        style: &'a <Theme as StyleSheet>::Style,
    }

    impl<'a, Message, Theme> TabBar<'a, Message, Theme>
    where
        Theme: StyleSheet,
    {
        fn calculate_close_(close_size: f32) -> (f32, f32) {
            (close_size * 1.5 + 1.0, close_size * 1.5 + 1.0)
        }

        pub fn new<F>(on_select: F, style: &'a <Theme as StyleSheet>::Style) -> Self
        where
            F: 'static + Fn(usize) -> Message,
        {
            Self::new_with_tabs(Vec::new(), on_select, style)
        }

        pub fn new_with_tabs<F>(
            tabs: impl Into<Vec<TabLabel>>,
            on_select: F,
            style: &'a <Theme as StyleSheet>::Style,
        ) -> Self
        where
            F: 'static + Fn(usize) -> Message,
        {
            let close_size = 0.0;
            let (close_width, close_height) = Self::calculate_close_(close_size);

            Self {
                active_tab: 0,
                on_close: None,
                on_expand: None,
                width: Length::Fill,
                height: Length::Shrink,
                on_select: Box::new(on_select),
                close_size,
                spacing: 0.0,
                tab_padding: Padding::ZERO,
                bar_padding: Padding::ZERO,
                labels: tabs.into(),
                tab_height: Length::Shrink,
                style,
                close_height,
                close_width,
            }
        }

        pub fn close_tab(&mut self, index: usize) {
            self.labels.remove(index);
        }

        /// Sets the tab at `index` to be the active tab. If tab is present at `index` nothing is changed.
        pub fn set_active_tab(&mut self, index: usize) {
            if index < self.labels.len() {
                self.active_tab = index;
            }
        }

        /// Pushes a tab into the tab bar. The tab becomes the new active tab
        pub fn push(&mut self, tab: TabLabel) {
            self.labels.push(tab);
            self.active_tab = self.labels.len() - 1;
        }

        pub fn push_all(&mut self, tabs: impl IntoIterator<Item = TabLabel>) {
            for tab in tabs.into_iter() {
                self.labels.push(tab);
            }

            self.active_tab = self.labels.len() - 1;
        }

        pub fn on_close<F>(mut self, on_close: F) -> Self
        where
            F: 'static + Fn(usize) -> Message,
        {
            self.on_close = Some(Box::new(on_close));
            self
        }

        pub fn on_expand<F>(mut self, on_expand: F) -> Self
        where
            F: 'static + Fn() -> Message,
        {
            self.on_expand = Some(Box::new(on_expand));
            self
        }

        pub fn spacing(mut self, spacing: f32) -> Self {
            self.spacing = spacing;
            self
        }

        pub fn tab_padding(mut self, padding: impl Into<Padding>) -> Self {
            self.tab_padding = padding.into();
            self
        }

        pub fn tab_height(mut self, height: impl Into<Length>) -> Self {
            self.tab_height = height.into();
            self
        }

        pub fn bar_padding(mut self, padding: impl Into<Padding>) -> Self {
            self.bar_padding = padding.into();
            self
        }

        pub fn height(mut self, height: impl Into<Length>) -> Self {
            self.height = height.into();
            self
        }

        pub fn width(mut self, width: impl Into<Length>) -> Self {
            self.width = width.into();
            self
        }

        pub fn close_size(mut self, close_size: f32) -> Self {
            self.close_size = close_size;
            let (close_width, close_height) = Self::calculate_close_(close_size);
            self.close_width = close_width;
            self.close_height = close_height;
            self
        }

        pub fn style(mut self, style: impl Into<&'a <Theme as StyleSheet>::Style>) -> Self {
            self.style = style.into();
            self
        }
    }

    impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for TabBar<'a, Message, Theme>
    where
        Theme: text::StyleSheet + StyleSheet,
        Renderer: renderer::Renderer + advanced::text::Renderer<Font = iced::Font>,
    {
        fn size(&self) -> Size<Length> {
            Size::new(self.width, self.height)
        }

        fn tag(&self) -> tree::Tag {
            tree::Tag::of::<TabBarState>()
        }

        fn state(&self) -> tree::State {
            tree::State::new(TabBarState::new())
        }

        fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
            let state = tree.state.downcast_mut::<TabBarState>();

            // Dynamic tab widths
            {
                let bar_width = limits
                    .clone()
                    .resolve(self.width, self.height, Size::UNIT)
                    .width;
                let tabs_space = bar_width - state.add_tabs_width - self.bar_padding.horizontal();

                let total_spacing = state.tabs_spacing * ((self.labels.len() - 1) as f32);

                let mut tabs_width = (tabs_space - total_spacing) / (self.labels.len() as f32);

                tabs_width = tabs_width.clamp(state.min_tab_width, state.max_tab_width);

                state.tab_width = tabs_width;
            }

            let icon_text_width =
                state.tab_width - self.tab_padding.horizontal() - self.close_width;

            let mut row = self
                .labels
                .iter()
                .fold(Row::<Message, Theme, Renderer>::new(), |row, label| {
                    let text = {
                        let font = label.text_font.unwrap_or(renderer.default_font());

                        Text::<Theme, Renderer>::new(label.text.clone())
                            .font(font)
                            .horizontal_alignment(alignment::Horizontal::Center)
                            .width(Length::Shrink)
                    };

                    let icon = {
                        let font = label.icon_font.unwrap_or(renderer.default_font());

                        Text::<Theme, Renderer>::new(label.icon.to_string())
                            .font(font)
                            .horizontal_alignment(alignment::Horizontal::Center)
                            .vertical_alignment(alignment::Vertical::Center)
                            .shaping(advanced::text::Shaping::Advanced)
                            .width(Length::Shrink)
                    };

                    let icon_text = Row::new()
                        .spacing(self.spacing)
                        .push(icon)
                        .push(text)
                        .width(icon_text_width);

                    let mut label_row = Row::new()
                        .push(icon_text)
                        .width(state.tab_width)
                        .height(self.tab_height)
                        .padding(self.tab_padding)
                        .align_items(Alignment::Center);

                    if self.on_close.is_some() {
                        label_row = label_row.push(horizontal_space()).push(
                            Row::new()
                                .width(Length::Fixed(self.close_width))
                                .height(Length::Fixed(self.close_height))
                                .align_items(Alignment::Center),
                        )
                    }

                    row.push(label_row)
                })
                .spacing(state.tabs_spacing)
                .padding(self.bar_padding)
                .align_items(Alignment::Center)
                .width(self.width)
                .height(self.height);

            if self.on_expand.is_some() {
                let add_tabs = Row::new()
                    .width(state.add_tabs_width)
                    .height(32.0)
                    .align_items(Alignment::Center);

                row = row.push(add_tabs);
            };

            let element = Element::new(row);

            let tab_tree = if let Some(child_tree) = tree.children.get_mut(0) {
                child_tree.diff(element.as_widget());
                child_tree
            } else {
                let child_tree = Tree::new(element.as_widget());
                tree.children.insert(0, child_tree);
                &mut tree.children[0]
            };

            element
                .as_widget()
                .layout(tab_tree, renderer, &limits.loose())
        }

        fn draw(
            &self,
            _tree: &Tree,
            renderer: &mut Renderer,
            theme: &Theme,
            _style: &renderer::Style,
            layout: advanced::Layout<'_>,
            cursor: advanced::mouse::Cursor,
            viewport: &iced::Rectangle,
        ) {
            let style = StyleSheet::appearance(theme, &self.style);
            let bounds = layout.bounds();
            let children = layout.children();

            if bounds.intersects(viewport) {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds,
                        border: Border::default(),
                        shadow: Shadow::default(),
                    },
                    style.bar_background,
                )
            }

            for ((idx, tab), layout) in self.labels.iter().enumerate().zip(children) {
                let is_selected = idx == self.active_tab;
                let icon_data = (
                    tab.icon_font.unwrap_or(renderer.default_font()),
                    tab.icon_size,
                );
                let text_data = (
                    tab.text_font.unwrap_or(renderer.default_font()),
                    tab.text_size,
                );

                let bounds = layout.bounds();
                let is_mouse_over = cursor
                    .position()
                    .map_or(false, |point| bounds.contains(point));

                let mut children = layout.children();
                let label_layout = children
                    .next()
                    .expect("TabBar: Layout should have a label layout");

                let mut label_layout_children = label_layout.children();

                if bounds.intersects(viewport) {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds,
                            border: style.tab_border,
                            shadow: style.tab_shadow,
                        },
                        if is_selected || is_mouse_over {
                            style.tab_background_hovered
                        } else {
                            style.tab_background_active
                        },
                    )
                };

                let icon_bounds = label_layout_children
                    .next()
                    .expect("TabBar: Layout should have an icon layout")
                    .bounds();
                let text_bounds = label_layout_children
                    .next()
                    .expect("TabBar: Layout should have an text layout")
                    .bounds();

                renderer.fill_text(
                    advanced::Text {
                        content: &tab.icon.to_string(),
                        bounds: Size::new(icon_bounds.width, icon_bounds.height),
                        size: Pixels(icon_data.1),
                        line_height: LineHeight::default(),
                        font: icon_data.0,
                        horizontal_alignment: alignment::Horizontal::Center,
                        vertical_alignment: alignment::Vertical::Center,
                        shaping: Shaping::Advanced,
                    },
                    Point::new(icon_bounds.center_x(), icon_bounds.center_y()),
                    style.tab_icon_color.unwrap_or(style.tab_text_color),
                    icon_bounds,
                );

                renderer.fill_text(
                    advanced::Text {
                        content: &tab.text.to_string(),
                        bounds: Size::new(text_bounds.width, text_bounds.height),
                        size: Pixels(text_data.1),
                        line_height: LineHeight::default(),
                        font: text_data.0,
                        horizontal_alignment: alignment::Horizontal::Center,
                        vertical_alignment: alignment::Vertical::Center,
                        shaping: Shaping::Advanced,
                    },
                    Point::new(text_bounds.center_x(), text_bounds.center_y()),
                    style.tab_text_color,
                    text_bounds,
                );

                if is_selected || is_mouse_over {
                    let _ = children.next();
                    if let Some(close_layout) = children.next() {
                        let close_bounds = close_layout.bounds();
                        let is_mouse_over = cursor.is_over(close_bounds);

                        let font = Font::with_name(icons::toast::NAME);

                        renderer.fill_text(
                            advanced::Text {
                                content: &icons::toast::CLOSE.to_string(),
                                bounds: Size::new(close_bounds.width, close_bounds.height),
                                size: Pixels(
                                    self.close_size * if is_mouse_over { 1.05 } else { 1.0 },
                                ),
                                line_height: LineHeight::default(),
                                font,
                                horizontal_alignment: alignment::Horizontal::Center,
                                vertical_alignment: alignment::Vertical::Center,
                                shaping: Shaping::Advanced,
                            },
                            Point::new(close_bounds.center_x(), close_bounds.center_y()),
                            style.close_text_color.unwrap_or(style.tab_text_color),
                            close_bounds,
                        );

                        if is_mouse_over && close_bounds.intersects(viewport) {
                            renderer.fill_quad(
                                renderer::Quad {
                                    bounds: close_bounds,
                                    border: style.close_border,
                                    shadow: Shadow::default(),
                                },
                                style.close_background,
                            )
                        }
                    }
                }
            }

            if self.on_expand.is_some() {
                let add_tabs_layout = layout
                    .children()
                    .last()
                    .expect("TabBar: Layout should have an add tabs layout");
                let add_tabs_bounds = add_tabs_layout.bounds();

                let is_mouse_over_add_tabs = cursor
                    .position()
                    .map_or(false, |point| add_tabs_bounds.contains(point));

                if add_tabs_bounds.intersects(viewport) {
                    let reduction = 0.80;
                    let offset = bounds.height * (1.0 - reduction) * 0.5;

                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: Rectangle::new(
                                Point::new(add_tabs_bounds.x, bounds.y + offset),
                                Size::new(add_tabs_bounds.width, bounds.height * reduction),
                            ),
                            border: style.add_button_border,
                            shadow: Shadow::default(),
                        },
                        if is_mouse_over_add_tabs {
                            style.add_button_background_hovered
                        } else {
                            style.add_button_background_active
                        },
                    );

                    renderer.fill_text(
                        advanced::Text {
                            content: "+",
                            bounds: Size::new(add_tabs_bounds.width, add_tabs_bounds.height),
                            size: Pixels(16.0 * if is_mouse_over_add_tabs { 1.05 } else { 1.0 }),
                            line_height: LineHeight::default(),
                            font: renderer.default_font(),
                            horizontal_alignment: alignment::Horizontal::Center,
                            vertical_alignment: alignment::Vertical::Center,
                            shaping: Shaping::Basic,
                        },
                        Point::new(add_tabs_bounds.center_x(), add_tabs_bounds.center_y()),
                        style.add_button_text_color.unwrap_or(style.tab_text_color),
                        add_tabs_bounds,
                    )
                }
            }
        }

        fn mouse_interaction(
            &self,
            _state: &Tree,
            layout: advanced::Layout<'_>,
            cursor: advanced::mouse::Cursor,
            _viewport: &Rectangle,
            _renderer: &Renderer,
        ) -> mouse::Interaction {
            let children = layout.children();
            let mut mouse_interaction = mouse::Interaction::default();

            for layout in children {
                let is_mouse_over = cursor
                    .position()
                    .map_or(false, |point| layout.bounds().contains(point));

                let new_interaction = if is_mouse_over {
                    mouse::Interaction::Pointer
                } else {
                    mouse::Interaction::default()
                };

                if new_interaction > mouse_interaction {
                    mouse_interaction = new_interaction;
                }
            }

            mouse_interaction
        }

        fn on_event(
            &mut self,
            _state: &mut Tree,
            event: Event,
            layout: advanced::Layout<'_>,
            cursor: mouse::Cursor,
            _renderer: &Renderer,
            _clipboard: &mut dyn advanced::Clipboard,
            shell: &mut advanced::Shell<'_, Message>,
            _viewport: &Rectangle,
        ) -> Status {
            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. }) => {
                    if cursor
                        .position()
                        .map_or(false, |point| layout.bounds().contains(point))
                    {
                        let tabs_map: Vec<bool> = layout
                            .children()
                            .map(|layout| {
                                cursor
                                    .position()
                                    .map_or(false, |point| layout.bounds().contains(point))
                            })
                            .collect();

                        if let Some(selected) = tabs_map.iter().position(|b| *b) {
                            if self.on_expand.is_some() && selected == tabs_map.len() - 1 {
                                let on_expand = self.on_expand.as_ref().unwrap();

                                shell.publish((on_expand)())
                            } else {
                                let message = self.on_close.as_ref().filter(|_on_close| {
                                    let tab_layout = layout.children().nth(selected).expect("TabBar: Layout should have a tab layout at selected index");
                                    let cross_layout = tab_layout.children().nth(2).expect("TabBar: Layout should have a close layout");

                                    cursor.position().map_or(false, |point| cross_layout.bounds().contains(point))
                                }).map_or_else(|| (self.on_select) (selected), |on_close| (on_close)(selected));

                                shell.publish(message);
                            }
                        }
                    }
                    Status::Ignored
                }
                _ => Status::Ignored,
            }
        }
    }

    impl<'a, Message, Theme, Renderer> From<TabBar<'a, Message, Theme>>
        for Element<'a, Message, Theme, Renderer>
    where
        Message: 'a,
        Theme: text::StyleSheet + StyleSheet + 'a,
        Renderer: renderer::Renderer + advanced::text::Renderer<Font = iced::Font>,
    {
        fn from(value: TabBar<'a, Message, Theme>) -> Self {
            Element::new(value)
        }
    }
}
