//! One valid-C body matrix, shared with native controls. Refusal cases live in
//! Rust tests and must never be dispatched into an unsafe original C body.
use crate::{Entry, memory::*};

#[derive(Clone, Debug)]
pub struct BodyFixture {
    pub name: String,
    pub entry: Entry,
    pub wram: Vec<u8>,
}

pub(crate) fn base() -> Vec<u8> {
    let mut ram = vec![0; SIZE];
    for (at, value) in [
        (0x998, 8),
        (0xd914, 5),
        (0x7a5, 16),
        (0x7a7, 16),
        (X, 40),
        (Y, 40),
        (XR, 8),
        (YR, 8),
        (FEEDBACK, 0xa55a),
        (COLLISION, 0x4567),
        (INDEX, 0x7777),
        (UP, 0x1234),
        (UP + 2, 0x5678),
        (DOWN, 0x9abc),
        (DOWN + 2, 0xdef0),
        (LIST, 0xffff),
    ] {
        put(&mut ram, at, value);
    }
    ram[MOVEMENT - 1] = 8;
    ram[MOVEMENT] = 2;
    for direction in 0..4 {
        put(&mut ram, COLLIDING + 2 * direction, 0x5555 + direction as u16);
    }
    // Distant sentinels exercise whole-WRAM preservation, including bank7F.
    ram[0] = 0x5a;
    ram[0x1fffe] = 0x69;
    ram[0x1ffff] = 0xa5;
    ram
}

pub(crate) fn enemy(ram: &mut [u8], slot: u16, x: u16, y: u16, properties: u16, frozen: u16) {
    let at = ENEMIES + usize::from(slot);
    for (offset, value) in [(2, x), (6, y), (0x0a, 8), (0x0c, 8), (0x0e, properties), (0x26, frozen)] {
        put(ram, at + offset, value);
    }
}

pub(crate) fn list(ram: &mut [u8], slots: &[u16]) {
    assert!(slots.len() < 32);
    put(ram, LIST_GATE, 0x8000); // Nonzero gate is not an entry count.
    for (n, &slot) in slots.iter().enumerate() {
        put(ram, LIST + n * 2, slot);
    }
    put(ram, LIST + slots.len() * 2, 0xffff);
}

fn push(cases: &mut Vec<BodyFixture>, name: impl Into<String>, entry: Entry, wram: Vec<u8>) {
    cases.push(BodyFixture { name: name.into(), entry, wram });
}

