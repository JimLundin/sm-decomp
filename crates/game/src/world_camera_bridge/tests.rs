use super::*;
use crate::state::room::{Camera, CameraDistance, Scrollers};
use crate::state::samus::{Fixed as Word, Motion};

fn follow_fixture(camera: &mut Camera, samus: &mut Samus, room: &Room) {
    let mut model = restore(
        camera,
        samus.previous_position,
        types::VerticalFraction::ZERO,
    );
    follow(&mut model, camera, samus, room);
}

const fn at(pixels: u16, subpixels: u16) -> Word {
    Word { pixels, subpixels }
}

const fn spot(x: Word, y: Word) -> Point {
    Point { x, y }
}

/// Samus at `now`, having been at `previous`, facing right and standing.
fn samus(previous: Point, now: Point) -> Samus {
    Samus {
        previous_position: previous,
        facing: Facing::Right,
        motion: Motion {
            position: now.into(),
            ..Motion::default()
        },
        ..Samus::default()
    }
}

const SCROLLERS: Scrollers = Scrollers {
    up: 0x70,
    down: 0xA0,
};

/// The camera at the room's top left.
fn camera() -> Camera {
    Camera {
        scrollers: SCROLLERS,
        ..Camera::default()
    }
}

/// The room of `areas`, `width` screens across and as many rows down as they fill, with
/// `below` after it.
fn of(areas: &[ScrollArea], below: &[ScrollArea], width: usize) -> Room {
    let size = Pair {
        x: width,
        y: areas.len().checked_div(width).unwrap_or_default(),
    };
    match room(areas, below, size, SCROLLERS) {
        Ok(room) => room,
        Err(error) => panic!("{error:?}"),
    }
}

#[test]
fn the_room_runs_on_into_the_scroll_buffer() {
    let areas = [
        ScrollArea::Green,
        ScrollArea::Blue,
        ScrollArea::Red,
        ScrollArea::Green,
    ];
    let below = [0x01, 0x80, 0x00].map(ScrollArea::from);
    let (open, short, closed) = (ScreenKind::Open, ScreenKind::Short, ScreenKind::Closed);
    // A byte that is no area reads as open.
    assert_eq!(
        run(&areas, &below),
        [open, short, closed, open, short, open, closed]
    );
    let Heights {
        while_not_rising,
        while_rising,
    } = heights(SCROLLERS);
    assert_eq!(
        (while_not_rising, while_rising),
        (Wrapping(0x70), Wrapping(0xA0))
    );
}

#[test]
fn a_room_of_no_screens_or_short_of_areas_is_no_room() {
    let areas = [ScrollArea::Green; 3];
    let size = |x, y| Pair { x, y };
    for empty in [size(0, 1), size(3, 0)] {
        assert_eq!(
            room(&areas, &[], empty, SCROLLERS).err(),
            Some(RoomError::NoScreens),
            "{empty:?}"
        );
    }
    assert_eq!(
        room(&areas, &[], size(2, 2), SCROLLERS).err(),
        Some(RoomError::MissingScreens)
    );
    assert!(room(&areas, &[], size(1, 3), SCROLLERS).is_ok());
}

#[test]
fn every_camera_distance_is_a_lead_setting() {
    let leads = CameraDistance::ALL.map(lead);
    assert_eq!(
        leads,
        [
            LeadSetting::Ahead,
            LeadSetting::Near,
            LeadSetting::KeepLeft,
            LeadSetting::KeepRight
        ]
    );
}

#[test]
fn the_camera_follows_samus_right_at_her_speed_plus_a_pixel() {
    // She runs 1.5 px; the aim, 96 px to her left, is ahead of the camera, which moves her
    // distance plus one: 2.5 px. It then remembers where it saw her.
    let mut camera = camera();
    let mut samus = samus(
        spot(at(0x90, 0), at(0x80, 0)),
        spot(at(0x91, 0x8000), at(0x80, 0)),
    );
    follow_fixture(&mut camera, &mut samus, &of(&[ScrollArea::Blue; 2], &[], 2));
    assert_eq!(camera.x, at(2, 0x8000));
    assert_eq!(camera.y, at(0, 0));
    assert_eq!(samus.previous_position, samus.motion.position.into());
}

#[test]
fn a_red_screen_on_the_right_stops_the_camera_at_its_edge() {
    let mut camera = camera();
    let mut samus = samus(
        spot(at(0x90, 0), at(0x80, 0)),
        spot(at(0x92, 0), at(0x80, 0)),
    );
    let room = of(&[ScrollArea::Blue, ScrollArea::Red], &[], 2);
    follow_fixture(&mut camera, &mut samus, &room);
    assert_eq!(camera.x.pixels, 0);
}

