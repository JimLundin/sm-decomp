//! Live compatibility fields and explicitly ordered foreign effects.
#![forbid(unsafe_code)]
use super::{compat::Execution, state::SlotField};
use crate::{
    ram_map::{word_arrays as A, words as W},
    types::{Enemy_MotherBrain, EnemyData, Ram7800_Kraid},
};
use core::mem::offset_of;

impl<'g> Execution<'g> {
    pub(super) fn mini_kraid_spit_x_velocity(&self, index: u16) -> u16 {
        self.read_edge(0x7800 + usize::from(index))
    }
    pub(super) fn mini_kraid_spit_y_velocity(&self, index: u16) -> u16 {
        self.read_edge(0x7802 + usize::from(index))
    }
    pub(super) fn mini_kraid_spit_x_offset(&self, index: u16) -> u16 {
        self.read_edge(0x7804 + usize::from(index))
    }

    fn read_edge(&self, address: usize) -> u16 {
        self.owner.as_ref().unwrap().read_le(address)
    }
    fn write_edge(&mut self, address: usize, value: u16) {
        self.owner.as_mut().unwrap().write_slice_le(address, value);
    }
    pub(super) fn player_x(&self) -> u16 {
        self.read_edge(W::samus_x_pos)
    }
    pub(super) fn player_y(&self) -> u16 {
        self.read_edge(W::samus_y_pos)
    }
    pub(super) fn current_enemy(&self) -> u16 {
        self.read_edge(W::cur_enemy_index)
    }
    pub(super) fn area(&self) -> u16 {
        self.read_edge(W::area_index)
    }
    pub(super) fn player_speed_divisor(&self) -> u16 {
        self.read_edge(W::samus_x_speed_divisor)
    }
    pub(super) fn random_number(&self) -> u16 {
        self.read_edge(W::random_number)
    }
    pub(super) fn layer1_x_pos(&self) -> u16 {
        self.read_edge(W::layer1_x_pos)
    }
    pub(super) fn layer1_y_pos(&self) -> u16 {
        self.read_edge(W::layer1_y_pos)
    }
    pub(super) fn fx_y_pos(&self) -> u16 {
        self.read_edge(W::fx_y_pos)
    }
    pub(super) fn earthquake_timer(&self) -> u16 {
        self.read_edge(W::earthquake_timer)
    }
    pub(super) fn nmi_frame(&self) -> u16 {
        self.read_edge(W::nmi_frame_counter_word)
    }
    pub(super) fn random_enemy_counter(&self) -> u16 {
        self.read_edge(W::random_enemy_counter)
    }
    pub(super) fn bitmask(&self) -> u16 {
        self.read_edge(W::bitmask)
    }
    pub(super) fn tourian_entrance_statue_animstate(&self) -> u16 {
        self.read_edge(W::tourian_entrance_statue_animstate)
    }
    pub(super) fn tourian_entrance_statue_finished(&self) -> u16 {
        self.read_edge(W::tourian_entrance_statue_finished)
    }
    pub(super) fn kraid_unk9000(&self) -> u16 {
        self.read_edge(W::kraid_unk9000)
    }
    pub(super) fn player_contact_damage_index(&self) -> u16 {
        self.read_edge(W::samus_contact_damage_index)
    }
    pub(super) fn vram_write_queue_tail(&self) -> u16 {
        self.read_edge(W::vram_write_queue_tail)
    }
    pub(super) fn set_player_y(&mut self, value: u16) {
        self.write_edge(W::samus_y_pos, value);
    }
    pub(super) fn set_player_speed_divisor(&mut self, value: u16) {
        self.write_edge(W::samus_x_speed_divisor, value);
    }
    pub(super) fn set_player_invincibility(&mut self, value: u16) {
        self.write_edge(W::samus_invincibility_timer, value);
    }
    pub(super) fn set_player_knockback(&mut self, value: u16) {
        self.write_edge(W::samus_knockback_timer, value);
    }
    pub(super) fn set_knockback_x_dir(&mut self, value: u16) {
        self.write_edge(W::knockback_x_dir, value);
    }
    pub(super) fn set_earthquake_type(&mut self, value: u16) {
        self.write_edge(W::earthquake_type, value);
    }
    pub(super) fn set_earthquake_timer(&mut self, value: u16) {
        self.write_edge(W::earthquake_timer, value);
    }
    pub(super) fn set_tourian_entrance_statue_finished(&mut self, value: u16) {
        self.write_edge(W::tourian_entrance_statue_finished, value);
    }
    pub(super) fn set_bg2_y_scroll(&mut self, value: u16) {
        self.write_edge(W::bg2_y_scroll, value);
    }
    pub(super) fn set_vram_write_queue_tail(&mut self, value: u16) {
        self.write_edge(W::vram_write_queue_tail, value);
    }
    pub(super) fn enemy_x(&self, index: u16) -> u16 {
        self.read_edge(0xf78 + usize::from(index) + offset_of!(EnemyData, x_pos))
    }
    pub(super) fn enemy_y(&self, index: u16) -> u16 {
        self.read_edge(0xf78 + usize::from(index) + offset_of!(EnemyData, y_pos))
    }
    pub(super) fn enemy_x_fraction(&self, index: u16) -> u16 {
        self.read_edge(0xf78 + usize::from(index) + offset_of!(EnemyData, x_subpos))
    }
    pub(super) fn enemy_y_fraction(&self, index: u16) -> u16 {
        self.read_edge(0xf78 + usize::from(index) + offset_of!(EnemyData, y_subpos))
    }
    pub(super) fn enemy_definition(&self, index: u16) -> u16 {
        self.read_edge(0xf78 + usize::from(index) + offset_of!(EnemyData, enemy_ptr))
    }
    pub(super) fn enemy_width(&self, index: u16) -> u16 {
        self.read_edge(0xf78 + usize::from(index) + offset_of!(EnemyData, x_width))
    }
    pub(super) fn enemy_height(&self, index: u16) -> u16 {
        self.read_edge(0xf78 + usize::from(index) + offset_of!(EnemyData, y_height))
    }
    pub(super) fn enemy_properties(&self, index: u16) -> u16 {
        self.read_edge(0xf78 + usize::from(index) + offset_of!(EnemyData, properties))
    }
    pub(super) fn enemy_parameter_1(&self, index: u16) -> u16 {
        self.read_edge(0xf78 + usize::from(index) + offset_of!(EnemyData, parameter_1))
    }
    pub(super) fn enemy_parameter_2(&self, index: u16) -> u16 {
        self.read_edge(0xf78 + usize::from(index) + offset_of!(EnemyData, parameter_2))
    }
    pub(super) fn enemy_ai_a(&self, index: u16) -> u16 {
        self.read_edge(0xf78 + usize::from(index) + offset_of!(EnemyData, ai_var_A))
    }
    pub(super) fn enemy_ai_d(&self, index: u16) -> u16 {
        self.read_edge(0xf78 + usize::from(index) + offset_of!(EnemyData, ai_var_D))
    }
    pub(super) fn enemy_ai_preinstr(&self, index: u16) -> u16 {
        self.read_edge(0xf78 + usize::from(index) + offset_of!(EnemyData, ai_preinstr))
    }
    pub(super) fn enemy_ai_var_B(&self, index: u16) -> u16 {
        self.read_edge(0xf78 + usize::from(index) + offset_of!(EnemyData, ai_var_B))
    }
    pub(super) fn enemy_flash_timer(&self, index: u16) -> u16 {
        self.read_edge(0xf78 + usize::from(index) + offset_of!(EnemyData, flash_timer))
    }
    pub(super) fn enemy_health(&self, index: u16) -> u16 {
        self.read_edge(0xf78 + usize::from(index) + offset_of!(EnemyData, health))
    }
    pub(super) fn enemy_tile(&self, index: u16) -> u16 {
        self.read_edge(0xf78 + usize::from(index) + offset_of!(EnemyData, vram_tiles_index))
    }
    pub(super) fn enemy_palette(&self, index: u16) -> u16 {
        self.read_edge(0xf78 + usize::from(index) + offset_of!(EnemyData, palette_index))
    }
    pub(super) fn set_enemy_health(&mut self, index: u16, value: u16) {
        self.write_edge(
            0xf78 + usize::from(index) + offset_of!(EnemyData, health),
            value,
        );
    }
    pub(super) fn nuclear_waffle_part_offset(&self, index: u16) -> u16 {
        self.read_edge(0x8000 + usize::from(index) + 20)
    }
    pub(super) fn enemy_projectile_origin_x(&self, index: u16) -> u16 {
        self.read_edge(0x8000 + usize::from(index) + 34)
    }
    pub(super) fn enemy_projectile_origin_y(&self, index: u16) -> u16 {
        self.read_edge(0x8000 + usize::from(index) + 36)
    }
    pub(super) fn maw_body_part_index(&self, index: u16) -> u16 {
        self.read_edge(0x8800 + usize::from(index) + 8)
    }
    pub(super) fn kraid_health_eighth(&self, index: u16) -> u16 {
        self.read_edge(0x7800 + usize::from(index) + offset_of!(Ram7800_Kraid, kraid_healths_8ths))
    }
    pub(super) fn mini_kraid_spike_variant(&self, index: u16) -> u16 {
        self.read_edge(0x7800 + usize::from(index) + offset_of!(Ram7800_Kraid, kraid_healths_8ths))
    }
    pub(super) fn set_enemy_linked_projectile(&mut self, index: u16, value: u16) {
        self.write_edge(
            0x7800 + usize::from(index) + offset_of!(Ram7800_Kraid, kraid_next),
            value,
        );
    }
    pub(super) fn set_botwoon_body_active(&mut self, index: u16, value: u16) {
        self.write_edge(
            0x7800 + usize::from(index) + offset_of!(Ram7800_Kraid, kraid_healths_4ths),
            value,
        );
    }
    pub(super) fn set_maw_body_link(&mut self, index: u16, value: u16) {
        self.write_edge(0x8800 + usize::from(index), value);
    }
    pub(super) fn mother_brain_active_bombs(&self) -> u16 {
        self.read_edge(0x7840 + offset_of!(Ram7800_Kraid, kraid_mouth_flags))
    }
    pub(super) fn mother_brain_blue_ring_contacts(&self) -> u16 {
        self.read_edge(0x7800 + offset_of!(Ram7800_Kraid, field_28))
    }
    pub(super) fn mother_brain_target_enemy(&self) -> u16 {
        self.read_edge(0x7840 + offset_of!(Ram7800_Kraid, kraid_healths_8ths) + 8)
    }
    pub(super) fn mother_brain_turret_destroyed(&self) -> u16 {
        self.read_edge(0x7800 + 0x1d * 2)
    }
    pub(super) fn botwoon_death_phase(&self) -> u16 {
        self.read_edge(0x8000 + 32)
    }
    pub(super) fn ridley_fireball_x_velocity(&self) -> u16 {
        self.read_edge(0x7800 + 0x19 * 2)
    }
    pub(super) fn ridley_fireball_y_velocity(&self) -> u16 {
        self.read_edge(0x7800 + 0x1a * 2)
    }
    pub(super) fn random_shifted_word(&self) -> u16 {
        self.read_edge(0x5e6)
    }
    pub(super) fn set_mother_brain_active_bombs(&mut self, value: u16) {
        self.write_edge(0x7840 + offset_of!(Ram7800_Kraid, kraid_mouth_flags), value);
    }
    pub(super) fn set_mother_brain_blue_ring_contacts(&mut self, value: u16) {
        self.write_edge(0x7800 + offset_of!(Ram7800_Kraid, field_28), value);
    }
    pub(super) fn set_mother_brain_hurt_frame(&mut self, value: u16) {
        self.write_edge(0x7840 + offset_of!(Ram7800_Kraid, kraid_hurt_frame), value);
    }
    pub(super) fn set_mother_brain_purple_breath_active(&mut self, value: u16) {
        self.write_edge(0x7840 + offset_of!(Ram7800_Kraid, kraid_hurt_frame), value);
    }
    pub(super) fn mother_brain_beam_x_fraction(&self) -> u16 {
        self.read_edge(0xf78 + offset_of!(Enemy_MotherBrain, var_23))
    }
    pub(super) fn set_mother_brain_beam_x_fraction(&mut self, value: u16) {
        self.write_edge(0xf78 + offset_of!(Enemy_MotherBrain, var_23), value);
    }
    pub(super) fn mother_brain_beam_x(&self) -> u16 {
        self.read_edge(0xf78 + offset_of!(Enemy_MotherBrain, var_24))
    }
    pub(super) fn set_mother_brain_beam_x(&mut self, value: u16) {
        self.write_edge(0xf78 + offset_of!(Enemy_MotherBrain, var_24), value);
    }
    pub(super) fn mother_brain_beam_y_fraction(&self) -> u16 {
        self.read_edge(0xf78 + offset_of!(Enemy_MotherBrain, var_25))
    }
    pub(super) fn set_mother_brain_beam_y_fraction(&mut self, value: u16) {
        self.write_edge(0xf78 + offset_of!(Enemy_MotherBrain, var_25), value);
    }
    pub(super) fn mother_brain_beam_y(&self) -> u16 {
        self.read_edge(0xf78 + offset_of!(Enemy_MotherBrain, var_26))
    }
    pub(super) fn set_mother_brain_beam_y(&mut self, value: u16) {
        self.write_edge(0xf78 + offset_of!(Enemy_MotherBrain, var_26), value);
    }
    pub(super) fn mother_brain_beam_velocity_x(&self) -> u16 {
        self.read_edge(0xf78 + offset_of!(Enemy_MotherBrain, var_27))
    }
    pub(super) fn set_mother_brain_beam_velocity_x(&mut self, value: u16) {
        self.write_edge(0xf78 + offset_of!(Enemy_MotherBrain, var_27), value);
    }
    pub(super) fn mother_brain_beam_velocity_y(&self) -> u16 {
        self.read_edge(0xf78 + offset_of!(Enemy_MotherBrain, var_28))
    }
    pub(super) fn set_mother_brain_beam_velocity_y(&mut self, value: u16) {
        self.write_edge(0xf78 + offset_of!(Enemy_MotherBrain, var_28), value);
    }
    pub(super) fn mother_brain_beam_angle(&self) -> u16 {
        self.read_edge(0xf78 + offset_of!(Enemy_MotherBrain, var_29))
    }
    pub(super) fn set_mother_brain_beam_angle(&mut self, value: u16) {
        self.write_edge(0xf78 + offset_of!(Enemy_MotherBrain, var_29), value);
    }
    pub(super) fn set_vram_queue_size(&mut self, cursor: u16, value: u16) {
        self.write_edge(0xd0 + usize::from(cursor) + 0, value);
    }
    pub(super) fn set_vram_queue_source(&mut self, cursor: u16, value: u16) {
        self.write_edge(0xd0 + usize::from(cursor) + 2, value);
    }
    pub(super) fn set_vram_queue_bank_word(&mut self, cursor: u16, value: u16) {
        self.write_edge(0xd0 + usize::from(cursor) + 4, value);
    }
    pub(super) fn set_vram_queue_destination(&mut self, cursor: u16, value: u16) {
        self.write_edge(0xd0 + usize::from(cursor) + 5, value);
    }
    pub(super) fn set_enemy_extra_health_fraction(
        &mut self,
        index: u16,
        element: usize,
        value: u16,
    ) {
        self.write_edge(
            0x7800
                + usize::from(index)
                + offset_of!(Ram7800_Kraid, kraid_healths_8ths)
                + element * 2,
            value,
        );
    }
    pub(super) fn nmi_frame_counter_low(&self) -> u8 {
        self.owner.as_ref().unwrap().byte(0x5b5)
    }
    pub(super) fn expanding_square_origin(&self) -> u16 {
        self.read_edge(0x9e00)
    }
    pub(super) fn door_bit_byte(&self, index: isize) -> u8 {
        self.owner
            .as_ref()
            .unwrap()
            .byte(0xd8b0usize.checked_add_signed(index).unwrap())
    }
    pub(super) fn set_palette_buffer(&mut self, index: isize, value: u16) {
        self.write_edge(
            A::palette_buffer
                .checked_add_signed(index.checked_mul(2).unwrap())
                .unwrap(),
            value,
        );
    }
    pub(super) fn set_palette_word(&mut self, index: u16, value: u16) {
        self.set_palette_buffer(isize::try_from(index).unwrap(), value);
    }
    pub(super) fn add_split(&mut self, high: SlotField, low: SlotField, index: isize, amount: u32) {
        let low_word = self.slot(low, index);
        let high_word = self.slot(high, index);
        let sum = ((u32::from(high_word) << 16) | u32::from(low_word)).wrapping_add(amount);
        self.set_slot(low, index, sum as u16);
        self.set_slot(high, index, (sum >> 16) as u16);
        if amount != 0 && matches!(high, SlotField::X | SlotField::Y) {
            crate::verify_probe::hit("eproj.motion.applied");
        }
    }
    pub(super) fn angle_to_player(&mut self, handle: u16) -> u16 {
        let x = self
            .player_x()
            .wrapping_sub(self.slot(SlotField::X, (handle >> 1) as isize));
        let y = self
            .player_y()
            .wrapping_sub(self.slot(SlotField::Y, (handle >> 1) as isize));
        self.calculate_angle_from_xy(x, y)
    }
}
impl Execution<'_> {
    pub(super) fn special_death_item_drop_x_origin_pos(&self) -> u16 {
        self.read_edge(W::special_death_item_drop_x_origin_pos)
    }
    pub(super) fn special_death_item_drop_y_origin_pos(&self) -> u16 {
        self.read_edge(W::special_death_item_drop_y_origin_pos)
    }
    pub(super) fn samus_reserve_health(&self) -> u16 {
        self.read_edge(W::samus_reserve_health)
    }
    pub(super) fn samus_health(&self) -> u16 {
        self.read_edge(W::samus_health)
    }
    pub(super) fn samus_max_health(&self) -> u16 {
        self.read_edge(W::samus_max_health)
    }
    pub(super) fn samus_max_reserve_health(&self) -> u16 {
        self.read_edge(W::samus_max_reserve_health)
    }
    pub(super) fn samus_missiles(&self) -> u16 {
        self.read_edge(W::samus_missiles)
    }
    pub(super) fn samus_max_missiles(&self) -> u16 {
        self.read_edge(W::samus_max_missiles)
    }
    pub(super) fn samus_super_missiles(&self) -> u16 {
        self.read_edge(W::samus_super_missiles)
    }
    pub(super) fn samus_max_super_missiles(&self) -> u16 {
        self.read_edge(W::samus_max_super_missiles)
    }
    pub(super) fn samus_power_bombs(&self) -> u16 {
        self.read_edge(W::samus_power_bombs)
    }
    pub(super) fn samus_max_power_bombs(&self) -> u16 {
        self.read_edge(W::samus_max_power_bombs)
    }
    pub(super) fn health_drop_bias_flag(&self) -> u16 {
        self.read_edge(W::health_drop_bias_flag)
    }
    pub(super) fn grapple_beam_end_x_pos(&self) -> u16 {
        self.read_edge(W::grapple_beam_end_x_pos)
    }
    pub(super) fn grapple_beam_end_y_pos(&self) -> u16 {
        self.read_edge(W::grapple_beam_end_y_pos)
    }
    pub(super) fn earthquake_type(&self) -> u16 {
        self.read_edge(W::earthquake_type)
    }
    pub(super) fn time_frozen(&self) -> u16 {
        self.read_edge(W::time_is_frozen_flag)
    }
    pub(super) fn set_health_drop_bias_flag(&mut self, value: u16) {
        self.write_edge(W::health_drop_bias_flag, value);
    }
    pub(super) fn next_random(&mut self) -> u16 {
        self.foreign(|game| super::legacy::next_random(game))
    }
    pub(super) fn call_player_code(&mut self, code: u16) -> u16 {
        self.foreign(|game| super::legacy::call_player_code(code, game))
    }
    pub(super) fn prepare_bit_access(&mut self, value: u16) -> u16 {
        self.foreign(|game| crate::game::progress_flags::PrepareBitAccess(value, game))
    }
    pub(super) fn calculate_angle_from_xy(&mut self, x: u16, y: u16) -> u16 {
        self.foreign(|game| super::legacy::calculate_angle_from_xy(x, y, game))
    }
    pub(super) fn reduce_suit_damage(&mut self, damage: u16) -> u16 {
        self.foreign(|game| crate::game::enemy_combat::reduce_suit_damage(damage, game))
    }
    pub(super) fn damage_player(&mut self, damage: u16) -> () {
        self.foreign(|game| super::legacy::damage_player(damage, game))
    }
    pub(super) fn create_sprite(&mut self, x: u16, y: u16, id: u16, graphics: u16) -> u16 {
        self.foreign(|game| super::legacy::create_sprite(x, y, id, graphics, game))
    }
    pub(super) fn ridley_offscreen(&mut self, x: u16, y: u16) -> u8 {
        self.foreign(|game| super::legacy::ridley_offscreen(x, y, game))
    }
    pub(super) fn queue_music(&mut self, track: u16) -> () {
        self.foreign(|game| crate::game::music::QueueMusic_Delayed8(track, game))
    }
    pub(super) fn convert_angle_to_xy(
        &mut self,
        angle: u16,
        amplitude: u16,
    ) -> crate::types::Point32 {
        self.foreign(|game| crate::game::sm_a0::ConvertAngleToXy(angle, amplitude, game))
    }
    pub(super) fn restore_health(&mut self, amount: u16) -> () {
        self.foreign(|game| crate::game::sm_91::Samus_RestoreHealth(amount, game))
    }
    pub(super) fn restore_missiles(&mut self, amount: u16) -> () {
        self.foreign(|game| crate::game::sm_91::Samus_RestoreMissiles(amount, game))
    }
    pub(super) fn restore_super_missiles(&mut self, amount: u16) -> () {
        self.foreign(|game| crate::game::sm_91::Samus_RestoreSuperMissiles(amount, game))
    }
    pub(super) fn restore_power_bombs(&mut self, amount: u16) -> () {
        self.foreign(|game| crate::game::sm_91::Samus_RestorePowerBombs(amount, game))
    }
    pub(super) fn grapple_pickup_query(&mut self) -> u16 {
        self.call_player_code(0xd)
    }
    pub(super) fn queue_sound(&mut self, channel: u8, max: u8, sound: u16) {
        self.foreign(|game| {
            if !(1..=3).contains(&channel) || !matches!(max, 1 | 3 | 6 | 9 | 15) {
                unreachable!("invalid projectile sound channel/limit");
            }
            crate::game::sm_80::queue_sfx(usize::from(channel - 1), sound as u8, max, game);
        });
    }
    pub(super) fn unreachable(&mut self) {
        self.foreign(super::legacy::unreachable);
    }
    pub(super) fn draw_spritemap(
        &mut self,
        offscreen: bool,
        bank: u8,
        map: u16,
        x: u16,
        y: u16,
        tile: u16,
        graphics: u16,
    ) {
        self.foreign(|game| {
            super::legacy::draw_spritemap(offscreen, bank, map, x, y, tile, graphics, game)
        });
        crate::verify_probe::hit("eproj.drawn");
    }
    pub(super) fn raw_record(&mut self, bank: u8, cursor: u16) -> super::content::NativeRecord {
        self.foreign(|game| super::legacy::raw_record(bank, cursor, game))
    }
    pub(super) fn raw_rom_word(&mut self, bank: u8, cursor: u16) -> u16 {
        self.raw_record(bank, cursor).word(0)
    }
    pub(super) fn raw_enemy_definition_properties(&mut self, definition: u16) -> u16 {
        self.raw_record(0xa0, definition).word(3)
    }
    pub(super) fn raw_turret_instruction(&mut self, index: isize) -> u16 {
        self.raw_record(0x86, 0xc040).word(index)
    }
    pub(super) fn raw_spore_delta(&mut self, index: isize) -> u8 {
        self.raw_record(0x86, 0xdd6c).byte(index)
    }
    pub(super) fn raw_lavaquake_delta(&mut self, index: isize) -> u32 {
        self.raw_record(0xa0, 0xcbc7)
            .long(index.checked_mul(4).unwrap())
    }
    pub(super) fn raw_slope_height(&mut self, index: u16) -> u8 {
        self.raw_record(0x94, 0x8b2b)
            .byte(isize::try_from(index).unwrap())
    }
}