/// Deterministic body cases. Native controls supply actual 94:9763 for root
/// no-hit cases, and append their separately identified terrain-reaction cases.
pub fn body_fixtures() -> Vec<BodyFixture> {
    let mut cases = Vec::new();
    for (name, gate, slots) in [
        ("checker-disabled", 0, Vec::new()),
        ("checker-empty", 1, Vec::new()),
        ("checker-ordinary-ignored", 1, vec![0x40]),
        ("checker-duplicate-no-hit", 1, vec![0x40, 0x40]),
        ("checker-last-list-word-terminator", 1, vec![0x40; 31]),
    ] {
        let mut ram = base();
        enemy(&mut ram, 0x40, 40, 56, 0, 0);
        list(&mut ram, &slots);
        put(&mut ram, LIST_GATE, gate);
        put(&mut ram, DIRECTION, 3);
        push(&mut cases, name, Entry::CheckSolidEnemy(0x20000), ram);
    }
    for direction in 0..4u16 {
        for (label, fraction) in [("less", 0x0fff), ("equal", 0x1000), ("greater", 0x1001)] {
            for gap in 0..=2u16 {
                let mut ram = base();
                put(&mut ram, DIRECTION, direction | 0x80);
                put(&mut ram, if direction < 2 { XF } else { YF }, fraction);
                let coordinate = if direction & 1 == 0 { 24 - gap } else { 56 + gap };
                let (x, y) = if direction < 2 { (coordinate, 40) } else { (40, coordinate) };
                enemy(&mut ram, 0x40, x, y, 0x8000, 0);
                list(&mut ram, &[0x40]);
                push(
                    &mut cases,
                    format!("checker-dir{direction}-fraction-{label}-gap{gap}"),
                    Entry::CheckSolidEnemy(0x11000),
                    ram,
                );
            }
        }
        for gap in [-1i16, 0, 1] {
            let mut ram = base();
            put(&mut ram, DIRECTION, direction);
            put(&mut ram, YF, 0x1234);
            let coordinate = if direction & 1 == 0 { 24 - gap } else { 56 + gap } as u16;
            let (x, y) = if direction < 2 { (coordinate, 40) } else { (40, coordinate) };
            enemy(&mut ram, 0x7c0, x, y, 0, 1);
            list(&mut ram, &[0x7c0]);
            push(&mut cases, format!("checker-dir{direction}-frozen-gap{gap}"), Entry::CheckSolidEnemy(0x20000), ram);
        }
        for (label, position, fraction, amount) in [
            ("zero-borrow", 0, 0, 1),
            ("one-borrow", 1, 0, 1),
            ("ffff-carry", 0xffff, 0xffff, 1),
            ("fffe-carry", 0xfffe, 0xffff, 1),
        ] {
            let mut ram = base();
            let axis = if direction < 2 { X } else { Y };
            let sub = if direction < 2 { XF } else { YF };
            put(&mut ram, axis, position);
            put(&mut ram, sub, fraction);
            put(&mut ram, DIRECTION, direction);
            let coordinate = if direction & 1 == 0 { position.wrapping_sub(16) } else { position.wrapping_add(16) };
            let (x, y) = if direction < 2 { (coordinate, 40) } else { (40, coordinate) };
            enemy(&mut ram, 0x40, x, y, 0x8000, 0);
            list(&mut ram, &[0x40]);
            push(&mut cases, format!("checker-dir{direction}-{label}"), Entry::CheckSolidEnemy(amount), ram);
        }
    }
    for direction in [0u16, 2] {
        let mut ram = base();
        put(&mut ram, if direction == 0 { X } else { Y }, 0);
        put(&mut ram, YF, 0x1234);
        put(&mut ram, DIRECTION, direction);
        let (x, y) = if direction == 0 { (0xfff0, 40) } else { (40, 0xfff0) };
        enemy(&mut ram, 0x40, x, y, 0x8000, 0);
        list(&mut ram, &[0x40]);
        push(
            &mut cases,
            format!("checker-dir{direction}-wrapped-zero-not-exact"),
            Entry::CheckSolidEnemy(0x10000),
            ram,
        );
    }
    for (label, slots) in [("first-is-farther", [0x80, 0x40]), ("first-is-nearer", [0x40, 0x80])] {
        let mut ram = base();
        put(&mut ram, DIRECTION, 3);
        enemy(&mut ram, 0x40, 40, 57, 0, 1);
        enemy(&mut ram, 0x80, 40, 58, 0x8000, 0);
        list(&mut ram, &slots);
        push(&mut cases, format!("checker-{label}"), Entry::CheckSolidEnemy(0x40000), ram);
    }
    for (label, width) in [("abs8000-overlap", 0x8001), ("abs8000-touch", 0x8000)] {
        let mut ram = base();
        put(&mut ram, DIRECTION, 3);
        put(&mut ram, XR, width);
        enemy(&mut ram, 0x40, 0x8028, 56, 0x8000, 0);
        put(&mut ram, ENEMIES + 0x40 + 0x0a, 0);
        list(&mut ram, &[0x40]);
        push(&mut cases, format!("checker-{label}"), Entry::CheckSolidEnemy(0x10000), ram);
    }
    for movement in 0..28u8 {
        for collision in [0u16, 1, 0xffff] {
            let mut ram = base();
            ram[MOVEMENT] = movement;
            put(&mut ram, COLLISION, collision);
            push(
                &mut cases,
                format!("down-feedback-mode{movement}-collision{collision:04x}"),
                Entry::DownFeedback,
                ram,
            );
        }
    }
    for (label, movement, collision, input) in [
        ("preserve-low5", 2, 0, 0xab05),
        ("unreachable-index", 0xff, 0, 0xab05),
        ("collision-overrides-low5", 9, 1, 0xab05),
    ] {
        let mut ram = base();
        ram[MOVEMENT] = movement;
        put(&mut ram, COLLISION, collision);
        put(&mut ram, FEEDBACK, input);
        push(&mut cases, format!("down-feedback-{label}"), Entry::DownFeedback, ram);
    }
    for collision in [0u16, 1, 0xffff] {
        let mut ram = base();
        put(&mut ram, COLLISION, collision);
        push(&mut cases, format!("up-feedback-{collision:04x}"), Entry::UpFeedback, ram);
    }
    for (label, position, fraction, amount) in [
        ("fraction-positive", 40, 0x1000, 0x18000),
        ("fraction-negative", 40, 0x9000, -0x18000),
        ("wrap-zero", 0, 0, 1),
        ("wrap-ffff", 0xffff, 0xffff, 1),
        ("zero", 40, 0xabcd, 0),
        ("signed-max", 40, 0xabcd, i32::MAX),
    ] {
        for upward in [false, true] {
            let mut ram = base();
            put(&mut ram, Y, position);
            put(&mut ram, YF, fraction);
            let entry = if upward { Entry::MoveUpNoColl(amount) } else { Entry::MoveDownNoColl(amount) };
            push(&mut cases, format!("no-coll-up{upward}-{label}"), entry, ram);
        }
    }
    push(&mut cases, "no-coll-down-signed-min", Entry::MoveDownNoColl(i32::MIN), base());
    for upward in [false, true] {
        for amount in [0, 0x10000, -0x10000, 0x18000, -0x18000] {
            let ram = base();
            let entry = if upward { Entry::MoveUp(amount) } else { Entry::MoveDown(amount) };
            push(&mut cases, format!("root-up{upward}-air-{amount}"), entry, ram);
        }
        for gap in [0u16, 1] {
            let mut ram = base();
            put(&mut ram, YF, 0x1234);
            enemy(&mut ram, 0x40, 40, if upward { 24 - gap } else { 56 + gap }, 0x8000, 0);
            list(&mut ram, &[0x40]);
            let entry = if upward { Entry::MoveUp(-0x20000) } else { Entry::MoveDown(0x20000) };
            push(&mut cases, format!("root-up{upward}-enemy-gap{gap}"), entry, ram);
        }
    }
    cases
}
