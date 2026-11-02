//! Suit and X-ray HDMA callbacks without another Samus lease.
#![forbid(unsafe_code)]
use super::context::Context;
use crate::display::preparation::{Field, Store};

use crate::player_transitions::xray::{BeamPhase, BlendPhase};
use crate::ram::{word_arrays as arrays, words as ram};
use crate::scene::hdma_sources::LiveOwner;

pub fn set_window_word<H: Context>(h: &mut H, index: u16, value: u16) {
    h.store_source_word(
        LiveOwner::HdmaTable1,
        2 * usize::from(index),
        crate::scene::state::WordOrder::Little,
        value,
    );
}
fn clear_window<H: Context>(h: &mut H) {
    for row in (0..256).rev() {
        set_window_word(h, row, 255);
    }
}
pub fn blend_mask<H: Context>(h: &H) -> u16 {
    if h.session().room.liquid.kind == 0x24 {
        0x1000
    } else if matches!(h.session().room_key(), 0xa66a | 0xcefb)
        || matches!(h.current_word(ram::BOSS_ID), 3 | 6 | 7 | 8 | 10)
    {
        0x2000
    } else {
        0x4000
    }
}
pub fn apply_blending<H: Context>(h: &mut H, phase: BlendPhase) {
    let mask = blend_mask(h);
    let colors = match phase {
        BlendPhase::Restore => false,
        BlendPhase::Beam => mask == 0x4000,
        BlendPhase::Setup => {
            mask == 0x4000
                || (mask == 0x1000 && h.display_control_byte(Field::FixedColor0) & 0x1f < 7)
        }
    };
    if colors {
        // Original overlapping native words zero the following byte each time.
        for (first, second, value) in [
            (Field::FixedColor0, Field::FixedColor1, 0x27),
            (Field::FixedColor1, Field::FixedColor2, 0x47),
            (Field::FixedColor2, Field::Geometry, 0x87),
        ] {
            let [low, high] = u16::to_ne_bytes(value);
            h.store_display_control_byte(first, low, Store::Owned);
            h.store_display_control_byte(second, high, Store::Owned);
        }
    }
    let mut fx = h.fx_controls();
    fx.blending |= mask;
    h.publish_fx_controls(fx, &[crate::fx_state::Field::Blending]);
}
fn publish_colors<H: Context>(h: &mut H) {
    // Reacquire the palette byte for each store, as the original byte loop.
    for (lane, field) in [Field::FixedColor0, Field::FixedColor1, Field::FixedColor2]
        .into_iter()
        .enumerate()
    {
        let palette = h.session().presentation.palette();
        let value = if lane < 2 {
            palette.suit_red_green.to_le_bytes()[lane]
        } else {
            palette.crystal_flash_delay.to_le_bytes()[0]
        };
        h.store_display_control_byte(field, value, Store::Owned);
    }
}
pub fn initialize_hdma<H: Context>(h: &mut H) {
    for (field, value) in [
        (Field::MainLayers, 19),
        (Field::MainWindow, 19),
        (Field::SubLayers, 4),
        (Field::SubWindow, 4),
        (Field::NextColorSelection, 16),
        (Field::WindowSelection0, 0),
        (Field::WindowSelection1, 2),
        (Field::WindowSelection2, 32),
        (Field::NextColorMath, 51),
    ] {
        h.store_display_control_byte(field, value, Store::Owned);
    }
    publish_colors(h);
    for (address, value) in [(0xa88, 0x9800e4u32), (0xa8b, 0x98c8e4), (0xa8e, 0x999098)] {
        h.hdma_slice_native(
            address,
            u16::from_le_bytes([value.to_le_bytes()[0], value.to_le_bytes()[1]]),
        );
        h.set_current_byte(address + 2, value.to_le_bytes()[2]);
    }
    h.hdma_slice_le(0xa92, 0);
}
pub fn draw_window<H: Context>(h: &mut H) {
    let player = &h.session().player;
    let facing = player.movement_and_facing as u8;
    let x = player
        .x
        .wrapping_sub(h.inputs().camera_x)
        .wrapping_add(if facing == 4 { 65533 } else { 3 });
    let crouched = player.movement_and_facing >> 8 == 5;
    let y = player
        .y
        .wrapping_sub(h.inputs().camera_y)
        .wrapping_sub(if crouched { 12 } else { 16 });
    let off_screen = (x as i16) < 0 || x >= 256;
    if ((x as i16) < 0 && facing == 4) || ((x as i16) >= 256 && facing == 8) {
        clear_window(h);
        return;
    }
    let (angle, width) = (player.transitions.xray_angle, player.demo.held);
    let window = crate::beam_windows::XrayWindow {
        x,
        y,
        angle,
        width,
        off_screen,
        table: 0x9800,
    };
    super::beam::calculate_xray_window(window, h);
}

