pub mod runtime;
pub mod input;

use std::fmt::{Debug, Formatter};
use crate::caribou::window::{Backend, Window, WindowImpl};
use crate::cb_skia_gl::runtime::{skia_gl_get_env, skia_gl_launch};

pub fn skia_gl_create_window() -> Window {
    let backend = Backend::new(SkiaGLWindowImpl);
    let window = Window::new(backend);
    window
}

#[derive(Debug)]
pub struct SkiaGLWindowImpl;

impl WindowImpl for SkiaGLWindowImpl {
    fn debug_fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt(f)
    }

    fn launch(&self, window: Window) {
        skia_gl_launch(window);
    }

    fn request_redraw(&self) {
        skia_request_redraw();
    }
}

pub fn skia_request_redraw() {
    skia_gl_get_env().windowed_context.window().request_redraw();
}