
use nalgebra_glm::Vec3;

pub struct Light {
    pub position: Vec3,
    pub color: [u8; 3],
    pub intensity: f32,
}

impl Light {
    pub fn new(position: Vec3, color: [u8; 3], intensity: f32) -> Self {
        Light {
            position,
            color,
            intensity,
        }
    }
}
