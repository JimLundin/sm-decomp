//! Admission for the nine bank86 definitions observed in the complete replay.
//!
//! The live bridge chooses C for a refusal before publishing any state. Once
//! admitted, the maintained host dispatcher owns the call, including errors.
use crate::Wram;
pub use crate::enemy_projectiles::ProjectileHost;

const DEFINITIONS: [(u16, [u16; 7]); 9] = [
    (0x9642, [0x93ca, 0x940e, 0x9552, 0x0606, 0x5003, 0, 0x84fc]),
    (0x9650, [0x947f, 0x950c, 0x95a0, 0x0606, 0x5003, 0, 0x84fc]),
    (0x965e, [0x947f, 0x950c, 0x95d3, 0x0606, 0x5003, 0, 0x84fc]),
    (0x966c, [0x94c8, 0x950d, 0x9606, 0x0606, 0x5003, 0, 0x84fc]),
    (0x967a, [0x94dc, 0x950d, 0x9606, 0x0606, 0x5003, 0, 0x84fc]),
    (0x9688, [0x94a0, 0x9522, 0x9606, 0x0606, 0x5003, 0, 0x84fc]),
    (0x9696, [0x94b4, 0x9522, 0x9606, 0x0606, 0x5003, 0, 0x84fc]),
    (0xa387, [0xa2ee, 0xa328, 0xa28d, 0x0101, 0x3000, 0, 0xa28d]),
    (0xa395, [0xa31b, 0xa364, 0xa299, 0x0101, 0x3000, 0, 0xa299]),
];
// Only script data is pinned here. Code interleaved between these scripts is
// implemented by the maintained opcode and preinstruction dispatchers.
const SCRIPTS: [(u16, &[u8]); 6] = [
    (
        0x9552,
        &[
            0x6a, 0x81, 0x04, 0x00, 0xca, 0x80, 0x61, 0x81, 0x0e, 0x94, 0x04, 0x00, 0xca, 0x80, 0x02, 0x00, 0xca, 0x80,
            0x02, 0x00, 0xd1, 0x80, 0x02, 0x00, 0xd8, 0x80, 0x02, 0x00, 0xdf, 0x80, 0xab, 0x81, 0x60, 0x95,
        ],
    ),
    (
        0x9574,
        &[
            0x6a, 0x81, 0x05, 0x00, 0xe6, 0x80, 0x05, 0x00, 0xed, 0x80, 0x05, 0x00, 0xf4, 0x80, 0x05, 0x00, 0xfb, 0x80,
            0x05, 0x00, 0x02, 0x81, 0x54, 0x81,
        ],
    ),
    (
        0x95a0,
        &[
            0x6a, 0x81, 0x05, 0x00, 0xe6, 0x80, 0xba, 0x95, 0x05, 0x00, 0xed, 0x80, 0x05, 0x00, 0xf4, 0x80, 0x05, 0x00,
            0xfb, 0x80, 0x05, 0x00, 0x02, 0x81, 0x54, 0x81,
        ],
    ),
    (
        0x95d3,
        &[
            0x6a, 0x81, 0x05, 0x00, 0xe6, 0x80, 0xed, 0x95, 0x05, 0x00, 0xed, 0x80, 0x05, 0x00, 0xf4, 0x80, 0x05, 0x00,
            0xfb, 0x80, 0x05, 0x00, 0x02, 0x81, 0x54, 0x81,
        ],
    ),
    (
        0x9606,
        &[
            0x6a, 0x81, 0x05, 0x00, 0xe6, 0x80, 0x20, 0x96, 0x05, 0x00, 0xed, 0x80, 0x05, 0x00, 0xf4, 0x80, 0x05, 0x00,
            0xfb, 0x80, 0x05, 0x00, 0x02, 0x81, 0x54, 0x81,
        ],
    ),
    (
        0xa28b,
        &[
            0x54, 0x81, 0x01, 0x00, 0xba, 0xb1, 0x01, 0x00, 0xd0, 0xb1, 0xab, 0x81, 0x8d, 0xa2, 0x01, 0x00, 0x6d, 0x84,
            0xab, 0x81, 0x99, 0xa2,
        ],
    ),
];
fn bank86(rom: &[u8], address: u16, count: usize) -> Option<&[u8]> {
    let offset = 0x30000 + usize::from(address & 0x7fff);
    rom.get(offset..offset + count)
}
fn pinned_rom(rom: &[u8]) -> bool {
    DEFINITIONS.iter().all(|(address, words)| {
        bank86(rom, *address, 14).is_some_and(|bytes| {
            bytes.as_chunks::<2>().0.iter().zip(words).all(|(b, w)| u16::from_le_bytes([b[0], b[1]]) == *w)
        })
    }) && SCRIPTS.iter().all(|(address, bytes)| bank86(rom, *address, bytes.len()) == Some(*bytes))
        && bank86(rom, 0x9408, 6) == Some(&[0x03, 0x50, 0x3c, 0x50, 0x50, 0x50])
}
fn definition(id: u16) -> Option<&'static [u16; 7]> {
    DEFINITIONS.iter().find(|(address, _)| *address == id).map(|(_, words)| words)
}
fn split(id: u16) -> bool {
    matches!(id, 0x966c | 0x967a | 0x9688 | 0x9696)
}
fn slot_state(r: &Wram<'_>, k: usize, id: u16) -> bool {
    let Some(words) = definition(id) else { return false };
    let pre = r.word(0x1a03 + k);
    if pre != 0x8170 && pre != words[1] {
        return false;
    }
    let pc = r.word(0x1b47 + k);
    match id {
        0x9642 => matches!(pc, 0x9552 | 0x9558 | 0x9560 | 0x9564 | 0x9568 | 0x956c | 0x9570),
        0x9650 => matches!(pc, 0x95a0 | 0x95a6 | 0x95ac | 0x95b0 | 0x95b4 | 0x95b8),
        0x965e => matches!(pc, 0x95d3 | 0x95d9 | 0x95df | 0x95e3 | 0x95e7 | 0x95eb),
        0x966c | 0x967a | 0x9688 | 0x9696 => {
            r.word(0x1b23 + k) == id
                && matches!(
                    pc,
                    0x9606
                        | 0x960c
                        | 0x9612
                        | 0x9616
                        | 0x961a
                        | 0x961e
                        | 0x9574
                        | 0x957a
                        | 0x957e
                        | 0x9582
                        | 0x9586
                        | 0x958a
                )
        }
        0xa387 => matches!(pc, 0xa28b | 0xa28d | 0xa291 | 0xa295),
        0xa395 => matches!(pc, 0xa28b | 0xa299 | 0xa29d),
        _ => false,
    }
}

