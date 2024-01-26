use std::io::{stdout, Write};

use crate::{
    color::color_rgb,
    config::{Color, Film, Float},
    film::{PresampledFilm},
    hit::Hit,
    material::Scatter,
    ray::Ray,
    scene::Scene,
    Sampler,
};

fn print_progress(prog: Float) {
    const WIDTH: usize = 70;

    let pos = (prog * WIDTH as Float) as usize;

    let mut lock = stdout().lock();
    write!(lock, "[").unwrap();
    for _i in 0..pos { write!(lock, "=").unwrap(); }
    write!(lock, ">").unwrap();
    for _i in (pos + 1)..WIDTH { write!(lock, " ").unwrap(); }
    write!(lock, "] {}%\r", (prog * 100.) as usize).unwrap();
    stdout().flush().unwrap();
}

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
    sampler: Sampler,
}

impl<'a> Integrator<'a> {
    pub fn new(scene: &'a Scene, sampler: Sampler) -> Integrator<'a> {
        Integrator { scene, sampler, }
    }

    // TODO we could probably carry depth and attenuation info in the ray itself
    // maybe we could use this to make li non-recursive?
    // but then how do we return the final result to the dispatcher?
    // and would a larger ray struct negatively impact hit detection performance?
    fn li(&self, r: Ray, max_bounces: usize) -> Color {
        if max_bounces == 0 { return color_rgb(0., 0., 0.); }

        match self.scene.objects.hit(r.clone(), 0.001, Float::INFINITY) {
            Some(hit_record) => match hit_record.material.scatter(r.clone(), hit_record.clone()) {
                Some(scatter_record) => scatter_record.attenuation * self.li(scatter_record.out, max_bounces - 1),
                None => color_rgb(0., 0., 0.)
            },
            None => { (self.scene.background_color)(r) }
        }
    }

    // TODO passing desired sample count here is a temporary measure until we get variable sampling rates working
    pub fn dispatch(&self, film: &mut Film, samples: usize) {
        for x in 0..film.width {
            for y in 0..film.height {
                for _n in 0..samples {
                    let (s, t) = self.sampler.get_pixel_sample(x, y);
                    let r = self.scene.cam.get_ray(s, t);
                    film.add_sample(x, y, self.li(r, 8));
                }
                print_progress((x * film.height + y) as Float / (film.width * film.height) as Float);
            }
        }
    }
}
