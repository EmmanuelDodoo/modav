use std::{fmt::Debug, path::PathBuf};

use barchart::BarChartConfig;
use iced::{
    alignment::{Alignment, Horizontal, Vertical},
    theme,
    widget::{
        self, button, column, component, container, horizontal_space, pick_list, row, text,
        vertical_space, Component, Space,
    },
    Border, Element, Length, Theme,
};

use crate::views::{BarChartTabData, EditorTabData, FileType, LineTabData, View};

use crate::utils::{icons, AppError};
use crate::ViewType;

use super::style::dialog_container;

mod line;
pub use line::LineConfigState;
use line::LineGraphConfig;

mod barchart;
pub use barchart::BarChartConfigState;

mod sheet;
use sheet::{SheetConfig, SheetConfigState};

pub mod shared;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Portal {
    #[default]
    FileSelection,
    SheetConfig,
    ModelConfig,
}

#[derive(Debug, Clone)]
pub struct Hex {
    model: ViewType,
    current_view: Portal,
    config: View,
    sheet_config: SheetConfigState,
    line_config: Option<LineConfigState>,
    bar_config: Option<BarChartConfigState>,
}

impl Default for Hex {
    fn default() -> Self {
        Self {
            model: ViewType::Editor,
            current_view: Portal::FileSelection,
            config: View::Editor(EditorTabData::default()),
            sheet_config: SheetConfigState::default(),
            line_config: None,
            bar_config: None,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub enum Charm {
    ReselectFile,
    ModelSelected(ViewType),
    ViewConfig,
    ChangeView(Portal),
    Cancel,
    ConfigSubmit(View),
    SheetSubmit(SheetConfigState),
    SheetPrevious(SheetConfigState),
    LinePrevious(LineConfigState),
    BarChartPrevious(BarChartConfigState),
    Error(AppError),
    Submit,
    #[default]
    None,
}

pub struct Wizard<'a, Message>
where
    Message: Debug + Clone,
{
    on_reselect_file: Option<Message>,
    on_cancel: Option<Message>,
    on_error: Box<dyn Fn(AppError) -> Message + 'a>,
    on_submit: Box<dyn Fn(PathBuf, View) -> Message + 'a>,
    file: PathBuf,
}

impl<'a, Message> Wizard<'a, Message>
where
    Message: Debug + Clone,
{
    pub fn new<F, E>(file: PathBuf, on_submit: F, on_error: E) -> Self
    where
        F: 'a + Fn(PathBuf, View) -> Message,
        E: 'a + Fn(AppError) -> Message,
    {
        Self {
            file,
            on_reselect_file: None,
            on_submit: Box::new(on_submit),
            on_error: Box::new(on_error),
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
            ViewType::LineGraph => {
                let mut content = LineGraphConfig::new(
                    &self.file,
                    state.sheet_config.clone(),
                    Charm::ConfigSubmit,
                    Charm::LinePrevious,
                    Charm::Cancel,
                    Charm::Error,
                );

                if let Some(line_config) = state.line_config.clone() {
                    content = content.previous_state(line_config);
                };

                content.into()
            }
            ViewType::BarChart => {
                let mut content = BarChartConfig::new(
                    &self.file,
                    state.sheet_config.clone(),
                    Charm::ConfigSubmit,
                    Charm::Error,
                    Charm::BarChartPrevious,
                    Charm::Cancel,
                );

                if let Some(barchart_config) = state.bar_config.clone() {
                    content = content.previous_state(barchart_config);
                }

                content.into()
            }
            ViewType::None => Space::new(0, 0).into(),
        }
    }

    fn actions(&self, state: &Hex) -> Element<'_, Charm> {
        let cancel = button(text("Cancel").size(13.0)).on_press(Charm::Cancel);

        let action = if state.model.has_config() {
            row!(button(text("Next").size(13.0)).on_press(Charm::ChangeView(Portal::SheetConfig)))
        } else {
            row!(button(text("Open").size(13.0)).on_press(Charm::Submit))
        };

        row!(cancel, horizontal_space(), action).into()
    }

    fn default_view(&self, state: &Hex) -> Element<'_, Charm> {
        let file = {
            let label = text("File:").width(Length::FillPortion(1));

            let file_name = self
                .file
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("No File");

            let file_name = if file_name.len() > 18 {
                format!("{}...", &file_name[0..18])
            } else {
                file_name.to_string()
            };

            let file = container(text(file_name))
                .padding([5, 8])
                .center_y()
                .width(Length::FillPortion(12))
                .style(theme::Container::Custom(Box::new(FileBorderContainer)));

            let btn = button(
                icons::icon(icons::REDO)
                    .vertical_alignment(Vertical::Center)
                    .size(15.0),
            )
            .height(30.0)
            .width(Length::FillPortion(1))
            .on_press(Charm::ReselectFile);

            let temp = row!(label, file)
                .align_items(Alignment::Center)
                .spacing(46)
                .width(Length::FillPortion(10));

            row!(temp, horizontal_space(), btn)
                .width(Length::Fill)
                .align_items(Alignment::Center)
        };

        let options = {
            let label = text("Model:");

            let filetype = FileType::new(&self.file);

            let options: Vec<ViewType> = ViewType::WIZARD
                .iter()
                .map(|view| view.to_owned())
                .filter(|view| view.is_supported_filetype(&filetype))
                .collect();

            let list = pick_list(options, Some(state.model), Charm::ModelSelected);

            row!(label, list).spacing(8).align_items(Alignment::Center)
        };

        let top = column!(file, options).spacing(30);

        top.into()
    }
}

impl<'a, Message> Component<Message> for Wizard<'a, Message>
where
    Message: Debug + Clone,
{
    type State = Hex;
    type Event = Charm;

    fn update(&mut self, state: &mut Self::State, event: Self::Event) -> Option<Message> {
        match event {
            Charm::ConfigSubmit(config) => Some((self.on_submit)(self.file.clone(), config)),
            Charm::SheetSubmit(sheet) => {
                state.sheet_config = sheet;
                state.current_view = Portal::ModelConfig;
                None
            }
            Charm::SheetPrevious(sheet) => {
                state.sheet_config = sheet;
                state.current_view = Portal::FileSelection;
                None
            }
            Charm::LinePrevious(line) => {
                state.line_config = Some(line);
                state.current_view = Portal::SheetConfig;
                None
            }
            Charm::BarChartPrevious(barchart) => {
                state.bar_config = Some(barchart);
                state.current_view = Portal::SheetConfig;
                None
            }
            Charm::Error(err) => Some((self.on_error)(err)),
            Charm::ReselectFile => {
                // Reselecting file means returning to default state
                state.model = ViewType::Editor;

                let data = EditorTabData::new(Some(self.file.clone()), String::default());

                state.config = View::Editor(data);

                self.on_reselect_file.clone()
            }
            Charm::ModelSelected(model) => {
                state.model = model;
                let config = match state.model {
                    ViewType::LineGraph => {
                        LineTabData::new(self.file.clone(), LineConfigState::default())
                            .and_then(|data| Ok(View::LineGraph(data)))
                    }
                    ViewType::BarChart => {
                        BarChartTabData::new(self.file.clone(), BarChartConfigState::default())
                            .and_then(|data| Ok(View::BarChart(data)))
                    }
                    ViewType::Editor => {
                        let data = EditorTabData::new(Some(self.file.clone()), String::default());
                        Ok(View::Editor(data))
                    }
                    ViewType::None => Ok(View::None),
                };
                match config {
                    Err(_err) => {
                        // No need to report error at selecting stage. Configuration
                        // is still not complete
                        // Some((self.on_error)(err))
                        None
                    }
                    Ok(view) => {
                        state.config = view;
                        None
                    }
                }
            }
            Charm::ViewConfig => None,
            Charm::ChangeView(portal) => {
                state.current_view = portal;
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

        match state.current_view {
            Portal::ModelConfig => {
                let content = column!(
                    header,
                    vertical_space().height(50.0),
                    self.model_config(state),
                )
                .spacing(0);

                dialog_container(content)
                    .width(420.0)
                    .height(Length::Shrink)
                    .into()
            }
            Portal::FileSelection => {
                let content = column!(
                    header,
                    vertical_space(),
                    self.default_view(state),
                    vertical_space(),
                    self.actions(state)
                );
                dialog_container(content).height(250.0).into()
            }

            Portal::SheetConfig => {
                let view =
                    SheetConfig::new(Charm::SheetSubmit, Charm::SheetPrevious, Charm::Cancel)
                        .previous_state(state.sheet_config.clone());

                let content = column!(header, vertical_space().height(50.0), view).spacing(0);

                dialog_container(content)
                    .width(420.0)
                    .height(Length::Shrink)
                    .into()
            }
        }
    }
}

impl<'a, Message> From<Wizard<'a, Message>> for Element<'a, Message>
where
    Message: 'a + Clone + Debug,
{
    fn from(value: Wizard<'a, Message>) -> Self {
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

        let background = style.extended_palette().background.weak.color;

        container::Appearance {
            background: Some(background.into()),
            border,
            ..Default::default()
        }
    }
}
