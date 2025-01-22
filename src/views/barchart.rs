use std::{fmt::Debug, path::PathBuf};

use iced::{
    alignment,
    widget::{
        button, canvas, checkbox, column, container, horizontal_space, row, text, text_input,
        Canvas, Tooltip,
    },
    Alignment, Color, Element, Font, Length, Padding, Point, Renderer, Size, Theme,
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
    utils::{coloring::ColorEngine, icons, tooltip, AppError},
    widgets::{
        modal::Modal,
        toolbar::{ToolBarOrientation, ToolbarMenu},
        wizard::BarChartConfigState,
    },
    Message, ToolTipContainerStyle,
};

use super::{
    parse_seed,
    shared::{
        graph::{create_axis, Axis, DrawnOutput, Graph, Graphable, LegendPosition},
        ContentAreaContainer, EditorButtonStyle,
    },
    tabs::TabLabel,
    Viewable,
};

#[derive(Debug, Clone, PartialEq)]
pub struct GraphBar {
    point: GraphPoint,
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

    fn set_color(&mut self, color: impl Into<Color>) {
        self.color = color.into();
    }
}

impl From<Bar> for GraphBar {
    fn from(value: Bar) -> Self {
        let Bar { point, label } = value;

        Self::new(point, label, Color::BLACK)
    }
}

impl Graphable for GraphBar {
    type Data<'a> = bool;

    fn label(&self) -> Option<&String> {
        self.label.as_ref()
    }

    fn draw_legend_filter(&self, _data: &Self::Data<'_>) -> bool {
        self.label.is_some()
    }

    fn draw_legend(
        &self,
        frame: &mut canvas::Frame,
        bounds: iced::Rectangle,
        color: Color,
        idx: usize,
        _data: &Self::Data<'_>,
    ) {
        if idx > 4 {
            return;
        }

        let y_padding = bounds.height / (5.0);
        let spacing = 5.0;
        let text_size = 12.0;
        let color_size = Size::new(12.0, 12.0);

        let x = bounds.x;
        let y = bounds.position().y + (idx as f32 * y_padding);
        let position = Point::new(x, y);

        frame.fill_rectangle(position, color_size, self.color);

        let position = Point::new(x + spacing + color_size.width, y + 0.5 * color_size.height);

        let label = canvas::Text {
            content: self.label.clone().unwrap_or_default(),
            position,
            color,
            size: text_size.into(),
            vertical_alignment: alignment::Vertical::Center,
            ..Default::default()
        };

        frame.fill_text(label);
    }

    fn draw(
        &self,
        frame: &mut canvas::Frame,
        x_output: &DrawnOutput,
        y_output: &DrawnOutput,
        data: &Self::Data<'_>,
    ) {
        let is_horizontal = *data;

        let mut x_output = x_output;
        let mut y_output = y_output;

        if is_horizontal {
            let temp = x_output;
            x_output = y_output;
            y_output = temp;
        }

        let x = match x_output.get_closest(&self.point.x, true) {
            Some(x) => x,
            None => {
                warn!("BartChart x point, {} not found", &self.point.x);
                return;
            }
        };

        let y = match y_output.get_closest(&self.point.y, false) {
            Some(y) => y,
            None => {
                warn!("BarChart y point, {} not found", &self.point.y);
                return;
            }
        };

        let DrawnOutput {
            axis_pos: x_axis,
            spacing: x_spacing,
            ..
        } = x_output;

        if is_horizontal {
            let height = x_spacing / 2.0;

            let base = *x_axis;

            let top_left = Point::new(f32::min(base, y), x - (height / 2.0));
            let size = Size::new(f32::abs(base - y), height);

            frame.fill_rectangle(top_left, size, self.color)
        } else {
            let width = x_spacing / 2.0;
            let base = *x_axis;

            let top_left = Point::new(x - (width / 2.0), f32::min(y, base));
            let size = Size::new(width, f32::abs(base - y));

            frame.fill_rectangle(top_left, size, self.color);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BarChartTabData {
    file: PathBuf,
    title: String,
    barchart: BarChart,
    theme: Theme,
    order: bool,
    is_horizontal: bool,
    caption: Option<String>,
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
            order,
            caption,
            is_horizontal,
            ..
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
            order,
            caption,
            is_horizontal,
            theme: Theme::default(),
        })
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }
}

