use iced::{
    color,
    widget::{button, container},
    Background, Border, Shadow, Theme, Vector,
};

#[derive(Clone, Copy, Debug)]
pub struct ToolbarContainerStyle;

impl container::Catalog for ToolbarContainerStyle {
    type Class<'a> = Theme;

    fn default<'a>() -> Self::Class<'a> {
        <Theme as std::default::Default>::default()
    }

    fn style(&self, style: &Self::Class<'_>) -> container::Style {
        let pallete = style.extended_palette();

        let background = Background::Color(pallete.background.weak.color);

        let border = Border {
            color: pallete.primary.base.color,
            width: 0.5,
            radius: 5.0.into(),
        };

        container::Style {
            background: Some(background),
            border,
            ..Default::default()
        }
    }
}

pub struct EditorButtonStyle;
impl button::Catalog for EditorButtonStyle {
    type Class<'a> = Theme;

    fn default<'a>() -> Self::Class<'a> {
        <Theme as std::default::Default>::default()
    }

    fn style(&self, class: &Self::Class<'_>, status: button::Status) -> button::Style {
        let palette = class.extended_palette();

        match status {
            button::Status::Active => button::Style {
                text_color: palette.background.base.text,
                background: Some(Background::Color(palette.background.weak.color)),
                ..Default::default()
            },
            button::Status::Hovered => {
                let text_color = palette.primary.strong.color;
                let background = Background::Color(palette.background.weak.color);
                let border = Border {
                    color: palette.primary.strong.color,
                    width: 0.5,
                    radius: 3.0.into(),
                };

                button::Style {
                    text_color,
                    border,
                    background: Some(background),
                    ..Default::default()
                }
            }
            status => button::primary(class, status),
        }
    }
}

pub struct ContentAreaContainer;
impl container::Catalog for ContentAreaContainer {
    type Class<'a> = Theme;

    fn default<'a>() -> Self::Class<'a> {
        <Theme as std::default::Default>::default()
    }

    fn style(&self, class: &Self::Class<'_>) -> container::Style {
        let border_color = class.extended_palette().background.weak.color;
        let background_color = class.extended_palette().background.strong.color;

        let border = Border {
            color: border_color,
            width: 1.5,
            ..Default::default()
        };

        let background = Background::Color(background_color);

        container::Style {
            border,
            background: Some(background),
            ..Default::default()
        }
    }
}

pub struct ToolsButton;

impl button::Catalog for ToolsButton {
    type Class<'a> = Theme;

    fn default<'a>() -> Self::Class<'a> {
        <Theme as std::default::Default>::default()
    }

    fn style(&self, class: &Self::Class<'_>, status: button::Status) -> button::Style {
        let default = button::primary(class, button::Status::Active);

        let border = Border {
            radius: 5.0.into(),
            width: default.border.width * 0.5,
            ..default.border
        };

        let shadow = Shadow {
            color: color!(0, 0, 0, 0.5),
            offset: Vector::new(2.0, 2.0),
            blur_radius: 4.0,
        };

        match status {
            button::Status::Active => button::Style {
                border,
                shadow,
                ..default
            },
            button::Status::Hovered => {
                let default = button::primary(class, button::Status::Hovered);
                let border = Border {
                    radius: 5.0.into(),
                    width: default.border.width,
                    ..default.border
                };

                let shadow = Shadow {
                    color: color!(0, 0, 0, 0.5),
                    offset: Vector::new(1.0, 1.0),
                    blur_radius: 10.0,
                };

                button::Style {
                    border,
                    shadow,
                    ..default
                }
            }
            status => button::Style {
                border,
                shadow,
                ..button::primary(class, status)
            },
        }
    }
}
