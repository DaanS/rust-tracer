use crate::config::{Float};
use crate::ray::{Ray};
use crate::vec3::{Point, dot};

pub struct HitRecord {
    pub t: Float
}

pub trait Hit {
    fn hit(&self, r: Ray, t_min: Float, t_max: Float) -> Option<HitRecord>;
}

pub struct Sphere {
    pub center: Point,
    pub radius: Float
}

pub fn sphere(center: Point, radius: Float) -> Sphere {
    Sphere{center, radius}
}

impl Hit for Sphere {
    fn hit(&self, r: Ray, t_min: Float, t_max: Float) -> Option<HitRecord> {
        let oc = r.origin - self.center;
        let a = r.direction.length_squared();
        let half_b = dot(oc, r.direction);
        let c = oc.length_squared() - self.radius * self.radius;

        let d = half_b * half_b - a * c;
        if d < 0.0 { return None; }

        let sqrt_d = d.sqrt();
        let root_1 = (-half_b - sqrt_d) / a;
        if root_1 >= t_min && root_1 <= t_max { return Some(HitRecord{t: root_1}); }

        let root_2 = (-half_b - sqrt_d) / a;
        if root_2 >= t_min && root_2 <= t_max { return Some(HitRecord{t: root_2}); }

        None
    } 
}

#[test]
fn test_hit_sphere() {
    let s = sphere((0.0, 0.0, 0.0).into(), 1.0);

    assert!(s.hit(Ray{origin: (-10.0, 0.0, 0.0).into(), direction: (1.0, 0.0, 0.0).into()}, 0.0, f64::INFINITY).is_some());
}