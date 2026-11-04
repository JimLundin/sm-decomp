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

use sm_tools::session_compare::{Classes, Held};
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

/// What becomes of a Session field whose bytes a `NORMALIZED_RAM` range
/// covers, so that neither compare sees them today ("both").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Shared {
    /// The field is compared once its `LEGACY(verifier)` row retires: the
    /// row's divergence is fixed, then the range leaves `NORMALIZED_RAM` and
    /// the row leaves `docs/LEGACY.md` in the same change.
    UntilRowRetires,
    /// The engine keeps the field in RAM the original leaves to snesrev's
    /// bookkeeping or unused, so RAM cannot carry it: it goes on the direct
    /// compare's declared list of fields RAM cannot carry (slice 4).
    Permanent,
}

/// The Session fields the normalized ranges cover, in address order, each
/// with the field and its decision (docs/handoff/session-compare.md, "The
/// 107 both bytes").
pub(super) const SHARED_RAM: [(Range<usize>, &str, Shared); 16] = [
    // The direct-page long pointer scratch; the $0000-$0050 row.
    (0x0..0x4, "pointer_scratch", Shared::UntilRowRetires),
    // The enemy projectile spawn scratch; the $0000-$0050 row.
    (0x12..0x18, "spawn_scratch", Shared::UntilRowRetires),
    // The $060B-$0610 row.
    (
        0x60b..0x60c,
        "enemy_projectiles.spawn.angle",
        Shared::UntilRowRetires,
    ),
    // snesrev's coroutine state, which a faithful oracle never writes.
    (
        0x612..0x614,
        "control.primary, control.secondary",
        Shared::Permanent,
    ),
    // snesrev's counter RAM, which the original leaves unused ($077E..82).
    (
        0x77e..0x782,
        "messages.delay, room.unpause_queue_tail",
        Shared::Permanent,
    ),
    // The $0D82-$0D8F rows, which the grapple temporaries share.
    (
        0xd82..0xd90,
        "game_loop.saved_bindings",
        Shared::UntilRowRetires,
    ),
    // The $0D9C-$0D9D row, which the grapple swing's angular velocity shares.
    (0xd9c..0xd9e, "wavy_samus.enabled", Shared::UntilRowRetires),
    // The enemy temporary rows from here to $EF77.
    (
        0xe20..0xe46,
        "compatibility_enemy_scratch, actors.compatibility",
        Shared::UntilRowRetires,
    ),
    (
        0xe4a..0xe4c,
        "actors.combat.last_power",
        Shared::UntilRowRetires,
    ),
    (0xe54..0xe5a, "actors.controls", Shared::UntilRowRetires),
    (
        0x1784..0x178c,
        "actors.compatibility.pre_hook",
        Shared::UntilRowRetires,
    ),
    (0x17a8..0x17ac, "actors.schedule", Shared::UntilRowRetires),
    (0x1834..0x183c, "actors.schedule", Shared::UntilRowRetires),
    (
        0x1874..0x1876,
        "actors.compatibility.right_border",
        Shared::UntilRowRetires,
    ),
    (
        0x18a6..0x18a8,
        "room.collision_detection_index",
        Shared::UntilRowRetires,
    ),
    (
        0xef74..0xef78,
        "actors.graphics.compatibility",
        Shared::UntilRowRetires,
    ),
];

/// The summary line for the "both" bytes: by `SHARED_RAM`'s decision, and
/// the ranges no entry decides, which must stay none.
pub(super) fn shared_summary(classes: &Classes) -> String {
    let mut until = 0;
    let mut permanent = 0;
    let mut fields: Vec<&str> = Vec::new();
    let mut undecided = 0;
    let mut first_undecided = None;
    for at in classes.both_ranges.iter().flat_map(Clone::clone) {
        match SHARED_RAM.iter().find(|(range, _, _)| range.contains(&at)) {
            Some((_, _, Shared::UntilRowRetires)) => until += 1,
            Some(&(_, field, Shared::Permanent)) => {
                permanent += 1;
                if !fields.contains(&field) {
                    fields.push(field);
                }
            }
            None => {
                undecided += 1;
                first_undecided.get_or_insert(at);
            }
        }
    }
    format!(
        "Session RAM decoded and unowned by decision: until its row retires {until}, permanent {permanent} ({}), undecided {undecided}{}",
        fields.join("; "),
        first_undecided.map_or_else(String::new, |at| format!(" (first ${at:05X})"))
    )
}

