#![allow(unused_imports, dead_code)]
use core::panic;
use std::{collections::HashMap, fmt::Debug, path::PathBuf};

use iced::{
    alignment, theme,
    widget::{canvas, column, container, horizontal_space, row, text, Canvas},
    Alignment, Color, Element, Font, Length, Point, Renderer, Size, Theme,
};

use modav_core::{
    models::{
        stacked_bar::{StackedBar, StackedBarChart},
        AxisPoints, Scale,
    },
    repr::sheet::{builders::SheetBuilder, utils::Data},
};
use tracing::warn;

use crate::{
    utils::{coloring::ColorEngine, icons, AppError},
    widgets::{toolbar::ToolbarMenu, wizard::StackedBarChartConfigState},
    Message, ToolTipContainerStyle,
};

mod graph;
use graph::{Axis, AxisKind, DrawnOutput, Graph, Graphable};

use super::{
    shared::{ContentAreaContainer, ToolbarContainerStyle, ToolbarMenuStyle, ToolbarStyle},
    tabs::TabLabel,
    Viewable,
};

const DEFAULT_WIDTH: f32 = 50.0;

#[derive(Debug, Clone, PartialEq)]
struct GraphBar {
    id: usize,
    bar: StackedBar,
    colors: HashMap<String, Color>,
}

impl GraphBar {
    fn new(id: usize, bar: StackedBar, colors: HashMap<String, Color>) -> Self {
        Self { id, bar, colors }
    }

    fn x(&self) -> &Data {
        &self.bar.point.x
    }

    fn y(&self) -> &Data {
        &self.bar.point.y
    }
}

impl Graphable for GraphBar {
    type Data = (usize, bool);

    fn label(&self) -> Option<&String> {
        None
    }

    fn draw_legend(
        &self,
        frame: &mut canvas::Frame,
        bounds: iced::Rectangle,
        color: Color,
        data: &Self::Data,
    ) {
        if self.id != data.0 {
            return;
        }

        if self.colors.len() == 0 {
            return;
        }

        let y_padding = bounds.height / (self.colors.len() as f32);
        let spacing = 5.0;
        let text_size = 12.0;
        let color_size = Size::new(12.0, 12.0);

        let mut count = 0.0;

        for (label, label_color) in self.colors.iter() {
            let y = bounds.position().y + (count * y_padding);
            let position = Point::new(bounds.x, y);

            frame.fill_rectangle(position, color_size, *label_color);

            let position = Point::new(
                position.x + spacing + color_size.width,
                position.y + 0.5 * color_size.height,
            );

            let label = canvas::Text {
                content: label.clone(),
                position,
                color,
                size: text_size.into(),
                vertical_alignment: alignment::Vertical::Center,
                ..Default::default()
            };

            frame.fill_text(label);

            count += 1.0;
        }
    }

