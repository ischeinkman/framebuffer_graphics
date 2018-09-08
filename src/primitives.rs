use std::fmt;
use std::cmp::Ordering;
use graphics::{types};
use texture::RgbaTexture;

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
        let mut buf = [v1, v2, v3];
        buf.sort_unstable_by(|p1, p2| {
            match p1.y.cmp(&p2.y) {
                // if y's are equal, use x
                Ordering::Equal => p1.x.cmp(&p2.x),
                cmp => cmp
            }
        });
        Triangle {
            vertices: buf
        }
    }
    pub fn render(&self, graphics: &mut RgbaBufferGraphics, color: &types::Color) {
        // We want to write in descending scan lines because that's how the memory is laid out.
        // For each scan line between the top and bottom of the triangle, we want to find out which
        // parts of the line fall between 2 edges of the triangle.
        // each edge in increasing y order
        let edges = [
            (self.vertices[0], self.vertices[1]),
            (self.vertices[0], self.vertices[2]),
            (self.vertices[1], self.vertices[2])];
        // TODO this rendering isn't quite right; the way rounding etc is handled skips a few pixels
        // For each edge, if it's horizontal, just draw it as a line.
        for i in 0..edges.len() {
            let edge = edges[i];
            if edge.0.y == edge.1.y {
                // the edge is horizontal so just draw it as a line
                for x in (edge.0.x)..(edge.1.x + 1) {
                    let pixel_index = graphics.coords_to_pixel_index(&BufferPoint::new(x, edge.0.y));
                    graphics.write_color(pixel_index, color);
                }
                continue
            }
            // it's not horizontal, so look for any edges after this one to find out if they
            // vertically overlap. (Only look forward to avoid drawing the same areas twice.)
            for j in (i + 1)..edges.len() {
                let other_edge = edges[j];
                if other_edge.0.y == other_edge.1.y {
                    // it's horizontal; it will get drawn later
                    continue
                }
                // because other_edge is later in the edge list, we know that its starting
                // vertex has a y no less than the first edge's y. So, the vertical overlap
                // will be from the second edge's min y to the lesser of the two max y's, which
                // might be the empty set if the smaller of the edge ends is before the other edge's
                // min y.
                let overlap_y_start = other_edge.0.y;
                let overlap_y_end = edge.1.y.min(other_edge.1.y);
                // Inverse slope: how many x units should we move for a unit of y.
                // Can't divide by zero because neither of these is horizontal.
                let edge_1_inv_slope = (edge.1.x as f64 - edge.0.x as f64) / (edge.1.y as f64 - edge.0.y as f64);
                let edge_2_inv_slope = (other_edge.1.x as f64 - other_edge.0.x as f64) / (other_edge.1.y as f64 - other_edge.0.y as f64);
                for y in overlap_y_start..(overlap_y_end + 1) {
                    let edge_1_x = (edge.0.x as f64 + ((y - edge.0.y) as f64 * edge_1_inv_slope)).round() as usize;
                    let edge_2_x = (other_edge.0.x as f64 + ((y - other_edge.0.y) as f64 * edge_2_inv_slope)).round() as usize;
                    let start_x = edge_1_x.min(edge_2_x);
                    let end_x = edge_1_x.max(edge_2_x);
                    for x in start_x..(end_x + 1) {
                        let pixel_index = graphics.coords_to_pixel_index(&BufferPoint::new(x, y));
                        graphics.write_color(pixel_index, color);
                    }
                }
            }
        }
    }
}

pub struct TextureTriangle<'tri> {
    pub vertices : [BufferPoint ; 3],
    pub texture_vertices : [BufferPoint ; 3],
    pub texture : &'tri RgbaTexture,
}

