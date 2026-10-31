//! Prepared external edges for the actual enemy core.
#![allow(unused_variables)]
use super::game::{enemy_projectiles::host::*, session::Vector32};
use super::{Composed, Event};
impl ImportedAliases for Composed {
    fn enemy_word(&self, member: EnemyMember, signed_slot: isize) -> u16 {
        panic!("unprepared enemy operation: enemy_word")
    }
    fn set_enemy_word(&mut self, member: EnemyMember, signed_slot: isize, value: u16) {
        panic!("unprepared enemy operation: set_enemy_word")
    }
    fn enemy_byte(&self, member: EnemyMember, byte_index: isize) -> u8 {
        panic!("unprepared enemy operation: enemy_byte")
    }
    fn set_enemy_byte(&mut self, member: EnemyMember, byte_index: isize, value: u8) {
        self.events
            .push(Event::EnemyAlias(member, byte_index, value));
    }
    fn shot_word(&self, member: ShotMember, signed_slot: isize) -> u16 {
        panic!("unprepared enemy operation: shot_word")
    }
    fn set_shot_direction(&mut self, signed_slot: isize, value: u16) {
        panic!("unprepared enemy operation: set_shot_direction")
    }
    fn mechanism_word(&self, member: MechanismMember, signed_slot: isize) -> u16 {
        panic!("unprepared enemy operation: mechanism_word")
    }
    fn terrain_word(&self, block_index: u16) -> u16 {
        panic!("unprepared enemy operation: terrain_word")
    }
    fn behavior_byte(&self, block_index: u16) -> u8 {
        panic!("unprepared enemy operation: behavior_byte")
    }
}
impl SpeciesActors for Composed {
    fn mini_kraid_spit_x_velocity(&self, index: u16) -> u16 {
        panic!("unprepared enemy operation: mini_kraid_spit_x_velocity")
    }
    fn mini_kraid_spit_y_velocity(&self, index: u16) -> u16 {
        panic!("unprepared enemy operation: mini_kraid_spit_y_velocity")
    }
    fn mini_kraid_spit_x_offset(&self, index: u16) -> u16 {
        panic!("unprepared enemy operation: mini_kraid_spit_x_offset")
    }
    fn boss_death_quake_sound_timer(&self) -> u16 {
        panic!("unprepared enemy operation: kraid_unk9000")
    }
    fn fire_arc_part_offset(&self, index: u16) -> u16 {
        panic!("unprepared enemy operation: nuclear_waffle_part_offset")
    }
    fn enemy_projectile_origin_x(&self, index: u16) -> u16 {
        panic!("unprepared enemy operation: enemy_projectile_origin_x")
    }
    fn enemy_projectile_origin_y(&self, index: u16) -> u16 {
        panic!("unprepared enemy operation: enemy_projectile_origin_y")
    }
    fn maw_body_part_index(&self, index: u16) -> u16 {
        panic!("unprepared enemy operation: maw_body_part_index")
    }
    fn kraid_health_eighth(&self, index: u16) -> u16 {
        panic!("unprepared enemy operation: kraid_health_eighth")
    }
    fn mini_kraid_spike_variant(&self, index: u16) -> u16 {
        panic!("unprepared enemy operation: mini_kraid_spike_variant")
    }
    fn set_enemy_linked_projectile(&mut self, index: u16, value: u16) {
        panic!("unprepared enemy operation: set_enemy_linked_projectile")
    }
    fn set_botwoon_body_active(&mut self, index: u16, value: u16) {
        panic!("unprepared enemy operation: set_botwoon_body_active")
    }
    fn set_maw_body_link(&mut self, index: u16, value: u16) {
        panic!("unprepared enemy operation: set_maw_body_link")
    }
    fn mother_brain_active_bombs(&self) -> u16 {
        panic!("unprepared enemy operation: mother_brain_active_bombs")
    }
    fn mother_brain_blue_ring_contacts(&self) -> u16 {
        panic!("unprepared enemy operation: mother_brain_blue_ring_contacts")
    }
    fn mother_brain_target_enemy(&self) -> u16 {
        panic!("unprepared enemy operation: mother_brain_target_enemy")
    }
    fn mother_brain_turret_destroyed(&self) -> u16 {
        panic!("unprepared enemy operation: mother_brain_turret_destroyed")
    }
    fn botwoon_death_phase(&self) -> u16 {
        panic!("unprepared enemy operation: botwoon_death_phase")
    }
    fn ridley_fireball_x_velocity(&self) -> u16 {
        panic!("unprepared enemy operation: ridley_fireball_x_velocity")
    }
    fn ridley_fireball_y_velocity(&self) -> u16 {
        panic!("unprepared enemy operation: ridley_fireball_y_velocity")
    }
    fn set_mother_brain_active_bombs(&mut self, value: u16) {
        panic!("unprepared enemy operation: set_mother_brain_active_bombs")
    }
    fn set_mother_brain_blue_ring_contacts(&mut self, value: u16) {
        panic!("unprepared enemy operation: set_mother_brain_blue_ring_contacts")
    }
    fn set_mother_brain_hurt_frame(&mut self, value: u16) {
        panic!("unprepared enemy operation: set_mother_brain_hurt_frame")
    }
    fn set_mother_brain_purple_breath_active(&mut self, value: u16) {
        panic!("unprepared enemy operation: set_mother_brain_purple_breath_active")
    }
    fn mother_brain_beam_x_fraction(&self) -> u16 {
        panic!("unprepared enemy operation: mother_brain_beam_x_fraction")
    }
    fn set_mother_brain_beam_x_fraction(&mut self, value: u16) {
        panic!("unprepared enemy operation: set_mother_brain_beam_x_fraction")
    }
    fn mother_brain_beam_x(&self) -> u16 {
        panic!("unprepared enemy operation: mother_brain_beam_x")
    }
    fn set_mother_brain_beam_x(&mut self, value: u16) {
        panic!("unprepared enemy operation: set_mother_brain_beam_x")
    }
    fn mother_brain_beam_y_fraction(&self) -> u16 {
        panic!("unprepared enemy operation: mother_brain_beam_y_fraction")
    }
    fn set_mother_brain_beam_y_fraction(&mut self, value: u16) {
        panic!("unprepared enemy operation: set_mother_brain_beam_y_fraction")
    }
    fn mother_brain_beam_y(&self) -> u16 {
        panic!("unprepared enemy operation: mother_brain_beam_y")
    }
    fn set_mother_brain_beam_y(&mut self, value: u16) {
        panic!("unprepared enemy operation: set_mother_brain_beam_y")
    }
    fn mother_brain_beam_velocity_x(&self) -> u16 {
        panic!("unprepared enemy operation: mother_brain_beam_velocity_x")
    }
    fn set_mother_brain_beam_velocity_x(&mut self, value: u16) {
        panic!("unprepared enemy operation: set_mother_brain_beam_velocity_x")
    }
    fn mother_brain_beam_velocity_y(&self) -> u16 {
        panic!("unprepared enemy operation: mother_brain_beam_velocity_y")
    }
    fn set_mother_brain_beam_velocity_y(&mut self, value: u16) {
        panic!("unprepared enemy operation: set_mother_brain_beam_velocity_y")
    }
    fn mother_brain_beam_angle(&self) -> u16 {
        panic!("unprepared enemy operation: mother_brain_beam_angle")
    }
    fn set_mother_brain_beam_angle(&mut self, value: u16) {
        panic!("unprepared enemy operation: set_mother_brain_beam_angle")
    }
    fn set_enemy_extra_health_fraction(&mut self, index: u16, element: usize, value: u16) {
        panic!("unprepared enemy operation: set_enemy_extra_health_fraction")
    }
    fn set_botwoon_body_landed(&mut self, value: u16) {
        panic!("unprepared enemy operation: set_botwoon_body_landed")
    }
    fn spore_stem_anchor_x(&self) -> u16 {
        panic!("unprepared enemy operation: spore_stem_anchor_x")
    }
    fn spore_stem_anchor_y(&self) -> u16 {
        panic!("unprepared enemy operation: spore_stem_anchor_y")
    }
    fn set_spore_stem_control_x(&mut self, part: u8, value: u16) {
        panic!("unprepared enemy operation: set_spore_stem_control_x")
    }
    fn set_spore_stem_control_y(&mut self, part: u8, value: u16) {
        panic!("unprepared enemy operation: set_spore_stem_control_y")
    }
    fn maw_segment_handle(&self, enemy: u16, part: u8) -> u16 {
        panic!("unprepared enemy operation: maw_segment_handle")
    }
    fn maw_anchor_x(&self, enemy: u16) -> u16 {
        panic!("unprepared enemy operation: maw_anchor_x")
    }
    fn maw_anchor_y(&self, enemy: u16) -> u16 {
        panic!("unprepared enemy operation: maw_anchor_y")
    }
    fn maw_segment_offset_x(&self, enemy: u16, part: u8) -> u16 {
        panic!("unprepared enemy operation: maw_segment_offset_x")
    }
    fn maw_segment_offset_y(&self, enemy: u16, part: u8) -> u16 {
        panic!("unprepared enemy operation: maw_segment_offset_y")
    }
    fn maw_palette(&self, enemy: u16) -> u16 {
        panic!("unprepared enemy operation: maw_palette")
    }
    fn expanding_square_origin(&self) -> u16 {
        panic!("unprepared enemy operation: expanding_square_origin")
    }
    fn fire_arc_field(&self, enemy: usize, field: FireArcField) -> u16 {
        panic!("unprepared enemy operation: waffle_field")
    }
    fn set_fire_arc_field(&mut self, enemy: usize, field: FireArcField, value: u16) {
        panic!("unprepared enemy operation: set_waffle_field")
    }
    fn botwoon_field(&self, enemy: usize, field: BotwoonField) -> u16 {
        panic!("unprepared enemy operation: botwoon_field")
    }
    fn set_botwoon_field(&mut self, enemy: usize, field: BotwoonField, value: u16) {
        panic!("unprepared enemy operation: set_botwoon_field")
    }

