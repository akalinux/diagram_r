#![cfg(test)]

use diagram_r::bsp::iter::{IdxBoxAction, IdxBoxIter};

#[test]
fn iter_box_same_area_tests() {
    let mut iter = IdxBoxIter::new((1..=1, 1..=1, 0.0), (1..=1, 1..=1, 0.0), 1);
    assert!(iter.next().is_none());
    iter = IdxBoxIter::new((1..=2, 1..=2, 0.0), (2..=3, 2..=3, 0.0), 1);
    assert_eq!(iter.next(), Some((1, 1, IdxBoxAction::Remove, 0.0)));
    assert_eq!(iter.next(), Some((2, 1, IdxBoxAction::Remove, 0.0)));
    assert_eq!(iter.next(), Some((1, 2, IdxBoxAction::Remove, 0.0)));
    assert_eq!(iter.next(), Some((3, 2, IdxBoxAction::Add, 0.0)));
    assert_eq!(iter.next(), Some((2, 3, IdxBoxAction::Add, 0.0)));
    assert_eq!(iter.next(), Some((3, 3, IdxBoxAction::Add, 0.0)));
    assert!(iter.next().is_none());
}

#[test]
fn iter_no_overlap() {
    let mut iter = IdxBoxIter::new((1..=1, 1..=1, 0.0), (2..=2, 2..=2, 0.0), 1);
    assert_eq!(iter.next(), Some((1, 1, IdxBoxAction::Remove, 0.0)));
    assert_eq!(iter.next(), Some((2, 2, IdxBoxAction::Add, 0.0)));
    assert!(iter.next().is_none());

    let mut iter = IdxBoxIter::new((1..=1, 1..=1, 0.0), (3..=3, 3..=3, 0.0), 1);
    assert_eq!(iter.next(), Some((1, 1, IdxBoxAction::Remove, 0.0)));
    assert_eq!(iter.next(), Some((3, 3, IdxBoxAction::Add, 0.0)));
    assert!(iter.next().is_none());
}
#[test]
fn iter_box_diff_area_tests() {
    let mut iter = IdxBoxIter::new((1..=1, 1..=1, 2.0), (1..=1, 1..=1, 1.0), 1);
    assert_eq!(iter.next(), Some((1, 1, IdxBoxAction::Add, 1.0)));
    assert_eq!(iter.next(), Some((1, 1, IdxBoxAction::Remove, 2.0)));
    assert!(iter.next().is_none());
    iter = IdxBoxIter::new((1..=2, 1..=2, 1.0), (2..=3, 2..=3, 2.0), 1);
    assert_eq!(iter.next(), Some((1, 1, IdxBoxAction::Remove, 1.0)));
    assert_eq!(iter.next(), Some((2, 1, IdxBoxAction::Remove, 1.0)));
    assert_eq!(iter.next(), Some((1, 2, IdxBoxAction::Remove, 1.0)));
    assert_eq!(iter.next(), Some((2, 2, IdxBoxAction::Add, 2.0)));
    assert_eq!(iter.next(), Some((2, 2, IdxBoxAction::Remove, 1.0)));
    assert_eq!(iter.next(), Some((3, 2, IdxBoxAction::Add, 2.0)));
    assert_eq!(iter.next(), Some((2, 3, IdxBoxAction::Add, 2.0)));
    assert_eq!(iter.next(), Some((3, 3, IdxBoxAction::Add, 2.0)));
    assert!(iter.next().is_none());
}
