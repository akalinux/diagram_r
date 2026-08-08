pub mod bsp;
pub mod constants;
pub mod diagram;
pub mod imgcache;
pub mod link;
pub mod node;
pub mod render;
pub mod square;
pub mod utils;
use wasm_bindgen::prelude::*;

use crate::{constants::DEFAULT_COLOR, utils::to_map_xy};

#[wasm_bindgen]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Transform {
    pub x: f64,
    pub y: f64,
    pub k: f64,
}

#[wasm_bindgen]
impl Transform {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f64, y: f64, k: f64) -> Self {
        Self { x, y, k }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum LabelPosition {
    Top,
    Center,
    Bottom,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug)]
pub struct ElementOpt {
    pub img: String,
    pub color: String,
    pub label_position: LabelPosition,
}
#[wasm_bindgen]
impl ElementOpt {
    #[wasm_bindgen(constructor)]
    pub fn new(img: String, color: String, label_position: LabelPosition) -> Self {
        Self {
            img,
            label_position,
            color,
        }
    }
}

impl ElementOpt {
    pub fn defaults() -> Self {
        return Self {
            img: String::from(""),
            color: String::from(DEFAULT_COLOR),
            label_position: LabelPosition::Top,
        };
    }
}
#[wasm_bindgen()]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    pub fn to_map_xy(&self, t: &Transform) -> Self {
        to_map_xy(&self, t)
    }
    /// Using self as the starting point, how far did we move to get to: p?
    pub fn get_move_distance(&self, p: &Self) -> Self {
        Self {
            x: p.x - self.x,
            y: p.y - self.y,
        }
    }
    pub fn move_distance(&self, distance: &Point) -> Point {
        Self::new(self.x + distance.x, self.y + distance.y)
    }

    pub fn idx(&self, step: i64) -> (i64, i64) {
        let mut x = self.x as i64;
        let mut y = self.y as i64;
        for i in [&mut x, &mut y] {
            let m = *i % step;
            if m < 0 {
                *i -= step + m;
            } else {
                *i -= m;
            }
        }
        (x, y)
    }
}
