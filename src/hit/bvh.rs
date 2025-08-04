use std::{fmt::Debug, mem::swap, ops::Index, sync::Arc};

use crate::{config::Float, hit::{Bound, Hit, HitRecord}, material::Material, ray::Ray, util::Interval, vec3::Point};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AABB {
    pub x: Interval,
    pub y: Interval,
    pub z: Interval,
}

impl AABB {
    pub fn new(x: Interval, y: Interval, z: Interval) -> Self {
        AABB { x, y, z }
    }

    pub fn from_points(a: Point, b: Point) -> Self {
        AABB {
            x: if a.x < b.x { (a.x, b.x).into() } else { (b.x, a.x).into() },
            y: if a.y < b.y { (a.y, b.y).into() } else { (b.y, a.y).into() },
            z: if a.z < b.z { (a.z, b.z).into() } else { (b.z, a.z).into() }
        }
    }

    pub fn enclosing(a: AABB, b: AABB) -> Self {
        AABB {
            x: Interval::enclosing(a.x, b.x),
            y: Interval::enclosing(a.y, b.y),
            z: Interval::enclosing(a.z, b.z),
        }
    }

    pub fn longest_axis(&self) -> usize {
        let x_len = self.x.length();
        let y_len = self.y.length();
        let z_len = self.z.length();

        if x_len >= y_len && x_len >= z_len {
            0
        } else if y_len >= z_len {
            1
        } else {
            2
        }
    }
}

impl Hit for AABB {
    fn hit(&self, r: Ray, mut t_min: Float, mut t_max: Float) -> Option<HitRecord> {
        for i in 0..3 {
            let (min, max) = match i {
                0 => (self.x.min, self.x.max),
                1 => (self.y.min, self.y.max),
                _ => (self.z.min, self.z.max),
            };
            let inv_d = 1.0 / r.direction[i];
            let mut t0 = (min - r.origin[i]) * inv_d;
            let mut t1 = (max - r.origin[i]) * inv_d;
            
            if t1 < t0 { swap(&mut t0, &mut t1); }
            if t0 > t_min { t_min = t0; }
            if t1 < t_max { t_max = t1; }

            if t_max <= t_min {
                return None;
            }
        }
        Some(HitRecord{ t: t_min, material: Material::None, normal: vec3!(0., 0., 0.), pos: r.at(t_min) })
    }
}

impl Index<usize> for AABB {
    type Output = Interval;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            _ => &self.z,
        }
    }
}

pub trait AxisAlignedBound: Hit + Bound<HitType = AABB> {}
impl<T: Hit + Bound<HitType = AABB>> AxisAlignedBound for T {}

impl Bound for &[Arc<dyn AxisAlignedBound + Send + Sync>] {
    type HitType = AABB;

    // TODO cache this probably (LazyCell or OnceCell maybe?)
    fn bound(&self) -> AABB {
        self.iter().fold(
            AABB::new((0., 0.).into(), (0., 0.).into(), (0., 0.).into()),
            |aabb, s| AABB::enclosing(aabb, s.bound())
        )
    }
}

pub struct Bvh {
    pub aabb: AABB,
    pub left: Arc<dyn AxisAlignedBound + Send + Sync>,
    pub right: Arc<dyn AxisAlignedBound + Send + Sync>,
}

impl Bvh {
    pub fn new(left: Arc<dyn AxisAlignedBound + Send + Sync>, right: Arc<dyn AxisAlignedBound + Send + Sync>) -> Self {
        let aabb = AABB::enclosing(left.bound(), right.bound());
        Bvh { aabb, left, right }
    }

