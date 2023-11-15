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
        let mut root = (-half_b - sqrt_d) / a;
        if root < t_min || root > t_max { 
            root = (-half_b + sqrt_d) / a;
            if root < t_min || root > t_max { 
                return None;
            }
        }

        let pos = r.at(root);
        let normal = (pos - self.center) / self.radius;

        Some(HitRecord { t: root, material: self.material.clone(), normal, pos }) 
    }
}

// TODO put this somewhere sensible when we get more shapes
impl Hit for Vec<Sphere> {
    fn hit(&self, r: Ray, t_min: Float, t_max: Float) -> Option<HitRecord> {
        let mut res = None;
        let mut cur_t_max = t_max;

        for s in self {
            if let Some(hit_record) = s.hit(r.clone(), t_min, cur_t_max) {
                cur_t_max = hit_record.t;
                res = Some(hit_record);
            }
        }

        res
    }
}

#[test]
fn test_hit_sphere() {
    let s = sphere((0., 0., 0.).into(), 1., Material::None);

    assert!(s.hit(ray!((-10, 0, 0) -> (1., 0., 0.)), 0., f64::INFINITY).is_some());

    assert!(s.hit(ray!((-2, 0, 0) -> (1, 0, 0)), 0., f64::MAX).unwrap().t == 1.);
    assert!(s.hit(ray!((2, 0, 0) -> (-1, 0, 0)), 0., f64::MAX).unwrap().t == 1.);
    assert!(s.hit(ray!((0, 0, 0) -> (1, 0, 0)), 0., f64::MAX).unwrap().t == 1.);

    assert!(s.hit(ray!((-2, 0, 0) -> (1, 0, 0)), 0., f64::MAX).unwrap().normal == vec3!(-1, 0, 0));
    assert!(s.hit(ray!((2, 0, 0) -> (-1, 0, 0)), 0., f64::MAX).unwrap().normal == vec3!(1, 0, 0));
    assert!(s.hit(ray!((0, 0, 0) -> (1, 0, 0)), 0., f64::MAX).unwrap().normal == vec3!(1, 0, 0));
}

#[test]
fn test_hit_vec() {
    let s1 = sphere((1., 0., 0.).into(), 1., Material::None);
    let s2 = sphere((-1., 0., 0.).into(), 1., Material::None);
    let v = vec![s1, s2];

    assert_eq!(v.hit(ray!((3, 0, 0) -> (-1, 0, 0)), 0., f64::MAX).unwrap().t, 1.);
    assert_eq!(v.hit(ray!((-3, 0, 0) -> (1, 0, 0)), 0., f64::MAX).unwrap().t, 1.);
}