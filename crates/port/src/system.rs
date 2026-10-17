//! System state routines translated from pinned snesrev/sm `src/sm_80.c`.
//! MIT license and upstream copyright: crates/decomp/LICENSE.upstream.txt.
//! No hardware ownership: these routines operate only on canonical WRAM.

use crate::Wram;

const LAST_KEYS: usize = 0x8b;
const NEW_KEYS: usize = 0x8f;
const RELEASED_KEYS: usize = 0x5d9;
const HELD_TIMER: usize = 0x5db;
const HELD_TIMER_RESET: usize = 0x5dd;
const HELD_INPUT: usize = 0x5df;
const NEW_HELD_INPUT: usize = 0x5e1;
const PREVIOUS_HELD_INPUT: usize = 0x5e3;
const BIT_MASK: usize = 0x5e7;
const EVENTS: usize = 0xd820;
const AREA_INDEX: usize = 0x79f;
const BOSS_BITS: usize = 0xd828;
const SCREEN_FADE_DELAY: usize = 0x723;
const SCREEN_FADE_COUNTER: usize = 0x725;
const INIDISP: usize = 0x51;
const OAM_ENTRIES: usize = 0x370;
const OAM_EXT: usize = 0x570;
const OAM_NEXT: usize = 0x590;
const MAP_3000: usize = 0x3000;
const MAP_4000: usize = 0x4000;
const MAP_6000: usize = 0x6000;

fn fill_map(ram: &mut Wram<'_>, base: usize, value: u16) {
    for offset in (base..base + 0x800).step_by(2) {
        ram.set_word(offset, value);
    }
}

fn handle_fade(ram: &mut Wram<'_>, fade_in: bool) {
    let remaining = ram.word(SCREEN_FADE_COUNTER).wrapping_sub(1);
    // C converts the subtraction to int16 *before* comparing, including 0x8000.
    if (remaining as i16) < 0 {
        ram.set_word(SCREEN_FADE_COUNTER, ram.word(SCREEN_FADE_DELAY));
        let brightness = ram.byte(INIDISP);
        if fade_in {
            let next = brightness.wrapping_add(1) & 0xf;
            if next != 0 {
                ram.set_byte(INIDISP, next);
            }
        } else {
            match brightness & 0xf {
                0 => {}
                1 => ram.set_byte(INIDISP, 0x80),
                level => ram.set_byte(INIDISP, level - 1),
            }
        }
    } else {
        ram.set_word(SCREEN_FADE_COUNTER, remaining);
    }
}

fn prepare_bit_access(ram: &mut Wram<'_>, bit: u16) -> usize {
    ram.set_word(BIT_MASK, 1u16 << (bit & 7));
    usize::from(bit >> 3)
}

fn release_buttons_filter(ram: &mut Wram<'_>, reset: u16) {
    ram.set_word(HELD_TIMER_RESET, reset);
    let released = !ram.word(NEW_KEYS) & ram.word(LAST_KEYS);
    let unchanged = released == ram.word(RELEASED_KEYS);
    ram.set_word(RELEASED_KEYS, released);
    if !unchanged {
        ram.set_word(HELD_TIMER, reset);
        ram.set_word(HELD_INPUT, 0);
    } else {
        let remaining = ram.word(HELD_TIMER).wrapping_sub(1);
        ram.set_word(HELD_TIMER, remaining);
        if remaining & 0x8000 == 0 {
            ram.set_word(HELD_INPUT, 0);
        } else {
            ram.set_word(HELD_TIMER, 0);
            ram.set_word(PREVIOUS_HELD_INPUT, ram.word(HELD_INPUT));
            ram.set_word(HELD_INPUT, released);
        }
    }
    let held = ram.word(HELD_INPUT);
    ram.set_word(NEW_HELD_INPUT, held & (ram.word(PREVIOUS_HELD_INPUT) ^ held));
}

pub fn dispatch(ram: &mut Wram<'_>, symbol: &str, a: u16, b: u16) -> Result<u32, &'static str> {
    match symbol {
        "ReleaseButtonsFilter" => {
            release_buttons_filter(ram, a);
            Ok(0)
        }
        "PrepareBitAccess" => Ok(prepare_bit_access(ram, a) as u32),
        // The pinned C deliberately does not update removed multiplication scratch.
        "Multiply16x16" => Ok(u32::from(a) * u32::from(b)),
        "HandleFadeIn" => {
            handle_fade(ram, true);
            Ok(0)
        }
        "HandleFadeOut" => {
            handle_fade(ram, false);
            Ok(0)
        }
        "ClearUnusedOam" => {
            // Oversized cursors are a defined no-op loop in C, then reset.
            for sprite in usize::from(ram.word(OAM_NEXT) >> 2)..128 {
                ram.set_byte(OAM_ENTRIES + sprite * 4 + 1, 0xf0);
            }
            ram.set_word(OAM_NEXT, 0);
            Ok(0)
        }
        "ClearOamExt" => {
            for offset in OAM_EXT..OAM_EXT + 32 {
                ram.set_byte(offset, 0);
            }
            Ok(0)
        }
        "sub_8088EB" => {
            fill_map(ram, MAP_3000, a);
            Ok(0)
        }
        "sub_8088FE" => {
            fill_map(ram, MAP_4000, a);
            Ok(0)
        }
        "sub_808911" => {
            fill_map(ram, MAP_6000, a);
            Ok(0)
        }
        "WriteLotsOf0x1c2f" => {
            for base in [MAP_3000, MAP_4000, MAP_6000] {
                fill_map(ram, base, 0x1c2f);
            }
            Ok(0)
        }
        "SetBossBitForCurArea" | "ClearBossBitForCurArea" | "CheckBossBitForCurArea" => {
            let area = usize::from(ram.word(AREA_INDEX));
            if area >= 8 {
                return Err("area index must be below 8");
            }
            let value = ram.byte(BOSS_BITS + area);
            match symbol {
                "SetBossBitForCurArea" => {
                    ram.set_byte(BOSS_BITS + area, value | a as u8);
                    Ok(0)
                }
                "ClearBossBitForCurArea" => {
                    ram.set_byte(BOSS_BITS + area, value & !(a as u8));
                    Ok(0)
                }
                _ => Ok(u32::from(u16::from(value) & a != 0)),
            }
        }
        "SetEventHappened" | "ClearEventHappened" | "CheckEventHappened" => {
            // Eight event bytes precede the distinct boss state array.
            if a >= 64 {
                return Err("event index must be below 64");
            }
            let offset = EVENTS + prepare_bit_access(ram, a);
            let mask = ram.word(BIT_MASK) as u8;
            let value = ram.byte(offset);
            match symbol {
                "SetEventHappened" => {
                    ram.set_byte(offset, value | mask);
                    Ok(0)
                }
                "ClearEventHappened" => {
                    ram.set_byte(offset, value & !mask);
                    Ok(0)
                }
                _ => Ok(u32::from(value & mask != 0)),
            }
        }
        _ => Err("unknown system routine"),
    }
}
