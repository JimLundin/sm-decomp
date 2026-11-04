//! The zebetites: the regenerating barriers in Mother Brain's room. Each one
//! stands alone or as a pair sharing health; the room counts how many have
//! been destroyed in three event flags and places the next one from it.
//!
//! Bank $A6:FB72-$A6:FD54.
use crate::actors::boss_lease::BossLease;
use crate::actors::host::{Death, Shot};
use crate::actors::slot_view::{SlotState, slot_view};
use crate::constants::enemy_properties::K_ENEMY_PROPS_DELETED;
use crate::palette_copies::copy_rom_colors_to_palette;
use crate::zebetite_data;

/// The lease the zebetites run on, with the room's event flags and spawning the
/// next zebetites; dying and the shared boss effects are the game's.
pub trait Lease: BossLease + SlotState {
    fn event_happened(&mut self, event: u16) -> u16;
    fn set_event(&mut self, event: u16);
    fn clear_event(&mut self, event: u16);
    /// Spawn the bank $A6 enemy population at `population`; the first slot.
    fn spawn_population(&mut self, bank: u8, population: u16) -> u16;
}

slot_view! {
    /// A zebetite's slot: its bank $A6 state function (one of [`state`]); the
    /// glow palette frame, 0 to 7 (only the first enemy slot's is used); how many
    /// zebetites have been destroyed, from the event flags; bit 15 set when this
    /// placement is a pair of small zebetites; nonzero for the second zebetite of
    /// a pair; and the pair's other zebetite.
    Zebetite, ZebetiteEdit {
        record: [
            state = var_a, glow_frame = var_c, destroyed = var_d, pair = var_f,
            second_of_pair = parameter_1, partner = parameter_2,
        ],
        extended: [],
    }
}

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

fn current<L: Lease>(game: &L) -> u16 {
    game.current_enemy()
}

/// Places the next zebetite from the destroyed count.
pub fn initialize<L: Lease>(game: &mut L) {
    let me = current(game);
    {
        let mut zebetite = ZebetiteEdit::of(game, me);
        let actor = zebetite.actor();
        actor.properties |= 0xa000;
        actor.instruction_delay = 1;
        actor.timer = 0;
        actor.palette = 1024;
        actor.tile = 128;
        *zebetite.glow_frame() = 0;
        let mut first_state = state::SPAWN_PARTNER;
        if *zebetite.second_of_pair() != 0 {
            first_state = state::WAIT_FOR_DOOR;
        }
        *zebetite.state() = first_state;
    }
    let mut destroyed: u16 = 0;
    for event in DESTROYED_EVENTS.into_iter().rev() {
        destroyed = ((destroyed as i32) << 1 | game.event_happened(event) as i32) as u16;
    }
    *ZebetiteEdit::of(game, me).destroyed() = destroyed;
    if (destroyed as i32 - ZEBETITES as i32) & 0x8000 != 0 {
        let placement = zebetite_data::placement(game, destroyed as isize);
        let mut zebetite = ZebetiteEdit::of(game, me);
        *zebetite.pair() = placement.multi_part;
        let actor = zebetite.actor();
        actor.radius.y = placement.height;
        actor.instruction = placement.instruction_list;
        actor.x.whole = placement.x_position;
        let y = if *zebetite.second_of_pair() != 0 {
            placement.y_position_alternate
        } else {
            placement.y_position
        };
        zebetite.actor().y.whole = y;
    } else {
        ZebetiteEdit::of(game, me).actor().properties |= K_ENEMY_PROPS_DELETED as u16;
    }
}

/// Stills the earthquake shake and runs the current state.
pub fn main_ai<L: Lease>(game: &mut L) {
    if game.session().feedback.earthquake_timer == 0 {
        let me = current(game);
        ZebetiteEdit::of(game, me).actor().shake = 0;
    }
    let state = Zebetite::of(game, current(game)).state();
    let k = game.current_enemy();
    match state {
        state::SPAWN_PARTNER => spawn_partner(game, k),
        state::WAIT_FOR_DOOR => wait_for_door(game, k),
        state::ACTIVE => run(game),
        _ => game.unreachable(),
    }
}

/// A pair's first zebetite spawns the second and links them.
fn spawn_partner<L: Lease>(game: &mut L, k: u16) {
    if (Zebetite::of(game, k).pair() & 0x8000) != 0 {
        let partner = game.spawn_population(0xa6, PARTNER);
        *ZebetiteEdit::of(game, partner).partner() = k;
        *ZebetiteEdit::of(game, k).partner() = partner;
    }
    *ZebetiteEdit::of(game, k).state() = state::WAIT_FOR_DOOR;
    wait_for_door(game, k);
}

