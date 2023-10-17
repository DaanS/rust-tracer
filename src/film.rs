use crate::config::{Color, Float};

use crate::util::clamp;

// maybe move this elsewhere?
pub fn map_color_component(comp: Float) -> u8 {
    (clamp(comp, 0., 1.) * 255 as Float) as u8
}

pub struct Film {
    pub width: usize,
    pub height: usize,
    pub pix: Vec<Color>
}

impl Film {
    pub fn new(width: usize, height: usize) -> Self {
        Film{width, height, pix: vec![Color::default(); width * height]}
    }

    pub fn write_pixel(&mut self, x: usize, y: usize, col: Color) {
        self.pix[x + y * self.width] = col;
    }

    pub fn to_rgb8(&self) -> Vec<u8> {
        let mut v = vec![0; self.width * self.height * 3];
        for (i, col) in self.pix.iter().enumerate() {
            let c = col.to_rgb();
            v[i * 3] = map_color_component(c.r);
            v[i * 3 + 1] = map_color_component(c.g);
            v[i * 3 + 2] = map_color_component(c.b);
        }
        v
    }
}

#[test]
fn test() {
    let mut f = Film::new(2, 3);
    f.write_pixel(0, 1, (0., 0.5, 1.).into());
    let v = f.to_rgb8();

    assert_eq!(v.len(), f.width * f.height * 3);
    for x in 0..f.width {
        for y in 0..f.height {
            for c in 0..3 {
                assert_eq!(v[(x + y * f.width) * 3 + c], 
                    if x == 0 && y == 1 && c == 1 { 127 }
                    else if x == 0 && y == 1 && c == 2 { 255 }
                    else { 0 }
                );
            }
        }
    }
}