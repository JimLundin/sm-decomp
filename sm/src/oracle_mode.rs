//! How closely the oracle follows the original ROM.
//!
//! `SM_ORACLE=patched` (the default) keeps snesrev's compatibility patches: the
//! forced carries, bug-fix hooks and ROM patches the C port relied on.
//! `SM_ORACLE=faithful` runs the original game logic instead: patched-carry
//! sites keep the hardware carry, bug-fix hooks and the `PEA $C2AE` Y reset are
//! not installed, and ROM patches that change game logic are skipped. Only the
//! patches that let the oracle step whole frames without a beam remain;
//! `tools/snesrev_patches.py` lists every patch and why it is kept or removed.
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
