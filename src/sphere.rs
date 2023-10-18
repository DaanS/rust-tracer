use crate::{
    config::Float,
    hit::{Hit, HitRecord},
    ray::Ray,
    vec3::{dot, Point}, material::Material,
};

pub struct Sphere {
    pub center: Point,
    pub radius: Float,
    pub material: Material,
}

pub fn sphere(center: Point, radius: Float, material: Material) -> Sphere { 
    Sphere { center, radius, material } 
}

impl Hit for Sphere {
    fn hit(&self, r: Ray, t_min: Float, t_max: Float) -> Option<HitRecord> {
        let oc = r.origin - self.center;
        let a = r.direction.length_squared();
        let half_b = dot(oc, r.direction);
        let c = oc.length_squared() - self.radius * self.radius;

        let d = half_b * half_b - a * c;
        if d < 0. { return None; }

        let sqrt_d = d.sqrt();
        let root_1 = (-half_b - sqrt_d) / a;
        if root_1 >= t_min && root_1 <= t_max { return Some(HitRecord { t: root_1, material: self.material.clone() }); }

        let root_2 = (-half_b + sqrt_d) / a;
        if root_2 >= t_min && root_2 <= t_max { return Some(HitRecord { t: root_2, material: self.material.clone() }); }

        None
    }
}

#[test]
fn test_hit_sphere() {
    let s = sphere((0., 0., 0.).into(), 1., Material::None);

    assert!(s.hit(ray!((-10, 0, 0) -> (1., 0., 0.)), 0., f64::INFINITY).is_some());
    assert!(s.hit(ray!((-2, 0, 0) -> (1, 0, 0)), 0., f64::MAX).unwrap().t == 1.);
    assert!(s.hit(ray!((2, 0, 0) -> (-1, 0, 0)), 0., f64::MAX).unwrap().t == 1.);
    assert!(s.hit(ray!((0, 0, 0) -> (1, 0, 0)), 0., f64::MAX).unwrap().t == 1.);
}