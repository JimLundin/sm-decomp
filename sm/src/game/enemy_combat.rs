//! Standard enemy combat. Slot handles survive effects; live selectors do not.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::game::combat_rules::{
    self as rules, GrappleOutcome, HealthResponse, LatchMode, Shot, ShotResponse,
};
use crate::game::constants::bank_86::addr_kEproj_EnemyDeathExplosion;
use crate::game::constants::bank_a0::addr_kEnemyDef_DAFF;
use crate::game::constants::bank_b4::addr_stru_B4EC1C;
use crate::game::enemy_ram::{native_write as write, EnemyField as F, EnemySlot};
use crate::game::sm_80::{QueueSfx1_Max3, QueueSfx2_Max1, QueueSfx2_Max3, QueueSfx3_Max3};
use crate::game::sm_86::SpawnEprojWithGfx;
use crate::game::sm_90::CallSomeSamusCode;
use crate::game::sm_91::Samus_DealDamage;
use crate::game::sprite_objects::create_sprite;
use crate::game_state::Game;
use crate::ram_map::checked_word as word;
use crate::rom::RomRecord as Record;
use crate::rtl::rom_record;
use crate::types::{EnemyData, EnemyDef, EnemySpawnData};
use core::mem::offset_of;

fn current(game: &Game) -> EnemySlot {
    EnemySlot::new(word!(game.cur_enemy_index))
}
impl EnemySlot {
    fn freeze(self, game: &mut Game) {
        self.set(
            F::Frozen,
            rules::freeze_duration(word!(game.area_index)),
            game,
        );
        self.set(F::AiHandlers, self.word(F::AiHandlers, game) | 4, game);
        self.set(F::Invincibility, 10, game);
    }
    unsafe fn definition(self, game: &mut Game) -> Record {
        unsafe { rom_record(0xa0, self.word(F::Definition, game), game) }
    }
}
impl Record {
    fn vulnerability(&self) -> u16 {
        let value = self.word(offset_of!(EnemyDef, vulnerability_ptr));
        if value == 0 {
            addr_stru_B4EC1C as u16
        } else {
            value
        }
    }
}
unsafe fn vulnerability(address: u16, offset: u16, field: usize, game: &mut Game) -> u8 {
    unsafe { rom_record(0xb4, address.wrapping_add(offset), game).byte(field) }
}
fn cause(value: u16, game: &mut Game) {
    write(
        game,
        0x7000
            + usize::from(word!(game.cur_enemy_index))
            + offset_of!(EnemySpawnData, cause_of_death),
        value,
    );
}
pub fn reduce_suit_damage(amount: u16, game: &mut Game) -> u16 {
    rules::suit_damage(amount, word!(game.equipped_items))
}
pub fn select_frozen_ai(game: &mut Game) {
    current(game).set(F::AiHandlers, 4, game);
}

unsafe fn death(animation: u16, limit: u16, enemy: EnemySlot, game: &mut Game) {
    let animation = if animation.wrapping_sub(limit) & 0x8000 == 0 {
        0
    } else {
        animation
    };
    unsafe {
        SpawnEprojWithGfx(
            animation,
            word!(game.cur_enemy_index),
            addr_kEproj_EnemyDeathExplosion as u16,
            game,
        );
    }
    let preserve = enemy.word(F::Properties, game) & 0x4000 != 0;
    enemy.clear(game);
    if preserve {
        enemy.set(F::Definition, addr_kEnemyDef_DAFF as u16, game);
        enemy.set_bank(0xa3, game);
    }
}
pub unsafe fn enemy_death(_k: u16, animation: u16, game: &mut Game) {
    let enemy = current(game);
    if enemy.word(F::AiHandlers, game) == 1 {
        word!(game.grapple_beam_function = 0xc8c5);
    }
    unsafe {
        death(animation, 5, enemy, game);
    }
    word!(game.num_enemies_killed_in_room += 1);
}
pub unsafe fn rinka_death(animation: u16, game: &mut Game) {
    unsafe {
        death(animation, 3, current(game), game);
    }
}
pub unsafe fn grapple_death(game: &mut Game) {
    cause(4, game);
    unsafe {
        enemy_death(word!(game.cur_enemy_index), 0, game);
    }
    current(game).set(F::AiHandlers, 0, game);
}

