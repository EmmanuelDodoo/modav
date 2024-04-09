use std::path::PathBuf;

use iced_aw::TabLabel;

use super::{TabBarMessage, Viewable};

use iced::{
    color, theme,
    widget::{column, container, horizontal_space, row, text},
    Alignment, Background, Border, Element, Length, Renderer, Theme,
};

mod graph;
use graph::{Axis, Graph, GraphLine, GraphPoint};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ModelTabData {
    file: PathBuf,
    title: String,
}

impl ModelTabData {
    pub fn new(file: PathBuf) -> Self {
        Self {
            file,
            title: "Untitled".into(),
        }
    }

    pub fn title(mut self, title: String) -> Self {
        self.title = title;
        self
    }
}

#[derive(Clone, Debug)]
pub enum ModelMessage {}

#[derive(Clone, Debug, PartialEq)]
pub struct ModelTab {
    id: usize,
    file: PathBuf,
    title: String,
}

impl ModelTab {
    fn graph() -> Graph<String, String> {
        let x_axis = Axis::new(
            Some("German".into()),
            vec![
                String::from("Eins"),
                String::from("Zwei"),
                String::from("Drei"),
                String::from("Vier"),
                String::from("Funf"),
                String::from("Sechs"),
                String::from("Sieben"),
                String::from("Acht"),
                String::from("Neun"),
            ],
        );

        let y_axis = Axis::new(
            Some("English".into()),
            vec![
                String::from("One"),
                String::from("Two"),
                String::from("Three"),
                String::from("Four"),
                String::from("Five"),
                String::from("Six"),
                String::from("Seven"),
                String::from("Eight"),
            ],
        );

        let l1 = {
            let p = vec![
                GraphPoint::new(String::from("Eins"), String::from("One")),
                GraphPoint::new(String::from("Zwei"), String::from("Two")),
                GraphPoint::new(String::from("Drei"), String::from("Three")),
                GraphPoint::new(String::from("Vier"), String::from("Four")),
                GraphPoint::new(String::from("Funf"), String::from("Five")),
                GraphPoint::new(String::from("Sechs"), String::from("Six")),
            ];

            GraphLine::new(p, None)
        };

        let l2 = {
            let p = vec![
                GraphPoint::new(String::from("Funf"), String::from("One")),
                GraphPoint::new(String::from("Funf"), String::from("Two")),
                GraphPoint::new(String::from("Funf"), String::from("Three")),
                GraphPoint::new(String::from("Funf"), String::from("Four")),
                GraphPoint::new(String::from("Funf"), String::from("Five")),
            ];

            GraphLine::new(p, String::from("Second Line").into()).color(color!(0, 0, 255))
        };

        let l3 = {
            let p = vec![
                GraphPoint::new(String::from("Eins"), String::from("Five")),
                GraphPoint::new(String::from("Zwei"), String::from("Four")),
                GraphPoint::new(String::from("Drei"), String::from("Three")),
                GraphPoint::new(String::from("Vier"), String::from("Two")),
                GraphPoint::new(String::from("Funf"), String::from("One")),
            ];

            GraphLine::new(p, String::from("Other").into()).color(color!(150, 0, 255))
        };

        Graph::new(x_axis, y_axis, vec![l1, l2, l3])
    }
}

impl Viewable for ModelTab {
    type Message = ModelMessage;
    type Data = ModelTabData;

    fn new(id: usize, data: Self::Data) -> Self {
        let ModelTabData { file, title } = data;
        Self { id, file, title }
    }

    fn id(&self) -> usize {
        self.id
    }

    fn is_dirty(&self) -> bool {
        false
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn tab_label(&self) -> TabLabel {
        let file_name = self
            .file
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("New File");

        TabLabel::Text(format!("{} - {}", self.title, file_name))
    }

    fn content(&self) -> Option<String> {
        None
    }

    fn path(&self) -> Option<PathBuf> {
        Some(self.file.clone())
    }

    fn can_save(&self) -> bool {
        false
    }

    fn modal_msg(&self) -> String {
        "Nothing to show in modal".into()
    }

    fn refresh(&mut self, data: Self::Data) {
        let ModelTabData { file, title } = data;

        self.title = title;
        self.file = file;
    }

    fn update(&mut self, message: Self::Message) {
        match message {}
    }

    fn view(&self) -> Element<'_, TabBarMessage, Theme, Renderer> {
        let title = {
            let text = text(format!("{} - Model", self.title));
            row!(horizontal_space(), text, horizontal_space())
                .width(Length::Fill)
                .align_items(Alignment::Center)
        };

        let content_area = container(ModelTab::graph())
            .max_width(1100)
            // .padding([5, 10])
            // .width(Length::FillPortion(20))
            // .height(Length::FillPortion(3))
            .style(theme::Container::Custom(Box::new(ContentAreaContainer)));

        let content = column!(title, content_area)
            .align_items(Alignment::Center)
            .spacing(20)
            .height(Length::Fill)
            .width(Length::Fill);

        container(content)
            .padding([10, 30, 30, 30])
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

pub struct ContentAreaContainer;
impl container::StyleSheet for ContentAreaContainer {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        let border_color = style.extended_palette().primary.weak.color;
        let background_color = style.extended_palette().background.strong.color;

        let border = Border {
            color: border_color,
            width: 1.5,
            ..Default::default()
        };

        let background = Background::Color(background_color);

        container::Appearance {
            border,
            background: Some(background),
            ..Default::default()
        }
    }
}
