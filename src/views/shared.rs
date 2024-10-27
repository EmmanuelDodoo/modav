use iced::{
    alignment, theme,
    widget::{button, text, Button},
    Font,
};

use crate::utils::icons;

pub mod graph;
pub mod styles;

pub use graph::*;
pub use styles::*;

pub fn tools_button<'a, Message>() -> Button<'a, Message> {
    button(
        text(icons::TOOLS)
            .font(Font::with_name(icons::NAME))
            .width(18.0)
            .vertical_alignment(alignment::Vertical::Center)
            .horizontal_alignment(alignment::Horizontal::Center),
    )
    .padding([4, 8])
    .style(theme::Button::Custom(Box::new(ToolsButton)))
}
