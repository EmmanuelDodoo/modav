use std::{fmt::Debug, path::PathBuf};

use modav_core::{
    models::line::{Line, LineGraph, Scale},
    repr::csv::{utils::Data, SheetBuilder},
};

use crate::utils::{coloring, icons, AppError};
use crate::widgets::wizard::LineConfigState;

use super::{TabLabel, Viewable};
use crate::Message;

use super::common::graph::{Axis, Graph, GraphLine};

use iced::{
    theme,
    widget::{column, container, horizontal_space, row, text},
    Alignment, Background, Border, Element, Length, Renderer, Theme,
};

use coloring::ColorEngine;

#[derive(Debug, Clone, PartialEq)]
pub struct LineTabData {
    file: PathBuf,
    title: String,
    theme: Theme,
    line: LineGraph<String, Data>,
}

impl LineTabData {
    pub fn new(file: PathBuf, config: LineConfigState) -> Result<Self, AppError> {
        let LineConfigState {
            title,
            x_label,
            y_label,
            label_strat,
            row_exclude,
            col_exclude,
            trim,
            flexible,
            header_types,
            header_labels,
            ..
        } = config;

        let sht = SheetBuilder::new(file.clone().into())
            .trim(trim)
            .flexible(flexible)
            .labels(header_labels)
            .types(header_types)
            .build()
            .map_err(AppError::CSVError)?;

        let line = sht
            .create_line_graph(
                Some(x_label),
                Some(y_label),
                label_strat,
                row_exclude,
                col_exclude,
            )
            .map_err(AppError::CSVError)?;

        Ok(Self {
            file,
            title,
            line,
            theme: Theme::default(),
        })
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }
}

#[derive(Clone, Debug)]
pub enum ModelMessage {
    OpenEditor,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LineGraphTab {
    file: PathBuf,
    title: String,
    x_scale: Scale<String>,
    y_scale: Scale<Data>,
    x_label: Option<String>,
    y_label: Option<String>,
    lines: Vec<GraphLine<String, Data>>,
    theme: Theme,
}

impl LineGraphTab {
    fn graph(&self) -> Graph<ModelMessage, String, Data> {
        let x_axis = Axis::new(self.x_label.clone(), self.x_scale.points().clone());

        let y_axis = Axis::new(self.y_label.clone(), self.y_scale.points().clone());

        Graph::new(x_axis, y_axis, &self.lines).on_editor(ModelMessage::OpenEditor)
    }
}

impl Viewable for LineGraphTab {
    type Event = ModelMessage;
    type Data = LineTabData;

    fn new(data: Self::Data) -> Self {
        let LineTabData {
            file,
            title,
            line,
            theme,
        } = data;

        let LineGraph {
            x_scale,
            y_scale,
            lines,
            y_label,
            x_label,
        } = line;

        let title = if title.is_empty() {
            "Untitled".into()
        } else {
            title
        };

        let colors = ColorEngine::new(&theme);

        let lines = lines
            .into_iter()
            .zip(colors)
            .map(|(line, color)| {
                let Line { points, label } = line;
                GraphLine::new(points, label, color)
            })
            .collect();

        Self {
            file,
            title,
            lines,
            theme,
            x_scale,
            y_scale,
            x_label: Some(x_label),
            y_label: Some(y_label),
        }
    }

    fn is_dirty(&self) -> bool {
        false
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn label(&self) -> TabLabel {
        let file_name = self
            .file
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("New File");

        TabLabel::new(
            icons::CHART,
            format!("{} - {}", self.title, file_name),
        )
    }

    fn content(&self) -> Option<String> {
        None
    }

    fn path(&self) -> Option<PathBuf> {
        Some(self.file.clone())
    }

    fn can_save(&self) -> bool {
        false
    }

    fn modal_msg(&self) -> String {
        "Nothing to show in modal".into()
    }

    fn refresh(&mut self, data: Self::Data) {
        let LineTabData {
            file,
            title,
            line,
            theme,
        } = data;

        let LineGraph {
            x_scale,
            y_scale,
            x_label,
            y_label,
            lines,
        } = line;

        let colors = ColorEngine::new(&self.theme);

        let lines = lines
            .into_iter()
            .zip(colors)
            .map(|(line, color)| {
                let Line { points, label } = line;
                GraphLine::new(points, label, color)
            })
            .collect();

        self.title = title;
        self.file = file;
        self.lines = lines;
        self.theme = theme;
        self.x_scale = x_scale;
        self.y_scale = y_scale;
        self.x_label = Some(x_label);
        self.y_label = Some(y_label);
    }

    fn update(&mut self, message: Self::Event) -> Option<Message> {
        match message {
            ModelMessage::OpenEditor => Some(Message::OpenEditor(Some(self.file.clone()))),
        }
    }

    fn view<'a, Message, F>(&'a self, map: F) -> Element<'a, Message, Theme, Renderer>
    where
        F: 'a + Fn(Self::Event) -> Message,
        Message: 'a + Clone + Debug,
    {
        let title = {
            let text = text(format!("{} - Model", self.title));
            row!(horizontal_space(), text, horizontal_space())
                .width(Length::Fill)
                .align_items(Alignment::Center)
        }
        .height(Length::Shrink);

        let content_area = container(self.graph())
            .max_width(1450)
            // .padding([5, 10])
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::Container::Custom(Box::new(ContentAreaContainer)));

        let content = column!(title, content_area)
            .align_items(Alignment::Center)
            .spacing(20)
            .height(Length::Fill)
            .width(Length::Fill);

        let content: Element<Self::Event, Theme, Renderer> = container(content)
            .padding([10, 30, 30, 15])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

        content.map(map)
    }
}

pub struct ContentAreaContainer;
impl container::StyleSheet for ContentAreaContainer {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        let border_color = style.extended_palette().primary.weak.color;
        let background_color = style.extended_palette().background.strong.color;

        let border = Border {
            color: border_color,
            width: 1.5,
            ..Default::default()
        };

        let background = Background::Color(background_color);

        container::Appearance {
            border,
            background: Some(background),
            ..Default::default()
        }
    }
}
