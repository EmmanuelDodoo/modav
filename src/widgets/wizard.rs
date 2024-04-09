use crate::views::{EditorTabData, ModelTabData, View};

use crate::utils::icon;
use crate::ViewType;
use iced::{
    alignment::{Alignment, Horizontal, Vertical},
    theme,
    widget::{
        self, button, column, component, container, horizontal_space, pick_list, row, text,
        vertical_space, Component, Space,
    },
    Border, Element, Length, Theme,
};
use std::{fmt::Debug, path::PathBuf};

#[derive(Debug, Clone)]
pub struct Hex {
    model: ViewType,
    config_view: bool,
    config: View,
}

impl Default for Hex {
    fn default() -> Self {
        Self {
            model: ViewType::Editor,
            config_view: false,
            config: View::Editor(EditorTabData::default()),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum Charm {
    ReselectFile,
    ModelSelected(ViewType),
    ViewConfig,
    ViewDefault,
    Cancel,
    Submit,
    #[default]
    None,
}

pub struct Wizard<Message>
where
    Message: Debug + Clone,
{
    on_reselect_file: Option<Message>,
    on_cancel: Option<Message>,
    on_submit: Box<dyn Fn(PathBuf, View) -> Message>,
    file: PathBuf,
}

impl<Message> Wizard<Message>
where
    Message: Debug + Clone,
{
    pub fn new<F>(file: PathBuf, on_submit: F) -> Self
    where
        F: 'static + Fn(PathBuf, View) -> Message,
    {
        Self {
            file,
            on_reselect_file: None,
            on_submit: Box::new(on_submit),
            on_cancel: None,
        }
    }

    pub fn on_reselect(mut self, msg: Message) -> Self {
        self.on_reselect_file = Some(msg);
        self
    }

    pub fn on_cancel(mut self, msg: Message) -> Self {
        self.on_cancel = Some(msg);
        self
    }

    fn model_config(&self, state: &Hex) -> Element<'_, Charm> {
        match &state.model {
            ViewType::Editor => Space::new(0, 0).into(),
            ViewType::Counter => Space::new(0, 0).into(),
            ViewType::Model => Space::new(0, 0).into(),
            ViewType::None => Space::new(0, 0).into(),
        }
    }

    fn actions(&self, state: &Hex) -> Element<'_, Charm> {
        let cancel = button(text("Cancel").size(13.0)).on_press(Charm::Cancel);

        let next = if state.model.has_config() && state.config_view {
            let prev = button(text("Previous").size(13.0)).on_press(Charm::ViewDefault);
            let submit = button(text("Open").size(13.0));
            row!(prev, submit).spacing(10.0)
        } else if state.model.has_config() {
            row!(button(text("Next").size(13.0)).on_press(Charm::ViewConfig))
        } else {
            row!(button(text("Open").size(13.0)).on_press(Charm::Submit))
        };

        row!(cancel, horizontal_space(), next).into()
    }

    fn default_view(&self, state: &Hex) -> Element<'_, Charm> {
        let file = {
            let label = text("File:").width(Length::FillPortion(1));

            let file_name = self
                .file
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("No File");

            let file_name = if file_name.len() > 8 {
                format!("{}...", &file_name[0..8])
            } else {
                file_name.to_string()
            };

            let file = container(text(file_name))
                .padding([8, 8])
                .center_y()
                .width(Length::FillPortion(8))
                .style(theme::Container::Custom(Box::new(FileBorderContainer)));

            let btn = button(
                icon('\u{E802}', "wizard-icons")
                    .vertical_alignment(Vertical::Center)
                    .size(15.0),
            )
            .height(30.0)
            .width(Length::FillPortion(1))
            .on_press(Charm::ReselectFile);

            let temp = row!(label, file)
                .align_items(Alignment::Center)
                .spacing(46)
                .width(Length::FillPortion(5));

            row!(temp, horizontal_space(), btn)
                .width(Length::Fill)
                .align_items(Alignment::Center)
        };

        let options = {
            let label = text("Model:");

            let list = pick_list(ViewType::WIZARD, Some(state.model), Charm::ModelSelected);

            row!(label, list).spacing(8).align_items(Alignment::Center)
        };

        let top = column!(file, options).spacing(30);

        top.into()
    }
}

impl<Message> Component<Message> for Wizard<Message>
where
    Message: Debug + Clone,
{
    type State = Hex;
    type Event = Charm;

    fn update(&mut self, state: &mut Self::State, event: Self::Event) -> Option<Message> {
        match event {
            Charm::ReselectFile => self.on_reselect_file.clone(),
            Charm::ModelSelected(model) => {
                state.model = model;
                state.config = match state.model {
                    ViewType::Editor => {
                        let data = EditorTabData::new(Some(self.file.clone()), String::default());
                        View::Editor(data)
                    }
                    ViewType::Model => {
                        let data = ModelTabData::new(self.file.clone());
                        View::Model(data)
                    }
                    ViewType::Counter => View::Counter,
                    ViewType::None => View::None,
                };
                None
            }
            Charm::ViewConfig => {
                state.config_view = true;
                None
            }
            Charm::ViewDefault => {
                state.config_view = false;
                None
            }
            Charm::Cancel => self.on_cancel.clone(),
            Charm::Submit => Some((self.on_submit)(self.file.clone(), state.config.clone())),
            Charm::None => None,
        }
    }

    fn view(&self, state: &Self::State) -> Element<'_, Self::Event> {
        let header = text("File Wizard")
            .size(18.0)
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center);

        let content = if state.config_view {
            column!(
                header,
                vertical_space(),
                self.model_config(state),
                vertical_space(),
                self.actions(state)
            )
        } else {
            column!(
                header,
                vertical_space(),
                self.default_view(state),
                vertical_space(),
                self.actions(state)
            )
        };

        container(content)
            .padding([20.0, 25.0])
            .width(300.0)
            .height(350.0)
            .style(theme::Container::Custom(Box::new(WizardContainer::new(
                10.0,
            ))))
            .into()
    }
}

impl<'a, Message> From<Wizard<Message>> for Element<'a, Message>
where
    Message: 'a + Clone + Debug,
{
    fn from(value: Wizard<Message>) -> Self {
        component(value)
    }
}

pub struct FileBorderContainer;

impl widget::container::StyleSheet for FileBorderContainer {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        let border_color = style.extended_palette().primary.weak.color;

        let border = Border {
            color: border_color,
            width: 1.0,
            ..Default::default()
        };

        container::Appearance {
            border,
            ..Default::default()
        }
    }
}

pub struct WizardContainer {
    pub radius: f32,
}

impl WizardContainer {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

impl widget::container::StyleSheet for WizardContainer {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        let border = Border {
            radius: self.radius.into(),
            ..Default::default()
        };

        let background = style.extended_palette().background.base.color;

        container::Appearance {
            background: Some(background.into()),
            border,
            ..Default::default()
        }
    }
}
