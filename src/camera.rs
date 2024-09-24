use crate::vec3::Vec3;
use crate::ray::Ray;

pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32,
    pub aspect_ratio: f32,
    pub near_plane: f32,
    pub far_plane: f32,
}

impl Camera {
    pub fn new(position: Vec3, target: Vec3, up: Vec3, fov: f32, aspect_ratio: f32, near_plane: f32, far_plane: f32) -> Self {
        Camera {
            position,
            target,
            up,
            fov,
            aspect_ratio,
            near_plane,
            far_plane,
        }
    }

    pub fn get_ray(&self, u: f32, v: f32) -> Ray {
        let forward = (self.target - self.position).normalize();
        let right = forward.cross(self.up).normalize();
        let up = right.cross(forward).normalize();

        let half_height = (self.fov.to_radians() / 2.0).tan();
        let half_width = self.aspect_ratio * half_height;

        let lower_left_corner = self.position - right * half_width - up * half_height + forward;
        let horizontal = right * 2.0 * half_width;
        let vertical = up * 2.0 * half_height;

        Ray {
            origin: self.position,
            direction: (lower_left_corner + horizontal * u + vertical * v - self.position).normalize(),
        }
    }

    pub fn zoom(&mut self, amount: f32) {
        let direction = (self.target - self.position).normalize();
        self.position = self.position + direction * amount;
    }
}
