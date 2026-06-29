
use crate::{
    config::{Float, PI}, hit::{Bound, Hit, HitRecord, bvh::AABB}, material::simple::Material, ray::Ray, texture::UV, vec3::{Point, dot}
};

#[derive(Clone, Copy, Default)]
pub struct Sphere {
    pub center: Point,
    pub radius: Float,
    pub material: Material,
}

pub fn sphere(center: (Float, Float, Float), radius: Float, material: Material) -> Sphere {
    Sphere { center: center.into(), radius, material }
}

impl Hit for Sphere {
    fn hit(&self, r: Ray, t_min: Float, t_max: Float) -> Option<HitRecord> {
        let oc = r.origin - self.center;
        let a = r.direction.length_squared();
        let half_b = dot(oc, r.direction);
        let c = oc.length_squared() - self.radius * self.radius;

        let d = half_b * half_b - a * c;
        if d < 0. { return None; }

        let sqrt_d = d.sqrt();
        let mut root = (-half_b - sqrt_d) / a;
        if root < t_min || root > t_max {
            root = (-half_b + sqrt_d) / a;
            if root < t_min || root > t_max {
                return None;
            }
        }

        let pos = r.at(root);
        let normal = (pos - self.center) / self.radius;
        let uv = match self.material {
            Material::LambertianTexture { texture: _ } => self.uv(pos),
            _ => (0.0, 0.0),
        };

        Some(HitRecord { t: root, material: self.material, normal, pos, uv })
    }
}

impl UV for Sphere {
    fn uv(&self, pos: Point) -> (Float, Float) {
        let pos = (pos - self.center) / self.radius;
        let phi = pos.z.atan2(pos.x);
        let theta = pos.y.asin();
        (1. - (phi + PI) / (2. * PI), (theta + PI / 2.) / PI)
    }
}

impl Bound for Sphere {
    type HitType = AABB;
    fn bound(&self) -> AABB {
        AABB {
            x: (self.center.x - self.radius, self.center.x + self.radius).into(),
            y: (self.center.y - self.radius, self.center.y + self.radius).into(),
            z: (self.center.z - self.radius, self.center.z + self.radius).into(),
        }
    }
}

