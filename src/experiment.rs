use image::{ConvertColorOptions, ImageReader, Rgb, metadata::Cicp};

use crate::{color::ColorRgb, film::{SampleCollector, SamplingFilm}, png::Png};

mod conversion;
mod film;
mod random;
mod color;
mod config;
mod util;
mod vec3;
mod texture;
mod png;

pub trait Ploop<const Z: usize> {
    fn ploop(&self, x: usize, y: usize) -> String;
}

struct Plooper {}

impl<const Z: usize> Ploop<Z> for Plooper {
    fn ploop(&self, x: usize, y: usize) -> String {
        format!("Ploop at {}, {}, {}", x, y, Z)
    }
}

fn main() {
    let pl = Plooper {};
    println!("{}", pl.ploop(3, 4));
}