#[derive(Debug, Clone)]
pub enum BarChartMessage {
    OpenEditor,
    ToggleConfig,
    TitleChanged(String),
    SequentialX(bool),
    SequentialY(bool),
    Clean(bool),
    Horizontal(bool),
    CaptionChange(String),
    XLabelChanged(String),
    YLabelChanged(String),
    Legend(LegendPosition),
    ChangeSeed(String),
    ApplySeed,
    RandomSeed,
}

#[derive(Debug)]
pub struct BarChartTab {
    file: PathBuf,
    title: String,
    x_axis: Scale,
    y_axis: Scale,
    x_label: Option<String>,
    y_label: Option<String>,
    bars: Vec<GraphBar>,
    caption: Option<String>,
    is_horizontal: bool,
    config_shown: bool,
    sequential_x: bool,
    sequential_y: bool,
    clean: bool,
    color_seed: f32,
    cache: canvas::Cache,
    legend: LegendPosition,
    theme: Theme,
    order: bool,
}

impl BarChartTab {
    fn tools(&self) -> Element<'_, BarChartMessage> {
        let spacing = 10.0;

        let header = {
            let header = text("Model Config").size(17.0);

            row!(horizontal_space(), header, horizontal_space())
                .padding([2, 0])
                .align_y(Alignment::Center)
        };

        let title =
            text_input("Graph Title", self.title.as_str()).on_input(BarChartMessage::TitleChanged);

        let x_label = text_input(
            "X axis label",
            self.x_label
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or_default(),
        )
        .on_input(BarChartMessage::XLabelChanged);

        let y_label = text_input(
            "Y axis label",
            self.y_label
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or_default(),
        )
        .on_input(BarChartMessage::YLabelChanged);

        let caption = text_input(
            "Graph Caption",
            self.caption
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or_default(),
        )
        .on_input(BarChartMessage::CaptionChange);

        let ranged_x = {
            let check = {
                let check = checkbox("", self.sequential_x).on_toggle(BarChartMessage::SequentialX);
                let label = text("Ranged X axis");

                row!(label, check).spacing(8.0).align_y(Alignment::Center)
            };

            let tip = tooltip("Each point on the axis is produced consecutively");

            row!(check, tip).spacing(spacing)
        };

        let ranged_y = {
            let check = {
                let check = checkbox("", self.sequential_y).on_toggle(BarChartMessage::SequentialY);
                let label = text("Ranged Y axis");

                row!(label, check).align_y(Alignment::Center).spacing(8.0)
            };

            let tip = tooltip("Each point on the axis is produced consecutively");

            row!(check, tip).spacing(spacing)
        };

        let clean = {
            let check = {
                let check = checkbox("", self.clean).on_toggle(BarChartMessage::Clean);
                let label = text("Clean graph");

                row!(label, check).align_y(Alignment::Center).spacing(8.0)
            };

            let tip = tooltip("Only points on the axes have their outline drawn");

            row!(check, tip).spacing(spacing)
        };

        let seed = {
            let value = self.color_seed;
            let value = format!("{value:.4}");

            let label = text("Coloring Seed");

            let tip = tooltip("Sets the seed used to generate graph colors");

            let btn = button(
                icons::icon(icons::REDO)
                    .align_y(alignment::Vertical::Center)
                    .align_x(alignment::Horizontal::Center),
            )
            .padding([4, 8])
            //.style(button::secondary)
            .on_press(BarChartMessage::ApplySeed);

            let rand = button(icons::icon(icons::SHUFFLE).align_y(alignment::Vertical::Center))
                .padding([4, 8])
                .style(button::secondary)
                .on_press(BarChartMessage::RandomSeed);

            let input = text_input("", &value)
                .on_input(BarChartMessage::ChangeSeed)
                .padding([2, 5])
                .width(67.0);

            row!(label, input, btn, rand, tip)
                .spacing(spacing)
                .align_y(Alignment::Center)
        };

