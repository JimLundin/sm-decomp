//! Projections for the observed floor clip, selected fit and ordered touchdown consumers.

use audio_transport::{CueId, QueueAllowance, SenderProgress, SoundGroup};
use game::production::{
    Production,
    appearance::TouchdownGraphics,
    movement_touchdown::{Outputs, Publication, State},
};
use game::state::owned::OwnedState;
use serde::Deserialize;
use std::collections::VecDeque;
use types::{
    Action, BodyFitConsideration, BodyFitResult, CollisionPurpose, Direction, FloorResponseKind,
    MotionAttempt, MotionResult, MovementWork, RequestEffect, RequestedWorld,
};

use super::super::{
    compare::Observation, data_failure, readers::ExpectedCut, result::Outcome, sender_offer,
};

const CLIP: &str = "movement.touchdown.floor_clip";
const FIT: &str = "movement.touchdown.body_fit";
const RESPONSE: &str = "movement.touchdown.floor_response";
const OFFER: &str = "sender.touchdown_offer";
const GRAPHICS: &str = "appearance.touchdown_graphics";

#[derive(Deserialize)]
pub(super) struct Captured {
    pub(super) cut: u64,
    selected_action: String,
    fit: BodyFitResult,
    vertical_requested_down_fp16: i64,
    movement_floor_clipped_down_fp16: i64,
    body_before_fit_x_fp16: i64,
    body_before_fit_y_fp16: i64,
    old_body_radius_px: [u16; 2],
    selected_body_y_radius_px: u16,
    fit_growth_y_px: u16,
    enemy_up_clear_px: u16,
    enemy_down_clear_px: u16,
    terrain_up_clear_px: u16,
    terrain_down_blocked_clear_px: u16,
    terrain_up_retest_clear_px: u16,
    body_after_fit_x_fp16: i64,
    body_after_fit_y_fp16: i64,
    accepted_floor: FloorResponseKind,
    pulled_speed_before_response_fp16: i64,
    cue: CueId,
    group: SoundGroup,
    allowance: QueueAllowance,
    touchdown_graphics: bool,
    graphics_placement: String,
    contact_service_queue: VecDeque<CueId>,
    contact_service_progress: SenderProgress,
    contact_service_master_ticks: u64,
    observed: Markers,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Markers {
    contact_ingress: Marker,
    graphics_opportunity: Marker,
    clear_graphics_slots: Marker,
    graphics_complete: Marker,
    clear_vertical_motion: Marker,
}

#[derive(Deserialize)]
struct Marker {
    master_ticks: u64,
}

impl Captured {
    pub(super) fn compare_with(
        self,
        fifo: &sender_offer::ContactFifo,
    ) -> Result<ExpectedCut, Outcome> {
        let graphics_arm = self.graphics_placement == "ClearAtmosphericSlots2And3";
        if self.selected_action != "Landing"
            || !self.touchdown_graphics
            || !graphics_arm
            || self.group != SoundGroup::Contact
            || fifo.ingress_prefetch != self.observed.contact_ingress.master_ticks
            || self.observed.contact_ingress.master_ticks
                >= self.observed.graphics_opportunity.master_ticks
            || self.observed.graphics_opportunity.master_ticks
                >= self.observed.clear_graphics_slots.master_ticks
            || self.observed.clear_graphics_slots.master_ticks
                >= self.observed.graphics_complete.master_ticks
            || self.observed.graphics_complete.master_ticks
                >= self.observed.clear_vertical_motion.master_ticks
            || self.observed.clear_vertical_motion.master_ticks >= self.contact_service_master_ticks
            || fifo.at != audio_transport::CpuMasterTicks(self.contact_service_master_ticks)
            || fifo.queue != self.contact_service_queue
            || fifo.progress != self.contact_service_progress
        {
            return Err(data_failure(
                Some(self.cut),
                "touchdown independent consumer bracket",
            ));
        }
        let semantics = self.fields(fifo);
        let mut fields = semantics.clone();
        fields.push(offer([
            0,
            i64::from(sender_offer::cue_tag(self.cue)),
            allowance(self.allowance),
        ]));
        fields.push(graphics([1, 1, i64::from(graphics_arm)]));
        Ok(ExpectedCut {
            fields,
            semantic_count: semantics.len(),
            semantics,
            output_count: 2,
            replies: 0,
        })
    }

