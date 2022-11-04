use std::any::Any;
use std::hash::Hash;
use std::ops::Deref;
use std::sync::Arc;
use crate::caribou::math::{Region, Scalar, ScalarPair};
use crate::caribou::state::{Arbitrary, MutableArbitrary};

#[derive(Clone, PartialEq)]
pub struct FontInfo {
    pub family: String,
    pub size: FontSize,
    pub weight: FontWeight,
    pub width: FontWidth,
    pub slant: FontSlant,
}

impl FontInfo {
    pub fn new(family: String, size: FontSize, weight: FontWeight, width: FontWidth, slant: FontSlant) -> FontInfo {
        FontInfo {
            family,
            size,
            weight,
            width,
            slant,
        }
    }

    pub fn with_family(self, family: String) -> FontInfo {
        FontInfo {
            family,
            ..self
        }
    }

    pub fn with_size(self, size: FontSize) -> FontInfo {
        FontInfo {
            size,
            ..self
        }
    }

    pub fn with_weight(self, weight: FontWeight) -> FontInfo {
        FontInfo {
            weight,
            ..self
        }
    }

    pub fn with_width(self, width: FontWidth) -> FontInfo {
        FontInfo {
            width,
            ..self
        }
    }

    pub fn with_slant(self, slant: FontSlant) -> FontInfo {
        FontInfo {
            slant,
            ..self
        }
    }

    pub fn resolve(self) -> Option<Font> {
        todo!()
    }
}

impl Default for FontInfo {
    fn default() -> Self {
        FontInfo::new(FontFamily::ui(),
                      FontSize::default(),
                      FontWeight::default(),
                      FontWidth::default(),
                      FontSlant::Upright)
    }
}

pub struct FontFamily;

impl FontFamily {
    pub fn ui_windows() -> String {
        "Segoe UI".to_string()
    }

    pub fn ui_cjk_windows() -> String {
        "Microsoft YaHei".to_string()
    }

    pub fn ui_macos() -> String {
        "Helvetica".to_string()
    }

    pub fn ui_cjk_macos() -> String {
        "PingFang SC".to_string()
    }

    pub fn ui_linux() -> String {
        "Noto Sans".to_string()
    }

    pub fn ui_cjk_linux() -> String {
        "Noto Sans CJK SC".to_string()
    }

    pub fn ui() -> String {
        if cfg!(target_os = "windows") {
            FontFamily::ui_windows()
        } else if cfg!(target_os = "macos") {
            FontFamily::ui_macos()
        } else {
            FontFamily::ui_linux()
        }
    }

    pub fn ui_cjk() -> String {
        if cfg!(target_os = "windows") {
            FontFamily::ui_cjk_windows()
        } else if cfg!(target_os = "macos") {
            FontFamily::ui_cjk_macos()
        } else {
            FontFamily::ui_cjk_linux()
        }
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub struct FontSize {
    size: Scalar,
}

impl FontSize {
    const NORMAL: FontSize = FontSize::from_scaled_pixels(16.0);
    const SMALL: FontSize = FontSize::from_scaled_pixels(12.0);
    const LARGE: FontSize = FontSize::from_scaled_pixels(24.0);

    pub const fn from_scaled_pixels(pixels: Scalar) -> Self {
        Self { size: pixels }
    }

    pub const fn into_scaled_pixels(self) -> Scalar {
        self.size
    }
}

impl Default for FontSize {
    fn default() -> Self {
        FontSize::NORMAL
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub struct FontWeight {
    weight: Scalar,
}

impl FontWeight {
    const THIN: FontWeight = FontWeight::from_scalar(100.0);
    const EXTRA_LIGHT: FontWeight = FontWeight::from_scalar(200.0);
    const LIGHT: FontWeight = FontWeight::from_scalar(300.0);
    const NORMAL: FontWeight = FontWeight::from_scalar(400.0);
    const MEDIUM: FontWeight = FontWeight::from_scalar(500.0);
    const SEMI_BOLD: FontWeight = FontWeight::from_scalar(600.0);
    const BOLD: FontWeight = FontWeight::from_scalar(700.0);
    const EXTRA_BOLD: FontWeight = FontWeight::from_scalar(800.0);
    const BLACK: FontWeight = FontWeight::from_scalar(900.0);

    pub const fn from_scalar(weight: Scalar) -> Self {
        Self { weight }
    }

    pub const fn into_scalar(self) -> Scalar {
        self.weight
    }

    pub fn thicker(self) -> Self {
        Self::from_scalar(self.weight + 50.0)
    }

    pub fn thinner(self) -> Self {
        Self::from_scalar(self.weight - 50.0)
    }
}

impl Default for FontWeight {
    fn default() -> Self {
        FontWeight::NORMAL
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub struct FontWidth {
    width: Scalar,
}

impl FontWidth {
    const ULTRA_CONDENSED: FontWidth = FontWidth::from_scalar(50.0);
    const EXTRA_CONDENSED: FontWidth = FontWidth::from_scalar(62.5);
    const CONDENSED: FontWidth = FontWidth::from_scalar(75.0);
    const SEMI_CONDENSED: FontWidth = FontWidth::from_scalar(87.5);
    const NORMAL: FontWidth = FontWidth::from_scalar(100.0);
    const SEMI_EXPANDED: FontWidth = FontWidth::from_scalar(112.5);
    const EXPANDED: FontWidth = FontWidth::from_scalar(125.0);
    const EXTRA_EXPANDED: FontWidth = FontWidth::from_scalar(150.0);
    const ULTRA_EXPANDED: FontWidth = FontWidth::from_scalar(200.0);

    pub const fn from_scalar(width: Scalar) -> Self {
        Self { width }
    }

    pub const fn into_scalar(self) -> Scalar {
        self.width
    }
}

impl Default for FontWidth {
    fn default() -> Self {
        FontWidth::NORMAL
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum FontSlant {
    Upright,
    Italic,
    Oblique,
}

#[derive(Clone)]
pub struct Font {
    info: FontInfo,
    native: Arc<dyn NativeFont>
}

impl PartialEq for Font {
    fn eq(&self, other: &Self) -> bool {
        self.info == other.info
    }
}

impl Deref for Font {
    type Target = Arc<dyn NativeFont>;

    fn deref(&self) -> &Self::Target {
        &self.native
    }
}

impl Font {
    pub fn info(&self) -> &FontInfo {
        &self.info
    }
}

pub trait NativeFont: Send + Sync {
    fn measure(&self, text: &str) -> TextMeasurement;

}

#[repr(transparent)]
#[derive(Clone, PartialEq)]
pub struct TextMeasurement {
    bounds: Vec<Region>
}

impl TextMeasurement {
    pub fn new(bounds: Vec<Region>) -> Self {
        Self { bounds }
    }

    pub fn bounds(&self) -> &[Region] {
        &self.bounds
    }

    pub fn width(&self) -> Scalar {
        self.bounds.iter()
            .map(|b| b.width())
            .sum()
    }

    pub fn height(&self) -> Option<Scalar> {
        self.bounds.iter()
            .map(|b| b.height())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
    }

    pub fn size(&self) -> Option<ScalarPair> {
        let width = self.width();
        let height = self.height()?;
        Some((width, height).into())
    }
}