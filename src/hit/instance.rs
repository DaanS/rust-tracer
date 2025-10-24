use crate::{config::Float, hit::{bvh::{AxisAlignedBound, AABB}, Bound, Hit, HitRecord}, ray::Ray, vec3::Vec3};

#[inline(always)]
fn translate_hit(r: Ray, offset: Vec3, object: &dyn Hit, t_min: Float, t_max: Float) -> Option<HitRecord> {
    let moved_ray = Ray { origin: r.origin - offset, direction: r.direction, time: r.time, };
    object.hit(moved_ray, t_min, t_max).and_then(|mut hit| { hit.pos += offset; Some(hit)})
}

#[inline(always)]
fn translate_bound(offset: Vec3, object: &dyn Bound<HitType=AABB>) -> AABB {
    let b = object.bound();
    AABB {
        x: (b.x.min + offset.x, b.x.max + offset.x).into(),
        y: (b.y.min + offset.y, b.y.max + offset.y).into(),
        z: (b.z.min + offset.z, b.z.max + offset.z).into(),
    }
}

pub struct Translate {
    pub offset: Vec3,
    pub object: Box<dyn AxisAlignedBound + Send + Sync>,
}

impl Hit for Translate {
    fn hit(&self, r: Ray, t_min: Float, t_max: Float) -> Option<HitRecord> {
        translate_hit(r, self.offset, self.object.as_ref(), t_min, t_max)
    }
}

impl Bound for Translate {
    type HitType = AABB;
    fn bound(&self) -> AABB {
        translate_bound(self.offset, self.object.as_ref())
    }
}

pub struct Animate {
    pub offset_start: Vec3,
    pub offset_end: Vec3,
    pub interpolation: Box<dyn Fn(Float) -> Float + Send + Sync>,
    pub object: Box<dyn AxisAlignedBound + Send + Sync>,
}

impl Hit for Animate {
    fn hit(&self, r: Ray, t_min: Float, t_max: Float) -> Option<HitRecord> {
        let offset = self.offset_start + (self.interpolation)(r.time) * (self.offset_end - self.offset_start);
        translate_hit(r, offset, self.object.as_ref(), t_min, t_max)
    }
}

impl Bound for Animate {
    type HitType = AABB;
    fn bound(&self) -> AABB {
        let b1 = translate_bound(self.offset_start, self.object.as_ref());
        let b2 = translate_bound(self.offset_end, self.object.as_ref());
        AABB::enclosing(b1, b2)
    }
}

#[cfg(test)]
mod tests {
    use crate::{config::Float, hit::{instance::{Animate, Translate}, sphere::sphere, Hit}, material::simple::lambertian, ray::Ray};

    fn test_translate() {
        let s = sphere((0., 0., -1.), 0.5, lambertian((0.7, 0.3, 0.3)));
        let t = Translate { offset: vec3!(0., 1., 0.), object: Box::new(s) };

        let r1 = ray!((0, 1, 0) -> (0, 0, -1));
        let r2 = ray!((0, 0, 0) -> (0, 0, -1));
        assert_eq!(t.hit(r1, 0.001, Float::INFINITY), s.hit(r2, 0.001, Float::INFINITY));
    }

    fn test_animate() {
        let s = sphere((0., 0., -1.), 0.5, lambertian((0.7, 0.3, 0.3)));
        let a = Animate {
            offset_start: vec3!(0., 0., 0.),
            offset_end: vec3!(0., 1., 0.),
            interpolation: Box::new(|t| t),
            object: Box::new(s)
        };

        let r1 = ray!((0, 0, 0) -> (0, 0, -1));
        let r2 = Ray { origin: vec3!(0., 0.5, 0.), direction: vec3!(0., 0., -1.), time: 0.5 };
        let r3 = Ray { origin: vec3!(0., 1., 0.), direction: vec3!(0., 0., -1.), time: 1.0 };
        assert_eq!(a.hit(r2, 0.001, Float::INFINITY), s.hit(r1, 0.001, Float::INFINITY));
        assert_eq!(a.hit(r3, 0.001, Float::INFINITY), s.hit(r1, 0.001, Float::INFINITY));
    }
}