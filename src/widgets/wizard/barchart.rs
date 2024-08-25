use std::{
    collections::HashSet,
    fmt::{self, Debug},
    path::PathBuf,
};

use iced::{
    widget::{
        button, column, component, container, horizontal_space, pick_list, row, text, text_input,
        vertical_space, Component, Space,
    },
    Alignment, Element, Renderer, Theme,
};
use modav_core::repr::sheet::utils::{
    BarChartAxisLabelStrategy, BarChartBarLabels, HeaderLabelStrategy, HeaderTypesStrategy,
};

use crate::{
    utils::AppError,
    views::{BarChartTabData, View},
};

use super::{shared::tooltip, sheet::SheetConfigState};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum AxisStrategy {
    #[default]
    None,
    Headers,
    Provided,
}

impl fmt::Display for AxisStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::None => "No labels",
                Self::Headers => "Use headers",
                Self::Provided => "Provide labels",
            }
        )
    }
}

impl From<BarChartAxisLabelStrategy> for AxisStrategy {
    fn from(value: BarChartAxisLabelStrategy) -> Self {
        match value {
            BarChartAxisLabelStrategy::None => Self::None,
            BarChartAxisLabelStrategy::Headers => Self::Headers,
            BarChartAxisLabelStrategy::Provided { .. } => Self::Provided,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum BarLabels {
    #[default]
    None,
    FromColumn,
    Provided,
}

impl fmt::Display for BarLabels {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::None => "No labels",
                Self::FromColumn => "Use Column",
                Self::Provided => "Provide labels",
            }
        )
    }
}

impl From<BarChartBarLabels> for BarLabels {
    fn from(value: BarChartBarLabels) -> Self {
        match value {
            BarChartBarLabels::None => Self::None,
            BarChartBarLabels::Provided(_) => Self::Provided,
            BarChartBarLabels::FromColumn(_) => Self::FromColumn,
        }
    }
}

#[derive(Debug, Clone)]
pub enum BarChartConfigMessage {
    TitleChanged(String),
    AxisLabel(AxisStrategy),
    BarLabel(BarLabels),
    BarLabelColumn(String),
    XCol(String),
    YCol(String),
    XLabelChanged(String),
    YLabelChanged(String),
    Previous,
    Cancel,
    Submit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BarChartConfigState {
    pub title: String,
    pub x_col: usize,
    pub y_col: usize,
    pub row_exclude: HashSet<usize>,
    pub bar_label: BarChartBarLabels,
    pub axis_label: BarChartAxisLabelStrategy,
    pub trim: bool,
    pub flexible: bool,
    pub header_types: HeaderTypesStrategy,
    pub header_labels: HeaderLabelStrategy,
}

impl Default for BarChartConfigState {
    fn default() -> Self {
        Self {
            title: "Untitled".into(),
            x_col: 0,
            y_col: 0,
            bar_label: BarChartBarLabels::default(),
            axis_label: BarChartAxisLabelStrategy::default(),
            row_exclude: HashSet::default(),
            trim: true,
            flexible: false,
            header_types: HeaderTypesStrategy::Infer,
            header_labels: HeaderLabelStrategy::ReadLabels,
        }
    }
}

impl BarChartConfigState {
    pub fn diff(&mut self, sheet_config: SheetConfigState) {
        let SheetConfigState {
            trim,
            flexible,
            header_type,
            header_labels,
        } = sheet_config;

        self.trim = trim;
        self.flexible = flexible;
        self.header_labels = header_labels;
        self.header_types = header_type;
    }
}

pub struct BarChartConfig<'a, Message> {
    file: &'a PathBuf,
    sheet_config: SheetConfigState,
    on_submit: Box<dyn Fn(View) -> Message + 'a>,
    on_error: Box<dyn Fn(AppError) -> Message + 'a>,
    on_previous: Message,
    on_cancel: Message,
}

