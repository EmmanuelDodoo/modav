use iced::{
    advanced::{
        self, layout,
        widget::{tree, Operation},
        Widget,
    },
    alignment, color, event, mouse, overlay, touch,
    widget::{self, text::LineHeight, Scrollable},
    Background, Element, Event, Font, Length, Padding, Point, Rectangle, Renderer, Size, Theme,
    Vector,
};

use crate::utils::icons;

/// An icon widget which when clicked opens an overlay on the right
pub struct Tools<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Renderer: advanced::Renderer + advanced::text::Renderer,
    Theme: overlay::menu::StyleSheet + widget::button::StyleSheet,
{
    on_open: Option<Message>,
    on_close: Option<Message>,
    width: Length,
    height: Length,
    spacing: f32,
    children: Vec<Element<'a, Message, Theme, Renderer>>,
    icon: char,
    icon_size: f32,
    icon_font: Renderer::Font,
    padding: Padding,
    menu_padding: Padding,
    style: <Theme as widget::button::StyleSheet>::Style,
    menu_style: <Theme as overlay::menu::StyleSheet>::Style,
}

impl<'a, Message> Tools<'a, Message> {
    pub fn new() -> Self {
        Self::from_children(vec![])
    }

    pub fn from_children(
        children: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        let children = children.into_iter().collect();

        Self {
            children,
            on_open: None,
            on_close: None,
            width: Length::Shrink,
            height: Length::Shrink,
            spacing: 0.0,
            padding: [4, 6].into(),
            icon: icons::TOOLS,
            icon_size: 18.0,
            icon_font: Font::with_name(icons::NAME),
            menu_padding: Padding::ZERO,
            style: <Theme as widget::button::StyleSheet>::Style::default(),
            menu_style: <Theme as overlay::menu::StyleSheet>::Style::default(),
        }
    }

    pub fn push(mut self, child: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        let child = child.into();

        self.children.push(child);
        self
    }

    pub fn on_open(mut self, message: Message) -> Self {
        self.on_open = Some(message);
        self
    }

