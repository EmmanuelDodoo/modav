// Code derived from Iced Github repo

use crate::styles::ColoredContainer;
use crate::utils::icons::toast;
use std::{
    fmt::Display,
    time::{Duration, Instant},
};

use iced::{
    advanced::{
        layout, overlay, renderer,
        widget::{self, Operation, Tree},
        Clipboard, Layout, Shell,
    },
    color,
    event::{self, Event},
    mouse, theme,
    widget::{button, column, container, horizontal_space, row, text, Text},
    window, Alignment, Color, Element, Length, Point, Rectangle, Renderer, Size, Theme, Vector,
};

use iced::advanced::Widget;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Status {
    #[default]
    Info,
    Warn,
    Success,
    Error,
}

impl Status {
    fn color(&self, is_dark: bool) -> Color {
        match (self, is_dark) {
            (Self::Success, true) => color!(74, 222, 128),
            (Self::Success, false) => color!(5, 46, 22),
            (Self::Info, true) => color!(125, 211, 252),
            (Self::Info, false) => color!(30, 58, 138),
            (Self::Warn, true) => color!(250, 204, 21),
            (Self::Warn, false) => color!(69, 26, 3),
            (Self::Error, true) => color!(248, 113, 113),
            (Self::Error, false) => color!(69, 10, 10),
        }
    }

    fn icon(&self, is_dark: bool) -> Text<'static> {
        let style = theme::Text::Color(self.color(is_dark));
        match self {
            Self::Success => toast::icon(toast::SUCCESS).style(style),
            Self::Info => toast::icon(toast::INFO).style(style),
            Self::Warn => toast::icon(toast::WARN).style(style),
            Self::Error => toast::icon(toast::ERROR).style(style),
        }
    }

    fn background_color(&self, is_dark: bool) -> Color {
        match (self, is_dark) {
            (Self::Success, true) => color!(20, 83, 45),
            (Self::Success, false) => color!(74, 222, 128),
            (Self::Info, true) => color!(30, 64, 175),
            (Self::Info, false) => color!(147, 197, 253),
            (Self::Warn, true) => color!(180, 83, 9),
            (Self::Warn, false) => color!(245, 158, 11),
            (Self::Error, true) => color!(153, 27, 27),
            (Self::Error, false) => color!(248, 113, 113),
        }
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Info => "Info",
            Status::Error => "Error",
            Status::Success => "Success",
            Status::Warn => "Warning",
        }
        .fmt(f)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Toast {
    pub body: String,
    pub status: Status,
}

pub struct Manager<'a, Message> {
    content: Element<'a, Message>,
    toasts: Vec<Element<'a, Message>>,
    timeout: u64,
    on_close: Box<dyn Fn(usize) -> Message + 'a>,
}

impl<'a, Message> Manager<'a, Message>
where
    Message: 'a + Clone,
{
    pub fn new(
        content: impl Into<Element<'a, Message>>,
        toasts: &'a [Toast],
        on_close: impl Fn(usize) -> Message + 'a,
        theme: &Theme,
    ) -> Self {
        let toasts = toasts
            .iter()
            .enumerate()
            .map(|(index, toast)| {
                let text_color = toast.status.color(theme.extended_palette().is_dark);

                let status_icon = toast.status.icon(theme.extended_palette().is_dark).size(28);

                let content = {
                    let title = text(toast.status.to_string())
                        .size(18)
                        .style(theme::Text::Color(text_color));
                    let body = text(toast.body.as_str())
                        .size(15)
                        .style(theme::Text::Color(text_color));

                    column!(title, body).spacing(4).height(Length::Fill)
                };

                let close = button(
                    toast::icon(toast::CLOSE)
                        .size(24)
                        .style(theme::Text::Color(text_color)),
                )
                .on_press((on_close)(index))
                .style(theme::Button::Text);

                let background = ColoredContainer {
                    color: toast
                        .status
                        .background_color(theme.extended_palette().is_dark),
                    radius: 5.0,
                };

                container(
                    row!(status_icon, content, horizontal_space(), close)
                        .align_items(Alignment::Center)
                        .height(Length::Fill)
                        .spacing(16),
                )
                .max_width(350)
                .height(65)
                .padding([6.0, 8.0])
                .style(theme::Container::Custom(Box::new(background)))
                .into()
            })
            .collect();

        Self {
            content: content.into(),
            toasts,
            timeout: 5,
            on_close: Box::new(on_close),
        }
    }

    pub fn timeout(mut self, seconds: u64) -> Self {
        self.timeout = seconds;
        self
    }
}

impl<'a, Message> Widget<Message, Theme, Renderer> for Manager<'a, Message> {
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn tag(&self) -> widget::tree::Tag {
        struct Marker;
        widget::tree::Tag::of::<Marker>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(Vec::<Option<Instant>>::new())
    }

    fn children(&self) -> Vec<Tree> {
        std::iter::once(Tree::new(&self.content))
            .chain(self.toasts.iter().map(Tree::new))
            .collect()
    }

