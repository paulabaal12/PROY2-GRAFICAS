use image::{DynamicImage, GenericImageView, Rgba};

#[derive(Clone)]
#[derive(Debug)]
pub struct Material {
    pub texture: Option<DynamicImage>,
    pub specular: f32,
    pub albedo: [f32; 4],
    pub refractive_index: f32,
    }

impl Material {
    pub fn new(texture: Option<DynamicImage>,
        specular: f32,
        albedo: [f32; 4],
        refractive_index: f32, ) -> Self {
        Material { texture ,
            specular,
            albedo,
            refractive_index,}
        
    }

    pub fn get_texture_color(&self, u: f32, v: f32) -> Rgba<u8> {
        if let Some(ref texture) = self.texture {
            let (width, height) = texture.dimensions();
            let x = (u * width as f32) as u32 % width;
            let y = (v * height as f32) as u32 % height;
            texture.get_pixel(x, y)
        } else {
            Rgba([255, 255, 255, 255])
        }
    }
}