#[test]
fn facing_left_or_skidding_turns_the_lead() {
    // Facing left at 1C0h, the aim is 160 px to her left: 120h. The camera at 130h moves left
    // by her 4 px plus one. Skidding, she faces right but the view looks left the same way.
    let start = Camera {
        x: at(0x130, 0),
        ..camera()
    };
    let moving = samus(
        spot(at(0x1C4, 0), at(0x80, 0)),
        spot(at(0x1C0, 0), at(0x80, 0)),
    );
    let room = of(&[ScrollArea::Blue; 2], &[], 2);
    for her in [
        Samus {
            facing: Facing::Left,
            ..moving
        },
        Samus {
            motion: Motion {
                mode: SpeedMode::TurningAround,
                ..moving.motion
            },
            ..moving
        },
    ] {
        let (mut camera, mut her) = (start, her);
        follow_fixture(&mut camera, &mut her, &room);
        assert_eq!(camera.x.pixels, 0x12B);
    }
}

#[test]
fn rising_the_camera_aims_lower_above_her() {
    // She rises 4 px to 100h: the aim is 160 px above her, 60h, not 112 px, 90h. The camera at
    // 70h moves up 5 px towards it.
    let start = Camera {
        y: at(0x70, 0),
        ..camera()
    };
    let mut rising = samus(
        spot(at(0x80, 0), at(0x104, 0)),
        spot(at(0x80, 0), at(0x100, 0)),
    );
    rising.motion.vertical = VerticalMotion::Up;
    let room = of(&[ScrollArea::Green; 2], &[], 1);
    let mut camera = start;
    follow_fixture(&mut camera, &mut rising, &room);
    assert_eq!(camera.y.pixels, 0x6B);
}

#[test]
fn a_still_camera_in_a_red_screen_leaves_it_three_pixels_a_frame() {
    let mut camera = Camera {
        x: at(0x10, 0),
        ..camera()
    };
    let still = spot(at(0x80, 0), at(0x80, 0));
    let mut samus = samus(still, still);
    follow_fixture(
        &mut camera,
        &mut samus,
        &of(&[ScrollArea::Red, ScrollArea::Blue], &[], 2),
    );
    assert_eq!(camera.x.pixels, 0x13);
}

#[test]
fn scrolling_down_reads_the_scroll_buffer_past_the_room() {
    // area-brinstar's frame 414: the camera moves into the room's blue bottom row ahead of the
    // aim, which pulls it back above. The move down then reads the screen below the one it moved
    // into, past the room's table. The buffer holds 80h there, not red, so the camera stays on
    // the aim.
    let start = Camera {
        y: at(0xFD, 0),
        ..camera()
    };
    let samus_at = |below: &[ScrollArea]| {
        let mut camera = start;
        let mut samus = samus(
            spot(at(0x80, 0), at(0x16D, 0)),
            spot(at(0x80, 0), at(0x16F, 0)),
        );
        let room = of(&[ScrollArea::Green, ScrollArea::Blue], below, 1);
        follow_fixture(&mut camera, &mut samus, &room);
        camera.y.pixels
    };
    assert_eq!(samus_at(&[ScrollArea::Unknown(0x80)]), 0xFF);
    // A red byte there backs the camera out, as one past the buffer does.
    assert!(samus_at(&[ScrollArea::Red]) < 0xFF);
    assert!(samus_at(&[]) < 0xFF);
}

#[test]
fn with_no_room_loaded_the_camera_stays_and_too_few_areas_are_a_fault() {
    let mut state = crate::state::State::default();
    let before = state.room.camera;
    assert_eq!(
        scroll(&mut state.camera.0, &mut state.room, &mut state.samus),
        Ok(())
    );
    assert_eq!(state.room.camera, before);
    state.room.scroll_areas = vec![ScrollArea::Green; 3];
    state.room.width = crate::state::room::BlockColumns(4 * SCREEN_BLOCKS);
    state.room.height = crate::state::room::BlockRows(SCREEN_BLOCKS);
    assert_eq!(
        scroll(&mut state.camera.0, &mut state.room, &mut state.samus),
        Err(RoomError::MissingScreens)
    );
}

#[test]
fn skipped_follow_keeps_nonzero_receipt_and_placement_keeps_remembered_fraction() {
    let mut state = crate::state::State::default();
    state.camera.0 = restore(
        &state.room.camera,
        state.samus.previous_position,
        types::VerticalFraction(0xBEEF),
    );
    let saved = state.camera.clone();
    scroll(&mut state.camera.0, &mut state.room, &mut state.samus).unwrap();
    assert_eq!(state.camera, saved);

    state.samus.previous_position.y = at(100, 0x9000);
    // Movement's placement writes only the remembered pixel, before the scroll.
    state.samus.previous_position.y.pixels = 98;
    remember(&mut state.camera.0, state.samus.previous_position);
    assert_eq!(state.camera.0.seen().y, Wrapping(0x0062_9000));
    assert_eq!(
        state.camera.0.vertical_fraction(),
        types::VerticalFraction(0xBEEF)
    );
    let saved = canonical::to_bytes(&state);
    state = canonical::from_bytes(&saved).unwrap();
    state.samus.motion.position.y = Exact(0x0063_3000);
    let room = of(&[ScrollArea::Blue], &[], 1);
    follow(
        &mut state.camera.0,
        &mut state.room.camera,
        &mut state.samus,
        &room,
    );
    assert_eq!(
        state.camera.0.vertical_fraction(),
        types::VerticalFraction(0xA000)
    );
    assert_eq!(state.samus.previous_position.y, at(99, 0x3000));
}
