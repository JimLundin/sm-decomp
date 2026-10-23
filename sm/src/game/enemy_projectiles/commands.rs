//! Enemy-projectile command identities, ordered state changes and captured operands.
#![forbid(unsafe_code)]
use super::script::EprojScript;
use super::state::Point;
use super::{Execution, SlotField as F};
use crate::game::constants::{bank_86::*, bank_a0::*};
use crate::game::sm_94::kSinCosTable8bit_Sext;

fn reset_position(ctx: &mut Execution<'_>, slot: isize) {
    ctx.set_slot(F::X, slot, ctx.slot(F::BehaviorA, slot));
    ctx.set_slot(F::Y, slot, ctx.slot(F::BehaviorB, slot));
}
fn drop_at_projectile(ctx: &mut Execution<'_>, k: u16, definition: u16) {
    let slot = (k >> 1) as isize;
    ctx.state_mut().spawn.point = Point {
        x: ctx.slot(F::X, slot),
        y: ctx.slot(F::Y, slot),
    };
    super::drops::spawn(definition, k, 0, ctx);
}
fn random_sprite(ctx: &mut Execution<'_>, k: u16, script: &EprojScript, mask: u16, half: u16) {
    let slot = (k >> 1) as isize;
    let x = ctx
        .slot(F::X, slot)
        .wrapping_add(ctx.next_random() & mask)
        .wrapping_sub(half);
    let y = ctx
        .slot(F::Y, slot)
        .wrapping_add((ctx.random_number() & (mask << 8)) >> 8)
        .wrapping_sub(half);
    ctx.create_sprite(x, y, script.word(0), 0);
}
fn random_move(ctx: &mut Execution<'_>, slot: isize, script: &EprojScript) {
    let signs = ctx.next_random();
    let x = loop {
        let random = ctx.next_random() as i8;
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
    ctx.set_slot(
        F::X,
        slot,
        (i32::from(ctx.slot(F::X, slot)) + amount) as u16,
    );
    let y = loop {
        let random = ctx.next_random() as u8;
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
    ctx.set_slot(F::Y, slot, ctx.slot(F::Y, slot).wrapping_add(amount as u16));
}
fn set_velocity_toward(ctx: &mut Execution<'_>, k: u16, angle: u16) {
    let slot = (k >> 1) as isize;
    ctx.set_slot(
        F::VelocityX,
        slot,
        (4 * i32::from(kSinCosTable8bit_Sext[usize::from(angle) + 64])) as u16,
    );
    ctx.set_slot(
        F::VelocityY,
        slot,
        (4 * i32::from(kSinCosTable8bit_Sext[usize::from(angle)])) as u16,
    );
}
pub(super) fn dispatch(
    ea: u32,
    k: u16,
    script: &EprojScript,
    ctx: &mut Execution<'_>,
) -> Option<usize> {
    let slot = (k >> 1) as isize;
    let result = match ea {
        8814932 => {
            ctx.set_slot(F::Definition, slot, 0);
            None
        }
        8814937 => {
            ctx.set_slot(F::Program, slot, script.bank_address().wrapping_sub(2));
            None
        }
        8814945 | 8822864 => {
            ctx.set_slot(F::PreAction, slot, script.word(0));
            script.offset(if ea == 8814945 { 2 } else { 0 })
        }
        8814954 => {
            ctx.set_slot(F::PreAction, slot, 0x8170);
            script.offset(0)
        }
        8826770 => {
            ctx.set_slot(F::Y, slot, ctx.slot(F::Y, slot).wrapping_sub(4));
            script.offset(0)
        }
        8814961 => {
            match script.long() {
                8833019 => super::shared_pre::SpawnMotherBrainDeathBeam(k, ctx),
                _ => ctx.unreachable(),
            }
            script.offset(3)
        }
        8815061 => {
            ctx.set_slot(F::Counter, slot, script.word(0));
            script.offset(2)
        }
        8818073 => {
            super::collision::power_bomb(k, ctx);
            let divisor = ctx.player_speed_divisor().wrapping_add(1);
            if (i32::from(ctx.player_speed_divisor()) - 5) & 0x8000 != 0 {
                ctx.set_player_speed_divisor(ctx.player_speed_divisor().wrapping_add(1));
                ctx.set_slot(F::BehaviorB, slot, divisor);
                ctx.set_slot(F::BehaviorA, slot, 256);
                ctx.set_slot(F::PreAction, slot, 0x8dca);
                ctx.set_slot(
                    F::Properties,
                    slot,
                    ctx.slot(F::Properties, slot) & 0x5fff | 0x2000,
                );
                ctx.set_player_invincibility(0);
                ctx.set_player_knockback(0);
            }
            script.offset(0)
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
            drop_at_projectile(ctx, k, definition);
            script.offset(0)
        }
        8823742 | 8826678 | 8827958 => {
            reset_position(ctx, slot);
            script.offset(0)
        }
        8825738 => {
            ctx.state_mut().spawn.point = Point {
                x: ctx.slot(F::X, slot),
                y: ctx.slot(F::Y, slot),
            };
            let definition = script.word(if ctx.area() != 0 { 2 } else { 0 });
            super::drops::spawn(definition, k, 0, ctx);
            script.offset(4)
        }
        8827198 => script.jump(if ctx.slot(F::BehaviorA, slot) & 0x8000 != 0 {
            addr_off_86B166 as u16
        } else {
            addr_off_86B14B as u16
        }),
        8828917 => {
            ctx.set_earthquake_type(1);
            ctx.set_earthquake_timer(ctx.earthquake_timer() | 0x20);
            script.offset(0)
        }
        8828993 => {
            let old = ctx.slot(F::Y, slot);
            ctx.set_slot(
                F::Y,
                slot,
                (i32::from(old) + i32::from(script.word(0))) as u16,
            );
            script.offset(2)
        }
        8833232 => {
            ctx.set_slot(F::Y, slot, ctx.slot(F::Y, slot).wrapping_add(12));
            script.offset(0)
        }
        8840499 => {
            ctx.set_slot(F::VelocityY, slot, script.word(0));
            script.offset(2)
        }
        8815046 => {
            if ctx.decrement(F::Counter, slot) == 1 {
                script.offset(2)
            } else {
                script.jump(script.word(0))
            }
        }
        8815054 => {
            if ctx.decrement(F::Counter, slot) == 1 {
                script.offset(1)
            } else {
                script.offset(script.byte(0) as i8 as isize)
            }
        }
        8815071 => {
            random_move(ctx, slot, script);
            script.offset(4)
        }
        8815152 | 8815164 => {
            let old = ctx.slot(F::Properties, slot);
            let mask = script.word(0);
            ctx.set_slot(
                F::Properties,
                slot,
                if ea == 8815152 {
                    old | mask
                } else {
                    old & mask
                },
            );
            script.offset(2)
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
            let old = ctx.slot(F::Properties, slot);
            ctx.set_slot(
                F::Properties,
                slot,
                if set { old | mask } else { old & !mask },
            );
            script.offset(0)
        }
        8815256 => {
            ctx.set_slot(F::Radius, slot, script.word(0));
            script.offset(2)
        }
        8815265 => {
            ctx.set_slot(F::Radius, slot, 0);
            script.offset(0)
        }
        8815269 => {
            let x = ctx.player_x().wrapping_sub(ctx.slot(F::X, slot));
            let y = ctx.player_y().wrapping_sub(ctx.slot(F::Y, slot));
            let angle = ctx.calculate_angle_from_xy(x, y).wrapping_mul(2);
            let selected = (ctx.state().selected >> 1) as isize;
            ctx.set_slot(F::BehaviorA, selected, angle);
            let index = usize::from(angle >> 1);
            ctx.set_slot(
                F::VelocityX,
                selected,
                kSinCosTable8bit_Sext[index + 64] as u16,
            );
            ctx.set_slot(F::VelocityY, selected, kSinCosTable8bit_Sext[index] as u16);
            script.offset(0)
        }
        8815317 => {
            let mut color = script.word(2);
            let mut remaining = i32::from(script.byte(4));
            let mut source = script.word(0);
            loop {
                let value = ctx.raw_rom_word(0x86, source);
                ctx.set_palette_word(color >> 1, value);
                source = source.wrapping_add(2);
                color = color.wrapping_add(2);
                remaining -= 1;
                if remaining & 0x8000 != 0 {
                    break;
                }
            }
            script.offset(5)
        }
        8815357 => {
            ctx.queue_music(u16::from(script.byte(0)));
            script.offset(1)
        }
        8815369..=8815495 if (ea - 8815369) % 9 == 0 => {
            let command = (ea - 8815369) / 9;
            ctx.queue_sound(
                (command % 3 + 1) as u8,
                [6, 15, 3, 9, 1][(command / 3) as usize],
                u16::from(script.byte(0)),
            );
            script.offset(1)
        }
        8823894 => {
            if ctx.next_random() & 0xc000 == 0xc000 {
                script.jump(script.word(0))
            } else {
                script.offset(2)
            }
        }
        8826258 => {
            let amount = (i32::from(ctx.slot(F::VelocityX, slot) as i16) << 8) as u32;
            ctx.add_split(F::X, F::XFraction, slot, amount);
            script.jump(script.word(if ctx.slot(F::VelocityX, slot) & 0x8000 == 0 {
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
            super::lifecycle::spawn_room(ctx, definition, k);
            script.offset(0)
        }
        8833774 => {
            ctx.set_mother_brain_purple_breath_active(0);
            script.offset(0)
        }
        8836577 | 8836762 => {
            ctx.next_random();
            ctx.set_slot(F::VelocityX, slot, ctx.random_shifted_word());
            if ea == 8836577 {
                ctx.set_slot(F::VelocityY, slot, 192);
            }
            script.offset(0)
        }
        8817910 | 8817917 => {
            ctx.set_slot(
                F::PreAction,
                slot,
                if ea == 8817910 { 0x8dff } else { 0x8dca },
            );
            script.offset(0)
        }
        8820154 | 8820205 => {
            let definitions = if ea == 8820154 {
                [addr_stru_86966C, addr_stru_86967A]
            } else {
                [addr_stru_869688, addr_stru_869696]
            };
            for definition in definitions {
                ctx.state_mut().spawn.secondary = 0;
                super::lifecycle::spawn_room(ctx, definition as u16, k);
            }
            script.offset(0)
        }
        8820256 => {
            let count = ctx.slot_byte(F::BehaviorA, k as isize).wrapping_sub(1);
            ctx.set_slot_byte(F::BehaviorA, k as isize, count);
            if count as i8 >= 0 {
                let handler = ctx.slot(F::BehaviorB, slot);
                super::lifecycle::spawn_room(ctx, handler, k);
            }
            script.offset(0)
        }
        8827497 | 8827506 => {
            let angle = ctx.angle_to_player(k);
            set_velocity_toward(
                ctx,
                k,
                if ea == 8827497 {
                    angle & 0x7f
                } else {
                    angle | 0x80
                },
            );
            script.offset(0)
        }
        8827832 => {
            if ctx.kraid_health_eighth(ctx.slot(F::BehaviorB, slot)) & 0x8000 == 0 {
                script.jump(script.word(0))
            } else {
                script.offset(2)
            }
        }
        8831347 => script.jump(script.word(ctx.slot(F::BehaviorA, slot) as isize)),
        8831412 | 8835527 => {
            ctx.set_slot(F::Graphics, slot, 0);
            script.offset(0)
        }
        8835420 => {
            super::pre_actions::EprojPreInstr_KagosBugs_Func1(k, ctx);
            super::pre_actions::EprojPreInstr_KagosBugs_Func2(k, ctx);
            let mut direction = (ctx.random_number() & 0x300).wrapping_add(2048);
            ctx.set_slot(F::VelocityY, slot, direction.wrapping_neg());
            let enemy = ctx.slot(F::BehaviorB, slot);
            let distance = ctx.enemy_x(enemy).wrapping_sub(ctx.slot(F::X, slot));
            let absolute = if distance & 0x8000 != 0 {
                distance.wrapping_neg()
            } else {
                distance
            };
            if (absolute as i16).wrapping_sub(0x30) >= 0 {
                direction = if distance & 0x8000 != 0 { 0xffff } else { 0 };
            }
            ctx.set_slot(
                F::VelocityX,
                slot,
                if direction & 0x100 != 0 {
                    (-0x200i16) as u16
                } else {
                    0x200
                },
            );
            ctx.set_slot(F::PreAction, slot, 0xd0ec);
            script.offset(0)
        }
        8835510 => {
            ctx.set_slot(
                F::BehaviorC,
                slot,
                (ctx.random_number() & 0x1f).wrapping_add(1),
            );
            ctx.set_slot(F::PreAction, slot, 0xd0ca);
            script.offset(0)
        }
        8836594 => {
            let odd = ctx.nmi_frame() & 1 != 0;
            ctx.set_slot(
                F::X,
                slot,
                if odd {
                    ctx.slot(F::BehaviorB, slot)
                } else {
                    (256 - i32::from(ctx.slot(F::BehaviorB, slot))) as u16
                },
            );
            ctx.set_slot(F::Spritemap, slot, script.word(if odd { 0 } else { 2 }));
            ctx.set_slot(F::Program, slot, script.bank_address().wrapping_add(4));
            ctx.set_slot(F::Delay, slot, 1);
            None
        }
        8836650 => {
            ctx.set_slot(
                F::X,
                slot,
                if ctx.nmi_frame() & 1 != 0 {
                    ctx.slot(F::BehaviorB, slot)
                } else {
                    (-4608i16) as u16
                },
            );
            ctx.set_slot(F::Spritemap, slot, script.word(0));
            ctx.set_slot(F::Program, slot, script.bank_address().wrapping_add(2));
            ctx.set_slot(F::Delay, slot, 1);
            None
        }
        8838234 => {
            ctx.set_slot(F::Properties, slot, 12288);
            script.offset(0)
        }
        8838263 => {
            ctx.state_mut().spawn.point = Point {
                x: ctx.slot(F::X, slot),
                y: ctx.slot(F::Y, slot),
            };
            super::lifecycle::spawn_room(ctx, addr_kEproj_Spores as u16, 0);
            script.offset(0)
        }
        8842467 | 8842519 => {
            let (mask, half) = if ea == 8842467 {
                (0x3f, 32)
            } else {
                (0x1f, 16)
            };
            random_sprite(ctx, k, script, mask, half);
            script.offset(2)
        }
        8842891 | 8842903 | 8842915 => {
            ctx.queue_sound(
                2,
                1,
                match ea {
                    8842891 => 9,
                    8842903 => 0x24,
                    _ => 0xb,
                },
            );
            script.offset(0)
        }
        8842927 => script.jump(super::drops::convert(k, ctx)),
        8843024 => {
            super::drops::respawn(k, ctx);
            script.offset(0)
        }
        8815019 => script.jump(script.word(0)),
        8815024 => script.offset(script.byte(0) as i8 as isize),
        8822906 => {
            super::pre_actions::EprojPreInstr_PirateMotherBrainLaser_MoveRight(k, ctx);
            script.offset(0)
        }
        8822876 => {
            super::pre_actions::EprojPreInstr_PirateMotherBrainLaser_MoveLeft(k, ctx);
            script.offset(0)
        }
        8815070 => script.offset(0),
        _ => {
            ctx.unreachable();
            None
        }
    };
    crate::verify_probe::hit("eproj.command.executed");
    result
}
