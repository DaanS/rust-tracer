use std::{fmt::Debug, mem::take};

use crate::{config::Float, hit::{Bound, Hit, HitRecord, aabb::AABB, sphere::Sphere}, ray::Ray};


pub trait AxisAlignedBound: Hit + Bound<HitType = AABB> {}
impl<T: Hit + Bound<HitType = AABB>> AxisAlignedBound for T {}

impl Bound for &mut [Box<dyn AxisAlignedBound + Send + Sync>] {
    type HitType = AABB;

    // TODO cache this probably (LazyCell or OnceCell maybe?)
    fn bound(&self) -> AABB {
        self.iter().fold(
            AABB::new((0., 0.).into(), (0., 0.).into(), (0., 0.).into()),
            |aabb, s| AABB::enclosing(aabb, s.bound())
        )
    }
}

impl Default for Box<dyn AxisAlignedBound + Send + Sync> {
    fn default() -> Self {
        Box::new(Sphere::default())
    }
}

pub struct Bvh {
    pub aabb: AABB,
    pub left: Box<dyn AxisAlignedBound + Send + Sync>,
    pub right: Box<dyn AxisAlignedBound + Send + Sync>,
}

impl Bvh {
    pub fn new(left: Box<dyn AxisAlignedBound + Send + Sync>, right: Box<dyn AxisAlignedBound + Send + Sync>) -> Self {
        let aabb = AABB::enclosing(left.bound(), right.bound());
        Bvh { aabb, left, right }
    }

    pub fn from_slice(objects: &mut [Box<dyn AxisAlignedBound + Send + Sync>]) -> Self {
        match objects.len() {
            0 => panic!("Cannot create BVH from an empty slice"),
            1 => panic!("It makes no sense to create a BVH from a single object"),
            2 => { Bvh::new(take(&mut objects[0]), take(&mut objects[1])) },
            _ => {
                let aabb = objects.bound();

                objects.sort_by(|a, b| {
                    let axis = aabb.longest_axis();
                    // XXX without the 'as AABB' rust-analyzer seems to think min is a function
                    (a.bound() as AABB)[axis].min.partial_cmp(&(b.bound() as AABB)[axis].min).unwrap()
                });

                let (left_objects, right_objects) = objects.split_at_mut(objects.len() / 2);
                assert!(left_objects.len() <= right_objects.len(), "Left side of BVH must not be greater than right side");
                Bvh {
                    aabb,
                    left: if left_objects.len() == 1 {
                        take(&mut left_objects[0])
                    } else {
                        Box::new(Bvh::from_slice(left_objects))
                    },
                    right: Box::new(Bvh::from_slice(right_objects))
                }
            }
        }
    }
}

impl Hit for Bvh {
    fn hit(&self, r: Ray, t_min: Float, t_max: Float) -> Option<HitRecord> {
        if !self.aabb.intersects(r, t_min, t_max) {
            return None;
        }

        let left_hit = self.left.hit(r, t_min, t_max);
        let right_hit = self.right.hit(r, t_min, 
            if let Some(ref left_hit) = left_hit { left_hit.t } else { t_max }
        );

        match (left_hit, right_hit) {
            (Some(_l), Some(r)) => {
                // if left was hit, right only checks for closer hits, so if it's hit too it must be closer than left
                Some(r)
            },
            (Some(l), None) => Some(l),
            (None, Some(r)) => Some(r),
            (None, None) => None,
        }
    }
}

impl Bound for Bvh {
    type HitType = AABB;
    fn bound(&self) -> AABB {
        self.aabb
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
use crate::{hit::sphere::sphere, material::simple::Material};

#[test]
fn test_aabb() {
    let a = vec3!(0, 0, 0);
    let b = vec3!(1, 1, 1);
    let aabb = AABB::from_points(a, b);

    assert!(aabb.intersects(ray!((0.5, 0.5, -0.5) -> (0, 0, 1)), 0., Float::MAX));
    assert!(!aabb.intersects(ray!((1.5, 1.5, -0.5) -> (0, 0, 1)), 0., Float::MAX));

    let c = vec3!(1, 1, 0);
    let aabb2 = AABB::from_points(a, c);
    assert!(aabb2.intersects(ray!((0.5, 0.5, -0.5) -> (0, 0, 1)), 0., Float::MAX));
    assert!(!aabb2.intersects(ray!((1.5, 1.5, -0.5) -> (0, 0, 1)), 0., Float::MAX));
}

#[test]
fn test_bvh_hit() {
    let s1 = sphere((0., 0., 0.), 1., Material::None);
    let s2 = sphere((4., 0., 0.), 1., Material::None);
    let bvh = Bvh {
        aabb: AABB::enclosing(s1.bound(), s2.bound()),
        left: Box::new(s1),
        right: Box::new(s2),
    };

    let r1 = ray!((-2, 0, 0) -> (1, 0, 0));
    assert_eq!(bvh.hit(r1, 0., Float::MAX), s1.hit(r1, 0., Float::MAX));
    let r2 = ray!((6, 0, 0) -> (-1, 0, 0));
    assert_eq!(bvh.hit(r2, 0., Float::MAX), s2.hit(r2, 0., Float::MAX));
    let r3 = ray!((2, -2, 0) -> (0, 1, 0));
    assert!(bvh.hit(r3, 0., Float::MAX).is_none());
    let r4 = ray!((2, 0, 0) -> (1, 0, 0));
    assert_eq!(bvh.hit(r4, 0., Float::MAX), s2.hit(r4, 0., Float::MAX));
    let r5 = ray!((-1, 0, 0) -> (1, 0, 0));
    assert_eq!(bvh.hit(r5, 3., Float::MAX), s2.hit(r5, 3., Float::MAX));
    let r6 = ray!((0, 0, 0) -> (1, 0, 0));
    assert_eq!(bvh.hit(r6, 0., Float::MAX), s1.hit(r6, 0., Float::MAX));

    let bvh2 = Bvh::from_slice(&mut [Box::new(s1), Box::new(s2)]);
    assert_eq!(bvh.aabb, bvh2.aabb);

    let s3 = sphere((2., 2., 0.), 1., Material::None);
    let bvh3 = Bvh::from_slice(&mut [Box::new(s1), Box::new(s2), Box::new(s3)]);
    assert_eq!(bvh3.hit(r1, 0., Float::MAX), s1.hit(r1, 0., Float::MAX));
    assert_eq!(bvh3.hit(r2, 0., Float::MAX), s2.hit(r2, 0., Float::MAX));
    assert_eq!(bvh3.hit(r3, 0., Float::MAX), s3.hit(r3, 0., Float::MAX));
    assert!(bvh3.hit(ray!((2, 0, 0) -> (0, 0, 1)), 0., Float::MAX).is_none());
}