    fn draw(
        &self,
        frame: &mut canvas::Frame,
        x_output: &DrawnOutput,
        y_output: &DrawnOutput,
        data: &Self::Data,
    ) {
        let is_horizontal = data.1;

        let mut x_output = x_output;
        let mut y_output = y_output;

        if is_horizontal {
            let temp = x_output;
            x_output = y_output;
            y_output = temp;
        }

        let DrawnOutput {
            record: x_points,
            axis_pos: x_axis,
            spacing: x_spacing,
            step: x_step,
            ..
        } = x_output;

        let DrawnOutput {
            record: y_points,
            spacing: y_spacing,
            step: y_step,
            ..
        } = y_output;

        let x = match x_points.get(self.x()) {
            Some(x) => *x,
            None => {
                let closest = x_points
                    .keys()
                    .into_iter()
                    .fold(None, |acc, curr| match acc {
                        Some(prev) => {
                            if curr < self.x() && curr > prev {
                                Some(curr)
                            } else if curr < self.x() && prev > self.x() {
                                Some(curr)
                            } else {
                                Some(prev)
                            }
                        }
                        None => Some(curr),
                    });

                let closest = closest.expect("Stacked BarChart: Empty graph not possible");

                let x = x_points.get(closest).unwrap();

                match (self.x(), closest) {
                    (Data::Integer(a), Data::Integer(b)) => {
                        let diff = a - b;
                        let ratio = diff as f32 / x_step;
                        x + (ratio * x_spacing)
                    }
                    (Data::Number(a), Data::Number(b)) => {
                        let diff = a - b;
                        let ratio = diff as f32 / x_step;
                        x + (ratio * x_spacing)
                    }
                    (Data::Float(a), Data::Float(b)) => {
                        let diff = a - b;
                        let ratio = diff / x_step;
                        x + (ratio * x_spacing)
                    }
                    _ => {
                        warn!("Stacked BartChart x point, {} not found", self.x());
                        return;
                    }
                }
            }
        };

        let y = match y_points.get(self.y()) {
            Some(y) => *y,
            None => {
                let closest = y_points
                    .keys()
                    .into_iter()
                    .fold(None, |acc, curr| match acc {
                        Some(prev) => {
                            if curr < self.y() && curr > prev {
                                Some(curr)
                            } else if curr < self.y() && prev > self.y() {
                                Some(curr)
                            } else {
                                Some(prev)
                            }
                        }
                        None => Some(curr),
                    });

                let closest = closest.expect("Stacked BarChart: Empty graph not possible");
                let y = y_points.get(closest).unwrap();

                match (self.y(), closest) {
                    (Data::Integer(a), Data::Integer(b)) => {
                        let diff = a - b;
                        let ratio = diff as f32 / y_step;
                        y - (ratio * y_spacing)
                    }
                    (Data::Number(a), Data::Number(b)) => {
                        let diff = a - b;
                        let ratio = diff as f32 / y_step;
                        y - (ratio * y_spacing)
                    }
                    (Data::Float(a), Data::Float(b)) => {
                        let diff = a - b;
                        let ratio = diff / y_step;
                        y - (ratio * y_spacing)
                    }

                    _ => {
                        warn!("Stacked BarChart y point, {} not found", self.y());
                        return;
                    }
                }
            }
        };

        let mut fractions = self.bar.fractions.iter().collect::<Vec<(&String, &f64)>>();
        fractions.sort_by(|x, y| {
            let x = *x.1;
            let y = y.1;

            x.total_cmp(y)
        });

        if is_horizontal {
            let height = x_spacing / 2.0;

            let mut base = *x_axis;
            let y = y - x_axis;

            for (label, fraction) in fractions.iter().rev() {
                let width = (*fraction * y as f64) as f32;

                let size = Size::new(width, height);
                let top_left = Point::new(base, x - (0.5 * height));
                let color = self.colors.get(*label).copied().unwrap_or(Color::BLACK);
                base += width;

                frame.fill_rectangle(top_left, size, color);
            }
        } else {
            let width = x_spacing / 2.0;
            let mut base = *x_axis;
            let y = x_axis - y;

            for (label, fraction) in fractions.iter().rev() {
                let height = (*fraction * y as f64) as f32;
                base -= height;

                let size = Size::new(width, height);
                let top_left = Point::new(x - (width * 0.5), base);
                let color = self.colors.get(*label).copied().unwrap_or(Color::BLACK);

                frame.fill_rectangle(top_left, size, color);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StackedBarChartMessage {
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StackedBarChartTabData {
    title: String,
    file: PathBuf,
    order: bool,
    caption: Option<String>,
    is_horizontal: bool,
    theme: Theme,
    chart: StackedBarChart,
}

impl StackedBarChartTabData {
    pub fn new(file: PathBuf, config: StackedBarChartConfigState) -> Result<Self, AppError> {
        let StackedBarChartConfigState {
            title,
            x_col,
            acc_cols,
            is_horizontal,
            order,
            axis_label,
            header_types,
            header_labels,
            flexible,
            trim,
            caption,
            ..
        } = config;

        let sht = SheetBuilder::new(file.clone().into())
            .trim(trim)
            .flexible(flexible)
            .labels(header_labels)
            .types(header_types)
            .build()
            .map_err(AppError::CSVError)?;

        let stacked = sht
            .create_stacked_bar_chart(x_col, acc_cols, axis_label)
            .map_err(AppError::CSVError)?;

        Ok(Self {
            file,
            title,
            chart: stacked,
            order,
            is_horizontal,
            caption,
            theme: Theme::default(),
        })
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }
}

#[derive(Debug)]
pub struct StackedBarChartTab {
    title: String,
    file: PathBuf,
    x_axis: Axis,
    y_axis: Axis,
    is_horizontal: bool,
    order: bool,
    bars: Vec<GraphBar>,
    cache: canvas::Cache,
    labels_len: usize,
    caption: Option<String>,
}

impl StackedBarChartTab {
    fn create_axis(x_scale: Scale, y_scale: Scale, _order: bool) -> (Axis, Axis) {
        let sequential_x = false;
        let sequential_y = false;

        let (x_kind, y_fraction) = match x_scale.axis_points(sequential_x) {
            AxisPoints::Categorical(points) => {
                let kind = AxisKind::BaseHorizontal(points);
                (kind, 1.0)
            }
            AxisPoints::Numeric {
                positives,
                negatives,
            } => {
                if positives.is_empty() {
                    let kind = AxisKind::BaseHorizontal(negatives);
                    (kind, 0.0)
                } else if negatives.is_empty() {
                    let kind = AxisKind::BaseHorizontal(positives);
                    (kind, 1.0)
                } else if positives.is_empty() && negatives.is_empty() {
                    // Scale is never empty.
                    panic!("StackedBarChart: Empty Scale")
                } else {
                    let fraction =
                        positives.len() as f32 / (positives.len() + negatives.len()) as f32;
                    let kind = AxisKind::SplitHorizontal(positives, negatives);
                    (kind, fraction)
                }
            }
        };

        let y_points = y_scale.axis_points(sequential_y);

        let (y_kind, x_fraction) = match y_points {
            AxisPoints::Categorical(points) => {
                let kind = AxisKind::BaseVertical(points);
                (kind, 1.0)
            }
            AxisPoints::Numeric {
                positives,
                negatives,
            } => {
                if positives.is_empty() {
                    let kind = AxisKind::BaseVertical(negatives);
                    (kind, 0.0)
                } else if negatives.is_empty() {
                    let kind = AxisKind::BaseVertical(positives);
                    (kind, 1.0)
                } else if positives.is_empty() && negatives.is_empty() {
                    // Scale is never empty.
                    panic!("StackedBarChart: Empty Scale")
                } else {
                    let fraction =
                        positives.len() as f32 / (positives.len() + negatives.len()) as f32;

                    let kind = AxisKind::SplitVertical(positives, negatives);

                    (kind, fraction)
                }
            }
        };

        let x_axis = Axis::new(x_kind, x_fraction, y_fraction);
        let y_axis = Axis::new(y_kind, y_fraction, x_fraction);

        return (x_axis, y_axis);
    }

    fn graph(&self) -> Element<'_, StackedBarChartMessage> {
        let (x_axis, y_axis) = if self.is_horizontal {
            (&self.y_axis, &self.x_axis)
        } else {
            (&self.x_axis, &self.y_axis)
        };

        let content = Canvas::new(
            Graph::new(x_axis, y_axis, &self.bars, &self.cache)
                .caption(self.caption.as_ref())
                .labels_len(self.labels_len)
                .data((0, self.is_horizontal)),
        )
        .width(Length::FillPortion(24))
        .height(Length::Fill);

        content.into()
    }
}

impl Viewable for StackedBarChartTab {
    type Event = StackedBarChartMessage;
    type Data = StackedBarChartTabData;

    fn new(data: Self::Data) -> Self {
        let StackedBarChartTabData {
            file,
            title,
            theme,
            chart,
            order,
            caption,
            is_horizontal,
        } = data;

        let StackedBarChart {
            x_axis,
            mut x_scale,
            y_axis,
            mut y_scale,
            labels,
            mut bars,
            ..
        } = chart;

        let labels_len = labels.len();

        if order {
            bars.sort_by(|one, two| one.point.y.cmp(&two.point.y));
        }

        let engine = ColorEngine::new(&theme).gradual(order);

        let colors = labels
            .into_iter()
            .zip(engine)
            .collect::<HashMap<String, Color>>();

        let bars = bars
            .into_iter()
            .enumerate()
            .map(|(id, bar)| GraphBar::new(id, bar, colors.clone()))
            .collect::<Vec<GraphBar>>();

        if is_horizontal {
            let temp = x_scale;
            x_scale = y_scale;
            y_scale = temp;
        }

        let (mut x, mut y) = Self::create_axis(x_scale, y_scale, order);

        if is_horizontal {
            let temp = x;
            x = y;
            y = temp;
        }

        if let Some(label) = x_axis {
            x = x.label(label);
        }
        if let Some(label) = y_axis {
            y = y.label(label);
        }

        Self {
            title,
            file,
            labels_len,
            x_axis: x,
            y_axis: y,
            is_horizontal,
            bars,
            order,
            caption,
            cache: canvas::Cache::default(),
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

        let font = Font::with_name(icons::NAME);

        TabLabel::new(icons::BARCHART, format!("{} - {}", self.title, file_name)).icon_font(font)
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
        "Seeing this means a logic error occurred".into()
    }

    fn refresh(&mut self, data: Self::Data) {
        let new = <Self as Viewable>::new(data);

        *self = new;
    }

    fn update(&mut self, message: Self::Event) -> Option<Message> {
        match message {
            StackedBarChartMessage::None => None,
        }
    }

    fn view<'a, Message, F>(&'a self, map: F) -> Element<'a, Message, Theme, Renderer>
    where
        F: 'a + Fn(Self::Event) -> Message,
        Message: 'a + Clone + Debug,
    {
        let title = {
            let text = text(format!("{} - Stacked Bar Chart", self.title));
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
