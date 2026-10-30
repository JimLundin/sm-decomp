use std::ffi::{c_char, c_void};

#[link(name = "sm_stb_image", kind = "static", modifiers = "-bundle")]
unsafe extern "C" {
    fn stbi_load(
        filename: *const c_char,
        width: *mut i32,
        height: *mut i32,
        channels: *mut i32,
        requested_channels: i32,
    ) -> *mut u8;
    fn stbi_image_free(pixels: *mut c_void);
}

pub struct TextureImage {
    pub width: usize,
    pub height: usize,
    pub channels: usize,
    pub pixels: Vec<u8>,
}

unsafe fn image(pixels: *mut u8, width: i32, height: i32, channels: i32) -> Option<TextureImage> {
    unsafe {
        if pixels.is_null() {
            return None;
        }
        let result = TextureImage {
            width: width as usize,
            height: height as usize,
            channels: channels as usize,
            pixels: std::slice::from_raw_parts(pixels, (width * height * channels) as usize)
                .to_vec(),
        };
        stbi_image_free(pixels.cast());
        Some(result)
    }
}

pub(super) unsafe fn load(filename: *const std::ffi::c_char) -> Option<TextureImage> {
    unsafe {
        let (mut width, mut height, mut channels) = (0, 0, 0);
        let pixels = stbi_load(filename, &mut width, &mut height, &mut channels, 0);
        image(pixels, width, height, channels)
    }
}
