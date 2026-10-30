//! Sole byte-compatible projection of the migrated aggregate and shared continuations.
#![forbid(unsafe_code)]
use crate::game::player_transitions::state as transition_state;
use crate::game::room_entry::{effects::AssetSlot, world::WorldBinding};
use crate::game::{external_inputs::ExternalInputs, session::Session};
use crate::ram_map::{word_arrays as arrays, words as ram};
use std::sync::Arc;
// Head0 owns only these ten bytes; full/odd accesses still project each lane.
macro_rules! botwoon_death_words {
    ($s:ident, $f:ident, ($($borrow:tt)*)) => {
        $f(0x801e, $($borrow)* $s.botwoon_death.pending_hit);
        $f(0x8020, $($borrow)* $s.botwoon_death.active);
        $f(0x803c, $($borrow)* $s.botwoon_death.tail_ready);
        $f(0x8818, $($borrow)* $s.botwoon_death.previous_health);
        $f(0x883e, $($borrow)* $s.botwoon_death.body_landed);
    };
}
fn botwoon_death_byte(s: &Session, at: usize) -> Option<u8> {
    let mut result = None;
    let mut select = |base: usize, word: &u16| {
        if (base..base + 2).contains(&at) {
            result = Some(word.to_le_bytes()[at - base]);
        }
    };
    botwoon_death_words!(s, select, (&));
    result
}
/// Adopt only this new cohort after a successful native Maw store. Existing
/// owned regions retain their former raw-write/Drop-overwrite behavior.
pub(crate) fn write_botwoon_death_byte(s: &mut Session, at: usize, value: u8) -> bool {
    let mut written = false;
    let mut store = |base: usize, word: &mut u16| {
        if (base..base + 2).contains(&at) {
            let mut bytes = word.to_le_bytes();
            bytes[at - base] = value;
            *word = u16::from_le_bytes(bytes);
            written = true;
        }
    };
    botwoon_death_words!(s, store, (&mut));
    written
}
// Finite typed words share one lossless projection; no raw-word mirrors.
macro_rules! transition_words {
    ($s:ident, $f:ident) => {
        $f!(
            0x0a28,
            $s.player.transitions.ordinary,
            transition_state::PoseRequestWord
        );
        $f!(
            0x0a2a,
            $s.player.transitions.interrupted,
            transition_state::PoseRequestWord
        );
        $f!(
            0x0a2c,
            $s.player.transitions.transitional,
            transition_state::PoseRequestWord
        );
        $f!(
            0x0a2e,
            $s.player.transitions.momentum,
            transition_state::MomentumAction
        );
        $f!(
            0x0a30,
            $s.player.transitions.interruption,
            transition_state::InterruptionAction
        );
        $f!(
            0x0a32,
            $s.player.transitions.hurt,
            transition_state::HurtAction
        );
        $f!(
            0x0a9a,
            $s.player.transitions.animation_skip,
            transition_state::AnimationSkip
        );
        $f!(
            0x0a58,
            $s.player.movement_handler,
            transition_state::MovementMode
        );
        $f!(
            0x0a5c,
            $s.player.transitions.draw_handler,
            transition_state::DrawMode
        );
        $f!(
            0x0a42,
            $s.player.transitions.frame_alpha,
            transition_state::FrameAlphaMode
        );
        $f!(
            0x0a44,
            $s.player.transitions.frame_beta,
            transition_state::FrameBetaMode
        );
        $f!(
            0x0a5a,
            $s.player.transitions.frame_gamma,
            transition_state::FrameGammaMode
        );
        $f!(
            0x0a56,
            $s.player.transitions.bomb_jump,
            transition_state::BombJump
        );
        $f!(
            0x0a60,
            $s.player.transitions.input_mode,
            transition_state::InputMode
        );
        $f!(
            0x0b20,
            $s.player.transitions.bounce,
            transition_state::LandingBounce
        );
    };
}
// Identities have only one mutable representation. Byte writes resolve each
// intermediate key through the same binding; no key mirror is retained.
macro_rules! room_keys {
    ($f:ident) => {
        $f!(0x078d, exit, exit_key, exit_identity);
        $f!(0x079b, room, room_key, room_identity);
        $f!(0x07b5, doors, door_list_key, door_list);
        $f!(0x07bb, state, state_key, state_identity);
    };
}
macro_rules! room_assets {
    ($f:ident) => {
        $f!(0x07bd, level, AssetSlot::Level);
        $f!(0x07c0, tile_table, AssetSlot::TileTable);
        $f!(0x07c3, tiles, AssetSlot::Tiles);
        $f!(0x07c6, palette, AssetSlot::Palette);
    };
}
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
macro_rules! presentation_ref {
    ($s:ident, visual, (&)) => {
        $s.presentation.visual()
    };
    ($s:ident, visual, (&mut)) => {
        $s.presentation.visual_mut()
    };
    ($s:ident, palette, (&)) => {
        $s.presentation.palette()
    };
    ($s:ident, palette, (&mut)) => {
        $s.presentation.palette_mut()
    };
    ($s:ident, atmospheric, (&)) => {
        $s.presentation.atmospheric()
    };
    ($s:ident, atmospheric, (&mut)) => {
        $s.presentation.atmospheric_mut()
    };
}

