//! Live foreign adapters over the shared Session projection.
#![forbid(unsafe_code)]
use super::host::{ActorAccess, BotwoonField, WaffleField};
use crate::game::session_runtime::Runtime;
use crate::{
    game::enemy_layouts::{EnemyLayout72, EnemyLayout104},
    ram_map::{word_arrays as A, words as W},
    types::{Enemy_MotherBrain, EnemyData, Ram7800_Kraid},
};
use core::mem::offset_of;
fn waffle_offset(field: WaffleField) -> usize {
    match field {
        WaffleField::Behavior => offset_of!(EnemyLayout72, var_A),
        WaffleField::Reverse => offset_of!(EnemyLayout72, var_E),
        WaffleField::Radius => offset_of!(EnemyLayout72, var_D),
        WaffleField::PhaseFraction => offset_of!(EnemyLayout72, var_20),
        WaffleField::Phase => offset_of!(EnemyLayout72, var_21),
        WaffleField::StartPhase => offset_of!(EnemyLayout72, var_23),
        WaffleField::PhaseSpeedFraction => offset_of!(EnemyLayout72, var_24),
        WaffleField::PhaseSpeed => offset_of!(EnemyLayout72, var_25),
        WaffleField::CenterX => offset_of!(EnemyLayout72, var_26),
        WaffleField::CenterY => offset_of!(EnemyLayout72, var_27),
        WaffleField::PartCursor => offset_of!(EnemyLayout72, var_2A),
        WaffleField::Spacing => offset_of!(EnemyLayout72, var_2B),
        WaffleField::PhaseLead => offset_of!(EnemyLayout72, var_2C),
        WaffleField::EndPhase => offset_of!(EnemyLayout72, var_2E),
        WaffleField::FirstTransition => offset_of!(EnemyLayout72, var_2F),
        WaffleField::SecondTransition => offset_of!(EnemyLayout72, var_30),
        WaffleField::HeadStage => offset_of!(EnemyLayout72, var_33),
        WaffleField::Palette => offset_of!(EnemyLayout72, var_34),
        WaffleField::Projectile => offset_of!(EnemyLayout72, var_00),
        WaffleField::PartStage => offset_of!(EnemyLayout72, var_08),
        WaffleField::Sprite => offset_of!(EnemyLayout72, var_10),
        WaffleField::SpriteStage => offset_of!(EnemyLayout72, var_18),
        WaffleField::X => offset_of!(EnemyLayout72, base.x_pos),
        WaffleField::Y => offset_of!(EnemyLayout72, base.y_pos),
        WaffleField::Properties => offset_of!(EnemyLayout72, base.properties),
    }
}
fn botwoon_offset(field: BotwoonField) -> usize {
    match field {
        BotwoonField::HistoryCursor => offset_of!(EnemyLayout104, var_B),
        BotwoonField::HistorySpacing => offset_of!(EnemyLayout104, var_C),
        BotwoonField::Projectile => offset_of!(EnemyLayout104, var_00),
        BotwoonField::Visible => offset_of!(EnemyLayout104, var_10),
        BotwoonField::TransitionPending => offset_of!(EnemyLayout104, var_35),
        BotwoonField::TransitionCursor => offset_of!(EnemyLayout104, var_36),
        BotwoonField::TailState => offset_of!(EnemyLayout104, var_3E),
        BotwoonField::Pattern => offset_of!(EnemyLayout104, var_33),
        BotwoonField::X => offset_of!(EnemyLayout104, base.x_pos),
        BotwoonField::Y => offset_of!(EnemyLayout104, base.y_pos),
    }
}
impl ActorAccess for Runtime<'_> {
    fn mini_kraid_spit_x_velocity(&self, index: u16) -> u16 {
        self.word(0x7800 + usize::from(index))
    }
    fn mini_kraid_spit_y_velocity(&self, index: u16) -> u16 {
        self.word(0x7802 + usize::from(index))
    }
    fn mini_kraid_spit_x_offset(&self, index: u16) -> u16 {
        self.word(0x7804 + usize::from(index))
    }
    fn kraid_unk9000(&self) -> u16 {
        self.word(W::kraid_unk9000)
    }
    fn enemy_x(&self, index: u16) -> u16 {
        self.word(0xf78 + usize::from(index) + offset_of!(EnemyData, x_pos))
    }
    fn enemy_y(&self, index: u16) -> u16 {
        self.word(0xf78 + usize::from(index) + offset_of!(EnemyData, y_pos))
    }
    fn enemy_x_fraction(&self, index: u16) -> u16 {
        self.word(0xf78 + usize::from(index) + offset_of!(EnemyData, x_subpos))
    }
    fn enemy_y_fraction(&self, index: u16) -> u16 {
        self.word(0xf78 + usize::from(index) + offset_of!(EnemyData, y_subpos))
    }
    fn enemy_definition(&self, index: u16) -> u16 {
        self.word(0xf78 + usize::from(index) + offset_of!(EnemyData, enemy_ptr))
    }
    fn enemy_width(&self, index: u16) -> u16 {
        self.word(0xf78 + usize::from(index) + offset_of!(EnemyData, x_width))
    }
    fn enemy_height(&self, index: u16) -> u16 {
        self.word(0xf78 + usize::from(index) + offset_of!(EnemyData, y_height))
    }
    fn enemy_properties(&self, index: u16) -> u16 {
        self.word(0xf78 + usize::from(index) + offset_of!(EnemyData, properties))
    }
    fn enemy_parameter_1(&self, index: u16) -> u16 {
        self.word(0xf78 + usize::from(index) + offset_of!(EnemyData, parameter_1))
    }
    fn enemy_parameter_2(&self, index: u16) -> u16 {
        self.word(0xf78 + usize::from(index) + offset_of!(EnemyData, parameter_2))
    }
    fn enemy_ai_a(&self, index: u16) -> u16 {
        self.word(0xf78 + usize::from(index) + offset_of!(EnemyData, ai_var_A))
    }
    fn enemy_ai_d(&self, index: u16) -> u16 {
        self.word(0xf78 + usize::from(index) + offset_of!(EnemyData, ai_var_D))
    }
    fn enemy_ai_preinstr(&self, index: u16) -> u16 {
        self.word(0xf78 + usize::from(index) + offset_of!(EnemyData, ai_preinstr))
    }
    fn enemy_ai_var_B(&self, index: u16) -> u16 {
        self.word(0xf78 + usize::from(index) + offset_of!(EnemyData, ai_var_B))
    }
    fn enemy_flash_timer(&self, index: u16) -> u16 {
        self.word(0xf78 + usize::from(index) + offset_of!(EnemyData, flash_timer))
    }
    fn enemy_health(&self, index: u16) -> u16 {
        self.word(0xf78 + usize::from(index) + offset_of!(EnemyData, health))
    }
    fn enemy_tile(&self, index: u16) -> u16 {
        self.word(0xf78 + usize::from(index) + offset_of!(EnemyData, vram_tiles_index))
    }
    fn enemy_palette(&self, index: u16) -> u16 {
        self.word(0xf78 + usize::from(index) + offset_of!(EnemyData, palette_index))
    }
    fn set_enemy_health(&mut self, index: u16, value: u16) {
        self.write_slice_le(
            0xf78 + usize::from(index) + offset_of!(EnemyData, health),
            value,
        );
    }
    fn nuclear_waffle_part_offset(&self, index: u16) -> u16 {
        self.word(0x8000 + usize::from(index) + 20)
    }
    fn enemy_projectile_origin_x(&self, index: u16) -> u16 {
        self.word(0x8000 + usize::from(index) + 34)
    }
    fn enemy_projectile_origin_y(&self, index: u16) -> u16 {
        self.word(0x8000 + usize::from(index) + 36)
    }
    fn maw_body_part_index(&self, index: u16) -> u16 {
        self.word(0x8800 + usize::from(index) + 8)
    }
    fn kraid_health_eighth(&self, index: u16) -> u16 {
        self.word(0x7800 + usize::from(index) + offset_of!(Ram7800_Kraid, kraid_healths_8ths))
    }
    fn mini_kraid_spike_variant(&self, index: u16) -> u16 {
        self.word(0x7800 + usize::from(index) + offset_of!(Ram7800_Kraid, kraid_healths_8ths))
    }
    fn set_enemy_linked_projectile(&mut self, index: u16, value: u16) {
        self.write_slice_le(
            0x7800 + usize::from(index) + offset_of!(Ram7800_Kraid, kraid_next),
            value,
        );
    }
    fn set_botwoon_body_active(&mut self, index: u16, value: u16) {
        self.write_slice_le(
            0x7800 + usize::from(index) + offset_of!(Ram7800_Kraid, kraid_healths_4ths),
            value,
        );
    }
    fn set_maw_body_link(&mut self, index: u16, value: u16) {
        self.write_slice_le(0x8800 + usize::from(index), value);
    }
    fn mother_brain_active_bombs(&self) -> u16 {
        self.word(0x7840 + offset_of!(Ram7800_Kraid, kraid_mouth_flags))
    }
    fn mother_brain_blue_ring_contacts(&self) -> u16 {
        self.word(0x7800 + offset_of!(Ram7800_Kraid, field_28))
    }
    fn mother_brain_target_enemy(&self) -> u16 {
        self.word(0x7840 + offset_of!(Ram7800_Kraid, kraid_healths_8ths) + 8)
    }
    fn mother_brain_turret_destroyed(&self) -> u16 {
        self.word(0x7800 + 0x1d * 2)
    }
    fn botwoon_death_phase(&self) -> u16 {
        self.word(0x8000 + 32)
    }
    fn ridley_fireball_x_velocity(&self) -> u16 {
        self.word(0x7800 + 0x19 * 2)
    }
    fn ridley_fireball_y_velocity(&self) -> u16 {
        self.word(0x7800 + 0x1a * 2)
    }
    fn set_mother_brain_active_bombs(&mut self, value: u16) {
        self.write_slice_le(0x7840 + offset_of!(Ram7800_Kraid, kraid_mouth_flags), value);
    }
    fn set_mother_brain_blue_ring_contacts(&mut self, value: u16) {
        self.write_slice_le(0x7800 + offset_of!(Ram7800_Kraid, field_28), value);
    }
    fn set_mother_brain_hurt_frame(&mut self, value: u16) {
        self.write_slice_le(0x7840 + offset_of!(Ram7800_Kraid, kraid_hurt_frame), value);
    }
    fn set_mother_brain_purple_breath_active(&mut self, value: u16) {
        self.write_slice_le(0x7840 + offset_of!(Ram7800_Kraid, kraid_hurt_frame), value);
    }
    fn mother_brain_beam_x_fraction(&self) -> u16 {
        self.word(0xf78 + offset_of!(Enemy_MotherBrain, var_23))
    }
    fn set_mother_brain_beam_x_fraction(&mut self, value: u16) {
        self.write_slice_le(0xf78 + offset_of!(Enemy_MotherBrain, var_23), value);
    }
    fn mother_brain_beam_x(&self) -> u16 {
        self.word(0xf78 + offset_of!(Enemy_MotherBrain, var_24))
    }
    fn set_mother_brain_beam_x(&mut self, value: u16) {
        self.write_slice_le(0xf78 + offset_of!(Enemy_MotherBrain, var_24), value);
    }
    fn mother_brain_beam_y_fraction(&self) -> u16 {
        self.word(0xf78 + offset_of!(Enemy_MotherBrain, var_25))
    }
    fn set_mother_brain_beam_y_fraction(&mut self, value: u16) {
        self.write_slice_le(0xf78 + offset_of!(Enemy_MotherBrain, var_25), value);
    }
    fn mother_brain_beam_y(&self) -> u16 {
        self.word(0xf78 + offset_of!(Enemy_MotherBrain, var_26))
    }
    fn set_mother_brain_beam_y(&mut self, value: u16) {
        self.write_slice_le(0xf78 + offset_of!(Enemy_MotherBrain, var_26), value);
    }
    fn mother_brain_beam_velocity_x(&self) -> u16 {
        self.word(0xf78 + offset_of!(Enemy_MotherBrain, var_27))
    }
    fn set_mother_brain_beam_velocity_x(&mut self, value: u16) {
        self.write_slice_le(0xf78 + offset_of!(Enemy_MotherBrain, var_27), value);
    }
    fn mother_brain_beam_velocity_y(&self) -> u16 {
        self.word(0xf78 + offset_of!(Enemy_MotherBrain, var_28))
    }
    fn set_mother_brain_beam_velocity_y(&mut self, value: u16) {
        self.write_slice_le(0xf78 + offset_of!(Enemy_MotherBrain, var_28), value);
    }
    fn mother_brain_beam_angle(&self) -> u16 {
        self.word(0xf78 + offset_of!(Enemy_MotherBrain, var_29))
    }
    fn set_mother_brain_beam_angle(&mut self, value: u16) {
        self.write_slice_le(0xf78 + offset_of!(Enemy_MotherBrain, var_29), value);
    }
    fn set_enemy_extra_health_fraction(&mut self, index: u16, element: usize, value: u16) {
        self.write_slice_le(
            0x7800
                + usize::from(index)
                + offset_of!(Ram7800_Kraid, kraid_healths_8ths)
                + element * 2,
            value,
        );
    }
    fn set_botwoon_body_landed(&mut self, value: u16) {
        self.write_slice_le(0x883e, value);
    }
    fn spore_stem_anchor_x(&self) -> u16 {
        self.word(0xf78 + offset_of!(crate::game::enemy_layouts::EnemyLayout104, var_04))
    }
    fn spore_stem_anchor_y(&self) -> u16 {
        self.word(0xf78 + offset_of!(crate::game::enemy_layouts::EnemyLayout104, var_05))
    }
    fn set_spore_stem_control_x(&mut self, part: u8, value: u16) {
        use crate::game::enemy_layouts::EnemyLayout104 as Layout;
        let offset = [
            offset_of!(Layout, var_20),
            offset_of!(Layout, var_21),
            offset_of!(Layout, var_22),
        ][usize::from(part)];
        self.write_slice_le(0xf78 + offset, value);
    }
    fn set_spore_stem_control_y(&mut self, part: u8, value: u16) {
        use crate::game::enemy_layouts::EnemyLayout104 as Layout;
        let offset = [
            offset_of!(Layout, var_23),
            offset_of!(Layout, var_24),
            offset_of!(Layout, var_25),
        ][usize::from(part)];
        self.write_slice_le(0xf78 + offset, value);
    }
    fn maw_segment_handle(&self, enemy: u16, part: u8) -> u16 {
        use crate::game::enemy_layouts::EnemyLayout104 as Layout;
        let offset = [
            offset_of!(Layout, var_40),
            offset_of!(Layout, var_41),
            offset_of!(Layout, var_42),
            offset_of!(Layout, var_43),
        ][usize::from(part)];
        self.word(0xf78 + usize::from(enemy) + offset)
    }
    fn maw_anchor_x(&self, enemy: u16) -> u16 {
        self.word(
            0xf78
                + usize::from(enemy)
                + offset_of!(crate::game::enemy_layouts::EnemyLayout104, var_0C),
        )
    }
    fn maw_anchor_y(&self, enemy: u16) -> u16 {
        self.word(
            0xf78
                + usize::from(enemy)
                + offset_of!(crate::game::enemy_layouts::EnemyLayout104, var_0D),
        )
    }
    fn maw_segment_offset_x(&self, enemy: u16, part: u8) -> u16 {
        use crate::game::enemy_layouts::EnemyLayout104 as Layout;
        let offset = [
            offset_of!(Layout, var_00),
            offset_of!(Layout, var_02),
            offset_of!(Layout, var_04),
        ][usize::from(part).checked_sub(1).unwrap()];
        self.word(0xf78 + usize::from(enemy) + offset)
    }
    fn maw_segment_offset_y(&self, enemy: u16, part: u8) -> u16 {
        use crate::game::enemy_layouts::EnemyLayout104 as Layout;
        let offset = [
            offset_of!(Layout, var_01),
            offset_of!(Layout, var_03),
            offset_of!(Layout, var_05),
        ][usize::from(part).checked_sub(1).unwrap()];
        self.word(0xf78 + usize::from(enemy) + offset)
    }
    fn maw_palette(&self, enemy: u16) -> u16 {
        self.word(
            0xf78
                + usize::from(enemy)
                + offset_of!(crate::game::enemy_layouts::EnemyLayout104, var_45),
        )
    }
    fn maw_frozen_timer(&self, enemy: u16) -> u16 {
        self.word(0xf78 + usize::from(enemy) + offset_of!(EnemyData, frozen_timer))
    }
    fn expanding_square_origin(&self) -> u16 {
        self.word(0x9e00)
    }
    fn waffle_field(&self, enemy: usize, field: WaffleField) -> u16 {
        self.word(0xf78 + enemy + waffle_offset(field))
    }
    fn set_waffle_field(&mut self, enemy: usize, field: WaffleField, value: u16) {
        self.write_slice_le(0xf78 + enemy + waffle_offset(field), value);
    }
    fn botwoon_field(&self, enemy: usize, field: BotwoonField) -> u16 {
        self.word(0xf78 + enemy + botwoon_offset(field))
    }
    fn set_botwoon_field(&mut self, enemy: usize, field: BotwoonField, value: u16) {
        self.write_slice_le(0xf78 + enemy + botwoon_offset(field), value);
    }
    fn waffle_sprite_x(&self, index: u16) -> u16 {
        self.word(A::sprite_x_pos + usize::from(index) * 2)
    }
    fn set_waffle_sprite_x(&mut self, index: u16, value: u16) {
        self.write_slice_le(A::sprite_x_pos + usize::from(index) * 2, value);
    }
    fn waffle_sprite_y(&self, index: u16) -> u16 {
        self.word(A::sprite_y_pos + usize::from(index) * 2)
    }
    fn set_waffle_sprite_y(&mut self, index: u16, value: u16) {
        self.write_slice_le(A::sprite_y_pos + usize::from(index) * 2, value);
    }
    fn botwoon_history_x(&self, cursor: u16) -> u16 {
        self.word(0x9000 + usize::from(cursor))
    }
    fn set_botwoon_history_x(&mut self, cursor: u16, value: u16) {
        self.write_slice_le(0x9000 + usize::from(cursor), value);
    }
    fn botwoon_history_y(&self, cursor: u16) -> u16 {
        self.word(0x9002 + usize::from(cursor))
    }
    fn set_botwoon_history_y(&mut self, cursor: u16, value: u16) {
        self.write_slice_le(0x9002 + usize::from(cursor), value);
    }
}