/// Waits for the door transition to finish.
fn wait_for_door<L: Lease>(game: &mut L, k: u16) {
    if game.session().room.entry.elevator_interlock == 0 {
        *ZebetiteEdit::of(game, k).state() = state::ACTIVE;
        run(game);
    }
}

/// Glows, shows its health stage and regrows; once destroyed, counts itself
/// and makes way for the next zebetite.
fn run<L: Lease>(game: &mut L) {
    glow(game);
    show_health_stage(game);
    let me = current(game);
    let health = Zebetite::of(game, me).actor().health;
    if health != 0 {
        let mut health: i16 = (health as i16).wrapping_add(1);
        if (health as i32 - MAX_HEALTH as i32) & 0x8000 == 0 {
            health = MAX_HEALTH;
        }
        ZebetiteEdit::of(game, me).actor().health = health as u16;
    } else if Zebetite::of(game, me).second_of_pair() != 0 || {
        count_destroyed(game);
        (Zebetite::of(game, me).destroyed() as i32 - ZEBETITES as i32) & 0x8000 == 0
    } {
        die(game);
    } else {
        die(game);
        game.spawn_population(0xa6, NEXT_ZEBETITES);
    }
}

fn die<L: Lease>(game: &mut L) {
    crate::actors::boss_lease::steps::normal_death(game, 0);
}

/// Adds this zebetite to the destroyed count in the event flags.
fn count_destroyed<L: Lease>(game: &mut L) {
    let me = current(game);
    let destroyed = {
        let mut zebetite = ZebetiteEdit::of(game, me);
        let destroyed = zebetite.destroyed();
        *destroyed = (*destroyed as i16).wrapping_add(1) as u16;
        *destroyed
    };
    for (bit, event) in DESTROYED_EVENTS.into_iter().enumerate() {
        if (destroyed as i32 >> bit & 1) as u8 != 0 {
            game.set_event(event);
        } else {
            game.clear_event(event);
        }
    }
}

/// Shows the animation for the current health: five stages from full to
/// almost destroyed.
fn show_health_stage<L: Lease>(game: &mut L) {
    let mut stage: u16 = 0;
    let me = current(game);
    let health = Zebetite::of(game, me).actor().health;
    for threshold in [800, 600, 400, 200] {
        if (health as i32 - threshold) & 0x8000 != 0 {
            stage += 2;
        } else {
            break;
        }
    }
    let lists = zebetite_data::health_lists(game, (stage as i32 >> 1) as isize);
    let mut zebetite = ZebetiteEdit::of(game, me);
    let mut animation: u16 = lists.big;
    if (*zebetite.pair() & 0x8000) != 0 {
        animation = lists.small_pair;
    }
    let actor = zebetite.actor();
    actor.instruction = animation;
    actor.instruction_delay = 1;
    actor.timer = 0;
}

/// Advances the shared glow one frame, when no palette change is pending and
/// this is a pair's first zebetite.
fn glow<L: Lease>(game: &mut L) {
    if crate::sequence_word!(game.PALETTE_CHANGE_NUM) == 0
        && Zebetite::of(game, current(game)).second_of_pair() == 0
    {
        let first = 0;
        let frame = {
            let mut zebetite = ZebetiteEdit::of(game, first);
            let frame = zebetite.glow_frame();
            *frame = ((*frame as i32 + 1) & 7) as u16;
            *frame
        };
        copy_rom_colors_to_palette(
            GLOW_PALETTE_WORD,
            0xa6,
            GLOW_COLORS.wrapping_add(4 * frame),
            2,
            game,
        );
    }
}

/// Touched by Samus.
pub fn touch<L: Lease>(game: &mut L) {
    game.react_to_contact(Death::LeavePending);
}

/// Hit by a shot: a pair shares its health and flash.
pub fn shot<L: Lease>(game: &mut L) {
    game.queue_sound(2, HIT_SOUND, 6);
    game.react_to_shot(Shot::ReactionOnly);
    let me = Zebetite::of(game, current(game));
    let (partner, health) = (me.partner(), me.actor().health);
    ZebetiteEdit::of(game, partner).actor().health = health;
    let flash = Zebetite::of(game, current(game)).actor().flash;
    ZebetiteEdit::of(game, partner).actor().flash = flash;
}
