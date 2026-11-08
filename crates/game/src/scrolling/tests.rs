use super::*;
use crate::state::room::{CameraDistance, Layer2Scrolls, StaleScroll};
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

#[test]
fn a_stale_scroll_byte_goes_back_out_byte_exact() {
    for byte in 0..=u8::MAX {
        assert_eq!(u8::from(StaleScroll::from(byte)), byte);
    }
}
