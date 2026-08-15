use crate::bsp::IndexXY;

enum IdxBoxIterSection {
    Old,
    New,
    Done,
}

pub type IdxWork = (i64, i64, IdxBoxAction, f32);

pub struct IdxBoxIter {
    old: IndexXY,
    new: IndexXY,
    step: i64,
    next: Option<(i64, i64, IdxBoxIterSection)>,
    all: bool,
    wants_add: bool,
}

impl IdxBoxIter {
    pub fn new(old: IndexXY, new: IndexXY, step: i64) -> Self {
        if old.0 == new.0 && old.1 == new.1 && old.2 == new.2 {
            return Self {
                old,
                new,
                step,
                next: None,
                all: false,
                wants_add: true,
            };
        }
        let x = *old.0.start();
        let y = *old.1.start();
        if old.2 == new.2 {
            Self {
                all: false,
                old,
                new,
                step,
                next: Some((x, y, IdxBoxIterSection::Old)),
                wants_add: true,
            }
        } else {
            Self {
                all: true,
                old,
                new,
                step,
                next: Some((x, y, IdxBoxIterSection::Old)),
                wants_add: true,
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
                    let (a, b, t, is_old) = match s {
                        IdxBoxIterSection::Old => {
                            (&self.old, &self.new, IdxBoxAction::Remove, true)
                        }
                        IdxBoxIterSection::New => (&self.new, &self.old, IdxBoxAction::Add, false),
                        IdxBoxIterSection::Done => return None,
                    };
                    let x = *cx;
                    let y = *cy;
                    let (cmp_x, cmp_y, _) = b;

                    if cmp_x.contains(&x) && cmp_y.contains(&y) {
                        if is_old && self.all {
                            if self.wants_add && self.old.0.contains(&x) && self.old.1.contains(&y)
                            {
                                self.wants_add = false;
                                return Some((x, y, IdxBoxAction::Add, self.new.2));
                            }
                        } else {
                            *cx = cmp_x.end() + self.step;
                            continue;
                        }
                    }

                    if x > *a.0.end() {
                        *cx = *a.0.start();
                        *cy += self.step;
                        continue;
                    } else if y > *a.1.end() {
                        Self::h_next(cx, cy, s, b);
                        continue;
                    }
                    if !is_old && !self.wants_add {
                        self.wants_add = true;
                        continue;
                    }
                    self.wants_add = true;
                    *cx += self.step;
                    return Some((x, y, t, a.2));
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
