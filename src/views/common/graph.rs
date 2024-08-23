/// Text always being overlayed is an Iced issue. Keep eye out for fix
use std::{
    collections::HashMap,
    fmt::{self, Debug, Display},
    hash::Hash,
    rc::Rc,
};

use iced::{
    alignment::{self, Horizontal, Vertical},
    color, mouse, theme,
    widget::{
        self, button,
        canvas::{self, Canvas, Frame, Geometry, Path, Stroke, Text},
        column, component, container, overlay, row, text, Component, Tooltip,
    },
    Alignment, Background, Border, Color, Element, Font, Length, Point, Rectangle, Renderer, Size,
    Theme,
};

pub use modav_core::models::line::Point as GraphPoint;

use crate::widgets::toolbar::{ToolbarMenu, ToolbarOption};
use crate::{utils::icons, ToolTipContainerStyle};

#[allow(dead_code)]
const WHITE: Color = color!(255, 255, 255);
#[allow(dead_code)]
const BLACK: Color = color!(0, 0, 0);
#[allow(dead_code)]
const BLUE: Color = color!(0, 0, 255);
#[allow(dead_code)]
const RED: Color = color!(255, 0, 0);
#[allow(dead_code)]
const GREEN: Color = color!(0, 255, 0);
#[allow(dead_code)]
const MAGENTA: Color = color!(205, 0, 150);

#[derive(Debug, Clone)]
pub struct Axis<T>
where
    T: Clone + Display + Hash + Eq,
{
    points: Vec<T>,
    label: Option<String>,
}

