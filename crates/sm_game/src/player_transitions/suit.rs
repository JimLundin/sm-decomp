//! Shared Varia/Gravity acquisition and light-beam sequence.
#![forbid(unsafe_code)]
use super::{effects::Host, state::LandingBounce};
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
