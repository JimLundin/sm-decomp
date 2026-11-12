//! One frame of her motion, in the order `PoseFitAfterMoves` gives: she moves as the pose at
//! its start says (`MovesAsShePosed`, `moves`); what those moves met is known; the pose is
//! chosen (`PoseFollows`, `chart`) and the aim change's step made; then the pose change
//! (`change`): pose fit or the shrink places her, and what she met and the change's effects act,
//! where the change is taken.

use crate::chart::{Choice, Moment};
use crate::jump_held::FrameBefore;
use crate::rows::Doing;
use crate::rules::Rules;
use crate::run::Dashing;
use crate::samus::{Outcome, Samus, StandingAnimation, sprite};
use crate::situation::{UnknownPose, classify};
use crate::tick::{Frame, Tick};
use crate::vertical::Contact;
use types::button::JUMP;
use types::{Body, World};

impl Samus {
    /// `PoseFollows`: one frame, in the order it runs. `TouchedCells`: the cells she is inside
    /// come first, from where she starts. At its start the spin counts one frame more (`WallJump`)
    /// and a rise spent or let go ends (`RisingAndFalling`). Then her moves as the pose at its
    /// start says (`MovesAsShePosed`) and what they met, which tells whether she is off the
    /// ground; the pose chosen, the pad chart reading jump held through as a press (`Jumping`),
    /// and the aim change's step; the pose change; at its end, what she remembers of jump held.
    /// `RidingTheElevator`: on the ride she keeps the pose she has, whatever the pad, so no pose
    /// is chosen. `Dashing`: her dashing is read once her moves are made, and again at the
    /// frame's end.
    ///
    /// # Errors
    ///
    /// `UnknownPose`, where no situation names her pose: the frame gives her no motion, and she
    /// is left as she was.
    pub fn step(
        &mut self,
        frame: &Frame,
        world: &dyn World,
        rules: &Rules,
    ) -> Result<Outcome, UnknownPose> {
        let situation = classify(self.pose)?;
        let mut t = Tick {
            frame,
            world,
            rules,
            reached: Vec::new(),
        };
        let inside = t.inside(self.bounds(&t));
        let start = *self;
        self.spin = self.spin.then(situation.doing == Doing::Spinning);
        self.vertical.end_rise(situation.doing, frame.pad);
        let meeting = self.moves(situation, &mut t);
        let dashing_after_moves = self.run.dashing;
        let springs = start.pose.body == Body::SpringBall && frame.pad.any(JUMP);
        let contact = self
            .vertical
            .contact(meeting.met, situation.footing, springs, rules);
        let moment = Moment {
            pad: frame.pad,
            pressed: start.jump_held.pressed(frame.pressed, rules),
            items: frame.items,
            animation_ends: frame.animation_ends,
            speed: start.run.speed,
            airborne: contact.map_or_else(|| !self.vertical.on_ground(), Contact::rises),
            met: meeting.met,
        };
        let chosen = if start.rides_the_elevator(frame) {
            Choice::instead(start.pose)
        } else {
            moment.choose(start.pose, situation)
        };
        let chosen = self.step_forward(chosen, &mut t);
        let placed = self.change_pose(chosen, contact, &start, &mut t);
        let left_behind =
            FrameBefore::left_behind(meeting.met, chosen.handed_over, start.pose, self.pose);
        self.jump_held = start.jump_held.then(rules, frame, left_behind);
        let was_dashing = start.run.dashing == Dashing::On;
        Ok(Outcome {
            wall_test: meeting.wall_test,
            standing_animation: StandingAnimation::of(start.pose, frame.pad),
            sprite: sprite(self.pose, &frame.items),
            skidding: self.run.skids(self.pose.facing),
            reached: t.reached,
            inside,
            placed,
            rising: self.vertical.rising(),
            dashing_after_moves,
            dash_began: !was_dashing && meeting.ran_with_run_held,
            dash_lost: (was_dashing || meeting.ran_with_run_held)
                && self.run.dashing == Dashing::Off,
            handed_over: chosen.handed_over,
            heading_down: meeting.heading_down,
            vertical_speed_after_moves: meeting.vertical_speed,
        })
    }
}
