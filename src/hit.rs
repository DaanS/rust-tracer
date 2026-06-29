use crate::{config::Float, material::simple::Material, ray::Ray, vec3::{Point, Vec3}};

pub mod sphere;
pub mod quad;
pub mod bvh;
pub mod instance;

#[derive(PartialEq, Debug)]
pub struct HitRecord {
    pub t: Float,
    pub material: Material,
    pub normal: Vec3,
    pub pos: Point,
    pub uv: (Float, Float),
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

// TODO originally this was intended to capture the idea of 'something that can produce a bounding volume',
// with the idea that a bounding volume could be anything that implements Hit.
// But it turned out that, at least inside Bvh, Hit was inefficient, since it requires producing a full HitRecord.
// Bvh was really only interested in boolean intersection checks for its AABBs.
pub trait Bound {
    type HitType;
    fn bound(&self) -> Self::HitType;
}