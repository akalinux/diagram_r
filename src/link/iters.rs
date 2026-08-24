use crate::{
    Point,
    constants::{HALF, NINTY_DEGREES},
    link::get_line_width,
    square::Corners,
    utils::{get_radians, get_xy_r, invert_dst},
};

pub struct LineIter {
    pub start: Point,
    pub end: Point,
    pub distance: Point,
    pub width: f32,
    pub total: usize,
    pub pos: usize,
    pub init: Point,
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
impl LineIter {
    pub fn shared(
        src: &Point,
        dst: &Point,
        full_width: f32,
        total: usize,
        width: f32,
        inital_scale: f32,
        scale: f32,
    ) -> Self {
        let r = full_width * HALF;
        let rad = get_radians(src.x, src.y, dst.x, dst.y);
        let north = rad + NINTY_DEGREES;
        let left = get_xy_r(src.x, src.y, r, north);

        let d = left.get_move_distance(src);

        let right = dst.sub_distance(&d);
        let distance = d.scale(2.0);

        let init = distance.scale(inital_scale);
        let chunk = distance.scale(scale);
        let start = left.add_distance(&init);
        let end = right.add_distance(&init);
        Self {
            start,
            end,
            distance: chunk,
            init,
            total: total - 1,
            pos: 0,
            width,
        }
    }
    pub fn new(src: &Point, dst: &Point, full_width: f32, total: usize) -> Self {
        let (width, inital_scale, scale) = get_line_width(total, full_width);
        Self::shared(src, dst, full_width, total, width, inital_scale, scale)
    }
}

impl Iterator for LineIter {
    type Item = (Point, Point);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos > self.total {
            return None;
        }
        let i = self.pos as f32;
        self.pos += 1;
        let d = self.distance.scale(i);
        Some((self.start.add_distance(&d), self.end.add_distance(&d)))
    }
}

pub struct ArcIter {
    pub a: LineIter,
    pub b: LineIter,
    pub init: Point,
}

impl ArcIter {
    pub fn new(begin: &Point, center: &Point, end: &Point, full_width: f32, total: usize) -> Self {
        let (width, inital_scale, scale) = get_line_width(total, full_width);
        let (a_end, rad_a) = invert_dst(begin, center, full_width);
        let a = LineIter::shared(begin, &a_end, full_width, total, width, inital_scale, scale);
        let (b_end, _) = invert_dst(center, end, full_width);

        let b = LineIter::shared(&b_end, end, full_width, total, width, inital_scale, scale);

        let center_start = get_xy_r(a_end.x, a_end.y, full_width, rad_a + NINTY_DEGREES);
        let d = center_start.get_move_distance(center);
        let init = d.scale(inital_scale);

        Self { a, b, init }
    }
}
impl Iterator for ArcIter {
    type Item = ((Point, Point), (Point, Point));

    fn next(&mut self) -> Option<Self::Item> {
        match (self.a.next(), self.b.next()) {
            (Some(a), Some(b)) => Some((a, b)),
            _ => None,
        }
    }
}
