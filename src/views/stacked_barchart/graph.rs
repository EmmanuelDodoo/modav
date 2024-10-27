use core::f32;
use modav_core::repr::Data;
use std::{
    collections::HashMap,
    fmt::{self, Debug},
};

use iced::{
    alignment::{Horizontal, Vertical},
    font,
    widget::canvas::{self, Frame, Geometry, Path, Stroke, Text},
    Color, Pixels, Point, Rectangle, Renderer, Size, Theme, Vector,
};

use modav_core::models::{AxisPoints, Scale};

use crate::widgets::toolbar::ToolbarOption;

#[derive(Debug, Clone, Default, Copy, PartialEq)]
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
    None,
}

#[allow(dead_code)]
impl LegendPosition {
    pub const ALL: [Self; 10] = [
        Self::TopLeft,
        Self::TopCenter,
        Self::TopRight,
        Self::CenterLeft,
        Self::Center,
        Self::CenterRight,
        Self::BottomLeft,
        Self::BottomCenter,
        Self::BottomRight,
        Self::None,
    ];

    fn icon(&self) -> char {
        match self {
            Self::TopLeft => '\u{E808}',
            Self::TopCenter => '\u{E807}',
            Self::TopRight => '\u{E809}',
            Self::CenterLeft => '\u{E804}',
            Self::Center => '\u{E803}',
            Self::CenterRight => '\u{E805}',
            Self::BottomLeft => '\u{E801}',
            Self::BottomCenter => '\u{E800}',
            Self::BottomRight => '\u{E802}',
            Self::None => '\u{E806}',
        }
    }

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
            LegendPosition::None => Point::new(f32::INFINITY, f32::INFINITY),
        }
    }
}

impl fmt::Display for LegendPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                LegendPosition::TopLeft => "Top Left",
                LegendPosition::TopCenter => "Top Center",
                LegendPosition::TopRight => "Top Right",
                LegendPosition::CenterLeft => "Center Left",
                LegendPosition::Center => "Center",
                LegendPosition::CenterRight => "Center Right",
                LegendPosition::BottomLeft => "Bottom Left",
                LegendPosition::BottomCenter => "Bottom Center",
                LegendPosition::BottomRight => "Bottom Right",
                LegendPosition::None => "No Legend",
            }
        )
    }
}

impl ToolbarOption for LegendPosition {
    fn icon(&self) -> char {
        self.icon()
    }
}

pub trait Graphable {
    type Data: Default + Debug;

    fn label(&self) -> Option<&String>;

    fn draw_legend(
        &self,
        frame: &mut Frame,
        bounds: Rectangle,
        color: Color,
        idx: usize,
        data: &Self::Data,
    );

    /// Returns true if `Self` should not be skipped when drawing legend
    fn draw_legend_filter(&self, _data: &Self::Data) -> bool {
        true
    }

