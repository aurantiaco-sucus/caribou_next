use std::future::Future;
use std::thread;
use log::info;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;

pub mod batch;
pub mod math;
pub mod gadget;
pub mod value;
pub mod event;
pub mod window;
pub mod layout;
pub mod native;
pub mod input;
pub mod focus;
pub mod timer;

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

pub struct AsyncTask<T: Send + 'static = ()> {
    data: Box<dyn Future<Output = T> + Send>
}

impl<T: Send + 'static> AsyncTask<T> {
    pub fn wrap(fut: impl Future<Output = T> + Send + 'static) -> Self {
        Self { data: Box::new(fut) }
    }

    pub fn spawn(self) -> JoinHandle<T> {
        async_runtime().spawn(async move {
            let data = self.data;
            Box::into_pin(data).await
        })
    }
}

pub trait IntoAsyncTask<T: Send + 'static> {
    fn into_async_task(self) -> AsyncTask<T>;
}

#[macro_export]
macro_rules! async_task {
    ($expr: expr) => {
        crate::caribou::AsyncTask::wrap(async move { $expr })
    };
}