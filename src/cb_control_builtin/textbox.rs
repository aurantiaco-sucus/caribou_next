use crate::caribou::batch::{begin_draw, Brush, Colors, Material, Painting, SolidColor, Transform};
use crate::caribou::gadget::Gadget;
use crate::caribou::math::ScalarPair;
use crate::caribou::state::{Arbitrary, State};

pub struct TextBox;

impl TextBox {
    pub async fn create(style: TextBoxStyle) -> Gadget {
        let gadget = Gadget::default();

        // Fill common properties
        gadget.dim.set(ScalarPair::new(100.0, 30.0)).await;
        gadget.accept_focus.set(true).await;
        gadget.lock_focus.set(false).await;

        // Initial update
        textbox_batch_update(gadget.clone()).await;

        // Listen property updates
        gadget.dim.listen(
            "textbox_batch_update",
            move |event| Box::pin(async move {
                textbox_batch_update(event.gadget.get().unwrap()).await;
            }));

        gadget.focused.listen(
            "textbox_state_update",
            move |event| Box::pin(async move {
                textbox_batch_update(event.gadget.get().unwrap()).await;
            }));

        // Fill specialized data
        let data = TextBoxData {
            content: State::new_from(gadget.refer(), ""),
            state: State::new(gadget.refer(), TextBoxState::Unfocused),
        };

        // Listen data updates
        data.content.listen(
            "textbox_batch_update",
            move |event| Box::pin(async move {
                textbox_batch_update(event.gadget.get().unwrap()).await;
            }));

        data.state.listen(
            "textbox_batch_update",
            move |event| Box::pin(async move {
                textbox_batch_update(event.gadget.get().unwrap()).await;
            }));

        // Finish specialized data
        gadget.data.set_any(data).await;

        gadget
    }
}

pub struct TextBoxData {
    pub content: State<String>,
    state: State<TextBoxState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextBoxState {
    Unfocused,
    Edit,
    ImePreEdit,
}

async fn textbox_batch_update(textbox: Gadget) {

}

pub struct TextBoxStyle {
    style_impl: Box<dyn TextBoxStyleImpl + Send + Sync>,
}

impl TextBoxStyle {
    pub fn from_impl(style_impl: impl TextBoxStyleImpl + Send + Sync + 'static) -> Self {
        Self { style_impl: Box::new(style_impl) }
    }
}

pub trait TextBoxStyleImpl {
    fn draw_backdrop(&self,
                     painting: Painting,
                     dim: ScalarPair,
                     enabled: bool,
                     focused: bool) -> Painting;
    fn draw_overlay(&self,
                    painting: Painting,
                    dim: ScalarPair,
                    enabled: bool,
                    focused: bool) -> Painting;
    fn text_brush(&self,
                  enabled: bool,
                  focused: bool) -> Brush;
    fn cursor_brush(&self,
                    enabled: bool,
                    focused: bool) -> Brush;
}

pub struct SimpleTextBoxStyleImpl {
    pub bg_unfocused: Material,
    pub bg_focused: Material,
    pub bg_disabled: Material,
    pub fg_unfocused: Material,
    pub fg_focused: Material,
    pub fg_disabled: Material,
    pub li_unfocused: Material,
    pub li_focused: Material,
    pub li_disabled: Material,
}

impl Default for SimpleTextBoxStyleImpl {
    fn default() -> Self {
        Self {
            bg_unfocused: Colors::WHITE.into(),
            bg_focused: Colors::WHITE.into(),
            bg_disabled: SolidColor::gray(0.9).into(),
            fg_unfocused: Colors::BLACK.into(),
            fg_focused: Colors::BLACK.into(),
            fg_disabled: SolidColor::gray(0.5).into(),
            li_unfocused: SolidColor::gray(0.9).into(),
            li_focused: SolidColor::gray(0.8).into(),
            li_disabled: SolidColor::gray(0.5).into(),
        }
    }
}

impl TextBoxStyleImpl for SimpleTextBoxStyleImpl {
    fn draw_backdrop(&self,
                     painting: Painting,
                     dim: ScalarPair,
                     enabled: bool,
                     focused: bool) -> Painting {
        let fill = if enabled {
            if focused {
                &self.bg_focused
            } else {
                &self.bg_unfocused
            }
        } else {
            &self.bg_disabled
        };
        painting
            .path(Transform::default(),
                  begin_draw()
                      .rect((0.0, 0.0), dim)
                      .finish(),
                  Brush::from_fill(*fill))
    }

    fn draw_overlay(&self,
                    painting: Painting,
                    dim: ScalarPair,
                    enabled: bool,
                    focused: bool) -> Painting {
        let stroke = if enabled {
            if focused {
                &self.li_focused
            } else {
                &self.li_unfocused
            }
        } else {
            &self.li_disabled
        };
        painting
            .path(Transform::default(),
                  begin_draw()
                      .rect((0.0, 0.0), dim)
                      .finish(),
                  Brush::from_stroke(*stroke, 2.0))
    }

    fn text_brush(&self, enabled: bool, focused: bool) -> Brush {
        if enabled {
            if focused {
                Brush::from_fill(self.fg_focused)
            } else {
                Brush::from_fill(self.fg_unfocused)
            }
        } else {
            Brush::from_fill(self.fg_disabled)
        }
    }

    fn cursor_brush(&self, enabled: bool, focused: bool) -> Brush {
        if enabled {
            if focused {
                Brush::from_fill(self.li_focused)
            } else {
                Brush::from_fill(self.li_unfocused)
            }
        } else {
            Brush::from_fill(self.li_disabled)
        }
    }
}