/// One range's differences over a replay: the Ticks whose two results differ
/// in it before normalization, the first of them, and which of its bytes
/// ever differed.
struct Differing {
    range: Range<usize>,
    ticks: u64,
    first: Option<i32>,
    /// Per byte of `range`, whether it ever differed.
    bytes: Vec<bool>,
}

impl Differing {
    fn new(range: Range<usize>) -> Self {
        Self {
            bytes: vec![false; range.len()],
            range,
            ticks: 0,
            first: None,
        }
    }

    fn note(&mut self, tick: i32, mine: &[u8], theirs: &[u8]) {
        let (mine, theirs) = (&mine[self.range.clone()], &theirs[self.range.clone()]);
        if mine == theirs {
            return;
        }
        self.ticks += 1;
        self.first.get_or_insert(tick);
        for ((differed, ours), oracle) in self.bytes.iter_mut().zip(mine).zip(theirs) {
            *differed |= ours != oracle;
        }
    }

    /// The runs of bytes that never differed.
    fn unchanged(&self) -> Vec<Range<usize>> {
        let mut runs: Vec<Range<usize>> = Vec::new();
        for (at, _) in self
            .range
            .clone()
            .zip(&self.bytes)
            .filter(|&(_, &differed)| !differed)
        {
            match runs.last_mut() {
                Some(run) if run.end == at => run.end = at + 1,
                _ => runs.push(at..at + 1),
            }
        }
        runs
    }

    /// `$0000-$0003 <ticks> from @<tick> in <bytes> bytes`.
    fn text(&self) -> String {
        format!(
            "{} {}{} in {} bytes",
            named(&self.range),
            self.ticks,
            self.first
                .map_or_else(String::new, |tick| format!(" from @{tick}")),
            self.bytes.iter().filter(|&&differed| differed).count()
        )
    }
}

/// `$0000-$0003`, or `$0612` for one byte.
fn named(range: &Range<usize>) -> String {
    if range.len() == 1 {
        format!("${:04X}", range.start)
    } else {
        format!("${:04X}-${:04X}", range.start, range.end - 1)
    }
}

/// Per `SHARED_RAM` entry, the Ticks whose two results differ in its bytes
/// before normalization copies one side over the other: the evidence that
/// retires a `LEGACY(verifier)` row (an entry that never differs on the
/// corpus can leave the normalized range) or names the Ticks to fix. An
/// entry that differs also names its bytes that never did, which a split of
/// its row could compare.
pub(super) struct SharedDifferences {
    entries: Vec<Differing>,
    compared: u64,
}

impl Default for SharedDifferences {
    fn default() -> Self {
        Self {
            entries: SHARED_RAM
                .iter()
                .map(|(range, _, _)| Differing::new(range.clone()))
                .collect(),
            compared: 0,
        }
    }
}

impl SharedDifferences {
    /// Count one paired frame's results.
    pub(super) fn note(&mut self, tick: i32, mine: &[u8], theirs: &[u8]) {
        self.compared += 1;
        for entry in &mut self.entries {
            entry.note(tick, mine, theirs);
        }
    }

    /// The summary line: each entry waiting on its row, by differing Ticks.
    pub(super) fn summary(&self) -> String {
        let entries: Vec<String> = SHARED_RAM
            .iter()
            .zip(&self.entries)
            .filter(|((_, _, shared), _)| *shared == Shared::UntilRowRetires)
            .map(|((range, field, _), entry)| {
                let first = entry
                    .first
                    .map_or_else(String::new, |tick| format!(" from @{tick}"));
                let unchanged = entry.unchanged();
                let unchanged = if entry.first.is_none() || unchanged.is_empty() {
                    String::new()
                } else {
                    let runs: Vec<String> = unchanged.iter().map(named).collect();
                    format!(" (never differed: {})", runs.join(", "))
                };
                format!(
                    "${:04X}-${:04X} {field} {}{first}{unchanged}",
                    range.start,
                    range.end - 1,
                    entry.ticks
                )
            })
            .collect();
        format!(
            "Session RAM both bytes differing before normalization, of {} ticks: {}",
            self.compared,
            entries.join("; ")
        )
    }
}

