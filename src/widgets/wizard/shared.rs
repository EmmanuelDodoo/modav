use crate::{utils::icons, ToolTipContainerStyle};

use iced::{
    alignment, theme,
    widget::{container, text, tooltip::Tooltip},
    Length,
};

use iced::widget::tooltip as tt;

pub fn tooltip<'a, Message>(description: impl ToString) -> Tooltip<'a, Message>
where
    Message: 'a,
{
    let text = text(description).size(13.0);
    let desc = container(text)
        .max_width(200.0)
        .padding([6.0, 8.0])
        .height(Length::Shrink)
        .style(theme::Container::Custom(Box::new(ToolTipContainerStyle)));

    let icon = icons::icon(icons::HELP)
        .horizontal_alignment(alignment::Horizontal::Center)
        .vertical_alignment(alignment::Vertical::Center);

    Tooltip::new(icon, desc, tt::Position::Right)
        .gap(10.0)
        .snap_within_viewport(true)
}
