//! Samus palette phases share the actual visual, player and inventory owner.
use crate::player_transitions::state::FrameBetaMode;
use crate::{
    imported::ImportedWords,
    liquid_surface::{self, LiquidAtHeight},
    records::Records,
    session::Session,
    visual_host::{Host, Record, Source},
    visual_state::{PaletteBuffer, PaletteCarry},
};
use sm_data::cartridge::RomImage;
use sm_data::samus_palettes::*;

/// Samus's palette pointer tables, imported from the loaded image.
#[derive(Default)]
pub struct ImportedPaletteTables {
    suits: ImportedWords<SUIT_PALETTES, SUITS>,
    charge_sequences: ImportedWords<CHARGE_SEQUENCES, SUITS>,
    screw_attack: ImportedWords<SCREW_ATTACK_FRAMES, SUITS>,
    speed_boost: ImportedWords<SPEED_BOOST_FRAMES, SUITS>,
    shine: ImportedWords<SHINE_FRAMES, SUITS>,
    shinespark: ImportedWords<SHINESPARK_FRAMES, SUITS>,
    visor: ImportedWords<VISOR_COLOURS, VISOR_FRAMES>,
    charge: ImportedWords<CHARGE_PALETTES, CHARGE_STEPS>,
    crystal: ImportedWords<CRYSTAL_LAST_COLOURS, CRYSTAL_FRAMES>,
    glow: ImportedWords<GLOW_PALETTES, GLOW_PHASES>,
    beams: ImportedWords<BEAM_PALETTES, BEAM_COMBINATIONS>,
}

pub fn import(image: &RomImage) -> ImportedPaletteTables {
    ImportedPaletteTables {
        suits: ImportedWords::import(image),
        charge_sequences: ImportedWords::import(image),
        screw_attack: ImportedWords::import(image),
        speed_boost: ImportedWords::import(image),
        shine: ImportedWords::import(image),
        shinespark: ImportedWords::import(image),
        visor: ImportedWords::import(image),
        charge: ImportedWords::import(image),
        crystal: ImportedWords::import(image),
        glow: ImportedWords::import(image),
        beams: ImportedWords::import(image),
    }
}

