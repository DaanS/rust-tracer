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
mod texture;

use std::{fs::create_dir_all, time::Instant};

use crate::{config::{Film, Float}, integrator::{Integrate, MultiCoreTiledIntegrator, SimpleRayEvaluator}, sampler::SquareSampler, scene::random_scene};
#[cfg(not(feature = "bench"))]
use crate::{film::SampleCollector, png::Png, ppm::Ppm};

fn variance_stats(film: &Film) {
    let mut vals: Vec<Float> = film.pix.iter().map(|sc| sc.avg_variance().r).collect();
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap());

    println!();
    println!(" === variance stats === ");
    println!("min: {}", vals[0]);
    println!("med: {}", vals[vals.len() / 2]);
    println!("avg: {}", vals.iter().sum::<Float>() / vals.len() as Float);
    println!("max: {}", vals[vals.len() - 1]);
}

fn main() {
    let start = Instant::now();

    const WIDTH: usize = 800;
    const HEIGHT: usize = 450;
    const MIN_SAMPLES: usize = 32;
    const MAX_SAMPLES: usize = 64;

    create_dir_all("out/jobs").unwrap();

    let mut film = Film::new((WIDTH, HEIGHT));
    let scene = random_scene(&film);

    let init_dur = start.elapsed();
    let render_start = Instant::now();

    MultiCoreTiledIntegrator::<SquareSampler, SimpleRayEvaluator, 50, 50, 8, MIN_SAMPLES, MAX_SAMPLES>::integrate(&scene, &mut film, 0.004);

    let render_dur = render_start.elapsed();
    let post_start = Instant::now();

    #[cfg(not(feature = "bench"))]
    {
        Ppm::write(WIDTH, HEIGHT, film.to_rgb8(SampleCollector::gamma_corrected_mean), "out/out.ppm");

        let base_path = format!("out/out-{}x{}@{}", WIDTH, HEIGHT, MAX_SAMPLES);
        Png::write(WIDTH, HEIGHT, film.to_rgb8(SampleCollector::gamma_corrected_mean), &format!("{}-mean.png", base_path));
        Png::write(WIDTH, HEIGHT, film.to_rgb8(SampleCollector::variance), &format!("{}-variance.png", base_path));
        Png::write(WIDTH, HEIGHT, film.to_rgb8(SampleCollector::avg_variance), &format!("{}-avg-variance.png", base_path));
    }

    variance_stats(&film);

    println!();
    println!("Initialization took: {:?}", init_dur);
    println!("Rendering took: {:?}", render_dur);
    println!("Post took: {:?}", post_start.elapsed());
    println!("Everything took: {:?}", start.elapsed());
}
