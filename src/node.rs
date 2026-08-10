use crate::square::Square;
use std::{cmp::Ordering, hash::Hash};
use wasm_bindgen::prelude::*;
#[wasm_bindgen(inspectable, getter_with_clone)]
#[derive(Clone, Debug)]
pub struct Node {
    pub layout: Square,
    pub label: String,
    pub opt: usize,
    pub groups: Vec<u32>,
}

#[wasm_bindgen]
impl Node {
    #[wasm_bindgen(constructor)]
    pub fn new(layout: Square, label: String, opt: usize, groups: Vec<u32>) -> Self {
        Self {
            label,
            layout,
            opt,
            groups,
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
