use graphics;
use primitives::BufferPoint;

use std::error::Error;
use std::path::Path;

extern crate image;
use self::image::{DynamicImage, RgbaImage, ImageRgba8};

use graphics::types;

/// Maps an f32 in [0_f32, 1.0] to [0_u8, 255]
#[inline]
pub fn piston_color_channel_to_byte(f: f32) -> u8 {
    (f * 255.0) as u8
}

pub struct RgbaTexture {
    width : u32,
    height : u32,
    buffer : Vec<u8>,
}

impl RgbaTexture {
    pub fn empty(width : u32, height : u32) -> RgbaTexture {
        let buff_size = width as usize * height as usize * 4;
        let mut buff : Vec<u8> = Vec::with_capacity(buff_size);
        buff.resize(buff_size, 0);
        RgbaTexture {
            width : width,
            height : height,
            buffer : buff,
        }
    }

    pub fn from_bytes(width : u32, height : u32, bytes : &[u8]) -> RgbaTexture {
        let buff : Vec<u8> = bytes.to_vec();
        RgbaTexture {
            width : width,
            height : height,
            buffer : buff,
        }
    }

    pub fn from_piston_image(rgba_image : RgbaImage) -> RgbaTexture {
        let width = rgba_image.width();
        let height = rgba_image.height();

        let buff : Vec<u8> = rgba_image.into_raw();

        RgbaTexture {
            width : width, 
            height : height,
            buffer : buff
        }
    }

    pub fn from_file_path(path : impl AsRef<Path>) -> Result<RgbaTexture, String> {
        let found_image : DynamicImage = image::open(path).map_err(|e| e.description().to_owned())?;
        let rgba_image : RgbaImage = if let ImageRgba8(im) = found_image { im } else { found_image.to_rgba() };
        Ok(RgbaTexture::from_piston_image(rgba_image))
    }


    pub fn vertex_to_pixel_coords(&self, v: [f32; 2]) -> BufferPoint {
        let vx = v[0];
        let vy = v[1];
        // it seems that the vertices are in a space where 0,0 is the center of the screen and
        // negative y is up.
        // translate into pixel where 0,0 is top left
        let x = if vx < -(self.width as f32) / 2.0 {
            0
        } else if vx > self.width as f32 / 2.0 {
            self.width - 1
        } else {
            (vx + self.width as f32 / 2.0) as u32
        };
        let y = if vy < -(self.height as f32) / 2.0 {
            0
        } else if vy > self.height as f32 / 2.0 {
            self.height - 1
        } else {
            (vy + self.height as f32 / 2.0) as u32
        };
        assert!(x < self.width);
        assert!(y < self.height);
        BufferPoint::new(x as usize, y as usize)
    }

    pub fn get_pixel(&self, x : u32, y : u32) -> [u8 ; 4] {
        if x > self.width || y > self.height {
            return [0, 0, 0, 0];
        }

        let begin_idx = (y * self.width + x) as usize * 4;

        if begin_idx > self.buffer.len() {
            return [0, 0, 0, 0];
        }

        let r = self.buffer[begin_idx + 0];
        let g = self.buffer[begin_idx + 1];
        let b = self.buffer[begin_idx + 2];
        let a = self.buffer[begin_idx + 3];

        [r, g, b, a]
    }

    pub fn put_pixel(&mut self, x : u32, y : u32, color : &types::Color) { 
        let red = piston_color_channel_to_byte(color[0]);
        let blue = piston_color_channel_to_byte(color[1]);
        let green = piston_color_channel_to_byte(color[2]);
        let alpha = piston_color_channel_to_byte(color[3]);

        let index = x as usize * y as usize *4;

        self.buffer[index + 0] = red;
        self.buffer[index + 1] = blue;
        self.buffer[index + 2] = green;
        self.buffer[index + 3] = alpha;
    }
}

impl graphics::ImageSize for RgbaTexture {
    fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}