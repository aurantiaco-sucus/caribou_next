use log::info;
use crate::caribou::batch::{begin_paint};
use crate::caribou::gadget::{Gadget, GadgetParent, GadgetRef};
use crate::caribou::math::{Region};
use crate::caribou::state::{Listener, State};

pub struct Layout;

impl Layout {
    pub async fn create() -> Gadget {
        let gadget = Gadget::default();

        gadget.children.listen_add(
            "layout_children_add",
            |event| { Box::pin(async move {
                info!("Child added.");
                let new_child = event.new_value;
                new_child.batch.listen("layout_child_batch",
                                       layout_child_listen(event.gadget.clone()))
                    .await;
                new_child.pos.listen("layout_child_pos",
                                     layout_child_listen(event.gadget.clone()))
                    .await;
                layout_update_batch(event.gadget.get().unwrap()).await;
            }) }).await;

        gadget.children.listen_remove(
            "layout_children_remove",
            |event| { Box::pin(async move {
                let old_child = event.old_value;
                old_child.batch.remove_listener("layout_child_batch").await;
                old_child.pos.remove_listener("layout_child_pos").await;
                layout_update_batch(event.gadget.get().unwrap()).await;
            }) }).await;

        gadget.mouse_pos.listen_set(
            "layout_mouse_pos_set",
            |event| { Box::pin(async move {
                let gadget = event.gadget.get().unwrap();
                let pos = event.value;
                let children = gadget.children.get_vec().await;
                for child in children.iter() {
                    let child_pos = child.pos.get_cloned().await;
                    let child_dim = child.dim.get_cloned().await;
                    let region = Region::from_origin_size(child_pos, child_dim);
                    if region.contains(pos) {
                        child.mouse_pos.put(pos - child_pos).await;
                    }
                }
            }) }).await;

        gadget.mouse_pos.listen_change(
            "layout_mouse_pos_change",
            |event| { Box::pin(async move {
                let gadget = event.gadget.get().unwrap();
                let pos = event.new_value;
                let children = gadget.children.get_vec().await;
                for child in children.iter() {
                    let child_pos = child.pos.get_cloned().await;
                    let child_dim = child.dim.get_cloned().await;
                    let region = Region::from_origin_size(child_pos, child_dim);
                    if region.contains(pos) {
                        child.mouse_pos.put(pos - child_pos).await;
                    } else {
                        child.mouse_down.clear().await;
                        child.mouse_pos.take().await;
                    }
                }
            }) }).await;

        gadget.mouse_pos.listen_unset(
            "layout_mouse_pos_unset",
            |event| { Box::pin(async move {
                let gadget = event.gadget.get().unwrap();
                let children = gadget.children.get_vec().await;
                for child in children.iter() {
                    child.mouse_down.clear().await;
                    child.mouse_pos.take().await;
                }
            }) }).await;

        gadget.mouse_down.listen_add(
            "layout_mouse_down_add",
            |event| { Box::pin(async move {
                let gadget = event.gadget.get().unwrap();
                let children = gadget.children.get_vec().await;
                for child in children.iter() {
                    if child.mouse_pos.is_set().await {
                        child.mouse_down.push(event.new_value).await;
                    }
                }
            }) }).await;

        gadget.mouse_down.listen_remove(
            "layout_mouse_down_remove",
            |event| { Box::pin(async move {
                let gadget = event.gadget.get().unwrap();
                let children = gadget.children.get_vec().await;
                for child in children.iter() {
                    if child.mouse_pos.is_set().await {
                        child.mouse_down.remove(&event.old_value).await;
                    }
                }
            }) }).await;

        // Fill specialized data
        let data = LayoutData {
            hovering: State::new(gadget.refer(), None),
        };
        gadget.data.set_any(data).await;

        // Fill common properties
        gadget.dim.set_from((150.0, 150.0)).await;
        gadget.propagate.set(true).await;

        gadget
    }

    pub async fn add_child(parent: &Gadget, child: Gadget) {
        child.parent.set(GadgetParent::Gadget(parent.refer())).await;
        parent.children.push(child).await;
    }

    pub async fn remove_child(parent: &Gadget, child: Gadget) {
        child.parent.set(GadgetParent::None).await;
        parent.children.get_vec_mut().await.retain(|c| c != &child);
    }
}

pub struct LayoutData {
    hovering: State<Option<Gadget>>,
}

async fn layout_update_batch(layout: Gadget) {
    let children = layout.children.get_vec().await;
    let mut artist = begin_paint();
    for child in children.iter() {
        artist = artist.batch(
            child.pos.get().await.into_translate(),
            child.batch.get_cloned().await
        );
    }
    let batch = artist.finish();
    // info!("Layout batch: {:?}", batch);
    layout.batch.set(batch).await;
}

fn layout_child_listen<E: Send + Sync>(layout: GadgetRef) -> Listener<E> {
    Box::new(move |event| {
        let layout = layout.clone();
        Box::pin(async move {
            layout_update_batch(layout.get().unwrap()).await;
        })
    })
}