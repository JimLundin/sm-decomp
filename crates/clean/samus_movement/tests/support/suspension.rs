//! Spec: `SuspendedMovement`. Strict additive fixtures and full serialized continuations.
use super::*;
use samus_movement::suspended::{LocalRule, MovementDecision, SuspendedMovement};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionChangeCase {
    id: String,
    source: String,
    base: String,
    pub(crate) entries: Vec<String>,
    boundary: DecisionBoundary,
    required_world: Option<RequiredWorld>,
    change: PremiseChange,
    expected: DecisionExpected,
    restoration: Restoration,
    save_points: SavePoints,
    ordinary_equivalence: Equivalence,
    #[serde(default)]
    supplied_poses: Vec<PoseTable>,
}
#[derive(Deserialize)]
enum Equivalence {
    SameActualDecisionInputs,
}
#[derive(Deserialize)]
enum DecisionEntry {
    RisingAndFalling,
}
#[derive(Clone, Copy, Deserialize)]
enum DecisionName {
    RiseEnds,
    VerticalSpeedGrowth,
}
#[derive(Clone, Copy, Deserialize)]
enum ChoicePoint {
    BeforeChoice,
    AfterChoiceBeforeEffect,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DecisionBoundary {
    entry: DecisionEntry,
    decision: DecisionName,
    point: ChoicePoint,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RequiredWorld {
    direction: Direction,
    purpose: types::CollisionPurpose,
    result: types::MotionResult,
    accepted: Signed,
    after_position: [Quantity; 2],
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PremiseChange {
    rules: Option<RulesTable>,
    pad: Option<Vec<ButtonName>>,
}
impl PremiseChange {
    fn apply(&self, f: &mut Frame, rules: &mut Rules) {
        if let Some(r) = &self.rules {
            *rules = r.applied(rules.clone());
        }
        if let Some(pad) = &self.pad {
            f.pad = buttons(pad);
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DecisionExpected {
    before_speed: Signed,
    chosen_speed: Signed,
    completed_speed: Signed,
    matching_world_queries: Option<usize>,
    rising: Option<bool>,
}
impl DecisionName {
    fn matches(self, d: &MovementDecision) -> bool {
        matches!(
            (self, d),
            (
                Self::RiseEnds,
                MovementDecision::Local {
                    rule: LocalRule::RiseEnds,
                    ..
                }
            ) | (
                Self::VerticalSpeedGrowth,
                MovementDecision::Local {
                    rule: LocalRule::VerticalSpeedGrowth,
                    ..
                }
            )
        )
    }
}
impl DecisionChangeCase {
    fn after_choice(&self, saved: &mut Interrupted, d: &MovementDecision) {
        if self.boundary.decision.matches(d) {
            if let MovementDecision::Local { before, answer, .. } = d {
                assert_eq!(before.vertical.speed, self.expected.before_speed.value());
                assert_eq!(answer.vertical.speed, self.expected.chosen_speed.value());
                if let Some(rising) = self.expected.rising {
                    assert_eq!(answer.vertical.phase == Phase::Rising, rising);
                }
            }
            if matches!(self.boundary.point, ChoicePoint::AfterChoiceBeforeEffect) {
                saved.changed = true;
            }
        }
    }
    fn after_completion(&self, saved: &mut Interrupted, d: &MovementDecision) {
        if self.boundary.decision.matches(d) {
            assert_eq!(
                saved.movement.state.vertical.speed,
                self.expected.completed_speed.value()
            );
        }
        let ready = match (self.boundary.decision, d) {
            (DecisionName::RiseEnds, MovementDecision::InsideAnswer(_)) => true,
            (
                DecisionName::VerticalSpeedGrowth,
                MovementDecision::MotionAnswer {
                    request,
                    answer,
                    outputs,
                },
            ) => {
                let required = self.required_world.as_ref().unwrap();
                let matches =
                    request.direction == required.direction && request.purpose == required.purpose;
                if matches {
                    assert_eq!(answer.accepted.0, required.accepted.value());
                    assert_eq!(
                        saved.movement.state.position.x,
                        required.after_position[0].0
                    );
                    assert_eq!(
                        saved.movement.state.position.y,
                        required.after_position[1].0
                    );
                    assert!(outputs.movement_work.iter().any(|w| matches!(w, types::MovementWork::Motion(m) if m.result == required.result)));
                }
                matches
            }
            _ => false,
        };
        if ready && matches!(self.boundary.point, ChoicePoint::BeforeChoice) {
            saved.changed = true;
        }
    }
}
fn drive_decision(c: &DecisionChangeCase, saving: bool) -> Interrupted {
    let mut saved = initial(&c.base);
    while !saved.movement.is_complete() {
        saved = one_changed(saved, None, Some(c), saving);
    }
    if saving {
        saved = restore(saved);
    }
    saved
}
#[test]
fn changed_local_answers_and_complete_restoration() {
    let source = file();
    assert_eq!(source.decision_change_case.len(), 4);
    for c in &source.decision_change_case {
        let _ = (
            &c.source,
            &c.restoration,
            &c.save_points,
            &c.ordinary_equivalence,
            &c.boundary.entry,
        );
        let v = source.vector.iter().find(|v| v.id == c.base).unwrap();
        assert_eq!(run(v), Vec::<String>::new(), "{} unchanged base", c.id);
        let unchanged = drive(&c.base, None, true);
        let (state, outputs) = receipt(v);
        assert_eq!(unchanged.movement.state, state);
        assert_eq!(unchanged.movement.outputs, outputs);
        let ordinary = drive_decision(c, false);
        let resumed = drive_decision(c, true);
        continuation_matches(&ordinary, &resumed);
        assert!(resumed.changed, "{} reached actual boundary", c.id);
        if let Some(required) = &c.required_world {
            let count = resumed.trace.windows(2).filter(|pair| {
                pair[1].completions > pair[0].completions && matches!(&pair[0].movement.offered,
                    Some(MovementDecision::MotionRequest(r)) if r.direction == required.direction && r.purpose == required.purpose)
            }).count();
            assert_eq!(Some(count), c.expected.matching_world_queries);
        }
    }
}
#[test]
fn decision_schema_rejects_unknown_fields() {
    let value: toml::Value = toml::from_str(VECTORS).unwrap();
    let cases = value["decision_change_case"].as_array().unwrap();
    for original in cases {
        assert!(original.clone().try_into::<DecisionChangeCase>().is_ok());
        for field in ["boundary", "change", "expected", "required_world"] {
            let mut bad = original.clone();
            if let Some(table) = bad.get_mut(field).and_then(toml::Value::as_table_mut) {
                table.insert("unknown".into(), toml::Value::Integer(1));
                assert!(bad.try_into::<DecisionChangeCase>().is_err());
            }
        }
        let mut bad = original.clone();
        bad.as_table_mut()
            .unwrap()
            .insert("unknown".into(), toml::Value::Integer(1));
        assert!(bad.try_into::<DecisionChangeCase>().is_err());
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuspendedCase {
    id: String,
    source: String,
    base: String,
    pub(crate) entries: Vec<String>,
    rules: Option<RulesTable>,
    save_points: SavePoints,
    observations: Vec<Observation>,
}
#[derive(Deserialize)]
enum SavePoints {
    AllOfferedAndCompleted,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeCase {
    id: String,
    source: String,
    base: String,
    pub(crate) entries: Vec<String>,
    boundary: Boundary,
    world_replace: WorldTable,
    restoration: Restoration,
    expected: ResumeExpected,
}
#[derive(Deserialize)]
enum Restoration {
    SerializeDropDeserialize,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Boundary {
    purpose: SupportPurpose,
    surface: types::SupportSurface,
    point: BoundaryPoint,
}
#[derive(Deserialize)]
enum SupportPurpose {
    Support,
}
#[derive(Clone, Copy, Deserialize)]
enum BoundaryPoint {
    BeforeQuery,
    AfterAnswerBeforeEffect,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ResumeExpected {
    floor_answer: types::SupportResult,
    floor_after_y: Quantity,
    ceiling_centre_y: Quantity,
    intended_pull: Signed,
    floor_queries: usize,
    ceiling_queries: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
enum Event {
    DashBuilding,
    DashBuilt,
    SpeedBuilding,
    SpeedBuilt,
    HorizontalDisplacement,
    HorizontalResult,
    GroundPull,
    PoseChoice,
    AimTravel,
    AimResult,
    BodyFit,
    PoseInitialized,
    SupportResult,
    SupportConsidered,
}
#[derive(Clone, Copy, Debug, Deserialize)]
enum ObservedAnswer {
    Gain,
    Cap,
    Kept,
    WordwiseCapped,
    NoGrowth,
    GrowthClear,
    PushedAway,
    Refused,
    CrouchedInstead,
    NoSlope,
    NoPlacement,
    Lifted,
    PushedDown,
}
#[derive(Clone, Copy, Debug, Deserialize)]
enum Disposition {
    Ordinary,
    Handover,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Observation {
    event: Event,
    before: Option<Quantity>,
    step: Option<Quantity>,
    sum: Option<Quantity>,
    top: Option<Quantity>,
    answer: Option<ObservedAnswer>,
    speed: Option<Quantity>,
    intended: Option<Signed>,
    retained_total: Option<Quantity>,
    extra: Option<Quantity>,
    gain: Option<Quantity>,
    cap: Option<Quantity>,
    dash: Option<Quantity>,
    accepted: Option<Signed>,
    result: Option<types::MotionResult>,
    current_speed: Option<Quantity>,
    current_dash: Option<Quantity>,
    action: Option<types::Action>,
    aim: Option<types::Aim>,
    disposition: Option<Disposition>,
    old_radius: Option<[u16; 2]>,
    selected_radius: Option<[u16; 2]>,
    surface: Option<types::SupportSurface>,
    before_y: Option<Quantity>,
    after_y: Option<Quantity>,
    centre_y: Option<Quantity>,
    seated: Option<bool>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Receipt {
    event: Event,
    fields: BTreeMap<String, String>,
}
impl Receipt {
    fn new(event: Event) -> Self {
        Self {
            event,
            fields: BTreeMap::new(),
        }
    }
    fn field(mut self, key: &str, value: impl Into<String>) -> Self {
        self.fields.insert(key.to_owned(), value.into());
        self
    }
}
fn radius(r: Radius) -> String {
    format!("{},{}", r.x.0, r.y.0)
}
fn pose_receipt(event: Event, p: Pose) -> Receipt {
    Receipt::new(event)
        .field("action", format!("{:?}", p.action))
        .field("aim", format!("{:?}", p.aim))
}
fn support_offer(d: &MovementDecision) -> Option<Receipt> {
    if let MovementDecision::SupportRequest { bounds, surface } = d {
        Some(
            Receipt::new(Event::SupportConsidered)
                .field("surface", format!("{surface:?}"))
                .field("centre_y", show(bounds.at.y)),
        )
    } else {
        None
    }
}
fn offered(d: &MovementDecision) -> Option<Receipt> {
    if let Some(support) = support_offer(d) {
        return Some(support);
    }
    let MovementDecision::Local { rule, answer, .. } = d else {
        return None;
    };
    match rule {
        LocalRule::DashBuilding { before, gain, cap } => Some(
            Receipt::new(Event::DashBuilding)
                .field("before", show(*before))
                .field("gain", show(*gain))
                .field("cap", show(*cap))
                .field(
                    "answer",
                    if answer.run.dash == *cap {
                        "Cap"
                    } else {
                        "Gain"
                    },
                ),
        ),
        LocalRule::SpeedBuilding {
            before,
            step,
            sum,
            top,
        } => Some(
            Receipt::new(Event::SpeedBuilding)
                .field("before", show(*before))
                .field("step", show(*step))
                .field("sum", show(*sum))
                .field("top", show(*top))
                .field(
                    "answer",
                    if answer.run.speed == *sum {
                        "Kept"
                    } else {
                        "WordwiseCapped"
                    },
                ),
        ),
        LocalRule::HorizontalDisplacement {
            intended,
            retained_total,
        } => Some(
            Receipt::new(Event::HorizontalDisplacement)
                .field("intended", speed_y(intended.0).show())
                .field("retained_total", show(*retained_total)),
        ),
        LocalRule::GroundPull {
            retained_total,
            extra,
            intended,
            seated,
        } => Some(
            Receipt::new(Event::GroundPull)
                .field("retained_total", show(*retained_total))
                .field("extra", show(*extra))
                .field("intended", speed_y(intended.0).show())
                .field("seated", seated.to_string()),
        ),
        LocalRule::PoseChoice { choice, .. } => {
            Some(pose_receipt(Event::PoseChoice, choice.pose).field(
                "disposition",
                if choice.handed_over {
                    "Handover"
                } else {
                    "Ordinary"
                },
            ))
        }
        LocalRule::AimTravel { intended } => {
            Some(Receipt::new(Event::AimTravel).field("intended", speed_y(intended.0).show()))
        }
        LocalRule::BodyFit {
            old_radius,
            selected_radius,
            selected_pose,
            result,
        } => Some(
            pose_receipt(Event::BodyFit, *selected_pose)
                .field("answer", format!("{result:?}"))
                .field("old_radius", radius(*old_radius))
                .field("selected_radius", radius(*selected_radius)),
        ),
        LocalRule::RiseEnds
        | LocalRule::VerticalSpeedGrowth
        | LocalRule::WallOutcome { .. }
        | LocalRule::BodyGrowth { .. }
        | LocalRule::FrameStart
        | LocalRule::Stride
        | LocalRule::NearWall
        | LocalRule::SpaceJump
        | LocalRule::VerticalDisplacement { .. }
        | LocalRule::PoseInitialization { .. }
        | LocalRule::FloorResponse { .. }
        | LocalRule::PoseEffect { .. }
        | LocalRule::Finish => None,
    }
}
fn completed(d: &MovementDecision, work: &SuspendedMovement) -> Option<Receipt> {
    match d {
        MovementDecision::Local {
            rule: LocalRule::DashBuilding { .. },
            ..
        } => Some(Receipt::new(Event::DashBuilt).field("dash", show(work.state.run.dash))),
        MovementDecision::Local {
            rule: LocalRule::SpeedBuilding { .. },
            ..
        } => Some(Receipt::new(Event::SpeedBuilt).field("speed", show(work.state.run.speed))),
        MovementDecision::Local {
            rule: LocalRule::PoseInitialization { .. },
            ..
        } => Some(pose_receipt(Event::PoseInitialized, work.state.pose)),
        MovementDecision::SupportAnswer { check } => Some(
            Receipt::new(Event::SupportResult)
                .field("surface", format!("{:?}", check.surface))
                .field("before_y", show(check.centre.y))
                .field("after_y", show(work.state.position.y))
                .field("answer", format!("{:?}", check.result)),
        ),
        MovementDecision::MotionAnswer {
            request,
            answer,
            outputs,
        } if !request.direction.vertical() => {
            let result = outputs
                .movement_work
                .iter()
                .find_map(|w| {
                    if let types::MovementWork::Motion(m) = w {
                        Some(m.result)
                    } else {
                        None
                    }
                })
                .unwrap();
            horizontal_receipt(work, request.purpose, answer.accepted, result)
        }
        MovementDecision::Local { .. }
        | MovementDecision::MotionRequest(_)
        | MovementDecision::MotionAnswer { .. }
        | MovementDecision::SupportRequest { .. }
        | MovementDecision::InsideRequest(_)
        | MovementDecision::InsideAnswer(_) => None,
    }
}
fn horizontal_receipt(
    work: &SuspendedMovement,
    purpose: types::CollisionPurpose,
    accepted: TravelDistance,
    result: types::MotionResult,
) -> Option<Receipt> {
    match purpose {
        types::CollisionPurpose::AimChange => Some(
            pose_receipt(
                Event::AimResult,
                work.accepted_choice.as_ref().unwrap().pose,
            )
            .field("result", format!("{result:?}")),
        ),
        types::CollisionPurpose::OrdinaryMove => Some(
            Receipt::new(Event::HorizontalResult)
                .field("accepted", speed_y(accepted.0).show())
                .field("result", format!("{result:?}"))
                .field("current_speed", show(work.state.run.speed))
                .field("current_dash", show(work.state.run.dash)),
        ),
        types::CollisionPurpose::GroundFollowing
        | types::CollisionPurpose::NearWallReach
        | types::CollisionPurpose::FitRoomAbove
        | types::CollisionPurpose::FitRoomBelow
        | types::CollisionPurpose::FitFarClearance
        | types::CollisionPurpose::BodyLowering => None,
    }
}
impl Observation {
    fn matches(&self, actual: &Receipt) -> bool {
        self.event == actual.event
            && self
                .fields()
                .into_iter()
                .all(|(k, v)| actual.fields.get(k).is_some_and(|a| *a == v))
    }
    fn fields(&self) -> Vec<(&'static str, String)> {
        let quantities = [
            ("before", self.before),
            ("step", self.step),
            ("sum", self.sum),
            ("top", self.top),
            ("speed", self.speed),
            ("retained_total", self.retained_total),
            ("extra", self.extra),
            ("gain", self.gain),
            ("cap", self.cap),
            ("dash", self.dash),
            ("current_speed", self.current_speed),
            ("current_dash", self.current_dash),
            ("before_y", self.before_y),
            ("after_y", self.after_y),
            ("centre_y", self.centre_y),
        ];
        let mut fields: Vec<_> = quantities
            .into_iter()
            .filter_map(|(k, v)| v.map(|v| (k, show(v.0))))
            .collect();
        fields.extend(
            [("intended", self.intended), ("accepted", self.accepted)]
                .into_iter()
                .filter_map(|(k, v)| v.map(|v| (k, v.show()))),
        );
        fields.extend(self.answer.map(|v| ("answer", format!("{v:?}"))));
        fields.extend(self.action.map(|v| ("action", format!("{v:?}"))));
        fields.extend(self.disposition.map(|v| ("disposition", format!("{v:?}"))));
        fields.extend(self.aim.map(|v| ("aim", format!("{v:?}"))));
        fields.extend(self.result.map(|v| ("result", format!("{v:?}"))));
        fields.extend(self.surface.map(|v| ("surface", format!("{v:?}"))));
        fields.extend(self.seated.map(|v| ("seated", v.to_string())));
        fields.extend(
            [
                ("old_radius", self.old_radius),
                ("selected_radius", self.selected_radius),
            ]
            .into_iter()
            .filter_map(|(k, v)| v.map(|[x, y]| (k, format!("{x},{y}")))),
        );
        fields
    }
}
#[derive(Serialize, Deserialize)]
struct Provider {
    room: Room,
    queries: RefCell<Vec<Query>>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum Query {
    Block(Cell, Block),
    Through(Cell, Way, bool),
    Enemies(usize),
}
impl World for Provider {
    fn block(&self, c: Cell) -> Block {
        let a = self.room.block(c);
        self.queries.borrow_mut().push(Query::Block(c, a));
        a
    }
    fn lets_through(&self, c: Cell, w: Way) -> bool {
        let a = self.room.lets_through(c, w);
        self.queries.borrow_mut().push(Query::Through(c, w, a));
        a
    }
    fn solid_enemies(&self) -> &[Hitbox] {
        self.queries
            .borrow_mut()
            .push(Query::Enemies(self.room.enemies.len()));
        &self.room.enemies
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct SavePoint {
    movement: SuspendedMovement,
    queries: Vec<Query>,
    completions: usize,
    room: String,
    receipts: Vec<Receipt>,
    changed: bool,
    floor_queries: usize,
    ceiling_queries: usize,
}
#[derive(Serialize, Deserialize)]
struct Interrupted {
    movement: SuspendedMovement,
    provider: Provider,
    input: String,
    base: String,
    receipts: Vec<Receipt>,
    trace: Vec<SavePoint>,
    completions: usize,
    changed: bool,
    floor_queries: usize,
    ceiling_queries: usize,
}
fn restore(original: Interrupted) -> Interrupted {
    let encoded = toml::to_string(&original).expect("complete interrupted work serializes");
    drop(original);
    toml::from_str(&encoded).expect("fresh complete interrupted work deserializes")
}
fn fixture_start(v: &Vector) -> Samus {
    let checked = v.checked();
    let frames: Vec<Step> = v.lead_in.iter().chain([&checked]).map(step_of).collect();
    let last_next = v.next_pose.as_ref().map(PoseTable::sized);
    let mut known: Vec<SizedPose> = frames.iter().map(|f| (f.pose, f.radius)).collect();
    known.extend(last_next);
    known.extend(v.after.pose.as_ref().map(PoseTable::sized));
    let mut samus = start(&v.before, frames[0].pose);
    let mut parity = first_parity(v.frame.model(), frames.len());
    for (i, step) in frames.iter().copied().enumerate().take(v.lead_in.len()) {
        let end = frames.get(i.saturating_add(1)).map(|f| (f.pose, f.radius));
        let body = |p| body_size(p, step, end, &known).expect("fixture body");
        let heading = |_| None;
        let f = frame(v, step, parity, &body, &heading);
        samus
            .step(&f, &room(&v.world), &applied_rules(v))
            .expect("lead-in frame");
        parity = other(parity);
    }
    samus
}
fn frame<'a>(
    v: &Vector,
    s: Step,
    parity: Parity,
    body: &'a dyn Fn(Pose) -> Radius,
    heading: &'a dyn Fn(Pose) -> Option<types::FiringDirection>,
) -> Frame<'a> {
    Frame {
        sprite_before_movement: v.sprite_before_movement.unwrap_or(types::Sprite::Plain),
        cinematic_active: v.cinematic_active,
        pad: s.pad,
        pressed: s.pressed,
        items: v.items.iter().map(|i| (i.model(), true)).collect(),
        spin_turned_far: s.spin_turned_far,
        elevator_moving: v.world.elevator_moving,
        parity,
        animation_ends: s.animation_ends,
        body,
        cannon_heading: heading,
    }
}
fn drive(base: &str, change: Option<&ResumeCase>, saving: bool) -> Interrupted {
    let mut saved = initial(base);
    while !saved.movement.is_complete() {
        saved = one(saved, change, saving);
    }
    if saving {
        saved = restore(saved);
    }
    saved
}
fn initial(base: &str) -> Interrupted {
    let source = file();
    let v = source
        .vector
        .iter()
        .find(|v| v.id == base)
        .expect("named base");
    Interrupted {
        movement: SuspendedMovement::new(fixture_start(v)).unwrap(),
        provider: Provider {
            room: room(&v.world),
            queries: RefCell::new(Vec::new()),
        },
        input: VECTORS.to_owned(),
        base: base.to_owned(),
        receipts: Vec::new(),
        trace: Vec::new(),
        completions: 0,
        changed: false,
        floor_queries: 0,
        ceiling_queries: 0,
    }
}
fn one(saved: Interrupted, change: Option<&ResumeCase>, saving: bool) -> Interrupted {
    one_changed(saved, change, None, saving)
}
fn at_inputs(
    saved: &mut Interrupted,
    change: Option<&ResumeCase>,
    local: Option<&DecisionChangeCase>,
    completing: bool,
) -> Option<MovementDecision> {
    let source: VectorFile = toml::from_str(&saved.input).unwrap();
    let v = source.vector.iter().find(|v| v.id == saved.base).unwrap();
    let s = step_of(&v.checked());
    let end = v.next_pose.as_ref().map(PoseTable::sized);
    let mut known = vec![v.pose.sized()];
    known.extend(end);
    known.extend(v.after.pose.as_ref().map(PoseTable::sized));
    // Changed support can walk off into Falling. Import that body's content from the
    // supplied C-ground-rise fixture; the additive base omits this newly reachable pose.
    if change.is_some() {
        let content = source
            .vector
            .iter()
            .find(|v| v.id == "C-ground-rise")
            .unwrap();
        known.extend(content.next_pose.as_ref().map(PoseTable::sized));
    }
    if let Some(c) = local {
        known.extend(c.supplied_poses.iter().map(PoseTable::sized));
    }
    let body = |p| body_size(p, s, end, &known).expect("actual selected body is supplied");
    let heading = |p| {
        if p == s.pose {
            v.previous_cannon_heading
        } else {
            end.filter(|e| e.0 == p).and(v.cannon_heading)
        }
    };
    let mut f = frame(v, s, v.frame.model(), &body, &heading);
    let mut rules = applied_rules(v);
    if saved.changed
        && let Some(c) = local
    {
        c.change.apply(&mut f, &mut rules);
    }
    if completing {
        saved
            .movement
            .complete(&f, &saved.provider, &rules)
            .unwrap();
        None
    } else {
        saved
            .movement
            .offer(&f, &saved.provider, &rules)
            .unwrap()
            .cloned()
    }
}
fn one_changed(
    mut saved: Interrupted,
    change: Option<&ResumeCase>,
    local: Option<&DecisionChangeCase>,
    saving: bool,
) -> Interrupted {
    let d = at_inputs(&mut saved, change, local, false).expect("unfinished offer");
    if let Some(receipt) = offered(&d) {
        saved.receipts.push(receipt);
    }
    if let Some(change) = change {
        replace(&mut saved, change, &d);
    }
    if let Some(c) = local {
        c.after_choice(&mut saved, &d);
    }
    checkpoint(&mut saved);
    if saving {
        saved = restore(saved);
    }
    if let MovementDecision::SupportRequest { surface, .. } = d {
        if surface == types::SupportSurface::Floor {
            saved.floor_queries = saved.floor_queries.saturating_add(1);
        } else {
            saved.ceiling_queries = saved.ceiling_queries.saturating_add(1);
        }
    }
    at_inputs(&mut saved, change, local, true);
    saved.completions = saved.completions.saturating_add(1);
    if let Some(c) = local {
        c.after_completion(&mut saved, &d);
    }
    if let Some(receipt) = completed(&d, &saved.movement) {
        saved.receipts.push(receipt);
    }
    checkpoint(&mut saved);
    if saving {
        saved = restore(saved);
    }
    saved
}
fn replace(saved: &mut Interrupted, c: &ResumeCase, d: &MovementDecision) {
    if saved.changed {
        return;
    }
    let at = match (c.boundary.point, d) {
        (BoundaryPoint::BeforeQuery, MovementDecision::SupportRequest { surface, .. }) => {
            *surface == c.boundary.surface
        }
        (BoundaryPoint::AfterAnswerBeforeEffect, MovementDecision::SupportAnswer { check }) => {
            check.surface == c.boundary.surface
        }
        _ => false,
    };
    if at {
        saved.provider.room = room(&c.world_replace);
        saved.changed = true;
    }
}
fn checkpoint(saved: &mut Interrupted) {
    saved.trace.push(SavePoint {
        movement: saved.movement.clone(),
        queries: saved.provider.queries.borrow().clone(),
        completions: saved.completions,
        room: toml::to_string(&saved.provider.room).unwrap(),
        receipts: saved.receipts.clone(),
        changed: saved.changed,
        floor_queries: saved.floor_queries,
        ceiling_queries: saved.ceiling_queries,
    });
}
fn once_only_queries(saved: &Interrupted) {
    for pair in saved.trace.windows(2) {
        assert!(
            pair[1]
                .movement
                .outputs
                .movement_work
                .starts_with(&pair[0].movement.outputs.movement_work)
        );
        assert!(
            pair[1]
                .movement
                .outputs
                .reached
                .starts_with(&pair[0].movement.outputs.reached)
        );
        assert!(
            pair[1]
                .movement
                .outputs
                .collision_tests
                .starts_with(&pair[0].movement.outputs.collision_tests)
        );
        if pair[1].completions == pair[0].completions {
            continue;
        }
        match &pair[0].movement.offered {
            Some(MovementDecision::MotionRequest(request)) => {
                assert!(
                    matches!(&pair[1].movement.offered, Some(MovementDecision::MotionAnswer { request: answered, .. }) if answered == request)
                );
                let enemy_calls = pair[1].queries[pair[0].queries.len()..]
                    .iter()
                    .filter(|q| matches!(q, Query::Enemies(_)))
                    .count();
                let Some(MovementDecision::MotionAnswer { outputs, .. }) =
                    &pair[1].movement.offered
                else {
                    panic!("a completed motion request exposes its actual answer")
                };
                let world = outputs
                    .movement_work
                    .iter()
                    .find_map(|w| match w {
                        types::MovementWork::Motion(m) => Some(m.world),
                        types::MovementWork::Support(_) | types::MovementWork::BodyFit(_) => None,
                    })
                    .unwrap();
                assert_eq!(
                    enemy_calls,
                    usize::from(world != types::RequestedWorld::Blocks)
                );
            }
            Some(MovementDecision::InsideRequest(_) | MovementDecision::SupportRequest { .. }) => {}
            _ => assert_eq!(
                pair[0].queries, pair[1].queries,
                "answer/effect completion queries nothing"
            ),
        }
    }
}
fn continuation_matches(a: &Interrupted, b: &Interrupted) {
    once_only_queries(a);
    once_only_queries(b);
    assert_eq!(a.input, b.input, "complete supplied input content");
    assert_eq!(a.base, b.base);
    assert_eq!(a.changed, b.changed);
    assert_eq!(a.floor_queries, b.floor_queries);
    assert_eq!(a.ceiling_queries, b.ceiling_queries);
    assert_eq!(
        toml::to_string(&a.provider.room).unwrap(),
        toml::to_string(&b.provider.room).unwrap()
    );
    assert_eq!(
        a.trace, b.trace,
        "complete actual State, pending work and ordered outputs at EVERY offer and completion"
    );
    assert_eq!(a.movement.state, b.movement.state, "complete State");
    assert_eq!(
        a.movement.outputs, b.movement.outputs,
        "all ordered outputs"
    );
    assert_eq!(
        *a.provider.queries.borrow(),
        *b.provider.queries.borrow(),
        "all actual world calls and answers once, in order"
    );
    assert_eq!(
        a.receipts, b.receipts,
        "complete semantic continuation, not just expected subsequence"
    );
    assert_eq!(a.completions, b.completions, "consumed progress");
}
#[test]
fn additive_tables_reject_unknown_fields_and_names() {
    let fixture = "id='strict'\nsource='constructed'\nbase='C-cap'\nentries=['SuspendedMovement']\nsave_points='AllOfferedAndCompleted'\nobservations=[{event='SpeedBuilding', answer='Kept', action='Running', disposition='Ordinary'}]";
    assert!(toml::from_str::<SuspendedCase>(fixture).is_ok());
    for (old, new) in [
        ("event='SpeedBuilding'", "event='SpeedGuess'"),
        ("answer='Kept'", "answer='Guessed'"),
        ("action='Running'", "action='Flying'"),
        ("disposition='Ordinary'", "disposition='Cached'"),
        ("answer='Kept'", "unknown=1"),
        ("AllOfferedAndCompleted", "OnlyListed"),
    ] {
        assert!(
            toml::from_str::<SuspendedCase>(&fixture.replace(old, new)).is_err(),
            "{new}"
        );
    }
    assert!(toml::from_str::<SuspendedCase>(&format!("{fixture}\nunknown=1")).is_err());
}
#[test]
fn all_additive_suspended_vectors() {
    let source = file();
    assert_eq!(source.suspended_update.len(), 8);
    for c in &source.suspended_update {
        let _ = (&c.source, &c.save_points);
        let v = source.vector.iter().find(|v| v.id == c.base).unwrap();
        if let Some(rules) = &c.rules {
            assert_eq!(rules.applied(Rules::default()), applied_rules(v));
        }
        assert_eq!(run(v), Vec::<String>::new(), "{} base", c.id);
        let ordinary = drive(&c.base, None, false);
        let resumed = drive(&c.base, None, true);
        continuation_matches(&ordinary, &resumed);
        let (state, outputs) = receipt(v);
        assert_eq!(
            resumed.movement.state, state,
            "{} ordinary entry complete State",
            c.id
        );
        assert_eq!(
            resumed.movement.outputs, outputs,
            "{} ordinary entry all outputs",
            c.id
        );
        let mut receipts = resumed.receipts.iter();
        for expected in &c.observations {
            assert!(
                receipts.any(|a| expected.matches(a)),
                "{} missing {:?} {:?}",
                c.id,
                expected.event,
                expected.fields()
            );
        }
    }
}
#[test]
fn changed_world_before_and_after_support_answer() {
    let source = file();
    assert_eq!(source.resume_case.len(), 2);
    for c in &source.resume_case {
        let _ = (&c.source, &c.restoration, &c.boundary.purpose);
        let ordinary = drive(&c.base, Some(c), false);
        let resumed = drive(&c.base, Some(c), true);
        continuation_matches(&ordinary, &resumed);
        assert!(resumed.changed, "{} reached boundary", c.id);
        check_changed(&resumed, &c.expected);
    }
}
#[test]
fn nested_fit_candidate_and_cell_order_survive_all_save_points() {
    for base in [
        "C-attempt-fit-push",
        "C-attempt-fit-substitute",
        "C-attempt-fit-refuse",
        "C-fit-step-ceiling",
        "C-fit-step-enemy",
        "C-attempt-running-gap-touch",
        "C-attempt-running-touch-gap",
    ] {
        let ordinary = drive(base, None, false);
        let resumed = drive(base, None, true);
        continuation_matches(&ordinary, &resumed);
    }
}

fn resume_from(done: &Interrupted, point: &SavePoint) -> Interrupted {
    restore(Interrupted {
        movement: point.movement.clone(),
        provider: Provider {
            room: toml::from_str(&point.room).unwrap(),
            queries: RefCell::new(point.queries.clone()),
        },
        input: done.input.clone(),
        base: done.base.clone(),
        receipts: point.receipts.clone(),
        trace: Vec::new(),
        completions: point.completions,
        changed: point.changed,
        floor_queries: point.floor_queries,
        ceiling_queries: point.ceiling_queries,
    })
}
fn drain(mut saved: Interrupted) -> Interrupted {
    while !saved.movement.is_complete() {
        saved = one(saved, None, true);
    }
    saved
}
#[test]
fn lost_pre_stop_amount_and_repeated_or_reordered_outputs_are_detected() {
    let done = drive("C-aim-enemy-stop", None, false);
    let point = done
        .trace
        .iter()
        .find(|p| {
            matches!(
                p.movement.offered,
                Some(MovementDecision::SupportRequest {
                    surface: types::SupportSurface::Floor,
                    ..
                })
            )
        })
        .unwrap();
    let mut lost = resume_from(&done, point);
    assert_eq!(lost.movement.state.run.speed, types::ZERO);
    assert_ne!(lost.movement.horizontal_amount, types::ZERO);
    lost.movement.horizontal_amount = types::ZERO;
    let lost = drain(lost);
    assert_ne!(
        done.movement.outputs, lost.movement.outputs,
        "ground pull must use the pre-stop amount"
    );
    let mut repeated = done.movement.outputs.clone();
    repeated
        .movement_work
        .push(repeated.movement_work[0].clone());
    assert_ne!(
        done.movement.outputs, repeated,
        "a repeated request fails the complete comparison"
    );
    let mut reordered = done.movement.outputs.clone();
    reordered.movement_work.swap(0, 1);
    assert_ne!(done.movement.outputs, reordered, "order is load bearing");
}
#[test]
fn stale_support_geometry_is_detected() {
    let done = drive("C-attempt-running-empty", None, false);
    let point = done
        .trace
        .iter()
        .find(|p| {
            matches!(
                p.movement.offered,
                Some(MovementDecision::SupportRequest {
                    surface: types::SupportSurface::Ceiling,
                    ..
                })
            )
        })
        .unwrap();
    let mut stale = resume_from(&done, point);
    if let Some(MovementDecision::SupportRequest { bounds, .. }) = &mut stale.movement.offered {
        bounds.at.y = stale.movement.start.position.y;
    }
    let stale = drain(stale);
    assert_ne!(
        done.movement.outputs, stale.movement.outputs,
        "ceiling must use the floor placement"
    );
}
#[test]
fn already_offered_local_answers_ignore_changed_completion_inputs() {
    for base in ["C-adj-aim-step", "C-adj-dash-gain", "C-attempt-fit-push"] {
        let done = drive(base, None, false);
        for pair in done.trace.as_chunks::<2>().0 {
            if matches!(
                pair[0].movement.offered,
                Some(MovementDecision::Local { .. })
            ) {
                changed_local_completion(&done, &pair[0], &pair[1]);
            }
        }
    }
}
fn changed_local_completion(done: &Interrupted, offered: &SavePoint, completed: &SavePoint) {
    let mut saved = resume_from(done, offered);
    let source = file();
    let v = source.vector.iter().find(|v| v.id == done.base).unwrap();
    let body = |_| px_pair([1, 1]);
    let heading = |_| None;
    let mut frame = frame(v, step_of(&v.checked()), v.frame.model(), &body, &heading);
    frame.pad = types::Buttons(0);
    frame.pressed = types::Buttons(0);
    frame.items = types::Set::default();
    frame.elevator_moving = !frame.elevator_moving;
    frame.cinematic_active = !frame.cinematic_active;
    let rules = Rules {
        aim_step: types::ZERO,
        ground_pull: types::ZERO,
        dash_gain: types::ZERO,
        ..Rules::default()
    };
    assert_eq!(
        saved
            .movement
            .offer(&frame, &saved.provider, &rules)
            .unwrap(),
        offered.movement.offered.as_ref()
    );
    saved
        .movement
        .complete(&frame, &saved.provider, &rules)
        .unwrap();
    assert_eq!(
        saved.movement, completed.movement,
        "retained answer and dependent effects applied once despite changed premises"
    );
    assert_eq!(*saved.provider.queries.borrow(), completed.queries);
}

fn after_world(done: &Interrupted, purpose: types::CollisionPurpose) -> &SavePoint {
    done.trace.windows(2).find_map(|pair| {
        (pair[1].completions > pair[0].completions && matches!(&pair[0].movement.offered, Some(MovementDecision::MotionAnswer { request, .. }) if request.purpose == purpose))
            .then_some(&pair[1])
    }).unwrap()
}
#[test]
fn later_wall_outcome_uses_current_then_retained_animation_and_press() {
    let done = drive("WK1", None, false);
    let point = after_world(&done, types::CollisionPurpose::NearWallReach);
    let source = file();
    let v = source.vector.iter().find(|v| v.id == done.base).unwrap();
    let body = |_| v.pose.sized().1;
    let heading = |_| None;
    let mut f = frame(v, step_of(&v.checked()), v.frame.model(), &body, &heading);
    let mut current = resume_from(&done, point);
    let queries = current.provider.queries.borrow().clone();
    f.spin_turned_far = false;
    let d = current
        .movement
        .offer(&f, &current.provider, &Rules::default())
        .unwrap()
        .cloned()
        .unwrap();
    assert!(matches!(
        d,
        MovementDecision::Local {
            rule: LocalRule::WallOutcome {
                result: samus_movement::WallTest::Readied
            },
            ..
        }
    ));
    assert_eq!(
        current.movement.outputs.wall_test,
        samus_movement::WallTest::Neither
    );
    current = restore(current);
    f.spin_turned_far = true;
    f.pressed = types::button::JUMP;
    assert_eq!(
        current
            .movement
            .offer(&f, &current.provider, &Rules::default())
            .unwrap(),
        Some(&d)
    );
    current
        .movement
        .complete(&f, &current.provider, &Rules::default())
        .unwrap();
    assert_eq!(
        current.movement.outputs.wall_test,
        samus_movement::WallTest::Readied
    );
    assert_eq!(*current.provider.queries.borrow(), queries);
    let mut current = resume_from(&done, point);
    f.pressed = Buttons::default();
    let d = current
        .movement
        .offer(&f, &current.provider, &Rules::default())
        .unwrap()
        .cloned()
        .unwrap();
    assert!(matches!(
        d,
        MovementDecision::Local {
            rule: LocalRule::WallOutcome {
                result: samus_movement::WallTest::Neither
            },
            ..
        }
    ));
}
#[test]
fn later_clearance_reads_current_body_but_retains_queried_geometry() {
    let done = drive("C-attempt-fit-push", None, false);
    let point = after_world(&done, types::CollisionPurpose::FitRoomAbove);
    let source = file();
    let v = source.vector.iter().find(|v| v.id == done.base).unwrap();
    let body = |_| px_pair([6, 19]);
    let other_body = |_| px_pair([7, 19]);
    let heading = |_| None;
    let f = frame(v, step_of(&v.checked()), v.frame.model(), &body, &heading);
    let other = frame(
        v,
        step_of(&v.checked()),
        v.frame.model(),
        &other_body,
        &heading,
    );
    let mut saved = resume_from(&done, point);
    let d = saved
        .movement
        .offer(&f, &saved.provider, &Rules::default())
        .unwrap()
        .cloned()
        .unwrap();
    assert!(
        matches!(&d, MovementDecision::MotionRequest(r) if r.bounds.r == body(v.pose.pose()) && r.direction == Direction::Down)
    );
    saved = restore(saved);
    assert_eq!(
        saved
            .movement
            .offer(&other, &saved.provider, &Rules::default())
            .unwrap(),
        Some(&d)
    );
    saved
        .movement
        .complete(&other, &saved.provider, &Rules::default())
        .unwrap();
    let answer = saved.movement.offered.clone();
    let queries = saved.provider.queries.borrow().clone();
    saved = restore(saved);
    assert_eq!(saved.movement.offered, answer);
    saved
        .movement
        .complete(&other, &saved.provider, &Rules::default())
        .unwrap();
    assert_eq!(*saved.provider.queries.borrow(), queries);
    let d = saved
        .movement
        .offer(&other, &saved.provider, &Rules::default())
        .unwrap()
        .cloned()
        .unwrap();
    assert!(
        matches!(d, MovementDecision::MotionRequest(r) if r.bounds.r == other_body(v.pose.pose()))
    );
}

#[test]
fn finished_restoration_cannot_repeat_effects_or_world_queries() {
    let mut done = drive("C-attempt-aim-no-growth", None, true);
    let before = done.movement.clone();
    let queries = done.provider.queries.borrow().clone();
    let body = |_| px_pair([5, 21]);
    let heading = |_| None;
    let source = file();
    let v = source.vector.iter().find(|v| v.id == done.base).unwrap();
    let frame = frame(v, step_of(&v.checked()), v.frame.model(), &body, &heading);
    assert!(
        done.movement
            .offer(&frame, &done.provider, &Rules::default())
            .unwrap()
            .is_none()
    );
    done.movement
        .complete(&frame, &done.provider, &Rules::default())
        .unwrap();
    assert_eq!(done.movement, before);
    assert_eq!(*done.provider.queries.borrow(), queries);
}

fn check_changed(resumed: &Interrupted, e: &ResumeExpected) {
    let support: Vec<_> = resumed
        .movement
        .outputs
        .movement_work
        .iter()
        .filter_map(|w| {
            if let types::MovementWork::Support(s) = w {
                Some(s)
            } else {
                None
            }
        })
        .collect();
    let floor = support
        .iter()
        .find(|s| s.surface == types::SupportSurface::Floor)
        .unwrap();
    let ceiling = support
        .iter()
        .find(|s| s.surface == types::SupportSurface::Ceiling)
        .unwrap();
    assert_eq!(floor.result, e.floor_answer);
    assert_eq!(
        floor.centre.y + types::px(floor.placement.0),
        e.floor_after_y.0
    );
    assert_eq!(ceiling.centre.y, e.ceiling_centre_y.0);
    let pull = resumed
        .movement
        .outputs
        .movement_work
        .iter()
        .find_map(|w| match w {
            types::MovementWork::Motion(m)
                if m.purpose == types::CollisionPurpose::GroundFollowing =>
            {
                Some(m.intended)
            }
            types::MovementWork::Motion(_)
            | types::MovementWork::Support(_)
            | types::MovementWork::BodyFit(_) => None,
        })
        .unwrap();
    assert_eq!(pull.0, e.intended_pull.value());
    assert_eq!(resumed.floor_queries, e.floor_queries);
    assert_eq!(resumed.ceiling_queries, e.ceiling_queries);
}
