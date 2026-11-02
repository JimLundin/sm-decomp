//! Sidehoppers, desgeegas and their super variants hop small or big about
//! Samus, on the floor or upside down on the ceiling (bank $A3:AA48-$A3:B3A0).
use crate::actors::host::{ActorLease, LeaseActors};
use crate::actors::layouts::part_view;
use crate::enemy_script::EnemyScript;
use sm_data::cartridge::RomImage;

part_view! {
    /// A sidehopper's slot and extended RAM; a non-zero first parameter is upside down.
    Sidehopper: EnemyLayout40 {
        words: [var_a, function, y_speed_index, x_velocity, y_speed_index_delta, var_f, parameter_1, parameter_2],
        pad1: [u8; 26696],
        words: [
            new_instruction_list, small_hop_y_speed_index, big_hop_y_speed_index, falling, ready_to_hop,
            hop_table_index, variant_index,
        ],
        _: [u16; 25],
    }
}

/// Bank $A3 sidehopper functions; hopping backwards sets each jumping original, forwards its clone.
pub mod function {
    pub const HOP: u16 = 0xabd6;
    pub const SMALL_HOP: u16 = 0xabe6;
    pub const BIG_HOP: u16 = 0xac13;
    pub const HOP_UPSIDE_UP: u16 = 0xac40;
    pub const HOP_UPSIDE_DOWN: u16 = 0xac56;
    pub const HOP_BACKWARDS_UPSIDE_UP: u16 = 0xac6c;
    pub const HOP_FORWARDS_UPSIDE_UP: u16 = 0xac8f;
    pub const HOP_BACKWARDS_UPSIDE_DOWN: u16 = 0xaca8;
    pub const HOP_FORWARDS_UPSIDE_DOWN: u16 = 0xaccb;
    pub const LANDED: u16 = 0xace4;
    pub const JUMPING_UPSIDE_UP: u16 = 0xad0e;
    pub const JUMPING_UPSIDE_UP_CLONE: u16 = 0xad20;
    pub const JUMPING_UPSIDE_DOWN: u16 = 0xad32;
    pub const JUMPING_UPSIDE_DOWN_CLONE: u16 = 0xad44;
    pub const WAIT_TO_HOP: u16 = 0xad56;
}

/// $A3:AAC2: landed then hopping instruction lists, upside up then down, by variant.
/// $A3:AAE2: the small and big hop functions, drawn at random.
/// $A3:AAE6: small and big hop Y speed index deltas, X speeds 4 bytes on, heights 8 on.
/// $A2:838F: the quadratic speed table, 8 bytes an entry.
const QUADRATIC_SPEEDS: u32 = 0xa2_838f;
/// Initialisation seeds the random number with 25h; super variants use the
/// second hop table entry; falling speeds up to Y speed table index 40h.
const RANDOM_SEED: u16 = 37;
const SUPER_HOPS: u16 = 2;
const MAX_FALLING_Y_SPEED_INDEX: u16 = 64;
const SIGN: u16 = 0x8000;

#[path = "sidehopper_owned.rs"]
pub mod owned;

pub fn initial_hop_phase(height: u16, step: u16, image: RomImage) -> u16 {
    owned::initial_hop_phase(height, step, image)
}
pub fn initialize(runtime: &mut impl ActorLease) {
    owned::initialize(runtime);
}
pub fn main_ai<H: LeaseActors + ?Sized>(game: &mut H) {
    owned::main_ai(&mut game.lease());
}
pub fn queue_sound<H: LeaseActors + ?Sized>(script: EnemyScript, game: &mut H) -> EnemyScript {
    owned::queue_sound(script, &mut game.lease())
}
pub fn ready_to_hop(script: EnemyScript, runtime: &mut impl ActorLease) -> EnemyScript {
    owned::ready_to_hop(script, runtime)
}
