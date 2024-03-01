use super::{TabBarMessage, TabMessage, Viewable};
use iced::{
    theme::Theme,
    widget::{button, column, container, row, text},
    Alignment, Element,
};
use iced_aw::TabLabel;

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

impl Viewable for CounterTab {
    type Message = CounterMessage;
    type Data = ();

    fn new(id: usize, _data: ()) -> Self {
        Self { id, value: 0 }
    }

    fn id(&self) -> usize {
        self.id
    }

    fn is_dirty(&self) -> bool {
        false
    }

    fn update(&mut self, message: CounterMessage) {
        match message {
            CounterMessage::Increase => {
                self.value += 1;
            }
            CounterMessage::Decrease => {
                self.value -= 1;
            }
        }
    }

    fn tab_label(&self) -> TabLabel {
        TabLabel::Text(format!("Counter {}", self.id))
    }

    fn view(&self) -> iced::Element<'_, TabBarMessage, Theme> {
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

        content.map(|msg| TabBarMessage::UpdateTab((self.id, TabMessage::Counter(msg))))
    }

    fn refresh(&mut self, _data: Self::Data) {}
}
