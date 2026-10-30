//! Candidate dispatcher and observation adapters; no predecessor gameplay rules.
#![forbid(unsafe_code)]
use crate::{actor_movement_cases::Operation, game_state::Game};
use {
    crate::game::actors::{
        api::{MovementResult, SlopePolicy, VerticalProbeEdge},
        contacts, movement, queries,
    },
    sm_game::actors::access::{AccessMode, ActorAccess, ActorKey},
};
use {
    crate::game::session_runtime::Runtime,
    sm_game::session::{Point, Vector32},
};

fn result(value: MovementResult) -> u64 {
    match value {
        MovementResult::Clear => 0,
        MovementResult::Blocked => 1,
    }
}
fn flag(value: bool) -> u64 {
    if value { 0xffff } else { 0 }
}
pub(crate) fn run(op: Operation, game: &mut Game) -> Option<u64> {
    let mut h = Runtime::new(game);
    match op {
        Operation::Vector { phase, value } => {
            movement::apply_vector(
                &mut h,
                phase,
                Vector32 {
                    x: value.x,
                    y: value.y,
                },
            );
            None
        }
        Operation::ProbeX { k, amount } => Some(result(movement::probe_horizontal(
            &mut h,
            ActorKey::from_legacy(k),
            amount,
        ))),
        Operation::ProbeY { k, amount } => Some(result(movement::probe_vertical(
            &mut h,
            ActorKey::from_legacy(k),
            amount,
        ))),
        Operation::ProbeOffset { k, edge, amount } => {
            Some(result(movement::probe_vertical_offset(
                &mut h,
                ActorKey::from_legacy(k),
                if edge & 1 != 0 {
                    VerticalProbeEdge::Positive
                } else {
                    VerticalProbeEdge::Negative
                },
                amount,
            )))
        }
        Operation::Right { k, amount, policy } => Some(result(movement::move_horizontal(
            &mut h,
            ActorKey::from_legacy(k),
            amount,
            match policy {
                0 => SlopePolicy::Ignore,
                1 => SlopePolicy::Solid,
                2 => SlopePolicy::Follow,
                _ => unreachable!(),
            },
        ))),
        Operation::Down { k, amount } => Some(result(movement::move_vertical(
            &mut h,
            ActorKey::from_legacy(k),
            amount,
        ))),
        Operation::Solid(amount) => {
            let contact =
                crate::game::player_collision::verification::solid_contact(&mut h, amount);
            Some(
                (u64::from(contact.displacement as u32) << 16)
                    | if matches!(
                        contact.obstruction,
                        crate::game::player_collision::operations::Obstruction::SolidActor(_)
                    ) {
                        0xffff
                    } else {
                        0
                    },
            )
        }
        Operation::Touch(k) => Some(flag(queries::touches_player_from_above(
            &h,
            ActorKey::from_legacy(k),
        ))),
        Operation::Padding(k) => Some(flag(queries::overlaps_player_with_padding(
            &h,
            ActorKey::from_legacy(k),
        ))),
        Operation::Distance(a, b) => {
            let r = queries::distance_and_direction(
                &mut h,
                Point { x: a.x, y: a.y },
                Point { x: b.x, y: b.y },
            );
            Some(u64::from(r.distance) | (u64::from(r.direction) << 16))
        }
        Operation::Body => {
            contacts::movement_verification_body(&mut h);
            None
        }
        Operation::Contacts => {
            contacts::process_contacts(&mut h);
            None
        }
    }
}
pub(super) fn vector_store(h: &impl ActorAccess, key: ActorKey, y: bool) {
    if crate::actor_movement_observer::active_case().is_none() {
        return;
    }
    let actor = h.resolve_actor(key, AccessMode::LittleEndian);
    let coordinate = if y { actor.y } else { actor.x };
    let [a, b] = coordinate.whole.to_le_bytes();
    let [c, d] = coordinate.fraction.to_le_bytes();
    crate::actor_movement_observer::vector_store(&[a, b, c, d]);
}
pub(super) fn body_stage(h: &mut impl ActorAccess) {
    if crate::actor_movement_observer::active_case().is_none() {
        return;
    }
    let stage = h.session().room.enemy_processing_stage;
    let map = h
        .resolve_actor(ActorKey::from_legacy(0xe3ea), AccessMode::LittleEndian)
        .spritemap;
    if crate::actor_movement_observer::check_body_stage(stage, map) {
        h.actor_movement_observe(6);
    }
}
