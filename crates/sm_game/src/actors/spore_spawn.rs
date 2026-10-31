//! Spore Spawn, Brinstar's spore-spewing plant boss (bank $A5:E75F-$A5:EC48):
//! its descent, orbit and return, reactions, palettes and instructions.
//!
//! The swing instructions lived with Draygon as `increase_grab_speed`,
//! `clear_grab_rotation`, `grab_timers`, `place_grab_start` and `set_state`.
#![forbid(unsafe_code)]
use crate::actors::boss_lease::BossLease;
use crate::actors::host::{BossEffects, Death, Shot, abs16, below};
use crate::actors::layouts::{EnemyBase, same_words};
use crate::actors::queries::angle_from_delta;
use crate::actors::session_slots::session_slot;
use crate::enemy_script::EnemyScript;
use crate::geometry::Vector32;
use crate::ram::{word, word_arrays};

/// What Spore Spawn needs beyond the shared boss effects.
pub trait Host: BossEffects + BossLease<Game = Self> {
    /// Move the stem's segments between its anchor and Spore Spawn.
    fn update_spore_stem(&mut self);
    /// The boss bits of `area`.
    fn boss_bits(&mut self, area: u16) -> u8;
    /// Scatter Spore Spawn's death drops for enemy `k`.
    fn scatter_spore_spawn_drop(&mut self, k: u16);
}

/// A lease on a game with Spore Spawn's effects.
pub trait Lease: BossLease<Game: Host> {}
impl<T: BossLease<Game: Host> + ?Sized> Lease for T {}

/// Spore Spawn's view of its slot: its orbit's origin, the stem's anchor,
/// the swing, and its 16.16 speeds and angle back to the middle.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Spore {
    pub base: EnemyBase,
    pub function: u16,
    pad_0faa: [u16; 1],
    pub origin_x: u16,
    pub origin_y: u16,
    pad_0fb0: [u16; 1],
    /// Cleared at initialisation; nothing reads it.
    pub unread_0fb2: u16,
    pad_0fb4: [u8; 26708],
    pub stem_anchor_x: u16,
    pub stem_anchor_y: u16,
    pad_780c: [u16; 4],
    pub angle: u16,
    pub max_x_radius: u16,
    pub angle_delta: u16,
    pad_781a: [u16; 1],
    /// Cleared on death; nothing reads it.
    pub unread_781c: u16,
    pad_781e: [u8; 2034],
    pub return_x_speed: u16,
    pub return_x_subspeed: u16,
    pub return_y_speed: u16,
    pub return_y_subspeed: u16,
    pad_8018: [u16; 3],
    pub damaged: u16,
    pad_8020: [u8; 2016],
    pub palette_health: u16,
    pad_8802: [u16; 2],
    pub return_angle: u16,
    pad_8808: [u16; 28],
}

same_words!(Spore: EnemyLayout104 {
    function = var_a, origin_x = var_c, origin_y = var_d, unread_0fb2 = var_f,
    stem_anchor_x = var_04, stem_anchor_y = var_05, angle = var_0a, max_x_radius = var_0b,
    angle_delta = var_0c, unread_781c = var_0e, return_x_speed = var_28,
    return_x_subspeed = var_29, return_y_speed = var_2a, return_y_subspeed = var_2b,
    damaged = var_2f, palette_health = var_40, return_angle = var_43,
});

/// Bank $A5 Spore Spawn functions, stored in `function`; $EB1A is an RTS.
pub mod function {
    pub const NOTHING: u16 = 0xeb1a;
    pub const DESCEND: u16 = 0xeb1b;
    pub const ORBIT: u16 = 0xeb52;
    pub const BEGIN_RETURN: u16 = 0xeb9b;
    pub const FINISH_RETURN: u16 = 0xebee;
}

/// Bank $A5 Spore Spawn instructions.
pub mod instruction {
    pub const INCREASE_MAX_X_RADIUS: u32 = 0xa5_e75f;
    pub const CLEAR_DAMAGED_FLAG: u32 = 0xa5_e771;
    pub const SET_MAX_X_RADIUS_AND_ANGLE_DELTA: u32 = 0xa5_e82d;
    pub const HARDEN: u32 = 0xa5_e87c;
    pub const SET_FUNCTION: u32 = 0xa5_e8ba;
}

