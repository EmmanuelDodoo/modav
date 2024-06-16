use std::borrow::Borrow;

use iced::{
    advanced::{
        self, layout, mouse, renderer,
        text::Paragraph,
        widget::{tree, Tree},
        Layout, Text, Widget,
    },
    alignment, event, keyboard, touch,
    widget::{
        overlay,
        pick_list::StyleSheet,
        scrollable,
        text::{self, LineHeight},
        Scrollable,
    },
    Element, Event, Length, Padding, Pixels, Point, Rectangle, Size, Vector,
};

pub trait ToolbarOption {
    fn icon(&self) -> char;
}

pub struct ToolbarMenu<'a, T, L, V, Message, Theme, Renderer>
where
    T: ToString + PartialEq + Clone + ToolbarOption,
    L: Borrow<[T]>,
    V: Borrow<T>,
    Theme: StyleSheet + overlay::menu::StyleSheet,
    Renderer: advanced::text::Renderer,
{
    on_select: Box<dyn Fn(T) -> Message + 'a>,
    on_open: Option<Message>,
    on_close: Option<Message>,
    selected: V,
    options: L,
    width: Length,
    height: Length,
    menu_width: Option<f32>,
    orientation: ToolBarOrientation,
    padding: Padding,
    menu_padding: Padding,
    text_size: Option<Pixels>,
    text_font: Option<Renderer::Font>,
    icon_size: Option<Pixels>,
    icon_font: Renderer::Font,
    spacing: f32,
    style: <Theme as StyleSheet>::Style,
    menu_style: <Theme as overlay::menu::StyleSheet>::Style,
}