pub unsafe fn apply_contact(game: &mut Game) {
    unsafe {
        contact_without_death(game);
    }
    if current(game).word(F::Health, game) == 0 {
        cause(6, game);
        unsafe {
            enemy_death(word!(game.cur_enemy_index), 1, game);
        }
    }
}
pub unsafe fn contact_without_death(game: &mut Game) {
    let enemy = current(game);
    let definition = unsafe { enemy.definition(game) };
    let mode = word!(game.samus_contact_damage_index);
    if mode == 0 {
        unsafe {
            Samus_DealDamage(
                reduce_suit_damage(definition.word(offset_of!(EnemyDef, damage)), game),
                game,
            );
        }
        word!(game.samus_invincibility_timer = 96);
        word!(game.samus_knockback_timer = 5);
        word!(
            game.knockback_x_dir =
                (word!(game.samus_x_pos).wrapping_sub(enemy.word(F::X, game)) as i16 >= 0) as u16
        );
        return;
    }
    let (offset, base_damage) = rules::contact_attack(mode);
    if mode == 4 {
        unsafe {
            CallSomeSamusCode(4, game);
        }
    }
    let address = definition.vulnerability().wrapping_add(offset);
    let power = unsafe { rom_record(0xb4, address, game).word(0) };
    word!(game.last_enemy_power = power);
    let damage = (base_damage >> 1).wrapping_mul(power & 0x7f);
    if damage != 0 {
        enemy.set(
            F::Flash,
            rules::hurt_duration(definition.byte(offset_of!(EnemyDef, hurt_ai_time)), 0),
            game,
        );
        enemy.set(F::AiHandlers, enemy.word(F::AiHandlers, game) | 2, game);
        word!(game.samus_invincibility_timer = 0);
        word!(game.samus_knockback_timer = 0);
        enemy.set(
            F::Health,
            rules::contact_health(enemy.word(F::Health, game), damage),
            game,
        );
        unsafe {
            QueueSfx2_Max1(0xb, game);
        }
    }
}

pub unsafe fn power_bomb_without_death(game: &mut Game) {
    let definition = unsafe { current(game).definition(game) };
    let damage =
        rules::power_bomb_damage(unsafe { vulnerability(definition.vulnerability(), 0, 15, game) });
    if damage == 0 {
        return;
    }
    current(game).set(F::Invincibility, 48, game);
    let definition = unsafe { current(game).definition(game) };
    let enemy = current(game);
    enemy.set(
        F::Flash,
        rules::hurt_duration(definition.byte(offset_of!(EnemyDef, hurt_ai_time)), 8),
        game,
    );
    enemy.set(F::AiHandlers, enemy.word(F::AiHandlers, game) | 2, game);
    enemy.set(
        F::Health,
        enemy.word(F::Health, game).saturating_sub(damage),
        game,
    );
}
pub unsafe fn apply_power_bomb(game: &mut Game) {
    unsafe {
        power_bomb_without_death(game);
    }
    if current(game).word(F::Health, game) == 0 {
        cause(3, game);
        unsafe {
            enemy_death(word!(game.cur_enemy_index), 0, game);
        }
    }
}

pub unsafe fn shot_reaction(game: &mut Game) {
    unsafe {
        shot_without_death(game);
    }
}
pub unsafe fn shot_without_death(game: &mut Game) -> u16 {
    let index = word!(game.collision_detection_index);
    let shot = Shot {
        damage: word!(game.projectile_damage[index]),
        kind: word!(game.projectile_type[index]),
    };
    let definition = unsafe { current(game).definition(game) };
    let address = definition.vulnerability();
    let primary = if let Some((offset, field)) = shot.vulnerability_offset() {
        unsafe { vulnerability(address, offset, field, game) }
    } else {
        0
    };
    if shot.beam() {
        word!(game.last_enemy_power = u16::from(primary));
    }
    let charged = if shot.charged(primary) {
        unsafe { vulnerability(address, 0, 19, game) }
    } else {
        0
    };
    let damage = match shot.response(primary, charged) {
        ShotResponse::Dud => {
            let index = word!(game.collision_detection_index);
            word!(game.projectile_dir[index] = word!(game.projectile_dir[index]) | 0x10);
            unsafe {
                create_sprite(
                    word!(game.projectile_x_pos[index]),
                    word!(game.projectile_y_pos[index]),
                    6,
                    0,
                    game,
                );
                QueueSfx1_Max3(0x3d, game);
            }
            return 0;
        }
        ShotResponse::Freeze => {
            let enemy = current(game);
            if enemy.word(F::Frozen, game) == 0 {
                unsafe {
                    QueueSfx3_Max3(0xa, game);
                }
            }
            enemy.freeze(game);
            return 0;
        }
        ShotResponse::Damage(damage) => damage,
    };
    let definition = unsafe { current(game).definition(game) };
    let enemy = current(game);
    enemy.set(
        F::Flash,
        rules::hurt_duration(definition.byte(offset_of!(EnemyDef, hurt_ai_time)), 8),
        game,
    );
    enemy.set(F::AiHandlers, enemy.word(F::AiHandlers, game) | 2, game);
    let flash = enemy.word(F::Frozen, game) == 0;
    if flash {
        let sound = unsafe { enemy.definition(game) }.word(offset_of!(EnemyDef, hurt_sfx));
        if sound != 0 {
            unsafe {
                QueueSfx2_Max3(sound, game);
            }
        }
    }
    let enemy = current(game);
    let projectile = word!(game.projectile_type[word!(game.collision_detection_index)]);
    if projectile & 8 != 0 {
        enemy.set(F::Invincibility, 16, game);
    }
    match rules::shot_health(
        enemy.word(F::Health, game),
        damage,
        word!(game.projectile_type[word!(game.collision_detection_index)]),
        word!(game.last_enemy_power),
        enemy.word(F::Frozen, game),
    ) {
        HealthResponse::Health(value) => enemy.set(F::Health, value, game),
        HealthResponse::Freeze => {
            enemy.freeze(game);
            unsafe {
                QueueSfx3_Max3(0xa, game);
            }
        }
    }
    u16::from(flash)
}
pub unsafe fn apply_shot(game: &mut Game) {
    let flash = unsafe { shot_without_death(game) };
    let enemy = current(game);
    if flash != 0 {
        unsafe {
            create_sprite(enemy.word(F::X, game), enemy.word(F::Y, game), 55, 0, game);
        }
    }
    if enemy.word(F::Health, game) == 0 {
        let projectile = word!(game.projectile_type[word!(game.collision_detection_index)]);
        let kind = (projectile >> 8) & 15;
        cause(kind, game);
        let animation = unsafe { enemy.definition(game) }.word(offset_of!(EnemyDef, death_anim));
        let animation = if kind == 2 && animation.wrapping_sub(3) & 0x8000 != 0 {
            2
        } else {
            animation
        };
        unsafe {
            enemy_death(enemy.word(F::Definition, game), animation, game);
        }
    }
}
pub unsafe fn flash_on_shot(game: &mut Game) {
    if unsafe { shot_without_death(game) } != 0 {
        let enemy = current(game);
        unsafe {
            create_sprite(enemy.word(F::X, game), enemy.word(F::Y, game), 55, 0, game);
        }
    }
}
pub unsafe fn create_dud(game: &mut Game) {
    let index = word!(game.collision_detection_index);
    unsafe {
        create_sprite(
            word!(game.projectile_x_pos[index]),
            word!(game.projectile_y_pos[index]),
            6,
            0,
            game,
        );
        QueueSfx1_Max3(0x3d, game);
    }
    let index = word!(game.collision_detection_index);
    word!(game.projectile_dir[index] = word!(game.projectile_dir[index]) | 0x10);
}

