use iced::{
    theme,
    widget::{self, container, Container},
    Border, Element, Renderer, Theme,
};

pub fn dialog_container<'a, Message, Theme>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme>
where
    Theme: widget::container::StyleSheet,
    <Theme as widget::container::StyleSheet>::Style: From<theme::Container>,
{
    container(content)
        .padding([20.0, 25.0])
        .width(375.0)
        .height(400.0)
        .style(theme::Container::Custom(Box::new(
            DialogContainer::default(),
        )))
        .into()
}

#[derive(Debug, Clone, Copy)]
pub struct DialogContainer {
    pub radius: f32,
}

impl Default for DialogContainer {
    fn default() -> Self {
        Self { radius: 10.0 }
    }
}

impl DialogContainer {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

impl widget::container::StyleSheet for DialogContainer {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> widget::container::Appearance {
        let border = Border {
            radius: self.radius.into(),
            ..Default::default()
        };

        let background = style.extended_palette().background.base.color;

        widget::container::Appearance {
            background: Some(background.into()),
            border,
            ..Default::default()
        }
    }
}
