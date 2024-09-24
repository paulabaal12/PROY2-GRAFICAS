use crate::material::Material;
use crate::vec3::Vec3;

pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub material: Material,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, material: Material) -> Self {
        Sphere { center, radius, material }
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
}

pub enum Object {
    Sphere(Sphere),
    Cube(Cube),
}