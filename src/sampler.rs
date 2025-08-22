use crate::{config::Float, random::random_float};

pub trait PixelSample: Default {
    fn pixel_sample(&self, x: usize, y: usize) -> (Float, Float);
}

#[derive(Clone, Copy, Default)]
pub struct CenterSampler {}
impl PixelSample for CenterSampler {
    fn pixel_sample(&self, x: usize, y: usize) -> (Float, Float) {
        (x as Float + 0.5, y as Float + 0.5)
    }
}

#[derive(Clone, Copy, Default)]
pub struct SquareSampler{}
impl PixelSample for SquareSampler {
    fn pixel_sample(&self, x: usize, y: usize) -> (Float, Float) {
        (x as Float + random_float(), y as Float + random_float())
    }
}