use nalgebra_glm::{Vec3, vec3};
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::material::Material;

pub struct Cube {
    pub min: Vec3,
    pub max: Vec3,
    pub material: Material,
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, orig: &Vec3, dir: &Vec3) -> Intersect {
        let inv_dir = vec3(1.0, 1.0, 1.0).component_div(dir);
        let t0s = (self.min - orig).component_mul(&inv_dir);
        let t1s = (self.max - orig).component_mul(&inv_dir);

        let t_min = t0s.zip_map(&t1s, |a, b| a.min(b));
        let t_max = t0s.zip_map(&t1s, |a, b| a.max(b));

        let t_near = t_min.max();
        let t_far = t_max.min();

        if t_near > t_far || t_far < 0.0 {
            return Intersect::empty();
        }

        let t_hit = t_near;
        let hit_point = orig + dir * t_hit;
        let normal = self.get_normal(&hit_point);

        // Coordenadas de textura simples basadas en la posición del punto de impacto
        let u = (hit_point.x - self.min.x) / (self.max.x - self.min.x);
        let v = (hit_point.y - self.min.y) / (self.max.y - self.min.y);
        let color = self.material.get_color(u, v);

        Intersect {
            is_intersecting: true,
            distance: t_hit,
            point: hit_point,
            normal,
            material: Material {
                diffuse: color,
                ..self.material.clone()
            },
        }
    }
}

impl Cube {
    fn get_normal(&self, point: &Vec3) -> Vec3 {
        if (point.x - self.min.x).abs() < 1e-4 {
            return Vec3::new(-1.0, 0.0, 0.0);
        }
        if (point.x - self.max.x).abs() < 1e-4 {
            return Vec3::new(1.0, 0.0, 0.0);
        }
        if (point.y - self.min.y).abs() < 1e-4 {
            return Vec3::new(0.0, -1.0, 0.0);
        }
        if (point.y - self.max.y).abs() < 1e-4 {
            return Vec3::new(0.0, 1.0, 0.0);
        }
        if (point.z - self.min.z).abs() < 1e-4 {
            return Vec3::new(0.0, 0.0, -1.0);
        }
        if (point.z - self.max.z).abs() < 1e-4 {
            return Vec3::new(0.0, 0.0, 1.0);
        }
        Vec3::zeros()
    }
}

impl Clone for Material {
    fn clone(&self) -> Self {
        Self {
            diffuse: self.diffuse,
            specular: self.specular,
            albedo: self.albedo,
            refractive_index: self.refractive_index,
            texture: self.texture.clone(),
        }
    }
}
