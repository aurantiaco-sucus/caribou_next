use std::fmt::{Debug, Formatter, Pointer, Write};
use std::ops::Deref;
use std::sync::{Arc, Weak};
use crate::caribou::AsyncTask;
use crate::caribou::event::Event;
use crate::caribou::focus::FocusTracker;
use crate::caribou::gadget::{Gadget, GadgetInner, GadgetParent};
use crate::caribou::input::KeyEventInfo;
use crate::caribou::value::Value;

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct Window {
    inner: Arc<WindowInner>
}

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct WindowRef {
    inner: Weak<WindowInner>
}

impl Window {
    pub fn refer(&self) -> WindowRef {
        WindowRef {
            inner: Arc::downgrade(&self.inner)
        }
    }
}

impl WindowRef {
    pub fn get(&self) -> Option<Window> {
        self.inner.upgrade().map(|inner| Window { inner })
    }
}

impl Deref for Window {
    type Target = WindowInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug)]
pub struct WindowInner {
    // Values
    pub title: Value<String>,
    pub pos: Value<(i32, i32)>,
    pub dim: Value<(u32, u32)>,
    pub root: Value<Gadget>,
    // Mechanisms
    pub focus_tracker: FocusTracker,
    backend: Backend,
    // Events
    pub key: Event<dyn Fn(KeyEventInfo) -> AsyncTask<()> + Send + Sync>,
}

pub struct Backend {
    pub window_impl: Box<dyn WindowImpl>,
}

impl Backend {
    pub fn new<I: 'static + WindowImpl>(window_impl: I) -> Self {
        Self { window_impl: Box::new(window_impl) }
    }
}

impl Deref for Backend {
    type Target = dyn WindowImpl;

    fn deref(&self) -> &Self::Target {
        self.window_impl.deref()
    }
}

impl Debug for Backend {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.window_impl.fmt(f)
    }
}

pub trait WindowImpl: Send + Sync {
    fn debug_fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result;
    fn launch(&self, window: Window);
    fn request_redraw(&self);
}

impl Window {
    pub async fn new(backend: Backend, root: Gadget) -> Window {
        let window = Window {
            inner: Arc::new(WindowInner {
                title: Value::new("Caribou".to_string()),
                pos: Value::new((0, 0)),
                dim: Value::new((800, 600)),
                root: Value::new(root.clone()),
                focus_tracker: FocusTracker::default(),
                backend,
                key: Event::default(),
            })
        };
        window.focus_tracker.attach_tab_listener(&window).await;
        root.parent.set(GadgetParent::Window(window.refer())).await;
        window
    }

    pub fn launch_on_new_thread(&self) {
        let window = self.clone();
        std::thread::spawn(move || {
            let window = window;
            window.backend.window_impl
                .launch(window.clone())
        });
    }

    pub fn launch_on_current_thread(&self) {
        self.backend.window_impl
            .launch(self.clone())
    }
    
    pub fn request_redraw(&self) {
        self.backend.window_impl.request_redraw()
    }
}