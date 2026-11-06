use data::level::{Block, BlockKind, Bts, PlmChoice};

use super::*;
use crate::state::projectile::{Damage, Heading, ProjectileCommon, Velocity};
use crate::state::room::{BlockColumns, Crumble, PlmBody};
use crate::state::samus::Point;

/// The room's width: eight screens.
const WIDTH: u16 = 128;
/// The block the beam reaches in the Morph Ball room's shot: column 76, row 44.
const TARGET: BlockIndex = BlockIndex(44 * WIDTH + 76);

/// Eight screens across and three down of air, the camera on the beam.
fn state() -> State {
    let mut state = State::default();
    state.room.blocks = vec![Block::default(); usize::from(WIDTH) * 48];
    state.room.width = BlockColumns(WIDTH);
    state.room.camera.x.pixels = 0x0400;
    state.room.camera.y.pixels = 0x0200;
    state
}

const fn at(pixels: u16, subpixels: u16) -> Fixed {
    Fixed { pixels, subpixels }
}

/// The uncharged power beam fired down-right, as the oracle has it in slot 0.
fn beam(x: Fixed, y: Fixed, speed: i16, frame_timer: u16) -> ProjectileBody {
    ProjectileBody {
        position: Point { x, y },
        radius: Radius { x: 8, y: 4 },
        velocity: Velocity {
            x: SubpixelSpeed(speed),
            y: SubpixelSpeed(speed),
        },
        heading: Heading {
            direction: Direction::DownRight,
            deleted: false,
        },
        kind: ProjectileType {
            shot: Shot::default(),
            touches_samus: false,
        },
        damage: Damage(0x14),
        frame_timer: Frames(frame_timer),
    }
}

fn fire(state: &mut State, body: ProjectileBody, trail: u16) {
    state.projectiles.count = ShotCount(1);
    if let Some(slot) = state.projectiles.slots.slots.first_mut() {
        *slot = Slot {
            common: ProjectileCommon {
                trail_timer: Frames(trail),
            },
            occupancy: Occupancy::Live(body),
            ..*slot
        };
    }
}

fn slot_zero(state: &State) -> Slot<ProjectileCommon, ProjectileBody> {
    state.projectiles.slots.slots[0]
}

#[test]
fn the_power_beam_speeds_up_by_10h_and_moves_by_its_new_speed() {
    let mut state = state();
    fire(
        &mut state,
        beam(at(0x04a1, 0x5f00), at(0x02b2, 0x5f00), 0x037b, 3),
        3,
    );
    handle(&mut state).unwrap();
    let slot = slot_zero(&state);
    assert_eq!(slot.common.trail_timer, Frames(2));
    assert_eq!(
        slot.occupancy,
        Occupancy::Live(beam(at(0x04a4, 0xea00), at(0x02b5, 0xea00), 0x038b, 2))
    );
}

#[test]
fn the_trail_timer_and_the_frame_timer_start_again() {
    let mut state = state();
    fire(
        &mut state,
        beam(at(0x04a8, 0x8500), at(0x02b9, 0x8500), 0x039b, 1),
        1,
    );
    handle(&mut state).unwrap();
    let slot = slot_zero(&state);
    assert_eq!(slot.common.trail_timer, Frames(4));
    assert_eq!(
        slot.occupancy,
        Occupancy::Live(beam(at(0x04ac, 0x3000), at(0x02bd, 0x3000), 0x03ab, 0x0f))
    );
}

fn explosion(x: Fixed, y: Fixed) -> ProjectileBody {
    ProjectileBody {
        radius: Radius::default(),
        kind: ProjectileType {
            shot: Shot::BeamExplosion {
                beam: BeamShot::default(),
                step: ExplosionStep(1),
            },
            touches_samus: false,
        },
        damage: Damage(EXPLOSION_DAMAGE),
        frame_timer: Frames(3),
        ..beam(x, y, 0x03eb, 3)
    }
}

#[test]
fn a_beam_that_reaches_a_solid_block_steps_to_its_edge_and_explodes() {
    let mut state = state();
    state.room.blocks[usize::from(TARGET.0)].kind = BlockKind::Solid(Bts(0));
    fire(
        &mut state,
        beam(at(0x04b7, 0x9100), at(0x02c8, 0x9100), 0x03db, 0x0c),
        1,
    );
    handle(&mut state).unwrap();
    // It moved across, collided, and so didn't move down.
    assert_eq!(
        slot_zero(&state).occupancy,
        Occupancy::Live(explosion(at(0x04c3, 0x7c00), at(0x02cc, 0x9100)))
    );
}