    pub fn from_slice(objects: &[Arc<dyn AxisAlignedBound + Send + Sync>]) -> Self {
        match objects.len() {
            0 => panic!("Cannot create BVH from an empty slice"),
            1 => { Bvh::new(objects[0].clone(), objects[0].clone()) },
            2 => { Bvh::new(objects[0].clone(), objects[1].clone()) },
            _ => {
                let aabb = objects.bound();

                let mut objects = Vec::from(objects);
                objects.sort_by(|a, b| {
                    let axis = aabb.longest_axis();
                    a.bound()[axis].min.partial_cmp(&b.bound()[axis].min).unwrap()
                });

                let (left_objects, right_objects) = objects.split_at(objects.len() / 2);
                assert!(left_objects.len() <= right_objects.len(), "Left side of BVH must not be greater than right side");
                Bvh {
                    aabb,
                    left: if left_objects.len() == 1 {
                        left_objects[0].clone()
                    } else {
                        Arc::new(Bvh::from_slice(left_objects))
                    },
                    right: Arc::new(Bvh::from_slice(right_objects))
                }
            }
        }
    }
}

impl Hit for Bvh {
    fn hit(&self, r: Ray, t_min: Float, t_max: Float) -> Option<HitRecord> {
        self.aabb.hit(r, t_min, t_max).and_then(|hit_record| {
            let left_hit = self.left.hit(r, hit_record.t, t_max);
            let right_hit = self.right.hit(r, hit_record.t, t_max);

            match (left_hit, right_hit) {
                (Some(l), Some(r)) => {
                    if l.t < r.t { Some(l) } else { Some(r) }
                },
                (Some(l), None) => Some(l),
                (None, Some(r)) => Some(r),
                (None, None) => None,
            }
        })
    }
}

impl Bound for Bvh {
    type HitType = AABB;
    fn bound(&self) -> AABB {
        self.aabb.clone()
    }
}

impl Debug for Bvh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bvh")
            .field("aabb", &self.aabb)
            .field("left", &self.left.bound())
            .field("right", &self.right.bound())
            .finish()
    }
}

#[cfg(test)]
use approx::assert_ulps_eq;
#[cfg(test)]
use crate::hit::sphere::sphere;

#[test]
fn test_aabb() {
    let a = vec3!(0, 0, 0);
    let b = vec3!(1, 1, 1);
    let aabb = AABB::from_points(a, b);
    
    assert_ulps_eq!(aabb.hit(ray!((0.5, 0.5, -0.5) -> (0, 0, 1)), 0., f64::MAX).unwrap().t, 0.5);
    assert!(aabb.hit(ray!((1.5, 1.5, -0.5) -> (0, 0, 1)), 0., f64::MAX).is_none());
}

#[test]
fn test_bvh_hit() {
    let s1 = sphere((0., 0., 0.), 1., Material::None);
    let s2 = sphere((4., 0., 0.), 1., Material::None);
    let bvh = Bvh {
        aabb: AABB::enclosing(s1.bound(), s2.bound()),
        left: Arc::new(s1),
        right: Arc::new(s2),
    };

    let r1 = ray!((-2, 0, 0) -> (1, 0, 0));
    assert_eq!(bvh.hit(r1, 0., f64::MAX), s1.hit(r1, 0., f64::MAX));
    let r2 = ray!((6, 0, 0) -> (-1, 0, 0));
    assert_eq!(bvh.hit(r2, 0., f64::MAX), s2.hit(r2, 0., f64::MAX));
    let r3 = ray!((2, -2, 0) -> (0, 1, 0));
    assert!(bvh.hit(r3, 0., f64::MAX).is_none());

    let bvh2 = Bvh::from_slice(&[Arc::new(s1), Arc::new(s2)]);
    assert_eq!(bvh.aabb, bvh2.aabb);

    let s3 = sphere((2., 2., 0.), 1., Material::None);
    let bvh3 = Bvh::from_slice(&[Arc::new(s1), Arc::new(s2), Arc::new(s3)]);
    assert_eq!(bvh3.hit(r1, 0., f64::MAX), s1.hit(r1, 0., f64::MAX));
    assert_eq!(bvh3.hit(r2, 0., f64::MAX), s2.hit(r2, 0., f64::MAX));
    assert_eq!(bvh3.hit(r3, 0., f64::MAX), s3.hit(r3, 0., f64::MAX));
    assert!(bvh3.hit(ray!((2, 0, 0) -> (0, 0, 1)), 0., f64::MAX).is_none());
}