use std::io::{stdout, Write};

use crate::{
    color::color_rgb, config::{Color, Film, Float}, conversion::color_gamma, film::SampleCollector, hit::Hit, material::Scatter, png::Png, ray::Ray, scene::Scene, util::is_power_of_2, window::MinifbWindow, Sampler
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
    pub fn dispatch(&self, film: &mut Film, min_samples: usize, max_samples: usize) {
        let mut win = MinifbWindow::new(film.width, film.height);
        let mut win2 = MinifbWindow::new(film.width, film.height);
        let mut win3 = MinifbWindow::new(film.width, film.height);

        let mut sample_count = 0;

        for n in 0..max_samples {
            win.update(&film, SampleCollector::gamma_corrected_mean);
            win2.update(&film, SampleCollector::variance);
            win3.update(&film, SampleCollector::avg_variance);

            for x in 0..film.width {
                for y in 0..film.height {
                    if n < min_samples || film.sample_collector(x, y).max_variance() > 0.004 {
                        sample_count += 1;
                        let (s, t) = self.sampler.get_pixel_sample(x, y);
                        let r = self.scene.cam.get_ray(s, t);
                        film.add_sample(x, y, self.li(r, 8));
                    }
                }
                print_progress((n * film.width * film.height + x * film.height) as Float / (film.width * film.height * max_samples) as Float);
            }

            if (n == 0) || is_power_of_2(n) {
                Png::write(film.width, film.height, film.to_rgb8(SampleCollector::gamma_corrected_mean), format!("out/mean-{n}.png").as_str());
                Png::write(film.width, film.height, film.to_rgb8(SampleCollector::variance), format!("out/variance-{n}.png").as_str());
                Png::write(film.width, film.height, film.to_rgb8(SampleCollector::avg_variance), format!("out/avg_variance-{n}.png").as_str());
            }
        }

        println!("");
        println!("{} samples collected, {:.2}%", sample_count, sample_count as Float * 100. / (film.width * film.height * max_samples) as Float);
    }
}
