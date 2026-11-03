//! Botwoon, Maridia's serpent boss, which weaves between four holes in its
//! room's walls (bank $B3:9319-$B3:E2FF).
//! Native-field compatibility retains independent body and callback epochs.
use crate::actors::access::{AccessMode::Native, ActorAccess, ActorKey as Key};
use crate::actors::api::{DeathHandling, ShotHandling};
use crate::actors::boss_interaction as interaction;
use crate::actors::boss_lease::BossLease;
use crate::actors::botwoon_death;
use crate::actors::host::BossEffects as _;

use crate::actors::slot_view::{SlotState, slot_view};
use crate::actors::{combat, queries};
use crate::constants::{bank_86::*, enemy_properties::*};
use crate::enemy_script::EnemyScript;
use sm_data::botwoon_holes::Hole;
use sm_data::botwoon_speeds::BotwoonSpeed;
use sm_data::enemy_speeds::SpeedTable;

/// A lease on a game with Botwoon's effects.
pub trait Lease: BossLease<Game: Effects> + ActorAccess + SlotState {}
impl<T: BossLease<Game: Effects> + ActorAccess + SlotState + ?Sized> Lease for T {}

slot_view! {
    /// Botwoon's slot: a head or body part's functions, history and timers.
    Botwoon, BotwoonEdit {
        record: [
            body_projectile_index = var_a, position_history_index = var_b,
            body_travel_time = var_c, function = var_d, movement_function = var_e,
            head_function = var_f,
        ],
        extended: [
            initial_delay = var_20, spit_timer = var_21, pre_death_counter = var_22,
            death_counter = var_23, large_wall_explosion_timer = var_24,
            wall_smoke_timer = var_25, unread_counter = var_26, falling_y_speed_index = var_28,
            unread_801c = var_2e, head_hidden = var_33, previous_head_hidden = var_34,
            hole_collision_disabled = var_35, target_history_index = var_36, target_hole = var_37,
            speed = var_38, unread_hole_angle = var_39, target_hole_angle = var_3a,
            instruction_list = var_3b, going_through_hole = var_3c, spit_angle = var_3d,
            speed_table_index = var_3f, movement_table_index = var_40, unused_8802 = var_41,
            movement_data_pointer = var_42, movement_direction = var_44, unused_880a = var_45,
            unused_8816 = var_4b, palette_data_offset = var_4e, health_palette_index = var_4f,
            x_1_frame_ago = var_50, y_1_frame_ago = var_51, x_2_frames_ago = var_52,
            y_2_frames_ago = var_53, x_3_frames_ago = var_54, y_3_frames_ago = var_55,
            x_4_frames_ago = var_56, y_4_frames_ago = var_57, initial_leave_hole_action = var_59,
            spitting = var_5a, max_health = var_5b, half_health = var_5c, quarter_health = var_5d,
        ],
    }
}

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
/// Hidden, the head is drawn behind the background (layer 7); shown, layer 2.
const HIDDEN_LAYER: u16 = 7;
const SHOWN_LAYER: u16 = 2;
/// Spits fan out 10h apart in the spit projectile angle ($7E:060B).
const SPIT_SPREAD: u8 = 16;
/// $B3:9E77: the spit speeds by speed table index.
const SPIT_SPEEDS: [u16; 3] = [2, 3, 4];
/// Angles are measured from the top, clockwise; 40h less is the maths angle.
const QUARTER_TURN: u8 = 0x40;

/// The retained imported tables, boss flags, passage, drop and body-history
/// effects. Botwoon's actor state and head dispatch stay on the lease.
pub trait Effects: crate::actors::host::BossEffects {
    /// The speed level whose two words start at word `index` of $B3:94BB.
    fn botwoon_speed_level(&self, index: isize) -> BotwoonSpeed;
    /// The hole whose four words start at word `index` of $B3:949B.
    fn botwoon_hole(&self, index: isize) -> Hole;
    /// Set boss bit `mask` for the current area.
    fn set_boss_bit(&mut self, mask: u16);
    /// The boss byte for `area`.
    fn boss_byte(&mut self, area: u16) -> u8;
    fn open_passage(&mut self);
    /// Botwoon's death drop for `area`.
    fn scatter_botwoon_drop(&mut self, area: u16);
    fn record_botwoon_head(&mut self);
    fn follow_botwoon_history(&mut self);
    fn orient_botwoon_body(&mut self);
    fn advance_botwoon_history(&mut self);
}

/// Botwoon's state and reactions run on the scheduler's lease.
pub trait RuntimeHost:
    Lease
    + crate::actors::effects::ActorHost
    + crate::enemy_projectiles::host::Host
    + crate::room_mechanisms::host::Effects
    + botwoon_death::Host
    + crate::actors::boss_interaction::Access
{
}
impl<T> RuntimeHost for T where
    T: crate::actors::host::ActorLease
        + BossLease<Game = <T as crate::actors::host::ActorLease>::Game>
        + SlotState
        + crate::actors::boss_interaction::Access
        + ?Sized
{
}

