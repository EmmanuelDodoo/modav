#![allow(deprecated)]
use std::{
    fmt::{self, Debug},
    path::PathBuf,
};

use super::sheet::SheetConfigState;

use crate::{
    utils::{tooltip, AppError},
    views::{StackedBarChartTabData, View},
};
use iced::{
    widget::{
        button, checkbox, column, component, container, horizontal_space, pick_list, row, text,
        text_input, vertical_space, Component, Space,
    },
    Alignment, Element, Renderer, Theme,
};

use modav_core::repr::sheet::utils::{
    HeaderLabelStrategy, HeaderTypesStrategy, StackedBarChartAxisLabelStrategy,
};

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
                Self::Headers => "Provide Y axis label",
                Self::Provided => "Provide labels",
            }
        )
    }
}

impl From<StackedBarChartAxisLabelStrategy> for AxisStrategy {
    fn from(value: StackedBarChartAxisLabelStrategy) -> Self {
        match value {
            StackedBarChartAxisLabelStrategy::None => Self::None,
            StackedBarChartAxisLabelStrategy::Header(_) => Self::Headers,
            StackedBarChartAxisLabelStrategy::Provided { .. } => Self::Provided,
        }
    }
}

#[derive(Debug, Clone)]
pub(super) enum StackedBarChartConfigMessage {
    Previous,
    Cancel,
    Submit,
    TitleChanged(String),
    XCol(String),
    YCol(String),
    AxisLabel(AxisStrategy),
    Order(bool),
    Horizontal(bool),
    XLabelChanged(String),
    YLabelChanged(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StackedBarChartConfigState {
    pub title: String,
    pub x_col: usize,
    pub acc_cols_str: String,
    pub is_horizontal: bool,
    pub use_previous: bool,
    pub order: bool,
    pub axis_label: StackedBarChartAxisLabelStrategy,
    pub trim: bool,
    pub flexible: bool,
    pub header_types: HeaderTypesStrategy,
    pub header_labels: HeaderLabelStrategy,
    pub caption: Option<String>,
}

impl StackedBarChartConfigState {
    pub(super) fn diff(&mut self, sheet_config: SheetConfigState) {
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

impl Default for StackedBarChartConfigState {
    fn default() -> Self {
        Self {
            title: "Untitled".into(),
            x_col: 0,
            acc_cols_str: String::default(),
            is_horizontal: false,
            use_previous: true,
            order: false,
            axis_label: StackedBarChartAxisLabelStrategy::default(),
            trim: true,
            flexible: false,
            header_labels: HeaderLabelStrategy::ReadLabels,
            header_types: HeaderTypesStrategy::Infer,
            caption: None,
        }
    }
}

pub(super) struct StackedBarChartConfig<'a, Message> {
    file: &'a PathBuf,
    sheet_config: SheetConfigState,
    on_submit: Box<dyn Fn(View) -> Message + 'a>,
    on_error: Box<dyn Fn(AppError) -> Message + 'a>,
    on_previous: Box<dyn Fn(StackedBarChartConfigState) -> Message + 'a>,
    on_cancel: Message,
    on_clear_error: Message,
    previous_state: Option<StackedBarChartConfigState>,
}

impl<'a, Message> StackedBarChartConfig<'a, Message> {
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
        P: 'a + Fn(StackedBarChartConfigState) -> Message,
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

    pub fn previous_state(mut self, state: StackedBarChartConfigState) -> Self {
        self.previous_state = Some(state);
        self
    }

    fn actions(&self) -> Element<'_, StackedBarChartConfigMessage> {
        let cancel_btn =
            button(text("Cancel").size(13.0)).on_press(StackedBarChartConfigMessage::Cancel);

        let prev_btn =
            button(text("Back").size(13.0)).on_press(StackedBarChartConfigMessage::Previous);

        let submit = button(text("Open").size(13.0)).on_press(StackedBarChartConfigMessage::Submit);

        let actions = row!(
            cancel_btn,
            horizontal_space(),
            row!(prev_btn, submit).spacing(10.0)
        );

        actions.into()
    }

    fn config(
        &self,
        state: &StackedBarChartConfigState,
    ) -> Element<'_, StackedBarChartConfigMessage> {
        let state = if state.use_previous {
            match &self.previous_state {
                Some(prev_state) => prev_state,
                None => state,
            }
        } else {
            state
        };

        let title = text_input("Graph Title", state.title.as_str())
            .on_input(StackedBarChartConfigMessage::TitleChanged);

        let x_col = {
            let label = text("X column: ");

            let input = {
                let value = state.x_col.to_string();

                text_input("", &value)
                    .on_input(StackedBarChartConfigMessage::XCol)
                    .width(50)
            };

            let tip = tooltip("Column on the x-axis");

            row!(label, input, tip)
                .spacing(15)
                .align_y(Alignment::Center)
        };

        let y_col = {
            let label = text("Y columns: ");

            let input = {
                text_input("", &state.acc_cols_str)
                    .on_input(StackedBarChartConfigMessage::YCol)
                    .width(100)
            };

            let tip = tooltip("Columns to use for the stack, separated by `,`. You can also use `:` to denote a range of columns");

            row!(label, input, tip)
                .spacing(15)
                .align_y(Alignment::Center)
        };

        let axis_label = {
            let label = text("Axis Labels: ");

            let options = [
                AxisStrategy::Headers,
                AxisStrategy::Provided,
                AxisStrategy::None,
            ];

            let selected: AxisStrategy = state.axis_label.clone().into();

            let list = pick_list(
                options,
                Some(selected),
                StackedBarChartConfigMessage::AxisLabel,
            )
            .text_size(13.0);

            let tip = tooltip("How labels for the axis are determined");

            let content = row!(label, list, tip).spacing(8).align_y(Alignment::Center);

            let extra: Element<'_, StackedBarChartConfigMessage> = match &state.axis_label {
                StackedBarChartAxisLabelStrategy::Provided { x, y } => {
                    let x_label = text_input("X axis label", x.as_str())
                        .on_input(StackedBarChartConfigMessage::XLabelChanged);

                    let y_label = text_input("Y axis label", y.as_str())
                        .on_input(StackedBarChartConfigMessage::YLabelChanged);

                    column!(x_label, y_label).spacing(10.0).into()
                }
                StackedBarChartAxisLabelStrategy::Header(y) => {
                    let y_label = text_input("Y axis label", y.as_str())
                        .on_input(StackedBarChartConfigMessage::YLabelChanged);
                    y_label.into()
                }
                StackedBarChartAxisLabelStrategy::None => Space::new(0.0, 0.0).into(),
            };

            column!(content, extra).spacing(10.0)
        };

        let order = {
            let check =
                checkbox("Order", state.order).on_toggle(StackedBarChartConfigMessage::Order);

            let tip = tooltip("Order the bars?");

            row!(check, tip).spacing(25.0)
        };

        let horizontal = {
            let check = checkbox("Horizontal bars?", state.is_horizontal)
                .on_toggle(StackedBarChartConfigMessage::Horizontal);

            let tip = tooltip("Make the bars lie horizontally");

            row!(check, tip).spacing(25.0)
        };

        column!(title, x_col, y_col, axis_label, order, horizontal)
            .spacing(20.0)
            .into()
    }

