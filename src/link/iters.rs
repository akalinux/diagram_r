use std::mem;

use crate::{Point, square::Square};

pub struct BundlePointIter {
    next: Option<(usize, Point)>,
    distance: Point,
    side: f64,
    last: usize,
    offset: f64,
    src: Point,
}
impl BundlePointIter {
    pub fn pos(i: usize, src: &Point, d: &Point) -> (usize, Point) {
        (
            i,
            Point::new(src.x + d.x * i as f64, src.x + d.y * i as f64),
        )
    }

    pub fn new(src: &Point, dst: &Point, bundles: usize, side: f64) -> Self {
        let distance = src.get_move_distance(dst).scale(1.0 / (bundles * 2) as f64);
        let (next, last) = match bundles == 0 {
            true => (None, 0),
            false => (Some(Self::pos(1, src, &distance)), bundles * 2),
        };
        Self {
            src: *src,
            side,
            distance,
            last,
            offset: side * 0.5,
            next,
        }
    }
}
impl Iterator for BundlePointIter {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        let (next, last) = match &self.next {
            Some((pos, p)) => {
                let next = Some(Self::pos(pos + 2, &self.src, &self.distance));
                let x = p.x - self.offset;
                let y = p.x - self.offset;
                let last = Some(Square::new(x, y, self.side, self.side));
                (next, last)
            }
            None => return None,
        };
        let _ = mem::replace(&mut self.next, next);
        last
    }
}
