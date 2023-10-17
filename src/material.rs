use crate::{config::Color, ray::Ray, hit::HitRecord};

pub struct ScatterRecord {
    pub attenuation: Color,
    pub out: Ray
}

pub trait Scatter {
    fn scatter(&self, ray_in: Ray, hit: HitRecord) -> Option<ScatterRecord>;
}

pub struct Lambertian {
    color: Color
}

impl Scatter for Lambertian {
    fn scatter(&self, ray_in: Ray, hit: HitRecord) -> Option<ScatterRecord> {
        todo!()
    }
}