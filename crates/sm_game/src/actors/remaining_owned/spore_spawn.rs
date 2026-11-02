//! Spore Spawn, Brinstar's spore-spewing plant boss (bank $A5:E75F-$A5:EC48):
//! its descent, orbit and return, reactions, palettes and instructions.
//!
//! The swing instructions lived with Draygon as `increase_grab_speed`,
//! `clear_grab_rotation`, `grab_timers`, `place_grab_start` and `set_state`.
#![forbid(unsafe_code)]
use crate::actors::boss_interaction as interaction;
use crate::actors::host::{Death, Shot, below};
use crate::actors::layouts::{EnemyBase, same_words};
use crate::actors::remaining_owned::Core;
use crate::actors::session_slots::session_slot;
use crate::enemy_script::EnemyScript;
use crate::menus::prelude::battle;

/// What Spore Spawn needs beyond the shared boss effects.
pub trait Host: Core {
    fn run_spore_return(&mut self, phase: SporeReturn, key: u16);
    /// Native prefix stores at $19B1-2*word, including odd aliases. It is the
    /// explicit enemy-projectile definition interaction, never an arbitrary RAM write port.
    fn clear_spore_fx_word(&mut self, word: u16);

    /// Move the stem's segments between its anchor and Spore Spawn.
    fn update_spore_stem(&mut self);
    /// The boss bits of `area`.
    /// Scatter Spore Spawn's death drops for enemy `k`.
    fn scatter_spore_spawn_drop(&mut self, k: u16);
}

/// A lease on a game with Spore Spawn's effects.
pub trait Lease: Core + Host + crate::actors::boss_interaction::Access {}
impl<T: Core + Host + crate::actors::boss_interaction::Access> Lease for T {}

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
pub(crate) const MIDDLE_X: u16 = 0x80;
pub(crate) const MIDDLE_Y: u16 = 0x270;
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
pub fn increase_max_x_radius<L: Lease>(script: EnemyScript, game: &mut L) -> EnemyScript {
    if below(session_slot!(game, Spore[0].max_x_radius), RADIUS_LIMIT) {
        session_slot!(game, Spore[0].max_x_radius, |old| old
            .wrapping_add(RADIUS_STEP));
    }
    script
}

/// Instruction - clear Spore Spawn damaged flag ($A5:E771).
pub fn clear_damaged_flag<L: Lease>(script: EnemyScript, game: &mut L) -> EnemyScript {
    session_slot!(game, Spore[0].damaged = 0);
    script
}

/// Instruction - max X radius = [[Y]], angle delta = [[Y] + 2] ($A5:E82D).
pub fn set_max_x_radius_and_angle_delta<L: Lease>(
    script: EnemyScript,
    game: &mut L,
) -> EnemyScript {
    session_slot!(game, Spore[0]; max_x_radius = script.word(0), angle_delta = script.word(2));
    script.offset(2)
}

/// Instruction - Spore Spawn harden ($A5:E87C).
pub fn harden<L: Lease>(script: EnemyScript, game: &mut L) -> EnemyScript {
    session_slot!(game, Spore[0].base; x_pos = MIDDLE_X, y_pos = MIDDLE_Y,
        properties = (session_slot!(game, Spore[0].base.properties) & HARD_KEEP) | HARD_SET);
    script
}

/// Instruction - enemy function = [[Y]] ($A5:E8BA).
pub fn set_function<L: Lease>(script: EnemyScript, game: &mut L) -> EnemyScript {
    let k = game.current_enemy() as i32;
    session_slot!(game, Spore[k].function = script.word(0));
    script.offset(1)
}

/// Initialisation AI ($A5:EA2A): its palettes and stem; dead, it hangs closed
/// off behind the ceiling, otherwise it rises for the intro.
pub fn initialize<L: Lease>(game: &mut L) {
    copy_palette(game, true, 240, 0xa5_e359, 0);
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
    let area = crate::actors::owned_common_lifecycle::BossState::area(game);
    if { crate::progress::boss_byte_at(game, area) } & DEAD as u8 != 0 {
        session_slot!(game, Spore[0].base.current_instruction = list::DEAD);
        session_slot!(game, Spore[0].function = function::NOTHING);
        session_slot!(game, Spore[0].base.properties, |old| old | 0x8000);
        game.update_spore_stem();
        game.remaining_spawn_plm(7, 30, CLOSE_CEILING);
        battle!(game.ScrollingFinishedHook = 0);
    } else {
        session_slot!(game, Spore[0].base.current_instruction = list::FIGHT_INTRO);
        battle!(game.FlagProcessAllEnemies = 0xffff);
        let spawn_key = game.current_enemy();
        session_slot!(game, Spore[k].function = function::NOTHING);
        battle!(game.ScrollingFinishedHook = 0x9589);
        session_slot!(game, Spore[k].base.y_pos, |old| old.wrapping_sub(128));
        for part in 0..4 {
            game.spawn_actor_eproj(part, spawn_key, SPORE_SPAWNERS);
        }
        game.update_spore_stem();
    }
}