    fn diff(&self, tree: &mut Tree) {
        let instants = tree.state.downcast_mut::<Vec<Option<Instant>>>();

        // Invalidating removed instants to None allows us to remove
        // them here so that diffing for removed / new toast instants
        // is accurate
        instants.retain(Option::is_some);

        match (instants.len(), self.toasts.len()) {
            (old, new) if old > new => {
                instants.truncate(new);
            }
            (old, new) if old < new => {
                instants.extend(std::iter::repeat(Some(Instant::now())).take(new - old));
            }
            _ => {}
        }

        tree.diff_children(
            &std::iter::once(&self.content)
                .chain(self.toasts.iter())
                .collect::<Vec<_>>(),
        );
    }

    fn operate(
        &self,
        state: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<Message>,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.content
                .as_widget()
                .operate(&mut state.children[0], layout, renderer, operation);
        });
    }

    fn on_event(
        &mut self,
        state: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        self.content.as_widget_mut().on_event(
            &mut state.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        )
    }

    fn draw(
        &self,
        state: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &state.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &state.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let instants = state.state.downcast_mut::<Vec<Option<Instant>>>();

        let (content_state, toasts_state) = state.children.split_at_mut(1);

        let content = self.content.as_widget_mut().overlay(
            &mut content_state[0],
            layout,
            renderer,
            translation,
        );

        let toasts = (!self.toasts.is_empty()).then(|| {
            overlay::Element::new(Box::new(Overlay {
                position: layout.bounds().position() + translation,
                toasts: &mut self.toasts,
                state: toasts_state,
                instants,
                on_close: &self.on_close,
                timeout: self.timeout,
            }))
        });
        let overlays = content.into_iter().chain(toasts).collect::<Vec<_>>();

        (!overlays.is_empty()).then(|| overlay::Group::with_children(overlays).overlay())
    }
}

impl<'a, Message> From<Manager<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(value: Manager<'a, Message>) -> Self {
        Element::new(value)
    }
}

struct Overlay<'a, 'b, Message> {
    position: Point,
    toasts: &'b mut [Element<'a, Message>],
    state: &'b mut [Tree],
    instants: &'b mut [Option<Instant>],
    on_close: &'b dyn Fn(usize) -> Message,
    timeout: u64,
}

impl<'a, 'b, Message> overlay::Overlay<Message, Theme, Renderer> for Overlay<'a, 'b, Message> {
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let limits = layout::Limits::new(Size::ZERO, bounds);

        layout::flex::resolve(
            layout::flex::Axis::Vertical,
            renderer,
            &limits,
            Length::Fill,
            Length::Fill,
            10.into(),
            10.0,
            Alignment::End,
            self.toasts,
            self.state,
        )
        .translate(Vector::new(self.position.x, self.position.y))
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        if let Event::Window(_, window::Event::RedrawRequested(now)) = &event {
            let mut next_redraw: Option<window::RedrawRequest> = None;

            self.instants
                .iter_mut()
                .enumerate()
                .for_each(|(index, maybe_instant)| {
                    if let Some(instant) = maybe_instant.as_mut() {
                        let remaining =
                            Duration::from_secs(self.timeout).saturating_sub(instant.elapsed());

                        if remaining == Duration::ZERO {
                            maybe_instant.take();
                            shell.publish((self.on_close)(index));
                            next_redraw = Some(window::RedrawRequest::NextFrame);
                        } else {
                            let redraw_at = window::RedrawRequest::At(*now + remaining);
                            next_redraw = next_redraw
                                .map(|redraw| redraw.min(redraw_at))
                                .or(Some(redraw_at));
                        }
                    }
                });

            if let Some(redraw) = next_redraw {
                shell.request_redraw(redraw);
            }
        }

        let viewport = layout.bounds();

        self.toasts
            .iter_mut()
            .zip(self.state.iter_mut())
            .zip(layout.children())
            .zip(self.instants.iter_mut())
            .map(|(((child, state), layout), instant)| {
                let mut local_messages = vec![];
                let mut local_shell = Shell::new(&mut local_messages);

                let status = child.as_widget_mut().on_event(
                    state,
                    event.clone(),
                    layout,
                    cursor,
                    renderer,
                    clipboard,
                    &mut local_shell,
                    &viewport,
                );

                if !local_shell.is_empty() {
                    instant.take();
                }

                shell.merge(local_shell, std::convert::identity);

                status
            })
            .fold(event::Status::Ignored, event::Status::merge)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let viewport = layout.bounds();

        for ((child, state), layout) in self
            .toasts
            .iter()
            .zip(self.state.iter())
            .zip(layout.children())
        {
            child
                .as_widget()
                .draw(state, renderer, theme, style, layout, cursor, &viewport);
        }
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<Message>,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.toasts
                .iter()
                .zip(self.state.iter_mut())
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .as_widget()
                        .operate(state, layout, renderer, operation);
                });
        });
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.toasts
            .iter()
            .zip(self.state.iter())
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(state, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn is_over(&self, layout: Layout<'_>, _renderer: &Renderer, cursor_position: Point) -> bool {
        layout
            .children()
            .any(|layout| layout.bounds().contains(cursor_position))
    }
}
