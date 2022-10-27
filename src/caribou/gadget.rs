use std::ops::Deref;
use std::sync::{Arc, Weak};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::caribou::batch::{Batch, Brush};
use crate::caribou::AsyncTask;
use crate::caribou::event::Event;
use crate::caribou::focus::FocusEventInfo;
use crate::caribou::input::{KeyEventInfo, MouseEventInfo};
use crate::caribou::math::ScalarPair;
use crate::caribou::native::Native;
use crate::caribou::value::{DynValueMap, DynValue, Value};
use crate::caribou::window::WindowRef;
use crate::cb_backend_skia_gl::{skia_font_default_cjk};

#[repr(transparent)]
#[derive(Debug, Default, Clone)]
pub struct Gadget {
    inner: Arc<GadgetInner>
}

impl PartialEq for Gadget {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Deref for Gadget {
    type Target = GadgetInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug, Default, Clone)]
pub struct GadgetRef {
    inner: Weak<GadgetInner>
}

impl PartialEq for GadgetRef {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.inner, &other.inner)
    }
}

impl Gadget {
    pub fn refer(&self) -> GadgetRef {
        GadgetRef { inner: Arc::downgrade(&self.inner) }
    }
}

impl GadgetRef {
    pub fn get(&self) -> Option<Gadget> {
        Weak::upgrade(&self.inner)
            .map(|inner| Gadget { inner })
    }
}

#[derive(Debug)]
pub struct GadgetInner {
    // Values
    // - Common
    pub pos: Value<ScalarPair>,
    pub dim: Value<ScalarPair>,
    pub enabled: Value<bool>,
    // - Hierarchy
    pub parent: Value<GadgetParent>,
    pub children: Value<Vec<Gadget>>,
    // - Appearance
    pub brush: Value<Brush>,
    pub font: Value<Native>,
    // - Focusing
    pub propagate: Value<bool>,
    // - Specialized
    pub data: DynValue,
    pub values: DynValueMap,
    // Events
    // - Common
    pub draw: Event<dyn Fn() -> AsyncTask<Batch> + Send + Sync>,
    pub action: Event<dyn Fn() -> AsyncTask<()> + Send + Sync>,
    // - Input
    pub mouse: Event<dyn Fn(MouseEventInfo) -> AsyncTask<()> + Send + Sync>,
    pub focus: Event<dyn Fn(FocusEventInfo) -> AsyncTask<bool> + Send + Sync>,
    pub key: Event<dyn Fn(KeyEventInfo) -> AsyncTask<()> + Send + Sync>,
}

#[derive(Debug, Clone)]
pub enum GadgetParent {
    None,
    Gadget(GadgetRef),
    Window(WindowRef),
}

impl Default for GadgetParent {
    fn default() -> Self {
        Self::None
    }
}

impl Default for GadgetInner {
    fn default() -> Self {
        Self {
            pos: Value::default(),
            dim: Value::default(),
            enabled: Value::new(true),
            parent: Value::new(GadgetParent::None),
            children: Value::default(),
            brush: Value::default(),
            font: Value::new(skia_font_default_cjk(12.0).unwrap()),
            data: DynValue::default(),
            values: DynValueMap::default(),
            draw: Event::default(),
            action: Event::default(),
            mouse: Event::default(),
            focus: Event::default(),
            key: Event::default(),
            propagate: Default::default()
        }
    }
}

impl Gadget {
    pub async fn get_window(&self) -> Option<WindowRef> {
        let mut current = self.clone();
        loop {
            let next = match current.parent.get().await.clone() {
                GadgetParent::None => return None,
                GadgetParent::Gadget(gadget) => gadget,
                GadgetParent::Window(window) => return Some(window)
            };
            current = next.get()?;
        }
    }

    pub async fn is_focused(&self) -> bool {
        let window = self
            .get_window().await.unwrap();
        let window = window
            .get().unwrap();
        let focused = window
            .focus_tracker.focused.get().await;
        match &*focused {
            None => false,
            Some(gadget_ref) => match gadget_ref.get() {
                None => false,
                Some(gadget) => &gadget == self,
            }
        }
    }
}