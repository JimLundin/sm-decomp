//! Shapes the crate-wide rules allow: a parent names its child module and the child names
//! nothing back; structs with a role (two signatures, methods, a field, a return).
mod table;

use types::Speed;

/// Her body box, a concept: two signatures take it, and it has a method.
pub struct Body {
    at: i32,
    r: i32,
}

impl Body {
    const fn top(&self) -> i32 {
        self.at.wrapping_sub(self.r)
    }
}

/// Taken by one signature but returned by another: it has a role.
pub struct Step {
    d: i32,
}

/// Taken by one signature but a field's type.
pub struct Reach {
    d: i32,
}

/// A struct holding a reach.
pub struct Probe {
    reach: Reach,
}

fn moves(b: &Body, s: &Speed) -> i32 {
    b.top().wrapping_add(s.0)
}

fn looks(b: &Body) -> i32 {
    b.top()
}

fn take(s: Step) -> i32 {
    s.d
}

const fn step(d: i32) -> Step {
    Step { d }
}

fn far(r: Reach) -> i32 {
    r.d
}

/// Every shape at once.
pub fn all(s: &Speed) -> i32 {
    let b = Body { at: table::SPEED.0, r: 0 };
    let p = Probe { reach: Reach { d: 1 } };
    moves(&b, s)
        .wrapping_add(looks(&Body { at: 1, r: 1 }))
        .wrapping_add(take(step(1)))
        .wrapping_add(take(Step { d: 0 }))
        .wrapping_add(far(Reach { d: 0 }))
        .wrapping_add(p.reach.d)
}
