use std::path::PathBuf;

use iced_aw::TabLabel;

use modav_core::{
    models::line::{Line, LineGraph, Scale},
    repr::csv::{utils::Data, SheetBuilder},
};

use crate::utils::AppError;
use crate::widgets::wizard::LineConfigState;

use super::{TabBarMessage, Viewable};

use super::common::graph::{Axis, Graph, GraphLine};

use iced::{
    theme,
    widget::{column, container, horizontal_space, row, text},
    Alignment, Background, Border, Element, Length, Renderer, Theme,
};

#[derive(Debug, Clone, PartialEq)]
pub struct LineTabData {
    file: PathBuf,
    title: String,
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

        Ok(Self { file, title, line })
    }
}

#[derive(Clone, Debug)]
pub enum ModelMessage {}

#[derive(Clone, Debug, PartialEq)]
pub struct LineGraphTab {
    id: usize,
    file: PathBuf,
    title: String,
    x_scale: Scale<String>,
    y_scale: Scale<Data>,
    x_label: Option<String>,
    y_label: Option<String>,
    lines: Vec<GraphLine<String, Data>>,
}

impl LineGraphTab {
    fn graph(&self) -> Graph<String, Data> {
        let x_axis = Axis::new(self.x_label.clone(), self.x_scale.points().clone());

        let y_axis = Axis::new(self.y_label.clone(), self.y_scale.points().clone());

        Graph::new(x_axis, y_axis, &self.lines)
    }
}

impl Viewable for LineGraphTab {
    type Message = ModelMessage;
    type Data = LineTabData;

    fn new(id: usize, data: Self::Data) -> Self {
        let LineTabData { file, title, line } = data;
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

        let lines = lines
            .into_iter()
            .map(|line| {
                let Line { points, label } = line;
                GraphLine::new(points, label)
            })
            .collect();

        Self {
            id,
            file,
            title,
            lines,
            x_scale,
            y_scale,
            x_label: Some(x_label),
            y_label: Some(y_label),
        }
    }

    fn id(&self) -> usize {
        self.id
    }

    fn is_dirty(&self) -> bool {
        false
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn tab_label(&self) -> TabLabel {
        let file_name = self
            .file
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("New File");

        TabLabel::Text(format!("{} - {}", self.title, file_name))
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
        let LineTabData { file, title, line } = data;
        let LineGraph {
            x_scale,
            y_scale,
            x_label,
            y_label,
            lines,
        } = line;

        let lines = lines
            .into_iter()
            .map(|line| {
                let Line { points, label } = line;
                GraphLine::new(points, label)
            })
            .collect();

        self.title = title;
        self.file = file;
        self.lines = lines;
        self.x_scale = x_scale;
        self.y_scale = y_scale;
        self.x_label = Some(x_label);
        self.y_label = Some(y_label);
    }

    fn update(&mut self, message: Self::Message) {
        match message {}
    }

    fn view(&self) -> Element<'_, TabBarMessage, Theme, Renderer> {
        let title = {
            let text = text(format!("{} - Model", self.title));
            row!(horizontal_space(), text, horizontal_space())
                .width(Length::Fill)
                .align_items(Alignment::Center)
        };

        let content_area = container(self.graph())
            .max_width(1100)
            // .padding([5, 10])
            // .width(Length::FillPortion(20))
            // .height(Length::FillPortion(3))
            .style(theme::Container::Custom(Box::new(ContentAreaContainer)));

        let content = column!(title, content_area)
            .align_items(Alignment::Center)
            .spacing(20)
            .height(Length::Fill)
            .width(Length::Fill);

        container(content)
            .padding([10, 30, 30, 30])
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
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
