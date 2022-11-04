use std::ops::{Add, Sub};
use crate::caribou::batch::Transform;

pub type Scalar = f32;
pub type Integer = i32;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct ScalarPair {
    pub x: Scalar,
    pub y: Scalar,
}

impl ScalarPair {
    pub fn new(x: Scalar, y: Scalar) -> Self {
        Self { x, y }
    }

    pub fn one() -> Self {
        Self { x: 1.0, y: 1.0 }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn to_int(&self) -> IntPair {
        IntPair {
            x: self.x as Integer,
            y: self.y as Integer,
        }
    }

    pub fn element_wise_mul(&self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
        }
    }

    pub fn into_translate(self) -> Transform {
        Transform::from_translate(self)
    }

    pub fn into_scale(self) -> Transform {
        Transform::from_scale(self)
    }
}

impl From<(Scalar, Scalar)> for ScalarPair {
    fn from((x, y): (Scalar, Scalar)) -> Self {
        Self { x, y }
    }
}

impl Add for ScalarPair {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for ScalarPair {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl ScalarPair {
    pub fn times(&self, rhs: Scalar) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct IntPair {
    pub x: Integer,
    pub y: Integer,
}

impl IntPair {
    pub const fn new(x: Integer, y: Integer) -> Self {
        Self { x, y }
    }

    pub fn to_scalar(&self) -> ScalarPair {
        ScalarPair {
            x: self.x as Scalar,
            y: self.y as Scalar,
        }
    }
}

impl From<(Integer, Integer)> for IntPair {
    fn from((x, y): (Integer, Integer)) -> Self {
        Self { x, y }
    }
}

impl Add for IntPair {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for IntPair {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl IntPair {
    pub fn times(&self, rhs: Integer) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Region {
    pub origin: ScalarPair,
    pub size: ScalarPair,
}

impl Region {
    pub fn from_origin_size<T1: Into<ScalarPair>, T2: Into<ScalarPair>>
    (origin: T1, size: T2) -> Self {
        Self {
            origin: origin.into(),
            size: size.into(),
        }
    }

    pub fn from_begin_end<T1: Into<ScalarPair>, T2: Into<ScalarPair>>
    (begin: T1, end: T2) -> Self {
        let begin = begin.into();
        let end = end.into();
        Self {
            origin: begin,
            size: end - begin,
        }
    }

    pub fn contains<T: Into<ScalarPair>>(&self, point: T) -> bool {
        let point: ScalarPair = point.into();
        point.x >= self.origin.x && point.x <= self.origin.x + self.size.x &&
            point.y >= self.origin.y && point.y <= self.origin.y + self.size.y
    }

    pub fn contains_region<T: Into<Region>>(&self, other: T) -> bool {
        let other: Region = other.into();
        self.contains(other.origin) && self.contains(other.origin + other.size)
    }

    pub fn intersects<T: Into<Region>>(&self, other: T) -> bool {
        let other: Region = other.into();
        self.contains(other.origin) || self.contains(other.origin + other.size)
    }

    pub fn width(&self) -> Scalar {
        self.size.x
    }

    pub fn height(&self) -> Scalar {
        self.size.y
    }

    pub fn center(&self) -> ScalarPair {
        self.origin + self.size.times(0.5)
    }
}

impl From<(ScalarPair, ScalarPair)> for Region {
    fn from((origin, size): (ScalarPair, ScalarPair)) -> Self {
        Self { origin, size }
    }
}