use super::*;
use crate::state::room::{CameraDistance, DrawnBlocks, Layer2Scrolls};
use crate::state::samus::Fixed;

const fn at(pixels: u16, subpixels: u16) -> Fixed {
    Fixed { pixels, subpixels }
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
        ..Camera::default()
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

const fn point(x: u16, y: u16) -> BlockPoint {
    BlockPoint { x, y }
}

/// Layer 1 at `x`, `y` and BG1 with it, both layers last exposed from `exposed`, layer 2 fixed.
fn moved(x: u16, y: u16, exposed: BlockPoint) -> (Camera, Registers) {
    let camera = Camera {
        x: at(x, 0),
        y: at(y, 0),
        drawn: DrawnBlocks {
            layer1: exposed,
            layer2: exposed,
        },
        layer2_scroll: Layer2Scrolls {
            x: Layer2Scroll::Fixed,
            y: Layer2Scroll::Fixed,
        },
        ..Camera::default()
    };
    let mut mirrors = Registers::default();
    mirrors.scroll[0] = Scroll { x, y };
    (camera, mirrors)
}

/// Layer 1's line `line` from block (`x`, `y`), on BG1 at the same block.
const fn layer1(line: BlockLine, x: u16, y: u16) -> LayerExposure {
    LayerExposure {
        layer: LevelLayer::One,
        line,
        level: point(x, y),
        background: point(x, y),
    }
}

#[test]
fn crossing_a_block_column_rightward_exposes_the_column_coming_into_view() {
    let (mut camera, mirrors) = moved(0x10, 0, point(0, 0));
    assert_eq!(
        exposures(&mut camera, &mirrors),
        [layer1(BlockLine::Column, 17, 0)]
    );
    assert_eq!(camera.drawn.layer1, point(1, 0));
}

#[test]
fn crossing_a_block_column_leftward_exposes_the_left_edge() {
    let (mut camera, mirrors) = moved(0x4F, 0x30, point(5, 3));
    assert_eq!(
        exposures(&mut camera, &mirrors),
        [layer1(BlockLine::Column, 4, 3)]
    );
    assert_eq!(camera.drawn.layer1, point(4, 3));
}

#[test]
fn moving_down_a_block_exposes_the_row_below_the_view_and_up_the_one_below_the_top() {
    let (mut camera, mirrors) = moved(0x50, 0x10, point(5, 0));
    assert_eq!(
        exposures(&mut camera, &mirrors),
        [layer1(BlockLine::Row, 5, 16)]
    );
    assert_eq!(camera.drawn.layer1, point(5, 1));
    let (mut camera, mirrors) = moved(0, 0x10, point(0, 2));
    assert_eq!(
        exposures(&mut camera, &mirrors),
        [layer1(BlockLine::Row, 0, 2)]
    );
}

#[test]
fn a_layer_left_of_the_room_is_before_block_0() {
    // Layer 1 at -16 px is at block FFFFh, which is before block 0: the left edge comes in.
    // BG1's scroll is unsigned: its block is 0FFFh.
    let (mut camera, mirrors) = moved(0xFFF0, 0, point(0, 0));
    assert_eq!(
        exposures(&mut camera, &mirrors),
        [LayerExposure {
            level: point(0xFFFF, 0),
            background: point(0x0FFF, 0),
            ..layer1(BlockLine::Column, 0, 0)
        }]
    );
}

#[test]
fn a_line_lands_where_the_background_scrolled_not_where_the_layer_is() {
    let (mut camera, mut mirrors) = moved(0x10, 0, point(0, 0));
    mirrors.scroll[0].y = 0x100;
    assert_eq!(
        exposures(&mut camera, &mirrors),
        [LayerExposure {
            background: point(17, 16),
            ..layer1(BlockLine::Column, 17, 0)
        }]
    );
}

#[test]
fn a_layer_still_in_its_blocks_exposes_nothing() {
    let (mut camera, mirrors) = moved(0x1F, 0x2F, point(1, 2));
    assert_eq!(exposures(&mut camera, &mirrors), []);
    assert_eq!(camera.drawn.layer1, point(1, 2));
}

#[test]
fn layer_2_exposes_lines_only_on_an_axis_where_bg2_shows_its_level() {
    let (mut camera, mut mirrors) = moved(0x10, 0x10, point(0, 0));
    camera.layer2 = Scroll { x: 0x10, y: 0x10 };
    camera.layer2_scroll = Layer2Scrolls {
        x: Layer2Scroll::WithLayer1,
        y: Layer2Scroll::Scaled {
            rate: 0x80,
            custom: false,
        },
    };
    mirrors.scroll[1] = Scroll { x: 0x10, y: 0x10 };
    let layer2 = LayerExposure {
        layer: LevelLayer::Two,
        ..layer1(BlockLine::Column, 17, 1)
    };
    assert_eq!(
        exposures(&mut camera, &mirrors),
        [
            layer1(BlockLine::Column, 17, 1),
            layer1(BlockLine::Row, 1, 16),
            layer2
        ]
    );
    assert_eq!(camera.drawn.layer2, point(1, 0));
}