    fn draw(
        &self,
        frame: &mut Frame,
        x_output: &DrawnOutput,
        y_output: &DrawnOutput,
        data: &Self::Data,
    );
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct AxisData {
    axis_color: Color,
    label_color: Color,
    text_color: Color,
    outlines_color: Color,

    height: f32,
    width: f32,

    label_size: Pixels,
    caption_size: Pixels,
    point_size: Pixels,
    x_point_padding: f32,

    x_padding_left: f32,
    x_padding_right: f32,
    true_x_length: f32,
    x_offset_right: f32,
    x_offset_left: f32,
    x_offset_length: f32,

    y_padding_top: f32,
    y_padding_bottom: f32,
    true_y_length: f32,

    y_offset_top: f32,
    y_offset_bottom: f32,
    y_offset_length: f32,

    y_top: f32,
    y_bottom: f32,
    x_left: f32,
    x_right: f32,

    bottom_text_y: f32,
}

impl AxisData {
    fn new(frame: &Frame, theme: &Theme, x_pos: f32, y_pos: f32) -> Self {
        let background = theme.extended_palette().background;
        let axis_color = background.base.color;
        let label_color = theme.extended_palette().secondary.strong.text;
        let text_color = theme.palette().text;
        let outlines_color = theme.extended_palette().background.weak.color;

        let height = frame.height();
        let width = frame.width();

        let label_size = 16.0.into();
        let caption_size = 14.0.into();
        let point_size = 14.0.into();
        let x_point_padding = 5.0;

        let x_padding_left = 0.05 * width;
        let x_padding_right = x_padding_left;
        let true_x_length = width - x_padding_left - x_padding_right;
        let x_offset_right = 0.015 * true_x_length;
        let x_offset_left = 0.045 * true_x_length;
        let x_offset_length = true_x_length - x_offset_left - x_offset_right;

        let y_padding_top = 0.025 * height;
        let y_padding_bottom = 2.5 * y_padding_top;
        let true_y_length = height - y_padding_top - y_padding_bottom;

        let y_offset_top = 0.025 * true_y_length;
        let y_offset_bottom = y_offset_top;
        let y_offset_length = true_y_length - y_offset_top - y_offset_bottom;
        let y = height - (0.5 * y_padding_bottom);

        let y_top = x_pos * y_offset_length;
        let y_bottom = y_offset_length - y_top;

        let x_right = y_pos * x_offset_length;
        let x_left = x_offset_length - x_right;

        Self {
            axis_color,
            label_color,
            text_color,
            outlines_color,
            height,
            width,
            label_size,
            caption_size,
            point_size,
            x_point_padding,
            x_padding_left,
            x_padding_right,
            true_x_length,
            x_offset_right,
            x_offset_left,
            x_offset_length,
            y_padding_top,
            y_padding_bottom,
            true_y_length,
            y_offset_top,
            y_offset_bottom,
            y_offset_length,
            y_top,
            y_bottom,
            x_left,
            x_right,
            bottom_text_y: y,
        }
    }
}

/// Information about an Axis
#[derive(Debug, Clone, PartialEq)]
pub struct DrawnOutput {
    /// Mapping of where a given [`Data`] point is located on this Axis
    pub record: HashMap<Data, f32>,
    /// The location of the axis. For the X axis, this is how far down it is.
    /// For the Y axis, this is how far left it is
    pub axis_pos: f32,
    /// Spacing of points on the axis
    pub spacing: f32,
    /// The smallest difference between any two numeric points
    pub step: f32,
}

impl DrawnOutput {
    /// Returns the position of data if present, else the closest approximate point
    /// of where data would be
    pub fn get_closest(&self, data: &Data, is_x: bool) -> Option<f32> {
        let data = match self.record.get(data) {
            Some(point) => *point,
            None => {
                let closest = self
                    .record
                    .keys()
                    .into_iter()
                    .fold(None, |acc, curr| match acc {
                        Some(prev) => {
                            if curr < data && curr > prev {
                                Some(curr)
                            } else if curr < data && prev > data {
                                Some(curr)
                            } else {
                                Some(prev)
                            }
                        }
                        None => Some(curr),
                    });

                let closest = closest?;

                let point = self.record.get(closest).unwrap();

                match (data, closest) {
                    (Data::Integer(a), Data::Integer(b)) => {
                        let diff = a - b;
                        let ratio = diff as f32 / self.step;
                        if is_x {
                            point + (ratio * self.spacing)
                        } else {
                            point - (ratio * self.spacing)
                        }
                    }
                    (Data::Number(a), Data::Number(b)) => {
                        let diff = a - b;
                        let ratio = diff as f32 / self.step;
                        if is_x {
                            point + (ratio * self.spacing)
                        } else {
                            point - (ratio * self.spacing)
                        }
                    }
                    (Data::Float(a), Data::Float(b)) => {
                        let diff = a - b;
                        let ratio = diff / self.step;
                        if is_x {
                            point + (ratio * self.spacing)
                        } else {
                            point - (ratio * self.spacing)
                        }
                    }
                    _ => {
                        return None;
                    }
                }
            }
        };

        Some(data)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AxisKind {
    BaseHorizontal(Vec<Data>),
    BaseVertical(Vec<Data>),
    SplitHorizontal(Vec<Data>, Vec<Data>),
    SplitVertical(Vec<Data>, Vec<Data>),
}

#[allow(dead_code)]
impl AxisKind {
    const AXIS_THICKNESS: f32 = 2.0;
    const OUTLINES_THICKNESS: f32 = 0.5;
    const POINT_THICKNESS: f32 = Self::OUTLINES_THICKNESS * 2.0;

    fn is_split(&self) -> bool {
        match self {
            Self::BaseVertical(_) | Self::BaseHorizontal(_) => false,
            Self::SplitVertical(_, _) | Self::SplitHorizontal(_, _) => true,
        }
    }

    fn positives_number(&self) -> usize {
        match self {
            Self::SplitHorizontal(pos, _) => pos.len(),
            Self::SplitVertical(pos, _) => pos.len(),
            Self::BaseVertical(points) => points.len(),
            Self::BaseHorizontal(points) => points.len(),
        }
    }

    fn negatives_number(&self) -> usize {
        match self {
            Self::SplitHorizontal(_, neg) => neg.len(),
            Self::SplitVertical(_, neg) => neg.len(),
            Self::BaseVertical(_) => 0,
            Self::BaseHorizontal(_) => 0,
        }
    }

    fn total(&self) -> usize {
        match self {
            Self::BaseVertical(points) => points.len(),
            Self::BaseHorizontal(points) => points.len(),
            Self::SplitHorizontal(pos, neg) => pos.len() + neg.len(),
            Self::SplitVertical(pos, neg) => pos.len() + neg.len(),
        }
    }

    fn positives(&self) -> &[Data] {
        match self {
            Self::BaseHorizontal(points) => points,
            Self::BaseVertical(points) => points,
            Self::SplitVertical(pos, _) => pos,
            Self::SplitHorizontal(pos, _) => pos,
        }
    }

    fn draw_base_horizontal(
        frame: &mut Frame,
        points: &[Data],
        axis_data: AxisData,
        clean: bool,
    ) -> DrawnOutput {
        let mut record = HashMap::new();
        let points_len = points.len();

        let axis_color = axis_data.axis_color;
        let text_color = axis_data.text_color;
        let outlines_color = axis_data.outlines_color;

        let point_size = axis_data.point_size.into();
        let x_point_padding = axis_data.x_point_padding;

        let x_padding_left = axis_data.x_padding_left;
        let true_x_length = axis_data.true_x_length;
        let x_offset_left = axis_data.x_offset_left;
        let x_offset_length = axis_data.x_offset_length;

        let y_padding_top = axis_data.y_padding_top;

        let y_offset_top = axis_data.y_offset_top;

        let y_top = axis_data.y_top;
        let y_bottom = axis_data.y_bottom;

        let x = x_padding_left + (0.75 * x_offset_left);
        let y = y_padding_top + y_offset_top + y_top;
        let axis_pos = y;

        let axis_start = Point::new(x, y);
        let axis_end = Point::new(x + true_x_length, y);

        let line = Path::line(axis_start, axis_end);
        frame.stroke(
            &line,
            Stroke::default()
                .with_width(Self::AXIS_THICKNESS)
                .with_color(axis_color),
        );

        let x = x_padding_left + x_offset_left;
        let mut prev_x = x;
        let mut x_dist = 0.0;
        let mut prev = Data::None;
        let mut prev_prev = Data::None;
        let y = y + y_bottom;

        let dx = x_offset_length / (points.len() as f32);

        let outlines_number = match dx {
            0.0..50.0 => 1,
            50.0..250.0 => 5,
            _ => 10,
        };
        let outlines_width = (dx * 0.9) / (outlines_number as f32);
        let mut outlines_count = 1.0;
        let mut point_count = 0;

        let mut points = points.iter();

        while (outlines_width * outlines_count) <= x_offset_length {
            let x = x + outlines_width * outlines_count;

            if (outlines_count as i32) % outlines_number == 0 && point_count < points_len {
                x_dist = prev_x - x;
                prev_x = x;
                point_count += 1;

                if let Some(point) = points.next() {
                    record.insert(point.clone(), x);

                    if prev_prev == Data::None {
                        match point {
                            Data::Float(f) => prev_prev = Data::Float(*f),
                            Data::Integer(i) => prev_prev = Data::Integer(*i),
                            Data::Number(n) => prev_prev = Data::Number(*n),
                            _ => {}
                        }
                    } else if prev == Data::None {
                        match point {
                            Data::Float(f) => prev = Data::Float(*f),
                            Data::Integer(i) => prev = Data::Integer(*i),
                            Data::Number(n) => prev = Data::Number(*n),
                            _ => {}
                        }
                    };

                    let text_position = Point::new(x, y + x_point_padding);
                    let text = Text {
                        content: point.clone().to_string(),
                        position: text_position,
                        horizontal_alignment: Horizontal::Center,
                        color: text_color,
                        size: point_size,
                        ..Default::default()
                    };

                    frame.fill_text(text);
                }

                let outline = Path::line([x, y].into(), [x, y_offset_top].into());
                frame.stroke(
                    &outline,
                    Stroke::default()
                        .with_width(Self::POINT_THICKNESS)
                        .with_color(outlines_color),
                );
            } else {
                if !clean {
                    let outline = Path::line([x, y].into(), [x, y_offset_top].into());
                    frame.stroke(
                        &outline,
                        Stroke::default()
                            .with_color(outlines_color)
                            .with_width(Self::OUTLINES_THICKNESS),
                    );
                }
            }

            outlines_count += 1.0;
        }

        let step = match (prev, prev_prev) {
            (Data::Float(x), Data::Float(y)) => f32::abs(x - y),
            (Data::Integer(x), Data::Integer(y)) => i32::abs(x - y) as f32,
            (Data::Number(x), Data::Number(y)) => isize::abs(x - y) as f32,
            _ => 0.0,
        };

        DrawnOutput {
            record,
            step,
            axis_pos: axis_pos - Self::AXIS_THICKNESS,
            spacing: x_dist,
        }
    }

    fn draw_base_vertical(
        frame: &mut Frame,
        points: &[Data],
        axis_data: AxisData,
        clean: bool,
    ) -> DrawnOutput {
        let mut record = HashMap::new();
        let points_len = points.len();

        let axis_color = axis_data.axis_color;
        let text_color = axis_data.text_color;
        let outlines_color = axis_data.outlines_color;

        let point_size = axis_data.point_size;

        let x_padding_left = axis_data.x_padding_left;
        let x_offset_right = axis_data.x_offset_right;
        let x_offset_left = axis_data.x_offset_left;
        let x_offset_length = axis_data.x_offset_length;

        let y_padding_top = axis_data.y_padding_top;
        let true_y_length = axis_data.true_y_length;

        let y_offset_top = axis_data.y_offset_top;
        let y_offset_bottom = axis_data.y_offset_bottom;
        let y_offset_length = axis_data.y_offset_length;

        let x_left = axis_data.x_left;

        let x = x_padding_left + x_offset_left + x_left;
        let y = y_padding_top;
        let axis_pos = x;

        let axis_start = Point::new(x, y);
        let axis_end = Point::new(x, y + true_y_length - (0.75 * y_offset_bottom));

        let line = Path::line(axis_start, axis_end);
        frame.stroke(
            &line,
            Stroke::default()
                .with_color(axis_color)
                .with_width(Self::AXIS_THICKNESS),
        );

        let x = x_padding_left + (0.5 * x_offset_left);
        let y = y_padding_top + y_offset_top;
        let mut prev_y = y;
        let mut y_dist = 0.0;
        let mut prev = Data::None;
        let mut prev_prev = Data::None;

        let dy = y_offset_length / (points_len as f32);
        let outlines_number = match dy {
            0.0..50.0 => 1,
            50.0..250.0 => 5,
            _ => 10,
        };

        let outlines_height = (dy * 0.9) / outlines_number as f32;
        let mut outlines_count = 1.0;
        let mut point_count = 0;

        let mut points = points.iter().rev();

        while (outlines_height * outlines_count) <= y_offset_length {
            let y = y + outlines_height * outlines_count;
            let offset_end = x + (0.5 * x_offset_left) + x_offset_length + x_offset_right;

            if (outlines_count as i32) % outlines_number == 0 && point_count <= points_len {
                y_dist = y - prev_y;
                prev_y = y;
                point_count += 1;

                if let Some(point) = points.next() {
                    record.insert(point.clone(), y);

                    if prev_prev == Data::None {
                        match point {
                            Data::Float(f) => prev_prev = Data::Float(*f),
                            Data::Integer(i) => prev_prev = Data::Integer(*i),
                            Data::Number(n) => prev_prev = Data::Number(*n),
                            _ => {}
                        }
                    } else if prev == Data::None {
                        match point {
                            Data::Float(f) => prev = Data::Float(*f),
                            Data::Integer(i) => prev = Data::Integer(*i),
                            Data::Number(n) => prev = Data::Number(*n),
                            _ => {}
                        }
                    };

                    let text_position = Point::new(x, y);
                    let text = Text {
                        content: point.clone().to_string(),
                        position: text_position,
                        horizontal_alignment: Horizontal::Center,
                        vertical_alignment: Vertical::Center,
                        color: text_color,
                        size: point_size,
                        ..Default::default()
                    };
                    frame.fill_text(text);
                }

                let outline = Path::line(
                    [x + (0.5 * x_offset_left), y].into(),
                    [offset_end, y].into(),
                );

                frame.stroke(
                    &outline,
                    Stroke::default()
                        .with_width(Self::POINT_THICKNESS)
                        .with_color(outlines_color),
                );
            } else {
                if !clean {
                    let outline = Path::line(
                        [x + (0.5 * x_offset_left), y].into(),
                        [offset_end, y].into(),
                    );
                    frame.stroke(
                        &outline,
                        Stroke::default()
                            .with_color(outlines_color)
                            .with_width(Self::OUTLINES_THICKNESS),
                    );
                }
            }

            outlines_count += 1.0;
        }

        let step = match (prev, prev_prev) {
            (Data::Float(x), Data::Float(y)) => f32::abs(x - y),
            (Data::Integer(x), Data::Integer(y)) => i32::abs(x - y) as f32,
            (Data::Number(x), Data::Number(y)) => isize::abs(x - y) as f32,
            _ => 0.0,
        };

        DrawnOutput {
            record,
            axis_pos: axis_pos + Self::AXIS_THICKNESS,
            spacing: y_dist,
            step,
        }
    }

    fn draw_split_vertical(
        frame: &mut Frame,
        pos_points: &[Data],
        neg_points: &[Data],
        axis_data: AxisData,
        clean: bool,
    ) -> DrawnOutput {
        let mut record = HashMap::new();
        let pos_points_len = pos_points.len();
        let neg_points_len = neg_points.len();

        let axis_color = axis_data.axis_color;
        let text_color = axis_data.text_color;
        let outlines_color = axis_data.outlines_color;

        let point_size = axis_data.point_size;

        let x_padding_left = axis_data.x_padding_left;
        let x_offset_right = axis_data.x_offset_right;
        let x_offset_left = axis_data.x_offset_left;
        let x_offset_length = axis_data.x_offset_length;

        let y_padding_top = axis_data.y_padding_top;
        let true_y_length = axis_data.true_y_length;

        let y_offset_top = axis_data.y_offset_top;
        let y_offset_length = axis_data.y_offset_length;

        let x_left = axis_data.x_left;

        let y_top = axis_data.y_top;
        let y_bottom = axis_data.y_bottom;

        let x = x_padding_left + x_offset_left + x_left;
        let y = y_padding_top;
        let axis_pos = x;

        let axis_start = Point::new(x, y);
        let axis_end = Point::new(x, y + true_y_length);

        let line = Path::line(axis_start, axis_end);
        frame.stroke(
            &line,
            Stroke::default()
                .with_color(axis_color)
                .with_width(Self::AXIS_THICKNESS),
        );

        let x = x_padding_left + (0.5 * x_offset_left);
        let y = y_padding_top + y_offset_top + y_top;
        let mut prev_y = y;
        let mut y_dist = 0.0;
        let mut prev_prev = Data::None;
        let mut prev = Data::None;

        let dy = y_offset_length / (pos_points_len + neg_points_len - 1) as f32;
        let outlines_number = match dy {
            0.0..50.0 => 1,
            50.0..250.0 => 5,
            _ => 10,
        };
        let offset_end = x + (0.5 * x_offset_left) + x_offset_length + x_offset_right;

        let outlines_height = (dy * 0.85) / outlines_number as f32;

        let has_zero = match pos_points.get(0).unwrap_or(&Data::None) {
            Data::Integer(0) | Data::Number(0) | Data::Float(0.0) => true,
            _ => false,
        };

        let mut outlines_count = if has_zero { 0.0 } else { 1.0 };
        let mut point_count = 0;

        let mut points = pos_points.iter();

        while (outlines_height * outlines_count) <= y_top {
            let y = y - outlines_height * outlines_count;

            if (outlines_count as i32) % outlines_number == 0 && point_count <= pos_points_len {
                y_dist = prev_y - y;
                prev_y = y;
                point_count += 1;

                if let Some(point) = points.next() {
                    record.insert(point.clone(), y);

                    if prev_prev == Data::None {
                        match point {
                            Data::Float(f) => prev_prev = Data::Float(*f),
                            Data::Integer(i) => prev_prev = Data::Integer(*i),
                            Data::Number(n) => prev_prev = Data::Number(*n),
                            _ => {}
                        }
                    } else if prev == Data::None {
                        match point {
                            Data::Float(f) => prev = Data::Float(*f),
                            Data::Integer(i) => prev = Data::Integer(*i),
                            Data::Number(n) => prev = Data::Number(*n),
                            _ => {}
                        }
                    };

                    let text_position = Point::new(x, y);
                    let text = Text {
                        content: point.clone().to_string(),
                        position: text_position,
                        horizontal_alignment: Horizontal::Center,
                        vertical_alignment: Vertical::Center,
                        color: text_color,
                        size: point_size,
                        ..Default::default()
                    };
                    frame.fill_text(text);
                }

                let outline = Path::line(
                    [x + (0.5 * x_offset_left), y].into(),
                    [offset_end, y].into(),
                );

                frame.stroke(
                    &outline,
                    Stroke::default()
                        .with_width(Self::POINT_THICKNESS)
                        .with_color(outlines_color),
                );
            } else {
                if !clean {
                    let outline = Path::line(
                        [x + (0.5 * x_offset_left), y].into(),
                        [offset_end, y].into(),
                    );
                    frame.stroke(
                        &outline,
                        Stroke::default()
                            .with_color(outlines_color)
                            .with_width(Self::OUTLINES_THICKNESS),
                    );
                }
            }

            outlines_count += 1.0;
        }

        let mut outlines_count = 1.0;
        let mut point_count = 0;
        let mut points = neg_points.iter().rev();

        let set_y_dist = y_dist == 0.0;
        let set_prev = prev_prev == Data::None;

        while (outlines_height * outlines_count) <= y_bottom {
            let y = y + outlines_height * outlines_count;

            if (outlines_count as i32) % outlines_number == 0 && point_count <= neg_points_len {
                if set_y_dist {
                    y_dist = y - prev_y;
                    prev_y = y;
                }

                point_count += 1;

                if let Some(point) = points.next() {
                    record.insert(point.clone(), y);

                    if set_prev {
                        if prev_prev == Data::None {
                            match point {
                                Data::Float(f) => prev_prev = Data::Float(*f),
                                Data::Integer(i) => prev_prev = Data::Integer(*i),
                                Data::Number(n) => prev_prev = Data::Number(*n),
                                _ => {}
                            }
                        } else if prev == Data::None {
                            match point {
                                Data::Float(f) => prev = Data::Float(*f),
                                Data::Integer(i) => prev = Data::Integer(*i),
                                Data::Number(n) => prev = Data::Number(*n),
                                _ => {}
                            }
                        };
                    }

                    let text_position = Point::new(x, y);
                    let text = Text {
                        content: point.clone().to_string(),
                        position: text_position,
                        horizontal_alignment: Horizontal::Center,
                        vertical_alignment: Vertical::Center,
                        color: text_color,
                        size: point_size,
                        ..Default::default()
                    };
                    frame.fill_text(text);
                }

                let outline = Path::line(
                    [x + (0.5 * x_offset_left), y].into(),
                    [offset_end, y].into(),
                );

                frame.stroke(
                    &outline,
                    Stroke::default()
                        .with_width(Self::POINT_THICKNESS)
                        .with_color(outlines_color),
                );
            } else {
                if !clean {
                    let outline = Path::line(
                        [x + (0.5 * x_offset_left), y].into(),
                        [offset_end, y].into(),
                    );
                    frame.stroke(
                        &outline,
                        Stroke::default()
                            .with_color(outlines_color)
                            .with_width(Self::OUTLINES_THICKNESS),
                    );
                }
            }

            outlines_count += 1.0;
        }

        let step = match (prev, prev_prev) {
            (Data::Float(x), Data::Float(y)) => f32::abs(x - y),
            (Data::Integer(x), Data::Integer(y)) => i32::abs(x - y) as f32,
            (Data::Number(x), Data::Number(y)) => isize::abs(x - y) as f32,
            _ => 0.0,
        };

        DrawnOutput {
            record,
            axis_pos: axis_pos + Self::AXIS_THICKNESS,
            spacing: y_dist,
            step,
        }
    }

    fn draw_split_horizontal(
        frame: &mut Frame,
        pos_points: &[Data],
        neg_points: &[Data],
        axis_data: AxisData,
        clean: bool,
    ) -> DrawnOutput {
        let mut record: HashMap<Data, f32> = HashMap::new();
        let pos_points_len = pos_points.len();
        let neg_points_len = neg_points.len();

        let axis_color = axis_data.axis_color;
        let text_color = axis_data.text_color;
        let outlines_color = axis_data.outlines_color;

        let point_size = axis_data.point_size;
        let x_point_padding = axis_data.x_point_padding;

        let x_padding_left = axis_data.x_padding_left;
        let true_x_length = axis_data.true_x_length;
        let x_offset_left = axis_data.x_offset_left;
        let x_offset_length = axis_data.x_offset_length;

        let y_padding_top = axis_data.y_padding_top;

        let y_offset_top = axis_data.y_offset_top;

        let y_top = axis_data.y_top;
        let y_bottom = axis_data.y_bottom;

        let x_left = axis_data.x_left;
        let x_right = axis_data.x_right;

        let x = x_padding_left + (0.75 * x_offset_left);
        let y = y_padding_top + y_offset_top + y_top;
        let axis_pos = y;

        let axis_start = Point::new(x, y);
        let axis_end = Point::new(x + true_x_length, y);

        let line = Path::line(axis_start, axis_end);
        frame.stroke(
            &line,
            Stroke::default()
                .with_color(axis_color)
                .with_width(Self::AXIS_THICKNESS),
        );

        let x = x_padding_left + x_offset_left + x_left;
        let y = y + y_bottom;
        let mut prev_x = x;
        let mut x_dist = 0.0;
        let mut prev = Data::None;
        let mut prev_prev = Data::None;

        let dx = x_offset_length / (pos_points_len + neg_points_len - 1) as f32;
        let outlines_number = match dx {
            0.0..50.0 => 1,
            50.0..250.0 => 5,
            _ => 10,
        };
        let outlines_width = (dx * 0.85) / (outlines_number as f32);
        let has_zero = match pos_points.get(0).unwrap_or(&Data::None) {
            Data::Integer(0) | Data::Number(0) | Data::Float(0.0) => true,
            _ => false,
        };

        let mut outlines_count = if has_zero { 0.0 } else { 1.0 };
        let mut points_count = 0;

        let mut points = pos_points.iter();

        while (outlines_width * outlines_count) <= x_right {
            let x = x + outlines_width * outlines_count;

            if (outlines_count as i32) % outlines_number == 0 && points_count < pos_points_len {
                x_dist = prev_x - x;
                prev_x = x;
                points_count += 1;

                if let Some(point) = points.next() {
                    record.insert(point.clone(), x);

                    if prev_prev == Data::None {
                        match point {
                            Data::Float(f) => prev_prev = Data::Float(*f),
                            Data::Integer(i) => prev_prev = Data::Integer(*i),
                            Data::Number(n) => prev_prev = Data::Number(*n),
                            _ => {}
                        }
                    } else if prev == Data::None {
                        match point {
                            Data::Float(f) => prev = Data::Float(*f),
                            Data::Integer(i) => prev = Data::Integer(*i),
                            Data::Number(n) => prev = Data::Number(*n),
                            _ => {}
                        }
                    };

                    let text_position = Point::new(x, y + x_point_padding);
                    let text = Text {
                        content: point.clone().to_string(),
                        position: text_position,
                        horizontal_alignment: Horizontal::Center,
                        color: text_color,
                        size: point_size,
                        ..Default::default()
                    };

                    frame.fill_text(text);
                }

                let outline = Path::line([x, y].into(), [x, y_offset_top].into());
                frame.stroke(
                    &outline,
                    Stroke::default()
                        .with_width(Self::POINT_THICKNESS)
                        .with_color(outlines_color),
                );
            } else {
                if !clean {
                    let outline = Path::line([x, y].into(), [x, y_offset_top].into());
                    frame.stroke(
                        &outline,
                        Stroke::default()
                            .with_color(outlines_color)
                            .with_width(Self::OUTLINES_THICKNESS),
                    );
                }
            }

            outlines_count += 1.0;
        }

        let mut outlines_count = 1.0;
        let mut points_count = 0;
        let mut points = neg_points.iter().rev();

        let set_x_dist = x_dist == 0.0;
        let set_prev = prev_prev == Data::None;

        while (outlines_width * outlines_count) <= x_left {
            let x = x - outlines_width * outlines_count;

            if (outlines_count as i32) % outlines_number == 0 && points_count < pos_points_len {
                if set_x_dist {
                    x_dist = x - prev_x;
                    prev_x = x;
                }

                points_count += 1;

                if let Some(point) = points.next() {
                    record.insert(point.clone(), x);

                    if set_prev {
                        if prev_prev == Data::None {
                            match point {
                                Data::Float(f) => prev_prev = Data::Float(*f),
                                Data::Integer(i) => prev_prev = Data::Integer(*i),
                                Data::Number(n) => prev_prev = Data::Number(*n),
                                _ => {}
                            }
                        } else if prev == Data::None {
                            match point {
                                Data::Float(f) => prev = Data::Float(*f),
                                Data::Integer(i) => prev = Data::Integer(*i),
                                Data::Number(n) => prev = Data::Number(*n),
                                _ => {}
                            }
                        };
                    }

                    let text_position = Point::new(x, y + x_point_padding);
                    let text = Text {
                        content: point.clone().to_string(),
                        position: text_position,
                        horizontal_alignment: Horizontal::Center,
                        color: text_color,
                        size: point_size,
                        ..Default::default()
                    };

                    frame.fill_text(text);

                    let outline = Path::line([x, y].into(), [x, y_offset_top].into());
                    frame.stroke(
                        &outline,
                        Stroke::default()
                            .with_width(Self::POINT_THICKNESS)
                            .with_color(outlines_color),
                    );
                }
            } else {
                if !clean {
                    let outline = Path::line([x, y].into(), [x, y_offset_top].into());
                    frame.stroke(
                        &outline,
                        Stroke::default()
                            .with_color(outlines_color)
                            .with_width(Self::OUTLINES_THICKNESS),
                    );
                }
            }

            outlines_count += 1.0;
        }

        let step = match (prev, prev_prev) {
            (Data::Float(x), Data::Float(y)) => f32::abs(x - y),
            (Data::Integer(x), Data::Integer(y)) => i32::abs(x - y) as f32,
            (Data::Number(x), Data::Number(y)) => isize::abs(x - y) as f32,
            _ => 0.0,
        };

        DrawnOutput {
            record,
            step,
            axis_pos: axis_pos - Self::AXIS_THICKNESS,
            spacing: x_dist,
        }
    }

    fn draw(&self, frame: &mut Frame, axis_data: AxisData, clean: bool) -> DrawnOutput {
        match self {
            Self::BaseHorizontal(points) => {
                Self::draw_base_horizontal(frame, points, axis_data, clean)
            }
            Self::BaseVertical(points) => Self::draw_base_vertical(frame, points, axis_data, clean),
            Self::SplitVertical(pos, neg) => {
                Self::draw_split_vertical(frame, pos, neg, axis_data, clean)
            }
            Self::SplitHorizontal(pos, neg) => {
                Self::draw_split_horizontal(frame, pos, neg, axis_data, clean)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Axis {
    label: Option<String>,
    clean: bool,
    kind: AxisKind,
    axis_pos: f32,
    alt_axis_pos: f32,
}

impl Axis {
    pub fn new(kind: AxisKind, axis_pos: f32, alt_axis_pos: f32) -> Self {
        Self {
            kind,
            axis_pos,
            alt_axis_pos,
            label: None,
            clean: false,
        }
    }

    pub fn label(mut self, label: Option<impl Into<String>>) -> Self {
        self.label = label.map(Into::into);
        self
    }

    pub fn clean(mut self, clean: bool) -> Self {
        self.clean = clean;
        self
    }

    fn draw(&self, frame: &mut Frame, axis_data: AxisData) -> DrawnOutput {
        self.kind.draw(frame, axis_data, self.clean)
    }
}

pub struct Graph<'a, G>
where
    G: Graphable,
{
    x_axis: Axis,
    y_axis: Axis,
    cache: &'a canvas::Cache,
    graphables: &'a [G],
    data: <G as Graphable>::Data,
    legend_position: LegendPosition,
    labels_len: usize,
    caption: Option<&'a String>,
}

impl<'a, G> Graph<'a, G>
where
    G: Graphable,
{
    pub fn new(x_axis: Axis, y_axis: Axis, graphables: &'a [G], cache: &'a canvas::Cache) -> Self {
        Self {
            x_axis,
            y_axis,
            cache,
            graphables,
            data: <G as Graphable>::Data::default(),
            legend_position: LegendPosition::default(),
            labels_len: 0,
            caption: None,
        }
    }

    pub fn data(mut self, data: impl Into<<G as Graphable>::Data>) -> Self {
        self.data = data.into();
        self
    }

    pub fn legend(mut self, legend: LegendPosition) -> Self {
        self.legend_position = legend;
        self
    }

    pub fn caption(mut self, caption: Option<&'a String>) -> Self {
        self.caption = caption;
        self
    }

    pub fn labels_len(mut self, len: usize) -> Self {
        self.labels_len = len;
        self
    }

    fn draw_legend(&self, renderer: &Renderer, bounds: Rectangle, theme: &Theme) -> Geometry {
        let mut frame = Frame::new(renderer, bounds.size());

        if self.legend_position == LegendPosition::None {
            return frame.into_geometry();
        }

        if self.graphables.is_empty() {
            return frame.into_geometry();
        }

        if self.labels_len == 0 {
            return frame.into_geometry();
        }

        let size = {
            let width = f32::min(frame.size().width * 0.15, 175.0);
            let height = 25.0 + 20.0 * (self.labels_len as f32);
            Size::new(width, height)
        };

        let position = self.legend_position.position(bounds, size);
        let background = theme.extended_palette().background.weak.color;
        let text_color = theme.extended_palette().background.base.text;

        frame.stroke(
            &Path::rectangle(position, size),
            Stroke::default().with_width(1.5),
        );

        frame.fill(&Path::rectangle(position, size), background);

        let x_padding = 5.0;
        let y_padding = 2.5;

        let position = Point::new(position.x + x_padding, position.y + y_padding);

        let header_size = 16.0;

        let header = Text {
            content: "Legend".into(),
            position,
            size: header_size.into(),
            color: text_color,
            ..Default::default()
        };

        frame.fill_text(header);

        let new_y = position.y + (header_size * 1.5);
        let position = Point::new(position.x, new_y);

        let width = size.width - 2.0 * x_padding;
        let height = size.height - y_padding - (header_size * 1.5);

        let bounds = Rectangle::new(position, Size::new(width, height));

        for (i, graphable) in self
            .graphables
            .iter()
            .filter(|graphable| graphable.draw_legend_filter(&self.data))
            .enumerate()
        {
            graphable.draw_legend(&mut frame, bounds, text_color, i, &self.data);
        }

        frame.into_geometry()
    }
}

impl<'a, G, Message> canvas::Program<Message> for Graph<'a, G>
where
    G: Graphable,
{
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::advanced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let content = self.cache.draw(renderer, bounds.size(), |frame| {
            let data = AxisData::new(frame, theme, self.x_axis.axis_pos, self.y_axis.axis_pos);

            let x_output = self.x_axis.draw(frame, data);
            let y_output = self.y_axis.draw(frame, data);

            self.graphables.iter().for_each(|graphable| {
                graphable.draw(frame, &x_output, &y_output, &self.data);
            });

            if let Some(label) = self.x_axis.label.clone() {
                let x = (data.x_offset_length / 2.0) + data.x_offset_left + data.x_padding_left;
                let y = data.bottom_text_y;
                let label_position = Point::new(x, y);
                let label_size = data.label_size;

                let text = Text {
                    content: label.clone(),
                    position: label_position,
                    horizontal_alignment: Horizontal::Center,
                    vertical_alignment: Vertical::Center,
                    size: label_size,
                    color: data.label_color,
                    ..Default::default()
                };

                frame.fill_text(text);
            }

            if let Some(caption) = self.caption {
                let x = (data.x_offset_length * 0.80) + data.x_padding_left + data.x_offset_left;
                let y = data.bottom_text_y;
                let caption_position = Point::new(x, y);
                let caption_size = data.caption_size;

                let text = Text {
                    content: caption.clone(),
                    position: caption_position,
                    horizontal_alignment: Horizontal::Center,
                    vertical_alignment: Vertical::Center,
                    size: caption_size,
                    color: data.label_color,
                    font: font::Font {
                        style: font::Style::Italic,
                        ..Default::default()
                    },
                    ..Default::default()
                };
                frame.fill_text(text);
            }

            if let Some(label) = self.y_axis.label.clone() {
                let x_padding = 0.5 * data.x_padding_left;
                let y_padding = data.y_padding_top + (0.5 * data.true_y_length);

                frame.translate(Vector::new(
                    Point::ORIGIN.x + x_padding,
                    Point::ORIGIN.y + y_padding,
                ));
                frame.rotate(-90.0 * f32::consts::PI / 180.0);
                let text = Text {
                    content: label,
                    position: Point::new(0.0, 0.0),
                    horizontal_alignment: Horizontal::Center,
                    vertical_alignment: Vertical::Center,
                    color: data.label_color,
                    size: data.label_size,
                    ..Default::default()
                };
                frame.fill_text(text);
            }
        });

        vec![content, self.draw_legend(renderer, bounds, theme)]
    }
}

pub fn create_axis(
    x_scale: &Scale,
    y_scale: &Scale,
    sequential_x: bool,
    sequential_y: bool,
    clean: bool,
) -> (Axis, Axis) {
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
                let fraction = positives.len() as f32 / (positives.len() + negatives.len()) as f32;
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
                let fraction = positives.len() as f32 / (positives.len() + negatives.len()) as f32;

                let kind = AxisKind::SplitVertical(positives, negatives);

                (kind, fraction)
            }
        }
    };

    let x_axis = Axis::new(x_kind, x_fraction, y_fraction).clean(clean);
    let y_axis = Axis::new(y_kind, y_fraction, x_fraction).clean(clean);

    return (x_axis, y_axis);
}
