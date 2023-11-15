use crate::{config::Float, random::random_float};

pub trait Sampler {
    fn get_pixel_sample(&self, x: usize, y: usize) -> (Float, Float);
}

pub struct CenterSampler {}
impl Sampler for CenterSampler {
    fn get_pixel_sample(&self, x: usize, y: usize) -> (Float, Float) {
        (x as Float + 0.5, y as Float + 0.5)
    }
}

pub struct SquareSampler{}
impl Sampler for SquareSampler {
    fn get_pixel_sample(&self, x: usize, y: usize) -> (Float, Float) {
        (x as Float + random_float(), y as Float + random_float())
    }
}