use crate::bsp::IndexXY;

enum IdxBoxIterSection {
    Old,
    New,
    Done,
}

pub type IdxWork = (i64, i64, IdxBoxAction);

pub struct IdxBoxIter {
    old: IndexXY,
    new: IndexXY,
    step: i64,
    next: Option<(i64, i64, IdxBoxIterSection)>,
}

impl IdxBoxIter {
    pub fn new(old: IndexXY, new: IndexXY, step: i64) -> Self {
        if old.0 == new.0 && old.1 == new.1 {
            return Self {
                old,
                new,
                step,
                next: None,
            };
        } else {
            let x = *old.0.start();
            let y = *old.1.start();
            Self {
                old,
                new,
                step,
                next: Some((x, y, IdxBoxIterSection::Old)),
            }
        }
    }
    fn h_next(x: &mut i64, y: &mut i64, s: &mut IdxBoxIterSection, n: &IndexXY) {
        match s {
            IdxBoxIterSection::Old => {
                *x = *n.0.start();
                *y = *n.1.start();
                *s = IdxBoxIterSection::New;
            }
            _ => *s = IdxBoxIterSection::Done,
        }
    }

    fn next_same_area(&mut self) -> Option<IdxWork> {
        loop {
            match &mut self.next {
                Some((cx, cy, s)) => {
                    let (a, b, t) = match s {
                        IdxBoxIterSection::Old => (&self.old, &self.new, IdxBoxAction::Remove),
                        IdxBoxIterSection::New => (&self.new, &self.old, IdxBoxAction::Add),
                        IdxBoxIterSection::Done => return None,
                    };
                    let x = *cx;
                    let y = *cy;
                    let (cmp_x, cmp_y) = b;

                    if cmp_x.contains(&x) && cmp_y.contains(&y) {
                        *cx = cmp_x.end() + self.step;
                        continue;
                    }

                    if x > *a.0.end() {
                        *cx = *a.0.start();
                        *cy += self.step;
                        continue;
                    } else if y > *a.1.end() {
                        Self::h_next(cx, cy, s, b);
                        continue;
                    }

                    *cx += self.step;
                    return Some((x, y, t));
                }
                None => return None,
            }
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum IdxBoxAction {
    Add,
    Remove,
}
impl Iterator for IdxBoxIter {
    type Item = IdxWork;
    fn next(&mut self) -> Option<Self::Item> {
        self.next_same_area()
    }
}
