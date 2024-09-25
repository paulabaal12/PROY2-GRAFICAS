use image::RgbaImage;

#[derive(Debug)]
pub struct Texture {
    image: RgbaImage,
}

impl Texture {
    pub fn new(path: &str) -> Self {
        let img = image::open(path).expect("Failed to load texture").to_rgba8();
        let flipped_img = image::imageops::flip_vertical(&img);
        Texture { image: flipped_img }
    }

    pub fn get_color(&self, u: f32, v: f32) -> [u8; 3] {
        let x = ((u * self.image.width() as f32) as u32).min(self.image.width() - 1);
        let y = ((v * self.image.height() as f32) as u32).min(self.image.height() - 1);
        let pixel = self.image.get_pixel(x, y);
        [pixel[0], pixel[1], pixel[2]]
    }
}