impl Execution<'_> {
    pub(super) fn current_block(&self) -> u16 {
        self.read_edge(W::cur_block_index)
    }
    pub(super) fn room_width_in_blocks(&self) -> u16 {
        self.read_edge(W::room_width_in_blocks)
    }
    pub(super) fn power_bomb_radius(&self) -> u16 {
        self.read_edge(W::power_bomb_explosion_radius)
    }
    pub(super) fn power_bomb_x(&self) -> u16 {
        self.read_edge(W::power_bomb_explosion_x_pos)
    }
    pub(super) fn power_bomb_y(&self) -> u16 {
        self.read_edge(W::power_bomb_explosion_y_pos)
    }
    pub(super) fn player_invincibility(&self) -> u16 {
        self.read_edge(W::samus_invincibility_timer)
    }
    pub(super) fn player_x_radius(&self) -> u16 {
        self.read_edge(W::samus_x_radius)
    }
    pub(super) fn player_y_radius(&self) -> u16 {
        self.read_edge(W::samus_y_radius)
    }
    pub(super) fn player_projectile_count(&self) -> u16 {
        self.read_edge(W::projectile_counter)
    }
    pub(super) fn set_current_block(&mut self, value: u16) {
        self.write_edge(W::cur_block_index, value);
    }
    pub(super) fn set_current_slope_bts(&mut self, value: u16) {
        self.write_edge(W::current_slope_bts, value);
    }
    pub(super) fn set_enemy_processing_stage(&mut self, value: u16) {
        self.write_edge(W::enemy_processing_stage, value);
    }
    pub(super) fn set_collision_detection_index(&mut self, value: u16) {
        self.write_edge(W::collision_detection_index, value);
    }
    pub(super) fn player_projectile_type(&self, index: u16) -> u16 {
        self.read_edge(A::projectile_type + usize::from(index) * 2)
    }
    pub(super) fn player_projectile_direction(&self, index: u16) -> u16 {
        self.read_edge(A::projectile_dir + usize::from(index) * 2)
    }
    pub(super) fn set_player_projectile_direction(&mut self, index: u16, value: u16) {
        self.write_edge(A::projectile_dir + usize::from(index) * 2, value);
    }
    pub(super) fn block_bts(&self, block: u16) -> u8 {
        self.owner
            .as_ref()
            .unwrap()
            .byte(0x16402 + usize::from(block))
    }
    pub(super) fn current_level_block(&self) -> u16 {
        self.read_edge(0x10002 + usize::from(self.current_block()) * 2)
    }
    pub(super) fn power_bomb_radius_high(&self) -> u8 {
        self.owner.as_ref().unwrap().byte(0xceb)
    }
}

