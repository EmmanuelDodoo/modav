use std::{collections::HashSet, fmt::Debug, path::PathBuf};

use crate::{
    utils::AppError,
    views::{LineTabData, View},
};

use tip::tooltip;

use iced::{
    widget::{
        button, checkbox, column, component, container, horizontal_space, pick_list, row, text,
        text_input, vertical_space, Component,
    },
    Alignment, Element, Renderer, Theme,
};

use modav_core::repr::sheet::utils::{HeaderLabelStrategy, HeaderTypesStrategy, LineLabelStrategy};

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
    TrimToggled(bool),
    FlexibleTogglged(bool),
    HeaderTypeChanged(HeaderTypesStrategy),
    HeaderLabelChanged(HeaderLabelStrategy),
    Error(AppError),
    LineLabelOption(LineLabelOptions),
    LineLabelColumn(String),
    Next,
    Cancel,
    Previous,
    Submit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineConfigState {
    pub title: String,
    pub x_label: String,
    pub y_label: String,
    pub label_strat: LineLabelStrategy,
    pub row_exclude: HashSet<usize>,
    pub col_exclude: HashSet<usize>,
    is_line_view: bool,
    pub trim: bool,
    pub flexible: bool,
    pub header_types: HeaderTypesStrategy,
    pub header_labels: HeaderLabelStrategy,
}

impl Default for LineConfigState {
    fn default() -> Self {
        Self {
            title: "Untitled".into(),
            x_label: String::default(),
            y_label: String::default(),
            label_strat: LineLabelStrategy::None,
            row_exclude: HashSet::default(),
            col_exclude: HashSet::default(),
            is_line_view: false,
            trim: true,
            flexible: false,
            header_types: HeaderTypesStrategy::Infer,
            header_labels: HeaderLabelStrategy::ReadLabels,
        }
    }
}

pub struct LineGraphConfig<'a, Message>
where
    Message: Debug + Clone,
{
    file: &'a PathBuf,
    on_submit: Box<dyn Fn(View) -> Message + 'a>,
    on_error: Box<dyn Fn(AppError) -> Message + 'a>,
    on_previous: Message,
    on_cancel: Message,
}