impl <'tri> TextureTriangle<'tri> {
    pub fn new<'a>(v1: (BufferPoint, BufferPoint), v2: (BufferPoint, BufferPoint), v3: (BufferPoint, BufferPoint),  texture : &'a RgbaTexture) -> TextureTriangle<'a> {
        let mut buf = [v1, v2, v3];

        //Sort in decreasing y, increasing x
        buf.sort_unstable_by(|p1, p2| {
            match p1.0.y.cmp(&p2.0.y) {
                // if y's are equal, use x
                Ordering::Equal => p1.0.x.cmp(&p2.0.x),

                //Otherwise, reverse the sort
                Ordering::Greater => Ordering::Less,
                Ordering::Less => Ordering::Greater,
            }
        });
        let verts = [buf[0].0, buf[1].0, buf[2].0];
        let text_verts = [buf[0].1, buf[0].1, buf[2].1];

        TextureTriangle {
            vertices : verts,
            texture_vertices : text_verts,
            texture : texture
        }
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

        println!("Mapping point ({}, {}) to texture point ({}, {}).", v1_x, v1_y, t1_x, t1_y);
        println!("Mapping point ({}, {}) to texture point ({}, {}).", v2_x, v2_y, t2_x, t2_y);
        println!("Mapping point ({}, {}) to texture point ({}, {}).", v3_x, v3_y, t3_x, t3_y);

        // The coordinates for the framebuffer triangle's basis vectors
        let va_x = v1_x - v3_x;
        let va_y = v1_y - v3_y;
        let vb_x = v2_x - v3_x;
        let vb_y = v2_y - v3_y;

        println!("Frambuffer basis: ({}, {}) x ({}, {}) at origin ({}, {})", va_x, va_y, vb_x, vb_y, v3_x, v3_y);

        // the coordinates of the texture triangle's basis vectors
        let ta_x = t1_x - t3_x;
        let ta_y = t1_y - t3_y;
        let tb_x = t2_x - t3_x;
        let tb_y = t2_y - t3_y;
        
        println!("Texture basis: ({}, {}) x ({}, {}) at origin ({}, {})", ta_x, ta_y, tb_x, tb_y, t3_x, t3_y);

        let mapper = | framebuffer_x : usize, framebuffer_y : usize | {
            
            println!("Mapping point ({}, {})", framebuffer_x, framebuffer_y);

            let a_coeff_bottom = (va_x * va_x + va_y + va_y) as f64;
            let a_coeff_top = (va_x * (framebuffer_x as i64 - v3_x) + va_y * (framebuffer_y as i64 - v3_y)) as f64;
            let a_coeff = a_coeff_top/a_coeff_bottom;

            println!("Got first term coefficient of {}.", a_coeff);
            
            let b_coeff_bottom = (vb_x * vb_x + vb_y + vb_y) as f64;
            let b_coeff_top = (vb_x * (framebuffer_x as i64 - v3_x) + vb_y * (framebuffer_y as i64 - v3_y)) as f64;
            let b_coeff = b_coeff_top/b_coeff_bottom;
            
            println!("Got second term coefficient of {}.", b_coeff);


            let x_component = a_coeff.ceil() as i64 * ta_x + b_coeff.ceil() as i64 * tb_x + t3_x;
            let y_component = a_coeff.ceil() as i64 * ta_y + b_coeff.ceil() as i64 * tb_y + t3_y;

            println!("Mapped ({}, {}) to texture point ({}, {})", framebuffer_x, framebuffer_y, x_component, y_component);


            (x_component as u32, y_component as u32)
        };

        //Same algorithm as the other triangle, only replacing the color with the mapping function.

        let edges = [
            (self.vertices[2], self.vertices[1]),
            (self.vertices[2], self.vertices[0]),
            (self.vertices[1], self.vertices[0]),
        ];
        for i in 0..edges.len() {

            let edge = edges[i];
            if edge.0.y == edge.1.y {
                for x in (edge.0.x)..(edge.1.x + 1) {
                    let pixel_index = graphics.coords_to_pixel_index(&BufferPoint::new(x, edge.0.y));
                    let pixel_coords = mapper(x, edge.0.y);
                    let clr = self.texture.get_pixel(pixel_coords.0, pixel_coords.1);
                    graphics.write_color_bytes(pixel_index, clr);
                }
                continue
            }

            for j in (i + 1)..edges.len() {
                let other_edge = edges[j];
                if other_edge.0.y == other_edge.1.y {
                    continue
                }
                let overlap_y_start = other_edge.0.y;
                let overlap_y_end = edge.1.y.min(other_edge.1.y);
                let edge_1_inv_slope = (edge.1.x as f64 - edge.0.x as f64) / (edge.1.y as f64 - edge.0.y as f64);
                let edge_2_inv_slope = (other_edge.1.x as f64 - other_edge.0.x as f64) / (other_edge.1.y as f64 - other_edge.0.y as f64);
                for y in overlap_y_start..(overlap_y_end + 1) {
                    let edge_1_x = (edge.0.x as f64 + ((y - edge.0.y) as f64 * edge_1_inv_slope)).round() as usize;
                    let edge_2_x = (other_edge.0.x as f64 + ((y - other_edge.0.y) as f64 * edge_2_inv_slope)).round() as usize;
                    let start_x = edge_1_x.min(edge_2_x);
                    let end_x = edge_1_x.max(edge_2_x);
                    for x in start_x..(end_x + 1) {
                        let pixel_index = graphics.coords_to_pixel_index(&BufferPoint::new(x, y));
                        let pixel_coords = mapper(x, edge.0.y);
                        let clr = self.texture.get_pixel(pixel_coords.0, pixel_coords.1);
                        graphics.write_color_bytes(pixel_index, clr);
                    }
                }
            }

        }
    }
}