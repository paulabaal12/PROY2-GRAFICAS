use crate::texture::Texture;
use crate::vec3::Vec3;

#[derive(Clone)]
pub struct Material {
    pub albedo: Vec3,
    pub specular: f32,
    pub transparency: f32,
    pub reflectivity: f32,
    pub texture: Option<Texture>,
}

impl Material {
    pub fn new(albedo: Vec3, specular: f32, transparency: f32, reflectivity: f32, texture_path: Option<&str>) -> Self {
        let texture = texture_path.map(|path| Texture::load(path));
        Material { albedo, specular, transparency, reflectivity, texture }
    }
}