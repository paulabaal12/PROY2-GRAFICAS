use crate::vec3::Vec3;

pub struct Material {
    pub albedo: Vec3,
    pub diffuse: f32,
    pub specular: f32,
    pub shininess: f32,
    pub texture: Option<crate::texture::Texture>,
}

impl Material {
    pub fn new(albedo: Vec3, diffuse: f32, specular: f32, shininess: f32, texture: Option<crate::texture::Texture>) -> Self {
        Material {
            albedo,
            diffuse,
            specular,
            shininess,
            texture,
        }
    }
}