    fn update_state(&self, state: &mut StackedBarChartConfigState) {
        if state.use_previous {
            if let Some(prevous_state) = self.previous_state.clone() {
                *state = prevous_state;
            }
            state.use_previous = false;
        }
    }
}

impl<'a, Message> Component<Message> for StackedBarChartConfig<'a, Message>
where
    Message: Debug + Clone,
{
    type Event = StackedBarChartConfigMessage;
    type State = StackedBarChartConfigState;

    fn update(&mut self, state: &mut Self::State, event: Self::Event) -> Option<Message> {
        match event {
            StackedBarChartConfigMessage::Cancel => {
                return Some(self.on_cancel.clone());
            }
            StackedBarChartConfigMessage::Previous => {
                let submit_state = if state.use_previous {
                    match &self.previous_state {
                        Some(prev_state) => prev_state,
                        None => state,
                    }
                } else {
                    state
                };

                return Some((self.on_previous)(submit_state.submit()));
            }
            StackedBarChartConfigMessage::Submit => {
                let state = if state.use_previous {
                    match self.previous_state {
                        Some(ref mut prev_state) => prev_state,
                        None => state,
                    }
                } else {
                    state
                };

                state.diff(self.sheet_config.clone());
                let data = StackedBarChartTabData::new(self.file.clone(), state.submit());
                match data {
                    Err(error) => {
                        return Some((self.on_error)(error));
                    }
                    Ok(data) => {
                        let view = View::StackedBarChart(data);
                        return Some((self.on_submit)(view));
                    }
                }
            }
            StackedBarChartConfigMessage::TitleChanged(title) => {
                self.update_state(state);
                state.title = title;
            }
            StackedBarChartConfigMessage::AxisLabel(strat) => {
                self.update_state(state);
                let strat = match strat {
                    AxisStrategy::None => StackedBarChartAxisLabelStrategy::None,
                    AxisStrategy::Headers => {
                        StackedBarChartAxisLabelStrategy::Header(String::default())
                    }
                    AxisStrategy::Provided => StackedBarChartAxisLabelStrategy::Provided {
                        x: String::default(),
                        y: String::default(),
                    },
                };

                state.axis_label = strat;
            }
            StackedBarChartConfigMessage::XLabelChanged(label) => {
                self.update_state(state);
                match &state.axis_label {
                    StackedBarChartAxisLabelStrategy::Provided { y, .. } => {
                        state.axis_label = StackedBarChartAxisLabelStrategy::Provided {
                            x: label,
                            y: y.clone(),
                        };
                    }
                    _ => {}
                }
            }
            StackedBarChartConfigMessage::YLabelChanged(label) => {
                self.update_state(state);
                match &state.axis_label {
                    StackedBarChartAxisLabelStrategy::Provided { x, .. } => {
                        state.axis_label = StackedBarChartAxisLabelStrategy::Provided {
                            x: x.clone(),
                            y: label,
                        };
                    }
                    StackedBarChartAxisLabelStrategy::Header(_) => {
                        state.axis_label = StackedBarChartAxisLabelStrategy::Header(label);
                    }
                    _ => {}
                }
            }
            StackedBarChartConfigMessage::XCol(col) => {
                self.update_state(state);
                let input = col.trim().to_string();
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
            }
            StackedBarChartConfigMessage::YCol(input) => {
                self.update_state(state);

                state.acc_cols_str = input;
            }
            StackedBarChartConfigMessage::Order(order) => {
                self.update_state(state);
                state.order = order;
            }
            StackedBarChartConfigMessage::Horizontal(is_horizontal) => {
                self.update_state(state);
                state.is_horizontal = is_horizontal;
            }
        };
        Some(self.on_clear_error.clone())
    }

    fn view(&self, state: &Self::State) -> Element<'_, Self::Event, Theme, Renderer> {
        let config = self.config(state);

        let content = column!(config, vertical_space().height(50.0), self.actions()).spacing(10.0);

        container(content).into()
    }
}

impl<'a, Message> From<StackedBarChartConfig<'a, Message>> for Element<'a, Message>
where
    Message: Clone + Debug + 'a,
{
    fn from(value: StackedBarChartConfig<'a, Message>) -> Self {
        component(value)
    }
}
