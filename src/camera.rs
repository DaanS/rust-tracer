use crate::{ray::Ray, ray::ray, config::Float, film::Film, vec3::{Vec3, cross, random_vector_in_unit_disk}, util::radians};

pub struct Camera {
    cam_pos: Vec3,
    pixel_center_upper_left: Vec3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    defocus_disk_u: Vec3,
    defocus_disk_v: Vec3
}

impl Camera {
    pub fn new(film: &Film, from: Vec3, to: Vec3, up: Vec3, vfov: Float, focal_len: Float, defocus_angle: Float) -> Camera {
        let aspect_ratio = film.width as Float / film.height as Float;
        let theta = radians(vfov);
        let h = (theta / 2.).tan();
        let viewport_height = 2. * h * focal_len;
        let viewport_width = viewport_height * aspect_ratio;

        let w = (from - to).normalize();
        let u = cross(up, w).normalize();
        let v = cross(w, u);

        let viewport_u = viewport_width * u;
        let viewport_v = viewport_height * -v;

        let pixel_delta_u = viewport_u / film.width as Float;
        let pixel_delta_v = viewport_v / film.height as Float;

        let viewport_upper_left = from - (focal_len * w) - viewport_u / 2. - viewport_v / 2.;
        let pixel_center_upper_left = viewport_upper_left + pixel_delta_u / 2. + pixel_delta_v / 2.;

        let defocus_radius = focal_len * radians(defocus_angle / 2.).tan();
        let defocus_disk_u = u * defocus_radius;
        let defocus_disk_v = v * defocus_radius;

        Camera { cam_pos: from, pixel_center_upper_left, pixel_delta_u, pixel_delta_v, defocus_disk_u, defocus_disk_v }
    }

    pub fn get_ray(&self, s: Float, t: Float) -> Ray {
        let target = self.pixel_center_upper_left + s * self.pixel_delta_u + t * self.pixel_delta_v;
        let offset = random_vector_in_unit_disk();
        let pos = self.cam_pos + offset.x * self.defocus_disk_u + offset.y * self.defocus_disk_v;
        let direction = target - pos;

        ray(pos, direction)
    }
}