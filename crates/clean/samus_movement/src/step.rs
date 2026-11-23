//! One whole frame, in the order the spec gives it (`PoseFollows`, `MovesAsShePosed`): the
//! cells she is inside; what she does and how she moves, as the pose she starts it in says;
//! what her moves met; the pose she takes; and what she remembers for the next frame.

use crate::jump_held::FrameBefore;
use crate::rules::Rules;
use crate::run::Dashing;
use crate::samus::{Outcome, Samus, StandingAnimation, sprite};
use crate::situation::{UnknownPose, classify};
use crate::tick::{Frame, Tick};
use crate::touchdown::opportunities;
use crate::vertical::Contact;
use types::World;

impl Samus {
    /// Plays one 60 Hz frame: `frame` goes in, she becomes her State at the frame's end, and
    /// the frame's outputs come out.
    ///
    /// `TouchedCells`: first the cells she is inside, from where she starts. Then her moves, as
    /// the pose she starts the frame in says (`MovesAsShePosed`). Then the pose change, from
    /// what her moves met (`PoseFollows`). Then jump held through counts on (`Jumping`), and
    /// the touchdown's requests follow from the floor response she took (`TouchdownRequests`).
    ///
    /// # Errors
    ///
    /// `UnknownPose`, where no situation names the pose she starts in; she is left as she was.
    pub fn step(
        &mut self,
        frame: &Frame,
        world: &dyn World,
        rules: &Rules,
    ) -> Result<Outcome, UnknownPose> {
        let situation = classify(self.pose)?;
        let start = *self;
        let mut tick = Tick::new(frame, world, rules);
        let inside = tick.inside(self.bounds(frame));
        let moves = self.make_moves(situation, &mut tick);
        let change = self.take_pose(&start, situation, &moves, &mut tick);
        let left = FrameBefore::left_behind(moves.met, change.handed_over, start.pose, self.pose);
        self.jump_held = start.jump_held.then(rules, frame, left);
        let kicked = change.contact == Some(Contact::WallKick);
        let (touchdown_requests, touchdown_graphics) =
            opportunities(change.floor_response, kicked, start.pose.action, frame);
        let dashed = start.run.dashing == Dashing::On || moves.ran;
        Ok(Outcome {
            floor_response: change.floor_response,
            touchdown_requests,
            touchdown_graphics,
            pending_aim: change.pending_aim,
            travelled: moves.travelled,
            wall_test: moves.wall_test,
            standing_animation: StandingAnimation::of(start.pose, frame.pad),
            sprite: sprite(self.pose, &frame.items),
            skidding: self.run.skids(self.pose.facing),
            reached: tick.reached,
            inside,
            placed: change.placed,
            position_after_moves: moves.left.position,
            rising: self.vertical.rising(),
            dashing_after_moves: moves.left.run.dashing,
            dash_began: start.run.dashing == Dashing::Off && moves.ran,
            dash_lost: dashed && self.run.dashing == Dashing::Off,
            handed_over: change.handed_over,
            heading_down: moves.left.vertical.heading_down(),
            vertical_speed_after_moves: moves.left.vertical.speed,
        })
    }
}
