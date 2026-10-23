//! Checked phase fields and explicit calls into remaining external player leaves.
use crate::game::player_preparation::{self as shared, Context, Entry, Field, Input, Step};
use crate::game_state::Game;
use crate::ram_map::words as ram;

pub fn entry(address: u32) -> Option<Entry> {
    match address {
        0x90e695 => Some(Entry::Normal),
        0x90e6c9 => Some(Entry::Demo),
        0x90e713 => Some(Entry::ProjectilesOnly),
        0x90e8cd => Some(Entry::Inactive),
        _ => None,
    }
}
#[forbid(unsafe_code)]
pub fn read(game: &Game, input: Input) -> u16 {
    let address = match input {
        Input::Held => ram::joypad1_lastkeys,
        Input::Pressed => ram::joypad1_newkeys,
        Input::FilteredHeld => ram::joypad1_input_samusfilter,
        Input::FilteredPressed => ram::joypad1_newinput_samusfilter,
        Input::AimHistory => ram::UNUSED_word_7E0DFA,
        Input::MovementAndFacing => 0xa1e,
    };
    u16::from_le_bytes([game.g_ram[address], game.g_ram[address + 1]])
}
#[forbid(unsafe_code)]
pub fn write(game: &mut Game, field: Field, value: u16) {
    let address = match field {
        Field::NextPose => ram::samus_new_pose,
        Field::InterruptedPose => ram::samus_new_pose_interrupted,
        Field::TransitionalPose => ram::samus_new_pose_transitional,
        Field::MomentumRoutine => ram::samus_momentum_routine_index,
        Field::SpecialGraphics => ram::samus_special_transgfx_index,
        Field::HurtSwitch => ram::samus_hurt_switch_index,
        Field::DemoHeld => ram::controller1_input_for_demo,
        Field::DemoPressed => ram::controller1_new_input_for_demo,
        Field::DemoFilteredHeld => ram::demo_backup_prev_controller_input,
        Field::DemoFilteredPressed => ram::demo_backup_prev_controller_input_new,
        Field::AnimationSkip => ram::samus_anim_frame_skip,
        Field::PendingAim => ram::new_projectile_direction_changed_pose,
        Field::AimHistory => ram::UNUSED_word_7E0DFA,
        // Includes previous movement at A11; A22 belongs to a different pose history.
        Field::PreviousMovementAndFacing => 0xa10,
    };
    game.g_ram[address..address + 2].copy_from_slice(&value.to_le_bytes());
}
struct Runtime;
impl<'a> Context<&'a mut Game> for Runtime {
    fn read(&self, game: &&mut Game, input: Input) -> u16 {
        read(game, input)
    }
    fn write(&mut self, game: &mut &mut Game, field: Field, value: u16) {
        write(game, field, value);
    }
    fn clear_launch_history(&mut self, game: &mut &mut Game) {
        crate::game::launch_motion_ram::finish_projectile_phase(game);
    }
    fn step(&mut self, game: &'a mut Game, step: Step) -> &'a mut Game {
        use crate::game::{sm_90, sm_94};
        match step {
            Step::Radius => sm_90::Samus_SetRadius(game),
            Step::Input => unsafe { sm_90::Samus_CallInputHandler(game) },
            Step::SuitIndex => unsafe { sm_90::Samus_UpdateSuitPaletteIndex(game) },
            Step::Acceleration => unsafe { sm_90::Samus_DetermineAccel_Y(game) },
            Step::InsideBlock => unsafe { sm_94::BlockInsideDetection(game) },
            Step::Weapons => crate::game::weapon_coordinator_ram::coordinate(game),
            Step::Projectiles => unsafe { sm_90::HandleProjectile(game) },
        }
        game
    }
}
pub fn dispatch(address: u32, game: &mut Game) {
    if let Some(entry) = entry(address) {
        shared::prepare(game, entry, &mut Runtime);
    } else {
        unsafe {
            crate::rtl::Unreachable(game);
        }
    }
}
pub fn prepare(game: &mut Game) {
    let address = 0x900000
        | u32::from(u16::from_le_bytes([
            game.g_ram[ram::frame_handler_alfa],
            game.g_ram[ram::frame_handler_alfa + 1],
        ]));
    dispatch(address, game);
}
