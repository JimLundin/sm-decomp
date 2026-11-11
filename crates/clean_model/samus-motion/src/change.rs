//! The pose change at a frame's end (`PoseFollows`, `PoseFitAfterMoves`): whether she takes the
//! pose chosen, where pose fit or the shrink places her, then what her moves met and the
//! change's effects.

use crate::chart::Choice;
use crate::collide::Probe::Blocks;
use crate::fit::{Fitted, crouched};
use crate::rows::Effect::{
    self, AsRunning, ComesToTheGround, EndSkid, Halt, Launch, LiftIfUnaimed, LoseDash, Rise,
    SpinOn, TurnRound, TurnRoundIfFacingChanges,
};
use crate::rules::Lowers;
use crate::samus::{Dashing, Samus, Spin};
use crate::situation::ENTERING;
use crate::tick::Tick;
use crate::vertical::Contact;
use clean_types::{Aim, Fixed, Parts, ZERO, px};

impl Effect {
    /// `Dashing`: whether it is a let-go effect other than none (she follows the pad again, as
    /// running, dash 0, or a halt). Taken from the pad chart at the pose change, one ends her
    /// dashing; the same dash 0 that a turn's end hands over doesn't.
    const fn lets_go(self) -> bool {
        matches!(self, EndSkid | AsRunning | LoseDash | Halt)
    }
}

impl Samus {
    /// `PoseFollows`, `PoseFitAfterMoves`: what she becomes. Pose fit (or a morph refused) says
    /// whether she takes the chosen pose with what her moves met (`contact`), or ends in the
    /// pose she had or crouched `Still` with neither: her speeds, dash and whether she carries on
    /// or skids stay as her moves left them, her position and seat as her looks left them. What
    /// she becomes brings, in order: after pose fit's push the shrink (`shrink`), then what she
    /// met, then its row's effects, then the change of action's own (`ENTERING`). `Dashing`: the
    /// pad chart's let-go effect ends her dashing. `RidingTheElevator`: on the ride nothing else
    /// of hers changes, so none of this comes.
    ///
    /// §1, `PoseFitAfterMoves`, `ShrinkingLowersHer`: where pose fit moved her, or a change of
    /// shape starts (a lowering of 0 px included), she is placed at her y pixel just after that,
    /// before the effects, with the subpixel her y had at the frame's start. Returns that y, the
    /// placement, or none on any other frame.
    pub(crate) fn change_pose(
        &mut self,
        chosen: Choice,
        contact: Option<Contact>,
        start: &Self,
        t: &mut Tick,
    ) -> Option<Fixed> {
        let fitted = self.fit(t, chosen.pose);
        let (became, contact) = match fitted {
            Fitted::Taken | Fitted::Pushed => (chosen, contact),
            Fitted::Refused => (Choice::instead(self.pose), None),
            Fitted::Crouched { .. } => (Choice::instead(crouched(self.pose.facing)), None),
        };
        self.pose = became.pose;
        if let Some(lowers) = became.lower {
            self.shrink(lowers, t);
        }
        let placed = (fitted.moved() || became.lower.is_some())
            .then(|| self.position.y.with_sub(start.position.y.sub()));
        if start.rides_the_elevator(t.frame) {
            return placed;
        }
        if let Some(contact) = contact {
            self.vertical.meet(contact, self.run.dash, t);
            self.run.meet(contact);
        }
        if !became.handed_over && became.effects.iter().any(|e| e.lets_go()) {
            self.dashing = Dashing::Off;
        }
        let entered = ENTERING
            .iter()
            .filter(|e| e.on(start.pose.action, became.pose));
        for effect in became.effects.iter().chain(entered.map(|e| &e.effect)) {
            self.apply(*effect, start, t);
        }
        placed
    }

    /// `ShrinkingLowersHer`: the frame a change of shape starts, she looks down its distance
    /// (`lowers`) at her new size, a block look, and is lowered as far as it got.
    fn shrink(&mut self, lowers: Lowers, t: &mut Tick) {
        let look = t.vmove(self.bounds(t), t.rules.lowers(lowers), Blocks);
        // What stopped the look writes her y subpixel FFFF, or seats her on a stepped block.
        self.take_result(&look);
        self.position.y += px(look.gone);
    }

    /// `Jumping`, `TurningRound`, `Moonfall`: one effect of the change (`PoseFollows`) on her
    /// speeds, dash and position, from how she was at the frame's start (`start`). `Jumping`:
    /// changing into a take-off from a crouch without aim she is also lifted; a rise at the jump
    /// speed takes its share of her dash as the frame's moves left it. `TurningRound`: a turn
    /// makes her skid, the moonwalk's jump (the one turn that keeps her facing) too; a pose that
    /// faces her the other way (a spin reversed, the ball turning round) makes her skid and ends
    /// her dashing, and a spin reversed while skidding turns her travel round. `WallJump`: a spin
    /// that followed a wall jump is carried on.
    fn apply(&mut self, effect: Effect, start: &Self, t: &Tick) {
        let pose = self.pose;
        match effect {
            Rise => self.vertical.jump(self.run.dash, t),
            LiftIfUnaimed if start.pose.aim == Aim::Forward => {
                self.position.y -= t.rules.crouch_lift;
            }
            TurnRound => self.run.turn_round(pose.facing),
            TurnRoundIfFacingChanges if pose.facing != start.pose.facing => {
                self.run.turn_round(pose.facing);
                self.dashing = Dashing::Off;
            }
            EndSkid => self.run.end_skid(pose.facing),
            AsRunning => self.run.as_running(start.run.speed),
            LoseDash => self.run.dash = ZERO,
            Halt => self.run.halt(),
            Effect::Moonfall => self.vertical.enter_spin_from_a_turn(self.run.dash, t),
            Launch => self.run.launch(pose.facing),
            SpinOn => self.spin = Spin::CarriedOn,
            ComesToTheGround => self.vertical.come_to_the_ground(),
            LiftIfUnaimed | TurnRoundIfFacingChanges => {}
        }
    }
}