#[forbid(unsafe_code)]
pub fn resume_ai(game: &mut Game) {
    let enemy = current(game);
    for field in [F::AiHandlers, F::Invincibility, F::Frozen, F::Shake] {
        enemy.set(field, 0, game);
    }
}

#[forbid(unsafe_code)]
fn snap_grapple(enemy: EnemySlot, game: &mut Game) {
    word!(game.grapple_beam_end_x_pos = enemy.word(F::X, game));
    word!(game.grapple_beam_end_y_pos = enemy.word(F::Y, game));
}

pub unsafe fn latch_enemy(mode: LatchMode, game: &mut Game) {
    let enemy = current(game);
    if mode == LatchMode::Passive {
        snap_grapple(enemy, game);
    }
    if enemy.word(F::Frozen, game) != 0 {
        enemy.set(F::AiHandlers, 4, game);
    } else {
        // Active latching captures the bank before the definition diagnostic.
        let bank = enemy.bank(game);
        let definition = unsafe { enemy.definition(game) };
        match mode {
            LatchMode::Passive => enemy.set(
                F::Flash,
                rules::hurt_duration(definition.byte(offset_of!(EnemyDef, hurt_ai_time)), 0),
                game,
            ),
            LatchMode::Active => unsafe {
                crate::game::sm_a0::CallEnemyAi(
                    (u32::from(bank) << 16)
                        | u32::from(definition.word(offset_of!(EnemyDef, main_ai))),
                    game,
                );
            },
        }
        enemy.set(F::AiHandlers, 0, game);
    }
    if mode == LatchMode::Active {
        snap_grapple(enemy, game);
    }
}

pub unsafe fn resolve_grapple(reaction: u16, definition: u16, game: &mut Game) -> GrappleOutcome {
    match reaction {
        0 | 2 => GrappleOutcome::Miss,
        3 => GrappleOutcome::Cancel,
        1 | 4 | 5 => GrappleOutcome::Attach,
        6 => {
            let definition = unsafe { rom_record(0xa0, definition, game) };
            let damage = reduce_suit_damage(definition.word(offset_of!(EnemyDef, damage)), game);
            unsafe {
                Samus_DealDamage(damage, game);
            }
            word!(game.samus_invincibility_timer = 96);
            word!(game.samus_knockback_timer = 5);
            word!(game.knockback_x_dir = u16::from(game.samus_pose_x_dir() == 4));
            GrappleOutcome::Cancel
        }
        _ => {
            unsafe {
                crate::rtl::Unreachable(game);
            }
            GrappleOutcome::Attach
        }
    }
}
