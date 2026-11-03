//! The glass tubes that break off and fall when Mother Brain's tank is
//! destroyed: each drops, trails smoke, and bursts into dust on the floor.
//! The tube falling with the brain also ends the screen flash and sets up the
//! neck for the fake death's ascent.
//!
//! Was `MotherBrainsTubesFalling_*` in the original decompilation (bank
//! $A9:8B35-$A9:8C69). Its state words are named fields of the Session's slot
//! record, at `EnemyLayout8`'s words.
use crate::actors::mother_brain::{BODY_INDEX, BRAIN_INDEX};
use crate::actors::mother_brain_fight::Lease;
use crate::actors::slot_view::slot_view;
use crate::constants::enemy_properties::{K_ENEMY_PROPS_DELETED, K_ENEMY_PROPS_INVISIBLE};
use crate::mother_brain_tables;

/// What the falling tubes still reach in the compatibility runtime.
pub trait Host {
    /// Start an earthquake of `kind`. Was `MotherBrain`'s `start_earthquake`.
    fn start_earthquake(&mut self, kind: u16);
}

slot_view! {
    /// A falling tube's slot: its function, burst height, fall speed, smoke
    /// timer and index, kind and fall delay.
    Tube, TubeEdit {
        record: [
            function = var_a, burst_y = var_b, fall_speed = var_c, smoke_timer = var_d,
            smoke_index = var_e, kind = parameter_1, fall_delay = parameter_2,
        ],
        extended: [],
    }
}

/// Bank $A9 functions, stored in the `function` word.
pub mod function {
    /// Falls until it reaches its `burst_y`.
    pub const FALL: u16 = 0x8b88;
    /// The tube with the brain: waits `fall_delay` frames before falling.
    pub const WAIT_TO_FALL: u16 = 0x8bcb;
    /// The tube with the brain, falling.
    pub const FALL_WITH_BRAIN: u16 = 0x8bd6;
}

/// The dust cloud projectile and the parameters of a burst and of smoke.
pub const DUST_CLOUD: u16 = 0xe509;
const BURST_PARAMETER: u16 = 3;
const SMOKE_PARAMETER: u16 = 9;
const BURST_SOUND: u8 = 0x24;
/// Sizes and burst heights by tube kind (the `kind` word index).
const WIDTHS: [u16; 5] = [0x10, 0x10, 8, 8, 0x10];
const HEIGHTS: [u16; 5] = [0x20, 0x20, 0x18, 0x18, 0x20];
const BURST_HEIGHTS: [u16; 5] = [0xf8, 0xf8, 0xf0, 0xf0, 0xf6];
/// Horizontal smoke offsets, cycled per puff.
const SMOKE_OFFSETS: [u16; 4] = [0xfff8, 2, 0xfffc, 6];
const SMOKE_Y: u16 = 208;
const SMOKE_DELAY: u16 = 8;
/// Pixels per frame the fall speeds up by.
const GRAVITY: u16 = 6;
/// The tube with the brain: hidden once this low, and the brain is this far above it.
const HIDE_Y: u16 = 244;
const BRAIN_OFFSET: u16 = 56;
/// The brain's landing height and where the fake death puts the body.
const BRAIN_LANDING_Y: u16 = 0xc4;
const BODY_X: u16 = 59;
const BODY_Y: u16 = 279;
/// The earthquake the brain's landing starts.
const LANDING_EARTHQUAKE: u16 = 0x19;
/// The slot the landing's STZ $0FAC,x ($A9:8C12) clears. It meant the tube's
/// Y velocity, but X is the $FE the room palette writes ($A9:D2E4) leave, so
/// it clears $7E:10AA: slot 4's $0FAA, its burst height if it is a tube.
const STALE_SLOT: u16 = 0x100;
/// The body's function once the brain has landed: the fake death's ascent.
pub const BODY_ASCENT: u16 = 0x8c87;

