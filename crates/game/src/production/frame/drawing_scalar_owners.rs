//! Drawing scalar ownership joined with the actual admitted frame-start inputs.

use super::{borrowed_player_preparation as preparation, hud, hud_controls_inputs, hud_stage};
use canonical::Canonical;
use interface_hud::Observation;
use serde::{Deserialize, Serialize};

/// A required actual producer is absent or its retained current owner differs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// No actual Maintenance completion establishes current Drawing energy.
    MissingProducer,
    /// The actual input owner, bindings or completed consumer differs.
    Owner,
}

/// Complete admitted frame-start input shared with the actual PoseInput consumer.
/// Full held/pressed masks and configured bindings stay in their existing production type.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Physical {
    dependency: preparation::PoseInputOrigin,
}

impl Physical {
    /// Retain the independently admitted frame-start owner left by the preceding input poll.
    /// No new poll or source-stage identity is created or required.
    #[must_use]
    pub fn from_origin(dependency: &preparation::PoseInputOrigin) -> Self {
        Self {
            dependency: dependency.clone(),
        }
    }

    /// The sole complete dependency supplied to the real production PoseInput operation.
    #[must_use]
    pub const fn dependency(&self) -> &preparation::PoseInputOrigin {
        &self.dependency
    }

    /// Join the actual PoseInput completion to exactly the admitted current owner.
    /// Existing production admission and completion own the physical-to-action qualification.
    /// # Errors
    /// Physical held/pressed, starting pose or configured bindings changed.
    pub fn pose_input(&self, completion: &preparation::PoseInputCompletion) -> Result<(), Fault> {
        if completion.dependency != self.dependency {
            return Err(Fault::Owner);
        }
        Ok(())
    }

    /// Both held aliases and the pressed alias read this sole owner after real PoseInput.
    /// # Errors
    /// The actual completion consumed a different current dependency.
    pub fn at_drawing(
        &self,
        completion: &preparation::PoseInputCompletion,
    ) -> Result<(u16, u16), Fault> {
        self.pose_input(completion)?;
        Ok((self.dependency.held.0, self.dependency.pressed.0))
    }

    /// Join the same input through existing configured HUD projection and its current answer.
    /// # Errors
    /// Different PoseInput, current bindings/inventory, incomplete or stale HUD selection.
    pub fn controls(
        &self,
        bindings: hud_controls_inputs::Bindings,
        current: hud_controls_inputs::CurrentControls<'_>,
        completion: &hud_stage::ControlsCompletion,
        hud: &hud::Hud,
    ) -> Result<(), Fault> {
        self.pose_input(current.pose_input)?;
        let inputs = bindings.inputs(current).map_err(|_| Fault::Owner)?;
        if completion.answer.0.inputs != inputs
            || !matches!(hud.controls(), Some(Observation::Completed(answer)) if answer == &completion.answer.0)
            || hud.state().selection != completion.answer.0.selection
        {
            return Err(Fault::Owner);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use preparation::{
        BombJumpDirection, InputBindings, PhysicalButtons, PoseCommand, PoseInputCompletion,
        PoseInputOrigin, PoseInputTerminal,
    };

    fn completion() -> PoseInputCompletion {
        let pose = types::Pose {
            body: types::Body::Upright,
            action: types::Action::Running,
            facing: types::Facing::Right,
            aim: types::Aim::Forward,
            gun: types::Gun::Stowed,
            legs: types::Legs::Together,
        };
        PoseInputCompletion {
            dependency: PoseInputOrigin {
                pose,
                action: pose.action,
                held: PhysicalButtons(1026),
                pressed: PhysicalButtons(512),
                bindings: InputBindings {
                    shoot: PhysicalButtons(1),
                    jump: PhysicalButtons(2),
                    run: PhysicalButtons(2),
                    cancel: PhysicalButtons(4),
                    aim_up: PhysicalButtons(8),
                    aim_down: PhysicalButtons(16),
                },
            },
            terminal: PoseInputTerminal {
                held_actions: types::Buttons::default(),
                pressed_actions: types::Buttons::default(),
                prospective: pose,
                special: None,
                super_special: None,
                commands: [PoseCommand::None; 3],
                radius: types::Pair {
                    x: core::num::Wrapping(5),
                    y: core::num::Wrapping(21),
                },
                bomb_jump: BombJumpDirection::None,
            },
        }
    }

    #[test]
    fn frame_start_owner_retains_unmapped_bits_and_exact_bindings_without_new_poll() {
        let completion = completion();
        let owner = Physical::from_origin(&completion.dependency);
        assert!(owner.at_drawing(&completion) == Ok((1026, 512)));
        assert!(owner.dependency() == &completion.dependency);
        let mut changed = completion.clone();
        changed.dependency.held = PhysicalButtons(2);
        assert!(owner.at_drawing(&changed).is_err());
        changed = completion.clone();
        changed.dependency.pressed = PhysicalButtons(0);
        assert!(owner.at_drawing(&changed).is_err());
        changed = completion.clone();
        changed.dependency.bindings.run = PhysicalButtons(1024);
        assert!(owner.at_drawing(&changed).is_err());
    }
    #[test]
    fn existing_hud_projection_joins_the_same_current_input_and_completed_answer() {
        use super::super::borrowed_gameplay::{
            GameplayDispatchReceipt, GameplayRoute, SceneController,
        };
        use interface_hud::{
            Ammunition, Equipped, HudGrid, HudPictures, HudState, MapCellSet, MiniMapPower,
            Selection, Shown, Stock,
        };
        let pose_input = completion();
        let owner = Physical::from_origin(&pose_input.dependency);
        let gameplay = GameplayDispatchReceipt {
            scene: SceneController::MainGameplay,
            route: GameplayRoute::MainGameplay,
        };
        let ammunition =
            Ammunition::new(enum_map::enum_map! { _ => Stock { count: 0, capacity: 10 } }).unwrap();
        let equipped = Equipped {
            grapple_beam: false,
            x_ray_scope: false,
        };
        let inventory = hud_controls_inputs::CurrentInventory::from_current(ammunition, equipped);
        let current = hud_controls_inputs::CurrentControls {
            gameplay: &gameplay,
            scene: gameplay.scene,
            pose_input: &pose_input,
            pose: pose_input.dependency.pose,
            inventory: &inventory,
        };
        let bindings = hud_controls_inputs::Bindings {
            select: 512,
            can_switch: true,
        };
        let inputs = bindings.inputs(current).unwrap();
        let mut hud = hud::Hud::new(HudState {
            countdown: interface_hud::Countdown::Dormant,
            grid: HudGrid::filled(HudPictures::default().blank_cell()),
            shown: Shown::default(),
            selection: Selection::None,
            explored: MapCellSet::default(),
            mini_map: MiniMapPower::On,
        });
        hud_stage::choose_controls(&mut hud, inputs).unwrap();
        let controls =
            hud_stage::complete_controls_receipt(&mut hud, crate::production::timing::WorkId(16))
                .unwrap();
        assert!(owner.controls(bindings, current, &controls, &hud).is_ok());
        let changed = hud_controls_inputs::Bindings {
            select: 256,
            can_switch: true,
        };
        assert!(owner.controls(changed, current, &controls, &hud).is_err());
        let mut stale = controls.clone();
        stale.answer.0.inputs.cancel = interface_hud::CancelButton::Held;
        assert!(owner.controls(bindings, current, &stale, &hud).is_err());
    }
}
