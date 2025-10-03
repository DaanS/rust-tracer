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

pub trait Bound {
    type HitType: Hit;
    fn bound(&self) -> Self::HitType;
}