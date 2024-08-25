#![allow(unused_imports, dead_code)]
use std::{collections::HashMap, fmt::Debug, path::PathBuf, rc::Rc};

use iced::{
    alignment, theme,
    widget::{
        button, canvas, column, component, container, horizontal_space, row, text, Canvas,
        Component, Tooltip,
    },
    Alignment, Color, Element, Font, Length, Point, Renderer, Size, Theme,
};

use modav_core::{
    models::{
        bar::{Bar, BarChart},
        Point as GraphPoint, Scale,
    },
    repr::sheet::{builders::SheetBuilder, utils::Data},
};
use tracing::warn;

use crate::{
    utils::{coloring::ColorEngine, icons, AppError},
    widgets::{toolbar::ToolbarMenu, wizard::BarChartConfigState},
    Message, ToolTipContainerStyle,
};

use super::{
    shared::{
        Axis, ContentAreaContainer, EditorButtonStyle, GraphCanvas, Graphable, LegendPosition,
        ToolbarContainerStyle, ToolbarMenuStyle, ToolbarStyle,
    },
    tabs::TabLabel,
    Viewable,
};

const DEFAULT_WIDTH: f32 = 50.0;

#[derive(Debug, Clone, PartialEq)]
pub struct GraphBar {
    point: GraphPoint<Data, Data>,
    label: Option<String>,
    color: Color,
}

impl GraphBar {
    fn new(point: GraphPoint<Data, Data>, label: Option<String>, color: Color) -> Self {
        Self {
            point,
            label,
            color,
        }
    }

    fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

impl From<Bar<Data, Data>> for GraphBar {
    fn from(value: Bar<Data, Data>) -> Self {
        let Bar { point, label } = value;

        Self::new(point, label, Color::BLACK)
    }
}

impl Graphable<Data, Data> for GraphBar {
    type Data = ();

    fn label(&self) -> Option<&String> {
        self.label.as_ref()
    }

    fn color(&self) -> Color {
        self.color
    }

