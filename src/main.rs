pub mod caribou;
pub mod cb_backend_skia_gl;
pub mod cb_control_builtin;


use crate::caribou::{async_runtime, caribou_init};
use crate::caribou::layout::Layout;
use crate::cb_backend_skia_gl::skia_gl_create_window;
use crate::cb_control_builtin::button::{Button, ButtonStyle};

fn main() {
    caribou_init();
    let window = async_runtime().block_on(async {
        let layout = Layout::create().await;
        layout.dim.set((800.0, 600.0).into()).await;
        let button1 = Button::create(ButtonStyle::default()).await;
        button1.enabled.set(false).await;
        let button2 = Button::create(ButtonStyle::default()).await;
        button2.pos.set((75.0, 25.0).into()).await;
        Layout::add_child(&layout, button1.clone()).await;
        Layout::add_child(&layout, button2.clone()).await;
        let window = skia_gl_create_window(layout).await;
        window
    });
    window.launch_on_current_thread();
}