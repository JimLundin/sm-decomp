//! One whole frame, in its two steps (§12, the frame's two steps; `PoseFollows`,
//! `MovesAsShePosed`). Her moves: the cells she is inside; what she does and how she moves, as
//! the pose she starts it in says; what her moves met; and the pose they, the pad and her
//! animation's end ask for. Then, once the frame's hit is known, her reaction: the hit's push
//! time, the push's start in place of that pose or the pose itself taken, the push's end, and
//! what she remembers for the next frame.

use crate::change::{PoseChange, Selection};
use crate::chart::{Choice, Decider};
use crate::getaway::{DeferredMotion, Getaway};
use crate::jump_held::{FrameBefore, JumpButton};
use crate::knockback::{Recoil, recoiling};
use crate::moves::MovesMade;
use crate::rows::{FORWARD, seen};
use crate::rules::Rules;
use crate::run::Dashing;
use crate::samus::{Outcome, Samus, StandingAnimation, sprite};
use crate::situation::{UnknownPose, classify};
use crate::tick::{Frame, Tick};
use crate::touchdown::opportunities;
use crate::vertical::Contact;
use core::num;
use types::{Action, Cell, Hit, Parts, Pose, Position, Px, Side, World, px};

const VIEW_CENTRE: Px = num::Wrapping(128);
const FIT_HEIGHT: Px = num::Wrapping(21);

/// `GetawayTravels`: discard a backward leap together with its command, not other candidates.
fn getaway_choice(had: Pose, candidate: Option<Choice>) -> Choice {
    candidate
        .filter(|choice| choice.pose.action != Action::Leaping)
        .unwrap_or_else(|| Choice::instead(had))
}

/// The first of a frame's two steps done: her moves made, and the pose they, the pad and her
/// animation's end ask for chosen, not yet taken (`PoseFollows`).
///
/// The hitter judges the frame's hit from `position_after_moves` and the body of the pose she
/// started the frame in; `Samus::react` then plays the rest of the same frame from here. Until
/// it does, the caller's `Samus` is still her State at the frame's start.
pub struct HerMoves<'a> {
    /// The frame, its room and rules, and the cells her moves reached.
    tick: Tick<'a>,
    /// She, at the frame's start.
    start: Samus,
    /// She, as her moves and what they met leave her.
    her: Samus,
    /// What her moves left for the rest of the frame.
    made: MovesMade,
    /// The pose chosen, and the contact her moves made that comes with it.
    selection: Selection,
    /// `TouchedCells`: the cells she was inside at the frame's start.
    inside: Vec<Cell>,
}

impl HerMoves<'_> {
    /// `GetawayTravels`: supply the ordinary candidate and its effects at the resolver seam
    /// on a travel cut. `None` clears the candidate; a repeated pose is still a choice.
    /// A backward-leap candidate is discarded along with its effects. Other cuts ignore this.
    pub fn supply_getaway_candidate(&mut self, candidate: Option<Choice>) {
        if matches!(self.start.getaway, Getaway::Travelling(_)) {
            self.selection.choice = getaway_choice(
                self.start.pose,
                candidate.map(|mut choice| {
                    choice.decided_by = Decider::GetawayCandidate;
                    choice
                }),
            );
        }
    }

    /// §1, `MovesAsShePosed`: her position after her moves, before the aim change's step and the
    /// pose change move her: what the hitter judges the frame's hit from.
    #[must_use]
    pub const fn position_after_moves(&self) -> Position {
        self.made.left.position
    }
}

