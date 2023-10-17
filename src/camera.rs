use crate::{ray::Ray, ray::ray, config::Float, film::Film, vec3::Vec3};

pub struct Camera {
    width: usize,
    height: usize,
    cam_pos: Vec3,
    focal_len: Vec3
}

impl Camera {
    pub fn new(film: &Film, cam_pos: Vec3, focal_len: Vec3) -> Camera {
        Camera { width: film.width, height: film.height, cam_pos, focal_len }
    }

    pub fn get_ray(&self, s: Float, t: Float) -> Ray {
        let aspect_ratio = self.width as Float / self.height as Float;
        let viewport_height = 2.;
        let viewport_width = viewport_height * aspect_ratio;
        let viewport_u = vec3!(viewport_width, 0, 0);
        let viewport_v = vec3!(0, -viewport_height, 0);
        let pixel_delta_u = viewport_u / self.width as Float;
        let pixel_delta_v = viewport_v / self.height as Float;
        let viewport_upper_left = self.cam_pos + self.focal_len - viewport_u / 2. - viewport_v / 2.;
        let pixel_center_upper_left = viewport_upper_left + pixel_delta_u / 2. + pixel_delta_v / 2.;

        let pixel_center = pixel_center_upper_left + s * pixel_delta_u + t * pixel_delta_v;
        let direction = pixel_center - self.cam_pos;
        let ray = ray(self.cam_pos, direction);

        ray
    }
}