    fn draw(
        &self,
        frame: &mut canvas::Frame,
        _cursor: iced::mouse::Cursor,
        x_points: &HashMap<Data, f32>,
        x_axis: f32,
        y_points: &HashMap<Data, f32>,
        _y_axis: f32,
        _data: &Self::Data,
    ) {
        let width = {
            let (x, y) = x_points
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

        let x = *x_points.get(&self.point.x).unwrap_or(&-1.0);

        if x < 0.0 {
            warn!("Bar char x point, {} not found", &self.point.x);
            return;
        }

        let y = *y_points.get(&self.point.y).unwrap_or(&-1.0);

        if y < 0.0 {
            warn!("Bar char x point, {} not found", &self.point.y);
            return;
        }

        let top_left = Point::new(x - (width / 2.0), y);
        let size = Size::new(width, x_axis - y);

        frame.fill_rectangle(top_left, size, self.color);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BarChartGraphMessage {
    Legend(LegendPosition),
    OpenEditor,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct BarChartGraphState {
    legend: LegendPosition,
}

pub struct BarChartGraph<'a, Message> {
    x_axis: Axis<Data>,
    y_axis: Axis<Data>,
    bars: &'a Vec<GraphBar>,
    cache: canvas::Cache,
    on_open_editor: Option<Message>,
}

impl<'a, Message> BarChartGraph<'a, Message> {
    fn new(x_axis: Axis<Data>, y_axis: Axis<Data>, bars: &'a Vec<GraphBar>) -> Self {
        Self {
            x_axis,
            y_axis,
            bars,
            cache: canvas::Cache::default(),
            on_open_editor: None,
        }
    }

    fn toolbar(
        &self,
        legend: LegendPosition,
    ) -> Element<'_, BarChartGraphMessage, Theme, Renderer> {
        let style = ToolbarStyle;
        let menu_style = ToolbarMenuStyle;

        let legend = {
            let icons = Font::with_name("legend-icons");

            let menu = ToolbarMenu::new(
                LegendPosition::ALL,
                legend,
                BarChartGraphMessage::Legend,
                icons,
            )
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

        let editor = {
            let font = Font::with_name(icons::NAME);

            let btn = button(
                text(icons::EDITOR)
                    .font(font)
                    .width(18.0)
                    .vertical_alignment(alignment::Vertical::Center)
                    .horizontal_alignment(alignment::Horizontal::Center),
            )
            .on_press(BarChartGraphMessage::OpenEditor)
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
            column!(legend, editor)
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

impl<'a, Message> Component<Message> for BarChartGraph<'a, Message>
where
    Message: Debug + Clone,
{
    type State = BarChartGraphState;
    type Event = BarChartGraphMessage;

    fn update(&mut self, state: &mut Self::State, event: Self::Event) -> Option<Message> {
        match event {
            BarChartGraphMessage::OpenEditor => self.on_open_editor.clone(),
            BarChartGraphMessage::Legend(legend) => {
                state.legend = legend;
                None
            }
        }
    }

    fn view(&self, state: &Self::State) -> Element<'_, Self::Event, Theme, Renderer> {
        let canvas = Canvas::new(
            GraphCanvas::<GraphBar, Data, Data>::new(
                &self.x_axis,
                &self.y_axis,
                &self.bars,
                &self.cache,
            )
            .legend_position(state.legend),
        )
        .width(Length::FillPortion(24))
        .height(Length::Fill);

        let toolbar = self.toolbar(if self.bars.iter().any(|bar| bar.label.is_some()) {
            state.legend
        } else {
            LegendPosition::None
        });

        row!(canvas, toolbar).into()
    }
}

impl<'a, Message> From<BarChartGraph<'a, Message>> for Element<'a, Message>
where
    Message: 'a + Debug + Clone,
{
    fn from(value: BarChartGraph<'a, Message>) -> Self {
        component(value)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BarChartTabData {
    file: PathBuf,
    title: String,
    barchart: BarChart<Data, Data>,
    theme: Theme,
}

impl BarChartTabData {
    pub fn new(file: PathBuf, config: BarChartConfigState) -> Result<Self, AppError> {
        let BarChartConfigState {
            title,
            trim,
            flexible,
            header_types,
            header_labels,
            row_exclude,
            bar_label,
            axis_label,
            x_col,
            y_col,
        } = config;

        let sht = SheetBuilder::new(file.clone().into())
            .trim(trim)
            .flexible(flexible)
            .labels(header_labels)
            .types(header_types)
            .build()
            .map_err(AppError::CSVError)?;

        let barchart = sht
            .create_bar_chart(x_col, y_col, bar_label, axis_label, row_exclude)
            .map_err(AppError::CSVError)?;

        Ok(Self {
            file,
            title,
            barchart,
            theme: Theme::default(),
        })
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BarChartMessage {
    OpenEditor,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BarChartTab {
    file: PathBuf,
    title: String,
    x_scale: Scale<Data>,
    y_scale: Scale<Data>,
    x_label: Option<String>,
    y_label: Option<String>,
    bars: Vec<GraphBar>,
}

impl BarChartTab {
    fn graph(&self) -> BarChartGraph<'_, BarChartMessage> {
        let x_axis = Axis::new(self.x_label.clone(), self.x_scale.points().clone());

        let y_axis = Axis::new(self.y_label.clone(), self.y_scale.points().clone());

        BarChartGraph::new(x_axis, y_axis, &self.bars).on_editor(BarChartMessage::OpenEditor)
    }
}

impl Viewable for BarChartTab {
    type Event = BarChartMessage;
    type Data = BarChartTabData;

    fn new(data: Self::Data) -> Self {
        let BarChartTabData {
            file,
            title,
            barchart,
            theme,
        } = data;

        let BarChart {
            x_label,
            x_scale,
            y_label,
            y_scale,
            bars,
        } = barchart;

        let colors = ColorEngine::new(&theme);

        let bars = bars
            .into_iter()
            .zip(colors)
            .map(|(bar, color)| Into::<GraphBar>::into(bar).color(color))
            .collect();

        Self {
            file,
            title,
            x_scale,
            x_label,
            y_scale,
            y_label,
            bars,
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
            BarChartMessage::OpenEditor => Some(Message::OpenEditor(Some(self.file.clone()))),
        }
    }

    fn view<'a, Message, F>(&'a self, map: F) -> Element<'a, Message, Theme, Renderer>
    where
        F: 'a + Fn(Self::Event) -> Message,
        Message: 'a + Clone + Debug,
    {
        let title = {
            let text = text(format!("{} - Bar Chart", self.title));
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
