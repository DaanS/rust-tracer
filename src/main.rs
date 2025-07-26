#[macro_use]
mod vec3;
#[macro_use]
mod ray;
mod camera;
mod color;
mod film;
mod hit;
mod integrator;
mod material;
mod ppm;
mod png;
mod scene;
mod util;
mod random;
mod config;
mod sampler;
mod window;
mod conversion;

use config::Float;

/// le current todos

use crate::{config::{Film, Sampler}, conversion::color_gamma, integrator::Integrator, png::Png, ppm::Ppm, scene::random_scene};

fn variance_stats(film: &Film) {
    let mut vals: Vec<Float> = film.pix.iter().map(|sc| sc.avg_variance().r).collect();
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap());

    println!("");
    println!(" === variance stats === ");
    println!("min: {}", vals[0]);
    println!("med: {}", vals[vals.len() / 2]);
    println!("avg: {}", vals.iter().sum::<Float>() / vals.len() as Float);
    println!("max: {}", vals[vals.len() - 1]);
}

fn main() {
    const WIDTH: usize = 800;
    const HEIGHT: usize = 450;
    const SAMPLES: usize = 129;
    let mut film = Film::new(WIDTH, HEIGHT);

    let sampler = Sampler{};
    let scene = random_scene(&film);
    // TODO find out good bounds for samples and variance targets
    Integrator::dispatch_tiled(&scene, sampler, &mut film, 32, SAMPLES, 0.004, 50, 50);
    //Integrator::dispatch(&scene, sampler, &mut film, 64, SAMPLES, 0.004);

    Ppm::write(WIDTH, HEIGHT, film.to_rgb8(|s| color_gamma(s.mean())), "out/out.ppm");
    Png::write(WIDTH, HEIGHT, film.to_rgb8(|s| color_gamma(s.mean())), "out/out.png");

    variance_stats(&film);
}
