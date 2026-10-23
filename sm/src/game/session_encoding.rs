//! Sole byte-compatible projection of the migrated aggregate. No continuation.
#![forbid(unsafe_code)]
use crate::game::{external_inputs::ExternalInputs, session::Session};
use crate::ram_map::{word_arrays as arrays, words as ram};
// Direct projection for the 1,024-byte pool. Normal gameplay uses typed members;
// this is solely the active/tail compatibility mapping, shared by reads/writes.
macro_rules! sprite_projection {
    ($s:ident,$offset:ident,($($borrow:tt)*)) => {{
        let index = ($offset % 128) / 2;
        let column = $offset / 128;
        if index < 32 {
            let slot = $($borrow)* $s.sprite_effects.active[index];
            match column {
                0 => $($borrow)* slot.instruction,
                1 => $($borrow)* slot.timer,
                2 => $($borrow)* slot.appearance,
                3 => $($borrow)* slot.x.whole,
                4 => $($borrow)* slot.x.fraction,
                5 => $($borrow)* slot.y.whole,
                6 => $($borrow)* slot.y.fraction,
                7 => $($borrow)* slot.disable_flags,
                _ => unreachable!(),
            }
        } else {
            let index = index - 32;
            let tail = $($borrow)* $s.sprite_compatibility;
            match column {
                0 => $($borrow)* tail.instruction[index],
                1 => $($borrow)* tail.timer[index],
                2 => $($borrow)* tail.appearance[index],
                3 => $($borrow)* tail.x[index],
                4 => $($borrow)* tail.x_fraction[index],
                5 => $($borrow)* tail.y[index],
                6 => $($borrow)* tail.y_fraction[index],
                7 => $($borrow)* tail.disable_flags[index],
                _ => unreachable!(),
            }
        }
    }};
}
// Byte collections have one compatibility projection, shared by reads/writes.
macro_rules! flag_projection {
    ($s:ident,$offset:ident,($($borrow:tt)*)) => {
        match $offset {
            0..=7 => $($borrow)* $s.gameplay_flags.events[$offset],
            8..=15 => $($borrow)* $s.gameplay_flags.bosses_by_area[$offset - 8],
            16..=79 => $($borrow)* $s.gameplay_flags.used_room_objects[$offset - 16],
            80..=143 => $($borrow)* $s.gameplay_flags.collected_locations[$offset - 80],
            144..=207 => $($borrow)* $s.gameplay_flags.opened_doors[$offset - 144],
            _ => unreachable!(),
        }
    };
}
macro_rules! scalar_words {
    ($s:ident,$f:ident,($($borrow:tt)*))=>{
        $f(0x05e7,$($borrow)* $s.flag_compatibility.bitmask);
        $f(0x1844,$($borrow)* $s.sprite_effects.selected);
        for index in 0..32 {
            $f(0xef78+index*2,$($borrow)* $s.sprite_effects.active[index].instruction);
            $f(0xeff8+index*2,$($borrow)* $s.sprite_effects.active[index].timer);
            $f(0xf078+index*2,$($borrow)* $s.sprite_effects.active[index].appearance);
            $f(0xf0f8+index*2,$($borrow)* $s.sprite_effects.active[index].x.whole);
            $f(0xf178+index*2,$($borrow)* $s.sprite_effects.active[index].x.fraction);
            $f(0xf1f8+index*2,$($borrow)* $s.sprite_effects.active[index].y.whole);
            $f(0xf278+index*2,$($borrow)* $s.sprite_effects.active[index].y.fraction);
            $f(0xf2f8+index*2,$($borrow)* $s.sprite_effects.active[index].disable_flags);
            $f(0xefb8+index*2,$($borrow)* $s.sprite_compatibility.instruction[index]);
            $f(0xf038+index*2,$($borrow)* $s.sprite_compatibility.timer[index]);
            $f(0xf0b8+index*2,$($borrow)* $s.sprite_compatibility.appearance[index]);
            $f(0xf138+index*2,$($borrow)* $s.sprite_compatibility.x[index]);
            $f(0xf1b8+index*2,$($borrow)* $s.sprite_compatibility.x_fraction[index]);
            $f(0xf238+index*2,$($borrow)* $s.sprite_compatibility.y[index]);
            $f(0xf2b8+index*2,$($borrow)* $s.sprite_compatibility.y_fraction[index]);
            $f(0xf338+index*2,$($borrow)* $s.sprite_compatibility.disable_flags[index]);
        }
        $f(0xa48,$($borrow)* $s.player.palette.hurt_flash);
        $f(0xa72,$($borrow)* $s.player.palette.visor_timer);
        $f(0xa74,$($borrow)* $s.player.palette.suit_index);
        $f(0xa80,$($borrow)* $s.player.palette.xray_progress);
        $f(0xad0,$($borrow)* $s.player.palette.timer);
        $f(0xdf2,$($borrow)* $s.player.palette.crystal_flash_delay);
        $f(0xa4e,$($borrow)* $s.player.periodic_damage_fraction);
        $f(0x1e6f,$($borrow)* $s.mechanisms.statue_animation);
        for index in 0..256 {
            $f(0xc000+index*2,$($borrow)* $s.visual.colors[index]);
        }
        for index in 0..256 {
            $f(0xc200+index*2,$($borrow)* $s.visual.targets[index]);
        }
        for index in 0..8 {
            $f(0x1e7d+index*2,$($borrow)* $s.visual.palette_effects.slots[index].definition);
            $f(0x1e8d+index*2,$($borrow)* $s.visual.palette_effects.slots[index].color_offset);
            $f(0x1ead+index*2,$($borrow)* $s.visual.palette_effects.slots[index].pre_action);
            $f(0x1ebd+index*2,$($borrow)* $s.visual.palette_effects.slots[index].program);
            $f(0x1ecd+index*2,$($borrow)* $s.visual.palette_effects.slots[index].delay);
            $f(0x1edd+index*2,$($borrow)* $s.visual.palette_effects.slots[index].counter);
        }
        for index in 0..6 {
            $f(0x1ef5+index*2,$($borrow)* $s.visual.animated_tiles.slots[index].definition);
            $f(0x1f01+index*2,$($borrow)* $s.visual.animated_tiles.slots[index].counter);
            $f(0x1f0d+index*2,$($borrow)* $s.visual.animated_tiles.slots[index].program);
            $f(0x1f19+index*2,$($borrow)* $s.visual.animated_tiles.slots[index].delay);
            $f(0x1f25+index*2,$($borrow)* $s.visual.animated_tiles.slots[index].source);
            $f(0x1f31+index*2,$($borrow)* $s.visual.animated_tiles.slots[index].size);
            $f(0x1f3d+index*2,$($borrow)* $s.visual.animated_tiles.slots[index].vram);
        }
        $f(0x1e79,$($borrow)* $s.visual.palette_effects.flags);
        $f(0x1e7b,$($borrow)* $s.visual.palette_effects.selected);
        $f(0x1eed,$($borrow)* $s.visual.palette_effects.heat);
        $f(0x1eef,$($borrow)* $s.visual.palette_effects.previous_heat);
        $f(0x1ef1,$($borrow)* $s.visual.animated_tiles.flags);
        $f(0x1ef3,$($borrow)* $s.visual.animated_tiles.selected);
        $f(0x1f49,$($borrow)* $s.visual.animated_tiles.last_instruction);
        $f(ram::projectile_counter,$($borrow)* $s.projectiles.count);
        $f(ram::bomb_counter,$($borrow)* $s.projectiles.bomb_count);
        $f(ram::projectile_index,$($borrow)* $s.projectiles.selected);
        $f(ram::cooldown_timer,$($borrow)* $s.projectiles.controls.cooldown);
        $f(ram::bomb_spread_charge_timeout_counter,$($borrow)* $s.projectiles.controls.spread_charge);
        $f(ram::used_for_sba_attacksB60,$($borrow)* $s.projectiles.controls.orbit_step);
        $f(ram::flare_counter,$($borrow)* $s.projectiles.controls.charge);
        $f(ram::prev_beam_charge_counter,$($borrow)* $s.projectiles.controls.previous_charge);
        $f(ram::charged_shot_glow_timer,$($borrow)* $s.projectiles.controls.glow);
        $f(ram::samus_charge_palette_index,$($borrow)* $s.projectiles.controls.charge_palette);
        $f(ram::play_resume_charging_beam_sfx,$($borrow)* $s.projectiles.controls.resume_sound);
        $f(ram::samus_echoes_sound_flag,$($borrow)* $s.projectiles.controls.echo_sound);
        $f(ram::new_projectile_direction_changed_pose,$($borrow)* $s.projectiles.controls.pending_aim);
        $f(ram::projectile_invincibility_timer,$($borrow)* $s.projectiles.controls.protection);
        $f(ram::hud_item_index,$($borrow)* $s.projectiles.controls.selected_weapon);
        $f(ram::samus_auto_cancel_hud_item_index,$($borrow)* $s.projectiles.controls.automatic_weapon);
        $f(ram::hud_item_changed_this_frame,$($borrow)* $s.projectiles.controls.weapon_changed);
        $f(ram::samus_x_pos,$($borrow)* $s.player.x);
        $f(ram::samus_y_pos,$($borrow)* $s.player.y);
        $f(ram::samus_missiles,$($borrow)* $s.inventory.ammunition[0].current);
        $f(ram::samus_super_missiles,$($borrow)* $s.inventory.ammunition[1].current);
        $f(ram::samus_power_bombs,$($borrow)* $s.inventory.ammunition[2].current);
        $f(ram::grapple_beam_function,$($borrow)* $s.player.grapple);
        $f(ram::samus_movement_handler,$($borrow)* $s.player.movement_handler);
        $f(ram::samus_shine_timer,$($borrow)* $s.player.shine_timer);
        $f(ram::timer_for_shine_timer,$($borrow)* $s.player.shine_tick);
        $f(ram::special_samus_palette_frame,$($borrow)* $s.player.palette_frame);
        $f(ram::samus_special_super_palette_flags,$($borrow)* $s.player.special_palette_flags);
        $f(ram::samus_new_pose,$($borrow)* $s.projectiles.preparation.next_pose);
        $f(ram::samus_new_pose_interrupted,$($borrow)* $s.projectiles.preparation.interrupted_pose);
        $f(ram::samus_new_pose_transitional,$($borrow)* $s.projectiles.preparation.transitional_pose);
        $f(ram::samus_momentum_routine_index,$($borrow)* $s.projectiles.preparation.momentum_routine);
        $f(ram::samus_special_transgfx_index,$($borrow)* $s.projectiles.preparation.special_graphics);
        $f(ram::samus_hurt_switch_index,$($borrow)* $s.projectiles.preparation.hurt_switch);
        $f(ram::controller1_input_for_demo,$($borrow)* $s.projectiles.preparation.demo_held);
        $f(ram::controller1_new_input_for_demo,$($borrow)* $s.projectiles.preparation.demo_pressed);
        $f(ram::demo_backup_prev_controller_input,$($borrow)* $s.projectiles.preparation.demo_filtered_held);
        $f(ram::demo_backup_prev_controller_input_new,$($borrow)* $s.projectiles.preparation.demo_filtered_pressed);
        $f(ram::samus_anim_frame_skip,$($borrow)* $s.projectiles.preparation.animation_skip);
        $f(ram::UNUSED_word_7E0DFA,$($borrow)* $s.projectiles.preparation.aim_history);
        $f(0xa10,$($borrow)* $s.projectiles.preparation.previous_movement_and_facing);
        $f(ram::power_bomb_flag,$($borrow)* $s.projectiles.power_bomb.flag);
        $f(ram::power_bomb_explosion_x_pos,$($borrow)* $s.projectiles.power_bomb.x);
        $f(ram::power_bomb_explosion_y_pos,$($borrow)* $s.projectiles.power_bomb.y);
        $f(ram::cur_block_index,$($borrow)* $s.room.collision_cursor);
        $f(ram::earthquake_type,$($borrow)* $s.feedback.earthquake_kind);
        $f(ram::earthquake_timer,$($borrow)* $s.feedback.earthquake_timer);
        $f(ram::speed_echoes_index,$($borrow)* $s.projectiles.echoes.index);
        for index in 0..3 {
            $f(ram::flare_animation_frame+index*2,$($borrow)* $s.projectiles.controls.flare_frames[index]);
            $f(ram::flare_animation_timer+index*2,$($borrow)* $s.projectiles.controls.flare_timers[index]);
        }
        for index in 0..4 {
            $f(arrays::speed_echo_xpos+index*2,$($borrow)* $s.projectiles.echoes.x[index]);
            $f(arrays::speed_echo_ypos+index*2,$($borrow)* $s.projectiles.echoes.y[index]);
            $f(arrays::speed_echo_xspeed+index*2,$($borrow)* $s.projectiles.echoes.x_speed[index]);
        }
        for index in 0..10 {
            $f(0xb64+index*2,$($borrow)* $s.projectiles.shots[index].x);
            $f(0xb78+index*2,$($borrow)* $s.projectiles.shots[index].y);
            $f(0xb8c+index*2,$($borrow)* $s.projectiles.shots[index].x_fraction);
            $f(0xba0+index*2,$($borrow)* $s.projectiles.shots[index].y_fraction);
            $f(0xbb4+index*2,$($borrow)* $s.projectiles.shots[index].x_radius);
            $f(0xbc8+index*2,$($borrow)* $s.projectiles.shots[index].y_radius);
            $f(0xbdc+index*2,$($borrow)* $s.projectiles.shots[index].x_speed);
            $f(0xbf0+index*2,$($borrow)* $s.projectiles.shots[index].y_speed);
            $f(0xc04+index*2,$($borrow)* $s.projectiles.shots[index].direction);
            $f(0xc18+index*2,$($borrow)* $s.projectiles.shots[index].kind);
            $f(0xc2c+index*2,$($borrow)* $s.projectiles.shots[index].damage);
            $f(0xc40+index*2,$($borrow)* $s.projectiles.shots[index].cursor);
            $f(0xc54+index*2,$($borrow)* $s.projectiles.shots[index].remaining);
            $f(0xc68+index*2,$($borrow)* $s.projectiles.shots[index].motion);
            $f(0xc7c+index*2,$($borrow)* $s.projectiles.shots[index].variable);
            $f(0xc90+index*2,$($borrow)* $s.projectiles.shots[index].trail_time);
            $f(0xca4+index*2,$($borrow)* $s.projectiles.shots[index].auxiliary);
            $f(0xcb8+index*2,$($borrow)* $s.projectiles.shots[index].sprite);
        }
        for index in 0..18 {
            $f(0xd658+index*2,$($borrow)* $s.projectiles.trails[index].left.remaining);
            $f(0xd67c+index*2,$($borrow)* $s.projectiles.trails[index].right.remaining);
            $f(0xd6a0+index*2,$($borrow)* $s.projectiles.trails[index].left.cursor);
            $f(0xd6c4+index*2,$($borrow)* $s.projectiles.trails[index].right.cursor);
            $f(0xd6e8+index*2,$($borrow)* $s.projectiles.trails[index].left.tile);
            $f(0xd70c+index*2,$($borrow)* $s.projectiles.trails[index].right.tile);
            $f(0xd730+index*2,$($borrow)* $s.projectiles.trails[index].left.x);
            $f(0xd754+index*2,$($borrow)* $s.projectiles.trails[index].right.x);
            $f(0xd778+index*2,$($borrow)* $s.projectiles.trails[index].left.y);
            $f(0xd79c+index*2,$($borrow)* $s.projectiles.trails[index].right.y);
        }
        $f(0x12,$($borrow)* $s.spawn_scratch.header);
        $f(0x14,$($borrow)* $s.spawn_scratch.coordinates);
        $f(0x16,$($borrow)* $s.spawn_scratch.argument);
        $f(0x9a2,$($borrow)* $s.inventory.equipment.active);
        $f(0x9a4,$($borrow)* $s.inventory.equipment.owned);
        $f(0x9a6,$($borrow)* $s.inventory.beams.active);
        $f(0x9a8,$($borrow)* $s.inventory.beams.owned);
        $f(0x9c2,$($borrow)* $s.inventory.health.current);
        $f(0x9c4,$($borrow)* $s.inventory.health.maximum);
        $f(0x9c8,$($borrow)* $s.inventory.ammunition[0].maximum);
        $f(0x9cc,$($borrow)* $s.inventory.ammunition[1].maximum);
        $f(0x9d0,$($borrow)* $s.inventory.ammunition[2].maximum);
        $f(0x9d4,$($borrow)* $s.inventory.reserve_capacity);
        $f(0x9c0,$($borrow)* $s.inventory.reserve_mode);
        $f(0xa1c,$($borrow)* $s.player.pose);
        $f(0xa24,$($borrow)* $s.player.previous_pose);
        $f(0xa1e,$($borrow)* $s.player.movement_and_facing);
        $f(0x18a8,$($borrow)* $s.player.invincibility);
        $f(0xa50,$($borrow)* $s.player.periodic_damage);
        $f(0xb00,$($borrow)* $s.player.contact.radius);
        $f(0xb4c,$($borrow)* $s.player.contact.deceleration);
        $f(0xb3c,$($borrow)* $s.player.contact.momentum);
        $f(0xb3e,$($borrow)* $s.player.contact.speed_boost);
        $f(0xb44,$($borrow)* $s.player.contact.extra_run.fraction);
        $f(0xb42,$($borrow)* $s.player.contact.extra_run.whole);
        $f(0xb48,$($borrow)* $s.player.contact.base_run.fraction);
        $f(0xb46,$($borrow)* $s.player.contact.base_run.whole);
        $f(0xb36,$($borrow)* $s.player.contact.y_direction);
        $f(0xb2c,$($borrow)* $s.player.contact.y_speed.fraction);
        $f(0xb2e,$($borrow)* $s.player.contact.y_speed.whole);
        $f(0xb56,$($borrow)* $s.player.contact.external_x.fraction);
        $f(0xb58,$($borrow)* $s.player.contact.external_x.whole);
        $f(0xb5a,$($borrow)* $s.player.contact.external_y.fraction);
        $f(0xb5c,$($borrow)* $s.player.contact.external_y.whole);
        $f(0xb32,$($borrow)* $s.player.contact.y_acceleration.fraction);
        $f(0xb34,$($borrow)* $s.player.contact.y_acceleration.whole);
        $f(0xaf4,$($borrow)* $s.player.contact.auto_jump);
        $f(0xb02,$($borrow)* $s.player.contact.collision_direction);
        $f(0xa6e,$($borrow)* $s.player.contact.contact_attack);
        $f(0x1e73,$($borrow)* $s.player.contact.reaction_point);
        $f(0x1e71,$($borrow)* $s.player.contact.sand_visits);
        $f(0xa6c,$($borrow)* $s.player.contact.movement_speed_table);
        $f(0x79f,$($borrow)* $s.room.area);
        $f(0x7a5,$($borrow)* $s.room.width);
        $f(0x7a7,$($borrow)* $s.room.height);
        $f(0x7a9,$($borrow)* $s.room.scroll_width);
        $f(0x7ab,$($borrow)* $s.room.scroll_height);
        $f(0x7b9,$($borrow)* $s.room.size_bytes);
        $f(0x197e,$($borrow)* $s.room.liquid_options);
        $f(0x1978,$($borrow)* $s.room.liquid_height);
        $f(0x197a,$($borrow)* $s.room.liquid_target);
        $f(0x197c,$($borrow)* $s.room.liquid_velocity);
        $f(0x1980,$($borrow)* $s.room.liquid_delay);
        $f(0xe50,$($borrow)* $s.room.enemy_deaths);
        $f(0xe52,$($borrow)* $s.room.enemy_quota);
        $f(0xd91a,$($borrow)* $s.room.loaded_items);
        $f(0xcea,$($borrow)* $s.room.power_bomb_radius);
        $f(0x5f5,$($borrow)* $s.feedback.sound_disabled);
        $f(0xdc8,$($borrow)* $s.messages.queued);
        $f(0x1e75,$($borrow)* $s.mechanisms.save_locked);
        $f(0x1c23,$($borrow)* $s.mechanisms.flags);
        $f(0x1c25,$($borrow)* $s.mechanisms.draw_cursor);
        $f(0x1c27,$($borrow)* $s.mechanisms.selected);
        $f(0x1c29,$($borrow)* $s.mechanisms.block_x);
        $f(0x1c2b,$($borrow)* $s.mechanisms.block_y);
        $f(0x1c2d,$($borrow)* $s.mechanisms.next_graphics);
        for index in 0..4 { $f(0x1c2f+index*2,$($borrow)* $s.mechanisms.saved_graphics[index]); }
        for index in 0..256 { $f(0xc6c8+index*2,$($borrow)* $s.mechanisms.tilemap[index]); }
        for index in 0..3 { $f(0x1e67+index*2,$($borrow)* $s.mechanisms.custom_draw[index]); }
        for index in 0..40 {
            $f(0x1c37+index*2,$($borrow)* $s.mechanisms.slots[index].header);
            $f(0x1c87+index*2,$($borrow)* $s.mechanisms.slots[index].block);
            $f(0x1cd7+index*2,$($borrow)* $s.mechanisms.slots[index].pre_action);
            $f(0x1d27+index*2,$($borrow)* $s.mechanisms.slots[index].program);
            $f(0x1d77+index*2,$($borrow)* $s.mechanisms.slots[index].counter);
            $f(0x1dc7+index*2,$($borrow)* $s.mechanisms.slots[index].room_argument);
            $f(0x1e17+index*2,$($borrow)* $s.mechanisms.slots[index].work0);
            $f(0xde1c+index*2,$($borrow)* $s.mechanisms.slots[index].instruction_delay);
            $f(0xde6c+index*2,$($borrow)* $s.mechanisms.slots[index].draw_program);
            $f(0xdebc+index*2,$($borrow)* $s.mechanisms.slots[index].return_program);
            $f(0xdf0c+index*2,$($borrow)* $s.mechanisms.slots[index].work1);
        }
        $f(0x198d,$($borrow)* $s.enemy_projectiles.flags);
        $f(0x1991,$($borrow)* $s.enemy_projectiles.selected);
        $f(0x1993,$($borrow)* $s.enemy_projectiles.spawn.primary);
        $f(0x1995,$($borrow)* $s.enemy_projectiles.spawn.secondary);
        $f(ram::health_drop_bias_flag,$($borrow)* $s.enemy_projectiles.drop_bias);
        $f(ram::samus_reserve_health,$($borrow)* $s.inventory.reserve_health);
        $f(ram::samus_reserve_missiles,$($borrow)* $s.inventory.reserve_missiles);
        $f(ram::samus_x_speed_divisor,$($borrow)* $s.player.speed_divisor);
        $f(ram::samus_knockback_timer,$($borrow)* $s.player.knockback_timer);
        $f(ram::knockback_x_dir,$($borrow)* $s.player.knockback_x_direction);
        $f(ram::enemy_processing_stage,$($borrow)* $s.room.enemy_processing_stage);
        $f(ram::collision_detection_index,$($borrow)* $s.room.collision_detection_index);
        $f(ram::current_slope_bts,$($borrow)* $s.room.slope_behavior);
        $f(ram::tourian_entrance_statue_finished,$($borrow)* $s.mechanisms.statue_finished);
        for index in 0..18 {
            $f(0x1997+index*2,$($borrow)* $s.enemy_projectiles.slots[index].definition);
            $f(0x19bb+index*2,$($borrow)* $s.enemy_projectiles.slots[index].graphics);
            $f(0x19df+index*2,$($borrow)* $s.enemy_projectiles.slots[index].counter);
            $f(0x1a03+index*2,$($borrow)* $s.enemy_projectiles.slots[index].pre_action);
            $f(0x1a27+index*2,$($borrow)* $s.enemy_projectiles.slots[index].x.fraction);
            $f(0x1a4b+index*2,$($borrow)* $s.enemy_projectiles.slots[index].x.whole);
            $f(0x1a6f+index*2,$($borrow)* $s.enemy_projectiles.slots[index].y.fraction);
            $f(0x1a93+index*2,$($borrow)* $s.enemy_projectiles.slots[index].y.whole);
            $f(0x1ab7+index*2,$($borrow)* $s.enemy_projectiles.slots[index].velocity.x);
            $f(0x1adb+index*2,$($borrow)* $s.enemy_projectiles.slots[index].velocity.y);
            $f(0x1aff+index*2,$($borrow)* $s.enemy_projectiles.slots[index].behavior_a);
            $f(0x1b23+index*2,$($borrow)* $s.enemy_projectiles.slots[index].behavior_b);
            $f(0x1b47+index*2,$($borrow)* $s.enemy_projectiles.slots[index].program);
            $f(0x1b6b+index*2,$($borrow)* $s.enemy_projectiles.slots[index].spritemap);
            $f(0x1b8f+index*2,$($borrow)* $s.enemy_projectiles.slots[index].delay);
            $f(0x1bb3+index*2,$($borrow)* $s.enemy_projectiles.slots[index].radius);
            $f(0x1bd7+index*2,$($borrow)* $s.enemy_projectiles.slots[index].properties);
            $f(0x1bfb+index*2,$($borrow)* $s.enemy_projectiles.slots[index].behavior_c);
            $f(0x97dc+index*2,$($borrow)* $s.enemy_projectiles.slots[index].angle);
            $f(0xf380+index*2,$($borrow)* $s.enemy_projectiles.slots[index].drop.flags);
            $f(0xf3c8+index*2,$($borrow)* $s.enemy_projectiles.slots[index].drop.enemy_definition);
            $f(0xf410+index*2,$($borrow)* $s.enemy_projectiles.slots[index].drop.killed_enemy);
        }
    };
}

