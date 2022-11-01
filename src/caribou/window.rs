use std::fmt::{Debug, Formatter, Pointer};
use std::ops::Deref;
use std::sync::{Arc, Weak};
use log::info;
use crate::caribou::focus::CaribouFocus;
use crate::caribou::gadget::{Gadget, GadgetParent, GadgetRef};
use crate::caribou::input::{Key, MouseButton};
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

        window_root_setup(window.clone(), root.clone()).await;

        let wr = window.refer();
        window.root.listen("root_switch", move |event| {
            let wr = wr.clone();
            Box::pin(async move {
                let window = wr.get().unwrap();
                let old_root = (*event.old_value).clone();
                let new_root = event.state.get_cloned().await;
                window_root_setup_reverse(window.clone(), old_root).await;
                window_root_setup(window.clone(), new_root).await;
            })
        }).await;

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

async fn window_root_setup(window: Window, root: Gadget) {
    root.parent.set(GadgetParent::Window(window.refer())).await;

    let wr = window.refer();
    root.batch.listen(
        "window_update",
        move |_| {
            let window = wr.get().unwrap();
            Box::pin(async move {
                //info!("Requesting redraw!");
                window.request_redraw();
            })
        }).await;

    let gr = root.refer();
    window.mouse_down.listen_add(
        "mouse_down_add_sync",
        move |event| {
            let gadget = gr.get().unwrap();
            Box::pin(async move {
                gadget.mouse_down.push(event.new_value).await;
            })
        }).await;

    let gr = root.refer();
    window.mouse_down.listen_remove(
        "mouse_down_remove_sync",
        move |event| {
            let gadget = gr.get().unwrap();
            Box::pin(async move {
                gadget.mouse_down.remove(&event.old_value).await;
            })
        }).await;

    let gr = root.refer();
    window.mouse_pos.listen_set(
        "mouse_pos_set_sync",
        move |event| {
            let gadget = gr.get().unwrap();
            Box::pin(async move {
                gadget.mouse_pos.put(event.value).await;
            })
        }).await;

    let gr = root.refer();
    window.mouse_pos.listen_unset(
        "mouse_pos_unset_sync",
        move |_| {
            let gadget = gr.get().unwrap();
            Box::pin(async move {
                gadget.mouse_pos.take().await;
            })
        }).await;

    let gr = root.refer();
    window.mouse_pos.listen_change(
        "mouse_pos_change_sync",
        move |event| {
            let gadget = gr.get().unwrap();
            Box::pin(async move {
                gadget.mouse_pos.put(event.new_value).await;
            })
        }).await;
}

async fn window_root_setup_reverse(window: Window, root: Gadget) {
    root.batch.remove_listener("window_update").await;
    window.mouse_down.remove_listener_add("mouse_down_add_sync").await;
    window.mouse_down.remove_listener_remove("mouse_down_remove_sync").await;
    window.mouse_pos.remove_listener_set("mouse_pos_set_sync").await;
    window.mouse_pos.remove_listener_unset("mouse_pos_unset_sync").await;
    window.mouse_pos.remove_listener_change("mouse_pos_change_sync").await;
}