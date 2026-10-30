//! Four natural checked table failures; original forty fixtures remain separate.
//! Child module reuses their actual seed, fixture setup, dispatch and serializer.
use super::{Game, Operation, RomImage, offset, w};
pub(super) const CASES: u32 = 4;
const REUSED: [u32; 4] = [26, 21, 22, 35];
fn loaded_length(id: u32) -> usize {
    match id {
        // Preserve the low byte; the original whole-word preflight rejects both.
        0 => offset(0xa0c2da) + 1,
        1 => offset(0xa0c49f) + 22 + 1,
        // BTS5 *16 + (actor X136 &15) =88, first required byte is absent.
        2 => offset(0x948b2b) + 88,
        // Existing ACA8(136,136)->(145,143): angle88, sine phase216 &127=88.
        3 => offset(0xa0b143) + 88,
        _ => unreachable!(),
    }
}
pub(super) fn fixture(id: u32, g: &mut Game, full: &RomImage) -> Operation {
    assert!(id < CASES);
    let operation = super::fixture(REUSED[id as usize], g, full);
    let short = RomImage::zeroed(loaded_length(id));
    for at in 0..short.len() {
        short.write(at, full.read(at));
    }
    // Native allocation stays fully initialized, registered and owned by carrier.
    // No callback mutates content; the short checked allocation exists at entry.
    g.loaded_rom = Some(short);
    operation
}
pub(super) fn checks(id: u32, g: &Game, before: &[u8], native: &RomImage) {
    assert_eq!(native.len(), 0x200000, "full native allocation retained");
    assert_eq!(g.loaded_rom.as_ref().unwrap().len(), loaded_length(id));
    let mut expected = before.to_vec();
    if id != 3 {
        // All three seeded scans select row8,column8 before their first table read.
        expected[w::cur_block_index..w::cur_block_index + 2].copy_from_slice(&264u16.to_le_bytes());
    }
    if id == 1 {
        // Nonsquare horizontal dispatch publishes BTS before multiplier acquisition.
        expected[w::current_slope_bts..w::current_slope_bts + 2]
            .copy_from_slice(&5u16.to_le_bytes());
    }
    assert_eq!(
        &(0..g.ram().len())
            .map(|at| g.ram().read_byte(at))
            .collect::<Vec<_>>(),
        expected.as_slice(),
        "exact actual table-failure write prefix; no actor/PLM commit"
    );
    assert!(
        !g.g_fail,
        "no low-window diagnostic for valid fixed table addresses"
    );
}
