//! Enemy-projectile command identities, ordered state changes and captured operands.
#![forbid(unsafe_code)]
use super::access::{enemy, enemy_byte};
use super::addresses;
use super::host::{EnemyMember as F, Host, NativeRecord};
use crate::actors::access::{AccessMode::LittleEndian as Le, ActorKey as Key};
use crate::constants::{bank_86::*, bank_a0::*};
use crate::enemy_script::EnemyScript;
use crate::geometry::Point;
use crate::projectile_geometry::SINE_COSINE as kSinCosTable8bit_Sext;
use crate::{
    sprite_effect_state::{LegacySpriteDefinition, SpriteSpawn},
    sprite_objects,
};

fn reset_position(h: &mut impl Host, slot: isize) {
    enemy!(h, slot, x = enemy!(h, slot, behavior_a));
    enemy!(h, slot, y = enemy!(h, slot, behavior_b));
}
fn drop_at_projectile(h: &mut impl Host, k: u16, definition: u16) {
    let slot = (k >> 1) as isize;
    h.session_mut().enemy_projectiles.spawn.point = Point {
        x: enemy!(h, slot, x),
        y: enemy!(h, slot, y),
    };
    super::drops::spawn(definition, k, h);
}
fn random_sprite(h: &mut impl Host, k: u16, script: &EnemyScript, mask: u16, half: u16) {
    let slot = (k >> 1) as isize;
    let x = enemy!(h, slot, x)
        .wrapping_add(h.next_random() & mask)
        .wrapping_sub(half);
    let y = enemy!(h, slot, y)
        .wrapping_add((h.inputs().random & (mask << 8)) >> 8)
        .wrapping_sub(half);
    sprite_objects::spawn(
        h,
        SpriteSpawn {
            position: Point { x, y },
            definition: LegacySpriteDefinition(script.word(0)),
            appearance: 0,
        },
    );
}
fn random_move(h: &mut impl Host, slot: isize, script: &EnemyScript) {
    let signs = h.next_random();
    let x = loop {
        let random = h.next_random() as i8;
        let amount =
            ((i32::from(script.byte(0)) & i32::from(random)) - i32::from(script.byte(1))) as i8;
        if amount >= 0 {
            break amount as u8;
        }
    };
    let amount = if signs & 0x8000 != 0 {
        -i32::from(x)
    } else {
        i32::from(x)
    };
    enemy!(h, slot, x = (i32::from(enemy!(h, slot, x)) + amount) as u16);
    let y = loop {
        let random = h.next_random() as u8;
        let amount =
            ((i32::from(script.byte(2)) & i32::from(random)) - i32::from(script.byte(3))) as u8;
        if amount & 0x80 == 0 {
            break amount;
        }
    };
    let amount = if signs & 0x4000 != 0 {
        -i16::from(y)
    } else {
        i16::from(y)
    };
    enemy!(h, slot, y = enemy!(h, slot, y).wrapping_add(amount as u16));
}
fn set_velocity_toward(h: &mut impl Host, k: u16, angle: u16) {
    let slot = (k >> 1) as isize;
    enemy!(
        h,
        slot,
        velocity_x = (4 * i32::from(kSinCosTable8bit_Sext[usize::from(angle) + 64])) as u16
    );
    enemy!(
        h,
        slot,
        velocity_y = (4 * i32::from(kSinCosTable8bit_Sext[usize::from(angle)])) as u16
    );
}
pub fn dispatch(ea: u32, k: u16, script: &EnemyScript, h: &mut impl Host) -> EnemyScript {
    use addresses::instructions::*;
    let slot = (k >> 1) as isize;
    let result = match ea {
        DELETE => {
            enemy!(h, slot, definition = 0);
            EnemyScript::Stop
        }
        SLEEP => {
            enemy!(
                h,
                slot,
                program = script.bank_address(0x28000).wrapping_sub(2)
            );
            EnemyScript::Stop
        }
        SET_PRE_INSTRUCTION | SET_AND_RUN_PRE_INSTRUCTION => {
            enemy!(h, slot, pre_action = script.word(0));
            script.advance(if ea == SET_PRE_INSTRUCTION { 2 } else { 0 })
        }
        CLEAR_PRE_INSTRUCTION => {
            enemy!(h, slot, pre_action = 0x8170);
            script.advance(0)
        }
        MOVE_UP_4 => {
            enemy!(h, slot, y = enemy!(h, slot, y).wrapping_sub(4));
            script.advance(0)
        }
        CALL_EXTERNAL_FUNCTION => {
            // This operand reads three bytes; Load24 also reads the fourth byte.
            match u32::from(script.word(0)) | (u32::from(script.byte(2)) << 16) {
                AT_C7FB => super::shared_pre::SpawnMotherBrainDeathBeam(k, h),
                _ => h.unreachable(),
            }
            script.advance(3)
        }
        SET_TIMER => {
            enemy!(h, slot, counter = script.word(0));
            script.advance(2)
        }
        DRAYGON_GOOP_SAMUS_COLLISION => {
            super::collision::power_bomb(k, h);
            let divisor = h.session().player.speed_divisor.wrapping_add(1);
            if (i32::from(h.session().player.speed_divisor) - 5) & 0x8000 != 0 {
                h.session_mut().player.speed_divisor =
                    h.session().player.speed_divisor.wrapping_add(1);
                enemy!(h, slot, behavior_b = divisor);
                enemy!(h, slot, behavior_a = 256);
                enemy!(h, slot, pre_action = 0x8dca);
                enemy!(
                    h,
                    slot,
                    properties = enemy!(h, slot, properties) & 0x5fff | 0x2000
                );
                h.session_mut().player.invincibility = 0;
                h.session_mut().player.knockback_timer = 0;
            }
            script.advance(0)
        }
        SPAWN_CROCOMIRE_DROPS
        | SPAWN_PHANTOON_DROP
        | AT_DFEA
        | SPAWN_DRAYGON_EYE_DROPS
        | AT_D1CE
        | AT_DC61 => {
            let definition = match ea {
                SPAWN_CROCOMIRE_DROPS => addr_kEnemyDef_DDBF,
                SPAWN_PHANTOON_DROP => addr_kEnemyDef_E4FF,
                AT_DFEA => addr_kEnemyDef_E83F,
                SPAWN_DRAYGON_EYE_DROPS => addr_kEnemyDef_DE7F,
                AT_D1CE => addr_kEnemyDef_E7FF,
                _ => addr_kEnemyDef_DF7F,
            } as u16;
            drop_at_projectile(h, k, definition);
            script.advance(0)
        }
        RESET_POSITION | AT_AF36 | AT_B436 => {
            reset_position(h, slot);
            script.advance(0)
        }
        SPAWN_AREA_DROP => {
            h.session_mut().enemy_projectiles.spawn.point = Point {
                x: enemy!(h, slot, x),
                y: enemy!(h, slot, y),
            };
            let definition = script.word(if h.session().room.area != 0 { 2 } else { 0 });
            super::drops::spawn(definition, k, h);
            script.advance(4)
        }
        GO_TO_BY_DIRECTION => EnemyScript::jump(if enemy!(h, slot, behavior_a) & 0x8000 != 0 {
            addr_off_86B166 as u16
        } else {
            addr_off_86B14B as u16
        }),
        START_EARTHQUAKE => {
            h.session_mut().feedback.earthquake_kind = 1;
            h.session_mut().feedback.earthquake_timer =
                h.session().feedback.earthquake_timer | 0x20;
            script.advance(0)
        }
        MOVE_Y_BY => {
            let old = enemy!(h, slot, y);
            enemy!(
                h,
                slot,
                y = (i32::from(old) + i32::from(script.word(0))) as u16
            );
            script.advance(2)
        }
        MOVE_DOWN_12 => {
            enemy!(h, slot, y = enemy!(h, slot, y).wrapping_add(12));
            script.advance(0)
        }
        SET_Y_VELOCITY => {
            enemy!(h, slot, velocity_y = script.word(0));
            script.advance(2)
        }
        DECREMENT_TIMER_AND_GO_TO => {
            let old = enemy!(h, slot, counter);
            enemy!(h, slot, counter = old.wrapping_sub(1));
            if old == 1 {
                script.advance(2)
            } else {
                EnemyScript::jump(script.word(0))
            }
        }
        DECREMENT_TIMER_AND_GO_TO_RELATIVE => {
            let old = enemy!(h, slot, counter);
            enemy!(h, slot, counter = old.wrapping_sub(1));
            if old == 1 {
                script.advance(1)
            } else {
                script.advance(script.byte(0) as i8 as isize)
            }
        }
        MOVE_RANDOMLY => {
            random_move(h, slot, script);
            script.advance(4)
        }
        SET_PROPERTIES | CLEAR_PROPERTIES => {
            let old = enemy!(h, slot, properties);
            let mask = script.word(0);
            enemy!(
                h,
                slot,
                properties = if ea == SET_PROPERTIES {
                    old | mask
                } else {
                    old & mask
                }
            );
            script.advance(2)
        }
        ENABLE_SAMUS_PROJECTILE_COLLISION
        | DISABLE_SAMUS_PROJECTILE_COLLISION
        | DISABLE_SAMUS_COLLISION
        | ENABLE_SAMUS_COLLISION
        | SET_NOT_DIE_ON_CONTACT
        | SET_DIE_ON_CONTACT
        | SET_HIGH_PRIORITY
        | SET_LOW_PRIORITY
        | AT_9475 => {
            let (mask, set) = match ea {
                ENABLE_SAMUS_PROJECTILE_COLLISION => (0x8000, true),
                DISABLE_SAMUS_PROJECTILE_COLLISION => (0x8000, false),
                DISABLE_SAMUS_COLLISION | AT_9475 => (0x2000, true),
                ENABLE_SAMUS_COLLISION => (0x2000, false),
                SET_NOT_DIE_ON_CONTACT => (0x4000, true),
                SET_DIE_ON_CONTACT => (0x4000, false),
                SET_HIGH_PRIORITY => (0x1000, true),
                _ => (0x1000, false),
            };
            let old = enemy!(h, slot, properties);
            enemy!(
                h,
                slot,
                properties = if set { old | mask } else { old & !mask }
            );
            script.advance(0)
        }
        SET_RADII => {
            enemy!(h, slot, radius = script.word(0));
            script.advance(2)
        }
        CLEAR_RADII => {
            enemy!(h, slot, radius = 0);
            script.advance(0)
        }
        AIM_AT_SAMUS => {
            let x = h.session().player.x.wrapping_sub(enemy!(h, slot, x));
            let y = h.session().player.y.wrapping_sub(enemy!(h, slot, y));
            let angle = h.calculate_angle_from_xy(x, y).wrapping_mul(2);
            let selected = (h.session().enemy_projectiles.selected >> 1) as isize;
            enemy!(h, selected, behavior_a = angle);
            let index = usize::from(angle >> 1);
            enemy!(
                h,
                selected,
                velocity_x = kSinCosTable8bit_Sext[index + 64] as u16
            );
            enemy!(
                h,
                selected,
                velocity_y = kSinCosTable8bit_Sext[index] as u16
            );
            script.advance(0)
        }
        WRITE_COLOURS => {
            let mut color = script.word(2);
            let mut remaining = i32::from(script.byte(4));
            let mut source = script.word(0);
            loop {
                let value = h.native_record(0x86, source).word_le(0);
                h.set_palette_buffer((color >> 1) as isize, value);
                source = source.wrapping_add(2);
                color = color.wrapping_add(2);
                remaining -= 1;
                if remaining & 0x8000 != 0 {
                    break;
                }
            }
            script.advance(5)
        }
        QUEUE_MUSIC => {
            h.queue_music(u16::from(script.byte(0)));
            script.advance(1)
        }
        QUEUE_SOUND_FIRST..=QUEUE_SOUND_LAST if (ea - QUEUE_SOUND_FIRST).is_multiple_of(9) => {
            let command = (ea - QUEUE_SOUND_FIRST) / 9;
            h.queue_sound(
                (command % 3 + 1) as u8,
                [6, 15, 3, 9, 1][(command / 3) as usize],
                u16::from(script.byte(0)),
            );
            script.advance(1)
        }
        GO_TO_WITH_QUARTER_CHANCE => {
            if h.next_random() & 0xc000 == 0xc000 {
                EnemyScript::jump(script.word(0))
            } else {
                script.advance(2)
            }
        }
        MOVE_X_AND_GO_TO_BY_DIRECTION => {
            let amount = (i32::from(enemy!(h, slot, velocity_x) as i16) << 8) as u32;
            super::pre_helpers::add_split(F::X, F::XFraction, slot, amount, h);
            EnemyScript::jump(script.word(if enemy!(h, slot, velocity_x) & 0x8000 == 0 {
                2
            } else {
                0
            }))
        }
        AT_B7EA | AT_B818 => {
            let definition = if ea == AT_B7EA {
                addr_kEproj_TourianStatueUnlockingParticle
            } else {
                addr_kEproj_TourianStatueUnlockingParticleTail
            } as u16;
            super::lifecycle::spawn_room(h, definition, k);
            script.advance(0)
        }
        STOP_MOTHER_BRAIN_PURPLE_BREATH => {
            h.set_mother_brain_purple_breath_active(0);
            script.advance(0)
        }
        AT_D5E1 | AT_D69A => {
            h.next_random();
            enemy!(
                h,
                slot,
                velocity_x =
                    (h.inputs().random >> 8) | (h.session().flag_compatibility.bitmask << 8)
            );
            if ea == AT_D5E1 {
                enemy!(h, slot, velocity_y = 192);
            }
            script.advance(0)
        }
        ARM_DRAYGON_WALL_TURRET | AT_8CFD => {
            enemy!(
                h,
                slot,
                pre_action = if ea == ARM_DRAYGON_WALL_TURRET {
                    0x8dff
                } else {
                    0x8dca
                }
            );
            script.advance(0)
        }
        SPAWN_HORIZONTAL_AFTERBURN | SPAWN_VERTICAL_AFTERBURN => {
            let definitions = if ea == SPAWN_HORIZONTAL_AFTERBURN {
                [addr_stru_86966C, addr_stru_86967A]
            } else {
                [addr_stru_869688, addr_stru_869696]
            };
            for definition in definitions {
                h.session_mut().enemy_projectiles.spawn.secondary = 0;
                super::lifecycle::spawn_room(h, definition as u16, k);
            }
            script.advance(0)
        }
        SPAWN_NEXT_AFTERBURN => {
            let count = enemy_byte!(h, k as isize, behavior_a).wrapping_sub(1);
            enemy_byte!(h, k as isize, behavior_a = count);
            if count as i8 >= 0 {
                let handler = enemy!(h, slot, behavior_b);
                super::lifecycle::spawn_room(h, handler, k);
            }
            script.advance(0)
        }
        AT_B269 | AT_B272 => {
            let angle = super::pre_helpers::angle_to_player(k, h);
            set_velocity_toward(
                h,
                k,
                if ea == AT_B269 {
                    angle & 0x7f
                } else {
                    angle | 0x80
                },
            );
            script.advance(0)
        }
        GO_TO_IF_KRAID_HEALTHY => {
            if h.kraid_health_eighth(enemy!(h, slot, behavior_b)) & 0x8000 == 0 {
                EnemyScript::jump(script.word(0))
            } else {
                script.advance(2)
            }
        }
        GO_TO_INDEXED => EnemyScript::jump(script.word(enemy!(h, slot, behavior_a) as isize)),
        AT_C1B4 | AT_D1C7 => {
            enemy!(h, slot, graphics = 0);
            script.advance(0)
        }
        KAGO_BUG_HOP => {
            super::pre_actions::kago_bug_cry_countdown(k, h);
            super::pre_actions::kago_bug_flag_when_far(k, h);
            let mut direction = (h.inputs().random & 0x300).wrapping_add(2048);
            enemy!(h, slot, velocity_y = direction.wrapping_neg());
            let enemy = enemy!(h, slot, behavior_b);
            let distance = h
                .resolve_actor(Key::from_legacy(enemy), Le)
                .x
                .whole
                .wrapping_sub(enemy!(h, slot, x));
            let absolute = if distance & 0x8000 != 0 {
                distance.wrapping_neg()
            } else {
                distance
            };
            if (absolute as i16).wrapping_sub(0x30) >= 0 {
                direction = if distance & 0x8000 != 0 { 0xffff } else { 0 };
            }
            enemy!(
                h,
                slot,
                velocity_x = if direction & 0x100 != 0 {
                    (-0x200i16) as u16
                } else {
                    0x200
                }
            );
            enemy!(h, slot, pre_action = 0xd0ec);
            script.advance(0)
        }
        KAGO_BUG_REST => {
            enemy!(
                h,
                slot,
                behavior_c = (h.inputs().random & 0x1f).wrapping_add(1)
            );
            enemy!(h, slot, pre_action = 0xd0ca);
            script.advance(0)
        }
        ALTERNATE_X_BY_FRAME => {
            let odd = h.inputs().frame & 1 != 0;
            enemy!(
                h,
                slot,
                x = if odd {
                    enemy!(h, slot, behavior_b)
                } else {
                    (256 - i32::from(enemy!(h, slot, behavior_b))) as u16
                }
            );
            enemy!(h, slot, spritemap = script.word(if odd { 0 } else { 2 }));
            enemy!(
                h,
                slot,
                program = script.bank_address(0x28000).wrapping_add(4)
            );
            enemy!(h, slot, delay = 1);
            EnemyScript::Stop
        }
        ALTERNATE_X_OFFSCREEN_BY_FRAME => {
            enemy!(
                h,
                slot,
                x = if h.inputs().frame & 1 != 0 {
                    enemy!(h, slot, behavior_b)
                } else {
                    (-4608i16) as u16
                }
            );
            enemy!(h, slot, spritemap = script.word(0));
            enemy!(
                h,
                slot,
                program = script.bank_address(0x28000).wrapping_add(2)
            );
            enemy!(h, slot, delay = 1);
            EnemyScript::Stop
        }
        SET_PROPERTIES_3000 => {
            enemy!(h, slot, properties = 12288);
            script.advance(0)
        }
        SPAWN_AT_POSITION => {
            h.session_mut().enemy_projectiles.spawn.point = Point {
                x: enemy!(h, slot, x),
                y: enemy!(h, slot, y),
            };
            let parameter = h.session().enemy_projectiles.spawn.point.y;
            super::lifecycle::spawn_room(h, addr_kEproj_Spores as u16, parameter);
            script.advance(0)
        }
        AT_ECE3 | AT_ED17 => {
            let (mask, half) = if ea == AT_ECE3 {
                (0x3f, 32)
            } else {
                (0x1f, 16)
            };
            random_sprite(h, k, script, mask, half);
            script.advance(2)
        }
        AT_EE8B | AT_EE97 | AT_EEA3 => {
            h.queue_sound(
                2,
                1,
                match ea {
                    AT_EE8B => 9,
                    AT_EE97 => 0x24,
                    _ => 0xb,
                },
            );
            script.advance(0)
        }
        GO_TO_CONVERTED_DROP => EnemyScript::jump(super::drops::convert(k, h)),
        RESPAWN_DROP => {
            super::drops::respawn(k, h);
            script.advance(0)
        }
        GO_TO => EnemyScript::jump(script.word(0)),
        GO_TO_RELATIVE => script.advance(script.byte(0) as i8 as isize),
        PIRATE_LASER_MOVE_RIGHT => {
            super::pre_actions::EprojPreInstr_PirateMotherBrainLaser_MoveRight(k, h);
            script.advance(0)
        }
        PIRATE_LASER_MOVE_LEFT => {
            super::pre_actions::EprojPreInstr_PirateMotherBrainLaser_MoveLeft(k, h);
            script.advance(0)
        }
        NOP => script.advance(0),
        _ => {
            h.unreachable();
            EnemyScript::Stop
        }
    };
    h.probe("eproj.command.executed");
    result
}