impl<'a, T, L, V, Message, Theme, Renderer> ToolbarMenu<'a, T, L, V, Message, Theme, Renderer>
where
    T: ToString + PartialEq + ToolbarOption + Clone,
    L: Borrow<[T]>,
    V: Borrow<T>,
    Theme: StyleSheet + overlay::menu::StyleSheet,
    Renderer: advanced::text::Renderer,
{
    pub fn new(
        options: L,
        selected: V,
        on_select: impl Fn(T) -> Message + 'a,
        icon_font: Renderer::Font,
    ) -> Self {
        let default_padding = [2, 4];

        Self {
            on_select: Box::new(on_select),
            selected,
            on_close: None,
            on_open: None,
            width: Length::Shrink,
            height: Length::Shrink,
            menu_width: None,
            padding: default_padding.into(),
            menu_padding: default_padding.into(),
            text_size: None,
            text_font: None,
            icon_size: None,
            icon_font,
            orientation: ToolBarOrientation::default(),
            spacing: 0.0,
            style: <Theme as StyleSheet>::Style::default(),
            menu_style: <Theme as overlay::menu::StyleSheet>::Style::default(),
            options,
        }
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    pub fn menu_width(mut self, width: f32) -> Self {
        self.menu_width = Some(width);
        self
    }

    pub fn on_close(mut self, on_close: Message) -> Self {
        self.on_close = Some(on_close);
        self
    }

    pub fn on_open(mut self, on_open: Message) -> Self {
        self.on_open = Some(on_open);
        self
    }

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn menu_padding(mut self, padding: impl Into<Padding>) -> Self {
        self.menu_padding = padding.into();
        self
    }

    pub fn orientation(mut self, orientation: ToolBarOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn text_size(mut self, text_size: impl Into<Pixels>) -> Self {
        self.text_size = Some(text_size.into());
        self
    }

    pub fn text_font(mut self, text_font: Renderer::Font) -> Self {
        self.text_font = Some(text_font);
        self
    }

    pub fn icon_size(mut self, icon_size: impl Into<Pixels>) -> Self {
        self.icon_size = Some(icon_size.into());
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn style(mut self, style: impl Into<<Theme as StyleSheet>::Style>) -> Self {
        self.style = style.into();
        self
    }

    pub fn menu_style(
        mut self,
        style: impl Into<<Theme as overlay::menu::StyleSheet>::Style>,
    ) -> Self {
        self.menu_style = style.into();
        self
    }
}

impl<'a, T, L, V, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for ToolbarMenu<'a, T, L, V, Message, Theme, Renderer>
where
    T: ToString + PartialEq + Clone + ToolbarOption + 'a,
    L: Borrow<[T]> + 'a,
    V: Borrow<T> + 'a,
    Theme: overlay::menu::StyleSheet + StyleSheet + scrollable::StyleSheet + 'a,
    Renderer: iced::advanced::Renderer + advanced::text::Renderer + 'a,
    Message: Clone + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<MenuState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(MenuState::new())
    }

    fn size(&self) -> iced::Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<MenuState>();
        let icon_size = self.icon_size.unwrap_or(renderer.default_size());

        if self.menu_width.is_none() {
            let options = self.options.borrow();
            let mut max_width = 0.0;

            let text_size = self.text_size.unwrap_or(renderer.default_size());
            let text_font = self.text_font.unwrap_or(renderer.default_font());
            let line_height = LineHeight::default();
            let shaping = text::Shaping::default();

            let options_text = Text {
                content: "",
                bounds: Size::new(f32::INFINITY, line_height.to_absolute(text_size).into()),
                size: text_size,
                line_height,
                font: text_font,
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Center,
                shaping,
            };

            let options_icon = Text {
                size: icon_size,
                font: self.icon_font,
                bounds: Size::new(f32::INFINITY, line_height.to_absolute(icon_size).into()),
                ..options_text
            };

            for option in options.iter() {
                let mut text_par = <Renderer>::Paragraph::default();

                let text = Text {
                    content: &option.to_string(),
                    ..options_text
                };

                text_par.update(text);

                let mut icon_par = <Renderer>::Paragraph::default();

                let icon = Text {
                    content: &option.icon().to_string(),
                    ..options_icon
                };

                icon_par.update(icon);

                max_width = f32::max(
                    max_width,
                    text_par.min_width() + icon_par.min_width() + self.menu_padding.horizontal(),
                );
            }

            state.width = max_width + self.spacing;
        }

        let size = {
            let intrinsic = Size::new(
                icon_size.0 + self.padding.left,
                f32::from(LineHeight::default().to_absolute(icon_size)),
            );

            limits
                .width(self.width)
                .height(self.height)
                .shrink(self.padding)
                .resolve(self.width, self.height, intrinsic)
                .expand(self.padding)
        };

        layout::Node::new(size)
    }

    fn on_event(
        &mut self,
        state: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn advanced::Clipboard,
        shell: &mut advanced::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                let state = state.state.downcast_mut::<MenuState>();
                if state.is_open {
                    state.is_open = false;

                    if let Some(on_close) = &self.on_close {
                        shell.publish(on_close.clone());
                    }

                    return event::Status::Captured;
                } else if cursor.is_over(layout.bounds()) {
                    let selected: &T = self.selected.borrow();

                    state.is_open = true;

                    state.hovered_option = self
                        .options
                        .borrow()
                        .iter()
                        .position(|option| Borrow::<T>::borrow(option) == selected);

                    if let Some(on_open) = &self.on_open {
                        shell.publish(on_open.clone())
                    }

                    event::Status::Captured
                } else {
                    return event::Status::Ignored;
                }
            }

            Event::Mouse(mouse::Event::WheelScrolled {
                delta: mouse::ScrollDelta::Lines { y, .. },
            }) => {
                let state = state.state.downcast_mut::<MenuState>();

                if state.keyboard_modifiers.command()
                    && cursor.is_over(layout.bounds())
                    && !state.is_open
                {
                    fn find_next<'a, T: ToString + Clone + PartialEq>(
                        selected: &'a T,
                        mut options: impl Iterator<Item = &'a T>,
                    ) -> Option<&'a T> {
                        let _ = options.find(|&option| Borrow::<T>::borrow(option) == selected);
                        if let Some(option) = options.next() {
                            return Some(Borrow::<T>::borrow(option));
                        }
                        None
                    }

                    let options = &self.options;
                    let selected: &T = self.selected.borrow();

                    let next_option = if y < 0.0 {
                        find_next(selected, options.borrow().iter())
                    } else if y > 0.0 {
                        find_next(selected, options.borrow().iter().rev())
                    } else {
                        None
                    };

                    if let Some(next_option) = next_option {
                        shell.publish((self.on_select)(next_option.clone()))
                    }

                    event::Status::Captured
                } else {
                    event::Status::Ignored
                }
            }

            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                let state = state.state.downcast_mut::<MenuState>();

                state.keyboard_modifiers = modifiers;

                event::Status::Ignored
            }
            _ => event::Status::Ignored,
        }
    }

    fn mouse_interaction(
        &self,
        _state: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();
        let is_mouse_over = cursor.is_over(bounds);

        if is_mouse_over {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &iced::advanced::renderer::Style,
        layout: layout::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        let icon_size = self.icon_size.unwrap_or(renderer.default_size());

        let selected = &self.selected;
        let state = tree.state.downcast_ref::<MenuState>();

        let bounds = layout.bounds();
        let is_mouse_over = cursor.is_over(bounds);

        let style = if !state.is_open && is_mouse_over {
            StyleSheet::hovered(theme, &self.style)
        } else {
            StyleSheet::active(theme, &self.style)
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: style.border,
                ..renderer::Quad::default()
            },
            style.background,
        );

        let icon = Text {
            content: &selected.borrow().icon().to_string(),
            size: icon_size * 1.25,
            line_height: LineHeight::default(),
            bounds: Size::new(
                bounds.width - self.padding.horizontal(),
                f32::from(LineHeight::default().to_absolute(icon_size)),
            ),
            font: self.icon_font,
            horizontal_alignment: alignment::Horizontal::Center,
            vertical_alignment: alignment::Vertical::Center,
            shaping: text::Shaping::default(),
        };

        renderer.fill_text(
            icon,
            Point::new(bounds.center_x(), bounds.center_y()),
            style.text_color,
            *viewport,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        let state = state.state.downcast_mut::<MenuState>();
        let text_font = self.text_font.unwrap_or(renderer.default_font());

        if state.is_open {
            let bounds = layout.bounds();
            let on_select = &self.on_select;

            let mut menu = Menu::new(
                &mut state.menu,
                self.options.borrow(),
                &mut state.hovered_option,
                |option| {
                    state.is_open = false;
                    (on_select)(option)
                },
                None,
                self.icon_font,
                &self.menu_style,
            )
            .width(self.menu_width.unwrap_or(state.width))
            .height(bounds.height)
            .padding(self.menu_padding)
            .text_font(text_font)
            .spacing(self.spacing);

            if let Some(text_size) = self.text_size {
                menu = menu.text_size(text_size);
            }

            if let Some(icon_size) = self.icon_size {
                menu = menu.icon_size(icon_size);
            }

            Some(menu.overlay(
                layout.position() + translation,
                match self.orientation {
                    ToolBarOrientation::Horizontal => Orientation::Horizontal(bounds.height),
                    ToolBarOrientation::Vertical => Orientation::Vertical(bounds.width),
                    ToolBarOrientation::Both => Orientation::Both(bounds.width, bounds.height),
                },
            ))
        } else {
            None
        }
    }
}

