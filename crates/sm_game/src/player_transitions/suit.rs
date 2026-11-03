//! Shared Varia/Gravity acquisition and light-beam sequence.
use super::{effects::Host, state::LandingBounce};
use crate::player_transitions::effects::SamusLease;
use crate::{
    samus_palette,
    session::Session,
    visual_host::{Record, Source},
    visual_state::PaletteBuffer,
};

pub fn prepare<H: Host>(h: &mut H, gravity: bool, write_window: fn(&mut H, u16, u16)) {
    let palette = &mut h.session_mut().presentation.palette_mut();
    palette.suit_red_green = u16::from_le_bytes([48, if gravity { 73 } else { 80 }]);
    palette.crystal_flash_delay = if gravity { 0x0190 } else { 0x0080 };
    samus_palette::cancel_speed_boost(h);
    let s = h.session_mut();
    s.player.contact.extra_run = Default::default();
    s.player.contact.base_run = Default::default();
    s.player.contact.y_speed = Default::default();
    s.player.contact.y_direction = 0;
    s.player.transitions.bounce = LandingBounce::from_raw(0);
    s.player.contact.acceleration_mode = 0;
    s.room.entry.elevator_status = 0;
    s.shared_sequence.stage = 0;
    s.shared_sequence.value = 0;
    s.presentation.palette_mut().suit_beam_speed = 256;
    for index in (0..256).rev() {
        write_window(h, index, 255);
    }
    if matches!(h.session().player.movement_and_facing >> 8, 3 | 20) {
        h.sound(1, 9, 0x32);
    }
    h.session_mut().player.pose =
        if h.session().inventory.equipment.active & if gravity { 1 } else { 0x20 } != 0 {
            0x9b
        } else {
            0
        };
    super::refresh_pose(h);
}

pub fn center(h: &mut impl Host) {
    let x = h.inputs().camera_x.wrapping_add(120);
    let y = h.inputs().camera_y.wrapping_add(136);
    let p = &mut h.session_mut().player;
    p.x = x;
    p.previous_position.x.whole = x;
    p.y = y;
    p.previous_position.y.whole = y;
}

fn colors(s: &mut Session, white: bool) {
    let palette = &mut s.presentation.palette_mut();
    let [red, green] = palette.suit_red_green.to_le_bytes();
    let [blue, gravity] = palette.crystal_flash_delay.to_le_bytes();
    let targets = if white {
        [63, 95, 159]
    } else if gravity != 0 {
        [48, 73, 144]
    } else {
        [63, 77, 131]
    };
    let mut channels = [red, green, blue];
    for (channel, target) in channels.iter_mut().zip(targets) {
        if *channel != target {
            *channel = if white {
                channel.wrapping_add(2)
            } else {
                channel.wrapping_sub(1)
            };
            if white && channel.wrapping_sub(target.wrapping_add(1)) & 0x80 == 0 {
                *channel = target;
            }
        }
    }
    palette.suit_red_green = u16::from_le_bytes([channels[0], channels[1]]);
    palette.crystal_flash_delay = u16::from_le_bytes([channels[2], gravity]);
}

fn fill_edges<H: Host>(
    h: &mut H,
    word: u16,
    shorten_bottom: bool,
    write_window: fn(&mut H, u16, u16),
) {
    for (mut offset, step, shorter) in [(0u16, 2u16, false), (510, (-2i16) as u16, shorten_bottom)]
    {
        let mut count = (h.session().shared_sequence.value as i16).wrapping_sub(i16::from(shorter));
        // Both original loops write once even for zero/negative counts.
        loop {
            write_window(h, offset >> 1, word);
            offset = offset.wrapping_add(step);
            count = count.wrapping_sub(1);
            if count <= 0 {
                break;
            }
        }
    }
}

pub fn expand<H: Host>(h: &mut H, write_window: fn(&mut H, u16, u16)) {
    let speed = (h.session().presentation.palette().suit_beam_speed >> 8) as u8;
    let [left, right] = h.session().shared_sequence.value.to_le_bytes();
    h.session_mut().shared_sequence.value =
        u16::from_le_bytes([left.wrapping_sub(speed), right.wrapping_add(speed)]);
    for index in (0..256).rev() {
        let word = h.session().shared_sequence.value;
        write_window(h, index, word);
    }
    if (h.session().shared_sequence.value as u8) < 97 {
        h.session_mut().shared_sequence.stage = h.session().shared_sequence.stage.wrapping_add(1);
        h.session_mut().shared_sequence.value = 0x846c;
    }
}

