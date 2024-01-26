use crate::{config::Float, random::random_float};

pub struct CenterSampler {}
impl CenterSampler {
    pub fn get_pixel_sample(&self, x: usize, y: usize) -> (Float, Float) {
        (x as Float + 0.5, y as Float + 0.5)
    }
}

pub struct SquareSampler{}
impl SquareSampler {
    pub fn get_pixel_sample(&self, x: usize, y: usize) -> (Float, Float) {
        (x as Float + random_float(), y as Float + random_float())
    }
}