/// Bank $A5 Spore Spawn instruction lists.
pub mod list {
    pub const DEAD: u16 = 0xe6b9;
    pub const FIGHT_INTRO: u16 = 0xe6c7;
    pub const DESCENDED: u16 = 0xe6d5;
    pub const HURT: u16 = 0xe729;
    pub const DEATH: u16 = 0xe77d;
}

/// The max X radius grows by 8 while below 30h; it is always 30h or 40h, so
/// this never takes effect.
const RADIUS_STEP: u16 = 8;
const RADIUS_LIMIT: u16 = 0x30 - 8;
/// Spore Spawn's resting spot, the middle of the room, within 8 px.
const MIDDLE_X: u16 = 0x80;
const MIDDLE_Y: u16 = 0x270;
/// Hardening puts it at the middle, solid to Samus, processing its
/// instructions and tangible.
const HARD_KEEP: u16 = 0x5bff;
const HARD_SET: u16 = 0xa000;
/// Enemy projectiles: the stem ($86:DE6C), the intro's spore spawners
/// ($86:DE88) and the spores ($86:E509).
const STEM: u16 = 0xde6c;
const SPORE_SPAWNERS: u16 = 0xde88;
const SPORES: u16 = 0xe509;
/// The PLMs that close ($B7:93) and open ($B7:8F) the ceiling at (7, 1Eh).
const CLOSE_CEILING: u16 = 0xb793;
const OPEN_CEILING: u16 = 0xb78f;
/// Boss bit 1: Spore Spawn is dead.
const DEAD: u16 = 2;

/// Instruction - increase Spore Spawn max X radius ($A5:E75F).
pub fn increase_max_x_radius<L: Lease + ?Sized>(script: EnemyScript, game: &mut L) -> EnemyScript {
    if below(session_slot!(game, Spore[0].max_x_radius), RADIUS_LIMIT) {
        session_slot!(game, Spore[0].max_x_radius, |old| old
            .wrapping_add(RADIUS_STEP));
    }
    script
}

/// Instruction - clear Spore Spawn damaged flag ($A5:E771).
pub fn clear_damaged_flag<L: Lease + ?Sized>(script: EnemyScript, game: &mut L) -> EnemyScript {
    session_slot!(game, Spore[0].damaged = 0);
    script
}

/// Instruction - max X radius = [[Y]], angle delta = [[Y] + 2] ($A5:E82D).
pub fn set_max_x_radius_and_angle_delta<L: Lease + ?Sized>(
    script: EnemyScript,
    game: &mut L,
) -> EnemyScript {
    session_slot!(game, Spore[0]; max_x_radius = script.word(0), angle_delta = script.word(2));
    script.offset(2)
}

/// Instruction - Spore Spawn harden ($A5:E87C).
pub fn harden<L: Lease + ?Sized>(script: EnemyScript, game: &mut L) -> EnemyScript {
    session_slot!(game, Spore[0].base; x_pos = MIDDLE_X, y_pos = MIDDLE_Y,
        properties = (session_slot!(game, Spore[0].base.properties) & HARD_KEEP) | HARD_SET);
    script
}

/// Instruction - enemy function = [[Y]] ($A5:E8BA).
pub fn set_function<L: Lease + ?Sized>(script: EnemyScript, game: &mut L) -> EnemyScript {
    let k = game.current_enemy() as i32;
    session_slot!(game, Spore[k].function = script.word(0));
    script.offset(1)
}

