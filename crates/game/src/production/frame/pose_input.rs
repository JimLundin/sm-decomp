//! Bind actual controller history to the clean chart before deferred pose placement.

use super::{
    borrowed_gameplay::{GameplayDispatchReceipt, GameplayRoute, SceneController},
    borrowed_player_preparation::{
        BombJumpDirection, InputBindings, PhysicalButtons, PoseCommand, PoseInputCompletion,
        PoseInputOrigin, PoseInputTerminal,
    },
};
use crate::production::MovementContent;
use canonical::Canonical;
use data::pose::transition::Control;
use types::{Action, Aim, Body, Buttons, Facing, Gun, Pose, Set, ZERO, button};

/// Live pad configuration carried while action bindings stay in this qualified scope.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Configuration {
    bindings: InputBindings,
    pending: Option<Pending>,
}

#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
struct Pending {
    path: Path,
    words: [u16; 2],
}

/// Qualified physical paths selected from the live chart query.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Path {
    AimDown,
    ReleaseAim,
}

/// A `PadChart` choice retained as a request until its later placement owner.
pub(super) struct Decision {
    pub(super) path: Path,
    pub(super) completion: PoseInputCompletion,
}

impl Configuration {
    /// Retain the actual input bindings from initial input-owner admission.
    #[must_use]
    pub const fn new(bindings: InputBindings) -> Self {
        Self {
            bindings,
            pending: None,
        }
    }

    pub(super) const fn charging(self) -> bool {
        self.pending.is_some()
    }

    pub(super) const fn begin(&mut self, decision: &Decision) -> Option<()> {
        if self.charging() {
            return None;
        }
        self.pending = Some(decision.pending());
        Some(())
    }

    pub(super) fn finish(&mut self, decision: Decision) -> Option<PoseInputCompletion> {
        if self.pending != Some(decision.pending()) {
            return None;
        }
        self.pending = None;
        Some(decision.completion)
    }

    pub(super) fn decide(
        self,
        pose: Pose,
        [held, pressed]: [u16; 2],
        route: &GameplayDispatchReceipt,
        content: &MovementContent,
    ) -> Option<Decision> {
        if route.scene != SceneController::MainGameplay
            || route.route != GameplayRoute::MainGameplay
            || pose.action != Action::Running
            || pose.body != Body::Upright
            || pose.facing != Facing::Right
            || pose.gun != Gun::Stowed
        {
            return None;
        }
        let held_actions = mapped(held, self.bindings);
        let pressed_actions = mapped(pressed, self.bindings);
        let running = button::RIGHT.0 | button::RUN.0;
        let aiming = running | button::AIM_DOWN.0;
        let (path, requested_aim) = match (pose.aim, held_actions.0, pressed_actions.0) {
            (Aim::Forward, held, pressed) if held == aiming && pressed == aiming => {
                (Path::AimDown, Aim::DownForward)
            }
            (Aim::DownForward, held, 0) if held == running => (Path::ReleaseAim, Aim::Forward),
            _ => return None,
        };
        // These running paths do not consult equipment, motion or animation end.
        // The later movement owner applies the chosen pose and any movement effects.
        let prospective = samus_movement::Moment {
            pad: held_actions,
            pressed: pressed_actions,
            items: Set::default(),
            animation_ends: None,
            speed: ZERO,
            airborne: false,
            met: None,
        }
        .next_pose(pose)
        .ok()?;
        if prospective
            != (Pose {
                aim: requested_aim,
                ..pose
            })
            || content.radius(prospective).is_none()
        {
            return None;
        }
        Some(Decision {
            path,
            completion: PoseInputCompletion {
                dependency: PoseInputOrigin {
                    pose,
                    action: pose.action,
                    held: PhysicalButtons(held),
                    pressed: PhysicalButtons(pressed),
                    bindings: self.bindings,
                },
                terminal: PoseInputTerminal {
                    pressed_actions,
                    held_actions,
                    prospective,
                    special: None,
                    super_special: None,
                    commands: [PoseCommand::None; 3],
                    radius: content.radius(pose)?,
                    bomb_jump: BombJumpDirection::None,
                },
            },
        })
    }
}

