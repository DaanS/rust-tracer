use std::{io::{stdout, Write}};

use crate::{
    color::color_rgb, config::{Color, Film, Float}, film::SampleCollector, hit::Hit, material::Scatter, png::Png, ray::Ray, scene::Scene, util::is_power_of_2, window::MinifbWindow, Sampler
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

struct RenderRegionJob<'a> {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    min_samples: usize,
    max_samples: usize,
    variance_target: Float,
    scene: &'a Scene,
    sampler: Sampler,
    film: Film
}

/// integrator concepts
///
/// for pixel based approaches:
/// - dispatch parts of image (e.g. tiles) -> Film
/// - generate samples for pixels -> Sampler
/// - turn samples into rays -> Camera
/// - calculate radiance contribution for rays
///

pub struct Integrator { }

impl Integrator {
    // TODO we could probably carry depth and attenuation info in the ray itself
    // maybe we could use this to make li non-recursive?
    // but then how do we return the final result to the dispatcher?
    // and would a larger ray struct negatively impact hit detection performance?
    fn li(scene: &Scene, r: Ray, max_bounces: usize) -> Color {
        if max_bounces == 0 { return color_rgb(0., 0., 0.); }

        match scene.objects.hit(r.clone(), 0.001, Float::INFINITY) {
            Some(hit_record) => match hit_record.material.scatter(r.clone(), hit_record.clone()) {
                Some(scatter_record) => scatter_record.attenuation * Self::li(scene, scatter_record.out, max_bounces - 1),
                None => color_rgb(0., 0., 0.)
            },
            None => { (scene.background_color)(r) }
        }
    }

    pub fn dispatch_tile(scene: &Scene, sampler: Sampler, film: &mut Film, tile_x: usize, tile_y: usize, min_samples: usize, max_samples: usize, variance_target: Float) -> usize {
        let mut sample_count = 0;
        for y in 0..film.height {
            for x in 0..film.width {
                for n in 0..max_samples {
                    if n >= min_samples && film.sample_collector(x, y).max_variance() <= variance_target { break; }

                    sample_count += 1;
                    let (s, t) = sampler.get_pixel_sample(x + tile_x * film.width, y + tile_y * film.height);
                    let r = scene.cam.get_ray(s, t);
                    film.add_sample(x, y, Self::li(scene, r, 8));
                }
            }
        }
        sample_count
    }

    pub fn dispatch_tiled(scene: &Scene, sampler: Sampler, film: &mut Film, min_samples: usize, max_samples: usize, variance_target: Float, tile_width: usize, tile_height: usize) {
        let mut win = MinifbWindow::new(film.width, film.height);
        let mut win2 = MinifbWindow::new(film.width, film.height);
        let mut win3 = MinifbWindow::new(film.width, film.height);

        let tiles_hor = film.width / tile_width + 1.min(film.width % tile_width);
        let tiles_ver = film.height / tile_height + 1.min(film.height % tile_height);

        let mut sample_count = 0;

        for x in 0..tiles_hor {
            for y in 0..tiles_ver {
                let mut tile_film = Film::new(tile_width, tile_height);
                sample_count += Self::dispatch_tile(scene, sampler, &mut tile_film, x, y, min_samples, max_samples, variance_target);
                film.overwrite_with(x * tile_width, y * tile_height, &tile_film);

                win.update(&film, SampleCollector::gamma_corrected_mean);
                win2.update(&film, SampleCollector::variance);
                win3.update(&film, SampleCollector::avg_variance);

                print_progress((y + x * tiles_ver) as Float / (tiles_hor * tiles_ver) as Float);
            }
        }

        println!("");
        println!("{} samples collected, {:.2}%", sample_count, sample_count as Float * 100. / (film.width * film.height * max_samples) as Float);
    }

    // TODO might be neater to factor out the window updates and progress printing
    pub fn dispatch(scene: &Scene, sampler: Sampler, film: &mut Film, min_samples: usize, max_samples: usize, variance_target: Float) {
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
                    if n < min_samples || film.sample_collector(x, y).max_variance() > variance_target {
                        sample_count += 1;
                        let (s, t) = sampler.get_pixel_sample(x, y);
                        let r = scene.cam.get_ray(s, t);
                        film.add_sample(x, y, Self::li(scene, r, 8));
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
