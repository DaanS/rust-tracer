use crate::{config::{Float, PI}, random::random_in_range};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Interval {
    pub min: Float,
    pub max: Float,
}

impl From<(Float, Float)> for Interval {
    fn from((min, max): (Float, Float)) -> Self {
        assert!(min <= max, "Interval min must be less than or equal to max");
        assert!(!min.is_nan() && !max.is_nan(), "Interval values must not be NaN");
        Self { min, max }
    }
}

impl Interval {
    pub fn enclosing(a: Self, b: Self) -> Self {
        Self {
            min: a.min.min(b.min),
            max: a.max.max(b.max),
        }
    }

    pub fn contains(&self, value: Float) -> bool {
        value >= self.min && value <= self.max
    }

    pub fn length(&self) -> Float {
        self.max - self.min
    }

    pub fn random(&self) -> Float {
        random_in_range(self.min, self.max)
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