impl<'a, Message> BarChartConfig<'a, Message> {
    pub fn new<S, E>(
        file: &'a PathBuf,
        sheet_config: SheetConfigState,
        on_submit: S,
        on_error: E,
        on_previous: Message,
        on_cancel: Message,
    ) -> Self
    where
        S: 'a + Fn(View) -> Message,
        E: 'a + Fn(AppError) -> Message,
    {
        Self {
            file,
            sheet_config,
            on_submit: Box::new(on_submit),
            on_error: Box::new(on_error),
            on_previous,
            on_cancel,
        }
    }

    fn actions(&self) -> Element<'_, BarChartConfigMessage> {
        let cancel_btn = button(text("Cancel").size(13.0)).on_press(BarChartConfigMessage::Cancel);

        let prev_btn = button(text("Back").size(13.0)).on_press(BarChartConfigMessage::Previous);

        let submit = button(text("Open").size(13.0)).on_press(BarChartConfigMessage::Submit);

        let actions = row!(
            cancel_btn,
            horizontal_space(),
            row!(prev_btn, submit).spacing(10.0)
        );

        actions.into()
    }

    fn barchart_config(&self, state: &BarChartConfigState) -> Element<'_, BarChartConfigMessage> {
        let title = text_input("Graph Title", state.title.as_str())
            .on_input(BarChartConfigMessage::TitleChanged);

        let x_col = {
            let label = text("X column: ");

            let input = {
                let value = state.x_col.to_string();

                text_input("", &value)
                    .on_input(BarChartConfigMessage::XCol)
                    .width(50)
            };

            let tip = tooltip("Columns on the x-axis");

            row!(label, input, tip)
                .spacing(15)
                .align_items(Alignment::Center)
        };

        let y_col = {
            let label = text("Y column: ");

            let input = {
                let value = state.y_col.to_string();

                text_input("", &value)
                    .on_input(BarChartConfigMessage::YCol)
                    .width(50)
            };

            let tip = tooltip("Columns on the y-axis");

            row!(label, input, tip)
                .spacing(15)
                .align_items(Alignment::Center)
        };

        let axis_label = {
            let label = text("Axis Labels: ");

            let options = [
                AxisStrategy::Headers,
                AxisStrategy::Provided,
                AxisStrategy::None,
            ];

            let selected: AxisStrategy = state.axis_label.clone().into();

            let list = pick_list(options, Some(selected), BarChartConfigMessage::AxisLabel)
                .text_size(13.0);

            let tip = tooltip("How labels for the axis are determined");

            let content = row!(label, list, tip)
                .spacing(8)
                .align_items(Alignment::Center);

            let extra: Element<'_, BarChartConfigMessage> = match &state.axis_label {
                BarChartAxisLabelStrategy::Provided { x, y } => {
                    let x_label = text_input("X axis label", x.as_str())
                        .on_input(BarChartConfigMessage::XLabelChanged);

                    let y_label = text_input("Y axis label", y.as_str())
                        .on_input(BarChartConfigMessage::YLabelChanged);

                    column!(x_label, y_label).spacing(10.0).into()
                }
                _ => Space::new(0.0, 0.0).into(),
            };

            column!(content, extra).spacing(10.0)
        };

        let bar_labels = {
            let label = text("Bar labels: ");

            let options = [BarLabels::FromColumn, BarLabels::None];

            let selected: BarLabels = match &state.bar_label {
                BarChartBarLabels::None => BarLabels::None,
                BarChartBarLabels::FromColumn(_) => BarLabels::FromColumn,
                BarChartBarLabels::Provided(_) => BarLabels::None,
            };

            let list =
                pick_list(options, Some(selected), BarChartConfigMessage::BarLabel).text_size(13.0);

            let tip = tooltip("How labels for the bars are determined");

            let input = {
                let value = match &state.bar_label {
                    BarChartBarLabels::FromColumn(col) => col.to_string(),
                    _ => "-".to_string(),
                };

                text_input("", &value)
                    .on_input(BarChartConfigMessage::BarLabelColumn)
                    .width(50.0)
            };

            row!(label, list, input, tip)
                .spacing(15)
                .align_items(Alignment::Center)
        };

        column!(title, x_col, y_col, axis_label, bar_labels)
            .spacing(20.0)
            .into()
    }
}