impl<T> Axis<T>
where
    T: Clone + Display + Hash + Eq,
{
    const AXIS_THICKNESS: f32 = 2.0;
    const OUTLINES_THICKNESS: f32 = 0.5;

    pub fn new(label: Option<String>, points: Vec<T>) -> Self {
        Self { label, points }
    }

    pub fn set_label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }

    #[allow(unused_variables)]
    fn draw(&self, frame: &mut Frame, is_x_axis: bool, theme: &Theme) -> HashMap<T, f32> {
        let mut record = HashMap::new();

        let axis_color = color!(205, 0, 150);
        let label_color = theme.palette().primary;
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

            let dx = x_offset_length / (self.points.len() as f32);
            let stump_height = 0.01 * height;

            let outlines_number = if dx < 50.0 {
                1
            } else if dx < 250.0 {
                5
            } else {
                10
            };
            let outlines_width = dx / ((outlines_number) as f32);
            let mut outlines_count = 1;

            let mut points = self.points.iter();

            while (outlines_width * (outlines_count as f32)) <= x_offset_length {
                let x = x + outlines_width * (outlines_count as f32);
                // This is a point outline
                if outlines_count % outlines_number == 0 {
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

                    frame.stroke(
                        &outlines,
                        Stroke::default()
                            .with_width(Self::OUTLINES_THICKNESS)
                            .with_color(outlines_color),
                    );
                } else {
                    let outlines = Path::line([x, y].into(), [x, y_padding_top].into());

                    frame.stroke(
                        &outlines,
                        Stroke::default()
                            .with_width(Self::OUTLINES_THICKNESS)
                            .with_color(outlines_color),
                    );
                }

                outlines_count += 1;
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

            if let Some(label) = &self.label {
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

            let dy = y_offset_length / (self.points.len() as f32);
            let stump_length = 0.01 * height;

            let outlines_number = if dy < 50.0 { 1 } else { 5 };
            let outlines_height = dy / ((outlines_number) as f32);
            let mut outlines_count = 1;

            let mut points = self.points.iter();

            while (outlines_height * (outlines_count as f32)) <= y_offset_length {
                let y = y - outlines_height * (outlines_count as f32);
                // This is a point outline
                if outlines_count % outlines_number == 0 {
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

                    frame.stroke(
                        &outlines,
                        Stroke::default()
                            .with_width(Self::OUTLINES_THICKNESS)
                            .with_color(outlines_color),
                    );
                } else {
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
                }

                outlines_count += 1;
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

        return record;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphLine<X, Y>
where
    X: Clone + Display + Hash + Eq + Debug,
    Y: Clone + Display + Hash + Eq + Debug,
{
    points: Vec<GraphPoint<X, Y>>,
    label: Option<String>,
    color: Color,
}

impl<X, Y> GraphLine<X, Y>
where
    X: Clone + Display + Hash + Eq + Debug,
    Y: Clone + Display + Hash + Eq + Debug,
{
    pub fn new(points: Vec<GraphPoint<X, Y>>, label: Option<String>, color: Color) -> Self {
        Self {
            points,
            color,
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
        kind: GraphType,
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

            match kind {
                GraphType::Point => {
                    let path = Path::circle(point.clone(), 4.5);

                    frame.fill(&path, line.color);
                }

                GraphType::Line => {
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
                }

                GraphType::LinePoint => {
                    let path = Path::circle(point.clone(), 3.5);

                    frame.fill(&path, line.color);

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
                }
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

#[derive(Debug, Clone, Copy)]
pub enum GraphMessage {
    Legend(LegendPosition),
    GraphType(GraphType),
    OpenEditor,
}

#[derive(Debug, Default, Clone)]
pub struct GraphState {
    legend_position: LegendPosition,
    graph_type: GraphType,
}

#[derive(Debug)]
pub struct Graph<'a, Message, X, Y>
where
    X: Clone + Display + Hash + Eq + Debug,
    Y: Clone + Display + Hash + Eq + Debug,
{
    x_axis: Axis<X>,
    y_axis: Axis<Y>,
    lines: &'a Vec<GraphLine<X, Y>>,
    cache: canvas::Cache,
    on_open_editor: Option<Message>,
}

impl<'a, Message, X, Y> Graph<'a, Message, X, Y>
where
    X: Clone + Display + Hash + Eq + Debug,
    Y: Clone + Display + Hash + Eq + Debug,
{
    pub fn new(x_axis: Axis<X>, y_axis: Axis<Y>, lines: &'a Vec<GraphLine<X, Y>>) -> Self {
        Self {
            x_axis,
            y_axis,
            lines,
            cache: canvas::Cache::default(),
            on_open_editor: None,
        }
    }

    fn toolbar(
        &self,
        legend: LegendPosition,
        kind: GraphType,
    ) -> Element<'_, GraphMessage, Theme, Renderer> {
        let style = ToolbarStyle;
        let menu_style = ToolbarMenuStyle;

        let legend = {
            let icons = Font::with_name("legend-icons");

            let menu = ToolbarMenu::new(LegendPosition::ALL, legend, GraphMessage::Legend, icons)
                .padding([4, 4])
                .menu_padding([4, 10, 4, 8])
                .spacing(5.0)
                .menu_style(theme::Menu::Custom(Rc::new(menu_style)))
                .style(theme::PickList::Custom(Rc::new(style), Rc::new(menu_style)));

            let tooltip = container(text("Legend Position").size(12.0))
                .max_width(200.0)
                .padding([6, 8])
                .style(theme::Container::Custom(Box::new(ToolTipContainerStyle)))
                .height(Length::Shrink);

            let menu = Tooltip::new(menu, tooltip, iced::widget::tooltip::Position::Bottom)
                .gap(2.0)
                .snap_within_viewport(true);

            menu
        };

        let kind = {
            let icons = Font::with_name("line-type-icons");

            let menu = ToolbarMenu::new(GraphType::ALL, kind, GraphMessage::GraphType, icons)
                .padding([4, 4])
                .menu_padding([4, 10, 4, 8])
                .spacing(17.0)
                .menu_style(theme::Menu::Custom(Rc::new(menu_style)))
                .style(theme::PickList::Custom(Rc::new(style), Rc::new(menu_style)));

            let tooltip = container(text("Graph Type").size(12.0))
                .max_width(200.0)
                .padding([6, 8])
                .style(theme::Container::Custom(Box::new(ToolTipContainerStyle)))
                .height(Length::Shrink);

            let menu = Tooltip::new(menu, tooltip, iced::widget::tooltip::Position::Bottom)
                .gap(2.0)
                .snap_within_viewport(true);

            menu
        };

        let editor = {
            let font = Font::with_name(icons::NAME);

            let btn = button(
                text(icons::EDITOR)
                    .font(font)
                    .width(18.0)
                    .vertical_alignment(alignment::Vertical::Center)
                    .horizontal_alignment(alignment::Horizontal::Center),
            )
            .on_press(GraphMessage::OpenEditor)
            .style(theme::Button::Custom(Box::new(EditorButtonStyle)))
            .padding([4, 4]);

            let tooltip = container(text("Open in Editor").size(12.0))
                .max_width(200.0)
                .padding([6, 8])
                .style(theme::Container::Custom(Box::new(ToolTipContainerStyle)))
                .height(Length::Shrink);

            let menu = Tooltip::new(btn, tooltip, iced::widget::tooltip::Position::Bottom)
                .gap(2.0)
                .snap_within_viewport(true);

            menu
        };

        container(
            column!(legend, kind, editor)
                .width(Length::Fill)
                .align_items(Alignment::Center)
                .spacing(8.0),
        )
        .width(Length::Fixed(40.0))
        .padding([6.0, 2.0])
        .style(theme::Container::Custom(Box::new(ToolbarContainerStyle)))
        .into()
    }

    pub fn on_editor(mut self, message: Message) -> Self {
        self.on_open_editor = Some(message);
        self
    }
}

impl<'a, Message, X, Y> Component<Message> for Graph<'a, Message, X, Y>
where
    X: Clone + Display + Hash + Eq + Debug,
    Y: Clone + Display + Hash + Eq + Debug,
    Message: Clone,
{
    type Event = GraphMessage;
    type State = GraphState;

    fn update(&mut self, state: &mut Self::State, event: Self::Event) -> Option<Message> {
        match event {
            GraphMessage::Legend(position) => {
                state.legend_position = position;
                None
            }
            GraphMessage::GraphType(kind) => {
                state.graph_type = kind;
                None
            }
            GraphMessage::OpenEditor => self.on_open_editor.clone(),
        }
    }

    fn view(&self, state: &Self::State) -> Element<'_, Self::Event, Theme, Renderer> {
        let canvas = Canvas::new(
            GraphCanvas::new(&self.x_axis, &self.y_axis, &self.lines, &self.cache)
                .legend_position(state.legend_position)
                .graph_type(state.graph_type),
        )
        .height(Length::Fill)
        .width(Length::FillPortion(24));

        let toolbar = self.toolbar(
            if self.lines.iter().any(|line| line.label.is_some()) {
                state.legend_position
            } else {
                LegendPosition::None
            },
            state.graph_type,
        );

        row!(canvas, toolbar).into()
    }
}

impl<'a, Message, X, Y> From<Graph<'a, Message, X, Y>> for Element<'a, Message>
where
    Message: 'a + Clone + Debug,
    X: 'a + Clone + Display + Hash + Eq + Debug,
    Y: 'a + Clone + Display + Hash + Eq + Debug,
{
    fn from(value: Graph<'a, Message, X, Y>) -> Self {
        component(value)
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
    const ALL: [Self; 10] = [
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

#[derive(Debug, Clone, Default, Copy, PartialEq)]
pub enum GraphType {
    Line,
    Point,
    #[default]
    LinePoint,
}

impl GraphType {
    const ALL: [Self; 3] = [Self::LinePoint, Self::Line, Self::Point];
}

impl fmt::Display for GraphType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Line => "Line Graph",
                Self::Point => "Points Graph",
                Self::LinePoint => "Line Graph with Points",
            }
        )
    }
}

impl ToolbarOption for GraphType {
    fn icon(&self) -> char {
        match self {
            Self::Line => '\u{E800}',
            Self::Point => '\u{E801}',
            Self::LinePoint => '\u{E802}',
        }
    }
}

#[derive(Debug)]
pub struct GraphCanvas<'a, X, Y>
where
    X: Clone + Display + Hash + Eq + Debug,
    Y: Clone + Display + Hash + Eq + Debug,
{
    x_axis: &'a Axis<X>,
    y_axis: &'a Axis<Y>,
    lines: &'a Vec<GraphLine<X, Y>>,
    cache: &'a canvas::Cache,
    legend: LegendPosition,
    graph_type: GraphType,
}

impl<'a, X, Y> GraphCanvas<'a, X, Y>
where
    X: Clone + Display + Hash + Eq + Debug,
    Y: Clone + Display + Hash + Eq + Debug,
{
    fn new(
        x_axis: &'a Axis<X>,
        y_axis: &'a Axis<Y>,
        lines: &'a Vec<GraphLine<X, Y>>,
        cache: &'a canvas::Cache,
    ) -> Self {
        Self {
            x_axis,
            y_axis,
            lines,
            cache,
            legend: LegendPosition::default(),
            graph_type: GraphType::default(),
        }
    }

    fn legend_position(mut self, position: LegendPosition) -> Self {
        self.legend = position;
        self
    }

    fn graph_type(mut self, kind: GraphType) -> Self {
        self.graph_type = kind;
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
            .lines
            .iter()
            .map(|line| line.label.is_some())
            .filter(|has_label| *has_label)
            .count();

        if !self.lines.iter().any(|line| line.label.is_some()) {
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

        for (i, line) in self
            .lines
            .iter()
            .filter(|line| line.label.is_some())
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

            line.draw_legend(&mut frame, start_point, size, text_color)
        }

        frame.into_geometry()
    }
}

impl<'a, X, Y> canvas::Program<GraphMessage> for GraphCanvas<'a, X, Y>
where
    X: Clone + Display + Hash + Eq + Debug,
    Y: Clone + Display + Hash + Eq + Debug,
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
            let x_record = Axis::draw(&self.x_axis, frame, true, theme);

            let y_record = Axis::draw(&self.y_axis, frame, false, theme);

            self.lines.iter().for_each(|line| {
                GraphLine::draw(line, frame, &x_record, &y_record, self.graph_type)
            });
        });

        vec![content, self.legend(renderer, bounds, self.legend, theme)]
    }
}

#[derive(Clone, Copy, Debug)]
struct ToolbarContainerStyle;

impl widget::container::StyleSheet for ToolbarContainerStyle {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        let pallete = style.extended_palette();

        let background = Background::Color(pallete.background.weak.color);

        let border = Border {
            color: pallete.primary.base.color,
            width: 0.5,
            radius: 5.0.into(),
        };

        container::Appearance {
            background: Some(background),
            border,
            ..Default::default()
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct ToolbarStyle;

impl widget::pick_list::StyleSheet for ToolbarStyle {
    type Style = Theme;

    fn active(
        &self,
        style: &<Self as widget::pick_list::StyleSheet>::Style,
    ) -> widget::pick_list::Appearance {
        let pallete = style.extended_palette();
        let text_color = pallete.background.base.text;
        let background = Background::Color(pallete.background.weak.color);
        let border = Border {
            color: pallete.background.weak.color,
            width: 0.25,
            radius: 3.0.into(),
        };

        widget::pick_list::Appearance {
            text_color,
            placeholder_color: text_color,
            handle_color: text_color,
            background,
            border,
        }
    }

    fn hovered(
        &self,
        style: &<Self as widget::pick_list::StyleSheet>::Style,
    ) -> widget::pick_list::Appearance {
        let pallete = style.extended_palette();
        let text_color = pallete.primary.strong.color;
        let background = Background::Color(pallete.background.weak.color);
        let border = Border {
            color: pallete.primary.strong.color,
            width: 0.5,
            radius: 3.0.into(),
        };

        widget::pick_list::Appearance {
            text_color,
            placeholder_color: text_color,
            handle_color: text_color,
            background,
            border,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct ToolbarMenuStyle;

impl overlay::menu::StyleSheet for ToolbarMenuStyle {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> overlay::menu::Appearance {
        let pallete = style.extended_palette();

        let text_color = pallete.background.base.text;
        let background = Background::Color(pallete.background.weak.color);
        let border = Border {
            width: 1.0,
            radius: 3.5.into(),
            color: pallete.background.strong.color,
        };
        let selected_background = Background::Color(pallete.primary.base.color);

        overlay::menu::Appearance {
            text_color,
            selected_text_color: pallete.primary.base.text,
            background,
            border,
            selected_background,
        }
    }
}

struct EditorButtonStyle;
impl widget::button::StyleSheet for EditorButtonStyle {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        let palette = style.extended_palette();

        button::Appearance {
            text_color: palette.background.base.text,
            background: Some(Background::Color(palette.background.weak.color)),
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let pallete = style.extended_palette();
        let text_color = pallete.primary.strong.color;
        let background = Background::Color(pallete.background.weak.color);
        let border = Border {
            color: pallete.primary.strong.color,
            width: 0.5,
            radius: 3.0.into(),
        };

        button::Appearance {
            text_color,
            border,
            background: Some(background),
            ..Default::default()
        }
    }
}
