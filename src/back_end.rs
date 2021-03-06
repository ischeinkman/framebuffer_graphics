use std::ptr;

use color::*;

use graphics::{self, types};

use primitives::{BufferPoint, Triangle};

pub struct RgbaBufferGraphics {
    width: usize,
    height: usize,
    buffer: *mut u8,
}


impl RgbaBufferGraphics {
    pub fn new(width: usize, height: usize, buffer: *mut u8) -> RgbaBufferGraphics {
        RgbaBufferGraphics {
            width,
            height,
            buffer,
        }
    }

    #[inline]
    pub fn coords_to_pixel_index(&self, p: &BufferPoint) -> usize {
        assert!(p.x < self.width);
        assert!(p.y < self.height);
        p.x + p.y * self.width
    }

    #[inline]
    pub fn write_color(&mut self, pixel_index: usize, color: &types::Color) {
        let red_new = piston_color_channel_to_byte(color[0]);
        let green_new = piston_color_channel_to_byte(color[1]);
        let blue_new = piston_color_channel_to_byte(color[2]);
        let alpha_new = piston_color_channel_to_byte(color[3]);
            
        let byte_index = pixel_index * 4;
        let pixel_loc = unsafe { self.buffer.offset(byte_index as isize) };

        let color = if alpha_new != 255 {
            let red_old : u8 = unsafe { ptr::read(pixel_loc.offset(0isize)) };
            let green_old : u8 = unsafe { ptr::read(pixel_loc.offset(1isize)) };
            let blue_old : u8 = unsafe { ptr::read(pixel_loc.offset(2isize)) };
            let alpha_old : u8 = unsafe { ptr::read(pixel_loc.offset(3isize)) };

            let alpha_new_frac = (alpha_new as f32)/(255f32);
            let alpha_old_frac = 1.0 - alpha_new_frac;
            
            let red = ((red_new as f32 * alpha_new_frac) + (red_old as f32 * alpha_old_frac)) as u8;
            let green = ((green_new as f32 * alpha_new_frac) + (green_old as f32 * alpha_old_frac)) as u8;
            let blue = ((blue_new as f32 * alpha_new_frac) + (blue_old as f32 * alpha_old_frac)) as u8;
            let alpha = if alpha_old > 255 - alpha_new { 255 } else { alpha_old + alpha_new};
            [red, green, blue, alpha]

        } else { [red_new, green_new, blue_new, alpha_new] };

        for idx in 0 .. 4 {
            unsafe {
                let buff_idx = pixel_loc.offset(idx as isize);
                ptr::write(buff_idx, color[idx]);
            }
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
            (vx + self.width as f32 / 2.0) as usize
        };
        let y = if vy < -(self.height as f32) / 2.0 {
            0
        } else if vy > self.height as f32 / 2.0 {
            self.height - 1
        } else {
            (vy + self.height as f32 / 2.0) as usize
        };
        assert!(x < self.width);
        assert!(y < self.height);
        BufferPoint::new(x, y)
    }
}


impl graphics::Graphics for RgbaBufferGraphics {
    type Texture = RgbaTexture;
    fn clear_color(&mut self, color: types::Color) {
        let num_pixels = self.width * self.height;
        for i in 0..num_pixels {
            self.write_color(i, &color);
        }
    }
    fn clear_stencil(&mut self, _value: u8) {
        //TODO:this
    }
    fn tri_list<F>(&mut self, _draw_state: &graphics::DrawState, color: &[f32; 4], mut f: F) where F: FnMut(&mut FnMut(&[[f32; 2]])) {
        f(&mut |verts: &[[f32; 2]]| {
            for t in 0..verts.len() / 3 {
                let v1 = verts[t * 3];
                let v2 = verts[t * 3 + 1];
                let v3 = verts[t * 3 + 2];
                let tri = Triangle::new(self.vertex_to_pixel_coords(v1),
                                        self.vertex_to_pixel_coords(v2),
                                        self.vertex_to_pixel_coords(v3));
                tri.render(self, color);
            }
        })
    }
    fn tri_list_uv<F>(&mut self, _draw_state: &graphics::DrawState, _color: &[f32; 4], _texture: &<Self as graphics::Graphics>::Texture, _f: F) where F: FnMut(&mut FnMut(&[[f32; 2]], &[[f32; 2]])) {
        //TODO:this
    }
}