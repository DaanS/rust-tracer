use image::{save_buffer, ColorType};

pub struct Png { }

impl Png {
    pub fn write(width: usize, height: usize, pix: Vec<u8>, path_str: &str) {
        save_buffer(path_str, &pix, width as u32, height as u32, ColorType::Rgb8).unwrap();
    }
}