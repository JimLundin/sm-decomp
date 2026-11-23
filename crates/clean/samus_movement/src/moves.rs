//! `MovesAsShePosed`: a frame's moves, as the pose she starts it in says: her horizontal move
//! and the slope checks after it, the near-wall test, then the ground pull or her vertical
//! move; facing the screen, the elevator's descent or nothing.

use crate::collide::{Moved, Probe, Stop, vertical_heading};
use crate::rows::{Doing, Footing, Met, Moves, Situation};
use crate::run::{Dashing, Stride, running_with_run_held};
use crate::samus::{Samus, Seat, Spin, WallTest};
use crate::tick::Tick;
use crate::travel::TravelledDistances;
use crate::vertical::Course;
use types::button::{JUMP, LEFT, SIDES};
use types::{
    Action, DirectionalTravels, Facing, Fixed, FloorResponse, Heading, TravelDistance, ZERO, px,
};

/// What her moves leave for the rest of the frame: what they met, the floor response a stop
/// within them made, the near-wall test's outcome, how far each move got, whether she ran with
/// run held, and Samus as they leave her, before what they met acts.
pub struct MovesMade {
    /// `WhatHerMovesMet`: what her moves met, which can choose her next pose.
    pub met: Option<Met>,
    /// `AcceptedFloorResponse`: the response a floor made within her vertical move.
    pub floor_stop: Option<FloorResponse>,
    /// `WallJump`: the near-wall test's outcome.
    pub wall_test: WallTest,
    /// `TravelledDistances`: each ordinary move's signed travel, 0 where she made none.
    pub travelled: DirectionalTravels,
    /// `Dashing`: her moves were a run with run held, which starts her dashing.
    pub ran: bool,
    /// Samus as her moves leave her, before what they met acts on her vertical motion: the
    /// §1.4 outputs and her position after her moves are read from her here.
    pub left: Samus,
}

impl Samus {
    /// `MovesAsShePosed`: the frame's moves. At the frame's start a rise ends where it does
    /// (`RisingAndFalling`) and the spin counts on (`WallJump`). Facing the screen she makes
    /// no move of her own, but a moving elevator carries her down (`RidingTheElevator`).
    /// `WhatHerMovesMet`: a wall stop while running wins over walking off a ledge, once that
    /// walk off has acted on her vertical motion.
    pub(crate) fn make_moves(&mut self, situation: &Situation, t: &mut Tick) -> MovesMade {
        self.vertical.end_rise(situation.doing, t.frame.pad);
        self.spin = self.spin.then(situation.doing == Doing::Spinning);
        let mut made = MovesMade {
            met: None,
            floor_stop: None,
            wall_test: WallTest::Neither,
            travelled: TravelledDistances::fresh().travels,
            ran: false,
            left: *self,
        };
        let walled = if self.pose.facing == Facing::Screen {
            if self.rides_the_elevator(t.frame) {
                self.ride_down(&mut made, t);
            }
            false
        } else {
            self.move_as_posed(situation, &mut made, t)
        };
        made.left = *self;
        self.vertical.meet_in_moves(made.met);
        if walled && self.pose.action == Action::Running {
            made.met = Some(Met::Wall);
        }
        made
    }

    /// `SpeedBuildsAndBrakes`, `Dash`, `Dashing`, `WallJump`, `Jumping`: her moves as her
    /// situation says: her dash, then her speed, then her horizontal move; her moves as standing,
    /// against the wall or as any turn end her dashing; the near-wall test; space jump's jump
    /// again; then, unless she kicked off a wall or is taking off, her vertical move. Returns
    /// whether a wall stopped her horizontal move.
    fn move_as_posed(&mut self, s: &Situation, made: &mut MovesMade, t: &mut Tick) -> bool {
        made.ran =
            matches!(s.moves, Moves::Build(gait, _) if running_with_run_held(gait, t.frame.pad));
        self.run.dash_for_moves(s.moves, t);
        let stride = self
            .run
            .stride(s.moves, self.pose.facing, self.vertical.has_motion(), t);
        let amount = self.run.speed + self.run.dash;
        let walled =
            stride != Stride::Nothing && self.walk(stride, s.doing, amount, &mut made.travelled, t);
        if matches!(
            s.doing,
            Doing::Standing | Doing::AgainstWall | Doing::Turning
        ) {
            self.run.dashing = Dashing::Off;
        }
        made.wall_test = self.near_wall(t);
        self.vertical.space_jump(s.doing, self.run.dash, t);
        if made.wall_test == WallTest::Kicked {
            made.met = Some(Met::WallKick);
        } else if s.doing != Doing::TakingOff {
            let (met, floor_stop) = self.move_vertically(s.footing, amount, &mut made.travelled, t);
            made.met = met;
            made.floor_stop = floor_stop;
        }
        walled
    }

