use std::{collections::HashMap, fmt::Debug, path::PathBuf};

use iced::{
    alignment, theme,
    widget::{
        button, canvas, checkbox, column, container, horizontal_space, row, text, text_input,
        vertical_space, Canvas, Tooltip,
    },
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
    utils::{coloring::ColorEngine, icons, parse_ints, tooltip, AppError, Selection},
    widgets::{
        modal::Modal,
        style::dialog_container,
        toolbar::{ToolBarOrientation, ToolbarMenu},
        wizard::StackedBarChartConfigState,
    },
    Message, ToolTipContainerStyle,
};

use super::{
    shared::{
        graph::{create_axis, Axis, DrawnOutput, Graph, Graphable, LegendPosition},
        tools_button, ContentAreaContainer, EditorButtonStyle,
    },
    tabs::TabLabel,
    Viewable,
};

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

    fn draw_legend_filter(&self, data: &Self::Data) -> bool {
        data.0 == self.id
    }

    fn draw_legend(
        &self,
        frame: &mut canvas::Frame,
        bounds: iced::Rectangle,
        color: Color,
        _idx: usize,
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

        let x = match x_output.get_closest(self.x(), true) {
            Some(x) => x,
            None => {
                warn!("Stacked BartChart x point, {} not found", self.x());
                return;
            }
        };

        let y = match y_output.get_closest(self.y(), false) {
            Some(y) => y,
            None => {
                warn!("Stacked BarChart y point, {} not found", self.y());
                return;
            }
        };

        let DrawnOutput {
            axis_pos: x_axis,
            spacing: x_spacing,
            ..
        } = x_output;

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
    ToggleConfig,
    OpenEditor,
    SequentialX(bool),
    SequentialY(bool),
    Clean(bool),
    Horizontal(bool),
    CaptionChange(String),
    XLabelChanged(String),
    YLabelChanged(String),
    TitleChanged(String),
    Legend(LegendPosition),
    Debug,
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
            acc_cols_str,
            x_col,
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

        let acc_cols = parse_ints(&acc_cols_str);
        let end = if sht.width() == 0 { 0 } else { sht.width() - 1 };
        let acc_cols = Selection::to_vec(acc_cols, end);

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

#[allow(dead_code)]
#[derive(Debug)]
pub struct StackedBarChartTab {
    title: String,
    file: PathBuf,
    x_axis: Scale,
    x_label: Option<String>,
    y_axis: Scale,
    y_label: Option<String>,
    is_horizontal: bool,
    config_shown: bool,
    order: bool,
    sequential_x: bool,
    sequential_y: bool,
    bars: Vec<GraphBar>,
    clean: bool,
    cache: canvas::Cache,
    labels_len: usize,
    caption: Option<String>,
    legend: LegendPosition,
}

impl StackedBarChartTab {
    fn create_axis(&self) -> (Axis, Axis) {
        let (x_scale, y_scale) = if self.is_horizontal {
            (&self.y_axis, &self.x_axis)
        } else {
            (&self.x_axis, &self.y_axis)
        };

        let (x_axis, y_axis) = create_axis(
            x_scale,
            y_scale,
            self.sequential_x,
            self.sequential_y,
            self.clean,
        );

        let (x_label, y_label) = if self.is_horizontal {
            (self.y_label.clone(), self.x_label.clone())
        } else {
            (self.x_label.clone(), self.y_label.clone())
        };

        return (x_axis.label(x_label), y_axis.label(y_label));
    }

    fn graph(&self) -> Element<'_, StackedBarChartMessage> {
        let (x_axis, y_axis) = self.create_axis();

        let content = Canvas::new(
            Graph::new(x_axis, y_axis, &self.bars, &self.cache)
                .caption(self.caption.as_ref())
                .labels_len(self.labels_len)
                .legend(self.legend)
                .data((0, self.is_horizontal)),
        )
        .width(Length::FillPortion(24))
        .height(Length::Fill);

        content.into()
    }

    fn tools(&self) -> Element<'_, StackedBarChartMessage> {
        let header = {
            let header = text("Model Config").size(17.0);

            row!(horizontal_space(), header, horizontal_space())
                .padding([2, 0])
                .align_items(Alignment::Center)
        };

        let title = text_input("Graph Title", self.title.as_str())
            .on_input(StackedBarChartMessage::TitleChanged);

        let x_label = text_input(
            "X axis label",
            self.x_label
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or_default(),
        )
        .on_input(StackedBarChartMessage::XLabelChanged);

        let y_label = text_input(
            "Y axis label",
            self.y_label
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or_default(),
        )
        .on_input(StackedBarChartMessage::YLabelChanged);

        let caption = text_input(
            "Graph Caption",
            self.caption
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or_default(),
        )
        .on_input(StackedBarChartMessage::CaptionChange);

        let sequential = {
            let x = {
                let check = checkbox("Ranged X axis", self.sequential_x)
                    .on_toggle(StackedBarChartMessage::SequentialX);

                let tip = tooltip("Each point on the axis is produced consecutively");

                row!(check, tip).spacing(10.0)
            };

            let y = {
                let check = checkbox("Ranged Y axis", self.sequential_y)
                    .on_toggle(StackedBarChartMessage::SequentialY);

                let tip = tooltip("Each point on the axis is produced consecutively");

                row!(check, tip).spacing(10.0)
            };

            row!(x, horizontal_space(), y).align_items(Alignment::Center)
        };

        let clean = {
            let check =
                checkbox("Clean graph", self.clean).on_toggle(StackedBarChartMessage::Clean);

            let tip = tooltip("Only points on the axes have their outline drawn");

            row!(check, tip).spacing(10.0)
        };

        let horizontal = {
            let check = checkbox("Horizontal bars", self.is_horizontal)
                .on_toggle(StackedBarChartMessage::Horizontal);

            let tip = tooltip("The bars are drawn horizontally");

            row!(check, tip).spacing(10.0)
        };

        let clean_horizontal =
            row!(clean, horizontal_space(), horizontal).align_items(Alignment::Center);

        let legend = {
            let icons = Font::with_name("legend-icons");

            let menu = ToolbarMenu::new(
                LegendPosition::ALL,
                self.legend,
                StackedBarChartMessage::Legend,
                icons,
            )
            .orientation(ToolBarOrientation::Both)
            .padding([4, 4])
            .menu_padding([4, 10, 4, 8])
            .spacing(5.0);

            let tooltip = container(text("Legend Position").size(12.0))
                .max_width(200.0)
                .padding([6, 8])
                .style(theme::Container::Custom(Box::new(ToolTipContainerStyle)))
                .height(Length::Shrink);

            let menu = Tooltip::new(menu, tooltip, iced::widget::tooltip::Position::Bottom)
                .gap(2.0)
                .snap_within_viewport(true);

            let text = text("Legend Position");

            row!(menu, text)
                .spacing(10.0)
                .align_items(Alignment::Center)
        };

        let editor = {
            let font = Font::with_name(icons::NAME);

            let btn = button(
                text(icons::EDITOR)
                    .font(font)
                    .width(16.0)
                    .vertical_alignment(alignment::Vertical::Center)
                    .horizontal_alignment(alignment::Horizontal::Center),
            )
            .on_press(StackedBarChartMessage::OpenEditor)
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

            let text = text("Open in Editor");

            row!(menu, text)
                .spacing(10.0)
                .align_items(Alignment::Center)
        };

        let legend_editor = row!(legend, horizontal_space(), editor).align_items(Alignment::Center);

        let content = column!(
            header,
            title,
            x_label,
            y_label,
            caption,
            sequential,
            clean_horizontal,
            legend_editor,
        )
        .spacing(30.0);

        dialog_container(content)
            .width(450.0)
            .height(Length::Shrink)
            .into()
    }

    fn content(&self) -> Element<'_, StackedBarChartMessage> {
        let graph = self.graph();

        let toolbar = tools_button().on_press(StackedBarChartMessage::ToggleConfig);

        row!(
            graph,
            column!(vertical_space(), toolbar, vertical_space())
                .padding([0, 5, 0, 0])
                .align_items(Alignment::Center)
        )
        .into()
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

        Self {
            title,
            file,
            labels_len,
            x_axis: x_scale,
            x_label: x_axis,
            y_axis: y_scale,
            y_label: y_axis,
            is_horizontal,
            config_shown: false,
            bars,
            order,
            sequential_x: false,
            sequential_y: false,
            caption,
            clean: false,
            cache: canvas::Cache::default(),
            legend: LegendPosition::default(),
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
            StackedBarChartMessage::Debug => {
                dbg!("Debugging!");
                None
            }
            StackedBarChartMessage::ToggleConfig => {
                self.config_shown = !self.config_shown;
                None
            }
            StackedBarChartMessage::OpenEditor => {
                self.config_shown = false;
                Some(Message::OpenEditor(Some(self.file.clone())))
            }
            StackedBarChartMessage::SequentialX(seq) => {
                self.sequential_x = seq;
                self.cache.clear();
                None
            }
            StackedBarChartMessage::SequentialY(seq) => {
                self.sequential_y = seq;
                self.cache.clear();
                None
            }
            StackedBarChartMessage::Clean(clean) => {
                self.clean = clean;
                self.cache.clear();
                None
            }
            StackedBarChartMessage::Horizontal(is_horizontal) => {
                self.is_horizontal = is_horizontal;
                self.cache.clear();
                None
            }
            StackedBarChartMessage::CaptionChange(caption) => {
                self.caption = if caption.is_empty() {
                    None
                } else {
                    Some(caption)
                };
                self.cache.clear();
                None
            }
            StackedBarChartMessage::XLabelChanged(label) => {
                self.x_label = if label.is_empty() { None } else { Some(label) };
                self.cache.clear();
                None
            }
            StackedBarChartMessage::YLabelChanged(label) => {
                self.y_label = if label.is_empty() { None } else { Some(label) };
                self.cache.clear();
                None
            }
            StackedBarChartMessage::TitleChanged(title) => {
                self.title = title;
                None
            }
            StackedBarChartMessage::Legend(legend) => {
                self.legend = legend;
                None
            }
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

        let content_area = container(self.content())
            .max_width(1450)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::Container::Custom(Box::new(ContentAreaContainer)));

        let content = column!(title, content_area)
            .align_items(Alignment::Center)
            .spacing(20)
            .height(Length::Fill)
            .width(Length::Fill);

        let content: Element<Self::Event, Theme, Renderer> = if self.config_shown {
            Modal::new(content, self.tools())
                .on_blur(StackedBarChartMessage::ToggleConfig)
                .into()
        } else {
            content.into()
        };

        let content: Element<Self::Event, Theme, Renderer> = container(content)
            .padding([10, 30, 30, 15])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

        content.map(map)
    }
}