/// Initialisation AI ($A5:EA2A): its palettes and stem; dead, it hangs closed
/// off behind the ceiling, otherwise it rises for the intro.
pub fn initialize<L: Lease + ?Sized>(game: &mut L) {
    crate::word_transfers!(game, ascending, 16; le(TARGET_PALETTES, 240) <- rom(0xa5e359, 0));
    for part in 0..4 {
        let k = game.current_enemy();
        game.spawn_actor_eproj(part, k, STEM);
    }
    let k = game.current_enemy() as i32;
    let (x, y) = (
        session_slot!(game, Spore[k].base.x_pos),
        session_slot!(game, Spore[k].base.y_pos),
    );
    session_slot!(game, Spore[k]; stem_anchor_x = x, stem_anchor_y = y.wrapping_sub(72), origin_x = x,
        origin_y = y, unread_0fb2 = 0);
    let area = game.area();
    if game.foreign(|game| game.boss_bits(area)) & DEAD as u8 != 0 {
        session_slot!(game, Spore[0].base.current_instruction = list::DEAD);
        session_slot!(game, Spore[0].function = function::NOTHING);
        session_slot!(game, Spore[0].base.properties, |old| old | 0x8000);
        game.foreign(|game| {
            game.update_spore_stem();
            game.spawn_plm(7, 30, CLOSE_CEILING);
        });
        word!(game.SCROLLING_FINISHED_HOOK = 0);
    } else {
        session_slot!(game, Spore[0].base.current_instruction = list::FIGHT_INTRO);
        word!(game.FLAG_PROCESS_ALL_ENEMIES = 0xffff);
        let spawn_key = game.current_enemy();
        session_slot!(game, Spore[k].function = function::NOTHING);
        word!(game.SCROLLING_FINISHED_HOOK = 0x9589);
        session_slot!(game, Spore[k].base.y_pos, |old| old.wrapping_sub(128));
        for part in 0..4 {
            game.spawn_actor_eproj(part, spawn_key, SPORE_SPAWNERS);
        }
        game.foreign(|game| game.update_spore_stem());
    }
}

/// Main AI ($A5:EB13): its function.
pub fn main_ai<L: Lease + ?Sized>(game: &mut L) {
    let k = game.current_enemy();
    match session_slot!(game, Spore[k as i32].function) {
        function::NOTHING => {}
        function::DESCEND => descend(game),
        function::ORBIT => orbit(k, game),
        function::BEGIN_RETURN => begin_return(game),
        function::FINISH_RETURN => finish_return(game),
        _ => game.foreign(|game| game.unreachable()),
    }
}

/// Function - descend ($A5:EB1B): sink a pixel a frame to the middle, then
/// start swinging.
pub fn descend<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| game.update_spore_stem());
    let k = game.current_enemy() as i32;
    let y = session_slot!(game, Spore[k].base.y_pos).wrapping_add(1);
    session_slot!(game, Spore[k].base.y_pos = y);
    if !below(y, MIDDLE_Y) {
        session_slot!(game, Spore[k].base; current_instruction = list::DESCENDED, instruction_timer = 1);
    }
    session_slot!(game, Spore[0]; max_x_radius = 48, angle_delta = 1, angle = 192);
}

/// Function - orbit ($A5:EB52): swing round the origin, half as high as wide,
/// by the angle delta's low byte a frame.
pub fn orbit<L: Lease + ?Sized>(k: u16, game: &mut L) {
    game.foreign(|game| game.update_spore_stem());
    let e = i32::from(k);
    let (angle, radius) = (
        session_slot!(game, Spore[0].angle),
        session_slot!(game, Spore[0].max_x_radius),
    );
    let x = session_slot!(game, Spore[e].origin_x).wrapping_add(game.scaled_cosine(angle, radius));
    session_slot!(game, Spore[e].base.x_pos = x);
    let phase = (2 * (i32::from(session_slot!(game, Spore[0].angle)) - 64)) as u16;
    let radius = session_slot!(game, Spore[0].max_x_radius).wrapping_sub(16);
    let y = session_slot!(game, Spore[e].origin_y).wrapping_add(game.scaled_sine(phase, radius));
    session_slot!(game, Spore[e].base.y_pos = y);
    let delta = session_slot!(byte game, Spore[0].angle_delta[0]);
    session_slot!(game, Spore[0].angle, |old| u16::from(
        delta.wrapping_add(old as u8)
    ));
}

