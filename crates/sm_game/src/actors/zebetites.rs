//! The zebetites: the regenerating barriers in Mother Brain's room. Each one
//! stands alone or as a pair sharing health; the room counts how many have
//! been destroyed in three event flags and places the next one from it.
//!
//! Was `Zebetites` in the original decompilation (bank $A6:FB72-$A6:FD54). Its
//! view names the slot words over the same RAM bytes as `EnemyLayout8`.
#![forbid(unsafe_code)]
use crate::actors::host::{Death, Shot};
use crate::actors::layouts::{EnemyBase, EnemyLayout8};
use crate::constants::enemy_properties::kEnemyProps_Deleted;
use crate::palette_copies::copy_rom_colors_to_palette;
use crate::ram::RawRam as _;
use crate::ram::{slot, word};
use crate::zebetite_data;

/// What the zebetites need beyond the shared boss effects: dying, their tables,
/// the room's event flags and spawning the next zebetites.
pub trait Host:
    crate::actors::host::BossEffects + crate::actors::deaths::Deaths + crate::records::Records
{
    fn event_happened(&mut self, event: u16) -> u16;
    fn set_event(&mut self, event: u16);
    fn clear_event(&mut self, event: u16);
    /// Spawn the bank $A6 enemy population at `population`; the first slot.
    fn spawn_population(&mut self, bank: u8, population: u16) -> u16;
}

/// A zebetite's view of its `EnemyLayout8` slot.
#[derive(Clone, Copy)]
#[repr(C)]
#[allow(non_snake_case)]
pub struct Zebetite {
    pub base: EnemyBase,
    /// Bank $A6 state function, one of [`state`].
    pub state: u16,
    pub var_B: u16,
    /// The glow palette frame, 0 to 7. Only the first enemy slot's is used.
    pub glow_frame: u16,
    /// How many zebetites have been destroyed, from the event flags.
    pub destroyed: u16,
    pub var_E: u16,
    /// Bit 15 set when this placement is a pair of small zebetites.
    pub pair: u16,
    /// Nonzero for the second zebetite of a pair.
    pub second_of_pair: u16,
    /// The other zebetite of the pair.
    pub partner: u16,
}

const _: () = {
    use core::mem::{offset_of, size_of};
    assert!(offset_of!(Zebetite, state) == offset_of!(EnemyLayout8, var_A));
    assert!(offset_of!(Zebetite, glow_frame) == offset_of!(EnemyLayout8, var_C));
    assert!(offset_of!(Zebetite, pair) == offset_of!(EnemyLayout8, var_F));
    assert!(offset_of!(Zebetite, partner) == offset_of!(EnemyLayout8, parameter_2));
    assert!(size_of::<Zebetite>() == size_of::<EnemyLayout8>());
};

/// Bank $A6 state functions, stored in the `state` word.
pub mod state {
    pub const SPAWN_PARTNER: u16 = 0xfc41;
    pub const WAIT_FOR_DOOR: u16 = 0xfc5b;
    pub const ACTIVE: u16 = 0xfc67;
}

/// Bank $A6 enemy populations: the next zebetites, and a pair's partner.
const NEXT_ZEBETITES: u16 = 0xfce1;
const PARTNER: u16 = 0xfcf9;
/// The events whose bits count destroyed zebetites, lowest bit first.
const DESTROYED_EVENTS: [u16; 3] = [3, 4, 5];
/// Four zebetites stand in all.
const ZEBETITES: u16 = 4;
/// Health regrows one point a frame up to this.
const MAX_HEALTH: i16 = 1000;
/// The glow colours: two per frame from $A6:FD87, into palette word 0x158.
const GLOW_COLORS: u16 = 0xfd87;
const GLOW_PALETTE_WORD: u16 = 0x158;
/// The sound of a shot striking a zebetite.
const HIT_SOUND: u8 = 9;

fn current<H: Host + ?Sized>(game: &H) -> i32 {
    word!(game.cur_enemy_index) as i32
}

