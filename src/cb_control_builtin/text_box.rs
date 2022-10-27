use crate::caribou::batch::{Colors, Material, Painting, SolidColor};
use crate::caribou::gadget::Gadget;
use crate::caribou::math::ScalarPair;
use crate::caribou::native::Native;
use crate::caribou::value::Value;

pub struct TextBox;

impl TextBox {
    pub async fn create(style: TextBoxStyle) -> Gadget {
        let gadget = Gadget::default();
        gadget
    }
}

pub struct TextBoxData {
    pub content: Value<String>,
    state: TextBoxState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextBoxState {
    Unfocused,
    Focused,
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
    fn draw_content(&self,
                    painting: Painting,
                    dim: ScalarPair,
                    enabled: bool,
                    focused: bool,
                    text: String,
                    font: Native) -> Painting;
    fn draw_overlay(&self,
                    painting: Painting,
                    dim: ScalarPair,
                    focused: bool) -> Painting;
    fn draw_cursor(&self,
                   painting: Painting,
                   data: &TextBoxData) -> (Painting, ScalarPair);
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