pub fn advance<H: Host>(h: &mut H, gravity: bool, write_window: fn(&mut H, u16, u16)) -> bool {
    match h.session().shared_sequence.stage {
        0 => {
            h.session_mut().shared_sequence.value =
                h.session().shared_sequence.value.wrapping_add(8);
            fill_edges(h, 0x7878, false, write_window);
            if (h.session().shared_sequence.value as i16).wrapping_sub(128) >= 0 {
                h.session_mut().shared_sequence.stage =
                    h.session().shared_sequence.stage.wrapping_add(1);
                h.session_mut().shared_sequence.value = 0x7878;
            }
        }
        1 => expand(h, write_window),
        2 => {
            colors(h.session_mut(), true);
            let speed = (h.session().presentation.palette().suit_beam_speed >> 8) as u8;
            let [left, right] = h.session().shared_sequence.value.to_le_bytes();
            let left = left.wrapping_sub(speed);
            h.session_mut().shared_sequence.value = if left == 0 || (left as i8) < 0 {
                0xff00
            } else {
                u16::from_le_bytes([left, right.saturating_add(speed)])
            };
            for row in 0..256u16 {
                let width_index = if row < 128 { row } else { 255 - row };
                let width = h
                    .table(Source::Native, 0x88e3c9)
                    .byte(usize::from(width_index));
                let [left, right] = h.session().shared_sequence.value.to_le_bytes();
                let left = (left as i8).wrapping_sub(width as i8).max(0) as u8;
                write_window(
                    h,
                    row,
                    u16::from_le_bytes([left, right.saturating_add(width)]),
                );
            }
            h.session_mut().presentation.palette_mut().suit_beam_speed = h
                .session()
                .presentation
                .palette()
                .suit_beam_speed
                .wrapping_add(96);
            if h.session().shared_sequence.value == 0xff00 {
                h.session_mut().shared_sequence.stage =
                    h.session().shared_sequence.stage.wrapping_add(1);
                h.session_mut().presentation.palette_mut().suit_beam_speed >>= 1;
                h.session_mut().shared_sequence.value = 0;
            }
        }
        3 => {
            let mask = if gravity { 0x20 } else { 1 };
            h.session_mut().inventory.equipment.active |= mask;
            h.session_mut().inventory.equipment.owned |= mask;
            h.session_mut().player.pose = 0x9b;
            super::finish_pose_change(h);
            samus_palette::load_suit(h, PaletteBuffer::Current);
            h.session_mut().shared_sequence.stage =
                h.session().shared_sequence.stage.wrapping_add(1);
        }
        4 => {
            colors(h.session_mut(), false);
            h.session_mut().shared_sequence.value = h
                .session()
                .shared_sequence
                .value
                .wrapping_add(h.session().presentation.palette().suit_beam_speed >> 8);
            fill_edges(h, 255, true, write_window);
            let speed = h
                .session()
                .presentation
                .palette()
                .suit_beam_speed
                .wrapping_sub(32);
            h.session_mut().presentation.palette_mut().suit_beam_speed =
                if (speed as i16).wrapping_sub(256) < 0 {
                    256
                } else {
                    speed
                };
            if (h.session().shared_sequence.value as i16).wrapping_sub(128) >= 0 {
                h.session_mut().shared_sequence.stage =
                    h.session().shared_sequence.stage.wrapping_add(1);
                h.session_mut().shared_sequence.value = (-1793i16) as u16;
            }
        }
        5 => {
            let [left, right] = h.session().shared_sequence.value.to_le_bytes();
            h.session_mut().shared_sequence.value =
                u16::from_le_bytes([left.wrapping_add(8), right.wrapping_sub(8)]);
            let word = h.session().shared_sequence.value;
            write_window(h, 128, word);
            if (h.session().shared_sequence.value as u8) >= 112 {
                h.session_mut().shared_sequence.stage =
                    h.session().shared_sequence.stage.wrapping_add(1);
            }
        }
        6 => return false,
        _ => panic!("invalid suit pickup stage"),
    }
    true
}
pub fn publish_colors(h: &mut (impl crate::runtime_access::SessionRam + ?Sized)) {
    for lane in 0..3 {
        h.set_byte(0x74 + lane, h.byte(0xdf0 + lane));
    }
}
/// Set up the suit pickup's window, blending and HDMA tables; copy its colours.
pub fn initialize_hdma(
    h: &mut (impl crate::runtime_access::SessionRam + crate::session_access::SessionAccess + ?Sized),
) {
    for (address, value) in [
        (0x69, 19),
        (0x6c, 19),
        (0x6b, 4),
        (0x6d, 4),
        (0x6e, 16),
        (0x60, 0),
        (0x61, 2),
        (0x62, 32),
        (0x71, 51),
    ] {
        h.set_byte(address, value);
    }
    publish_colors(h);
    // The indirect HDMA table at $7E:0A88..$7E:0A90: three entries of a line
    // count and a pointer, each stored as a native word and its bank byte.
    // It overlays the demo's words from its enable word on and the low byte
    // of its filtered held input.
    let mut table = [0u8; 9];
    for (at, value) in [(0, 0x9800e4u32), (3, 0x98c8e4), (6, 0x999098)] {
        table[at..at + 2].copy_from_slice(&(value as u16).to_ne_bytes());
        table[at + 2] = (value >> 16) as u8;
    }
    let word = |at: usize| u16::from_le_bytes([table[at], table[at + 1]]);
    let s = h.session_mut();
    let demo = &mut s.player.demo;
    demo.enabled = word(0);
    demo.input_frames = word(2);
    demo.previous_held = word(4);
    demo.previous_pressed = word(6);
    // $7E:0A91..$7E:0A92 (the X-ray HDMA table's end) are the high byte of
    // the demo's filtered held input and the low byte of its filtered newly
    // pressed input.
    let preparation = &mut s.projectiles.preparation;
    crate::actors::slot_view::store_byte(&mut preparation.demo_filtered_held, 0, table[8]);
    crate::actors::slot_view::store_byte(&mut preparation.demo_filtered_held, 1, 0);
    crate::actors::slot_view::store_byte(&mut preparation.demo_filtered_pressed, 0, 0);
    h.words_stored();
}
/// Start a suit pickup: close the window, centre Samus and spawn its HDMA.
pub fn begin_pickup<H: SamusLease>(h: &mut H, gravity: bool) {
    use crate::{display::hdma_spawn::HdmaObjectDefinition, player_transitions::xray};
    prepare(h, gravity, xray::set_window_word);
    h.native_player_command(0x15);
    center(h);
    crate::visual_host::Effects::sound(h, 2, 6, 0x56);
    let program = if gravity { 0xd67a } else { 0xd5a2 };
    h.spawn_hdma_object(&HdmaObjectDefinition::new(0x41, 0x26, program));
}
/// Advance the suit pickup's window, then end it and release Samus.
pub fn advance_pickup<
    H: SamusLease + crate::hdma_objects::HdmaTable + crate::fx_domain::Controls,