macro_rules! scalar_words {
    ($s:ident,$f:ident,($($borrow:tt)*))=>{
        $f(0x0da2,$($borrow)* $s.player.camera_distance[0].whole);
        $f(0x0da4,$($borrow)* $s.player.camera_distance[0].fraction);
        $f(0x0da6,$($borrow)* $s.player.camera_distance[1].whole);
        $f(0x0da8,$($borrow)* $s.player.camera_distance[1].fraction);
        $f(0x0a64,$($borrow)* $s.player.grapple_interaction_flags);
        $f(0x0cf4,$($borrow)* $s.player.grapple_flags);
        $f(0x0cf6,$($borrow)* $s.player.grapple_delay);
        $f(0x0cf8,$($borrow)* $s.player.grapple_slow_scrolling);
        $f(0x0d1e,$($borrow)* $s.player.grapple_extension_compatibility.x);
        $f(0x0d20,$($borrow)* $s.player.grapple_extension_compatibility.y);
        $f(0x0d34,$($borrow)* $s.player.grapple_direction);
        $f(0x0d36,$($borrow)* $s.player.grapple_direction_compatibility);
        $f(0x0cfa,$($borrow)* $s.player.grapple_angle);
        $f(0x0cfe,$($borrow)* $s.player.grapple_length);
        $f(0x0d02,$($borrow)* $s.player.grapple_origin_offset.x);
        $f(0x0d04,$($borrow)* $s.player.grapple_origin_offset.y);
        $f(0x0d06,$($borrow)* $s.player.grapple_endpoint_fraction.x);
        $f(0x0d08,$($borrow)* $s.player.grapple_endpoint.x);
        $f(0x0d0a,$($borrow)* $s.player.grapple_endpoint_fraction.y);
        $f(0x0d0e,$($borrow)* $s.player.grapple_offset[0].fraction);
        $f(0x0d10,$($borrow)* $s.player.grapple_offset[0].whole);
        $f(0x0d12,$($borrow)* $s.player.grapple_offset[1].fraction);
        $f(0x0d14,$($borrow)* $s.player.grapple_offset[1].whole);
        $f(0x0d22,$($borrow)* $s.player.grapple_velocity.x);
        $f(0x0d24,$($borrow)* $s.player.grapple_velocity.y);
        for i in 0..16 {
            $f(0x0d42 + 2*i,$($borrow)* $s.player.grapple_animation_delay[i]);
            $f(0x0d62 + 2*i,$($borrow)* $s.player.grapple_animation_program[i]);
        }
        $f(0x0d0c,$($borrow)* $s.player.grapple_endpoint.y);
        $f(0x0791,$($borrow)* $s.room.entry.direction);
        $f(0x0793,$($borrow)* $s.room.entry.elevator_properties);
        $f(0x0795,$($borrow)* $s.room.entry.elevator_interlock);
        $f(0x0797,$($borrow)* $s.room.entry.enemy_interlock);
        $f(0x0799,$($borrow)* $s.room.entry.elevator_direction);
        $f(0x079d,$($borrow)* $s.room.entry.room_index);
        $f(0x07a1,$($borrow)* $s.room.entry.map_x);
        $f(0x07a3,$($borrow)* $s.room.entry.map_y);
        $f(0x07ad,$($borrow)* $s.room.entry.scroll_up);
        $f(0x07af,$($borrow)* $s.room.entry.scroll_down);
        $f(0x07b1,$($borrow)* $s.room.entry.previous_cre);
        $f(0x07b3,$($borrow)* $s.room.entry.cre);
        $f(0x07c9,$($borrow)* $s.room.entry.music_track);
        $f(0x07cb,$($borrow)* $s.room.entry.music_data);
        $f(0x07cd,$($borrow)* $s.room.entry.fx_key);
        $f(0x07cf,$($borrow)* $s.room.entry.enemy_population_key);
        $f(0x07d1,$($borrow)* $s.room.entry.enemy_tilesets_key);
        $f(0x07df,$($borrow)* $s.room.entry.main_code_key);
        $f(0x07e9,$($borrow)* $s.room.entry.scrolling_hook);
        $f(0x0925,$($borrow)* $s.room.entry.frame);
        $f(0x0927,$($borrow)* $s.room.entry.arrival_x);
        $f(0x0929,$($borrow)* $s.room.entry.arrival_y);
        $f(0x092b,$($borrow)* $s.room.entry.speed_fraction);
        $f(0x092d,$($borrow)* $s.room.entry.speed_whole);
        $f(0x092f,$($borrow)* $s.room.entry.elevator_delay);
        $f(0x0931,$($borrow)* $s.room.entry.completion);
        $f(0x099c,$($borrow)* $s.room.entry.transition_selector);
        $f(0x0e16,$($borrow)* $s.room.entry.elevator_flags);
        $f(0x0e18,$($borrow)* $s.room.entry.elevator_status);
        $f(0x00a9,$($borrow)* $s.room.entry.irq_policy);
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
        $f(0xac8,$($borrow)* $s.player.top_sprite);
        $f(0xaca,$($borrow)* $s.player.bottom_sprite);
        $f(0xb04,$($borrow)* $s.player.body_position.x);
        $f(0xb06,$($borrow)* $s.player.body_position.y);
        $f(0xa62,$($borrow)* $s.player.scripted_movement_direction);
        $f(0xa7a,$($borrow)* $s.player.demo.pre_action);
        $f(0xa7c,$($borrow)* $s.player.demo.delay);
        $f(0xa7e,$($borrow)* $s.player.demo.program);
        $f(0xa84,$($borrow)* $s.player.demo.held);
        $f(0xa86,$($borrow)* $s.player.demo.pressed);
        $f(0xa88,$($borrow)* $s.player.demo.enabled);
        $f(0xa8c,$($borrow)* $s.player.demo.previous_held);
        $f(0xa8e,$($borrow)* $s.player.demo.previous_pressed);
        $f(0xa48,$($borrow)* presentation_ref!($s, palette, ($($borrow)*)).hurt_flash);
        $f(0xa72,$($borrow)* presentation_ref!($s, palette, ($($borrow)*)).visor_timer);
        $f(0xa74,$($borrow)* presentation_ref!($s, palette, ($($borrow)*)).suit_index);
        $f(0xa76,$($borrow)* $s.inventory.hyper_beam);
        $f(0xa80,$($borrow)* presentation_ref!($s, palette, ($($borrow)*)).xray_progress);
        $f(0xad0,$($borrow)* presentation_ref!($s, palette, ($($borrow)*)).timer);
        $f(0xddc,$($borrow)* presentation_ref!($s, palette, ($($borrow)*)).suit_beam_speed);
        $f(0xdf0,$($borrow)* presentation_ref!($s, palette, ($($borrow)*)).suit_red_green);
        $f(0xdf2,$($borrow)* presentation_ref!($s, palette, ($($borrow)*)).crystal_flash_delay);
        $f(0xa4e,$($borrow)* $s.player.periodic_damage_fraction);
        $f(0x1e6f,$($borrow)* $s.mechanisms.statue_animation);
        for index in 0..256 {
            $f(0xc000+index*2,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).colors[index]);
        }
        for index in 0..256 {
            $f(0xc200+index*2,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).targets[index]);
        }
        for index in 0..8 {
            $f(0x1e7d+index*2,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).palette_effects.slots[index].definition);
            $f(0x1e8d+index*2,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).palette_effects.slots[index].color_offset);
            $f(0x1ead+index*2,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).palette_effects.slots[index].pre_action);
            $f(0x1ebd+index*2,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).palette_effects.slots[index].program);
            $f(0x1ecd+index*2,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).palette_effects.slots[index].delay);
            $f(0x1edd+index*2,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).palette_effects.slots[index].counter);
        }
        for index in 0..6 {
            $f(0x1ef5+index*2,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).animated_tiles.slots[index].definition);
            $f(0x1f01+index*2,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).animated_tiles.slots[index].counter);
            $f(0x1f0d+index*2,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).animated_tiles.slots[index].program);
            $f(0x1f19+index*2,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).animated_tiles.slots[index].delay);
            $f(0x1f25+index*2,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).animated_tiles.slots[index].source);
            $f(0x1f31+index*2,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).animated_tiles.slots[index].size);
            $f(0x1f3d+index*2,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).animated_tiles.slots[index].vram);
        }
        $f(0x1e79,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).palette_effects.flags);
        $f(0x1e7b,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).palette_effects.selected);
        $f(0x1eed,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).palette_effects.heat);
        $f(0x1eef,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).palette_effects.previous_heat);
        $f(0x1ef1,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).animated_tiles.flags);
        $f(0x1ef3,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).animated_tiles.selected);
        $f(0x1f49,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).animated_tiles.last_instruction);
        $f(0x0a18,$($borrow)* $s.player.transitions.landing_compatibility);
        $f(0x0a52,$($borrow)* $s.player.transitions.knockback);
        $f(0x0a82,$($borrow)* $s.player.transitions.xray_angle);
        $f(0x0a9c,$($borrow)* $s.player.transitions.animation_buffer);
        $f(0x0a9e,$($borrow)* $s.player.transitions.grapple_walljump_timer);
        $f(0x0aa0,$($borrow)* $s.player.transitions.elevator_fade_timer);
        $f(0x0aa2,$($borrow)* $s.player.transitions.shinespark_start_stop_timer);
        $f(0x0aa4,$($borrow)* $s.player.crouch_transition_work);
        $f(0x0ad2,$($borrow)* $s.player.transitions.liquid_physics);
        $f(0x0b1a,$($borrow)* $s.player.transitions.landing_motion_compatibility);
        $f(0x0b22,$($borrow)* $s.player.transitions.falling);
        $f(0x0b2a,$($borrow)* $s.player.transitions.vertical_motion_compatibility);
        $f(0x0b38,$($borrow)* $s.player.transitions.vertical_auxiliary_compatibility);
        for index in 0..4 {
            $f(0x0ad4+index*2,$($borrow)* presentation_ref!($s, atmospheric, ($($borrow)*)).slots[index].remaining);
            $f(0x0adc+index*2,$($borrow)* presentation_ref!($s, atmospheric, ($($borrow)*)).slots[index].x);
            $f(0x0ae4+index*2,$($borrow)* presentation_ref!($s, atmospheric, ($($borrow)*)).slots[index].y);
            $f(0x0aec+index*2,$($borrow)* presentation_ref!($s, atmospheric, ($($borrow)*)).slots[index].frame_and_kind);
        }
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
        $f(0xaa6,$($borrow)* $s.projectiles.controls.cannon_flags);
        $f(ram::arm_cannon_frame,$($borrow)* $s.projectiles.controls.cannon_frame);
        $f(ram::arm_cannon_drawing_mode,$($borrow)* $s.projectiles.controls.cannon_drawing_mode);
        $f(ram::samus_x_pos,$($borrow)* $s.player.x);
        $f(ram::samus_x_subpos,$($borrow)* $s.player.x_fraction);
        $f(ram::samus_y_pos,$($borrow)* $s.player.y);
        $f(ram::samus_y_subpos,$($borrow)* $s.player.y_fraction);
        $f(ram::samus_missiles,$($borrow)* $s.inventory.ammunition[0].current);
        $f(ram::samus_super_missiles,$($borrow)* $s.inventory.ammunition[1].current);
        $f(ram::samus_power_bombs,$($borrow)* $s.inventory.ammunition[2].current);
        $f(ram::grapple_beam_function,$($borrow)* $s.player.grapple);
        $f(ram::samus_shine_timer,$($borrow)* $s.player.shine_timer);
        $f(ram::timer_for_shine_timer,$($borrow)* $s.player.shine_tick);
        $f(ram::special_samus_palette_frame,$($borrow)* $s.player.palette_frame);
        $f(ram::samus_special_super_palette_flags,$($borrow)* $s.player.special_palette_flags);
        $f(ram::controller1_input_for_demo,$($borrow)* $s.projectiles.preparation.demo_held);
        $f(ram::controller1_new_input_for_demo,$($borrow)* $s.projectiles.preparation.demo_pressed);
        $f(ram::demo_backup_prev_controller_input,$($borrow)* $s.projectiles.preparation.demo_filtered_held);
        $f(ram::demo_backup_prev_controller_input_new,$($borrow)* $s.projectiles.preparation.demo_filtered_pressed);
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
        $f(0xe20,$($borrow)* $s.compatibility_enemy_scratch);
        $f(0x9a2,$($borrow)* $s.inventory.equipment.active);
        $f(0x9a4,$($borrow)* $s.inventory.equipment.owned);
        $f(0x9a6,$($borrow)* $s.inventory.beams.active);
        $f(0x9a8,$($borrow)* $s.inventory.beams.owned);
        $f(0x9c2,$($borrow)* $s.inventory.health.current);
        $f(0xa4c,$($borrow)* $s.inventory.health_fraction);
        $f(0x9c4,$($borrow)* $s.inventory.health.maximum);
        $f(0x9c8,$($borrow)* $s.inventory.ammunition[0].maximum);
        $f(0x9cc,$($borrow)* $s.inventory.ammunition[1].maximum);
        $f(0x9d0,$($borrow)* $s.inventory.ammunition[2].maximum);
        $f(0x9d4,$($borrow)* $s.inventory.reserve_capacity);
        $f(0x9c0,$($borrow)* $s.inventory.reserve_mode);
        $f(0xa1c,$($borrow)* $s.player.pose);
        $f(0xa12,$($borrow)* $s.player.previous_health_for_flash);
        $f(0xa24,$($borrow)* $s.player.history.last_different.pose);
        $f(0x5e9,$($borrow)* $s.multiply_scratch.left);
        $f(0x5eb,$($borrow)* $s.multiply_scratch.right);
        $f(0xa20,$($borrow)* $s.player.history.previous.pose);
        $f(0xa22,$($borrow)* $s.player.history.previous.movement_and_facing);
        $f(0xa26,$($borrow)* $s.player.history.last_different.movement_and_facing);
        $f(0xa34,$($borrow)* $s.player.clearance.actor_flags);
        $f(0xa36,$($borrow)* $s.player.clearance.terrain_flags);
        $f(0xa38,$($borrow)* $s.player.clearance.terrain_up);
        $f(0xa3a,$($borrow)* $s.player.clearance.radius_difference);
        $f(0xa3c,$($borrow)* $s.player.clearance.terrain_down);
        $f(0xa3e,$($borrow)* $s.player.clearance.actor_up);
        $f(0xa40,$($borrow)* $s.player.clearance.actor_down);
        $f(0xa46,$($borrow)* $s.player.collision.horizontal_slope_flags);
        $f(0xa94,$($borrow)* $s.player.animation_clock.remaining);
        $f(0xa96,$($borrow)* $s.player.animation_clock.frame);
        $f(0xafe,$($borrow)* $s.player.contact.x_radius);
        $f(0xb10,$($borrow)* $s.player.previous_position.x.whole);
        $f(0xb12,$($borrow)* $s.player.previous_position.x.fraction);
        $f(0xb14,$($borrow)* $s.player.previous_position.y.whole);
        $f(0xb16,$($borrow)* $s.player.previous_position.y.fraction);
        $f(0xb4a,$($borrow)* $s.player.contact.acceleration_mode);
        $f(0xdba,$($borrow)* $s.player.collision.slope_adjusted);
        $f(0xdbc,$($borrow)* $s.player.contact.total_x.whole);
        $f(0xdbe,$($borrow)* $s.player.contact.total_x.fraction);
        $f(0xdc6,$($borrow)* $s.player.collision.pose_input);
        $f(0xdce,$($borrow)* $s.player.collision.solid_actor_flags);
        $f(0xdd0,$($borrow)* $s.player.collision.flags);
        for index in 0..4 {
            $f(0xe04+index*2,$($borrow)* $s.player.collision.grapple_ejection[index]);
        }
        $f(0xdea,$($borrow)* $s.player.transitions.crystal_flash_ammo);
        $f(0xdec,$($borrow)* $s.shared_sequence.stage);
        $f(0xdee,$($borrow)* $s.shared_sequence.value);
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
        $f(0x5f7,$($borrow)* $s.minimap.disabled);
        for row in 0..3 {
            for column in 0..5 {
                $f(0xc608+2*(26+column+32*row),$($borrow)* $s.minimap.tiles[row][column]);
            }
        }
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

