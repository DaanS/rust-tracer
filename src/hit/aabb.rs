use std::{mem::swap, ops::Index};

use crate::{config::Float, ray::Ray, util::Interval, vec3::Point};


#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AABB {
    pub x: Interval,
    pub y: Interval,
    pub z: Interval,
}

impl AABB {
    pub const MIN_LENGTH: Float = 1e-4;

    pub fn new(x: Interval, y: Interval, z: Interval) -> Self {
        let mut aabb = AABB { x, y, z };
        aabb.pad_to_minimum();
        aabb
    }

    pub fn from_points(a: Point, b: Point) -> Self {
        let mut aabb = AABB {
            x: if a.x < b.x { (a.x, b.x).into() } else { (b.x, a.x).into() },
            y: if a.y < b.y { (a.y, b.y).into() } else { (b.y, a.y).into() },
            z: if a.z < b.z { (a.z, b.z).into() } else { (b.z, a.z).into() }
        };
        aabb.pad_to_minimum();
        aabb
    }

    pub fn enclosing(a: AABB, b: AABB) -> Self {
        let mut aabb = AABB {
            x: Interval::enclosing(a.x, b.x),
            y: Interval::enclosing(a.y, b.y),
            z: Interval::enclosing(a.z, b.z),
        };
        aabb.pad_to_minimum();
        aabb
    }

    fn pad_to_minimum(&mut self) {
        if self.x.length() < Self::MIN_LENGTH {
            let mid = (self.x.min + self.x.max) / 2.;
            self.x.min = mid - Self::MIN_LENGTH / 2.;
            self.x.max = mid + Self::MIN_LENGTH / 2.;
        }
        if self.y.length() < Self::MIN_LENGTH {
            let mid = (self.y.min + self.y.max) / 2.;
            self.y.min = mid - Self::MIN_LENGTH / 2.;
            self.y.max = mid + Self::MIN_LENGTH / 2.;
        }
        if self.z.length() < Self::MIN_LENGTH {
            let mid = (self.z.min + self.z.max) / 2.;
            self.z.min = mid - Self::MIN_LENGTH / 2.;
            self.z.max = mid + Self::MIN_LENGTH / 2.;
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

impl AABB {
    pub fn intersects(&self, r: Ray, mut t_min: Float, mut t_max: Float) -> bool {
        for i in 0..3 {
            let (min, max) = match i {
                0 => (self.x.min, self.x.max),
                1 => (self.y.min, self.y.max),
                _ => (self.z.min, self.z.max),
            };
            let inv_d = r.inv_direction[i];
            let mut t0 = (min - r.origin[i]) * inv_d;
            let mut t1 = (max - r.origin[i]) * inv_d;

            if t1 < t0 { swap(&mut t0, &mut t1); }
            if t0 > t_min { t_min = t0; }
            if t1 < t_max { t_max = t1; }

            if t_max <= t_min {
                return false;
            }
        }
        true
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