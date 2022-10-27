pub mod runtime;
pub mod input;
pub mod batch;

use std::any::Any;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use crate::caribou::gadget::Gadget;
use crate::caribou::math::Scalar;
use crate::caribou::native::{Native, Wrapper};
use crate::caribou::window::{Backend, Window, WindowImpl};
use crate::cb_backend_skia_gl::runtime::{ENV_REGISTRY, skia_gl_launch};

pub async fn skia_gl_create_window(root: Gadget) -> Window {
    let env_id = ENV_REGISTRY.read().unwrap().len();
    let backend = Backend::new(SkiaGLWindowImpl { env_id });
    Window::new(backend, root).await
}

#[derive(Debug)]
pub struct SkiaGLWindowImpl {
    env_id: usize,
}

impl WindowImpl for SkiaGLWindowImpl {
    fn debug_fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }

    fn launch(&self, window: Window) {
        skia_gl_launch(window, self.env_id);
    }

    fn request_redraw(&self) {
        skia_request_redraw(self.env_id);
    }
}

pub fn skia_request_redraw(env_id: usize) {
    ENV_REGISTRY.read().unwrap()[env_id].windowed_context.window().request_redraw();
}

type WrappedSkiaFont = Arc<SkiaFont>;

struct SkiaFontWrapper {
    font: Arc<SkiaFont>
}

impl Wrapper for SkiaFontWrapper {
    fn debug_fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.font.fmt(f)
    }

    fn get(&self) -> Box<dyn Any> {
        Box::new(self.font.clone())
    }
}

type SkiaFont = skia_safe::Font;
type SkiaFontStyle = skia_safe::FontStyle;

pub type SkiaFontWeight = skia_safe::font_style::Weight;
pub type SkiaFontWidth = skia_safe::font_style::Width;
pub type SkiaFontSlant = skia_safe::font_style::Slant;

pub fn skia_create_font(
    family_name: String,
    weight: SkiaFontWeight,
    width: SkiaFontWidth,
    slant: SkiaFontSlant,
    font_size: Scalar,
) -> Option<Native> {
    let font_mgr = skia_safe::FontMgr::default();
    let typeface = font_mgr
        .match_family_style(family_name,
                            SkiaFontStyle::new(weight, width, slant))?;
    let font = SkiaFont::from_typeface(typeface, font_size);
    Native::wrap(SkiaFontWrapper { font: Arc::new(font) }).into()
}

const DEFAULT_FAMILY_NAME_WINDOWS: &str = "Segoe UI";
const DEFAULT_FAMILY_NAME_WINDOWS_CJK: &str = "微软雅黑";

pub fn skia_font_default(size: Scalar) -> Option<Native> {
    skia_create_font(DEFAULT_FAMILY_NAME_WINDOWS.to_string(),
                     SkiaFontWeight::NORMAL,
                     SkiaFontWidth::NORMAL,
                     SkiaFontSlant::Upright,
                     size)
}

pub fn skia_font_default_cjk(size: Scalar) -> Option<Native> {
    skia_create_font(DEFAULT_FAMILY_NAME_WINDOWS_CJK.to_string(),
                     SkiaFontWeight::NORMAL,
                     SkiaFontWidth::NORMAL,
                     SkiaFontSlant::Upright,
                     size)
}