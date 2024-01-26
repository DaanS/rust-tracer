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
mod sphere;
mod util;
mod random;
mod config;
mod sampler;

/// le current todos

use crate::{
    config::{Film, Sampler}, integrator::Integrator, png::Png, ppm::Ppm, scene::random_scene
};

fn main() {
    const WIDTH: usize = 400;
    const HEIGHT: usize = 225;
    const SAMPLES: usize = 64;
    let mut film = Film::new(WIDTH, HEIGHT);

    let sampler = Sampler{};
    let scene = random_scene(&film);
    let integrator = Integrator::new(&scene, sampler);
    integrator.dispatch(&mut film, SAMPLES);

    Ppm::write(WIDTH, HEIGHT, film.to_rgb8(), "out/out.ppm");
    Png::write(WIDTH, HEIGHT, film.to_rgb8(), "out/out.png");
}
