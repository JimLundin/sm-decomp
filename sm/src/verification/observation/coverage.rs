//! What the Session RAM coverage map (docs/handoff/session-compare.md, slice
//! 2) classes the Work RAM the Session codec does not decode by: the ranges
//! no Session field owns, with the reason for each, and the ranges an engine
//! owner outside the Session holds.
//!
//! The reasons come from the bank logs' RAM map
//! (<https://patrickjohnston.org/bank/>, "RAM map.asm") and `sm_game::ram`'s
//! name table: a range is listed here only when the original never needs it.
//! Engine state with no owner yet stays "neither", so the coverage line keeps
//! counting it.

use sm_tools::session_compare::Held;
use std::ops::Range;

/// Why no Session field owns a range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    /// The original never reads or writes it: the RAM map lists it as unused,
    /// or only unused code touches it, and `sm_game::ram` names nothing there.
    Unused,
    /// Debug state that only the debug controller inputs change.
    Debug,
    /// Written by the original but never read.
    NeverRead,
    /// The emulated hardware's handshake, which the engine has no part in.
    Handshake,
}

/// The ranges the original never needs, beside `NORMALIZED_RAM`'s (whose
/// reasons are `docs/LEGACY.md`'s `LEGACY(verifier)` rows), in address
/// order, each with what the RAM map says of it.
pub(super) const UNOWNED_RAM: [(Range<usize>, Kind); 45] = [
    // $9B..A2: unused.
    (0x9b..0xa3, Kind::Unused),
    // $C1: unused, after the BG4 Y scroll.
    (0xc1..0xc2, Kind::Unused),
    // $C4..CF: unused, before the VRAM write table.
    (0xc4..0xd0, Kind::Unused),
    // $0332: unused, between the VRAM write and Mode 7 transfer tails.
    (0x332..0x334, Kind::Unused),
    // $0336..3F: unused, after the Mode 7 transfer tail.
    (0x336..0x340, Kind::Unused),
    // $0362..6F: unused, after the VRAM read table tail.
    (0x362..0x370, Kind::Unused),
    // $0594: unused.
    (0x594..0x596, Kind::Unused),
    // $059E: HUD BG2 X position during scrolling sky, never written.
    (0x59e..0x5a0, Kind::Unused),
    // $05A0: the contracting flag of the unused HDMA effect $88:B17F.
    (0x5a0..0x5a2, Kind::Unused),
    // $0641: the APU upload's remaining acknowledgement polls, which the host's audio upload writes.
    (0x641..0x643, Kind::Handshake),
    // $071C: the block timer of the unused $80:8577.
    (0x71c..0x71d, Kind::Unused),
    // $0785: the Mode 7 rotation angle of the unused $80:B032/B0C2.
    (0x785..0x787, Kind::Unused),
    // $07D3..DE: unused.
    (0x7d3..0x7df, Kind::Unused),
    // $07ED..F2: unused.
    (0x7ed..0x7f3, Kind::Unused),
    // $093D: unused.
    (0x93d..0x93f, Kind::Unused),
    // $094C..4F: unused.
    (0x94c..0x950, Kind::Unused),
    // $099A: unused.
    (0x99a..0x99c, Kind::Unused),
    // $09A0: unused.
    (0x9a0..0x9a2, Kind::Unused),
    // $0A02: unused.
    (0xa02..0xa04, Kind::Unused),
    // $0A1A: unused.
    (0xa1a..0xa1c, Kind::Unused),
    // $0A70: unused.
    (0xa70..0xa72, Kind::Unused),
    // $0A98: unused.
    (0xa98..0xa9a, Kind::Unused),
    // $0B08: unused.
    (0xb08..0xb0a, Kind::Unused),
    // $0B0C: unused.
    (0xb0c..0xb0e, Kind::Unused),
    // $0B1C..1F: unused.
    (0xb1c..0xb20, Kind::Unused),
    // $0B28: unused.
    (0xb28..0xb2a, Kind::Unused),
    // $0B30: unused.
    (0xb30..0xb32, Kind::Unused),
    // $0B3A: unused.
    (0xb3a..0xb3c, Kind::Unused),
    // $0B4E..55: unused.
    (0xb4e..0xb56, Kind::Unused),
    // $0D9A: unused, between the grapple collision temporaries.
    (0xd9a..0xd9c, Kind::Unused),
    // $0DCA..CD: unused.
    (0xdca..0xdce, Kind::Unused),
    // $0DD8..DB: unused.
    (0xdd8..0xddc, Kind::Unused),
    // $0DF4, $0DF6: the spare CPU display and Samus tile viewer flags, toggled only with select + L held.
    (0xdf4..0xdf8, Kind::Debug),
    // $1886..99: unused.
    (0x1886..0x189a, Kind::Unused),
    // $198A..8C: unused.
    (0x198a..0x198d, Kind::Unused),
    // $1C21: unused.
    (0x1c21..0x1c23, Kind::Unused),
    // $1E9D..AC: the palette FX objects' variables, which no instruction uses.
    (0x1e9d..0x1ead, Kind::Unused),
    // $1F4B..50: unused.
    (0x1f4b..0x1f51, Kind::Unused),
    // $7E:CCF0..CD1F: unused.
    (0xccf0..0xcd20, Kind::Unused),
    // $7E:D552..D5F1: the debug enemy set name and data, which enemy set loading writes and nothing reads.
    (0xd552..0xd5f2, Kind::NeverRead),
    // $7E:D5F2..D651: unused.
    (0xd5f2..0xd652, Kind::Unused),
    // $7E:F3A4..C7: an unused enemy projectile array.
    (0xf3a4..0xf3c8, Kind::Unused),
    // $7E:F3EC..F40F: an unused enemy projectile array.
    (0xf3ec..0xf410, Kind::Unused),
    // $7E:F438..9F: unused, after the special death item drop origin.
    (0xf438..0xf4a0, Kind::Unused),
    // $7E:F4A0..FFFF: unused.
    (0xf4a0..0x10000, Kind::Unused),
];

