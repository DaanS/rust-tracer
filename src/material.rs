use crate::{config::{Color, Float}, ray::Ray, hit::HitRecord};

pub struct ScatterRecord {
    pub attenuation: Color,
    pub out: Ray
}

pub trait Scatter {
    fn scatter(&self, ray_in: Ray, hit: HitRecord) -> Option<ScatterRecord>;
}

#[derive(Clone)]
pub enum Material {
    None,
    Lambertian { color: Color },
    Metal { color: Color, roughness: Float },
    Dielectric { ir: Float }
}

pub fn lambertian(r: Float, g: Float, b: Float) -> Material { Material::Lambertian { color: (r, g, b).into() }}

impl Scatter for Material {
    fn scatter(&self, ray_in: Ray, hit: HitRecord) -> Option<ScatterRecord> {
        match self {
            Material::Lambertian{color} => { 
                Some(ScatterRecord { attenuation: color.clone(), out: Ray::default() }) 
            },
            Material::Metal{color, roughness} => {
                None
            },
            Material::Dielectric{ir} => {
                None
            },
            Material::None => { None }
        }
    }
}