use iced::{
    alignment,
    widget::{button, text, Button},
    Font,
};

use crate::utils::icons;

pub mod graph;
pub mod styles;

pub use styles::*;

pub fn tools_button<'a, Message>() -> Button<'a, Message> {
    button(
        text(icons::TOOLS)
            .font(Font::with_name(icons::NAME))
            .width(18.0)
            .align_y(alignment::Vertical::Center)
            .align_x(alignment::Horizontal::Center),
    )
    .padding([4, 8])
    .style(|theme, status| <ToolsButton as button::Catalog>::style(&ToolsButton, theme, status))
}
