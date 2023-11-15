use crate::{sphere::{Sphere, sphere}, ray::Ray, config::Color, color::color_rgb, material::lambertian};

pub struct Scene {
    pub objects: Vec<Sphere>,
    pub background_color: fn(Ray) -> Color
}

pub fn simple_scene() -> Scene {
    let center_sphere = sphere((0., 0., -1.).into(), 0.5, lambertian(0.7, 0.3, 0.3));
    let ground_sphere = sphere((0., -100.5, -1.).into(), 100., lambertian(1., 1., 0.));

    Scene { objects: vec![center_sphere, ground_sphere], background_color: |r| {
        let normalized_direction = r.direction.normalize();
        let a = (normalized_direction.y + 1.) * 0.5;
        (1. - a) * color_rgb(1., 1., 1.) + a * color_rgb(0.5, 0.7, 1.)
    }}
}