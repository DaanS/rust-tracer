use std::io::{stdout, Write};

use crate::{
    color::color_rgb,
    config::{Color, Float},
    film::Film,
    hit::Hit,
    material::Scatter,
    ray::Ray,
    sampler::Sampler,
    scene::Scene,
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

// TODO we can probably get rid of the dyn for Sampler and turn it into a compile time configuration option
pub struct Integrator<'a> {
    scene: &'a Scene,
    sampler: &'a dyn Sampler,
}

impl<'a> Integrator<'a> {
    pub fn new(scene: &'a Scene, sampler: &'a dyn Sampler) -> Integrator<'a> {
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

    pub fn dispatch(&self, film: &mut Film) {
        for x in 0..film.width {
            for y in 0..film.height {
                let mut col = color_rgb(0., 0., 0.);
                for _n in 0..film.samples {
                    let (s, t) = self.sampler.get_pixel_sample(x, y);
                    let r = self.scene.cam.get_ray(s, t);
                    col = col + self.li(r, 8);
                }
                film.add_sample(x, y, col);
                print_progress((x * film.height + y) as Float / (film.width * film.height) as Float);
            }
        }
    }
}
