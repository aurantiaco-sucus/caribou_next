use crate::caribou::AsyncTask;
use crate::caribou::batch::{begin_paint, Painting, Batch, Brush, Drawing, Material, Transform, begin_draw, TextAlign, SolidColor, Colors};
use crate::caribou::focus::FocusEventInfo;
use crate::caribou::gadget::Gadget;
use crate::caribou::input::{Key, MouseEventInfo};
use crate::caribou::math::ScalarPair;
use crate::caribou::native::Native;
use crate::caribou::value::{new_value, Value};

pub struct Button;

impl Button {
    pub async fn create(style: ButtonStyle) -> Gadget {
        let gadget = Gadget::default();

        // Handle events

        let gr = gadget.refer();
        gadget.draw.handle(move || {
            let gr = gr.clone();
            AsyncTask::wrap(async move {
                let gadget = gr.get().unwrap();
                // Gadget properties
                let enabled = *gadget.enabled.get().await;
                let focused = gadget.is_focused().await;
                let dim = *gadget.dim.get().await;
                let font = gadget.font.get().await;
                let data = gadget.data.get::<ButtonData>().await;
                // Data properties
                let style = data.style.get().await;
                let state = data.state;
                let caption = data.caption.get().await;
                begin_paint()
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
                    .finish()
            })
        }).await;

        let gr = gadget.refer();
        gadget.mouse.handle(move |info| {
            let gr = gr.clone();
            AsyncTask::wrap(async move {
                let gadget = gr.get().unwrap();
                let enabled = *gadget.enabled.get().await;
                let mut data = gadget.data
                    .get_mut::<ButtonData>().await;
                if enabled {
                    match &info {
                        MouseEventInfo::Enter => {
                            data.state = ButtonState::Hover;
                        }
                        MouseEventInfo::Leave => {
                            data.state = ButtonState::Normal;
                        }
                        MouseEventInfo::Down { .. } => {
                            data.state = ButtonState::Pressed;
                        }
                        MouseEventInfo::Up { .. } => {
                            data.state = ButtonState::Hover;
                            gadget.action.broadcast().await;
                        }
                        MouseEventInfo::Move { .. } => {}
                    }
                }
                gadget.get_window().await.unwrap().get().unwrap().request_redraw();
            })
        }).await;

        let gr = gadget.refer();
        gadget.focus.handle(move |focused| {
            let gr = gr.clone();
            AsyncTask::wrap(async move {
                let gadget = gr.get().unwrap();
                gadget.get_window().await.unwrap().get().unwrap().request_redraw();
                true
            })
        }).await;

        let gr = gadget.refer();
        gadget.key.handle(move |info| {
            let gr = gr.clone();
            AsyncTask::wrap(async move {
                let gadget = gr.get().unwrap();
                let mut data = gadget.data
                    .get_mut::<ButtonData>().await;
                let enabled = *gadget.enabled.get().await;
                if enabled {
                    if info.is_down && info.key == Key::Return {
                        data.state = ButtonState::Pressed;
                    } else if !info.is_down && info.key == Key::Return {
                        data.state = ButtonState::Normal;
                        gadget.action.broadcast().await;
                    }
                    gadget.get_window().await.unwrap().get().unwrap().request_redraw();
                }
            })
        }).await;

        // Fill specialized data

        let data = ButtonData {
            style: new_value(style),
            caption: new_value(String::from("Button")),
            state: ButtonState::Normal,
        };
        gadget.data.set(data).await;

        // Fill common properties

        gadget.dim.set(ScalarPair::new(100.0, 30.0)).await;
        gadget
    }
}

pub struct ButtonData {
    pub style: Value<ButtonStyle>,
    pub caption: Value<String>,
    state: ButtonState,
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
                    font: Native) -> Painting;
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
                    font: Native
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
                      Brush::from_stroke(self.li_focused)
                          .width(2.0)))
    }
}