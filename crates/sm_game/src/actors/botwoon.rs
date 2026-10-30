//! Botwoon, Maridia's serpent boss, which weaves between four holes in its
//! room's walls (bank $B3:9319-$B3:E2FF).
//! Native-field compatibility retains independent body and callback epochs.
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode::Native, ActorKey as Key};
use crate::actors::api::{DeathHandling, ShotHandling};
use crate::actors::botwoon_death;
use crate::actors::host::{BossEffects as _, Random as _};
use crate::actors::layouts::{EnemyBase, same_words};
use crate::actors::session_slots::session_slot;
use crate::actors::{combat, queries};
use crate::cartridge::rom_table_word;
use crate::constants::{bank_86::*, enemy_properties::*};
use crate::ram::RawRam as _;
use crate::ram::{CompatibilityRam as _, WorkRamAccess as _};
use crate::ram::{slot, word};
use sm_data::botwoon_holes::Hole;
use sm_data::botwoon_speeds::BotwoonSpeed;
use sm_data::enemy_speeds::SpeedTable;

/// Botwoon's view of its slot.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Botwoon {
    pub base: EnemyBase,
    pub body_projectile_index: u16,
    pub position_history_index: u16,
    pub body_travel_time: u16,
    pub function: u16,
    pub movement_function: u16,
    pub head_function: u16,
    pad_0fb4: [u8; 28748],
    pub initial_delay: u16,
    pub spit_timer: u16,
    pub pre_death_counter: u16,
    pub death_counter: u16,
    pub large_wall_explosion_timer: u16,
    pub wall_smoke_timer: u16,
    pub unread_counter: u16,
    pad_800e: [u16; 1],
    pub falling_y_speed_index: u16,
    pad_8012: [u16; 5],
    pub unread_801c: u16,
    pad_801e: [u16; 4],
    pub head_hidden: u16,
    pub previous_head_hidden: u16,
    pub hole_collision_disabled: u16,
    pub target_history_index: u16,
    pub target_hole: u16,
    pub speed: u16,
    pub unread_hole_angle: u16,
    pub target_hole_angle: u16,
    pub instruction_list: u16,
    pub going_through_hole: u16,
    pub spit_angle: u16,
    pad_803c: [u16; 1],
    pub speed_table_index: u16,
    pad_8040: [u8; 1984],
    pub movement_table_index: u16,
    pub unused_8802: u16,
    pub movement_data_pointer: u16,
    pad_8806: [u16; 1],
    pub movement_direction: u16,
    pub smoothed_angle: u16,
    pad_880c: [u16; 5],
    pub unused_8816: u16,
    pad_8818: [u16; 2],
    pub palette_data_offset: u16,
    pub health_palette_index: u16,
    pub x_1_frame_ago: u16,
    pub y_1_frame_ago: u16,
    pub x_2_frames_ago: u16,
    pub y_2_frames_ago: u16,
    pub x_3_frames_ago: u16,
    pub y_3_frames_ago: u16,
    pub x_4_frames_ago: u16,
    pub y_4_frames_ago: u16,
    pad_8830: [u16; 1],
    pub initial_leave_hole_action: u16,
    pub spitting: u16,
    pub max_health: u16,
    pub half_health: u16,
    pub quarter_health: u16,
    pad_883c: [u16; 2],
}

same_words!(Botwoon: EnemyLayout104 {
    body_projectile_index = var_a, position_history_index = var_b, body_travel_time = var_c,
    function = var_d, movement_function = var_e, head_function = var_f, initial_delay = var_20,
    spit_timer = var_21, pre_death_counter = var_22, death_counter = var_23,
    large_wall_explosion_timer = var_24, wall_smoke_timer = var_25, unread_counter = var_26,
    falling_y_speed_index = var_28, unread_801c = var_2e, head_hidden = var_33,
    previous_head_hidden = var_34, hole_collision_disabled = var_35, target_history_index = var_36,
    target_hole = var_37, speed = var_38, unread_hole_angle = var_39, target_hole_angle = var_3a,
    instruction_list = var_3b, going_through_hole = var_3c, spit_angle = var_3d,
    speed_table_index = var_3f, movement_table_index = var_40, unused_8802 = var_41,
    movement_data_pointer = var_42, movement_direction = var_44, smoothed_angle = var_45,
    unused_8816 = var_4b, palette_data_offset = var_4e, health_palette_index = var_4f,
    x_1_frame_ago = var_50, y_1_frame_ago = var_51, x_2_frames_ago = var_52,
    y_2_frames_ago = var_53, x_3_frames_ago = var_54, y_3_frames_ago = var_55,
    x_4_frames_ago = var_56, y_4_frames_ago = var_57, initial_leave_hole_action = var_59,
    spitting = var_5a, max_health = var_5b, half_health = var_5c, quarter_health = var_5d,
});

/// Bank $B3 Botwoon functions, stored in `function`.
pub mod function {
    pub const INITIAL: u16 = 0x9878;
    pub const GO_THROUGH_HOLE: u16 = 0x989d;
    pub const MOVING_AROUND: u16 = 0x99a4;
    pub const SPITTING: u16 = 0x99e4;
    pub const DEATH_PRE_DEATH_DELAY: u16 = 0x9a46;
    pub const DEATH_FALLING_TO_GROUND: u16 = 0x9a5e;
    pub const DEATH_WAIT_FOR_BODY_TO_FALL: u16 = 0x9aca;
    pub const DEATH_CRUMBLING_WALL: u16 = 0x9af9;
}

