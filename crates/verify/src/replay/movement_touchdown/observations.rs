//! Projections for her landing body, her downward travel and position after her moves, the
//! accepted floor response and the ordered touchdown consumers.

use audio_transport::{CueId, QueueAllowance, SenderProgress, SoundGroup};
use game::production::{
    Production,
    appearance::TouchdownGraphics,
    movement_touchdown::{Outputs, Publication, State},
};
use game::state::owned::OwnedState;
use serde::Deserialize;
use std::collections::VecDeque;
use types::{Action, Direction, FloorResponseKind};

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
    movement_floor_clipped_down_fp16: i64,
    body_before_fit_x_fp16: i64,
    body_before_fit_y_fp16: i64,
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
            || fifo.service_master_ticks != self.contact_service_master_ticks
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
        fields.push(sequence(CLIP, [self.movement_floor_clipped_down_fp16]));
        fields.push(sequence(
            FIT,
            [self.body_before_fit_x_fp16, self.body_before_fit_y_fp16],
        ));
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
    let movement = &outputs.movement;
    fields.push(sequence(
        CLIP,
        [i64::from(movement.travelled[Direction::Down].0.0)],
    ));
    fields.push(sequence(
        FIT,
        [
            i64::from(movement.position_after_moves.x.0),
            i64::from(movement.position_after_moves.y.0),
        ],
    ));
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
    let contact = &state.audio.0.senders[SoundGroup::Contact];
    fields.extend(sender_offer::observations(
        contact.queue.cues(),
        &contact.progress,
    ));
    fields
}
