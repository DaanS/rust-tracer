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
mod sampler;
mod scene;
mod sphere;
mod util;
mod random;

/// le current todos
/// - implement scattering

use std::fmt::Write;

use crate::{
    camera::Camera, film::Film, integrator::Integrator, ppm::Ppm, sampler::SquareSampler, scene::simple_scene,
};

fn main() {
    const WIDTH: usize = 900;
    const HEIGHT: usize = 600;
    const SAMPLES: usize = 256;
    let mut film = Film::new(WIDTH, HEIGHT, SAMPLES);

    let cam = Camera::new(&film, vec3!(0, 0, 0), vec3!(0, 0, -1));
    let sampler = SquareSampler{};
    let scene = simple_scene();
    let integrator = Integrator::new(&scene, &cam, &sampler);
    integrator.dispatch(&mut film);

    let mut buf = String::new();
    write!(buf, "{}", Ppm::new(WIDTH, HEIGHT, &film.to_rgb8())).unwrap();
    std::fs::create_dir_all("out").unwrap();
    std::fs::write("out/out.ppm", buf).unwrap();
}