/// Bank $B3 Botwoon movement functions, stored in `movement_function`.
pub mod movement_function {
    pub const MOVE_TOWARD_TARGET_HOLE: u16 = 0x9bb7;
    pub const START_MOVING_BY_MOVEMENT_DATA: u16 = 0xe250;
    pub const MOVE_BY_MOVEMENT_DATA: u16 = 0xe28c;
}

/// Bank $B3 Botwoon head functions, stored in `head_function`.
pub mod head_function {
    pub const MOVING_AROUND: u16 = 0x9dc0;
    pub const SPITTING_SET_ANGLE_AND_SHOW: u16 = 0x9e7d;
    pub const SPITTING_SPAWN_5: u16 = 0x9ee0;
    pub const SPITTING_SPAWN_3: u16 = 0x9f34;
    pub const SPITTING_COOLDOWN: u16 = 0x9f7a;
}

/// $B3:9389: the hide instruction list.
pub const HIDE: u16 = 0x9389;
const BANK: u32 = 0xb3;
/// Properties bit 10: intangible (the decompiler's `K_ENEMY_PROPS_TANGIBLE`).
const INTANGIBLE: u16 = K_ENEMY_PROPS_TANGIBLE as u16;
const SIGN: i32 = 0x8000;
/// Botwoon trails 24 body projectiles and waits 100h frames before its first move.
const BODY_PROJECTILES: u16 = 24;
const INITIAL_DELAY: u16 = 0x100;
/// $B3:971B: the health-based palettes, 10h bytes each, and $B3:981B their
/// health thresholds; the last of 8 is reached at index 10h.
const HEALTH_PALETTES: u32 = 0xb3_971b;
const HEALTH_THRESHOLDS: u32 = 0xb3_981b;
const LAST_HEALTH_PALETTE: u16 = 16;
const PALETTE_END: u16 = 0x200;
/// Leaving a hole picks one of 8 actions; the first 3 move around, the rest spit.
const LEAVE_HOLE_ACTIONS: u8 = 0xe;
const LAST_MOVING_ACTION: u8 = 4;
const SPIT_DELAY: u16 = 0x30;
/// The pre-death delay lasts 100h frames; the head falls to Y C8h with a dust
/// cloud ($86:E509 parameter 1Dh) and a small explosion (sound 24h).
const PRE_DEATH_FRAMES: i32 = 0x100;
const GROUND_Y: u16 = 200;
const FALL_STEP: u16 = 0xc0;
const DUST_CLOUD_PARAMETER: u16 = 0x1d;
const SMALL_EXPLOSION: u8 = 0x24;
/// Landing makes Botwoon intangible and its graphics drawn last.
const LANDED_PROPERTIES: u16 = 0x500;
/// The wall crumbles from death counter 40h to C0h: large explosions (sprite
/// object 1Dh) every 0Ch frames and two smoke clouds (sprite object 9) every 4,
/// around X E8h and E0h.
const CRUMBLE_START: i32 = 0x40;
const CRUMBLE_END: i32 = 0xc0;
const EXPLOSION_FRAMES: u16 = 12;
const SMOKE_FRAMES: u16 = 4;
const LARGE_EXPLOSION: u16 = 0x1d;
const SMOKE: u16 = 9;
const WALL_SPRITE_PALETTE: u16 = 0xa00;
const EXPLOSION_X: u16 = 0xe8;
const SMOKE_X: u16 = 0xe0;
/// Boss bit 1: Botwoon is dead; the elevator music track plays.
const BOTWOON_DEAD: u16 = 2;
const ELEVATOR_MUSIC: u16 = 3;
/// Bank $84 PLMs: clear Botwoon's wall and crumble it, at block (0Fh, 4).
const CLEAR_WALL: u16 = 0xb797;
const CRUMBLE_WALL: u16 = 0xb79b;
const WALL_BLOCK: (u8, u8) = (15, 4);
/// Maridia's area index.
const MARIDIA: u16 = 4;
/// Hidden, the movement path is one of the second 80h entries.
const HIDDEN_PATHS: i32 = 128;
const PATH_CHOICES: u16 = 0x18;
/// $B3:E150: the movement table, whose entries point at movement data:
/// byte pairs of X and Y steps, ended by 80h.
const MOVEMENT_TABLE: u16 = 0xe150;
const END_OF_DATA: u16 = 0xff80;
/// Offsets to the target hole are clamped to +/-FFh.
const MAX_OFFSET: u16 = 255;
/// $B3:946B: the head's instruction lists by angle, and from $B3:948B, its
/// spit instruction lists.
const HEAD_INSTRUCTION_LISTS: u32 = 0xb3_946b;
const SPIT_INSTRUCTION_LISTS: u32 = 0xb3_948b;
/// Hidden, the head is drawn behind the background (layer 7); shown, layer 2.
const HIDDEN_LAYER: u16 = 7;
const SHOWN_LAYER: u16 = 2;
/// $7E:060B: the spit projectile angle; spits fan out 10h apart.
const SPIT_PROJECTILE_ANGLE: usize = 0x60b;
const SPIT_SPREAD: u8 = 16;
/// $B3:9E77: the spit speeds by speed table index.
const SPIT_SPEEDS: [u16; 3] = [2, 3, 4];
/// Angles are measured from the top, clockwise; 40h less is the maths angle.
const QUARTER_TURN: u8 = 0x40;

