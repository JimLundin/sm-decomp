//! Her moves in a frame, as the pose at its start says (`MovesAsShePosed`): horizontally,
//! the near-wall test, then the ground pull or her vertical move, and after the pose is chosen
//! the aim change's step; what they met (`WhatHerMovesMet`), and what they do to her dashing
//! (`Dashing`).

use crate::chart::Choice;
use crate::collide::{Probe, Stop};
use crate::rows::{Doing, Footing, Met, Moves, Situation};
use crate::run::{Dashing, Stride, running_with_run_held};
use crate::samus::{Samus, Seat, Spin, WallTest};
use crate::tick::Tick;
use crate::vertical::Course;
use types::button::{JUMP, LEFT, SIDES};
use types::{Facing, Fixed, Side, ZERO};

/// `WhatHerMovesMet`: what a frame's moves met, and what else the step reports of them.
#[derive(Default)]
pub struct Meeting {
    /// What her moves met.
    pub met: Option<Met>,
    /// `WallJump`: the near-wall test's outcome.
    pub wall_test: WallTest,
    /// `Dashing`: her moves were a run with run held, which dashes.
    pub ran_with_run_held: bool,
    /// §1.4, `RisingAndFalling`: she is heading down as her moves leave her, before what they
    /// met acts.
    pub heading_down: bool,
    /// §1.4, `RisingAndFalling`: her vertical speed where `heading_down` is read.
    pub vertical_speed: Fixed,
}

/// `WallsAndCeilings`, `SpeedBuildsAndBrakes`: why her walk halted her.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Halt {
    /// A wall stop: something stopped her horizontal move.
    WallStop,
    /// She moved by her dash alone, then halted.
    AfterDash,
}

/// `FollowingTheGround`, `WallsAndCeilings`, `Dashing`: what a walk did: the speed the ground
/// pull reads, from before any stop, the halt it ended in, and whether it was a run with run
/// held.
struct Walked {
    /// Her horizontal speed and dash.
    pull: Fixed,
    /// The halt the walk ended in, if any.
    halt: Option<Halt>,
    /// `Dashing`: a run with run held.
    ran_with_run_held: bool,
}

/// `Dashing`: whether her moves end her dashing: moves as standing still (a landing's too), as
/// against the wall or as any turn, or moves that halt her: a wall stop, or the ball halting
/// after its dash (at rest on the ground, or falling with neither side held). Crouched, the
/// halt after her dash leaves her dashing.
fn moves_end_dashing(doing: Doing, halt: Option<Halt>) -> bool {
    matches!(doing, Doing::Standing | Doing::AgainstWall | Doing::Turning)
        || halt == Some(Halt::WallStop)
        || doing == Doing::Ball && halt == Some(Halt::AfterDash)
}

impl Samus {
    /// `MovesAsShePosed`: her moves, as the pose at the frame's start says, and what they met.
    /// `SpeedBuildsAndBrakes`: facing the screen she doesn't move; `RidingTheElevator`: on a
    /// moving elevator she moves down 1 px, a move stopped as any downward move. `WallJump`:
    /// kicking off she doesn't move vertically that frame; `Jumping`: nor taking off.
    /// `WhatHerMovesMet`: a wall stop while running wins over walking off a ledge. `Dashing`:
    /// her moves may end her dashing. `RisingAndFalling`: whether she heads down, and her
    /// vertical speed, are read once her moves are made, before what they met acts.
    pub(crate) fn moves(&mut self, s: &Situation, t: &mut Tick) -> Meeting {
        if self.pose.facing == Facing::Screen {
            if self.rides_the_elevator(t.frame) {
                let ride = t.vmove(self.bounds(t), t.rules.elevator_descent, Probe::Move);
                self.position = ride.to;
            }
            return Meeting {
                heading_down: self.vertical.heading_down(),
                vertical_speed: self.vertical.speed,
                ..Meeting::default()
            };
        }
        let walk = self.walk(s, t);
        if moves_end_dashing(s.doing, walk.halt) {
            self.run.dashing = Dashing::Off;
        }
        let wall_test = self.wall_test(t);
        self.vertical.space_jump(s.doing, self.run.dash, t);
        let met = if wall_test == WallTest::Kicked {
            Some(Met::WallKick)
        } else if s.doing == Doing::TakingOff {
            None
        } else {
            self.move_vertically(s.footing, walk.pull, t)
        };
        let (heading_down, vertical_speed) = (self.vertical.heading_down(), self.vertical.speed);
        self.vertical.meet_in_moves(met);
        let stopped_running = walk.halt == Some(Halt::WallStop) && s.doing == Doing::Running;
        Meeting {
            met: stopped_running.then_some(Met::Wall).or(met),
            wall_test,
            ran_with_run_held: walk.ran_with_run_held,
            heading_down,
            vertical_speed,
        }
    }

