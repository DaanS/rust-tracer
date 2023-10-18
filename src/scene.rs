use crate::{sphere::{Sphere, sphere}, ray::Ray, config::Color, color::color_rgb, material::lambertian};

pub struct Scene {
    pub objects: Vec<Sphere>,
    pub background_color: fn(Ray) -> Color
}

pub fn simple_scene() -> Scene {
    let s = sphere((0., 0., -2.).into(), 0.5, lambertian(1., 0., 0.));
    Scene { objects: vec![s], background_color: |r| {
        let normalized_direction = r.direction.normalize();
        let a = normalized_direction.y * 0.5 + 1.;
        (1. - a) * color_rgb(1., 1., 1.) + a * color_rgb(0.5, 0.7, 1.)
    }}
}