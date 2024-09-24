use crate::vec3::Vec3;
use crate::material::Material;
use crate::texture::Texture;
use std::rc::Rc;
use crate::ray::Ray;
use crate::scene::HitRecord;

pub struct Cube {
    pub min: Vec3,
    pub max: Vec3,
    pub top_texture: Rc<Texture>,
    pub side_texture: Rc<Texture>,
    pub bottom_texture: Rc<Texture>,
    pub material: Material,
}

impl Cube {
    pub fn new(min: Vec3, max: Vec3, material: Material) -> Self {
        Cube {
            min,
            max,
            top_texture: Rc::new(Texture::default()),
            side_texture: Rc::new(Texture::default()),
            bottom_texture: Rc::new(Texture::default()),
            material,
        }
    }

    pub fn new_with_textures(
        min: Vec3,
        max: Vec3,
        top_texture: Rc<Texture>,
        side_texture: Rc<Texture>,
        bottom_texture: Rc<Texture>,
    ) -> Self {
        Cube {
            min,
            max,
            top_texture,
            side_texture,
            bottom_texture,
            material: Material::new(Vec3::new(0.5, 0.5, 0.5), 0.5, 0.0, 0.0, None),
        }
    }

    pub fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {

        None
    }
}
