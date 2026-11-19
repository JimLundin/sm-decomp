//! Spec: `PerformedMotionDecisions`, `SuspendedMovement`.
//!
//! An owned ordinary-update continuation. Offers retain real answers; completion never
//! consults a rule or world again to reconstruct an answer. External inputs are borrowed
//! only while making the next decision, so unanswered requests see the current provider.

use crate::chart::{Choice, Moment};
use crate::collide::{Axis, Bounds, Moved, Probe, Stop};
use crate::fit::{Fitted, crouched};
use crate::jump_held::FrameBefore;
use crate::rows::{Doing, Effect, Footing, Met, Moves};
use crate::rules::{Lowers, Rules};
use crate::run::{Dashing, Stride, running_with_run_held};
use crate::samus::{Outcome, Samus, Seat, Spin, StandingAnimation, WallTest, sprite};
use crate::situation::{ENTERING, UnknownPose, classify};
use crate::tick::{Frame, Tick};
use crate::vertical::{Contact, Course};
use types::button::{JUMP, LEFT, SIDES};
use types::{
    Body, Cell, CollisionPurpose, Direction, Facing, Fixed, Parts, Position, Px, Radius, Side,
    TravelDistance, World, ZERO, px,
};

/// Spec: `PoseFollows`. Owned choice, including the row effects still to be consumed.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SelectedChoice {
    /// Selected or accepted pose.
    pub pose: types::Pose,
    /// Ordered remaining row effects.
    pub effects: Vec<Effect>,
    /// Whether this selection admits aim travel.
    pub aim_step: bool,
    /// Lowering admitted by this selection.
    pub lower: Option<Lowers>,
    /// Animation handover disposition.
    pub handed_over: bool,
}
impl From<Choice> for SelectedChoice {
    fn from(c: Choice) -> Self {
        Self {
            pose: c.pose,
            effects: c.effects.to_vec(),
            aim_step: c.aim_step,
            lower: c.lower,
            handed_over: c.handed_over,
        }
    }
}
impl SelectedChoice {
    fn blocked(&self) -> Self {
        if self.pose.action == types::Action::Running {
            Choice::instead(crate::chart::table::STEP_BLOCKED.pose_from(self.pose, None)).into()
        } else {
            self.clone()
        }
    }
}

/// Spec: `PerformedMotionDecisions`. Semantic operands of a local rule. Complete before
/// and answered State accompany these operands, rather than an inferred final-frame value.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LocalRule {
    /// `WallJump`: advance spin history and decide ordinary/elevator admission.
    FrameStart,
    /// `RisingAndFalling`: current held-jump premise decides whether the rise ends.
    RiseEnds,
    /// `RisingAndFalling`: fresh pull after completed travel, before its effect.
    VerticalSpeedGrowth,
    /// `WallJump`: current animation and press decide the returned reach's outcome.
    WallOutcome {
        /// Actual chosen readiness or kick.
        result: WallTest,
    },
    /// `Dash`: gain or cap from the actual dash before speed building.
    DashBuilding {
        /// Dash before this decision.
        before: Fixed,
        /// Configured dash increment.
        gain: Fixed,
        /// Cap selected by current items.
        cap: Fixed,
    },
    /// `SpeedBuildsAndBrakes`: build or brake according to actual carrying and pad.
    SpeedBuilding {
        /// Speed before this decision.
        before: Fixed,
        /// Gait increment.
        step: Fixed,
        /// Wrapping sum, before cap.
        sum: Fixed,
        /// Gait cap.
        top: Fixed,
    },
    /// `SpeedBuildsAndBrakes`: resting, dash loss or dash-only motion.
    Stride,
    /// `FollowingTheGround`: retain the total before a stop independently of travel.
    HorizontalDisplacement {
        /// Signed speed-derived intent.
        intended: TravelDistance,
        /// Speed plus dash before collision.
        retained_total: Fixed,
    },
    /// `WallJump`: decide whether reach work is admitted.
    NearWall,
    /// `Jumping`: current space-jump admission.
    SpaceJump,
    /// `FollowingTheGround`: current seating and retained total decide the pull.
    GroundPull {
        /// Total before any horizontal stop.
        retained_total: Fixed,
        /// Configured extra pull.
        extra: Fixed,
        /// Signed vertical intent.
        intended: TravelDistance,
        /// Actual seating after support.
        seated: bool,
    },
    /// `RisingAndFalling`: travel by current vertical speed.
    VerticalDisplacement {
        /// Signed speed-derived intent.
        intended: TravelDistance,
    },
    /// `PoseFollows`: actual selection before aim travel.
    PoseChoice {
        /// Current chart inputs and earlier movement result.
        moment: Moment,
        /// Actual answer, before aim or fit can change it.
        choice: SelectedChoice,
    },
    /// `AimingAndShooting`: the selected choice's forward adjustment.
    AimTravel {
        /// Signed configured adjustment.
        intended: TravelDistance,
    },
    /// `PoseFitAfterMoves`: actual growth operands, before requesting clearance.
    BodyGrowth {
        /// Body at consideration; its height decides the retained growth distance.
        old_radius: Radius,
        /// Selected body's size at this consideration.
        selected_radius: Radius,
    },
    /// `PoseFitAfterMoves`: actual fit decision, after any nested clearance work.
    BodyFit {
        /// Body before selection.
        old_radius: Radius,
        /// Body being considered.
        selected_radius: Radius,
        /// Target after aim, before fit acceptance.
        selected_pose: types::Pose,
        /// Actual fit answer.
        result: types::BodyFitResult,
    },
    /// `PoseFollows`: initialization uses the accepted answer, not the desired pose.
    PoseInitialization {
        /// Accepted choice and remaining effects.
        accepted: SelectedChoice,
    },
    /// `AcceptedFloorResponse`: accepted response before it changes motion.
    FloorResponse {
        /// Actual accepted contact, if any.
        contact: Option<Contact>,
    },
    /// `PoseFollows`: one accepted row or entry effect.
    PoseEffect {
        /// Effect at the owned consumed-input cursor.
        effect: Effect,
    },
    /// `Jumping`, `TouchdownRequests`: finish history and output publication.
    Finish,
}

/// Spec: `MotionAttempts`. An unanswered actual request, with its submitted geometry.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MotionRequest {
    /// Actual starting centre and body.
    pub bounds: Bounds,
    /// Direction of the requested travel or look.
    pub direction: Direction,
    /// Requested magnitude, before terrain shortening.
    pub distance: Fixed,
    /// Requested world and walk/look semantics.
    pub probe: Probe,
    /// Semantic owner of the request.
    pub purpose: CollisionPurpose,
    axis: Axis,
}
impl MotionRequest {
    const fn new(
        bounds: Bounds,
        direction: Direction,
        distance: Fixed,
        probe: Probe,
        purpose: CollisionPurpose,
    ) -> Self {
        Self {
            bounds,
            direction,
            distance,
            probe,
            purpose,
            axis: bounds.axis(direction, distance, probe, purpose),
        }
    }
}

