//! Draygon's eye, which looks at Samus and puffs breath bubbles (bank
//! $A5:C46B-$A5:C598), and the function-setting instructions.
//! Common actor fields use the existing owner; outer boss dispatch stays in bank A5.
use crate::actors::host::{ActorLease, LeaseActors};
use crate::enemy_script::EnemyScript;

/// Every 80h frames the eye puffs long breath bubbles (dust cloud / explosion
/// $86:E509, parameter 18h) from 18h px ahead of and 20h px above it.
const BUBBLE_FRAMES: u16 = 0x7f;
const DUST_CLOUD: u16 = 0xe509;
const LONG_BREATH_BUBBLES: u16 = 0x18;
const BUBBLE_X: u16 = 0x18;
const BUBBLE_Y: u16 = 0x20;
/// Angles are a byte; each quarter turn, from 20h before up, picks a look.
const EIGHTH_TURN: u16 = 0x20;
const QUARTER_TURN: usize = 0x40;

/// Initialisation AI - Draygon eye ($A5:C46B).
pub fn initialize(runtime: &mut impl ActorLease) {
    owned::initialize(runtime)
}
/// Instruction - enemy function = [[Y]] ($A5:9736), for actor `k`; with `k`
/// the body or the eye, Draygon body function = [[Y]] ($A5:9F57) or Draygon
/// eye function = [[Y]] ($A5:C47B). Was `set_mode`.
pub fn set_function<H: LeaseActors + ?Sized>(
    k: u16,
    script: EnemyScript,
    game: &mut H,
) -> EnemyScript {
    owned::set_function(k, script, &mut game.lease())
}
/// Draygon eye function - facing left ($A5:C48D) or right ($A5:C513): puff
/// breath bubbles and look towards Samus. Was `advance`.
pub fn look_at_samus<H: LeaseActors + ?Sized>(k: u16, right: bool, game: &mut H) {
    owned::look_at_samus(k, right, &mut game.lease())
}

#[path = "draygon_eye_owned.rs"]
pub mod owned;
