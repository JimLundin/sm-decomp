use super::*;
use crate::production::frame::borrowed_gameplay::{
    GameplayDispatchReceipt, GameplayRoute, SceneController,
};
use crate::production::frame::borrowed_player_preparation::{
    BombJumpDirection, InputBindings, PhysicalButtons, PoseCommand, PoseInputCompletion,
    PoseInputOrigin, PoseInputTerminal,
};
use crate::production::frame::hud_controls_inputs::{CurrentInventory, Stock};
use crate::production::timing::WorkId;
use core::num::Wrapping;
use data::timing::{Cycle, ExposureKind, Sample, TemplateRun};
use interface_hud::{HudGrid, HudState, MapCellSet, MiniMapPower, Shown};
use types::{Action, Aim, Body, Buttons, Facing, Gun, Legs, Pair, Pose};

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
    PoseInputCompletion {
        dependency: PoseInputOrigin {
            pose: pose(),
            action: Action::Running,
            held: PhysicalButtons(0x0100),
            pressed: PhysicalButtons(0),
            bindings: InputBindings {
                shoot: PhysicalButtons(0x8000),
                jump: PhysicalButtons(0x0080),
                run: PhysicalButtons(0x0100),
                cancel: PhysicalButtons(0x2000),
                aim_up: PhysicalButtons(0x0020),
                aim_down: PhysicalButtons(0x0010),
            },
        },
        terminal: PoseInputTerminal {
            pressed_actions: Buttons::default(),
            held_actions: Buttons::default(),
            prospective: pose(),
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

fn owner() -> Hud {
    Hud::new(HudState {
        countdown: interface_hud::Countdown::Dormant,
        grid: HudGrid::filled(interface_hud::HudPictures::default().blank_cell()),
        shown: Shown::default(),
        selection: Selection::None,
        explored: MapCellSet::default(),
        mini_map: MiniMapPower::On,
    })
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

fn template() -> Template {
    Template {
        runs: vec![TemplateRun {
            count: 3,
            cycle: Cycle::Six,
            kind: ExposureKind::Internal,
            poll_before: true,
            effect: TemplateEffect::None,
        }],
    }
}

#[test]
fn actual_quiet_choice_retains_full_answer_until_physical_completion() {
    let gameplay = GameplayDispatchReceipt {
        scene: SceneController::MainGameplay,
        route: GameplayRoute::MainGameplay,
    };
    let completed = completed();
    let inventory = inventory();
    let bindings = Bindings {
        select: 0x0400,
        can_switch: true,
    };
    let current = CurrentControls {
        gameplay: &gameplay,
        scene: gameplay.scene,
        pose_input: &completed,
        pose: pose(),
        inventory: &inventory,
    };
    let mut hud = owner();
    let physical = choose(&mut hud, &bindings, current, &template()).unwrap();
    assert_eq!(physical.runs.len(), 1);
    assert_eq!(physical.runs.first().unwrap().count, 3);
    assert!(physical.runs.first().unwrap().exposure.poll_before);
    assert_eq!(hud.events(), []);
    let answer = hud
        .controls()
        .and_then(|observation| match observation {
            Observation::Chosen(answer) => Some(answer.clone()),
            Observation::Unread | Observation::Completed(_) => None,
        })
        .unwrap();
    assert_eq!(answer.inputs, bindings.inputs(current).unwrap());
    assert_eq!(answer.selection, Selection::None);
    assert!(answer.switched.is_none());
    let chosen = hud.clone();
    assert!(choose(&mut hud, &bindings, current, &template()).is_err());
    assert_eq!(hud, chosen);
    let receipt = hud_stage::complete_controls_receipt(&mut hud, WorkId(7)).unwrap();
    assert_eq!(receipt.work, WorkId(7));
    assert_eq!(receipt.answer.0, answer);
}

#[test]
fn active_physical_item_control_rejects_without_retaining_or_billing() {
    let gameplay = GameplayDispatchReceipt {
        scene: SceneController::MainGameplay,
        route: GameplayRoute::MainGameplay,
    };
    let mut completed = completed();
    completed.dependency.pressed = PhysicalButtons(0x0400);
    let inventory = inventory();
    let current = CurrentControls {
        gameplay: &gameplay,
        scene: gameplay.scene,
        pose_input: &completed,
        pose: pose(),
        inventory: &inventory,
    };
    let mut hud = owner();
    let initial = hud.clone();
    assert_eq!(
        choose(
            &mut hud,
            &Bindings {
                select: 0x0400,
                can_switch: true
            },
            current,
            &template()
        ),
        Err(Fault::Quiet)
    );
    assert_eq!(hud, initial);
}

#[test]
fn unexpected_device_sample_rejects_the_entire_quiet_owner_atomically() {
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
    let mut template = template();
    let run = template.runs.first_mut().unwrap();
    run.kind = ExposureKind::Read;
    run.effect = TemplateEffect::Sample(Sample::Blanking);
    let mut hud = owner();
    let initial = hud.clone();
    assert_eq!(
        choose(
            &mut hud,
            &Bindings {
                select: 0x0400,
                can_switch: true
            },
            current,
            &template
        ),
        Err(Fault::Profile)
    );
    assert_eq!(hud, initial);
}
