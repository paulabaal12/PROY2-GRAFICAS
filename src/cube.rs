use nalgebra_glm::Vec3;
use crate::ray_intersect::{RayIntersect, Intersect};
use crate::texture::Texture;
use std::rc::Rc;
use crate::material::Material;
use crate::color::Color;

#[derive(Clone)]
pub struct Cube {
    pub min: Vec3,  
    pub max: Vec3,  
    pub top_texture: Rc<Texture>,    
    pub bottom_texture: Rc<Texture>,
    pub side_texture: Rc<Texture>,   
}

impl Cube {
    pub fn new(min: Vec3, max: Vec3, top_texture: Rc<Texture>, bottom_texture: Rc<Texture>, side_texture: Rc<Texture>) -> Self {
        Cube {
            min,
            max,
            top_texture,
            bottom_texture,
            side_texture,
        }
    }

    fn get_texture(&self, normal: &Vec3) -> &Rc<Texture> {
        if normal.y.abs() > 0.9 {
            if normal.y > 0.0 {
                &self.top_texture
            } else {
                &self.bottom_texture
            }
        } else {
            &self.side_texture
        }
    }

    fn get_uv(&self, point: &Vec3, normal: &Vec3) -> (f32, f32) {
        let (u, v) = if normal.y.abs() > 0.9 {
            let u = (point.x - self.min.x) / (self.max.x - self.min.x);
            let v = (point.z - self.min.z) / (self.max.z - self.min.z);
            (u, v)
        } else if normal.x.abs() > 0.9 {
            let u = (point.z - self.min.z) / (self.max.z - self.min.z);
            let v = (point.y - self.min.y) / (self.max.y - self.min.y);
            (u, v)
        } else {
            let u = (point.x - self.min.x) / (self.max.x - self.min.x);
            let v = (point.y - self.min.y) / (self.max.y - self.min.y);
            (u, v)
        };

        let v = 1.0 - v; 

        self.clamp_uv(u, v)
    }

    fn clamp_uv(&self, u: f32, v: f32) -> (f32, f32) {
        (u.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
    }

    fn calculate_normal(&self, point: Vec3) -> Vec3 {
        let epsilon = 1e-4;
        if (point.x - self.min.x).abs() < epsilon {
            Vec3::new(-1.0, 0.0, 0.0)
        } else if (point.x - self.max.x).abs() < epsilon {
            Vec3::new(1.0, 0.0, 0.0)
        } else if (point.y - self.min.y).abs() < epsilon {
            Vec3::new(0.0, -1.0, 0.0)
        } else if (point.y - self.max.y).abs() < epsilon {
            Vec3::new(0.0, 1.0, 0.0)
        } else if (point.z - self.min.z).abs() < epsilon {
            Vec3::new(0.0, 0.0, -1.0)
        } else {
            Vec3::new(0.0, 0.0, 1.0)
        }
    }
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect {
        let mut t_min = (self.min.x - ray_origin.x) / ray_direction.x;
        let mut t_max = (self.max.x - ray_origin.x) / ray_direction.x;

        if t_min > t_max {
            std::mem::swap(&mut t_min, &mut t_max);
        }

        let mut t_y_min = (self.min.y - ray_origin.y) / ray_direction.y;
        let mut t_y_max = (self.max.y - ray_origin.y) / ray_direction.y;

        if t_y_min > t_y_max {
            std::mem::swap(&mut t_y_min, &mut t_y_max);
        }

        if (t_min > t_y_max) || (t_y_min > t_max) {
            return Intersect::empty();
        }

        if t_y_min > t_min {
            t_min = t_y_min;
        }

        if t_y_max < t_max {
            t_max = t_y_max;
        }

        let mut t_z_min = (self.min.z - ray_origin.z) / ray_direction.z;
        let mut t_z_max = (self.max.z - ray_origin.z) / ray_direction.z;

        if t_z_min > t_z_max {
            std::mem::swap(&mut t_z_min, &mut t_z_max);
        }

        if (t_min > t_z_max) || (t_z_min > t_max) {
            return Intersect::empty();
        }

        if t_z_min > t_min {
            t_min = t_z_min;
        }

        if t_min < 0.0 {
            return Intersect::empty();
        }

        let point_on_surface = ray_origin + ray_direction * t_min;
        let normal = self.calculate_normal(point_on_surface);
        let texture = self.get_texture(&normal);
        let (u, v) = self.get_uv(&point_on_surface, &normal);

        Intersect {
            point: point_on_surface,
            normal,
            distance: t_min,
            material: Material::new(
                [1.0, 1.0, 1.0, 1.0],  // albedo
                [255, 255, 255],       // diffuse_color
                0.5,                   // specular
                1.0,                   // refractive_index
                Some(texture.clone())  // texture
            ), 
            is_intersecting: true,
            u: Some(u),
            v: Some(v),
        }
    }
}
