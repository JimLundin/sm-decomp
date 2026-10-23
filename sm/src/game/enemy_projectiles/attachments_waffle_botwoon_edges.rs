//! Byte projection for the remaining attached-enemy state and real operations.
#![forbid(unsafe_code)]
use super::{
    attachments_waffle_botwoon::{self as behavior, BotwoonField, WaffleField},
    compat::Execution,
};
use crate::{
    game::enemy_layouts::{EnemyLayout72, EnemyLayout104},
    game_state::Game,
    ram_map::word_arrays as A,
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
impl<'g> Execution<'g> {
    pub(super) fn waffle_field(&self, enemy: usize, field: WaffleField) -> u16 {
        self.owner
            .as_ref()
            .unwrap()
            .read_le(0xf78 + enemy + waffle_offset(field))
    }
    pub(super) fn set_waffle_field(&mut self, enemy: usize, field: WaffleField, value: u16) {
        self.owner
            .as_mut()
            .unwrap()
            .write_slice_le(0xf78 + enemy + waffle_offset(field), value);
    }
    pub(super) fn botwoon_field(&self, enemy: usize, field: BotwoonField) -> u16 {
        self.owner
            .as_ref()
            .unwrap()
            .read_le(0xf78 + enemy + botwoon_offset(field))
    }
    pub(super) fn set_botwoon_field(&mut self, enemy: usize, field: BotwoonField, value: u16) {
        self.owner
            .as_mut()
            .unwrap()
            .write_slice_le(0xf78 + enemy + botwoon_offset(field), value);
    }
    pub(super) fn waffle_sprite_x(&self, index: u16) -> u16 {
        self.owner
            .as_ref()
            .unwrap()
            .read_le(A::sprite_x_pos + usize::from(index) * 2)
    }
    pub(super) fn set_waffle_sprite_x(&mut self, index: u16, value: u16) {
        self.owner
            .as_mut()
            .unwrap()
            .write_slice_le(A::sprite_x_pos + usize::from(index) * 2, value);
    }
    pub(super) fn waffle_sprite_y(&self, index: u16) -> u16 {
        self.owner
            .as_ref()
            .unwrap()
            .read_le(A::sprite_y_pos + usize::from(index) * 2)
    }
    pub(super) fn set_waffle_sprite_y(&mut self, index: u16, value: u16) {
        self.owner
            .as_mut()
            .unwrap()
            .write_slice_le(A::sprite_y_pos + usize::from(index) * 2, value);
    }
    pub(super) fn botwoon_history_x(&self, cursor: u16) -> u16 {
        self.owner
            .as_ref()
            .unwrap()
            .read_le(0x9000 + usize::from(cursor))
    }
    pub(super) fn set_botwoon_history_x(&mut self, cursor: u16, value: u16) {
        self.owner
            .as_mut()
            .unwrap()
            .write_slice_le(0x9000 + usize::from(cursor), value);
    }
    pub(super) fn botwoon_history_y(&self, cursor: u16) -> u16 {
        self.owner
            .as_ref()
            .unwrap()
            .read_le(0x9002 + usize::from(cursor))
    }
    pub(super) fn set_botwoon_history_y(&mut self, cursor: u16, value: u16) {
        self.owner
            .as_mut()
            .unwrap()
            .write_slice_le(0x9002 + usize::from(cursor), value);
    }
    pub(super) fn waffle_sine_sample(&mut self, phase: u8) -> u8 {
        // Preserve the existing loaded-ROM source and its checked-read failure behavior.
        self.foreign(|game| game.read_rom_byte(0xa0b143, usize::from(phase)))
    }
}
pub(crate) fn advance_waffle_orbit(game: &mut Game) {
    behavior::advance_waffle_orbit(&mut Execution::acquire(game));
}
pub(crate) fn record_botwoon_head(game: &mut Game) {
    behavior::record_botwoon_head(&mut Execution::acquire(game));
}
pub(crate) fn follow_botwoon_history(game: &mut Game) {
    behavior::follow_botwoon_history(&mut Execution::acquire(game));
}
pub(crate) fn advance_botwoon_history(game: &mut Game) {
    behavior::advance_botwoon_history(&mut Execution::acquire(game));
}
pub(crate) fn orient_botwoon_body(game: &mut Game) {
    behavior::orient_botwoon_body(&mut Execution::acquire(game));
}