/// Sets up a tube from its kind. Was `MotherBrainsTubesFalling_Init`.
pub fn initialize(game: &mut impl Lease) {
    let k = game.current_enemy();
    let kind = Tube::of(game, k).kind() as usize >> 1;
    TubeEdit::of(game, k).actor().radius.x = WIDTHS[kind];
    TubeEdit::of(game, k).actor().radius.y = HEIGHTS[kind];
    *TubeEdit::of(game, k).burst_y() = BURST_HEIGHTS[kind];
    *TubeEdit::of(game, k).smoke_timer() = 0;
    *TubeEdit::of(game, k).smoke_index() = 0;
    *TubeEdit::of(game, k).fall_speed() = 0;
    *TubeEdit::of(game, k).function() =
        mother_brain_tables::falling_tube(game, kind as isize).function;
}

/// Runs the tube's current function. Was `MotherBrainsTubesFalling_Main`.
pub fn main_ai(game: &mut impl Lease, k: u16) {
    match Tube::of(game, k).function() {
        function::FALL => fall(game, k),
        function::WAIT_TO_FALL => wait_to_fall(game, k),
        function::FALL_WITH_BRAIN => fall_with_brain(game, k),
        _ => game.unreachable(),
    }
}

/// Adds `amount` (8.8 pixels) to the slot's Y position, carrying through the
/// subpixel high byte. Was `Enemy_IncreaseYpos`.
fn move_down(game: &mut impl Lease, k: u16, amount: u16) {
    let carry: i32 = Tube::of(game, k).actor().y.fraction.to_le_bytes()[1] as i32
        + amount.to_ne_bytes()[0] as i32;
    let [low, _] = Tube::of(game, k).actor().y.fraction.to_le_bytes();
    TubeEdit::of(game, k).actor().y.fraction = u16::from_le_bytes([low, carry as u8]);
    TubeEdit::of(game, k).actor().y.whole = (Tube::of(game, k).actor().y.whole as i32
        + ((amount >> 8) as i8 as i32 + (carry >> 8)))
        as u16;
}

/// Speeds up and falls, trailing smoke until the tube reaches its burst height.
/// Was `MotherBrainsTubesFalling_Main_NonMain`.
fn fall(game: &mut impl Lease, k: u16) {
    let speed = Tube::of(game, k).fall_speed().wrapping_add(GRAVITY);
    *TubeEdit::of(game, k).fall_speed() = speed;
    move_down(game, k, speed);
    if (Tube::of(game, k).actor().y.whole as i16).wrapping_sub(Tube::of(game, k).burst_y() as i16)
        < 0
    {
        trail_smoke(game, k);
    } else {
        burst(game, k);
    }
}

/// Counts down to the next puff of smoke. Was `MotherBrainsTubesFalling_HandleSmoke`.
fn trail_smoke(game: &mut impl Lease, k: u16) {
    *TubeEdit::of(game, k).smoke_timer() = Tube::of(game, k).smoke_timer().wrapping_sub(1);
    if Tube::of(game, k).smoke_timer() & 0x8000 != 0 {
        puff_smoke(game, k);
    }
}

/// Deletes the tube and bursts it into dust. Was `MotherBrainsTubesFalling_Explode`.
fn burst(game: &mut impl Lease, k: u16) {
    TubeEdit::of(game, k).actor().properties |= K_ENEMY_PROPS_DELETED as u16;
    let x = Tube::of(game, k).actor().x.whole;
    let y = Tube::of(game, k).actor().y.whole;
    game.owned_step(|game| {
        game.place_projectile_spawn(x, y);
        game.spawn_room_eproj(DUST_CLOUD, BURST_PARAMETER);
    });
    crate::effects::SoundEffects::queue_sound(game, 1, BURST_SOUND, 3);
}

