//! Shared full-state fixture inputs for Rust semantics and actual-C pairs.
//! These definitions claim no native execution or filesystem persistence.
use crate::{Entry, PAYLOAD_SIZE, SAVED, SLOTS, SRAM_SIZE, WRAM_SIZE, put, word};

#[derive(Clone, Debug)]
pub struct BodyFixture {
    pub name: String,
    pub entry: Entry,
    pub argument: u16,
    pub wram: Vec<u8>,
    pub sram: Vec<u8>,
}

fn patterned(size: usize, salt: usize) -> Vec<u8> {
    (0..size).map(|i| (i.wrapping_mul(73).wrapping_add(i / 251).wrapping_add(salt)) as u8).collect()
}
fn base(name: String, entry: Entry, argument: u16) -> BodyFixture {
    BodyFixture { name, entry, argument, wram: patterned(WRAM_SIZE, 19), sram: patterned(SRAM_SIZE, 137) }
}

/// Thirty behavior inputs: nine saves, nineteen loads and two direct map helpers.
/// Invalid checksums are safe C inputs; masked slot 3 is deliberately excluded.
pub fn body_fixtures() -> Vec<BodyFixture> {
    let mut fixtures = Vec::new();
    for area in 0..6_u16 {
        let slot = area % 3;
        let mut fixture = base(format!("save-slot-{slot}-area-{area}"), Entry::Save, slot);
        put(&mut fixture.wram, 0x079f, area);
        put(&mut fixture.wram, 0x078b, 0xcafe);
        fixtures.push(fixture);
    }
    for slot in 0..3_u16 {
        let mut fixture = base(format!("save-slot-{slot}-high-alias"), Entry::Save, 0xfffc | slot);
        put(&mut fixture.wram, 0x079f, slot);
        put(&mut fixture.wram, 0x078b, 0xffff);
        fixtures.push(fixture);
    }
    for slot in 0..3_u16 {
        for mode in ["both", "lower", "upper", "neither", "split"] {
            fixtures.push(load_fixture(slot, mode, slot));
        }
        fixtures.push(load_fixture(slot, "both", 0xfffc | slot));
    }
    let mut wrap = base("load-checksum-wrap".into(), Entry::Load, 0);
    wrap.sram[SLOTS[0]..SLOTS[0] + PAYLOAD_SIZE].fill(0xff);
    checksums(&mut wrap.sram, 0, "both");
    fixtures.push(wrap);
    fixtures.push(base("pack-all-map-indices".into(), Entry::Pack, 0));
    fixtures.push(base("unpack-exact-clear-and-indices".into(), Entry::Unpack, 0));
    fixtures
}

fn load_fixture(slot: u16, mode: &str, argument: u16) -> BodyFixture {
    let alias = if argument == slot { "" } else { "-high-alias" };
    let mut fixture = base(format!("load-slot-{slot}-{mode}{alias}"), Entry::Load, argument);
    let start = SLOTS[usize::from(slot)];
    // Valid payload station/area values are copied, never range-validated.
    put(&mut fixture.sram, start + 0xd916 - SAVED, 0xcafe);
    put(&mut fixture.sram, start + 0xd918 - SAVED, 0xfedc);
    checksums(&mut fixture.sram, usize::from(slot), mode);
    fixture
}
fn checksums(sram: &mut [u8], slot: usize, mode: &str) {
    let sum =
        (0..PAYLOAD_SIZE).step_by(2).fold(0_u16, |sum, offset| sum.wrapping_add(word(sram, SLOTS[slot] + offset)));
    let corrupt = sum ^ 1;
    // "split" has a matching lower sum and upper inverse, but neither pair.
    let (lower, lower_inverse, upper, upper_inverse) = match mode {
        "both" => (sum, !sum, sum, !sum),
        "lower" => (sum, !sum, corrupt, !corrupt),
        "upper" => (corrupt, !corrupt, sum, !sum),
        "neither" => (corrupt, !corrupt, corrupt, !corrupt),
        "split" => (sum, !corrupt, corrupt, !sum),
        _ => unreachable!("private fixture checksum mode"),
    };
    put(sram, slot * 2, lower);
    put(sram, 8 + slot * 2, lower_inverse);
    put(sram, 0x1ff0 + slot * 2, upper);
    put(sram, 0x1ff8 + slot * 2, upper_inverse);
}
