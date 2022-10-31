use std::fmt::{Debug, Formatter, Pointer};
use std::ops::Deref;
use std::sync::{Arc, Weak};
use crate::caribou::focus::CaribouFocus;
use crate::caribou::gadget::{Gadget, GadgetParent, GadgetRef};
use crate::caribou::input::{Key, KeyEventInfo, MouseButton};
use crate::caribou::math::{IntPair, ScalarPair};
use crate::caribou::state::{OptionalState, State, StateVec};

#[repr(transparent)]
#[derive(Clone)]
pub struct Window {
    inner: Arc<WindowInner>
}

#[repr(transparent)]
#[derive(Clone)]
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

pub struct WindowInner {
    // Values
    pub title: State<String>,
    pub pos: State<IntPair>,
    pub dim: State<IntPair>,
    pub root: State<Gadget>,
    pub mouse_down: StateVec<MouseButton>,
    pub mouse_pos: OptionalState<ScalarPair>,
    pub key_down: StateVec<Key>,
    // Mechanisms
    pub cb_focus: CaribouFocus,
    backend: Backend,
    // Events
    //pub key: Event<dyn Fn(KeyEventInfo) -> AsyncTask<()> + Send + Sync>,
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
        let dummy = GadgetRef::from_weak(Weak::new());
        let window = Window {
            inner: Arc::new(WindowInner {
                title: State::new_from(dummy.clone(), "Caribou"),
                pos: State::new_from(dummy.clone(), (0, 0)),
                dim: State::new_from(dummy.clone(), (800, 600)),
                root: State::new_from(dummy.clone(), root.clone()),
                mouse_down: StateVec::new(dummy.clone()),
                mouse_pos: OptionalState::new_empty(dummy.clone()),
                key_down: Default::default(),
                cb_focus: CaribouFocus::default(),
                backend,
            })
        };
        window.cb_focus.attach_tab_listener(&window).await;
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