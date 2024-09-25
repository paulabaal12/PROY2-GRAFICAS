use nalgebra_glm::Vec3;
use crate::material::Material;
use crate::ray_intersect::{RayIntersect, Intersect};

pub struct Cube {
    pub min: Vec3,
    pub max: Vec3,
    pub material: Material,
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        let mut t_min = (self.min.x - ray_origin.x) / ray_direction.x;
        let mut t_max = (self.max.x - ray_origin.x) / ray_direction.x;

        if t_min > t_max {
            std::mem::swap(&mut t_min, &mut t_max);
        }

        let mut ty_min = (self.min.y - ray_origin.y) / ray_direction.y;
        let mut ty_max = (self.max.y - ray_origin.y) / ray_direction.y;

        if ty_min > ty_max {
            std::mem::swap(&mut ty_min, &mut ty_max);
        }

        if (t_min > ty_max) || (ty_min > t_max) {
            return Intersect::empty();
        }

        if ty_min > t_min {
            t_min = ty_min;
        }

        if ty_max < t_max {
            t_max = ty_max;
        }

        let mut tz_min = (self.min.z - ray_origin.z) / ray_direction.z;
        let mut tz_max = (self.max.z - ray_origin.z) / ray_direction.z;

        if tz_min > tz_max {
            std::mem::swap(&mut tz_min, &mut tz_max);
        }

        if (t_min > tz_max) || (tz_min > t_max) {
            return Intersect::empty();
        }

        if tz_min > t_min {
            t_min = tz_min;
        }

        if tz_max < t_max {
            t_max = tz_max;
        }

        let point = ray_origin + ray_direction * t_min;
        let normal = if point.x == self.min.x {
            Vec3::new(-1.0, 0.0, 0.0)
        } else if point.x == self.max.x {
            Vec3::new(1.0, 0.0, 0.0)
        } else if point.y == self.min.y {
            Vec3::new(0.0, -1.0, 0.0)
        } else if point.y == self.max.y {
            Vec3::new(0.0, 1.0, 0.0)
        } else if point.z == self.min.z {
            Vec3::new(0.0, 0.0, -1.0)
        } else {
            Vec3::new(0.0, 0.0, 1.0)
        };

        Intersect::new(point, normal, t_min, self.material)
    }
}
