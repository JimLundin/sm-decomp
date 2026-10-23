//! Charge/flare/sound at their original phase boundaries, with live owner observations.
use super::{Context, Transaction};
use crate::game::beam_charge_palette::{self as shared, Field, Input, Output};
use crate::ram_map::words;
fn offset(field: Field) -> usize {
    match field {
        Field::Glow => words::charged_shot_glow_timer,
        Field::Phase => words::samus_charge_palette_index,
    }
}
impl<'g> shared::Context<Transaction<'g>> for Context {
    fn field(&self, game: &Transaction<'g>, field: Field) -> u16 {
        let at = offset(field);
        game.read_le(at)
    }
    fn set_field(&mut self, game: &mut Transaction<'g>, field: Field, value: u16) {
        let at = offset(field);
        game.write_slice_le(at, value);
    }
    fn input(&self, game: &Transaction<'g>, input: Input) -> u16 {
        match input {
            Input::Hyper => word!(game.hyper_beam_flag),
            Input::GrappleInactive => u16::from(word!(game.grapple_beam_function) == 0xc4f0),
            Input::Charge => word!(game.flare_counter),
            Input::ContactAttack => u16::from(word!(game.samus_contact_damage_index) == 4),
            Input::Suit => word!(game.samus_suit_palette_index),
        }
    }
    fn output(&mut self, game: Transaction<'g>, output: Output) -> (Transaction<'g>, u8) {
        game.foreign(|g| {
            crate::game::beam_charge_palette_ram::output_with_image(g, output, |g| {
                g.loaded_rom.as_ref().expect("ROM must be loaded").clone()
            })
        })
    }
}
pub(super) fn palette(game: Transaction<'_>) -> (Transaction<'_>, u8) {
    shared::advance(game, &mut Context::default())
}

