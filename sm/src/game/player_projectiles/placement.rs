//! Direct placement with its original captured optional ROM image.
use super::{Context, Transaction};
use crate::game::projectile_placement::{self as placement, Axis, Input, PoseField};
pub(crate) fn input(game: &Transaction<'_>, field: Input) -> u16 {
    match field {
        Input::Pose => word!(game.samus_pose),
        Input::PreviousPose => word!(game.samus_last_different_pose),
        Input::X => word!(game.samus_x_pos),
        Input::Y => word!(game.samus_y_pos),
        Input::Running => u16::from(game.byte(0xa1f)),
    }
}
pub(crate) fn pose_offset(pose: u16, field: PoseField) -> usize {
    crate::rtl::lorom_offset(0x91b629) as usize
        + 8 * usize::from(pose)
        + match field {
            PoseField::Direction => 3,
            PoseField::Height => 4,
        }
}
impl<'a> placement::Context<Transaction<'a>> for Context {
    fn input(&self, game: &Transaction<'a>, field: Input) -> u16 {
        input(game, field)
    }
    fn pending(&mut self, game: &mut Transaction<'a>) -> Option<u8> {
        let pending = word!(game.new_projectile_direction_changed_pose);
        if pending == 0 {
            None
        } else {
            word!(game.new_projectile_direction_changed_pose = 0);
            Some(pending as u8)
        }
    }
    fn pose_byte(
        &mut self,
        game: Transaction<'a>,
        pose: u16,
        field: PoseField,
    ) -> (Transaction<'a>, u8) {
        let byte = self
            .image
            .as_ref()
            .expect("ROM must be loaded")
            .read(pose_offset(pose, field));
        (game, byte)
    }
    fn reject(&mut self, game: &mut Transaction<'a>) {
        word!(game.projectile_counter = word!(game.projectile_counter).wrapping_sub(1));
    }
    fn set_direction(&mut self, game: &mut Transaction<'a>, slot: u16, direction: u8) {
        word!(game.projectile_dir[(slot >> 1)] = u16::from(direction));
    }
    fn direction(&self, game: &Transaction<'a>, slot: u16) -> u16 {
        word!(game.projectile_dir[(slot >> 1)])
    }
    fn set_position(&mut self, game: &mut Transaction<'a>, slot: u16, axis: Axis, value: u16) {
        match axis {
            Axis::X => word!(game.projectile_x_pos[(slot >> 1)] = value),
            Axis::Y => word!(game.projectile_y_pos[(slot >> 1)] = value),
        }
    }
    fn invalid(&mut self, _: Transaction<'a>, index: usize) -> ! {
        panic!("index {index} outside 10")
    }
}
pub(super) fn place(slot: u16, owner: Transaction<'_>) -> (Transaction<'_>, bool) {
    let image = owner.capture_rom();
    placement::place(
        owner,
        slot,
        &mut Context {
            image,
            ..Context::default()
        },
    )
}
