use log::info;
use crate::caribou::batch::{begin_draw, begin_paint, Brush, Colors, Material, Painting, SolidColor, TextAlign, Transform};
use crate::caribou::gadget::Gadget;
use crate::caribou::input::{Key, MouseButton};
use crate::caribou::math::ScalarPair;
use crate::caribou::state::{Arbitrary, State};

pub struct Button;

impl Button {
    pub async fn create(style: ButtonStyle) -> Gadget {
        let gadget = Gadget::default();

        // Fill specialized data
        let data = ButtonData {
            style: State::new(gadget.refer(), style),
            caption: State::new(gadget.refer(), String::from("Button")),
            state: State::new(gadget.refer(), ButtonState::Normal),
        };
        gadget.data.set_any(data).await;

        // Fill common properties
        gadget.dim.set(ScalarPair::new(100.0, 30.0)).await;
        gadget.accept_focus.set(true).await;
        gadget.lock_focus.set(false).await;
        gadget.accept_text.set(true).await;

        // Initial update
        button_batch_update(gadget.clone()).await;

        // Listen batch update
        gadget.dim.listen(
            "button_batch_update",
            |event| Box::pin(async move {
                button_batch_update(event.gadget.get().unwrap()).await;
            })).await;

        gadget.enabled.listen(
            "button_batch_update",
            |event| Box::pin(async move {
                button_batch_update(event.gadget.get().unwrap()).await;
            })).await;

        gadget.focused.listen(
            "button_batch_update",
            |event| Box::pin(async move {
                button_batch_update(event.gadget.get().unwrap()).await;
            })).await;

        let data = gadget.data.get_cloned().await;
        let data = data.get::<ButtonData>().await;

        data.state.listen(
            "button_batch_update",
            |event| Box::pin(async move {
                button_batch_update(event.gadget.get().unwrap()).await;
            })).await;

        data.caption.listen(
            "button_batch_update",
            |event| Box::pin(async move {
                button_batch_update(event.gadget.get().unwrap()).await;
            })).await;

        drop(data);

        // Listen state update
        gadget.mouse_pos.listen_set(
            "button_state_update",
            |event| Box::pin(async move {
                event.gadget.get().unwrap().data.get_cloned().await
                    .get::<ButtonData>().await
                    .state.set(ButtonState::Hover).await;
            })).await;

        gadget.mouse_pos.listen_unset(
            "button_state_update",
            |event| Box::pin(async move {
                event.gadget.get().unwrap().data.get_cloned().await
                    .get::<ButtonData>().await
                    .state.set(ButtonState::Normal).await;
            })).await;

        gadget.mouse_down.listen_add(
            "button_state_update",
            |event| Box::pin(async move {
                if event.new_value != MouseButton::Primary {
                    return;
                }
                event.gadget.get().unwrap().data.get_cloned().await
                    .get::<ButtonData>().await
                    .state.set(ButtonState::Pressed).await;
            })).await;

        gadget.mouse_down.listen_remove(
            "button_state_update",
            |event| Box::pin(async move {
                if event.old_value != MouseButton::Primary {
                    return;
                }
                event.gadget.get().unwrap().data.get_cloned().await
                    .get::<ButtonData>().await
                    .state.set(ButtonState::Hover).await;
            })).await;

        gadget.key_down.listen_add(
            "button_state_update",
            |event| Box::pin(async move {
                if event.new_value != Key::Return {
                    return;
                }
                event.gadget.get().unwrap().data.get_cloned().await
                    .get::<ButtonData>().await
                    .state.set(ButtonState::Pressed).await;
            })).await;

        gadget.key_down.listen_remove(
            "button_state_update",
            |event| Box::pin(async move {
                let gadget = event.gadget.get().unwrap();
                let data = gadget.data.get_cloned().await;
                let data = data.get::<ButtonData>().await;
                if event.old_value != Key::Return {
                    return;
                }
                if gadget.mouse_pos.get().await.is_some() {
                    if gadget.mouse_down.get_vec().await.contains(&MouseButton::Primary) {
                        return;
                    } else {
                        data.state.set(ButtonState::Hover).await;
                    }
                } else {
                    data.state.set(ButtonState::Normal).await;
                }
            })).await;

        // Listen focus management
        gadget.enabled.listen(
            "button_focus_update",
            |event| Box::pin(async move {
                let gadget = event.gadget.get().unwrap();
                let enabled = gadget.enabled.get_cloned().await;
                gadget.accept_focus.set(enabled).await;
            })).await;

        gadget
    }
}

pub struct ButtonData {
    pub style: State<ButtonStyle>,
    pub caption: State<String>,
    state: State<ButtonState>,
}

