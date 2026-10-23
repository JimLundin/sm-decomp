//! Ordered player and shot contact with owned enemy projectiles.
#![forbid(unsafe_code)]
use super::compat::Execution;
use super::state::SlotField as F;
use crate::game::contact_rules::{axis, is_shot};
use crate::types::EprojDef;
use core::mem::offset_of;

pub(super) fn enemy_projectiles_hit_samus(ctx: &mut Execution<'_>) {
    ctx.set_enemy_processing_stage(11);
    if ctx.player_invincibility() != 0 || ctx.player_contact_damage_index() != 0 {
        return;
    }
    for index in (0..18).rev() {
        if ctx.slot(F::Definition, index) == 0
            || ctx.slot(F::Properties, index) & 0x2000 != 0
            || ctx.slot(F::Radius, index) == 0
        {
            continue;
        }
        let radii = ctx.slot(F::Radius, index);
        if axis(
            ctx.player_x(),
            ctx.slot(F::X, index),
            ctx.player_x_radius(),
            radii & 0xff,
        ) && axis(
            ctx.player_y(),
            ctx.slot(F::Y, index),
            ctx.player_y_radius(),
            radii >> 8,
        ) {
            ctx.set_collision_detection_index((index * 2) as u16);
            hit_samus((index * 2) as u16, ctx);
        }
    }
}

pub(super) fn hit_samus(offset: u16, ctx: &mut Execution<'_>) {
    ctx.set_player_invincibility(96);
    ctx.set_player_knockback(5);
    // The original definition access uses an unrounded native byte offset.
    let definition = u16::from_ne_bytes([
        ctx.slot_byte(F::Definition, offset as isize),
        ctx.slot_byte(F::Definition, offset as isize + 1),
    ]);
    let definition = ctx.record(0x86, definition);
    let instruction = definition.word(offset_of!(EprojDef, hit_instruction_list));
    let index = (offset >> 1) as isize;
    if instruction != 0 {
        ctx.set_slot(F::Program, index, instruction);
        ctx.set_slot(F::Delay, index, 1);
    }
    if ctx.slot(F::Properties, index) & 0x4000 == 0 {
        ctx.set_slot_byte(F::Definition, offset as isize, 0);
        ctx.set_slot_byte(F::Definition, offset as isize + 1, 0);
    }
    let damage = ctx.reduce_suit_damage(ctx.slot(F::Properties, index) & 0xfff);
    ctx.damage_player(damage);
    let direction = (ctx.player_x().wrapping_sub(ctx.slot(F::X, index)) as i16 >= 0) as u16;
    ctx.set_knockback_x_dir(direction);
}

pub(super) fn shoot_enemy_projectiles(ctx: &mut Execution<'_>) {
    ctx.set_enemy_processing_stage(12);
    if ctx.player_projectile_count() == 0 {
        return;
    }
    for enemy in (0..18).rev() {
        if ctx.slot(F::Definition, enemy) == 0 || ctx.slot(F::Properties, enemy) & 0x8000 == 0 {
            continue;
        }
        for shot in 0..5 {
            if ctx.slot(F::DropFlags, enemy) == 2 {
                break;
            }
            if is_shot(ctx.player_projectile_type(shot))
                && ctx.slot(F::X, enemy) & 0xffe0
                    == ctx.player_projectile_x_pos((shot) as isize) & 0xffe0
                && ctx.slot(F::Y, enemy) & 0xffe0
                    == ctx.player_projectile_y_pos((shot) as isize) & 0xffe0
            {
                hit_enemy_projectile((enemy * 2) as u16, shot * 2, ctx);
            }
        }
    }
}

pub(super) fn hit_enemy_projectile(enemy_offset: u16, shot_offset: u16, ctx: &mut Execution<'_>) {
    let shot = shot_offset >> 1;
    let enemy = (enemy_offset >> 1) as isize;
    if ctx.player_projectile_type(shot) & 8 == 0 {
        let direction = ctx.player_projectile_direction(shot) | 0x10;
        ctx.set_player_projectile_direction(shot, direction);
    }
    if ctx.slot(F::DropFlags, enemy) == 1 {
        // Preserve the dud's enemy-slot indexing into the player-shot columns.
        let x = ctx.player_projectile_x_pos((enemy as u16) as isize);
        let y = ctx.player_projectile_y_pos((enemy as u16) as isize);
        ctx.create_sprite(x, y, 6, 0);
        ctx.queue_sound(1, 6, 0x3d);
    } else {
        ctx.set_slot(F::BehaviorC, enemy, ctx.player_projectile_type(shot));
        let definition = ctx.record(0x86, ctx.slot(F::Definition, enemy));
        ctx.set_slot(
            F::Program,
            enemy,
            definition.word(offset_of!(EprojDef, shot_instruction_list)),
        );
        ctx.set_slot(F::Delay, enemy, 1);
        ctx.set_slot(F::PreAction, enemy, 0x84fb);
        ctx.set_slot(F::Properties, enemy, ctx.slot(F::Properties, enemy) & 0xfff);
    }
}