/// Entry `index` of the selected table; indices past it keep the original load.
fn entry<const ADDRESS: u32, const ROWS: usize>(
    h: &(impl Records + ?Sized),
    table: impl Fn(&ImportedPaletteTables) -> &ImportedWords<ADDRESS, ROWS>,
    index: usize,
) -> u16 {
    table(&h.records().samus_palettes).word(h.loaded_image(), index as isize)
}

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
pub fn update_suit_index(session: &mut Session) {
    session.presentation.palette_mut().suit_index = match suit_source(session) {
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
    // Bank $9B's $6000-$7FFF window is unmapped. The copy's LDA $00xx,X then
    // reads the open bus, which still holds the operand's high byte, $00.
    let bytes =
        if (0x6000..0x8000).contains(&usize::from(source)) && usize::from(source) + N <= 0x8000 {
            [0; N]
        } else {
            h.record(Source::Native, 0x9b, source).bytes::<N>(0)
        };
    fn colors(h: &mut impl Host, buffer: PaletteBuffer) -> &mut [u16] {
        match buffer {
            PaletteBuffer::Current => &mut h.session_mut().presentation.visual_mut().colors,
            PaletteBuffer::Target => &mut h.session_mut().presentation.visual_mut().targets,
        }
    }
    for (i, pair) in bytes.as_chunks::<2>().0.iter().enumerate() {
        #[cfg(sm_verify_dll)]
        let observed_before = if h.palette_publications_observed() {
            colors(h, buffer)[first + i]
        } else {
            0
        };
        colors(h, buffer)[first + i] = u16::from_le_bytes([pair[0], pair[1]]);
        #[cfg(sm_verify_dll)]
        {
            let after = colors(h, buffer)[first + i];
            h.palette_publication(
                11,
                if buffer == PaletteBuffer::Current {
                    0xc000
                } else {
                    0xc200
                } + 2 * (first + i),
                observed_before,
                after,
            );
        }
    }
}
pub fn copy_suit(h: &mut impl Host, buffer: PaletteBuffer, source: u16) {
    #[cfg(sm_verify_dll)]
    if buffer == PaletteBuffer::Current {
        h.samus_palette_event("copy", source);
        h.probe("transitions.suit.copy");
    }
    copy_colors::<32>(h, buffer, 192, source);
}
pub fn copy_crystal_first10(h: &mut impl Host, source: u16) {
    copy_colors::<20>(h, PaletteBuffer::Current, 224, source);
}
pub fn copy_crystal_last6(h: &mut impl Host, source: u16) {
    copy_colors::<12>(h, PaletteBuffer::Current, 234, source);
}
pub fn load_suit(h: &mut impl Host, buffer: PaletteBuffer) {
    #[cfg(sm_verify_dll)]
    if buffer == PaletteBuffer::Current {
        h.samus_palette_event("load", 0);
        h.probe("transitions.suit.load");
    }
    let source = suit_source(h.session());
    copy_suit(h, buffer, source);
}
fn suit_table<const ADDRESS: u32>(
    h: &impl Host,
    table: impl Fn(&ImportedPaletteTables) -> &ImportedWords<ADDRESS, SUITS>,
) -> u16 {
    entry(
        h,
        table,
        usize::from(h.session().presentation.palette().suit_index >> 1),
    )
}
fn frame_palette<const ADDRESS: u32>(
    h: &mut impl Host,
    table: impl Fn(&ImportedPaletteTables) -> &ImportedWords<ADDRESS, SUITS>,
) {
    let base = suit_table(h, table);
    let address = base.wrapping_add(h.session().player.palette_frame);
    let source = h.record(Source::Native, 0x91, address).native_word(0);
    copy_suit(h, PaletteBuffer::Current, source);
}
pub fn advance(h: &mut impl Host) {
    h.probe("samus.palette.entry");
    if h.session().player.special_palette_flags & 0x8000 == 0 {
        let restore = if charge_feedback(h) & 1 != 0 {
            true
        } else {
            #[cfg(sm_verify_dll)]
            h.samus_palette_inject("samus.palette.dispatch");
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
                    sm_data::table::check_index(usize::from(index), 11);
                    unreachable!()
                }
            };
            phase == PaletteCarry::Clear
        };
        if restore {
            let source = suit_table(h, |t| &t.suits);
            copy_suit(h, PaletteBuffer::Current, source);
        }
    }
    h.probe("samus.palette.tail");
    misc(h);
    h.probe("samus.palette.return");
}
pub fn visor(h: &mut impl Host) -> u8 {
    if h.session().player.shine_tick == 8 {
        return 0;
    }
    if matches!(h.inputs().blending_config, 40 | 42) {
        let timer = h
            .session()
            .presentation
            .palette()
            .visor_timer
            .wrapping_sub(1);
        h.session_mut().presentation.palette_mut().visor_timer = timer;
        if timer as u8 != 0 {
            return 0;
        }
        h.session_mut().presentation.palette_mut().visor_timer = timer | 5;
        let value = entry(h, |t| &t.visor, usize::from((timer >> 8) >> 1));
        h.session_mut().presentation.visual_mut().colors[196] = value;
        let frame = (timer >> 8) + 2;
        let high = if frame.wrapping_sub(12) & 0x8000 != 0 {
            frame.swap_bytes()
        } else {
            0x600
        };
        h.session_mut().presentation.palette_mut().visor_timer =
            high | (h.session().presentation.palette().visor_timer & 0xff);
    } else {
        h.session_mut().presentation.palette_mut().visor_timer = 1537;
    }
    0
}
/// Carry is the original phase result observed at91D715, independently checked
/// against the oracle. The unconditional final misc call discards this result.
pub fn misc(h: &mut impl Host) -> PaletteCarry {
    #[cfg(sm_verify_dll)]
    h.samus_palette_event("misc", 0);
    if h.session().player.special_palette_flags == 0 {
        let hurt = h.session().presentation.palette().hurt_flash;
        if h.session().presentation.palette().hurt_flash == 0 {
            return PaletteCarry::Clear;
        }
        let restore = if h.session().presentation.palette().hurt_flash == 2
            && h.inputs().cinematic == 0
            && (h.session().player.transitions.frame_beta != FrameBetaMode::Drained
                || h.session().player.pose != 0x54)
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
        let value = h
            .session()
            .presentation
            .palette()
            .hurt_flash
            .wrapping_add(1);
        h.session_mut().presentation.palette_mut().hurt_flash = value;
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
            h.session_mut().presentation.palette_mut().hurt_flash = 0;
        }
        return carry(value >= 60);
    }
    if h.session().player.special_palette_flags & 0x8000 != 0 {
        let shift_carry = h.session().projectiles.controls.charge_palette & 0x8000 != 0;
        let source = entry(
            h,
            |t| &t.charge,
            usize::from(h.session().projectiles.controls.charge_palette),
        );
        copy_suit(h, PaletteBuffer::Current, source);
        h.session_mut().presentation.palette_mut().timer =
            h.session().presentation.palette().timer.wrapping_sub(1);
        let timer = h.session().presentation.palette().timer;
        if timer != 0 && timer & 0x8000 == 0 {
            return carry(shift_carry);
        }
        h.session_mut().presentation.palette_mut().timer = h.session().player.palette_frame;
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
            let source = suit_table(h, |t| &t.charge_sequences);
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
pub fn screw_speed(h: &mut impl Host) -> PaletteCarry {
    if h.session().presentation.palette().suit_index & 4 == 0 {
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
        } else if h.session().player.animation_clock.frame != 0 {
            if h.session().player.animation_clock.frame.wrapping_sub(27) & 0x8000 == 0 {
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
        if h.session().player.animation_clock.frame.wrapping_sub(3) & 0x8000 == 0 {
            true
        } else {
            h.session_mut().player.palette_frame = 0;
            return PaletteCarry::Set;
        }
    } else {
        false
    };
    if screw {
        frame_palette(h, |t| &t.screw_attack);
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
        h.session_mut().presentation.palette_mut().timer =
            h.session().presentation.palette().timer.wrapping_sub(1);
        let negative = h.session().presentation.palette().timer & 0x8000 != 0;
        if h.session().presentation.palette().timer == 0 || negative {
            h.session_mut().presentation.palette_mut().timer = 4;
            // A frame past 6, left by the screw attack, indexes past the suit's
            // four pointers, so the original shows whatever palette follows.
            frame_palette(h, |t| &t.speed_boost);
            let next = h.session().player.palette_frame.wrapping_add(2);
            h.session_mut().player.palette_frame = if next.wrapping_sub(8) & 0x8000 != 0 {
                next
            } else {
                6
            };
        }
    }
    PaletteCarry::Set
}
pub fn shine(h: &mut impl Host) -> PaletteCarry {
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
        frame_palette(h, |t| &t.shine);
        let frame = h.session().player.palette_frame;
        h.session_mut().player.palette_frame = if frame.wrapping_sub(10) & 0x8000 == 0 {
            0
        } else {
            frame.wrapping_add(2)
        };
        PaletteCarry::Set
    }
}
pub fn shinespark(h: &mut impl Host) -> PaletteCarry {
    h.session_mut().player.shine_timer = h.session().player.shine_timer.wrapping_sub(1);
    let negative = h.session().player.shine_timer & 0x8000 != 0;
    if h.session().player.shine_timer == 0 || negative {
        h.session_mut().player.shine_tick = 0;
        h.session_mut().player.palette_frame = 0;
        PaletteCarry::Clear
    } else {
        frame_palette(h, |t| &t.shinespark);
        let frame = h.session().player.palette_frame;
        h.session_mut().player.palette_frame = if frame.wrapping_sub(6) & 0x8000 == 0 {
            0
        } else {
            frame.wrapping_add(2)
        };
        PaletteCarry::Set
    }
}
pub fn crystal_flash(h: &mut impl Host) -> PaletteCarry {
    if h.session().player.shine_timer & 0x8000 != 0 {
        let beam = h.session().inventory.beams.active & 0xfff;
        write_beam_palette(h, beam);
        h.session_mut().player.shine_tick = 0;
        h.session_mut().player.palette_frame = 0;
        h.session_mut().presentation.palette_mut().timer = 0;
        h.session_mut().player.shine_timer = 0;
        PaletteCarry::Clear
    } else {
        h.session_mut().player.shine_timer = h.session().player.shine_timer.wrapping_sub(1);
        if (h.session().player.shine_timer as i16) <= 0 {
            h.session_mut().player.shine_timer = 5;
            let source = entry(
                h,
                |t| &t.crystal,
                usize::from(h.session().player.palette_frame >> 1),
            );
            copy_crystal_last6(h, source);
            let frame = h.session().player.palette_frame;
            h.session_mut().player.palette_frame = if frame.wrapping_sub(10) & 0x8000 == 0 {
                0
            } else {
                frame.wrapping_add(2)
            };
        }
        h.session_mut()
            .presentation
            .palette_mut()
            .crystal_flash_delay = h
            .session()
            .presentation
            .palette()
            .crystal_flash_delay
            .wrapping_sub(1);
        let negative = h.session().presentation.palette().crystal_flash_delay & 0x8000 != 0;
        if h.session().presentation.palette().crystal_flash_delay == 0 || negative {
            let record = h.table(Source::Native, 0x91dc00);
            let delay =
                record.native_word(2 + usize::from(h.session().presentation.palette().timer));
            h.session_mut()
                .presentation
                .palette_mut()
                .crystal_flash_delay = u16::from_le_bytes(delay.to_ne_bytes());
            let record = h.table(Source::Native, 0x91dc00);
            let source = record.native_word(usize::from(h.session().presentation.palette().timer));
            copy_crystal_first10(h, source);
            let frame = h.session().presentation.palette().timer;
            h.session_mut().presentation.palette_mut().timer =
                if frame.wrapping_sub(36) & 0x8000 == 0 {
                    0
                } else {
                    frame.wrapping_add(4)
                };
        }
        PaletteCarry::Set
    }
}
fn xray_color(h: &mut impl Host) {
    let value = entry(
        h,
        |t| &t.visor,
        usize::from(h.session().player.palette_frame >> 1),
    );
    h.session_mut().presentation.visual_mut().colors[196] = value;
}
pub fn xray(h: &mut impl Host) -> PaletteCarry {
    if h.session().presentation.palette().xray_progress & 0x8000 != 0 {
        h.session_mut().player.shine_tick = 0;
        h.session_mut().player.palette_frame = 0;
        h.session_mut().presentation.palette_mut().timer = 0;
        h.session_mut().presentation.palette_mut().xray_progress = 0;
        return PaletteCarry::Clear;
    }
    if h.session().presentation.palette().xray_progress == 0 {
        if h.session().player.demo.pre_action.wrapping_sub(2) & 0x8000 != 0 {
            h.session_mut().presentation.palette_mut().timer =
                h.session().presentation.palette().timer.wrapping_sub(1);
            let negative = h.session().presentation.palette().timer & 0x8000 != 0;
            if h.session().presentation.palette().timer == 0 || negative {
                h.session_mut().presentation.palette_mut().timer = 5;
                xray_color(h);
                if h.session().player.palette_frame.wrapping_sub(4) & 0x8000 != 0 {
                    h.session_mut().player.palette_frame =
                        h.session().player.palette_frame.wrapping_add(2);
                }
            }
            return PaletteCarry::Set;
        }
        h.session_mut().player.palette_frame = 6;
        h.session_mut().presentation.palette_mut().timer = 1;
        h.session_mut().presentation.palette_mut().xray_progress = 1;
    }
    h.session_mut().presentation.palette_mut().timer =
        h.session().presentation.palette().timer.wrapping_sub(1);
    let negative = h.session().presentation.palette().timer & 0x8000 != 0;
    if h.session().presentation.palette().timer != 0 && !negative {
        return PaletteCarry::Set;
    }
    h.session_mut().presentation.palette_mut().timer = 5;
    xray_color(h);
    let frame = h.session().player.palette_frame;
    h.session_mut().player.palette_frame = if frame.wrapping_sub(10) & 0x8000 == 0 {
        6
    } else {
        frame.wrapping_add(2)
    };
    PaletteCarry::Set
}
pub fn cancel_speed_boost(h: &mut impl Host) {
    #[cfg(sm_verify_dll)]
    h.observe_speed_boost_cancel([
        h.session().player.contact.base_run.whole,
        h.session().player.contact.extra_run.whole,
        h.session().player.contact.acceleration_mode,
    ]);
    if h.session().player.contact.momentum != 0 {
        h.session_mut().player.contact.momentum = 0;
        h.session_mut().player.contact.speed_boost = 0;
        h.session_mut().player.palette_frame = 0;
        h.session_mut().presentation.palette_mut().timer = 0;
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
pub fn write_beam_palette(h: &mut impl Host, table_index: u16) {
    let source = entry(h, |t| &t.beams, usize::from(table_index));
    for offset in (0u16..32).step_by(2) {
        let color = h
            .record(Source::Native, 0x90, source.wrapping_add(offset))
            .native_word(0);
        h.session_mut().presentation.visual_mut().colors[224 + usize::from(offset >> 1)] =
            u16::from_le_bytes(color.to_ne_bytes());
    }
}
/// Charge and glow colors share this owner with the later Samus palette phases.
pub fn charge_feedback(h: &mut impl Host) -> u8 {
    if h.session().projectiles.controls.glow != 0 {
        h.probe("samus.palette.charge.glow");
        if h.session().inventory.hyper_beam != 0 {
            if h.session().projectiles.controls.glow & 1 == 0 {
                if h.session().projectiles.controls.glow & 0x1e == 0 {
                    h.session_mut().projectiles.controls.glow = 0;
                    return 1;
                }
                let phase = (h.session().projectiles.controls.glow & 0x1e) >> 1;
                let source = entry(h, |t| &t.glow, usize::from(phase));
                copy_suit(h, PaletteBuffer::Current, source);
            }
            // Color lookup can change the owner; decrement its current glow.
            let glow = h.session().projectiles.controls.glow.wrapping_sub(1);
            h.session_mut().projectiles.controls.glow = glow;
            return 0;
        }
        let glow = h.session().projectiles.controls.glow.wrapping_sub(1);
        h.session_mut().projectiles.controls.glow = glow;
        if h.session().projectiles.controls.glow == 0 {
            return 1;
        }
        for index in (193..208).rev() {
            h.session_mut().presentation.visual_mut().colors[index] = 1023;
        }
        return 0;
    }
    if h.session().player.grapple == 0xc4f0
        && h.session().projectiles.controls.charge != 0
        && h.session().projectiles.controls.charge.wrapping_sub(60) & 0x8000 == 0
    {
        h.probe("samus.palette.charge.held");
        // Capture the output selectors before either content lookup can reenter.
        let contact_attack = h.session().player.contact.contact_attack == 4;
        let suit = h.session().presentation.palette().suit_index;
        let phase = h.session().projectiles.controls.charge_palette;
        let source = {
            let record = h.table(
                Source::Loaded,
                if contact_attack { 0x91d7ff } else { 0x91d7d5 },
            );
            let base = record.native_word(usize::from(suit >> 1) * 2);
            // Charge's dynamic lookup is loaded; frame_palette uses native records.
            h.record(Source::Loaded, 0x91, base.wrapping_add(phase))
                .native_word(0)
        };
        copy_suit(h, PaletteBuffer::Current, source);
        let phase = if h
            .session()
            .projectiles
            .controls
            .charge_palette
            .wrapping_sub(10)
            & 0x8000
            == 0
        {
            0
        } else {
            h.session()
                .projectiles
                .controls
                .charge_palette
                .wrapping_add(2)
        };
        h.session_mut().projectiles.controls.charge_palette = phase;
        0
    } else {
        h.session_mut().projectiles.controls.charge_palette = 0;
        visor(h)
    }
}
