use super::*;
use crate::state::room::{CameraDistance, Layer2Scrolls, Scrollers};
use crate::state::samus::Point;

const fn at(pixels: u16, subpixels: u16) -> Fixed {
    Fixed { pixels, subpixels }
}

/// Samus at `now`, having been at `previous`, facing right and standing.
fn samus(previous: Point, now: Point) -> Samus {
    Samus {
        position: now,
        previous_position: previous,
        facing: Facing::Right,
        ..Samus::default()
    }
}

fn point(x: Fixed, y: Fixed) -> Point {
    Point { x, y }
}

/// The camera at the room's top left with an up scroller of $70.
fn camera() -> Camera {
    Camera {
        scrollers: Scrollers {
            up: 0x70,
            down: 0xA0,
        },
        ..Camera::default()
    }
}

/// Two screens across, one down.
fn across(areas: &[ScrollArea; 2]) -> ScrollAreas<'_> {
    ScrollAreas {
        areas,
        width: 2,
        height: 1,
        stale: &[],
    }
}

/// One screen across, two down.
fn down(areas: &[ScrollArea; 2]) -> ScrollAreas<'_> {
    ScrollAreas {
        areas,
        width: 1,
        height: 2,
        stale: &[],
    }
}

#[test]
fn the_camera_follows_samus_right_at_her_speed_plus_a_pixel() {
    // She runs 1.5 px; the ideal edge, $60 to her left, is ahead of the camera, which moves
    // her distance plus one: 2.5 px.
    let mut camera = camera();
    let mut samus = samus(
        point(at(0x90, 0), at(0x80, 0)),
        point(at(0x91, 0x8000), at(0x80, 0)),
    );
    follow(
        &mut camera,
        &mut samus,
        across(&[ScrollArea::Blue, ScrollArea::Blue]),
    );
    assert_eq!(camera.x, at(2, 0x8000));
    assert_eq!(camera.y, at(0, 0));
    assert_eq!(samus.previous_position, samus.position);
}

#[test]
fn a_red_screen_on_the_right_stops_the_camera_at_its_edge() {
    // The same run of 2 px moves the camera 3 px into screen 0, but screen 1 is red: the
    // camera goes back by its speed and 2 more, past screen 0's left edge, so it stays there.
    let mut camera = camera();
    let mut samus = samus(
        point(at(0x90, 0), at(0x80, 0)),
        point(at(0x92, 0), at(0x80, 0)),
    );
    follow(
        &mut camera,
        &mut samus,
        across(&[ScrollArea::Blue, ScrollArea::Red]),
    );
    assert_eq!(camera.x.pixels, 0);
}

#[test]
fn the_camera_stops_at_the_ideal_position_and_drops_its_subpixels() {
    // The camera is 1 px short of the ideal edge and Samus moved 3.25 px: it stops on the
    // ideal edge with no fraction.
    let mut camera = Camera {
        x: at(0x31, 0x4000),
        ..camera()
    };
    let mut samus = samus(
        point(at(0x8F, 0xC000), at(0x80, 0)),
        point(at(0x93, 0), at(0x80, 0)),
    );
    follow(
        &mut camera,
        &mut samus,
        across(&[ScrollArea::Blue, ScrollArea::Blue]),
    );
    assert_eq!(camera.x, at(0x33, 0));
}

#[test]
fn facing_left_the_camera_aims_further_ahead() {
    // Facing left at $1C0, the ideal edge is $A0 to her left: $120. The camera at $130 moves
    // left by her 4 px plus one.
    let mut camera = Camera {
        x: at(0x130, 0),
        ..camera()
    };
    let mut samus = Samus {
        facing: Facing::Left,
        ..samus(
            point(at(0x1C4, 0), at(0x80, 0)),
            point(at(0x1C0, 0), at(0x80, 0)),
        )
    };
    follow(
        &mut camera,
        &mut samus,
        across(&[ScrollArea::Blue, ScrollArea::Blue]),
    );
    assert_eq!(camera.x.pixels, 0x12B);
}

#[test]
fn the_camera_follows_samus_down() {
    // She falls 4 px to $84; the ideal top edge, the up scroller above her, is $14. The
    // camera moves down her 4 px plus one, and the blue screen below lets it.
    let mut camera = camera();
    let mut samus = samus(
        point(at(0x80, 0), at(0x80, 0)),
        point(at(0x80, 0), at(0x84, 0)),
    );
    follow(
        &mut camera,
        &mut samus,
        down(&[ScrollArea::Blue, ScrollArea::Blue]),
    );
    assert_eq!(camera.y, at(5, 0));
    assert_eq!(camera.x, at(0, 0));
}

