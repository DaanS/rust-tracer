use crate::{film::Film, scene::Scene, camera::Camera, sampler::Sampler, config::{Color, Float}, ray::Ray, color::color_rgb, hit::Hit};

/// integrator concepts
/// 
/// for pixel based approaches:
/// - dispatch parts of image (e.g. tiles) -> Film
/// - generate samples for pixels -> Sampler
/// - turn samples into rays -> Camera
/// - calculate radiance contribution for rays
///

pub struct Integrator<'a> {
    scene: &'a Scene,
    cam: &'a Camera,
    sampler: &'a dyn Sampler
}

impl<'a> Integrator<'a> {
    pub fn new(scene: &'a Scene, cam: &'a Camera, sampler: &'a dyn Sampler) -> Integrator<'a> {
        Integrator { scene, cam, sampler }
    }

    fn li(&self, r: Ray) -> Color {
        match self.scene.objects[0].hit(r.clone(), 0., Float::INFINITY) {
            Some(hit_record) => {
                (1., 0., 0.).into()
            },
            None => {
                let normalized_direction = r.direction.normalize();
                let a = normalized_direction.y * 0.5 + 1.;
                (1. - a) * color_rgb(1., 1., 1.) + a * color_rgb(0.5, 0.7, 1.)
            }
        }
    }

    pub fn dispatch(&self, film: &mut Film) {
        for x in 0..film.width {
            for y in 0..film.height {
                let (s, t) = self.sampler.get_pixel_sample(x, y);
                let r = self.cam.get_ray(s, t);
                let col = self.li(r);
                film.write_pixel(x, y, col);
            } 
        }
    }
}