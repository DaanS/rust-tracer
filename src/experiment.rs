use image::{ConvertColorOptions, ImageReader, Rgb, metadata::Cicp};

use crate::{color::ColorRgb, film::{SampleCollector, SamplingFilm}, png::Png, texture::load_texture};

mod conversion;
mod film;
mod random;
mod color;
mod config;
mod util;
mod vec3;
mod texture;
mod png;

fn component_gamma_u8(c: u8) -> u8 {
    const GAMMA: f64 = 2.2;
    ((c as f64 / 255.).powf(1. / GAMMA) * 255.) as u8
}

fn get_pixel_as_color(i: usize, pixels: &[u8]) -> ColorRgb {
    color::color_rgb(pixels[i + 0] as f64 / 255., pixels[i + 1] as f64 / 255., pixels[i + 2] as f64 / 255.)
}

fn variance_stats(film: &SamplingFilm) {
    let mut vals: Vec<f64> = film.pix.iter().map(|sc| sc.avg_variance().r).collect();
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap());

    println!();
    println!(" === variance stats === ");
    println!("min: {}", vals[0]);
    println!("med: {}", vals[vals.len() / 2]);
    println!("avg: {}", vals.iter().sum::<f64>() / vals.len() as f64);
    println!("max: {}", vals[vals.len() - 1]);
}

fn main() {
    let rgb_img = ImageReader::open("res/earthmap.jpg").unwrap().with_guessed_format().unwrap().decode().unwrap().into_rgb8();
    print!("loaded image with color space {:?}\n", rgb_img.color_space());
    rgb_img.save("out/earth-map-image.png").unwrap();

    let og_buffy = rgb_img.as_raw().clone();
    let gammied_buffy = rgb_img.to_color_space::<Rgb<u8>>(Cicp::SRGB_LINEAR, ConvertColorOptions::default()).unwrap().as_raw().clone().iter().map(|&v| component_gamma_u8(v)).collect::<Vec<u8>>();

    let mut film = SamplingFilm::new((rgb_img.width() as usize, rgb_img.height() as usize));
    for y in 0..rgb_img.height() as usize {
        for x in 0..rgb_img.width()  as usize{
            let i = (x + y * rgb_img.width() as usize) * 3;
            film.add_sample((x, y), get_pixel_as_color(i, &og_buffy));
            film.add_sample((x, y), get_pixel_as_color(i, &gammied_buffy));
        }
    }
    let base_path = "out/earth-map".to_string();
    Png::write(rgb_img.width() as usize, rgb_img.height() as usize, film.to_rgb8(SampleCollector::gamma_corrected_mean), &format!("{}-mean.png", base_path));
    Png::write(rgb_img.width() as usize, rgb_img.height() as usize, film.to_rgb8(SampleCollector::variance), &format!("{}-variance.png", base_path));
    Png::write(rgb_img.width() as usize, rgb_img.height() as usize, film.to_rgb8(SampleCollector::avg_variance), &format!("{}-avg-variance.png", base_path));

    variance_stats(&film);

    Png::write(rgb_img.width() as usize, rgb_img.height() as usize, og_buffy, "out/earth-map-mine.png");
    Png::write(rgb_img.width() as usize, rgb_img.height() as usize, gammied_buffy, "out/earth-map-mine-gamma.png");
}