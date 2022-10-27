use log::debug;
use crate::async_task;
use crate::caribou::batch::{Painting, begin_paint, Batch, BatchFlattening, BatchOp, Transform};
use crate::caribou::{async_runtime, AsyncTask};
use crate::caribou::gadget::{Gadget, GadgetParent};
use crate::caribou::input::MouseEventInfo;
use crate::caribou::math::{Region, ScalarPair};

pub struct Layout;

impl Layout {
    pub async fn create() -> Gadget {
        let gadget = Gadget::default();

        // Handle events

        let gr = gadget.refer();
        gadget.draw.handle(move || {
            let gadget = gr.get().unwrap();
            AsyncTask::wrap(async move {
                let mut artist = begin_paint();
                let children = gadget.children.get().await;
                for child in children.iter() {
                    artist = artist.batch(
                        child.pos.get().await.into_translate(),
                        child.draw.gather().await.flatten());
                }
                artist.finish()
            })
        }).await;

        let gr = gadget.refer();
        gadget.mouse.handle(move |info| {
            let gadget = gr.get().unwrap();
            AsyncTask::wrap(async move {
                let mut data = gadget.data
                    .get_mut::<LayoutData>().await;
                match &info {
                    MouseEventInfo::Enter => {}
                    MouseEventInfo::Leave => {
                        if let Some(child) = &data.hovering {
                            child.mouse.broadcast(info.clone()).await;
                            data.hovering = None;
                        }
                    }
                    MouseEventInfo::Down {
                        button, pos, modifiers
                    } => {
                        if let Some(child) = &data.hovering {
                            let child_pos = *child.pos.get().await;
                            child.mouse.broadcast(MouseEventInfo::Down {
                                button: *button,
                                pos: *pos - child_pos,
                                modifiers: modifiers.clone(),
                            }).await;
                        }
                    }
                    MouseEventInfo::Up {
                        button, pos, modifiers
                    } => {
                        if let Some(child) = &data.hovering {
                            let child_pos = *child.pos.get().await;
                            child.mouse.broadcast(MouseEventInfo::Up {
                                button: *button,
                                pos: *pos - child_pos,
                                modifiers: modifiers.clone(),
                            }).await;
                        }
                    }
                    MouseEventInfo::Move {
                        pos, modifiers
                    } => {
                        // Check if there is a child being hovered
                        let children = gadget.children.get().await;
                        for child in children.iter().rev() {
                            let child_pos = *child.pos.get().await;
                            let child_dim = *child.dim.get().await;
                            let region = Region::from_origin_size(
                                child_pos, child_dim);
                            if region.contains(*pos) {
                                if let Some(hovering) = &data.hovering {
                                    if hovering == child {
                                        // The mouse is still hovering current child
                                        hovering.mouse.broadcast(
                                            MouseEventInfo::Move {
                                                pos: *pos - child_pos,
                                                modifiers: modifiers.clone()
                                            }).await;
                                        return;
                                    } else {
                                        // The mouse is no longer hovering current child
                                        hovering.mouse.broadcast(
                                            MouseEventInfo::Leave).await;
                                    }
                                }
                                // The mouse is now hovering new child
                                child.mouse.broadcast(
                                    MouseEventInfo::Enter).await;
                                data.hovering = Some(child.clone());
                                return;
                            }
                        }
                        // The mouse is not hovering any child
                        if let Some(hovering) = &data.hovering {
                            hovering.mouse.broadcast(MouseEventInfo::Leave).await;
                            data.hovering = None;
                        }
                    }
                }
            })
        }).await;

        // Fill specialized data

        gadget.data.set(LayoutData {
            hovering: None,
        }).await;

        // Fill common properties

        gadget.dim.set(ScalarPair::new(150.0, 150.0)).await;
        gadget.propagate.set(true).await;

        gadget
    }

    pub async fn add_child(parent: &Gadget, child: Gadget) {
        child.parent.set(GadgetParent::Gadget(parent.refer())).await;
        let mut children = parent.children.get_mut().await;
        children.push(child);
    }

    pub async fn remove_child(parent: &Gadget, child: Gadget) {
        child.parent.set(GadgetParent::None).await;
        let mut children = parent.children.get_mut().await;
        children.retain(|c| c != &child);
    }
}

pub struct LayoutData {
    hovering: Option<Gadget>,
}