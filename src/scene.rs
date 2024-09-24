use crate::object::{Object, Sphere, Cube};
use crate::vec3::Vec3;
use crate::camera::Ray;
use crate::texture::Texture;

pub struct Light {
    pub position: Vec3,
    pub color: Vec3,
}

impl Light {
    pub fn new(position: Vec3, color: Vec3) -> Self {
        Light { position, color }
    }
}

pub struct Scene {
    pub objects: Vec<Object>,
    pub lights: Vec<Light>,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            objects: Vec::new(),
            lights: Vec::new(),
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
            let color = self.trace_ray(x, y);
            *pixel = ((color.x * 255.0) as u32) << 16 | ((color.y * 255.0) as u32) << 8 | ((color.z * 255.0) as u32);
        }
    }

    fn trace_ray(&self, u: f32, v: f32) -> Vec3 {
        // Aquí iría el código para calcular el color del píxel utilizando raytracing
        // Por ahora, devolvemos un color de ejemplo
        Vec3::new(0.5, 0.5, 0.5) // Color de ejemplo
    }
}