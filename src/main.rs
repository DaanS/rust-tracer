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

/// le current todos
/// - implement scattering

use std::fmt::Write;

use crate::{
    camera::Camera, film::Film, integrator::Integrator, ppm::Ppm, sampler::CenterSampler, scene::simple_scene,
};

fn main() {
    const WIDTH: usize = 300;
    const HEIGHT: usize = 200;
    let mut film = Film::new(WIDTH, HEIGHT);

    let cam = Camera::new(&film, vec3!(0, 0, 0), vec3!(0, 0, -1));
    let sampler = CenterSampler {};
    let scene = simple_scene();
    let integrator = Integrator::new(&scene, &cam, &sampler);
    integrator.dispatch(&mut film);

    let mut buf = String::new();
    write!(buf, "{}", Ppm::new(WIDTH, HEIGHT, &film.to_rgb8())).unwrap();
    std::fs::create_dir_all("out").unwrap();
    std::fs::write("out/out.ppm", buf).unwrap();
}