impl Execution<'_> {
    pub(super) fn respawn_enemy(&mut self, index: u16) {
        self.foreign(|game| super::legacy::respawn_enemy(index, game));
    }
    pub(super) fn reset_room_enemy_workspace(&mut self) {
        self.foreign(|game| game.g_ram[0x7000..0x9800].fill(0));
    }
}

impl Execution<'_> {
    pub(super) fn player_projectile_variables(&self, index: isize) -> u16 {
        self.read_edge(
            A::projectile_variables
                .checked_add_signed(index.checked_mul(2).unwrap())
                .unwrap(),
        )
    }
    pub(super) fn player_projectile_x_pos(&self, index: isize) -> u16 {
        self.read_edge(
            A::projectile_x_pos
                .checked_add_signed(index.checked_mul(2).unwrap())
                .unwrap(),
        )
    }
    pub(super) fn player_projectile_y_pos(&self, index: isize) -> u16 {
        self.read_edge(
            A::projectile_y_pos
                .checked_add_signed(index.checked_mul(2).unwrap())
                .unwrap(),
        )
    }
    pub(super) fn player_projectile_x_radius(&self, index: isize) -> u16 {
        self.read_edge(
            A::projectile_x_radius
                .checked_add_signed(index.checked_mul(2).unwrap())
                .unwrap(),
        )
    }
    pub(super) fn player_projectile_y_radius(&self, index: isize) -> u16 {
        self.read_edge(
            A::projectile_y_radius
                .checked_add_signed(index.checked_mul(2).unwrap())
                .unwrap(),
        )
    }
    pub(super) fn bomb_counter(&self) -> u16 {
        self.read_edge(W::bomb_counter)
    }
    pub(super) fn raw_turret_direction(&mut self, cursor: u16, index: isize) -> u8 {
        self.raw_record(0x86, cursor).byte(index)
    }
    pub(super) fn set_botwoon_body_landed(&mut self, value: u16) {
        self.write_edge(0x883e, value);
    }
}

