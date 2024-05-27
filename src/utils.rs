use super::styles::*;
use super::Message;
use iced::{
    alignment, font,
    widget::{text, Text},
    Font,
};
use std::fmt::{Debug, Display};
use std::io;
use std::path::PathBuf;

use modav_core::repr::csv::utils::CSVError;

#[allow(dead_code)]
pub mod coloring {
    use rand::{thread_rng, Rng};
    use std::fmt::Display;

    use iced::{color, Color, Theme};

    /// Floating point values in the range [0,1]
    #[derive(Clone, Copy, Debug, PartialEq)]
    struct F2(f32);

    impl Display for F2 {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl Default for F2 {
        fn default() -> Self {
            F2(0.0)
        }
    }

    impl From<f32> for F2 {
        fn from(value: f32) -> Self {
            match F2::new(value) {
                Some(res) => res,
                None => F2::default(),
            }
        }
    }

    impl From<F2> for f32 {
        fn from(value: F2) -> Self {
            value.0
        }
    }

    impl F2 {
        /// Returns a new F2 value if value is between the expected range.
        /// Else returns None
        fn new(value: f32) -> Option<Self> {
            if value >= 0.0 && value <= 1.0 {
                return Some(Self(value));
            }
            None
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Default)]
    struct HSV {
        /// The hue expressed as a fraction of 360 degrees
        h: F2,
        /// The saturation expressed as a fraction of 100
        s: F2,
        /// The value expressed as a fraction of 100
        v: F2,
    }

    impl HSV {
        fn new(hue: impl Into<F2>, saturation: impl Into<F2>, value: impl Into<F2>) -> Self {
            Self {
                h: hue.into(),
                s: saturation.into(),
                v: value.into(),
            }
        }
    }

    impl From<Color> for HSV {
        fn from(value: Color) -> Self {
            let r = value.r;
            let g = value.g;
            let b = value.b;

            let cmin = f32::min(r, g.min(b));
            let cmax = f32::max(r, g.max(b));
            let cdiff = cmax - cmin;

            let h = if cmin == cmax {
                0.0
            } else if cmax == r {
                (60.0 * ((g - b) / cdiff) + 360.0) % 360.0
            } else if cmax == g {
                (60.0 * ((b - r) / cdiff) + 120.0) % 360.0
            } else {
                (60.0 * ((r - g) / cdiff) + 240.0) % 360.0
            };

            let s = if cmax == 0.0 {
                0.0
            } else {
                (cdiff / cmax) * 100.0
            };

            let v = cmax * 100.0;

            let h = h / 360.0;
            let s = s / 100.0;
            let v = v / 100.0;

            HSV::new(h, s, v)
        }
    }

    impl From<HSV> for Color {
        fn from(value: HSV) -> Self {
            let h: f32 = value.h.into();
            let s: f32 = value.s.into();
            let v: f32 = value.v.into();

            let h = h * 360.0;

            let c = v * s;
            let hcomp = h / 60.0;
            let x = c * (1.0 - f32::abs((hcomp % 2.0) - 1.0));
            let m = v - c;

            let (r, g, b) = if 0.0 <= h && h < 60.0 {
                (c, x, 0.0)
            } else if 60.0 <= h && h < 120.0 {
                (x, c, 0.0)
            } else if 120.0 <= h && h < 180.0 {
                (0.0, c, x)
            } else if 180.0 <= h && h < 240.0 {
                (0.0, x, c)
            } else if 240.0 <= h && h < 300.0 {
                (x, 0.0, c)
            } else {
                (c, 0.0, x)
            };

            let r = (r + m) * 255.0;
            let g = (g + m) * 255.0;
            let b = (b + m) * 255.0;

            color!(r, g, b)
        }
    }

    #[test]
    fn test_rgb_to_hsv() {
        let rgb = color!(20, 145, 200);
        let hsv: HSV = rgb.into();

        assert_eq!(hsv.h, F2(0.5509259));
        assert_eq!(hsv.s, F2(0.9));
        assert_eq!(hsv.v, F2(0.78431374));

        let rgb = color!(255, 0, 0);
        let hsv: HSV = rgb.into();

        assert_eq!(hsv.h, F2(0.0));
        assert_eq!(hsv.s, F2(1.));
        assert_eq!(hsv.v, F2(1.));

        let rgb = color!(0, 255, 255);
        let hsv: HSV = rgb.into();

        assert_eq!(hsv.h, F2(0.5));
        assert_eq!(hsv.s, F2(1.0));
        assert_eq!(hsv.v, F2(1.0));
    }