impl<'a, Message> LineGraphConfig<'a, Message>
where
    Message: Debug + Clone,
{
    pub fn new<F, E>(
        file: &'a PathBuf,
        on_submit: F,
        on_previous: Message,
        on_cancel: Message,
        on_error: E,
    ) -> Self
    where
        F: 'a + Fn(View) -> Message,
        E: 'a + Fn(AppError) -> Message,
    {
        Self {
            file,
            on_submit: Box::new(on_submit),
            on_error: Box::new(on_error),
            on_previous,
            on_cancel,
        }
    }

    fn actions(&self, state: &LineConfigState) -> Element<'_, ConfigMessage> {
        let cancel_btn = button(text("Cancel").size(13.0)).on_press(ConfigMessage::Cancel);

        let prev_btn = button(text("Back").size(13.0)).on_press(ConfigMessage::Previous);

        let submit = if state.is_line_view {
            button(text("Open").size(13.0)).on_press(ConfigMessage::Submit)
        } else {
            button(text("Next").size(13.0)).on_press(ConfigMessage::Next)
        };

        let actions = row!(
            cancel_btn,
            horizontal_space(),
            row!(prev_btn, submit).spacing(10.0)
        );

        actions.into()
    }

    fn line_config(&self, state: &LineConfigState) -> Element<'_, ConfigMessage> {
        let title =
            text_input("Graph Title", state.title.as_str()).on_input(ConfigMessage::TitleChanged);

        let x_label = text_input("X axis label", state.x_label.as_str())
            .on_input(ConfigMessage::XLabelChanged);

        let y_label = text_input("Y axis label", state.y_label.as_str())
            .on_input(ConfigMessage::YLabelChanged);

        column!(title, x_label, y_label).spacing(10.0).into()
    }

    fn sheet_config(&self, state: &LineConfigState) -> Element<'_, ConfigMessage> {
        let trim = {
            let check = checkbox("Trim?", state.trim).on_toggle(ConfigMessage::TrimToggled);

            let tip = tooltip("Remove leading and trailing whitespace for each cell");

            row!(check, tip).spacing(25.0)
        };

        let flexible = {
            let check =
                checkbox("Flexible?", state.flexible).on_toggle(ConfigMessage::FlexibleTogglged);

            let tip = tooltip("Handle unequal row lengths");

            row!(check, tip).spacing(25.0)
        };

        let header_types = {
            let label = text("Column Types:");

            let options = [HeaderTypesStrategy::None, HeaderTypesStrategy::Infer];

            let list = pick_list(
                options,
                Some(state.header_types.clone()),
                ConfigMessage::HeaderTypeChanged,
            )
            .text_size(13.0);

            let tip = tooltip("How the types for each column are handled");

            row!(label, list, tip)
                .spacing(8)
                .align_items(Alignment::Center)
        };

        let header_labels = {
            let label = text("Header labels: ");

            let options = [
                HeaderLabelStrategy::NoLabels,
                HeaderLabelStrategy::ReadLabels,
            ];

            let list = pick_list(
                options,
                Some(state.header_labels.clone()),
                ConfigMessage::HeaderLabelChanged,
            )
            .text_size(13.0);

            let tip = tooltip("How the header labels are handled");

            row!(label, list, tip)
                .spacing(8)
                .align_items(Alignment::Center)
        };

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

        column!(trim, flexible, header_labels, header_types, line_labels,)
            .align_items(Alignment::Start)
            .spacing(30.0)
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
            ConfigMessage::HeaderTypeChanged(ht) => {
                state.header_types = ht;
                None
            }
            ConfigMessage::HeaderLabelChanged(hl) => {
                state.header_labels = hl;
                None
            }
            ConfigMessage::FlexibleTogglged(val) => {
                state.flexible = val;
                None
            }
            ConfigMessage::TitleChanged(title) => {
                state.title = title;
                None
            }
            ConfigMessage::XLabelChanged(label) => {
                state.x_label = label;
                None
            }
            ConfigMessage::YLabelChanged(label) => {
                state.y_label = label;
                None
            }
            ConfigMessage::Cancel => Some(self.on_cancel.clone()),
            ConfigMessage::Previous if state.is_line_view => {
                state.is_line_view = false;
                None
            }
            ConfigMessage::Previous => Some(self.on_previous.clone()),
            ConfigMessage::Next => {
                state.is_line_view = true;
                None
            }
            ConfigMessage::TrimToggled(val) => {
                state.trim = val;
                None
            }

            ConfigMessage::LineLabelOption(option) => {
                let strat = match option {
                    LineLabelOptions::None => LineLabelStrategy::None,
                    LineLabelOptions::FromColumn => LineLabelStrategy::FromCell(0),
                };
                state.label_strat = strat;
                None
            }
            ConfigMessage::LineLabelColumn(input) => {
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
                None
            }
            ConfigMessage::Submit => {
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
        let config = if state.is_line_view {
            self.line_config(state)
        } else {
            self.sheet_config(state)
        };
        let content =
            column!(config, vertical_space().height(50.0), self.actions(state)).spacing(10.0);

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

mod tip {
    use crate::{utils::icons, ToolTipContainerStyle};

    use iced::{
        alignment, theme,
        widget::{container, text, tooltip::Tooltip},
        Length,
    };

    use iced::widget::tooltip as tt;

    pub fn tooltip<'a, Message>(description: impl ToString) -> Tooltip<'a, Message>
    where
        Message: 'a,
    {
        let text = text(description).size(13.0);
        let desc = container(text)
            .max_width(200.0)
            .padding([6.0, 8.0])
            .height(Length::Shrink)
            .style(theme::Container::Custom(Box::new(ToolTipContainerStyle)));

        let icon = icons::icon(icons::HELP)
            .horizontal_alignment(alignment::Horizontal::Center)
            .vertical_alignment(alignment::Vertical::Center);

        Tooltip::new(icon, desc, tt::Position::Right)
            .gap(10.0)
            .snap_within_viewport(true)
    }
}
