use crate::object::Object;
use crate::vec3::Vec3;
use crate::ray::Ray;
use crate::light::Light;
use crate::material::Material;
use crate::camera::Camera;

pub struct Scene {
    pub objects: Vec<Object>,
    pub lights: Vec<Light>,
    camera: Camera
}

impl Scene {
    pub fn new(camera: Camera) -> Self {

        Scene {

            camera,

            objects: Vec::new(),
            lights: Vec::new(),
        }
    }

    pub fn add_object(&mut self, object: Object) {
        self.objects.push(object);
    }
    pub fn set_camera(&mut self, camera: Camera) {
        self.camera = camera;
    }

    pub fn add_light(&mut self, light: Light) {
        self.lights.push(light);
    }

    fn trace_ray(&self, ray: &Ray, _depth: u32) -> Vec3 {
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

    fn compute_lighting(&self, _point: &Vec3, light_dir: &Vec3, normal: &Vec3) -> f32 {
        normal.dot(*light_dir).max(0.0)
    }
       
    pub fn render(&self, width: u32, height: u32, buffer: &mut [u32]) {
            // Lógica de renderizado aquí
            // Por ejemplo, podrías recorrer cada píxel y calcular el color basado en los objetos en la escena y las luces
    
            for j in 0..height {
                for i in 0..width {
                    let color = Vec3::new(0.0, 0.0, 0.0); // Aquí iría el cálculo del color basado en ray tracing
                    let index = (j * width + i) as usize;
                    buffer[index] = (color.x * 255.0) as u32 | ((color.y * 255.0) as u32) << 8 | ((color.z * 255.0) as u32) << 16;
                }
            }
        }
    }
pub struct HitRecord {
    pub point: Vec3,
    pub normal: Vec3,
    pub t: f32,
    pub material: Material,
}
