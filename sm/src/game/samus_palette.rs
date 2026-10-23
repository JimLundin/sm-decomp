//! Samus palette phases share the actual visual, player and inventory owner.
#![forbid(unsafe_code)]
use crate::game::{
    beam_charge_palette::Output,
    liquid_surface::{self, LiquidAtHeight},
    session::Session,
    visual_host::{Host, Record, Source},
    visual_state::{PaletteBuffer, PaletteCarry},
};

fn carry(set: bool) -> PaletteCarry {
    if set {
        PaletteCarry::Set
    } else {
        PaletteCarry::Clear
    }
}
fn suit_source(session: &Session) -> u16 {
    if session.inventory.equipment.active & 0x20 != 0 {
        0x9800
    } else if session.inventory.equipment.active & 1 != 0 {
        0x9520
    } else {
        0x9400
    }
}
pub(crate) fn update_suit_index(session: &mut Session) {
    session.player.palette.suit_index = match suit_source(session) {
        0x9800 => 4,
        0x9520 => 2,
        _ => 0,
    };
}
fn copy_colors<const N: usize>(
    h: &mut impl Host,
    buffer: PaletteBuffer,
    first: usize,
    source: u16,
) {
    let bytes = h.record(Source::Native, 0x9b, source).bytes::<N>(0);
    let colors = match buffer {
        PaletteBuffer::Current => &mut h.session_mut().visual.colors,
        PaletteBuffer::Target => &mut h.session_mut().visual.targets,
    };
    for (i, pair) in bytes.chunks_exact(2).enumerate() {
        colors[first + i] = u16::from_le_bytes([pair[0], pair[1]]);
    }
}
pub(crate) fn copy_suit(h: &mut impl Host, buffer: PaletteBuffer, source: u16) {
    #[cfg(sm_verify_dll)]
    if buffer == PaletteBuffer::Current {
        crate::game::visual_runtime::samus_observer::event("copy", source);
    }
    copy_colors::<32>(h, buffer, 192, source);
}
pub(crate) fn copy_crystal_first10(h: &mut impl Host, source: u16) {
    copy_colors::<20>(h, PaletteBuffer::Current, 224, source);
}
pub(crate) fn copy_crystal_last6(h: &mut impl Host, source: u16) {
    copy_colors::<12>(h, PaletteBuffer::Current, 234, source);
}
pub(crate) fn load_suit(h: &mut impl Host, buffer: PaletteBuffer) {
    #[cfg(sm_verify_dll)]
    if buffer == PaletteBuffer::Current {
        crate::game::visual_runtime::samus_observer::event("load", 0);
    }
    let source = suit_source(h.session());
    copy_suit(h, buffer, source);
}
fn suit_table(h: &impl Host, address: u32) -> u16 {
    let record = h.table(Source::Loaded, address);
    record.native_word(usize::from(h.session().player.palette.suit_index >> 1) * 2)
}
fn frame_palette(h: &mut impl Host, table: u32) {
    let base = suit_table(h, table);
    let address = base.wrapping_add(h.session().player.palette_frame);
    let source = h.record(Source::Native, 0x91, address).native_word(0);
    copy_suit(h, PaletteBuffer::Current, source);
}
pub(crate) fn advance(h: &mut impl Host) {
    h.probe("samus.palette.entry");
    if h.session().player.special_palette_flags & 0x8000 == 0 {
        let restore = if h.advance_charge_palette() & 1 != 0 {
            true
        } else {
            #[cfg(sm_verify_dll)]
            crate::game::visual_runtime::samus_observer::inject(
                "samus.palette.dispatch",
                h.session_mut(),
            );
            h.probe("samus.palette.dispatch");
            let phase = match h.session().player.shine_tick {
                0 => screw_speed(h),
                1 => shine(h),
                2 => {
                    let result = misc(h);
                    h.probe(if result == PaletteCarry::Set {
                        "samus.palette.carry.set"
                    } else {
                        "samus.palette.carry.clear"
                    });
                    result
                }
                3..=5 => panic!("non-null function pointer"),
                6 => shinespark(h),
                7 => crystal_flash(h),
                8 => xray(h),
                9 => {
                    visor(h);
                    PaletteCarry::Clear
                }
                10 => PaletteCarry::Clear,
                index => {
                    let _ = [(); 11][usize::from(index)];
                    unreachable!()
                }
            };
            phase == PaletteCarry::Clear
        };
        if restore {
            let source = suit_table(h, 0x91d727);
            copy_suit(h, PaletteBuffer::Current, source);
        }
    }
    h.probe("samus.palette.tail");
    misc(h);
    h.probe("samus.palette.return");
}
pub(crate) fn visor(h: &mut impl Host) -> u8 {
    if h.session().player.shine_tick == 8 {
        return 0;
    }
    if matches!(h.inputs().blending_config, 40 | 42) {
        let timer = h.session().player.palette.visor_timer.wrapping_sub(1);
        h.session_mut().player.palette.visor_timer = timer;
        if timer as u8 != 0 {
            return 0;
        }
        h.session_mut().player.palette.visor_timer = timer | 5;
        let record = h.table(Source::Loaded, 0x9ba3c0);
        let value = record.native_word(usize::from((timer >> 8) >> 1) * 2);
        h.session_mut().visual.colors[196] = value;
        let frame = (timer >> 8) + 2;
        let high = if frame.wrapping_sub(12) & 0x8000 != 0 {
            frame.swap_bytes()
        } else {
            0x600
        };
        h.session_mut().player.palette.visor_timer =
            high | (h.session().player.palette.visor_timer & 0xff);
    } else {
        h.session_mut().player.palette.visor_timer = 1537;
    }
    0
}
/// Carry is the original phase result observed at91D715, independently checked
/// against the oracle. The unconditional final misc call discards this result.
pub(crate) fn misc(h: &mut impl Host) -> PaletteCarry {
    #[cfg(sm_verify_dll)]
    crate::game::visual_runtime::samus_observer::event("misc", 0);
    if h.session().player.special_palette_flags == 0 {
        let hurt = h.session().player.palette.hurt_flash;
        if h.session().player.palette.hurt_flash == 0 {
            return PaletteCarry::Clear;
        }
        let restore = if h.session().player.palette.hurt_flash == 2
            && h.inputs().cinematic == 0
            && (h.inputs().frame_handler_beta != 0xe8d9 || h.session().player.pose != 0x54)
        {
            h.sound(1, 6, 0x35);
            true
        } else if hurt.wrapping_sub(7) & 0x8000 == 0 {
            false
        } else if hurt & 1 != 0 {
            copy_suit(h, PaletteBuffer::Current, 0xa380);
            false
        } else {
            true
        };
        if restore {
            if h.inputs().cinematic != 0 {
                copy_suit(h, PaletteBuffer::Current, 0xa3a0);
            } else {
                load_suit(h, PaletteBuffer::Current);
            }
        }
        let value = h.session().player.palette.hurt_flash.wrapping_add(1);
        h.session_mut().player.palette.hurt_flash = value;
        if value == 40 {
            let grapple = h.session().player.grapple;
            if grapple == 0xc4f0 {
                if matches!(h.session().player.movement_and_facing >> 8, 3 | 20) {
                    h.resume_movement_sound();
                    return PaletteCarry::Set;
                }
                let charge = h.session().projectiles.controls.charge;
                if charge.wrapping_sub(16) & 0x8000 == 0
                    && h.inputs().shoot_button & h.inputs().held != 0
                {
                    h.session_mut().projectiles.controls.resume_sound = 1;
                }
                return carry(charge >= 16);
            }
            if grapple.wrapping_add(0x37aa) & 0x8000 != 0 {
                h.sound(1, 9, 6);
            }
            return carry(grapple >= 0xc856);
        }
        if value.wrapping_sub(60) & 0x8000 == 0 {
            h.session_mut().player.palette.hurt_flash = 0;
        }
        return carry(value >= 60);
    }
    if h.session().player.special_palette_flags & 0x8000 != 0 {
        let shift_carry = h.session().projectiles.controls.charge_palette & 0x8000 != 0;
        let record = h.table(Source::Loaded, 0x91d99e);
        let source =
            record.native_word(usize::from(h.session().projectiles.controls.charge_palette) * 2);
        copy_suit(h, PaletteBuffer::Current, source);
        h.session_mut().player.palette.timer = h.session().player.palette.timer.wrapping_sub(1);
        let timer = h.session().player.palette.timer;
        if timer != 0 && timer & 0x8000 == 0 {
            return carry(shift_carry);
        }
        h.session_mut().player.palette.timer = h.session().player.palette_frame;
        let next = h
            .session()
            .projectiles
            .controls
            .charge_palette
            .wrapping_add(1);
        h.session_mut().projectiles.controls.charge_palette = next;
        let result = next >= 10;
        if next.wrapping_sub(10) & 0x8000 == 0 {
            h.session_mut().projectiles.controls.charge_palette = 0;
        }
        return carry(result);
    } else {
        if h.session().player.special_palette_flags & 1 != 0 {
            let source = suit_table(h, 0x91d998);
            copy_suit(h, PaletteBuffer::Current, source);
        } else {
            load_suit(h, PaletteBuffer::Current);
        }
        h.session_mut().player.special_palette_flags =
            h.session().player.special_palette_flags.wrapping_add(1);
    }
    // Positive flags return clear at the phase observation.
    PaletteCarry::Clear
}
pub(crate) fn screw_speed(h: &mut impl Host) -> PaletteCarry {
    if h.session().player.palette.suit_index & 4 == 0 {
        let y = h.session().player.y;
        let pose = h.session().player.pose;
        let record = h.table(Source::Loaded, 0x91b629);
        let radius = record.byte(usize::from(pose) * 8 + 6);
        let top = y.wrapping_sub(u16::from(radius));
        if liquid_surface::at_height(
            h.inputs().fx_y,
            h.inputs().lava_y,
            h.session().room.liquid_options,
            top,
        ) != LiquidAtHeight::None
        {
            return PaletteCarry::Set;
        }
    }
    let movement = h.session().player.movement_and_facing >> 8;
    let screw = if movement == 3 {
        if h.session().inventory.equipment.active & 8 == 0 {
            false
        } else if h.inputs().player_animation_frame != 0 {
            if h.inputs().player_animation_frame.wrapping_sub(27) & 0x8000 == 0 {
                return PaletteCarry::Clear;
            }
            true
        } else {
            h.session_mut().player.palette_frame = 0;
            return PaletteCarry::Set;
        }
    } else if movement == 20 {
        if h.session().inventory.equipment.active & 8 == 0 {
            return PaletteCarry::Set;
        }
        if h.inputs().player_animation_frame.wrapping_sub(3) & 0x8000 == 0 {
            true
        } else {
            h.session_mut().player.palette_frame = 0;
            return PaletteCarry::Set;
        }
    } else {
        false
    };
    if screw {
        frame_palette(h, 0x91da4a);
        let frame = h.session().player.palette_frame;
        h.session_mut().player.palette_frame = if frame >= 10 {
            0
        } else {
            frame.wrapping_add(2)
        };
    } else {
        if h.session().player.contact.speed_boost & 0xff00 != 1024 {
            return PaletteCarry::Set;
        }
        h.session_mut().player.palette.timer = h.session().player.palette.timer.wrapping_sub(1);
        let negative = h.session().player.palette.timer & 0x8000 != 0;
        if h.session().player.palette.timer == 0 || negative {
            h.session_mut().player.palette.timer = 4;
            let base = suit_table(h, 0x91daa9);
            if h.session().player.palette_frame > 6 {
                h.session_mut().player.palette_frame = 6;
            }
            let address = base.wrapping_add(h.session().player.palette_frame);
            let source = h.record(Source::Native, 0x91, address).native_word(0);
            copy_suit(h, PaletteBuffer::Current, source);
            let frame = h.session().player.palette_frame;
            h.session_mut().player.palette_frame =
                if frame >= 6 { 6 } else { frame.wrapping_add(2) };
        }
    }
    PaletteCarry::Set
}
pub(crate) fn shine(h: &mut impl Host) -> PaletteCarry {
    let mut timer = h.session().player.shine_timer;
    if h.session().player.shine_timer == 170 {
        let saved = h.session().player.shine_timer;
        h.sound(3, 9, 0xc);
        timer = saved;
    }
    h.session_mut().player.shine_timer = timer.wrapping_sub(1);
    if (timer as i16).wrapping_sub(1) <= 0 {
        h.session_mut().player.palette_frame = 0;
        h.session_mut().player.shine_tick = 0;
        PaletteCarry::Clear
    } else {
        frame_palette(h, 0x91db10);
        let frame = h.session().player.palette_frame;
        h.session_mut().player.palette_frame = if frame.wrapping_sub(10) & 0x8000 == 0 {
            0
        } else {
            frame.wrapping_add(2)
        };
        PaletteCarry::Set
    }
}
pub(crate) fn shinespark(h: &mut impl Host) -> PaletteCarry {
    h.session_mut().player.shine_timer = h.session().player.shine_timer.wrapping_sub(1);
    let negative = h.session().player.shine_timer & 0x8000 != 0;
    if h.session().player.shine_timer == 0 || negative {
        h.session_mut().player.shine_tick = 0;
        h.session_mut().player.palette_frame = 0;
        PaletteCarry::Clear
    } else {
        frame_palette(h, 0x91db75);
        let frame = h.session().player.palette_frame;
        h.session_mut().player.palette_frame = if frame.wrapping_sub(6) & 0x8000 == 0 {
            0
        } else {
            frame.wrapping_add(2)
        };
        PaletteCarry::Set
    }
}
pub(crate) fn crystal_flash(h: &mut impl Host) -> PaletteCarry {
    if h.session().player.shine_timer & 0x8000 != 0 {
        let beam = h.session().inventory.beams.active & 0xfff;
        write_beam_palette(h, beam);
        h.session_mut().player.shine_tick = 0;
        h.session_mut().player.palette_frame = 0;
        h.session_mut().player.palette.timer = 0;
        h.session_mut().player.shine_timer = 0;
        PaletteCarry::Clear
    } else {
        h.session_mut().player.shine_timer = h.session().player.shine_timer.wrapping_sub(1);
        if (h.session().player.shine_timer as i16) <= 0 {
            h.session_mut().player.shine_timer = 5;
            let record = h.table(Source::Loaded, 0x91dc28);
            let source = record.native_word(usize::from(h.session().player.palette_frame >> 1) * 2);
            copy_crystal_last6(h, source);
            let frame = h.session().player.palette_frame;
            h.session_mut().player.palette_frame = if frame.wrapping_sub(10) & 0x8000 == 0 {
                0
            } else {
                frame.wrapping_add(2)
            };
        }
        h.session_mut().player.palette.crystal_flash_delay = h
            .session()
            .player
            .palette
            .crystal_flash_delay
            .wrapping_sub(1);
        let negative = h.session().player.palette.crystal_flash_delay & 0x8000 != 0;
        if h.session().player.palette.crystal_flash_delay == 0 || negative {
            let record = h.table(Source::Native, 0x91dc00);
            let delay = record.native_word(2 + usize::from(h.session().player.palette.timer));
            h.session_mut().player.palette.crystal_flash_delay =
                u16::from_le_bytes(delay.to_ne_bytes());
            let record = h.table(Source::Native, 0x91dc00);
            let source = record.native_word(usize::from(h.session().player.palette.timer));
            copy_crystal_first10(h, source);
            let frame = h.session().player.palette.timer;
            h.session_mut().player.palette.timer = if frame.wrapping_sub(36) & 0x8000 == 0 {
                0
            } else {
                frame.wrapping_add(4)
            };
        }
        PaletteCarry::Set
    }
}
fn xray_color(h: &mut impl Host) {
    let record = h.table(Source::Loaded, 0x9ba3c0);
    let value = record.native_word(usize::from(h.session().player.palette_frame >> 1) * 2);
    h.session_mut().visual.colors[196] = value;
}
pub(crate) fn xray(h: &mut impl Host) -> PaletteCarry {
    if h.session().player.palette.xray_progress & 0x8000 != 0 {
        h.session_mut().player.shine_tick = 0;
        h.session_mut().player.palette_frame = 0;
        h.session_mut().player.palette.timer = 0;
        h.session_mut().player.palette.xray_progress = 0;
        return PaletteCarry::Clear;
    }
    if h.session().player.palette.xray_progress == 0 {
        if h.inputs().demo_input_pre_action.wrapping_sub(2) & 0x8000 != 0 {
            h.session_mut().player.palette.timer = h.session().player.palette.timer.wrapping_sub(1);
            let negative = h.session().player.palette.timer & 0x8000 != 0;
            if h.session().player.palette.timer == 0 || negative {
                h.session_mut().player.palette.timer = 5;
                xray_color(h);
                if h.session().player.palette_frame.wrapping_sub(4) & 0x8000 != 0 {
                    h.session_mut().player.palette_frame =
                        h.session().player.palette_frame.wrapping_add(2);
                }
            }
            return PaletteCarry::Set;
        }
        h.session_mut().player.palette_frame = 6;
        h.session_mut().player.palette.timer = 1;
        h.session_mut().player.palette.xray_progress = 1;
    }
    h.session_mut().player.palette.timer = h.session().player.palette.timer.wrapping_sub(1);
    let negative = h.session().player.palette.timer & 0x8000 != 0;
    if h.session().player.palette.timer != 0 && !negative {
        return PaletteCarry::Set;
    }
    h.session_mut().player.palette.timer = 5;
    xray_color(h);
    let frame = h.session().player.palette_frame;
    h.session_mut().player.palette_frame = if frame.wrapping_sub(10) & 0x8000 == 0 {
        6
    } else {
        frame.wrapping_add(2)
    };
    PaletteCarry::Set
}
pub(crate) fn cancel_speed_boost(h: &mut impl Host) {
    if h.session().player.contact.momentum != 0 {
        h.session_mut().player.contact.momentum = 0;
        h.session_mut().player.contact.speed_boost = 0;
        h.session_mut().player.palette_frame = 0;
        h.session_mut().player.palette.timer = 0;
        load_suit(h, PaletteBuffer::Current);
    }
    if h.session().projectiles.echoes.index & 0x8000 == 0 {
        h.session_mut().projectiles.echoes.index = 0xffff;
        if h.session().player.movement_and_facing as u8 == 4 {
            h.session_mut().projectiles.echoes.x_speed[0] = (-8i16) as u16;
            h.session_mut().projectiles.echoes.x_speed[1] = (-8i16) as u16;
        } else {
            h.session_mut().projectiles.echoes.x_speed[0] = 8;
            h.session_mut().projectiles.echoes.x_speed[1] = 8;
        }
    }
}
/// One loop serves byte-offset Y callers and masked beam-bitfield A/Load callers.
/// Each color reacquires native90 after wrapping the logical bank address.
pub(crate) fn write_beam_palette(h: &mut impl Host, table_index: u16) {
    let record = h.table(Source::Loaded, 0x90c3c9);
    let source = record.native_word(usize::from(table_index) * 2);
    for offset in (0u16..32).step_by(2) {
        let color = h
            .record(Source::Native, 0x90, source.wrapping_add(offset))
            .native_word(0);
        h.session_mut().visual.colors[224 + usize::from(offset >> 1)] =
            u16::from_le_bytes(color.to_ne_bytes());
    }
}
pub(crate) fn apply_charge_output(h: &mut impl Host, output: Output) -> u8 {
    let source = match output {
        Output::Hyper(phase) => {
            let record = h.table(Source::Loaded, 0x91d829);
            record.native_word(usize::from(phase) * 2)
        }
        Output::Charge {
            contact_attack,
            suit,
            phase,
        } => {
            let record = h.table(
                Source::Loaded,
                if contact_attack { 0x91d7ff } else { 0x91d7d5 },
            );
            let base = record.native_word(usize::from(suit >> 1) * 2);
            // This dynamic lookup is loaded, unlike frame_palette's native lookup.
            h.record(Source::Loaded, 0x91, base.wrapping_add(phase))
                .native_word(0)
        }
        Output::Highlight => {
            for index in (193..208).rev() {
                h.session_mut().visual.colors[index] = 1023;
            }
            return 0;
        }
        Output::Visor => return visor(h),
    };
    copy_suit(h, PaletteBuffer::Current, source);
    0
}