/// The ranges the host's typed domains hold while the presentation is
/// Legacy (an owned Display carries them in the Session): the frame display
/// (`crate::game_state::frame_display_ranges`, `display::preparation`) and
/// the BG scroll words (`display_scroll`, `$B1..C0`).
pub(super) fn held() -> Vec<Held> {
    let mut held: Vec<Held> = crate::game_state::frame_display_ranges()
        .into_iter()
        .map(|range| Held {
            owner: "frame display",
            range,
        })
        .collect();
    held.push(Held {
        owner: "scroll",
        range: 0xb1..0xc1,
    });
    held
}

/// Every range the coverage map counts as unowned: the normalized ranges and
/// `UNOWNED_RAM`.
pub(super) fn unowned() -> Vec<Range<usize>> {
    crate::cpu_infra::NORMALIZED_RAM
        .into_iter()
        .map(|(_, range)| range)
        .chain(UNOWNED_RAM.into_iter().map(|(range, _)| range))
        .collect()
}

/// The summary line for `UNOWNED_RAM`: its bytes by kind.
pub(super) fn unowned_summary() -> String {
    let kinds = [
        (Kind::Unused, "unused"),
        (Kind::Debug, "debug"),
        (Kind::NeverRead, "never read"),
        (Kind::Handshake, "handshake"),
    ];
    let counts: Vec<String> = kinds
        .iter()
        .map(|&(kind, name)| {
            let bytes: usize = UNOWNED_RAM
                .iter()
                .filter(|(_, entry)| *entry == kind)
                .map(|(range, _)| range.len())
                .sum();
            format!("{name} {bytes}")
        })
        .collect();
    format!(
        "Session RAM unowned beside the normalized ranges: {}",
        counts.join(", ")
    )
}
