
use crate::{
    config::Float, hit::{bvh::AABB, Bound, Hit, HitRecord}, material::simple::Material, ray::Ray, vec3::{dot, Point}
};

#[derive(Clone, Copy)]
pub struct Sphere {
    pub center: Point,
    pub radius: Float,
    pub material: Material,
}

pub fn sphere(center: (Float, Float, Float), radius: Float, material: Material) -> Sphere { 
    Sphere { center: center.into(), radius, material } 
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

        Some(HitRecord { t: root, material: self.material, normal, pos }) 
    }
}

impl Bound for Sphere {
    type HitType = AABB;
    fn bound(&self) -> AABB {
        AABB {
            x: (self.center.x - self.radius, self.center.x + self.radius).into(),
            y: (self.center.y - self.radius, self.center.y + self.radius).into(),
            z: (self.center.z - self.radius, self.center.z + self.radius).into(),
        }
    }
}

// TODO put this somewhere sensible when we get more shapes
impl Hit for Vec<Sphere> {
    fn hit(&self, r: Ray, t_min: Float, t_max: Float) -> Option<HitRecord> {
        let mut res = None;
        let mut cur_t_max = t_max;

        for s in self {
            if let Some(hit_record) = s.hit(r, t_min, cur_t_max) {
                cur_t_max = hit_record.t;
                res = Some(hit_record);
            }
        }

        res
    }
}

impl Bound for Vec<Sphere> {
    type HitType = AABB;
    // TODO cache this probably (LazyCell or OnceCell maybe?)
    // TODO also move this somewhere sensible along with the Hit impl
    fn bound(&self) -> AABB {
        self.iter().fold(
            AABB::new((0., 0.).into(), (0., 0.).into(), (0., 0.).into()),
            |aabb, s| AABB::enclosing(aabb, s.bound())
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_ulps_eq;

    #[test]
    fn test_hit_sphere() {
        let s = sphere((0., 0., 0.), 1., Material::None);

        assert!(s.hit(ray!((-10, 0, 0) -> (1., 0., 0.)), 0., f64::INFINITY).is_some());

        assert!(s.hit(ray!((-2, 0, 0) -> (1, 0, 0)), 0., f64::MAX).unwrap().t == 1.);
        assert!(s.hit(ray!((2, 0, 0) -> (-1, 0, 0)), 0., f64::MAX).unwrap().t == 1.);
        assert!(s.hit(ray!((0, 0, 0) -> (1, 0, 0)), 0., f64::MAX).unwrap().t == 1.);

        assert!(s.hit(ray!((-2, 0, 0) -> (1, 0, 0)), 0., f64::MAX).unwrap().normal == vec3!(-1, 0, 0));
        assert!(s.hit(ray!((2, 0, 0) -> (-1, 0, 0)), 0., f64::MAX).unwrap().normal == vec3!(1, 0, 0));
        assert!(s.hit(ray!((0, 0, 0) -> (1, 0, 0)), 0., f64::MAX).unwrap().normal == vec3!(1, 0, 0));
    }

    #[test]
    fn test_bound_sphere() {
        let s = sphere((0., 0., 0.), 1., Material::None);
        let aabb = s.bound();

        assert_ulps_eq!(aabb.x.min, -1.);
        assert_ulps_eq!(aabb.x.max, 1.);
        assert_ulps_eq!(aabb.y.min, -1.);
        assert_ulps_eq!(aabb.y.max, 1.);
        assert_ulps_eq!(aabb.z.min, -1.);
        assert_ulps_eq!(aabb.z.max, 1.);
    }

    #[test]
    fn test_hit_vec() {
        let s1 = sphere((1., 0., 0.), 1., Material::None);
        let s2 = sphere((-1., 0., 0.), 1., Material::None);
        let v = vec![s1, s2];

        assert_eq!(v.hit(ray!((3, 0, 0) -> (-1, 0, 0)), 0., f64::MAX).unwrap().t, 1.);
        assert_eq!(v.hit(ray!((-3, 0, 0) -> (1, 0, 0)), 0., f64::MAX).unwrap().t, 1.);
    }

    #[test]
    fn test_bound_vec() {
        let s1 = sphere((1., 0., 0.), 1., Material::None);
        let s2 = sphere((-1., 0., 0.), 1., Material::None);

        let v = vec![s1, s2];
        let aabb = v.bound();

        assert_ulps_eq!(aabb.x.min, -2.);
        assert_ulps_eq!(aabb.x.max, 2.);
        assert_ulps_eq!(aabb.y.min, -1.);
        assert_ulps_eq!(aabb.y.max, 1.);
        assert_ulps_eq!(aabb.z.min, -1.);
        assert_ulps_eq!(aabb.z.max, 1.);

        let v_empty: Vec<Sphere> = vec![];
        let aabb_empty = v_empty.bound();
        assert_ulps_eq!(aabb_empty.x.min, 0.);
        assert_ulps_eq!(aabb_empty.x.max, 0.);
        assert_ulps_eq!(aabb_empty.y.min, 0.);
        assert_ulps_eq!(aabb_empty.y.max, 0.);
        assert_ulps_eq!(aabb_empty.z.min, 0.);
        assert_ulps_eq!(aabb_empty.z.max, 0.);
    }
}