    pub fn on_close(mut self, message: Message) -> Self {
        self.on_close = Some(message);
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

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn icon(mut self, icon: char) -> Self {
        self.icon = icon;
        self
    }

    pub fn icon_font(mut self, font: Font) -> Self {
        self.icon_font = font;
        self
    }

    pub fn menu_padding(mut self, padding: impl Into<Padding>) -> Self {
        self.menu_padding = padding.into();
        self
    }

    pub fn menu_style(mut self, style: <Theme as overlay::menu::StyleSheet>::Style) -> Self {
        self.menu_style = style;
        self
    }
}

impl<'a, Message> Widget<Message, Theme, Renderer> for Tools<'a, Message>
where
    Renderer: advanced::Renderer + advanced::text::Renderer<Font = iced::Font>,
    Message: Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<MenuState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(MenuState::new())
    }

    fn size(&self) -> iced::Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    //fn diff(&self, tree: &mut tree::Tree) {
    //    tree.diff_children(&self.children)
    //}

    fn layout(
        &self,
        _tree: &mut tree::Tree,
        _renderer: &Renderer,
        limits: &advanced::layout::Limits,
    ) -> advanced::layout::Node {
        //let dummy: Text<'_, Theme, Renderer> = widget::text(self.icon)
        //    .font(self.icon_font.clone())
        //    .width(self.width)
        //    .height(self.height)
        //    .size(self.icon_size);
        //
        //let _temp: Row<'_, Message, _, Renderer> = row!(dummy).padding(self.padding);
        //temp.layout(tree, renderer, limits)

        let size = {
            let intrinsic = Size::new(
                self.icon_size + self.padding.left,
                f32::from(LineHeight::default().to_absolute(self.icon_size.into())),
            );

            limits
                .width(Length::Shrink)
                .height(Length::Shrink)
                .shrink(self.padding)
                .resolve(Length::Shrink, Length::Shrink, intrinsic)
                .expand(self.padding)
        };

        layout::Node::new(size)
    }

    fn draw(
        &self,
        _tree: &tree::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &advanced::renderer::Style,
        layout: advanced::Layout<'_>,
        cursor: advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        //let state = tree.state.downcast_ref::<MenuState>();
        let bounds = layout.bounds();
        let is_mouse_over = cursor.is_over(bounds);

        let style = if is_mouse_over {
            widget::button::StyleSheet::hovered(theme, &self.style)
        } else {
            widget::button::StyleSheet::active(theme, &self.style)
        };

        <Renderer as advanced::Renderer>::fill_quad(
            renderer,
            advanced::renderer::Quad {
                bounds,
                border: style.border,
                shadow: style.shadow,
            },
            style
                .background
                .unwrap_or(Background::Color(color!(50.0, 100.0, 225.0, 0.9))),
        );

        let icon = advanced::text::Text {
            content: &self.icon.to_string(),
            size: self.icon_size.into(),
            line_height: LineHeight::default(),
            bounds: Size::new(
                bounds.width - self.padding.horizontal(),
                f32::from(LineHeight::default().to_absolute(self.icon_size.into())),
            ),
            font: self.icon_font,
            horizontal_alignment: alignment::Horizontal::Center,
            vertical_alignment: alignment::Vertical::Center,
            shaping: advanced::text::Shaping::default(),
        };

        <Renderer as advanced::text::Renderer>::fill_text(
            renderer,
            icon,
            Point::new(bounds.center_x(), bounds.center_y()),
            style.text_color,
            *viewport,
        )
    }

    fn mouse_interaction(
        &self,
        _state: &tree::Tree,
        layout: layout::Layout<'_>,
        cursor: advanced::mouse::Cursor,
        _viewport: &iced::Rectangle,
        _renderer: &Renderer,
    ) -> advanced::mouse::Interaction {
        let bounds = layout.bounds();
        let is_mouse_over = cursor.is_over(bounds);

        if is_mouse_over {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn on_event(
        &mut self,
        state: &mut tree::Tree,
        event: iced::Event,
        layout: layout::Layout<'_>,
        cursor: advanced::mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn advanced::Clipboard,
        shell: &mut advanced::Shell<'_, Message>,
        _viewport: &iced::Rectangle,
    ) -> event::Status {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                let state = state.state.downcast_mut::<MenuState>();

                if state.is_open {
                    if let Some(_bounds) = state.overlay_bounds {
                        // This is a hack. For some reason, the cursor is
                        // unavailable when clicking in the overlay
                        //if cursor.is_over(bounds){
                        if cursor.position().is_none() {
                            return event::Status::Ignored;
                        }
                    };

                    state.is_open = false;
                    state.overlay_bounds.take();

                    if let Some(on_close) = &self.on_close {
                        shell.publish(on_close.clone());
                    }

                    event::Status::Captured
                } else if cursor.is_over(layout.bounds()) {
                    state.is_open = true;

                    if let Some(on_open) = &self.on_open {
                        shell.publish(on_open.clone())
                    }

                    event::Status::Captured
                } else {
                    event::Status::Ignored
                }
            }
            _ => event::Status::Ignored,
        }
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut tree::Tree,
        layout: layout::Layout<'_>,
        _renderer: &Renderer,
        translation: Vector,
    ) -> Option<advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        let state = state.state.downcast_mut::<MenuState>();

        if !state.is_open {
            return None;
        }

        let children = unsafe {
            std::mem::transmute::<&mut Vec<Element<'a, Message>>, &mut Vec<Element<'b, Message>>>(
                &mut self.children,
            )
        };

        let overlay: Overlay<'_, Message> = Overlay::new(
            state,
            children,
            self.spacing,
            self.padding,
            layout.position() + translation,
            &self.menu_style,
        );

        let overlay = advanced::overlay::Element::new(Box::new(overlay));

        return Some(overlay);
    }
}

impl<'a, Message> From<Tools<'a, Message>> for Element<'a, Message>
where
    Message: 'a + Clone,
{
    fn from(value: Tools<'a, Message>) -> Self {
        Self::new(value)
    }
}

#[derive(Debug)]
struct MenuState {
    tree: tree::Tree,
    is_open: bool,
    overlay_bounds: Option<Rectangle>,
}

impl MenuState {
    fn new() -> Self {
        Self {
            tree: tree::Tree::empty(),
            is_open: false,
            overlay_bounds: None,
        }
    }
}

struct Overlay<'a, Message> {
    position: Point,
    state: &'a mut MenuState,
    list: Scrollable<'a, Message, iced::Theme, iced::Renderer>,
    style: &'a <Theme as overlay::menu::StyleSheet>::Style,
}

#[allow(dead_code)]
impl<'a, Message> Overlay<'a, Message> {
    fn new(
        state: &'a mut MenuState,
        elements: &'a mut [Element<'a, Message, Theme>],
        spacing: f32,
        padding: Padding,
        position: Point,
        style: &'a <Theme as overlay::menu::StyleSheet>::Style,
    ) -> Self {
        let list = List {
            elements,
            spacing,
            padding,
        };

        let list = Scrollable::new(list);

        state.tree.diff(&list as &dyn Widget<_, _, _>);

        Self {
            state,
            position,
            list,
            style,
        }
    }

    fn style(mut self, style: &'a <Theme as overlay::menu::StyleSheet>::Style) -> Self {
        self.style = style;
        self
    }
}