#[test]
fn a_red_screen_below_stops_the_camera_going_down() {
    let mut upper = camera();
    let mut start = samus(
        point(at(0x80, 0), at(0x80, 0)),
        point(at(0x80, 0), at(0x84, 0)),
    );
    follow(
        &mut upper,
        &mut start,
        down(&[ScrollArea::Green, ScrollArea::Red]),
    );
    // Screen 0 isn't blue, so the camera may go $1F past its top; it went 5.
    assert_eq!(upper.y.pixels, 5);
    let mut lower = Camera {
        y: at(0x1E, 0),
        ..camera()
    };
    let mut falling = samus(
        point(at(0x80, 0), at(0xA0, 0)),
        point(at(0x80, 0), at(0xA4, 0)),
    );
    follow(
        &mut lower,
        &mut falling,
        down(&[ScrollArea::Green, ScrollArea::Red]),
    );
    // From $1E it reaches $23, past $1F with a red screen below: it goes back by its speed
    // and 2, which would take it above $1F, so it stops there.
    assert_eq!(lower.y.pixels, 0x1F);
}

#[test]
fn a_still_camera_in_a_red_screen_leaves_it_three_pixels_a_frame() {
    // Samus hasn't moved: the camera's speed is the one pixel, and it moves 3 px toward the
    // open screen on the right.
    let mut camera = Camera {
        x: at(0x10, 0),
        ..camera()
    };
    let still = point(at(0x80, 0), at(0x80, 0));
    let mut samus = samus(still, still);
    follow(
        &mut camera,
        &mut samus,
        across(&[ScrollArea::Red, ScrollArea::Blue]),
    );
    assert_eq!(camera.x.pixels, 0x13);
}

#[test]
fn a_still_camera_keeps_inside_the_room() {
    let mut camera = Camera {
        x: at(0x180, 0),
        y: at(0xFFF0, 0),
        ..camera()
    };
    let still = point(at(0x80, 0), at(0x80, 0));
    let mut samus = samus(still, still);
    let areas = [ScrollArea::Blue; 4];
    let areas = ScrollAreas {
        areas: &areas,
        width: 2,
        height: 2,
        stale: &[],
    };
    follow(&mut camera, &mut samus, areas);
    assert_eq!((camera.x.pixels, camera.y.pixels), (0x100, 0));
}

#[test]
fn the_bg_scrolls_follow_the_camera_and_layer_2_its_rates() {
    let mut camera = Camera {
        x: at(0x123, 0x4000),
        y: at(0x40, 0),
        bg1_offset: Scroll { x: 0, y: 0x100 },
        bg2_offset: Scroll { x: 4, y: 0 },
        layer2_scroll: Layer2Scrolls {
            x: Layer2Scroll::from(0xC1),
            y: Layer2Scroll::Fixed,
        },
        layer2: Scroll { x: 0, y: 0x77 },
        ..camera()
    };
    let mut mirrors = Registers::default();
    mirrors.scroll[1].y = 0x55;
    scroll_backgrounds(&mut camera, &mut mirrors);
    assert_eq!(mirrors.scroll[0], Scroll { x: 0x123, y: 0x140 });
    // $123 times $C0 / $100 is $DA.25: layer 2 is at $DA, BG2 4 px on.
    assert_eq!(camera.layer2, Scroll { x: 0xDA, y: 0x77 });
    assert_eq!(mirrors.scroll[1], Scroll { x: 0xDE, y: 0x55 });
}

#[test]
fn the_scroll_settings_go_back_out_byte_exact() {
    for byte in 0..=u8::MAX {
        assert_eq!(u8::from(Layer2Scroll::from(byte)), byte);
    }
    for distance in CameraDistance::ALL {
        assert_eq!(CameraDistance::try_from(u16::from(distance)), Ok(distance));
    }
    assert_eq!(CameraDistance::try_from(1), Err(1));
}

#[test]
fn scrolling_down_reads_the_scroll_buffer_past_the_room() {
    // area-brinstar's frame 414: the camera moves into the room's blue bottom row ahead of
    // the ideal position, which pulls it back above. Scrolling down (`$80:A8FC`) then reads
    // the screen below the one it moved into, past the room's table. The buffer holds $80
    // there, not red, so the camera stays at the ideal position.
    let start = Camera {
        y: at(0xFD, 0),
        ..camera()
    };
    let samus_at = |stale: &[StaleScroll]| {
        let mut camera = start;
        let mut samus = samus(
            point(at(0x80, 0), at(0x16D, 0)),
            point(at(0x80, 0), at(0x16F, 0)),
        );
        let areas = [ScrollArea::Green, ScrollArea::Blue];
        let areas = ScrollAreas {
            stale,
            ..down(&areas)
        };
        follow(&mut camera, &mut samus, areas);
        camera.y.pixels
    };
    assert_eq!(samus_at(&[StaleScroll::from(0x80)]), 0xFF);
    // A red byte there backs the camera out, as one past the buffer does.
    assert!(samus_at(&[StaleScroll::from(0)]) < 0xFF);
    assert!(samus_at(&[]) < 0xFF);
}

#[test]
fn a_stale_scroll_byte_goes_back_out_byte_exact() {
    for byte in 0..=u8::MAX {
        assert_eq!(u8::from(StaleScroll::from(byte)), byte);
    }
}
