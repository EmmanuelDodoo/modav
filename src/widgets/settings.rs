#![allow(deprecated)]
use std::fmt::Debug;

use iced::{
    alignment::Horizontal,
    widget::{
        button, column, component, horizontal_space, pick_list, row, text, text_input,
        vertical_space, Component,
    },
    Alignment, Element, Length, Renderer, Theme,
};

use super::style::dialog_container;

const THEMES: [Theme; 7] = [
    Theme::TokyoNight,
    Theme::TokyoNightLight,
    Theme::GruvboxDark,
    Theme::GruvboxLight,
    Theme::SolarizedDark,
    Theme::SolarizedLight,
    Theme::Nightfly,
];

#[derive(Debug, Clone)]
pub struct State {
    selected_theme: Theme,
    theme_changed: bool,
    toast_timeout: Option<u64>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            selected_theme: Theme::TokyoNight,
            theme_changed: false,
            toast_timeout: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    ThemeSelected(Theme),
    Submit,
    TimeOutChanged(String),
    OpenLogFile,
    Cancel,
}

pub struct SettingsDialog<'a, Message> {
    on_theme_change: Option<Box<dyn Fn(Theme) -> Message + 'a>>,
    current_theme: Theme,
    current_timeout: Option<u64>,
    on_cancel: Option<Message>,
    on_log: Option<Message>,
    on_submit: Option<Box<dyn Fn(Theme, u64) -> Message + 'a>>,
}

impl<'a, Message> SettingsDialog<'a, Message>
where
    Message: Debug + Clone,
{
    pub fn new(current_theme: Theme, current_timeout: u64) -> Self {
        Self {
            current_theme,
            current_timeout: Some(current_timeout),
            on_theme_change: None,
            on_cancel: None,
            on_submit: None,
            on_log: None,
        }
    }

    fn actions(&self) -> Element<'_, SettingsMessage> {
        let cancel = button(text("Cancel").size(13.0)).on_press(SettingsMessage::Cancel);

        let submit = button(text("Save").size(13.0)).on_press(SettingsMessage::Submit);

        row!(cancel, horizontal_space(), submit).into()
    }

    fn theme(&self) -> Element<'_, SettingsMessage> {
        let label = text("Change Theme:");

        let pick_list = pick_list(
            THEMES,
            Some(self.current_theme.clone()),
            SettingsMessage::ThemeSelected,
        );

        row!(label, pick_list)
            .spacing(10)
            .align_y(Alignment::Center)
            .into()
    }

    pub fn on_theme_change(mut self, on_change: impl Fn(Theme) -> Message + 'a) -> Self {
        self.on_theme_change = Some(Box::new(on_change));
        self
    }

    pub fn on_submit(mut self, on_submit: impl Fn(Theme, u64) -> Message + 'a) -> Self {
        self.on_submit = Some(Box::new(on_submit));
        self
    }

    pub fn on_cancel(mut self, on_cancel: Message) -> Self {
        self.on_cancel = Some(on_cancel);
        self
    }

    pub fn on_log(mut self, on_log: Message) -> Self {
        self.on_log = Some(on_log);
        self
    }
}

impl<'a, Message> Component<Message, Theme, Renderer> for SettingsDialog<'a, Message>
where
    Message: Debug + Clone,
{
    type State = State;
    type Event = SettingsMessage;

    fn update(&mut self, state: &mut Self::State, event: Self::Event) -> Option<Message> {
        match event {
            SettingsMessage::Submit => match &self.on_submit {
                Some(on_submit) => {
                    let timeout = state
                        .toast_timeout
                        .filter(|time| time > &0)
                        .unwrap_or(self.current_timeout.unwrap());

                    let theme = if state.theme_changed {
                        state.selected_theme.clone()
                    } else {
                        self.current_theme.clone()
                    };

                    Some((on_submit)(theme, timeout))
                }
                _ => None,
            },
            SettingsMessage::Cancel => self.on_cancel.clone(),
            SettingsMessage::OpenLogFile => self.on_log.clone(),
            SettingsMessage::TimeOutChanged(mut timeout) => {
                if !timeout.is_empty() {
                    if let Some(first) = timeout.chars().next() {
                        if state.toast_timeout == Some(0) && first != '0' {
                            timeout.pop();
                        }
                    }
                    state.toast_timeout = timeout.parse().ok();
                } else {
                    state.toast_timeout = Some(0);
                }

                None
            }
            SettingsMessage::ThemeSelected(theme) => {
                state.selected_theme = theme.clone();
                state.theme_changed = true;
                if let Some(on_change) = &self.on_theme_change {
                    Some((on_change)(theme))
                } else {
                    None
                }
            }
        }
    }

    fn view(&self, state: &Self::State) -> Element<'_, Self::Event, Theme, Renderer> {
        let header = text("Settings Menu")
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .size(18.0);

        let theme = self.theme();

        let timeout = {
            let value = if state.toast_timeout.is_some() {
                state.toast_timeout.as_ref()
            } else {
                self.current_timeout.as_ref()
            };

            let label = text("Toast timeout:");

            let input = text_input("", value.map(u64::to_string).as_deref().unwrap_or(""))
                .on_input(SettingsMessage::TimeOutChanged)
                .width(40.0);

            row!(
                label,
                row!(input, text("seconds"))
                    .align_y(Alignment::Center)
                    .spacing(5)
            )
            .spacing(20.0)
        };

        let log = button(text("Open Log File").size(15.0)).on_press(SettingsMessage::OpenLogFile);

        let content = column!(theme, timeout, log).spacing(20.0);

        let content = column!(
            header,
            vertical_space().height(Length::Fixed(25.0)),
            content,
            vertical_space(),
            self.actions(),
        )
        .height(Length::Fill);

        dialog_container(content).into()
    }
}

impl<'a, Message> From<SettingsDialog<'a, Message>> for Element<'a, Message>
where
    Message: 'a + Clone + Debug,
{
    fn from(value: SettingsDialog<'a, Message>) -> Self {
        component(value)
    }
}
