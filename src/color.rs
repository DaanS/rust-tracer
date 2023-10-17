use std::ops::{Mul, Add};

use approx::{UlpsEq, AbsDiffEq};

use crate::config::Float;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ColorRgb {
    pub r: Float,
    pub g: Float,
    pub b: Float
}

pub fn color_rgb(r: Float, g: Float, b: Float) -> ColorRgb {
    ColorRgb { r, g, b }
}

impl ColorRgb {
    pub fn to_rgb(self) -> Self {
        self
    }
}

impl From<(Float, Float, Float)> for ColorRgb {
    fn from(t: (Float, Float, Float)) -> Self {
        ColorRgb{r: t.0, g: t.1, b: t.2}
    }
}

impl Add<ColorRgb> for ColorRgb { type Output = Self;
    fn add(self, c: ColorRgb) -> Self { color_rgb(self.r + c.r, self.g + c.g, self.b + c.b) }
}

impl Mul<ColorRgb> for ColorRgb { type Output = Self;
    fn mul(self, c: ColorRgb) -> Self { color_rgb(self.r * c.r, self.g * c.g, self.b * c.b) }
}

impl Mul<Float> for ColorRgb { type Output = Self;
    fn mul(self, f: Float) -> Self { color_rgb(self.r * f, self.g * f, self.b * f) }
}

impl Mul<ColorRgb> for Float { type Output = ColorRgb;
    fn mul(self, c: ColorRgb) -> ColorRgb { color_rgb(self * c.r, self * c.g, self * c.b) }
}

impl AbsDiffEq for ColorRgb {
    type Epsilon = <Float as AbsDiffEq>::Epsilon;

    fn default_epsilon() -> Self::Epsilon {
        Float::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        Float::abs_diff_eq(&self.r, &other.r, epsilon) &&
        Float::abs_diff_eq(&self.g, &other.g, epsilon) &&
        Float::abs_diff_eq(&self.b, &other.b, epsilon)
    }
}

impl UlpsEq for ColorRgb {
    fn default_max_ulps() -> u32 {
        Float::default_max_ulps()
    }

    fn ulps_eq(&self, other: &Self, epsilon: <Float as AbsDiffEq>::Epsilon, max_ulps: u32) -> bool {
        Float::ulps_eq(&self.r, &other.r, epsilon, max_ulps) &&
        Float::ulps_eq(&self.g, &other.g, epsilon, max_ulps) &&
        Float::ulps_eq(&self.b, &other.b, epsilon, max_ulps)
    }
}

#[test]
fn test_math() {
    use approx::assert_ulps_eq;

    let c1 = color_rgb(1., 0.8, 0.6);
    assert_eq!(0.5 * c1, color_rgb(0.5, 0.4, 0.3));
    assert_eq!(0.4 * c1, c1 * 0.4);
    assert_ulps_eq!(c1 * c1, color_rgb(1., 0.64, 0.36));
}