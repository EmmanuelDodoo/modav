/// Text always being overlayed is an Iced issue. Keep eye out for fix
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use iced::widget::{
    canvas::{self, Canvas, Frame, Geometry, Path, Stroke, Text},
    component, Component,
};
use iced::{
    alignment::{Horizontal, Vertical},
    color, mouse, Element, Length, Point, Rectangle, Renderer, Theme,
};
use iced::{Color, Size};

#[allow(dead_code)]
const WHITE: Color = color!(255, 255, 255);
#[allow(dead_code)]
const BLACK: Color = color!(0, 0, 0);
#[allow(dead_code)]
const BLUE: Color = color!(0, 0, 255);
#[allow(dead_code)]
const RED: Color = color!(255, 0, 0);
#[allow(dead_code)]
const GREEN: Color = color!(255, 0, 0);
#[allow(dead_code)]
const MAGENTA: Color = color!(205, 0, 150);

#[derive(Debug, Clone, Default)]
pub struct Axis<T>
where
    T: Clone + Into<String> + Hash + Eq,
{
    points: Vec<T>,
    label: Option<String>,
}

impl<T> Axis<T>
where
    T: Clone + Into<String> + Hash + Eq,
{
    const AXIS_THICKNESS: f32 = 2.0;
    const OUTLINES_THICKNESS: f32 = 0.5;

    pub fn new(label: Option<String>, points: Vec<T>) -> Self {
        Self { label, points }
    }

    #[allow(unused_variables)]
    fn draw(axis: &Self, frame: &mut Frame, is_x_axis: bool, theme: &Theme) -> HashMap<T, f32> {
        let mut record = HashMap::new();

        let axis_color = color!(205, 0, 150);
        let label_color = theme.palette().primary;
        let text_color = theme.palette().text;
        let outlines_color = theme.extended_palette().background.strong.color;

        let height = frame.height();
        let width = frame.width();

        let x_padding_right = 0.075 * width;
        let x_padding_left = 0.5 * x_padding_right;
        let true_x_length = width - (1.5 * x_padding_right);
        let x_offset_left = 0.05 * true_x_length;
        let x_offset_right = x_offset_left * 0.5;
        let x_offset_length = true_x_length - x_offset_left - x_offset_right;

        let y_padding_top = 0.035 * height;
        let y_padding_bottom = 0.5 * y_padding_top;
        let true_y_length = height - (1.5 * y_padding_top);
        let y_offset_bottom = 0.05 * true_y_length;
        let y_offset_top = 0.5 * y_offset_bottom;
        let y_offset_length = true_y_length - y_offset_bottom - y_offset_top;

        if is_x_axis {
            let x = x_padding_left;
            let y = y_padding_top + y_offset_length + y_offset_top;

            let axis_start = Point::new(x, y);

            let x = x + true_x_length;

            let axis_end = Point::new(x, y);

            let line = Path::line(axis_start, axis_end);
            frame.stroke(
                &line,
                Stroke::default()
                    .with_width(Self::AXIS_THICKNESS)
                    .with_color(axis_color),
            );

            let x = x_padding_left + x_offset_left;

            let dx = x_offset_length / (axis.points.len() as f32);
            let stump_height = 0.01 * height;

            for (i, point) in axis.points.iter().enumerate() {
                let i = i + 1;
                let x = x + (i as f32) * dx;

                let path = Path::line([x, y].into(), [x, y + stump_height].into());

                frame.stroke(
                    &path,
                    Stroke::default()
                        .with_width(Self::AXIS_THICKNESS)
                        .with_color(axis_color),
                );

                record.insert(point.clone(), x);

                let text_position = Point {
                    x,
                    y: y + stump_height,
                };
                let text = Text {
                    content: point.clone().into(),
                    position: text_position,
                    horizontal_alignment: Horizontal::Center,
                    color: text_color,

                    ..Default::default()
                };

                frame.fill_text(text);

                {
                    let outlines = Path::line([x, y].into(), [x, y_padding_top].into());

                    frame.stroke(
                        &outlines,
                        Stroke::default()
                            .with_width(Self::OUTLINES_THICKNESS)
                            .with_color(outlines_color),
                    );

                    let outlines = Path::line(
                        [x - 0.5 * dx, y].into(),
                        [x - 0.5 * dx, y_padding_top].into(),
                    );

                    frame.stroke(
                        &outlines,
                        Stroke::default()
                            .with_width(Self::OUTLINES_THICKNESS)
                            .with_color(outlines_color),
                    );
                }
            }

            // Axis end arrows
            {
                let diff_x = 0.75 * x_offset_right;
                let diff_y = 0.15 * x_offset_right;

                let top = Point::new(x + x_offset_length + diff_x, y - diff_y);
                let top_line = Path::line(axis_end, top);
                frame.stroke(
                    &top_line,
                    Stroke::default()
                        .with_width(Self::AXIS_THICKNESS)
                        .with_color(axis_color),
                );

                let bottom = Point::new(x + x_offset_length + diff_x, y + diff_y);
                let bottom_line = Path::line(axis_end, bottom);
                frame.stroke(
                    &bottom_line,
                    Stroke::default()
                        .with_width(Self::AXIS_THICKNESS)
                        .with_color(axis_color),
                );
            };

            if let Some(label) = &axis.label {
                let label_position = Point::new(axis_end.x + (0.45 * x_padding_right), y);
                let label_size = f32::clamp(0.25 * x_padding_right, 10.0, 16.0);

                let text = Text {
                    content: label.clone(),
                    position: label_position,
                    horizontal_alignment: Horizontal::Center,
                    vertical_alignment: Vertical::Center,
                    size: label_size.into(),
                    color: label_color,
                    ..Default::default()
                };

                frame.fill_text(text);
            }
        } else {
            let x = x_padding_left + x_offset_left;
            let y = y_padding_top;

            let axis_start = Point::new(x, y);

            let y = y + true_y_length;

            let axis_end = Point::new(x, y);

            let line = Path::line(axis_start, axis_end);
            frame.stroke(
                &line,
                Stroke::default()
                    .with_width(Self::AXIS_THICKNESS)
                    .with_color(axis_color),
            );

            let y = y - y_offset_bottom;

            let dy = y_offset_length / (axis.points.len() as f32);
            let stump_length = 0.01 * height;

            for (i, point) in axis.points.iter().enumerate() {
                let i = i + 1;

                let y = y - (i as f32) * dy;

                let path = Path::line([x, y].into(), [x - stump_length, y].into());

                frame.stroke(
                    &path,
                    Stroke::default()
                        .with_width(Self::AXIS_THICKNESS)
                        .with_color(axis_color),
                );

                record.insert(point.clone(), y);

                let text_position = Point {
                    x: (x * 0.95) - stump_length,
                    y,
                };

                let text = Text {
                    content: point.clone().into(),
                    position: text_position,
                    vertical_alignment: Vertical::Center,
                    horizontal_alignment: Horizontal::Right,
                    color: text_color,
                    ..Default::default()
                };

                frame.fill_text(text);

                {
                    let outlines = Path::line(
                        [x, y].into(),
                        [x + x_offset_length + x_offset_right, y].into(),
                    );

                    frame.stroke(
                        &outlines,
                        Stroke::default()
                            .with_width(Self::OUTLINES_THICKNESS)
                            .with_color(outlines_color),
                    );

                    let outlines = Path::line(
                        [x, y + 0.5 * dy].into(),
                        [x + x_offset_length + x_offset_right, y + 0.5 * dy].into(),
                    );

                    frame.stroke(
                        &outlines,
                        Stroke::default()
                            .with_width(Self::OUTLINES_THICKNESS)
                            .with_color(outlines_color),
                    );
                }
            }

            //Axis end arrow
            {
                let diff_x = 0.2 * y_offset_top;
                let diff_y = 0.45 * y_offset_top;

                let left = Point::new(x - diff_x, y_padding_top + diff_y);
                let left_line = Path::line(axis_start, left);
                frame.stroke(
                    &left_line,
                    Stroke::default()
                        .with_width(Self::AXIS_THICKNESS)
                        .with_color(axis_color),
                );

                let right = Point::new(x + diff_x, y_padding_top + diff_y);
                let right_line = Path::line(axis_start, right);
                frame.stroke(
                    &right_line,
                    Stroke::default()
                        .with_width(Self::AXIS_THICKNESS)
                        .with_color(axis_color),
                );
            };

            if let Some(label) = &axis.label {
                let label_position = Point::new(x, axis_start.y * 0.65);

                let text = Text {
                    content: label.clone(),
                    position: label_position,
                    horizontal_alignment: Horizontal::Center,
                    vertical_alignment: Vertical::Center,
                    color: label_color,
                    ..Default::default()
                };

                frame.fill_text(text);
            }
        }

        return record;
    }
}

