//! `PoseFollows`: the pose change, once her moves are made. The pose is chosen; the aim
//! change's step comes; pose fit takes the change, refuses it or makes it a crouch; she takes
//! the pose, and a change of shape lowers her; then what her moves met and the change's effects
//! act on her speeds and dash.

use crate::chart::{Choice, Moment};
use crate::collide::{Probe, Stop};
use crate::fit::{Fitted, crouched};
use crate::moves::MovesMade;
use crate::rows::{Effect, Met, Situation};
use crate::rules::Lowers;
use crate::run::Dashing;
use crate::samus::{Samus, Spin};
use crate::situation::ENTERING;
use crate::tick::{Frame, Tick};
use crate::vertical::Contact;
use types::button::JUMP;
use types::{
    Action, Aim, Body, Facing, FiringDirection, Fixed, FloorResponse, FloorResponseKind, Item,
    Parts, Side, VerticalSpeed, ZERO, px,
};

/// What the pose change reports: whether her animation's end handed over the pose chosen, where
/// a placement put her, the firing aim it publishes, the floor response accepted this frame, and
/// what her moves' contact did, once pose fit took it.
pub struct PoseChange {
    /// `AnimationEnds`: her animation's end handed over the pose chosen.
    pub handed_over: bool,
    /// `PoseFitAfterMoves`, `ShrinkingLowersHer`: the y a placement put her at, if any.
    pub placed: Option<Fixed>,
    /// `TransitionFiringAim`: the cannon heading the change publishes, if any.
    pub pending_aim: Option<FiringDirection>,
    /// `AcceptedFloorResponse`: the floor response accepted this frame, if any.
    pub floor_response: Option<FloorResponse>,
    /// `Landing`, `WallJump`: the contact her moves made that she took, if any.
    pub contact: Option<Contact>,
}

impl Samus {
    /// `PoseFollows`: the pose change, from `start`, her at the frame's start, after her moves
    /// (`moves`). The pose is chosen; a running pose other than hers brings the aim change's
    /// step (`AimingAndShooting`); pose fit takes the pose, refuses it or takes the crouch
    /// instead (`PoseFitAfterMoves`), and none of the chosen pose's effects come with the last
    /// two. She takes the pose, a change of shape lowers her (`ShrinkingLowersHer`) and a
    /// placement puts her; then her moves' contact acts (`Landing`, `WallJump`), and then the
    /// change's effects (`PadChart`, `AnimationEnds`).
    pub(crate) fn take_pose(
        &mut self,
        start: &Self,
        situation: &'static Situation,
        moves: &MovesMade,
        t: &mut Tick,
    ) -> PoseChange {
        let springs = start.pose.body == Body::SpringBall && t.frame.pad.any(JUMP);
        let contact = self
            .vertical
            .contact(moves.met, situation.footing, springs, t.rules);
        let chosen = self.choose(start, situation, moves.met, contact, t);
        let stepped = if chosen.aim_step {
            self.step_forward(start, chosen, t)
        } else {
            chosen
        };
        let (accepted, fitted) = self.fit_choice(stepped, t);
        let contact = contact.filter(|_| fitted.takes_the_change());
        self.pose = accepted.pose;
        let pending_aim = transition_firing_aim(start.pose, accepted.pose, t.frame);
        if let Some(lower) = accepted.lower {
            self.lower(lower, t);
        }
        let placed = (fitted.moved() || accepted.lower.is_some())
            .then(|| self.position.y.with_sub(start.position.y.sub()));
        let response = contact.and_then(|c| self.respond(c, start, t));
        self.apply_effects(start, accepted, t);
        PoseChange {
            handed_over: chosen.handed_over,
            placed,
            pending_aim,
            floor_response: response.or(moves.floor_stop),
            contact,
        }
    }

    /// `PoseFollows`: the pose chosen from her pose at the frame's start: on the elevator's ride
    /// the pose she has (`RidingTheElevator`); else an animation's end, what her moves met or
    /// the pad chart, read with jump held through as a press (`Jumping`), her horizontal speed
    /// at the frame's start, and whether her moves and their contact leave her off the ground.
    fn choose(
        &self,
        start: &Self,
        situation: &'static Situation,
        met: Option<Met>,
        contact: Option<Contact>,
        t: &Tick,
    ) -> Choice {
        let frame = t.frame;
        if start.rides_the_elevator(frame) {
            return Choice::instead(start.pose);
        }
        let moment = Moment {
            pad: frame.pad,
            pressed: start.jump_held.pressed(frame.pressed, t.rules),
            items: frame.items,
            animation_ends: frame.animation_ends,
            speed: start.run.speed,
            airborne: contact.map_or_else(|| !self.vertical.on_ground(), Contact::rises),
            met,
        };
        moment.choose(start.pose, situation)
    }

