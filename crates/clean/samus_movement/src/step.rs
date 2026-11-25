//! One whole frame, in its two steps (§12, the frame's two steps; `PoseFollows`,
//! `MovesAsShePosed`). Her moves: the cells she is inside; what she does and how she moves, as
//! the pose she starts it in says; what her moves met; and the pose they, the pad and her
//! animation's end ask for. Then, once the frame's hit is known, her reaction: the hit's push
//! time, the push's start in place of that pose or the pose itself taken, the push's end, and
//! what she remembers for the next frame.

use crate::change::Selection;
use crate::jump_held::FrameBefore;
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
use types::{Cell, Hit, Position, World};

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
        let situation = classify(self.pose)?;
        let mut tick = Tick::new(frame, world, rules);
        let inside = tick.inside(self.bounds(frame));
        let mut her = *self;
        let made = her.make_moves(situation, &mut tick);
        let selection = her.choose_pose(self, situation, &made, &tick);
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
    pub fn react(&mut self, moves: HerMoves<'_>, hit: Option<Hit>) -> Outcome {
        let HerMoves {
            mut tick,
            start,
            mut her,
            made,
            selection,
            inside,
        } = moves;
        let (frame, rules) = (tick.frame, tick.rules);
        let (selection, push_started) = her.meet_the_hit(&start, hit, selection, frame);
        let change = her.take_pose(&start, &made, selection, &mut tick);
        her.after_the_change(&start, push_started, rules);
        let left = FrameBefore::left_behind(made.met, change.handed_over, start.pose, her.pose);
        her.jump_held = start.jump_held.then(rules, frame, left);
        her.knockback.count_down();
        *self = her;
        let kicked = change.contact == Some(Contact::WallKick);
        let (touchdown_requests, touchdown_graphics) =
            opportunities(change.floor_response, kicked, start.pose.action, frame);
        let dashed = start.run.dashing == Dashing::On || made.ran;
        Outcome {
            floor_response: change.floor_response,
            touchdown_requests,
            touchdown_graphics,
            pending_aim: change.pending_aim,
            travelled: made.travelled,
            wall_test: made.wall_test,
            standing_animation: StandingAnimation::of(start.pose, frame.pad),
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
            vertical_speed_after_moves: made.left.vertical.speed,
            push_started,
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
