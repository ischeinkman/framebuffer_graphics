use std::fmt;
use std::cmp::Ordering;
use graphics::types;

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