use nalgebra_glm::Vec3;
use crate::material::Material;

#[derive(Debug, Clone)]
pub struct Intersect {
    pub is_intersecting: bool,
    pub distance: f32,
    pub point: Vec3,
    pub normal: Vec3,
    pub material: Material,
    pub u: Option<f32>,
    pub v: Option<f32>,
}

impl Intersect {
    pub fn empty() -> Self {
        Intersect {
            is_intersecting: false,
            distance: f32::INFINITY,
            point: Vec3::zeros(),
            normal: Vec3::zeros(),
            material: Material::new(None),
            u: None,
            v: None,
        }
    }
}

pub trait RayIntersect {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Intersect;
}
