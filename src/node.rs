use crate::square::Square;
use std::{cmp::Ordering, hash::Hash};
use wasm_bindgen::prelude::*;
#[wasm_bindgen(inspectable, getter_with_clone)]
#[derive(Clone, Debug)]
pub struct Node {
    pub layout: Square,
    pub label: String,
    pub opt: u32,
    pub id: u32,
    pub groups: Vec<u32>,
}

#[wasm_bindgen]
impl Node {
    #[wasm_bindgen(constructor)]
    pub fn new(id: u32, layout: Square, label: String, opt: u32, groups: Vec<u32>) -> Self {
        Self {
            label,
            layout,
            opt,
            id,
            groups,
        }
    }
}
impl Hash for Node {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Node {}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let cmp = self.id.cmp(&other.id);
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
