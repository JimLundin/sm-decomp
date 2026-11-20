use super::*;
use crate::production::frame::borrowed_player_preparation::{
    BombJumpDirection, InputBindings, PhysicalButtons, PoseCommand, PoseInputOrigin,
    PoseInputTerminal,
};
use core::num::Wrapping;
use types::{Aim, Body, Buttons, Facing, Gun, Legs, Pair};

const fn pose() -> Pose {
    Pose {
        body: Body::Upright,
        action: Action::Running,
        facing: Facing::Right,
        aim: Aim::Forward,
        gun: Gun::Stowed,
        legs: Legs::Together,
    }
}

fn completed() -> PoseInputCompletion {
    let mut prospective = pose();
    prospective.facing = Facing::Left;
    PoseInputCompletion {
        dependency: PoseInputOrigin {
            pose: pose(),
            action: Action::Running,
            held: PhysicalButtons(0x0100),
            pressed: PhysicalButtons(0x0400),
            bindings: InputBindings {
                shoot: PhysicalButtons(0x8000),
                jump: PhysicalButtons(0x0080),
                run: PhysicalButtons(0x0040),
                cancel: PhysicalButtons(0x0100),
                aim_up: PhysicalButtons(0x0020),
                aim_down: PhysicalButtons(0x0010),
            },
        },
        terminal: PoseInputTerminal {
            pressed_actions: Buttons::default(),
            held_actions: Buttons::default(),
            prospective,
            special: None,
            super_special: None,
            commands: [PoseCommand::None; 3],
            radius: Pair {
                x: Wrapping(5),
                y: Wrapping(21),
            },
            bomb_jump: BombJumpDirection::None,
        },
    }
}

const fn inventory() -> CurrentInventory {
    CurrentInventory {
        missiles: Stock {
            count: 7,
            capacity: 11,
        },
        super_missiles: Stock {
            count: 3,
            capacity: 5,
        },
        power_bombs: Stock {
            count: 2,
            capacity: 4,
        },
        grapple_beam: true,
        x_ray_scope: false,
    }
}

#[test]
fn actual_completed_physical_controls_drive_select_and_cancel() {
    let bindings = Bindings {
        select: 0x0400,
        can_switch: true,
    };
    let gameplay = GameplayDispatchReceipt {
        scene: SceneController::MainGameplay,
        route: GameplayRoute::MainGameplay,
    };
    let mut completed = completed();
    let inventory = inventory();
    let actual = bindings
        .inputs(CurrentControls {
            gameplay: &gameplay,
            scene: gameplay.scene,
            pose_input: &completed,
            pose: pose(),
            inventory: &inventory,
        })
        .unwrap();
    assert_eq!(actual.select, SelectButton::Pressed);
    assert_eq!(actual.cancel, CancelButton::Held);
    assert_eq!(actual.ammunition.of(Ammo::Missiles).count, 7);
    assert!(actual.equipped.grapple_beam);
    completed.dependency.pressed.0 |= 0x0100;
    let actual = bindings
        .inputs(CurrentControls {
            gameplay: &gameplay,
            scene: gameplay.scene,
            pose_input: &completed,
            pose: pose(),
            inventory: &inventory,
        })
        .unwrap();
    assert_eq!(actual.cancel, CancelButton::Pressed);
}

#[test]
fn prospective_pose_and_changed_scene_do_not_replace_actual_starting_owners() {
    let bindings = Bindings {
        select: 0x0400,
        can_switch: true,
    };
    let gameplay = GameplayDispatchReceipt {
        scene: SceneController::MainGameplay,
        route: GameplayRoute::MainGameplay,
    };
    let completed = completed();
    let inventory = inventory();
    let current = CurrentControls {
        gameplay: &gameplay,
        scene: gameplay.scene,
        pose_input: &completed,
        pose: pose(),
        inventory: &inventory,
    };
    assert!(bindings.inputs(current).is_ok());
    assert_eq!(
        bindings.inputs(CurrentControls {
            pose: completed.terminal.prospective,
            ..current
        }),
        Err(Fault::Pose)
    );
    assert_eq!(
        bindings.inputs(CurrentControls {
            scene: SceneController::Other,
            ..current
        }),
        Err(Fault::Gameplay)
    );
    assert_eq!(
        Bindings {
            can_switch: false,
            ..bindings
        }
        .inputs(current),
        Err(Fault::Permission)
    );
}

#[test]
fn canonical_inventory_preserves_counts_capacities_and_checks_restored_domain() {
    let inventory = inventory();
    let bytes = canonical::to_bytes(&inventory);
    let mut restored: CurrentInventory = canonical::from_bytes(&bytes).unwrap();
    assert_eq!(restored, inventory);
    let checked = restored.ammunition().unwrap();
    assert_eq!(
        *checked.of(Ammo::PowerBombs),
        interface_hud::Stock {
            count: 2,
            capacity: 4
        }
    );
    assert_eq!(
        CurrentInventory::from_current(checked, restored.equipped()),
        inventory
    );
    restored.power_bombs.capacity = u16::MAX;
    assert_eq!(restored.ammunition(), Err(Fault::Inventory));
}