pub(crate) fn read(bytes: &impl Fn(usize) -> u8, at: usize) -> u16 {
    u16::from_le_bytes([bytes(at), bytes(at + 1)])
}
pub(crate) fn decode(
    bytes: &impl Fn(usize) -> u8,
    byte_len: usize,
    world: Arc<WorldBinding>,
) -> Session {
    let mut s = Session::with_world(world);
    assert!(0x8f7 <= byte_len, "short minimap import");
    for (index, value) in s.minimap.explored.iter_mut().enumerate() {
        *value = bytes(0x7f7 + index);
    }
    macro_rules! load_key {
        ($at:expr, $field:ident, $get:ident, $set:ident) => {
            s.room.entry.$field = s.world.$set(read(bytes, $at));
        };
    }
    room_keys!(load_key);
    macro_rules! load_transition {
        ($at:expr, $field:expr, $ty:path) => {
            $field = <$ty>::from_raw(read(bytes, $at));
        };
    }
    transition_words!(s, load_transition);
    macro_rules! load_asset {
        ($at:expr, $field:ident, $slot:expr) => {
            assert!($at + 3 <= byte_len, "short room asset import");
            s.room.entry.$field = s
                .world
                .asset($slot, [bytes($at), bytes($at + 1), bytes($at + 2)]);
        };
    }
    room_assets!(load_asset);
    *s.control.primary_mut() = bytes(0x612);
    *s.control.secondary_mut() = bytes(0x613);
    s.room.entry.layer2_x = bytes(0x91b);
    s.room.entry.layer2_y = bytes(0x91c);
    for (base, flags) in [
        (0xd820, s.gameplay_flags.events.as_mut_slice()),
        (0xd828, s.gameplay_flags.bosses_by_area.as_mut_slice()),
        (0xd830, s.gameplay_flags.used_room_objects.as_mut_slice()),
        (0xd870, s.gameplay_flags.collected_locations.as_mut_slice()),
        (0xd8b0, s.gameplay_flags.opened_doors.as_mut_slice()),
    ] {
        let end = base + flags.len();
        assert!(end <= byte_len, "short gameplay flags import");
        for (index, value) in flags.iter_mut().enumerate() {
            *value = bytes(base + index);
        }
    }
    s.enemy_projectiles.spawn.angle = bytes(0x60b);
    let mut load = |at: usize, value: &mut u16| *value = read(bytes, at);
    scalar_words!(s,load,(&mut));
    for (index, value) in s.projectiles.launch.displacements.iter_mut().enumerate() {
        *value = ((u32::from(read(bytes, 0xdaa + index * 4)) << 16)
            | u32::from(read(bytes, 0xdac + index * 4))) as i32;
    }
    for (index, cell) in s.room.terrain.cells_mut().iter_mut().enumerate() {
        cell.level = read(bytes, 0x10002 + index * 2);
        cell.behavior = bytes(0x16402 + index);
    }
    for (index, tile) in s.room.terrain.tiles_mut().iter_mut().enumerate() {
        for (corner, value) in tile.iter_mut().enumerate() {
            *value = read(bytes, 0xa000 + index * 8 + corner * 2);
        }
    }
    sm_game::actors::codec::import_common(&mut s, bytes, byte_len);
    assert!(byte_len >= 0x8840, "short Botwoon death import");
    let mut load = |at, word: &mut u16| *word = read(bytes, at);
    botwoon_death_words!(s, load, (&mut));
    s
}
pub(crate) fn visit(s: &Session, mut visit: impl FnMut(usize, u16)) {
    // These signals formerly published immediately, before a fallible export.
    let mut emit = |at, word: &u16| visit(at, *word);
    botwoon_death_words!(s, emit, (&));
    for (index, pair) in s.minimap.explored.chunks_exact(2).enumerate() {
        visit(0x7f7 + 2 * index, u16::from_le_bytes([pair[0], pair[1]]));
    }
    sm_game::actors::codec::visit_common(s, &mut visit);
    macro_rules! emit_key {
        ($at:expr, $field:ident, $get:ident, $set:ident) => {
            visit($at, s.world.$get(&s.room.entry.$field));
        };
    }
    room_keys!(emit_key);
    macro_rules! emit_transition {
        ($at:expr, $field:expr, $ty:path) => {
            visit($at, $field.to_raw());
        };
    }
    transition_words!(s, emit_transition);
    for at in (0x7bd..0x7c9).step_by(2) {
        visit(
            at,
            u16::from_le_bytes([
                projected_byte(s, at).unwrap(),
                projected_byte(s, at + 1).unwrap(),
            ]),
        );
    }
    visit(
        0x612,
        u16::from_le_bytes([*s.control.primary(), *s.control.secondary()]),
    );
    visit(
        0x91b,
        u16::from_le_bytes([s.room.entry.layer2_x, s.room.entry.layer2_y]),
    );
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
pub(crate) fn encode(s: &Session, mut span: impl FnMut(usize, &[u8])) {
    span(0x60b, &[s.enemy_projectiles.spawn.angle]);
    visit(s, |at, value| span(at, &value.to_le_bytes()));
    for (index, cell) in s.room.terrain.cells().iter().enumerate() {
        span(0x16402 + index, &[cell.behavior]);
    }
    s.presentation.encode_projection(&mut span);
    s.control.encode_projection(span);
}
pub(crate) fn projected_byte(s: &Session, at: usize) -> Option<u8> {
    if let Some(value) = s.control.projected_byte(at) {
        return Some(value);
    }
    if let Some(value) = s.presentation.projected_byte(at) {
        return Some(value);
    }
    if let Some(value) = botwoon_death_byte(s, at) {
        return Some(value);
    }
    if (0x7f7..0x8f7).contains(&at) {
        return Some(s.minimap.explored[at - 0x7f7]);
    }
    if let Some(value) = sm_game::actors::codec::projected_byte(s, at) {
        return Some(value);
    }
    macro_rules! key_byte {
        ($base:expr, $field:ident, $get:ident, $set:ident) => {
            if ($base..$base + 2).contains(&at) {
                return Some(s.world.$get(&s.room.entry.$field).to_le_bytes()[at - $base]);
            }
        };
    }
    room_keys!(key_byte);
    macro_rules! transition_byte {
        ($base:expr, $field:expr, $ty:path) => {
            if ($base..$base + 2).contains(&at) {
                return Some($field.to_raw().to_le_bytes()[at - $base]);
            }
        };
    }
    transition_words!(s, transition_byte);
    macro_rules! asset_byte {
        ($base:expr, $field:ident, $slot:expr) => {
            if ($base..$base + 3).contains(&at) {
                return Some(s.world.asset_bytes(&s.room.entry.$field)[at - $base]);
            }
        };
    }
    room_assets!(asset_byte);
    match at {
        0x612 => return Some(*s.control.primary()),
        0x613 => return Some(*s.control.secondary()),
        0x91b => return Some(s.room.entry.layer2_x),
        0x91c => return Some(s.room.entry.layer2_y),
        _ => {}
    }
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
            &s.presentation.visual().colors
        } else {
            &s.presentation.visual().targets
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
    if s.control.write_projected_byte(at, value) {
        return true;
    }
    if s.presentation.write_projected_byte(at, value) {
        return true;
    }
    if write_botwoon_death_byte(s, at, value) {
        return true;
    }
    if (0x7f7..0x8f7).contains(&at) {
        s.minimap.explored[at - 0x7f7] = value;
        return true;
    }
    if sm_game::actors::codec::write_projected_byte(s, at, value) {
        return true;
    }
    macro_rules! write_key {
        ($base:expr, $field:ident, $get:ident, $set:ident) => {
            if ($base..$base + 2).contains(&at) {
                let mut bytes = s.world.$get(&s.room.entry.$field).to_le_bytes();
                bytes[at - $base] = value;
                s.room.entry.$field = s.world.$set(u16::from_le_bytes(bytes));
                return true;
            }
        };
    }
    room_keys!(write_key);
    macro_rules! write_transition {
        ($base:expr, $field:expr, $ty:path) => {
            if ($base..$base + 2).contains(&at) {
                let mut bytes = $field.to_raw().to_le_bytes();
                bytes[at - $base] = value;
                $field = <$ty>::from_raw(u16::from_le_bytes(bytes));
                return true;
            }
        };
    }
    transition_words!(s, write_transition);
    macro_rules! write_asset {
        ($base:expr, $field:ident, $slot:expr) => {
            if ($base..$base + 3).contains(&at) {
                let mut bytes = s.world.asset_bytes(&s.room.entry.$field);
                bytes[at - $base] = value;
                s.room.entry.$field = s.world.asset($slot, bytes);
                return true;
            }
        };
    }
    room_assets!(write_asset);
    let byte = match at {
        0x612 => Some(s.control.primary_mut()),
        0x613 => Some(s.control.secondary_mut()),
        0x91b => Some(&mut s.room.entry.layer2_x),
        0x91c => Some(&mut s.room.entry.layer2_y),
        _ => None,
    };
    if let Some(byte) = byte {
        *byte = value;
        return true;
    }
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
            &mut s.presentation.visual_mut().colors
        } else {
            &mut s.presentation.visual_mut().targets
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
pub(crate) fn inputs(bytes: &impl Fn(usize) -> u8) -> ExternalInputs {
    let w = |at| read(bytes, at);
    ExternalInputs {
        transitions: crate::game::player_transitions::state::TransitionObservations {
            frame_gamma: transition_state::FrameGammaMode::from_raw(w(0xa5a)),
            debug_invincibility: w(0xde0),
            moonwalk_enabled: w(0x9e4),
            turn_request: w(0xdf8),
            game_state: w(0x998),
            game_time_frames: w(0x9da),
            fx_kind: w(0x196e),
        },
        debug_enemy_freeze: w(ram::debug_time_frozen_for_enemies),
        enemy_zero_health: u16::from_ne_bytes([bytes(0xf8c), bytes(0xf8d)]),
        blending_config: w(0x1982),
        lava_y: w(0x1962),
        current_enemy: w(ram::cur_enemy_index),
        random: w(ram::random_number),
        random_enemy_counter: w(ram::random_enemy_counter),
        fx_y: w(ram::fx_y_pos),
        drop_origin_x: w(ram::special_death_item_drop_x_origin_pos),
        drop_origin_y: w(ram::special_death_item_drop_y_origin_pos),
        nmi_counter_low: bytes(0x5b5),

        shoot_button: w(ram::button_config_shoot_x),
        jump_button: w(ram::button_config_jump_a),
        cancel_button: w(ram::button_config_itemcancel_y),
        switch_button: w(ram::button_config_itemswitch),
        run_button: w(ram::button_config_run_b),
        aim_up_button: w(ram::button_config_aim_up_R),
        aim_down_button: w(ram::button_config_aim_down_L),
        held: w(ram::joypad1_lastkeys),
        pressed: w(ram::joypad1_newkeys),
        filtered_held: w(ram::joypad1_input_samusfilter),
        filtered_pressed: w(ram::joypad1_newinput_samusfilter),
        frozen: w(ram::time_is_frozen_flag),
        cinematic: w(ram::cinematic_function),
        hud_auto_cancel: w(ram::hud_auto_cancel_flag),
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
    }
}
