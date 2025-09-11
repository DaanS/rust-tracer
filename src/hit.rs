use std::{rc::Rc, sync::Arc};

use crate::{config::Float, material::{simple::Material, Scatter}, ray::Ray, vec3::{Point, Vec3}};

pub mod sphere;
pub mod bvh;

pub struct HitRecord {
    pub t: Float,
    pub material: Arc<dyn Scatter>,
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