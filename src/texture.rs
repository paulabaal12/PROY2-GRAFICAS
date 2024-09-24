use image::DynamicImage;
use std::path::Path;

#[derive(Clone)]
pub struct Texture {
    pub data: DynamicImage,
}

impl Texture {
    pub fn load(path: &str) -> Self {
        let img = image::open(&Path::new(path)).expect("Failed to load texture");
        Texture { data: img }
    }
}