    #[test]
    fn test_hsv_to_rgb() {
        let hsv = HSV::new(0.0, 1.0, 1.0);
        let rbg: Color = hsv.into();

        assert_eq!(rbg.r, 1.0);
        assert_eq!(rbg.g, 0.0);
        assert_eq!(rbg.b, 0.0);

        let hsv = HSV::new(2.0 / 3.0, 1.0, 0.5);
        let rbg: Color = hsv.into();

        assert_eq!(rbg.r, 0.0);
        assert_eq!(rbg.g, 0.0);
        assert_eq!(rbg.b, 0.5);

        let hsv = HSV::new(1.0 / 3.0, 0.5, 0.75);
        let rbg: Color = hsv.into();

        assert_eq!(rbg.r, 0.375);
        assert_eq!(rbg.g, 0.75);
        assert_eq!(rbg.b, 0.375);
    }

    #[derive(Clone, Debug)]
    pub struct ColorEngine {
        seed: HSV,
        is_dark: bool,
        temp: f32,
    }

    impl ColorEngine {
        const RATIO: f32 = 0.60;

        pub fn new<'a>(seed: &'a Theme) -> Self {
            let rng: f32 = thread_rng().gen();
            Self {
                seed: seed.extended_palette().secondary.base.color.into(),
                is_dark: seed.extended_palette().is_dark,
                temp: rng,
            }
        }

        /// Generates a Color taking into consideration previously generated colors
        fn generate(&mut self) -> Color {
            let seed: f32 = self.seed.h.into();
            let h = (self.temp + Self::RATIO + seed) % 1.0;

            let generated = if self.is_dark {
                HSV::new(h, 0.8, 0.5)
            } else {
                HSV::new(h, 0.69, 0.85)
            };

            self.seed = generated;

            generated.into()
        }
    }

    impl Iterator for ColorEngine {
        type Item = Color;

        fn next(&mut self) -> Option<Self::Item> {
            Some(self.generate())
        }
    }
}

#[derive(Debug, Default)]
pub enum AppError {
    FontLoading(font::Error),
    FileDialogClosed,
    FileLoading(io::ErrorKind),
    FileSaving(io::ErrorKind),
    CSVError(CSVError),
    Simple(String),
    #[default]
    None,
}

impl Clone for AppError {
    fn clone(&self) -> Self {
        match self {
            Self::FileSaving(err) => Self::FileSaving(err.clone()),
            Self::FileLoading(err) => Self::FileLoading(err.clone()),
            Self::FileDialogClosed => Self::FileDialogClosed,
            Self::FontLoading(err) => Self::FontLoading(err.clone()),
            Self::Simple(s) => Self::Simple(s.clone()),
            Self::CSVError(err) => AppError::Simple(err.to_string()),
            Self::None => Self::None,
        }
    }
}

impl AppError {
    pub fn message(&self) -> String {
        match self {
            Self::FontLoading(_) => String::from("Error while loading a font"),
            Self::FileDialogClosed => String::from("File Dialog closed prematurely"),
            Self::FileLoading(_) => String::from("Error while loading file"),
            Self::FileSaving(_) => String::from("Error while saving file"),
            Self::Simple(s) => s.clone(),
            Self::CSVError(err) => err.to_string(),
            Self::None => String::new(),
        }
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = self.message();
        match self {
            Self::FontLoading(e) => e.fmt(f),
            Self::FileLoading(err) => std::fmt::Display::fmt(err, f),
            Self::FileSaving(err) => std::fmt::Display::fmt(err, f),
            Self::FileDialogClosed => write!(f, "{}", msg),
            Self::CSVError(err) => std::fmt::Display::fmt(err, f),
            Self::Simple(s) => write!(f, "{s}"),
            Self::None => write!(f, "{}", msg),
        }
    }
}

pub fn icon(unicode: char, name: &'static str) -> Text<'static> {
    let fnt: Font = Font::with_name(name);
    text(unicode.to_string())
        .font(fnt)
        .horizontal_alignment(alignment::Horizontal::Center)
}

pub fn status_icon(unicode: char) -> Text<'static> {
    icon(unicode, "status-icons")
}

pub mod menus {

    use iced_aw::{
        menu::{Item, Menu, MenuBar},
        menu_bar, style,
    };

    use crate::{
        styles::{ColoredContainer, CustomMenuBarStyle},
        ViewType,
    };

    use super::{icon, MenuButtonStyle, Message};

    use iced::{
        color,
        theme::{self, Theme},
        widget::{button, container, row, text, Button, Container, Row, Text},
        Element, Length, Renderer,
    };

