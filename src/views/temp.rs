#![allow(unused_imports, dead_code)]
use iced::{
    border::Radius,
    theme,
    widget::{column, container, horizontal_rule, row, text, vertical_space, Row, Space},
    Background, Border, Color, Element, Length, Renderer, Size, Theme,
};

use super::{tabs::TabLabel, Viewable};

use crate::utils::coloring::ColorEngine;

#[derive(Clone, Debug, Default)]
pub struct EngineData {
    light: Theme,
    dark: Theme,
    amount: u32,
}

impl EngineData {
    pub fn light(mut self, light: Theme) -> Self {
        self.light = light;
        self
    }

    pub fn dark(mut self, dark: Theme) -> Self {
        self.dark = dark;
        self
    }

    pub fn amount(mut self, amount: u32) -> Self {
        self.amount = amount;
        self
    }
}

#[derive(Debug, Clone)]
pub struct ColorEngineTesting {
    light: Theme,
    dark: Theme,
    amount: u32,
    size: Size,
}

impl ColorEngineTesting {
    fn sample(&self, theme: &Theme) -> Row<'_, ()> {
        let Size { width, height } = self.size;

        let colors = ColorEngine::new(theme).gradual(true).count(self.amount);

        let iter = 0..self.amount;

        let mut row = row!().align_items(iced::Alignment::Center).spacing(5.0);

        for (_, color) in iter.into_iter().zip(colors.into_iter()) {
            let style = Style(color);
            let cont = container(Space::new(Length::Fill, Length::Fill))
                .width(width)
                .height(height)
                .style(theme::Container::Custom(Box::new(style)));
            row = row.push(cont);
        }

        row
    }
}

impl Viewable for ColorEngineTesting {
    type Data = EngineData;
    type Event = ();

    fn new(data: Self::Data) -> Self {
        let EngineData {
            light,
            dark,
            amount,
        } = data;

        Self {
            light,
            dark,
            amount,
            size: Size::new(50.0, 50.0),
        }
    }

    fn is_dirty(&self) -> bool {
        false
    }

    fn label(&self) -> super::tabs::TabLabel {
        TabLabel::new('\u{E800}', "Color Engine Test")
    }

    fn title(&self) -> String {
        String::from("Untitled")
    }

    fn update(&mut self, _message: Self::Event) -> Option<crate::Message> {
        None
    }

    fn view<'a, Message, F>(&'a self, map: F) -> Element<'a, Message, Theme, Renderer>
    where
        F: 'a + Fn(Self::Event) -> Message,
        Message: 'a + Clone + std::fmt::Debug,
    {
        let dark = {
            let text = text("Dark theme test");
            column!(text, self.sample(&self.dark))
                .spacing(2.0)
                .align_items(iced::Alignment::Center)
        };

        let light = {
            let text = text("Light theme test");
            column!(text, self.sample(&self.light))
                .spacing(2.0)
                .align_items(iced::Alignment::Center)
        };

        let content: Element<'a, Self::Event> = column!(
            vertical_space(),
            dark,
            horizontal_rule(10.0),
            light,
            vertical_space()
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_items(iced::Alignment::Center)
        .spacing(20)
        .into();

        content.map(map)
    }

    fn content(&self) -> Option<String> {
        None
    }

    fn modal_msg(&self) -> String {
        String::default()
    }

    fn refresh(&mut self, _data: Self::Data) {}

    fn path(&self) -> Option<std::path::PathBuf> {
        None
    }

    fn can_save(&self) -> bool {
        false
    }
}

struct Style(Color);
impl iced::widget::container::StyleSheet for Style {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(self.0)),
            border: Border {
                radius: Radius::from(50.0),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
