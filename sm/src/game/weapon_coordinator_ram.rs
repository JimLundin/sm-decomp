//! Checked fields and real consumer bindings for weapon selection/coordinating.
use crate::game::{weapon_selection as selection, weapon_coordinator as coordinator};
use crate::game_state::Game;
use crate::ram_map::checked_word as word;
pub struct Runtime;
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
impl<'g> selection::Context<&'g mut Game> for Runtime {
    fn input(&self, g: &&'g mut Game, input: selection::Input) -> u16 {
        match input {
            selection::Input::CancelButton => word!(g.button_config_itemcancel_y),
            selection::Input::SwitchButton => word!(g.button_config_itemswitch),
            selection::Input::Pressed => word!(g.joypad1_newkeys),
            selection::Input::Held => word!(g.joypad1_lastkeys),
        }
    }
    fn field(&self, g: &&'g mut Game, f: selection::Field) -> u16 {
        match f {
            selection::Field::Selected => word!(g.hud_item_index),
            selection::Field::Automatic => word!(g.samus_auto_cancel_hud_item_index),
            selection::Field::Changed => word!(g.hud_item_changed_this_frame),
            selection::Field::Charge => word!(g.flare_counter),
        }
    }
    fn write(&mut self, g: &mut &'g mut Game, f: selection::Field, v: u16) {
        match f {
            selection::Field::Selected => word!(g.hud_item_index = v),
            selection::Field::Automatic => word!(g.samus_auto_cancel_hud_item_index = v),
            selection::Field::Changed => word!(g.hud_item_changed_this_frame = v),
            selection::Field::Charge => word!(g.flare_counter = v),
        }
    }
    fn available(&self, g: &&'g mut Game, c: selection::Choice) -> bool {
        match c {
            selection::Choice::Beam => true,
            selection::Choice::Missiles => word!(g.samus_missiles) != 0,
            selection::Choice::SuperMissiles => word!(g.samus_super_missiles) != 0,
            selection::Choice::PowerBombs => word!(g.samus_power_bombs) != 0,
            selection::Choice::Grapple => word!(g.equipped_items) & 0x4000 != 0,
            selection::Choice::Xray => word!(g.equipped_items) & 0x8000 != 0,
        }
    }
    fn grapple_inactive(&self, g: &&'g mut Game) -> bool {
        word!(g.grapple_beam_function) == 0xc4f0
    }
    fn clear_animation(&mut self, g: &mut &'g mut Game) {
        crate::game::beam_firing_ram::clear_animation(g);
    }
    fn palette(&mut self, g: &'g mut Game) -> &'g mut Game {
        unsafe { crate::game::sm_91::Samus_LoadSuitPalette(g) };
        g
    }
    fn return_grapple_inactive(&mut self, g: &'g mut Game) -> &'g mut Game {
        word!(g.grapple_beam_function = 0xc4f0);
        g
    }
    fn invalid(&mut self, _: &'g mut Game, t: selection::Table, i: usize, _: usize) -> ! {
        invalid(t, i)
    }
}
impl<'g> coordinator::Context<&'g mut Game> for Runtime {
    fn input(&self, g: &&'g mut Game, input: coordinator::Input) -> u16 {
        match input {
            coordinator::Input::Pose => word!(g.samus_pose),
            coordinator::Input::Movement => u16::from(g.g_ram[0xa1f]),
            coordinator::Input::PendingAim => word!(g.new_projectile_direction_changed_pose),
            coordinator::Input::Frozen => word!(g.time_is_frozen_flag),
            coordinator::Input::RunButton => word!(g.button_config_run_b),
            coordinator::Input::Held => word!(g.joypad1_lastkeys),
            coordinator::Input::Selected => word!(g.hud_item_index),
        }
    }
    fn cooldown(&self, g: &&'g mut Game) -> u16 {
        word!(g.cooldown_timer)
    }
    fn set_cooldown(&mut self, g: &mut &'g mut Game, v: u16) {
        word!(g.cooldown_timer = v);
    }
    fn grapple_active(&self, g: &&'g mut Game) -> bool {
        word!(g.grapple_beam_function) != 0xc4f0
    }
    fn switch(&mut self, g: &'g mut Game) -> &'g mut Game {
        selection::switch(g, self)
    }
    fn cancel_grapple(&mut self, g: &mut &'g mut Game) {
        word!(g.grapple_beam_function = 0xc856);
    }
    fn run(&mut self, g: &'g mut Game, action: coordinator::Action) -> &'g mut Game {
        match action {
            coordinator::Action::BeamInput => crate::game::beam_firing_ram::update_input(g),
            coordinator::Action::Missile => crate::game::missile_firing_ram::fire(g),
            coordinator::Action::Bomb => crate::game::bomb_creation_ram::fire(g),
            coordinator::Action::Grapple => unsafe { crate::game::sm_9b::GrappleBeamHandler(g) },
            coordinator::Action::Xray => unsafe { crate::game::sm_91::XrayRunHandler(g) },
            coordinator::Action::UpdateProjectiles => unsafe {
                crate::game::sm_90::HandleProjectile(g)
            },
        };
        g
    }
    fn invalid(&mut self, _: &'g mut Game, t: selection::Table, i: usize, _: usize) -> ! {
        invalid(t, i)
    }
}
pub fn coordinate(g: &mut Game) {
    coordinator::coordinate(g, &mut Runtime);
}
pub fn deselect(g: &mut Game) -> u8 {
    selection::select(g, selection::Choice::Beam, &mut Runtime);
    0
}