    fn dash_icon(unicode: char) -> Text<'static> {
        icon(unicode, "dash-icons")
    }

    /// The last item in a Menu Tree
    fn base_tree(label: &str, msg: Message) -> Item<'_, Message, Theme, Renderer> {
        let btn = button(text(label).width(Length::Shrink).height(Length::Shrink))
            .on_press(msg)
            .style(theme::Button::Custom(Box::new(MenuButtonStyle {})))
            .padding([4, 16])
            .width(Length::Shrink)
            .height(Length::Shrink);

        Item::new(btn)
    }

    pub fn create_children(
        labels: Vec<(&str, Message)>,
    ) -> Vec<Item<'_, Message, Theme, Renderer>> {
        labels
            .into_iter()
            .map(|curr| {
                let label = curr.0;
                let msg = curr.1;
                base_tree(label, msg)
            })
            .collect()
    }

    fn create_label<'a>(
        icon: char,
        label: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Row<'a, Message> {
        let icon = dash_icon(icon);
        row!(icon, label.into()).spacing(8).padding([0, 8])
    }

    pub fn create_menu<'a>(
        label: impl Into<Element<'a, Message, Theme, Renderer>>,
        icon: char,
        children: Vec<Item<'a, Message, Theme, Renderer>>,
    ) -> MenuBar<'a, Message, Theme, Renderer> {
        let label = create_label(icon, label.into());
        let item = container(label).width(Length::Fill);
        let menu = Menu::new(children).offset(5.0).width(Length::Shrink);

        menu_bar!((item, menu))
            .check_bounds_width(30.0)
            .width(Length::Fill)
            .style(style::MenuBarStyle::Custom(Box::new(CustomMenuBarStyle)))
    }

    pub fn container_wrap<'a>(
        item: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Container<'a, Message> {
        container(item)
            .padding([8, 0])
            .width(Length::Fixed(125.0))
            .style(theme::Container::Custom(Box::new(ColoredContainer {
                color: color!(255, 0, 255, 0.3),
                radius: 8.0,
            })))
    }

    pub fn models_menu<'a>() -> Container<'a, Message> {
        let actions_labels = vec![("Line Graph", Message::Convert)];

        let children = create_children(actions_labels);

        let bar = create_menu("Models", '\u{E802}', children);

        container_wrap(bar)
    }

    pub fn views_menu<'a, F>(on_select: F) -> Container<'a, Message>
    where
        F: Fn(ViewType) -> Message,
    {
        let action_labels = {
            vec![
                ("Add Counter", (on_select)(ViewType::Counter)),
                ("Open Editor", (on_select)(ViewType::Editor)),
            ]
        };

        let children = create_children(action_labels);

        let bar = create_menu("Views", '\u{E800}', children);

        container_wrap(bar)
    }

    pub fn about_menu<'a>() -> Container<'a, Message> {
        let label = create_label('\u{E801}', text("About"));
        let btn: Button<'_, Message, Theme, Renderer> = button(label)
            .style(theme::Button::Text)
            .padding([0, 0])
            .on_press(Message::None);

        container_wrap(btn)
    }

    pub fn settings_menu<'a>() -> Container<'a, Message> {
        let actions_labels = vec![("Toggle Theme", Message::ToggleTheme)];

        let children = create_children(actions_labels);

        let bar = create_menu("Settings", '\u{E800}', children);

        container_wrap(bar)
    }
}

pub async fn pick_file() -> Result<PathBuf, AppError> {
    let handle = rfd::AsyncFileDialog::new()
        .pick_file()
        .await
        .ok_or(AppError::FileDialogClosed)?;

    Ok(handle.path().into())
}

pub async fn load_file(path: PathBuf) -> (Result<String, AppError>, PathBuf) {
    let res = tokio::fs::read_to_string(path.clone())
        .await
        .map_err(|err| AppError::FileLoading(err.kind()));

    (res, path)
}

pub async fn save_file(
    path: Option<PathBuf>,
    content: String,
) -> Result<(PathBuf, String), AppError> {
    let path = if let Some(path) = path {
        path
    } else {
        rfd::AsyncFileDialog::new()
            .set_title("Choose File name")
            .save_file()
            .await
            .ok_or(AppError::FileDialogClosed)
            .map(|handle| handle.path().to_owned())?
    };

    tokio::fs::write(&path, content.clone())
        .await
        .map_err(|err| AppError::FileSaving(err.kind()))?;

    Ok((path, content))
}
