//! Player preparation, weapon selection and launch history over ordinary state.
#![forbid(unsafe_code)]
use super::{
    host::{Host, PaletteEffect, PlayerStep},
    state::PrepareEntry,
};
#[derive(Clone, Copy, PartialEq)]
enum Choice {
    Beam,
    Missiles,
    SuperMissiles,
    PowerBombs,
    Grapple,
    Xray,
}
#[derive(Clone, Copy)]
enum Mode {
    Normal,
    Jump,
    Bomb,
    Grapple,
    Turning,
    Crouch,
    Draygon,
}
#[derive(Clone, Copy)]
enum Table {
    Selection,
    Movement,
    Normal,
    Crouch,
}
fn invalid(table: Table, index: usize) -> ! {
    match table {
        Table::Selection => {
            sm_data::table::check_index(index, 6);
        }
        Table::Movement => {
            sm_data::table::check_index(index, 28);
        }
        Table::Normal => {
            sm_data::table::check_index(index, 7);
        }
        Table::Crouch => {
            sm_data::table::check_index(index, 12);
        }
    }
    unreachable!()
}
pub fn prepare(h: &mut impl Host, entry: PrepareEntry) {
    match entry {
        PrepareEntry::Inactive => return,
        PrepareEntry::ProjectilesOnly => super::motion::advance_all(h),
        PrepareEntry::Normal | PrepareEntry::Demo => {
            h.session_mut().player.transitions.clear_pose_requests();
            if entry == PrepareEntry::Demo {
                h.session_mut().projectiles.preparation.demo_held = h.inputs().held;
                h.session_mut().projectiles.preparation.demo_pressed = h.inputs().pressed;
                h.session_mut().projectiles.preparation.demo_filtered_held =
                    h.inputs().filtered_held;
                h.session_mut()
                    .projectiles
                    .preparation
                    .demo_filtered_pressed = h.inputs().filtered_pressed;
            }
            h.player_step(PlayerStep::Radius);
            for step in if entry == PrepareEntry::Normal {
                [PlayerStep::Input, PlayerStep::SuitIndex]
            } else {
                [PlayerStep::SuitIndex, PlayerStep::Input]
            } {
                h.player_step(step);
            }
            h.player_step(PlayerStep::Acceleration);
            h.player_step(PlayerStep::InsideBlock);
            h.probe("player_projectiles.prepare.weapons");
            coordinate(h);
        }
    }
    h.session_mut().projectiles.launch.finish_projectile_phase();
    h.probe("player_projectiles.history.finished");
    h.session_mut().player.transitions.animation_skip =
        crate::player_transitions::state::AnimationSkip::from_raw(0);
    h.session_mut().projectiles.controls.pending_aim = 0;
    h.session_mut().projectiles.preparation.aim_history = h
        .session()
        .projectiles
        .preparation
        .aim_history
        .wrapping_shl(8);
    h.session_mut()
        .projectiles
        .preparation
        .previous_movement_and_facing = h.session().player.movement_and_facing;
}
fn available(h: &impl Host, choice: Choice) -> bool {
    match choice {
        Choice::Beam => true,
        Choice::Missiles => h.session().inventory.ammunition[0].current != 0,
        Choice::SuperMissiles => h.session().inventory.ammunition[1].current != 0,
        Choice::PowerBombs => h.session().inventory.ammunition[2].current != 0,
        Choice::Grapple => h.session().inventory.equipment.active & 0x4000 != 0,
        Choice::Xray => h.session().inventory.equipment.active & 0x8000 != 0,
    }
}
fn select(h: &mut impl Host, choice: Choice) -> bool {
    if !available(h, choice) {
        return false;
    }
    if choice == Choice::Grapple {
        if h.session().player.grapple == 0xc4f0 {
            h.palette(PaletteEffect::Suit);
            h.session_mut().projectiles.controls.charge = 0;
            super::weapons::clear_animation(h);
            h.session_mut().player.grapple = 0xc4f0;
        }
    } else {
        h.session_mut().projectiles.controls.charge = 0;
        super::weapons::clear_animation(h);
        h.palette(PaletteEffect::Suit);
    }
    true
}
pub fn deselect(h: &mut impl Host) {
    select(h, Choice::Beam);
}
fn switch(h: &mut impl Host) {
    let previous = h.session().projectiles.controls.selected_weapon;
    let mut automatic = false;
    let selection = if h.inputs().cancel_button & h.inputs().pressed != 0 {
        h.session_mut().projectiles.controls.automatic_weapon = 0;
        Some(0)
    } else {
        automatic = h.inputs().cancel_button & h.inputs().held != 0;
        if h.inputs().switch_button & h.inputs().pressed == 0 {
            None
        } else {
            let next = h
                .session()
                .projectiles
                .controls
                .selected_weapon
                .wrapping_add(1);
            Some(
                if h.session()
                    .projectiles
                    .controls
                    .selected_weapon
                    .wrapping_sub(5)
                    & 0x8000
                    == 0
                {
                    0
                } else {
                    next
                },
            )
        }
    };
    if let Some(mut selected) = selection {
        h.session_mut().projectiles.controls.selected_weapon = selected;
        loop {
            let choices = [
                Choice::Beam,
                Choice::Missiles,
                Choice::SuperMissiles,
                Choice::PowerBombs,
                Choice::Grapple,
                Choice::Xray,
            ];
            let Some(&choice) = choices.get(usize::from(selected)) else {
                invalid(Table::Selection, usize::from(selected));
            };
            if select(h, choice) {
                break;
            }
            selected = h
                .session()
                .projectiles
                .controls
                .selected_weapon
                .wrapping_add(1);
            h.session_mut().projectiles.controls.selected_weapon = selected;
            if selected.wrapping_sub(6) & 0x8000 == 0 {
                selected = 0;
                h.session_mut().projectiles.controls.selected_weapon = 0;
            }
        }
        h.session_mut().projectiles.controls.automatic_weapon = if automatic {
            h.session().projectiles.controls.selected_weapon
        } else {
            0
        };
    }
    h.session_mut().projectiles.controls.weapon_changed =
        if h.session().projectiles.controls.selected_weapon == previous {
            let next = h
                .session()
                .projectiles
                .controls
                .weapon_changed
                .wrapping_add(1);
            if h.session()
                .projectiles
                .controls
                .weapon_changed
                .wrapping_sub(2)
                & 0x8000
                == 0
            {
                2
            } else {
                next
            }
        } else {
            1
        };
}
fn coordinate(h: &mut impl Host) {
    if h.session().player.pose == 0 || h.session().player.pose == 0x9b {
        return super::motion::advance_all(h);
    }
    let cooldown = h.session().projectiles.controls.cooldown;
    h.session_mut().projectiles.controls.cooldown = if h.inputs().frozen != 0 {
        32
    } else if cooldown == 0 || cooldown & 0x8000 != 0 {
        0
    } else {
        let next = cooldown.wrapping_sub(1);
        if next & 0x8000 != 0 { 0 } else { next }
    };
    switch(h);
    use Mode::*;
    let modes = [
        Normal, Normal, Normal, Jump, Bomb, Normal, Normal, Bomb, Bomb, Bomb, Jump, Grapple,
        Grapple, Jump, Turning, Crouch, Normal, Bomb, Bomb, Bomb, Jump, Normal, Grapple, Turning,
        Turning, Jump, Draygon, Jump,
    ];
    let index = usize::from(h.session().player.movement_and_facing >> 8);
    let Some(&mode) = modes.get(index) else {
        invalid(Table::Movement, index);
    };
    match mode {
        Normal => normal(h),
        Jump => jump(h),
        Bomb => super::bomb_creation::fire(h),
        Grapple => h.player_step(PlayerStep::Grapple),
        Turning => turning(h),
        Crouch => crouch(h),
        Draygon => {
            if h.session().player.pose == 0xdf {
                super::bomb_creation::fire(h);
            } else {
                normal(h);
            }
        }
    }
    if h.inputs().frozen == 0 {
        super::motion::advance_all(h);
    }
}
fn normal(h: &mut impl Host) {
    let index = if h.session().player.grapple != 0xc4f0 {
        4
    } else if h.inputs().frozen != 0 {
        5
    } else {
        h.session()
            .projectiles
            .controls
            .selected_weapon
            .wrapping_mul(2)
            >> 1
    };
    match index {
        0 | 3 => super::weapons::update_input(h),
        1 | 2 => super::missiles::fire(h),
        4 => h.player_step(PlayerStep::Grapple),
        5 => {
            if h.inputs().run_button & h.inputs().held != 0 {
                h.player_step(PlayerStep::Xray);
            } else {
                super::weapons::update_input(h);
            }
        }
        6 => turning(h),
        _ => invalid(Table::Normal, usize::from(index)),
    }
}
fn turning(h: &mut impl Host) {
    if h.session().projectiles.controls.pending_aim != 0 {
        return normal(h);
    }
    if h.session().player.grapple != 0xc4f0 {
        h.session_mut().player.grapple = 0xc856;
    }
}
fn jump(h: &mut impl Host) {
    if h.session().player.grapple != 0xc4f0 {
        h.session_mut().player.grapple = 0xc856;
        normal(h);
    }
}
fn crouch(h: &mut impl Host) {
    if h.session().player.pose.wrapping_sub(0xf1) & 0x8000 == 0 {
        return normal(h);
    }
    if h.session().player.pose.wrapping_sub(0xdb) & 0x8000 == 0 {
        return;
    }
    let index = usize::from(h.session().player.pose.wrapping_sub(53));
    let choices = [
        false, false, true, true, true, true, false, false, true, true, true, true,
    ];
    let Some(&cancel) = choices.get(index) else {
        invalid(Table::Crouch, index);
    };
    if cancel {
        jump(h);
    } else {
        normal(h);
    }
}
