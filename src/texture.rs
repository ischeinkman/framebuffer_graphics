use graphics;
use primitives::BufferPoint;

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
}

impl graphics::ImageSize for RgbaTexture {
    fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}