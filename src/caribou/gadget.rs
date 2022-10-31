
use std::ops::Deref;
use std::sync::{Arc, Weak};


use crate::caribou::batch::{Batch, Brush};
use crate::caribou::AsyncTask;
use crate::caribou::event::Event;
use crate::caribou::focus::FocusEventInfo;
use crate::caribou::input::{Key, KeyEventInfo, MouseButton, MouseEventInfo};
use crate::caribou::math::ScalarPair;
use crate::caribou::state::{Arbitrary, MutableArbitrary, OptionalState, State, StateMap, StateVec};
use crate::caribou::window::WindowRef;
use crate::cb_backend_skia_gl::skia_font_default_cjk;

#[repr(transparent)]
#[derive(Clone)]
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

#[derive(Default, Clone)]
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

    pub(crate) fn from_weak(inner: Weak<GadgetInner>) -> Self {
        GadgetRef { inner }
    }
}

pub struct GadgetInner {
    // Common
    pub pos: State<ScalarPair>,
    pub dim: State<ScalarPair>,
    pub enabled: State<bool>,
    // Hierarchy
    pub parent: State<GadgetParent>,
    pub children: StateVec<Gadget>,
    // Appearance
    pub brush: State<Brush>,
    pub font: State<Arbitrary>,
    pub batch: State<Batch>,
    // Focusing
    pub propagate: State<bool>,
    pub accept_focus: State<bool>,
    pub lock_focus: State<bool>,
    pub focused: State<bool>,
    // Specialized
    pub data: State<MutableArbitrary>,
    pub values: StateMap<String, Arbitrary>,
    // Interactive
    pub mouse_down: StateVec<MouseButton>,
    pub mouse_pos: OptionalState<ScalarPair>,
    pub key_down: StateVec<Key>,
}

#[derive(Clone)]
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

impl Default for Gadget {
    fn default() -> Self {
        Self {
            inner: Arc::new_cyclic(|weak| {
                let back_ref = GadgetRef::from_weak(weak.clone());
                GadgetInner {
                    // Values
                    // - Common
                    pos: State::new_from(back_ref.clone(), (0.0, 0.0)),
                    dim: State::new_from(back_ref.clone(), (0.0, 0.0)),
                    enabled: State::new(back_ref.clone(), true),
                    // - Hierarchy
                    parent: State::new(back_ref.clone(), GadgetParent::None),
                    children: StateVec::new(back_ref.clone()),
                    // - Appearance
                    brush: State::new(back_ref.clone(), Brush::default()),
                    font: State::new_any(
                        back_ref.clone(),
                        skia_font_default_cjk(12.0).unwrap()),
                    // - Focusing
                    batch: State::new(back_ref.clone(), Batch::default()),
                    propagate: State::new(back_ref.clone(), true),
                    // - Specialized
                    accept_focus: State::new(back_ref.clone(), false),
                    lock_focus: State::new(back_ref.clone(), false),
                    focused: State::new(back_ref.clone(), false),
                    data: State::new(back_ref.clone(), MutableArbitrary::placeholder()),
                    values: StateMap::new(back_ref.clone()),
                    // Events
                    // - Common
                    mouse_down: Default::default(),
                    mouse_pos: OptionalState::new(back_ref.clone(), None),
                    key_down: Default::default(),
                }
            })
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
            .cb_focus.focused.get_cloned().await;
        match focused {
            None => false,
            Some(gadget_ref) => match gadget_ref.get() {
                None => false,
                Some(gadget) => &gadget == self,
            }
        }
    }
}