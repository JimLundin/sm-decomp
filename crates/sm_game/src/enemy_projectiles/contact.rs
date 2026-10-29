//! Ordered player and shot contact with owned enemy projectiles.
#![forbid(unsafe_code)]
use super::access::{enemy, enemy_byte, set_shot_direction, shot_word};
use super::host::{DefinitionField, Host, LoadedRecord, ShotMember};
use crate::contact_rules::{axis, is_shot};
use crate::geometry::Point;
use crate::{
    sprite_effect_state::{LegacySpriteDefinition, SpriteSpawn},
    sprite_objects,
};

pub fn enemy_projectiles_hit_samus(h: &mut impl Host) {
    h.session_mut().room.enemy_processing_stage = 11;
    if h.session().player.invincibility != 0 || h.session().player.contact.contact_attack != 0 {
        return;
    }
    for index in (0..18).rev() {
        if enemy!(h, index, definition) == 0 || enemy!(h, index, properties) & 0x2000 != 0 {
            continue;
        }
        let x_radius = u16::from(enemy_byte!(h, index * 2, radius));
        if x_radius == 0 {
            continue;
        }
        h.session_mut().compatibility_enemy_scratch = x_radius;
        let y_radius = u16::from(enemy_byte!(h, index * 2 + 1, radius));
        if y_radius == 0 {
            continue;
        }
        if axis(
            h.session().player.x,
            enemy!(h, index, x),
            h.session().player.contact.x_radius,
            x_radius,
        ) && axis(
            h.session().player.y,
            enemy!(h, index, y),
            h.session().player.contact.radius,
            y_radius,
        ) {
            h.session_mut().room.collision_detection_index = (index * 2) as u16;
            hit_samus((index * 2) as u16, h);
        }
    }
}

pub fn hit_samus(offset: u16, h: &mut impl Host) {
    h.session_mut().player.invincibility = 96;
    h.session_mut().player.knockback_timer = 5;
    // The original definition access uses an unrounded native byte offset.
    let definition = u16::from_ne_bytes([
        enemy_byte!(h, offset as isize, definition),
        enemy_byte!(h, offset as isize + 1, definition),
    ]);
    let definition = h.loaded_record(0x86, definition);
    let instruction = definition.definition_word(DefinitionField::HitProgram);
    let index = (offset >> 1) as isize;
    if instruction != 0 {
        enemy!(h, index, program = instruction);
        enemy!(h, index, delay = 1);
    }
    if enemy!(h, index, properties) & 0x4000 == 0 {
        enemy_byte!(h, offset as isize, definition = 0);
        enemy_byte!(h, offset as isize + 1, definition = 0);
    }
    let damage = crate::combat_rules::suit_damage(
        enemy!(h, index, properties) & 0xfff,
        h.session().inventory.equipment.active,
    );
    h.damage_player(damage);
    let direction = (h.session().player.x.wrapping_sub(enemy!(h, index, x)) as i16 >= 0) as u16;
    h.session_mut().player.knockback_x_direction = direction;
}

pub fn shoot_enemy_projectiles(h: &mut impl Host) {
    h.session_mut().room.enemy_processing_stage = 12;
    if h.session().projectiles.count == 0 {
        return;
    }
    for enemy in (0..18).rev() {
        if enemy!(h, enemy, definition) == 0 || enemy!(h, enemy, properties) & 0x8000 == 0 {
            continue;
        }
        for shot in 0..5 {
            if enemy!(h, enemy, drop_flags) == 2 {
                break;
            }
            if is_shot(shot_word(h, ShotMember::Kind, shot as isize))
                && enemy!(h, enemy, x) & 0xffe0
                    == shot_word(h, ShotMember::X, shot as isize) & 0xffe0
                && enemy!(h, enemy, y) & 0xffe0
                    == shot_word(h, ShotMember::Y, shot as isize) & 0xffe0
            {
                hit_enemy_projectile((enemy * 2) as u16, shot * 2, h);
            }
        }
    }
}

pub fn hit_enemy_projectile(enemy_offset: u16, shot_offset: u16, h: &mut impl Host) {
    let shot = shot_offset >> 1;
    let enemy = (enemy_offset >> 1) as isize;
    if shot_word(h, ShotMember::Kind, shot as isize) & 8 == 0 {
        let direction = shot_word(h, ShotMember::Direction, shot as isize) | 0x10;
        set_shot_direction(h, shot as isize, direction);
    }
    if enemy!(h, enemy, drop_flags) == 1 {
        // Preserve the dud's enemy-slot indexing into the player-shot columns.
        let x = shot_word(h, ShotMember::X, (enemy as u16) as isize);
        let y = shot_word(h, ShotMember::Y, (enemy as u16) as isize);
        sprite_objects::spawn(
            h,
            SpriteSpawn {
                position: Point { x, y },
                definition: LegacySpriteDefinition(6),
                appearance: 0,
            },
        );
        h.queue_sound(1, 6, 0x3d);
    } else {
        enemy!(
            h,
            enemy,
            behavior_c = shot_word(h, ShotMember::Kind, shot as isize)
        );
        let definition = h.loaded_record(0x86, enemy!(h, enemy, definition));
        enemy!(
            h,
            enemy,
            program = definition.definition_word(DefinitionField::ShotProgram)
        );
        enemy!(h, enemy, delay = 1);
        enemy!(h, enemy, pre_action = 0x84fb);
        enemy!(h, enemy, properties = enemy!(h, enemy, properties) & 0xfff);
    }
}
