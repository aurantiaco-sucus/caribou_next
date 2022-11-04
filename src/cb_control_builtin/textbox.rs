use crate::caribou::batch::{begin_draw, begin_paint, Brush, Colors, Material, Painting, SolidColor, Transform};
use crate::caribou::gadget::Gadget;
use crate::caribou::math::ScalarPair;
use crate::caribou::state::{Arbitrary, State};

pub struct Textbox;

impl Textbox {
    pub async fn create(style: TextboxStyle) -> Gadget {
        let gadget = Gadget::default();

        // Fill common properties
        gadget.dim.set(ScalarPair::new(100.0, 30.0)).await;
        gadget.accept_focus.set(true).await;
        gadget.lock_focus.set(false).await;

        // Listen property updates
        gadget.dim.listen(
            "textbox_batch_update",
            move |event| Box::pin(async move {
                textbox_batch_update(event.gadget.get().unwrap()).await;
            })).await;

        gadget.focused.listen(
            "textbox_state_update",
            move |event| Box::pin(async move {
                textbox_batch_update(event.gadget.get().unwrap()).await;
            })).await;

        // Fill specialized data
        let data = TextBoxData {
            content: State::new_from(gadget.refer(), ""),
            state: State::new(gadget.refer(), TextBoxState::Unfocused),
            cursor: State::new(gadget.refer(), 0),
            style: State::new_any(gadget.refer(), style),
        };

        // Listen data updates
        data.content.listen(
            "textbox_batch_update",
            move |event| Box::pin(async move {
                textbox_batch_update(event.gadget.get().unwrap()).await;
            })).await;

        data.state.listen(
            "textbox_batch_update",
            move |event| Box::pin(async move {
                textbox_batch_update(event.gadget.get().unwrap()).await;
            })).await;

        data.cursor.listen(
            "textbox_batch_update",
            move |event| Box::pin(async move {
                textbox_batch_update(event.gadget.get().unwrap()).await;
            })).await;

        // Finish specialized data
        gadget.data.set_any(data).await;

        // Initial update
        textbox_batch_update(gadget.clone()).await;

        gadget
    }
}

pub struct TextBoxData {
    pub content: State<String>,
    state: State<TextBoxState>,
    pub cursor: State<usize>,
    pub style: State<Arbitrary>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextBoxState {
    Unfocused,
    Edit,
    PreEdit,
}

async fn textbox_batch_update(textbox: Gadget) {
    let dim = textbox.dim.get_cloned().await;
    let enabled = textbox.enabled.get_cloned().await;
    let pre = textbox.pre_edit.get_cloned().await;
    let pre_pos = textbox.pre_edit_pos.get_cloned().await;
    let font = textbox.font.get_cloned().await;
    let data = textbox.data.get().await;
    let data = data.get::<TextBoxData>().await;
    let content = data.content.get_cloned().await;
    let state = data.state.get_cloned().await;
    let cursor = data.cursor.get_cloned().await;
    let style = data.style.get_cloned().await;
    let style = style.get::<TextboxStyle>().unwrap();
    drop(data);
    let mut painting = begin_paint();
    if !enabled {
        painting = painting
            .with(|p| style.style_impl
                .draw_backdrop(p, dim, enabled, false));
        // todo: disabled ui: no cursor, no positioning, not interactive
    } else {
        match state {
            TextBoxState::Unfocused => {
                // todo: display ui: no cursor, positioning
            }
            TextBoxState::Edit => {
                // todo: edit ui: blinking cursor, positioning
            }
            TextBoxState::PreEdit => {
                // todo: pre-edit ui: pre-edit static cursor, positioning, pre-edit text
            }
        }
    }
    let batch = begin_paint()
        .batch(Transform::from_clip(dim), painting.finish())
        .finish();
    textbox.batch.set(batch).await;
}

async fn textbox_text_plain(painting: Painting,
                            content: String,
                            brush: Brush,
                            font: Arbitrary) -> Painting {
    painting
}

pub struct TextboxStyle {
    style_impl: Box<dyn TextBoxStyleImpl + Send + Sync>,
}

impl TextboxStyle {
    pub fn from_impl(style_impl: impl TextBoxStyleImpl + Send + Sync + 'static) -> Self {
        Self { style_impl: Box::new(style_impl) }
    }
}

impl Default for TextboxStyle {
    fn default() -> Self {
        Self { style_impl: Box::new(SimpleTextBoxStyleImpl::default()) }
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