/// Run one of Spore Spawn's script instructions, keyed by address, for actor `k`.
pub fn run_instruction<L: Lease>(
    address: u32,
    k: u16,
    script: EnemyScript,
    game: &mut L,
) -> EnemyScript {
    match address {
        instruction::INCREASE_MAX_X_RADIUS => increase_max_x_radius(script, game),
        instruction::CLEAR_DAMAGED_FLAG => clear_damaged_flag(script, game),
        instruction::SET_MAX_X_RADIUS_AND_ANGLE_DELTA => {
            set_max_x_radius_and_angle_delta(script, game)
        }
        instruction::HARDEN => harden(script, game),
        instruction::SET_FUNCTION => set_function(script, game),
        // Instruction - spawn Spore Spawn's death drops ($A5:E8B1).
        10873009 => {
            game.scatter_spore_spawn_drop(k);
            script
        }
        10873034 | 10873116 => fade(address == 10873116, script, game),
        10873198 => emit_spore(script, game),
        10873265 => emit_hurt(script, game),
        _ => unreachable!("unrecognized Spore Spawn instruction"),
    }
}

/// Main AI ($A5:EB13): its function.
pub fn main_ai<L: Lease>(game: &mut L) {
    let k = game.current_enemy();
    match session_slot!(game, Spore[k as i32].function) {
        function::NOTHING => {}
        function::DESCEND => game.run_spore_return(SporeReturn::Descend, k),
        function::ORBIT => game.run_spore_return(SporeReturn::Orbit, k),
        function::BEGIN_RETURN => game.run_spore_return(SporeReturn::BeginReturn, k),
        function::FINISH_RETURN => game.run_spore_return(SporeReturn::FinishReturn, k),
        _ => game.remaining_unknown(),
    }
}

/// After contact or a shot: once dead, start the death sequence, mark it dead
/// and open the ceiling.
pub fn after_damage<L: Lease>(game: &mut L) {
    let k = game.current_enemy() as i32;
    if session_slot!(game, Spore[k].base.health) != 0 {
        return;
    }
    // The original reselects after its health query, before clearing actor timers.
    let k = game.current_enemy() as i32;
    session_slot!(game, Spore[0].unread_781c = 0);
    session_slot!(game, Spore[k].base; invincibility_timer = 0, flash_timer = 0, ai_handler_bits = 0);
    session_slot!(game, Spore[k].base.properties, |old| old | 0x400);
    for word in 0..14 {
        game.clear_spore_fx_word(word);
    }
    session_slot!(game, Spore[0].base; current_instruction = list::DEATH, instruction_timer = 1);
    let area = crate::actors::owned_common_lifecycle::BossState::area(game);
    {
        let bits =
            crate::progress::boss_word_at(game, area, crate::progress::BossWordOrder::Little);
        crate::progress::set_boss_word_le(game, area, bits | DEAD);
    }
    battle!(game.ScrollingFinishedHook = 0);
    game.remaining_spawn_plm(7, 30, OPEN_CEILING);
}

/// Touch AI ($A5:EDEC).
pub fn touch<L: Lease>(game: &mut L) {
    game.react_to_contact(Death::LeavePending);
    after_damage(game);
}

/// Shot AI ($A5:ED5A): only charged shots and missiles hurt it; the first hit
/// of a swing reverses it, and its palette darkens with health.
pub fn shot<L: Lease>(game: &mut L) {
    let _projectile = battle!(game.CollisionDetectionIndex).wrapping_mul(2);
    let kind = interaction::get_at(
        game,
        interaction::Array::ProjectileType,
        battle!(game.CollisionDetectionIndex),
    );
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
                interaction::set_at(game, interaction::Array::PaletteBuffer, index, colour);
                cursor = cursor.wrapping_add(2);
            }
        }
    }
    after_damage(game);
}

