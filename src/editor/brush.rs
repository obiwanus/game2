pub struct Brush {
    pub size: f32,
    pixels: Vec<u8>, // @speed: swizzle?
    width: usize,
    height: usize,
}

impl Brush {
    pub fn new(img_path: &str) -> Self {
        let image = stb_image::load_u8(img_path, 1, false).unwrap();
        Brush {
            size: 5.0,
            pixels: image.data,
            width: image.width,
            height: image.height,
        }
    }

    // TODO
    pub fn sample_at(u: f32, v: f32) {}
}