// Check extension chains without calling a mutating collision routine. A
// successful chain is cached for this admission only, since terrain can change
// between frames. Both axes share its extent/cycle conditions.
fn terrain_chain(r: &Wram<'_>, start: u16, known: &mut [u8]) -> bool {
    let mut block = start;
    let mut path = Vec::new();
    loop {
        let i = usize::from(block);
        if i >= known.len() || known[i] == 1 {
            return false;
        }
        if known[i] == 2 {
            break;
        }
        known[i] = 1;
        path.push(i);
        let tile = r.word(0x10002 + i * 2) >> 12;
        let bts = r.byte(0x16402 + i);
        if !matches!(tile, 5 | 13) || bts == 0 {
            break;
        }
        let scale = if tile == 13 { r.word(0x7a5) } else { 1 };
        block = block.wrapping_add((bts as i8 as u16).wrapping_mul(scale));
    }
    for i in path {
        known[i] = 2;
    }
    true
}
fn terrain_envelope(r: &Wram<'_>, k: usize, known: &mut [u8]) -> bool {
    let width = r.word(0x7a5);
    if width == 0 || width > 255 || r.word(0x1bb3 + k) != 0x0606 {
        return false;
    }
    // A callback moves at most ceil(abs(8.8 speed)) on either axis. Collision
    // alignment contributes at most fifteen pixels, and a lower newborn split
    // can move fourteen pixels before its initial script clears preinstruction.
    // This rectangle covers both branches and all such descending-slot reuse.
    let speed = |address| (i32::from(r.word(address + k) as i16).abs() + 255) / 256;
    let x = i32::from(r.word(0x1a4b + k));
    let y = i32::from(r.word(0x1a93 + k));
    let dx = speed(0x1ab7) + 6 + 32;
    let dy = speed(0x1adb) + 6 + 32;
    // Coordinates and radius edges wrap as uint16. A collision scan starts
    // from a wrapped row/column, then increments its block index; crossing
    // a coordinate boundary does not restart that increment at zero. Inspect
    // both the wrapped origin and those contiguous scan indices.
    let low_row = (y - dy).div_euclid(16);
    let high_row = (y + dy).div_euclid(16);
    let low_col = (x - dx).div_euclid(16);
    let high_col = (x + dx).div_euclid(16);
    for row in low_row..=high_row {
        let row_offsets = if row.div_euclid(256) > low_row.div_euclid(256) { &[0, 256][..] } else { &[0][..] };
        for col in low_col..=high_col {
            let col_offsets = if col.div_euclid(4096) > low_col.div_euclid(4096) { &[0, 4096][..] } else { &[0][..] };
            for row_offset in row_offsets {
                for col_offset in col_offsets {
                    // Native hardware multiplies low row/width bytes. Later
                    // scan increments and the doubled uint16 index wrap too.
                    let block = (u16::from(row as u8).wrapping_add(*row_offset).wrapping_mul(width))
                        .wrapping_add((col as u16 & 0x0fff).wrapping_add(*col_offset))
                        .wrapping_mul(2)
                        >> 1;
                    if !terrain_chain(r, block, known) {
                        return false;
                    }
                }
            }
        }
    }
    true
}

