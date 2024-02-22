use crate::views::{TabBarMessage, TabMessage};
use iced::Element;
use iced_aw::TabLabel;
use iced_widget::{button, column, container, row, text};

#[derive(Clone, Debug)]
pub enum CounterMessage {
    Increase,
    Decrease,
}

#[derive(Clone, Debug)]
pub struct CounterTab {
    value: i32,
    id: usize,
}

impl CounterTab {
    pub fn new(id: usize) -> Self {
        Self { id, value: 0 }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn is_dirty(&self) -> bool {
        false
    }

    pub fn update(&mut self, message: CounterMessage) {
        match message {
            CounterMessage::Increase => {
                self.value += 1;
            }
            CounterMessage::Decrease => {
                self.value -= 1;
            }
        }
    }

    pub fn tab_label(&self) -> TabLabel {
        TabLabel::Text(format!("Counter {}", self.id))
    }

    pub fn content(&self) -> iced::Element<'_, TabBarMessage> {
        let header = text(format!("Count {}", self.value)).size(32);

        let rw = {
            let btn1 = button("Increase").on_press(CounterMessage::Increase);
            let btn2 = button("Decrease").on_press(CounterMessage::Decrease);

            row!(btn1, btn2).spacing(10)
        };

        let content = column!(header, rw)
            .align_items(iced::Alignment::Center)
            .max_width(600)
            .padding(20)
            .spacing(16);

        let content: Element<'_, CounterMessage> = container(content).into();

        content.map(|msg| TabBarMessage::UpdateTab((self.id, TabMessage::Counter(msg))))
    }
}
