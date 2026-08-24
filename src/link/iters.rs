use crate::{
    Point,
    constants::HALF,
    link::get_line_width,
    square::Corners,
    utils::{angle_needs_normalization, get_angle, get_xy},
};

pub struct LineIter {
    pub start: Point,
    pub end: Point,
    pub distance: Point,
    pub width: f32,
    pub total: usize,
    pub pos: usize,
    pub init: Point,
    pub angle: f32,
    pub normalized: bool,
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
    pub fn new(src: &Point, dst: &Point, full_width: f32, total: usize) -> Self {
        let r = full_width * HALF;
        let angle = get_angle(src.x, src.y, dst.x, dst.y);
        let north = angle + 90.0;
        let nw = get_xy(src.x, src.y, r, north);
        let d = nw.get_move_distance(src);
        let (width, inital_scale, scale) = get_line_width(total, full_width);

        let (distance, left, right, normalized) = match angle_needs_normalization(angle) {
            false => (
                d.scale(-2.0),
                src.add_distance(&d),
                dst.add_distance(&d),
                false,
            ),
            true => (d.scale(2.0), nw, dst.sub_distance(&d), true),
        };
        let init = distance.scale(inital_scale);
        let chunk = distance.scale(scale);
        let start = left.add_distance(&init);
        let end = right.add_distance(&init);
        Self {
            angle,
            start,
            end,
            distance: chunk,
            init,
            total: total - 1,
            pos: 0,
            width,
            normalized,
        }
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
