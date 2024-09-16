use std::fmt::Debug;

use iced::{
    widget::{
        button, checkbox, column, component, container, horizontal_space, pick_list, row, text,
        text_input, vertical_space, Component,
    },
    Alignment, Element, Renderer, Theme,
};

use modav_core::repr::sheet::utils::{HeaderLabelStrategy, HeaderTypesStrategy};

use super::shared::tooltip;

#[derive(Debug, Clone)]
pub struct SheetConfigState {
    pub trim: bool,
    pub flexible: bool,
    pub header_type: HeaderTypesStrategy,
    pub header_labels: HeaderLabelStrategy,
    pub caption: Option<String>,
    pub use_previous: bool,
}

impl SheetConfigState {
    fn submit(&self) -> Self {
        Self {
            trim: self.trim,
            flexible: self.flexible,
            header_labels: self.header_labels.clone(),
            header_type: self.header_type.clone(),
            caption: self.caption.clone(),
            use_previous: true,
        }
    }
}

impl Default for SheetConfigState {
    fn default() -> Self {
        Self {
            trim: true,
            flexible: false,
            header_labels: HeaderLabelStrategy::ReadLabels,
            header_type: HeaderTypesStrategy::Infer,
            caption: None,
            use_previous: true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SheetConfigMessage {
    Cancel,
    Previous,
    Submit,
    TrimToggled(bool),
    FlexibleToggled(bool),
    HeaderTypeChanged(HeaderTypesStrategy),
    HeaderLabelChanged(HeaderLabelStrategy),
    CaptionChange(String),
}

pub struct SheetConfig<'a, Message> {
    on_submit: Box<dyn Fn(SheetConfigState) -> Message + 'a>,
    on_previous: Box<dyn Fn(SheetConfigState) -> Message + 'a>,
    on_cancel: Message,
    previous_state: Option<SheetConfigState>,
    on_clear_error: Message,
}

impl<'a, Message> SheetConfig<'a, Message> {
    pub fn new<S, P>(
        on_submit: S,
        on_previous: P,
        on_cancel: Message,
        on_clear_error: Message,
    ) -> Self
    where
        S: 'a + Fn(SheetConfigState) -> Message,
        P: 'a + Fn(SheetConfigState) -> Message,
    {
        Self {
            on_cancel,
            on_previous: Box::new(on_previous),
            on_submit: Box::new(on_submit),
            previous_state: None,
            on_clear_error,
        }
    }

    pub fn previous_state(mut self, state: SheetConfigState) -> Self {
        self.previous_state = Some(state);
        self
    }

    fn update_state(&self, state: &mut SheetConfigState) {
        if state.use_previous {
            if let Some(previous_state) = self.previous_state.clone() {
                *state = previous_state;
            }
            state.use_previous = false;
        };
    }

    fn actions(&self) -> Element<'_, SheetConfigMessage> {
        let cancel_btn = button(text("Cancel").size(13.0)).on_press(SheetConfigMessage::Cancel);

        let prev_btn = button(text("Back").size(13.0)).on_press(SheetConfigMessage::Previous);

        let submit = button(text("Next").size(13.0)).on_press(SheetConfigMessage::Submit);

        let actions = row!(
            cancel_btn,
            horizontal_space(),
            row!(prev_btn, submit).spacing(10.0)
        );

        actions.into()
    }

    fn sheet_config(&self, state: &SheetConfigState) -> Element<'_, SheetConfigMessage> {
        let state = if state.use_previous {
            match &self.previous_state {
                Some(prev_state) => prev_state,
                None => state,
            }
        } else {
            state
        };

        let trim = {
            let check = checkbox("Trim?", state.trim).on_toggle(SheetConfigMessage::TrimToggled);

            let tip = tooltip("Remove leading and trailing whitespace for each cell");

            row!(check, tip).spacing(25.0)
        };

        let flexible = {
            let check = checkbox("Flexible?", state.flexible)
                .on_toggle(SheetConfigMessage::FlexibleToggled);

            let tip = tooltip("Handle unequal row lengths");

            row!(check, tip).spacing(25.0)
        };

        let header_types = {
            let label = text("Column Types:");

            let options = [HeaderTypesStrategy::None, HeaderTypesStrategy::Infer];

            let list = pick_list(
                options,
                Some(state.header_type.clone()),
                SheetConfigMessage::HeaderTypeChanged,
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
                SheetConfigMessage::HeaderLabelChanged,
            )
            .text_size(13.0);

            let tip = tooltip("How the header labels are handled");

            row!(label, list, tip)
                .spacing(8)
                .align_items(Alignment::Center)
        };

        let caption = text_input(
            "Graph Caption",
            state
                .caption
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or_default(),
        )
        .on_input(SheetConfigMessage::CaptionChange);

        column!(trim, flexible, header_labels, header_types, caption)
            .align_items(Alignment::Start)
            .spacing(30.0)
            .into()
    }
}

impl<'a, Message> Component<Message> for SheetConfig<'a, Message>
where
    Message: Debug + Clone,
{
    type State = SheetConfigState;
    type Event = SheetConfigMessage;

    fn update(&mut self, state: &mut Self::State, event: Self::Event) -> Option<Message> {
        match event {
            SheetConfigMessage::Cancel => Some(self.on_cancel.clone()),
            SheetConfigMessage::Previous => {
                let submit_state = if state.use_previous {
                    match &self.previous_state {
                        Some(previous_state) => previous_state,
                        None => state,
                    }
                } else {
                    state
                };
                Some((self.on_previous)(submit_state.submit()))
            }
            SheetConfigMessage::Submit => {
                let submit_state = if state.use_previous {
                    match &self.previous_state {
                        Some(previous_state) => previous_state,
                        None => state,
                    }
                } else {
                    state
                };
                Some((self.on_submit)(submit_state.submit()))
            }
            SheetConfigMessage::TrimToggled(trim) => {
                self.update_state(state);
                state.trim = trim;
                Some(self.on_clear_error.clone())
            }
            SheetConfigMessage::FlexibleToggled(flexible) => {
                self.update_state(state);
                state.flexible = flexible;
                Some(self.on_clear_error.clone())
            }
            SheetConfigMessage::HeaderTypeChanged(ht) => {
                self.update_state(state);
                state.header_type = ht;
                Some(self.on_clear_error.clone())
            }
            SheetConfigMessage::HeaderLabelChanged(label) => {
                self.update_state(state);
                state.header_labels = label;
                Some(self.on_clear_error.clone())
            }
            SheetConfigMessage::CaptionChange(caption) => {
                self.update_state(state);
                if !caption.is_empty() {
                    state.caption = Some(caption);
                } else {
                    state.caption = None;
                }

                Some(self.on_clear_error.clone())
            }
        }
    }

    fn view(&self, state: &Self::State) -> Element<'_, Self::Event, Theme, Renderer> {
        let config = self.sheet_config(state);

        let content = column!(config, vertical_space().height(50.0), self.actions()).spacing(10.0);

        container(content).into()
    }
}

impl<'a, Message> From<SheetConfig<'a, Message>> for Element<'a, Message>
where
    Message: 'a + Debug + Clone,
{
    fn from(value: SheetConfig<'a, Message>) -> Self {
        component(value)
    }
}