async fn button_batch_update(button: Gadget) {
    let gadget = button;
    // Gadget properties
    let enabled = gadget.enabled.get_cloned().await;
    let focused = gadget.is_focused().await;
    let dim = gadget.dim.get_cloned().await;
    let font = gadget.font.get().await;
    let data = gadget
        .data.get().await;
    let data = data
        .get::<ButtonData>().await;
    // Data properties
    let style = data.style.get().await;
    let state = data.state.get_cloned().await;
    let caption = data.caption.get_cloned().await;
    let batch = begin_paint()
        .with(|p| style.style_impl
            .draw_backdrop(p,
                           dim,
                           enabled,
                           state))
        .with(|p| style.style_impl
            .draw_caption(p,
                          dim,
                          enabled,
                          state,
                          caption.clone(),
                          font.clone()))
        .with(|p| style.style_impl
            .draw_overlay(p,
                          dim,
                          focused))
        .finish();
    gadget.batch.set(batch).await;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Normal,
    Hover,
    Pressed,
}

pub struct ButtonStyle {
    style_impl: Box<dyn ButtonStyleImpl + Send + Sync>,
}

impl ButtonStyle {
    pub fn from_impl(style_impl: impl ButtonStyleImpl + Send + Sync + 'static) -> Self {
        Self { style_impl: Box::new(style_impl) }
    }
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self::from_impl(SimpleButtonStyleImpl::default())
    }
}

pub trait ButtonStyleImpl {
    fn draw_backdrop(&self,
                     painting: Painting,
                     dim: ScalarPair,
                     enabled: bool,
                     state: ButtonState) -> Painting;
    fn draw_caption(&self,
                    painting: Painting,
                    dim: ScalarPair,
                    enabled: bool,
                    state: ButtonState,
                    text: String,
                    font: Arbitrary) -> Painting;
    fn draw_overlay(&self, painting: Painting,
                    dim: ScalarPair,
                    focused: bool) -> Painting;
}

pub struct SimpleButtonStyleImpl {
    pub bg_normal: Material,
    pub bg_hover: Material,
    pub bg_pressed: Material,
    pub bg_disabled: Material,
    pub fg_normal: Material,
    pub fg_hover: Material,
    pub fg_pressed: Material,
    pub fg_disabled: Material,
    pub li_focused: Material
}

impl Default for SimpleButtonStyleImpl {
    fn default() -> Self {
        SimpleButtonStyleImpl {
            bg_normal: SolidColor::gray(0.95).into(),
            bg_hover: SolidColor::gray(0.9).into(),
            bg_pressed: SolidColor::gray(0.85).into(),
            bg_disabled: SolidColor::gray(0.95).into(),
            fg_normal: Colors::BLACK.into(),
            fg_hover: Colors::BLACK.into(),
            fg_pressed: Colors::BLACK.into(),
            fg_disabled: SolidColor::gray(0.5).into(),
            li_focused: Colors::BLACK.into()
        }
    }
}

impl ButtonStyleImpl for SimpleButtonStyleImpl {
    fn draw_backdrop(&self,
                     painting: Painting,
                     dim: ScalarPair,
                     enabled: bool,
                     state: ButtonState
    ) -> Painting {
        let filling = if enabled {
            match state {
                ButtonState::Normal => self.bg_normal,
                ButtonState::Hover => self.bg_hover,
                ButtonState::Pressed => self.bg_pressed,
            }
        } else {
            self.bg_disabled
        };
        painting
            .path(
                Transform::default(),
                begin_draw()
                    .rect((0.0, 0.0), dim)
                    .finish(),
                Brush::from_fill(filling))
    }
    fn draw_caption(&self,
                    painting: Painting,
                    dim: ScalarPair,
                    enabled: bool,
                    state: ButtonState,
                    text: String,
                    font: Arbitrary
    ) -> Painting {
        let filling = if enabled {
            match state {
                ButtonState::Normal => self.fg_normal,
                ButtonState::Hover => self.fg_hover,
                ButtonState::Pressed => self.fg_pressed,
            }
        } else {
            self.fg_disabled
        };
        painting
            .text(
                Transform::from_translate(dim.times(0.5)),
                text, font,
                TextAlign::Center,
                Brush::from_fill(filling))
    }
    fn draw_overlay(&self,
                    painting: Painting,
                    dim: ScalarPair,
                    focused: bool
    ) -> Painting {
        painting
            .cond_with(focused, |p| p
                .path(Transform::default(),
                      begin_draw()
                          .rect((1.0, 1.0), dim - (2.0, 2.0).into())
                          .finish(),
                      Brush::from_stroke(self.li_focused, 2.0)))
    }
}