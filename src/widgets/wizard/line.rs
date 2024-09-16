use std::{collections::HashSet, fmt::Debug, path::PathBuf};

use iced::{
    widget::{
        button, column, component, container, horizontal_space, pick_list, row, text, text_input,
        vertical_space, Component,
    },
    Alignment, Element, Renderer, Theme,
};

use modav_core::repr::sheet::utils::{HeaderLabelStrategy, HeaderTypesStrategy, LineLabelStrategy};

use crate::{
    utils::AppError,
    views::{LineTabData, View},
};

use super::{shared::tooltip, sheet::SheetConfigState};

#[derive(Debug, Default, Clone, Copy, PartialEq)]

pub enum LineLabelOptions {
    #[default]
    None,
    FromColumn,
}

impl std::fmt::Display for LineLabelOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::None => "No line labels",
                Self::FromColumn => "Use labels from a column",
            }
        )
    }
}

impl From<LineLabelStrategy> for LineLabelOptions {
    fn from(value: LineLabelStrategy) -> Self {
        match value {
            LineLabelStrategy::None => LineLabelOptions::None,
            LineLabelStrategy::FromCell(_) => LineLabelOptions::FromColumn,
            LineLabelStrategy::Provided(_) => LineLabelOptions::None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConfigMessage {
    TitleChanged(String),
    XLabelChanged(String),
    YLabelChanged(String),
    Error(AppError),
    LineLabelOption(LineLabelOptions),
    LineLabelColumn(String),
    Cancel,
    Previous,
    Submit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineConfigState {
    pub title: String,
    pub x_label: String,
    pub y_label: String,
    pub caption: Option<String>,
    pub label_strat: LineLabelStrategy,
    pub row_exclude: HashSet<usize>,
    pub col_exclude: HashSet<usize>,
    pub trim: bool,
    pub flexible: bool,
    pub header_types: HeaderTypesStrategy,
    pub header_labels: HeaderLabelStrategy,
    pub use_previous: bool,
}

impl Default for LineConfigState {
    fn default() -> Self {
        Self {
            title: "Untitled".into(),
            x_label: String::default(),
            y_label: String::default(),
            caption: None,
            label_strat: LineLabelStrategy::FromCell(0),
            row_exclude: HashSet::default(),
            col_exclude: HashSet::default(),
            trim: true,
            flexible: false,
            header_types: HeaderTypesStrategy::Infer,
            header_labels: HeaderLabelStrategy::ReadLabels,
            use_previous: true,
        }
    }
}

impl LineConfigState {
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

pub struct LineGraphConfig<'a, Message>
where
    Message: Debug + Clone,
{
    file: &'a PathBuf,
    sheet_config: SheetConfigState,
    on_submit: Box<dyn Fn(View) -> Message + 'a>,
    on_error: Box<dyn Fn(AppError) -> Message + 'a>,
    on_previous: Box<dyn Fn(LineConfigState) -> Message + 'a>,
    on_cancel: Message,
    previous_state: Option<LineConfigState>,
    on_clear_error: Message,
}

impl<'a, Message> LineGraphConfig<'a, Message>
where
    Message: Debug + Clone,
{
    pub fn new<S, P, E>(
        file: &'a PathBuf,
        sheet_config: SheetConfigState,
        on_submit: S,
        on_previous: P,
        on_cancel: Message,
        on_error: E,
        on_clear_error: Message,
    ) -> Self
    where
        S: 'a + Fn(View) -> Message,
        P: 'a + Fn(LineConfigState) -> Message,
        E: 'a + Fn(AppError) -> Message,
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

    pub fn previous_state(mut self, state: LineConfigState) -> Self {
        self.previous_state = Some(state);
        self
    }

    fn update_state(&self, state: &mut LineConfigState) {
        if state.use_previous {
            if let Some(previous_state) = self.previous_state.clone() {
                *state = previous_state;
            }
            state.use_previous = false;
        }
    }

    fn actions(&self) -> Element<'_, ConfigMessage> {
        let cancel_btn = button(text("Cancel").size(13.0)).on_press(ConfigMessage::Cancel);

        let prev_btn = button(text("Back").size(13.0)).on_press(ConfigMessage::Previous);

        let submit = button(text("Open").size(13.0)).on_press(ConfigMessage::Submit);

        let actions = row!(
            cancel_btn,
            horizontal_space(),
            row!(prev_btn, submit).spacing(10.0)
        );

        actions.into()
    }

    fn line_config(&self, state: &LineConfigState) -> Element<'_, ConfigMessage> {
        let state = if state.use_previous {
            match &self.previous_state {
                Some(prev_state) => prev_state,
                None => state,
            }
        } else {
            state
        };

        let title =
            text_input("Graph Title", state.title.as_str()).on_input(ConfigMessage::TitleChanged);

        let x_label = text_input("X axis label", state.x_label.as_str())
            .on_input(ConfigMessage::XLabelChanged);

        let y_label = text_input("Y axis label", state.y_label.as_str())
            .on_input(ConfigMessage::YLabelChanged);

        let line_labels = {
            let label = text("Line Labels: ");

            let options = [LineLabelOptions::None, LineLabelOptions::FromColumn];

            let selected: LineLabelOptions = state.label_strat.clone().into();

            let list =
                pick_list(options, Some(selected), ConfigMessage::LineLabelOption).text_size(13.0);

            let tip = tooltip("How the labels for the lines are determined");

            let input = {
                let value = match &state.label_strat {
                    LineLabelStrategy::FromCell(col) => col.to_string(),
                    _ => "-".to_string(),
                };

                text_input("", &value)
                    .on_input(ConfigMessage::LineLabelColumn)
                    .width(40.0)
            };

            row!(label, list, input, tip)
                .spacing(8)
                .align_items(Alignment::Center)
        };

        column!(title, x_label, y_label, line_labels)
            .spacing(20.0)
            .into()
    }
}

impl<'a, Message> Component<Message> for LineGraphConfig<'a, Message>
where
    Message: Debug + Clone,
{
    type State = LineConfigState;
    type Event = ConfigMessage;

    fn update(&mut self, state: &mut Self::State, event: Self::Event) -> Option<Message> {
        match event {
            ConfigMessage::Error(err) => Some((self.on_error)(err)),
            ConfigMessage::TitleChanged(title) => {
                self.update_state(state);
                state.title = title;
                Some(self.on_clear_error.clone())
            }
            ConfigMessage::XLabelChanged(label) => {
                self.update_state(state);
                state.x_label = label;
                Some(self.on_clear_error.clone())
            }
            ConfigMessage::YLabelChanged(label) => {
                self.update_state(state);
                state.y_label = label;
                Some(self.on_clear_error.clone())
            }
            ConfigMessage::Cancel => Some(self.on_cancel.clone()),
            ConfigMessage::Previous => {
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
            ConfigMessage::LineLabelOption(option) => {
                self.update_state(state);
                let strat = match option {
                    LineLabelOptions::None => LineLabelStrategy::None,
                    LineLabelOptions::FromColumn => LineLabelStrategy::FromCell(0),
                };
                state.label_strat = strat;
                Some(self.on_clear_error.clone())
            }
            ConfigMessage::LineLabelColumn(input) => {
                self.update_state(state);
                if let LineLabelStrategy::FromCell(col) = state.label_strat {
                    let input = input.trim().to_string();
                    let col = if input.is_empty() {
                        0
                    } else {
                        let first = input.chars().next().unwrap();

                        let mut input = input;
                        if first == '-' {
                            input = input.replace("-", "");
                        }

                        if col == 0 && first != '0' {
                            input.pop();
                        }

                        input.parse().unwrap_or_default()
                    };
                    state.label_strat = LineLabelStrategy::FromCell(col)
                };
                Some(self.on_clear_error.clone())
            }
            ConfigMessage::Submit => {
                let state = if state.use_previous {
                    match self.previous_state {
                        Some(ref mut prev_state) => prev_state,
                        None => state,
                    }
                } else {
                    state
                };

                state.diff(self.sheet_config.clone());
                let data = LineTabData::new(self.file.clone(), state.clone());
                match data {
                    Err(err) => Some((self.on_error)(err)),
                    Ok(data) => {
                        let view = View::LineGraph(data);

                        Some((self.on_submit)(view))
                    }
                }
            }
        }
    }

    fn view(&self, state: &Self::State) -> Element<'_, Self::Event, Theme, Renderer> {
        let config = self.line_config(state);
        let content = column!(config, vertical_space().height(50.0), self.actions()).spacing(10.0);

        container(content).into()
    }
}

impl<'a, Message> From<LineGraphConfig<'a, Message>> for Element<'a, Message>
where
    Message: 'a + Debug + Clone,
{
    fn from(value: LineGraphConfig<'a, Message>) -> Self {
        component(value)
    }
}
