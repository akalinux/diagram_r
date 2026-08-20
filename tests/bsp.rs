#![cfg(test)]

use diagram_r::bsp::iter::{IdxBoxAction, IdxBoxIter};
use wasm_bindgen_test::wasm_bindgen_test;

#[test]
#[wasm_bindgen_test]
fn iter_no_overlap() {
    let mut iter = IdxBoxIter::new((1..=1, 1..=1), (2..=2, 2..=2), 1);
    assert_eq!(iter.next(), Some((1, 1, IdxBoxAction::Remove,)));
    assert_eq!(iter.next(), Some((2, 2, IdxBoxAction::Add,)));
    assert!(iter.next().is_none());

    let mut iter = IdxBoxIter::new((1..=1, 1..=1), (3..=3, 3..=3), 1);
    assert_eq!(iter.next(), Some((1, 1, IdxBoxAction::Remove,)));
    assert_eq!(iter.next(), Some((3, 3, IdxBoxAction::Add,)));
    assert!(iter.next().is_none());
}