/// Spec: `PerformedMotionDecisions`. Current external semantic premises of a local answer.
/// Geometry and selected pose content are retained in the rule and its pending outputs.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DecisionInputs {
    /// Held actions at this decision.
    pub pad: types::Buttons,
    /// Just-pressed actions at this decision.
    pub pressed: types::Buttons,
    /// Currently switched-on equipment.
    pub items: types::Set<types::Item>,
    /// Actual imported animation end.
    pub animation_ends: Option<types::ClipEnd>,
    /// Actual spin animation admission.
    pub spin_turned_far: bool,
    /// Current elevator admission.
    pub elevator_moving: bool,
    /// Current cinematic admission for output cues.
    pub cinematic_active: bool,
    /// Actual preceding appearance figure.
    pub sprite_before_movement: types::Sprite,
    /// Frame's collision ordering.
    pub parity: types::Parity,
    /// Actual profile used to choose this answer.
    pub rules: Rules,
}
impl DecisionInputs {
    fn current(frame: &Frame, rules: &Rules) -> Self {
        Self {
            pad: frame.pad,
            pressed: frame.pressed,
            items: frame.items,
            animation_ends: frame.animation_ends,
            spin_turned_far: frame.spin_turned_far,
            elevator_moving: frame.elevator_moving,
            cinematic_active: frame.cinematic_active,
            sprite_before_movement: frame.sprite_before_movement,
            parity: frame.parity,
            rules: rules.clone(),
        }
    }
}

/// Spec: `PerformedMotionDecisions`. The actual offered answer or unanswered world request.
/// World answers own all performed candidates/cells and outputs, before dependent State effects.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MovementDecision {
    /// Local answer computed by the actual rule, not yet applied.
    Local {
        /// Named rule and its semantic operands.
        rule: LocalRule,
        /// Actual external premises when the answer was chosen.
        inputs: Box<DecisionInputs>,
        /// Actual complete State before its effect.
        before: Samus,
        /// Retained complete answered State.
        answer: Samus,
    },
    /// Intended geometry, before asking the current world.
    MotionRequest(MotionRequest),
    /// Actual world answer before dependent movement effects.
    MotionAnswer {
        /// Request actually queried.
        request: MotionRequest,
        /// Returned travel, room and stopping answer.
        answer: Moved,
        /// Actual considered candidates and cells, in performed order.
        outputs: DecisionOutputs,
    },
    /// Independent support query, before asking the current world.
    SupportRequest {
        /// Current centre and ordinary body.
        bounds: Bounds,
        /// Floor or ceiling being asked.
        surface: types::SupportSurface,
    },
    /// Retained actual support answer before placement.
    SupportAnswer {
        /// Complete returned check and placement.
        check: types::SupportCheck,
    },
    /// `TouchedCells`: before resolving the frame's inside cells.
    InsideRequest(Bounds),
    /// `TouchedCells`: resolved inside identities before publication.
    InsideAnswer(Vec<Cell>),
}

