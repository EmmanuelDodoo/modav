#![allow(dead_code, unused_variables, unused_imports)]
use core::f32;
use modav_core::repr::Data;
use std::{
    collections::HashMap,
    fmt::{self, Debug, Display},
    hash::Hash,
};

use iced::{
    alignment::{self, Horizontal, Vertical},
    color, font, mouse,
    widget::canvas::{self, Frame, Geometry, Path, Stroke, Text},
    Color, Pixels, Point, Rectangle, Renderer, Size, Theme, Vector,
};

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

pub trait Graphable<X, Y> {
    type Data: Default + Debug;

    fn label(&self) -> Option<&String>;

    fn draw_legend(&self, frame: &mut Frame, bounds: Rectangle, color: Color, data: &Self::Data);

    //fn color(&self) -> &Color;

    fn draw(
        &self,
        frame: &mut Frame,
        x_points: &HashMap<X, f32>,
        x_axis: f32,
        y_points: &HashMap<Y, f32>,
        y_axis: f32,
        data: &Self::Data,
    );
}

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

    bottom_text_y: f32,
}

impl AxisData {
    fn new(frame: &Frame, theme: &Theme) -> Self {
        let background = theme.extended_palette().background;
        let axis_color = background.base.color;
        let label_color = theme.extended_palette().secondary.strong.text;
        let text_color = theme.palette().text;
        let outlines_color = theme.extended_palette().background.weak.color;

        let height = frame.height();
        let width = frame.width();

        let label_size = 16.0.into();
        let caption_size = 14.0.into();
        let point_size = 12.0.into();

        let x_padding_left = 0.05 * width;
        let x_padding_right = x_padding_left;
        let true_x_length = width - x_padding_left - x_padding_right;
        let x_offset_right = 0.025 * true_x_length;
        let x_offset_left = 0.045 * true_x_length;
        let x_offset_length = true_x_length - x_offset_left - x_offset_right;

        let y_padding_top = 0.025 * height;
        let y_padding_bottom = 2.5 * y_padding_top;
        let true_y_length = height - y_padding_top - y_padding_bottom;

        let y_offset_top = 0.025 * true_y_length;
        let y_offset_bottom = y_offset_top;
        let y_offset_length = true_y_length - y_offset_top - y_offset_bottom;
        let y = height - (0.5 * y_padding_bottom);

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
            bottom_text_y: y,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AxisKind<T> {
    BaseHorizontal(Vec<T>),
    BaseVertical(Vec<T>),
    SplitHorizontal(Vec<T>, Vec<T>),
    SplitVertical(Vec<T>, Vec<T>),
}

impl<T> AxisKind<T>
where
    T: Hash + Eq + Display + Clone,
{
    const AXIS_THICKNESS: f32 = 2.0;
    const OUTLINES_THICKNESS: f32 = 0.5;

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

    fn positives(&self) -> &[T] {
        match self {
            Self::BaseHorizontal(points) => points,
            Self::BaseVertical(points) => points,
            Self::SplitVertical(pos, _) => pos,
            Self::SplitHorizontal(pos, _) => pos,
        }
    }

    #[allow(unused_variables)]
    fn draw_base_horizontal(
        frame: &mut Frame,
        theme: &Theme,
        points: &[T],
        label: Option<String>,
        caption: Option<String>,
        y_fraction: f32,
        clean: bool,
    ) -> (f32, HashMap<T, f32>) {
        let mut record = HashMap::new();
        let data = AxisData::new(frame, theme);
        let points_len = points.len();

        let axis_color = data.axis_color;
        let label_color = data.label_color;
        let text_color = data.text_color;
        let outlines_color = data.outlines_color;

        let point_size = data.point_size.into();

        let x_padding_left = data.x_padding_left;
        let x_padding_right = data.x_padding_right;
        let true_x_length = data.true_x_length;
        let x_offset_right = data.x_offset_right;
        let x_offset_left = data.x_offset_left;
        let x_offset_length = data.x_offset_length;

        let y_padding_top = data.y_padding_top;
        let y_padding_bottom = data.y_padding_bottom;
        let true_y_length = data.true_y_length;

        let y_offset_top = data.y_offset_top;
        let y_offset_bottom = data.y_offset_bottom;
        let y_offset_length = data.y_offset_length;
        let bottom_text_y = data.bottom_text_y;

        let y_top = y_fraction * y_offset_length;
        let y_bottom = y_offset_length - y_top;

        let x = x_padding_left;
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
        let y = y + y_bottom;

        let dx = x_offset_length / (points.len() as f32);

        let outlines_number = match dx {
            0.0..50.0 => 1,
            50.0..250.0 => 5,
            _ => 10,
        };
        let outlines_width = (dx * 0.85) / (outlines_number as f32);
        let mut outlines_count = 1.0;
        let mut point_count = 0;

        let mut points = points.iter();

        while (outlines_width * outlines_count) <= x_offset_length {
            let x = x + outlines_width * outlines_count;
            //println!("{}", (outlines_count as i32) % outlines_number);

            if (outlines_count as i32) % outlines_number == 0 && point_count < points_len {
                point_count += 1;

                if let Some(point) = points.next() {
                    record.insert(point.clone(), x);

                    let text_position = Point::new(x, y + 5.0);
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
                        .with_width(Self::OUTLINES_THICKNESS)
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

        if let Some(label) = label {
            let x = (x_offset_length / 2.0) + x_offset_left + x_padding_left;
            let y = bottom_text_y;
            let label_position = Point::new(x, y);
            let label_size = data.label_size;

            let text = Text {
                content: label.clone(),
                position: label_position,
                horizontal_alignment: Horizontal::Center,
                vertical_alignment: Vertical::Center,
                size: label_size,
                color: label_color,
                ..Default::default()
            };

            frame.fill_text(text);
        }

        if let Some(caption) = caption {
            let x = (x_offset_length * 0.80) + x_padding_left + x_offset_left;
            let y = bottom_text_y;
            let caption_position = Point::new(x, y);
            let caption_size = data.caption_size;

            let text = Text {
                content: caption.clone(),
                position: caption_position,
                horizontal_alignment: Horizontal::Center,
                vertical_alignment: Vertical::Center,
                size: caption_size,
                color: label_color,
                font: font::Font {
                    style: font::Style::Italic,
                    ..Default::default()
                },
                ..Default::default()
            };
            frame.fill_text(text);
        }

        (axis_pos - Self::AXIS_THICKNESS, record)
    }

    #[allow(unused_variables)]
    fn draw_base_vertical(
        frame: &mut Frame,
        theme: &Theme,
        points: &[T],
        x_fraction: f32,
        clean: bool,
    ) -> (f32, HashMap<T, f32>) {
        let mut record = HashMap::new();
        let data = AxisData::new(frame, theme);
        let points_len = points.len();

        let axis_color = data.axis_color;
        let label_color = data.label_color;
        let text_color = data.text_color;
        let outlines_color = data.outlines_color;

        let point_size = data.point_size;

        let x_padding_left = data.x_padding_left;
        let x_padding_right = data.x_padding_right;
        let true_x_length = data.true_x_length;
        let x_offset_right = data.x_offset_right;
        let x_offset_left = data.x_offset_left;
        let x_offset_length = data.x_offset_length;

        let y_padding_top = data.y_padding_top;
        let y_padding_bottom = data.y_padding_bottom;
        let true_y_length = data.true_y_length;

        let y_offset_top = data.y_offset_top;
        let y_offset_bottom = data.y_offset_bottom;
        let y_offset_length = data.y_offset_length;
        let bottom_text_y = data.bottom_text_y;

        let x_left = (1.0 - x_fraction) * x_offset_length;
        let x_right = x_offset_length - x_left;

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
        let y = y_padding_top + y_offset_top;

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
                point_count += 1;

                if let Some(point) = points.next() {
                    record.insert(point.clone(), y);

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
                        .with_width(Self::OUTLINES_THICKNESS)
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

        (axis_pos + Self::AXIS_THICKNESS, record)
    }

    fn draw_split_vertical(_frame: &mut Frame, _theme: &Theme) -> (f32, HashMap<T, f32>) {
        todo!()
    }

    fn draw_split_horizontal(_frame: &mut Frame, _theme: &Theme) -> (f32, HashMap<T, f32>) {
        todo!()
    }

    fn draw(
        &self,
        frame: &mut Frame,
        theme: &Theme,
        label: Option<String>,
        caption: Option<String>,
        clean: bool,
        fraction: f32,
    ) -> (f32, HashMap<T, f32>) {
        match self {
            Self::BaseHorizontal(points) => {
                Self::draw_base_horizontal(frame, theme, points, label, caption, fraction, clean)
            }
            Self::BaseVertical(points) => {
                Self::draw_base_vertical(frame, theme, points, fraction, clean)
            }
            Self::SplitVertical(pos, neg) => Self::draw_split_vertical(frame, theme),
            Self::SplitHorizontal(pos, neg) => Self::draw_split_horizontal(frame, theme),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Axis<T = Data> {
    label: Option<String>,
    caption: Option<String>,
    clean: bool,
    kind: AxisKind<T>,
    fraction: f32,
}

impl<T> Axis<T>
where
    T: Hash + Eq + Clone + Display,
{
    pub fn new(kind: AxisKind<T>, fraction: f32) -> Self {
        Self {
            kind,
            fraction,
            label: None,
            caption: None,
            clean: false,
        }
    }

    pub fn caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn clean(mut self, clean: bool) -> Self {
        self.clean = clean;
        self
    }

    fn draw(&self, frame: &mut Frame, theme: &Theme) -> (f32, HashMap<T, f32>) {
        self.kind.draw(
            frame,
            theme,
            self.label.clone(),
            self.caption.clone(),
            self.clean,
            self.fraction,
        )
    }

    //fn draw(&self, frame: &mut Frame, theme: &Theme) -> (f32, HashMap<T, f32>) {
    //    let mut record = HashMap::new();
    //
    //    let background = theme.extended_palette().background;
    //    let axis_color = background.base.color;
    //    let label_color = theme.extended_palette().secondary.strong.text;
    //    let text_color = background.strong.text;
    //    let outlines_color = theme.extended_palette().background.weak.color;
    //
    //    let height = frame.height();
    //    let width = frame.width();
    //    let point_size = 14.0.into();
    //
    //    let x_padding_left = 0.05 * width;
    //    let x_padding_right = x_padding_left;
    //    let true_x_length = width - x_padding_left - x_padding_right;
    //    let x_offset_right = 0.025 * true_x_length;
    //    let x_offset_left = 0.075 * true_x_length;
    //    let x_offset_length = true_x_length - x_offset_left - x_offset_right;
    //
    //    let y_padding_top = 0.025 * height;
    //    let y_padding_bottom = 2.5 * y_padding_top;
    //    let true_y_length = height - y_padding_top - y_padding_bottom;
    //
    //    let y_offset_top = 0.025 * true_y_length;
    //    let y_offset_bottom = y_offset_top;
    //    let y_offset_length = true_y_length - y_offset_top - y_offset_bottom;
    //
    //    let y_top = (self.positives as f32 / self.points.len() as f32) * y_offset_length;
    //    let y_bottom = y_offset_length - y_top;
    //
    //    #[allow(unused_assignments)]
    //    let mut axis_pos = 0.0;
    //
    //    if self.is_x_axis {
    //        let x = x_padding_left;
    //        let y = y_padding_top + y_offset_top + y_top;
    //        axis_pos = y;
    //
    //        let axis_start = Point::new(x, y);
    //        let axis_end = Point::new(x + true_x_length, y);
    //
    //        let line = Path::line(axis_start, axis_end);
    //        frame.stroke(
    //            &line,
    //            Stroke::default()
    //                .with_width(Self::AXIS_THICKNESS)
    //                .with_color(axis_color),
    //        );
    //
    //        let x = x_padding_left + x_offset_left;
    //
    //        let dx = x_offset_length / (self.points.len() as f32);
    //
    //        let outlines_number = match dx {
    //            0.0..50.0 => 1,
    //            50.0..250.0 => 5,
    //            _ => 10,
    //        };
    //        let outlines_width = (dx * 0.85) / (outlines_number as f32);
    //        let mut outlines_count = 1.0;
    //        let mut point_count = 0;
    //
    //        let mut points = self.points.iter();
    //        let y = y + y_bottom;
    //
    //        while (outlines_width * outlines_count) <= x_offset_length {
    //            let x = x + outlines_width * outlines_count;
    //
    //            if outlines_count as i32 % outlines_number == 0 && point_count < self.points.len() {
    //                point_count += 1;
    //
    //                if let Some(point) = points.next() {
    //                    record.insert(point.clone(), x);
    //
    //                    let text_position = Point::new(x, y);
    //                    let text = Text {
    //                        content: point.clone().to_string(),
    //                        position: text_position,
    //                        horizontal_alignment: Horizontal::Center,
    //                        color: text_color,
    //                        size: point_size,
    //                        ..Default::default()
    //                    };
    //
    //                    frame.fill_text(text);
    //                }
    //
    //                let outline = Path::line([x, y].into(), [x, y_offset_top].into());
    //                frame.stroke(
    //                    &outline,
    //                    Stroke::default()
    //                        .with_width(Self::OUTLINES_THICKNESS)
    //                        .with_color(outlines_color),
    //                );
    //            } else {
    //                if !self.clean {
    //                    let outline = Path::line([x, y].into(), [x, y_offset_top].into());
    //                    frame.stroke(
    //                        &outline,
    //                        Stroke::default()
    //                            .with_color(outlines_color)
    //                            .with_width(Self::OUTLINES_THICKNESS),
    //                    );
    //                }
    //            }
    //
    //            outlines_count += 1.0;
    //        }
    //
    //        if let Some(label) = &self.label {
    //            let y = y_padding_top + true_y_length;
    //            let x = (x_offset_length / 2.0) + x_offset_left + x_padding_left;
    //            let label_position = Point::new(x, y);
    //            let label_size = 16.0;
    //
    //            let text = Text {
    //                content: label.clone(),
    //                position: label_position,
    //                horizontal_alignment: Horizontal::Center,
    //                vertical_alignment: Vertical::Center,
    //                size: label_size.into(),
    //                color: label_color,
    //                ..Default::default()
    //            };
    //
    //            frame.fill_text(text);
    //        }
    //
    //        if let Some(caption) = &self.caption {
    //            let y = y_padding_top + true_y_length;
    //            let x = (x_offset_length * 0.80) + x_padding_left + x_offset_left;
    //            let caption_position = Point::new(x, y);
    //            let caption_size = 14.0;
    //
    //            let text = Text {
    //                content: caption.clone(),
    //                position: caption_position,
    //                horizontal_alignment: Horizontal::Center,
    //                vertical_alignment: Vertical::Center,
    //                size: caption_size.into(),
    //                color: label_color,
    //                font: font::Font {
    //                    style: font::Style::Italic,
    //                    ..Default::default()
    //                },
    //                ..Default::default()
    //            };
    //            frame.fill_text(text);
    //        }
    //    } else {
    //        let x = x_padding_left + x_offset_left;
    //        let y = y_padding_top;
    //        axis_pos = x;
    //
    //        let axis_start = Point::new(x, y);
    //        let axis_end = Point::new(x, y + true_y_length);
    //
    //        let line = Path::line(axis_start, axis_end);
    //        frame.stroke(
    //            &line,
    //            Stroke::default()
    //                .with_color(axis_color)
    //                .with_width(Self::AXIS_THICKNESS),
    //        );
    //
    //        let dy = y_offset_length / (self.points.len() as f32);
    //        let dty = y_top / (self.positives as f32);
    //        let dby = y_bottom / (self.points.len() - self.positives) as f32;
    //        let stump_width = 0.005 * height;
    //
    //        let outlines_number = if dy < 50.0 { 1 } else { 5 };
    //        let outlines_height = (dy * 0.9) / (outlines_number as f32);
    //        let mut outlines_count = 1.0;
    //        let mut point_count = 0;
    //    }
    //
    //    (axis_pos - (Self::AXIS_THICKNESS / 2.0), record)
    //}
}

pub struct Graph<'a, G, X = Data, Y = Data>
where
    X: Clone + Display + Hash + Eq + Debug,
    Y: Clone + Display + Hash + Eq + Debug,
    G: Graphable<X, Y>,
{
    x_axis: &'a Axis<X>,
    y_axis: &'a Axis<Y>,
    cache: &'a canvas::Cache,
    graphables: &'a [G],
    data: <G as Graphable<X, Y>>::Data,
    legend_position: LegendPosition,
    labels_len: usize,
}

impl<'a, X, Y, G> Graph<'a, G, X, Y>
where
    X: Clone + Display + Hash + Eq + Debug,
    Y: Clone + Display + Hash + Eq + Debug,
    G: Graphable<X, Y>,
{
    pub fn new(
        x_axis: &'a Axis<X>,
        y_axis: &'a Axis<Y>,
        graphables: &'a [G],
        cache: &'a canvas::Cache,
    ) -> Self {
        Self {
            x_axis,
            y_axis,
            cache,
            graphables,
            data: <G as Graphable<X, Y>>::Data::default(),
            legend_position: LegendPosition::default(),
            labels_len: 0,
        }
    }

    pub fn data(mut self, data: <G as Graphable<X, Y>>::Data) -> Self {
        self.data = data;
        self
    }

    pub fn legend(mut self, legend: LegendPosition) -> Self {
        self.legend_position = legend;
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

        for graphable in self.graphables.iter() {
            graphable.draw_legend(&mut frame, bounds, text_color, &self.data);
        }

        frame.into_geometry()
    }
}

impl<'a, G, X, Y, Message> canvas::Program<Message> for Graph<'a, G, X, Y>
where
    X: Clone + Display + Hash + Eq + Debug,
    Y: Clone + Display + Hash + Eq + Debug,
    G: Graphable<X, Y>,
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
            let (x_axis, x_record) = self.x_axis.draw(frame, theme);
            let (y_axis, y_record) = self.y_axis.draw(frame, theme);

            self.graphables.iter().for_each(|graphable| {
                graphable.draw(frame, &x_record, x_axis, &y_record, y_axis, &self.data);
            });

            if let Some(label) = self.y_axis.label.clone() {
                let data = AxisData::new(frame, theme);
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
