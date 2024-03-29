use std::time::Duration;

use minifb::{Window, WindowOptions};

use crate::{config::{Color, Film}, conversion::color_to_u32, film::SampleCollector};

pub struct MinifbWindow {
    width: usize,
    height: usize,
    buffer: Vec<u32>,
    window: Window,
}

impl MinifbWindow {
    pub fn new(width: usize, height: usize) -> Self {
        let mut window = Window::new("BallMaker 9000", width, height, WindowOptions::default()).unwrap();
        window.limit_update_rate(Some(Duration::from_micros(16600)));
        MinifbWindow { width, height, buffer: vec![0; width * height], window }
    }

    pub fn update<ExtractFunc: Fn(&SampleCollector) -> Color>(& mut self, film: &Film, f: ExtractFunc) {
        for (idx, pixel) in film.pix.iter().enumerate() {
            self.buffer[idx] = color_to_u32(f(pixel));
        }
        self.window.update_with_buffer(&self.buffer, self.width, self.height).unwrap();
    }
}