/// Function - begin return ($A5:EB9B): aim at the middle at 1 px a frame.
pub fn begin_return<L: Lease + ?Sized>(game: &mut L) {
    let dx = MIDDLE_X.wrapping_sub(session_slot!(game, Spore[0].base.x_pos));
    let dy = MIDDLE_Y.wrapping_sub(session_slot!(game, Spore[0].base.y_pos));
    let angle = 64u8.wrapping_sub(angle_from_delta(dx, dy) as u8);
    session_slot!(game, Spore[0].return_angle = u16::from(angle));
    let k = game.current_enemy() as i32;
    let angle = u16::from(session_slot!(byte game, Spore[0].return_angle[0]));
    let v = game.direction_step(angle, 1);
    session_slot!(game, Spore[k]; return_x_speed = (v.x >> 16) as u16, return_x_subspeed = v.x as u16,
        return_y_speed = (v.y >> 16) as u16, return_y_subspeed = v.y as u16);
}

/// Function - finish return ($A5:EBEE): move back to the middle, then hang
/// still, bubbling.
pub fn finish_return<L: Lease + ?Sized>(game: &mut L) {
    let k = game.current_enemy() as i32;
    let phase = u16::from(session_slot!(byte game, Spore[0].return_angle[0]));
    let joined = |high: u16, low: u16| (u32::from(high) << 16) | u32::from(low);
    let x = joined(
        session_slot!(game, Spore[k].return_x_speed),
        session_slot!(game, Spore[k].return_x_subspeed),
    );
    let y = joined(
        session_slot!(game, Spore[k].return_y_speed),
        session_slot!(game, Spore[k].return_y_subspeed),
    );
    game.set_step_scratch(phase);
    game.step_along(phase, Vector32 { x, y });
    let near = |at: u16, middle: u16| below(abs16(at.wrapping_sub(middle)), 8);
    if near(session_slot!(game, Spore[0].base.x_pos), MIDDLE_X)
        && near(session_slot!(game, Spore[0].base.y_pos), MIDDLE_Y)
    {
        session_slot!(game, Spore[0].function = function::NOTHING);
    }
    game.foreign(|game| game.update_spore_stem());
    if game.frame() & 0xf == 0 {
        let random = game.roll_random();
        let (x, y) = ((random & 0x3f) + 96, ((random >> 8) & 0xf) + 480);
        game.foreign(|game| game.create_sprite(x, y, 21, 0));
    }
}

/// After contact or a shot: once dead, start the death sequence, mark it dead
/// and open the ceiling.
pub fn after_damage<L: Lease + ?Sized>(game: &mut L) {
    let k = game.current_enemy() as i32;
    if session_slot!(game, Spore[k].base.health) != 0 {
        return;
    }
    // The original reselects after its health query, before clearing actor timers.
    let k = game.current_enemy() as i32;
    session_slot!(game, Spore[0].unread_781c = 0);
    session_slot!(game, Spore[k].base; invincibility_timer = 0, flash_timer = 0, ai_handler_bits = 0);
    session_slot!(game, Spore[k].base.properties, |old| old | 0x400);
    crate::word_transfers!(game, descending, 14; native(0x19b1) <- value(0));
    session_slot!(game, Spore[0].base; current_instruction = list::DEATH, instruction_timer = 1);
    let area = game.area();
    game.foreign(|game| {
        let bits = game.boss_word(area);
        game.set_boss_word(area, bits | DEAD);
    });
    word!(game.SCROLLING_FINISHED_HOOK = 0);
    game.foreign(|game| game.spawn_plm(7, 30, OPEN_CEILING));
}

/// Touch AI ($A5:EDEC).
pub fn touch<L: Lease + ?Sized>(game: &mut L) {
    game.react_to_contact(Death::LeavePending);
    after_damage(game);
}

