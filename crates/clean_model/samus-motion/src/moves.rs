//! Her moves in a frame, as the pose at its start says (`MovesAsShePosed`): horizontally,
//! the near-wall test, then the ground pull or her vertical move, and after the pose is chosen
//! the aim change's step; and what they met (`WhatHerMovesMet`).

use crate::chart::{Choice, Moment};
use crate::collide::Probe::{Look, Move, Walk};
use crate::collide::Stop;
use crate::rows::Doing::{Running, TakingOff};
use crate::rows::Met::{self, Wall, WallKick};
use crate::rows::{Footing, Situation};
use crate::run::Stride::{Both, DashThenHalt, Nothing};
use crate::samus::Seat::{Seated, Unseated};
use crate::samus::WallTest::{Kicked, Neither, Readied};
use crate::samus::{Samus, Spin, WallTest};
use crate::tick::Tick;
use crate::vertical::{Course, MoveEnd};
use clean_types::button::{JUMP, LEFT, SIDES};
use clean_types::{Facing, Fixed, Side, ZERO};

/// `WhatHerMovesMet`: what a frame's moves met, and the near-wall test's outcome (`WallJump`).
#[derive(Default)]
pub struct Meeting {
    /// What her moves met.
    pub met: Option<Met>,
    /// The near-wall test's outcome.
    pub wall_test: WallTest,
}

/// `FollowingTheGround`, `WallsAndCeilings`: what a walk did: the speed the ground pull reads,
/// from before any stop, and a wall stop while running.
struct Walked {
    /// Her horizontal speed and dash.
    pull: Fixed,
    /// A wall stop while running (`WhatHerMovesMet`).
    wall: Option<Met>,
}

impl Samus {
    /// `MovesAsShePosed`: her moves, as the pose at the frame's start says, and what they met.
    /// `SpeedBuildsAndBrakes`: facing the screen she doesn't move; `RidingTheElevator`: on a
    /// moving elevator she moves down 1 px, a move stopped as any downward move. `WallJump`:
    /// kicking off she doesn't move vertically that frame; `Jumping`: nor taking off.
    /// `WhatHerMovesMet`: a wall stop while running wins over walking off a ledge.
    pub(crate) fn moves(&mut self, s: &Situation, t: &mut Tick) -> Meeting {
        if self.pose.facing == Facing::Screen {
            if t.frame.elevator_moving {
                self.position = t.vmove(self.bounds(), t.rules.elevator, Move).to;
            }
            return Meeting::default();
        }
        let walk = self.walk(s, t);
        let wall_test = self.wall_test(t);
        self.vertical.space_jump(s.doing, t);
        let met = if wall_test == Kicked {
            Some(WallKick)
        } else if s.doing == TakingOff {
            None
        } else {
            self.move_vertically(s.footing, walk.pull, t)
        };
        Meeting {
            met: walk.wall.or(met),
            wall_test,
        }
    }

    /// `SpeedBuildsAndBrakes`, `Dash`, `DriftInTheAir`, `WallsAndCeilings`: her horizontal move
    /// by her speed and dash, as her situation moves her, and the halt that follows a wall stop
    /// or a move by her dash alone. `SlopesCarry`: the checks after it may seat her.
    fn walk(&mut self, s: &Situation, t: &mut Tick) -> Walked {
        let facing = self.pose.facing;
        let stride = self
            .run
            .stride(s.moves, facing, self.vertical.has_motion(), t);
        let pull = self.run.speed + self.run.dash;
        if stride == Nothing {
            return Walked { pull, wall: None };
        }
        let by = if stride == Both { pull } else { self.run.dash };
        let stop = self.hmove(self.run.moving(facing), by, t);
        if let Some(y) = t.seat(self.bounds()) {
            self.position.y = y;
            self.seat = Seated;
        }
        if stop.is_some() || stride == DashThenHalt {
            self.run.halt();
        }
        Walked {
            pull,
            wall: (stop.is_some() && s.doing == Running).then_some(Wall),
        }
    }

    /// `BlocksStopFlush`, `SlopesSlowTheWalk`: a horizontal move, a walk when it is made with
    /// her vertical speed exactly 0. Returns what stopped it.
    fn hmove(&mut self, way: Side, d: Fixed, t: &mut Tick) -> Option<Stop> {
        let probe = if self.vertical.speed == ZERO {
            Walk
        } else {
            Move
        };
        let moved = t.go(self.bounds().axis(way.heading(), d, probe));
        self.position = moved.to;
        moved.stop
    }

    /// `AimingAndShooting`: a frame whose chosen pose brings the step moves her 1 px forward, by
    /// her facing at the frame's start, after her moves: a horizontal move with no `SlopesCarry`
    /// checks after it. A solid enemy within its reach stops it with no move at all, one she
    /// touches still clearing her y subpixel. `WhatHerMovesMet`: something in its way puts the
    /// running pose chosen against the wall (`Moment::step_blocked`).
    pub(crate) fn step_forward(
        &mut self,
        chosen: Choice,
        moment: &Moment,
        s: &'static Situation,
        t: &mut Tick,
    ) -> Choice {
        if !chosen.steps {
            return chosen;
        }
        let x = self.position.x;
        let stop = self.hmove(Side::of(self.pose.facing), t.rules.aim_step, t);
        if stop == Some(Stop::Enemy) {
            self.position.x = x;
        }
        if stop.is_some() {
            moment.step_blocked(chosen, s)
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
            return Neither;
        }
        let wall = if f.pad.any(LEFT) {
            Side::Right
        } else {
            Side::Left
        };
        let seen = t.go(self.bounds().axis(wall.heading(), t.rules.wall_reach, Look));
        self.position = seen.to;
        if seen.stop.is_none() {
            Neither
        } else if !f.spin_turned_far {
            Readied
        } else if f.pressed.any(JUMP) {
            Kicked
        } else {
            Neither
        }
    }

    /// `FollowingTheGround`, `RisingAndFalling`: her vertical move, its course as her vertical
    /// motion and her situation's footing say, and what it met (`Vertical::end_move`). Pulled,
    /// she moves down by `pull` (the frame's horizontal speed and dash) plus 1 px
    /// (`SlopesCarry`: 1 px seated); else by her vertical speed. `SlopesCarry`: a move of more
    /// than 0 unseats her as it starts.
    fn move_vertically(&mut self, footing: Footing, pull: Fixed, t: &mut Tick) -> Option<Met> {
        let r = t.rules;
        let course = self.vertical.course(footing);
        let by = match course {
            Course::Pulled if self.seat == Seated => r.ground_pull,
            Course::Pulled => pull + r.ground_pull,
            Course::OnTheGround | Course::InTheAir => self.vertical.speed,
        };
        let moved = t.vmove(self.bounds(), by, Move);
        if by != ZERO {
            self.seat = Unseated;
        }
        match self
            .vertical
            .end_move(course, by, moved.stop.is_some(), footing, r)
        {
            MoveEnd::Stays => None,
            MoveEnd::Moved(met) => {
                self.end_at(&moved);
                met
            }
        }
    }
}
