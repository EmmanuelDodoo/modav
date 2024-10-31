use std::{
    fmt::{self, Debug},
    path::PathBuf,
};

use iced::{
    theme::{self, Theme},
    widget::{button, column, container, horizontal_space, row, text, Button, Container, Row},
    Element, Length, Renderer,
};

use crate::Message;

pub mod tabs;
pub use tabs::Refresh;
use tabs::{TabBarMessage, TabLabel, TabsState};

mod editor;
pub use editor::EditorTabData;

mod line;
pub use line::LineTabData;

mod barchart;
pub use barchart::BarChartTabData;

mod shared;

mod stacked_barchart;
pub use stacked_barchart::StackedBarChartTabData;

use crate::utils::icons;

#[derive(Debug, Clone, PartialEq, Default, Copy)]
pub enum FileType {
    CSV,
    JSON,
    TXT,
    #[default]
    Other,
}

#[allow(dead_code)]
impl FileType {
    fn create<'a>(ext: &'a str) -> Self {
        match ext {
            "csv" => Self::CSV,
            "json" => Self::JSON,
            "txt" => Self::TXT,
            _ => Self::Other,
        }
    }

    pub fn new(file: &PathBuf) -> Self {
        file.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| Some(Self::create(ext)))
            .unwrap_or(Self::default())
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum View {
    Editor(EditorTabData),
    LineGraph(LineTabData),
    BarChart(BarChartTabData),
    StackedBarChart(StackedBarChartTabData),
    #[default]
    None,
}

impl View {
    /// Returns true if this tab requires the contents of a file to be loaded
    pub fn should_load(&self) -> bool {
        match self {
            Self::Editor(_) => true,
            Self::LineGraph(_) => false,
            Self::BarChart(_) => false,
            Self::StackedBarChart(_) => false,
            Self::None => false,
        }
    }
}

pub trait Viewable {
    type Event: Clone + Debug;
    type Data: Clone + Debug;

    fn new(data: Self::Data) -> Self;

    fn is_dirty(&self) -> bool;

    fn update(&mut self, message: Self::Event) -> Option<Message>;

    fn label(&self) -> TabLabel;

    /// Returns the content of this view if any
    fn content(&self) -> Option<String> {
        None
    }

    fn refresh(&mut self, data: Self::Data);

    fn modal_msg(&self) -> String;

    fn title(&self) -> String {
        String::default()
    }

    fn view<'a, Message, F>(&'a self, map: F) -> Element<'a, Message, Theme, Renderer>
    where
        F: 'a + Fn(Self::Event) -> Message,
        Message: 'a + Clone + Debug;

    fn path(&self) -> Option<PathBuf> {
        None
    }

    /// Returns true if self can be saved. Unlike [`is_dirty`], this represents
    /// the logical reasoning of whether a view can be saved.
    fn can_save(&self) -> bool {
        false
    }
}

#[derive(Debug, Default, Clone, PartialEq, Copy)]
pub enum ViewType {
    Editor,
    LineGraph,
    BarChart,
    StackedBarChart,
    #[default]
    None,
}

impl ViewType {
    pub const ALL: &'static [Self] = &[Self::Editor, Self::LineGraph, Self::BarChart, Self::None];

    /// Options for the setup wizard
    pub const WIZARD: &'static [Self] = &[
        Self::Editor,
        Self::LineGraph,
        Self::BarChart,
        Self::StackedBarChart,
    ];

    pub fn name(&self) -> String {
        match self {
            Self::None => "None".into(),
            Self::Editor => "Editor".into(),
            Self::LineGraph => "Line Graph".into(),
            Self::BarChart => "Bar Chart".into(),
            Self::StackedBarChart => "Stacked Bar Chart".into(),
        }
    }

    pub fn display(&self) -> Row<'_, Message, Theme, Renderer> {
        let txt = text(self.name());
        match self {
            Self::Editor => {
                let icon = icons::icon(icons::EDITOR);
                row!(icon, txt).spacing(5)
            }
            Self::LineGraph => {
                let icon = icons::icon(icons::CHART);
                row!(icon, txt).spacing(5)
            }
            Self::BarChart => {
                let icon = icons::icon(icons::BARCHART);
                row!(icon, txt).spacing(5)
            }
            Self::StackedBarChart => {
                let icon = icons::icon(icons::BARCHART);
                row!(icon, txt).spacing(5)
            }
            Self::None => Row::new(),
        }
    }

    /// Returns true if this view type needs a setup configuration
    pub fn has_config(&self) -> bool {
        match self {
            Self::Editor => false,
            Self::LineGraph => true,
            Self::BarChart => true,
            Self::StackedBarChart => true,
            Self::None => false,
        }
    }

    pub fn is_supported_filetype(&self, extn: &FileType) -> bool {
        match self {
            Self::LineGraph => match extn {
                FileType::CSV => true,
                _ => false,
            },
            Self::BarChart => match extn {
                FileType::CSV => true,
                _ => false,
            },
            Self::StackedBarChart => match extn {
                FileType::CSV => true,
                _ => false,
            },
            Self::Editor => true,
            Self::None => false,
        }
    }
}

impl fmt::Display for ViewType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

pub fn home_view<'a>() -> Container<'a, Message, Theme, Renderer> {
    let new_btn: Button<'_, Message, Theme, Renderer> = button("New File")
        .on_press(Message::OpenTab(
            None,
            View::Editor(EditorTabData::default()),
        ))
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

pub type Tabs<Theme> = TabsState<Theme>;
pub type TabsMessage = TabBarMessage;
