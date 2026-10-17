//! Norfair and Lower Norfair Rio state machines, pinned MIT snesrev/sm bank A2.
//! Base records, extended variables and paired drawing/spawn overlays alias WRAM.
use crate::{Wram, audio, enemy, enemy_ai};
pub use enemy_ai::Host;
const BASE: usize = 0xf78;
const CURRENT: usize = 0xe54;
const DRAW: usize = 0xe84;
const EXTENDED: usize = 0x7800;
const SPAWN: usize = 0x7000;
const RECORD_SIZE: usize = 64;
const STATE: usize = 0x3a;
const PROPERTIES: usize = 0xe;
const FROZEN: usize = 0x26;
const X: usize = 2;
const Y: usize = 6;
const PARAMETER: usize = 0x3c;
const ANIMATION: usize = 0x1a;
const ANIMATION_TIMER: usize = 0x1c;
const TIMER: usize = 0x18;
const SAMUS_X: usize = 0xaf6;
const RANDOM: usize = 0x5e5;
fn record(k: u16) -> Result<usize, &'static str> {
    if k >= 0x800 || k & 63 != 0 {
        return Err("invalid Rio record");
    }
    Ok(BASE + usize::from(k))
}
fn extra(k: u16) -> usize {
    EXTENDED + usize::from(k)
}
fn rom_word(rom: &[u8], address: u32) -> Result<u16, &'static str> {
    let offset = (((address >> 16) & 127) * 32768 + (address & 32767)) as usize;
    let bytes = rom.get(offset..offset + 2).ok_or("missing Rio ROM table")?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}