impl<'a, Message> advanced::overlay::Overlay<Message, Theme, Renderer> for Overlay<'a, Message> {
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let left_space = self.position.x;

        let limits = layout::Limits::new(
            Size::ZERO,
            Size::new(left_space, bounds.height - self.position.y),
        );

        let node = self.list.layout(&mut self.state.tree, renderer, &limits);
        let size = node.size();

        let node = node.move_to(self.position - Vector::new(size.width, 0.0));

        self.state.overlay_bounds = Some(node.bounds());

        node
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &advanced::renderer::Style,
        layout: layout::Layout<'_>,
        cursor: advanced::mouse::Cursor,
    ) {
        let bounds = layout.bounds();
        let own_style = <Theme as overlay::menu::StyleSheet>::appearance(theme, &self.style);

        <Renderer as advanced::Renderer>::fill_quad(
            renderer,
            advanced::renderer::Quad {
                bounds,
                border: own_style.border,
                ..Default::default()
            },
            own_style.background,
        );

        self.list.draw(
            &self.state.tree,
            renderer,
            theme,
            style,
            layout,
            cursor,
            &bounds,
        )
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn advanced::Clipboard,
        shell: &mut advanced::Shell<'_, Message>,
    ) -> advanced::graphics::core::event::Status {
        let bounds = layout.bounds();

        self.list.on_event(
            &mut self.state.tree,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &bounds,
        )
    }

    fn mouse_interaction(
        &self,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.list
            .mouse_interaction(&self.state.tree, layout, cursor, viewport, renderer)
    }
}

/// Because normal wdigets can't hold &\[Element\]
struct List<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Theme: overlay::menu::StyleSheet,
    Renderer: advanced::Renderer,
{
    elements: &'a mut [Element<'a, Message, Theme, Renderer>],
    spacing: f32,
    padding: Padding,
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for List<'a, Message, Theme, Renderer>
where
    Theme: overlay::menu::StyleSheet,
    Renderer: advanced::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    fn diff(&self, tree: &mut tree::Tree) {
        tree.diff_children(self.elements)
    }

    fn children(&self) -> Vec<tree::Tree> {
        self.elements.iter().map(tree::Tree::new).collect()
    }

    fn layout(
        &self,
        tree: &mut tree::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::flex::resolve(
            layout::flex::Axis::Vertical,
            renderer,
            limits,
            Length::Shrink,
            Length::Shrink,
            self.padding,
            self.spacing,
            iced::Alignment::Center,
            &self.elements,
            &mut tree.children,
        )
    }

    fn draw(
        &self,
        tree: &tree::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &advanced::renderer::Style,
        layout: layout::Layout<'_>,
        cursor: advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        if let Some(clipped_viewport) = layout.bounds().intersection(viewport) {
            for ((child, state), layout) in self
                .elements
                .iter()
                .zip(&tree.children)
                .zip(layout.children())
            {
                child.as_widget().draw(
                    state,
                    renderer,
                    theme,
                    style,
                    layout,
                    cursor,
                    &clipped_viewport,
                );
            }
        }
    }

    fn on_event(
        &mut self,
        tree: &mut tree::Tree,
        event: Event,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn advanced::Clipboard,
        shell: &mut advanced::Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        self.elements
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child.as_widget_mut().on_event(
                    state,
                    event.clone(),
                    layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                )
            })
            .fold(event::Status::Ignored, event::Status::merge)
    }

    fn mouse_interaction(
        &self,
        tree: &tree::Tree,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.elements
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(state, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn operate(
        &self,
        tree: &mut tree::Tree,
        layout: layout::Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<Message>,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.elements
                .iter()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .as_widget()
                        .operate(state, layout, renderer, operation);
                });
        });
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut tree::Tree,
        layout: layout::Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        advanced::overlay::from_children(&mut self.elements, tree, layout, renderer, translation)
    }
}

impl<'a, Message, Theme, Renderer> From<List<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Theme: overlay::menu::StyleSheet + 'a,
    Renderer: advanced::Renderer + 'a,
    Message: 'a,
{
    fn from(value: List<'a, Message, Theme, Renderer>) -> Self {
        Self::new(value)
    }
}