    fn botwoon_history_x(&self, cursor: u16) -> u16 {
        panic!("unprepared enemy operation: botwoon_history_x")
    }
    fn set_botwoon_history_x(&mut self, cursor: u16, value: u16) {
        panic!("unprepared enemy operation: set_botwoon_history_x")
    }
    fn botwoon_history_y(&self, cursor: u16) -> u16 {
        panic!("unprepared enemy operation: botwoon_history_y")
    }
    fn set_botwoon_history_y(&mut self, cursor: u16, value: u16) {
        panic!("unprepared enemy operation: set_botwoon_history_y")
    }
}
impl Effects for Composed {
    fn next_random(&mut self) -> u16 {
        self.events.push(Event::EnemyRandom);
        if let Some(callback) = self.on_enemy_random {
            callback(self);
        }
        let result = self.enemy_random.pop_front().expect("unprepared enemy RNG");
        self.world.player_inputs.random = result;
        result
    }
    fn grapple_pickup_query(&mut self) -> u16 {
        self.events.push(Event::EnemyGrapple);
        0
    }
    fn resume_player_after_elevator(&mut self) -> u16 {
        panic!("unprepared enemy operation: resume_player_after_elevator")
    }
    fn calculate_angle_from_xy(&mut self, x: u16, y: u16) -> u16 {
        panic!("unprepared enemy operation: calculate_angle_from_xy")
    }
    fn damage_player(&mut self, damage: u16) -> () {
        assert_eq!(damage, 10, "1054 actual suit damage argument");
        panic!("unprepared enemy operation: damage_player")
    }

