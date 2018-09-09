use std::fmt;
use graphics::{types};
use texture::{RgbaTexture, piston_color_channel_to_byte};

use back_end::RgbaBufferGraphics;

/// A point in 2d space
#[derive(Clone, Copy)]
pub struct BufferPoint {
    pub x: usize,
    pub y: usize,
}

impl BufferPoint {
    pub fn new(x: usize, y: usize) -> BufferPoint {
        BufferPoint {
            x,
            y,
        }
    }
}

impl fmt::Debug for BufferPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

/// A triangle in pixel coordinates
pub struct Triangle {
    // vertices ordered by increasing y (top to bottom) then x coordinate (left to right)
    pub vertices: [BufferPoint; 3]
}

impl Triangle {
    pub fn new(v1: BufferPoint, v2: BufferPoint, v3: BufferPoint) -> Triangle {
        let buf = [v1, v2, v3];
        Triangle {
            vertices: buf
        }
    }
    pub fn render(&self, graphics: &mut RgbaBufferGraphics, color: &types::Color) {
        let mapper = |_x : usize, _y : usize| {
            [
                piston_color_channel_to_byte(color[0]),
                piston_color_channel_to_byte(color[1]),
                piston_color_channel_to_byte(color[2]),
                piston_color_channel_to_byte(color[3]),
            ]
        };

        render_triangle(self.vertices, graphics, &mapper)
    }
}

pub struct TextureTriangle<'tri> {
    pub vertices : [BufferPoint ; 3],
    pub texture_vertices : [BufferPoint ; 3],
    pub texture : &'tri RgbaTexture,
}

impl <'tri> TextureTriangle<'tri> {
    pub fn new<'a>(v1: (BufferPoint, BufferPoint), v2: (BufferPoint, BufferPoint), v3: (BufferPoint, BufferPoint),  texture : &'a RgbaTexture) -> TextureTriangle<'a> {
        let buf = [v1, v2, v3];
        let verts = [buf[0].0, buf[1].0, buf[2].0];
        let text_verts = [buf[0].1, buf[1].1, buf[2].1];

        let retval = TextureTriangle {
            vertices : verts,
            texture_vertices : text_verts,
            texture : texture
        };
        retval
    }
    
    pub fn render(&self, graphics: &mut RgbaBufferGraphics, _color: &types::Color) {
       
        // We map the texture to the framebuffer by first reinterpretting the framebuffer
        // coordinate in terms of 2 basis vectors from the framebuffer triangle, 
        // translating that to the equivalent basis for the texture's triangle, 
        // and then reinterpretting those coordinates to the texture's regular coordinates. 

        //First, we get signed integers to make things easier. 
        let v1_x = self.vertices[0].x as i64; 
        let v1_y = self.vertices[0].y as i64; 
        let v2_x = self.vertices[1].x as i64; 
        let v2_y = self.vertices[1].y as i64; 
        let v3_x = self.vertices[2].x as i64; 
        let v3_y = self.vertices[2].y as i64; 

        let t1_x = self.texture_vertices[0].x as i64;
        let t1_y = self.texture_vertices[0].y as i64;
        let t2_x = self.texture_vertices[1].x as i64;
        let t2_y = self.texture_vertices[1].y as i64;
        let t3_x = self.texture_vertices[2].x as i64;
        let t3_y = self.texture_vertices[2].y as i64;

        // If the "triangle" is just a single mapped point, draw the point immediatly. 
        if v1_x == v2_x && v2_x == v3_x && v1_y == v2_y && v2_y == v3_y {
            let pixel_index = graphics.coords_to_pixel_index(&BufferPoint::new(self.vertices[0].x, self.vertices[0].y));
            let pixel_coords = (t1_x as u32, t1_y as u32);
            let clr = self.texture.get_pixel(pixel_coords.0, pixel_coords.1);
            graphics.write_color_bytes(pixel_index, clr);

        }

        let mapper = { 
            // The coordinates for the framebuffer triangle's basis vectors
            let va_x = v1_x - v3_x;
            let va_y = v1_y - v3_y;
            let vb_x = v2_x - v3_x;
            let vb_y = v2_y - v3_y;

            // the coordinates of the texture triangle's basis vectors
            let ta_x = t1_x - t3_x;
            let ta_y = t1_y - t3_y;
            let tb_x = t2_x - t3_x;
            let tb_y = t2_y - t3_y;
            
            move |framebuffer_x : usize, framebuffer_y : usize | {
                
                let a_coeff_bottom = (va_x * va_x + va_y + va_y) as f64;
                let a_coeff_top = (va_x * (framebuffer_x as i64 - v3_x) + va_y * (framebuffer_y as i64 - v3_y)) as f64;
                let a_coeff = a_coeff_top/a_coeff_bottom;

                let b_coeff_bottom = (vb_x * vb_x + vb_y + vb_y) as f64;
                let b_coeff_top = (vb_x * (framebuffer_x as i64 - v3_x) + vb_y * (framebuffer_y as i64 - v3_y)) as f64;
                let b_coeff = b_coeff_top/b_coeff_bottom;
                

                let x_component = a_coeff.ceil() as i64 * ta_x + b_coeff.ceil() as i64 * tb_x + t3_x;
                let y_component = a_coeff.ceil() as i64 * ta_y + b_coeff.ceil() as i64 * tb_y + t3_y;

                self.texture.get_pixel(x_component as u32, y_component as u32)
            }
        };

        render_triangle(self.vertices, graphics, &mapper)
    }
}

