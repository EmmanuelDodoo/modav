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

impl container::StyleSheet for BorderedContainer {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        let color = style.extended_palette().secondary.strong.color;
        let border = Border {
            color,
            width: self.width,
            ..Default::default()
        };
        container::Appearance {
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

impl container::StyleSheet for ColoredContainer {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        let border = Border {
            radius: self.radius.into(),
            ..Default::default()
        };
        container::Appearance {
            background: Some(self.color.into()),
            border,
            ..Default::default()
        }
    }
}

pub struct MenuButtonStyle;
impl button::StyleSheet for MenuButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        let border = Border {
            radius: [4.0; 4].into(),
            ..Default::default()
        };
        button::Appearance {
            border,
            background: Some(Color::TRANSPARENT.into()),
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let plt = style.extended_palette();

        button::Appearance {
            background: Some(plt.primary.weak.color.into()),
            text_color: plt.primary.weak.text,
            ..self.active(style)
        }
    }
}

pub struct CustomMenuBarStyle;
impl iced_aw::menu::StyleSheet for CustomMenuBarStyle {
    type Style = Theme;

    fn appearance(&self, _style: &Self::Style) -> iced_aw::menu::Appearance {
        let border = Border {
            radius: [8.0; 4].into(),
            ..Default::default()
        };
        iced_aw::menu::Appearance {
            bar_border: border,
            bar_background: Background::Color(Color::TRANSPARENT),
            path: Background::Color(Color::TRANSPARENT),
            ..Default::default()
        }
    }
}

pub struct ToolTipContainerStyle;

impl container::StyleSheet for ToolTipContainerStyle {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        let background = style.extended_palette().background.weak.color;
        let shadow = Shadow {
            color: style.extended_palette().primary.strong.color,
            offset: [0.0, 1.0].into(),
            blur_radius: 1.0,
        };
        let border = Border {
            width: 0.5,
            color: style.extended_palette().secondary.weak.color,
            radius: 5.0.into(),
        };
        container::Appearance {
            background: Some(Background::Color(background)),
            border,
            shadow,
            ..Default::default()
        }
    }
}