fn initialize_living<L: Lease + ?Sized>(e: u16, game: &mut L) {
    *BotwoonEdit::of(game, e).body_projectile_index() = BODY_PROJECTILES;
    loop {
        let k = game.current_enemy();
        game.spawn_projectile(BODY_PROJECTILES, k, ADDR_K_EPROJ_BOTWOONS_BODY as u16);
        {
            let mut botwoon = BotwoonEdit::of(game, e);
            let old = botwoon.body_projectile_index();
            *old = old.wrapping_sub(1);
        }
        let finished = (Botwoon::of(game, e).body_projectile_index() as i16).wrapping_sub(1) < 0;
        {
            let mut botwoon = BotwoonEdit::of(game, e);
            let old = botwoon.body_projectile_index();
            *old = old.wrapping_sub(1);
        }
        if finished {
            break;
        }
    }
    BotwoonEdit::of(game, e).actor().instruction = HIDE;
    let initial = game.foreign(|game| game.botwoon_speed_level(0));
    {
        let mut botwoon = BotwoonEdit::of(game, e);
        *botwoon.instruction_list() = HIDE;
        *botwoon.function() = function::INITIAL;
        *botwoon.movement_function() = movement_function::MOVE_TOWARD_TARGET_HOLE;
        *botwoon.head_function() = head_function::MOVING_AROUND;
        *botwoon.initial_delay() = INITIAL_DELAY;
        *botwoon.speed() = initial.speed;
        *botwoon.body_travel_time() = initial.body_travel_time;
        *botwoon.head_hidden() = 1;
        *botwoon.previous_head_hidden() = 1;
        *botwoon.initial_leave_hole_action() = 1;
        *botwoon.target_history_index() = u16::MAX;
        *botwoon.target_hole() = 0;
    }
    BotwoonEdit::of(game, e).actor().properties |= INTANGIBLE;
    let health = Botwoon::of(game, e).actor().health;
    {
        let mut botwoon = BotwoonEdit::of(game, e);
        *botwoon.max_health() = health;
        *botwoon.half_health() = health >> 1;
        *botwoon.quarter_health() = (health >> 1) >> 1;
    }
    let x = Botwoon::of(game, e).actor().x.whole;
    {
        let mut botwoon = BotwoonEdit::of(game, e);
        *botwoon.x_1_frame_ago() = x;
        *botwoon.x_2_frames_ago() = x;
        *botwoon.x_3_frames_ago() = x;
        *botwoon.x_4_frames_ago() = x;
    }
    let y = Botwoon::of(game, e).actor().y.whole;
    *BotwoonEdit::of(game, e).y_1_frame_ago() = y;
    *BotwoonEdit::of(game, e).y_2_frames_ago() = y;
    *BotwoonEdit::of(game, e).y_3_frames_ago() = y;
    *BotwoonEdit::of(game, e).y_4_frames_ago() = y;
    *BotwoonEdit::of(game, e).palette_data_offset() =
        ((i32::from(Botwoon::of(game, e).actor().palette) >> 4) + 0x100) as u16;
}
/// Botwoon health-based palette handling ($B3:982B).
fn health_palette<R: RuntimeHost>(game: &mut R) {
    let e = game.current_enemy();
    if Botwoon::of(game, e).health_palette_index() != LAST_HEALTH_PALETTE
        && (Botwoon::of(game, e).actor().health as i16).wrapping_sub(
            crate::cartridge::rom_table_word!(
                game,
                HEALTH_THRESHOLDS,
                (Botwoon::of(game, e).health_palette_index() >> 1) as isize,
            ) as i16,
        ) < 0
    {
        let mut source = 16u16.wrapping_mul(Botwoon::of(game, e).health_palette_index());
        let mut target = Botwoon::of(game, e).palette_data_offset();
        loop {
            interaction::set_at(
                game,
                interaction::Array::PaletteBuffer,
                target >> 1,
                crate::cartridge::rom_table_word!(game, HEALTH_PALETTES, (source >> 1) as isize),
            );
            source = source.wrapping_add(2);
            target = target.wrapping_add(2);
            if target == PALETTE_END {
                break;
            }
        }
        {
            let mut botwoon = BotwoonEdit::of(game, e);
            let old = botwoon.health_palette_index();
            *old = old.wrapping_add(2);
        }
    }
}
/// Botwoon function - initial ($B3:9878): wait, then go through a hole. Was
/// `wait_entry`.
fn initial<R: RuntimeHost>(game: &mut R) {
    let e = game.current_enemy();
    let delay = Botwoon::of(game, e).initial_delay().wrapping_sub(1);
    *BotwoonEdit::of(game, e).initial_delay() = delay;
    if delay == 0 {
        *BotwoonEdit::of(game, e).function() = function::GO_THROUGH_HOLE;
    }
}
/// Botwoon function - go through hole ($B3:989D), leaving it: set up moving
/// around ($B3:98EC) or spitting ($B3:9913). Was `select_attack`.
pub fn leave_hole<L: Lease + ?Sized>(e: u16, game: &mut L) -> bool {
    if Botwoon::of(game, e).going_through_hole() == 0 {
        return false;
    }
    *BotwoonEdit::of(game, e).going_through_hole() = 0;
    let choice = if Botwoon::of(game, e).head_hidden() == 0
        && Botwoon::of(game, e).initial_leave_hole_action() == 0
        && Botwoon::of(game, e).speed_table_index() == 0
    {
        game.roll_random() as u8 & LEAVE_HOLE_ACTIONS
    } else {
        0
    };
    *BotwoonEdit::of(game, e).initial_leave_hole_action() = 0;
    // Masking above admits exactly 0,2,4,6,8,10,12,14.
    let current = game.current_enemy();
    if choice <= LAST_MOVING_ACTION {
        {
            let mut botwoon = BotwoonEdit::of(game, current);
            *botwoon.function() = function::MOVING_AROUND;
            *botwoon.movement_function() = movement_function::START_MOVING_BY_MOVEMENT_DATA;
            *botwoon.spit_timer() = 0;
            *botwoon.head_function() = head_function::MOVING_AROUND;
            *botwoon.unused_8816() = 0;
        }
        choose_movement_path(game.current_enemy(), game);
    } else {
        {
            let mut botwoon = BotwoonEdit::of(game, current);
            *botwoon.function() = function::SPITTING;
            *botwoon.head_function() = head_function::SPITTING_SET_ANGLE_AND_SHOW;
            *botwoon.spit_timer() = SPIT_DELAY;
        }
        BotwoonEdit::of(game, current).actor().properties &= !INTANGIBLE;
    }
    true
}
/// Botwoon function - moving around ($B3:99A4): once at a hole, go through it.
/// Was `resume_chase`.
pub fn moving_around<L: Lease + ?Sized>(e: u16, game: &mut L) -> bool {
    if Botwoon::of(game, e).going_through_hole() == 0 {
        return false;
    }
    {
        let mut botwoon = BotwoonEdit::of(game, e);
        *botwoon.going_through_hole() = 0;
        *botwoon.function() = function::GO_THROUGH_HOLE;
        *botwoon.movement_function() = movement_function::MOVE_TOWARD_TARGET_HOLE;
    }
    if Botwoon::of(game, e).head_hidden() != 0 {
        *BotwoonEdit::of(game, e).hole_collision_disabled() = 0;
    }
    true
}
/// Botwoon function - spitting ($B3:99E4): until the spit timer runs out, then
/// move around again. Was `await_spit`.
fn spitting<R: RuntimeHost>(e: u16, runtime: &mut R) -> bool {
    if Botwoon::of(runtime, e).spit_timer() != 0 {
        *BotwoonEdit::of(runtime, e).unread_801c() = 0;
        return true;
    }
    {
        let mut botwoon = BotwoonEdit::of(runtime, e);
        *botwoon.going_through_hole() = 0;
        *botwoon.function() = function::MOVING_AROUND;
        *botwoon.movement_function() = movement_function::START_MOVING_BY_MOVEMENT_DATA;
        *botwoon.head_function() = head_function::MOVING_AROUND;
    }
    let direct = botwoon_death::pending(runtime, e) || {
        let choice = runtime.roll_random() & 1;
        {
            let mut botwoon = BotwoonEdit::of(runtime, e);
            *botwoon.head_hidden() = choice;
            *botwoon.previous_head_hidden() = choice;
        }
        Botwoon::of(runtime, e).previous_head_hidden() == 0
    };
    if direct {
        *BotwoonEdit::of(runtime, e).unused_8816() = 0;
    } else {
        {
            let mut botwoon = BotwoonEdit::of(runtime, e);
            *botwoon.hole_collision_disabled() = 0;
            *botwoon.target_history_index() = u16::MAX;
        }
    }
    choose_movement_path(runtime.current_enemy(), runtime);
    false
}
/// Botwoon function - death sequence - pre-death delay ($B3:9A46). Was
/// `wait_collapse`.
fn pre_death_delay<R: RuntimeHost>(game: &mut R) {
    let e = game.current_enemy();
    let delay = Botwoon::of(game, e).pre_death_counter().wrapping_add(1);
    *BotwoonEdit::of(game, e).pre_death_counter() = delay;
    if (i32::from(delay) - PRE_DEATH_FRAMES) & SIGN == 0 {
        *BotwoonEdit::of(game, e).function() = function::DEATH_FALLING_TO_GROUND;
    }
}
/// Botwoon function - death sequence - falling to ground ($B3:9A5E). Was `fall`.
pub fn fall_to_ground<L: Lease + ?Sized>(game: &mut L) {
    let e = game.current_enemy();
    let index = ((8 * i32::from(Botwoon::of(game, e).falling_y_speed_index().to_le_bytes()[1]))
        >> 1) as isize;
    let high = game.enemy_speed(SpeedTable::Quadratic, index + 1);
    let low = game.enemy_speed(SpeedTable::Quadratic, index);
    {
        let value: u32 = (u32::from(high) << 16) | u32::from(low);
        let mut botwoon = BotwoonEdit::of(game, e);
        let sum = ((u32::from(botwoon.actor().y.whole) << 16)
            | u32::from(botwoon.actor().y.fraction))
        .wrapping_add(value);
        botwoon.actor().y.fraction = sum as u16;
        botwoon.actor().y.whole = (sum >> 16) as u16;
    }
    if (i32::from(Botwoon::of(game, e).actor().y.whole) - i32::from(GROUND_Y)) & SIGN != 0 {
        {
            let mut botwoon = BotwoonEdit::of(game, e);
            let old = botwoon.falling_y_speed_index();
            *old = old.wrapping_add(FALL_STEP);
        }
    } else {
        BotwoonEdit::of(game, e).actor().y.whole = GROUND_Y;
        *BotwoonEdit::of(game, e).function() = function::DEATH_WAIT_FOR_BODY_TO_FALL;
        let x = Botwoon::of(game, e).actor().x.whole;
        let y = Botwoon::of(game, e).actor().y.whole;
        game.place_projectile_spawn(x, y);
        game.spawn_room_projectile(
            ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16,
            DUST_CLOUD_PARAMETER,
        );
        game.queue_sound(1, SMALL_EXPLOSION, 6);
        BotwoonEdit::of(game, e).actor().properties |= LANDED_PROPERTIES;
    }
}
/// Botwoon function - death sequence - crumbling wall ($B3:9AF9). Was `rubble`.
pub fn crumbling_wall<L: Lease + ?Sized>(k: u16, game: &mut L) {
    let e = k;
    if (i32::from(Botwoon::of(game, e).death_counter()) - CRUMBLE_END) & SIGN != 0 {
        if (i32::from(Botwoon::of(game, e).death_counter()) - CRUMBLE_START) & SIGN == 0 {
            let explosion =
                (Botwoon::of(game, e).large_wall_explosion_timer() as i16).wrapping_sub(1);
            *BotwoonEdit::of(game, e).large_wall_explosion_timer() = explosion as u16;
            if explosion < 0 {
                *BotwoonEdit::of(game, e).large_wall_explosion_timer() = EXPLOSION_FRAMES;
                let x = (game.roll_random() & 0x1f) + EXPLOSION_X;
                let y = (i32::from(Botwoon::of(game, e).death_counter())
                    + i32::from(game.roll_random() & 0x1f)
                    - 8) as u16;
                game.foreign(|game| game.create_sprite(x, y, LARGE_EXPLOSION, WALL_SPRITE_PALETTE));
                game.queue_sound(1, SMALL_EXPLOSION, 6);
            }
            let dust = (Botwoon::of(game, e).wall_smoke_timer() as i16).wrapping_sub(1);
            *BotwoonEdit::of(game, e).wall_smoke_timer() = dust as u16;
            if dust < 0 {
                *BotwoonEdit::of(game, e).wall_smoke_timer() = SMOKE_FRAMES;
                for _ in 0..2 {
                    let x = (game.roll_random() & 0x3f) + SMOKE_X;
                    let y = (i32::from(Botwoon::of(game, e).death_counter())
                        + (i32::from(game.roll_random() & 0x1f) - 8))
                        as u16;
                    game.foreign(|game| game.create_sprite(x, y, SMOKE, WALL_SPRITE_PALETTE));
                }
            }
        }
        {
            let mut botwoon = BotwoonEdit::of(game, e);
            let old = botwoon.death_counter();
            *old = old.wrapping_add(1);
        }
    } else {
        BotwoonEdit::of(game, e).actor().properties |= K_ENEMY_PROPS_DELETED as u16;
        game.foreign(|game| {
            game.set_boss_bit(BOTWOON_DEAD);
            game.queue_music_delayed(ELEVATOR_MUSIC);
        });
    }
}
fn mark_hit<R: RuntimeHost>(h: &mut R, actor: Key) {
    if h.resolve_actor(actor, Native).health == 0 {
        h.set_death_signal(actor.byte_index(), botwoon_death::Signal::PendingHit, 1);
        h.edit_actor(Key::from_legacy(0), Native, |actor| {
            actor.properties |= INTANGIBLE;
        });
    }
}
/// Enemy shot - Botwoon.
pub fn shot<R: RuntimeHost>(h: &mut R) {
    let actor = h.selected_actor();
    let health = h.resolve_actor(actor, Native).health;
    h.set_death_signal(
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
fn choose_movement_path<L: Lease + ?Sized>(k: u16, game: &mut L) {
    let e = k;
    if Botwoon::of(game, e).head_hidden() == 0 {
        *BotwoonEdit::of(game, e).speed_table_index() = 0;
        let health = Botwoon::of(game, e).actor().health as i16;
        if health > 0 && health.wrapping_sub(Botwoon::of(game, e).half_health() as i16) < 0 {
            let speed = if health.wrapping_sub(Botwoon::of(game, e).quarter_health() as i16) < 0 {
                2
            } else {
                1
            };
            *BotwoonEdit::of(game, e).speed_table_index() = speed;
            let index = (4u16.wrapping_mul(Botwoon::of(game, e).speed_table_index()) >> 1) as isize;
            let level = game.foreign(|game| game.botwoon_speed_level(index));
            {
                let mut botwoon = BotwoonEdit::of(game, e);
                *botwoon.speed() = level.speed;
                *botwoon.body_travel_time() = level.body_travel_time;
            }
        }
    }
    let bias = if Botwoon::of(game, e).head_hidden() != 0 {
        HIDDEN_PATHS
    } else {
        0
    };
    *BotwoonEdit::of(game, e).movement_table_index() =
        (i32::from(game.roll_random() & PATH_CHOICES)
            + bias
            + 4 * i32::from(Botwoon::of(game, e).target_hole())) as u16;
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
pub fn move_toward_target_hole<L: Lease + ?Sized>(game: &mut L) {
    let e = game.current_enemy();
    let index = (Botwoon::of(game, e).target_hole() >> 1) as isize;
    let target = game.foreign(|game| game.botwoon_hole(index));
    let x = clamp_delta(
        (i32::from(target.left) + 4 - i32::from(Botwoon::of(game, e).actor().x.whole)) as u16,
    );
    let y = clamp_delta(
        (i32::from(target.top) + 4 - i32::from(Botwoon::of(game, e).actor().y.whole)) as u16,
    );
    let angle = queries::angle_from_delta(x, y);
    let e = game.current_enemy();
    {
        let mut botwoon = BotwoonEdit::of(game, e);
        *botwoon.unread_hole_angle() = angle;
        *botwoon.target_hole_angle() = u16::from(QUARTER_TURN.wrapping_sub(angle as u8));
    }
    if Botwoon::of(game, e).head_hidden() == Botwoon::of(game, e).previous_head_hidden() {
        move_by_angle(game);
    } else {
        *BotwoonEdit::of(game, e).previous_head_hidden() = Botwoon::of(game, e).head_hidden();
        *BotwoonEdit::of(game, e).going_through_hole() = 1;
    }
}
/// Move Botwoon according to speed and angle to target hole ($B3:9C48).
fn move_by_angle<L: Lease + ?Sized>(game: &mut L) {
    let e = game.current_enemy();
    let phase = Botwoon::of(game, e).target_hole_angle();
    let amplitude = Botwoon::of(game, e).speed();
    let vector = game.direction_step(phase, amplitude);
    let scratch = Botwoon::of(game, e).target_hole_angle();
    game.set_step_scratch(scratch);
    let phase = Botwoon::of(game, e).target_hole_angle();
    game.step_along(phase, vector);
}
/// Botwoon movement function - start moving Botwoon according to movement data
/// ($B3:E250). Was `load_path`.
pub fn start_moving_by_movement_data<L: Lease + ?Sized>(game: &mut L) {
    let e = game.current_enemy();
    {
        let mut botwoon = BotwoonEdit::of(game, e);
        *botwoon.movement_function() = movement_function::MOVE_BY_MOVEMENT_DATA;
        *botwoon.going_through_hole() = 0;
    }
    let record = crate::cartridge::native_record(
        BANK as u8,
        Botwoon::of(game, e)
            .movement_table_index()
            .wrapping_add(MOVEMENT_TABLE),
        game,
    );
    {
        let mut botwoon = BotwoonEdit::of(game, e);
        *botwoon.movement_data_pointer() = record.word(0);
        *botwoon.target_hole() = record.word(4);
    }
    let direction = record.word(2) as i16;
    *BotwoonEdit::of(game, e).movement_direction() = direction as u16;
    if direction < 0 {
        {
            let mut botwoon = BotwoonEdit::of(game, e);
            let old = botwoon.movement_data_pointer();
            *old = old.wrapping_sub(4);
        }
    }
    move_by_movement_data(game);
}
/// Botwoon movement function - move Botwoon according to movement data
/// ($B3:E28C), one step per pixel of speed, until the data ends. Was
/// `advance_path`.
pub fn move_by_movement_data<L: Lease + ?Sized>(game: &mut L) {
    let e = game.current_enemy();
    let mut delta = [0u16; 2];
    let mut remaining = i32::from(Botwoon::of(game, e).speed());
    let step = if Botwoon::of(game, e).movement_direction() as i32 & SIGN == 0 {
        2
    } else {
        -2
    };
    loop {
        let record = crate::cartridge::native_record(
            BANK as u8,
            Botwoon::of(game, e).movement_data_pointer(),
            game,
        );
        for (axis, total) in delta.iter_mut().enumerate() {
            let amount = (record.byte(axis) as i8) as i16 as u16;
            if amount == END_OF_DATA {
                {
                    let mut botwoon = BotwoonEdit::of(game, e);
                    *botwoon.unused_8802() = 0;
                    *botwoon.going_through_hole() = 1;
                }
                return;
            }
            *total = total.wrapping_add(amount);
        }
        {
            let mut botwoon = BotwoonEdit::of(game, e);
            let old = botwoon.movement_data_pointer();
            *old = (i32::from(*old) + step) as u16;
        }
        remaining -= 1;
        if remaining == 0 {
            break;
        }
    }
    if Botwoon::of(game, e).movement_direction() as i32 & SIGN != 0 {
        delta = delta.map(u16::wrapping_neg);
    }
    {
        let mut botwoon = BotwoonEdit::of(game, e);
        let old = &mut botwoon.actor().x.whole;
        *old = old.wrapping_add(delta[0]);
    }
    {
        let mut botwoon = BotwoonEdit::of(game, e);
        let old = &mut botwoon.actor().y.whole;
        *old = old.wrapping_add(delta[1]);
    }
}
/// Start the head's instruction list `instruction`.
fn set_instruction_list<L: Lease + ?Sized>(e: u16, instruction: u16, game: &mut L) {
    BotwoonEdit::of(game, e).actor().instruction = instruction;
    *BotwoonEdit::of(game, e).instruction_list() = instruction;
    {
        let mut botwoon = BotwoonEdit::of(game, e);
        let actor = botwoon.actor();
        actor.instruction_delay = 1;
        actor.timer = 0;
    }
}
/// Botwoon head function - moving around ($B3:9DC0): hide or face along the
/// head's motion over the last 4 frames, then shift the position history. Was
/// `animate_motion`.
pub fn head_moving_around<L: Lease + ?Sized>(game: &mut L) {
    let e = game.current_enemy();
    let x = Botwoon::of(game, e)
        .actor()
        .x
        .whole
        .wrapping_sub(Botwoon::of(game, e).x_4_frames_ago());
    let y = Botwoon::of(game, e)
        .actor()
        .y
        .whole
        .wrapping_sub(Botwoon::of(game, e).y_4_frames_ago());
    let angle = queries::angle_from_delta(x, y);
    if x != 0 || y != 0 {
        let instruction = if Botwoon::of(game, e).head_hidden() != 0 {
            BotwoonEdit::of(game, e).actor().layer = HIDDEN_LAYER;
            BotwoonEdit::of(game, e).actor().properties |= INTANGIBLE;
            HIDE
        } else {
            BotwoonEdit::of(game, e).actor().layer = SHOWN_LAYER;
            BotwoonEdit::of(game, e).actor().properties &= !INTANGIBLE;
            // The original faces the raw angle; snesrev smoothed it.
            crate::actor_tables::word(game, |t| &t.botwoon_head_lists, (angle >> 5) as isize)
        };
        if instruction != Botwoon::of(game, e).instruction_list() {
            set_instruction_list(e, instruction, game);
        }
    }
    *BotwoonEdit::of(game, e).x_4_frames_ago() = Botwoon::of(game, e).x_3_frames_ago();
    *BotwoonEdit::of(game, e).y_4_frames_ago() = Botwoon::of(game, e).y_3_frames_ago();
    *BotwoonEdit::of(game, e).x_3_frames_ago() = Botwoon::of(game, e).x_2_frames_ago();
    *BotwoonEdit::of(game, e).y_3_frames_ago() = Botwoon::of(game, e).y_2_frames_ago();
    *BotwoonEdit::of(game, e).x_2_frames_ago() = Botwoon::of(game, e).x_1_frame_ago();
    *BotwoonEdit::of(game, e).y_2_frames_ago() = Botwoon::of(game, e).y_1_frame_ago();
    *BotwoonEdit::of(game, e).x_1_frame_ago() = Botwoon::of(game, e).actor().x.whole;
    *BotwoonEdit::of(game, e).y_1_frame_ago() = Botwoon::of(game, e).actor().y.whole;
}
/// Botwoon head function - spitting - set angle and show ($B3:9E7D): aim at
/// Samus, then spit 5 or 3; returns that head function. Was `aim_spit`.
pub fn head_set_spit_angle<L: Lease + ?Sized>(game: &mut L) -> u32 {
    let e = game.current_enemy();
    BotwoonEdit::of(game, e).actor().layer = SHOWN_LAYER;
    let actor = game.current_enemy();
    let angle = game.player_angle(actor);
    *BotwoonEdit::of(game, e).spit_angle() = angle;
    let instruction = crate::actor_tables::word(
        game,
        |t| &t.botwoon_spit_lists,
        ((angle as u8).wrapping_add(16) >> 5) as isize,
    );
    set_instruction_list(e, instruction, game);
    *BotwoonEdit::of(game, e).spit_angle() =
        u16::from(QUARTER_TURN.wrapping_sub(Botwoon::of(game, e).spit_angle() as u8));
    *BotwoonEdit::of(game, e).head_function() = head_function::SPITTING_SPAWN_5;
    if Botwoon::of(game, e).function() != function::SPITTING {
        *BotwoonEdit::of(game, e).head_function() = head_function::SPITTING_SPAWN_3;
    }
    BANK << 16 | u32::from(Botwoon::of(game, e).head_function())
}
/// Botwoon head function - spitting - spawn 5 ($B3:9EE0, `wide`, once the
/// spitting flag is set) or 3 ($B3:9F34) spit projectiles. Was `spit`.
pub fn head_spit<L: Lease + ?Sized>(wide: bool, game: &mut L) {
    let e = game.current_enemy();
    if wide && Botwoon::of(game, e).spitting() == 0 {
        return;
    }
    game.session_mut().enemy_projectiles.spawn.angle = (Botwoon::of(game, e).spit_angle() as u8)
        .wrapping_sub(if wide { 2 * SPIT_SPREAD } else { SPIT_SPREAD });
    let speed = SPIT_SPEEDS[Botwoon::of(game, e).speed_table_index() as usize];
    for _ in 0..if wide { 5 } else { 3 } {
        let k = game.current_enemy();
        game.spawn_projectile(speed, k, ADDR_K_EPROJ_BOTWOONS_SPIT as u16);
        let spawn = &mut game.session_mut().enemy_projectiles.spawn;
        spawn.angle = spawn.angle.wrapping_add(SPIT_SPREAD);
    }
    let target = if wide {
        *BotwoonEdit::of(game, e).spitting() = 0;
        e
    } else {
        game.current_enemy()
    };
    *BotwoonEdit::of(game, target).head_function() = head_function::SPITTING_COOLDOWN;
}
/// Botwoon head function - spitting - cooldown ($B3:9F7A). Was `wait_spit`.
pub fn head_cooldown<L: Lease + ?Sized>(k: u16, game: &mut L) {
    let e = k;
    let delay = (Botwoon::of(game, e).spit_timer() as i16).wrapping_sub(1);
    *BotwoonEdit::of(game, e).spit_timer() = delay as u16;
    if delay < 0 {
        {
            let mut botwoon = BotwoonEdit::of(game, e);
            *botwoon.spit_timer() = 0;
            *botwoon.head_function() = head_function::MOVING_AROUND;
        }
    }
}
/// Botwoon / hole collision detection ($B3:9F93): entering a hole toggles the
/// head between hidden and shown. Was `detect_portal`.
pub fn hole_collision<L: Lease + ?Sized>(k: u16, game: &mut L) {
    let e = game.current_enemy();
    if Botwoon::of(game, k).hole_collision_disabled() != 0 {
        return;
    }
    for index in [12, 8, 4, 0] {
        let hole = game.foreign(|game| game.botwoon_hole(index));
        let bounds = [hole.left, hole.right, hole.top, hole.bottom];
        if (0..4).all(|offset| {
            let coordinate = if offset < 2 {
                Botwoon::of(game, e).actor().x.whole
            } else {
                Botwoon::of(game, e).actor().y.whole
            };
            ((coordinate as i16).wrapping_sub(bounds[offset] as i16) >= 0) == (offset % 2 == 0)
        }) {
            *BotwoonEdit::of(game, e).hole_collision_disabled() = 1;
            *BotwoonEdit::of(game, e).head_hidden() ^= 1;
            *BotwoonEdit::of(game, e).target_history_index() =
                Botwoon::of(game, e).position_history_index();
            {
                let mut botwoon = BotwoonEdit::of(game, e);
                let old = botwoon.unread_counter();
                *old = old.wrapping_sub(1);
            }
            return;
        }
        *BotwoonEdit::of(game, e).hole_collision_disabled() = 0;
    }
}
/// Instruction - enemy radius = `x` x `y` ($B3:94C7, $B3:94D7, $B3:94E7).
pub fn set_hitbox<L: Lease + ?Sized>(x: u16, y: u16, j: EnemyScript, game: &mut L) -> EnemyScript {
    let e = game.current_enemy();
    {
        let mut botwoon = BotwoonEdit::of(game, e);
        let actor = botwoon.actor();
        actor.radius.x = x;
        actor.radius.y = y;
    }
    j
}
/// Instruction - set Botwoon spitting flag ($B3:9567).
pub fn set_spitting<L: Lease + ?Sized>(j: EnemyScript, game: &mut L) -> EnemyScript {
    let e = game.current_enemy();
    *BotwoonEdit::of(game, e).spitting() = 1;
    j
}

/// Initialisation AI - Botwoon ($B3:9583): once dead, clear its wall.
pub fn initialize<L: Lease + ?Sized>(game: &mut L) {
    let e = game.current_enemy();
    if u16::from(game.foreign(|game| game.boss_byte(MARIDIA))) & BOTWOON_DEAD != 0 {
        game.foreign(|game| {
            game.spawn_plm(WALL_BLOCK.0, WALL_BLOCK.1, CLEAR_WALL);
            game.open_passage();
        });
        BotwoonEdit::of(game, e).actor().instruction = HIDE;
        BotwoonEdit::of(game, e).actor().properties |= K_ENEMY_PROPS_DELETED as u16;
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
        function::GO_THROUGH_HOLE | function::MOVING_AROUND => {
            let game = &mut *runtime;
            let e = game.current_enemy();
            let changed = if state == function::GO_THROUGH_HOLE {
                leave_hole(e, game)
            } else {
                moving_around(e, game)
            };
            if !changed {
                step_body(e, game);
                if state == function::GO_THROUGH_HOLE {
                    hole_collision(game.current_enemy(), game);
                }
            }
        }
        function::SPITTING => {
            let e = runtime.selected_actor().byte_index();
            if spitting(e, runtime) {
                let address = (BANK << 16) | u32::from(Botwoon::of(runtime, e).head_function());
                pre_instruction(address, runtime.current_enemy(), runtime);
            }
        }
        function::DEATH_PRE_DEATH_DELAY => pre_death_delay(runtime),
        function::DEATH_FALLING_TO_GROUND => fall_to_ground(runtime),
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
                {
                    let mut botwoon = BotwoonEdit::of(runtime, k);
                    *botwoon.death_counter() = 0;
                    *botwoon.large_wall_explosion_timer() = 0;
                    *botwoon.wall_smoke_timer() = 0;
                }
            }
        }
        function::DEATH_CRUMBLING_WALL => crumbling_wall(k, runtime),
        _ => {
            runtime.foreign(|game| game.unreachable());
        }
    }
    health_palette(runtime);
}
/// Run the movement function, then the head's, carrying the body along the
/// position history.
fn step_body<L: Lease + ?Sized>(e: u16, game: &mut L) {
    match Botwoon::of(game, e).movement_function() {
        movement_function::MOVE_TOWARD_TARGET_HOLE => move_toward_target_hole(game),
        movement_function::MOVE_BY_MOVEMENT_DATA => move_by_movement_data(game),
        movement_function::START_MOVING_BY_MOVEMENT_DATA => start_moving_by_movement_data(game),
        _ => {
            game.foreign(|game| game.unreachable());
        }
    }
    game.foreign(|game| {
        game.record_botwoon_head();
        game.follow_botwoon_history();
    });
    let address = (BANK << 16) | u32::from(Botwoon::of(game, e).head_function());
    pre_instruction(address, game.current_enemy(), game);
    game.foreign(|game| {
        game.orient_botwoon_body();
        game.advance_botwoon_history();
    });
}

/// Run the head callback, including the immediate spawn after aiming.
pub fn pre_instruction<L: Lease + ?Sized>(address: u32, k: u16, game: &mut L) {
    match address {
        11763835 => (),
        11767039 => crate::actors::gamet::settle(game),
        11771328 => head_moving_around(game),
        11771517 => {
            let address = head_set_spit_angle(game);
            pre_instruction(address, game.current_enemy(), game);
        }
        11771616 => head_spit(true, game),
        11771700 => head_spit(false, game),
        11771770 => head_cooldown(k, game),
        _ => game.foreign(|game| game.unreachable()),
    }
}