pub(crate) fn read(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}
pub(crate) fn decode(bytes: &[u8]) -> Session {
    let mut s = Session::default();
    for (base, flags) in [
        (0xd820, s.gameplay_flags.events.as_mut_slice()),
        (0xd828, s.gameplay_flags.bosses_by_area.as_mut_slice()),
        (0xd830, s.gameplay_flags.used_room_objects.as_mut_slice()),
        (0xd870, s.gameplay_flags.collected_locations.as_mut_slice()),
        (0xd8b0, s.gameplay_flags.opened_doors.as_mut_slice()),
    ] {
        let end = base + flags.len();
        flags.copy_from_slice(&bytes[base..end]);
    }
    s.enemy_projectiles.spawn.angle = bytes[0x60b];
    let mut load = |at: usize, value: &mut u16| *value = read(bytes, at);
    scalar_words!(s,load,(&mut));
    for (index, value) in s.projectiles.launch.displacements.iter_mut().enumerate() {
        *value = ((u32::from(read(bytes, 0xdaa + index * 4)) << 16)
            | u32::from(read(bytes, 0xdac + index * 4))) as i32;
    }
    for (index, cell) in s.room.terrain.cells_mut().iter_mut().enumerate() {
        cell.level = read(bytes, 0x10002 + index * 2);
        cell.behavior = bytes[0x16402 + index];
    }
    for (index, tile) in s.room.terrain.tiles_mut().iter_mut().enumerate() {
        for (corner, value) in tile.iter_mut().enumerate() {
            *value = read(bytes, 0xa000 + index * 8 + corner * 2);
        }
    }
    s
}
pub(crate) fn visit(s: &Session, mut visit: impl FnMut(usize, u16)) {
    for (base, flags) in [
        (0xd820, s.gameplay_flags.events.as_slice()),
        (0xd828, s.gameplay_flags.bosses_by_area.as_slice()),
        (0xd830, s.gameplay_flags.used_room_objects.as_slice()),
        (0xd870, s.gameplay_flags.collected_locations.as_slice()),
        (0xd8b0, s.gameplay_flags.opened_doors.as_slice()),
    ] {
        for (index, pair) in flags.chunks_exact(2).enumerate() {
            visit(base + index * 2, u16::from_le_bytes([pair[0], pair[1]]));
        }
    }
    let mut emit = |at: usize, value: &u16| visit(at, *value);
    scalar_words!(s,emit,(&));
    for (index, &value) in s.projectiles.launch.displacements.iter().enumerate() {
        visit(0xdaa + index * 4, ((value as u32) >> 16) as u16);
        visit(0xdac + index * 4, value as u16);
    }
    for (index, cell) in s.room.terrain.cells().iter().enumerate() {
        visit(0x10002 + index * 2, cell.level);
    }
    for (index, tile) in s.room.terrain.tiles().iter().enumerate() {
        for (corner, &value) in tile.iter().enumerate() {
            visit(0xa000 + index * 8 + corner * 2, value);
        }
    }
}
pub(crate) fn encode(s: &Session, bytes: &mut [u8]) {
    bytes[0x60b] = s.enemy_projectiles.spawn.angle;
    visit(s, |at, value| {
        bytes[at..at + 2].copy_from_slice(&value.to_le_bytes())
    });
    for (index, cell) in s.room.terrain.cells().iter().enumerate() {
        bytes[0x16402 + index] = cell.behavior;
    }
}
pub(crate) fn projected_byte(s: &Session, at: usize) -> Option<u8> {
    if (0xd820..0xd8f0).contains(&at) {
        let offset = at - 0xd820;
        return Some(flag_projection!(s, offset, ()));
    }
    if (0xef78..0xf378).contains(&at) {
        let offset = at - 0xef78;
        let word = sprite_projection!(s,offset,(&));
        return Some(word.to_le_bytes()[offset % 2]);
    }
    if (0xc000..0xc400).contains(&at) {
        let offset = at - 0xc000;
        let colors = if offset < 512 {
            &s.visual.colors
        } else {
            &s.visual.targets
        };
        return Some(colors[(offset % 512) / 2].to_le_bytes()[offset % 2]);
    }
    if at == 0x60b {
        return Some(s.enemy_projectiles.spawn.angle);
    }
    if (0x10002..0x16402).contains(&at) {
        let offset = at - 0x10002;
        return Some(s.room.terrain.cells()[offset / 2].level.to_le_bytes()[offset % 2]);
    }
    if (0x16402..0x19602).contains(&at) {
        return Some(s.room.terrain.cells()[at - 0x16402].behavior);
    }
    if (0xa000..0xc000).contains(&at) {
        let offset = at - 0xa000;
        return Some(s.room.terrain.tiles()[offset / 8][offset % 8 / 2].to_le_bytes()[offset % 2]);
    }
    if (0xdaa..0xdba).contains(&at) {
        let index = (at - 0xdaa) / 4;
        let lane = (at - 0xdaa) % 4;
        let shift = if lane < 2 {
            16 + lane * 8
        } else {
            (lane - 2) * 8
        };
        return Some(((s.projectiles.launch.displacements[index] as u32) >> shift) as u8);
    }
    let mut result = None;
    let mut select = |base: usize, value: &u16| {
        if at == base || at == base + 1 {
            result = Some(value.to_le_bytes()[at - base]);
        }
    };
    scalar_words!(s,select,(&));
    result
}
pub(crate) fn write_projected_byte(s: &mut Session, at: usize, value: u8) -> bool {
    if (0xd820..0xd8f0).contains(&at) {
        let offset = at - 0xd820;
        *flag_projection!(s,offset,(&mut)) = value;
        return true;
    }
    if (0xef78..0xf378).contains(&at) {
        let offset = at - 0xef78;
        let word = sprite_projection!(s,offset,(&mut));
        let mut bytes = word.to_le_bytes();
        bytes[offset % 2] = value;
        *word = u16::from_le_bytes(bytes);
        return true;
    }
    if (0xc000..0xc400).contains(&at) {
        let offset = at - 0xc000;
        let colors = if offset < 512 {
            &mut s.visual.colors
        } else {
            &mut s.visual.targets
        };
        let color = &mut colors[(offset % 512) / 2];
        let mut bytes = color.to_le_bytes();
        bytes[offset % 2] = value;
        *color = u16::from_le_bytes(bytes);
        return true;
    }
    if at == 0x60b {
        s.enemy_projectiles.spawn.angle = value;
        return true;
    }
    if (0x16402..0x19602).contains(&at) {
        s.room.terrain.cells_mut()[at - 0x16402].behavior = value;
        return true;
    }
    let word = if (0x10002..0x16402).contains(&at) {
        Some((
            &mut s.room.terrain.cells_mut()[(at - 0x10002) / 2].level,
            (at - 0x10002) % 2,
        ))
    } else if (0xa000..0xc000).contains(&at) {
        let offset = at - 0xa000;
        Some((
            &mut s.room.terrain.tiles_mut()[offset / 8][offset % 8 / 2],
            offset % 2,
        ))
    } else {
        None
    };
    if let Some((word, lane)) = word {
        let mut bytes = word.to_le_bytes();
        bytes[lane] = value;
        *word = u16::from_le_bytes(bytes);
        return true;
    }
    if (0xdaa..0xdba).contains(&at) {
        let index = (at - 0xdaa) / 4;
        let lane = (at - 0xdaa) % 4;
        let shift = if lane < 2 {
            16 + lane * 8
        } else {
            (lane - 2) * 8
        };
        let old = s.projectiles.launch.displacements[index] as u32;
        s.projectiles.launch.displacements[index] =
            ((old & !(0xff << shift)) | (u32::from(value) << shift)) as i32;
        return true;
    }
    let mut changed = false;
    let mut write = |base: usize, word: &mut u16| {
        if at == base || at == base + 1 {
            let mut bytes = word.to_le_bytes();
            bytes[at - base] = value;
            *word = u16::from_le_bytes(bytes);
            changed = true;
        }
    };
    scalar_words!(s,write,(&mut));
    changed
}
pub(crate) fn inputs(bytes: &[u8]) -> ExternalInputs {
    let w = |at| read(bytes, at);
    ExternalInputs {
        debug_enemy_freeze: w(ram::debug_time_frozen_for_enemies),
        enemy_zero_health: u16::from_ne_bytes([bytes[0xf8c], bytes[0xf8d]]),
        frame_handler_beta: w(0xa44),
        blending_config: w(0x1982),
        player_animation_frame: w(0xa96),
        demo_input_pre_action: w(0xa7a),
        lava_y: w(0x1962),
        player_x_radius: w(ram::samus_x_radius),
        current_enemy: w(ram::cur_enemy_index),
        random: w(ram::random_number),
        random_enemy_counter: w(ram::random_enemy_counter),
        fx_y: w(ram::fx_y_pos),
        drop_origin_x: w(ram::special_death_item_drop_x_origin_pos),
        drop_origin_y: w(ram::special_death_item_drop_y_origin_pos),
        grapple_end_x: w(ram::grapple_beam_end_x_pos),
        grapple_end_y: w(ram::grapple_beam_end_y_pos),
        nmi_counter_low: bytes[0x5b5],

        shoot_button: w(ram::button_config_shoot_x),
        cancel_button: w(ram::button_config_itemcancel_y),
        switch_button: w(ram::button_config_itemswitch),
        run_button: w(ram::button_config_run_b),
        held: w(ram::joypad1_lastkeys),
        pressed: w(ram::joypad1_newkeys),
        filtered_held: w(ram::joypad1_input_samusfilter),
        filtered_pressed: w(ram::joypad1_newinput_samusfilter),
        hyper: w(ram::hyper_beam_flag),
        frozen: w(ram::time_is_frozen_flag),
        cinematic: w(ram::cinematic_function),
        hud_auto_cancel: w(ram::hud_auto_cancel_flag),
        door_definition: w(ram::door_def_ptr),
        door_direction: w(ram::door_direction),
        background_page_x: w(0x91d),
        power_bomb_active: w(0x592),
        camera_x: w(ram::layer1_x_pos),
        camera_y: w(ram::layer1_y_pos),
        frame: w(ram::nmi_frame_counter_word),
        ceres: w(ram::ceres_status),
        m7_x: w(ram::reg_M7X),
        m7_y: w(ram::reg_M7Y),
        m7_a: w(ram::reg_M7A),
        m7_b: w(ram::reg_M7B),
        m7_c: w(ram::reg_M7C),
        grapple_x: w(ram::x_pos_of_start_of_grapple_beam_prevframe),
        grapple_y: w(ram::y_pos_of_start_of_grapple_beam_prevframe),
        absolute_y_fraction: w(0xda8),
    }
}
