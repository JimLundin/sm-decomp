//! Draygon's swoops: the swoop Y positions, setups, descents, apexes and
//! ascents, mirrored for each side (bank $A5:87F4-$A5:8B09).
//! The native table remains authoritative and may alias other compatibility state.
use crate::actors::draygon_host::Lease;

/// $7E:9002: the swoop Y positions, one every 4 bytes, up to 800h bytes.
const SWOOP_STEP: u16 = 4;
const SWOOP_TABLE_BYTES: u16 = 0x800;
/// Swoops start descending from Y 180h.
const SWOOP_START_Y: u16 = 0x180;
/// The arms reach out at swoop index 68h.
const ARMS_OUT_INDEX: u16 = 0x68;
/// The right swoop ends at X 2A0h.
const RIGHT_END_X: i32 = 0x2a0;

/// Calculate Draygon swoop Y positions ($A5:8817): fall with the swoop Y
/// acceleration to the reset Y, then set the X speed that reaches Samus.

/// Draygon body function - swoop right ($A5:87F4) or left ($A5:89B3) - setup.
/// Was `begin`.
pub fn swoop_setup<L: Lease>(right: bool, game: &mut L) {
    game.foreign(|game| {
        owned::swoop_setup(
            right,
            &mut crate::runtime_access::RuntimeAccess::runtime(game),
        )
    })
}
/// Draygon body function - swoop right ($A5:8922) or left ($A5:8A50) - apex:
/// set the X speed that reaches the far side in the swoop's duration. Was `turn`.
pub fn swoop_apex<S: owned::Host>(right: bool, game: &mut S) {
    owned::swoop_apex(right, game)
}
/// Draygon body function - swoop right or left - descending ($A5:88B1,
/// $A5:8A00) or, with `ascending`, ascending ($A5:8951, $A5:8A90): follow the
/// swoop Y positions, then pick the next attack. Was `advance`.
pub fn swoop<L: Lease>(right: bool, ascending: bool, game: &mut L) {
    game.foreign(|game| {
        owned::swoop(
            right,
            ascending,
            &mut crate::runtime_access::RuntimeAccess::runtime(game),
        )
    })
}

#[path = "draygon_sweep_owned.rs"]
pub mod owned;
