use std::sync::Arc;

use crate::{camera::Camera, color::color_rgb, config::{Color, Film, Float}, hit::{bvh::{AxisAlignedBound, Bvh}, sphere::sphere, Hit}, material::simple::{dielectric, lambertian, metal}, random::{random_float, random_in_range}, ray::Ray, vec3::{vec3, Vec3}};

#[derive(Clone)]
pub struct Scene {
    pub objects: Arc<dyn Hit + Send + Sync>,
    pub background_color: fn(Ray) -> Color,
    pub cam: Camera
}

pub fn overcast_sky_background(r: Ray) -> Color {
    let normalized_direction = r.direction.normalize();
    let a = (normalized_direction.y + 1.) * 0.5;
    (1. - a) * color_rgb(1., 1., 1.) + a * color_rgb(0.5, 0.7, 1.)
}

// TODO camera sort of belongs to scene, since it's an important part of the composition
// should camera own the film?

pub fn simple_scene(film: &Film) -> Scene {
    let center_sphere = sphere((0., 0., -1.), 0.5, Arc::new(lambertian((0.7, 0.3, 0.3))));
    let left_sphere = sphere((-1., 0., -1.), 0.5, Arc::new(dielectric(1.5)));
    let right_sphere = sphere((1., 0., -1.), 0.5, Arc::new(metal((0.8, 0.6, 0.2), 0.1)));
    let ground_sphere = sphere((0., -100.5, -1.), 100., Arc::new(lambertian((1., 1., 0.))));

    let cam = Camera::new(&film, vec3!(0, 0, 0), vec3!(0, 0, -1), vec3!(0, 1, 0), 90., 1., 0.6);

    Scene { objects: Arc::new(vec![center_sphere, left_sphere, right_sphere, ground_sphere]), background_color: overcast_sky_background, cam }
}

pub fn random_scene(film: &Film) -> Scene {
    //for _i in 0..16 { random_float(); }
    for _i in 0..40315 { random_float(); }

    let ground_sphere = sphere((0., -1000., 0.), 1000., Arc::new(lambertian((1., 1., 0.))));
    let glass_sphere = sphere((0., 1., 0.), 1., Arc::new(dielectric(1.5)));
    let metal_sphere = sphere((4., 1., 0.), 1., Arc::new(metal((0.7, 0.6, 0.5), 0.)));
    let lamb_sphere = sphere((-4., 1., 0.), 1., Arc::new(lambertian((0.7, 0.3, 0.3))));
    let mut objects = vec![ground_sphere, glass_sphere, metal_sphere, lamb_sphere];

    for a in -11..11 {
        for b in -11..11 {
            let center = (a as Float + 0.9 * random_float(), 0.2, b as Float + 0.9 * random_float());

            if (Vec3::from(center) - vec3(4., 0.2, 0.)).length() > 0.9 {
                let mat_rng = random_float();
                if mat_rng < 0.8 {
                    objects.push(sphere(center, 0.2, Arc::new(lambertian(Color::random_in_range(0., 1.).into()))) );
                } else if mat_rng < 0.95 {
                    objects.push(sphere(center, 0.2, Arc::new(metal(Color::random_in_range(0.5, 1.0).into(), random_in_range(0., 0.2)))));
                } else {
                    objects.push(sphere(center, 0.2, Arc::new(dielectric(random_in_range(1.4, 1.6)))));
                }
            }
        }
    }

    let cam = Camera::new(film, vec3!(13, 2, 3), vec3!(0, 0, 0), vec3!(0, 1, 0), 20., 10., 0.6);

    let arced_objects = objects.iter().map(|o| Arc::new(o.clone()) as Arc<dyn AxisAlignedBound + Send + Sync>).collect::<Vec<_>>();
    Scene { objects: Arc::new(Bvh::from_slice(arced_objects.as_slice())), background_color: overcast_sky_background, cam }
}