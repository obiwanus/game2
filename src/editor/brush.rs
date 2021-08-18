use stb_image::{Image, LoadResult};

pub struct Brush {
    pub size: f32,
    pixels: Vec<u8>, // @speed: swizzle?
    width: usize,
    height: usize,
}

impl Brush {
    pub fn new(img_path: &str) -> Self {
        let image = load_image(img_path, false);
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

// @duplication: merge with texture.rs/load_image?
pub fn load_image(path: &str, flip: bool) -> Image<u8> {
    let flip = if flip { 1 } else { 0 };
    unsafe {
        stb_image::bindings::stbi_set_flip_vertically_on_load(flip);
    }
    match stb_image::load_with_depth(path, 1, false) {
        LoadResult::ImageU8(image) => image,
        LoadResult::ImageF32(_) => panic!(
            "Couldn't load brush {}. Only U8 grayscale brush images are supported.",
            path
        ),
        LoadResult::Error(msg) => panic!("Couldn't load brush {}: {}", path, msg),
    }
}
