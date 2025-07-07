use crate::{config::{Color, Float}, ray::{ray, Ray}, hit::HitRecord, vec3::{Vec3, dot, random_unit_vector}, random::random_float};

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

pub fn lambertian(color: (Float, Float, Float)) -> Material { Material::Lambertian { color: color.into() }}
pub fn metal(color: (Float, Float, Float), roughness: Float) -> Material { Material::Metal { color: color.into(), roughness }}
pub fn dielectric(ir: Float) -> Material { Material::Dielectric { ir }}

fn reflect(v: Vec3, n: Vec3) -> Vec3 {
    v - 2. * dot(v, n) * n
}

impl Scatter for Material {
    fn scatter(&self, ray_in: Ray, hit: HitRecord) -> Option<ScatterRecord> {
        match self {
            Material::Lambertian{color} => { 
                let out_dir = hit.normal + random_unit_vector();
                Some(ScatterRecord { 
                    attenuation: color.clone(), 
                    out: ray(hit.pos, if out_dir.near_zero() { hit.normal } else { out_dir }) 
                }) 
            },
            Material::Metal{color, roughness} => {
                let out_dir = reflect(ray_in.direction, hit.normal) + *roughness * random_unit_vector();
                if dot(out_dir, hit.normal) > 0. {
                    Some(ScatterRecord { attenuation: color.clone(), out: ray(hit.pos, out_dir) })
                } else {
                    None
                }
            },
            Material::Dielectric{ir} => {
                let inside = dot(ray_in.direction, hit.normal) > 0.;
                let normal = if inside { -hit.normal } else { hit.normal };

                let cos_theta = dot(-ray_in.direction.normalize(), normal);
                let sin_theta = (1. - cos_theta * cos_theta).sqrt();
                let refraction_ratio = if inside { *ir } else { 1. / *ir };

                let r0 = (1. - ir) / (1. + ir);
                let r02 = r0 * r0;
                let reflectance = r02 + (1. - r02) * (1. - cos_theta).powi(5);

                let out_dir = if refraction_ratio * sin_theta > 1. || reflectance > random_float() {
                    reflect(ray_in.direction, normal)
                } else {
                    let out_dir_perpendicular = refraction_ratio * (ray_in.direction.normalize() + cos_theta * normal);
                    let out_dir_parallel = -((1. - out_dir_perpendicular.length_squared()).abs().sqrt()) * normal;
                    out_dir_parallel + out_dir_perpendicular
                };
                Some(ScatterRecord { attenuation: (1., 1., 1.).into(), out: ray(hit.pos, out_dir) })
            },
            Material::None => { None }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{color::color_rgb, ray::Ray, hit::sphere::sphere, material::{Material, metal, dielectric, Scatter}, vec3::dot, hit::Hit};

    #[test]
    fn test_scatter_anti_normal() {
        let color = color_rgb(0.8, 0.6, 0.4);
        let mut s = sphere((0., 0., 0.), 0.5, Material::Lambertian { color });
        let r = ray!((-1, 0, 0) -> (1, 0, 0));
        let h = s.hit(r.clone(), 0.001, f64::INFINITY).unwrap();

        let scatter_rec = s.material.scatter(r.clone(), h.clone()).unwrap();
        assert_eq!(scatter_rec.attenuation, (0.8, 0.6, 0.4).into());
        assert!(dot(h.normal, scatter_rec.out.direction) > 0.);
        assert_eq!(scatter_rec.out.origin, h.pos);

        s.material = metal((0.8, 0.6, 0.4), 0.);
        let scatter_rec = s.material.scatter(r.clone(), h.clone()).unwrap();
        assert_eq!(scatter_rec.attenuation, (0.8, 0.6, 0.4).into());
        assert_eq!(scatter_rec.out.direction, -r.direction);
        assert_eq!(scatter_rec.out.origin, h.pos);

        s.material = dielectric(1.);
        let scatter_rec = s.material.scatter(r.clone(), h.clone()).unwrap();
        assert_eq!(scatter_rec.attenuation, (1., 1., 1.).into());
        assert_eq!(scatter_rec.out.direction, r.direction);
        assert_eq!(scatter_rec.out.origin, h.pos);
    }

    #[test]
    fn test_refraction() {
        let s = sphere((0., 0., 0.), 0.5, dielectric(1.3));

        let r = ray!((-1, 0, 0) -> (1, 0, 0));
        let h = s.hit(r.clone(), 0.001, f64::INFINITY).unwrap();
        let scatter_rec = s.material.scatter(r.clone(), h.clone()).unwrap();
        assert_eq!(scatter_rec.out.direction, r.direction);
        assert_eq!(scatter_rec.out.origin, h.pos);

        let r = ray!((-1.5, -1, 0) -> (1, 1, 0));
        let h = s.hit(r.clone(), 0.001, f64::INFINITY).unwrap();
        let scatter_rec = s.material.scatter(r.clone(), h.clone()).unwrap();
        assert!(dot(r.direction, scatter_rec.out.direction) > 0.);
        assert!(dot(h.normal, scatter_rec.out.direction) < 0.);
        assert!(dot(r.direction.normalize(), -h.normal) < dot(scatter_rec.out.direction, -h.normal));
        assert_eq!(scatter_rec.out.origin, h.pos);
    }
}