use crate::config::{Float, PI};

pub fn clamp(x: Float, min: Float, max: Float) -> Float {
    assert!(min <= max);
    if x < min { min }
    else if x > max { max }
    else { x }
}

pub fn radians(degrees: Float) -> Float {
    degrees * PI / 180.
}

#[test]
fn test_clamp() {
    assert_eq!(clamp(-1., 0., 1.), 0.);
    assert_eq!(clamp(0., 0., 1.), 0.);
    assert_eq!(clamp(0.5, 0., 1.), 0.5);
    assert_eq!(clamp(10., 0., 1.), 1.);
}