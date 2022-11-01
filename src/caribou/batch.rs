
use std::fmt::{Debug};
use std::ops::{Deref, DerefMut};


use crate::caribou::math::{Scalar, ScalarPair};
use crate::caribou::state::Arbitrary;

#[repr(transparent)]
#[derive(Debug, Default, Clone)]
pub struct Drawing {
    ops: Vec<PathOp>,
}

pub fn begin_draw() -> Drawing {
    Drawing::default()
}

impl Drawing {
    pub fn finish(self) -> Path {
        Path { ops: self.ops }
    }

    pub fn move_to<S: Into<ScalarPair>>(mut self, pos: S) -> Self {
        self.ops.push(PathOp::MoveTo(pos.into()));
        self
    }

    pub fn line_to<S: Into<ScalarPair>>(mut self, pos: S) -> Self {
        self.ops.push(PathOp::LineTo(pos.into()));
        self
    }

    pub fn quad_to<S1, S2>(mut self, pos: S1, ctrl: S2) -> Self
        where S1: Into<ScalarPair>, S2: Into<ScalarPair>
    {
        self.ops.push(PathOp::QuadTo(pos.into(),
                                     ctrl.into()));
        self
    }

    pub fn cubic_to<S1, S2, S3>(mut self, pos: S1, ctrl1: S2, ctrl2: S3) -> Self
        where S1: Into<ScalarPair>, S2: Into<ScalarPair>, S3: Into<ScalarPair>
    {
        self.ops.push(PathOp::CubicTo(pos.into(),
                                      ctrl1.into(),
                                      ctrl2.into()));
        self
    }

    pub fn close(mut self) -> Self {
        self.ops.push(PathOp::Close);
        self
    }

    pub fn line<S1, S2>(mut self, begin: S1, end: S2) -> Self
        where S1: Into<ScalarPair>, S2: Into<ScalarPair>
    {
        self.ops.push(PathOp::AddLine(begin.into(), end.into()));
        self
    }

    pub fn rect<S1, S2>(mut self, pos: S1, dim: S2) -> Self
        where S1: Into<ScalarPair>, S2: Into<ScalarPair>
    {
        self.ops.push(PathOp::AddRect(pos.into(),
                                      dim.into()));
        self
    }

    pub fn oval<S1, S2>(mut self, pos: S1, dim: S2) -> Self
        where S1: Into<ScalarPair>, S2: Into<ScalarPair>
    {
        self.ops.push(PathOp::AddOval(pos.into(),
                                      dim.into()));
        self
    }
}

#[repr(transparent)]
#[derive(Debug, Default, Clone)]
pub struct Painting {
    ops: Vec<BatchOp>,
}

pub fn begin_paint() -> Painting {
    Painting::default()
}

impl Painting {
    pub fn finish(self) -> Batch {
        Batch { ops: self.ops }
    }

    pub fn path(self, transform: Transform, path: Path, brush: Brush) -> Self {
        let mut ops = self.ops;
        ops.push(BatchOp::Path { transform, path, brush });
        Self { ops }
    }

    pub fn image(self, transform: Transform, image: Arbitrary) -> Self {
        let mut ops = self.ops;
        ops.push(BatchOp::Image { transform, image });
        Self { ops }
    }

    pub fn text(self,
                transform: Transform,
                text: String, font: Arbitrary,
                align: TextAlign,
                brush: Brush
    ) -> Self {
        let mut ops = self.ops;
        ops.push(BatchOp::Text(Box::new(TextOp {
            transform, text, font, align, brush
        })));
        Self { ops }
    }

    pub fn batch(self, transform: Transform, batch: Batch) -> Self {
        let mut ops = self.ops;
        ops.push(BatchOp::Batch { transform, batch });
        Self { ops }
    }

    pub fn with<F: Fn(Painting) -> Painting>(self, func: F) -> Self {
        func(self)
    }

    pub fn cond_with<F: Fn(Painting) -> Painting>(self, pred: bool, func: F) -> Self {
        if pred { func(self) } else { self }
    }
}

#[repr(transparent)]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Batch {
    ops: Vec<BatchOp>,
}

impl Deref for Batch {
    type Target = Vec<BatchOp>;

    fn deref(&self) -> &Self::Target {
        &self.ops
    }
}

impl DerefMut for Batch {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ops
    }
}

impl Batch {
    pub fn batch_append(&mut self, other: Batch) {
        self.ops.extend(other.ops);
    }

    pub fn unwrap(self) -> Vec<BatchOp> {
        self.ops
    }
}

pub trait BatchFlattening {
    fn flatten(self) -> Batch;
}

