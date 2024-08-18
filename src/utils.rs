use super::Message;
use iced::font;
use std::fmt::{Debug, Display};
use std::io;
use std::path::PathBuf;

use modav_core::repr::csv::utils::Error;

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
        random: f32,
    }

    impl ColorEngine {
        const RATIO: f32 = 0.60;

        pub fn new<'a>(seed: &'a Theme) -> Self {
            let rng: f32 = thread_rng().gen();
            Self {
                seed: seed.extended_palette().secondary.base.color.into(),
                is_dark: seed.extended_palette().is_dark,
                random: rng,
            }
        }

        /// Generates a Color taking into consideration previously generated colors
        fn generate(&mut self) -> Color {
            let seed: f32 = self.seed.h.into();
            let h = (self.random + Self::RATIO + seed) % 1.0;

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

#[allow(dead_code, unused_imports)]
pub mod icons {
    use iced::{
        alignment,
        widget::{text, Text},
        Font,
    };

    fn icon_maker(unicode: char, name: &'static str) -> Text<'static> {
        let fnt: Font = Font::with_name(name);
        text(unicode.to_string())
            .font(fnt)
            .horizontal_alignment(alignment::Horizontal::Center)
    }

    pub mod status {
        use super::icon_maker;
        use iced::widget::Text;

        const NAME: &'static str = "status-icons";

        pub const COUNTER: char = '\u{E800}';
        pub const EDITOR: char = '\u{E801}';
        pub const FILE: char = '\u{F0F6}';

        pub fn icon(unicode: char) -> Text<'static> {
            icon_maker(unicode, NAME)
        }
    }

    pub mod dashboard {
        use super::icon_maker;
        use iced::widget::Text;

        const NAME: &'static str = "dash-icons";
        pub const ANGLE_UP: char = '\u{F106}';
        pub const ANGLE_DOWN: char = '\u{F107}';
        pub const CHART: char = '\u{E802}';
        pub const SETTINGS: char = '\u{E800}';
        pub const FILE: char = '\u{F0F6}';
        pub const INFO: char = '\u{F129}';

        pub fn icon(unicode: char) -> Text<'static> {
            icon_maker(unicode, NAME)
        }
    }

    pub mod wizard {
        use super::icon_maker;
        use iced::widget::Text;

        const NAME: &'static str = "wizard-icons";

        pub const HELP: char = '\u{E800}';
        pub const REDO: char = '\u{E802}';

        pub fn icon(unicode: char) -> Text<'static> {
            icon_maker(unicode, NAME)
        }
    }

    pub mod toast {
        use super::icon_maker;
        use iced::widget::Text;

        pub const NAME: &'static str = "toast-icons";

        pub const SUCCESS: char = '\u{E802}';
        pub const INFO: char = '\u{F086}';
        pub const WARN: char = '\u{E806}';
        pub const ERROR: char = '\u{E807}';
        pub const CLOSE: char = '\u{E801}';

        pub fn icon(unicode: char) -> Text<'static> {
            icon_maker(unicode, NAME)
        }
    }
}

#[derive(Debug, Default)]
pub enum AppError {
    FontLoading(font::Error),
    FileDialogClosed,
    FileLoading(io::ErrorKind),
    FileSaving(io::ErrorKind),
    CSVError(Error),
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
            Self::FileLoading(err) => err.to_string(),
            Self::FileSaving(err) => err.to_string(),
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

impl std::error::Error for AppError {}

pub mod menus {

    use crate::{
        widgets::dashmenu::{DashMenu, DashMenuOption},
        ViewType,
    };

    use super::{icons::dashboard, Message};

    use iced::{Font, Length, Renderer};

    pub fn menu_styler(menu: DashMenu<Message, Renderer>) -> DashMenu<Message, Renderer> {
        menu.spacing(10.0)
            .width(Length::Fixed(150.0))
            .padding([2.0, 8.0])
            .submenu_padding([2.0, 8.0])
            .icon_font(Font::with_name("dash-icons"))
    }

    pub fn models_menu<'a>() -> DashMenu<Message, Renderer> {
        let options = vec![DashMenuOption::new("Line Graph", Some(Message::Convert))];

        let menu = DashMenu::new(dashboard::CHART, "Models").submenus(options);

        menu_styler(menu)
    }

    pub fn views_menu<'a, F>(on_select: F) -> DashMenu<Message, Renderer>
    where
        F: Fn(ViewType) -> Message,
    {
        let options = vec![
            DashMenuOption::new("Add Counter", Some((on_select)(ViewType::Counter))),
            DashMenuOption::new("Open Editor", Some((on_select)(ViewType::Editor))),
        ];

        let menu = DashMenu::new(dashboard::SETTINGS, "Views").submenus(options);

        menu_styler(menu)
    }

    pub fn about_menu<'a>() -> DashMenu<Message, Renderer> {
        let menu = DashMenu::new(dashboard::INFO, "About").on_select(Message::None);

        menu_styler(menu)
    }

    pub fn settings_menu<'a>() -> DashMenu<Message, Renderer> {
        let menu =
            DashMenu::new(dashboard::SETTINGS, "Settings").on_select(Message::ToggleSettingsDialog);

        menu_styler(menu)
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
