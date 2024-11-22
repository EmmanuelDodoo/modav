#![allow(deprecated)]
use crate::utils::icons;

use iced::{
    alignment,
    widget::{
        self, button, column, component, horizontal_space, row, text, Column, Component, Space,
    },
    Alignment, Background, Border, Element, Font, Length, Padding, Pixels, Renderer, Shadow, Theme,
};

#[derive(Clone, Copy, Debug)]
struct DashMenuStyle {
    shadow: Shadow,
    border: Border,
}

impl DashMenuStyle {
    fn new() -> Self {
        let border = Border::default().rounded(2.5);
        let shadow = Shadow::default();

        Self { shadow, border }
    }
}

impl button::Catalog for DashMenuStyle {
    type Class<'a> = Theme;

    fn default<'a>() -> Self::Class<'a> {
        <Theme as std::default::Default>::default()
    }

    fn style(&self, class: &Self::Class<'_>, status: button::Status) -> button::Style {
        let pallete = class.extended_palette();

        match status {
            button::Status::Active => {
                let text = pallete.primary.base.text;
                let background = Background::Color(pallete.primary.weak.color);

                let Self { shadow, border, .. } = *self;

                widget::button::Style {
                    shadow,
                    text_color: text,
                    background: Some(background),
                    border,
                }
            }
            button::Status::Hovered => {
                let text = pallete.primary.base.text;
                let background = Background::Color(pallete.primary.base.color);

                let Self { shadow, border, .. } = *self;

                widget::button::Style {
                    shadow,
                    text_color: text,
                    background: Some(background),
                    border,
                }
            }
            status => button::primary(class, status),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct DashMenuOptionStyle {
    shadow: Shadow,
    border: Border,
}

impl DashMenuOptionStyle {
    fn new() -> Self {
        let border = Border::default();
        let shadow = Shadow::default();

        Self { shadow, border }
    }
}

impl button::Catalog for DashMenuOptionStyle {
    type Class<'a> = Theme;

    fn default<'a>() -> Self::Class<'a> {
        <Theme as std::default::Default>::default()
    }

    fn style(&self, class: &Self::Class<'_>, status: button::Status) -> button::Style {
        let pallete = class.extended_palette();

        match status {
            button::Status::Active => {
                let text = pallete.primary.base.text;
                let background = Background::Color(pallete.primary.weak.color);

                let Self { shadow, border, .. } = *self;

                button::Style {
                    background: Some(background),
                    text_color: text,
                    shadow,
                    border,
                }
            }

            button::Status::Hovered => {
                let text = pallete.primary.base.text;
                let background = Background::Color(pallete.primary.base.color);

                let Self { shadow, border, .. } = *self;

                button::Style {
                    background: Some(background),
                    text_color: text,
                    shadow,
                    border,
                }
            }
            status => button::primary(class, status),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DashMenuOption<Message>
where
    Message: Clone,
{
    label: String,
    on_select: Option<Message>,
}

impl<Message> DashMenuOption<Message>
where
    Message: Clone,
{
    pub fn new(label: impl Into<String>, on_select: Option<Message>) -> Self {
        Self {
            label: label.into(),
            on_select,
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub enum DashMenuMessage<Message> {
    Pressed,
    OptionPressed(Message),
}

#[derive(Clone, Debug, Default, Copy)]
pub struct DashMenuState {
    is_open: bool,
}

pub struct DashMenu<Message>
where
    Message: Clone,
{
    icon: char,
    on_select: Option<Message>,
    label: String,
    options: Vec<DashMenuOption<Message>>,
    icon_font: Option<Font>,
    icon_size: Option<Pixels>,
    text_size: Option<Pixels>,
    submenu_text_size: Option<Pixels>,
    padding: Padding,
    submenu_padding: Padding,
    spacing: f32,
    width: Length,
    height: Length,
}

impl<Message> DashMenu<Message>
where
    Message: Clone,
{
    pub fn new(icon: char, label: impl Into<String>) -> Self {
        Self {
            icon,
            icon_font: None,
            icon_size: None,
            text_size: None,
            submenu_text_size: None,
            label: label.into(),
            on_select: None,
            options: Vec::default(),
            width: Length::Shrink,
            height: Length::Shrink,
            padding: Padding::ZERO,
            submenu_padding: Padding::ZERO,
            spacing: 0.0,
        }
    }

    pub fn submenus(mut self, submenus: Vec<DashMenuOption<Message>>) -> Self {
        self.options.extend(submenus);
        self
    }

    pub fn icon_font(mut self, icon_font: Font) -> Self {
        self.icon_font = Some(icon_font);
        self
    }

    pub fn icon_size(mut self, icon_size: impl Into<Pixels>) -> Self {
        self.icon_size = Some(icon_size.into());
        self
    }

    pub fn text_size(mut self, text_size: impl Into<Pixels>) -> Self {
        self.text_size = Some(text_size.into());
        self
    }

    pub fn submenu_text_size(mut self, text_size: impl Into<Pixels>) -> Self {
        self.submenu_text_size = Some(text_size.into());
        self
    }

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn submenu_padding(mut self, padding: impl Into<Padding>) -> Self {
        self.submenu_padding = padding.into();
        self
    }

    pub fn on_select(mut self, on_select: Message) -> Self {
        self.on_select = Some(on_select);
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
}

impl<Message> Component<Message, Theme, Renderer> for DashMenu<Message>
where
    Message: Clone,
{
    type State = DashMenuState;
    type Event = DashMenuMessage<Message>;

    fn update(&mut self, state: &mut Self::State, event: Self::Event) -> Option<Message> {
        match event {
            DashMenuMessage::Pressed if self.options.is_empty() => self.on_select.clone(),
            DashMenuMessage::Pressed => {
                state.is_open = !state.is_open;
                self.on_select.clone()
            }
            DashMenuMessage::OptionPressed(msg) => {
                state.is_open = false;
                Some(msg)
            }
        }
    }

    fn view(&self, state: &Self::State) -> Element<'_, Self::Event, Theme, Renderer> {
        let label = {
            let mut text = text(self.label.clone());

            if let Some(size) = self.text_size {
                text = text.size(size)
            };

            text
        };

        let icon = {
            let mut icon = text(self.icon.to_string()).align_x(alignment::Horizontal::Center);

            if let Some(font) = self.icon_font {
                icon = icon.font(font)
            };

            if let Some(size) = self.icon_size {
                icon = icon.size(size)
            };

            icon
        };

        let indicator: Element<'_, DashMenuMessage<Message>> = {
            let icon = if self.options.is_empty() {
                Space::new(0, 0).into()
            } else if state.is_open {
                icons::icon(icons::ANGLE_UP)
                    .align_x(alignment::Horizontal::Center)
                    .into()
            } else {
                icons::icon(icons::ANGLE_DOWN)
                    .align_x(alignment::Horizontal::Center)
                    .into()
            };

            icon
        };

        let pair = row!(icon, label).spacing(self.spacing);

        let main = button(
            column!(row!(pair, horizontal_space(), indicator).align_y(Alignment::Center))
                .width(self.width)
                .height(self.height)
                .align_x(Alignment::Start)
                .padding(self.padding),
        )
        .width(self.width)
        .height(self.height)
        .on_press(DashMenuMessage::Pressed)
        .style(|theme, status| {
            <DashMenuStyle as button::Catalog>::style(&DashMenuStyle::new(), theme, status)
        });

        if self.options.is_empty() || !state.is_open {
            return main.into();
        }

        let options = self.options.iter().map(|option| {
            let mut text = text(option.label.clone());
            if let Some(size) = self.submenu_text_size {
                text = text.size(size)
            };

            let btn = button(text)
                .width(Length::Fill)
                .padding(self.submenu_padding)
                .style(|theme, status| {
                    <DashMenuOptionStyle as button::Catalog>::style(
                        &DashMenuOptionStyle::new(),
                        theme,
                        status,
                    )
                });

            if let Some(on_select) = &option.on_select {
                btn.on_press(DashMenuMessage::OptionPressed(on_select.clone()))
                    .into()
            } else {
                btn.into()
            }
        });

        let options = Column::with_children(options).align_x(Alignment::Start);

        column!(main, options).width(self.width).into()
    }
}

impl<'a, Message> From<DashMenu<Message>> for Element<'a, Message, Theme>
where
    Message: Clone + 'a,
{
    fn from(value: DashMenu<Message>) -> Self {
        component(value)
    }
}
