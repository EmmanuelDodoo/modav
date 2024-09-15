/// Text always being overlayed is an Iced issue. Keep eye out for fix
use std::{
    collections::HashMap,
    fmt::{self, Debug, Display},
    hash::Hash,
};

use iced::{
    alignment::{self, Horizontal, Vertical},
    color, font, mouse,
    widget::canvas::{self, Frame, Geometry, Path, Stroke, Text},
    Color, Point, Rectangle, Renderer, Size, Theme,
};

use crate::widgets::toolbar::ToolbarOption;

const Y_LABEL_ROTATE_WIDTH: f32 = 16.0;
pub trait Graphable<X, Y> {
    type Data: Default + Debug;

    fn label(&self) -> Option<&String>;

    fn color(&self) -> Color;

    fn draw_legend(&self, frame: &mut Frame, position: Point, size: Size, color: Color) {
        let x = position.x;
        let y = position.y;

        let width = size.width;
        let height = size.height;

        frame.fill(
            &Path::rectangle([x, y].into(), Size::new(width, height)),
            self.color(),
        );

        let label = Text {
            content: self.label().cloned().unwrap_or(String::default()),
            position: Point::new(x + 1.25 * width, y + 0.5 * height),
            color,
            size: 12.0.into(),
            vertical_alignment: alignment::Vertical::Center,
            ..Default::default()
        };

        frame.fill_text(label);
    }

    fn draw(
        &self,
        frame: &mut Frame,
        cursor: mouse::Cursor,
        x_points: &HashMap<X, f32>,
        x_axis: f32,
        y_points: &HashMap<Y, f32>,
        y_axis: f32,
        data: &Self::Data,
    );
}

#[derive(Debug, Clone)]
pub struct Axis<T>
where
    T: Clone + Display + Hash + Eq,
{
    points: Vec<T>,
    label: Option<String>,
    clean: bool,
    caption: Option<String>,
}

