use std::f32::consts::PI;

use crate::{Point, Transform};

/// Used to slightly oversize link boxes for mouse over intersections preventing misses.
pub const AREA_SCALE_EPSILON: f32 = 1.00004;

/// Half a circle
pub const RAD2DEG: f32 = 180.0 / PI;
/// Full circle in radians.
pub const FULL_CIRCLE: f32 = 2.0 * PI;

/// Default height and width of a node
pub const DEFAULT_NODE_R: f32 = 12.0;

/// how much to scale a link down to relative to the size of a node.
pub const DEFAULT_LINK_SCALE: f32 = 0.85;

/// Default name for options.
pub const DEFAULT_OPT_NAME: &'static str = "defaults";

/// Default color used for nodesand links.
pub const DEFAULT_COLOR: &'static str = "rgb(0, 131, 138)";
// Default color used for bundles
pub const DEFAULT_BUNDLE_COLOR: &'static str = "rgb(225, 143, 56)";
pub const DEFAULT_HOVER_TIMEOUT: i32 = 300;

// default animation color
pub const DEFAULT_ANIMATION_COLOR: &'static str = "rgb(147, 25, 204)";
pub const DEFAULT_HIGHLIGHT_COLOR: &'static str = "#3c3c3c";
pub const DEFAULT_FONT_COLOR: &'static str = "#3c3c3c";
pub const DEFAULT_DIV_STYLE: &'static str =
    "position: relative; height: 100%;widht: 100%;box-sizing: border-box;overflow: clip;";

pub const DEFAULT_CANVAS_STYLE: &'static str =
    "position: absolute;box-sizing: border-box;overflow: clip;";
pub const DEFAULT_ANIMATION_DASHES: [f64; 2] = [5.0, 15.0];
pub const DEFAULT_ANIMATION_WIDTH_SCALE: f32 = 1.0 / 3.0;

pub const ZERO_POINT: Point = Point { x: 0.0, y: 0.0 };

pub const SCREEN_EPSILON: f32 = 0.001;

pub const ZERO_TRANSFORM: Transform = Transform {
    x: 0.0,
    y: 0.0,
    k: 1.0,
};

pub const DEFAULT_SCREEN_ZOOM: f32 = 0.05;
pub const DEFAULT_HIGHLIGHT_SCALE: f32 = 1.10;
pub const DEFAULT_HIGHLIGHT_ALPHA: f32 = 0.5;
pub const DEFAULT_FONT_FAMILY: &'static str = "24px Arial";
pub const NODE_FONT_SCALE: f32 = 0.6;
pub const DEFAULT_TEXT_ALIGN: &'static str = "center";
/// This is the size of the index in pixels.
/// Most monitors modern monitors default to width: 1920, height: 1080, so we assum half the width, or 960.
pub const DEFAULT_IDX_STEP: i64 = 960;
pub const NODE_ADD_ERROR: &'static str = "Cannot Add duplicate Nodes";
pub const LINK_ADD_ERROR: &'static str = "Cannot Add Link to Box Layer";
pub const DEFAULT_ELEMENT_ID: &'static str = "Diagram-r";

pub const CANVAS_ERROR: &'static str = "Unknown error manipulating canvas";
pub const WINDOW_ERROR: &'static str = "No global `window` exists";
pub const DOM_ERROR: &'static str = "No `document` exists";
pub const EL_ERROR: &'static str = "No Element by the given `id`, exists";
pub const LINK_NODE_MISSING_ERROR: &'static str = "Cannot add Link to node that does not exist";
pub const LINK_BUNDLE_MISSING_ERROR: &'static str = "Cannot add Bundle to node that does not exist";
pub const FRAME_TICK: f32 = 0.01;

pub const MAX_K: f32 = 10.0;
pub const MIN_K: f32 = 0.0;
