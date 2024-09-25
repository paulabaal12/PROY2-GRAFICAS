use std::rc::Rc;
use crate::texture::Texture;

#[derive(Clone)]
#[derive(Debug)]

pub struct Material {
    pub albedo: [f32; 4],
    pub diffuse_color: [u8; 3],
    pub specular: f32,
    pub refractive_index: f32,
    pub texture: Option<Rc<Texture>>,
}

impl Material {
    pub fn new(
        albedo: [f32; 4],
        diffuse_color: [u8; 3],
        specular: f32,
        refractive_index: f32,
        texture: Option<Rc<Texture>>,
    ) -> Self {
        Material {
            albedo,
            diffuse_color,
            specular,
            refractive_index,
            texture,
        }
    }

    pub fn default() -> Self {
        Material {
            albedo: [1.0, 0.0, 0.0, 0.0],
            diffuse_color: [255, 0, 0],
            specular: 50.0,
            refractive_index: 1.0,
            texture: None,
        }
    }

    pub fn get_texture_color(&self, u: f32, v: f32) -> [u8; 3] {
        if let Some(texture) = &self.texture {
            texture.get_color(u, v)
        } else {
            self.diffuse_color
        }
    }
}