    fn fields(&self, fifo: &sender_offer::ContactFifo) -> Vec<Observation> {
        let mut fields = body(self.body_after_fit_x_fp16, self.body_after_fit_y_fp16, 0);
        fields.push(sequence(
            CLIP,
            [
                self.vertical_requested_down_fp16,
                self.movement_floor_clipped_down_fp16,
                motion_result(MotionResult::TerrainStopped),
            ],
        ));
        let [old_x, old_y] = self.old_body_radius_px;
        let [before_x, before_y] = [self.body_before_fit_x_fp16, self.body_before_fit_y_fp16];
        let growth = i64::from(self.fit_growth_y_px) << 16;
        let mut fit = vec![
            fit_result(self.fit),
            before_x,
            before_y,
            i64::from(old_x),
            i64::from(old_y),
            i64::from(self.selected_body_y_radius_px),
            self.body_after_fit_y_fp16.saturating_sub(before_y) >> 16,
        ];
        for (direction, population, stop, clear) in [
            (
                Direction::Up,
                RequestedWorld::SolidEnemies,
                MotionResult::Unobstructed,
                self.enemy_up_clear_px,
            ),
            (
                Direction::Down,
                RequestedWorld::SolidEnemies,
                MotionResult::Unobstructed,
                self.enemy_down_clear_px,
            ),
            (
                Direction::Up,
                RequestedWorld::Blocks,
                MotionResult::Unobstructed,
                self.terrain_up_clear_px,
            ),
            (
                Direction::Down,
                RequestedWorld::Blocks,
                MotionResult::TerrainStopped,
                self.terrain_down_blocked_clear_px,
            ),
            (
                Direction::Up,
                RequestedWorld::Blocks,
                MotionResult::Unobstructed,
                self.terrain_up_retest_clear_px,
            ),
        ] {
            let requested = if direction == Direction::Up {
                growth.wrapping_neg()
            } else {
                growth
            };
            fit.extend(clearance(
                direction,
                population,
                stop,
                [before_x, before_y],
                requested,
                i64::from(clear),
                if population == RequestedWorld::SolidEnemies {
                    2
                } else {
                    0
                },
            ));
        }
        fields.push(sequence(FIT, fit));
        fields.push(sequence(
            RESPONSE,
            [
                1,
                floor_response(self.accepted_floor),
                self.pulled_speed_before_response_fp16,
            ],
        ));
        fields.extend(sender_offer::observations(&fifo.queue, &fifo.progress));
        fields
    }
}

fn body(x: i64, y: i64, downward_speed: i64) -> Vec<Observation> {
    vec![
        Observation::quantity("movement.position.x", x),
        Observation::quantity("movement.position.y", y),
        Observation::quantity("movement.vertical_speed", downward_speed),
    ]
}

fn sequence(field: &str, values: impl IntoIterator<Item = i64>) -> Observation {
    Observation {
        field: field.to_owned(),
        bytes: values.into_iter().flat_map(i64::to_le_bytes).collect(),
    }
}

fn offer(values: impl IntoIterator<Item = i64>) -> Observation {
    sequence(OFFER, values)
}

fn graphics(values: impl IntoIterator<Item = i64>) -> Observation {
    sequence(GRAPHICS, values)
}

const fn motion_result(result: MotionResult) -> i64 {
    match result {
        MotionResult::Unobstructed => 0,
        MotionResult::EnemyStopped => 1,
        MotionResult::TerrainStopped => 2,
    }
}

const fn fit_result(result: BodyFitResult) -> i64 {
    match result {
        BodyFitResult::NoGrowth => 0,
        BodyFitResult::GrowthClear => 1,
        BodyFitResult::PushedAway => 2,
        BodyFitResult::Refused => 3,
        BodyFitResult::CrouchedInstead => 4,
    }
}

const fn floor_response(kind: FloorResponseKind) -> i64 {
    match kind {
        FloorResponseKind::UprightTouchdown => 1,
        FloorResponseKind::BallSettles
        | FloorResponseKind::SpringHop
        | FloorResponseKind::Rebound(_)
        | FloorResponseKind::ChangingBodyStops
        | FloorResponseKind::TurningStops
        | FloorResponseKind::GroundedBallStops => 0,
    }
}

const fn allowance(allowance: QueueAllowance) -> i64 {
    match allowance {
        QueueAllowance::One => 1,
        QueueAllowance::Three => 3,
        QueueAllowance::Six => 6,
        QueueAllowance::Nine => 9,
        QueueAllowance::Fifteen => 15,
    }
}

const fn clearance(
    direction: Direction,
    population: RequestedWorld,
    stop: MotionResult,
    centre: [i64; 2],
    requested: i64,
    cleared: i64,
    solid_count: i64,
) -> [i64; 8] {
    let direction = match direction {
        Direction::Up => 0,
        Direction::Down => 1,
        Direction::Left => 2,
        Direction::Right => 3,
    };
    let population = match population {
        RequestedWorld::SolidEnemies => 0,
        RequestedWorld::Blocks => 1,
        RequestedWorld::EnemiesThenBlocks => 2,
    };
    let [x, y] = centre;
    [
        direction,
        population,
        motion_result(stop),
        x,
        y,
        requested,
        cleared,
        solid_count,
    ]
}

fn observed_clearance(attempt: &MotionAttempt) -> [i64; 8] {
    let RequestEffect::Clearance(available) = attempt.effect else {
        return [-1; 8];
    };
    clearance(
        attempt.direction,
        attempt.world,
        attempt.result,
        [i64::from(attempt.start.x.0), i64::from(attempt.start.y.0)],
        i64::from(i32::from_le_bytes(attempt.intended.0.0.to_le_bytes())),
        i64::from(available.0.0),
        attempt.enemy_search.as_ref().map_or(0, |search| {
            i64::try_from(search.supplied_count.0).unwrap_or(i64::MAX)
        }),
    )
}

fn observed_clip(attempt: &MotionAttempt) -> Option<[i64; 3]> {
    if attempt.direction != Direction::Down || attempt.purpose != CollisionPurpose::OrdinaryMove {
        return None;
    }
    let RequestEffect::Travel(distance) = attempt.effect else {
        return None;
    };
    Some([
        i64::from(attempt.intended.0.0),
        i64::from(distance.0.0),
        motion_result(attempt.result),
    ])
}

fn observed_fit(fit: &BodyFitConsideration) -> impl Iterator<Item = i64> {
    [
        fit_result(fit.result),
        i64::from(fit.centre.x.0),
        i64::from(fit.centre.y.0),
        i64::from(fit.old_radius.x.0),
        i64::from(fit.old_radius.y.0),
        i64::from(fit.new_radius.y.0),
        fit.placement.map_or(i64::MIN, |placement| {
            i64::from(i16::from_le_bytes(placement.0.0.to_le_bytes()))
        }),
    ]
    .into_iter()
    .chain(fit.checks.iter().flat_map(observed_clearance))
}

fn state(owner: &Production) -> Result<&State, Outcome> {
    let OwnedState::MovementTouchdown(state) = owner.state() else {
        return Err(data_failure(None, "touchdown projection scope"));
    };
    Ok(state)
}

pub(in crate::replay) fn actual(
    owner: &Production,
    outputs: &Outputs,
) -> Result<Vec<Observation>, Outcome> {
    let state = state(owner)?;
    let mut fields = state_fields(state);
    let (mut clip, mut fit) = (Vec::new(), Vec::new());
    for work in &outputs.movement.movement_work {
        match work {
            MovementWork::Motion(attempt) => {
                clip.extend(observed_clip(attempt).into_iter().flatten());
            }
            MovementWork::BodyFit(attempt) => fit.extend(observed_fit(attempt)),
            MovementWork::Support(_) => (),
        }
    }
    fields.push(sequence(CLIP, clip));
    fields.push(sequence(FIT, fit));
    let landing = matches!(state.figure.0.pose.action, Action::Landing(_));
    fields.push(sequence(
        RESPONSE,
        outputs
            .movement
            .floor_response
            .map(|response| {
                [
                    i64::from(landing),
                    floor_response(response.kind),
                    i64::from(response.speed.0.0),
                ]
            })
            .into_iter()
            .flatten(),
    ));
    let (mut offers, mut graphics_calls) = (Vec::new(), Vec::new());
    for (index, publication) in (0_i64..).zip(&outputs.publications) {
        match *publication {
            Publication::Contact {
                cue,
                allowance: issued,
            } => offers.extend([
                index,
                i64::from(sender_offer::cue_tag(cue)),
                allowance(issued),
            ]),
            Publication::Graphics(TouchdownGraphics::ClearLandingSlots { figure }) => {
                graphics_calls.extend([index, i64::from(figure == state.figure.0), 1]);
            }
        }
    }
    fields.push(offer(offers));
    fields.push(graphics(graphics_calls));
    Ok(fields)
}

pub(in crate::replay) fn initial(owner: &Production) -> Result<Vec<Observation>, Outcome> {
    let mut fields = state_fields(state(owner)?);
    fields.extend([
        sequence(CLIP, []),
        sequence(FIT, []),
        sequence(RESPONSE, []),
        offer([]),
        graphics([]),
    ]);
    Ok(fields)
}

fn state_fields(state: &State) -> Vec<Observation> {
    let mut fields = body(
        i64::from(state.movement.0.position.x.0),
        i64::from(state.movement.0.position.y.0),
        i64::from(state.movement.0.vertical.speed.0),
    );
    fields.extend(sender_offer::observations(
        &state.contact.0.queue,
        &state.contact.0.progress,
    ));
    fields
}