        let horizontal = {
            let check = {
                let check = checkbox("", self.is_horizontal).on_toggle(BarChartMessage::Horizontal);
                let label = text("Horizontal bars");
                row!(label, check).align_y(Alignment::Center).spacing(8.0)
            };

            let tip = tooltip("The bars are drawn horizontally");

            row!(check, tip).spacing(spacing)
        };

        let legend = {
            let icons = Font::with_name("legend-icons");

            let menu = ToolbarMenu::new(
                LegendPosition::ALL,
                self.legend,
                BarChartMessage::Legend,
                icons,
            )
            .orientation(ToolBarOrientation::Both)
            .padding([4, 4])
            .menu_padding(Padding {
                top: 4.,
                right: 10.,
                bottom: 4.,
                left: 8.,
            })
            .spacing(5.0);

            let tooltip = container(text("Legend Position").size(12.0))
                .max_width(200.0)
                .padding([6, 8])
                .style(|theme| {
                    <ToolTipContainerStyle as container::Catalog>::style(
                        &ToolTipContainerStyle,
                        theme,
                    )
                })
                .height(Length::Shrink);

            let menu = Tooltip::new(menu, tooltip, iced::widget::tooltip::Position::Bottom)
                .gap(2.0)
                .snap_within_viewport(true);

            let text = text("Legend Position");

            row!(text, menu).spacing(spacing).align_y(Alignment::Center)
        };

        let editor = {
            let font = Font::with_name(icons::NAME);

            let btn = button(
                text(icons::EDITOR)
                    .font(font)
                    .width(16.0)
                    .align_y(alignment::Vertical::Center)
                    .align_x(alignment::Horizontal::Center),
            )
            .on_press(BarChartMessage::OpenEditor)
            .style(|theme, status| {
                <EditorButtonStyle as button::Catalog>::style(&EditorButtonStyle, theme, status)
            })
            .padding([4, 4]);

            let tooltip = container(text("Open in Editor").size(12.0))
                .max_width(200.0)
                .padding([6, 8])
                .style(|theme| {
                    <ToolTipContainerStyle as container::Catalog>::style(
                        &ToolTipContainerStyle,
                        theme,
                    )
                })
                .height(Length::Shrink);

            let menu = Tooltip::new(btn, tooltip, iced::widget::tooltip::Position::Bottom)
                .gap(2.0)
                .snap_within_viewport(true);

            let text = text("Open in Editor");

            row!(text, menu).spacing(spacing).align_y(Alignment::Center)
        };

