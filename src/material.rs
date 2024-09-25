
use image::{DynamicImage, GenericImageView};
#[derive(Debug)]

pub struct Material {
    pub diffuse: [u8; 3],
    pub specular: f32,
    pub albedo: [f32; 4],
    pub refractive_index: f32,
    pub texture: Option<DynamicImage>,
}

impl Material {
    pub fn new(
        diffuse: [u8; 3],
        specular: f32,
        albedo: [f32; 4],
        refractive_index: f32,
        texture_path: Option<&str>,
    ) -> Self {
        let texture = texture_path.map(|path| {
            image::open(path).expect("Failed to load texture")
        });
        
        Material {
            diffuse,
            specular,
            albedo,
            refractive_index,
            texture,
        }
    }

    pub fn get_color(&self, u: f32, v: f32) -> [u8; 3] {
        if let Some(ref texture) = self.texture {
            let x = (u * texture.width() as f32) as u32 % texture.width();
            let y = (v * texture.height() as f32) as u32 % texture.height();
            let pixel = texture.get_pixel(x, y);
            [pixel[0], pixel[1], pixel[2]]
        } else {
            self.diffuse
        }
    }


  pub fn black() -> Self {
      Material {
          diffuse: [0, 0, 0],
          specular: 0.0,
          albedo: [0.0, 0.0, 0.0, 0.0],
          refractive_index: 1.0,
          texture: None,
      }
  }
}
