//! Bank A3 crawlers: canonical enemy records and extended velocity overlays.
use crate::{Wram, enemy, enemy_ai};
pub use enemy_ai::Host;
fn record(k: u16) -> Result<usize, &'static str> {
    if k >= 0x800 || k & 63 != 0 {
        return Err("invalid crawler record");
    }
    Ok(0xf78 + usize::from(k))
}
fn offset(a: u32) -> usize {
    (((a >> 16) & 127) * 32768 + (a & 32767)) as usize
}
fn word(rom: &[u8], a: u32) -> Result<u16, &'static str> {
    let b = rom.get(offset(a)..offset(a) + 2).ok_or("missing crawler ROM table")?;
    Ok(u16::from_le_bytes([b[0], b[1]]))
}
fn move_axis(
    r: &mut Wram<'_>,
    rom: &[u8],
    host: &mut impl Host,
    k: u16,
    amount: i32,
    horizontal: bool,
    slopes: bool,
) -> Result<bool, &'static str> {
    Ok(enemy_ai::dispatch_host(
        r,
        rom,
        host,
        if horizontal {
            if slopes { "Enemy_MoveRight_ProcessSlopes" } else { "Enemy_MoveRight_IgnoreSlopes" }
        } else {
            "Enemy_MoveDown"
        },
        i32::from(k),
        amount,
    )? != 0)
}
fn speed(v: u16) -> i32 {
    i32::from(v as i16) * 256
}
fn probe_speed(v: u16) -> i32 {
    let a = speed(v);
    a + if a < 0 { -65536 } else { 65536 }
}
fn block(r: &mut Wram<'_>, x: u16, y: u16) -> Result<usize, &'static str> {
    enemy::dispatch_rom(r, &[], "CalculateBlockContainingPixelPos", i32::from(x), i32::from(y))?;
    let b = usize::from(r.word(0xdc4));
    if b >= 0x3200 {
        return Err("crawler terrain index exceeds level allocation");
    }
    Ok(b)
}
fn align(r: &mut Wram<'_>, rom: &[u8], k: u16) -> Result<u32, &'static str> {
    let e = record(k)?;
    let mut hit = 0;
    for top in [false, true] {
        let y = if top {
            r.word(e + 6).wrapping_sub(r.word(e + 12))
        } else {
            r.word(e + 6).wrapping_add(r.word(e + 12)).wrapping_sub(1)
        };
        let b = block(r, r.word(e + 2), y)?;
        let t = r.byte(0x16402 + b);
        if r.word(0x10002 + 2 * b) & 0xf000 == 0x1000 && t & 31 >= 5 {
            hit = 1;
            if (t & 128 != 0) == top {
                let x = if t & 64 != 0 { r.word(e + 2) ^ 15 } else { r.word(e + 2) };
                let sample = *rom
                    .get(offset(0x948b2b) + usize::from(t & 31) * 16 + usize::from(x & 15))
                    .ok_or("missing crawler slope table")?;
                let delta =
                    i16::from(sample & 31) - i16::try_from(if top { (y & 15) ^ 15 } else { y & 15 }).unwrap() - 1;
                if delta < 0 {
                    r.set_word(
                        e + 6,
                        if top {
                            r.word(e + 6).wrapping_sub(delta as u16)
                        } else {
                            r.word(e + 6).wrapping_add(delta as u16)
                        },
                    );
                }
            }
        }
    }
    Ok(hit)
}
fn slope_speed(r: &mut Wram<'_>, rom: &[u8]) -> Result<i32, &'static str> {
    let e = record(r.word(0xe54))?;
    let y = if r.word(e + 0x32) & 0x8000 != 0 {
        r.word(e + 6).wrapping_sub(r.word(e + 12))
    } else {
        r.word(e + 6).wrapping_add(r.word(e + 12)).wrapping_sub(1)
    };
    let b = block(r, r.word(e + 2), y)?;
    if r.word(0x10002 + 2 * b) & 0xf000 == 0x1000 && r.byte(0x16402 + b) & 31 >= 5 {
        let scale = i32::from(word(rom, 0xa3e931 + u32::from(r.byte(0x16402 + b) & 31) * 4)?);
        Ok(i32::from(r.word(e + 0x30) as i16) * scale)
    } else {
        Ok(speed(r.word(e + 0x30)))
    }
}
fn initialize(r: &mut Wram<'_>, rom: &[u8], k: u16, orange: bool, script: bool) -> Result<(), &'static str> {
    let e = record(k)?;
    let p = r.word(e + 0x3c);
    if (orange || p != 255) && p >= 32 {
        return Err("invalid crawler speed index");
    }
    if script {
        r.set_word(
            e + 0x1a,
            word(rom, if orange { 0xa3e03b } else { 0xa3e2cc } + u32::from(r.word(e + 0x1a) & 3) * 2)?,
        );
    }
    if !orange {
        r.set_word(e + 0x16, 0x804d);
        r.set_word(e + 0x1c, 1);
    }
    r.set_word(e + 0x3a, if orange { 0xe08a } else { 0xe6c1 });
    if orange || p != 255 {
        let v = word(rom, 0xa3e5f0 + u32::from(p) * 2)?;
        r.set_word(e + 0x30, v);
        r.set_word(e + 0x32, v);
    }
    match r.word(e + 0xe) & 3 {
        0 => r.set_word(e + 0x30, r.word(e + 0x30).wrapping_neg()),
        2 => r.set_word(e + 0x32, r.word(e + 0x32).wrapping_neg()),
        _ => (),
    }
    Ok(())
}
fn crawl(
    r: &mut Wram<'_>,
    rom: &[u8],
    host: &mut impl Host,
    k: u16,
    vertical: bool,
    orange: bool,
) -> Result<(), &'static str> {
    let e = record(k)?;
    let ext = 0x7800 + usize::from(k);
    let parameter = r.word(e + 0x3e);
    if !orange && parameter >> 1 >= 6 {
        return Err("invalid crawler animation index");
    }
    if r.word(0x1840) == 30 && r.word(0x183e) == 20 {
        r.set_word(ext + 6, r.word(e + 0x3a));
        r.set_word(e + 0x3a, 0xe785);
    }
    let (primary, secondary) = if vertical { (0x32, 0x30) } else { (0x30, 0x32) };
    let hit = move_axis(r, rom, host, k, probe_speed(r.word(e + primary)), !vertical, orange)?;
    let animate;
    if hit {
        r.set_word(ext + 8, 0);
        if !vertical {
            align(r, rom, k)?;
        }
        let amount = if vertical && !orange { slope_speed(r, rom)? } else { speed(r.word(e + secondary)) };
        let second = move_axis(r, rom, host, k, amount, vertical, orange)?;
        if !second {
            if vertical {
                align(r, rom, k)?;
            }
            if orange {
                let position = if vertical { 2 } else { 6 };
                let samus = if vertical { 0xaf6 } else { 0xafa };
                let v = r.word(e + secondary);
                if ((r.word(samus).wrapping_sub(r.word(e + position)) as i16) < 0) != (v & 0x8000 != 0) {
                    r.set_word(e + secondary, v.wrapping_neg());
                }
            }
            return Ok(());
        }
        r.set_word(e + primary, r.word(e + primary).wrapping_neg());
        animate = true;
    } else {
        let count = r.word(ext + 8).wrapping_add(1);
        r.set_word(ext + 8, count);
        if (count.wrapping_sub(4) as i16) < 0 {
            r.set_word(e + secondary, r.word(e + secondary).wrapping_neg());
            animate = true;
        } else {
            r.set_word(ext + 6, r.word(e + 0x3a));
            r.set_word(e + 0x3a, 0xe785);
            animate = false;
        }
    }
    if animate {
        let negative = r.word(e + secondary) & 0x8000 != 0;
        let instruction = if orange {
            if vertical {
                if negative { 0xdfcb } else { 0xdfe7 }
            } else if negative {
                0xe003
            } else {
                0xe01f
            }
        } else {
            let table = if vertical {
                if negative { 0xa3e648 } else { 0xa3e654 }
            } else if negative {
                0xa3e630
            } else {
                0xa3e63c
            };
            word(rom, table + u32::from(parameter >> 1) * 2)?
        };
        r.set_word(e + 0x1a, instruction);
        r.set_word(e + 0x1c, 1);
    }
    Ok(())
}
pub fn dispatch_host(
    r: &mut Wram<'_>,
    rom: &[u8],
    host: &mut impl Host,
    symbol: &str,
    a: i32,
    b: i32,
) -> Result<u32, &'static str> {
    let k = if symbol.ends_with("_Init") || symbol.ends_with("_Main") {
        r.word(0xe54)
    } else {
        u16::try_from(a).map_err(|_| "invalid crawler argument")?
    };
    let e = record(k)?;
    let ext = 0x7800 + usize::from(k);
    match symbol {
        "StoneZoomer_Init" | "FireZoomer_Init" | "BigEyeBugs_Init" | "WreckedShipOrangeZoomer_Init" => {
            initialize(r, rom, k, symbol.starts_with("Wrecked"), true)?
        }
        "StoneZoomer_E67A" => initialize(r, rom, k, false, false)?,
        "StoneZoomer_Main" | "WreckedShipOrangeZoomer_Main" => {
            let name = match r.word(e + 0x3a) {
                0xe6c1 | 0xe08a => return Ok(0),
                0xe6c8 => "FireZoomer_Func_1",
                0xe785 => "FireZoomer_Func_2",
                0xe7f2 => "FireZoomer_Func_3",
                0xe091 => "WreckedShipOrangeZoomer_Func_2",
                0xe168 => "sub_A3E168",
                _ => return Err("invalid crawler preinstruction"),
            };
            return dispatch_host(r, rom, host, name, i32::from(r.word(0xe54)), 0);
        }
        "FireZoomer_Func_1" | "FireZoomer_Func_3" | "WreckedShipOrangeZoomer_Func_2" | "sub_A3E168" => crawl(
            r,
            rom,
            host,
            k,
            symbol == "FireZoomer_Func_3" || symbol == "sub_A3E168",
            symbol.starts_with("Wrecked") || symbol == "sub_A3E168",
        )?,
        "FireZoomer_Func_2" => {
            let amount = ((u32::from(r.word(ext + 4)) << 16) | u32::from(r.word(ext + 2))) as i32;
            if move_axis(r, rom, host, k, amount, false, false)? {
                if r.word(e + 0x3c) == 255 {
                    r.set_word(e + 0x30, 128);
                    r.set_word(e + 0x32, 128);
                }
                r.set_word(ext + 2, 0);
                r.set_word(ext + 4, 0);
                r.set_word(ext + 8, 0);
                r.set_word(e + 0x3a, r.word(ext + 6));
            } else {
                if (r.word(ext + 4).wrapping_sub(4) as i16) < 0 {
                    let v = (amount as u32).wrapping_add(0x8000);
                    r.set_word(ext + 2, v as u16);
                    r.set_word(ext + 4, (v >> 16) as u16);
                }
                if r.word(ext + 2) == 0 && r.word(ext + 4) == 0 {
                    r.set_word(e + 0x3a, 0xe6c8);
                }
            }
        }
        "FireZoomer_E8A5" => return Ok(slope_speed(r, rom)? as u32),
        "EnemyFunc_C8AD" => return align(r, rom, k),
        "Zoomer_Instr_SetPreinstr" | "WreckedShipOrangeZoomer_Func_1" => {
            let pointer = u32::try_from(b).map_err(|_| "invalid crawler script pointer")?;
            if pointer >> 16 != 0xa3 || pointer & 0xffff < 0x8000 || pointer & 0xffff > 0xfffc {
                return Err("invalid crawler script pointer");
            }
            r.set_word(e + 0x3a, word(rom, pointer)?);
            return Ok(pointer + 2);
        }
        _ => return Err("unknown crawler routine"),
    };
    Ok(0)
}