#[derive(Debug, Clone)]
pub struct GraphPoint<X, Y>
where
    X: Clone + Into<String> + Hash + Eq,
    Y: Clone + Into<String> + Hash + Eq,
{
    x: X,
    y: Y,
}

impl<X, Y> GraphPoint<X, Y>
where
    X: Clone + Into<String> + Hash + Eq,
    Y: Clone + Into<String> + Hash + Eq,
{
    pub fn new(x: X, y: Y) -> Self {
        GraphPoint { x, y }
    }
}

#[derive(Debug, Clone, Default)]
pub struct GraphLine<X, Y>
where
    X: Clone + Into<String> + Hash + Eq + Debug,
    Y: Clone + Into<String> + Hash + Eq + Debug,
{
    points: Vec<GraphPoint<X, Y>>,
    label: Option<String>,
    color: Color,
}

impl<X, Y> GraphLine<X, Y>
where
    X: Clone + Into<String> + Hash + Eq + Debug,
    Y: Clone + Into<String> + Hash + Eq + Debug,
{
    pub fn new(points: Vec<GraphPoint<X, Y>>, label: Option<String>) -> Self {
        Self {
            points,
            color: color!(0, 0, 0),
            label,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    fn draw(
        line: &Self,
        frame: &mut Frame,
        x_record: &HashMap<X, f32>,
        y_record: &HashMap<Y, f32>,
    ) {
        line.points.iter().fold(None, |prev, point| {
            let x = x_record
                .get(&point.x)
                .and_then(|x| Some(x.to_owned()))
                .unwrap_or(-1.0);

            if x < 0.0 {
                println!("X Point {:?} not found", point.x);
                return prev;
            }

            let y = y_record
                .get(&point.y)
                .and_then(|x| Some(x.to_owned()))
                .unwrap_or(-1.0);

            if y < 0.0 {
                println!("Y Point {:?} not found", point.y);
                return prev;
            }

            let point = Point { x, y };

            let path = Path::circle(point.clone(), 2.0);

            frame.stroke(
                &path,
                Stroke::default().with_width(3.0).with_color(line.color),
            );

            if let Some(prev) = prev {
                let path = Path::new(|bdr| {
                    bdr.move_to(prev);
                    bdr.line_to(point);
                });
                frame.stroke(
                    &path,
                    Stroke::default().with_width(3.0).with_color(line.color),
                );
            };

            return Some(point);
        });
    }

    fn draw_legend(&self, frame: &mut Frame, position: Point, size: Size, color: Color) {
        let x = position.x;
        let y = position.y;

        let width = size.width;
        let height = size.height;

        frame.fill(
            &Path::rectangle([x, y].into(), Size::new(width, height)),
            self.color,
        );

        let label = Text {
            content: self.label.clone().unwrap_or(String::default()),
            position: Point::new(x + 1.25 * width, y + 0.5 * height),
            color,
            size: 12.0.into(),
            vertical_alignment: Vertical::Center,
            ..Default::default()
        };

        frame.fill_text(label);
    }
}

#[derive(Debug, Default)]
pub struct Graph<X, Y>
where
    X: Clone + Into<String> + Hash + Eq + Debug,
    Y: Clone + Into<String> + Hash + Eq + Debug,
{
    x_axis: Axis<X>,
    y_axis: Axis<Y>,
    lines: Vec<GraphLine<X, Y>>,
    cache: canvas::Cache,
}

impl<X, Y> Graph<X, Y>
where
    X: Clone + Into<String> + Hash + Eq + Debug,
    Y: Clone + Into<String> + Hash + Eq + Debug,
{
    pub fn new(x_axis: Axis<X>, y_axis: Axis<Y>, lines: Vec<GraphLine<X, Y>>) -> Self {
        Self {
            x_axis,
            y_axis,
            lines,
            cache: canvas::Cache::default(),
        }
    }
}

impl<Message, X, Y> Component<Message> for Graph<X, Y>
where
    X: Clone + Into<String> + Hash + Eq + Debug,
    Y: Clone + Into<String> + Hash + Eq + Debug,
{
    type Event = GraphMessage;
    type State = ();

    fn update(&mut self, _state: &mut Self::State, _event: Self::Event) -> Option<Message> {
        None
    }

    fn view(&self, _state: &Self::State) -> Element<'_, Self::Event, Theme, Renderer> {
        Canvas::new(GraphCanvas {
            x_axis: self.x_axis.clone(),
            y_axis: self.y_axis.clone(),
            lines: self.lines.clone(),
            cache: &self.cache,
        })
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
    }
}

impl<'a, Message, X, Y> From<Graph<X, Y>> for Element<'a, Message>
where
    Message: 'a + Clone + Debug,
    X: 'a + Clone + Into<String> + Hash + Eq + Debug,
    Y: 'a + Clone + Into<String> + Hash + Eq + Debug,
{
    fn from(value: Graph<X, Y>) -> Self {
        component(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GraphMessage {}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub enum LegendPosition {
    TopLeft,
    TopCenter,
    #[default]
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl LegendPosition {
    /// Returns the top left point of the legend given its size and a bound.
    fn position(&self, bounds: Rectangle, size: Size) -> Point {
        // Bottom poistions also need some review
        match self {
            LegendPosition::TopLeft => {
                let padding = {
                    let x_padding = 0.01 * bounds.width;
                    let y_padding = 0.01 * bounds.height;
                    f32::max(x_padding, y_padding)
                };

                let x = Point::ORIGIN.x + padding;
                let y = Point::ORIGIN.y + padding;

                Point::new(f32::max(x, y), f32::max(x, y))
            }
            LegendPosition::TopCenter => {
                let padding = 0.01 * bounds.height;

                let x = Point::ORIGIN.x + bounds.width * 0.5 - 0.5 * size.width;
                let y = Point::ORIGIN.y + padding;

                Point::new(x, y)
            }
            LegendPosition::TopRight => {
                let padding = {
                    let x = 0.01 * bounds.width;
                    let y = 0.01 * bounds.height;

                    f32::max(x, y)
                };

                let x = Point::ORIGIN.x + bounds.width - size.width - padding;
                let y = Point::ORIGIN.y + padding;

                Point::new(x, y)
            }
            LegendPosition::CenterLeft => {
                let padding = 0.01 * bounds.width;

                let x = Point::ORIGIN.x + padding;
                let y = Point::ORIGIN.y + bounds.height * 0.5 - 0.5 * size.height;

                Point::new(x, y)
            }
            LegendPosition::Center => {
                let x = Point::ORIGIN.x + bounds.width * 0.5 - 0.5 * size.width;
                let y = Point::ORIGIN.y + bounds.height * 0.5 - 0.5 * size.height;

                Point::new(x, y)
            }
            LegendPosition::CenterRight => {
                let padding = 0.01 * bounds.width;

                let x = Point::ORIGIN.x + bounds.width - size.width - padding;
                let y = Point::ORIGIN.y + bounds.height * 0.5 - 0.5 * size.height;

                Point::new(x, y)
            }
            LegendPosition::BottomLeft => {
                let padding = {
                    let x_padding = 0.01 * bounds.width;
                    let y_padding = 0.01 * bounds.height;
                    f32::max(x_padding, y_padding)
                };

                let x = Point::ORIGIN.x + padding;
                let y = Point::ORIGIN.y + bounds.height - size.height - padding;

                Point::new(x, y)
            }
            LegendPosition::BottomCenter => {
                let padding = 0.01 * bounds.height;

                let x = Point::ORIGIN.x + bounds.width * 0.5 - 0.5 * size.width;
                let y = Point::ORIGIN.y + bounds.height - size.height - padding;

                Point::new(x, y)
            }
            LegendPosition::BottomRight => {
                let padding = {
                    let x_padding = 0.01 * bounds.width;
                    let y_padding = 0.01 * bounds.height;
                    f32::max(x_padding, y_padding)
                };

                let x = Point::ORIGIN.x + bounds.width - size.width - padding;
                let y = Point::ORIGIN.y + bounds.height - size.height - padding;

                Point::new(x, y)
            }
        }
    }
}

#[derive(Debug)]
pub struct GraphCanvas<'a, X, Y>
where
    X: Clone + Into<String> + Hash + Eq + Debug,
    Y: Clone + Into<String> + Hash + Eq + Debug,
{
    x_axis: Axis<X>,
    y_axis: Axis<Y>,
    lines: Vec<GraphLine<X, Y>>,
    cache: &'a canvas::Cache,
}

impl<'a, X, Y> GraphCanvas<'a, X, Y>
where
    X: Clone + Into<String> + Hash + Eq + Debug,
    Y: Clone + Into<String> + Hash + Eq + Debug,
{
    fn legend(
        &self,
        renderer: &Renderer,
        bounds: Rectangle,
        position: LegendPosition,
        theme: &Theme,
    ) -> Geometry {
        let mut frame = Frame::new(renderer, bounds.size());

        let labels_len = self
            .lines
            .iter()
            .map(|line| line.label.is_some())
            .filter(|has_label| *has_label)
            .count();

        if labels_len < 1 {
            return frame.into_geometry();
        }

        let size = {
            let width = f32::min(frame.size().width * 0.125, 150.0);
            let height = 45.0 + 15.0 * ((labels_len - 1) as f32);
            Size::new(width, height)
        };

        let position = position.position(bounds, size);

        let background = theme.extended_palette().background.weak.color;
        let text_color = theme.extended_palette().secondary.base.text;

        frame.stroke(
            &Path::rectangle(position, size),
            Stroke::default().with_width(2.0),
        );

        frame.fill(&Path::rectangle(position, size), background);

        let x_padding = 5.0;

        let header = {
            let x = position.x + x_padding;
            let y = position.y * 1.0005;

            Text {
                content: "Legend".into(),
                position: [x, y].into(),
                color: text_color,
                ..Default::default()
            }
        };

        frame.fill_text(header);

        for (i, line) in self
            .lines
            .iter()
            .filter(|line| line.label.is_some())
            .enumerate()
        {
            let width = 12.0;
            let height = 12.0;

            let x = position.x + x_padding;
            let y = position.y + 25.0 + (i as f32) * (height + 5.0);

            let size = Size::new(width, height);

            let start_point = Point::new(x, y);

            line.draw_legend(&mut frame, start_point, size, text_color)
        }

        frame.into_geometry()
    }
}

impl<'a, X, Y> canvas::Program<GraphMessage> for GraphCanvas<'a, X, Y>
where
    X: Clone + Into<String> + Hash + Eq + Debug,
    Y: Clone + Into<String> + Hash + Eq + Debug,
{
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let content = self.cache.draw(renderer, bounds.size(), |frame| {
            // frame.stroke(
            // &Path::rectangle(Point::ORIGIN, frame.size()),
            // Stroke::default()
            // .with_width(1.5)
            // .with_color(color!(0, 150, 255)),
            // );

            let x_record = Axis::draw(&self.x_axis, frame, true, theme);

            let y_record = Axis::draw(&self.y_axis, frame, false, theme);

            self.lines
                .iter()
                .for_each(|line| GraphLine::draw(line, frame, &x_record, &y_record));
        });

        vec![
            content,
            self.legend(renderer, bounds, LegendPosition::default(), theme),
        ]
    }
}
