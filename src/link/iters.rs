use crate::{
    Point,
    constants::HALF,
    link::get_line_width,
    utils::{FullBox, angle_needs_normalization, full_box_from},
};

pub struct LineIter {
    pub start: Point,
    pub end: Point,
    pub distance: Point,
    pub line_width: f32,
    pub total: usize,
    pub pos: usize,
    pub init: Point,
}

impl LineIter {
    pub fn new(src: Point, dst: Point, full_width: f32, total: usize) -> Self {
        let ((nw, ne, sw, se), (d, _, angle)) = full_box_from(&src, &dst, full_width);
        let (width, inital_scale, scale) = get_line_width(total, full_width);

        let (distance, left, right) = match angle_needs_normalization(angle) {
            false => (d.scale(-2.0), sw, se),
            true => (d.scale(2.0), nw, ne),
        };
        let init = distance.scale(inital_scale);
        let chunk = distance.scale(scale);
        let start = left.add_distance(&init);
        let end = right.add_distance(&init);
        Self {
            start,
            end,
            distance: chunk,
            init,
            total,
            pos: 0,
            line_width: width * HALF,
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
