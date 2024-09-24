#![allow(unused_imports, dead_code)]
use std::{collections::HashMap, fmt::Debug, path::PathBuf};

use iced::{
    alignment, theme,
    widget::{canvas, column, container, horizontal_space, row, text, Canvas},
    Alignment, Color, Element, Font, Length, Point, Renderer, Size, Theme,
};

use modav_core::{
    models::{
        stacked_bar::{StackedBar, StackedBarChart},
        Scale,
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
use graph::{Axis, AxisKind, Graph, Graphable};

use super::{
    shared::{ContentAreaContainer, ToolbarContainerStyle, ToolbarMenuStyle, ToolbarStyle},
    tabs::TabLabel,
    Viewable,
};

const DEFAULT_WIDTH: f32 = 50.0;

#[derive(Debug, Clone, Copy)]
enum StackType {
    Positives,
    Negatives,
    Mixed,
}

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

impl Graphable<Data, Data> for GraphBar {
    type Data = usize;

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
        if self.id != *data {
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
        x_points: &HashMap<Data, f32>,
        x_axis: f32,
        y_points: &HashMap<Data, f32>,
        _y_axis: f32,
        _data: &Self::Data,
    ) {
        let width = |base: &HashMap<Data, f32>| {
            let (x, y) = base
                .values()
                .into_iter()
                .fold((None, None), |acc, curr| match acc {
                    (None, None) => (Some(curr), None),
                    (Some(pp), None) | (None, Some(pp)) => {
                        if pp < curr {
                            (Some(pp), Some(curr))
                        } else {
                            (Some(curr), Some(pp))
                        }
                    }
                    (Some(pp), Some(p)) => {
                        if pp < curr && curr < p {
                            (Some(pp), Some(curr))
                        } else if curr < pp {
                            (Some(curr), Some(pp))
                        } else {
                            (Some(pp), Some(p))
                        }
                    }
                });
            match y {
                Some(y) => {
                    let width = f32::abs(
                        x.expect("BarChart draw: Empty Bar charts should not be possible") - y,
                    ) / 2.0;

                    width.min(DEFAULT_WIDTH)
                }
                None => DEFAULT_WIDTH,
            }
        };

        let width = width(x_points);

        let x = x_points.get(self.x()).cloned().unwrap_or(-1.0);

        if x < 0.0 {
            warn!("Bart char x point, {} not found", self.x());
            return;
        }

        let y = y_points.get(self.y()).cloned().unwrap_or(-1.0);

        if y < 0.0 {
            warn!("Bart char x point, {} not found", &self.y());
            return;
        }

        let mut base = x_axis;
        let y = x_axis - y;

        let mut fractions = self.bar.fractions.iter().collect::<Vec<(&String, &f64)>>();
        fractions.sort_by(|x, y| {
            let x = *x.1;
            let y = y.1;

            x.total_cmp(y)
        });

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
    x_axis: Axis<Data>,
    y_axis: Axis<Data>,
    is_horizontal: bool,
    order: bool,
    bars: Vec<GraphBar>,
    cache: canvas::Cache,
    labels_len: usize,
}

impl StackedBarChartTab {
    fn create_axis(
        is_horizontal: bool,
        x_scale: Scale<Data>,
        y_scale: Scale<Data>,
        stack_type: StackType,
        order: bool,
    ) -> (Axis<Data>, Axis<Data>) {
        if is_horizontal {
            todo!()
        } else {
            let mut x_points = x_scale.points();
            if order {
                x_points.sort();
            }
            let kind = AxisKind::BaseHorizontal(x_points);
            let x_axis = Axis::new(kind, 1.0);

            let y_axis = match stack_type {
                StackType::Mixed => {
                    let mut y_points = y_scale.points();
                    if order {
                        y_points.sort();
                    }
                    let mut pos = vec![];
                    let mut neg = vec![];

                    for point in y_points {
                        match point {
                            Data::Number(num) => {
                                if num < 0 {
                                    neg.push(Data::Number(num));
                                } else {
                                    pos.push(Data::Number(num));
                                }
                            }
                            Data::Integer(int) => {
                                if int < 0 {
                                    neg.push(Data::Integer(int));
                                } else {
                                    pos.push(Data::Integer(int));
                                }
                            }
                            Data::Float(float) => {
                                if float < 0.0 {
                                    neg.push(Data::Float(float));
                                } else {
                                    pos.push(Data::Float(float));
                                }
                            }
                            _ => {}
                        };
                    }
                    let fraction = pos.len() as f32 / (pos.len() + neg.len()) as f32;
                    let kind = AxisKind::SplitVertical(pos, neg);

                    Axis::new(kind, fraction)
                }
                StackType::Positives | StackType::Negatives => {
                    let mut y_points = y_scale.points();
                    if order {
                        y_points.sort();
                    }
                    let kind = AxisKind::BaseVertical(y_points);
                    Axis::new(kind, 1.0)
                }
            };

            return (x_axis, y_axis);
        }
    }

    fn graph(&self) -> Element<'_, StackedBarChartMessage> {
        let content = Canvas::new(
            Graph::new(&self.x_axis, &self.y_axis, &self.bars, &self.cache)
                .labels_len(self.labels_len)
                .data(0),
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

        let kind = if chart.has_negatives && chart.has_positives {
            StackType::Mixed
        } else if chart.has_positives {
            StackType::Positives
        } else {
            StackType::Negatives
        };

        let StackedBarChart {
            x_axis,
            x_scale,
            y_axis,
            y_scale,
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

        let (mut x, mut y) = Self::create_axis(is_horizontal, x_scale, y_scale, kind, order);

        if let Some(label) = x_axis {
            x = x.label(label);
        }
        if let Some(label) = y_axis {
            y = y.label(label);
        }
        if let Some(caption) = caption {
            x = x.caption(caption);
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