/// What Botwoon needs beyond the shared boss effects: its tables, the
/// player angle, the area boss flags, the passage it opens, its drop and
/// the bank $B3 pre-instruction and history hooks.
pub trait Host: crate::actors::host::BossEffects {
    /// The speed level whose two words start at word `index` of $B3:94BB.
    fn botwoon_speed_level(&self, index: isize) -> BotwoonSpeed;
    /// The hole whose four words start at word `index` of $B3:949B.
    fn botwoon_hole(&self, index: isize) -> Hole;
    /// The angle from actor `actor` to Samus.
    fn angle_to_player(&mut self, actor: u16) -> u16;
    /// Set boss bit `mask` for the current area.
    fn set_boss_bit(&mut self, mask: u16);
    /// The boss byte for `area`.
    fn boss_byte(&mut self, area: u16) -> u8;
    fn open_passage(&mut self);
    /// Botwoon's death drop for `area`.
    fn scatter_botwoon_drop(&mut self, area: u16);
    /// Run the bank $B3 pre-instruction at `address` for actor `k`.
    fn b3_pre_instruction(&mut self, address: u32, k: u16);
    fn record_botwoon_head(&mut self);
    fn follow_botwoon_history(&mut self);
    fn orient_botwoon_body(&mut self);
    fn advance_botwoon_history(&mut self);
}

/// The runtime lease Botwoon's death signals and shared reactions run in;
/// `foreign` runs the per-frame steps on the game it leases.
pub trait RuntimeHost:
    crate::actors::effects::ActorHost
    + crate::enemy_projectiles::host::Host
    + crate::room_mechanisms::host::Effects
    + botwoon_death::Host
    + crate::actors::session_slots::SessionSlots
    + crate::ram::CompatibilityRam
    + crate::cartridge::LoadedImage
{
    type Game: Host + ?Sized;
    fn foreign<T>(&mut self, action: impl FnOnce(&mut Self::Game) -> T) -> T;
}

