use crate::config::Float;

pub trait Sampler {
    fn get_pixel_sample(&self, x: usize, y: usize) -> (Float, Float);
}

pub struct CenterSampler {}
impl Sampler for CenterSampler {
    fn get_pixel_sample(&self, x: usize, y: usize) -> (Float, Float) {
        (x as Float + 0.5, y as Float + 0.5)
    }
}