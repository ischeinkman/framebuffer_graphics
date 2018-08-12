use graphics;

/// Maps an f32 in [0_f32, 1.0] to [0_u8, 255]
#[inline]
pub fn piston_color_channel_to_byte(f: f32) -> u8 {
    (f * 255.0) as u8
}


//TODO: textures
pub struct RgbaTexture {}
impl graphics::ImageSize for RgbaTexture {
    fn get_size(&self) -> (u32, u32) {
        (0, 0)
    }
}