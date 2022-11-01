use std::future::{Future};
use std::pin::Pin;
use std::thread;
use std::time::Duration;
use log::info;
use tokio::runtime::Runtime;

use tokio::time::sleep;

pub mod batch;
pub mod math;
pub mod gadget;
pub mod window;
pub mod layout;
pub mod input;
pub mod focus;
pub mod state;

static mut TOKIO_RUNTIME: Option<Runtime> = None;

pub fn caribou_init() {
    pretty_env_logger::init();
    info!("Caribou starting");
    thread::spawn(|| {
        info!("Tokio runtime starting");
        unsafe { TOKIO_RUNTIME = Some(Runtime::new().unwrap()); }
        info!("Tokio runtime started");
    });
    while unsafe { TOKIO_RUNTIME.is_none() } {
        thread::yield_now();
    }
    info!("Caribou started");
}

pub fn async_runtime() -> &'static Runtime {
    unsafe { TOKIO_RUNTIME.as_ref().unwrap_unchecked() }
}

#[macro_export]
macro_rules! async_task {
    ($expr: expr) => {
        crate::caribou::AsyncTask::wrap(async move { $expr })
    };
}

pub enum ScheduleResult {
    Repeat,
    RepeatAfter(Duration),
    Break,
}

pub fn schedule<F: 'static>(delay: Duration, proc: F)
    where F: Fn() -> Pin<Box<dyn Future<Output=ScheduleResult> + Send + Sync>> + Send + Sync
{
    let proc = Box::new(proc);
    async_runtime().spawn(async move {
        let mut delay = delay;
        let proc = proc;
        loop {
            sleep(delay).await;
            let result = async_runtime().spawn(proc()).await.unwrap();
            match result {
                ScheduleResult::Repeat => {}
                ScheduleResult::RepeatAfter(new_delay) => delay = new_delay,
                ScheduleResult::Break => break,
            }
        }
    });
}