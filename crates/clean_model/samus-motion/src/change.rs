//! The pose change at a frame's end (`PoseFollows`, `PoseFitAfterMoves`): whether she takes the
//! pose chosen, where pose fit or the shrink places her, then what her moves met and the
//! change's effects.

use crate::chart::Choice;
use crate::collide::Probe::Blocks;
use crate::fit::{Fitted, crouched};
use crate::rows::Effect::{
    self, AsRunning, EndSkid, Halt, Launch, Lift, LoseDash, OnTheGround, Rise, SpinOn, TurnRound,
};
use crate::rules::Lowers;
use crate::samus::{Samus, Spin};
use crate::situation::ENTERING;
use crate::tick::Tick;
use crate::vertical::Contact;
use clean_types::{Action, Aim, Fixed, Parts, px};

impl Samus {
    /// `PoseFollows`, `PoseFitAfterMoves`: what she becomes. Pose fit (or a morph refused) says
    /// whether she takes the chosen pose with what her moves met (`contact`), or ends in the
    /// pose she had or crouched `Still` with neither: her speeds, dash and whether she carries on
    /// or skids stay as her moves left them, her position and seat as her looks left them. What
    /// she becomes brings, in order: after pose fit's push the shrink (`shrink`), then what she
    /// met, then its row's effects, then the change of action's own (`ENTERING`).
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
        self.radius = t.body(became.pose);
        if let Some(lowers) = became.lower {
            self.shrink(lowers, t);
        }
        let placed = (fitted.moved() || became.lower.is_some())
            .then(|| self.position.y.with_sub(start.position.y.sub()));
        if let Some(contact) = contact {
            self.vertical.meet(contact, t);
            self.run.meet(contact);
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
    /// (`lowers`) at her new size, a block look, and is lowered as far as it got, after what
    /// stops it writes to her as a stop would.
    fn shrink(&mut self, lowers: Lowers, t: &mut Tick) {
        let seen = t.vmove(self.bounds(), t.rules.lowers(lowers), Blocks);
        self.end_at(&seen);
        self.position.y += px(seen.gone);
    }

    /// `Jumping`, `TurningRound`, `Moonfall`: one effect of the change (`PoseFollows`) on her
    /// speeds, dash and position, from how she was at the frame's start (`start`). `Jumping`:
    /// changing into a take-off from a crouch without aim she is also lifted. `TurningRound`: a
    /// turn, or a pose that faces her the other way (a spin reversed, the ball turning round),
    /// makes her skid; the moonwalk's jump, the one turn that keeps her facing, skids too, and a
    /// spin reversed while skidding turns her travel round. The ball held to the side it faces
    /// doesn't turn. `WallJump`: a spin that followed a wall jump is carried on.
    fn apply(&mut self, effect: Effect, start: &Self, t: &Tick) {
        let pose = self.pose;
        match effect {
            Rise => self.vertical.jump(t),
            Lift if start.pose.aim == Aim::Forward => self.position.y -= t.rules.crouch_lift,
            TurnRound
                if pose.facing != start.pose.facing
                    || matches!(pose.action, Action::Turning(_)) =>
            {
                self.run.turn_round(pose.facing);
            }
            EndSkid => self.run.end_skid(pose.facing),
            AsRunning => self.run.as_running(start.run.speed),
            LoseDash => self.run.clear_dash(),
            Halt => self.run.halt(),
            Effect::Moonfall => self.vertical.enter_spin_from_a_turn(t),
            Launch => self.run.launch(pose.facing),
            SpinOn => self.spin = Spin::CarriedOn,
            OnTheGround => self.vertical.come_to_the_ground(),
            Lift | TurnRound => {}
        }
    }
}
