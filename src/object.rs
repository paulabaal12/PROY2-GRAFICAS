use crate::vec3::Vec3;
use crate::ray::Ray;
use crate::material::Material;
use crate::scene::HitRecord;

pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub material: Material,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, material: Material) -> Self {
        Sphere { center, radius, material }
    }

    pub fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let oc = ray.origin - self.center;
        let a = ray.direction.dot(ray.direction);
        let b = oc.dot(ray.direction);
        let c = oc.dot(oc) - self.radius * self.radius;
        let discriminant = b * b - a * c;

        if discriminant > 0.0 {
            let temp = (-b - discriminant.sqrt()) / a;
            if temp < t_max && temp > t_min {
                let point = ray.at(temp);
                return Some(HitRecord {
                    t: temp,
                    point,
                    normal: (point - self.center) / self.radius,
                    material: self.material.clone(),
                });
            }
        }
        None
    }
}

pub struct Cube {
    pub min: Vec3,
    pub max: Vec3,
    pub material: Material,
}

impl Cube {
    pub fn new(min: Vec3, max: Vec3, material: Material) -> Self {
        Cube { min, max, material }
    }

    pub fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let inv_d = Vec3::new(1.0 / ray.direction.x, 1.0 / ray.direction.y, 1.0 / ray.direction.z);

        let t0x = (self.min.x - ray.origin.x) * inv_d.x;
        let t1x = (self.max.x - ray.origin.x) * inv_d.x;
        let t0y = (self.min.y - ray.origin.y) * inv_d.y;
        let t1y = (self.max.y - ray.origin.y) * inv_d.y;
        let t0z = (self.min.z - ray.origin.z) * inv_d.z;
        let t1z = (self.max.z - ray.origin.z) * inv_d.z;

        let t_min = t0x.min(t1x).max(t0y.min(t1y)).max(t0z.min(t1z)).max(t_min);
        let t_max = t0x.max(t1x).min(t0y.max(t1y)).min(t0z.max(t1z)).min(t_max);

        if t_max > t_min {
            Some(HitRecord {
                t: t_min,
                point: ray.at(t_min),
                normal: (ray.at(t_min) - self.min).normalize(), // Simplificación
                material: self.material.clone(),
            })
        } else {
            None
        }
    }
}

pub enum Object {
    Sphere(Sphere),
    Cube(Cube),
}

impl Object {
    pub fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        match self {
            Object::Sphere(sphere) => sphere.hit(ray, t_min, t_max),
            Object::Cube(cube) => cube.hit(ray, t_min, t_max),
        }
    }
}
