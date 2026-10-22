//! Checked compatibility access for the shared placement sequence.
#![forbid(unsafe_code)]
use crate::game::projectile_placement::{self as placement, Axis, Input, PoseField};
use crate::game_state::Game;
use crate::ram_map::checked_word;
use crate::rom::RomImage;

pub(crate) fn input(game: &Game, field: Input) -> u16 {
    match field {
        Input::Pose => checked_word!(game.samus_pose),
        Input::PreviousPose => checked_word!(game.samus_last_different_pose),
        Input::X => checked_word!(game.samus_x_pos),
        Input::Y => checked_word!(game.samus_y_pos),
        Input::Running => u16::from(game.g_ram[0xa1f]),
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
struct Ram {
    image: Option<RomImage>,
}
impl<'a> placement::Context<&'a mut Game> for Ram {
    fn input(&self, game: &&mut Game, field: Input) -> u16 {
        input(game, field)
    }
    fn pending(&mut self, game: &mut &'a mut Game) -> Option<u8> {
        let pending = checked_word!(game.new_projectile_direction_changed_pose);
        if pending == 0 {
            None
        } else {
            checked_word!(game.new_projectile_direction_changed_pose = 0);
            Some(pending as u8)
        }
    }
    fn pose_byte(&mut self, game: &'a mut Game, pose: u16, field: PoseField) -> (&'a mut Game, u8) {
        let byte = self
            .image
            .as_ref()
            .expect("ROM must be loaded")
            .read(pose_offset(pose, field));
        (game, byte)
    }
    fn reject(&mut self, game: &mut &'a mut Game) {
        checked_word!(
            game.projectile_counter = checked_word!(game.projectile_counter).wrapping_sub(1)
        );
    }
    fn set_direction(&mut self, game: &mut &'a mut Game, slot: u16, direction: u8) {
        checked_word!(game.projectile_dir[(slot >> 1)] = u16::from(direction));
    }
    fn direction(&self, game: &&mut Game, slot: u16) -> u16 {
        checked_word!(game.projectile_dir[(slot >> 1)])
    }
    fn set_position(&mut self, game: &mut &'a mut Game, slot: u16, axis: Axis, value: u16) {
        match axis {
            Axis::X => checked_word!(game.projectile_x_pos[(slot >> 1)] = value),
            Axis::Y => checked_word!(game.projectile_y_pos[(slot >> 1)] = value),
        }
    }
    fn invalid(&mut self, _: &'a mut Game, index: usize) -> ! {
        panic!("index {index} outside 10")
    }
}
pub(crate) fn place(slot: u16, game: &mut Game) -> u8 {
    let image = game.loaded_rom.clone();
    u8::from(!placement::place(game, slot, &mut Ram { image }).1)
}
pub(crate) fn place_with_image(slot: u16, game: &mut Game, image: RomImage) -> u8 {
    u8::from(!placement::place(game, slot, &mut Ram { image: Some(image) }).1)
}
