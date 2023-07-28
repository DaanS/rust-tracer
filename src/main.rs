mod vec3;
mod ppm;
mod config;
mod color;
mod film;
mod hit;
mod util;
mod ray;

use std::fmt::{Write};
use std::fs::{write, create_dir_all};

use vec3::{Point, Vec3};
use film::Film;
use config::Float;
use ppm::Ppm;

fn main() {
    const WIDTH: usize = 300;
    const HEIGHT: usize = 200;
    let mut f = Film::new(WIDTH, HEIGHT);
    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            f.write_pixel(x, y, (0.0 + (x as Float / WIDTH as Float), 1.0 - (y as Float / HEIGHT as Float), 1.0).into())
        }
    }
    let mut buf = String::new();
    write!(buf, "{}", Ppm::new(WIDTH, HEIGHT, &f.to_rgb8())).unwrap();
    std::fs::create_dir_all("out");
    std::fs::write("out/out.ppm", buf).unwrap();
}
