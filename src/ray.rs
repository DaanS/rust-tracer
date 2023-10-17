use crate::vec3::{Point, Vec3};

#[derive(Clone, Debug, PartialEq)]
pub struct Ray {
    pub origin: Point,
    pub direction: Vec3
}

pub fn ray(origin: Point, direction: Vec3) -> Ray {
    Ray { origin, direction }
}

macro_rules! ray {
    (($ox:expr, $oy:expr, $oz:expr) -> ($dx:expr, $dy:expr, $dz:expr)) => {
        Ray{ origin: vec3!($ox, $oy, $oz), direction: vec3!($dx, $dy, $dz) } 
    };
}

#[test]
fn test_creation() {
    let r1 = Ray { origin: vec3!(1, 0, 0), direction: vec3!(0, 1, 0) };
    let r2 = ray!((1, 0, 0) -> (0, 1, 0));
    assert_eq!(r1, r2);
}