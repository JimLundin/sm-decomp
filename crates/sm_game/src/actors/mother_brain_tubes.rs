//! The glass tubes that break off and fall when Mother Brain's tank is
//! destroyed: each drops, trails smoke, and bursts into dust on the floor.
//! The tube falling with the brain also ends the screen flash and sets up the
//! neck for the fake death's ascent.
//!
//! Was `MotherBrainsTubesFalling_*` in the original decompilation (bank
//! $A9:8B35-$A9:8C69). Its state words name the slot fields over the same RAM
//! bytes as `EnemyLayout8`.
#![forbid(unsafe_code)]
use crate::actors::layouts::part_view;
use crate::constants::enemy_properties::{kEnemyProps_Deleted, kEnemyProps_Invisible};
use crate::mother_brain_tables;
use crate::ram::RawRam as _;
use crate::ram::{slot, word};

/// What the falling tubes need beyond the shared boss effects.
pub trait Host: crate::actors::host::BossEffects + crate::records::Records {
    /// End the room-wide flash and restore the room palette.
    /// Was `MotherBrain_Pal_EndScreenFlash`.
    fn end_screen_flash(&mut self);
    /// Start an earthquake of `kind`. Was `MotherBrain`'s `start_earthquake`.
    fn start_earthquake(&mut self, kind: u16);
    /// Put the neck in its lowered pose for the fake death's ascent.
    /// Was `MotherBrain_SetupNeckForFakeAscent`.
    fn setup_neck_for_fake_ascent(&mut self);
}

