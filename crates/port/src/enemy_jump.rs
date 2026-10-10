//! Sidehopper/Desgeega floor and ceiling jumps, pinned MIT snesrev/sm bank A3.
use crate::{Wram, enemy_ai};
pub use enemy_ai::Host;
const BASE: usize = 0xf78;
const CURRENT: usize = 0xe54;
const EXT: usize = 0x7800;
const STATE: usize = 0x32;
const SPEED_INDEX: usize = 0x34;
const HORIZONTAL: usize = 0x36;
const ACCELERATION: usize = 0x38;
const PARAMETER: usize = 0x3c;
const RANDOM: usize = 0x5e5;
const STATES: [u16; 15] = [
    0xabd6, 0xabe6, 0xac13, 0xac40, 0xac56, 0xac6c, 0xac8f, 0xaca8, 0xaccb, 0xace4, 0xad0e, 0xad20, 0xad32, 0xad44,
    0xad56,
];
fn record(k: u16) -> Result<usize, &'static str> {
    if k >= 0x800 || k & 63 != 0 { Err("invalid jumping enemy record") } else { Ok(BASE + usize::from(k)) }
}
fn word(rom: &[u8], address: u32) -> Result<u16, &'static str> {
    let p = (((address >> 16) & 127) * 32768 + (address & 32767)) as usize;
    let b = rom.get(p..p + 2).ok_or("missing jumping enemy ROM table")?;
    Ok(u16::from_le_bytes([b[0], b[1]]))
}
fn set(r: &mut Wram<'_>, e: usize, field: usize, v: u16) {
    r.set_word(e + field, v);
}
fn random(r: &mut Wram<'_>) -> u16 {
    let n = r.word(RANDOM);
    let lo = u16::from(n as u8) * 5;
    let high = (u16::from((n >> 8) as u8) * 5) as u8;
    let carry = (lo >> 8) + u16::from(high) + 1;
    let result = ((lo & 255) | u16::from(carry as u8) << 8).wrapping_add(carry >> 8).wrapping_add(17);
    r.set_word(RANDOM, result);
    result
}
fn animate(r: &mut Wram<'_>, k: u16) -> Result<(), &'static str> {
    let e = record(k)?;
    set(r, e, 0x1a, r.word(EXT + usize::from(k)));
    set(r, e, 0x1c, 1);
    set(r, e, 0x18, 0);
    Ok(())
}
fn variant(r: &Wram<'_>, k: u16, field: usize) -> Result<u32, &'static str> {
    let n = r.word(EXT + usize::from(k) + field);
    if n > 6 || n & 1 != 0 {
        return Err("jump variant outside four animation rows");
    }
    Ok(u32::from(n))
}
/// Bounded admission for C's unaligned quadratic-speed integral. The selected
/// table has 95 rows; a nonterminating or out-of-table integral is rejected.
fn integrate(rom: &[u8], target: u16, step: u16) -> Result<u16, &'static str> {
    if step == 0 || step > 94 {
        return Err("invalid jump integral step");
    }
    let mut index = 0u16;
    let mut sum = 0u16;
    loop {
        index = index.wrapping_add(step);
        if index > 94 {
            return Err("jump integral leaves quadratic table");
        }
        sum = sum.wrapping_add(word(rom, 0xa2838f + 8 * u32::from(index) + 1)?);
        if sum.wrapping_sub(target) & 0x8000 == 0 {
            return Ok(index);
        }
    }
}
fn move_axis(
    r: &mut Wram<'_>,
    rom: &[u8],
    host: &mut impl Host,
    k: u16,
    amount: i32,
    horizontal: bool,
) -> Result<bool, &'static str> {
    Ok(enemy_ai::dispatch_host(
        r,
        rom,
        host,
        if horizontal { "Enemy_MoveRight_IgnoreSlopes" } else { "Enemy_MoveDown" },
        i32::from(k),
        amount,
    )? != 0)
}
fn jump(
    r: &mut Wram<'_>,
    rom: &[u8],
    host: &mut impl Host,
    k: u16,
    ceiling: bool,
    descending: bool,
) -> Result<(), &'static str> {
    let e = record(k)?;
    let x = EXT + usize::from(k);
    let index = r.word(e + SPEED_INDEX);
    if index > 94 {
        return Err("jump speed index outside 95 quadratic rows");
    }
    let pair = if ceiling ^ descending { 0 } else { 2 };
    let lo = word(rom, 0xa2838f + 8 * u32::from(index) + pair * 2)?;
    let hi = word(rom, 0xa2838f + 8 * u32::from(index) + (pair + 1) * 2)?;
    if move_axis(r, rom, host, k, ((u32::from(hi) << 16) | u32::from(lo)) as i32, false)? {
        if descending {
            r.set_word(x + 6, 0);
            set(r, e, STATE, 0xace4);
        } else {
            set(r, e, HORIZONTAL, r.word(e + HORIZONTAL).wrapping_neg());
            r.set_word(x + 6, 1);
        }
        return Ok(());
    }
    let hit = move_axis(r, rom, host, k, i32::from(r.word(e + HORIZONTAL) as i16) * 65536, true)?;
    if hit {
        set(r, e, HORIZONTAL, r.word(e + HORIZONTAL).wrapping_neg());
        if !descending {
            r.set_word(x + 6, 1);
            return Ok(());
        }
    }
    let acceleration = r.word(e + ACCELERATION);
    if descending {
        let v = index.wrapping_add(acceleration);
        set(r, e, SPEED_INDEX, if v.wrapping_sub(64) & 0x8000 == 0 { 64 } else { v });
    } else {
        let v = index.wrapping_sub(acceleration);
        set(r, e, SPEED_INDEX, v);
        if v & 0x8000 != 0 {
            r.set_word(x + 6, 1);
            set(r, e, SPEED_INDEX, 0);
        }
    }
    Ok(())
}
fn state(r: &mut Wram<'_>, rom: &[u8], host: &mut impl Host, k: u16, n: u8) -> Result<(), &'static str> {
    let e = record(k)?;
    let x = EXT + usize::from(k);
    match n {
        3 => animate(r, k)?,
        4 => {
            let choice = random(r) & 1;
            set(r, e, STATE, word(rom, 0xa3aae2 + 2 * u32::from(choice))?);
        }
        5 | 6 => {
            let v = variant(r, k, 10)?;
            if v > 2 {
                return Err("jump speed variant outside two rows");
            }
            let base = if n == 5 { 0xa3aae6 } else { 0xa3aaf2 };
            let accel = word(rom, base + v)?;
            let h = word(rom, base + 4 + v)?;
            set(r, e, ACCELERATION, accel);
            set(r, e, HORIZONTAL, h);
            set(r, e, SPEED_INDEX, r.word(x + if n == 5 { 2 } else { 4 }));
            set(r, e, STATE, if r.word(e + PARAMETER) != 0 { 0xac56 } else { 0xac40 });
        }
        7 | 8 => {
            let left = r.word(0xaf6).wrapping_sub(r.word(e + 2)) & 0x8000 != 0;
            set(
                r,
                e,
                STATE,
                if n == 7 {
                    if left { 0xac6c } else { 0xac8f }
                } else if left {
                    0xaca8
                } else {
                    0xaccb
                },
            );
        }
        9..=12 => {
            if n == 9 || n == 11 {
                set(r, e, HORIZONTAL, r.word(e + HORIZONTAL).wrapping_neg());
            }
            let v = variant(r, k, 12)?;
            r.set_word(x, word(rom, if n <= 10 { 0xa3aad2 } else { 0xa3aada } + v)?);
            animate(r, k)?;
            set(
                r,
                e,
                STATE,
                match n {
                    9 => 0xad0e,
                    10 => 0xad20,
                    11 => 0xad32,
                    _ => 0xad44,
                },
            );
        }
        13 => {
            let v = variant(r, k, 12)?;
            r.set_word(x, word(rom, if r.word(e + PARAMETER) != 0 { 0xa3aaca } else { 0xa3aac2 } + v)?);
            animate(r, k)?;
            set(r, e, STATE, 0xad56);
        }
        14 | 15 => jump(r, rom, host, k, false, r.word(x + 6) != 0)?,
        16 | 17 => jump(r, rom, host, k, true, r.word(x + 6) != 0)?,
        18 => {
            if r.word(x + 8) != 0 {
                r.set_word(x + 8, 0);
                set(r, e, STATE, 0xabd6);
            }
        }
        19..=22 => jump(r, rom, host, k, n >= 21, n == 20 || n == 22)?,
        _ => return Err("unsupported jump state"),
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
    let k = r.word(CURRENT);
    let e = record(k)?;
    let x = EXT + usize::from(k);
    match symbol {
        "Sidehopper_Init" => {
            let pointer = r.word(e);
            if !(0x8000..=0xffac).contains(&pointer) {
                return Err("enemy definition outside bank A0");
            }
            let kind = word(rom, 0xa00000 | (u32::from(pointer) + 0x2a))?;
            if kind > 3 {
                return Err("jump definition variant outside four rows");
            }
            // Validate all fallible table integration before C's unchecked accesses.
            let speed_variant = u32::from(kind != 0) * 2;
            let one = integrate(rom, word(rom, 0xa3aaee + speed_variant)?, word(rom, 0xa3aae6 + speed_variant)?)?;
            let two = integrate(rom, word(rom, 0xa3aafa + speed_variant)?, word(rom, 0xa3aaf2 + speed_variant)?)?;
            let list = word(rom, if r.word(e + PARAMETER) != 0 { 0xa3aaca } else { 0xa3aac2 } + 2 * u32::from(kind))?;
            r.set_word(RANDOM, 37);
            random(r);
            for f in [6, 8, 10] {
                r.set_word(x + f, 0);
            }
            r.set_word(x + 12, kind * 2);
            r.set_word(x, list);
            animate(r, k)?;
            r.set_word(x + 10, speed_variant as u16);
            r.set_word(x + 2, one);
            r.set_word(x + 4, two);
            set(r, e, STATE, 0xabd6);
        }
        "Sidehopper_Main" => {
            let s = r.word(e + STATE);
            let index = STATES.iter().position(|v| *v == s).ok_or("unsupported Sidehopper preinstruction")?;
            state(r, rom, host, k, index as u8 + 4)?;
        }
        "Sidehopper_Func_2" => {
            return Ok(u32::from(integrate(
                rom,
                u16::try_from(a).map_err(|_| "invalid jump threshold")?,
                u16::try_from(b).map_err(|_| "invalid jump step")?,
            )?));
        }
        _ => {
            let n = symbol
                .strip_prefix("Sidehopper_Func_")
                .and_then(|s| s.parse::<u8>().ok())
                .ok_or("unknown jump routine")?;
            let index = if n == 4 { u16::try_from(a).map_err(|_| "invalid jumping record")? } else { k };
            state(r, rom, host, index, n)?;
        }
    }
    Ok(0)
}
/// Bank A3 script callbacks remain separate from the host-dependent movement ABI.
pub fn dispatch_rom(r: &mut Wram<'_>, rom: &[u8], symbol: &str, a: i32, b: i32) -> Result<u32, &'static str> {
    let k = u16::try_from(a).map_err(|_| "invalid jumping script enemy")?;
    record(k)?;
    let pointer = b as u32;
    if pointer >> 16 != 0xa3 || pointer & 65535 < 0x8000 {
        return Err("jump script pointer outside bank A3");
    }
    match symbol {
        "Sidehopper_Instr_1" => {
            let current = r.word(CURRENT);
            record(current)?;
            r.set_word(EXT + usize::from(current) + 8, 1);
            Ok(pointer)
        }
        "Sidehopper_Func_1" => {
            if pointer & 65535 > 0xfffc {
                return Err("jump script argument crosses bank end");
            }
            let sound = word(rom, pointer)?;
            crate::audio::dispatch(r, "QueueSfx2_Max3", sound, 0)?;
            Ok(pointer + 2)
        }
        _ => Err("unknown jumping script instruction"),
    }
}
