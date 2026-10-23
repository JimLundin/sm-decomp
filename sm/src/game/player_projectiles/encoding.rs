//! Exact compatibility layout, private to the runtime edge.
#![forbid(unsafe_code)]
use super::state::{Inputs, Session};
use crate::ram_map::{word_arrays as arrays, words as ram};

macro_rules! scalar_words {
    ($s:ident,$f:ident,($($borrow:tt)*))=>{
        $f(ram::projectile_counter,$($borrow)* $s.count);
        $f(ram::bomb_counter,$($borrow)* $s.bomb_count);
        $f(ram::projectile_index,$($borrow)* $s.selected);
        $f(ram::cooldown_timer,$($borrow)* $s.controls.cooldown);
        $f(ram::bomb_spread_charge_timeout_counter,$($borrow)* $s.controls.spread_charge);
        $f(ram::used_for_sba_attacksB60,$($borrow)* $s.controls.orbit_step);
        $f(ram::flare_counter,$($borrow)* $s.controls.charge);
        $f(ram::prev_beam_charge_counter,$($borrow)* $s.controls.previous_charge);
        $f(ram::charged_shot_glow_timer,$($borrow)* $s.controls.glow);
        $f(ram::samus_charge_palette_index,$($borrow)* $s.controls.charge_palette);
        $f(ram::play_resume_charging_beam_sfx,$($borrow)* $s.controls.resume_sound);
        $f(ram::samus_echoes_sound_flag,$($borrow)* $s.controls.echo_sound);
        $f(ram::new_projectile_direction_changed_pose,$($borrow)* $s.controls.pending_aim);
        $f(ram::projectile_invincibility_timer,$($borrow)* $s.controls.protection);
        $f(ram::hud_item_index,$($borrow)* $s.controls.selected_weapon);
        $f(ram::samus_auto_cancel_hud_item_index,$($borrow)* $s.controls.automatic_weapon);
        $f(ram::hud_item_changed_this_frame,$($borrow)* $s.controls.weapon_changed);
        $f(ram::samus_x_pos,$($borrow)* $s.player.x);
        $f(ram::samus_y_pos,$($borrow)* $s.player.y);
        $f(ram::samus_missiles,$($borrow)* $s.player.missiles);
        $f(ram::samus_super_missiles,$($borrow)* $s.player.super_missiles);
        $f(ram::samus_power_bombs,$($borrow)* $s.player.power_bombs);
        $f(ram::grapple_beam_function,$($borrow)* $s.player.grapple);
        $f(ram::samus_movement_handler,$($borrow)* $s.player.movement_handler);
        $f(ram::samus_shine_timer,$($borrow)* $s.player.shine_timer);
        $f(ram::timer_for_shine_timer,$($borrow)* $s.player.shine_tick);
        $f(ram::special_samus_palette_frame,$($borrow)* $s.player.palette_frame);
        $f(ram::samus_special_super_palette_flags,$($borrow)* $s.player.special_palette_flags);
        $f(ram::samus_new_pose,$($borrow)* $s.preparation.next_pose);
        $f(ram::samus_new_pose_interrupted,$($borrow)* $s.preparation.interrupted_pose);
        $f(ram::samus_new_pose_transitional,$($borrow)* $s.preparation.transitional_pose);
        $f(ram::samus_momentum_routine_index,$($borrow)* $s.preparation.momentum_routine);
        $f(ram::samus_special_transgfx_index,$($borrow)* $s.preparation.special_graphics);
        $f(ram::samus_hurt_switch_index,$($borrow)* $s.preparation.hurt_switch);
        $f(ram::controller1_input_for_demo,$($borrow)* $s.preparation.demo_held);
        $f(ram::controller1_new_input_for_demo,$($borrow)* $s.preparation.demo_pressed);
        $f(ram::demo_backup_prev_controller_input,$($borrow)* $s.preparation.demo_filtered_held);
        $f(ram::demo_backup_prev_controller_input_new,$($borrow)* $s.preparation.demo_filtered_pressed);
        $f(ram::samus_anim_frame_skip,$($borrow)* $s.preparation.animation_skip);
        $f(ram::UNUSED_word_7E0DFA,$($borrow)* $s.preparation.aim_history);
        $f(0xa10,$($borrow)* $s.preparation.previous_movement_and_facing);
        $f(ram::power_bomb_flag,$($borrow)* $s.power_bomb.flag);
        $f(ram::power_bomb_explosion_x_pos,$($borrow)* $s.power_bomb.x);
        $f(ram::power_bomb_explosion_y_pos,$($borrow)* $s.power_bomb.y);
        $f(ram::cur_block_index,$($borrow)* $s.room_cursor);
        $f(ram::earthquake_type,$($borrow)* $s.earthquake_kind);
        $f(ram::earthquake_timer,$($borrow)* $s.earthquake_timer);
        $f(ram::speed_echoes_index,$($borrow)* $s.echoes.index);
        for index in 0..3 {
            $f(ram::flare_animation_frame+index*2,$($borrow)* $s.controls.flare_frames[index]);
            $f(ram::flare_animation_timer+index*2,$($borrow)* $s.controls.flare_timers[index]);
        }
        for index in 0..4 {
            $f(arrays::speed_echo_xpos+index*2,$($borrow)* $s.echoes.x[index]);
            $f(arrays::speed_echo_ypos+index*2,$($borrow)* $s.echoes.y[index]);
            $f(arrays::speed_echo_xspeed+index*2,$($borrow)* $s.echoes.x_speed[index]);
        }
        for index in 0..10 {
            $f(0xb64+index*2,$($borrow)* $s.shots[index].x);
            $f(0xb78+index*2,$($borrow)* $s.shots[index].y);
            $f(0xb8c+index*2,$($borrow)* $s.shots[index].x_fraction);
            $f(0xba0+index*2,$($borrow)* $s.shots[index].y_fraction);
            $f(0xbb4+index*2,$($borrow)* $s.shots[index].x_radius);
            $f(0xbc8+index*2,$($borrow)* $s.shots[index].y_radius);
            $f(0xbdc+index*2,$($borrow)* $s.shots[index].x_speed);
            $f(0xbf0+index*2,$($borrow)* $s.shots[index].y_speed);
            $f(0xc04+index*2,$($borrow)* $s.shots[index].direction);
            $f(0xc18+index*2,$($borrow)* $s.shots[index].kind);
            $f(0xc2c+index*2,$($borrow)* $s.shots[index].damage);
            $f(0xc40+index*2,$($borrow)* $s.shots[index].cursor);
            $f(0xc54+index*2,$($borrow)* $s.shots[index].remaining);
            $f(0xc68+index*2,$($borrow)* $s.shots[index].motion);
            $f(0xc7c+index*2,$($borrow)* $s.shots[index].variable);
            $f(0xc90+index*2,$($borrow)* $s.shots[index].trail_time);
            $f(0xca4+index*2,$($borrow)* $s.shots[index].auxiliary);
            $f(0xcb8+index*2,$($borrow)* $s.shots[index].sprite);
        }
        for index in 0..18 {
            $f(0xd658+index*2,$($borrow)* $s.trails[index].left.remaining);
            $f(0xd67c+index*2,$($borrow)* $s.trails[index].right.remaining);
            $f(0xd6a0+index*2,$($borrow)* $s.trails[index].left.cursor);
            $f(0xd6c4+index*2,$($borrow)* $s.trails[index].right.cursor);
            $f(0xd6e8+index*2,$($borrow)* $s.trails[index].left.tile);
            $f(0xd70c+index*2,$($borrow)* $s.trails[index].right.tile);
            $f(0xd730+index*2,$($borrow)* $s.trails[index].left.x);
            $f(0xd754+index*2,$($borrow)* $s.trails[index].right.x);
            $f(0xd778+index*2,$($borrow)* $s.trails[index].left.y);
            $f(0xd79c+index*2,$($borrow)* $s.trails[index].right.y);
        }
    };
}
pub fn read(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}
pub fn decode(bytes: &[u8]) -> Session {
    let mut s = Session::default();
    let mut load = |at: usize, value: &mut u16| *value = read(bytes, at);
    scalar_words!(s,load,(&mut));
    for (index, value) in s.launch.displacements.iter_mut().enumerate() {
        *value = ((u32::from(read(bytes, 0xdaa + index * 4)) << 16)
            | u32::from(read(bytes, 0xdac + index * 4))) as i32;
    }
    s
}
pub fn visit(s: &Session, mut visit: impl FnMut(usize, u16)) {
    let mut emit = |at: usize, value: &u16| visit(at, *value);
    scalar_words!(s,emit,(&));
    for (index, &value) in s.launch.displacements.iter().enumerate() {
        visit(0xdaa + index * 4, ((value as u32) >> 16) as u16);
        visit(0xdac + index * 4, value as u16);
    }
}
pub fn encode(s: &Session, bytes: &mut [u8]) {
    visit(s, |at, value| {
        bytes[at..at + 2].copy_from_slice(&value.to_le_bytes())
    });
}
pub fn projected_byte(s: &Session, at: usize) -> Option<u8> {
    let mut result = None;
    visit(s, |base, value| {
        if at == base || at == base + 1 {
            result = Some(value.to_le_bytes()[at - base]);
        }
    });
    result
}
pub fn inputs(bytes: &[u8]) -> Inputs {
    let w = |at| read(bytes, at);
    Inputs {
        pose: w(ram::samus_pose),
        previous_pose: w(ram::samus_last_different_pose),
        movement_and_facing: w(0xa1e),
        equipped_items: w(ram::equipped_items),
        equipped_beams: w(ram::equipped_beams),
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
        suit_palette: w(ram::samus_suit_palette_index),
        contact_attack: w(ram::samus_contact_damage_index),
        hud_auto_cancel: w(ram::hud_auto_cancel_flag),
        speed_boost: w(ram::speed_boost_counter),
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
        y_acceleration: w(ram::samus_y_accel),
        y_fractional_acceleration: w(ram::samus_y_subaccel),
        absolute_y_fraction: w(0xda8),
        area: w(ram::area_index),
        room_width: w(ram::room_width_in_blocks),
        room_height: w(ram::room_height_in_blocks),
        room_scroll_width: w(ram::room_width_in_scrolls),
        room_scroll_height: w(ram::room_height_in_scrolls),
        room_bytes: w(ram::room_size_in_blocks),
        power_bomb_radius: w(ram::power_bomb_explosion_radius),
    }
}
