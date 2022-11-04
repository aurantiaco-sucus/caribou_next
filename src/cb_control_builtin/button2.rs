use std::sync::{Arc, Mutex};
use crate::caribou::event::Event;
use crate::caribou::gadget2::{Gadget2, GadgetLike};
use crate::caribou::value::Value;
use crate::{as_clone, deref_to_super};
use crate::caribou::batch::{begin_draw, begin_paint, Brush, Colors, SolidColor, TextAlign, Transform};
use crate::caribou::text::{Font, FontInfo};

#[derive(Clone)]
struct Button {
    super_struct: Gadget2,
    state: Value<ButtonState>,
    pub caption: Value<String>,
    pub font: Value<Font>,
    pub on_action: Event,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ButtonState {
    Normal,
    Hover,
    Pressed,
}

deref_to_super!(Button => Gadget2);

impl GadgetLike for Button {
    fn type_name(&self) -> &'static str {
        "Caribou.Button"
    }
}

impl Button {
    pub async fn new() -> Self {
        button_create().await
    }
}

async fn button_create() -> Button {
    let button = Button {
        super_struct: Gadget2::default(),
        state: Value::new(ButtonState::Normal),
        caption: Value::new("".to_string()),
        font: Value::new(FontInfo::default().resolve().unwrap()),
        on_action: Event::new(),
    };

    {
        as_clone!(button => bc);
        button.on_paint.listen(move |_| {
            as_clone!(bc => button);
            Box::pin(async move {
                let dimension = button.dimension.get().await;
                let enabled = button.enabled.get().await;
                let focused = button.is_focused().await;
                let caption = button.caption.get().await;
                let font = button.font.get().await;
                let state = button.state.get().await;
                begin_paint()
                    .path(
                        Transform::default(),
                        begin_draw()
                            .rect((0.0, 0.0), dimension)
                            .finish(),
                        if enabled {
                            match state {
                                ButtonState::Normal =>
                                    Brush::from_fill(SolidColor::gray(0.95)),
                                ButtonState::Hover =>
                                    Brush::from_fill(SolidColor::gray(0.9)),
                                ButtonState::Pressed =>
                                    Brush::from_fill(SolidColor::gray(0.85)),
                            }
                        } else {
                            Brush::from_fill(SolidColor::gray(0.975))
                        }
                    )
                    .text2(
                        Transform::from_translate(dimension.times(0.5)),
                        caption,
                        font,
                        TextAlign::Center,
                        if enabled {
                            Brush::from_fill(Colors::BLACK)
                        } else {
                            Brush::from_fill(SolidColor::gray(0.5))
                        }
                    )
                    .cond_with(focused, |p| p
                        .path(
                            Transform::default(),
                            begin_draw()
                                .rect((0.0, 0.0), dimension)
                                .finish(),
                            Brush::from_stroke(SolidColor::gray(0.8), 2.0)
                        ))
                    .finish()
            })
        }).await;
    }

    button
}