// I'd like to make this generic over Hit, but that runs into some conceptual issues if I want to support different types of bounding volumes.
// The way I'm doing this right now would require the bounding volume type to implement a default and enclosing method. This feels a bit much
// to put directly into Hit, and I'm also not sure I want to introduce a separate trait for bounding volumes.
impl Bound for Vec<Sphere> {
    type HitType = AABB;
    // TODO cache this probably (LazyCell or OnceCell maybe?)
    fn bound(&self) -> AABB {
        self.iter().fold(
            AABB::new((0., 0.).into(), (0., 0.).into(), (0., 0.).into()),
            |aabb, s| AABB::enclosing(aabb, s.bound())
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{color::color_rgb, config::Color, texture::load_image_linear};

    use super::*;
    use approx::assert_ulps_eq;

    #[test]
    fn test_hit_sphere() {
        let s = sphere((0., 0., 0.), 1., Material::None);

        assert!(s.hit(ray!((-10, 0, 0) -> (1., 0., 0.)), 0., Float::INFINITY).is_some());

        assert!(s.hit(ray!((-2, 0, 0) -> (1, 0, 0)), 0., Float::MAX).unwrap().t == 1.);
        assert!(s.hit(ray!((2, 0, 0) -> (-1, 0, 0)), 0., Float::MAX).unwrap().t == 1.);
        assert!(s.hit(ray!((0, 0, 0) -> (1, 0, 0)), 0., Float::MAX).unwrap().t == 1.);

        assert!(s.hit(ray!((-2, 0, 0) -> (1, 0, 0)), 0., Float::MAX).unwrap().normal == vec3!(-1, 0, 0));
        assert!(s.hit(ray!((2, 0, 0) -> (-1, 0, 0)), 0., Float::MAX).unwrap().normal == vec3!(1, 0, 0));
        assert!(s.hit(ray!((0, 0, 0) -> (1, 0, 0)), 0., Float::MAX).unwrap().normal == vec3!(1, 0, 0));
    }

    #[test]
    fn test_uv_sphere() {
        let s = sphere((0., 0., 0.), 1., Material::None);

        let pos = vec3!(1., 0., 0.);
        let (u, v) = s.uv(pos);
        assert_ulps_eq!(u, 0.5);
        assert_ulps_eq!(v, 0.5);

        let pos = vec3!(0., 1., 0.);
        let (u, v) = s.uv(pos);
        assert_ulps_eq!(u, 0.5);
        assert_ulps_eq!(v, 1.0);

        let pos = vec3!(0., 0., 1.);
        let (u, v) = s.uv(pos);
        assert_ulps_eq!(u, 0.25);
        assert_ulps_eq!(v, 0.5);

        let pos = vec3!(-1., 0., 0.);
        let (u, v) = s.uv(pos);
        assert_ulps_eq!(u, 0.0);
        assert_ulps_eq!(v, 0.5);

        let pos = vec3!(0., -1., 0.);
        let (u, v) = s.uv(pos);
        assert_ulps_eq!(u, 0.5);
        assert_ulps_eq!(v, 0.0);

        let pos = vec3!(0., 0., -1.);
        let (u, v) = s.uv(pos);
        assert_ulps_eq!(u, 0.75);
        assert_ulps_eq!(v, 0.5);
    }

    fn pixel_to_color(pixel: image::Rgb<u8>) -> Color {
        color_rgb(pixel[0] as Float / 255., pixel[1] as Float / 255., pixel[2] as Float / 255.)
    }

    #[test]
    fn test_textured_sphere() {
        let mut repo = crate::texture::TextureRepository::new();
        let idx = repo.load_texture("res/earthmap.jpg");
        let img = load_image_linear("res/earthmap.jpg");
        let s = sphere((0., 0., 0.), 1., crate::material::simple::lambertian_texture(idx));

        let pos = vec3!(-1., 0., 0.);
        let color = repo.texture_value(idx, s.uv(pos), pos);
        let color2 = pixel_to_color(*img.get_pixel(0, img.height() / 2));

        assert_eq!(color, color2);
    }

    #[test]
    fn test_bound_sphere() {
        let s = sphere((0., 0., 0.), 1., Material::None);
        let aabb = s.bound();

        assert_ulps_eq!(aabb.x.min, -1.);
        assert_ulps_eq!(aabb.x.max, 1.);
        assert_ulps_eq!(aabb.y.min, -1.);
        assert_ulps_eq!(aabb.y.max, 1.);
        assert_ulps_eq!(aabb.z.min, -1.);
        assert_ulps_eq!(aabb.z.max, 1.);
    }

    #[test]
    fn test_hit_vec() {
        let s1 = sphere((1., 0., 0.), 1., Material::None);
        let s2 = sphere((-1., 0., 0.), 1., Material::None);
        let v = vec![s1, s2];

        assert_eq!(v.hit(ray!((3, 0, 0) -> (-1, 0, 0)), 0., Float::MAX).unwrap().t, 1.);
        assert_eq!(v.hit(ray!((-3, 0, 0) -> (1, 0, 0)), 0., Float::MAX).unwrap().t, 1.);
    }

    #[test]
    fn test_bound_vec() {
        let s1 = sphere((1., 0., 0.), 1., Material::None);
        let s2 = sphere((-1., 0., 0.), 1., Material::None);

        let v = vec![s1, s2];
        let aabb = v.bound();

        assert_ulps_eq!(aabb.x.min, -2.);
        assert_ulps_eq!(aabb.x.max, 2.);
        assert_ulps_eq!(aabb.y.min, -1.);
        assert_ulps_eq!(aabb.y.max, 1.);
        assert_ulps_eq!(aabb.z.min, -1.);
        assert_ulps_eq!(aabb.z.max, 1.);

        let v_empty: Vec<Sphere> = vec![];
        let aabb_empty = v_empty.bound();
        assert!(aabb_empty.x.min.abs() < AABB::MIN_LENGTH);
        assert!(aabb_empty.x.max.abs() < AABB::MIN_LENGTH);
        assert!(aabb_empty.y.min.abs() < AABB::MIN_LENGTH);
        assert!(aabb_empty.y.max.abs() < AABB::MIN_LENGTH);
        assert!(aabb_empty.z.min.abs() < AABB::MIN_LENGTH);
        assert!(aabb_empty.z.max.abs() < AABB::MIN_LENGTH);
    }
}