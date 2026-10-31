//! How closely the oracle follows the original ROM.
//!
//! `tools/snesrev_patches.py` lists every snesrev compatibility patch (forced
//! carries, bug-fix hooks, the `PEA $C2AE` Y reset and ROM patches) with what
//! faithful mode does with it and whether the Rust engine still reproduces it;
//! `tools/oracle_patch_policy.py` turns it into `oracle_patch_policy::PATCHES`.
//!
//! `SM_ORACLE=patched` (the default) installs each removable patch until the
//! Rust engine follows the original there. `SM_ORACLE=faithful` drops every
//! removable patch, for discovery. Both keep the patches the frame-stepping
//! oracle depends on.
#![forbid(unsafe_code)]
use std::sync::OnceLock;

/// Whether the oracle runs the original game logic without snesrev's patches.
pub fn faithful() -> bool {
    static FAITHFUL: OnceLock<bool> = OnceLock::new();
    *FAITHFUL.get_or_init(|| match std::env::var("SM_ORACLE").as_deref() {
        Err(std::env::VarError::NotPresent) | Ok("patched") => false,
        Ok("faithful") => true,
        other => panic!("SM_ORACLE must be `patched` or `faithful`, not {other:?}"),
    })
}

/// The kind of snesrev change, one table per kind in `tools/snesrev_patches.py`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PatchKind {
    /// A `kPatchedCarrys` site: forced carry before ADC/SBC.
    Carry,
    /// A `patch_bugs_parts` compatibility action.
    BugHook,
    /// A hook inside the instruction dispatch.
    OtherHook,
    /// Bytes written by `RomPatches`.
    Rom,
}

impl PatchKind {
    fn sites(self, table: Policy) -> &'static [u32] {
        use crate::oracle_patch_policy as policy;
        match (self, table) {
            (Self::Carry, Policy::Kept) => policy::KEPT_CARRY,
            (Self::BugHook, Policy::Kept) => policy::KEPT_BUG_HOOKS,
            (Self::OtherHook, Policy::Kept) => policy::KEPT_OTHER_HOOKS,
            (Self::Rom, Policy::Kept) => policy::KEPT_ROM_PATCHES,
            (Self::Carry, Policy::Changed) => policy::CHANGED_CARRY,
            (Self::BugHook, Policy::Changed) => policy::CHANGED_BUG_HOOKS,
            (Self::OtherHook, Policy::Changed) => policy::CHANGED_OTHER_HOOKS,
            (Self::Rom, Policy::Changed) => policy::CHANGED_ROM_PATCHES,
            (Self::Carry, Policy::Follows) => policy::FOLLOWS_CARRY,
            (Self::BugHook, Policy::Follows) => policy::FOLLOWS_BUG_HOOKS,
            (Self::OtherHook, Policy::Follows) => policy::FOLLOWS_OTHER_HOOKS,
            (Self::Rom, Policy::Follows) => policy::FOLLOWS_ROM_PATCHES,
        }
    }
}

#[derive(Clone, Copy)]
enum Policy {
    /// The frame-stepping oracle depends on it; both modes install it.
    Kept,
    /// Installed in a variant closer to the original once native follows it.
    Changed,
    /// The Rust engine follows the original game; the default oracle drops it.
    Follows,
}

/// Whether the oracle installs this patch. Kept and changed patches are
/// always installed; any other one only while the Rust engine still
/// reproduces it, and never in faithful mode.
pub(crate) fn installs(kind: PatchKind, address: u32) -> bool {
    let listed = |table| kind.sites(table).contains(&address);
    listed(Policy::Kept) || listed(Policy::Changed) || !drops(kind, address)
}

/// Whether an installed "changed" patch uses its variant closer to the original.
pub(crate) fn follows_original(kind: PatchKind, address: u32) -> bool {
    drops(kind, address)
}