#[inline]
fn render_triangle(points : [BufferPoint ; 3], graphics : &mut RgbaBufferGraphics, pixel_mapper : &Fn(usize, usize) -> [u8 ; 4]) {

    println!("T: ({}, {}), ({}, {}), ({}, {})", points[0].x, points[0].y, points[1].x, points[1].y, points[2].x, points[2].y);

    // Sort the points into bottom, middle, and top.
    let (bottom, mid, top) = if points[0].y < points[1].y && points[0].y < points[2].y {
        if points[1].y < points[2].y {
            (points[0], points[1], points[2])
        }
        else {
            (points[0], points[2], points[1])
        }
    }
    else if points[1].y < points[0].y && points[1].y < points[2].y {
        if points[0].y < points[2].y {
            (points[1], points[0], points[2])
        }
        else {
            (points[1], points[2], points[0])
        }
    }
    else {
        if points[0].y < points[1].y {
            (points[2], points[0], points[1])
        }
        else {
            (points[2], points[1], points[0])
        }
    };

    let mut al = 0;
    let mut ap = 0;
    // We split the larger triangle into 2 along the middle point's y line.
    if bottom.y != mid.y {

        // We create a new coordinate system centered at the bottom point so that we 
        // can just use the inverse slopes to calculate the borders of this y value's x line.
        let inv_slope_btm_mid : f32 = (bottom.x as f32 - mid.x as f32) /(bottom.y as f32 - mid.y as f32);
        let inv_slope_btm_top : f32 = (bottom.x as f32 - top.x as f32)/(bottom.y as f32 - top.y as f32);
        for y in bottom.y .. mid.y {
            let shifted_y = y as f32 - bottom.y as f32;

            let x1 = ((shifted_y * inv_slope_btm_mid) + bottom.x as f32).round() as usize;
            let x2 = ((shifted_y * inv_slope_btm_top) + bottom.x as f32).round() as usize;

            let (xmax, xmin) = if x1 > x2 {(x1, x2)} else {(x2, x1)};
            let idx_min = graphics.coords_to_pixel_index(&BufferPoint::new(xmin, y));
            let idx_max = graphics.coords_to_pixel_index(&BufferPoint::new(xmax, y));
            println!("A {}: {} -> {} => {} -> {}", y, xmin, xmax, idx_min, idx_max);
            al+=1;
            for x in xmin - 1 .. xmax + 1 {
                ap+=1;
                let idx = graphics.coords_to_pixel_index(&BufferPoint::new(x, y));
                let clr = pixel_mapper(x, y);
                graphics.write_color_bytes(idx, clr);
            }
        }
    }

    let mut bl = 0;
    let mut bp = 0;
    if mid.y != top.y {

        // Same thing as before, except for the upper sub-triangle we center the system on the top point.
        let inv_slope_top_mid : f32 = (top.x as f32 - mid.x as f32) /(top.y as f32 - mid.y as f32);
        let inv_slope_top_bottom : f32 = (bottom.x as f32 - top.x as f32)/(bottom.y as f32 - top.y as f32);
        for y in mid.y .. top.y {
            let shifted_y = y as f32 - top.y as f32;

            let x1 = ((shifted_y * inv_slope_top_bottom) + top.x as f32).round() as usize;
            let x2 = ((shifted_y * inv_slope_top_mid) + top.x as f32).round() as usize;

            let (xmax, xmin) = if x1 > x2 {(x1, x2)} else {(x2, x1)};
            let idx_min = graphics.coords_to_pixel_index(&BufferPoint::new(xmin, y));
            let idx_max = graphics.coords_to_pixel_index(&BufferPoint::new(xmax, y));
            println!("B {}: {} -> {} => {} -> {}", y, xmin, xmax, idx_min, idx_max);

            bl += 1;
            for x in xmin -1 .. xmax + 1 {
                let idx = graphics.coords_to_pixel_index(&BufferPoint::new(x, y));
                let clr = pixel_mapper(x, y);
                graphics.write_color_bytes(idx, clr);
                bp += 1;
            }
        }
    }
    println!("T fin: {}+{} = {}, {}+{} = {}.", al, bl, al+bl, ap, bp, ap+bp);
}