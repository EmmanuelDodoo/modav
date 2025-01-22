#![allow(deprecated)]
use std::{fmt::Debug, path::PathBuf};

use iced::{
    alignment::{Alignment, Horizontal, Vertical},
    color,
    widget::{
        self, button, column, component, container, horizontal_space, pick_list, row, text,
        vertical_space, Component, Space,
    },
    Element, Length, Theme,
};

use crate::views::{
    BarChartTabData, EditorTabData, FileType, LineTabData, StackedBarChartTabData, View,
};

use crate::styles::FileBorderContainer;
use crate::utils::{icons, AppError};
use crate::ViewType;

use super::style::dialog_container;

mod line;
pub use line::LineConfigState;
use line::LineGraphConfig;

mod barchart;
use barchart::BarChartConfig;
pub use barchart::BarChartConfigState;

mod sheet;
use sheet::{SheetConfig, SheetConfigState};

mod stacked_barchart;
use stacked_barchart::StackedBarChartConfig;
pub use stacked_barchart::StackedBarChartConfigState;

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
    stacked_bar_config: Option<StackedBarChartConfigState>,
    error: Option<String>,
}

impl Default for Hex {
    fn default() -> Self {
        Self {
            model: ViewType::Editor,
            current_view: Portal::FileSelection,
            config: View::Editor(EditorTabData::default()),
            sheet_config: SheetConfigState::default(),
            stacked_bar_config: None,
            line_config: None,
            bar_config: None,
            error: None,
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
    StackedBarChartPrevious(StackedBarChartConfigState),
    Error(AppError),
    Submit,
    ClearError,
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
                    Charm::ClearError,
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
                    Charm::ClearError,
                );

                if let Some(barchart_config) = state.bar_config.clone() {
                    content = content.previous_state(barchart_config);
                }

                content.into()
            }
            ViewType::StackedBarChart => {
                let mut content = StackedBarChartConfig::new(
                    &self.file,
                    state.sheet_config.clone(),
                    Charm::ConfigSubmit,
                    Charm::Error,
                    Charm::StackedBarChartPrevious,
                    Charm::Cancel,
                    Charm::ClearError,
                );
                if let Some(stacked_config) = state.stacked_bar_config.clone() {
                    content = content.previous_state(stacked_config);
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
            let label = text("File:");

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
                .center_y(Length::Shrink)
                .width(Length::FillPortion(12))
                .style(|theme| {
                    <FileBorderContainer as container::Catalog>::style(&FileBorderContainer, theme)
                });

            let btn = button(
                icons::icon(icons::REDO)
                    .align_y(Vertical::Center)
                    .size(15.0),
            )
            .height(30.0)
            .width(Length::FillPortion(1))
            .on_press(Charm::ReselectFile);

            let temp = row!(label, file).align_y(Alignment::Center).spacing(27);

            row!(temp, horizontal_space(), btn)
                .width(Length::Fill)
                .align_y(Alignment::Center)
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

            row!(label, list).spacing(8).align_y(Alignment::Center)
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
            Charm::StackedBarChartPrevious(barchart) => {
                state.stacked_bar_config = Some(barchart);
                state.current_view = Portal::SheetConfig;
                None
            }
            Charm::Error(err) => {
                state.error = Some(err.to_string());
                Some((self.on_error)(err))
            }
            Charm::ClearError => {
                state.error = None;
                None
            }
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
                    ViewType::StackedBarChart => StackedBarChartTabData::new(
                        self.file.clone(),
                        StackedBarChartConfigState::default(),
                    )
                    .and_then(|data| Ok(View::StackedBarChart(data))),
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
            .align_x(Horizontal::Center);

        let error_section: Element<'_, Self::Event> = match state.error.clone() {
            Some(msg) => {
                struct Background;

                impl widget::container::Catalog for Background {
                    type Class<'a> = Theme;

                    fn default<'a>() -> Self::Class<'a> {
                        <Theme as std::default::Default>::default()
                    }

                    fn style(&self, class: &Self::Class<'_>) -> container::Style {
                        if class.extended_palette().is_dark {
                            let text_color = color!(248, 133, 133);
                            let background = color!(153, 27, 27);

                            container::Style {
                                text_color: Some(text_color),
                                background: Some(iced::Background::Color(background)),
                                ..Default::default()
                            }
                        } else {
                            let text_color = color!(75, 20, 20);
                            let background = color!(248, 113, 113);

                            container::Style {
                                text_color: Some(text_color),
                                background: Some(iced::Background::Color(background)),
                                ..Default::default()
                            }
                        }
                    }
                }

                container(text(msg).size(15.0))
                    .width(Length::Fill)
                    .padding([4, 6])
                    .style(|theme| <Background as container::Catalog>::style(&Background, theme))
                    .into()
            }
            None => Space::new(0, 0).into(),
        };

        match state.current_view {
            Portal::ModelConfig => {
                let content = column!(
                    header,
                    if state.error.is_some() {
                        vertical_space().height(25.0)
                    } else {
                        Space::new(0, 0)
                    },
                    error_section,
                    vertical_space().height(25.0),
                    self.model_config(state),
                )
                .spacing(0);

                dialog_container(content)
                    .width(450.0)
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
                let view = SheetConfig::new(
                    Charm::SheetSubmit,
                    Charm::SheetPrevious,
                    Charm::Cancel,
                    Charm::ClearError,
                )
                .previous_state(state.sheet_config.clone());

                let content = column!(
                    header,
                    if state.error.is_some() {
                        vertical_space().height(25.0)
                    } else {
                        Space::new(0, 0)
                    },
                    error_section,
                    vertical_space().height(25.0),
                    view
                )
                .spacing(0);

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
