pub mod bsp;
pub mod constants;
pub mod diagram;
pub mod imgcache;
pub mod link;
pub mod node;
pub mod render;
pub mod square;
pub mod utils;
use std::ops::Add;

use js_sys::Function;
use wasm_bindgen::prelude::*;

use crate::{
    constants::{
        DEFAULT_ANIMATION_COLOR, DEFAULT_ANIMATION_DASHES, DEFAULT_COLOR, DEFAULT_FONT_COLOR,
        DEFAULT_FONT_FAMILY, DEFAULT_HIGHLIGHT_ALPHA, DEFAULT_HIGHLIGHT_COLOR,
        DEFAULT_HOVER_TIMEOUT, DEFAULT_IDX_STEP, DEFAULT_LINK_SCALE, DEFAULT_SCREEN_ZOOM,
        FRAME_TICK, GRID_COLOR, GRID_DIVIDER_WIDTH, GRID_LINE_WIDTH, GRID_SIZE, GRID_SLOTS, HALF,
        MAX_K, MIN_K, NODE_FONT_SCALE, ONE_THIRD,
    },
    utils::to_map_xy,
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
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Add for Point {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
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
    pub animation_dashes: Vec<f64>,
    pub highlight_alpha: f32,
    pub highlight_color: String,
    pub link_scale: f32,
    pub callback: Option<Function>,
    pub index_step: i64,
    pub node_font_scale: f32,
    pub animation_color: String,
    pub frame_tick: f32,
    pub interactive: bool,
    pub wheel_move: f32,
    pub min_k: f32,
    pub max_k: f32,
    pub font_color: String,
    pub grid_opt: Option<GridOpt>,
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
            node_font_scale: NODE_FONT_SCALE,
            animation_color: String::from(DEFAULT_ANIMATION_COLOR),
            frame_tick: FRAME_TICK,
            grid_opt: None,
        }
    }
}