part_view! {
    /// A falling tube's view of its slot.
    Tube: EnemyLayout8 {
        words: [
            function, burst_y, fall_speed, smoke_timer, smoke_index, var_F, parameter_1,
            fall_delay,
        ],
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
/// Sizes and burst heights by tube kind (the `parameter_1` word index).
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
/// Mother Brain's body slot and its brain slot.
const BODY: i32 = 0;
const BRAIN: i32 = 0x40;
/// The body's function once the brain has landed: the fake death's ascent.
pub const BODY_ASCENT: u16 = 0x8c87;

fn current<H: Host + ?Sized>(game: &H) -> i32 {
    word!(game.cur_enemy_index) as i32
}

/// Sets up a tube from its kind. Was `MotherBrainsTubesFalling_Init`.
pub fn initialize<H: Host + ?Sized>(game: &mut H) {
    let me = current(game);
    let kind = slot!(game, Tube[me].parameter_1) as usize >> 1;
    slot!(game, Tube[me].base.x_width = WIDTHS[kind]);
    slot!(game, Tube[me].base.y_height = HEIGHTS[kind]);
    slot!(game, Tube[me].burst_y = BURST_HEIGHTS[kind]);
    slot!(game, Tube[me]; smoke_timer = 0, smoke_index = 0, fall_speed = 0);
    slot!(
        game,
        Tube[me].function = mother_brain_tables::falling_tube(game, kind as isize).function
    );
}

/// Runs the tube's current function. Was `MotherBrainsTubesFalling_Main`.
pub fn main_ai<H: Host + ?Sized>(game: &mut H, k: u16) {
    let me = k as i32;
    match slot!(game, Tube[me].function) {
        function::FALL => fall(game, k),
        function::WAIT_TO_FALL => wait_to_fall(game, k),
        function::FALL_WITH_BRAIN => fall_with_brain(game, k),
        _ => game.unreachable(),
    }
}

/// Adds `amount` (8.8 pixels) to the slot's Y position, carrying through the
/// subpixel high byte. Was `Enemy_IncreaseYpos`.
fn move_down<H: Host + ?Sized>(game: &mut H, me: i32, amount: u16) {
    let carry: i32 =
        slot!(byte game, Tube[me].base.y_subpos[1]) as i32 + amount.to_ne_bytes()[0] as i32;
    slot!(byte game, Tube[me].base.y_subpos[1] = carry as u8);
    slot!(
        game,
        Tube[me].base.y_pos = (slot!(game, Tube[me].base.y_pos) as i32
            + ((amount >> 8) as i8 as i32 + (carry >> 8))) as u16
    );
}

/// Speeds up and falls, trailing smoke until the tube reaches its burst height.
/// Was `MotherBrainsTubesFalling_Main_NonMain`.
fn fall<H: Host + ?Sized>(game: &mut H, k: u16) {
    let me = k as i32;
    let speed = slot!(game, Tube[me].fall_speed).wrapping_add(GRAVITY);
    slot!(game, Tube[me].fall_speed = speed);
    move_down(game, me, speed);
    if (slot!(game, Tube[me].base.y_pos) as i16).wrapping_sub(slot!(game, Tube[me].burst_y) as i16)
        < 0
    {
        trail_smoke(game, k);
    } else {
        burst(game, k);
    }
}

/// Counts down to the next puff of smoke. Was `MotherBrainsTubesFalling_HandleSmoke`.
fn trail_smoke<H: Host + ?Sized>(game: &mut H, k: u16) {
    let me = k as i32;
    slot!(
        game,
        Tube[me].smoke_timer = slot!(game, Tube[me].smoke_timer).wrapping_sub(1)
    );
    if slot!(game, Tube[me].smoke_timer) & 0x8000 != 0 {
        puff_smoke(game, k);
    }
}

/// Deletes the tube and bursts it into dust. Was `MotherBrainsTubesFalling_Explode`.
fn burst<H: Host + ?Sized>(game: &mut H, k: u16) {
    let me = k as i32;
    slot!(
        game,
        Tube[me].base.properties =
            slot!(game, Tube[me].base.properties) | kEnemyProps_Deleted as u16
    );
    let x = slot!(game, Tube[me].base.x_pos);
    let y = slot!(game, Tube[me].base.y_pos);
    game.set_projectile_spawn_point(x, y);
    game.spawn_room_projectile(DUST_CLOUD, BURST_PARAMETER);
    game.queue_sound(1, BURST_SOUND, 3);
}

/// Waits out the fall delay. Was `MotherBrainsTubesFalling_WaitToFall`.
fn wait_to_fall<H: Host + ?Sized>(game: &mut H, k: u16) {
    let me = k as i32;
    slot!(
        game,
        Tube[me].fall_delay = slot!(game, Tube[me].fall_delay).wrapping_sub(1)
    );
    if slot!(game, Tube[me].fall_delay) & 0x8000 != 0 {
        slot!(game, Tube[me].function = function::FALL_WITH_BRAIN);
        fall_with_brain(game, k);
    }
}

/// Falls with the brain above it. When the brain lands it ends the flash,
/// shakes the room, moves the body into place for the fake death's ascent and
/// bursts. Was `MotherBrainsTubesFalling_Falling`.
fn fall_with_brain<H: Host + ?Sized>(game: &mut H, k: u16) {
    let me = k as i32;
    let speed = slot!(game, Tube[me].fall_speed).wrapping_add(GRAVITY);
    slot!(game, Tube[me].fall_speed = speed);
    move_down(game, me, speed);
    let y = slot!(game, Tube[me].base.y_pos);
    if (y as i32 - HIDE_Y as i32) & 0x8000 == 0 {
        slot!(
            game,
            Tube[me].base.properties =
                slot!(game, Tube[me].base.properties) | kEnemyProps_Invisible as u16
        );
    }
    slot!(game, Tube[BRAIN].base.y_pos = y.wrapping_sub(BRAIN_OFFSET));
    if (slot!(game, Tube[BRAIN].base.y_pos) as i32 - BRAIN_LANDING_Y as i32) & 0x8000 != 0 {
        trail_smoke(game, k);
    } else {
        game.end_screen_flash();
        game.start_earthquake(LANDING_EARTHQUAKE);
        crate::hdma_objects::free_hdma_object(0, game);
        crate::hdma_objects::free_hdma_object(2, game);
        slot!(game, Tube[BRAIN].base.y_pos = BRAIN_LANDING_Y);
        slot!(game, Tube[BODY].base.x_pos = BODY_X);
        slot!(game, Tube[BODY].base.y_pos = BODY_Y);
        game.setup_neck_for_fake_ascent();
        slot!(game, Tube[BODY].function = BODY_ASCENT);
        let current = word!(game.cur_enemy_index);
        burst(game, current);
    }
}

/// Puffs smoke below the tube, cycling its horizontal offset.
/// Was `MotherBrainsTubesFalling_SpawnSmoke`.
fn puff_smoke<H: Host + ?Sized>(game: &mut H, k: u16) {
    let me = k as i32;
    slot!(game, Tube[me].smoke_timer = SMOKE_DELAY);
    let index = ((slot!(game, Tube[me].smoke_index) as i32 + 1) & 3) as u16;
    slot!(game, Tube[me].smoke_index = index);
    let x = slot!(game, Tube[me].base.x_pos).wrapping_add(SMOKE_OFFSETS[index as usize]);
    game.set_projectile_spawn_point(x, SMOKE_Y);
    game.spawn_room_projectile(DUST_CLOUD, SMOKE_PARAMETER);
}