/// Shot AI ($A5:ED5A): only charged shots and missiles hurt it; the first hit
/// of a swing reverses it, and its palette darkens with health.
pub fn shot<L: Lease + ?Sized>(game: &mut L) {
    let _projectile = word!(game.COLLISION_DETECTION_INDEX).wrapping_mul(2);
    let kind = word!(game.PROJECTILE_TYPE[word!(game.COLLISION_DETECTION_INDEX)]);
    if kind & 0x700 == 0 && kind & 0x10 == 0 {
        return;
    }
    game.react_to_shot(Shot::Flash);
    let k = game.current_enemy() as i32;
    if session_slot!(game, Spore[k].base.flash_timer) == 0 {
        return after_damage(game);
    }
    session_slot!(game, Spore[k].function = function::ORBIT);
    if below(session_slot!(game, Spore[k].base.health), 400) {
        let reversed = session_slot!(game, Spore[0].angle_delta) & 0x8000 != 0;
        session_slot!(
            game,
            Spore[0].angle_delta = if reversed { 0xfffe } else { 2 }
        );
    }
    if session_slot!(game, Spore[0].damaged) == 0 {
        session_slot!(game, Spore[0].angle_delta, |old| 0u16.wrapping_sub(old));
        session_slot!(game, Spore[0].damaged = 1);
        session_slot!(game, Spore[k].base; current_instruction = list::HURT, instruction_timer = 1);
        let health = session_slot!(game, Spore[k].base.health);
        let darker = [70, 410, 770]
            .iter()
            .take_while(|&&threshold| !below(health, threshold))
            .count();
        if health != session_slot!(game, Spore[0].palette_health) {
            session_slot!(game, Spore[0].palette_health = health);
            let mut cursor = 96 - 32 * darker as u16;
            for index in 144..160 {
                let colour =
                    crate::gameplay_tables::spore_spawn_colour(game, (cursor >> 1) as isize);
                word!(game.PALETTE_BUFFER[index] = colour);
                cursor = cursor.wrapping_add(2);
            }
        }
    }
    after_damage(game);
}

/// Instruction - fade the palettes, or with `target` the target palettes, to
/// step [[Y]] ($A5:E8CA, $A5:E91C).
pub fn fade<L: Lease + ?Sized>(target: bool, script: EnemyScript, game: &mut L) -> EnemyScript {
    let cursor = script.word(0) >> 1;
    let field = [word_arrays::PALETTE_BUFFER, word_arrays::TARGET_PALETTES][usize::from(target)];
    crate::word_transfers!(game, ascending, 16; little(field + 144 * 2) <- rom(0xa5e3f9, cursor));
    crate::word_transfers!(game, ascending, 16; little(field + 64 * 2) <- rom(0xa5e4f9, cursor));
    crate::word_transfers!(game, ascending, 16; little(field + 112 * 2) <- rom(0xa5e5d9, cursor));
    script.offset(1)
}

/// Instruction - spawn a spore within 40h px of Spore Spawn ($A5:E96E).
pub fn emit_spore<L: Lease + ?Sized>(script: EnemyScript, game: &mut L) -> EnemyScript {
    let random = game.roll_random();
    let x = session_slot!(game, Spore[0].base.x_pos).wrapping_add((random & 0x7f).wrapping_sub(64));
    let y = session_slot!(game, Spore[0].base.y_pos)
        .wrapping_add(((random >> 8) & 0x7f).wrapping_sub(64));
    game.place_projectile_spawn(x, y);
    game.spawn_room_eproj(SPORES, 0x15);
    game.queue_sound(1, 0x29, 6);
    script
}

/// Instruction - smoke within 40h px of Spore Spawn as it is hurt ($A5:E9B1).
pub fn emit_hurt<L: Lease + ?Sized>(script: EnemyScript, game: &mut L) -> EnemyScript {
    let random = game.roll_random();
    let x = session_slot!(game, Spore[0].base.x_pos)
        .wrapping_add(random & 0x7f)
        .wrapping_sub(64);
    let y = session_slot!(game, Spore[0].base.y_pos)
        .wrapping_add((random >> 8) & 0x3f)
        .wrapping_sub(32);
    game.foreign(|game| game.create_sprite(x, y, 3, 0));
    game.queue_sound(1, 0x25, 6);
    script
}
