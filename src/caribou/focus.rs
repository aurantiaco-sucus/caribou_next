use std::sync::Arc;
use std::sync::atomic::Ordering;
use async_recursion::async_recursion;
use tokio::sync::RwLock;
use crate::caribou::AsyncTask;
use crate::caribou::gadget::{Gadget, GadgetParent, GadgetRef};
use crate::caribou::input::Key;
use crate::caribou::state::State;
use crate::caribou::window::{Window, WindowRef};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusEventInfo {
    Gain,
    Lose
}

#[derive(Clone)]
pub struct FocusTracker {
    pub focused: State<Option<GadgetRef>>,
    pub manual_order: Arc<RwLock<Option<Vec<GadgetRef>>>>,
    pub window_ref: Arc<RwLock<Option<WindowRef>>>,
}

impl Default for FocusTracker {
    fn default() -> Self {
        FocusTracker {
            focused: State::new(GadgetRef::default(), None),
            manual_order: Arc::new(RwLock::new(None)),
            window_ref: Arc::new(RwLock::new(None)),
        }
    }
}

impl FocusTracker {
    pub async fn cycle(&self) {
        let mut manual = self.manual_order.write().await;
        let mut focused = self.focused.get_mut().await;
        if let Some(manual) = manual.as_mut() {
            focus_clean_manual(manual).await;
            focus_cycle_manual(manual, &mut focused).await;
        } else {
            let window = self.window_ref.read().await;
            let root = window.clone().unwrap().get().unwrap().root.get_cloned().await;
            focus_cycle_auto(&mut focused, root).await;
        }
    }

    pub async fn attach_tab_listener(&self, window: &Window) {
        *self.window_ref.write().await = Some(window.refer());
        let wr = window.refer();
        window.key.handle(move |info| {
            let wr = wr.clone();
            AsyncTask::wrap(async move {
                let window = wr.get().unwrap();
                if info.key == Key::Tab {
                    if info.is_down {
                        window.focus_tracker.cycle().await;
                    }
                } else {
                    let focused = window.focus_tracker.focused.get_cloned().await;
                    if let Some(focused) = focused {
                        if let Some(focused) = focused.get() {
                            focused.key.broadcast(info).await;
                        }
                    }
                }
            })
        }).await;
    }
}

async fn focus_test_accept(gadget: &Gadget) -> bool {
    gadget.focus
        .gather(FocusEventInfo::Gain).await
        .into_iter().all(|x| x)
}

async fn focus_test_release(gadget: &Gadget) -> bool {
    gadget.focus
        .gather(FocusEventInfo::Lose).await
        .into_iter().all(|x| x)
}

async fn focus_clean_manual(manual: &mut Vec<GadgetRef>) {
    manual.retain(|gadget| gadget.get().is_some());
}

async fn focus_cycle_manual(manual: &mut Vec<GadgetRef>, focused_fld: &mut Option<GadgetRef>) {
    if manual.is_empty() {
        if let Some(focused_ref) = focused_fld {
            if let Some(focused) = focused_ref.get() {
                if !focus_test_release(&focused).await {
                    return;
                }
            }
            *focused_fld = None;
        }
        return;
    }
    let begin = match focused_fld {
        None => 0,
        Some(focused_ref) => match focused_ref.get() {
            None => 0,
            Some(focused) => {
                if !focus_test_release(&focused).await {
                    return;
                }
                match manual.iter().position(|x| x == focused_ref) {
                    None => 0,
                    Some(i) => (i + 1) % manual.len(),
                }
            }
        }
    };
    let mut cur = begin;
    loop {
        if let Some(gadget) = manual[cur].get() {
            if focus_test_accept(&gadget).await {
                *focused_fld = Some(manual[cur].clone());
                return;
            }
        }
        cur = (cur + 1) % manual.len();
        if cur == begin {
            *focused_fld = None;
            return;
        }
    }
}

#[async_recursion]
async fn focus_auto_backtrack(gadget: Gadget) -> Option<Gadget> {
    match gadget.parent.get_cloned().await {
        GadgetParent::None => None,
        GadgetParent::Gadget(gr) => {
            let parent = gr.get()?;
            if parent.propagate.get_cloned().await {
                match focus_auto_distribute(parent.clone(),
                                            Some(gadget.clone())).await
                {
                    None => focus_auto_backtrack(parent).await,
                    Some(target) => Some(target)
                }
            } else {
                focus_auto_backtrack(parent).await
            }
        }
        GadgetParent::Window(_) => None,
    }
}

#[async_recursion]
async fn focus_auto_distribute(gadget: Gadget, from: Option<Gadget>) -> Option<Gadget> {
    let children = gadget.children.get_vec().await;
    let mut from = match from {
        None => 0,
        Some(from) =>
            children.iter().position(|x| x == &from).unwrap() + 1,
    };
    while from < children.len() {
        let child = &children[from];
        if child.propagate.get_cloned().await {
            return focus_auto_distribute(child.clone(), None).await;
        } else if focus_test_accept(&child).await {
            return Some(child.clone());
        }
        from += 1;
    }
    None
}

async fn focus_try_choose(focused_fld: &mut Option<GadgetRef>, chosen: Gadget) {
    if focus_test_accept(&chosen).await {
        *focused_fld = Some(chosen.refer());
    }
}

async fn focus_try_root_distribute(focused_fld: &mut Option<GadgetRef>, root: Gadget) {
    if !root.propagate.get_cloned().await {
        return;
    }
    let chosen = focus_auto_distribute(root, None).await;
    if let Some(chosen) = chosen {
        focus_try_choose(focused_fld, chosen).await;
    }
}

async fn focus_cycle_auto(focused_fld: &mut Option<GadgetRef>, root: Gadget) {
    let focused_ref = match focused_fld {
        None => {
            focus_try_root_distribute(focused_fld, root).await;
            return;
        },
        Some(focused_ref) => focused_ref
    };
    let focused = match focused_ref.get() {
        None => {
            *focused_fld = None;
            focus_try_root_distribute(focused_fld, root).await;
            return;
        },
        Some(focused) => focused
    };
    if !focus_test_release(&focused).await {
        return;
    }
    *focused_fld = None;
    let chosen = focus_auto_backtrack(focused.clone()).await;
    if let Some(chosen) = chosen {
        focus_try_choose(focused_fld, chosen).await;
    } else {
        focus_try_root_distribute(focused_fld, root).await;
    }
}