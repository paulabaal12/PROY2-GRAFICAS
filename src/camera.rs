use crate::vec3::Vec3;
use crate::ray::Ray;

#[derive(Clone)]
pub struct Camera {
    pub origin: Vec3,
    pub lower_left_corner: Vec3,
    pub horizontal: Vec3,
    pub vertical: Vec3,
    pub position: Vec3, // Add position field
}

impl Camera {
    pub fn new(lookfrom: Vec3, lookat: Vec3, vup: Vec3, vfov: f32, aspect: f32, _aperture: f32, focus_dist: f32) -> Self {
        let theta = vfov.to_radians();
        let half_height = (theta / 2.0).tan();
        let half_width = aspect * half_height;
        let origin = lookfrom;
        let w = (lookfrom - lookat).normalize();
        let u = vup.cross(w).normalize();
        let v = w.cross(u);
        let lower_left_corner = origin - u * half_width * focus_dist - v * half_height * focus_dist - w * focus_dist;
        let horizontal = u * 2.0 * half_width * focus_dist;
        let vertical = v * 2.0 * half_height * focus_dist;

        Camera {
            origin,
            lower_left_corner,
            horizontal,
            vertical,
            position: lookfrom, // Initialize position
        }
    }

    pub fn get_ray(&self, s: f32, t: f32) -> Ray {
        Ray {
            origin: self.origin,
            direction: self.lower_left_corner + self.horizontal * s + self.vertical * t - self.origin,
        }
    }
}
