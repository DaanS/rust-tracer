use crate::config::Float;

pub fn clamp(x: Float, min: Float, max: Float) -> Float {
    assert!(min <= max);
    if x < min { min }
    else if x > max { max }
    else { x }
}

#[test]
fn test_clamp() {
    assert_eq!(clamp(-1.0, 0.0, 1.0), 0.0);
    assert_eq!(clamp(0.0, 0.0, 1.0), 0.0);
    assert_eq!(clamp(0.5, 0.0, 1.0), 0.5);
    assert_eq!(clamp(10.0, 0.0, 1.0), 1.0);
}