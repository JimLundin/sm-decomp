use super::*;
use crate::state::State;
use crate::state::video::{OamEntry, SpriteCount};
use data::pose::{PoseId, Poses};

/// A pose definition: facing, movement type, Y radius.
fn definition(facing: u8, movement: u8, y_radius: u8) -> [u8; 8] {
    [facing, movement, 0xFF, 0xFF, 0, 0, y_radius, 0]
}

/// Pose 0 stands facing right; pose 1 wall-jumps facing right, pose 2 facing left, both
/// 12 pixels from her middle to her feet.
fn content() -> Content {
    let poses = [
        definition(8, 0x00, 0x15),
        definition(8, 0x14, 12),
        definition(4, 0x14, 12),
    ]
    .concat();
    Content {
        poses: Poses::decode(&poses).unwrap(),
        ..Content::default()
    }
}

/// Samus at (0x150, 0x90) in pose `pose` after a change from pose 0, the camera at
/// (0x100, 0x20).
fn after_change(pose: u16) -> (State, PoseRecord) {
    let content = content();
    let mut state = State::default();
    let before = state.samus.previous;
    state.samus.pose = PoseId(pose);
    state.samus.facing = content.poses.get(PoseId(pose)).unwrap().facing;
    state.samus.previous.pose = PoseId(pose);
    state.samus.position.x.pixels = 0x150;
    state.samus.position.y.pixels = 0x90;
    state.room.camera.x.pixels = 0x100;
    state.room.camera.y.pixels = 0x20;
    (state, before)
}

fn dust(timer: u16, x: u16, y: u16) -> AtmosphericGraphic {
    AtmosphericGraphic {
        kind: AtmosphericKind::LandingDust,
        frame: AtmosphericFrame(0),
        timer: AtmosphericTimer(timer),
        x,
        y,
    }
}

fn drawn(state: &State) -> &[OamEntry] {
    &state.video.batch.oam.low[..usize::from(state.video.batch.sprites.0)]
}

#[test]
fn a_wall_jump_facing_right_raises_dust_at_her_feet_behind_her() {
    let (mut state, before) = after_change(1);
    initialise_pose(&mut state.atmospherics, before, &state.samus, &content());
    // `$91:FA76`: slot 3, X 6 pixels left of hers, Y her bottom boundary.
    assert_eq!(
        state.atmospherics.slots,
        [None, None, None, Some(dust(3, 0x14A, 0x90 + 12 - 1))]
    );
}

#[test]
fn a_wall_jump_facing_left_raises_it_on_her_right() {
    let (mut state, before) = after_change(2);
    initialise_pose(&mut state.atmospherics, before, &state.samus, &content());
    assert_eq!(
        state.atmospherics.slots[LANDING_SLOT].map(|graphic| graphic.x),
        Some(0x156)
    );
}

#[test]
fn no_pose_change_raises_nothing() {
    let (mut state, _) = after_change(1);
    let unchanged = state.samus.previous;
    initialise_pose(&mut state.atmospherics, unchanged, &state.samus, &content());
    assert_eq!(state.atmospherics, Atmospherics::default());
}

#[test]
fn a_change_to_another_movement_type_raises_nothing() {
    let (mut state, mut before) = after_change(0);
    before.pose = PoseId(1);
    initialise_pose(&mut state.atmospherics, before, &state.samus, &content());
    assert_eq!(state.atmospherics, Atmospherics::default());
}

#[test]
fn the_dust_rises_through_its_four_frames_and_ends() {
    let content = content();
    let (mut state, _) = after_change(1);
    state.atmospherics.slots[LANDING_SLOT] = Some(dust(3, 0x150, 0x90));
    let mut seen = Vec::new();
    for _ in 0..20 {
        state.video.batch.sprites = SpriteCount(0);
        handle(&mut state, &content);
        seen.push(drawn(&state).first().copied());
    }
    // `$90:8B93`: the spawn's 3 frames, then 3, 4 and 5 of frames 1 to 3 (each frame's timer
    // is its predecessor's entry); the last entry's 6 runs out after the last frame.
    let tiles: Vec<Option<u8>> = seen.iter().map(|entry| entry.map(|e| e.tile)).collect();
    let mut expected = vec![Some(0x48); 2];
    expected.extend([Some(0x49); 3]);
    expected.extend([Some(0x4A); 4]);
    expected.extend([Some(0x4B); 5]);
    expected.extend([None; 6]);
    assert_eq!(tiles, expected);
    assert_eq!(state.atmospherics, Atmospherics::default());
    // The corner 4 pixels up and left of it, a pixel higher every frame; palette 5,
    // priority 2.
    assert_eq!(
        seen[0],
        Some(OamEntry {
            x: 0x4C,
            y: 0x90 - 0x20 - 1 - 4,
            tile: 0x48,
            attributes: 0x2A,
        })
    );
    assert_eq!(seen[13].map(|entry| entry.y), Some(0x90 - 0x20 - 14 - 4));
}

#[test]
fn a_graphic_off_screen_animates_without_a_sprite() {
    let content = content();
    let (mut state, _) = after_change(1);
    // Its corner 2 pixels left of the screen.
    state.atmospherics.slots[0] = Some(dust(1, 0x102, 0x90));
    handle(&mut state, &content);
    assert_eq!(state.video.batch.sprites, SpriteCount(0));
    let graphic = state.atmospherics.slots[0].unwrap();
    assert_eq!(
        (graphic.frame, graphic.timer, graphic.y),
        (AtmosphericFrame(1), AtmosphericTimer(3), 0x8F)
    );
}

#[test]
fn a_held_timer_shows_the_graphic_only_once_the_hold_runs_out() {
    let content = content();
    let (mut state, _) = after_change(1);
    state.atmospherics.slots[0] = Some(AtmosphericGraphic {
        kind: AtmosphericKind::SpeedBoostDust,
        ..dust(0x8002, 0x150, 0x90)
    });
    handle(&mut state, &content);
    assert_eq!(state.video.batch.sprites, SpriteCount(0));
    handle(&mut state, &content);
    // `$90:8A70`: at the hold bit alone, the frame's timer starts and it is drawn.
    assert_eq!(state.video.batch.sprites, SpriteCount(1));
    assert_eq!(
        state.atmospherics.slots[0].map(|graphic| graphic.timer),
        Some(AtmosphericTimer(3))
    );
}

#[test]
fn lava_damage_spreads_outward_by_slot_and_draws_last_slot_first() {
    let content = content();
    let (mut state, _) = after_change(1);
    let lava = |x| {
        Some(AtmosphericGraphic {
            kind: AtmosphericKind::LavaSurfaceDamage,
            ..dust(2, x, 0x90)
        })
    };
    state.atmospherics.slots = [lava(0x140), None, lava(0x160), None];
    handle(&mut state, &content);
    let xs: Vec<u8> = drawn(&state).iter().map(|entry| entry.x).collect();
    // Slot 2 first, a pixel left; then slot 0, a pixel right.
    assert_eq!(xs, [0x60 - 1 - 4, 0x40 + 1 - 4]);
}
