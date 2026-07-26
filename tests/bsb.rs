#![cfg(test)]

use diagram_r::bsp::{IdxBoxAction, IdxBoxIter};

#[test]
fn iter_box_tests() {
    let mut iter = IdxBoxIter::new((1..=1, 1..=1), (1..=1, 1..=1), 1);
    assert!(iter.next().is_none());
    iter = IdxBoxIter::new((1..=2, 1..=2), (2..=3, 2..=3), 1);
    assert_eq!(iter.next(), Some((1, 1, IdxBoxAction::Remove)));
    assert_eq!(iter.next(), Some((2, 1, IdxBoxAction::Remove)));
    assert_eq!(iter.next(), Some((1, 2, IdxBoxAction::Remove)));
    assert_eq!(iter.next(), Some((3, 2, IdxBoxAction::Add)));
    assert_eq!(iter.next(), Some((2, 3, IdxBoxAction::Add)));
    assert_eq!(iter.next(), Some((3, 3, IdxBoxAction::Add)));
    assert!(iter.next().is_none());
}
