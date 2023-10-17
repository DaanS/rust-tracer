use crate::config::Float;
use crate::ray::Ray;

pub struct HitRecord {
    pub t: Float
}

pub trait Hit {
    fn hit(&self, r: Ray, t_min: Float, t_max: Float) -> Option<HitRecord>;
}