/// Waits out the fall delay. Was `MotherBrainsTubesFalling_WaitToFall`.
fn wait_to_fall(game: &mut impl Lease, k: u16) {
    *TubeEdit::of(game, k).fall_delay() = Tube::of(game, k).fall_delay().wrapping_sub(1);
    if Tube::of(game, k).fall_delay() & 0x8000 != 0 {
        *TubeEdit::of(game, k).function() = function::FALL_WITH_BRAIN;
        fall_with_brain(game, k);
    }
}

/// Falls with the brain above it. When the brain lands it ends the flash,
/// shakes the room, moves the body into place for the fake death's ascent and
/// bursts. Was `MotherBrainsTubesFalling_Falling`.
fn fall_with_brain(game: &mut impl Lease, k: u16) {
    let speed = Tube::of(game, k).fall_speed().wrapping_add(GRAVITY);
    *TubeEdit::of(game, k).fall_speed() = speed;
    move_down(game, k, speed);
    let y = Tube::of(game, k).actor().y.whole;
    if (y as i32 - HIDE_Y as i32) & 0x8000 == 0 {
        TubeEdit::of(game, k).actor().properties |= K_ENEMY_PROPS_INVISIBLE as u16;
    }
    TubeEdit::of(game, BRAIN_INDEX).actor().y.whole = y.wrapping_sub(BRAIN_OFFSET);
    if (Tube::of(game, BRAIN_INDEX).actor().y.whole as i32 - BRAIN_LANDING_Y as i32) & 0x8000 != 0 {
        trail_smoke(game, k);
    } else {
        crate::actors::mother_brain_fight::end_screen_flashing(game);
        game.foreign(|game| {
            Host::start_earthquake(game, LANDING_EARTHQUAKE);
            crate::hdma_objects::free_hdma_object(0, game);
            crate::hdma_objects::free_hdma_object(2, game);
        });
        *TubeEdit::of(game, STALE_SLOT).burst_y() = 0;
        TubeEdit::of(game, BRAIN_INDEX).actor().y.whole = BRAIN_LANDING_Y;
        TubeEdit::of(game, BODY_INDEX).actor().x.whole = BODY_X;
        TubeEdit::of(game, BODY_INDEX).actor().y.whole = BODY_Y;
        crate::actors::mother_brain_neck::setup_for_fake_ascent(game);
        *TubeEdit::of(game, BODY_INDEX).function() = BODY_ASCENT;
        let current = game.current_enemy();
        burst(game, current);
    }
}

/// Puffs smoke below the tube, cycling its horizontal offset.
/// Was `MotherBrainsTubesFalling_SpawnSmoke`.
fn puff_smoke(game: &mut impl Lease, k: u16) {
    *TubeEdit::of(game, k).smoke_timer() = SMOKE_DELAY;
    let index = ((Tube::of(game, k).smoke_index() as i32 + 1) & 3) as u16;
    *TubeEdit::of(game, k).smoke_index() = index;
    let x = Tube::of(game, k)
        .actor()
        .x
        .whole
        .wrapping_add(SMOKE_OFFSETS[index as usize]);
    game.owned_step(|game| {
        game.place_projectile_spawn(x, SMOKE_Y);
        game.spawn_room_eproj(DUST_CLOUD, SMOKE_PARAMETER);
    });
}

// The falling tubes' bank $A9 functions are the ones `CallMotherBrainsTubesFalling` dispatched on.
const _: () = {
    use crate::actors::mother_brain_tubes::{self as tubes, function as f};
    const fn at(word: u16) -> u32 {
        0xa9_0000 | word as u32
    }
    assert!(at(f::FALL) == 11111304 && at(f::WAIT_TO_FALL) == 11111371);
    assert!(at(f::FALL_WITH_BRAIN) == 11111382);
    // MotherBrainBody_FakeDeath_Ascent_0_DrawBG1Row23.
    assert!(at(tubes::BODY_ASCENT) == 11111559);
    assert!(
        tubes::DUST_CLOUD == crate::constants::bank_86::ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16
    );
};
