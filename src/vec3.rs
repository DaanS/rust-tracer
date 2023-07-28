use std::ops::{Add, Sub, Mul, Div, Neg, AddAssign};
use std::fmt::{Display, Formatter};

use crate::config::{Float, PI};
use approx::assert_ulps_eq;

pub type Point = Vec3;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: Float,
    pub y: Float,
    pub z: Float,
}

fn vec3(x: Float, y: Float, z: Float) -> Vec3 { Vec3{x, y, z} }

pub fn dot(u: Vec3, v: Vec3) -> Float {
    u.x * v.x +
    u.y * v.y +
    u.z * v.z
}

pub fn normalize(v: Vec3) -> Vec3 {
    v / v.length()
}

impl Vec3 {
    pub fn length_squared(self) -> Float {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(self) -> Float {
        self.length_squared().sqrt()
    }
}

impl From<(Float, Float, Float)> for Vec3 {
    fn from(t: (Float, Float, Float)) -> Self {
        Vec3{x: t.0, y: t.1, z: t.2}
    }
}

impl Display for Vec3 {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self { vec3(-self.x, -self.y, -self.z) }
}

impl Add for Vec3 {
    type Output = Self;
    fn add(self, v: Self) -> Self { vec3(self.x + v.x, self.y + v.y, self.z + v.z) }
}

impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, v: Self) -> Self { vec3(self.x - v.x, self.y - v.y, self.z - v.z) }
}

impl Mul<Float> for Vec3 {
    type Output = Self;
    fn mul(self, f: Float) -> Self { vec3(self.x * f, self.y * f, self.z * f) }
}

impl Div<Float> for Vec3 {
    type Output = Self;
    fn div(self, f: Float) -> Self { vec3(self.x / f, self.y / f, self.z / f) }
}

#[test]
fn test_basics() {
    let v1 = vec3(1.0, 2.0, 3.0);
    let v2 = vec3(6.0, 4.0, 2.0);
    assert_eq!(v1 + v2, vec3(7.0, 6.0, 5.0));
    assert_eq!(v2 - v1, vec3(5.0, 2.0, -1.0));
    assert_eq!(-v1, vec3(-1.0, -2.0, -3.0));
    assert_eq!(v1 * 2.0, vec3(2.0, 4.0, 6.0));
    assert_eq!(v2 / 2.0, vec3(3.0, 2.0, 1.0));
    //let mut mv1 = v1;
    //mv1 += v2;
    //assert_eq!(mv1, vec3(7.0, 6.0, 5.0));
    //mv1 -= v2;
    //assert_eq!(mv1, v1);
}

#[test]
fn test_dot() {
    let v_x = vec3(1.0, 0.0, 0.0);
    let v_y = vec3(0.0, 1.0, 0.0);
    assert_eq!(dot(v_x, v_x), 1.0);
    assert_eq!(dot(v_x, v_y), 0.0);
    assert_eq!(dot(v_x, -v_x), -1.0);
    let v_half = normalize(v_x + v_y);
    assert_ulps_eq!(dot(v_x, v_half), (PI / 4.0).cos());
}

#[test]
fn test_length() {
    let v = vec3(2.0, 0.0, 0.0);
    assert_eq!(v.length(), 2.0);
    assert_eq!(v.length_squared(), 4.0);
    assert_eq!(vec3(1.0, 1.0, 0.0).length(), Float::sqrt(2.0));
    assert_eq!(normalize(vec3(5.0, 3.0, 9.0)).length(), 1.0);
}