impl<T> Axis<T>
where
    T: Clone + Display + Hash + Eq,
{
    const AXIS_THICKNESS: f32 = 2.0;
    const OUTLINES_THICKNESS: f32 = 0.5;

    pub fn new(label: Option<String>, points: Vec<T>, clean: bool) -> Self {
        Self {
            label,
            points,
            clean,
            caption: None,
        }
    }

    pub fn caption(mut self, caption: String) -> Self {
        self.caption = Some(caption);
        self
    }

    pub fn clean(&mut self, clean: bool) {
        self.clean = clean;
    }

    pub fn set_label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }

    #[allow(unused_variables, unused_assignments)]
    fn draw(&self, frame: &mut Frame, is_x_axis: bool, theme: &Theme) -> (f32, HashMap<T, f32>) {
        let mut record = HashMap::new();

        let axis_color = color!(205, 0, 150);
        let label_color = theme.extended_palette().secondary.strong.text;
        let text_color = theme.palette().text;
        let outlines_color = theme.extended_palette().background.weak.color;

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

        let mut axis = 0.0;

        if is_x_axis {
            let x = x_padding_left;
            let y = y_padding_top + y_offset_length + y_offset_top;
            axis = y;

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

            let dx = x_offset_length / (self.points.len() as f32);
            let stump_height = 0.01 * height;

            let outlines_number = if dx < 50.0 {
                1
            } else if dx < 250.0 {
                5
            } else {
                10
            };
            let outlines_width = (dx * 0.85) / ((outlines_number) as f32);
            let mut outlines_count = 1;
            let mut point_count = 0;

            let mut points = self.points.iter();

            while (outlines_width * (outlines_count as f32)) <= x_offset_length {
                let x = x + outlines_width * (outlines_count as f32);
                // This is a point outline
                if outlines_count % outlines_number == 0 && point_count < self.points.len() {
                    point_count += 1;
                    let path = Path::line([x, y].into(), [x, y + stump_height].into());
                    frame.stroke(
                        &path,
                        Stroke::default()
                            .with_width(Self::AXIS_THICKNESS)
                            .with_color(axis_color),
                    );

                    if let Some(point) = points.next() {
                        record.insert(point.clone(), x);

                        let text_position = Point {
                            x,
                            y: y + stump_height,
                        };
                        let text = Text {
                            content: point.clone().to_string(),
                            position: text_position,
                            horizontal_alignment: Horizontal::Center,
                            color: text_color,

                            ..Default::default()
                        };

                        frame.fill_text(text);
                    }

                    let outlines = Path::line([x, y].into(), [x, y_padding_top].into());

                    if !self.clean {
                        frame.stroke(
                            &outlines,
                            Stroke::default()
                                .with_width(Self::OUTLINES_THICKNESS)
                                .with_color(outlines_color),
                        );
                    }
                } else {
                    let outlines = Path::line([x, y].into(), [x, y_padding_top].into());

                    if !self.clean {
                        frame.stroke(
                            &outlines,
                            Stroke::default()
                                .with_width(Self::OUTLINES_THICKNESS)
                                .with_color(outlines_color),
                        );
                    }
                }

                outlines_count += 1;
            }

            if let Some(label) = &self.label {
                let y = y_padding_top + true_y_length;
                let x = (x_offset_length / 2.0) + x_padding_left + x_offset_left;
                let label_position = Point::new(x, y);
                let label_size = 16.0;

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

            if let Some(caption) = &self.caption {
                let y = y_padding_top + true_y_length;
                let x = (x_offset_length * 0.80) + x_padding_left + x_offset_left;
                let caption_position = Point::new(x, y);
                let caption_size = 14.0;

                let text = Text {
                    content: caption.clone(),
                    position: caption_position,
                    horizontal_alignment: Horizontal::Center,
                    vertical_alignment: Vertical::Center,
                    size: caption_size.into(),
                    color: label_color,
                    font: font::Font {
                        style: font::Style::Italic,
                        ..Default::default()
                    },
                    ..Default::default()
                };

                frame.fill_text(text);
            }
        } else {
            let x = x_padding_left + x_offset_left;
            axis = x;
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

            let dy = y_offset_length / (self.points.len() as f32);
            let stump_length = 0.01 * height;

            let outlines_number = if dy < 50.0 { 1 } else { 5 };
            let outlines_height = (dy * 0.9) / ((outlines_number) as f32);
            let mut outlines_count = 1;
            let mut point_count = 0;

            let mut points = self.points.iter();

            while (outlines_height * (outlines_count as f32)) <= y_offset_length {
                let y = y - outlines_height * (outlines_count as f32);
                // This is a point outline
                if outlines_count % outlines_number == 0 && point_count < self.points.len() {
                    point_count += 1;
                    let path = Path::line([x, y].into(), [x - stump_length, y].into());
                    frame.stroke(
                        &path,
                        Stroke::default()
                            .with_width(Self::AXIS_THICKNESS)
                            .with_color(axis_color),
                    );

                    if let Some(point) = points.next() {
                        record.insert(point.clone(), y);

                        let text_position = Point {
                            x: (x * 0.95) - stump_length,
                            y,
                        };

                        let text = Text {
                            content: point.clone().to_string(),
                            position: text_position,
                            vertical_alignment: Vertical::Center,
                            horizontal_alignment: Horizontal::Right,
                            color: text_color,
                            ..Default::default()
                        };

                        frame.fill_text(text);
                    }

                    let outlines = Path::line(
                        [x, y].into(),
                        [x + x_offset_length + x_offset_right, y].into(),
                    );

                    if !self.clean {
                        frame.stroke(
                            &outlines,
                            Stroke::default()
                                .with_width(Self::OUTLINES_THICKNESS)
                                .with_color(outlines_color),
                        );
                    }
                } else {
                    let outlines = Path::line(
                        [x, y].into(),
                        [x + x_offset_length + x_offset_right, y].into(),
                    );

                    if !self.clean {
                        frame.stroke(
                            &outlines,
                            Stroke::default()
                                .with_width(Self::OUTLINES_THICKNESS)
                                .with_color(outlines_color),
                        );
                    }
                }

                outlines_count += 1;
            }

            if let Some(label) = &self.label {
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

        return (axis - (Self::AXIS_THICKNESS / 2.0), record);
    }
}

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

#[derive(Debug)]
pub struct GraphCanvas<'a, G, X, Y>
where
    X: Clone + Display + Hash + Eq + Debug,
    Y: Clone + Display + Hash + Eq + Debug,
    G: Graphable<X, Y>,
{
    x_axis: &'a Axis<X>,
    y_axis: &'a Axis<Y>,
    graphables: &'a Vec<G>,
    cache: &'a canvas::Cache,
    legend: LegendPosition,
    data: <G as Graphable<X, Y>>::Data,
}

impl<'a, G, X, Y> GraphCanvas<'a, G, X, Y>
where
    X: Clone + Display + Hash + Eq + Debug,
    Y: Clone + Display + Hash + Eq + Debug,
    G: Graphable<X, Y>,
{
    pub fn new(
        x_axis: &'a Axis<X>,
        y_axis: &'a Axis<Y>,
        graphables: &'a Vec<G>,
        cache: &'a canvas::Cache,
    ) -> Self {
        Self {
            x_axis,
            y_axis,
            graphables,
            cache,
            legend: LegendPosition::default(),
            data: <G as Graphable<X, Y>>::Data::default(),
        }
    }

    pub fn legend_position(mut self, position: LegendPosition) -> Self {
        self.legend = position;
        self
    }

    pub fn graph_data(mut self, data: <G as Graphable<X, Y>>::Data) -> Self {
        self.data = data;
        self
    }

    fn legend(
        &self,
        renderer: &Renderer,
        bounds: Rectangle,
        position: LegendPosition,
        theme: &Theme,
    ) -> Geometry {
        let mut frame = Frame::new(renderer, bounds.size());

        if position == LegendPosition::None {
            return frame.into_geometry();
        }

        let labels_len = self
            .graphables
            .iter()
            .map(|graphable| graphable.label().is_some())
            .filter(|has_label| *has_label)
            .count();

        if !self.graphables.iter().any(|line| line.label().is_some()) {
            return frame.into_geometry();
        }

        let labels_len = usize::min(labels_len, 7);

        let size = {
            let width = f32::min(frame.size().width * 0.125, 150.0);
            let height = 40.0 + 15.0 * (labels_len as f32);
            Size::new(width, height)
        };

        let position = position.position(bounds, size);

        let background = theme.extended_palette().background.weak.color;
        let text_color = theme.extended_palette().background.base.text;

        frame.stroke(
            &Path::rectangle(position, size),
            Stroke::default().with_width(1.5),
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

        for (i, graphable) in self
            .graphables
            .iter()
            .filter(|graphable| graphable.label().is_some())
            .enumerate()
        {
            if i > 5 {
                break;
            }

            let width = 12.0;
            let height = 12.0;

            let x = position.x + x_padding;
            let y = position.y + 25.0 + (i as f32) * (height + 5.0);

            let size = Size::new(width, height);

            let start_point = Point::new(x, y);

            graphable.draw_legend(&mut frame, start_point, size, text_color)
        }

        frame.into_geometry()
    }
}

impl<'a, G, X, Y, Message> canvas::Program<Message> for GraphCanvas<'a, G, X, Y>
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
        cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let content = self.cache.draw(renderer, bounds.size(), |frame| {
            let (x_axis, x_record) = Axis::draw(&self.x_axis, frame, true, theme);

            let (y_axis, y_record) = Axis::draw(&self.y_axis, frame, false, theme);

            self.graphables.iter().for_each(|graphable| {
                Graphable::draw(
                    graphable, frame, cursor, &x_record, x_axis, &y_record, y_axis, &self.data,
                )
            });
        });

        let _y_label_rotate = {
            let mut frame = Frame::new(renderer, Size::new(100.0, Y_LABEL_ROTATE_WIDTH));

            if let Some(label) = self.y_axis.label.clone() {
                let x_padding = bounds.width * 0.0375;
                let y_padding = {
                    let height = bounds.height;
                    let y_padding_top = 0.035 * height;
                    let true_y_length = height - (1.5 * y_padding_top);
                    let y_offset_bottom = 0.05 * true_y_length;
                    let y_offset_top = 0.5 * y_offset_bottom;
                    let y_offset_length = true_y_length - y_offset_bottom - y_offset_top;

                    (y_offset_length / 2.0) + y_padding_top + y_offset_top
                };
                let position = Point::new(Point::ORIGIN.x + x_padding, Point::ORIGIN.y + y_padding);

                let text = Text {
                    content: label,
                    position,
                    horizontal_alignment: Horizontal::Center,
                    vertical_alignment: Vertical::Center,
                    color: theme.palette().primary,
                    size: 16.0.into(),
                    ..Default::default()
                };

                frame.fill_text(text);
                frame.rotate(1.571);
            };

            frame.into_geometry()
        };

        vec![content, self.legend(renderer, bounds, self.legend, theme)]
    }
}