/// Why a byte is unowned, as the nightly report groups it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Reason {
    /// An `UNOWNED_RAM` range, with the RAM map's reason.
    Unowned(Kind),
    /// A `NORMALIZED_RAM` range outside the Session fields it covers: its
    /// reason is its `LEGACY(verifier)` row.
    Normalized,
}

impl Reason {
    const ALL: [Self; 5] = [
        Self::Unowned(Kind::Unused),
        Self::Unowned(Kind::Debug),
        Self::Unowned(Kind::NeverRead),
        Self::Unowned(Kind::Handshake),
        Self::Normalized,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::Unowned(Kind::Unused) => "unused",
            Self::Unowned(Kind::Debug) => "debug",
            Self::Unowned(Kind::NeverRead) => "never read",
            Self::Unowned(Kind::Handshake) => "handshake",
            Self::Normalized => "normalized (LEGACY(verifier) rows)",
        }
    }
}

/// The nightly whole-RAM report (#246; docs/handoff/session-compare.md):
/// per replay, the unowned bytes whose two results differ before
/// normalization, by reason. It keeps watching the bytes the Session compare
/// does not own after the locks drop `wram` (slice 6). A difference is worth
/// a look, never a failure: the RAM and Session compares decide. The
/// scheduled run turns it on (`SM_VERIFY_UNOWNED_REPORT=1`).
pub(super) struct UnownedDifferences {
    ranges: Vec<(Reason, Differing)>,
    compared: u64,
}

impl Default for UnownedDifferences {
    fn default() -> Self {
        let shared = |at: &usize| SHARED_RAM.iter().any(|(range, _, _)| range.contains(at));
        let mut ranges: Vec<(Reason, Differing)> = UNOWNED_RAM
            .iter()
            .map(|(range, kind)| (Reason::Unowned(*kind), Differing::new(range.clone())))
            .collect();
        for (_, range) in crate::cpu_infra::NORMALIZED_RAM {
            let mut runs: Vec<Range<usize>> = Vec::new();
            for at in range.filter(|at| !shared(at)) {
                match runs.last_mut() {
                    Some(run) if run.end == at => run.end = at + 1,
                    _ => runs.push(at..at + 1),
                }
            }
            ranges.extend(
                runs.into_iter()
                    .map(|run| (Reason::Normalized, Differing::new(run))),
            );
        }
        ranges.sort_by_key(|(_, entry)| entry.range.start);
        Self {
            ranges,
            compared: 0,
        }
    }
}

impl UnownedDifferences {
    /// The report, when the run asks for it.
    pub(super) fn requested() -> Option<Self> {
        (std::env::var_os("SM_VERIFY_UNOWNED_REPORT").as_deref() == Some("1".as_ref()))
            .then(Self::default)
    }

    /// Count one paired frame's results.
    pub(super) fn note(&mut self, tick: i32, mine: &[u8], theirs: &[u8]) {
        self.compared += 1;
        for (_, entry) in &mut self.ranges {
            entry.note(tick, mine, theirs);
        }
    }

    /// The summary line: per reason, the ranges that differed, or none.
    pub(super) fn summary(&self) -> String {
        let reasons: Vec<String> = Reason::ALL
            .iter()
            .map(|&reason| {
                let differing: Vec<String> = self
                    .ranges
                    .iter()
                    .filter(|(entry_reason, entry)| *entry_reason == reason && entry.ticks != 0)
                    .map(|(_, entry)| entry.text())
                    .collect();
                format!(
                    "{} {}",
                    reason.name(),
                    if differing.is_empty() {
                        "none".to_owned()
                    } else {
                        differing.join(", ")
                    }
                )
            })
            .collect();
        format!(
            "Unowned RAM differing before normalization, of {} ticks: {}",
            self.compared,
            reasons.join("; ")
        )
    }
}

/// The ranges an engine owner outside the Session holds: none since the
/// Legacy presentation owns the frame display and the BG scroll words
/// (`LegacyPresentation::frame`, `scroll`), which the host's typed domains
/// held before. A new holder is listed here, so the coverage line counts it.
pub(super) fn held() -> Vec<Held> {
    Vec::new()
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
