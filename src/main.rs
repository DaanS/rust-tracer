#![allow(dead_code)]

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

/// le current todos

use crate::{config::{Film, Float}, film::SampleCollector, integrator::{Integrate, MultiCoreTiledIntegrator}, png::Png, ppm::Ppm, scene::random_scene};

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
    const MAX_SAMPLES: usize = 64;

    type Sampler = sampler::SquareSampler;
    type Evaluator = integrator::SimpleRayEvaluator;

    let mut film = Film::new(WIDTH, HEIGHT);

    let scene = random_scene(&film);
    MultiCoreTiledIntegrator::<Sampler, Evaluator, 50, 50, 8>::integrate(&scene, &mut film, 32, MAX_SAMPLES, 0.004);

    Ppm::write(WIDTH, HEIGHT, film.to_rgb8(SampleCollector::gamma_corrected_mean), "out/out.ppm");

    let base_path = format!("out/out-{}x{}@{}", WIDTH, HEIGHT, MAX_SAMPLES);
    Png::write(WIDTH, HEIGHT, film.to_rgb8(SampleCollector::gamma_corrected_mean), &format!("{}-mean.png", base_path));
    Png::write(WIDTH, HEIGHT, film.to_rgb8(SampleCollector::variance), &format!("{}-variance.png", base_path));
    Png::write(WIDTH, HEIGHT, film.to_rgb8(SampleCollector::avg_variance), &format!("{}-avg-variance.png", base_path));

    variance_stats(&film);
}