pub(super) mod room_mechanisms {
    use super::Execution;
    pub(crate) fn selected(ctx: &mut Execution<'_>) -> u16 {
        ctx.foreign(crate::game::room_mechanisms::selected)
    }
    pub(crate) fn block_x(ctx: &mut Execution<'_>) -> u16 {
        ctx.foreign(crate::game::room_mechanisms::block_x)
    }
    pub(crate) fn block_y(ctx: &mut Execution<'_>) -> u16 {
        ctx.foreign(crate::game::room_mechanisms::block_y)
    }
    pub(crate) fn block_index(handle: u16, ctx: &mut Execution<'_>) -> u16 {
        ctx.foreign(|game| crate::game::room_mechanisms::block_index(handle, game))
    }
    pub(crate) fn projectile_argument(handle: u16, ctx: &mut Execution<'_>) -> u16 {
        ctx.foreign(|game| crate::game::room_mechanisms::projectile_argument(handle, game))
    }
}
pub(crate) fn calculate_coordinates(handle: u16, ctx: &mut Execution<'_>) {
    ctx.foreign(|game| crate::game::room_mechanisms::calculate_coordinates(handle, game));
}

// Live fields for the complete attached Spore Spawn and Yapping Maw operations.
impl Execution<'_> {
    pub(super) fn spore_stem_anchor_x(&self) -> u16 {
        self.read_edge(0xf78 + offset_of!(crate::game::enemy_layouts::EnemyLayout104, var_04))
    }
    pub(super) fn spore_stem_anchor_y(&self) -> u16 {
        self.read_edge(0xf78 + offset_of!(crate::game::enemy_layouts::EnemyLayout104, var_05))
    }
    pub(super) fn set_spore_stem_control_x(&mut self, part: u8, value: u16) {
        use crate::game::enemy_layouts::EnemyLayout104 as Layout;
        let offset = [
            offset_of!(Layout, var_20),
            offset_of!(Layout, var_21),
            offset_of!(Layout, var_22),
        ][usize::from(part)];
        self.write_edge(0xf78 + offset, value);
    }
    pub(super) fn set_spore_stem_control_y(&mut self, part: u8, value: u16) {
        use crate::game::enemy_layouts::EnemyLayout104 as Layout;
        let offset = [
            offset_of!(Layout, var_23),
            offset_of!(Layout, var_24),
            offset_of!(Layout, var_25),
        ][usize::from(part)];
        self.write_edge(0xf78 + offset, value);
    }
    pub(super) fn maw_segment_handle(&self, enemy: u16, part: u8) -> u16 {
        use crate::game::enemy_layouts::EnemyLayout104 as Layout;
        let offset = [
            offset_of!(Layout, var_40),
            offset_of!(Layout, var_41),
            offset_of!(Layout, var_42),
            offset_of!(Layout, var_43),
        ][usize::from(part)];
        self.read_edge(0xf78 + usize::from(enemy) + offset)
    }
    pub(super) fn maw_anchor_x(&self, enemy: u16) -> u16 {
        self.read_edge(
            0xf78
                + usize::from(enemy)
                + offset_of!(crate::game::enemy_layouts::EnemyLayout104, var_0C),
        )
    }
    pub(super) fn maw_anchor_y(&self, enemy: u16) -> u16 {
        self.read_edge(
            0xf78
                + usize::from(enemy)
                + offset_of!(crate::game::enemy_layouts::EnemyLayout104, var_0D),
        )
    }
    pub(super) fn maw_segment_offset_x(&self, enemy: u16, part: u8) -> u16 {
        use crate::game::enemy_layouts::EnemyLayout104 as Layout;
        let offset = [
            offset_of!(Layout, var_00),
            offset_of!(Layout, var_02),
            offset_of!(Layout, var_04),
        ][usize::from(part).checked_sub(1).unwrap()];
        self.read_edge(0xf78 + usize::from(enemy) + offset)
    }
    pub(super) fn maw_segment_offset_y(&self, enemy: u16, part: u8) -> u16 {
        use crate::game::enemy_layouts::EnemyLayout104 as Layout;
        let offset = [
            offset_of!(Layout, var_01),
            offset_of!(Layout, var_03),
            offset_of!(Layout, var_05),
        ][usize::from(part).checked_sub(1).unwrap()];
        self.read_edge(0xf78 + usize::from(enemy) + offset)
    }
    pub(super) fn maw_palette(&self, enemy: u16) -> u16 {
        self.read_edge(
            0xf78
                + usize::from(enemy)
                + offset_of!(crate::game::enemy_layouts::EnemyLayout104, var_45),
        )
    }
    pub(super) fn maw_frozen_timer(&self, enemy: u16) -> u16 {
        self.read_edge(0xf78 + usize::from(enemy) + offset_of!(EnemyData, frozen_timer))
    }
}