/// Decide ownership from immutable state. `false` selects C before mutation;
/// callers must never turn a subsequent Rust dispatch error into a C retry.
pub fn supports_call(r: &Wram<'_>, rom: &[u8], name: &str, a: i32, b: i32) -> Result<bool, &'static str> {
    match name {
        "SpawnEprojWithRoomGfx" => {
            // Saturation accesses no definition, initializer, script or host.
            if (0..36).step_by(2).all(|k| r.word(0x1997 + k) != 0) {
                return Ok(true);
            }
            let id = a as u16;
            if definition(id).is_none() || !pinned_rom(rom) {
                return Ok(false);
            }
            if split(id) && (b as u16 > 34 || b as u16 & 1 != 0) {
                return Ok(false);
            }
            Ok(true)
        }
        "EprojRunAll" => {
            if r.word(0x198d) & 0x8000 == 0 {
                return Ok(true);
            }
            let active: Vec<_> = (0..36).step_by(2).filter(|k| r.word(0x1997 + k) != 0).collect();
            if active.is_empty() {
                return Ok(true);
            }
            if !pinned_rom(rom) || active.iter().any(|k| !slot_state(r, *k, r.word(0x1997 + *k))) {
                return Ok(false);
            }
            let mut terrain = vec![0; 0x3200];
            for k in active {
                let id = r.word(0x1997 + k);
                let pre = r.word(0x1a03 + k);
                let pc = r.word(0x1b47 + k);
                let births = r.word(0x1b8f + k) == 1
                    && (matches!((id, pc), (0x9650, 0x95a6) | (0x965e, 0x95d9)) || (split(id) && pc == 0x960c));
                // A lower newly allocated split runs its collision callback in
                // this same descending traversal. Inspect that closure too.
                if (matches!(pre, 0x940e | 0x950d | 0x9522) || births)
                    && (rom.len() < 0xa0d2b || !terrain_envelope(r, k, &mut terrain))
                {
                    return Ok(false);
                }
                if pre == 0x940e && (r.byte(0x644) >= 16 || r.byte(0x647) >= 16) {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}

/// The direct offline seam deliberately delegates to the maintained dispatcher.
/// Live callers first use `supports_call`, then publish success or report error.
pub fn dispatch_host(
    r: &mut Wram<'_>,
    rom: &[u8],
    host: &mut ProjectileHost,
    name: &str,
    a: i32,
    b: i32,
) -> Result<u32, &'static str> {
    crate::enemy_projectiles::dispatch_host(r, rom, host, name, a, b)
}
