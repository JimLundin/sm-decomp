//! Live foreign adapters over the shared Session projection.
#![forbid(unsafe_code)]
use super::host::{BotwoonField, SpeciesActors, WaffleField};
use crate::game::session_runtime::Runtime;
use crate::{
    game::enemy_layouts::{EnemyLayout72, EnemyLayout104},
    ram_map::words as W,
    types::Ram7800_Kraid,
};
use core::mem::offset_of;
use sm_game::actors::mother_brain::MotherBrainBody;
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
    }
}
impl SpeciesActors for Runtime<'_> {
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
        self.word(0xf78 + offset_of!(MotherBrainBody, hand_beam_x_subpos))
    }
    fn set_mother_brain_beam_x_fraction(&mut self, value: u16) {
        self.write_slice_le(
            0xf78 + offset_of!(MotherBrainBody, hand_beam_x_subpos),
            value,
        );
    }
    fn mother_brain_beam_x(&self) -> u16 {
        self.word(0xf78 + offset_of!(MotherBrainBody, hand_beam_x))
    }
    fn set_mother_brain_beam_x(&mut self, value: u16) {
        self.write_slice_le(0xf78 + offset_of!(MotherBrainBody, hand_beam_x), value);
    }
    fn mother_brain_beam_y_fraction(&self) -> u16 {
        self.word(0xf78 + offset_of!(MotherBrainBody, hand_beam_y_subpos))
    }
    fn set_mother_brain_beam_y_fraction(&mut self, value: u16) {
        self.write_slice_le(
            0xf78 + offset_of!(MotherBrainBody, hand_beam_y_subpos),
            value,
        );
    }
    fn mother_brain_beam_y(&self) -> u16 {
        self.word(0xf78 + offset_of!(MotherBrainBody, hand_beam_y))
    }
    fn set_mother_brain_beam_y(&mut self, value: u16) {
        self.write_slice_le(0xf78 + offset_of!(MotherBrainBody, hand_beam_y), value);
    }
    fn mother_brain_beam_velocity_x(&self) -> u16 {
        self.word(0xf78 + offset_of!(MotherBrainBody, hand_beam_x_speed))
    }
    fn set_mother_brain_beam_velocity_x(&mut self, value: u16) {
        self.write_slice_le(
            0xf78 + offset_of!(MotherBrainBody, hand_beam_x_speed),
            value,
        );
    }
    fn mother_brain_beam_velocity_y(&self) -> u16 {
        self.word(0xf78 + offset_of!(MotherBrainBody, hand_beam_y_speed))
    }
    fn set_mother_brain_beam_velocity_y(&mut self, value: u16) {
        self.write_slice_le(
            0xf78 + offset_of!(MotherBrainBody, hand_beam_y_speed),
            value,
        );
    }
    fn mother_brain_beam_angle(&self) -> u16 {
        self.word(0xf78 + offset_of!(MotherBrainBody, hand_beam_angle))
    }
    fn set_mother_brain_beam_angle(&mut self, value: u16) {
        self.write_slice_le(0xf78 + offset_of!(MotherBrainBody, hand_beam_angle), value);
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
