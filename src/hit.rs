use crate::config::Float;
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::{Vec3, Point};

pub mod sphere;
pub mod bvh;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct HitRecord {
    pub t: Float,
    pub material: Material,
    pub normal: Vec3,
    pub pos: Point
}

pub trait Hit {
    fn hit(&self, r: Ray, t_min: Float, t_max: Float) -> Option<HitRecord>;
}

pub trait Bound {
    type HitType: Hit;
    fn bound(&self) -> Self::HitType;
}