/// Places the next zebetite from the destroyed count. Was `Zebetites_Init`.
pub fn initialize<H: Host + ?Sized>(game: &mut H) {
    let me = current(game);
    slot!(game, Zebetite[me].base; properties = (slot!(game, Zebetite[me].base.properties)) | (0xa000u16),
        instruction_timer = 1, timer = 0, palette_index = 1024, vram_tiles_index = 128, );
    slot!(game, Zebetite[me].glow_frame = 0);
    let mut first_state = state::SPAWN_PARTNER;
    if slot!(game, Zebetite[me].second_of_pair) != 0 {
        first_state = state::WAIT_FOR_DOOR;
    }
    slot!(game, Zebetite[me].state = first_state);
    let mut destroyed: u16 = 0;
    for event in DESTROYED_EVENTS.into_iter().rev() {
        destroyed = ((destroyed as i32) << 1 | game.event_happened(event) as i32) as u16;
    }
    slot!(game, Zebetite[me].destroyed = destroyed);
    if (destroyed as i32 - ZEBETITES as i32) & 0x8000 != 0 {
        let placement = zebetite_data::placement(game, destroyed as isize);
        slot!(game, Zebetite[me].pair = placement.multi_part);
        slot!(game, Zebetite[me].base.y_height = placement.height);
        slot!(
            game,
            Zebetite[me].base.current_instruction = placement.instruction_list
        );
        slot!(game, Zebetite[me].base.x_pos = placement.x_position);
        let y = if slot!(game, Zebetite[me].second_of_pair) != 0 {
            placement.y_position_alternate
        } else {
            placement.y_position
        };
        slot!(game, Zebetite[me].base.y_pos = y);
    } else {
        slot!(
            game,
            Zebetite[me].base.properties =
                (slot!(game, Zebetite[me].base.properties)) | (kEnemyProps_Deleted as u16)
        );
    }
}

/// Stills the earthquake shake and runs the current state. Was `Zebetites_Main`.
pub fn main_ai<H: Host + ?Sized>(game: &mut H) {
    if word!(game.earthquake_timer) == 0 {
        slot!(game, Zebetite[current(game)].base.shake_timer = 0);
    }
    let me = current(game);
    let state = slot!(game, Zebetite[me].state);
    let k = word!(game.cur_enemy_index);
    match state {
        state::SPAWN_PARTNER => spawn_partner(game, k),
        state::WAIT_FOR_DOOR => wait_for_door(game, k),
        state::ACTIVE => run(game),
        _ => game.unreachable(),
    }
}

/// A pair's first zebetite spawns the second and links them. Was `Zebetites_Func_1`.
fn spawn_partner<H: Host + ?Sized>(game: &mut H, k: u16) {
    let me = k as i32;
    if (slot!(game, Zebetite[me].pair) & 0x8000) != 0 {
        let partner = game.spawn_population(0xa6, PARTNER);
        slot!(game, Zebetite[partner as i32].partner = k);
        slot!(game, Zebetite[k as i32].partner = partner);
    }
    slot!(game, Zebetite[k as i32].state = state::WAIT_FOR_DOOR);
    wait_for_door(game, k);
}

/// Waits for the door transition to finish. Was `Zebetites_Func_2`.
fn wait_for_door<H: Host + ?Sized>(game: &mut H, k: u16) {
    if word!(game.door_transition_flag_elevator_zebetites) == 0 {
        slot!(game, Zebetite[k as i32].state = state::ACTIVE);
        run(game);
    }
}

