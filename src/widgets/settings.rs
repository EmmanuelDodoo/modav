use std::fmt::Debug;

use iced::{
    alignment::Horizontal,
    widget::{
        button, column, component, horizontal_space, pick_list, row, text, vertical_space,
        Component,
    },
    Alignment, Element, Length, Renderer, Theme,
};

use super::style::dialog_container;

const THEMES: [Theme; 5] = [
    Theme::TokyoNight,
    Theme::TokyoNightLight,
    Theme::GruvboxDark,
    Theme::GruvboxLight,
    Theme::Nightfly,
];

#[derive(Debug, Clone)]
pub struct State {
    selected_theme: Option<Theme>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            selected_theme: Some(Theme::TokyoNight),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    ThemeSelected(Theme),
    Submit,
    OpenLogFile,
    Cancel,
}

pub struct SettingsDialog<'a, Message> {
    on_theme_change: Option<Box<dyn Fn(Theme) -> Message + 'a>>,
    current_theme: Theme,
    on_cancel: Option<Message>,
    on_log: Option<Message>,
    on_submit: Option<Box<dyn Fn(Theme) -> Message + 'a>>,
}

impl<'a, Message> SettingsDialog<'a, Message>
where
    Message: Debug + Clone,
{
    pub fn new(current_theme: Theme) -> Self {
        Self {
            current_theme,
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
            .align_items(Alignment::Center)
            .into()
    }

    pub fn on_theme_change(mut self, on_change: impl Fn(Theme) -> Message + 'a) -> Self {
        self.on_theme_change = Some(Box::new(on_change));
        self
    }

    pub fn on_submit(mut self, on_submit: impl Fn(Theme) -> Message + 'a) -> Self {
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
            SettingsMessage::Submit => match (&self.on_submit, &state.selected_theme) {
                (Some(on_submit), Some(theme)) => Some((on_submit)(theme.clone())),
                _ => None,
            },
            SettingsMessage::Cancel => self.on_cancel.clone(),
            SettingsMessage::OpenLogFile => self.on_log.clone(),
            SettingsMessage::ThemeSelected(theme) => {
                state.selected_theme = Some(theme.clone());
                if let Some(on_change) = &self.on_theme_change {
                    Some((on_change)(theme))
                } else {
                    None
                }
            }
        }
    }

    fn view(&self, _state: &Self::State) -> Element<'_, Self::Event, Theme, Renderer> {
        let header = text("Settings Menu")
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center)
            .size(18.0);

        let theme = self.theme();
        let log = button(text("Open Log File").size(15.0)).on_press(SettingsMessage::OpenLogFile);

        let content = column!(theme, log).spacing(20.0);

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
