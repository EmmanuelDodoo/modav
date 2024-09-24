use std::{
    collections::HashSet,
    fmt::{self, Debug},
    path::PathBuf,
};

use iced::{
    widget::{
        button, checkbox, column, component, container, horizontal_space, pick_list, row, text,
        text_input, vertical_space, Component, Space,
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
    Order(bool),
    Horizontal(bool),
    Previous,
    Cancel,
    Submit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BarChartConfigState {
    pub title: String,
    pub x_col: usize,
    pub y_col: usize,
    pub caption: Option<String>,
    pub row_exclude: HashSet<usize>,
    pub bar_label: BarChartBarLabels,
    pub axis_label: BarChartAxisLabelStrategy,
    pub trim: bool,
    pub flexible: bool,
    pub header_types: HeaderTypesStrategy,
    pub header_labels: HeaderLabelStrategy,
    pub order: bool,
    pub is_horizontal: bool,
    pub use_previous: bool,
}

impl Default for BarChartConfigState {
    fn default() -> Self {
        Self {
            title: "Untitled".into(),
            x_col: 0,
            y_col: 0,
            caption: None,
            bar_label: BarChartBarLabels::default(),
            axis_label: BarChartAxisLabelStrategy::default(),
            row_exclude: HashSet::default(),
            trim: true,
            flexible: false,
            header_types: HeaderTypesStrategy::Infer,
            header_labels: HeaderLabelStrategy::ReadLabels,
            order: false,
            is_horizontal: false,
            use_previous: true,
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
            caption,
            ..
        } = sheet_config;

        self.trim = trim;
        self.flexible = flexible;
        self.header_labels = header_labels;
        self.header_types = header_type;
        self.caption = caption;
    }

    fn submit(&self) -> Self {
        Self {
            use_previous: true,
            ..self.clone()
        }
    }
}

pub struct BarChartConfig<'a, Message> {
    file: &'a PathBuf,
    sheet_config: SheetConfigState,
    on_submit: Box<dyn Fn(View) -> Message + 'a>,
    on_error: Box<dyn Fn(AppError) -> Message + 'a>,
    on_previous: Box<dyn Fn(BarChartConfigState) -> Message + 'a>,
    on_cancel: Message,
    on_clear_error: Message,
    previous_state: Option<BarChartConfigState>,
}

impl<'a, Message> BarChartConfig<'a, Message> {
    pub fn new<S, P, E>(
        file: &'a PathBuf,
        sheet_config: SheetConfigState,
        on_submit: S,
        on_error: E,
        on_previous: P,
        on_cancel: Message,
        on_clear_error: Message,
    ) -> Self
    where
        S: 'a + Fn(View) -> Message,
        E: 'a + Fn(AppError) -> Message,
        P: 'a + Fn(BarChartConfigState) -> Message,
    {
        Self {
            file,
            sheet_config,
            on_submit: Box::new(on_submit),
            on_error: Box::new(on_error),
            on_previous: Box::new(on_previous),
            on_cancel,
            previous_state: None,
            on_clear_error,
        }
    }

    pub fn previous_state(mut self, state: BarChartConfigState) -> Self {
        self.previous_state = Some(state);
        self
    }

    fn update_state(&self, state: &mut BarChartConfigState) {
        if state.use_previous {
            if let Some(previous_state) = self.previous_state.clone() {
                *state = previous_state;
            }
            state.use_previous = false;
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
        let state = if state.use_previous {
            match &self.previous_state {
                Some(prev_state) => prev_state,
                None => state,
            }
        } else {
            state
        };

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

        let order = {
            let check = checkbox("Order", state.order).on_toggle(BarChartConfigMessage::Order);

            let tip = tooltip("Order the bars?");

            row!(check, tip).spacing(25.0)
        };

        let horizontal = {
            let check = checkbox("Horizontal bars?", state.is_horizontal)
                .on_toggle(BarChartConfigMessage::Horizontal);

            let tip = tooltip("Make the bars lie horizontally");

            row!(check, tip).spacing(25.0)
        };

        column!(title, x_col, y_col, axis_label, bar_labels, order, horizontal)
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
            BarChartConfigMessage::Previous => {
                let submit_state = if state.use_previous {
                    match &self.previous_state {
                        Some(prev_state) => prev_state,
                        None => state,
                    }
                } else {
                    state
                };

                Some((self.on_previous)(submit_state.submit()))
            }
            BarChartConfigMessage::Submit => {
                let state = if state.use_previous {
                    match self.previous_state {
                        Some(ref mut prev_state) => prev_state,
                        None => state,
                    }
                } else {
                    state
                };

                state.diff(self.sheet_config.clone());
                let data = BarChartTabData::new(self.file.clone(), state.submit());
                match data {
                    Err(error) => Some((self.on_error)(error)),
                    Ok(data) => {
                        let view = View::BarChart(data);
                        Some((self.on_submit)(view))
                    }
                }
            }
            BarChartConfigMessage::TitleChanged(title) => {
                self.update_state(state);
                state.title = title;
                Some(self.on_clear_error.clone())
            }
            BarChartConfigMessage::AxisLabel(strat) => {
                self.update_state(state);
                let strat = match strat {
                    AxisStrategy::None => BarChartAxisLabelStrategy::None,
                    AxisStrategy::Headers => BarChartAxisLabelStrategy::Headers,
                    AxisStrategy::Provided => BarChartAxisLabelStrategy::Provided {
                        x: String::new(),
                        y: String::new(),
                    },
                };

                state.axis_label = strat;
                Some(self.on_clear_error.clone())
            }
            BarChartConfigMessage::XLabelChanged(label) => {
                self.update_state(state);
                match &state.axis_label {
                    BarChartAxisLabelStrategy::Provided { y, .. } => {
                        state.axis_label = BarChartAxisLabelStrategy::Provided {
                            x: label,
                            y: y.clone(),
                        };
                    }
                    _ => {}
                };
                Some(self.on_clear_error.clone())
            }
            BarChartConfigMessage::YLabelChanged(label) => {
                self.update_state(state);
                match &state.axis_label {
                    BarChartAxisLabelStrategy::Provided { x, .. } => {
                        state.axis_label = BarChartAxisLabelStrategy::Provided {
                            x: x.clone(),
                            y: label,
                        };
                    }
                    _ => {}
                };
                Some(self.on_clear_error.clone())
            }
            BarChartConfigMessage::BarLabel(label) => {
                self.update_state(state);
                let strat = match label {
                    BarLabels::FromColumn => BarChartBarLabels::FromColumn(0),
                    BarLabels::None => BarChartBarLabels::None,
                    BarLabels::Provided => BarChartBarLabels::Provided(vec![]),
                };
                state.bar_label = strat;
                Some(self.on_clear_error.clone())
            }
            BarChartConfigMessage::BarLabelColumn(input) => {
                self.update_state(state);
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

                Some(self.on_clear_error.clone())
            }

            BarChartConfigMessage::XCol(input) => {
                self.update_state(state);
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
                Some(self.on_clear_error.clone())
            }

            BarChartConfigMessage::YCol(input) => {
                self.update_state(state);
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
                Some(self.on_clear_error.clone())
            }
            BarChartConfigMessage::Order(order) => {
                self.update_state(state);
                state.order = order;
                Some(self.on_clear_error.clone())
            }
            BarChartConfigMessage::Horizontal(is_horizontal) => {
                self.update_state(state);
                state.is_horizontal = is_horizontal;
                Some(self.on_clear_error.clone())
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
