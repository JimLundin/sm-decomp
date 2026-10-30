//! Arm-cannon opening, animation and pose-selected graphics.
#![forbid(unsafe_code)]
use crate::{
    session::Controls,
    visual_host::{Host, Record, Source},
};

pub enum Tiles {
    Beam(u16),
    Cannon(u16),
}

fn direction(h: &impl Host, record: &impl Record, first: u8) -> u8 {
    if first & 0x80 != 0 {
        record.byte(if h.session().player.animation_clock.frame != 0 {
            2
        } else {
            0
        }) & 0x7f
    } else {
        first
    }
}

fn advance(controls: &mut Controls) {
    if controls.cannon_flags as u8 != 0 {
        if controls.cannon_frame.wrapping_sub(2) & 0x8000 != 0 {
            controls.cannon_frame = controls.cannon_frame.wrapping_add(1);
            return;
        }
        controls.cannon_frame = 3;
    } else {
        if controls.cannon_frame != 1 && (controls.cannon_frame as i16).wrapping_sub(1) >= 0 {
            controls.cannon_frame = controls.cannon_frame.wrapping_sub(1);
            return;
        }
        controls.cannon_frame = 0;
    }
    controls.cannon_flags &= 0xff;
}

pub fn update(h: &mut impl Host) {
    let controls = &mut h.session_mut().projectiles.controls;
    if controls.cannon_flags >> 8 != 0 {
        advance(controls);
    } else if controls.weapon_changed.wrapping_sub(2) & 0x8000 == 0 {
        let open = [0, 1, 1, 0, 1, 0][usize::from(controls.selected_weapon)];
        if u16::from(controls.cannon_flags as u8) != open {
            controls.cannon_frame = if open != 0 { 0 } else { 4 };
            controls.cannon_flags = 0x100 | open;
            advance(controls);
        }
    }
    let pointer = h
        .table(Source::Loaded, 0x90c7df)
        .native_word(2 * usize::from(h.session().player.pose));
    let mode = h.legacy_native_record(0x90, pointer).byte(1);
    h.session_mut().projectiles.controls.cannon_drawing_mode = u16::from(mode);
}

pub fn update_beam<H: Host>(h: &mut H, queue: fn(&mut H, Tiles)) {
    let beams = h.session().inventory.beams.active & 0xfff;
    queue(h, Tiles::Beam(beams));
    crate::samus_palette::write_beam_palette(h, beams);
}

pub fn draw<H: Host>(h: &mut H, tile: fn(&mut H, u16, u16, u16, u16), queue: fn(&mut H, Tiles)) {
    const CHARACTERS: [u16; 10] = [
        0x281f, 0x281f, 0x281f, 0x681f, 0xa81f, 0xe81f, 0x281f, 0x681f, 0x681f, 0x681f,
    ];
    if h.session().projectiles.controls.cannon_frame == 0
        || h.session().player.invincibility != 0 && h.inputs().frame & 1 != 0
    {
        return;
    }
    let pointer = h
        .table(Source::Loaded, 0x90c7df)
        .native_word(2 * usize::from(h.session().player.pose));
    let record = h.legacy_native_record(0x90, pointer);
    let first = record.byte(0);
    let character = CHARACTERS[usize::from(direction(h, &record, first))];
    let offsets = pointer.wrapping_add(if first & 0x80 != 0 { 4 } else { 2 });
    let frame = h.session().player.animation_clock.frame;
    let offsets = h.legacy_native_record(0x90, offsets.wrapping_add(frame.wrapping_mul(2)));
    let x = offsets.byte(0) as i8 as u16;
    let y = offsets.byte(1) as i8 as u16;
    let body_y = h
        .table(Source::Native, 0x91b629)
        .byte(usize::from(h.session().player.pose) * 8 + 4);
    tile(h, x, y, u16::from(body_y), character);
    // OAM output can alias pose/frame; select the upload from the resulting state.
    let pointer = h
        .table(Source::Loaded, 0x90c7df)
        .native_word(2 * usize::from(h.session().player.pose));
    let record = h.legacy_native_record(0x90, pointer);
    let direction = direction(h, &record, record.byte(0));
    let pointer = h
        .table(Source::Loaded, 0x90c7a5)
        .native_word(usize::from(direction & 0x7f) * 2)
        .wrapping_add(
            h.session()
                .projectiles
                .controls
                .cannon_frame
                .wrapping_mul(2),
        );
    queue(h, Tiles::Cannon(pointer));
}
