use super::Message;
use iced::font;
use std::fmt::{Debug, Display};
use std::io;
use std::path::PathBuf;

use modav_core::repr::sheet::error::Error;

pub use tooltip::tooltip;

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
            F2::new(value)
        }
    }

    impl From<F2> for f32 {
        fn from(value: F2) -> Self {
            value.0
        }
    }

    impl std::ops::Mul<f32> for F2 {
        type Output = Self;

        fn mul(self, rhs: f32) -> Self::Output {
            let temp: f32 = self.into();
            (temp * rhs).into()
        }
    }

    impl std::ops::Add<f32> for F2 {
        type Output = Self;

        fn add(self, rhs: f32) -> Self::Output {
            let temp: f32 = self.into();

            (temp + rhs).into()
        }
    }

    impl std::ops::Sub<f32> for F2 {
        type Output = Self;

        fn sub(self, rhs: f32) -> Self::Output {
            let temp: f32 = self.into();

            (temp - rhs).into()
        }
    }

    impl std::ops::Sub<F2> for f32 {
        type Output = f32;

        fn sub(self, rhs: F2) -> Self::Output {
            let temp: f32 = rhs.into();

            self - temp
        }
    }

    impl F2 {
        /// Hard clamps to 0.0 or 1.0 if [`value`] is below or above range.
        fn new(value: f32) -> Self {
            if value <= 0.0 {
                Self(value)
            } else if value >= 1.0 {
                Self(value)
            } else {
                Self(value)
            }
        }

        fn min(self, other: f32) -> Self {
            f32::min(self.0, other).into()
        }

        fn max(self, other: f32) -> Self {
            f32::max(self.0, other).into()
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

    #[derive(Debug, Default, Clone, Copy)]
    enum ColoringMode {
        /// Generated colors can vary wildly from each other
        #[default]
        Normal,
        /// Generated colors grow gradually darker or lighter depending on the
        /// theme
        Gradual,
    }

    #[derive(Clone, Copy, Debug)]
    pub struct ColorEngine {
        seed: HSV,
        is_dark: bool,
        random: f32,
        mode: ColoringMode,
        stable_h: f32,
        count: u32,
    }

    impl ColorEngine {
        const RATIO: f32 = 0.60;
        const DEFAULT_COUNT: u32 = 5;

        pub fn new<'a>(seed: &'a Theme) -> Self {
            let rng: f32 = thread_rng().gen();
            let is_dark = seed.extended_palette().is_dark;
            let seed: HSV = seed.extended_palette().secondary.base.color.into();

            let stable_h = {
                let seed: f32 = seed.h.into();
                (rng + Self::RATIO + seed) % 1.0
            };

            Self {
                seed,
                is_dark,
                stable_h,
                count: Self::DEFAULT_COUNT,
                random: rng,
                mode: ColoringMode::Normal,
            }
        }

        pub fn count(mut self, count: u32) -> Self {
            if count > 0 {
                self.count = count;
            }

            self
        }

        pub fn gradual(mut self, gradual: bool) -> Self {
            if gradual {
                if self.is_dark {
                    self.seed.v = (0.15).into()
                } else {
                    self.seed.v = (0.85).into()
                }
                self.mode = ColoringMode::Gradual;
            } else {
                self.mode = ColoringMode::Normal;
            }

            self
        }

        /// Generates a Color taking into consideration previously generated colors
        fn generate(&mut self) -> Color {
            let seed: f32 = self.seed.h.into();
            let h = (self.random + Self::RATIO + seed) % 1.0;

            match self.mode {
                ColoringMode::Normal => {
                    let generated = if self.is_dark {
                        HSV::new(h, 0.8, 0.5)
                    } else {
                        HSV::new(h, 0.69, 0.85)
                    };

                    self.seed = generated;

                    generated.into()
                }
                ColoringMode::Gradual => {
                    let diff = 0.85 / (self.count as f32);
                    let generated = if self.is_dark {
                        let v = { self.seed.v + diff }.min(0.925);
                        HSV::new(self.stable_h, 0.8, v)
                    } else {
                        let v = { self.seed.v - diff }.max(0.125);
                        HSV::new(self.stable_h, 0.8, v)
                    };

                    self.seed = generated;

                    generated.into()
                }
            }
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

    pub const NAME: &'static str = "util-icons";
    pub const COUNTER: char = '\u{E805}';
    pub const EDITOR: char = '\u{E804}';
    pub const FILE: char = '\u{F0F6}';
    pub const NEW_FILE: char = '\u{E801}';
    pub const ANGLE_UP: char = '\u{F106}';
    pub const ANGLE_DOWN: char = '\u{F107}';
    pub const CHART: char = '\u{E802}';
    pub const BARCHART: char = '\u{E80E}';
    pub const SETTINGS: char = '\u{E800}';
    pub const INFO: char = '\u{E80A}';
    pub const HELP: char = '\u{E807}';
    pub const REDO: char = '\u{E80B}';
    pub const SUCCESS: char = '\u{E803}';
    pub const WARN: char = '\u{E808}';
    pub const ERROR: char = '\u{E809}';
    pub const CLOSE: char = '\u{E806}';
    pub const TOOLS: char = '\u{E80D}';

    fn icon_maker(unicode: char, name: &'static str) -> Text<'static> {
        let fnt: Font = Font::with_name(name);
        text(unicode.to_string())
            .font(fnt)
            .horizontal_alignment(alignment::Horizontal::Center)
    }

    pub fn icon(unicode: char) -> Text<'static> {
        icon_maker(unicode, NAME)
    }
}

mod tooltip {
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

impl From<Error> for AppError {
    fn from(value: Error) -> Self {
        Self::CSVError(value)
    }
}

pub mod menus {

    use crate::widgets::dashmenu::{DashMenu, DashMenuOption};

    use super::{icons, Message};

    use iced::{Font, Length, Renderer};

    pub fn menu_styler(menu: DashMenu<Message, Renderer>) -> DashMenu<Message, Renderer> {
        menu.spacing(10.0)
            .width(Length::Fixed(150.0))
            .padding([2.0, 8.0])
            .submenu_padding([2.0, 8.0])
            .icon_font(Font::with_name(icons::NAME))
    }

    pub fn models_menu<'a>() -> DashMenu<Message, Renderer> {
        let options = vec![DashMenuOption::new("Line Graph", Some(Message::Convert))];

        let menu = DashMenu::new(icons::CHART, "Models").submenus(options);

        menu_styler(menu)
    }

    pub fn about_menu<'a>() -> DashMenu<Message, Renderer> {
        let menu = DashMenu::new(icons::INFO, "About").on_select(Message::OpenAboutDialog);

        menu_styler(menu)
    }

    pub fn settings_menu<'a>() -> DashMenu<Message, Renderer> {
        let menu =
            DashMenu::new(icons::SETTINGS, "Settings").on_select(Message::OpenSettingsDialog);

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
