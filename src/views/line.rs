use std::{
    collections::HashMap,
    fmt::{self, Debug, Display},
    hash::Hash,
    path::PathBuf,
    rc::Rc,
};
use tracing::warn;

use iced::{
    alignment, mouse, theme,
    widget::{
        button,
        canvas::{self, Canvas, Frame, Path, Stroke},
        column, component, container, horizontal_space, row, text, Component, Tooltip,
    },
    Alignment, Color, Element, Font, Length, Point, Renderer, Theme,
};

use modav_core::{
    models::{
        line::{self, Line},
        Point as GraphPoint, Scale,
    },
    repr::sheet::{builders::SheetBuilder, utils::Data},
};

use crate::{
    utils::{coloring::ColorEngine, icons, AppError},
    widgets::{
        toolbar::{ToolbarMenu, ToolbarOption},
        wizard::LineConfigState,
    },
    Message, ToolTipContainerStyle,
};

use super::{shared::ContentAreaContainer, TabLabel, Viewable};

use super::shared::{
    Axis, EditorButtonStyle, GraphCanvas, Graphable, LegendPosition, ToolbarContainerStyle,
    ToolbarMenuStyle, ToolbarStyle,
};

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
                warn!("X Point {:?} not found", point.x);
                return prev;
            }

            let y = y_record
                .get(&point.y)
                .and_then(|x| Some(x.to_owned()))
                .unwrap_or(-1.0);

            if y < 0.0 {
                warn!("Y Point {:?} not found", point.y);
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
}

impl<X, Y> Graphable<X, Y> for GraphLine<X, Y>
where
    X: Clone + Display + Hash + Eq + Debug,
    Y: Clone + Display + Hash + Eq + Debug,
{
    type Data = GraphType;

    fn draw(
        &self,
        frame: &mut Frame,
        _cursor: mouse::Cursor,
        x_points: &HashMap<X, f32>,
        _x_axis: f32,
        y_points: &HashMap<Y, f32>,
        _y_axis: f32,
        data: &Self::Data,
    ) {
        Self::draw(self, frame, x_points, y_points, *data)
    }

    fn label(&self) -> Option<&String> {
        self.label.as_ref()
    }

    fn color(&self) -> Color {
        self.color
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
pub struct LineGraph<'a, Message, X, Y>
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

impl<'a, Message, X, Y> LineGraph<'a, Message, X, Y>
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

impl<'a, Message, X, Y> Component<Message> for LineGraph<'a, Message, X, Y>
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
            GraphCanvas::<GraphLine<X, Y>, X, Y>::new(
                &self.x_axis,
                &self.y_axis,
                &self.lines,
                &self.cache,
            )
            .legend_position(state.legend_position)
            .graph_data(state.graph_type),
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

impl<'a, Message, X, Y> From<LineGraph<'a, Message, X, Y>> for Element<'a, Message>
where
    Message: 'a + Clone + Debug,
    X: 'a + Clone + Display + Hash + Eq + Debug,
    Y: 'a + Clone + Display + Hash + Eq + Debug,
{
    fn from(value: LineGraph<'a, Message, X, Y>) -> Self {
        component(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineTabData {
    file: PathBuf,
    title: String,
    theme: Theme,
    line: line::LineGraph<String, Data>,
    caption: Option<String>,
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
            caption,
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
            caption,
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
    caption: Option<String>,
}

impl LineGraphTab {
    fn graph(&self) -> LineGraph<ModelMessage, String, Data> {
        let mut x_axis = Axis::new(self.x_label.clone(), self.x_scale.points().clone(), false);

        if let Some(caption) = self.caption.clone() {
            x_axis = x_axis.caption(caption);
        }

        let y_axis = Axis::new(self.y_label.clone(), self.y_scale.points().clone(), false);

        LineGraph::new(x_axis, y_axis, &self.lines).on_editor(ModelMessage::OpenEditor)
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
            caption,
        } = data;

        let line::LineGraph {
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
            caption,
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

        TabLabel::new(icons::CHART, format!("{} - {}", self.title, file_name)).icon_font(font)
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
            caption,
        } = data;

        let line::LineGraph {
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
        self.caption = caption;
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
