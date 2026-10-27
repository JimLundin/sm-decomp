//! Enemy-projectile command identities, ordered state changes and captured operands.
#![forbid(unsafe_code)]
use super::access::{enemy, enemy_byte};
use super::host::{EnemyMember as F, Host, NativeRecord};
use crate::game::actors::access::{AccessMode::LittleEndian as Le, ActorKey as Key};
use crate::game::constants::{bank_86::*, bank_a0::*};
use crate::game::enemy_script::EnemyScript;
use crate::game::projectile_geometry::SINE_COSINE as kSinCosTable8bit_Sext;
use crate::game::session::Point;
use crate::game::{
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
pub(super) fn dispatch(ea: u32, k: u16, script: &EnemyScript, h: &mut impl Host) -> EnemyScript {
    let slot = (k >> 1) as isize;
    let result = match ea {
        8814932 => {
            enemy!(h, slot, definition = 0);
            EnemyScript::Stop
        }
        8814937 => {
            enemy!(
                h,
                slot,
                program = script.bank_address(0x28000).wrapping_sub(2)
            );
            EnemyScript::Stop
        }
        8814945 | 8822864 => {
            enemy!(h, slot, pre_action = script.word(0));
            script.advance(if ea == 8814945 { 2 } else { 0 })
        }
        8814954 => {
            enemy!(h, slot, pre_action = 0x8170);
            script.advance(0)
        }
        8826770 => {
            enemy!(h, slot, y = enemy!(h, slot, y).wrapping_sub(4));
            script.advance(0)
        }
        8814961 => {
            // This operand reads three bytes; Load24 also reads the fourth byte.
            match u32::from(script.word(0)) | (u32::from(script.byte(2)) << 16) {
                8833019 => super::shared_pre::SpawnMotherBrainDeathBeam(k, h),
                _ => h.unreachable(),
            }
            script.advance(3)
        }
        8815061 => {
            enemy!(h, slot, counter = script.word(0));
            script.advance(2)
        }
        8818073 => {
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
        8819312 | 8820750 | 8839146 | 8817768 | 8835534 | 8838241 => {
            let definition = match ea {
                8819312 => addr_kEnemyDef_DDBF,
                8820750 => addr_kEnemyDef_E4FF,
                8839146 => addr_kEnemyDef_E83F,
                8817768 => addr_kEnemyDef_DE7F,
                8835534 => addr_kEnemyDef_E7FF,
                _ => addr_kEnemyDef_DF7F,
            } as u16;
            drop_at_projectile(h, k, definition);
            script.advance(0)
        }
        8823742 | 8826678 | 8827958 => {
            reset_position(h, slot);
            script.advance(0)
        }
        8825738 => {
            h.session_mut().enemy_projectiles.spawn.point = Point {
                x: enemy!(h, slot, x),
                y: enemy!(h, slot, y),
            };
            let definition = script.word(if h.session().room.area != 0 { 2 } else { 0 });
            super::drops::spawn(definition, k, h);
            script.advance(4)
        }
        8827198 => EnemyScript::jump(if enemy!(h, slot, behavior_a) & 0x8000 != 0 {
            addr_off_86B166 as u16
        } else {
            addr_off_86B14B as u16
        }),
        8828917 => {
            h.session_mut().feedback.earthquake_kind = 1;
            h.session_mut().feedback.earthquake_timer =
                h.session().feedback.earthquake_timer | 0x20;
            script.advance(0)
        }
        8828993 => {
            let old = enemy!(h, slot, y);
            enemy!(
                h,
                slot,
                y = (i32::from(old) + i32::from(script.word(0))) as u16
            );
            script.advance(2)
        }
        8833232 => {
            enemy!(h, slot, y = enemy!(h, slot, y).wrapping_add(12));
            script.advance(0)
        }
        8840499 => {
            enemy!(h, slot, velocity_y = script.word(0));
            script.advance(2)
        }
        8815046 => {
            let old = enemy!(h, slot, counter);
            enemy!(h, slot, counter = old.wrapping_sub(1));
            if old == 1 {
                script.advance(2)
            } else {
                EnemyScript::jump(script.word(0))
            }
        }
        8815054 => {
            let old = enemy!(h, slot, counter);
            enemy!(h, slot, counter = old.wrapping_sub(1));
            if old == 1 {
                script.advance(1)
            } else {
                script.advance(script.byte(0) as i8 as isize)
            }
        }
        8815071 => {
            random_move(h, slot, script);
            script.advance(4)
        }
        8815152 | 8815164 => {
            let old = enemy!(h, slot, properties);
            let mask = script.word(0);
            enemy!(
                h,
                slot,
                properties = if ea == 8815152 {
                    old | mask
                } else {
                    old & mask
                }
            );
            script.advance(2)
        }
        8815176 | 8815186 | 8815196 | 8815206 | 8815216 | 8815226 | 8815236 | 8815246 | 8819829 => {
            let (mask, set) = match ea {
                8815176 => (0x8000, true),
                8815186 => (0x8000, false),
                8815196 | 8819829 => (0x2000, true),
                8815206 => (0x2000, false),
                8815216 => (0x4000, true),
                8815226 => (0x4000, false),
                8815236 => (0x1000, true),
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
        8815256 => {
            enemy!(h, slot, radius = script.word(0));
            script.advance(2)
        }
        8815265 => {
            enemy!(h, slot, radius = 0);
            script.advance(0)
        }
        8815269 => {
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
        8815317 => {
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
        8815357 => {
            h.queue_music(u16::from(script.byte(0)));
            script.advance(1)
        }
        8815369..=8815495 if (ea - 8815369) % 9 == 0 => {
            let command = (ea - 8815369) / 9;
            h.queue_sound(
                (command % 3 + 1) as u8,
                [6, 15, 3, 9, 1][(command / 3) as usize],
                u16::from(script.byte(0)),
            );
            script.advance(1)
        }
        8823894 => {
            if h.next_random() & 0xc000 == 0xc000 {
                EnemyScript::jump(script.word(0))
            } else {
                script.advance(2)
            }
        }
        8826258 => {
            let amount = (i32::from(enemy!(h, slot, velocity_x) as i16) << 8) as u32;
            super::pre_helpers::add_split(F::X, F::XFraction, slot, amount, h);
            EnemyScript::jump(script.word(if enemy!(h, slot, velocity_x) & 0x8000 == 0 {
                2
            } else {
                0
            }))
        }
        8828906 | 8828952 => {
            let definition = if ea == 8828906 {
                addr_kEproj_TourianStatueUnlockingParticle
            } else {
                addr_kEproj_TourianStatueUnlockingParticleTail
            } as u16;
            super::lifecycle::spawn_room(h, definition, k);
            script.advance(0)
        }
        8833774 => {
            h.set_mother_brain_purple_breath_active(0);
            script.advance(0)
        }
        8836577 | 8836762 => {
            h.next_random();
            enemy!(
                h,
                slot,
                velocity_x =
                    (h.inputs().random >> 8) | (h.session().flag_compatibility.bitmask << 8)
            );
            if ea == 8836577 {
                enemy!(h, slot, velocity_y = 192);
            }
            script.advance(0)
        }
        8817910 | 8817917 => {
            enemy!(
                h,
                slot,
                pre_action = if ea == 8817910 { 0x8dff } else { 0x8dca }
            );
            script.advance(0)
        }
        8820154 | 8820205 => {
            let definitions = if ea == 8820154 {
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
        8820256 => {
            let count = enemy_byte!(h, k as isize, behavior_a).wrapping_sub(1);
            enemy_byte!(h, k as isize, behavior_a = count);
            if count as i8 >= 0 {
                let handler = enemy!(h, slot, behavior_b);
                super::lifecycle::spawn_room(h, handler, k);
            }
            script.advance(0)
        }
        8827497 | 8827506 => {
            let angle = super::pre_helpers::angle_to_player(k, h);
            set_velocity_toward(
                h,
                k,
                if ea == 8827497 {
                    angle & 0x7f
                } else {
                    angle | 0x80
                },
            );
            script.advance(0)
        }
        8827832 => {
            if h.kraid_health_eighth(enemy!(h, slot, behavior_b)) & 0x8000 == 0 {
                EnemyScript::jump(script.word(0))
            } else {
                script.advance(2)
            }
        }
        8831347 => EnemyScript::jump(script.word(enemy!(h, slot, behavior_a) as isize)),
        8831412 | 8835527 => {
            enemy!(h, slot, graphics = 0);
            script.advance(0)
        }
        8835420 => {
            super::pre_actions::EprojPreInstr_KagosBugs_Func1(k, h);
            super::pre_actions::EprojPreInstr_KagosBugs_Func2(k, h);
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
        8835510 => {
            enemy!(
                h,
                slot,
                behavior_c = (h.inputs().random & 0x1f).wrapping_add(1)
            );
            enemy!(h, slot, pre_action = 0xd0ca);
            script.advance(0)
        }
        8836594 => {
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
        8836650 => {
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
        8838234 => {
            enemy!(h, slot, properties = 12288);
            script.advance(0)
        }
        8838263 => {
            h.session_mut().enemy_projectiles.spawn.point = Point {
                x: enemy!(h, slot, x),
                y: enemy!(h, slot, y),
            };
            let parameter = h.session().enemy_projectiles.spawn.point.y;
            super::lifecycle::spawn_room(h, addr_kEproj_Spores as u16, parameter);
            script.advance(0)
        }
        8842467 | 8842519 => {
            let (mask, half) = if ea == 8842467 {
                (0x3f, 32)
            } else {
                (0x1f, 16)
            };
            random_sprite(h, k, script, mask, half);
            script.advance(2)
        }
        8842891 | 8842903 | 8842915 => {
            h.queue_sound(
                2,
                1,
                match ea {
                    8842891 => 9,
                    8842903 => 0x24,
                    _ => 0xb,
                },
            );
            script.advance(0)
        }
        8842927 => EnemyScript::jump(super::drops::convert(k, h)),
        8843024 => {
            super::drops::respawn(k, h);
            script.advance(0)
        }
        8815019 => EnemyScript::jump(script.word(0)),
        8815024 => script.advance(script.byte(0) as i8 as isize),
        8822906 => {
            super::pre_actions::EprojPreInstr_PirateMotherBrainLaser_MoveRight(k, h);
            script.advance(0)
        }
        8822876 => {
            super::pre_actions::EprojPreInstr_PirateMotherBrainLaser_MoveLeft(k, h);
            script.advance(0)
        }
        8815070 => script.advance(0),
        _ => {
            h.unreachable();
            EnemyScript::Stop
        }
    };
    h.probe("eproj.command.executed");
    result
}
