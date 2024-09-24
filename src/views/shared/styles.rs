use iced::{
    widget::{button, container, overlay, pick_list},
    Background, Border, Theme,
};

#[derive(Clone, Copy, Debug)]
pub struct ToolbarContainerStyle;

impl container::StyleSheet for ToolbarContainerStyle {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        let pallete = style.extended_palette();

        let background = Background::Color(pallete.background.weak.color);

        let border = Border {
            color: pallete.primary.base.color,
            width: 0.5,
            radius: 5.0.into(),
        };

        container::Appearance {
            background: Some(background),
            border,
            ..Default::default()
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ToolbarStyle;

impl pick_list::StyleSheet for ToolbarStyle {
    type Style = Theme;

    fn active(&self, style: &<Self as pick_list::StyleSheet>::Style) -> pick_list::Appearance {
        let pallete = style.extended_palette();
        let text_color = pallete.background.base.text;
        let background = Background::Color(pallete.background.weak.color);
        let border = Border {
            color: pallete.background.weak.color,
            width: 0.25,
            radius: 3.0.into(),
        };

        pick_list::Appearance {
            text_color,
            placeholder_color: text_color,
            handle_color: text_color,
            background,
            border,
        }
    }

    fn hovered(&self, style: &<Self as pick_list::StyleSheet>::Style) -> pick_list::Appearance {
        let pallete = style.extended_palette();
        let text_color = pallete.primary.strong.color;
        let background = Background::Color(pallete.background.weak.color);
        let border = Border {
            color: pallete.primary.strong.color,
            width: 0.5,
            radius: 3.0.into(),
        };

        pick_list::Appearance {
            text_color,
            placeholder_color: text_color,
            handle_color: text_color,
            background,
            border,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ToolbarMenuStyle;

impl overlay::menu::StyleSheet for ToolbarMenuStyle {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> overlay::menu::Appearance {
        let pallete = style.extended_palette();

        let text_color = pallete.background.base.text;
        let background = Background::Color(pallete.background.weak.color);
        let border = Border {
            width: 1.0,
            radius: 3.5.into(),
            color: pallete.background.strong.color,
        };
        let selected_background = Background::Color(pallete.primary.base.color);

        overlay::menu::Appearance {
            text_color,
            selected_text_color: pallete.primary.base.text,
            background,
            border,
            selected_background,
        }
    }
}

pub struct EditorButtonStyle;
impl button::StyleSheet for EditorButtonStyle {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        let palette = style.extended_palette();

        button::Appearance {
            text_color: palette.background.base.text,
            background: Some(Background::Color(palette.background.weak.color)),
            ..Default::default()
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let pallete = style.extended_palette();
        let text_color = pallete.primary.strong.color;
        let background = Background::Color(pallete.background.weak.color);
        let border = Border {
            color: pallete.primary.strong.color,
            width: 0.5,
            radius: 3.0.into(),
        };

        button::Appearance {
            text_color,
            border,
            background: Some(background),
            ..Default::default()
        }
    }
}

pub struct ContentAreaContainer;
impl container::StyleSheet for ContentAreaContainer {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        let border_color = style.extended_palette().background.weak.color;
        let background_color = style.extended_palette().background.strong.color;

        let border = Border {
            color: border_color,
            width: 1.5,
            ..Default::default()
        };

        let background = Background::Color(background_color);

        container::Appearance {
            border,
            background: Some(background),
            ..Default::default()
        }
    }
}
