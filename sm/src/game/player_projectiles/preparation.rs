//! Complete preparation/weapon/update/history interval, interrupted only by native player leaves.
use super::{Context, Transaction};
use crate::game::player_preparation::{self as shared, Entry, Field, Input, Step};
use crate::game::{weapon_selection as selection, weapon_coordinator as coordinator};
use crate::ram_map::words as ram;
#[forbid(unsafe_code)]
pub fn read(game: &Transaction<'_>, input: Input) -> u16 {
    let address = match input {
        Input::Held => ram::joypad1_lastkeys,
        Input::Pressed => ram::joypad1_newkeys,
        Input::FilteredHeld => ram::joypad1_input_samusfilter,
        Input::FilteredPressed => ram::joypad1_newinput_samusfilter,
        Input::AimHistory => ram::UNUSED_word_7E0DFA,
        Input::MovementAndFacing => 0xa1e,
    };
    game.read_le(address)
}
#[forbid(unsafe_code)]
pub fn write(game: &mut Transaction<'_>, field: Field, value: u16) {
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
    game.write_slice_le(address, value);
}
impl<'a> shared::Context<Transaction<'a>> for Context {
    fn read(&self, game: &Transaction<'a>, input: Input) -> u16 {
        read(game, input)
    }
    fn write(&mut self, game: &mut Transaction<'a>, field: Field, value: u16) {
        write(game, field, value);
    }
    fn clear_launch_history(&mut self, game: &mut Transaction<'a>) {
        game.state.launch.finish_projectile_phase();
        crate::verify_probe::hit("player_projectiles.history.finished");
    }
    fn step(&mut self, game: Transaction<'a>, step: Step) -> Transaction<'a> {
        use crate::game::{sm_90, sm_94};
        match step {
            Step::Radius => game.foreign(|g| sm_90::Samus_SetRadius(g)).0,
            Step::Input => {
                game.foreign(|g| unsafe { sm_90::Samus_CallInputHandler(g) })
                    .0
            }
            Step::SuitIndex => {
                game.foreign(|g| unsafe { sm_90::Samus_UpdateSuitPaletteIndex(g) })
                    .0
            }
            Step::Acceleration => {
                game.foreign(|g| unsafe { sm_90::Samus_DetermineAccel_Y(g) })
                    .0
            }
            Step::InsideBlock => {
                game.foreign(|g| unsafe { sm_94::BlockInsideDetection(g) })
                    .0
            }
            Step::Weapons => {
                crate::verify_probe::hit("player_projectiles.prepare.weapons");
                coordinator::coordinate(game, self)
            }
            Step::Projectiles => super::motion::advance_all(game),
        }
    }
}
fn invalid(table: selection::Table, index: usize) -> ! {
    match table {
        selection::Table::Selection => {
            let _ = [(); 6][index];
        }
        selection::Table::Movement => {
            let _ = [(); 28][index];
        }
        selection::Table::Normal => {
            let _ = [(); 7][index];
        }
        selection::Table::Crouch => {
            let _ = [(); 12][index];
        }
    }
    unreachable!()
}
impl<'g> selection::Context<Transaction<'g>> for Context {
    fn input(&self, g: &Transaction<'g>, input: selection::Input) -> u16 {
        match input {
            selection::Input::CancelButton => word!(g.button_config_itemcancel_y),
            selection::Input::SwitchButton => word!(g.button_config_itemswitch),
            selection::Input::Pressed => word!(g.joypad1_newkeys),
            selection::Input::Held => word!(g.joypad1_lastkeys),
        }
    }
    fn field(&self, g: &Transaction<'g>, f: selection::Field) -> u16 {
        match f {
            selection::Field::Selected => word!(g.hud_item_index),
            selection::Field::Automatic => word!(g.samus_auto_cancel_hud_item_index),
            selection::Field::Changed => word!(g.hud_item_changed_this_frame),
            selection::Field::Charge => word!(g.flare_counter),
        }
    }
    fn write(&mut self, g: &mut Transaction<'g>, f: selection::Field, v: u16) {
        match f {
            selection::Field::Selected => word!(g.hud_item_index = v),
            selection::Field::Automatic => word!(g.samus_auto_cancel_hud_item_index = v),
            selection::Field::Changed => word!(g.hud_item_changed_this_frame = v),
            selection::Field::Charge => word!(g.flare_counter = v),
        }
    }
    fn available(&self, g: &Transaction<'g>, c: selection::Choice) -> bool {
        match c {
            selection::Choice::Beam => true,
            selection::Choice::Missiles => word!(g.samus_missiles) != 0,
            selection::Choice::SuperMissiles => word!(g.samus_super_missiles) != 0,
            selection::Choice::PowerBombs => word!(g.samus_power_bombs) != 0,
            selection::Choice::Grapple => word!(g.equipped_items) & 0x4000 != 0,
            selection::Choice::Xray => word!(g.equipped_items) & 0x8000 != 0,
        }
    }
    fn grapple_inactive(&self, g: &Transaction<'g>) -> bool {
        word!(g.grapple_beam_function) == 0xc4f0
    }
    fn clear_animation(&mut self, g: &mut Transaction<'g>) {
        super::weapons::clear_animation(g);
    }
    fn palette(&mut self, mut g: Transaction<'g>) -> Transaction<'g> {
        g.foreign(|g| unsafe { crate::game::sm_91::Samus_LoadSuitPalette(g) })
            .0
    }
    fn return_grapple_inactive(&mut self, mut g: Transaction<'g>) -> Transaction<'g> {
        word!(g.grapple_beam_function = 0xc4f0);
        g
    }
    fn invalid(&mut self, _: Transaction<'g>, t: selection::Table, i: usize, _: usize) -> ! {
        invalid(t, i)
    }
}
impl<'g> coordinator::Context<Transaction<'g>> for Context {
    fn input(&self, g: &Transaction<'g>, input: coordinator::Input) -> u16 {
        match input {
            coordinator::Input::Pose => word!(g.samus_pose),
            coordinator::Input::Movement => u16::from(g.byte(0xa1f)),
            coordinator::Input::PendingAim => word!(g.new_projectile_direction_changed_pose),
            coordinator::Input::Frozen => word!(g.time_is_frozen_flag),
            coordinator::Input::RunButton => word!(g.button_config_run_b),
            coordinator::Input::Held => word!(g.joypad1_lastkeys),
            coordinator::Input::Selected => word!(g.hud_item_index),
        }
    }
    fn cooldown(&self, g: &Transaction<'g>) -> u16 {
        word!(g.cooldown_timer)
    }
    fn set_cooldown(&mut self, g: &mut Transaction<'g>, v: u16) {
        word!(g.cooldown_timer = v);
    }
    fn grapple_active(&self, g: &Transaction<'g>) -> bool {
        word!(g.grapple_beam_function) != 0xc4f0
    }
    fn switch(&mut self, mut g: Transaction<'g>) -> Transaction<'g> {
        selection::switch(g, self)
    }
    fn cancel_grapple(&mut self, g: &mut Transaction<'g>) {
        word!(g.grapple_beam_function = 0xc856);
    }
    fn run(&mut self, mut g: Transaction<'g>, action: coordinator::Action) -> Transaction<'g> {
        match action {
            coordinator::Action::BeamInput => super::weapons::update_input(g),
            coordinator::Action::Missile => super::missiles::fire(g),
            coordinator::Action::Bomb => super::bomb_creation::fire(g),
            coordinator::Action::Grapple => {
                g.foreign(|g| unsafe { crate::game::sm_9b::GrappleBeamHandler(g) })
                    .0
            }
            coordinator::Action::Xray => {
                g.foreign(|g| unsafe { crate::game::sm_91::XrayRunHandler(g) })
                    .0
            }
            coordinator::Action::UpdateProjectiles => super::motion::advance_all(g),
        }
    }
    fn invalid(&mut self, _: Transaction<'g>, t: selection::Table, i: usize, _: usize) -> ! {
        invalid(t, i)
    }
}
pub(super) fn prepare(entry: Entry, owner: Transaction<'_>) -> Transaction<'_> {
    shared::prepare(owner, entry, &mut Context::default())
}
pub(super) fn deselect(owner: Transaction<'_>) -> Transaction<'_> {
    selection::select(owner, selection::Choice::Beam, &mut Context::default()).0
}
