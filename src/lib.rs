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
        DEFAULT_ANIMATION_COLOR, DEFAULT_ANIMATION_DASHES, DEFAULT_COLOR, DEFAULT_FONT_FAMILY,
        DEFAULT_HIGHLIGHT_ALPHA, DEFAULT_HIGHLIGHT_COLOR, DEFAULT_HIGHLIGHT_SCALE,
        DEFAULT_HOVER_TIMEOUT, DEFAULT_IDX_STEP, DEFAULT_LINK_SCALE, DEFAULT_SCREEN_ZOOM,
        DEFAULT_TEXT_ALIGN, FRAME_TICK, NODE_FONT_SCALE,
    },
    utils::to_map_xy,
};

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
#[wasm_bindgen(inspectable)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
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
    pub fn add_distance(&self, distance: &Point) -> Point {
        Self::new(self.x + distance.x, self.y + distance.y)
    }
    pub fn sub_distance(&self, distance: &Point) -> Point {
        Self::new(self.x - distance.x, self.y - distance.y)
    }

    pub fn scale(&self, scale: f64) -> Self {
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

#[wasm_bindgen(inspectable, getter_with_clone)]
#[derive(Clone, Debug)]
pub struct DiagramOpt {
    pub wheel_move: f64,
    pub timeout: i32,
    pub font_family: String,
    pub text_align: String,
    pub animation_dashes: Vec<f64>,
    pub highlight_alpha: f64,
    pub highlight_color: String,
    pub highlight_scale: f64,
    pub bulk_img_update: bool,
    pub link_scale: f64,
    pub callback: Option<Function>,
    pub index_step: i64,
    pub node_font_scale: f64,
    pub animation_color: String,
    pub frame_tick: f64,
    pub interactive: bool,
}

#[wasm_bindgen]
impl DiagramOpt {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            wheel_move: DEFAULT_SCREEN_ZOOM,
            timeout: DEFAULT_HOVER_TIMEOUT,
            font_family: String::from(DEFAULT_FONT_FAMILY),
            text_align: String::from(DEFAULT_TEXT_ALIGN),
            animation_dashes: Vec::from(DEFAULT_ANIMATION_DASHES),
            highlight_alpha: DEFAULT_HIGHLIGHT_ALPHA,
            highlight_color: String::from(DEFAULT_HIGHLIGHT_COLOR),
            highlight_scale: DEFAULT_HIGHLIGHT_SCALE,
            bulk_img_update: true,
            interactive: true,
            callback: None,
            link_scale: DEFAULT_LINK_SCALE,
            index_step: DEFAULT_IDX_STEP,
            node_font_scale: NODE_FONT_SCALE,
            animation_color: String::from(DEFAULT_ANIMATION_COLOR),
            frame_tick: FRAME_TICK,
        }
    }
}
