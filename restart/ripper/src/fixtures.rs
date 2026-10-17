//! Shared valid-C inputs. Native controls supply real terrain and PLM behavior.
use crate::{Entry, memory::*};

#[derive(Clone, Debug)]
pub struct BodyFixture {
    pub name: String,
    pub entry: Entry,
    pub wram: Vec<u8>,
}

pub(crate) fn base(name: &str, entry: Entry, slot: u16) -> BodyFixture {
    let mut ram: Vec<u8> = (0..SIZE).map(|i| (i * 73 + i / 251 + 19) as u8).collect();
    put(&mut ram, CURRENT, slot);
    put(&mut ram, 0x7a5, 16);
    put(&mut ram, 0x7a7, 16);
    ram[0x10002..0x10202].fill(0);
    ram[0x16402..0x16502].fill(0);
    for n in 0..40 {
        put(&mut ram, 0x1c37 + n * 2, 0);
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
        (INSTRUCTION, 0xe477),
        (INSTRUCTION_TIMER, 0x7654),
        (C, 0x1000),
        (D, 0),
        (E, 8),
        (PARAMETER1, 1),
        (PARAMETER2, 1),
    ] {
        put(&mut ram, b + offset, value);
    }
    BodyFixture { name: name.into(), entry, wram: ram }
}

pub(crate) fn field(fixture: &mut BodyFixture, offset: usize, value: u16) {
    let b = ENEMIES + usize::from(word(&fixture.wram, CURRENT));
    put(&mut fixture.wram, b + offset, value);
}

fn tile(fixture: &mut BodyFixture, index: usize, value: u16, bts: u8) {
    put(&mut fixture.wram, 0x10002 + index * 2, value);
    fixture.wram[0x16402 + index] = bts;
}

/// 46 inputs:20 Init and26 Main across both variants. Invalid domain inputs
/// are separate safe Rust/native admission controls, never original-C fixtures.
pub fn body_fixtures() -> Vec<BodyFixture> {
    let mut cases = Vec::new();
    for (variant, init, main) in
        [("jet", Entry::JetInit, Entry::JetMain), ("ripper", Entry::RipperInit, Entry::RipperMain)]
    {
        for parameter in [0u16, 1, 63, 64, 0x2001] {
            for direction in [0u16, 0x8000] {
                let slot = if parameter == 64 { 0x7c0 } else { 0x40 };
                let mut case = base(&format!("{variant}-init-param{parameter:04x}-dir{direction:04x}"), init, slot);
                field(&mut case, PARAMETER1, parameter);
                field(&mut case, PARAMETER2, direction);
                field(&mut case, E, 0xffff); // Init overwrites irrelevant old row.
                cases.push(case);
            }
        }
        for (name, row, integer, fraction, subposition) in [
            ("air-positive", 8, 0, 0x1000, 0),
            ("air-negative", 0x200, 0xfffc, 0, 0),
            ("air-carry", 8, 0, 0x8000, 0xf000),
            ("air-borrow", 8, 0xffff, 0x8000, 0x1000),
            ("air-zero", 0, 0, 0, 0xabcd),
        ] {
            let mut case = base(&format!("{variant}-main-{name}"), main, 0x40);
            field(&mut case, E, row);
            field(&mut case, D, integer);
            field(&mut case, C, fraction);
            field(&mut case, 4, subposition);
            cases.push(case);
        }
        for (name, row, integer, fraction, subposition, right) in [
            ("wall-zero-integer", 8, 0, 0x8000, 0xf000, true),
            ("wall-mismatch-positive", 8, 1, 0x8123, 0, true),
            ("wall-mismatch-negative", 8, 0xfffe, 0x8123, 0, false),
            ("wall-row0", 0, 1, 0, 0, true),
            ("wall-row63", 0x1f8, 3, 0xf000, 0, true),
            ("wall-row64", 0x200, 0xfffc, 0, 0, false),
        ] {
            let mut case = base(&format!("{variant}-main-{name}"), main, 0x40);
            field(&mut case, E, row);
            field(&mut case, D, integer);
            field(&mut case, C, fraction);
            field(&mut case, 4, subposition);
            field(&mut case, 2, if right { 0x38 } else { 0x48 });
            for index in if right { [52, 68] } else { [51, 67] } {
                tile(&mut case, index, 0x8000, 0);
            }
            cases.push(case);
        }
        for full in [false, true] {
            let mut case =
                base(&format!("{variant}-main-spike-{}", if full { "full-pool" } else { "allocate" }), main, 0x40);
            field(&mut case, E, 0x200);
            field(&mut case, D, 4);
            field(&mut case, C, 0);
            field(&mut case, 2, 0x38);
            tile(&mut case, 52, 0xa000, 15);
            if full {
                for n in 0..40 {
                    put(&mut case.wram, 0x1c37 + n * 2, 0xd094);
                }
            }
            cases.push(case);
        }
    }
    cases
}
