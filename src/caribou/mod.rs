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
pub mod gadget2;
pub mod window;
pub mod layout;
pub mod input;
pub mod focus;
pub mod state;
pub mod drag;
pub mod text;
pub mod value;
pub mod event;

#[macro_export]
macro_rules! deref_to_super {
    ($derived_ty:ty => $super_ty:ty) => {
        impl std::ops::Deref for $derived_ty {
            type Target = $super_ty;

            fn deref(&self) -> &Self::Target {
                &self.super_struct
            }
        }
    };
}

#[macro_export]
macro_rules! as_clone {
    ($($var:ident),*) => {
        $(let $var = $var.clone(););*
    };
    ($($var:ident => $target:ident),*) => {
        $(let $target = $var.clone(););*
    };
}

#[macro_export]
macro_rules! bit_flags {
    (pub enum $type_name:ident : $num_ty:ty { $($variant:ident = $value:literal),*, }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[repr(transparent)]
        pub struct $type_name($num_ty);
        impl $type_name {
            $(
            const $variant: Self = Self($value);
            paste::paste! {
                pub fn [<has_ $variant>](self) -> bool {
                    self.0 & Self::$variant.0 != 0
                }
            }
            )*
        }
        impl std::ops::BitOr for $type_name {
            type Output = Self;

            fn bitor(self, rhs: Self) -> Self::Output {
                Self(self.0 | rhs.0)
            }
        }
    };
}

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