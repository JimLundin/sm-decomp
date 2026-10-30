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
    listed(Policy::Kept) || listed(Policy::Changed) || !(faithful() || listed(Policy::Follows))
}

/// Whether an installed "changed" patch uses its variant closer to the original.
pub(crate) fn follows_original(kind: PatchKind, address: u32) -> bool {
    faithful() || kind.sites(Policy::Follows).contains(&address)
}

/// Whether `PEA $C2AE` also clears Y, as snesrev's port needed.
pub(crate) fn clears_y_on_enemy_instruction_end() -> bool {
    static CLEARS: OnceLock<bool> = OnceLock::new();
    *CLEARS.get_or_init(|| installs(PatchKind::OtherHook, 0xa08086))
}
