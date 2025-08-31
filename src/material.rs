use crate::{config::Color, ray::Ray, vec3::{Point, Vec3}};

pub mod simple;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ScatterRecord {
    pub attenuation: Color,
    pub out: Ray
}

pub trait Scatter {
    fn scatter(&self, ray_in: Ray, pos: Point, normal: Vec3) -> Option<ScatterRecord>;
}