impl<'a, Message> Component<Message> for BarChartConfig<'a, Message>
where
    Message: Debug + Clone,
{
    type Event = BarChartConfigMessage;
    type State = BarChartConfigState;

    fn update(&mut self, state: &mut Self::State, event: Self::Event) -> Option<Message> {
        match event {
            BarChartConfigMessage::Cancel => Some(self.on_cancel.clone()),
            BarChartConfigMessage::Previous => Some(self.on_previous.clone()),
            BarChartConfigMessage::Submit => {
                state.diff(self.sheet_config.clone());
                let data = BarChartTabData::new(self.file.clone(), state.clone());
                match data {
                    Err(error) => Some((self.on_error)(error)),
                    Ok(data) => {
                        let view = View::BarChart(data);
                        Some((self.on_submit)(view))
                    }
                }
            }
            BarChartConfigMessage::TitleChanged(title) => {
                state.title = title;
                None
            }
            BarChartConfigMessage::AxisLabel(strat) => {
                let strat = match strat {
                    AxisStrategy::None => BarChartAxisLabelStrategy::None,
                    AxisStrategy::Headers => BarChartAxisLabelStrategy::Headers,
                    AxisStrategy::Provided => BarChartAxisLabelStrategy::Provided {
                        x: String::new(),
                        y: String::new(),
                    },
                };

                state.axis_label = strat;
                None
            }
            BarChartConfigMessage::XLabelChanged(label) => match &state.axis_label {
                BarChartAxisLabelStrategy::Provided { y, .. } => {
                    state.axis_label = BarChartAxisLabelStrategy::Provided {
                        x: label,
                        y: y.clone(),
                    };
                    None
                }
                _ => None,
            },
            BarChartConfigMessage::YLabelChanged(label) => match &state.axis_label {
                BarChartAxisLabelStrategy::Provided { x, .. } => {
                    state.axis_label = BarChartAxisLabelStrategy::Provided {
                        x: x.clone(),
                        y: label,
                    };
                    None
                }
                _ => None,
            },
            BarChartConfigMessage::BarLabel(label) => {
                let strat = match label {
                    BarLabels::FromColumn => BarChartBarLabels::FromColumn(0),
                    BarLabels::None => BarChartBarLabels::None,
                    BarLabels::Provided => BarChartBarLabels::Provided(vec![]),
                };
                state.bar_label = strat;
                None
            }
            BarChartConfigMessage::BarLabelColumn(input) => {
                if let BarChartBarLabels::FromColumn(col) = state.bar_label {
                    let input = input.trim().to_string();
                    let col = if input.is_empty() {
                        0
                    } else {
                        let first = input.chars().next().unwrap();

                        let mut input = input;
                        if col == 0 && first != '0' {
                            input.pop();
                        }

                        input.parse().unwrap_or_default()
                    };

                    state.bar_label = BarChartBarLabels::FromColumn(col);
                }

                None
            }

            BarChartConfigMessage::XCol(input) => {
                let input = input.trim().to_string();
                let col = if input.is_empty() {
                    0
                } else {
                    let first = input.chars().next().unwrap();
                    let mut input = input;
                    if state.x_col == 0 && first != '0' {
                        input.pop();
                    }

                    input.parse().unwrap_or_default()
                };

                state.x_col = col;
                None
            }

            BarChartConfigMessage::YCol(input) => {
                let input = input.trim().to_string();
                let col = if input.is_empty() {
                    0
                } else {
                    let first = input.chars().next().unwrap();
                    let mut input = input;
                    if state.y_col == 0 && first != '0' {
                        input.pop();
                    }
                    input.parse().unwrap_or_default()
                };

                state.y_col = col;
                None
            }
        }
    }

    fn view(&self, state: &Self::State) -> Element<'_, Self::Event, Theme, Renderer> {
        let config = self.barchart_config(state);
        let content = column!(config, vertical_space().height(50.0), self.actions()).spacing(10.0);

        container(content).into()
    }
}

impl<'a, Message> From<BarChartConfig<'a, Message>> for Element<'a, Message>
where
    Message: Clone + Debug + 'a,
{
    fn from(value: BarChartConfig<'a, Message>) -> Self {
        component(value)
    }
}
