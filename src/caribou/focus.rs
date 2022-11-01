use std::borrow::Borrow;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use async_recursion::async_recursion;
use log::debug;
use tokio::sync::RwLock;
use crate::caribou::gadget::{Gadget, GadgetParent, GadgetRef};
use crate::caribou::input::Key;
use crate::caribou::state::{OptionalState, State};
use crate::caribou::window::{Window, WindowRef};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusEventInfo {
    Gain,
    Lose
}

#[derive(Clone)]
pub struct CaribouFocus {
    pub focused: OptionalState<GadgetRef>,
    pub manual_order: Arc<RwLock<Option<Vec<GadgetRef>>>>,
    pub window_ref: Arc<RwLock<Option<WindowRef>>>,
}

impl Default for CaribouFocus {
    fn default() -> Self {
        CaribouFocus {
            focused: OptionalState::new_empty(GadgetRef::default()),
            manual_order: Arc::new(RwLock::new(None)),
            window_ref: Arc::new(RwLock::new(None)),
        }
    }
}

impl CaribouFocus {
    pub async fn attach_tab_listener(&self, window: &Window) {
        *self.window_ref.write().await = Some(window.refer());
        let wr = window.refer();
        window.key_down.listen_add(
            "cb_focus",
            move |event| {
                let window = wr.get().unwrap();
                Box::pin(async move {
                    let key = event.new_value;
                    let focus = window.cb_focus.borrow();
                    if key == Key::Tab {
                        focus.cycle().await;
                    } else {
                        let focused = focus.focused.get().await;
                        if let Some(focused) = focused {
                            if let Some(focused) = focused.get() {
                                focused.key_down.push(key).await;
                            }
                        }
                    }
                })
            }
        ).await;

        let wr = window.refer();
        window.key_down.listen_remove(
            "cb_focus",
            move |event| {
                let window = wr.get().unwrap();
                Box::pin(async move {
                    let key = event.old_value;
                    let focused = window.cb_focus
                        .focused.get().await;
                    if key != Key::Tab {
                        if let Some(focused) = focused {
                            if let Some(focused) = focused.get() {
                                focused.key_down.remove(&key).await;
                            }
                        }
                    }
                })
            }
        ).await;
    }

    async fn focus_locked(&self) -> bool {
        match self.focused.get().await {
            None => false,
            Some(gr) => match gr.get() {
                None => false,
                Some(focused) => focused.lock_focus.get_cloned().await
            }
        }
    }

    async fn clear_focus(&self) {
        match self.focused.take().await {
            None => {
                debug!("No focus to clear");
            }
            Some(gr) => match gr.get() {
                None => {
                    debug!("Focused gadget no longer exists");
                }
                Some(gadget) => {
                    gadget.focused.set(false).await;
                    debug!("Cleared focus");
                }
            }
        }
    }

    pub async fn cycle(&self) {
        // The focus subsystem (CBF) is available in 2 modes:
        // * User provides a MANUAL tab order thus the dispatch process relies on it
        // * CBF tries to figure out an AUTOMATIC tab order based on the hierarchy of gadgets

        if let Some(manual_order) = &*self.manual_order.read().await {
            debug!("Manual tab order");
            // Manual order
            let begin = match self.focused.get().await {
                None => 0,
                Some(gr) => match manual_order.iter().position(|x| x == &gr) {
                    None => 0,
                    Some(index) => index + 1,
                }
            };
            let mut cur = begin;
            loop {
                if let Some(gadget) = manual_order[cur].get() {
                    if gadget.accept_focus.get_cloned().await {
                        if self.focus_locked().await {
                            return;
                        }
                        self.clear_focus().await;
                        debug!("Focus set to a gadget in manual order");
                        self.focused.put(gadget.refer()).await;
                        gadget.focused.set(true).await;
                        return;
                    }
                }
                cur = (cur + 1) % manual_order.len();
                if cur == begin {
                    // No gadget is eligible for focus
                    return;
                }
            }
        } else {
            debug!("Automatic tab order");
            // Automatic order
            let focused = match self.focused.get().await {
                None => None,
                Some(gr) => match gr.get() {
                    None => None,
                    Some(gadget) => {
                        if gadget.lock_focus.get_cloned().await {
                            return;
                        }
                        self.clear_focus().await;
                        Some(gadget)
                    }
                }
            };
            match focused {
                None => {
                    debug!("No focus, starting from root");
                    // Propagate from the root gadget
                    let window = self.window_ref.read().await
                        .clone().unwrap().get().unwrap();
                    let root = window.root.get_cloned().await;
                    if root.propagate.get_cloned().await {
                        let next = focus_propagate(&root, 0).await;
                        if let Some(next) = next {
                            //debug!("Focus set to a gadget in automatic order");
                            self.focused.put(next.refer()).await;
                            next.focused.set(true).await;
                        }
                    }
                }
                Some(gadget) => {
                    //debug!("Focus exists, starting from the next gadget");
                    // Seek the nearest container and continue propagating
                    let mut cur: Gadget = gadget.clone();
                    loop {
                        let parent = match cur.parent.get_cloned().await {
                            GadgetParent::None => return,
                            GadgetParent::Gadget(gr) => gr.get().unwrap(),
                            GadgetParent::Window(_) => return,
                        };
                        if parent.propagate.get_cloned().await {
                            let index = parent.children.get_vec().await
                                .iter().position(|x| x == &cur).unwrap();
                            let next = focus_propagate(&parent, index + 1).await;
                            if let Some(next) = next {
                                self.focused.put(next.refer()).await;
                                next.focused.set(true).await;
                                return;
                            }
                        }
                        cur = parent;
                    }
                }
            }
        }
    }
}

async fn focus_propagate(gadget: &Gadget, from: usize) -> Option<Gadget> {
    struct Frame {
        children: Vec<Gadget>,
        index: usize,
    }
    let mut stack = Vec::new();
    stack.push(Frame {
        children: gadget.children.get_vec().await.clone(),
        index: from,
    });
    while let Some(top) = stack.last_mut() {
        if top.index >= top.children.len() {
            stack.pop();
            continue;
        }
        let child = top.children[top.index].clone();
        top.index += 1;
        if child.accept_focus.get_cloned().await {
            return Some(child);
        }
        if child.propagate.get_cloned().await {
            stack.push(Frame {
                children: child.children.get_vec().await.clone(),
                index: 0,
            });
        }
    }
    None
}