impl<'a, T, L, V, Message, Theme, Renderer> From<ToolbarMenu<'a, T, L, V, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    T: ToString + PartialEq + Clone + ToolbarOption + 'a,
    L: Borrow<[T]> + 'a,
    V: Borrow<T> + 'a,
    Theme: overlay::menu::StyleSheet + StyleSheet + scrollable::StyleSheet + 'a,
    Renderer: iced::advanced::Renderer + advanced::text::Renderer + 'a,
    Message: Clone + 'a,
{
    fn from(value: ToolbarMenu<'a, T, L, V, Message, Theme, Renderer>) -> Self {
        Self::new(value)
    }
}

struct MenuState {
    keyboard_modifiers: keyboard::Modifiers,
    is_open: bool,
    hovered_option: Option<usize>,
    menu: State,
    width: f32,
}

impl MenuState {
    fn new() -> Self {
        Self {
            menu: State::default(),
            keyboard_modifiers: keyboard::Modifiers::default(),
            is_open: false,
            hovered_option: None,
            width: 100.0,
        }
    }
}

#[derive(Debug)]
struct State {
    tree: Tree,
}

impl State {
    fn new() -> Self {
        Self {
            tree: Tree::empty(),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

struct Menu<'a, T, Message, Theme, Renderer>
where
    T: ToString + PartialEq + Clone + ToolbarOption,
    Renderer: advanced::text::Renderer + 'a,
    Theme: overlay::menu::StyleSheet + scrollable::StyleSheet + 'a,
{
    options: &'a [T],
    state: &'a mut State,
    hovered_option: &'a mut Option<usize>,
    on_select: Box<dyn FnMut(T) -> Message + 'a>,
    on_option_hovered: Option<&'a dyn Fn(T) -> Message>,
    width: f32,
    height: f32,
    padding: Padding,
    text_size: Option<Pixels>,
    text_font: Option<Renderer::Font>,
    icon_size: Option<Pixels>,
    icon_font: Renderer::Font,
    spacing: f32,
    style: &'a <Theme as overlay::menu::StyleSheet>::Style,
}

#[allow(dead_code)]
impl<'a, T, Message, Theme, Renderer> Menu<'a, T, Message, Theme, Renderer>
where
    T: ToString + PartialEq + Clone + ToolbarOption,
    Renderer: advanced::text::Renderer + 'a,
    Theme: overlay::menu::StyleSheet + scrollable::StyleSheet + 'a,
{
    fn new(
        state: &'a mut State,
        options: &'a [T],
        hovered_option: &'a mut Option<usize>,
        on_select: impl FnMut(T) -> Message + 'a,
        on_option_hovered: Option<&'a dyn Fn(T) -> Message>,
        icon_font: Renderer::Font,
        style: &'a <Theme as overlay::menu::StyleSheet>::Style,
    ) -> Self {
        Self {
            state,
            options,
            hovered_option,
            on_select: Box::new(on_select),
            on_option_hovered,
            text_font: None,
            text_size: None,
            icon_font,
            icon_size: None,
            width: 0.0,
            height: 0.0,
            padding: Padding::ZERO,
            spacing: 0.0,
            style,
        }
    }

    fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    fn text_font(mut self, font: Renderer::Font) -> Self {
        self.text_font = Some(font);
        self
    }

    fn text_size(mut self, text_size: impl Into<Pixels>) -> Self {
        self.text_size = Some(text_size.into());
        self
    }

    fn icon_font(mut self, font: Renderer::Font) -> Self {
        self.icon_font = font;
        self
    }

    fn icon_size(mut self, icon_size: impl Into<Pixels>) -> Self {
        self.icon_size = Some(icon_size.into());
        self
    }

    fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    fn overlay(
        self,
        position: Point,
        orientation: Orientation,
    ) -> advanced::overlay::Element<'a, Message, Theme, Renderer> {
        advanced::overlay::Element::new(Box::new(Overlay::new(position, self, orientation)))
    }
}

#[allow(dead_code)]
struct Overlay<'a, Message, Theme, Renderer>
where
    Renderer: advanced::text::Renderer,
    Theme: overlay::menu::StyleSheet + scrollable::StyleSheet,
{
    position: Point,
    state: &'a mut Tree,
    list: Scrollable<'a, Message, Theme, Renderer>,
    width: f32,
    height: f32,
    orientation: Orientation,
    style: &'a <Theme as overlay::menu::StyleSheet>::Style,
}

impl<'a, Message, Theme, Renderer> Overlay<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: advanced::text::Renderer + 'a,
    Theme: overlay::menu::StyleSheet + scrollable::StyleSheet + 'a,
{
    fn new<T>(
        position: Point,
        menu: Menu<'a, T, Message, Theme, Renderer>,
        orientation: Orientation,
    ) -> Self
    where
        T: ToString + Clone + PartialEq + ToolbarOption,
    {
        let Menu {
            options,
            state,
            hovered_option,
            on_select,
            on_option_hovered,
            width,
            height,
            padding,
            text_size,
            text_font,
            icon_size,
            icon_font,
            spacing,
            style,
        } = menu;

        let list = Scrollable::new(List {
            options,
            hovered_option,
            on_option_hovered,
            on_select,
            padding,
            text_size,
            text_font,
            icon_size,
            icon_font,
            spacing,
            style,
        })
        .direction(scrollable::Direction::default());

        state.tree.diff(&list as &dyn Widget<_, _, _>);

        Self {
            position,
            state: &mut state.tree,
            list,
            width,
            height,
            orientation,
            style,
        }
    }
}

impl<'a, Message, Theme, Renderer> advanced::overlay::Overlay<Message, Theme, Renderer>
    for Overlay<'a, Message, Theme, Renderer>
where
    Renderer: advanced::text::Renderer,
    Theme: overlay::menu::StyleSheet + scrollable::StyleSheet,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        match self.orientation {
            Orientation::Horizontal(target_height) => {
                let space_below = bounds.height - (self.position.y + target_height);
                let space_above = self.position.y;

                let limits = layout::Limits::new(
                    Size::ZERO,
                    Size::new(
                        bounds.width - self.position.x,
                        if space_below > space_above {
                            space_below
                        } else {
                            space_above
                        },
                    ),
                )
                .width(self.width);

                let node = self.list.layout(self.state, renderer, &limits);
                let size = node.size();

                node.move_to(if space_below > space_above {
                    self.position + Vector::new(0.0, target_height)
                } else {
                    self.position - Vector::new(0.0, size.height)
                })
            }
            Orientation::Vertical(target_width) => {
                let left_space = self.position.x;
                let right_space = bounds.width - (self.position.x + target_width);

                let limits = layout::Limits::new(
                    Size::ZERO,
                    Size::new(
                        if right_space > left_space {
                            right_space
                        } else {
                            left_space
                        },
                        bounds.height - self.position.y,
                    ),
                )
                .width(self.width);

                let node = self.list.layout(self.state, renderer, &limits);
                let size = node.size();

                node.move_to(if right_space > left_space {
                    self.position + Vector::new(target_width, 0.0)
                } else {
                    self.position - Vector::new(size.width, 0.0)
                })
            }

            Orientation::Both(width, height) => {
                let space_below = bounds.height - (self.position.y + height);
                let space_above = self.position.y;

                let left_space = self.position.x;
                let right_space = bounds.width - (self.position.x + width);

                if space_below > space_above && right_space > left_space {
                    let limits =
                        layout::Limits::new(Size::ZERO, Size::new(right_space, space_below))
                            .width(self.width);
                    let node = self.list.layout(self.state, renderer, &limits);

                    return node.move_to(self.position + Vector::new(width, 0.0));
                } else if space_below > space_above {
                    let limits =
                        layout::Limits::new(Size::ZERO, Size::new(left_space, space_below))
                            .width(self.width);
                    let node = self.list.layout(self.state, renderer, &limits);
                    let size = node.size();

                    return node.move_to(self.position - Vector::new(size.width, 0.0));
                } else if right_space > left_space {
                    let limits =
                        layout::Limits::new(Size::ZERO, Size::new(right_space, space_above))
                            .width(self.width);
                    let node = self.list.layout(self.state, renderer, &limits);
                    let size = node.size();

                    return node.move_to(self.position + Vector::new(width, height - size.height));
                } else {
                    let limits =
                        layout::Limits::new(Size::ZERO, Size::new(left_space, space_above))
                            .width(self.width);
                    let node = self.list.layout(self.state, renderer, &limits);
                    let size = node.size();
                    return node
                        .move_to(self.position - Vector::new(size.width, size.height - height));
                };
            }
        }
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn advanced::Clipboard,
        shell: &mut advanced::Shell<'_, Message>,
    ) -> advanced::graphics::core::event::Status {
        let bounds = layout.bounds();

        self.list.on_event(
            self.state, event, layout, cursor, renderer, clipboard, shell, &bounds,
        )
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.list
            .mouse_interaction(self.state, layout, cursor, viewport, renderer)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let bounds = layout.bounds();
        let inner_style = theme.appearance(&self.style);

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: inner_style.border,
                ..renderer::Quad::default()
            },
            inner_style.background,
        );

        self.list
            .draw(&self.state, renderer, theme, style, layout, cursor, &bounds)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ToolBarOrientation {
    Horizontal,
    Both,
    #[default]
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Orientation {
    Vertical(f32),
    Horizontal(f32),
    Both(f32, f32),
}

struct List<'a, T, Message, Theme, Renderer>
where
    T: ToString + PartialEq + Clone + ToolbarOption,
    Renderer: advanced::text::Renderer,
    Theme: overlay::menu::StyleSheet,
{
    options: &'a [T],
    hovered_option: &'a mut Option<usize>,
    on_option_hovered: Option<&'a dyn Fn(T) -> Message>,
    on_select: Box<dyn FnMut(T) -> Message + 'a>,
    padding: Padding,
    text_size: Option<Pixels>,
    text_font: Option<Renderer::Font>,
    icon_size: Option<Pixels>,
    icon_font: Renderer::Font,
    spacing: f32,
    style: &'a <Theme as overlay::menu::StyleSheet>::Style,
}

impl<'a, T, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for List<'a, T, Message, Theme, Renderer>
where
    T: ToString + PartialEq + Clone + ToolbarOption,
    Renderer: advanced::text::Renderer,
    Theme: overlay::menu::StyleSheet,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Shrink,
        }
    }

    fn layout(
        &self,
        _tree: &mut advanced::widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        use std::f32;

        let text_size = self.text_size.unwrap_or(renderer.default_size());

        let text_line_height = LineHeight::default().to_absolute(text_size);

        let size = {
            let intrinsic = Size::new(
                0.0,
                (f32::from(text_line_height) + self.padding.vertical()) * self.options.len() as f32,
            );

            limits.resolve(Length::Fill, Length::Shrink, intrinsic)
        };

        layout::Node::new(size)
    }

    fn draw(
        &self,
        _tree: &advanced::widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &advanced::renderer::Style,
        layout: layout::Layout<'_>,
        _cursor: advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        use std::f32;

        let style = theme.appearance(self.style);
        let bounds = layout.bounds();

        let text_size = self.text_size.unwrap_or(renderer.default_size());
        let icon_size = self.icon_size.unwrap_or(renderer.default_size());

        let text_line_height = LineHeight::default().to_absolute(text_size);

        let option_height = f32::from(text_line_height) + self.padding.vertical();

        let offset = viewport.y - bounds.y;
        let start = (offset / option_height) as usize;
        let end = ((offset + viewport.height) / option_height).ceil() as usize;

        let visible_options = &self.options[start..end.min(self.options.len())];

        for (i, option) in visible_options.iter().enumerate() {
            let icon = option.icon();
            let i = start + i;
            let is_selected = *self.hovered_option == Some(i);

            let bounds = Rectangle {
                x: bounds.x,
                y: bounds.y + (option_height * i as f32),
                width: bounds.width,
                height: option_height,
            };

            if is_selected {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x + style.border.width,
                            width: bounds.width - style.border.width,
                            ..bounds
                        },
                        border: style.border,
                        ..renderer::Quad::default()
                    },
                    style.selected_background,
                )
            }

            let icon_text = Text {
                content: &icon.to_string(),
                bounds: Size::new(f32::INFINITY, bounds.height),
                size: icon_size,
                line_height: LineHeight::default(),
                font: self.icon_font,
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Center,
                shaping: text::Shaping::default(),
            };

            renderer.fill_text(
                icon_text,
                Point::new(bounds.x + self.padding.left, bounds.center_y()),
                if is_selected {
                    style.selected_text_color
                } else {
                    style.text_color
                },
                *viewport,
            );

            renderer.fill_text(
                Text {
                    content: &option.to_string(),
                    bounds: Size::new(f32::INFINITY, bounds.height),
                    size: text_size,
                    line_height: LineHeight::default(),
                    font: self.text_font.unwrap_or(renderer.default_font()),
                    horizontal_alignment: alignment::Horizontal::Left,
                    vertical_alignment: alignment::Vertical::Center,
                    shaping: text::Shaping::default(),
                },
                Point::new(
                    bounds.x + self.padding.left + icon_text.size.0 + self.spacing,
                    bounds.center_y(),
                ),
                if is_selected {
                    style.selected_text_color
                } else {
                    style.text_color
                },
                *viewport,
            );
        }
    }

    fn mouse_interaction(
        &self,
        _state: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let is_mouse_over = cursor.is_over(layout.bounds());

        if is_mouse_over {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn on_event(
        &mut self,
        _state: &mut Tree,
        event: iced::Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        _clipboard: &mut dyn advanced::Clipboard,
        shell: &mut advanced::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        let bounds = layout.bounds();

        let text_size = self.text_size.unwrap_or(renderer.default_size());

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if cursor.is_over(bounds) {
                    if let Some(index) = *self.hovered_option {
                        if let Some(option) = self.options.get(index) {
                            shell.publish((self.on_select)(option.clone()));
                            return event::Status::Captured;
                        }
                    }
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(cursor_position) = cursor.position_in(bounds) {
                    let option_height = f32::from(LineHeight::default().to_absolute(text_size))
                        + self.padding.vertical();

                    let new_hovered_option = (cursor_position.y / option_height) as usize;

                    if let Some(on_option_hovered) = self.on_option_hovered {
                        if *self.hovered_option != Some(new_hovered_option) {
                            if let Some(option) = self.options.get(new_hovered_option) {
                                shell.publish((on_option_hovered)(option.clone()))
                            }
                        }
                    }

                    *self.hovered_option = Some(new_hovered_option);
                }
            }

            Event::Touch(touch::Event::FingerPressed { .. }) => {
                if let Some(cursor_position) = cursor.position_in(bounds) {
                    let option_height = f32::from(LineHeight::default().to_absolute(text_size))
                        + self.padding.vertical();

                    *self.hovered_option = Some((cursor_position.y / option_height) as usize);

                    if let Some(index) = *self.hovered_option {
                        if let Some(option) = self.options.get(index) {
                            shell.publish((self.on_select)(option.clone()));
                            return event::Status::Captured;
                        }
                    }
                }
            }

            _ => {}
        }

        event::Status::Ignored
    }
}

impl<'a, T, Message, Theme, Renderer> From<List<'a, T, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    T: ToString + PartialEq + Clone + ToolbarOption + 'a,
    Message: 'a,
    Renderer: 'a + advanced::text::Renderer,
    Theme: 'a + overlay::menu::StyleSheet,
{
    fn from(value: List<'a, T, Message, Theme, Renderer>) -> Self {
        Element::new(value)
    }
}

