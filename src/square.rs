use std::cmp::Ordering;

use wasm_bindgen::prelude::*;

use crate::{Point, bsp::IndexXY, constants::SCREEN_EPSILON};

#[wasm_bindgen(inspectable)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Square {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[wasm_bindgen]
impl Square {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

/// The corners of a right angled rectangle or square.
pub type Corners = (
    f64, // min_x
    f64, // max_x
    f64, // min_y
    f64, // max_y
);

impl PartialOrd for Square {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.x < other.x {
            return Some(Ordering::Less);
        } else if other.x < self.x {
            return Some(Ordering::Greater);
        } else if self.max_x() > other.max_x() {
            return Some(Ordering::Less);
        } else if other.max_x() > self.max_x() {
            return Some(Ordering::Greater);
        } else
        // if we got here.. then both min and max x are equal
        if self.y < other.y {
            return Some(Ordering::Less);
        } else if other.y < self.y {
            return Some(Ordering::Greater);
        } else if self.max_y() > other.max_y() {
            return Some(Ordering::Less);
        } else if other.max_y() > self.max_y() {
            return Some(Ordering::Less);
            // if we got here.. then x and y axis are equal.. we just sort based on id
        }
        Some(Ordering::Equal)
    }
}
impl Eq for Square {}
impl Ord for Square {
    fn cmp(&self, other: &Self) -> Ordering {
        unsafe { self.partial_cmp(other).unwrap_unchecked() }
    }
}
impl Square {
    pub fn corners(&self) -> Corners {
        (self.x, self.max_x(), self.y, self.max_y())
    }
    pub fn from(c: Corners) -> Self {
        Self {
            x: c.0,
            y: c.2,
            width: c.1 - c.0,
            height: c.3 - c.2,
        }
    }
    pub fn get_center(&self) -> Point {
        Point {
            x: self.x + (self.width * 0.5),
            y: self.y + (self.height * 0.5),
        }
    }
    pub fn area(&self) -> f64 {
        self.width * self.height
    }
    pub fn contains_point(&self, p: &Point) -> bool {
        let dx = (p.x - self.x).abs();
        let dy = (p.y - self.y).abs();
        return !(dx > self.width || dy > self.height);
    }
    pub fn max_x(&self) -> f64 {
        self.x + self.width
    }
    pub fn max_y(&self) -> f64 {
        self.y + self.height
    }

    pub fn smallest_side(&self, b: &Self) -> f64 {
        let mut min = self.width;

        for c in [self.height, b.width, b.height] {
            if c < min {
                min = c
            }
        }
        min
    }
    pub fn idx(&self, step: i64) -> IndexXY {
        let mut x = self.x.floor() as i64;
        let mut y = self.y.floor() as i64;
        let mut x2 = self.width.ceil() as i64 + x;
        let mut y2 = self.height.ceil() as i64 + y;
        for i in [&mut x, &mut y, &mut x2, &mut y2] {
            let m = *i % step;
            if m < 0 {
                *i -= step + m;
            } else {
                *i -= m;
            }
        }
        (x..=x2, y..=y2)
    }
    pub fn move_distance(&mut self, distance: &Point) {
        self.x += distance.x;
        self.y += distance.y;
    }

    pub fn get_distance(&self, p: &Point) -> Point {
        Point {
            x: p.x - self.x,
            y: p.y - self.y,
        }
    }

    pub fn center(&self, screen: &Self) -> Point {
        let inlay_point = screen.get_center();
        let host_point = self.get_center();
        let mut x: f64 = 0.0;
        let mut y: f64 = 0.0;
        for (inlay, host, host_size, t) in [
            (inlay_point.x, host_point.x, self.width * 0.5, &mut x),
            (inlay_point.y, host_point.y, self.height * 0.5, &mut y),
        ] {
            if inlay.abs() < SCREEN_EPSILON {
                if host.abs() < SCREEN_EPSILON {
                    *t = host_size;
                } else {
                    *t = host + host_size;
                }
            } else if host.abs() < SCREEN_EPSILON {
                let distance = host - inlay;
                *t = distance + inlay + host_size;
            } else {
                let scale = host / inlay;
                *t = inlay * scale;
            }
        }

        return Point { x, y };
    }
}
