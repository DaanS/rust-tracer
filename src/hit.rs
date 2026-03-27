use crate::{config::Float, material::simple::Material, ray::Ray, vec3::{Point, Vec3}};

pub mod sphere;
pub mod bvh;
pub mod instance;

#[derive(PartialEq, Debug)]
pub struct HitRecord {
    pub t: Float,
    pub material: Material,
    pub normal: Vec3,
    pub pos: Point
}

pub trait Hit {
    fn hit(&self, r: Ray, t_min: Float, t_max: Float) -> Option<HitRecord>;
}

impl<T: Hit> Hit for Vec<T> {
    fn hit(&self, r: Ray, t_min: Float, t_max: Float) -> Option<HitRecord> {
        let mut res = None;
        let mut cur_t_max = t_max;

        for s in self {
            if let Some(hit) = s.hit(r, t_min, cur_t_max) {
                cur_t_max = hit.t;
                res = Some(hit);
            }
        }

        res
    }
}

pub trait Bound {
    type HitType: Hit;
    fn bound(&self) -> Self::HitType;
}