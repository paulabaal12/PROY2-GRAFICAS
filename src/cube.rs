use nalgebra_glm::{Vec3, dot};
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::material::Material;

pub struct Cube {
    pub min: Vec3,
    pub max: Vec3,
    pub materials: [Material; 6],
}

impl Cube {
    fn get_uv(&self, point: &Vec3, normal: &Vec3) -> (f32, f32) {
        let u;
        let v;

        if normal.x.abs() > 0.5 {
            u = (point.z - self.min.z) / (self.max.z - self.min.z);
            v = (point.y - self.min.y) / (self.max.y - self.min.y);
        } else if normal.y.abs() > 0.5 {
            u = (point.x - self.min.x) / (self.max.x - self.min.x);
            v = (point.z - self.min.z) / (self.max.z - self.min.z);
        } else {
            u = (point.x - self.min.x) / (self.max.x - self.min.x);
            v = (point.y - self.min.y) / (self.max.y - self.min.y);
        }

        (u, v)
    }

    fn intersect_plane(&self, origin: &Vec3, direction: &Vec3, plane_point: &Vec3, plane_normal: &Vec3) -> Option<Vec3> {
        let denom = dot(&plane_normal, direction);
        if denom.abs() > 1e-6 {
            let t = dot(&(plane_point - origin), plane_normal) / denom;
            if t >= 0.0 {
                return Some(origin + direction * t);
            }
        }
        None
    }
    
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, origin: &Vec3, direction: &Vec3) -> Intersect {
        let mut closest_intersect = Intersect::empty();
        let mut min_distance = f32::INFINITY;

        let normals = [
            Vec3::new(1.0, 0.0, 0.0),  // Right face
            Vec3::new(-1.0, 0.0, 0.0), // Left face
            Vec3::new(0.0, 1.0, 0.0),  // Top face
            Vec3::new(0.0, -1.0, 0.0), // Bottom face
            Vec3::new(0.0, 0.0, 1.0),  // Front face
            Vec3::new(0.0, 0.0, -1.0), // Back face
        ];

        let points = [
            Vec3::new(self.max.x, 0.0, 0.0),  // Right face
            Vec3::new(self.min.x, 0.0, 0.0),  // Left face
            Vec3::new(0.0, self.max.y, 0.0),  // Top face
            Vec3::new(0.0, self.min.y, 0.0),  // Bottom face
            Vec3::new(0.0, 0.0, self.max.z),  // Front face
            Vec3::new(0.0, 0.0, self.min.z),  // Back face
        ];

        for i in 0..6 {
            if let Some(intersect_point) = self.intersect_plane(origin, direction, &points[i], &normals[i]) {
                if intersect_point.x >= self.min.x && intersect_point.x <= self.max.x &&
                   intersect_point.y >= self.min.y && intersect_point.y <= self.max.y &&
                   intersect_point.z >= self.min.z && intersect_point.z <= self.max.z {
                    
                    let distance = dot(&(intersect_point - origin), direction);
                    if distance < min_distance {
                        min_distance = distance;
                        closest_intersect = Intersect {
                            is_intersecting: true,
                            distance,
                            point: intersect_point,
                            normal: normals[i],
                            u: None,
                            v: None,
                            material: self.materials[i].clone(),
                        };

                        let (u, v) = self.get_uv(&intersect_point, &normals[i]);
                        closest_intersect.u = Some(u);
                        closest_intersect.v = Some(v);
                    }
                }
            }
        }

        closest_intersect
    }
}