    /// `SpeedBuildsAndBrakes`, `Dash`, `DriftInTheAir`, `WallsAndCeilings`: her horizontal move
    /// by her speed and dash, as her situation moves her, and the halt that follows a wall stop
    /// or a move by her dash alone. `SlopesCarry`: the checks after it may seat her.
    fn walk(&mut self, s: &Situation, t: &mut Tick) -> Walked {
        let facing = self.pose.facing;
        let ran_with_run_held =
            matches!(s.moves, Moves::Build(gait, _) if running_with_run_held(gait, t.frame.pad));
        let stride = self
            .run
            .stride(s.moves, facing, self.vertical.has_motion(), t);
        let pull = self.run.speed + self.run.dash;
        if stride == Stride::Nothing {
            return Walked {
                pull,
                halt: None,
                ran_with_run_held,
            };
        }
        let by = if stride == Stride::SpeedAndDash {
            pull
        } else {
            self.run.dash
        };
        let stop = self.hmove(self.run.moving(facing), by, t);
        if let Some(y) = t.slope_checks(self.bounds(t)) {
            self.position.y = y;
            self.seat = Seat::Seated;
        }
        let halt = if stop.is_some() {
            Some(Halt::WallStop)
        } else {
            (stride == Stride::DashThenHalt).then_some(Halt::AfterDash)
        };
        if halt.is_some() {
            self.run.halt();
        }
        Walked {
            pull,
            halt,
            ran_with_run_held,
        }
    }

    /// `BlocksStopFlush`, `SlopesSlowTheWalk`: a horizontal move, a walk when it is made with
    /// her vertical speed exactly 0. Returns what stopped it.
    fn hmove(&mut self, way: Side, d: Fixed, t: &mut Tick) -> Option<Stop> {
        let probe = if self.vertical.speed == ZERO {
            Probe::Walk
        } else {
            Probe::Move
        };
        let moved = t.go(self.bounds(t).axis(way.heading(), d, probe));
        self.position = moved.to;
        moved.stop
    }

    /// `AimingAndShooting`: a frame whose chosen pose brings the step moves her 1 px forward, by
    /// her facing at the frame's start, after her moves: a horizontal move with no `SlopesCarry`
    /// checks after it. A solid enemy within its reach stops it with no move at all, one she
    /// touches still clearing her y subpixel. `WhatHerMovesMet`: something in its way puts the
    /// running pose chosen against the wall (`Choice::blocked`).
    pub(crate) fn step_forward(&mut self, chosen: Choice, t: &mut Tick) -> Choice {
        if !chosen.aim_step {
            return chosen;
        }
        let x = self.position.x;
        let stop = self.hmove(Side::of(self.pose.facing), t.rules.aim_step, t);
        if stop == Some(Stop::Enemy) {
            self.position.x = x;
        }
        if stop.is_some() {
            chosen.blocked()
        } else {
            chosen
        }
    }

    /// `WallJump`: the near-wall test, in a spin carried on with a side held, looking 8 px away
    /// from it (left held: a wall on her right; else right held: on her left) from where her
    /// move left her. What stops it writes to her as a stop would, without moving her: a block
    /// her x subpixel, a solid enemy she touches her y subpixel. A wall near her readies the
    /// wall jump; once the spin has turned far, pressing jump kicks her off.
    fn wall_test(&mut self, t: &mut Tick) -> WallTest {
        let f = t.frame;
        if self.spin != Spin::CarriedOn || !f.pad.any(SIDES) {
            return WallTest::Neither;
        }
        let wall = if f.pad.any(LEFT) {
            Side::Right
        } else {
            Side::Left
        };
        let towards_wall = self
            .bounds(t)
            .axis(wall.heading(), t.rules.wall_reach, Probe::NearWall);
        let look = t.go(towards_wall);
        // What stopped the look writes her x subpixel, or her y subpixel against an enemy.
        self.take_result(&look);
        if look.stop.is_none() {
            WallTest::Neither
        } else if !f.spin_turned_far {
            WallTest::Readied
        } else if f.pressed.any(JUMP) {
            WallTest::Kicked
        } else {
            WallTest::Neither
        }
    }

    /// `FollowingTheGround`, `RisingAndFalling`: her vertical move, its course as her vertical
    /// motion and her situation's footing say, and what it met (`Vertical::end_move`). Pulled,
    /// she moves down by `pull` (the frame's horizontal speed and dash) plus 1 px
    /// (`SlopesCarry`: 1 px seated); else by her vertical speed. `SlopesCarry`: a move of more
    /// than 0 unseats her as it starts. While her body changes shape, a pull that finds no floor
    /// leaves her where she is, meeting nothing.
    fn move_vertically(&mut self, footing: Footing, pull: Fixed, t: &mut Tick) -> Option<Met> {
        let r = t.rules;
        let course = self.vertical.course(footing);
        let by = match course {
            Course::Pulled if self.seat == Seat::Seated => r.ground_pull,
            Course::Pulled => pull + r.ground_pull,
            Course::BallMovesByIt | Course::InTheAir => self.vertical.speed,
        };
        let moved = t.vmove(self.bounds(t), by, Probe::Move);
        if by != ZERO {
            self.seat = Seat::Unseated;
        }
        let stopped = moved.stop.is_some();
        if course == Course::Pulled && footing == Footing::ChangingShape && !stopped {
            return None;
        }
        self.take_result(&moved);
        self.vertical.end_move(course, by, stopped, footing, r)
    }
}
