use iced::{
    widget::{self, container, Container},
    Border, Element, Renderer, Theme,
};

pub fn dialog_container<'a, Message>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Container<'a, Message, Theme> {
    container(content)
        .padding([20.0, 25.0])
        .width(375.0)
        .height(400.0)
        .style(|theme| {
            <DialogContainer as widget::container::Catalog>::style(
                &DialogContainer::default(),
                theme,
            )
        })
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

impl widget::container::Catalog for DialogContainer {
    type Class<'a> = Theme;

    fn default<'a>() -> Self::Class<'a> {
        <Theme as std::default::Default>::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> container::Style {
        let border = Border {
            radius: self.radius.into(),
            ..Default::default()
        };

        let background = class.extended_palette().background.base.color;

        widget::container::Style {
            background: Some(background.into()),
            border,
            ..Default::default()
        }
    }
}
