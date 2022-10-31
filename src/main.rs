pub mod caribou;
pub mod cb_backend_skia_gl;
pub mod cb_control_builtin;


use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use log::debug;
use crate::caribou::{async_runtime, AsyncTask, caribou_init, schedule, ScheduleResult};
use crate::caribou::layout::Layout;
use crate::cb_backend_skia_gl::skia_gl_create_window;
use crate::cb_control_builtin::button::{Button, ButtonData, ButtonStyle};

fn main() {
    caribou_init();
    let window = async_runtime().block_on(async {
        let layout = Layout::create().await;
        layout.dim.set((800.0, 600.0).into()).await;

        let button1 = Button::create(ButtonStyle::default()).await;
        button1.enabled.set(false).await;

        let button2 = Button::create(ButtonStyle::default()).await;
        button2.data
            .get_cloned().await
            .get::<ButtonData>().await
            .caption.set("Count: 0".into()).await;
        button2.pos.set((75.0, 25.0).into()).await;

        Layout::add_child(&layout, button1.clone()).await;
        Layout::add_child(&layout, button2.clone()).await;

        let button2_weak = button2.refer();
        let count = Arc::new(AtomicUsize::new(0));
        schedule(Duration::from_millis(1000 / 60), move || {
            let count = count.clone();
            let button2_weak = button2_weak.clone();
            AsyncTask::wrap(async move {
                let num = count.fetch_add(1, Ordering::Relaxed);
                //debug!("Count: {}", num);
                let button2_weak = button2_weak.clone();
                let button2 = button2_weak.get().unwrap();
                button2.data
                    .get_cloned().await
                    .get_mut::<ButtonData>().await
                    .caption.set(format!("Count: {}", num)).await;
                button2.get_window().await.unwrap().get().unwrap().request_redraw();
                ScheduleResult::Repeat
            })
        });

        let window = skia_gl_create_window(layout).await;
        window
    });
    schedule(Duration::from_secs(5), move || {
        AsyncTask::wrap(async {
            debug!("Repeating every 5 secs!");
            ScheduleResult::Repeat
        })
    });
    window.launch_on_current_thread();
}