    fn ridley_offscreen(&mut self, x: u16, y: u16) -> u8 {
        panic!("unprepared enemy operation: ridley_offscreen")
    }
    fn queue_music(&mut self, track: u16) -> () {
        panic!("unprepared enemy operation: queue_music")
    }
    fn convert_angle_to_xy(&mut self, angle: u16, amplitude: u16) -> Vector32 {
        panic!("unprepared enemy operation: convert_angle_to_xy")
    }
    fn queue_sound(&mut self, channel: u8, max: u8, sound: u16) {
        self.events.push(Event::EnemySound(channel, max, sound));
        if let Some(callback) = self.on_enemy_sound {
            callback(self);
        }
    }
    fn unreachable(&mut self) {
        panic!("unprepared enemy operation: unreachable")
    }
    fn draw_spritemap(
        &mut self,
        offscreen: bool,
        bank: u8,
        map: u16,
        position: sm_game::geometry::Point,
        tile: u16,
        graphics: u16,
    ) {
        panic!("unprepared enemy operation: draw_spritemap")
    }
    fn respawn_enemy(&mut self, index: u16) {
        panic!("unprepared enemy operation: respawn_enemy")
    }
    fn reset_room_enemy_workspace(&mut self) {
        panic!("unprepared enemy operation: reset_room_enemy_workspace")
    }
}
impl Graphics for Composed {
    fn vram_write_queue_tail(&self) -> u16 {
        panic!("unprepared enemy operation: vram_write_queue_tail")
    }
    fn set_vram_write_queue_tail(&mut self, value: u16) {
        panic!("unprepared enemy operation: set_vram_write_queue_tail")
    }
    fn set_vram_queue_size(&mut self, cursor: u16, value: u16) {
        panic!("unprepared enemy operation: set_vram_queue_size")
    }
    fn set_vram_queue_source(&mut self, cursor: u16, value: u16) {
        panic!("unprepared enemy operation: set_vram_queue_source")
    }
    fn set_vram_queue_bank_word(&mut self, cursor: u16, value: u16) {
        panic!("unprepared enemy operation: set_vram_queue_bank_word")
    }
    fn set_vram_queue_destination(&mut self, cursor: u16, value: u16) {
        panic!("unprepared enemy operation: set_vram_queue_destination")
    }
    fn set_bg2_y_scroll(&mut self, value: u16) {
        panic!("unprepared enemy operation: set_bg2_y_scroll")
    }
    fn set_palette_buffer(&mut self, index: isize, value: u16) {
        self.world.player.presentation.visual_mut().colors
            [usize::try_from(index).expect("negative palette index")] = value;
    }
}
impl Host for Composed {}

use super::RomImage;
use super::game::enemy_projectiles::content;
impl Content for Composed {
    type Loaded = content::LoadedRecord;
    type Native = content::NativeRecord;
    fn script_image(&self) -> RomImage {
        self.image.clone()
    }
    fn loaded_record(&mut self, bank: u8, address: u16) -> Self::Loaded {
        assert!(address & 0x8000 != 0, "unprepared enemy low-window record");
        content::LoadedRecord::new(
            self.image.clone(),
            usize::from(bank & 0x7f) * 0x8000 + usize::from(address & 0x7fff),
        )
    }
    fn native_record(&mut self, bank: u8, address: u16) -> Self::Native {
        assert!(
            address & 0x8000 != 0,
            "unprepared enemy low-window native record"
        );
        content::NativeRecord::new(
            self.image.clone(),
            usize::from(bank & 0x7f) * 0x8000 + usize::from(address & 0x7fff),
        )
    }
    fn screen_shake_word(&self, _: isize) -> u16 {
        panic!("unprepared screen shake")
    }
    fn fire_arc_sine_sample(&mut self, _: u8) -> u8 {
        panic!("unprepared waffle sine")
    }
}