/// Whether the oracle drops this removable patch: in faithful mode unless
/// `SM_ORACLE_KEEP` keeps it, otherwise once the Rust engine follows it.
fn drops(kind: PatchKind, address: u32) -> bool {
    if faithful() {
        !faithful_keeps().contains(&address)
    } else {
        kind.sites(Policy::Follows).contains(&address)
    }
}

/// The controller read snesrev moved from NMI; these four entries flip together.
const INPUT_ENTRIES: [u32; 4] = [0x8095e1, 0x828944, 0x82897a, 0x82f713];

/// Entries the faithful oracle still installs, from `SM_ORACLE_KEEP`: comma
/// separated addresses (`0x8095e1` or `$80:95E1`), or `input` for the four
/// controller-read entries. Lanes use it to triage faithful mode without
/// entries whose native work is pending.
fn faithful_keeps() -> &'static [u32] {
    static KEEPS: OnceLock<Vec<u32>> = OnceLock::new();
    KEEPS.get_or_init(|| {
        let Ok(text) = std::env::var("SM_ORACLE_KEEP") else {
            return Vec::new();
        };
        let mut keeps = Vec::new();
        for token in text
            .split(',')
            .map(str::trim)
            .filter(|token| !token.is_empty())
        {
            if token == "input" {
                keeps.extend(INPUT_ENTRIES);
                continue;
            }
            let digits: String = token
                .trim_start_matches("0x")
                .trim_start_matches('$')
                .chars()
                .filter(|&char| char != ':')
                .collect();
            match u32::from_str_radix(&digits, 16) {
                Ok(address) => keeps.push(address),
                Err(_) => panic!("SM_ORACLE_KEEP entry {token:?} is not an address or `input`"),
            }
        }
        // The controller-read entries only work as a group.
        if INPUT_ENTRIES.iter().any(|entry| keeps.contains(entry)) {
            keeps.extend(INPUT_ENTRIES);
        }
        keeps
    })
}

/// Whether `PEA $C2AE` also clears Y, as snesrev's port needed.
pub(crate) fn clears_y_on_enemy_instruction_end() -> bool {
    static CLEARS: OnceLock<bool> = OnceLock::new();
    *CLEARS.get_or_init(|| installs(PatchKind::OtherHook, 0xa08086))
}

/// The bug hooks that keep the original game off unmapped reads. Once the Rust
/// engine follows one, the default oracle reaches those reads too.
const OPEN_BUS_GUARDS: [u32; 1] = [0x91da89];

/// Whether the oracle keeps the CPU data bus value (MDR) and returns it for
/// unmapped reads, as bsnes and hardware do. Only an oracle without an
/// open-bus guard reaches such reads; otherwise it keeps its zero open bus and
/// stops on them.
pub(crate) fn models_open_bus() -> bool {
    static MODELS: OnceLock<bool> = OnceLock::new();
    *MODELS.get_or_init(|| {
        faithful()
            || OPEN_BUS_GUARDS
                .iter()
                .any(|&address| !installs(PatchKind::BugHook, address))
    })
}

thread_local! {
    static NEXT_FRAME_INPUT: std::cell::Cell<Option<u16>> = const { std::cell::Cell::new(None) };
}

/// Supply the controller word of the frame after the one about to run.
///
/// On hardware the NMI that ends a frame latches the input the next frame's
/// logic reads. An oracle that reads the controller there gets the next
/// frame's word for that NMI from the replay; `None` keeps the current one.
pub fn set_next_frame_input(input: Option<u16>) {
    NEXT_FRAME_INPUT.set(input);
}

/// Whether the oracle's NMI reads the controller, as the original game does,
/// instead of snesrev's main loop.
pub fn nmi_reads_controller() -> bool {
    static NMI: OnceLock<bool> = OnceLock::new();
    *NMI.get_or_init(|| !installs(PatchKind::Rom, 0x8095e1))
}

/// The controller word the oracle's end-of-frame NMI latches.
pub(crate) fn nmi_input() -> Option<u16> {
    nmi_reads_controller()
        .then(|| NEXT_FRAME_INPUT.get())
        .flatten()
}