pub fn advance_background<H: Context>(h: &mut H) {
    apply_blending(h, BlendPhase::Beam);
    let stages: [fn(&mut H); 6] = [
        |h| advance_beam(h, BeamPhase::Initial),
        |h| advance_beam(h, BeamPhase::Widening),
        |h| advance_beam(h, BeamPhase::Full),
        |h| {
            super::xray::restore_left(
                usize::from(h.hdma_word(crate::hdma::Word::Selected) >> 1),
                h,
            )
        },
        |h| super::xray::restore_right(h),
        |h| {
            super::xray::finish_restore(
                usize::from(h.hdma_word(crate::hdma::Word::Selected) >> 1),
                h,
            )
        },
    ];
    // Retain the six-entry bounds check after the blending writes.
    stages[usize::from(h.session().player.demo.pre_action)](h);
}

pub fn advance_beam<H: Context>(h: &mut H, phase: BeamPhase) {
    if h.inputs().run_button & h.inputs().held == 0 {
        let demo = &mut h.session_mut().player.demo;
        demo.pre_action = if phase == BeamPhase::Full {
            demo.pre_action.wrapping_add(1)
        } else {
            3
        };
        return;
    }
    if phase == BeamPhase::Widening {
        crate::player_transitions::xray::widen(&mut h.session_mut().player.demo);
    } else if phase == BeamPhase::Full {
        let up = h.session().game_loop.directions[0] & h.inputs().held != 0;
        if up || h.session().game_loop.directions[1] & h.inputs().held != 0 {
            let player = &mut h.session_mut().player;
            crate::player_transitions::xray::steer(
                &mut player.transitions.xray_angle,
                player.demo.held,
                up,
            );
        }
    }
    draw_window(h);
    if phase == BeamPhase::Initial {
        let demo = &mut h.session_mut().player.demo;
        demo.pre_action = demo.pre_action.wrapping_add(1);
    }
}

pub fn advance_pickup<H: Context>(h: &mut H, gravity: bool) {
    if crate::player_transitions::suit::advance(h, gravity, set_window_word) {
        publish_colors(h);
        let mut fx = h.fx_controls();
        fx.blending = 18;
        h.publish_fx_controls(fx, &[crate::fx_state::Field::Blending]);
        return;
    }
    if !gravity {
        crate::visual_host::Effects::music_delayed8(h, 3);
    }
    for (field, value) in [
        (Field::FixedColor2, 0x80),
        (Field::FixedColor1, 64),
        (Field::FixedColor0, 32),
    ] {
        h.store_display_control_byte(field, value, Store::Owned);
    }
    h.hdma_slice_native(0xa88, 1);
    h.set_current_byte(0xa8a, 0x98);
    h.hdma_slice_native(0xa8b, 0);
    for address in [
        ram::DEMO_INPUT_PREV,
        ram::DEMO_INPUT_PREV_NEW,
        ram::DEMO_BACKUP_PREV_CONTROLLER_INPUT,
    ] {
        h.hdma_slice_le(address, 0);
    }
    set_window_word(h, 0, 255);
    h.session_mut().shared_sequence.stage = 0;
    h.session_mut().shared_sequence.value = 0;
    let palette = h.session_mut().presentation.palette_mut();
    palette.suit_red_green = 0;
    palette.crystal_flash_delay = 0;
    // Retain the original full selector identity before any subsequent store.
    let offset = usize::from(h.hdma_word(crate::hdma::Word::Selected) >> 1) * 2;
    let program = arrays::HDMA_OBJECT_INSTRUCTION_LIST_POINTERS + offset;
    if let Some(word) = crate::hdma::Word::at_address(program) {
        let value = h.hdma_word(word).wrapping_add(2);
        h.store_hdma_word(word, value, crate::scene::state::WordOrder::Little);
    } else {
        // This full identity is outside the common owner, as in the original.
        h.hdma_slice_le(program, h.current_word(program).wrapping_add(2));
    }
    let timer = arrays::HDMA_OBJECT_INSTRUCTION_TIMERS + offset;
    if let Some(word) = crate::hdma::Word::at_address(timer) {
        h.store_hdma_word(word, 1, crate::scene::state::WordOrder::Little);
    } else {
        h.hdma_slice_le(timer, 1);
    }
    h.hdma_player_command(0xb);
}
