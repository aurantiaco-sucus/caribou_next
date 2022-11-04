use crate::caribou::batch::{Batch, Painting};
use crate::caribou::gadget::GadgetParent;
use crate::caribou::math::ScalarPair;
use crate::caribou::event::Event;
use crate::caribou::input::{ChainResult, FocusEvent, FocusResult, KeyEvent, MouseEvent};
use crate::caribou::value::Value;

pub trait GadgetLike: Clone + Send + Sync {
    fn type_name(&self) -> &'static str;
}

#[derive(Clone)]
pub struct Gadget2 {
    pub position: Value<ScalarPair>,
    pub dimension: Value<ScalarPair>,
    pub enabled: Value<bool>,
    pub parent: Value<GadgetParent>,
    pub on_paint: Event<(), Batch>,
    pub on_mouse: Event<MouseEvent, ChainResult<MouseEvent>>,
    pub on_key: Event<KeyEvent, ChainResult<KeyEvent>>,
    pub on_focus: Event<FocusEvent, FocusResult>,
}

impl GadgetLike for Gadget2 {
    fn type_name(&self) -> &'static str {
        "Caribou.Gadget"
    }
}

impl Default for Gadget2 {
    fn default() -> Self {
        Self {
            position: Value::new(ScalarPair::default()),
            dimension: Value::new(ScalarPair::default()),
            enabled: Value::new(true),
            parent: Value::new(GadgetParent::None),
            on_paint: Event::new(),
            on_mouse: Event::new(),
            on_key: Event::new(),
            on_focus: Event::new(),
        }
    }
}

impl Gadget2 {
    pub async fn is_focused(&self) -> bool {
        todo!()
    }
}