#[test]
fn a_beam_that_reaches_a_shot_block_spawns_its_crumble() {
    let mut state = state();
    state.room.blocks[usize::from(TARGET.0)].kind = BlockKind::Shootable(PlmChoice::Common(4));
    fire(
        &mut state,
        beam(at(0x04b7, 0x9100), at(0x02c8, 0x9100), 0x03db, 0x0c),
        1,
    );
    handle(&mut state).unwrap();
    assert!(matches!(
        slot_zero(&state).occupancy,
        Occupancy::Live(ProjectileBody {
            kind: ProjectileType {
                shot: Shot::BeamExplosion { .. },
                ..
            },
            ..
        })
    ));
    assert!(
        state
            .room
            .plms
            .slots
            .iter()
            .any(|slot| slot.common.block == TARGET
                && slot.occupancy
                    == Occupancy::Live(PlmBody::ShotBlock {
                        crumble: Crumble::Spawned
                    }))
    );
    assert_eq!(
        state.room.blocks[usize::from(TARGET.0)].kind,
        BlockKind::Solid(Bts(4))
    );
}

#[test]
fn the_explosion_shows_six_frames_of_three_then_deletes_itself() {
    let mut state = state();
    fire(&mut state, explosion(at(0x04c3, 0), at(0x02cc, 0)), 4);
    for _ in 0..5 * 3 {
        handle(&mut state).unwrap();
    }
    assert!(matches!(
        slot_zero(&state).occupancy,
        Occupancy::Live(ProjectileBody {
            kind: ProjectileType {
                shot: Shot::BeamExplosion {
                    step: ExplosionStep(6),
                    ..
                },
                ..
            },
            frame_timer: Frames(3),
            ..
        })
    ));
    for _ in 0..3 {
        handle(&mut state).unwrap();
    }
    let slot = slot_zero(&state);
    assert_eq!(slot.occupancy, Occupancy::Free);
    // The clear keeps the trail timer.
    assert_eq!(slot.common.trail_timer, Frames(4));
    assert_eq!(state.projectiles.count, ShotCount(0));
}

#[test]
fn a_beam_past_the_camera_is_cleared_and_the_count_stops_at_zero() {
    let mut state = state();
    state.room.camera.x.pixels = 0;
    fire(
        &mut state,
        beam(at(0x04a1, 0x5f00), at(0x02b2, 0x5f00), 0x037b, 3),
        3,
    );
    state.projectiles.count = ShotCount(0);
    handle(&mut state).unwrap();
    assert_eq!(slot_zero(&state).occupancy, Occupancy::Free);
    assert_eq!(state.projectiles.count, ShotCount(0));
}

#[test]
fn a_deleted_beam_is_cleared_without_moving() {
    let mut state = state();
    let mut body = beam(at(0x04a1, 0), at(0x02b2, 0), 0x037b, 3);
    body.heading.deleted = true;
    fire(&mut state, body, 3);
    handle(&mut state).unwrap();
    assert_eq!(slot_zero(&state).occupancy, Occupancy::Free);
    assert_eq!(slot_zero(&state).common.trail_timer, Frames(3));
    assert_eq!(state.projectiles.count, ShotCount(0));
}

#[test]
fn the_cooldown_counts_down_to_zero_and_a_negative_one_is_zero() {
    let mut projectiles = Projectiles {
        cooldown: Frames(2),
        ..Projectiles::default()
    };
    cool_down(&mut projectiles);
    assert_eq!(projectiles.cooldown, Frames(1));
    cool_down(&mut projectiles);
    cool_down(&mut projectiles);
    assert_eq!(projectiles.cooldown, Frames(0));
    projectiles.cooldown = Frames(0x8001);
    cool_down(&mut projectiles);
    assert_eq!(projectiles.cooldown, Frames(0));
}

#[test]
fn facing_forward_holds_the_cooldown() {
    let mut state = state();
    state.projectiles.cooldown = Frames(5);
    current_state(&mut state).unwrap();
    assert_eq!(state.projectiles.cooldown, Frames(5));
    state.samus.pose = data::pose::PoseId(1);
    current_state(&mut state).unwrap();
    assert_eq!(state.projectiles.cooldown, Frames(4));
}

#[test]
fn a_wide_span_reaches_no_block() {
    assert_eq!(span(0x0108, 8), 0);
    assert_eq!(span(0x0104, 8), 1);
    assert!(!below(0x10, WIDEST_SPAN));
}
