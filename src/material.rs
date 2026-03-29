use crate::{config::{Color, Float}, ray::Ray, scene::Scene, vec3::{Point, Vec3}};

pub mod simple;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ScatterRecord {
    pub attenuation: Color,
    pub out: Ray
}

pub trait Scatter {
    fn scatter(&self, scene: &Scene,ray_in: Ray, pos: Point, normal: Vec3, uv: (Float, Float)) -> Option<ScatterRecord>;
}