>(
    h: &mut H,
    gravity: bool,
) {
    use crate::{player_transitions::xray, ram::word_arrays as arrays};
    if advance(h, gravity, xray::set_window_word) {
        publish_colors(h);
        let mut fx = h.fx_controls();
        fx.blending = 18;
        h.publish_fx_controls(fx, &[crate::fx_state::Field::Blending]);
        return;
    }
    if !gravity {
        crate::visual_host::Effects::music_delayed8(h, 3);
    }
    for (address, value) in [(0x76, 0x80), (0x75, 64), (0x74, 32)] {
        h.set_byte(address, value);
    }
    // $7E:0A88, the HDMA pointer's low word, is the demo's enable word.
    // The bank byte $98 at $7E:0A8A and the zero word after it: the demo's
    // input frame count, and the previous held input's low byte, which is
    // cleared below.
    let demo = &mut h.session_mut().player.demo;
    demo.enabled = u16::from_le_bytes(1u16.to_ne_bytes());
    demo.input_frames = 0x98;
    h.words_stored();
    let s = h.session_mut();
    s.player.demo.previous_held = 0;
    s.player.demo.previous_pressed = 0;
    s.projectiles.preparation.demo_filtered_held = 0;
    h.words_stored();
    xray::set_window_word(h, 0, 255);
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
        // LEGACY(original-overflow): a selector past the HDMA object table
        // increments whatever RAM word lies at the list pointers + selector.
        let value = crate::legacy_overflow::ram_word(h, program).wrapping_add(2);
        crate::legacy_overflow::ram_write_slice_le(h, program, value);
    }
    let timer = arrays::HDMA_OBJECT_INSTRUCTION_TIMERS + offset;
    if let Some(word) = crate::hdma::Word::at_address(timer) {
        h.store_hdma_word(word, 1, crate::scene::state::WordOrder::Little);
    } else {
        // LEGACY(original-overflow): as above, at the instruction timers + selector.
        crate::legacy_overflow::ram_write_slice_le(h, timer, 1);
    }
    h.native_player_command(0xb);
}