        column!(
            header, title, x_label, y_label, caption, ranged_x, ranged_y, clean, horizontal, seed,
            legend, editor,
        )
        .spacing(25.0)
        .into()
    }

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

    fn graph(&self) -> Element<'_, BarChartMessage> {
        let (x_axis, y_axis) = self.create_axis();

        let content = Canvas::new(
            Graph::new(
                x_axis,
                y_axis,
                &self.bars,
                &self.theme,
                &self.cache,
                self.is_horizontal,
            )
            .caption(self.caption.as_ref())
            .labels_len(self.bars.iter().filter(|bar| bar.label.is_some()).count())
            .legend(self.legend),
        )
        .width(Length::FillPortion(24))
        .height(Length::Fill);

        content.into()
    }

    fn redraw(&mut self) {
        self.cache.clear()
    }

    fn recolor(&mut self, colors: ColorEngine) {
        self.bars.iter_mut().zip(colors).for_each(|(bar, color)| {
            bar.set_color(color);
        });

        self.redraw()
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
            order,
            caption,
            is_horizontal,
        } = data;

        let BarChart {
            x_label,
            x_scale,
            y_label,
            y_scale,
            mut bars,
        } = barchart;

        if order {
            bars.sort_by(|one, two| one.point.x.cmp(&two.point.x))
        };

        let colors = ColorEngine::new(&theme).gradual(order);
        let seed = colors.seed();

        let bars = bars
            .into_iter()
            .zip(colors)
            .map(|(bar, color)| Into::<GraphBar>::into(bar).color(color))
            .collect();

        Self {
            file,
            title,
            x_axis: x_scale,
            x_label,
            y_axis: y_scale,
            y_label,
            caption,
            bars,
            is_horizontal,
            theme,
            order,
            color_seed: seed,
            config_shown: false,
            sequential_x: false,
            sequential_y: false,
            clean: false,
            legend: LegendPosition::default(),
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

    fn theme_changed(&mut self, theme: &Theme) {
        if &self.theme == theme {
            return;
        }
        self.theme = theme.clone();

        let colors = ColorEngine::new(&self.theme).gradual(self.order);
        self.color_seed = colors.seed();

        self.recolor(colors);
    }

    fn has_config(&self) -> bool {
        true
    }

    fn config<'a, Message, F>(&'a self, map: F) -> Option<Element<'a, Message, Theme, Renderer>>
    where
        F: 'a + Fn(Self::Event) -> Message,
        Message: 'a + Clone + Debug,
    {
        Some(self.tools().map(map))
    }

    fn update(&mut self, message: Self::Event) -> Option<Message> {
        match message {
            BarChartMessage::OpenEditor => Some(Message::OpenEditor(Some(self.file.clone()))),
            BarChartMessage::ToggleConfig => {
                self.config_shown = !self.config_shown;
                None
            }
            BarChartMessage::TitleChanged(title) => {
                self.title = title;
                None
            }
            BarChartMessage::SequentialX(seq) => {
                self.sequential_x = seq;
                self.cache.clear();
                None
            }
            BarChartMessage::SequentialY(seq) => {
                self.sequential_y = seq;
                self.cache.clear();
                None
            }
            BarChartMessage::Clean(clean) => {
                self.clean = clean;
                self.cache.clear();
                None
            }
            BarChartMessage::Horizontal(is_horizontal) => {
                self.is_horizontal = is_horizontal;
                self.cache.clear();
                None
            }
            BarChartMessage::CaptionChange(caption) => {
                self.caption = if caption.is_empty() {
                    None
                } else {
                    Some(caption)
                };
                self.cache.clear();
                None
            }
            BarChartMessage::XLabelChanged(label) => {
                self.x_label = if label.is_empty() { None } else { Some(label) };
                self.cache.clear();
                None
            }
            BarChartMessage::YLabelChanged(label) => {
                self.y_label = if label.is_empty() { None } else { Some(label) };
                self.cache.clear();
                None
            }
            BarChartMessage::Legend(legend) => {
                self.legend = legend;
                None
            }
            BarChartMessage::ChangeSeed(seed) => {
                if let Some(seed) = parse_seed(seed, self.color_seed == 0.0) {
                    self.color_seed = seed;
                }

                None
            }
            BarChartMessage::ApplySeed => {
                let colors = ColorEngine::new_with_seed(&self.theme, self.color_seed);
                self.recolor(colors);
                None
            }
            BarChartMessage::RandomSeed => {
                use rand::{thread_rng, Rng};
                let seed: f32 = thread_rng().gen();
                self.color_seed = seed;

                let colors = ColorEngine::new_with_seed(&self.theme, self.color_seed);
                self.recolor(colors);
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
            let text = text(format!("{} - Bar Chart", self.title));
            row!(horizontal_space(), text, horizontal_space())
                .width(Length::Fill)
                .align_y(Alignment::Center)
        }
        .height(Length::Shrink);

        let content_area = container(self.graph())
            .max_width(1450)
            // .padding([5, 10])
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|theme| {
                <ContentAreaContainer as container::Catalog>::style(&ContentAreaContainer, theme)
            });

        let content = column!(title, content_area)
            .align_x(Alignment::Center)
            .spacing(20)
            .height(Length::Fill)
            .width(Length::Fill);

        let content: Element<Self::Event, Theme, Renderer> = if self.config_shown {
            Modal::new(content, self.tools())
                .on_blur(BarChartMessage::ToggleConfig)
                .into()
        } else {
            content.into()
        };

        let content: Element<Self::Event, Theme, Renderer> = container(content)
            .padding(Padding {
                top: 10.,
                right: 30.,
                bottom: 30.,
                left: 15.,
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

        content.map(map)
    }
}