/// Instruction - fade the palettes, or with `target` the target palettes, to
/// step [[Y]] ($A5:E8CA, $A5:E91C).
pub fn fade<L: Lease>(target: bool, script: EnemyScript, game: &mut L) -> EnemyScript {
    let cursor = script.word(0) >> 1;
    copy_palette(game, target, 144, 0xa5e3f9, cursor);
    copy_palette(game, target, 64, 0xa5e4f9, cursor);
    copy_palette(game, target, 112, 0xa5e5d9, cursor);
    script.offset(1)
}

/// Instruction - spawn a spore within 40h px of Spore Spawn ($A5:E96E).
pub fn emit_spore<L: Lease>(script: EnemyScript, game: &mut L) -> EnemyScript {
    let random = game.roll_random();
    let x = session_slot!(game, Spore[0].base.x_pos).wrapping_add((random & 0x7f).wrapping_sub(64));
    let y = session_slot!(game, Spore[0].base.y_pos)
        .wrapping_add(((random >> 8) & 0x7f).wrapping_sub(64));
    game.place_projectile_spawn(x, y);
    game.spawn_room_eproj(SPORES, 0x15);
    crate::effects::SoundEffects::queue_sound(game, 1, 0x29, 6);
    script
}

/// Instruction - smoke within 40h px of Spore Spawn as it is hurt ($A5:E9B1).
pub fn emit_hurt<L: Lease>(script: EnemyScript, game: &mut L) -> EnemyScript {
    let random = game.roll_random();
    let x = session_slot!(game, Spore[0].base.x_pos)
        .wrapping_add(random & 0x7f)
        .wrapping_sub(64);
    let y = session_slot!(game, Spore[0].base.y_pos)
        .wrapping_add((random >> 8) & 0x3f)
        .wrapping_sub(32);
    game.remaining_create_sprite(x, y, 3, 0);
    crate::effects::SoundEffects::queue_sound(game, 1, 0x25, 6);
    script
}

fn copy_palette(
    host: &mut impl Lease,
    target: bool,
    destination: usize,
    address: u32,
    cursor: u16,
) {
    // Capture once per original transfer, then independent native reads and
    // two ordered little-endian prefix stores, preserving failure progress.
    let image = crate::actors::content_records::ContentSource::content_image(host);
    let base = usize::try_from(sm_data::cartridge::lorom_offset(address)).expect("palette source")
        + 2 * usize::from(cursor);
    for offset in 0..16 {
        let at = base + 2 * offset;
        let color = u16::from_ne_bytes([image.read(at), image.read(at + 1)]);
        if target {
            host.session_mut().presentation.visual_mut().targets[destination + offset] = color;
        } else {
            host.session_mut().presentation.visual_mut().colors[destination + offset] = color;
        }
    }
}
#[derive(Clone, Copy)]
pub enum SporeReturn {
    Descend,
    Orbit,
    BeginReturn,
    FinishReturn,
}
impl Host for crate::actors::owned_execution::PreparedActorState<'_> {
    fn update_spore_stem(&mut self) {
        crate::enemy_projectiles::attachments_spore_maw::update_spore_stem(self);
    }
    fn scatter_spore_spawn_drop(&mut self, key: u16) {
        crate::actors::deaths::Deaths::scatter_enemy_drop(
            self,
            crate::enemy_projectiles::drops::DropSource::SporeSpawn,
            key,
        );
    }
    fn clear_spore_fx_word(&mut self, word: u16) {
        // The original native transfer writes each byte as an ordered prefix.
        let at = 0x19b1usize - 2 * usize::from(word);
        self.set_current_byte(at, 0);
        self.set_current_byte(at + 1, 0);
    }
    fn run_spore_return(&mut self, phase: SporeReturn, key: u16) {
        use crate::actors::spore_spawn::owned_return;
        match phase {
            SporeReturn::Descend => owned_return::descend(self),
            SporeReturn::Orbit => owned_return::orbit(key, self),
            SporeReturn::BeginReturn => owned_return::begin_return(self),
            SporeReturn::FinishReturn => owned_return::finish_return(self),
        }
    }
}

pub fn descend<H: Lease>(host: &mut H) {
    host.run_spore_return(SporeReturn::Descend, host.current_enemy());
}
pub fn orbit<H: Lease>(key: u16, host: &mut H) {
    host.run_spore_return(SporeReturn::Orbit, key);
}
pub fn begin_return<H: Lease>(host: &mut H) {
    host.run_spore_return(SporeReturn::BeginReturn, host.current_enemy());
}
pub fn finish_return<H: Lease>(host: &mut H) {
    host.run_spore_return(SporeReturn::FinishReturn, host.current_enemy());
}