    /// `SpeedBuildsAndBrakes`, `WallsAndCeilings`, `SlopesCarry`: her horizontal move, by her
    /// speed and dash together (`amount`) or her dash alone, the way she moves. A wall that stops
    /// it halts her and ends her dashing; a move by her dash that halts after it halts her, and
    /// as the ball ends her dashing too. Then the slope checks run. Returns whether a wall
    /// stopped it.
    fn walk(
        &mut self,
        stride: Stride,
        doing: Doing,
        amount: Fixed,
        travelled: &mut DirectionalTravels,
        t: &mut Tick,
    ) -> bool {
        let by = if stride == Stride::SpeedAndDash {
            amount
        } else {
            self.run.dash
        };
        let heading = self.run.moving(self.pose.facing).heading();
        let (from, probe) = (self.bounds(t.frame), self.horizontal_probe());
        let moved = t.sideways(from, heading, by, probe);
        self.position = moved.to;
        travelled[heading] = moved.accepted;
        let stopped = moved.stop.is_some();
        let halted = stopped || stride == Stride::DashThenHalt;
        if halted {
            self.run.halt();
        }
        if stopped || doing == Doing::Ball && halted {
            self.run.dashing = Dashing::Off;
        }
        self.slope_checks(t);
        stopped
    }

    /// `SlopesCarry`: after a horizontal move, the floor check, then the ceiling check from
    /// where the floor check left her; each that moves her seats her.
    fn slope_checks(&mut self, t: &Tick) {
        for edge in [Heading::Down, Heading::Up] {
            if let Some(by) = t.slope_check(self.bounds(t.frame), edge) {
                self.position.y += px(by);
                self.seat = Seat::Seated;
            }
        }
    }

    /// `WallJump`: the near-wall test, in a spin carried on holding a side: a test move of the
    /// wall reach away from the side held, which doesn't move her but writes as a stop would.
    /// A wall near her readies the jump until the spin has turned far, and then, with jump
    /// pressed, she kicks off.
    fn near_wall(&mut self, t: &mut Tick) -> WallTest {
        let (pad, frame) = (t.frame.pad, t.frame);
        if self.spin != Spin::CarriedOn || !pad.any(SIDES) {
            return WallTest::Neither;
        }
        let heading = if pad.any(LEFT) {
            Heading::Right
        } else {
            Heading::Left
        };
        let (from, reach) = (self.bounds(frame), t.rules.wall_reach);
        let tested = t.sideways(from, heading, reach, Probe::NearWall);
        self.take_result(&tested);
        if tested.stop.is_none() {
            WallTest::Neither
        } else if !frame.spin_turned_far {
            WallTest::Readied
        } else if frame.pressed.any(JUMP) {
            WallTest::Kicked
        } else {
            WallTest::Neither
        }
    }

    /// `FollowingTheGround`, `RisingAndFalling`, `SlopesCarry`: her vertical move, as its
    /// course says: on the ground the pull, her speed and dash before any wall stopped them
    /// (`amount`) plus 1 px, or 1 px seated on a slope; else by her vertical speed, which the
    /// pull then grows. While her body changes shape, the ground pull that finds no floor
    /// leaves her where she is. Returns what it met and the response a floor's stop made.
    fn move_vertically(
        &mut self,
        footing: Footing,
        amount: Fixed,
        travelled: &mut DirectionalTravels,
        t: &mut Tick,
    ) -> (Option<Met>, Option<FloorResponse>) {
        let course = self.vertical.course(footing);
        let by = match course {
            Course::Pulled if self.seat == Seat::Seated => t.rules.ground_pull,
            Course::Pulled => amount + t.rules.ground_pull,
            Course::BallMovesByIt | Course::InTheAir => self.vertical.speed,
        };
        let moved = self.move_up_or_down(by, t);
        let stays =
            course == Course::Pulled && footing == Footing::ChangingShape && moved.stop.is_none();
        if stays {
            travelled[vertical_heading(by)] = TravelDistance::ZERO;
            return (None, None);
        }
        travelled[vertical_heading(by)] = moved.accepted;
        self.take_result(&moved);
        let stopped = moved.stop.is_some();
        if course != Course::Pulled {
            self.vertical.grow(t.rules);
        }
        let floor_stop = self.vertical.floor_stop(course, by, stopped, footing);
        (
            self.vertical.end_move(course, by, stopped, footing),
            floor_stop,
        )
    }

    /// `RidingTheElevator`: facing the screen on a moving elevator, she moves down by its
    /// descent, which blocks stop as any downward move.
    fn ride_down(&mut self, made: &mut MovesMade, t: &mut Tick) {
        let descent = t.rules.elevator_descent;
        let moved = self.move_up_or_down(descent, t);
        made.travelled[vertical_heading(descent)] = moved.accepted;
        self.take_result(&moved);
    }

    /// `SlopesCarry`: a vertical move by the signed `by`. One of more than 0 that blocks
    /// decide, not a solid enemy, ends her seat on a slope.
    fn move_up_or_down(&mut self, by: Fixed, t: &mut Tick) -> Moved {
        let from = self.bounds(t.frame);
        let moved = t.up_or_down(from, by, Probe::Move);
        if by != ZERO && moved.stop != Some(Stop::Enemy) {
            self.seat = Seat::Unseated;
        }
        moved
    }
}
