
use crate::{config::{Color, Float}, util::clamp};

// maybe move this elsewhere?
pub fn map_color_component(comp: Float) -> u8 {
    const GAMMA: Float = 2.;
    (clamp(comp, 0., 1.).powf(1. / GAMMA) * 255 as Float) as u8
}

pub fn component_gamma(comp: Float) -> Float {
    const GAMMA: Float = 2.;
    comp.powf(1. / GAMMA)
}

pub fn color_component_to_u8(comp: Float) -> u8 {
    (clamp(comp, 0., 1.) * 255 as Float) as u8
}

pub fn color_gamma(color: Color) -> Color {
    (component_gamma(color.r), component_gamma(color.g), component_gamma(color.b)).into()
}

pub fn color_to_u32(c: Color) -> u32 {
    (color_component_to_u8(c.r) as u32) << 16 | (color_component_to_u8(c.g) as u32) << 8 | (color_component_to_u8(c.b) as u32)
}