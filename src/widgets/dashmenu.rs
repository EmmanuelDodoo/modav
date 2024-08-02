use iced::{
    advanced, alignment, theme,
    widget::{
        self, button, column, component, horizontal_space, row, text, Column, Component, Space,
    },
    Alignment, Background, Border, Element, Font, Length, Padding, Pixels, Renderer, Shadow, Theme,
    Vector,
};

#[derive(Clone, Copy, Debug)]
struct DashMenuStyle {
    shadow: Shadow,
    border: Border,
    shadow_offset: Vector,
}

impl DashMenuStyle {
    fn new() -> Self {
        let shadow_offset = Vector::default();
        let border = Border::with_radius(2.5);
        let shadow = Shadow::default();

        Self {
            shadow,
            border,
            shadow_offset,
        }
    }
}

impl widget::button::StyleSheet for DashMenuStyle {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> widget::button::Appearance {
        let pallete = style.extended_palette();

        let text = pallete.primary.base.text;
        let background = Background::Color(pallete.primary.weak.color);

        let Self {
            shadow,
            border,
            shadow_offset,
        } = *self;

        widget::button::Appearance {
            shadow,
            shadow_offset,
            text_color: text,
            background: Some(background),
            border,
        }
    }

    fn hovered(&self, style: &Self::Style) -> widget::button::Appearance {
        let pallete = style.extended_palette();

        let text = pallete.primary.base.text;
        let background = Background::Color(pallete.primary.base.color);

        let Self {
            shadow,
            border,
            shadow_offset,
        } = *self;

        widget::button::Appearance {
            shadow,
            shadow_offset,
            text_color: text,
            background: Some(background),
            border,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct DashMenuOptionStyle {
    shadow: Shadow,
    border: Border,
    shadow_offset: Vector,
}

impl DashMenuOptionStyle {
    fn new() -> Self {
        let shadow_offset = Vector::default();
        let border = Border::default();
        let shadow = Shadow::default();

        Self {
            shadow,
            shadow_offset,
            border,
        }
    }
}

impl widget::button::StyleSheet for DashMenuOptionStyle {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> widget::button::Appearance {
        let pallete = style.extended_palette();

        let text = pallete.primary.base.text;
        let background = Background::Color(pallete.primary.weak.color);

        let Self {
            shadow,
            border,
            shadow_offset,
        } = *self;

        widget::button::Appearance {
            background: Some(background),
            text_color: text,
            shadow,
            shadow_offset,
            border,
        }
    }

    fn hovered(&self, style: &Self::Style) -> widget::button::Appearance {
        let pallete = style.extended_palette();

        let text = pallete.primary.base.text;
        let background = Background::Color(pallete.primary.base.color);

        let Self {
            shadow,
            border,
            shadow_offset,
        } = *self;

        widget::button::Appearance {
            background: Some(background),
            text_color: text,
            shadow,
            shadow_offset,
            border,
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

pub struct DashMenu<Message, Renderer>
where
    Message: Clone,
    Renderer: advanced::text::Renderer,
{
    icon: char,
    on_select: Option<Message>,
    label: String,
    options: Vec<DashMenuOption<Message>>,
    icon_font: Option<Renderer::Font>,
    icon_size: Option<Pixels>,
    text_font: Option<Renderer::Font>,
    submenu_text_font: Option<Renderer::Font>,
    text_size: Option<Pixels>,
    submenu_text_size: Option<Pixels>,
    padding: Padding,
    submenu_padding: Padding,
    spacing: f32,
    width: Length,
    height: Length,
}

impl<Message, Renderer> DashMenu<Message, Renderer>
where
    Message: Clone,
    Renderer: advanced::text::Renderer,
{
    pub fn new(icon: char, label: impl Into<String>) -> Self {
        Self {
            icon,
            icon_font: None,
            icon_size: None,
            text_size: None,
            text_font: None,
            submenu_text_font: None,
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

    pub fn icon_font(mut self, icon_font: Renderer::Font) -> Self {
        self.icon_font = Some(icon_font);
        self
    }

    pub fn icon_size(mut self, icon_size: impl Into<Pixels>) -> Self {
        self.icon_size = Some(icon_size.into());
        self
    }

    pub fn text_font(mut self, text_font: Renderer::Font) -> Self {
        self.text_font = Some(text_font);
        self
    }

    pub fn submenu_text_font(mut self, text_font: Renderer::Font) -> Self {
        self.submenu_text_font = Some(text_font);
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

impl<Message> Component<Message, Theme, Renderer> for DashMenu<Message, Renderer>
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

            if let Some(font) = self.text_font {
                text = text.font(font)
            };

            if let Some(size) = self.text_size {
                text = text.size(size)
            };

            text
        };

        let icon = {
            let mut icon =
                text(self.icon.to_string()).horizontal_alignment(alignment::Horizontal::Center);

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
                text('\u{F106}')
                    .font(Font::with_name("dash-icons"))
                    .horizontal_alignment(alignment::Horizontal::Center)
                    .into()
            } else {
                text('\u{F107}')
                    .font(Font::with_name("dash-icons"))
                    .horizontal_alignment(alignment::Horizontal::Center)
                    .into()
            };

            icon
        };

        let pair = row!(icon, label).spacing(self.spacing);

        let main = button(
            column!(row!(pair, horizontal_space(), indicator).align_items(Alignment::Center))
                .width(self.width)
                .height(self.height)
                .align_items(Alignment::Start)
                .padding(self.padding),
        )
        .width(self.width)
        .height(self.height)
        .on_press(DashMenuMessage::Pressed)
        .style(theme::Button::Custom(Box::new(DashMenuStyle::new())));

        if self.options.is_empty() || !state.is_open {
            return main.into();
        }

        let options = self.options.iter().map(|option| {
            let mut text = text(option.label.clone());
            if let Some(font) = self.submenu_text_font {
                text = text.font(font)
            };
            if let Some(size) = self.submenu_text_size {
                text = text.size(size)
            };

            let btn = button(text)
                .width(Length::Fill)
                .padding(self.submenu_padding)
                .style(theme::Button::Custom(Box::new(DashMenuOptionStyle::new())));

            if let Some(on_select) = &option.on_select {
                btn.on_press(DashMenuMessage::OptionPressed(on_select.clone()))
                    .into()
            } else {
                btn.into()
            }
        });

        let options = Column::with_children(options).align_items(Alignment::Start);

        column!(main, options).width(self.width).into()
    }
}

impl<'a, Message> From<DashMenu<Message, Renderer>> for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
{
    fn from(value: DashMenu<Message, Renderer>) -> Self {
        component(value)
    }
}
