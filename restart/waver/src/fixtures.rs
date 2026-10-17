//! Shared full-WRAM actual-C fixture inputs; no terrain implementation lives here.
use crate::{Entry, memory::*};
#[derive(Clone, Debug)]
pub struct BodyFixture {
    pub name: String,
    pub entry: Entry,
    pub k: u16,
    pub jp: u16,
    pub wram: Vec<u8>,
}
fn base(name: &str, entry: Entry, slot: u16) -> BodyFixture {
    let mut ram: Vec<u8> = (0..SIZE).map(|i| (i * 73 + i / 251 + 19) as u8).collect();
    put(&mut ram, CURRENT, slot);
    put(&mut ram, 0x7a5, 16);
    put(&mut ram, 0x7a7, 16);
    ram[0x10002..0x10202].fill(0);
    ram[0x16402..0x16502].fill(0);
    for i in (0..80).step_by(2) {
        put(&mut ram, 0x1c37 + i, 0);
    }
    let b = ENEMIES + usize::from(slot);
    for (offset, value) in [
        (2, 0x40),
        (4, 0),
        (6, 0x40),
        (8, 0),
        (10, 8),
        (12, 8),
        (TIMER, 0x9876),
        (INSTRUCTION, 0x86a7),
        (INSTRUCTION_TIMER, 0x7654),
        (A, 0x8000),
        (B, 1),
        (C, 1),
        (D, 0),
        (E, 0),
        (F, 1),
        (PARAMETER, 1),
    ] {
        put(&mut ram, b + offset, value);
    }
    BodyFixture { name: name.into(), entry, k: 0, jp: 0, wram: ram }
}
fn field(fixture: &mut BodyFixture, offset: usize, value: u16) {
    let b = ENEMIES + usize::from(word(&fixture.wram, CURRENT));
    put(&mut fixture.wram, b + offset, value);
}
fn tile(fixture: &mut BodyFixture, index: usize, block: u16, bts: u8) {
    put(&mut fixture.wram, 0x10002 + index * 2, block);
    fixture.wram[0x16402 + index] = bts;
}
/// 24 inputs: four initializers, six helper cases, two event instructions and
/// twelve Main cases. Spike cases use the concrete D094 setup and free/full pool.
pub fn body_fixtures() -> Vec<BodyFixture> {
    let mut fixtures = Vec::new();
    for (name, slot, parameter) in [
        ("init-left", 0, 0),
        ("init-right", 0x40, 1),
        ("init-left-high-parameter", 0x7c0, 0xfffe),
        ("init-right-high-parameter", 0x780, 0xffff),
    ] {
        let mut f = base(name, Entry::Init, slot);
        field(&mut f, PARAMETER, parameter);
        field(&mut f, D, 0xbeef);
        field(&mut f, F, 0xfedc); // Init overwrites irrelevant old selectors.
        fixtures.push(f);
    }
    for (name, selector) in [("helper-unchanged", 1), ("helper-unchanged-ffff", 0xffff)] {
        let mut f = base(name, Entry::SelectAnimation, 0x40);
        field(&mut f, C, selector);
        field(&mut f, F, selector);
        fixtures.push(f);
    }
    for selector in 0..4 {
        let mut f = base(&format!("helper-select-{selector}"), Entry::SelectAnimation, 0x40);
        field(&mut f, C, 0xffff);
        field(&mut f, F, selector);
        fixtures.push(f);
    }
    for (name, slot, k, jp) in
        [("instruction-current", 0x40, 0x40, 0x86c5), ("instruction-distinct-k", 0x7c0, 0x80, 0x86d9)]
    {
        let mut f = base(name, Entry::Instruction, slot);
        f.k = k;
        f.jp = jp;
        fixtures.push(f);
    }
    for (name, phase, event) in [
        ("main-zero-sine", 0, 0),
        ("main-negative-sine", 0x40, 0),
        ("main-positive-sine", 0xc0, 0),
        ("main-phase-wrap", 0xfffe, 0),
        ("main-special-then-event", 54, 1),
    ] {
        let mut f = base(name, Entry::Main, 0x40);
        field(&mut f, D, phase);
        field(&mut f, E, event);
        fixtures.push(f);
    }
    let mut right = base("main-right-fractional-wall", Entry::Main, 0x40);
    field(&mut right, 2, 0x38);
    field(&mut right, A, 0x8123);
    tile(&mut right, 52, 0x8000, 0);
    tile(&mut right, 68, 0x8000, 0);
    fixtures.push(right.clone());
    right.name = "main-wall-special-then-event".into();
    field(&mut right, D, 56);
    field(&mut right, E, 1);
    fixtures.push(right);
    let mut left = base("main-left-wall", Entry::Main, 0x40);
    field(&mut left, 2, 0x48);
    field(&mut left, B, 0xfffe);
    tile(&mut left, 51, 0x8000, 0);
    tile(&mut left, 67, 0x8000, 0);
    fixtures.push(left);
    for (name, y, phase, indices) in [("main-down-hit", 0x38, 0xc0, [67, 68]), ("main-up-hit", 0x48, 0x40, [51, 52])] {
        let mut f = base(name, Entry::Main, 0x40);
        field(&mut f, 6, y);
        field(&mut f, D, phase);
        for index in indices {
            tile(&mut f, index, 0x8000, 0);
        }
        fixtures.push(f);
    }
    for full in [false, true] {
        let mut f = base(if full { "main-spike-full-pool" } else { "main-spike-allocate" }, Entry::Main, 0x40);
        field(&mut f, 2, 0x38);
        tile(&mut f, 52, 0xa000, 15);
        if full {
            for i in (0..80).step_by(2) {
                put(&mut f.wram, 0x1c37 + i, 0xd094);
            }
        }
        fixtures.push(f);
    }
    fixtures
}
