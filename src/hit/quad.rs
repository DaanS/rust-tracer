use approx::assert_ulps_eq;

use crate::{config::Float, hit::{Bound, Hit, HitRecord, bvh::AABB}, material::simple::Material, ray::Ray, texture::UV, vec3::{Point, Vec3, cross, dot}};

#[derive(Clone, Copy, Default)]
pub struct Quad {
    pub origin: Point,
    pub u: Vec3,
    pub v: Vec3,
    pub material: Material,
}

pub fn quad(origin: (Float, Float, Float), u: (Float, Float, Float), v: (Float, Float, Float), material: Material) -> Quad {
    Quad { origin: origin.into(), u: u.into(), v: v.into(), material }
}

impl Hit for Quad {
    fn hit(&self, r: Ray, t_min: Float, t_max: Float) -> Option<HitRecord> {
        // TODO maybe cache these?
        let n = cross(self.u, self.v);
        let normal = n.normalize();
        let normal = if dot(normal, r.direction) < 0. { normal } else { -normal };
        let d = dot(normal, self.origin); // TODO is it really worth using this as an intermediate?

        let denom = dot(normal, r.direction);
        if denom.abs() < Float::EPSILON {
            return None;
        }

        let t = (d - dot(normal, r.origin)) / denom;
        if t < t_min || t > t_max {
            return None;
        }

        let pos = r.at(t);
        let uv = self.uv(pos);
        if uv.0 < 0. || uv.0 > 1. || uv.1 < 0. || uv.1 > 1. {
            return None;
        }

        Some(HitRecord { t, material: self.material, normal, pos, uv })
    }
}

impl UV for Quad {
    fn uv(&self, pos: Point) -> (Float, Float) {
        let p = pos - self.origin;
        // TODO maybe cache w?
        let w = cross(self.u, self.v) / dot(cross(self.u, self.v), cross(self.u, self.v));
        (dot(w, cross(p, self.v)), dot(w, cross(self.u, p)))
    }
}

impl Bound for Quad {
    type HitType = AABB;
    fn bound(&self) -> AABB {
        AABB::from_points(self.origin, self.origin + self.u + self.v)
    }
}

#[test]
fn test_quad_hit() {
    let q = quad((0., 0., 0.), (1., 0., 0.), (0., 1., 0.), Material::None);
    let ray = ray!((0.5, 0.5, -1.) -> (0., 0., 1.));
    let hit = q.hit(ray, 0., Float::INFINITY);
    assert!(hit.is_some());
    let hit_record = hit.unwrap();
    assert_ulps_eq!(hit_record.t, 1.);
    assert_eq!(hit_record.pos, (0.5, 0.5, 0.).into());
    assert_eq!(hit_record.normal, (0., 0., -1.).into());
    assert_ulps_eq!(hit_record.uv.0, 0.5);
    assert_ulps_eq!(hit_record.uv.1, 0.5);

    let q = quad((-1., 0., 0.), (0., 2., 0.), (2.0, 0., -2.0), Material::None);
    let ray = ray!((0, 1, 0) -> (0, 0, -1));
    let hit = q.hit(ray, 0., Float::INFINITY);
    assert!(hit.is_some());
    let hit_record = hit.unwrap();
    assert_ulps_eq!(hit_record.t, 1.);
    assert_eq!(hit_record.pos, (0., 1., -1.).into());
    assert_eq!(hit_record.normal, Vec3::from((1., 0., 1.)).normalize());
    assert_ulps_eq!(hit_record.uv.0, 0.5);
    assert_ulps_eq!(hit_record.uv.1, 0.5);

    // test ray through plane but not through quad
    let ray = ray!((0.5, 0.5, 1.) -> (0., 1., -1.));
    assert!(q.hit(ray, 0., Float::INFINITY).is_none());
    
    // test ray along plane but outside quad
    let ray = ray!((1.5, 0., 0.) -> (0., 1., 0.));
    assert!(q.hit(ray, 0., Float::INFINITY).is_none());

    // test ray along quad
    let ray = ray!((0.5, 0.5, 0.) -> (1., 0., 0.));
    assert!(q.hit(ray, 0., Float::INFINITY).is_none());

    // test ray away from quad
    let ray = ray!((0.5, 0.5, 1.) -> (0., 0., 1.));
    assert!(q.hit(ray, 0., Float::INFINITY).is_none());
}

fn test_quad_uv() {
    let q = quad((0., 0., 0.), (1., 0., 0.), (0., 1., 0.), Material::None);
    let uv = q.uv((0.5, 0.5, 0.).into());
    assert_ulps_eq!(uv.0, 0.5);
    assert_ulps_eq!(uv.1, 0.5);

    let uv = q.uv((1., 1., 0.).into());
    assert_ulps_eq!(uv.0, 1.);
    assert_ulps_eq!(uv.1, 1.);

    let q = quad((0., 0., 0.), (0., 2., 0.), (-1., 0., 0.), Material::None);
    let uv = q.uv((-1.7, -1., 0.).into());
    assert_ulps_eq!(uv.0, -0.5);
    assert_ulps_eq!(uv.1, 1.7);
}

fn test_quad_bound() {
    let q = quad((0., 0., 0.), (1., 0., 0.), (0., 1., 0.), Material::None);
    let bound = q.bound();
    assert_ulps_eq!(bound.x.min, 0.);
    assert_ulps_eq!(bound.x.max, 1.);
    assert_ulps_eq!(bound.y.min, 0.);
    assert_ulps_eq!(bound.y.max, 1.);
    assert_ulps_eq!(bound.z.min, -AABB::MIN_LENGTH / 2.);
    assert_ulps_eq!(bound.z.max, AABB::MIN_LENGTH / 2.);

    let q = quad((-1., -1., -1.), (2., 0., 0.), (0., 2., 0.), Material::None);
    let bound = q.bound();
    assert_ulps_eq!(bound.x.min, -1.);
    assert_ulps_eq!(bound.x.max, 1.);
    assert_ulps_eq!(bound.y.min, -1.);
    assert_ulps_eq!(bound.y.max, 1.);
    assert_ulps_eq!(bound.z.min, -1.);
    assert_ulps_eq!(bound.z.max, -1.);

    let q = quad((-1., -1., 1.), (2., 0., -1.), (0., 2., -1.), Material::None);
    let bound = q.bound();
    assert_ulps_eq!(bound.x.min, -1.);
    assert_ulps_eq!(bound.x.max, 1.);
    assert_ulps_eq!(bound.y.min, -1.);
    assert_ulps_eq!(bound.y.max, 1.);
    assert_ulps_eq!(bound.z.min, -1.);
    assert_ulps_eq!(bound.z.max, 1.);
}