fn initialize_living<H: Host + ?Sized>(e: i32, game: &mut H) {
    slot!(game, Botwoon[e].body_projectile_index = BODY_PROJECTILES);
    loop {
        let k = word!(game.cur_enemy_index);
        game.spawn_projectile(BODY_PROJECTILES, k, ADDR_K_EPROJ_BOTWOONS_BODY as u16);
        slot!(game, Botwoon[e].body_projectile_index, |old| old
            .wrapping_sub(1));
        let finished = (slot!(game, Botwoon[e].body_projectile_index) as i16).wrapping_sub(1) < 0;
        slot!(game, Botwoon[e].body_projectile_index, |old| old
            .wrapping_sub(1));
        if finished {
            break;
        }
    }
    slot!(game, Botwoon[e].base.current_instruction = HIDE);
    let initial = game.botwoon_speed_level(0);
    slot!(game, Botwoon[e]; instruction_list = HIDE, function = function::INITIAL,
        movement_function = movement_function::MOVE_TOWARD_TARGET_HOLE,
        head_function = head_function::MOVING_AROUND, initial_delay = INITIAL_DELAY,
        speed = initial.speed, body_travel_time = initial.body_travel_time,
        head_hidden = 1, previous_head_hidden = 1, initial_leave_hole_action = 1, target_history_index = u16::MAX, target_hole = 0);
    slot!(game, Botwoon[e].base.properties, |old| old | INTANGIBLE);
    let health = slot!(game, Botwoon[e].base.health);
    slot!(game, Botwoon[e]; max_health = health, half_health = health >> 1, quarter_health = (health >> 1) >> 1);
    let x = slot!(game, Botwoon[e].base.x_pos);
    slot!(game, Botwoon[e]; x_1_frame_ago = x, x_2_frames_ago = x, x_3_frames_ago = x, x_4_frames_ago = x);
    let y = slot!(game, Botwoon[e].base.y_pos);
    slot!(game, Botwoon[e]; y_1_frame_ago = y, y_2_frames_ago = y, y_3_frames_ago = y, y_4_frames_ago = y,
        palette_data_offset = ((i32::from(slot!(game, Botwoon[e].base.palette_index)) >> 4) + 0x100) as u16);
}
/// Botwoon health-based palette handling ($B3:982B).
fn health_palette<R: RuntimeHost>(game: &mut R) {
    let e = i32::from(word!(game.cur_enemy_index));
    if session_slot!(game, Botwoon[e].health_palette_index) != LAST_HEALTH_PALETTE
        && (session_slot!(game, Botwoon[e].base.health) as i16).wrapping_sub(rom_table_word!(
            game,
            HEALTH_THRESHOLDS,
            (session_slot!(game, Botwoon[e].health_palette_index) >> 1) as isize
        ) as i16)
            < 0
    {
        let mut source = 16u16.wrapping_mul(session_slot!(game, Botwoon[e].health_palette_index));
        let mut target = session_slot!(game, Botwoon[e].palette_data_offset);
        loop {
            word!(
                game.palette_buffer[target >> 1] =
                    rom_table_word!(game, HEALTH_PALETTES, (source >> 1) as isize)
            );
            source = source.wrapping_add(2);
            target = target.wrapping_add(2);
            if target == PALETTE_END {
                break;
            }
        }
        session_slot!(game, Botwoon[e].health_palette_index, |old| old
            .wrapping_add(2));
    }
}
/// Botwoon function - initial ($B3:9878): wait, then go through a hole. Was
/// `wait_entry`.
fn initial<R: RuntimeHost>(game: &mut R) {
    let e = i32::from(word!(game.cur_enemy_index));
    let delay = session_slot!(game, Botwoon[e].initial_delay).wrapping_sub(1);
    session_slot!(game, Botwoon[e].initial_delay = delay);
    if delay == 0 {
        session_slot!(game, Botwoon[e].function = function::GO_THROUGH_HOLE);
    }
}
/// Botwoon function - go through hole ($B3:989D), leaving it: set up moving
/// around ($B3:98EC) or spitting ($B3:9913). Was `select_attack`.
pub fn leave_hole<H: Host + ?Sized>(e: i32, game: &mut H) -> bool {
    if slot!(game, Botwoon[e].going_through_hole) == 0 {
        return false;
    }
    slot!(game, Botwoon[e].going_through_hole = 0);
    let choice = if slot!(game, Botwoon[e].head_hidden) == 0
        && slot!(game, Botwoon[e].initial_leave_hole_action) == 0
        && slot!(game, Botwoon[e].speed_table_index) == 0
    {
        game.next_random() as u8 & LEAVE_HOLE_ACTIONS
    } else {
        0
    };
    slot!(game, Botwoon[e].initial_leave_hole_action = 0);
    // Masking above admits exactly 0,2,4,6,8,10,12,14.
    let current = i32::from(word!(game.cur_enemy_index));
    if choice <= LAST_MOVING_ACTION {
        slot!(game, Botwoon[current]; function = function::MOVING_AROUND,
            movement_function = movement_function::START_MOVING_BY_MOVEMENT_DATA, spit_timer = 0,
            head_function = head_function::MOVING_AROUND, unused_8816 = 0);
        choose_movement_path(word!(game.cur_enemy_index), game);
    } else {
        slot!(game, Botwoon[current]; function = function::SPITTING,
            head_function = head_function::SPITTING_SET_ANGLE_AND_SHOW, spit_timer = SPIT_DELAY);
        slot!(game, Botwoon[current].base.properties, |old| old
            & !INTANGIBLE);
    }
    true
}
/// Botwoon function - moving around ($B3:99A4): once at a hole, go through it.
/// Was `resume_chase`.
pub fn moving_around<H: Host + ?Sized>(e: i32, game: &mut H) -> bool {
    if slot!(game, Botwoon[e].going_through_hole) == 0 {
        return false;
    }
    slot!(game, Botwoon[e]; going_through_hole = 0, function = function::GO_THROUGH_HOLE,
        movement_function = movement_function::MOVE_TOWARD_TARGET_HOLE);
    if slot!(game, Botwoon[e].head_hidden) != 0 {
        slot!(game, Botwoon[e].hole_collision_disabled = 0);
    }
    true
}
/// Botwoon function - spitting ($B3:99E4): until the spit timer runs out, then
/// move around again. Was `await_spit`.
fn spitting<R: RuntimeHost>(e: i32, runtime: &mut R) -> bool {
    if session_slot!(runtime, Botwoon[e].spit_timer) != 0 {
        session_slot!(runtime, Botwoon[e].unread_801c = 0);
        return true;
    }
    session_slot!(runtime, Botwoon[e]; going_through_hole = 0, function = function::MOVING_AROUND,
        movement_function = movement_function::START_MOVING_BY_MOVEMENT_DATA,
        head_function = head_function::MOVING_AROUND);
    let direct = botwoon_death::pending(runtime, e as u16) || {
        let choice = runtime.foreign(|game| game.next_random()) & 1;
        session_slot!(runtime, Botwoon[e]; head_hidden = choice, previous_head_hidden = choice);
        session_slot!(runtime, Botwoon[e].previous_head_hidden) == 0
    };
    if direct {
        session_slot!(runtime, Botwoon[e].unused_8816 = 0);
    } else {
        session_slot!(runtime, Botwoon[e]; hole_collision_disabled = 0, target_history_index = u16::MAX);
    }
    runtime.foreign(|game| choose_movement_path(word!(game.cur_enemy_index), game));
    false
}
/// Botwoon function - death sequence - pre-death delay ($B3:9A46). Was
/// `wait_collapse`.
fn pre_death_delay<R: RuntimeHost>(game: &mut R) {
    let e = i32::from(word!(game.cur_enemy_index));
    let delay = session_slot!(game, Botwoon[e].pre_death_counter).wrapping_add(1);
    session_slot!(game, Botwoon[e].pre_death_counter = delay);
    if (i32::from(delay) - PRE_DEATH_FRAMES) & SIGN == 0 {
        session_slot!(
            game,
            Botwoon[e].function = function::DEATH_FALLING_TO_GROUND
        );
    }
}
/// Botwoon function - death sequence - falling to ground ($B3:9A5E). Was `fall`.
pub fn fall_to_ground<H: Host + ?Sized>(game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    let index =
        ((8 * i32::from(slot!(byte game, Botwoon[e].falling_y_speed_index[1]))) >> 1) as isize;
    let high = game.enemy_speed(SpeedTable::Quadratic, index + 1);
    let low = game.enemy_speed(SpeedTable::Quadratic, index);
    slot!(
        game,
        Botwoon[e].base(y_pos, y_subpos) += (u32::from(high) << 16) | u32::from(low)
    );
    if (i32::from(slot!(game, Botwoon[e].base.y_pos)) - i32::from(GROUND_Y)) & SIGN != 0 {
        slot!(game, Botwoon[e].falling_y_speed_index, |old| old
            .wrapping_add(FALL_STEP));
    } else {
        slot!(game, Botwoon[e].base.y_pos = GROUND_Y);
        slot!(
            game,
            Botwoon[e].function = function::DEATH_WAIT_FOR_BODY_TO_FALL
        );
        let x = slot!(game, Botwoon[e].base.x_pos);
        let y = slot!(game, Botwoon[e].base.y_pos);
        game.set_projectile_spawn_point(x, y);
        game.spawn_room_projectile(
            ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16,
            DUST_CLOUD_PARAMETER,
        );
        game.queue_sound(1, SMALL_EXPLOSION, 6);
        slot!(game, Botwoon[e].base.properties, |old| old
            | LANDED_PROPERTIES);
    }
}
/// Botwoon function - death sequence - crumbling wall ($B3:9AF9). Was `rubble`.
pub fn crumbling_wall<H: Host + ?Sized>(k: u16, game: &mut H) {
    let e = i32::from(k);
    if (i32::from(slot!(game, Botwoon[e].death_counter)) - CRUMBLE_END) & SIGN != 0 {
        if (i32::from(slot!(game, Botwoon[e].death_counter)) - CRUMBLE_START) & SIGN == 0 {
            let explosion =
                (slot!(game, Botwoon[e].large_wall_explosion_timer) as i16).wrapping_sub(1);
            slot!(
                game,
                Botwoon[e].large_wall_explosion_timer = explosion as u16
            );
            if explosion < 0 {
                slot!(
                    game,
                    Botwoon[e].large_wall_explosion_timer = EXPLOSION_FRAMES
                );
                let x = (game.next_random() & 0x1f) + EXPLOSION_X;
                let y = (i32::from(slot!(game, Botwoon[e].death_counter))
                    + i32::from(game.next_random() & 0x1f)
                    - 8) as u16;
                game.create_sprite(x, y, LARGE_EXPLOSION, WALL_SPRITE_PALETTE);
                game.queue_sound(1, SMALL_EXPLOSION, 6);
            }
            let dust = (slot!(game, Botwoon[e].wall_smoke_timer) as i16).wrapping_sub(1);
            slot!(game, Botwoon[e].wall_smoke_timer = dust as u16);
            if dust < 0 {
                slot!(game, Botwoon[e].wall_smoke_timer = SMOKE_FRAMES);
                for _ in 0..2 {
                    let x = (game.next_random() & 0x3f) + SMOKE_X;
                    let y = (i32::from(slot!(game, Botwoon[e].death_counter))
                        + (i32::from(game.next_random() & 0x1f) - 8))
                        as u16;
                    game.create_sprite(x, y, SMOKE, WALL_SPRITE_PALETTE);
                }
            }
        }
        slot!(game, Botwoon[e].death_counter, |old| old.wrapping_add(1));
    } else {
        slot!(game, Botwoon[e].base.properties, |old| old
            | K_ENEMY_PROPS_DELETED as u16);
        game.set_boss_bit(BOTWOON_DEAD);
        game.queue_music_delayed(ELEVATOR_MUSIC);
    }
}
fn mark_hit<R: RuntimeHost>(h: &mut R, actor: Key) {
    if h.resolve_actor(actor, Native).health == 0 {
        h.set_native_death_signal(actor.byte_index(), botwoon_death::Signal::PendingHit, 1);
        h.edit_actor(Key::from_legacy(0), Native, |actor| {
            actor.properties |= INTANGIBLE;
        });
    }
}
/// Enemy shot - Botwoon.
pub fn shot<R: RuntimeHost>(h: &mut R) {
    let actor = h.selected_actor();
    let health = h.resolve_actor(actor, Native).health;
    h.set_native_death_signal(
        actor.byte_index(),
        botwoon_death::Signal::PreviousHealth,
        health,
    );
    combat::shot_reaction(h, ShotHandling::ReactionOnly);
    mark_hit(h, actor);
}
/// Enemy touch ($B3:9FFF) or, with `power_bomb`, power bomb reaction - Botwoon.
pub fn contact<R: RuntimeHost>(power_bomb: bool, h: &mut R) {
    if power_bomb {
        combat::power_bomb_reaction(h, DeathHandling::LeavePending);
    } else {
        combat::contact_reaction(h, DeathHandling::LeavePending);
    }
    let actor = h.selected_actor();
    mark_hit(h, actor);
}
/// Choose Botwoon movement path ($B3:9933), setting its speed ($B3:995D) by
/// health.
fn choose_movement_path<H: Host + ?Sized>(k: u16, game: &mut H) {
    let e = i32::from(k);
    if slot!(game, Botwoon[e].head_hidden) == 0 {
        slot!(game, Botwoon[e].speed_table_index = 0);
        let health = slot!(game, Botwoon[e].base.health) as i16;
        if health > 0 && health.wrapping_sub(slot!(game, Botwoon[e].half_health) as i16) < 0 {
            let speed = if health.wrapping_sub(slot!(game, Botwoon[e].quarter_health) as i16) < 0 {
                2
            } else {
                1
            };
            slot!(game, Botwoon[e].speed_table_index = speed);
            let index =
                (4u16.wrapping_mul(slot!(game, Botwoon[e].speed_table_index)) >> 1) as isize;
            let level = game.botwoon_speed_level(index);
            slot!(game, Botwoon[e]; speed = level.speed, body_travel_time = level.body_travel_time);
        }
    }
    let bias = if slot!(game, Botwoon[e].head_hidden) != 0 {
        HIDDEN_PATHS
    } else {
        0
    };
    slot!(
        game,
        Botwoon[e].movement_table_index = (i32::from(game.next_random() & PATH_CHOICES)
            + bias
            + 4 * i32::from(slot!(game, Botwoon[e].target_hole)))
            as u16
    );
}
fn clamp_delta(value: u16) -> u16 {
    if (i32::from(value) - i32::from(MAX_OFFSET + 1)) & SIGN == 0 {
        MAX_OFFSET
    } else if (i32::from(value) + i32::from(MAX_OFFSET + 1)) & SIGN != 0 {
        MAX_OFFSET.wrapping_neg()
    } else {
        value
    }
}
/// Botwoon movement function - move directly toward target hole ($B3:9BB7),
/// from the offsets to it ($B3:9BF8). Was `seek_hole`.
pub fn move_toward_target_hole<H: Host + ?Sized>(game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    let target = game.botwoon_hole((slot!(game, Botwoon[e].target_hole) >> 1) as isize);
    let x = clamp_delta(
        (i32::from(target.left) + 4 - i32::from(slot!(game, Botwoon[e].base.x_pos))) as u16,
    );
    let y = clamp_delta(
        (i32::from(target.top) + 4 - i32::from(slot!(game, Botwoon[e].base.y_pos))) as u16,
    );
    let angle = queries::angle_from_delta(x, y);
    let e = i32::from(word!(game.cur_enemy_index));
    slot!(game, Botwoon[e]; unread_hole_angle = angle,
        target_hole_angle = u16::from(QUARTER_TURN.wrapping_sub(angle as u8)));
    if slot!(game, Botwoon[e].head_hidden) == slot!(game, Botwoon[e].previous_head_hidden) {
        move_by_angle(game);
    } else {
        slot!(game, Botwoon[e]; previous_head_hidden = slot!(game, Botwoon[e].head_hidden), going_through_hole = 1);
    }
}
/// Move Botwoon according to speed and angle to target hole ($B3:9C48).
fn move_by_angle<H: Host + ?Sized>(game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    let phase = slot!(game, Botwoon[e].target_hole_angle);
    let amplitude = slot!(game, Botwoon[e].speed);
    let vector = game.actor_direction_vector(phase, amplitude);
    let scratch = slot!(game, Botwoon[e].target_hole_angle);
    game.set_enemy_scratch(scratch);
    let phase = slot!(game, Botwoon[e].target_hole_angle);
    game.apply_vector(phase, vector);
}
/// Botwoon movement function - start moving Botwoon according to movement data
/// ($B3:E250). Was `load_path`.
pub fn start_moving_by_movement_data<H: Host + ?Sized>(game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    slot!(game, Botwoon[e]; movement_function = movement_function::MOVE_BY_MOVEMENT_DATA,
        going_through_hole = 0);
    let record = crate::cartridge::native_record(
        BANK as u8,
        slot!(game, Botwoon[e].movement_table_index).wrapping_add(MOVEMENT_TABLE),
        game,
    );
    slot!(game, Botwoon[e]; movement_data_pointer = record.word(0), target_hole = record.word(4));
    let direction = record.word(2) as i16;
    slot!(game, Botwoon[e].movement_direction = direction as u16);
    if direction < 0 {
        slot!(game, Botwoon[e].movement_data_pointer, |old| old
            .wrapping_sub(4));
    }
    move_by_movement_data(game);
}
/// Botwoon movement function - move Botwoon according to movement data
/// ($B3:E28C), one step per pixel of speed, until the data ends. Was
/// `advance_path`.
pub fn move_by_movement_data<H: Host + ?Sized>(game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    let mut delta = [0u16; 2];
    let mut remaining = i32::from(slot!(game, Botwoon[e].speed));
    let step = if slot!(game, Botwoon[e].movement_direction) as i32 & SIGN == 0 {
        2
    } else {
        -2
    };
    loop {
        let record = crate::cartridge::native_record(
            BANK as u8,
            slot!(game, Botwoon[e].movement_data_pointer),
            game,
        );
        for (axis, total) in delta.iter_mut().enumerate() {
            let amount = (record.byte(axis) as i8) as i16 as u16;
            if amount == END_OF_DATA {
                slot!(game, Botwoon[e]; unused_8802 = 0, going_through_hole = 1);
                return;
            }
            *total = total.wrapping_add(amount);
        }
        slot!(
            game,
            Botwoon[e].movement_data_pointer =
                (i32::from(slot!(game, Botwoon[e].movement_data_pointer)) + step) as u16
        );
        remaining -= 1;
        if remaining == 0 {
            break;
        }
    }
    if slot!(game, Botwoon[e].movement_direction) as i32 & SIGN != 0 {
        delta = delta.map(u16::wrapping_neg);
    }
    slot!(game, Botwoon[e].base.x_pos, |old| old
        .wrapping_add(delta[0]));
    slot!(game, Botwoon[e].base.y_pos, |old| old
        .wrapping_add(delta[1]));
}
/// Start the head's instruction list `instruction`.
fn set_instruction_list<H: Host + ?Sized>(e: i32, instruction: u16, game: &mut H) {
    slot!(game, Botwoon[e].base.current_instruction = instruction);
    slot!(game, Botwoon[e].instruction_list = instruction);
    slot!(game, Botwoon[e].base; instruction_timer = 1, timer = 0);
}
/// Botwoon head function - moving around ($B3:9DC0): hide or face along the
/// head's motion over the last 4 frames, then shift the position history. Was
/// `animate_motion`.
pub fn head_moving_around<H: Host + ?Sized>(game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    let x = slot!(game, Botwoon[e].base.x_pos).wrapping_sub(slot!(game, Botwoon[e].x_4_frames_ago));
    let y = slot!(game, Botwoon[e].base.y_pos).wrapping_sub(slot!(game, Botwoon[e].y_4_frames_ago));
    let angle = queries::angle_from_delta(x, y);
    if x != 0 || y != 0 {
        let instruction = if slot!(game, Botwoon[e].head_hidden) != 0 {
            slot!(game, Botwoon[e].base.layer = HIDDEN_LAYER);
            slot!(game, Botwoon[e].base.properties, |old| old | INTANGIBLE);
            HIDE
        } else {
            slot!(game, Botwoon[e].base.layer = SHOWN_LAYER);
            slot!(game, Botwoon[e].base.properties, |old| old & !INTANGIBLE);
            slot!(
                game,
                Botwoon[e].smoothed_angle = (i32::from(slot!(game, Botwoon[e].smoothed_angle))
                    + i32::from(
                        (angle as i8).wrapping_sub(slot!(game, Botwoon[e].smoothed_angle) as i8)
                    ) * 3
                        / 4) as u8 as u16
            );
            rom_table_word!(
                game,
                HEAD_INSTRUCTION_LISTS,
                (slot!(game, Botwoon[e].smoothed_angle) >> 5) as isize
            )
        };
        if instruction != slot!(game, Botwoon[e].instruction_list) {
            set_instruction_list(e, instruction, game);
        }
    }
    slot!(game, Botwoon[e]; x_4_frames_ago = slot!(game, Botwoon[e].x_3_frames_ago), y_4_frames_ago = slot!(game, Botwoon[e].y_3_frames_ago),
        x_3_frames_ago = slot!(game, Botwoon[e].x_2_frames_ago), y_3_frames_ago = slot!(game, Botwoon[e].y_2_frames_ago),
        x_2_frames_ago = slot!(game, Botwoon[e].x_1_frame_ago), y_2_frames_ago = slot!(game, Botwoon[e].y_1_frame_ago),
        x_1_frame_ago = slot!(game, Botwoon[e].base.x_pos), y_1_frame_ago = slot!(game, Botwoon[e].base.y_pos));
}
/// Botwoon head function - spitting - set angle and show ($B3:9E7D): aim at
/// Samus, then spit 5 or 3; returns that head function. Was `aim_spit`.
pub fn head_set_spit_angle<H: Host + ?Sized>(game: &mut H) -> u32 {
    let e = i32::from(word!(game.cur_enemy_index));
    slot!(game, Botwoon[e].base.layer = SHOWN_LAYER);
    let actor = word!(game.cur_enemy_index);
    let angle = game.angle_to_player(actor);
    slot!(game, Botwoon[e].spit_angle = angle);
    let instruction = rom_table_word!(
        game,
        SPIT_INSTRUCTION_LISTS,
        ((angle as u8).wrapping_add(16) >> 5) as isize
    );
    set_instruction_list(e, instruction, game);
    slot!(game, Botwoon[e];
        spit_angle = u16::from(QUARTER_TURN.wrapping_sub(slot!(game, Botwoon[e].spit_angle) as u8)),
        head_function = head_function::SPITTING_SPAWN_5);
    if slot!(game, Botwoon[e].function) != function::SPITTING {
        slot!(
            game,
            Botwoon[e].head_function = head_function::SPITTING_SPAWN_3
        );
    }
    BANK << 16 | u32::from(slot!(game, Botwoon[e].head_function))
}
/// Botwoon head function - spitting - spawn 5 ($B3:9EE0, `wide`, once the
/// spitting flag is set) or 3 ($B3:9F34) spit projectiles. Was `spit`.
pub fn head_spit<H: Host + ?Sized>(wide: bool, game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    if wide && slot!(game, Botwoon[e].spitting) == 0 {
        return;
    }
    game.ram().write_byte(
        SPIT_PROJECTILE_ANGLE,
        (slot!(game, Botwoon[e].spit_angle) as u8).wrapping_sub(if wide {
            2 * SPIT_SPREAD
        } else {
            SPIT_SPREAD
        }),
    );
    let speed = SPIT_SPEEDS[slot!(game, Botwoon[e].speed_table_index) as usize];
    for _ in 0..if wide { 5 } else { 3 } {
        let k = word!(game.cur_enemy_index);
        game.spawn_projectile(speed, k, ADDR_K_EPROJ_BOTWOONS_SPIT as u16);
        game.ram().write_byte(
            SPIT_PROJECTILE_ANGLE,
            game.ram()
                .read_byte(SPIT_PROJECTILE_ANGLE)
                .wrapping_add(SPIT_SPREAD),
        );
    }
    let target = if wide {
        slot!(game, Botwoon[e].spitting = 0);
        e
    } else {
        i32::from(word!(game.cur_enemy_index))
    };
    slot!(
        game,
        Botwoon[target].head_function = head_function::SPITTING_COOLDOWN
    );
}
/// Botwoon head function - spitting - cooldown ($B3:9F7A). Was `wait_spit`.
pub fn head_cooldown<H: Host + ?Sized>(k: u16, game: &mut H) {
    let e = i32::from(k);
    let delay = (slot!(game, Botwoon[e].spit_timer) as i16).wrapping_sub(1);
    slot!(game, Botwoon[e].spit_timer = delay as u16);
    if delay < 0 {
        slot!(game, Botwoon[e]; spit_timer = 0, head_function = head_function::MOVING_AROUND);
    }
}
/// Botwoon / hole collision detection ($B3:9F93): entering a hole toggles the
/// head between hidden and shown. Was `detect_portal`.
pub fn hole_collision<H: Host + ?Sized>(k: u16, game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    if slot!(game, Botwoon[i32::from(k)].hole_collision_disabled) != 0 {
        return;
    }
    for index in [12, 8, 4, 0] {
        let hole = game.botwoon_hole(index);
        let bounds = [hole.left, hole.right, hole.top, hole.bottom];
        if (0..4).all(|offset| {
            let coordinate = if offset < 2 {
                slot!(game, Botwoon[e].base.x_pos)
            } else {
                slot!(game, Botwoon[e].base.y_pos)
            };
            ((coordinate as i16).wrapping_sub(bounds[offset] as i16) >= 0) == (offset % 2 == 0)
        }) {
            slot!(game, Botwoon[e].hole_collision_disabled = 1);
            slot!(game, Botwoon[e].head_hidden, |old| old ^ 1);
            slot!(
                game,
                Botwoon[e].target_history_index = slot!(game, Botwoon[e].position_history_index)
            );
            slot!(game, Botwoon[e].unread_counter, |old| old.wrapping_sub(1));
            return;
        }
        slot!(game, Botwoon[e].hole_collision_disabled = 0);
    }
}
/// Instruction - enemy radius = `x` x `y` ($B3:94C7, $B3:94D7, $B3:94E7).
pub fn set_hitbox<H: Host + ?Sized>(x: u16, y: u16, game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    slot!(game, Botwoon[e].base; x_width = x, y_height = y);
}
/// Instruction - set Botwoon spitting flag ($B3:9567).
pub fn set_spitting<H: Host + ?Sized>(game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    slot!(game, Botwoon[e].spitting = 1);
}