impl BatchFlattening for Vec<Batch> {
    fn flatten(self) -> Batch {
        let mut batch = Batch::default();
        for mut b in self {
            batch.append(&mut b);
        }
        batch
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BatchOp {
    Path {
        transform: Transform,
        path: Path,
        brush: Brush,
    },
    Image {
        transform: Transform,
        image: Arbitrary,
    },
    Text(Box<TextOp>),
    Batch {
        transform: Transform,
        batch: Batch,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextOp {
    pub transform: Transform,
    pub text: String,
    pub font: Arbitrary,
    pub brush: Brush,
    pub align: TextAlign,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextAlign {
    Origin,
    Center
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub translate: ScalarPair,
    pub scale: ScalarPair,
    pub rotate: Scalar,
    pub rotate_center: ScalarPair,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translate: ScalarPair::zero(),
            scale: ScalarPair::one(),
            rotate: Scalar::default(),
            rotate_center: ScalarPair::zero(),
        }
    }
}

impl Transform {
    pub fn from_translate<S: Into<ScalarPair>>(translate: S) -> Self {
        Self {
            translate: translate.into(),
            ..Default::default()
        }
    }

    pub fn translate<S: Into<ScalarPair>>(&self, translate: S) -> Self {
        Self {
            translate: self.translate + translate.into(),
            ..*self
        }
    }

    pub fn from_scale<S: Into<ScalarPair>>(scale: S) -> Self {
        Self {
            scale: scale.into(),
            ..Default::default()
        }
    }

    pub fn scale<S: Into<ScalarPair>>(&self, scale: S) -> Self {
        Self {
            scale: self.scale.element_wise_mul(scale.into()),
            ..*self
        }
    }

    pub fn from_rotate<S: Into<Scalar>>(rotate: S) -> Self {
        Self {
            rotate: rotate.into(),
            ..Default::default()
        }
    }

    pub fn rotate<S: Into<Scalar>>(&self, rotate: S) -> Self {
        Self {
            rotate: self.rotate + rotate.into(),
            ..*self
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[repr(transparent)]
pub struct Path {
    ops: Vec<PathOp>,
}

impl Path {
    pub fn unwrap(self) -> Vec<PathOp> {
        self.ops
    }
}

impl Deref for Path {
    type Target = Vec<PathOp>;

    fn deref(&self) -> &Self::Target {
        &self.ops
    }
}

impl DerefMut for Path {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ops
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PathOp {
    MoveTo(ScalarPair),
    LineTo(ScalarPair),
    QuadTo(ScalarPair, ScalarPair),
    CubicTo(ScalarPair, ScalarPair, ScalarPair),
    Close,
    AddLine(ScalarPair, ScalarPair),
    AddRect(ScalarPair, ScalarPair),
    AddOval(ScalarPair, ScalarPair),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Brush {
    pub stroke: Material,
    pub fill: Material,
    pub width: Scalar,
}

impl Default for Brush {
    fn default() -> Self {
        Self {
            stroke: Material::Transparent,
            fill: Material::Transparent,
            width: 1.0,
        }
    }
}

impl Brush {
    pub fn from_stroke_fill<S: Into<Material>>(stroke: S, fill: S) -> Self {
        Self {
            stroke: stroke.into(),
            fill: fill.into(),
            ..Default::default()
        }
    }

    pub fn from_stroke<S: Into<Material>>(stroke: S, width: Scalar) -> Self {
        Self {
            stroke: stroke.into(),
            width,
            ..Default::default()
        }
    }

    pub fn from_fill<S: Into<Material>>(fill: S) -> Self {
        Self {
            fill: fill.into(),
            ..Default::default()
        }
    }

    pub fn stroke<S: Into<Material>>(self, stroke: S) -> Self {
        Self {
            stroke: stroke.into(),
            ..self
        }
    }

    pub fn fill<S: Into<Material>>(self, fill: S) -> Self {
        Self {
            fill: fill.into(),
            ..self
        }
    }

    pub fn width<S: Into<Scalar>>(self, width: S) -> Self {
        Self {
            width: width.into(),
            ..self
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Material {
    Transparent,
    Solid(SolidColor)
}

impl Material {
    pub fn transparent() -> Self {
        Material::Transparent
    }

    pub fn solid<S: Into<SolidColor>>(color: S) -> Self {
        Material::Solid(color.into())
    }

    pub fn is_transparent(&self) -> bool {
        match self {
            Material::Transparent => true,
            _ => false,
        }
    }
}

impl Into<Material> for SolidColor {
    fn into(self) -> Material {
        Material::Solid(self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolidColor {
    pub r: Scalar,
    pub g: Scalar,
    pub b: Scalar,
    pub a: Scalar,
}

impl Into<SolidColor> for (Scalar, Scalar, Scalar, Scalar) {
    fn into(self) -> SolidColor {
        SolidColor {
            r: self.0,
            g: self.1,
            b: self.2,
            a: self.3,
        }
    }
}

impl SolidColor {
    pub fn gray<S: Into<Scalar>>(gray: S) -> Self {
        let gray = gray.into();
        Self {
            r: gray,
            g: gray,
            b: gray,
            a: 1.0,
        }
    }

    pub fn gray_alpha<S: Into<Scalar>>(gray: S, alpha: S) -> Self {
        let gray = gray.into();
        Self {
            r: gray,
            g: gray,
            b: gray,
            a: alpha.into(),
        }
    }

    pub fn opaque<S: Into<Scalar>>(r: S, g: S, b: S) -> Self {
        Self {
            r: r.into(),
            g: g.into(),
            b: b.into(),
            a: 1.0,
        }
    }
}

pub struct Colors;

impl Colors {
    pub const BLACK: SolidColor = SolidColor { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const WHITE: SolidColor = SolidColor { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const RED: SolidColor = SolidColor { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN: SolidColor = SolidColor { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE: SolidColor = SolidColor { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
    pub const YELLOW: SolidColor = SolidColor { r: 1.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const CYAN: SolidColor = SolidColor { r: 0.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const MAGENTA: SolidColor = SolidColor { r: 1.0, g: 0.0, b: 1.0, a: 1.0 };
    pub const GRAY: SolidColor = SolidColor { r: 0.5, g: 0.5, b: 0.5, a: 1.0 };
    pub const GRAY_LIGHT: SolidColor = SolidColor { r: 0.75, g: 0.75, b: 0.75, a: 1.0 };
    pub const GRAY_DARK: SolidColor = SolidColor { r: 0.25, g: 0.25, b: 0.25, a: 1.0 };
    pub const TRANSPARENT: SolidColor = SolidColor { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
}