pub(super) fn movement_sound(mut o: Transaction<'_>) -> Transaction<'_> {
    if word!(o.play_resume_charging_beam_sfx) & 0x8000 != 0 {
        word!(o.play_resume_charging_beam_sfx = 1);
        return o;
    }
    if word!(o.play_resume_charging_beam_sfx) != 0 {
        if word!(o.button_config_shoot_x) & word!(o.joypad1_lastkeys) != 0 {
            o = o.foreign(|g| crate::game::sm_80::QueueSfx1_Max9(0x41, g)).0;
        }
        word!(o.play_resume_charging_beam_sfx = 0);
    }
    if word!(o.samus_echoes_sound_flag) != 0 && word!(o.speed_boost_counter) & 0x400 == 0 {
        word!(o.samus_echoes_sound_flag = 0);
        o = o
            .foreign(|g| crate::game::sm_80::QueueSfx3_Max15(0x25, g))
            .0;
    }
    if matches!(o.byte(0xa11), 3 | 20) && !matches!(o.byte(0xa1f), 3 | 20) {
        o = o
            .foreign(|g| crate::game::sm_80::QueueSfx1_Max15(0x32, g))
            .0;
        if word!(o.flare_counter).wrapping_sub(16) & 0x8000 == 0
            && word!(o.button_config_shoot_x) & word!(o.joypad1_lastkeys) != 0
        {
            word!(o.play_resume_charging_beam_sfx = 1);
        }
    }
    o
}
pub(super) fn restore_sound(mut o: Transaction<'_>) -> Transaction<'_> {
    if word!(o.speed_boost_counter) & 0xff00 == 0x400 {
        o = o.foreign(|g| crate::game::sm_80::QueueSfx3_Max6(0x2b, g)).0;
    }
    if word!(o.flare_counter).wrapping_sub(16) & 0x8000 == 0 {
        o = o.foreign(|g| crate::game::sm_80::QueueSfx1_Max6(0x41, g)).0;
    }
    o.foreign(|g| unsafe { crate::game::sm_90::CallSomeSamusCode(0x14, g) })
        .0
}
pub(super) fn flare(mut o: Transaction<'_>) -> Transaction<'_> {
    if word!(o.hyper_beam_flag) != 0 {
        if word!(o.flare_counter) == 0 {
            return o;
        }
        for part in [4, 2, 0] {
            let timer = 0xcdc + part;
            let frame = 0xcd6 + part;
            let expired = o.read_native(timer) == 1;
            o.write_slice_le(timer, o.read_le(timer).wrapping_sub(1));
            if expired || o.read_le(timer) & 0x8000 != 0 {
                let previous = o.read_le(frame);
                o.write_slice_le(frame, previous.wrapping_sub(1));
                if previous == 1 {
                    if part == 4 {
                        word!(o.flare_counter = 0);
                    }
                } else {
                    o.write_slice_native(timer, 3);
                }
            }
            o = o
                .foreign(|g| unsafe {
                    crate::game::sm_90::DrawFlareAnimationComponent(part as u16, g)
                })
                .0;
        }
        return o;
    }
    if word!(o.flare_counter) as i16 <= 0 {
        return o;
    }
    if word!(o.flare_counter) == 1 {
        for part in [0, 2, 4] {
            o.write_word_le(0, 0xcd6 + part);
        }
        for (part, duration) in [(0, 3), (2, 5), (4, 4)] {
            o.write_word_le(duration, 0xcdc + part);
        }
    }
    if word!(o.flare_counter).wrapping_sub(15) & 0x8000 != 0 {
        return o;
    }
    if word!(o.flare_counter) == 16 {
        o = o.foreign(|g| crate::game::sm_80::QueueSfx1_Max9(8, g)).0;
    }
    for part in [0, 2, 4] {
        let timer = 0xcdc + part;
        let frame = 0xcd6 + part;
        let remaining = o.read_native(timer).wrapping_sub(1);
        o.write_slice_native(timer, remaining);
        if remaining & 0x8000 != 0 {
            let advanced = o.read_native(frame).wrapping_add(1);
            o.write_slice_native(frame, advanced);
            let pointer = o.rom().table_word(
                crate::rtl::lorom_offset(0x90c481) as usize,
                (part / 2) as isize,
            );
            let (next, sequence) = unsafe { super::native_content::flare_sequence(o, pointer) };
            o = next;
            let mut selected = advanced;
            match unsafe { sequence.byte(usize::from(advanced)) } {
                255 => {
                    o.write_slice_native(frame, 0);
                    selected = 0;
                }
                254 => {
                    selected = o.read_native(frame).wrapping_sub(u16::from(unsafe {
                        sequence.byte(usize::from(advanced) + 1)
                    }));
                    o.write_slice_native(frame, selected);
                }
                _ => {}
            }
            o.write_slice_native(
                timer,
                u16::from(unsafe { sequence.byte(usize::from(selected)) }),
            );
        }
        o = o
            .foreign(|g| unsafe { crate::game::sm_90::DrawFlareAnimationComponent(part as u16, g) })
            .0;
        if word!(o.flare_counter).wrapping_sub(30) & 0x8000 != 0 {
            break;
        }
    }
    o
}
pub(super) fn grapple_flare(mut o: Transaction<'_>) -> Transaction<'_> {
    if word!(o.flare_counter) == 0 {
        return o;
    }
    if word!(o.flare_counter) == 1 {
        word!(o.flare_animation_frame = 16);
        word!(o.flare_animation_timer = 3);
    }
    word!(o.flare_animation_timer = word!(o.flare_animation_timer).wrapping_sub(1));
    if word!(o.flare_animation_timer) & 0x8000 != 0 {
        word!(o.flare_animation_frame = word!(o.flare_animation_frame).wrapping_add(1));
        let mut frame = word!(o.flare_animation_frame);
        let (next, sequence) = unsafe { super::native_content::flare_sequence(o, 0xc487) };
        o = next;
        if unsafe { sequence.byte(usize::from(word!(o.flare_animation_frame))) } == 254 {
            let prior = word!(o.flare_animation_frame);
            let count = unsafe { sequence.byte(usize::from(prior.wrapping_add(1))) };
            word!(o.flare_animation_frame = prior.wrapping_sub(u16::from(count)));
            frame = word!(o.flare_animation_frame);
        }
        word!(o.flare_animation_timer = u16::from(unsafe { sequence.byte(usize::from(frame)) }));
    }
    let address = if o.byte(0xa1e) == 4 {
        0x93a22b
    } else {
        0x93a225
    };
    let sprite = word!(o.flare_animation_frame).wrapping_add(
        o.rom()
            .table_word(crate::rtl::lorom_offset(address) as usize, 0),
    );
    let x = word!(o.x_pos_of_start_of_grapple_beam_prevframe).wrapping_sub(word!(o.layer1_x_pos));
    let y = word!(o.y_pos_of_start_of_grapple_beam_prevframe).wrapping_sub(word!(o.layer1_y_pos));
    if (i32::from(word!(o.y_pos_of_start_of_grapple_beam_prevframe))
        - i32::from(word!(o.layer1_y_pos)))
        & 0xff00
        == 0
    {
        o = o
            .foreign(|g| unsafe { crate::game::sm_81::DrawBeamGrappleSpritemap(sprite, x, y, g) })
            .0;
    }
    o
}