impl Samus {
    /// The frame's first step, her moves (one 60 Hz frame, `frame`, in `world` under `rules`).
    ///
    /// `TouchedCells`: first the cells she is inside, from where she starts. Then her moves, as
    /// the pose she starts the frame in says (`MovesAsShePosed`), and the pose chosen, from what
    /// her moves met, the pad and her animation's end (`PoseFollows`). She is left as she was;
    /// `react` plays the rest of the frame.
    ///
    /// # Errors
    ///
    /// `UnknownPose`, where no situation names the pose she starts in.
    pub fn moves<'a>(
        &self,
        frame: &'a Frame<'a>,
        world: &'a dyn World,
        rules: &'a Rules,
    ) -> Result<HerMoves<'a>, UnknownPose> {
        let mut tick = Tick::new(frame, world, rules);
        let inside = tick.inside(self.bounds(frame));
        let mut her = *self;
        let (made, selection) = match self.getaway {
            Getaway::Idle => {
                let situation = classify(self.pose)?;
                let made = her.make_moves(situation, &mut tick);
                let selection = her.choose_pose(self, situation, &made, &tick);
                (made, selection)
            }
            Getaway::Pending => (
                MovesMade::still(her),
                Selection::only(recoiling(self.pose.facing)),
            ),
            Getaway::Travelling(side) => {
                let situation = classify(self.pose)?;
                let made = her.getaway_moves(side, &mut tick);
                let mut selection = her.choose_pose(self, situation, &made, &tick);
                selection.choice = getaway_choice(self.pose, Some(selection.choice));
                (made, selection)
            }
        };
        Ok(HerMoves {
            tick,
            start: *self,
            her,
            made,
            selection,
            inside,
        })
    }

    /// The frame's second step, her reaction once the frame's `hit` is known (none, or one),
    /// after her `moves`: she becomes her State at the frame's end, and the frame's outputs come
    /// out.
    ///
    /// The hit's push time (`PushTime`); a push's start in place of the pose chosen
    /// (`PushStarts`), or a push's end keeping her pose (`PushEnds`); the pose change
    /// (`PoseFollows`); then jump held through counts on (`Jumping`), the push time counts down,
    /// and the touchdown's requests follow from the floor response she took
    /// (`TouchdownRequests`).
    ///
    /// For a frame that follows the previous history-writing draw directly: the frame's pad
    /// tells that draw's jump (`Jumping`). After a passage's draws, `react_after_draw`.
    pub fn react(&mut self, moves: HerMoves<'_>, hit: Option<Hit>) -> Outcome {
        let previous = JumpButton::previous_draw(moves.tick.frame);
        self.react_after_draw(moves, hit, previous)
    }

    /// The frame's second step as `react` plays it, with jump held through (`Jumping`) counted
    /// on against `last_draw`, the jump of the preceding draw that updated jump history, as a
    /// passage's `DrawnJumpHistory` keeps it: jump held now and then counts one more, jump up
    /// on either puts the count at 0, whatever the controller reported just pressed. The frame's
    /// own buttons, the pressed ones included, still decide her controls and her pose, and a
    /// landing or an animation's end the frame before still makes jump held a press and puts
    /// the count back to 0 first.
    pub fn react_after_draw(
        &mut self,
        moves: HerMoves<'_>,
        hit: Option<Hit>,
        last_draw: JumpButton,
    ) -> Outcome {
        let HerMoves {
            mut tick,
            start,
            mut her,
            made,
            selection,
            inside,
        } = moves;
        let (frame, rules) = (tick.frame, tick.rules);
        let (change, push_started, deferred_motion) = match start.getaway {
            Getaway::Idle => {
                let (selection, push_started) = her.meet_the_hit(&start, hit, selection, frame);
                let change = her.take_pose(&start, &made, selection, &mut tick);
                her.after_the_change(&start, push_started, rules);
                // `GetawayTravels`, `AfterThePush`: only this first ordinary cut can use
                // the saved speed, even if its pose moved without it.
                her.getaway_release_speed = None;
                (change, push_started, DeferredMotion::Keep)
            }
            Getaway::Pending => (
                her.start_getaway(selection.choice.pose, frame),
                false,
                DeferredMotion::Clear,
            ),
            Getaway::Travelling(_) => {
                // `GetawayTravels`: contact and any stop cleanup are already reflected in
                // `her`. Resolve and record the retained choice once, from that position.
                let change = her.take_pose(&start, &made, selection, &mut tick);
                (change, false, DeferredMotion::DiscardBackwardLeap)
            }
        };
        let left = FrameBefore::left_behind(made.met, change.handed_over, start.pose, her.pose);
        her.jump_held = start.jump_held.then(rules, frame, last_draw, left);
        her.knockback.count_down();
        *self = her;
        let kicked = change.contact == Some(Contact::WallKick);
        let (touchdown_requests, touchdown_graphics) =
            opportunities(change.floor_response, kicked, start.pose.action, frame);
        let dashed = start.run.dashing == Dashing::On || made.ran;
        Outcome {
            pose_change: change.pose,
            deferred_motion,
            floor_response: change.floor_response,
            touchdown_requests,
            touchdown_graphics,
            pending_aim: change.pending_aim,
            travelled: made.travelled,
            wall_test: made.wall_test,
            recorded_choice: change.recorded,
            standing_animation: if start.getaway == Getaway::Idle {
                StandingAnimation::of(start.pose, frame.pad)
            } else {
                StandingAnimation::PlaysOn
            },
            sprite: sprite(self.pose, &frame.items),
            skidding: self.run.skids(self.pose.facing),
            reached: tick.reached,
            inside,
            placed: change.placed,
            position_after_moves: made.left.position,
            rising: self.vertical.rising(),
            dashing_after_moves: made.left.run.dashing,
            dash_began: start.run.dashing == Dashing::Off && made.ran,
            dash_lost: dashed && self.run.dashing == Dashing::Off,
            handed_over: change.handed_over,
            heading_down: made.left.vertical.heading_down(),
            vertical_speed_after_moves: made
                .left
                .getaway
                .downward_speed()
                .unwrap_or(made.left.vertical.speed),
            push_started,
        }
    }

    /// `GetawayArms`: call after the ordinary cut's reaction. The request leaves that cut's
    /// outputs, including its travel, untouched.
    pub const fn request_getaway(&mut self) {
        if matches!(self.getaway, Getaway::Idle) {
            self.getaway = Getaway::Pending;
        }
    }

    /// `GetawayStarts`: lift without a movement query, using the old body's height, then
    /// publish the forced recoil selection and remember it.
    fn start_getaway(&mut self, pose: Pose, frame: &Frame) -> PoseChange {
        let relative = self.position.x.pixel() - frame.view_left - VIEW_CENTRE;
        self.getaway = Getaway::Travelling(if relative.0.cast_signed() < 0 {
            Side::Left
        } else {
            Side::Right
        });
        let lift = FIT_HEIGHT - (frame.body)(self.pose).y;
        self.position.y -= px(lift);
        self.pose = pose;
        PoseChange {
            pose: Some(pose),
            handed_over: false,
            placed: (lift.0 != 0).then_some(self.position.y),
            pending_aim: None,
            floor_response: None,
            contact: None,
            recorded: self.choices.record(Some(pose)),
        }
    }

    /// `PushTime`, `PushStarts`, `PushEnds`: the frame's hit gives its push time and side; then
    /// what the pose chosen gives way to, from `start`, her at the frame's start. A frame she
    /// starts with a push under way and no push time left keeps the pose she started in, and
    /// takes no pose the frame chose nor any of its effects: not the pad's row, not its let-go
    /// (her motion stays as her moves left it), nor a landing on a floor her moves met. Else,
    /// where the hit brings a push and the pose she began in can take one, the push starts, driven down holding forward
    /// and otherwise thrown up: upright or crouched she takes `Recoiling` in place of the pose
    /// chosen, and the ball keeps the pose chosen. Returns the selection she takes and whether a
    /// push started.
    fn meet_the_hit(
        &mut self,
        start: &Self,
        hit: Option<Hit>,
        selection: Selection,
        frame: &Frame,
    ) -> (Selection, bool) {
        self.knockback.take(hit);
        if start.knockback.ends() {
            return (Selection::only(start.pose), false);
        }
        let recoil = Recoil::of(start.pose);
        if !self.knockback.due() || recoil == Recoil::Waits {
            return (selection, false);
        }
        let forward_held = seen(frame.pad, start.pose.facing).any(FORWARD);
        self.knockback.push = Some(recoil.push(forward_held));
        let taken = if recoil == Recoil::Recoils {
            Selection::only(recoiling(start.pose.facing))
        } else {
            selection
        };
        (taken, true)
    }

    /// `PushStarts`, `PushEnds`: after the pose change, a push that started this frame starts
    /// her rising at the push's rise, her horizontal motion as it was; a push whose time ran out
    /// is over, and she is handed back falling at 0 (§1.2), whatever her motion did.
    fn after_the_change(&mut self, start: &Self, push_started: bool, rules: &Rules) {
        if push_started {
            self.vertical.recoil(rules);
        }
        if start.knockback.ends() {
            self.knockback.push = None;
            self.vertical.handed_back();
        }
    }
}
