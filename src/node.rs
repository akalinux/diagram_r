use crate::square::Square;
use std::{cmp::Ordering, hash::Hash};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(inspectable, getter_with_clone)]
#[derive(Clone, Debug)]
pub struct Node {
    pub layout: Square,
    pub label: String,
    pub opt: usize,
    pub nodes: Vec<usize>,
    pub boxes: Vec<usize>,
}

#[wasm_bindgen]
impl Node {
    #[wasm_bindgen(constructor)]
    pub fn new(
        layout: Square,
        label: String,
        opt: usize,
        nodes: Option<Vec<usize>>,
        boxes: Option<Vec<usize>>,
    ) -> Self {
        let nodes = match nodes {
            Some(nodes) => nodes,
            None => Vec::new(),
        };
        let boxes = match boxes {
            Some(nodes) => nodes,
            None => Vec::new(),
        };
        Self {
            label,
            layout,
            opt,
            nodes,
            boxes,
        }
    }
}
impl Hash for Node {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.label.hash(state);
    }
}
impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
    }
}
impl Eq for Node {}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let cmp = self.label.cmp(&other.label);
        match cmp {
            Ordering::Equal => Some(Ordering::Equal),
            _ => {
                let res = self.layout.cmp(&other.layout);
                match res {
                    Ordering::Equal => return Some(cmp),
                    _ => return Some(res),
                }
            }
        }
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        unsafe { self.partial_cmp(other).unwrap_unchecked() }
    }
}
