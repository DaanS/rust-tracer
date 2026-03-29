use std::collections::HashMap;

use crate::{color::color_rgb, config::{Color, Float}, util::clamp, vec3::Point};
use image::{ColorType, ConvertColorOptions, ImageReader, RgbImage, metadata::Cicp};

pub trait UV {
    fn uv(&self, pos: Point) -> (Float, Float);
}

pub trait TextureValue {
    fn value(&self, uv: (Float, Float), pos: Point) -> Color;
}

impl TextureValue for RgbImage {
    fn value(&self, uv: (Float, Float), _pos: Point) -> Color {
        let (u, v) = uv;
        let x = (clamp(u, (0., 1.)) * (self.width() - 1) as Float) as u32;
        let y = ((1. - clamp(v, (0., 1.))) * (self.height() - 1) as Float) as u32;
        let pixel = self.get_pixel(x, y);
        color_rgb(pixel[0] as Float / 255., pixel[1] as Float / 255., pixel[2] as Float / 255.)
    }
}

pub type TextureHandle = usize;
pub struct TextureRepository {
    textures: Vec<RgbImage>,
    path_indices: HashMap<String, TextureHandle>,
}

impl TextureRepository {
    pub fn new() -> Self {
        TextureRepository { textures: Vec::new(), path_indices: HashMap::new() }
    }

    pub fn load_texture(&mut self, path_str: &str) -> TextureHandle {
        if let Some(&index) = self.path_indices.get(path_str) {
            index
        } else {
            let index = self.textures.len();
            self.textures.push(load_image_linear(path_str));
            self.path_indices.insert(path_str.to_string(), index);
            index
        }
    }

    pub fn texture_value(&self, index: TextureHandle, uv: (Float, Float), pos: Point) -> Color {
        self.textures[index].value(uv, pos)
    }
}

pub fn load_image_linear(path_str: &str) -> RgbImage {
    let mut dyn_img = ImageReader::open(path_str).unwrap().with_guessed_format().unwrap().decode().unwrap();
    dyn_img.convert_color_space(Cicp::SRGB_LINEAR, ConvertColorOptions::default(), ColorType::Rgb8).unwrap();
    dyn_img.into_rgb8()
}

#[test]
fn test_texture_repository() {
    let mut repo = TextureRepository::new();
    let idx1 = repo.load_texture("res/earthmap.jpg");
    let idx2 = repo.load_texture("res/earthmap.jpg");
    assert_eq!(idx1, idx2);

    let color1 = repo.texture_value(idx1, (0., 0.), vec3!(0., 0., 0.));
    let color2 = repo.texture_value(idx2, (0., 0.), vec3!(0., 0., 0.));
    assert_eq!(color1, color2);

    let color3 = repo.texture_value(idx1, (0.5, 0.5), vec3!(0., 0., 0.));
    assert_ne!(color1, color3);
}