fn set(r: &mut Wram<'_>, e: usize, field: usize, v: u16) {
    r.set_word(e + field, v);
}
fn animate(r: &mut Wram<'_>, address: u16) -> Result<(), &'static str> {
    let k = r.word(CURRENT);
    let e = record(k)?;
    let x = extra(k);
    if r.word(x) != address {
        r.set_word(x, address);
        set(r, e, ANIMATION, address);
        set(r, e, ANIMATION_TIMER, 1);
        set(r, e, TIMER, 0);
    }
    Ok(())
}
fn move_axis(
    r: &mut Wram<'_>,
    rom: &[u8],
    host: &mut impl Host,
    k: u16,
    velocity: u16,
    horizontal: bool,
) -> Result<bool, &'static str> {
    Ok(enemy_ai::dispatch_host(
        r,
        rom,
        host,
        if horizontal { "Enemy_MoveRight_IgnoreSlopes" } else { "Enemy_MoveDown" },
        i32::from(k),
        i32::from(velocity as i16) * 256,
    )? != 0)
}
fn paired_child(r: &mut Wram<'_>, k: u16, lower: bool) -> Result<(), &'static str> {
    let e = record(k)?;
    // C's [31] is 31 packed 64-byte SpawnData records, not an enemy ordinal.
    let child = SPAWN + usize::from(k) + 31 * RECORD_SIZE;
    let draw = DRAW + usize::from(k);
    if child + RECORD_SIZE > 0x8800 || draw + 220 > 0x10000 {
        return Err("Rio child overlay crosses combined spawn/extended allocation");
    }
    if r.word(draw + 200) == 0 {
        set(r, e, PROPERTIES, r.word(e + PROPERTIES) | 0x200);
        return Ok(());
    }
    let frozen = r.word(draw + 218);
    set(r, e, FROZEN, frozen);
    let offset = r.word(child + 4);
    if lower {
        if frozen == 0 && offset != 0 {
            set(r, e, PROPERTIES, r.word(e + PROPERTIES) & !0x100);
            set(r, e, X, r.word(draw + 182));
            set(r, e, Y, r.word(draw + 186).wrapping_add(12));
        } else {
            set(r, e, PROPERTIES, r.word(e + PROPERTIES) | 0x100);
        }
    } else {
        if frozen != 0 {
            set(r, e, PROPERTIES, r.word(e + PROPERTIES) | 0x100);
        } else {
            set(r, e, PROPERTIES, r.word(e + PROPERTIES) & !0x100);
            if r.word(child) == 0xc0f1 {
                set(r, e, PROPERTIES, r.word(e + PROPERTIES) | 0x100);
            } else {
                animate(r, if offset & 0x8000 != 0 { 0xc1a3 } else { 0xc18f })?;
                set(r, e, PROPERTIES, r.word(e + PROPERTIES) & !0x100);
                set(r, e, X, r.word(draw + 182));
                set(r, e, Y, offset.wrapping_add(r.word(draw + 186)));
            }
        }
    }
    Ok(())
}
fn state(r: &mut Wram<'_>, rom: &[u8], host: &mut impl Host, k: u16, lower: bool, n: u8) -> Result<(), &'static str> {
    let e = record(k)?;
    let x = extra(k);
    let flag = r.word(x + 2);
    let vertical = if lower { 0x34 } else { 0x30 };
    let horizontal = if lower { 0x36 } else { 0x32 };
    let states = if lower {
        [0xc72e, 0xc771, 0xc7bb, 0xc7d6, 0xc82d, 0xc888]
    } else {
        [0xc281, 0xc2e7, 0xc33f, 0xc361, 0xc3b1, 0xc406]
    };
    match n {
        1 => paired_child(r, k, lower)?,
        2 => {
            if r.word(RANDOM) & 0x101 != 0
                && enemy::dispatch_rom(r, rom, "CompareDistToSamus_X", i32::from(k), if lower { 112 } else { 192 })? & 1
                    == 0
            {
                let v =
                    rom_word(rom, if lower { 0xa2c6ca } else { 0xa2c1c1 + u32::from((r.word(RANDOM) >> 2) & 1) * 2 })?;
                let mut h = rom_word(rom, if lower { 0xa2c6ce } else { 0xa2c1c5 })?;
                if r.word(SAMUS_X).wrapping_sub(r.word(e + X)) & 0x8000 != 0 {
                    h = h.wrapping_neg();
                }
                set(r, e, vertical, v);
                set(r, e, horizontal, h);
                animate(r, if lower { 0xc630 } else { 0xc107 })?;
                set(r, e, STATE, states[2]);
            } else if lower || flag != 0 {
                r.set_word(x + 2, 0);
                animate(r, if lower { 0xc61a } else { 0xc0f1 })?;
            }
        }
        3 => {
            if flag != 0 {
                r.set_word(x + 2, 0);
                animate(r, if lower { 0xc65a } else { 0xc12f })?;
                set(r, e, STATE, states[3]);
                if !lower {
                    audio::dispatch(r, "QueueSfx2_Max6", 0x65, 0)?;
                }
            }
        }
        4 => {
            if move_axis(r, rom, host, k, r.word(e + horizontal), true)? {
                set(r, e, horizontal, r.word(e + horizontal).wrapping_neg());
            }
            let hit = move_axis(r, rom, host, k, r.word(e + vertical), false)?;
            let v = if hit {
                r.word(e + vertical)
            } else {
                let v = r.word(e + vertical).wrapping_sub(32);
                set(r, e, vertical, v);
                v
            };
            if hit || v & 0x8000 != 0 {
                set(r, e, vertical, 65535);
                animate(r, if lower { 0xc662 } else { 0xc145 })?;
                set(r, e, STATE, states[4]);
                if lower {
                    audio::dispatch(r, "QueueSfx2_Max6", 0x64, 0)?;
                }
            }
        }
        5 => {
            if move_axis(r, rom, host, k, r.word(e + horizontal), true)? {
                set(r, e, horizontal, r.word(e + horizontal).wrapping_neg());
            }
            if move_axis(r, rom, host, k, r.word(e + vertical), false)? {
                if lower {
                    animate(r, 0xc686)?;
                }
                set(r, e, STATE, states[5]);
            } else {
                set(r, e, vertical, r.word(e + vertical).wrapping_sub(32));
                if flag != 0 {
                    r.set_word(x + 2, 0);
                    animate(r, if lower { 0xc674 } else { 0xc179 })?;
                }
            }
        }
        6 => {
            if !lower || flag != 0 {
                if lower {
                    r.set_word(x + 2, 0);
                    animate(r, 0xc686)?;
                }
                set(r, e, STATE, states[1]);
            }
        }
        _ => return Err("unknown Rio state number"),
    }
    Ok(())
}
fn next_random(r: &mut Wram<'_>) {
    let n = r.word(RANDOM);
    let low = u16::from(n as u8) * 5;
    let high = (u16::from((n >> 8) as u8) * 5) as u8;
    let carry = (low >> 8) + u16::from(high) + 1;
    r.set_word(RANDOM, ((low & 255) | u16::from(carry as u8) << 8).wrapping_add(carry >> 8).wrapping_add(17));
}
pub fn dispatch_host(
    r: &mut Wram<'_>,
    rom: &[u8],
    host: &mut impl Host,
    symbol: &str,
    a: i32,
    _b: i32,
) -> Result<u32, &'static str> {
    let lower = symbol.starts_with("LowerNorfairRio_");
    let suffix =
        symbol.strip_prefix(if lower { "LowerNorfairRio_" } else { "NorfairRio_" }).ok_or("unknown Rio routine")?;
    let k = r.word(CURRENT);
    let e = record(k)?;
    match suffix {
        "Init" => {
            r.set_word(extra(k) + 2, 0);
            if !lower {
                r.set_word(extra(k) + 4, 0);
            }
            let child = r.word(e + PARAMETER) & 0x8000 != 0;
            let instruction = if lower {
                if child { 0xc6b0 } else { 0xc61a }
            } else if child {
                0xc18f
            } else {
                0xc0f1
            };
            r.set_word(extra(k), instruction);
            set(r, e, ANIMATION, instruction);
            set(
                r,
                e,
                STATE,
                if lower {
                    if child { 0xc72e } else { 0xc771 }
                } else if child {
                    0xc281
                } else {
                    0xc2e7
                },
            );
        }
        "Main" => {
            let states = if lower {
                [0xc72e, 0xc771, 0xc7bb, 0xc7d6, 0xc82d, 0xc888]
            } else {
                [0xc281, 0xc2e7, 0xc33f, 0xc361, 0xc3b1, 0xc406]
            };
            let n = states.iter().position(|s| *s == r.word(e + STATE)).ok_or("unsupported Rio preinstruction")?;
            next_random(r);
            state(r, rom, host, k, lower, n as u8 + 1)?;
        }
        "Func_7" => animate(r, u16::try_from(a).map_err(|_| "invalid Rio animation address")?)?,
        _ => {
            let n = suffix.strip_prefix("Func_").and_then(|s| s.parse::<u8>().ok()).ok_or("unknown Rio routine")?;
            state(r, rom, host, u16::try_from(a).map_err(|_| "invalid Rio argument")?, lower, n)?;
        }
    }
    Ok(0)
}