/// Glows, shows its health stage and regrows; once destroyed, counts itself
/// and makes way for the next zebetite. Was `Zebetites_Func_3`.
fn run<H: Host + ?Sized>(game: &mut H) {
    glow(game);
    show_health_stage(game);
    let me = current(game);
    let health: u16 = slot!(game, Zebetite[me].base.health);
    if health != 0 {
        let mut health: i16 = (health as i16).wrapping_add(1);
        if (health as i32 - MAX_HEALTH as i32) & 0x8000 == 0 {
            health = MAX_HEALTH;
        }
        slot!(game, Zebetite[me].base.health = health as u16);
    } else if slot!(game, Zebetite[me].second_of_pair) != 0 || {
        count_destroyed(game);
        (slot!(game, Zebetite[me].destroyed) as i32 - ZEBETITES as i32) & 0x8000 == 0
    } {
        die(game);
    } else {
        die(game);
        game.spawn_population(0xa6, NEXT_ZEBETITES);
    }
}

fn die<H: Host + ?Sized>(game: &mut H) {
    let _actor = word!(game.cur_enemy_index);
    game.normal_death(0);
}

/// Adds this zebetite to the destroyed count in the event flags. Was
/// `Zebetites_Func_6`/`Zebetites_Func_7`.
fn count_destroyed<H: Host + ?Sized>(game: &mut H) {
    let me = current(game);
    let destroyed: i16 = ((slot!(game, Zebetite[me].destroyed)) as i16).wrapping_add(1);
    slot!(game, Zebetite[me].destroyed = destroyed as u16);
    for (bit, event) in DESTROYED_EVENTS.into_iter().enumerate() {
        if (destroyed as i32 >> bit & 1) as u8 != 0 {
            game.set_event(event);
        } else {
            game.clear_event(event);
        }
    }
}

/// Shows the animation for the current health: five stages from full to
/// almost destroyed. Was `Zebetites_Func_5`.
fn show_health_stage<H: Host + ?Sized>(game: &mut H) {
    let mut stage: u16 = 0;
    let me = current(game);
    let health: u16 = slot!(game, Zebetite[me].base.health);
    for threshold in [800, 600, 400, 200] {
        if (health as i32 - threshold) & 0x8000 != 0 {
            stage += 2;
        } else {
            break;
        }
    }
    let lists = zebetite_data::health_lists(game, (stage as i32 >> 1) as isize);
    let mut animation: u16 = lists.big;
    if (slot!(game, Zebetite[me].pair) & 0x8000) != 0 {
        animation = lists.small_pair;
    }
    slot!(game, Zebetite[me].base; current_instruction = animation, instruction_timer = 1);
    slot!(game, Zebetite[me].base.timer = 0);
}

/// Advances the shared glow one frame, when no palette change is pending and
/// this is a pair's first zebetite. Was `Zebetites_Func_4`.
fn glow<H: Host + ?Sized>(game: &mut H) {
    if word!(game.palette_change_num) == 0
        && slot!(
            game,
            Zebetite[word!(game.cur_enemy_index) as i32].second_of_pair
        ) == 0
    {
        let first: i32 = 0;
        let frame: u16 = ((slot!(game, Zebetite[first].glow_frame) as i32 + 1) & 7) as u16;
        slot!(game, Zebetite[first].glow_frame = frame);
        copy_rom_colors_to_palette(
            GLOW_PALETTE_WORD,
            0xa6,
            GLOW_COLORS.wrapping_add(4 * frame),
            2,
            game,
        );
    }
}

/// Touched by Samus. Was `Zebetites_Touch`.
pub fn touch<H: Host + ?Sized>(game: &mut H) {
    game.contact_reaction(Death::LeavePending);
}

/// Hit by a shot: a pair shares its health and flash. Was `Zebetites_Shot`.
pub fn shot<H: Host + ?Sized>(game: &mut H) {
    game.queue_sound(2, HIT_SOUND, 6);
    game.shot_reaction(Shot::ReactionOnly);
    let me = current(game);
    let partner: i32 = slot!(game, Zebetite[me].partner) as i32;
    slot!(
        game,
        Zebetite[partner].base.health = slot!(game, Zebetite[me].base.health)
    );
    slot!(
        game,
        Zebetite[partner].base.flash_timer = slot!(game, Zebetite[me].base.flash_timer)
    );
}