    /// `AimingAndShooting`: the aim change's step, forward by the facing she had at the frame's
    /// start, as a horizontal move. A solid enemy within its reach stops it with no move at all;
    /// a block stops it as any move; either in the way puts her against the wall instead.
    fn step_forward(&mut self, start: &Self, chosen: Choice, t: &mut Tick) -> Choice {
        let before = self.position;
        let heading = Side::of(start.pose.facing).heading();
        let (from, by, probe) = (
            self.bounds(t.frame),
            t.rules.aim_step,
            self.horizontal_probe(),
        );
        let moved = t.sideways(from, heading, by, probe);
        self.position = moved.to;
        if moved.stop == Some(Stop::Enemy) {
            self.position.x = before.x;
        }
        if moved.stop.is_some() {
            chosen.blocked()
        } else {
            chosen
        }
    }

    /// `PoseFollows`, `PoseFitAfterMoves`, `AnimationEnds`: the choice she takes, and what pose
    /// fit did. A handed-over pose, the pose she is in or one facing the screen is taken as it
    /// is. A change into `Morphing` without the morph ball is refused. Else pose fit decides; a
    /// change refused leaves her the pose she had, and the crouch taken instead is crouched
    /// `Still`, each with none of the chosen pose's effects.
    fn fit_choice(&mut self, chosen: Choice, t: &mut Tick) -> (Choice, Fitted) {
        if chosen.handed_over || chosen.pose == self.pose || chosen.pose.facing == Facing::Screen {
            return (chosen, Fitted::Taken);
        }
        let fitted = if chosen.pose.action == Action::Morphing && !t.frame.items[Item::MorphBall] {
            Fitted::Refused
        } else {
            self.fit(chosen.pose, t)
        };
        let accepted = match fitted {
            Fitted::Taken | Fitted::Pushed => chosen,
            Fitted::Refused => Choice::instead(self.pose),
            Fitted::Crouched { .. } => Choice::instead(crouched(self.pose.facing)),
        };
        (accepted, fitted)
    }

    /// `ShrinkingLowersHer`: the change of shape's look down its own distance at her new size,
    /// a block look as pose fit's are; she is lowered by the room it finds, in whole pixels.
    fn lower(&mut self, lower: Lowers, t: &mut Tick) {
        let (from, by) = (self.bounds(t.frame), t.rules.lowers(lower));
        let looked = t.up_or_down(from, by, Probe::Blocks);
        self.take_result(&looked);
        self.position.y += px(looked.gone);
    }

    /// `Landing`, `WallJump`, `AcceptedFloorResponse`: her moves' contact acts on her motion,
    /// and a floor's reports its response with her vertical speed just before it: an upright
    /// touchdown, the ball settling, the spring ball's hop, or a bounce with its count.
    fn respond(&mut self, contact: Contact, start: &Self, t: &Tick) -> Option<FloorResponse> {
        let kind = match contact {
            Contact::WallKick => None,
            Contact::Hop => Some(FloorResponseKind::SpringHop),
            Contact::Bounce(bounce) => Some(FloorResponseKind::Rebound(bounce.then)),
            Contact::Land if start.pose.body == Body::Upright => {
                Some(FloorResponseKind::UprightTouchdown)
            }
            Contact::Land => Some(FloorResponseKind::BallSettles),
        };
        let response = kind.map(|kind| FloorResponse {
            kind,
            speed: VerticalSpeed(self.vertical.speed),
        });
        self.vertical.meet(contact, self.run.dash, t);
        self.run.meet(contact);
        response
    }

    /// `PoseFollows`, `Dashing`: the change's effects, in order: its row's, then those of the
    /// change of action itself. A let-go effect among them ends her dashing first.
    fn apply_effects(&mut self, start: &Self, accepted: Choice, t: &Tick) {
        if accepted.effects.iter().any(|e| e.lets_go()) {
            self.run.dashing = Dashing::Off;
        }
        let entered = ENTERING
            .iter()
            .filter(|e| e.on(start.pose.action, accepted.pose))
            .map(|e| e.effect);
        for effect in accepted.effects.iter().copied().chain(entered) {
            self.apply(effect, start, t);
        }
    }
}

/// `TransitionFiringAim`: the cannon heading the pose she takes publishes, from her pose at
/// the frame's start (`old`): an upright straight jump entered with shoot pressed publishes the
/// jump pose's heading; a ground turn from the moonwalk, its jump's included, the moonwalk's.
/// Any other change, or none, publishes nothing.
pub fn transition_firing_aim(
    old: types::Pose,
    new: types::Pose,
    frame: &Frame,
) -> Option<FiringDirection> {
    if old == new {
        return None;
    }
    if new.body == Body::Upright
        && new.action == Action::Jumping
        && frame.pressed.any(types::button::SHOOT)
    {
        return (frame.cannon_heading)(new);
    }
    if old.action == Action::Moonwalking
        && matches!(
            new.action,
            Action::Turning(types::Turn::Ground | types::Turn::MoonwalkJump)
        )
    {
        return (frame.cannon_heading)(old);
    }
    None
}

impl Effect {
    /// `Dashing`: a let-go effect other than none ends her dashing at the pose change.
    pub(crate) const fn lets_go(self) -> bool {
        matches!(
            self,
            Self::EndSkid | Self::AsRunning | Self::LoseDash | Self::Halt
        )
    }
}
impl Samus {
    /// `PoseFollows`, `Jumping`, `TurningRound`, `Moonfall`, `ShrinkingLowersHer`: one of the
    /// change's effects on her speeds, dash and position, after pose fit and the lowering.
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