/// Spec: `SuspendedMovement`. Owned outputs performed by a returned world answer.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DecisionOutputs {
    /// Ordered reaction cells.
    pub reached: Vec<types::Touch>,
    /// Ordered leading-edge tests.
    pub collision_tests: types::CollisionTests,
    /// Actual request receipts.
    pub movement_work: Vec<types::MovementWork>,
    /// Accepted move-local response, if any.
    pub floor_response: Option<types::FloorResponse>,
    /// Accepted transition's firing publication, if any.
    pub pending_aim: Option<types::FiringDirection>,
}
impl DecisionOutputs {
    fn from_tick(t: Tick) -> Self {
        Self {
            reached: t.reached,
            collision_tests: t.collision_tests,
            movement_work: t.movement_work,
            floor_response: t.floor_response,
            pending_aim: t.pending_aim,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum Stage {
    Inside,
    Rise,
    Start,
    Growth,
    WallChoice,
    Dash,
    Speed,
    Horizontal,
    Walk,
    Floor,
    Ceiling,
    NearWall,
    WallLook,
    SpaceJump,
    Vertical,
    VerticalLook,
    Choice,
    Aim,
    AimLook,
    BodyFit,
    FitLook,
    FitDone,
    Initialize,
    Lower,
    LowerLook,
    Response,
    Effects,
    Finish,
    Done,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum FitStep {
    EnemyAbove,
    EnemyBelow,
    EnemyFar,
    BlockAbove,
    BlockBelow,
    BlockFar,
    Done,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct FitWork {
    old: Radius,
    new: Radius,
    centre: Position,
    checks: Vec<types::MotionAttempt>,
    step: FitStep,
    here: Rooms,
    found: Rooms,
    fitted: Fitted,
}

/// Spec: `SuspendedMovement`. Complete interrupted ordinary work.
///
/// Serialize this value with
/// serde, discard it, and deserialize a fresh value to continue. The caller saves its provider
/// and input cursor alongside it. `state` is the actual current complete movement State.
/// `offered` is stable until completion; making or inspecting it never applies its answer.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SuspendedMovement {
    /// Complete current movement State, including completed effects.
    pub state: Samus,
    /// Prior quantities read later in this interval.
    pub start: Samus,
    /// Uncompleted actual request or retained answer.
    pub offered: Option<MovementDecision>,
    /// Actual chart choice before aim work.
    pub choice_before_aim: Option<SelectedChoice>,
    /// Actual choice after aim and fit acceptance.
    pub accepted_choice: Option<SelectedChoice>,
    /// Speed plus dash before any horizontal stop.
    pub horizontal_amount: Fixed,
    /// Completed ordered outputs; unfinished work has not been appended.
    pub outputs: Outcome,
    stage: Stage,
    stride: Stride,
    walk_stopped: bool,
    ran: bool,
    course: Course,
    vertical_by: Fixed,
    met: Option<Met>,
    contact: Option<Contact>,
    body_fit: Option<FitWork>,
    request: Option<MotionRequest>,
    longer: Option<MotionRequest>,
    effect_cursor: usize,
    local_outputs: Option<DecisionOutputs>,
    premises: Option<DecisionInputs>,
    local_next: Option<Stage>,
    finish_answer: Option<Outcome>,
    vertical_stop: bool,
}

fn tick<'a>(frame: &'a Frame<'a>, world: &'a dyn World, rules: &'a Rules) -> Tick<'a> {
    Tick {
        frame,
        world,
        rules,
        reached: Vec::new(),
        collision_tests: Vec::new(),
        movement_work: Vec::new(),
        floor_response: None,
        pending_aim: None,
    }
}
fn signed(h: Direction, by: Fixed) -> TravelDistance {
    TravelDistance(if matches!(h, Direction::Up | Direction::Left) {
        -by
    } else {
        by
    })
}
fn direction(by: Fixed) -> Direction {
    if by.negative() {
        Direction::Up
    } else {
        Direction::Down
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct Rooms {
    above: Option<Px>,
    below: Option<Px>,
}
impl Rooms {
    const fn empty() -> Self {
        Self {
            above: None,
            below: None,
        }
    }
    fn merge(self, earlier: Self) -> Self {
        Self {
            above: self.above.or(earlier.above),
            below: self.below.or(earlier.below),
        }
    }
}
fn alone(rooms: Rooms, g: Px) -> Option<Px> {
    match (rooms.above, rooms.below) {
        (Some(room), None) => Some(g - room),
        (None, Some(room)) => Some(room - g),
        (Some(_), Some(_)) | (None, None) => None,
    }
}

impl SuspendedMovement {
    /// Spec: `SuspendedMovement`. Start one new interval; unknown poses leave State untouched.
    /// # Errors
    /// Returns `UnknownPose` when no ordinary situation names the supplied pose.
    pub fn new(state: Samus) -> Result<Self, UnknownPose> {
        classify(state.pose)?;
        let outputs = Outcome {
            movement_work: Vec::new(),
            movement_skip: None,
            floor_response: None,
            touchdown_requests: Vec::new(),
            touchdown_graphics: false,
            collision_tests: Vec::new(),
            pending_aim: None,
            travelled: crate::travel::TravelledDistances::fresh().travels,
            wall_test: WallTest::Neither,
            standing_animation: StandingAnimation::PlaysOn,
            sprite: types::Sprite::Plain,
            skidding: false,
            reached: Vec::new(),
            inside: Vec::new(),
            placed: None,
            rising: false,
            dashing_after_moves: state.run.dashing,
            dash_began: false,
            dash_lost: false,
            handed_over: false,
            heading_down: false,
            vertical_speed_after_moves: ZERO,
        };
        Ok(Self {
            state,
            start: state,
            offered: None,
            choice_before_aim: None,
            accepted_choice: None,
            horizontal_amount: ZERO,
            outputs,
            stage: Stage::Inside,
            stride: Stride::Nothing,
            walk_stopped: false,
            ran: false,
            course: Course::Pulled,
            vertical_by: ZERO,
            met: None,
            contact: None,
            body_fit: None,
            request: None,
            longer: None,
            effect_cursor: 0,
            local_outputs: None,
            premises: None,
            local_next: None,
            finish_answer: None,
            vertical_stop: false,
        })
    }
    /// Spec: `SuspendedMovement`. Finished work has no query, effect or publication left.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.stage == Stage::Done
    }

    fn bounds(&self, frame: &Frame) -> Bounds {
        Bounds {
            at: self.state.position,
            r: (frame.body)(self.state.pose),
        }
    }
    fn local(&mut self, rule: LocalRule, answer: Samus) {
        if let Some(inputs) = self.premises.take() {
            self.offered = Some(MovementDecision::Local {
                rule,
                inputs: Box::new(inputs),
                before: self.state,
                answer,
            });
        }
    }
    fn collect(&mut self, o: DecisionOutputs) {
        self.outputs.reached.extend(o.reached);
        self.outputs.collision_tests.extend(o.collision_tests);
        if self.stage == Stage::FitLook {
            if let Some(fit) = &mut self.body_fit {
                fit.checks
                    .extend(o.movement_work.into_iter().filter_map(|work| match work {
                        types::MovementWork::Motion(motion) => Some(motion),
                        types::MovementWork::Support(_) | types::MovementWork::BodyFit(_) => None,
                    }));
            }
        } else {
            self.outputs.movement_work.extend(o.movement_work);
        }
        if o.floor_response.is_some() {
            self.outputs.floor_response = o.floor_response;
        }
        if o.pending_aim.is_some() {
            self.outputs.pending_aim = o.pending_aim;
        }
    }
    fn vertical_request(
        &mut self,
        b: Bounds,
        by: Fixed,
        probe: Probe,
        purpose: CollisionPurpose,
        rules: &Rules,
    ) {
        let h = direction(by);
        let request = MotionRequest::new(b, h, by.size(), probe, purpose);
        if probe == Probe::Blocks && by.size() >= rules.long_look {
            self.longer = Some(request);
            self.request = Some(MotionRequest::new(b, h, rules.long_look, probe, purpose));
        } else {
            self.request = Some(request);
        }
    }
    fn request_offer(&mut self, frame: &Frame, rules: &Rules) {
        if let Some(request) = &self.request {
            let mut submitted = request.clone();
            if self.stage == Stage::FitLook {
                let pose = if self
                    .body_fit
                    .as_ref()
                    .is_some_and(|fit| fit.step == FitStep::EnemyFar)
                {
                    self.accepted_choice
                        .as_ref()
                        .map_or(self.state.pose, |c| c.pose)
                } else {
                    self.state.pose
                };
                submitted.bounds = Bounds {
                    at: self.state.position,
                    r: (frame.body)(pose),
                };
                submitted.axis = submitted.bounds.axis(
                    submitted.direction,
                    submitted.distance,
                    submitted.probe,
                    submitted.purpose,
                );
            }
            if !submitted.direction.vertical() {
                submitted.distance = submitted
                    .distance
                    .min(types::fixed(rules.move_limit, submitted.distance.sub()));
                submitted.axis = submitted.bounds.axis(
                    submitted.direction,
                    submitted.distance,
                    submitted.probe,
                    submitted.purpose,
                );
            }
            self.offered = Some(MovementDecision::MotionRequest(submitted));
        }
    }

    /// Spec: `PerformedMotionDecisions`. Offer the next real decision without applying it.
    /// Already offered answers are returned unchanged, even with different external inputs.
    /// # Errors
    /// Returns `UnknownPose` if the current start pose has no ordinary situation.
    pub fn offer(
        &mut self,
        frame: &Frame,
        world: &dyn World,
        rules: &Rules,
    ) -> Result<Option<&MovementDecision>, UnknownPose> {
        while self.offered.is_none() && !self.is_complete() {
            self.make_offer(frame, world, rules)?;
        }
        Ok(self.offered.as_ref())
    }

    fn make_offer(&mut self, f: &Frame, w: &dyn World, r: &Rules) -> Result<(), UnknownPose> {
        classify(self.start.pose)?;
        self.premises = Some(DecisionInputs::current(f, r));
        match self.stage {
            Stage::Inside => self.offered = Some(MovementDecision::InsideRequest(self.bounds(f))),
            Stage::Rise
            | Stage::Growth
            | Stage::WallChoice
            | Stage::Start
            | Stage::Dash
            | Stage::Speed
            | Stage::Horizontal
            | Stage::NearWall
            | Stage::SpaceJump
            | Stage::Vertical => self.motion_offer(f, w, r)?,
            Stage::Choice
            | Stage::Aim
            | Stage::BodyFit
            | Stage::FitDone
            | Stage::Initialize
            | Stage::Lower
            | Stage::Response
            | Stage::Effects
            | Stage::Finish => self.pose_offer(f, w, r)?,
            Stage::Walk
            | Stage::WallLook
            | Stage::VerticalLook
            | Stage::AimLook
            | Stage::FitLook
            | Stage::LowerLook => self.request_offer(f, r),
            Stage::Floor | Stage::Ceiling => {
                self.offered = Some(MovementDecision::SupportRequest {
                    bounds: self.bounds(f),
                    surface: if self.stage == Stage::Floor {
                        types::SupportSurface::Floor
                    } else {
                        types::SupportSurface::Ceiling
                    },
                });
            }
            Stage::Done => {}
        }
        Ok(())
    }

    fn motion_offer(&mut self, f: &Frame, w: &dyn World, r: &Rules) -> Result<(), UnknownPose> {
        let s = classify(self.start.pose)?;
        let mut answer = self.state;
        let context = tick(f, w, r);
        match self.stage {
            Stage::Rise => {
                answer.vertical.end_rise(s.doing, f.pad);
                self.local(LocalRule::RiseEnds, answer);
            }
            Stage::Growth => self.growth_offer(s.footing, &context),
            Stage::WallChoice => self.wall_offer(f),
            Stage::Start => {
                answer.spin = answer.spin.then(s.doing == Doing::Spinning);
                self.start_next(f, r);
                self.local(LocalRule::FrameStart, answer);
            }
            Stage::Dash => self.dash_offer(s.moves, &context),
            Stage::Speed => {
                self.ran =
                    matches!(s.moves, Moves::Build(gait, _) if running_with_run_held(gait, f.pad));
                self.stride = answer.run.stride(
                    s.moves,
                    answer.pose.facing,
                    answer.vertical.has_motion(),
                    &context,
                );
                let rule = if let Moves::Build(gait, _) = s.moves {
                    let limits = r.limits(gait);
                    LocalRule::SpeedBuilding {
                        before: self.state.run.speed,
                        step: limits.step,
                        sum: self.state.run.speed + limits.step,
                        top: limits.top,
                    }
                } else {
                    LocalRule::Stride
                };
                self.local(rule, answer);
            }
            Stage::Horizontal => {
                let total = self.state.run.speed + self.state.run.dash;
                let by = if self.stride == Stride::SpeedAndDash {
                    total
                } else {
                    self.state.run.dash
                };
                let heading = self.state.run.moving(self.state.pose.facing).heading();
                self.request = Some(MotionRequest::new(
                    self.bounds(f),
                    heading,
                    by,
                    self.travel_probe(),
                    CollisionPurpose::OrdinaryMove,
                ));
                self.local(
                    LocalRule::HorizontalDisplacement {
                        intended: signed(heading, by),
                        retained_total: total,
                    },
                    answer,
                );
            }
            Stage::NearWall => {
                if matches!(
                    s.doing,
                    Doing::Standing | Doing::AgainstWall | Doing::Turning
                ) {
                    answer.run.dashing = Dashing::Off;
                }
                self.near_wall_next(f, r);
                self.local(LocalRule::NearWall, answer);
            }
            Stage::SpaceJump => {
                answer
                    .vertical
                    .space_jump(s.doing, answer.run.dash, &context);
                self.local(LocalRule::SpaceJump, answer);
            }
            Stage::Vertical => self.vertical_offer(s.footing, f, r),
            Stage::Inside
            | Stage::Walk
            | Stage::Floor
            | Stage::Ceiling
            | Stage::WallLook
            | Stage::VerticalLook
            | Stage::Choice
            | Stage::Aim
            | Stage::AimLook
            | Stage::BodyFit
            | Stage::FitLook
            | Stage::FitDone
            | Stage::Initialize
            | Stage::Lower
            | Stage::LowerLook
            | Stage::Response
            | Stage::Effects
            | Stage::Finish
            | Stage::Done => {}
        }
        Ok(())
    }

    fn growth_offer(&mut self, footing: Footing, context: &Tick) {
        let mut answer = self.state;
        let mut t = tick(context.frame, context.world, context.rules);
        self.met = answer.vertical.end_move(
            self.course,
            self.vertical_by,
            self.vertical_stop,
            footing,
            &mut t,
        );
        self.local_outputs = Some(DecisionOutputs::from_tick(t));
        self.local(LocalRule::VerticalSpeedGrowth, answer);
    }
    fn wall_offer(&mut self, f: &Frame) {
        let result = if !self.vertical_stop {
            WallTest::Neither
        } else if !f.spin_turned_far {
            WallTest::Readied
        } else if f.pressed.any(JUMP) {
            WallTest::Kicked
        } else {
            WallTest::Neither
        };
        self.local(LocalRule::WallOutcome { result }, self.state);
    }

    fn dash_offer(&mut self, moves: Moves, context: &Tick) {
        let mut answer = self.state;
        answer.run.dash_for_moves(moves, context);
        let rule = if matches!(moves, Moves::Build(gait, _) if running_with_run_held(gait, context.frame.pad))
        {
            LocalRule::DashBuilding {
                before: self.state.run.dash,
                gain: context.rules.dash_gain,
                cap: context.rules.dash_cap.of(&context.frame.items),
            }
        } else {
            LocalRule::Stride
        };
        self.local(rule, answer);
    }

    fn vertical_offer(&mut self, footing: Footing, frame: &Frame, rules: &Rules) {
        let course = self.state.vertical.course(footing);
        let by = match course {
            Course::Pulled if self.state.seat == Seat::Seated => rules.ground_pull,
            Course::Pulled => self.horizontal_amount + rules.ground_pull,
            Course::BallMovesByIt | Course::InTheAir => self.state.vertical.speed,
        };
        self.course = course;
        self.vertical_by = by;
        let purpose = if course == Course::Pulled {
            CollisionPurpose::GroundFollowing
        } else {
            CollisionPurpose::OrdinaryMove
        };
        self.vertical_request(self.bounds(frame), by, Probe::Move, purpose, rules);
        let rule = if course == Course::Pulled {
            LocalRule::GroundPull {
                retained_total: self.horizontal_amount,
                extra: rules.ground_pull,
                intended: TravelDistance(by),
                seated: self.state.seat == Seat::Seated,
            }
        } else {
            LocalRule::VerticalDisplacement {
                intended: TravelDistance(by),
            }
        };
        self.local(rule, self.state);
    }

    fn pose_offer(&mut self, f: &Frame, w: &dyn World, r: &Rules) -> Result<(), UnknownPose> {
        let mut answer = self.state;
        match self.stage {
            Stage::Choice => self.choice_offer(f, r)?,
            Stage::Aim => {
                let heading = Side::of(self.start.pose.facing).heading();
                self.request = Some(MotionRequest::new(
                    self.bounds(f),
                    heading,
                    r.aim_step,
                    self.travel_probe(),
                    CollisionPurpose::AimChange,
                ));
                self.local(
                    LocalRule::AimTravel {
                        intended: signed(heading, r.aim_step),
                    },
                    answer,
                );
            }
            Stage::BodyFit => self.begin_fit(f, r),
            Stage::FitDone => self.fit_decision(f, r),
            Stage::Initialize => {
                if let Some(c) = &self.accepted_choice {
                    answer.pose = c.pose;
                    self.local_outputs = Some(DecisionOutputs {
                        reached: Vec::new(),
                        collision_tests: Vec::new(),
                        movement_work: Vec::new(),
                        floor_response: None,
                        pending_aim: crate::change::transition_firing_aim(
                            self.start.pose,
                            c.pose,
                            f,
                        ),
                    });
                    self.local(
                        LocalRule::PoseInitialization {
                            accepted: c.clone(),
                        },
                        answer,
                    );
                }
            }
            Stage::Lower => {
                if let Some(lower) = self.accepted_choice.as_ref().and_then(|c| c.lower) {
                    self.vertical_request(
                        self.bounds(f),
                        r.lowers(lower),
                        Probe::Blocks,
                        CollisionPurpose::BodyLowering,
                        r,
                    );
                }
                self.local(LocalRule::Stride, answer);
            }
            Stage::Response => self.response_offer(f, w, r),
            Stage::Effects => self.effect_offer(f, w, r),
            Stage::Finish => {
                let handed = self
                    .choice_before_aim
                    .as_ref()
                    .is_some_and(|c| c.handed_over);
                let left =
                    FrameBefore::left_behind(self.met, handed, self.start.pose, self.state.pose);
                answer.jump_held = self.start.jump_held.then(r, f, left);
                let mut finished = self.clone();
                finished.state = answer;
                finished.finish_outputs(f);
                self.finish_answer = Some(finished.outputs);
                self.local(LocalRule::Finish, answer);
            }
            Stage::Inside
            | Stage::Start
            | Stage::Dash
            | Stage::Speed
            | Stage::Horizontal
            | Stage::Walk
            | Stage::Floor
            | Stage::Ceiling
            | Stage::NearWall
            | Stage::WallLook
            | Stage::SpaceJump
            | Stage::Vertical
            | Stage::VerticalLook
            | Stage::AimLook
            | Stage::FitLook
            | Stage::LowerLook
            | Stage::Rise
            | Stage::Growth
            | Stage::WallChoice
            | Stage::Done => {}
        }
        Ok(())
    }

    fn choice_offer(&mut self, frame: &Frame, rules: &Rules) -> Result<(), UnknownPose> {
        let situation = classify(self.start.pose)?;
        self.contact = self.state.vertical.contact(
            self.met,
            situation.footing,
            self.start.pose.body == Body::SpringBall && frame.pad.any(JUMP),
            rules,
        );
        let moment = Moment {
            pad: frame.pad,
            pressed: self.start.jump_held.pressed(frame.pressed, rules),
            items: frame.items,
            animation_ends: frame.animation_ends,
            speed: self.start.run.speed,
            airborne: self
                .contact
                .map_or_else(|| !self.state.vertical.on_ground(), Contact::rises),
            met: self.met,
        };
        let choice = if self.start.rides_the_elevator(frame) {
            Choice::instead(self.start.pose)
        } else {
            moment.choose(self.start.pose, situation)
        }
        .into();
        self.local(LocalRule::PoseChoice { moment, choice }, self.state);
        Ok(())
    }

    /// Spec: `PerformedMotionDecisions`. Complete the offered request or retained answer once.
    /// A request queries once and becomes an offered actual answer; an answer's completion
    /// consumes it. With no offer (or after completion of the update), this is a no-op.
    /// # Errors
    /// Returns `UnknownPose` when the restored start pose has no ordinary situation.
    pub fn complete(&mut self, f: &Frame, w: &dyn World, r: &Rules) -> Result<(), UnknownPose> {
        let Some(decision) = self.offered.take() else {
            return Ok(());
        };
        match decision {
            MovementDecision::InsideRequest(b) => {
                self.offered = Some(MovementDecision::InsideAnswer(tick(f, w, r).inside(b)));
            }
            MovementDecision::InsideAnswer(cells) => {
                self.outputs.inside = cells;
                self.stage = Stage::Rise;
            }
            MovementDecision::MotionRequest(request) => {
                let mut t = tick(f, w, r);
                let answer = t.go(request.axis);
                self.offered = Some(MovementDecision::MotionAnswer {
                    request,
                    answer,
                    outputs: DecisionOutputs::from_tick(t),
                });
            }
            MovementDecision::MotionAnswer {
                request,
                answer,
                outputs,
            } => {
                self.collect(outputs);
                if let Some(longer) = self.longer.take()
                    && answer.stop.is_none()
                {
                    self.request = Some(longer);
                    return Ok(());
                }
                self.motion_complete(&request, &answer)?;
            }
            MovementDecision::SupportRequest { bounds, surface } => {
                let mut t = tick(f, w, r);
                t.support(
                    bounds,
                    if surface == types::SupportSurface::Floor {
                        Direction::Down
                    } else {
                        Direction::Up
                    },
                );
                if let Some(types::MovementWork::Support(check)) = t.movement_work.pop() {
                    self.offered = Some(MovementDecision::SupportAnswer { check });
                }
            }
            MovementDecision::SupportAnswer { check } => {
                self.state.position.y += px(check.placement.0);
                if matches!(
                    check.result,
                    types::SupportResult::Lifted | types::SupportResult::PushedDown
                ) {
                    self.state.seat = Seat::Seated;
                }
                self.outputs
                    .movement_work
                    .push(types::MovementWork::Support(check));
                self.stage = if self.stage == Stage::Floor {
                    Stage::Ceiling
                } else {
                    Stage::NearWall
                };
            }
            MovementDecision::Local { rule, answer, .. } => {
                self.state = answer;
                if let Some(o) = self.local_outputs.take() {
                    self.collect(o);
                }
                self.local_complete(rule)?;
            }
        }
        Ok(())
    }

    fn local_complete(&mut self, rule: LocalRule) -> Result<(), UnknownPose> {
        match self.stage {
            Stage::Rise => self.stage = Stage::Start,
            Stage::Growth => self.moves_done(),
            Stage::WallChoice => {
                if let LocalRule::WallOutcome { result } = rule {
                    self.outputs.wall_test = result;
                }
                self.stage = Stage::SpaceJump;
            }
            Stage::Start
            | Stage::Dash
            | Stage::Speed
            | Stage::Horizontal
            | Stage::NearWall
            | Stage::SpaceJump
            | Stage::Vertical => self.motion_local_complete()?,
            Stage::Choice
            | Stage::Aim
            | Stage::BodyFit
            | Stage::FitDone
            | Stage::Initialize
            | Stage::Lower
            | Stage::Response
            | Stage::Effects
            | Stage::Finish => self.pose_local_complete(rule),
            Stage::Inside
            | Stage::Walk
            | Stage::Floor
            | Stage::Ceiling
            | Stage::WallLook
            | Stage::VerticalLook
            | Stage::AimLook
            | Stage::FitLook
            | Stage::LowerLook
            | Stage::Done => {}
        }
        Ok(())
    }

    fn travel_probe(&self) -> Probe {
        if self.state.vertical.speed == ZERO {
            Probe::Walk
        } else {
            Probe::Move
        }
    }

    fn start_next(&mut self, frame: &Frame, rules: &Rules) {
        let next = if self.state.pose.facing != Facing::Screen {
            Stage::Dash
        } else if self.start.rides_the_elevator(frame) {
            self.vertical_by = rules.elevator_descent;
            self.vertical_request(
                self.bounds(frame),
                rules.elevator_descent,
                Probe::Move,
                CollisionPurpose::OrdinaryMove,
                rules,
            );
            Stage::VerticalLook
        } else {
            Stage::Choice
        };
        self.local_next = Some(next);
    }

    fn near_wall_next(&mut self, frame: &Frame, rules: &Rules) {
        let next = if self.state.spin == Spin::CarriedOn && frame.pad.any(SIDES) {
            let heading = if frame.pad.any(LEFT) {
                Direction::Right
            } else {
                Direction::Left
            };
            self.request = Some(MotionRequest::new(
                self.bounds(frame),
                heading,
                rules.wall_reach,
                Probe::NearWall,
                CollisionPurpose::NearWallReach,
            ));
            Stage::WallLook
        } else {
            Stage::SpaceJump
        };
        self.local_next = Some(next);
    }

    fn motion_local_complete(&mut self) -> Result<(), UnknownPose> {
        let s = classify(self.start.pose)?;
        match self.stage {
            Stage::Start => {
                if let Some(next) = self.local_next.take() {
                    if next == Stage::Choice {
                        self.outputs.movement_skip = Some(types::MotionSkipReason::FacingPlayer);
                        self.moves_done();
                    } else {
                        self.stage = next;
                    }
                }
            }
            Stage::Dash => self.stage = Stage::Speed,
            Stage::Speed => {
                self.horizontal_amount = self.state.run.speed + self.state.run.dash;
                if self.stride == Stride::Nothing {
                    self.stage = Stage::NearWall;
                } else {
                    self.stage = Stage::Horizontal;
                }
            }
            Stage::Horizontal => self.stage = Stage::Walk,
            Stage::NearWall => {
                if let Some(next) = self.local_next.take() {
                    self.stage = next;
                }
            }
            Stage::SpaceJump => {
                if self.outputs.wall_test == WallTest::Kicked {
                    self.met = Some(Met::WallKick);
                    self.moves_done();
                } else if s.doing == Doing::TakingOff {
                    self.moves_done();
                } else {
                    self.stage = Stage::Vertical;
                }
            }
            Stage::Vertical => self.stage = Stage::VerticalLook,
            Stage::Inside
            | Stage::Walk
            | Stage::Floor
            | Stage::Ceiling
            | Stage::WallLook
            | Stage::VerticalLook
            | Stage::Choice
            | Stage::Aim
            | Stage::AimLook
            | Stage::BodyFit
            | Stage::FitLook
            | Stage::FitDone
            | Stage::Initialize
            | Stage::Lower
            | Stage::LowerLook
            | Stage::Response
            | Stage::Effects
            | Stage::Finish
            | Stage::Rise
            | Stage::Growth
            | Stage::WallChoice
            | Stage::Done => {}
        }
        Ok(())
    }

    fn pose_local_complete(&mut self, rule: LocalRule) {
        match self.stage {
            Stage::Choice => {
                if let LocalRule::PoseChoice { choice, .. } = rule {
                    self.outputs.handed_over = choice.handed_over;
                    self.choice_before_aim = Some(choice.clone());
                    self.accepted_choice = Some(choice.clone());
                    self.stage = if choice.aim_step {
                        Stage::Aim
                    } else {
                        Stage::BodyFit
                    };
                }
            }
            Stage::Aim => self.stage = Stage::AimLook,
            Stage::BodyFit if matches!(rule, LocalRule::BodyGrowth { .. }) => {}
            Stage::BodyFit | Stage::FitDone => {
                self.finish_fit();
                self.stage = Stage::Initialize;
            }
            Stage::Initialize => {
                let lower = self.accepted_choice.as_ref().and_then(|c| c.lower);
                if lower.is_some() {
                    self.stage = Stage::Lower;
                } else {
                    self.set_placed();
                    self.stage = Stage::Response;
                }
            }
            Stage::Lower => self.stage = Stage::LowerLook,
            Stage::Response => self.stage = Stage::Effects,
            Stage::Effects => {
                self.effect_cursor = self.effect_cursor.saturating_add(1);
                self.stage = Stage::Effects;
            }
            Stage::Finish => {
                if let Some(outputs) = self.finish_answer.take() {
                    self.outputs = outputs;
                }
                self.stage = Stage::Done;
            }
            Stage::Inside
            | Stage::Start
            | Stage::Dash
            | Stage::Speed
            | Stage::Horizontal
            | Stage::Walk
            | Stage::Floor
            | Stage::Ceiling
            | Stage::NearWall
            | Stage::WallLook
            | Stage::SpaceJump
            | Stage::Vertical
            | Stage::VerticalLook
            | Stage::AimLook
            | Stage::FitLook
            | Stage::LowerLook
            | Stage::Rise
            | Stage::Growth
            | Stage::WallChoice
            | Stage::Done => {}
        }
    }

    fn motion_complete(
        &mut self,
        request: &MotionRequest,
        moved: &Moved,
    ) -> Result<(), UnknownPose> {
        let s = classify(self.start.pose)?;
        match self.stage {
            Stage::Walk => {
                self.state.position = moved.to;
                self.outputs.travelled[request.direction] = moved.accepted;
                self.walk_stopped = moved.stop.is_some();
                let halted = self.walk_stopped || self.stride == Stride::DashThenHalt;
                if halted {
                    self.state.run.halt();
                }
                if self.walk_stopped || s.doing == Doing::Ball && halted {
                    self.state.run.dashing = Dashing::Off;
                }
                self.stage = Stage::Floor;
            }
            Stage::WallLook => {
                self.state.take_result(moved);
                self.vertical_stop = moved.stop.is_some();
                self.stage = Stage::WallChoice;
            }
            Stage::VerticalLook => {
                if self.vertical_by != ZERO && moved.stop != Some(Stop::Enemy) {
                    self.state.seat = Seat::Unseated;
                }
                let ignored = self.course == Course::Pulled
                    && s.footing == Footing::ChangingShape
                    && moved.stop.is_none()
                    && self.start.pose.facing != Facing::Screen;
                self.outputs.travelled[request.direction] = if ignored {
                    TravelDistance::ZERO
                } else {
                    moved.accepted
                };
                if !ignored {
                    self.state.take_result(moved);
                    if self.start.pose.facing != Facing::Screen {
                        self.vertical_stop = moved.stop.is_some();
                        self.stage = Stage::Growth;
                        return Ok(());
                    }
                }
                self.moves_done();
            }
            Stage::AimLook => {
                self.state.position = moved.to;
                if moved.stop == Some(Stop::Enemy) {
                    self.state.position.x = request.bounds.at.x;
                }
                if moved.stop.is_some() {
                    self.accepted_choice =
                        self.accepted_choice.as_ref().map(SelectedChoice::blocked);
                }
                self.stage = Stage::BodyFit;
            }
            Stage::FitLook => {
                self.state.take_result(moved);
                self.fit_look_complete(moved.stop.map(|_| moved.gone));
                self.stage = Stage::BodyFit;
            }
            Stage::LowerLook => {
                self.state.take_result(moved);
                self.state.position.y += px(moved.gone);
                self.set_placed();
                self.stage = Stage::Response;
            }
            Stage::Inside
            | Stage::Start
            | Stage::Dash
            | Stage::Speed
            | Stage::Horizontal
            | Stage::Floor
            | Stage::Ceiling
            | Stage::NearWall
            | Stage::SpaceJump
            | Stage::Vertical
            | Stage::Choice
            | Stage::Aim
            | Stage::BodyFit
            | Stage::FitDone
            | Stage::Initialize
            | Stage::Lower
            | Stage::Response
            | Stage::Effects
            | Stage::Finish
            | Stage::Rise
            | Stage::Growth
            | Stage::WallChoice
            | Stage::Done => {}
        }
        Ok(())
    }

    fn moves_done(&mut self) {
        self.outputs.heading_down = self.state.vertical.heading_down();
        self.outputs.vertical_speed_after_moves = self.state.vertical.speed;
        self.outputs.dashing_after_moves = self.state.run.dashing;
        self.state.vertical.meet_in_moves(self.met);
        if self.walk_stopped && self.start.pose.action == types::Action::Running {
            self.met = Some(Met::Wall);
        }
        self.stage = Stage::Choice;
    }

    fn begin_fit(&mut self, f: &Frame, r: &Rules) {
        if self.body_fit.is_some() {
            self.next_fit_look(f, r);
            if self.stage == Stage::FitLook {
                self.request_offer(f, r);
            }
            return;
        }
        let Some(c) = &self.accepted_choice else {
            return;
        };
        if c.handed_over || c.pose == self.state.pose || c.pose.facing == Facing::Screen {
            self.stage = Stage::Initialize;
            return;
        }
        if c.pose.action == types::Action::Morphing && !f.items[types::Item::MorphBall] {
            self.accepted_choice = Some(Choice::instead(self.state.pose).into());
            self.contact = None;
            self.stage = Stage::Initialize;
            return;
        }
        let old = (f.body)(self.state.pose);
        let new = (f.body)(c.pose);
        self.body_fit = Some(FitWork {
            old,
            new,
            centre: self.state.position,
            checks: Vec::new(),
            step: FitStep::EnemyAbove,
            here: Rooms::empty(),
            found: Rooms::empty(),
            fitted: Fitted::Taken,
        });
        if new.y > old.y {
            self.local(
                LocalRule::BodyGrowth {
                    old_radius: old,
                    selected_radius: new,
                },
                self.state,
            );
        } else {
            self.local(
                LocalRule::BodyFit {
                    old_radius: old,
                    selected_radius: new,
                    selected_pose: c.pose,
                    result: types::BodyFitResult::NoGrowth,
                },
                self.state,
            );
        }
    }
    fn next_fit_look(&mut self, f: &Frame, r: &Rules) {
        let Some(fit) = &mut self.body_fit else {
            return;
        };
        let g = fit.new.y - fit.old.y;
        let (by, probe, purpose) = match fit.step {
            FitStep::EnemyAbove => (-g, Probe::Enemies, CollisionPurpose::FitRoomAbove),
            FitStep::EnemyBelow => (g, Probe::Enemies, CollisionPurpose::FitRoomBelow),
            FitStep::BlockAbove => (-g, Probe::Blocks, CollisionPurpose::FitRoomAbove),
            FitStep::BlockBelow => (g, Probe::Blocks, CollisionPurpose::FitRoomBelow),
            FitStep::EnemyFar => (
                alone(fit.here, g).unwrap_or_default(),
                Probe::Enemies,
                CollisionPurpose::FitFarClearance,
            ),
            FitStep::BlockFar => (
                alone(fit.here, g).unwrap_or_default(),
                Probe::Blocks,
                CollisionPurpose::FitFarClearance,
            ),
            FitStep::Done => {
                self.stage = Stage::FitDone;
                return;
            }
        };
        // Spec: PerformedMotionDecisions. Growth distance belongs to the completed
        // consideration; each genuinely later request reads its current body geometry.
        let pose = if fit.step == FitStep::EnemyFar {
            self.accepted_choice
                .as_ref()
                .map_or(self.state.pose, |c| c.pose)
        } else {
            self.state.pose
        };
        let b = Bounds {
            at: self.state.position,
            r: (f.body)(pose),
        };
        self.vertical_request(b, px(by), probe, purpose, r);
        self.stage = Stage::FitLook;
    }
    fn fit_look_complete(&mut self, room: Option<Px>) {
        let Some(fit) = &mut self.body_fit else {
            return;
        };
        match fit.step {
            FitStep::EnemyAbove => {
                fit.here.above = room;
                fit.step = FitStep::EnemyBelow;
            }
            FitStep::BlockAbove => {
                fit.here.above = room;
                fit.step = FitStep::BlockBelow;
            }
            FitStep::EnemyBelow => {
                fit.here.below = room;
                fit.step = if alone(fit.here, fit.new.y - fit.old.y).is_some() {
                    FitStep::EnemyFar
                } else {
                    FitStep::BlockAbove
                };
            }
            FitStep::BlockBelow => {
                fit.here.below = room;
                fit.step = if alone(fit.here, fit.new.y - fit.old.y).is_some() {
                    FitStep::BlockFar
                } else {
                    FitStep::Done
                };
            }
            FitStep::EnemyFar => {
                fit.step = if room.is_some() {
                    fit.fitted = Fitted::Refused;
                    FitStep::Done
                } else {
                    FitStep::BlockAbove
                }
            }
            FitStep::BlockFar => {
                if room.is_some() {
                    fit.fitted = Fitted::Refused;
                }
                fit.step = FitStep::Done;
            }
            FitStep::Done => {}
        }
        if matches!(fit.step, FitStep::BlockAbove | FitStep::Done) {
            fit.found = fit.here.merge(fit.found);
            fit.here = Rooms::empty();
        }
    }
    fn fit_decision(&mut self, f: &Frame, r: &Rules) {
        let Some(fit) = &mut self.body_fit else {
            return;
        };
        let mut answer = self.state;
        if fit.fitted != Fitted::Refused {
            if let Some(push) = alone(fit.found, fit.new.y - fit.old.y) {
                answer.position.y += px(push);
                fit.fitted = Fitted::Pushed;
            } else if fit.found.above.is_some() && fit.found.below.is_some() {
                if fit.old.y < r.small_body {
                    fit.fitted = Fitted::Refused;
                } else {
                    let rise = (f.body)(crouched(answer.pose.facing)).y.max(fit.old.y) - fit.old.y;
                    answer.position.y -= px(rise);
                    fit.fitted = Fitted::Crouched { rise };
                }
            }
        }
        let result = match fit.fitted {
            Fitted::Taken => types::BodyFitResult::GrowthClear,
            Fitted::Pushed => types::BodyFitResult::PushedAway,
            Fitted::Refused => types::BodyFitResult::Refused,
            Fitted::Crouched { .. } => types::BodyFitResult::CrouchedInstead,
        };
        let rule = LocalRule::BodyFit {
            old_radius: fit.old,
            selected_radius: fit.new,
            selected_pose: self
                .accepted_choice
                .as_ref()
                .map_or(self.state.pose, |c| c.pose),
            result,
        };
        self.local(rule, answer);
    }
    fn finish_fit(&mut self) {
        let Some(fit) = &self.body_fit else {
            return;
        };
        let result = match fit.fitted {
            Fitted::Taken if fit.new.y <= fit.old.y => types::BodyFitResult::NoGrowth,
            Fitted::Taken => types::BodyFitResult::GrowthClear,
            Fitted::Pushed => types::BodyFitResult::PushedAway,
            Fitted::Refused => types::BodyFitResult::Refused,
            Fitted::Crouched { .. } => types::BodyFitResult::CrouchedInstead,
        };
        let checks = fit.checks.clone();
        self.outputs
            .movement_work
            .push(types::MovementWork::BodyFit(types::BodyFitConsideration {
                old_radius: fit.old,
                new_radius: fit.new,
                centre: fit.centre,
                checks,
                result,
                placement: fit.fitted.moved().then(|| {
                    types::PlacementDistance(self.state.position.y.pixel() - fit.centre.y.pixel())
                }),
            }));
        match fit.fitted {
            Fitted::Refused => {
                self.accepted_choice = Some(Choice::instead(self.state.pose).into());
                self.contact = None;
            }
            Fitted::Crouched { .. } => {
                self.accepted_choice =
                    Some(Choice::instead(crouched(self.state.pose.facing)).into());
                self.contact = None;
            }
            Fitted::Taken | Fitted::Pushed => {}
        }
    }
    fn set_placed(&mut self) {
        let moved = self.body_fit.as_ref().is_some_and(|fit| fit.fitted.moved());
        let lower = self
            .accepted_choice
            .as_ref()
            .is_some_and(|c| c.lower.is_some());
        self.outputs.placed =
            (moved || lower).then(|| self.state.position.y.with_sub(self.start.position.y.sub()));
    }
    fn response_offer(&mut self, f: &Frame, w: &dyn World, r: &Rules) {
        let mut answer = self.state;
        let mut t = tick(f, w, r);
        if let Some(contact) = self.contact {
            let kind = match contact {
                Contact::WallKick => None,
                Contact::Hop => Some(types::FloorResponseKind::SpringHop),
                Contact::Bounce(bounce) => Some(types::FloorResponseKind::Rebound(bounce.then)),
                Contact::Land if self.start.pose.body == Body::Upright => {
                    Some(types::FloorResponseKind::UprightTouchdown)
                }
                Contact::Land => Some(types::FloorResponseKind::BallSettles),
            };
            t.floor_response = kind.map(|kind| types::FloorResponse {
                kind,
                speed: types::VerticalSpeed(answer.vertical.speed),
            });
            answer.vertical.meet(contact, answer.run.dash, &t);
            answer.run.meet(contact);
        }
        self.local_outputs = Some(DecisionOutputs::from_tick(t));
        self.local(
            LocalRule::FloorResponse {
                contact: self.contact,
            },
            answer,
        );
    }
    fn effect_offer(&mut self, f: &Frame, w: &dyn World, r: &Rules) {
        let Some(c) = &self.accepted_choice else {
            return;
        };
        let entered = ENTERING
            .iter()
            .filter(|e| e.on(self.start.pose.action, c.pose))
            .map(|e| e.effect);
        let effect = c
            .effects
            .iter()
            .copied()
            .chain(entered)
            .nth(self.effect_cursor);
        let Some(effect) = effect else {
            self.stage = Stage::Finish;
            return;
        };
        let mut answer = self.state;
        if self.effect_cursor == 0 && c.effects.iter().any(|e| e.lets_go()) {
            answer.run.dashing = Dashing::Off;
        }
        answer.apply(effect, &self.start, &tick(f, w, r));
        self.local(LocalRule::PoseEffect { effect }, answer);
    }
    fn finish_outputs(&mut self, f: &Frame) {
        let (requests, graphics) = crate::touchdown::opportunities(
            self.outputs.floor_response,
            self.contact == Some(Contact::WallKick),
            self.start.pose.action,
            f,
        );
        self.outputs.touchdown_requests = requests;
        self.outputs.touchdown_graphics = graphics;
        self.outputs.standing_animation = StandingAnimation::of(self.start.pose, f.pad);
        self.outputs.sprite = sprite(self.state.pose, &f.items);
        self.outputs.skidding = self.state.run.skids(self.state.pose.facing);
        self.outputs.rising = self.state.vertical.rising();
        self.outputs.dash_began = self.start.run.dashing == Dashing::Off && self.ran;
        self.outputs.dash_lost = (self.start.run.dashing == Dashing::On || self.ran)
            && self.state.run.dashing == Dashing::Off;
    }
}
