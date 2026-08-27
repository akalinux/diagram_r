use std::fmt::Display;

use crate::{
    Point,
    constants::{HALF, R_90, R_270},
    square::Corners,
    utils::get_intersection,
};

pub fn get_line_width(total_links: usize, full_width: f32) -> (f32, f32, f32) {
    let incremental_scale = 1.0 / total_links as f32;
    let (virtual_count, inital_scale) = match total_links {
        1 => (2.0, 0.5),
        _ => (total_links as f32 * 2.0 - 1.0, incremental_scale * 0.5),
    };
    let link_width = full_width / virtual_count;
    (link_width, inital_scale, incremental_scale)
}

pub struct FullBoxAccumulate(Option<(f32, f32, f32, f32)>);

impl FullBoxAccumulate {
    pub fn new() -> Self {
        FullBoxAccumulate(None)
    }
    pub fn step(&mut self, p: &Point) {
        match self.0.as_mut() {
            Some((min_x, max_x, min_y, max_y)) => {
                if *min_x > p.x {
                    *min_x = p.x
                }
                if *min_y > p.y {
                    *min_y = p.y
                }
                if *max_x < p.x {
                    *max_x = p.x
                }
                if *max_y < p.x {
                    *max_y = p.x
                }
            }

            None => self.0 = Some((p.x, p.x, p.y, p.y)),
        }
    }
    pub fn full_box_from(self) -> Corners {
        unsafe { self.0.unwrap_unchecked() }
    }
}

fn builder(
    src: &Point,
    dst: &Point,
    r: f32,
    inital_scale: f32,
    scale: f32,
    offset: f32,
) -> (Point, Point, Point) {
    let left = src.get_point(dst, r, offset);
    let d = left.get_move_distance(src);

    let distance = d.scale(2.0);

    let init = distance.scale(inital_scale);
    let chunk = distance.scale(scale);
    let start = left.add_distance(&init);
    (start, init, chunk)
}
pub struct LineIter {
    pub np: NextPointSet,
    pub width: f32,
    pub total: usize,
    pub pos: usize,
}

impl LineIter {
    pub fn new(src: &Point, dst: &Point, full_width: f32, total: usize) -> Self {
        let (width, inital_scale, scale) = get_line_width(total, full_width);
        let r = full_width * HALF;

        let np = NextPointSet::new(src, dst, r, inital_scale, scale, R_90);
        Self {
            np,
            total: total,
            pos: 0,
            width,
        }
    }
}

impl Iterator for LineIter {
    type Item = (Point, Point);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.total {
            let i = self.pos as f32;
            self.pos += 1;

            return Some(self.np.line(i));
        }
        None
    }
}

pub struct ArcIter {
    pub a: NextPointSet,
    pub b: NextPointSet,
    pub width: f32,
    pub pos: usize,
    pub total: usize,
}

#[derive(Debug)]
pub struct NextPointSet {
    pub root: Point,
    pub init: Point,
    pub chunk: Point,
    pub distance: Point,
}

impl Display for NextPointSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Root: {}, Init: {}, Chunk: {}, Distance: {}",
            self.root, self.init, self.chunk, self.distance,
        )
    }
}

impl NextPointSet {
    pub fn new(src: &Point, dst: &Point, r: f32, init_scale: f32, scale: f32, offset: f32) -> Self {
        let (root, init, chunk) = builder(src, dst, r, init_scale, scale, offset);
        Self {
            root,
            chunk,
            init,
            distance: src.get_move_distance(dst),
        }
    }
    pub fn point(&self, scale: f32) -> Point {
        self.root.add_distance(&self.chunk.scale(scale))
    }
    pub fn line(&self, scale: f32) -> (Point, Point) {
        let start = self.point(scale);
        let end = start.add_distance(&self.distance);
        (start, end)
    }
}

impl ArcIter {
    pub fn new(src: &Point, center: &Point, dst: &Point, full_width: f32, total: usize) -> Self {
        let (width, inital_scale, scale) = get_line_width(total, full_width);
        let r = full_width * HALF;

        let a = NextPointSet::new(src, center, r, inital_scale, scale, R_90);
        let b = NextPointSet::new(dst, center, r, inital_scale, scale, R_270);

        Self {
            width,
            pos: 0,
            total,
            a,
            b,
        }
    }
}

impl Iterator for ArcIter {
    type Item = (Point, Point, Point);

    fn next(&mut self) -> Option<Self::Item> {
        match self.pos < self.total {
            true => {
                let i = self.pos as f32;
                self.pos += 1;
                let a = self.a.line(i);

                let b = self.b.line(i);
                let c = match get_intersection(&a.0, &a.1, &b.0, &b.1) {
                    Some(c) => c,
                    None => a.0.get_center(&b.0),
                };
                return Some((a.0, c, b.0));
            }
            false => return None,
        }
    }
}
