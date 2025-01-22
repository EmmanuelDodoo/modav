use std::{
    fmt::{self, Debug},
    path::PathBuf,
};
use tracing::warn;

use iced::{
    alignment,
    widget::{
        button,
        canvas::{self, Canvas, Frame, Path, Stroke},
        checkbox, column, container, horizontal_space, row, text, text_input, Tooltip,
    },
    Alignment, Color, Element, Font, Length, Padding, Point, Renderer, Size, Theme,
};

use modav_core::{
    models::{
        line::{self, Line},
        Point as GraphPoint, Scale,
    },
    repr::sheet::builders::SheetBuilder,
};

use crate::{
    utils::{coloring::ColorEngine, icons, tooltip, AppError},
    widgets::{
        toolbar::{ToolBarOrientation, ToolbarMenu, ToolbarOption},
        wizard::LineConfigState,
    },
    Message, ToolTipContainerStyle,
};

use super::{
    parse_seed,
    shared::{
        graph::{create_axis, Axis, DrawnOutput, Graph, Graphable, LegendPosition},
        ContentAreaContainer,
    },
    TabLabel, Viewable,
};

use super::shared::EditorButtonStyle;

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

impl AsRef<str> for GraphType {
    fn as_ref(&self) -> &str {
        match self {
            Self::Line => "Line Graph",
            Self::Point => "Points Graph",
            Self::LinePoint => "Line Graph with Points",
        }
    }
}

impl fmt::Display for GraphType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_ref())
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
pub struct GraphLine {
    points: Vec<GraphPoint>,
    label: Option<String>,
    color: Color,
}

impl GraphLine {
    pub fn new(points: Vec<GraphPoint>, label: Option<String>, color: Color) -> Self {
        Self {
            points,
            color,
            label,
        }
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
}

impl Graphable for GraphLine {
    type Data<'a> = GraphType;

    fn label(&self) -> Option<&String> {
        self.label.as_ref()
    }

    fn draw_legend_filter(&self, _data: &Self::Data<'_>) -> bool {
        self.label().is_some()
    }

