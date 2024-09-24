use image::{DynamicImage, GenericImageView, Pixel};
use std::path::Path;

#[derive(Clone)]
pub struct Texture {
    data: DynamicImage,
}

impl Texture {
    pub fn load(path: &str) -> Self {
        let img = image::open(Path::new(path)).expect("Failed to load texture");
        Texture { data: img }
    }

    pub fn default() -> Self {
        Texture {
            data: DynamicImage::new_rgba8(1, 1),
        }
    }

    pub fn value(&self, u: f32, v: f32) -> [u8; 4] {
        let (width, height) = self.data.dimensions();
        let x = (u * width as f32) as u32;
        let y = ((1.0 - v) * height as f32) as u32;
        self.data.get_pixel(x.min(width - 1), y.min(height - 1)).0
    }
}