/// Initialisation AI - Botwoon ($B3:9583): once dead, clear its wall.
pub fn initialize<H: Host + ?Sized>(game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    if u16::from(game.boss_byte(MARIDIA)) & BOTWOON_DEAD != 0 {
        game.spawn_plm(WALL_BLOCK.0, WALL_BLOCK.1, CLEAR_WALL);
        game.open_passage();
        slot!(game, Botwoon[e].base.current_instruction = HIDE);
        slot!(game, Botwoon[e].base.properties, |old| old
            | K_ENEMY_PROPS_DELETED as u16);
    } else {
        initialize_living(e, game);
    }
}
/// Main AI - Botwoon ($B3:9668): run its function, then its palette. Was `update`.
pub fn main_ai<R: RuntimeHost>(runtime: &mut R) {
    botwoon_death::arm(runtime);
    let actor = runtime.selected_actor();
    let state = runtime.resolve_actor(actor, Native).variables[3];
    let k = runtime.selected_actor().byte_index();
    match state {
        function::INITIAL => initial(runtime),
        function::GO_THROUGH_HOLE | function::MOVING_AROUND => runtime.foreign(|game| {
            let e = i32::from(word!(game.cur_enemy_index));
            let changed = if state == function::GO_THROUGH_HOLE {
                leave_hole(e, game)
            } else {
                moving_around(e, game)
            };
            if !changed {
                step_body(e, game);
                if state == function::GO_THROUGH_HOLE {
                    hole_collision(word!(game.cur_enemy_index), game);
                }
            }
        }),
        function::SPITTING => {
            let e = i32::from(runtime.selected_actor().byte_index());
            if spitting(e, runtime) {
                runtime.foreign(|game| {
                    let address = BANK << 16 | u32::from(slot!(game, Botwoon[e].head_function));
                    let k = word!(game.cur_enemy_index);
                    game.b3_pre_instruction(address, k);
                });
            }
        }
        function::DEATH_PRE_DEATH_DELAY => pre_death_delay(runtime),
        function::DEATH_FALLING_TO_GROUND => runtime.foreign(fall_to_ground),
        function::DEATH_WAIT_FOR_BODY_TO_FALL => {
            let actor = runtime.selected_actor();
            if botwoon_death::landed(runtime, actor.byte_index()) {
                runtime.edit_actor(actor, Native, |actor| {
                    actor.variables[3] = function::DEATH_CRUMBLING_WALL
                });
                let k = runtime.selected_actor().byte_index();
                runtime.foreign(|game| {
                    // Spawn Botwoon item drops and start crumbling wall ($B3:9ADD).
                    game.spawn_plm(WALL_BLOCK.0, WALL_BLOCK.1, CRUMBLE_WALL);
                    game.scatter_botwoon_drop(k);
                });
                session_slot!(runtime, Botwoon[i32::from(k)]; death_counter = 0, large_wall_explosion_timer = 0, wall_smoke_timer = 0);
            }
        }
        function::DEATH_CRUMBLING_WALL => runtime.foreign(|game| crumbling_wall(k, game)),
        _ => {
            runtime.foreign(|game| game.unreachable());
        }
    }
    health_palette(runtime);
}
/// Run the movement function, then the head's, carrying the body along the
/// position history.
fn step_body<H: Host + ?Sized>(e: i32, game: &mut H) {
    match slot!(game, Botwoon[e].movement_function) {
        movement_function::MOVE_TOWARD_TARGET_HOLE => move_toward_target_hole(game),
        movement_function::MOVE_BY_MOVEMENT_DATA => move_by_movement_data(game),
        movement_function::START_MOVING_BY_MOVEMENT_DATA => start_moving_by_movement_data(game),
        _ => {
            game.unreachable();
        }
    }
    game.record_botwoon_head();
    game.follow_botwoon_history();
    game.b3_pre_instruction(
        BANK << 16 | u32::from(slot!(game, Botwoon[e].head_function)),
        word!(game.cur_enemy_index),
    );
    game.orient_botwoon_body();
    game.advance_botwoon_history();
}
