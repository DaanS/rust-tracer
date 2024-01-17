#[macro_use]
mod vec3;
#[macro_use]
mod ray;
mod camera;
mod color;
mod config;
mod film;
mod hit;
mod integrator;
mod material;
mod ppm;
mod png;
mod sampler;
mod scene;
mod sphere;
mod util;
mod random;

/// le current todos

use crate::{
    film::Film, integrator::Integrator, ppm::Ppm, sampler::SquareSampler, scene::simple_scene, png::Png,
};

fn main() {
    const WIDTH: usize = 800;
    const HEIGHT: usize = 450;
    const SAMPLES: usize = 64;
    let mut film = Film::new(WIDTH, HEIGHT, SAMPLES);

    let sampler = SquareSampler{};
    let scene = simple_scene(&film);
    let integrator = Integrator::new(&scene, &sampler);
    integrator.dispatch(&mut film);

    Ppm::write(WIDTH, HEIGHT, film.to_rgb8(), "out/out.ppm");
    Png::write(WIDTH, HEIGHT, film.to_rgb8(), "out/out.png");
}
