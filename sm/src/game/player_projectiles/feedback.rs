//! Charge feedback and flare placement over the same retained gameplay state.
#![forbid(unsafe_code)]
use super::{
    Host, content,
    host::{Drawing, PaletteEffect, PlayerStep},
};
use crate::game::beam_charge_palette::{self as shared, Field, Input, Output};

struct Palette;
impl<'a, H: Host> shared::Context<&'a mut H> for Palette {
    fn field(&self, h: &&'a mut H, field: Field) -> u16 {
        match field {
            Field::Glow => h.session().controls.glow,
            Field::Phase => h.session().controls.charge_palette,
        }
    }
    fn set_field(&mut self, h: &mut &'a mut H, field: Field, value: u16) {
        match field {
            Field::Glow => h.session_mut().controls.glow = value,
            Field::Phase => h.session_mut().controls.charge_palette = value,
        }
    }
    fn input(&self, h: &&'a mut H, input: Input) -> u16 {
        match input {
            Input::Hyper => h.inputs().hyper,
            Input::GrappleInactive => u16::from(h.session().player.grapple == 0xc4f0),
            Input::Charge => h.session().controls.charge,
            Input::ContactAttack => u16::from(h.inputs().contact_attack == 4),
            Input::Suit => h.inputs().suit_palette,
        }
    }
    fn output(&mut self, h: &'a mut H, output: Output) -> (&'a mut H, u8) {
        let result = h.palette(PaletteEffect::Charge(output));
        (h, result)
    }
}
pub fn palette(h: &mut impl Host) -> u8 {
    shared::advance(h, &mut Palette).1
}
pub fn movement_sound(h: &mut impl Host) {
    if h.session().controls.resume_sound & 0x8000 != 0 {
        h.session_mut().controls.resume_sound = 1;
        return;
    }
    if h.session().controls.resume_sound != 0 {
        if h.inputs().shoot_button & h.inputs().held != 0 {
            h.sound(1, 9, 0x41);
        }
        h.session_mut().controls.resume_sound = 0;
    }
    if h.session().controls.echo_sound != 0 && h.inputs().speed_boost & 0x400 == 0 {
        h.session_mut().controls.echo_sound = 0;
        h.sound(3, 15, 0x25);
    }
    if matches!(
        h.session()
            .preparation
            .previous_movement_and_facing
            .to_le_bytes()[1],
        3 | 20
    ) && !matches!(h.inputs().movement_and_facing.to_le_bytes()[1], 3 | 20)
    {
        h.sound(1, 15, 0x32);
        if h.session().controls.charge.wrapping_sub(16) & 0x8000 == 0
            && h.inputs().shoot_button & h.inputs().held != 0
        {
            h.session_mut().controls.resume_sound = 1;
        }
    }
}
pub fn restore_sound(h: &mut impl Host) {
    if h.inputs().speed_boost & 0xff00 == 0x400 {
        h.sound(3, 6, 0x2b);
    }
    if h.session().controls.charge.wrapping_sub(16) & 0x8000 == 0 {
        h.sound(1, 6, 0x41);
    }
    h.player_step(PlayerStep::RestoreControl);
}
fn native(value: u16) -> u16 {
    u16::from_ne_bytes(value.to_le_bytes())
}
fn stored_native(value: u16) -> u16 {
    u16::from_le_bytes(value.to_ne_bytes())
}
pub fn flare(h: &mut impl Host) {
    if h.inputs().hyper != 0 {
        if h.session().controls.charge == 0 {
            return;
        }
        for part in [2, 1, 0] {
            let expired = native(h.session().controls.flare_timers[part]) == 1;
            h.session_mut().controls.flare_timers[part] =
                h.session().controls.flare_timers[part].wrapping_sub(1);
            if expired || h.session().controls.flare_timers[part] & 0x8000 != 0 {
                let previous = h.session().controls.flare_frames[part];
                h.session_mut().controls.flare_frames[part] = previous.wrapping_sub(1);
                if previous == 1 {
                    if part == 2 {
                        h.session_mut().controls.charge = 0;
                    }
                } else {
                    h.session_mut().controls.flare_timers[part] = stored_native(3);
                }
            }
            draw_flare(h, (part * 2) as u16);
        }
        return;
    }
    if h.session().controls.charge as i16 <= 0 {
        return;
    }
    if h.session().controls.charge == 1 {
        h.session_mut().controls.flare_frames = [0; 3];
        h.session_mut().controls.flare_timers = [3, 5, 4];
    }
    if h.session().controls.charge.wrapping_sub(15) & 0x8000 != 0 {
        return;
    }
    if h.session().controls.charge == 16 {
        h.sound(1, 9, 8);
    }
    for part in 0..3 {
        let remaining = native(h.session().controls.flare_timers[part]).wrapping_sub(1);
        h.session_mut().controls.flare_timers[part] = stored_native(remaining);
        if remaining & 0x8000 != 0 {
            let advanced = native(h.session().controls.flare_frames[part]).wrapping_add(1);
            h.session_mut().controls.flare_frames[part] = stored_native(advanced);
            let pointer = content::table(h, 0x90c481, part as isize);
            let sequence = h.native_record(0x90, pointer);
            let mut selected = advanced;
            match sequence.byte(usize::from(advanced)) {
                255 => {
                    h.session_mut().controls.flare_frames[part] = 0;
                    selected = 0;
                }
                254 => {
                    selected = native(h.session().controls.flare_frames[part])
                        .wrapping_sub(u16::from(sequence.byte(usize::from(advanced) + 1)));
                    h.session_mut().controls.flare_frames[part] = stored_native(selected);
                }
                _ => {}
            }
            h.session_mut().controls.flare_timers[part] =
                stored_native(u16::from(sequence.byte(usize::from(selected))));
            h.probe("player_projectiles.flare.sampled");
        }
        draw_flare(h, (part * 2) as u16);
        if h.session().controls.charge.wrapping_sub(30) & 0x8000 != 0 {
            break;
        }
    }
}
pub fn grapple_flare(h: &mut impl Host) {
    if h.session().controls.charge == 0 {
        return;
    }
    if h.session().controls.charge == 1 {
        h.session_mut().controls.flare_frames[0] = 16;
        h.session_mut().controls.flare_timers[0] = 3;
    }
    h.session_mut().controls.flare_timers[0] = h.session().controls.flare_timers[0].wrapping_sub(1);
    if h.session().controls.flare_timers[0] & 0x8000 != 0 {
        h.session_mut().controls.flare_frames[0] =
            h.session().controls.flare_frames[0].wrapping_add(1);
        let mut frame = h.session().controls.flare_frames[0];
        let sequence = h.native_record(0x90, 0xc487);
        if sequence.byte(usize::from(h.session().controls.flare_frames[0])) == 254 {
            let prior = h.session().controls.flare_frames[0];
            let count = sequence.byte(usize::from(prior.wrapping_add(1)));
            h.session_mut().controls.flare_frames[0] = prior.wrapping_sub(u16::from(count));
            frame = h.session().controls.flare_frames[0];
        }
        h.session_mut().controls.flare_timers[0] = u16::from(sequence.byte(usize::from(frame)));
        h.probe("player_projectiles.grapple_flare.sampled");
    }
    let address = if h.inputs().movement_and_facing.to_le_bytes()[0] == 4 {
        0x93a22b
    } else {
        0x93a225
    };
    let sprite = h.session().controls.flare_frames[0].wrapping_add(content::table(h, address, 0));
    let x = h.inputs().grapple_x.wrapping_sub(h.inputs().camera_x);
    let y = h.inputs().grapple_y.wrapping_sub(h.inputs().camera_y);
    if (i32::from(h.inputs().grapple_y) - i32::from(h.inputs().camera_y)) & 0xff00 == 0 {
        h.draw(Drawing::Beam { sprite, x, y });
    }
}
pub fn draw_flare(h: &mut impl Host, raw: u16) {
    let part = usize::from(raw >> 1);
    let frame = h.session().controls.flare_frames[part].to_le_bytes()[usize::from(raw & 1)];
    let base = if h.inputs().movement_and_facing.to_le_bytes()[0] == 4 {
        [0u16, 0x2a, 0x30]
    } else {
        [0, 0x1e, 0x24]
    };
    let sprite = u16::from(frame).wrapping_add(base[part]);
    let height = h
        .native_image()
        .read(content::offset(0x91b629) + usize::from(h.inputs().pose) * 8 + 4);
    let direction = h
        .native_image()
        .read(content::offset(0x91b629) + usize::from(h.inputs().pose) * 8 + 3);
    if matches!(direction, 255 | 16) {
        return;
    }
    let direction = usize::from(direction & 15);
    let (old_x, old_y) = (h.session().player.x, h.session().player.y);
    if h.inputs().ceres & 0x8000 != 0 {
        super::presentation::project_player(h);
    }
    const X: [i16; 13] = [2, 18, 15, 17, 3, -4, -17, -15, -18, -2, -4, -4, -4];
    const Y: [i16; 13] = [-28, -19, 1, 6, 17, 17, 6, 1, -20, -28, -20, -2, 8];
    const RUN_X: [i16; 10] = [2, 19, 20, 18, 3, -4, -18, -20, -19, -2];
    const RUN_Y: [i16; 10] = [-32, -22, -3, 6, 25, 25, 6, -3, -20, -32];
    let (xs, ys): (&[i16], &[i16]) = if h.inputs().movement_and_facing.to_le_bytes()[1] == 1 {
        (&RUN_X, &RUN_Y)
    } else {
        (&X, &Y)
    };
    let x = h
        .session()
        .player
        .x
        .wrapping_add_signed(xs[direction])
        .wrapping_sub(h.inputs().camera_x);
    let y = h
        .session()
        .player
        .y
        .wrapping_add_signed(ys[direction])
        .wrapping_sub(u16::from(height))
        .wrapping_sub(h.inputs().camera_y);
    if y & 0xff00 == 0 {
        h.draw(Drawing::Beam { sprite, x, y });
    }
    if h.inputs().ceres & 0x8000 != 0 {
        h.session_mut().player.y = old_y;
        h.session_mut().player.x = old_x;
    }
}
