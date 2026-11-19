//! Spec: `PoseFollows`. Accepted pose effects, used by the owned ordinary continuation.
use crate::rows::Effect;
use crate::run::Dashing;
use crate::samus::{Samus, Spin};
use crate::tick::Tick;
use types::{Aim, ZERO};
/// Spec: `TransitionFiringAim`. Publication reads accepted pose content, never desired aim.
pub fn transition_firing_aim(
    old: types::Pose,
    new: types::Pose,
    frame: &crate::tick::Frame,
) -> Option<types::FiringDirection> {
    if old == new {
        return None;
    }
    if new.body == types::Body::Upright
        && new.action == types::Action::Jumping
        && frame.pressed.any(types::button::SHOOT)
    {
        return (frame.cannon_heading)(new);
    }
    if old.action == types::Action::Moonwalking
        && matches!(
            new.action,
            types::Action::Turning(types::Turn::Ground | types::Turn::MoonwalkJump)
        )
    {
        return (frame.cannon_heading)(old);
    }
    None
}
impl Effect {
    /// Spec: `Dashing`. A nonempty let-go effect ends dashing at the pose change.
    pub(crate) const fn lets_go(self) -> bool {
        matches!(
            self,
            Self::EndSkid | Self::AsRunning | Self::LoseDash | Self::Halt
        )
    }
}
impl Samus {
    /// Spec: `PoseFollows`, `Jumping`, `TurningRound`, `Moonfall`, `ShrinkingLowersHer`.
    /// Evaluate one accepted row or entry effect on a local answer, after fitting and lowering.
    pub(crate) fn apply(&mut self, effect: Effect, start: &Self, t: &Tick) {
        let pose = self.pose;
        match effect {
            Effect::Rise => self.vertical.jump(self.run.dash, t),
            Effect::LiftIfUnaimed if start.pose.aim == Aim::Forward => {
                self.position.y -= t.rules.crouch_lift;
            }
            Effect::TurnRound => self.run.turn_round(pose.facing),
            Effect::TurnRoundIfFacingChanges if pose.facing != start.pose.facing => {
                self.run.turn_round(pose.facing);
                self.run.dashing = Dashing::Off;
            }
            Effect::EndSkid => self.run.end_skid(pose.facing),
            Effect::AsRunning => self.run.as_running(start.run.speed),
            Effect::LoseDash => self.run.dash = ZERO,
            Effect::Halt => self.run.halt(),
            Effect::Moonfall => self.vertical.enter_spin_from_a_turn(self.run.dash, t),
            Effect::Launch => self.run.launch(pose.facing),
            Effect::SpinOn => self.spin = Spin::CarriedOn,
            Effect::ComesToTheGround => self.vertical.come_to_the_ground(),
            Effect::EndsTheBounce => self.vertical.end_bounce(),
            Effect::LiftIfUnaimed | Effect::TurnRoundIfFacingChanges => {}
        }
    }
}
