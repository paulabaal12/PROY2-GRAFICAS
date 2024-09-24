use crate::object::{Object, Sphere, Cube};
use crate::vec3::Vec3;
use crate::ray::Ray;
use crate::light::Light;
use crate::material::Material;
use crate::camera::Camera; // Importar la cámara

pub struct Scene {
    pub objects: Vec<Object>,
    pub lights: Vec<Light>,
    pub camera: Camera, // Incluir el campo de la cámara
}

impl Scene {
    pub fn new(camera: Camera) -> Self { // Modificar el método new para aceptar un parámetro de cámara
        Scene {
            objects: Vec::new(),
            lights: Vec::new(),
            camera, // Inicializar el campo de la cámara
        }
    }

    pub fn add_object(&mut self, object: Object) {
        self.objects.push(object);
    }

    pub fn add_light(&mut self, light: Light) {
        self.lights.push(light);
    }

    pub fn render(&self, width: u32, height: u32, buffer: &mut [u32]) {
        for (i, pixel) in buffer.iter_mut().enumerate() {
            let x = (i % width as usize) as f32 / width as f32;
            let y = (i / width as usize) as f32 / height as f32;
            let ray = self.camera.get_ray(x, y);
            let color = self.trace_ray(&ray, 0);
            *pixel = ((color.x * 255.0) as u32) << 16 | ((color.y * 255.0) as u32) << 8 | ((color.z * 255.0) as u32);
        }
    }

    fn trace_ray(&self, ray: &Ray, depth: u32) -> Vec3 {
        if let Some(hit) = self.hit_objects(ray) {
            let mut color = Vec3::new(0.0, 0.0, 0.0);
            for light in &self.lights {
                let light_dir = (light.position - hit.point).normalize();
                let light_intensity = self.compute_lighting(&hit.point, &light_dir, &hit.normal);
                color = color + hit.material.albedo.element_wise_mul(light.color) * light_intensity;
            }
            color
        } else {
            Vec3::new(0.5, 0.7, 1.0) // Color de fondo
        }
    }

    fn hit_objects(&self, ray: &Ray) -> Option<HitRecord> {
        let mut closest_hit = None;
        let mut closest_dist = f32::MAX;

        for object in &self.objects {
            if let Some(hit) = object.hit(ray, 0.001, closest_dist) {
                closest_dist = hit.t;
                closest_hit = Some(hit);
            }
        }

        closest_hit
    }

    fn compute_lighting(&self, point: &Vec3, light_dir: &Vec3, normal: &Vec3) -> f32 {
        normal.dot(*light_dir).max(0.0)
    }
}

pub struct HitRecord {
    pub point: Vec3,
    pub normal: Vec3,
    pub t: f32,
    pub material: Material,
}