impl Decision {
    const fn pending(&self) -> Pending {
        Pending {
            path: self.path,
            words: [
                self.completion.dependency.held.0,
                self.completion.dependency.pressed.0,
            ],
        }
    }
}

fn mapped(word: u16, bindings: InputBindings) -> Buttons {
    let directions = [
        (Control::Right, button::RIGHT),
        (Control::Left, button::LEFT),
        (Control::Down, button::DOWN),
        (Control::Up, button::UP),
    ];
    let actions = [
        (bindings.shoot.0, button::SHOOT),
        (bindings.jump.0, button::JUMP),
        (bindings.run.0, button::RUN),
    ];
    let aim_buttons = Control::AimUp.bit() | Control::AimDown.bit();
    let aim = [
        (bindings.aim_up.0, button::AIM_UP),
        (bindings.aim_down.0, button::AIM_DOWN),
    ];
    Buttons(
        directions
            .into_iter()
            .map(|(direction, action)| (direction.bit(), action))
            .chain(actions)
            .chain(
                aim.into_iter()
                    .filter(|(binding, _)| binding & aim_buttons != 0),
            )
            .filter(|(binding, _)| word & binding != 0)
            .fold(0, |bits, (_, action)| bits | action.0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::num::Wrapping;
    use types::{Legs, Pair};

    fn running(aim: Aim) -> Pose {
        Pose {
            action: Action::Running,
            body: Body::Upright,
            facing: Facing::Right,
            gun: Gun::Stowed,
            legs: Legs::Together,
            aim,
        }
    }

    #[test]
    fn live_buttons_choose_deferred_aim_and_release_without_stale_completion() {
        let forward = running(Aim::Forward);
        let down = running(Aim::DownForward);
        let content = MovementContent::new(
            data::assets::samus::map::MAP
                .iter()
                .map(|entry| crate::production::PoseContent {
                    pose: entry.key,
                    radius: Pair {
                        x: Wrapping(5),
                        y: Wrapping(21),
                    },
                    cannon_heading: None,
                })
                .collect(),
        )
        .unwrap();
        let route = GameplayDispatchReceipt {
            scene: SceneController::MainGameplay,
            route: GameplayRoute::MainGameplay,
        };
        let bindings = InputBindings {
            shoot: PhysicalButtons(Control::Shoot.bit()),
            jump: PhysicalButtons(Control::Jump.bit()),
            run: PhysicalButtons(Control::Run.bit()),
            cancel: PhysicalButtons(Control::ItemCancel.bit()),
            aim_up: PhysicalButtons(Control::AimUp.bit()),
            aim_down: PhysicalButtons(Control::AimDown.bit()),
        };
        let mut owner = Configuration::new(bindings);
        let held = Control::Right.bit() | Control::Run.bit();
        let aiming = held | Control::AimDown.bit();

        let first = owner
            .decide(forward, [aiming, aiming], &route, &content)
            .unwrap();
        assert_eq!(first.path, Path::AimDown);
        owner.begin(&first).unwrap();
        let first = owner.finish(first).unwrap();
        assert_eq!(first.dependency.pose, forward);
        assert_eq!(first.terminal.prospective, down);

        assert!(owner.decide(down, [held, held], &route, &content).is_none());
        assert!(owner.decide(forward, [held, 0], &route, &content).is_none());
        let next = || {
            Configuration::new(bindings)
                .decide(down, [held, 0], &route, &content)
                .unwrap()
        };
        let original = next();
        assert_eq!(original.path, Path::ReleaseAim);
        owner.begin(&original).unwrap();
        let mut restored: Configuration =
            canonical::from_bytes(&canonical::to_bytes(&owner)).unwrap();
        assert_eq!(
            owner.finish(next()).unwrap(),
            restored.finish(original).unwrap()
        );
        assert!(!owner.charging());
        assert!(owner.finish(next()).is_none());

        let mut changed = next();
        changed.completion.dependency.pressed.0 = Control::Run.bit();
        restored.begin(&next()).unwrap();
        assert!(restored.finish(changed).is_none());
        let next = restored.finish(next()).unwrap();
        assert_eq!(next.dependency.pose, down);
        assert_eq!(next.terminal.prospective, forward);
    }
}
