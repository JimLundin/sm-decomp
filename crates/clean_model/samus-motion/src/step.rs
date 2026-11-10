//! One frame of her motion, in the order `PoseFitAfterMoves` gives: she moves as the pose at
//! its start says (`MovesAsShePosed`, `moves`); what those moves met is known; the pose is
//! chosen (`PoseFollows`, `chart`) and the aim change's step made; then the pose change
//! (`change`): pose fit or the shrink places her, and what she met and the change's effects act,
//! where the change is taken.

use crate::chart::Moment;
use crate::jump_held::FrameBefore;
use crate::rows::Doing::Spinning;
use crate::rules::Rules;
use crate::samus::{Outcome, Samus, Sprite, StandingAnimation};
use crate::situation::{UnknownPose, classify};
use crate::tick::{Frame, Tick};
use crate::vertical::Contact;
use clean_types::Body::SpringBall;
use clean_types::button::JUMP;
use clean_types::{Item, World};

impl Samus {
    /// `PoseFollows`: one frame, in the order it runs. `TouchedCells`: the cells she is inside
    /// come first, from where she starts. At its start the spin counts one frame more (`WallJump`)
    /// and a rise spent or let go ends (`RisingAndFalling`). Then her moves as the pose at its
    /// start says (`MovesAsShePosed`) and what they met, which tells whether she is off the
    /// ground; the pose chosen, the pad chart reading jump held through as a press (`Jumping`),
    /// and the aim change's step; the pose change; at its end, what she remembers of jump held.
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
        self.spin = self.spin.then(situation.doing == Spinning);
        self.vertical.end_rise(situation.doing, frame.pad);
        let meeting = self.moves(situation, &mut t);
        let springs = start.pose.body == SpringBall && frame.pad.any(JUMP);
        let contact = self
            .vertical
            .contact(meeting.met, situation.footing, springs, rules);
        let moment = Moment {
            pad: frame.pad,
            pressed: start.jump_held.pressed(frame.pressed, rules),
            spring_ball: frame.items[Item::SpringBall],
            animation_ends: frame.animation_ends,
            speed: start.run.speed,
            airborne: contact.map_or_else(|| !self.vertical.on_ground(), Contact::rises),
            met: meeting.met,
        };
        let chosen = moment.choose(start.pose, situation);
        let chosen = self.step_forward(chosen, &mut t);
        let placed = self.change_pose(chosen, contact, &start, &mut t);
        let left = FrameBefore::left(meeting.met, frame.animation_ends, start.pose, self.pose);
        self.jump_held = start.jump_held.then(rules, frame, left);
        Ok(Outcome {
            wall_test: meeting.wall_test,
            standing_animation: StandingAnimation::of(start.pose, frame.pad),
            sprite: Sprite::of(self.pose, &frame.items),
            skidding: self.run.skids(self.pose.facing),
            reached: t.reached,
            inside,
            placed,
            rising: self.vertical.rising(),
        })
    }
}