    fn draw_legend(
        &self,
        frame: &mut Frame,
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
        frame: &mut Frame,
        x_output: &DrawnOutput,
        y_output: &DrawnOutput,
        data: &Self::Data<'_>,
    ) {
        self.points.iter().fold(None, |prev, point| {
            let x = match x_output.get_closest(&point.x, true) {
                Some(x) => x,
                None => {
                    warn!("X Point {:?} not found", point.x);
                    return prev;
                }
            };

            let y = match y_output.get_closest(&point.y, false) {
                Some(x) => x,
                None => {
                    warn!("Y Point {:?} not found", point.y);
                    return prev;
                }
            };

            let point = Point { x, y };

            match data {
                GraphType::Point => {
                    let path = Path::circle(point.clone(), 4.5);

                    frame.fill(&path, self.color);
                }

                GraphType::Line => {
                    if let Some(prev) = prev {
                        let path = Path::new(|bdr| {
                            bdr.move_to(prev);
                            bdr.line_to(point);
                        });
                        frame.stroke(
                            &path,
                            Stroke::default().with_width(3.0).with_color(self.color),
                        );
                    };
                }

                GraphType::LinePoint => {
                    let path = Path::circle(point.clone(), 3.5);

                    frame.fill(&path, self.color);

                    if let Some(prev) = prev {
                        let path = Path::new(|bdr| {
                            bdr.move_to(prev);
                            bdr.line_to(point);
                        });
                        frame.stroke(
                            &path,
                            Stroke::default().with_width(3.0).with_color(self.color),
                        );
                    };
                }
            };

            return Some(point);
        });
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineTabData {
    file: PathBuf,
    title: String,
    theme: Theme,
    line: line::LineGraph,
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
            ..
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
    ToggleConfig,
    Legend(LegendPosition),
    GraphType(GraphType),
    TitleChanged(String),
    XLabelChanged(String),
    YLabelChanged(String),
    CaptionChange(String),
    SequentialX(bool),
    SequentialY(bool),
    Clean(bool),
    ChangeSeed(String),
    ApplySeed,
    RandomSeed,
}

#[derive(Debug)]
pub struct LineGraphTab {
    file: PathBuf,
    title: String,
    x_scale: Scale,
    y_scale: Scale,
    x_label: Option<String>,
    y_label: Option<String>,
    lines: Vec<GraphLine>,
    theme: Theme,
    caption: Option<String>,
    sequential_x: bool,
    sequential_y: bool,
    clean: bool,
    color_seed: f32,
    config_shown: bool,
    legend: LegendPosition,
    graph_type: GraphType,
    cache: canvas::Cache,
}

impl LineGraphTab {
    fn create_axis(&self) -> (Axis, Axis) {
        let (x_axis, y_axis) = create_axis(
            &self.x_scale,
            &self.y_scale,
            self.sequential_x,
            self.sequential_y,
            self.clean,
        );

        return (
            x_axis.label(self.x_label.as_ref()),
            y_axis.label(self.y_label.as_ref()),
        );
    }

    fn graph(&self) -> Element<'_, ModelMessage> {
        let (x_axis, y_axis) = self.create_axis();

        let content = Canvas::new(
            Graph::new(
                x_axis,
                y_axis,
                &self.lines,
                &self.theme,
                &self.cache,
                self.graph_type,
            )
            .caption(self.caption.as_ref())
            .labels_len(
                self.lines
                    .iter()
                    .filter(|line| line.label.is_some())
                    .count(),
            )
            .legend(self.legend),
        )
        .width(Length::FillPortion(24))
        .height(Length::Fill);

        content.into()
    }

    fn tools(&self) -> Element<'_, ModelMessage> {
        let spacing = 10.0;

        let header = {
            let header = text("Model Config").size(17.0);

            row!(horizontal_space(), header, horizontal_space())
                .padding([2, 0])
                .align_y(Alignment::Center)
        };

        let title =
            text_input("Graph Title", self.title.as_str()).on_input(ModelMessage::TitleChanged);

        let x_label = text_input(
            "X axis label",
            self.x_label
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or_default(),
        )
        .on_input(ModelMessage::XLabelChanged);

        let y_label = text_input(
            "Y axis label",
            self.y_label
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or_default(),
        )
        .on_input(ModelMessage::YLabelChanged);

        let caption = text_input(
            "Graph Caption",
            self.caption
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or_default(),
        )
        .on_input(ModelMessage::CaptionChange);

        let ranged_x = {
            let check = {
                let check = checkbox("", self.sequential_x).on_toggle(ModelMessage::SequentialX);
                let label = text("Ranged X axis");

                row!(label, check).spacing(8.0).align_y(Alignment::Center)
            };

            let tip = tooltip("Each point on the axis is produced consecutively");

            row!(check, tip).spacing(spacing)
        };

        let ranged_y = {
            let check = {
                let check = checkbox("", self.sequential_y).on_toggle(ModelMessage::SequentialY);
                let label = text("Ranged Y axis");

                row!(label, check).align_y(Alignment::Center).spacing(8.0)
            };

            let tip = tooltip("Each point on the axis is produced consecutively");

            row!(check, tip).spacing(spacing)
        };

        let clean = {
            let check = {
                let check = checkbox("", self.clean).on_toggle(ModelMessage::Clean);
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
            .on_press(ModelMessage::ApplySeed);

            let rand = button(icons::icon(icons::SHUFFLE).align_y(alignment::Vertical::Center))
                .padding([4, 8])
                .style(button::secondary)
                .on_press(ModelMessage::RandomSeed);

            let input = text_input("", &value)
                .on_input(ModelMessage::ChangeSeed)
                .padding([2, 5])
                .width(67.0);

            row!(label, input, btn, rand, tip)
                .spacing(spacing)
                .align_y(Alignment::Center)
        };

        let kind = {
            let icons = Font::with_name("line-type-icons");

            let menu = ToolbarMenu::new(
                GraphType::ALL,
                self.graph_type,
                ModelMessage::GraphType,
                icons,
            )
            .orientation(ToolBarOrientation::Both)
            .padding([4, 4])
            .menu_padding(Padding::default().top(4.).right(10.).bottom(4.).left(8))
            .spacing(5.0);

            let tooltip = container(text("The type of line graph").size(12.0))
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

            let text = text("Graph Type");

            row!(text, menu).spacing(spacing).align_y(Alignment::Center)
        };

        let legend = {
            let icons = Font::with_name("legend-icons");

            let menu = ToolbarMenu::new(
                LegendPosition::ALL,
                self.legend,
                ModelMessage::Legend,
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
            .on_press(ModelMessage::OpenEditor)
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
            header, title, x_label, y_label, caption, ranged_x, ranged_y, clean, kind, seed,
            legend, editor
        )
        .spacing(25.0)
        .into()
    }

    fn redraw(&mut self) {
        self.cache.clear()
    }

    fn recolor(&mut self, colors: ColorEngine) {
        self.lines
            .iter_mut()
            .zip(colors)
            .for_each(|(line, color)| line.set_color(color));

        self.redraw()
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
        let seed = colors.seed();

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
            sequential_x: false,
            sequential_y: false,
            clean: false,
            color_seed: seed,
            config_shown: false,
            cache: canvas::Cache::default(),
            legend: LegendPosition::default(),
            graph_type: GraphType::default(),
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

    fn theme_changed(&mut self, theme: &Theme) {
        if &self.theme == theme {
            return;
        }

        self.theme = theme.clone();

        let colors = ColorEngine::new(&self.theme);
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
            ModelMessage::OpenEditor => Some(Message::OpenEditor(Some(self.file.clone()))),
            ModelMessage::ToggleConfig => {
                self.config_shown = !self.config_shown;
                None
            }
            ModelMessage::Legend(legend) => {
                self.legend = legend;
                self.cache.clear();
                None
            }
            ModelMessage::GraphType(kind) => {
                self.graph_type = kind;
                self.cache.clear();
                None
            }
            ModelMessage::TitleChanged(title) => {
                self.title = title;
                self.cache.clear();
                None
            }
            ModelMessage::Clean(clean) => {
                self.clean = clean;
                self.cache.clear();
                None
            }
            ModelMessage::SequentialX(seq) => {
                self.sequential_x = seq;
                self.cache.clear();
                None
            }
            ModelMessage::SequentialY(seq) => {
                self.sequential_y = seq;
                self.cache.clear();
                None
            }
            ModelMessage::CaptionChange(caption) => {
                self.caption = if caption.is_empty() {
                    None
                } else {
                    Some(caption)
                };
                self.cache.clear();
                None
            }
            ModelMessage::XLabelChanged(label) => {
                self.x_label = if label.is_empty() { None } else { Some(label) };
                self.cache.clear();
                None
            }
            ModelMessage::YLabelChanged(label) => {
                self.y_label = if label.is_empty() { None } else { Some(label) };
                self.cache.clear();
                None
            }
            ModelMessage::ApplySeed => {
                let colors = ColorEngine::new_with_seed(&self.theme, self.color_seed);
                self.recolor(colors);
                None
            }
            ModelMessage::ChangeSeed(seed) => {
                if let Some(seed) = parse_seed(seed, self.color_seed == 0.0) {
                    self.color_seed = seed;
                }

                None
            }
            ModelMessage::RandomSeed => {
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
            let text = text(format!("{} - Model", self.title));
            row!(horizontal_space(), text, horizontal_space())
                .width(Length::Fill)
                .align_y(Alignment::Center)
        }
        .height(Length::Shrink);

        let content_area = container(self.graph())
            .max_width(1450)
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
