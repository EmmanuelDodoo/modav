use iced::{
    widget::{button, container},
    Background, Border, Color, Shadow, Theme,
};

pub struct BorderedContainer {
    pub width: f32,
}

impl Default for BorderedContainer {
    fn default() -> Self {
        Self { width: 0.75 }
    }
}

impl container::Catalog for BorderedContainer {
    type Class<'a> = Theme;

    fn default<'a>() -> Self::Class<'a> {
        <Theme as std::default::Default>::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> container::Style {
        let color = class.extended_palette().secondary.strong.color;
        let border = Border {
            color,
            width: self.width,
            ..Default::default()
        };
        container::Style {
            border,
            ..Default::default()
        }
    }
}

pub struct ColoredContainer {
    pub color: Color,
    pub radius: f32,
}

impl Default for ColoredContainer {
    fn default() -> Self {
        Self {
            color: Color::TRANSPARENT,
            radius: 0.0,
        }
    }
}

impl container::Catalog for ColoredContainer {
    type Class<'a> = Theme;

    fn default<'a>() -> Self::Class<'a> {
        <Theme as std::default::Default>::default()
    }

    fn style(&self, _class: &Self::Class<'_>) -> container::Style {
        let border = Border {
            radius: self.radius.into(),
            ..Default::default()
        };
        container::Style {
            background: Some(self.color.into()),
            border,
            ..Default::default()
        }
    }
}

pub struct MenuButtonStyle;
impl button::Catalog for MenuButtonStyle {
    type Class<'a> = Theme;

    fn default<'a>() -> Self::Class<'a> {
        <Theme as std::default::Default>::default()
    }

    fn style(&self, class: &Self::Class<'_>, status: button::Status) -> button::Style {
        let palette = class.extended_palette();
        let border = Border {
            radius: 4.0.into(),
            ..Default::default()
        };

        match status {
            button::Status::Active => button::Style {
                border,
                background: Some(Color::TRANSPARENT.into()),
                ..Default::default()
            },
            button::Status::Hovered => button::Style {
                background: Some(palette.primary.weak.color.into()),
                text_color: palette.primary.weak.text,
                border,
                ..Default::default()
            },

            status => button::primary(class, status),
        }
    }
}

pub struct ToolTipContainerStyle;
impl container::Catalog for ToolTipContainerStyle {
    type Class<'a> = Theme;

    fn default<'a>() -> Self::Class<'a> {
        <Theme as std::default::Default>::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> container::Style {
        let background = class.extended_palette().background.weak.color;
        let shadow = Shadow {
            color: Color {
                a: 0.25,
                ..class.extended_palette().primary.strong.color
            },
            offset: [2.0, 3.0].into(),
            blur_radius: 5.0,
        };
        let border = Border {
            width: 0.5,
            color: class.extended_palette().secondary.weak.color,
            radius: 5.0.into(),
        };
        container::Style {
            background: Some(Background::Color(background)),
            border,
            shadow,
            ..Default::default()
        }
    }
}
