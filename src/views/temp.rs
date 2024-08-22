use iced::{
    theme::Theme,
    widget::{button, column, container, row, text},
    Alignment, Element,
};

use super::{TabLabel, Viewable};
use crate::Message;

#[derive(Clone, Debug)]
pub enum CounterMessage {
    Increase,
    Decrease,
}

#[derive(Clone, Debug)]
pub struct CounterTab {
    value: i32,
}

impl Viewable for CounterTab {
    type Event = CounterMessage;
    type Data = ();

    fn new(_data: ()) -> Self {
        Self { value: 0 }
    }

    fn is_dirty(&self) -> bool {
        false
    }

    fn update(&mut self, message: CounterMessage) -> Option<Message> {
        match message {
            CounterMessage::Increase => {
                self.value += 1;
            }
            CounterMessage::Decrease => {
                self.value -= 1;
            }
        };

        None
    }

    fn label(&self) -> TabLabel {
        TabLabel::new(char::default(), "Counter")
    }

    fn view<'a, Message, F>(&'a self, map: F) -> Element<'a, Message, Theme>
    where
        F: 'a + Fn(Self::Event) -> Message,
        Message: 'a,
    {
        let header = text(format!("Count {}", self.value)).size(32);

        let rw = {
            let btn1 = button("Increase").on_press(CounterMessage::Increase);
            let btn2 = button("Decrease").on_press(CounterMessage::Decrease);

            row!(btn1, btn2).spacing(10)
        };

        let content = column!(header, rw)
            .align_items(Alignment::Center)
            .max_width(600)
            .padding(20)
            .spacing(16);

        let content: Element<'_, CounterMessage, Theme> = container(content).into();

        content.map(map)
    }

    fn modal_msg(&self) -> String {
        String::from("Counter Modal Message here")
    }

    fn refresh(&mut self, _data: Self::Data) {}
}
