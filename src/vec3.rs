use std::fmt::{Display, Formatter};
use std::ops::{Add, Div, Mul, Neg, Sub};

use crate::config::Float;
use crate::random::random_in_range;

pub type Point = Vec3;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: Float,
    pub y: Float,
    pub z: Float,
}

pub fn vec3(x: Float, y: Float, z: Float) -> Vec3 {
    Vec3 { x, y, z }
}

macro_rules! vec3 {
    ($x:expr, $y: expr, $z: expr) => {
        crate::vec3::Vec3 { x: $x as crate::config::Float, y: $y as crate::config::Float, z: $z as crate::config::Float }
    };
}

pub fn dot(u: Vec3, v: Vec3) -> Float {
    u.x * v.x + u.y * v.y + u.z * v.z
}

pub fn cross(u: Vec3, v: Vec3) -> Vec3 {
    vec3(u.y * v.z - u.z * v.y,
         u.z * v.x - u.x * v.z,
         u.x * v.y - u.y * v.x)
}

impl Vec3 {
    pub fn length_squared(self) -> Float { self.x * self.x + self.y * self.y + self.z * self.z }

    pub fn length(self) -> Float { self.length_squared().sqrt() }

    pub fn normalize(self) -> Vec3 {
        self / self.length()
    }

    pub fn near_zero(self) -> bool {
        self.x.abs() < Float::EPSILON && self.y.abs() < Float::EPSILON && self.z.abs() < Float::EPSILON
    }
}

pub fn random_unit_vector() -> Vec3 {
    let mut candidate = vec3(random_in_range(-1.0, 1.0), random_in_range(-1.0, 1.0), random_in_range(-1.0, 1.0));
    while candidate.length_squared() > 1.0 {
        candidate = vec3(random_in_range(-1.0, 1.0), random_in_range(-1.0, 1.0), random_in_range(-1.0, 1.0));
    }
    candidate.normalize()
}

pub fn random_vector_in_unit_disk() -> Vec3 {
    let mut candidate = vec3(random_in_range(-1.0, 1.0), random_in_range(-1.0, 1.0), 0.);
    while candidate.length_squared() > 1.0 {
        candidate = vec3(random_in_range(-1.0, 1.0), random_in_range(-1.0, 1.0), 0.);
    }
    candidate
}

impl From<(Float, Float, Float)> for Vec3 {
    fn from(t: (Float, Float, Float)) -> Self {
        Vec3 { x: t.0, y: t.1, z: t.2, }
    }
}

impl From<Vec3> for (Float, Float, Float) {
    fn from(v: Vec3) -> Self {
        (v.x, v.y, v.z)
    }
}

impl Display for Vec3 {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl Neg for Vec3 { type Output = Self;
    fn neg(self) -> Self { vec3(-self.x, -self.y, -self.z) }
}

impl Add for Vec3 { type Output = Self;
    fn add(self, v: Self) -> Self { vec3(self.x + v.x, self.y + v.y, self.z + v.z) }
}

impl Sub for Vec3 { type Output = Self;
    fn sub(self, v: Self) -> Self { vec3(self.x - v.x, self.y - v.y, self.z - v.z) }
}

impl Mul<Float> for Vec3 { type Output = Self;
    fn mul(self, f: Float) -> Self { vec3(self.x * f, self.y * f, self.z * f) }
}

impl Mul<Vec3> for Float { type Output = Vec3;
    fn mul(self, v: Vec3) -> Vec3 { vec3(self * v.x, self * v.y, self * v.z) }
}

impl Div<Float> for Vec3 { type Output = Self;
    fn div(self, f: Float) -> Self { vec3(self.x / f, self.y / f, self.z / f) }
}

#[test]
fn test_basics() {
    let v1 = vec3!(1, 2, 3);
    let v2 = vec3!(6, 4, 2);
    assert_eq!(v1 + v2, vec3!(7, 6, 5));
    assert_eq!(v2 - v1, vec3!(5, 2, -1));
    assert_eq!(-v1, vec3!(-1, -2, -3));
    assert_eq!(v1 * 2., vec3!(2, 4, 6));
    assert_eq!(v2 / 2., vec3!(3, 2, 1));
    assert_eq!(v1 * 2., 2. * v1);
}

#[cfg(test)]
mod tests {
    use approx::assert_ulps_eq;
    use crate::{config::{PI, Float}, vec3::{dot, random_unit_vector}};

    #[test]
    fn test_dot() {

        let v_x = vec3!(1, 0, 0);
        let v_y = vec3!(0, 1, 0);
        assert_eq!(dot(v_x, v_x), 1.);
        assert_eq!(dot(v_x, v_y), 0.);
        assert_eq!(dot(v_x, -v_x), -1.);
        let v_half = (v_x + v_y).normalize();
        assert_ulps_eq!(dot(v_x, v_half), (PI / 4.).cos());
    }

    #[test]
    fn test_length() {
        let v = vec3!(2, 0, 0);
        assert_eq!(v.length(), 2.);
        assert_eq!(v.length_squared(), 4.);
        assert_eq!(vec3!(1, 1, 0).length(), Float::sqrt(2.));
        assert_eq!((vec3!(5, 3, 9)).normalize().length(), 1.);
    }

    #[test]
    fn test_random() {
        let v = random_unit_vector();
        assert_ulps_eq!(v.length(), 1.);
    }
}