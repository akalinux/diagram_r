pub mod bsp;
pub mod constants;
pub mod diagram;
pub mod imgcache;
pub mod link;
pub mod node;
pub mod render;
pub mod square;
pub mod utils;
use std::{
    fmt::{Display, Formatter},
    ops::Add,
};

use js_sys::Function;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::{
    constants::{
        DEFAULT_ANIMATION_COLOR, DEFAULT_ANIMATION_DASHES, DEFAULT_COLOR, DEFAULT_FONT_COLOR,
        DEFAULT_FONT_FAMILY, DEFAULT_FRAMERATE, DEFAULT_HIGHLIGHT_ALPHA, DEFAULT_HIGHLIGHT_COLOR,
        DEFAULT_HOVER_TIMEOUT, DEFAULT_IDX_STEP, DEFAULT_LINK_SCALE, DEFAULT_SCREEN_ZOOM,
        GRID_COLOR, GRID_DIVIDER_WIDTH, GRID_LINE_WIDTH, GRID_SIZE, GRID_SLOTS, HALF, MAX_K, MIN_K,
        ONE_THIRD,
    },
    utils::{get_distance, get_radians, get_xy_r, to_map_xy},
};

#[wasm_bindgen]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
    pub k: f32,
}

#[wasm_bindgen]
impl Transform {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f32, y: f32, k: f32) -> Self {
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
#[wasm_bindgen(inspectable)]
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Eq for Point {}

impl Add for Point {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

#[wasm_bindgen]
impl Point {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}
impl Point {
    pub fn abs(&self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
        }
    }
    pub fn get_radians(&self, a: &Self) -> f32 {
        get_radians(self.x, self.y, a.x, a.y)
    }
    pub fn get_degree(&self, a: &Self) -> f32 {
        self.get_radians(a).to_degrees()
    }
    pub fn get_center(&self, other: &Self) -> Self {
        Self {
            x: (self.x + other.x) * HALF,
            y: (self.y + other.y) * HALF,
        }
    }
    pub fn get_z_center(&self, a: &Self, b: &Self) -> Point {
        Self {
            x: (self.x + a.x + b.x) * ONE_THIRD,
            y: (self.y + a.y + b.y) * ONE_THIRD,
        }
    }
    pub fn distance(&self, b: &Self) -> f32 {
        get_distance(self.x, self.y, b.x, b.y)
    }
    pub fn to_map_xy(&self, t: &Transform) -> Self {
        to_map_xy(&self, t)
    }
    /// Using self as the starting point, how far did we move to get to: p?
    pub fn get_move_distance(&self, p: &Self) -> Self {
        p.sub_distance(self)
    }
    pub fn add_distance(&self, distance: &Point) -> Point {
        Self::new(self.x + distance.x, self.y + distance.y)
    }
    pub fn sub_distance(&self, distance: &Point) -> Point {
        Self::new(self.x - distance.x, self.y - distance.y)
    }

    pub fn scale(&self, scale: f32) -> Self {
        Self {
            x: self.x * scale,
            y: self.y * scale,
        }
    }

    pub fn get_xy(&self, r: f32, rad: f32) -> Point {
        get_xy_r(self.x, self.y, r, rad)
    }

    pub fn get_point(&self, dst: &Self, r: f32, offset_rad: f32) -> Point {
        let rad = self.get_radians(dst) + offset_rad;
        self.get_xy(r, rad)
    }

    pub fn get_center_x(&self, b: &Self) -> f32 {
        (self.x + b.x) * HALF
    }
    pub fn get_center_y(&self, b: &Self) -> f32 {
        (self.y + b.y) * HALF
    }

    pub fn slope(&self, b: &Self) -> f32 {
        let x = self.x - b.x;
        match x == 0.0 {
            true => return 0.0,
            false => (self.y - b.y) / x,
        }
    }
    pub fn get_distance_vec(&self, dst: &Self, r: f32, offset_rad: f32) -> Point {
        let p = self.get_point(dst, r, offset_rad);
        self.get_move_distance(&p)
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

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);

}

#[wasm_bindgen(getter_with_clone)]
pub struct DiagramOpt {
    pub timeout: i32,
    pub font_family: String,
    pub animation_dashes: Vec<i32>,
    pub highlight_alpha: f32,
    pub highlight_color: String,
    pub link_scale: f32,
    pub callback: Option<Function>,
    pub index_step: i64,
    pub animation_color: String,
    pub animate: bool,
    pub interactive: bool,
    pub wheel_move: f32,
    pub min_k: f32,
    pub max_k: f32,
    pub font_color: String,
    pub grid_opt: Option<GridOpt>,
    pub frame_rate: u32,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct GridOpt {
    pub grid_size: u32,
    pub grid_slots: u32,
    pub grid_color: String,
    pub grid_line_width: f32,
    pub grid_divider_width: f32,
}

#[wasm_bindgen]
impl GridOpt {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            grid_size: GRID_SIZE,
            grid_slots: GRID_SLOTS,
            grid_line_width: GRID_LINE_WIDTH,
            grid_divider_width: GRID_DIVIDER_WIDTH,
            grid_color: String::from(GRID_COLOR),
        }
    }
}

#[wasm_bindgen]
impl DiagramOpt {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            wheel_move: DEFAULT_SCREEN_ZOOM,
            min_k: MIN_K,
            max_k: MAX_K,
            timeout: DEFAULT_HOVER_TIMEOUT,
            font_family: String::from(DEFAULT_FONT_FAMILY),
            font_color: String::from(DEFAULT_FONT_COLOR),
            animation_dashes: Vec::from(DEFAULT_ANIMATION_DASHES),
            highlight_alpha: DEFAULT_HIGHLIGHT_ALPHA,
            highlight_color: String::from(DEFAULT_HIGHLIGHT_COLOR),
            interactive: true,
            callback: None,
            link_scale: DEFAULT_LINK_SCALE,
            index_step: DEFAULT_IDX_STEP,
            animation_color: String::from(DEFAULT_ANIMATION_COLOR),
            grid_opt: None,
            frame_rate: DEFAULT_FRAMERATE,
            animate: true,
        }
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "(X: {0:.2}, Y: {1:.2})", self.x, self.y)
    }
}
