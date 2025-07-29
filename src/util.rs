use crate::config::{Float, PI};

#[derive(Clone, Copy)]
pub struct Interval {
    pub min: Float,
    pub max: Float,
}

impl From<(Float, Float)> for Interval {
    fn from((min, max): (Float, Float)) -> Self {
        Interval { min, max }
    }
}

pub fn clamp(x: Float, (min, max): (Float, Float)) -> Float {
    assert!(min <= max);
    if x < min { min }
    else if x > max { max }
    else { x }
}

pub fn radians(degrees: Float) -> Float {
    degrees * PI / 180.
}

pub fn is_power_of_2(n: usize) -> bool {
    (n > 0) && ((n & (n - 1)) == 0)
}

#[test]
fn test_clamp() {
    assert_eq!(clamp(-1., (0., 1.)), 0.);
    assert_eq!(clamp(0., (0., 1.)), 0.);
    assert_eq!(clamp(0.5, (0., 1.)), 0.5);
    assert_eq!(clamp(10., (0., 1.)), 1.);
}