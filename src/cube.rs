use nalgebra_glm::Vec3;
use std::rc::Rc;
use image::DynamicImage; 
use crate::material::Material;
use crate::ray_intersect::{Intersect, RayIntersect};

pub struct Cube {
    pub min: Vec3,
    pub max: Vec3,
    pub topface_texture: Rc<DynamicImage>,
    pub sideface_texture: Rc<DynamicImage>,
    pub bottomface_texture: Rc<DynamicImage>,
}

impl Cube {
    fn get_uv(&self, point: &Vec3, normal: &Vec3) -> (f32, f32) {
        let (u, v) = if normal.y.abs() > 0.5 {  // Superior o inferior
            (
                (point.x - self.min.x) / (self.max.x - self.min.x),
                (point.z - self.min.z) / (self.max.z - self.min.z)
            )
        } else if normal.x.abs() > 0.5 {  // Lados (izquierda o derecha)
            (
                (point.z - self.min.z) / (self.max.z - self.min.z),
                (point.y - self.min.y) / (self.max.y - self.min.y)
            )
        } else {  // Frente o atrás
            (
                (point.x - self.min.x) / (self.max.x - self.min.x),
                (point.y - self.min.y) / (self.max.y - self.min.y)
            )
        };

        (u.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
    }

    fn intersect_plane(
        &self,
        origin: &Vec3,
        direction: &Vec3,
        plane_point: &Vec3,
        plane_normal: &Vec3
    ) -> Option<Vec3> {
        let denom = direction.dot(plane_normal);
        let epsilon = 1e-6; 
        if denom.abs() > epsilon {
            let t = (plane_point - origin).dot(plane_normal) / denom;
            if t >= 0.0 {
                return Some(origin + direction * t);
            }
        }
        None
    }
    
    fn get_material_for_face(&self, normal: &Vec3) -> Material {
        if normal.y > 0.5 {
            Material::new(Some((*self.topface_texture).clone()), 0.5, [1.0, 1.0, 1.0, 1.0], 1.0)
        } else if normal.y < -0.5 {
            Material::new(Some((*self.bottomface_texture).clone()), 0.5, [1.0, 1.0, 1.0, 1.0], 1.0)
        } else {
            Material::new(Some((*self.sideface_texture).clone()), 0.5, [1.0, 1.0, 1.0, 1.0], 1.0)
        }
    }
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, origin: &Vec3, direction: &Vec3) -> Intersect {
        let mut closest_intersect = Intersect::empty();
        let mut min_distance = f32::INFINITY;

        let normals = [
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(-1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, -1.0),
        ];

        let points = [
            Vec3::new(self.max.x, self.min.y, self.min.z),
            Vec3::new(self.min.x, self.min.y, self.min.z),
            Vec3::new(self.min.x, self.max.y, self.min.z),
            Vec3::new(self.min.x, self.min.y, self.min.z),
            Vec3::new(self.min.x, self.min.y, self.max.z),
            Vec3::new(self.min.x, self.min.y, self.min.z),
        ];

        for i in 0..6 {
            if let Some(intersect_point) = self.intersect_plane(origin, direction, &points[i], &normals[i]) {
                if intersect_point.x >= self.min.x && intersect_point.x <= self.max.x &&
                   intersect_point.y >= self.min.y && intersect_point.y <= self.max.y &&
                   intersect_point.z >= self.min.z && intersect_point.z <= self.max.z {
                    
                    let distance = (intersect_point - origin).magnitude();
                    if distance < min_distance {
                        min_distance = distance;
                        let (u, v) = self.get_uv(&intersect_point, &normals[i]);
                        closest_intersect = Intersect {
                            is_intersecting: true,
                            distance,
                            point: intersect_point,
                            normal: normals[i],
                            u: Some(u),
                            v: Some(v),
                            material: self.get_material_for_face(&normals[i]),
                        };
                    }
                }
            }
        }

        closest_intersect
    }
}