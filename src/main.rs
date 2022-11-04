pub mod caribou;
pub mod cb_backend_skia_gl;
pub mod cb_control_builtin;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use log::debug;
use crate::caribou::{async_runtime, caribou_init, schedule, ScheduleResult};
use crate::caribou::layout::Layout;
use crate::cb_backend_skia_gl::skia_gl_create_window;
use crate::cb_control_builtin::button::{Button, ButtonData, ButtonStyle};
use crate::cb_control_builtin::textbox::{Textbox, TextboxStyle};

fn main() {
    caribou_init();
    let window = async_runtime().block_on(async {
        let layout = Layout::create().await;
        layout.dim.set((800.0, 600.0).into()).await;

        let button1 = Button::create(ButtonStyle::default()).await;
        //button1.enabled.set(false).await;

        let button2 = Button::create(ButtonStyle::default()).await;
        button2.data
            .get_cloned().await
            .get::<ButtonData>().await
            .caption.set("Count: 0".into()).await;
        button2.pos.set((75.0, 25.0).into()).await;

        let textbox = Textbox::create(TextboxStyle::default()).await;

        Layout::add_child(&layout, button1.clone()).await;
        Layout::add_child(&layout, button2.clone()).await;

        let window = skia_gl_create_window(layout).await;
        window
    });
    schedule(Duration::from_secs(5), move || Box::pin(async {
        debug!("Repeating every 5 secs!");
        ScheduleResult::Repeat
    }));
    window.launch_on_current_thread();
}