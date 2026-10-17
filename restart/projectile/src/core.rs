//! Banks $90/$93: firing, velocity, instruction records, impact and deletion.
use crate::{Host, Result, Wram, audio, collision, rb, rw, trails};

pub(crate) const X: usize = 0xb64;
pub(crate) const Y: usize = 0xb78;
pub(crate) const SUB_X: usize = 0xb8c;
pub(crate) const SUB_Y: usize = 0xba0;
pub(crate) const RADIUS_X: usize = 0xbb4;
pub(crate) const RADIUS_Y: usize = 0xbc8;
pub(crate) const SPEED_X: usize = 0xbdc;
pub(crate) const SPEED_Y: usize = 0xbf0;
pub(crate) const DIRECTION: usize = 0xc04;
pub(crate) const TYPE: usize = 0xc18;
pub(crate) const DAMAGE: usize = 0xc2c;
pub(crate) const INSTRUCTION: usize = 0xc40;
pub(crate) const TIMER: usize = 0xc54;
pub(crate) const PRE: usize = 0xc68;
pub(crate) const SPRITEMAP: usize = 0xcb8;
pub(crate) const INDEX: usize = 0xdde;
const COUNT: usize = 0xcce;
const COOLDOWN: usize = 0xccc;

pub(crate) fn slot(raw: u16) -> Result<usize> {
    if raw > 18 || raw & 1 != 0 {
        return Err("projectile slot must be even and at most 18".into());
    }
    Ok(usize::from(raw))
}

pub(crate) fn direction(r: &Wram<'_>, k: usize) -> Result<usize> {
    let d = usize::from(r.word(DIRECTION + k) & 15);
    if d >= 10 {
        return Err("invalid projectile direction".into());
    }
    Ok(d)
}

fn bank_word(rom: &[u8], pointer: u16, delta: u16) -> Result<u16> {
    rw(rom, 0x930000 | u32::from(pointer.checked_add(delta).ok_or("projectile record wraps bank")?))
}

fn initialize(r: &mut Wram<'_>, rom: &[u8], k: usize) -> Result<()> {
    let pointer = rw(rom, 0x9383c1)?;
    let damage = bank_word(rom, pointer, 0)?;
    if damage & 0x8000 != 0 {
        return Err("invalid native projectile damage".into());
    }
    let ip = bank_word(rom, pointer, 2 + 2 * direction(r, k)? as u16)?;
    let radius = bank_word(rom, ip, 4)?;
    r.set_word(DAMAGE + k, damage);
    r.set_word(INSTRUCTION + k, ip);
    r.set_word(RADIUS_X + k, radius & 255);
    r.set_word(RADIUS_Y + k, radius >> 8);
    r.set_word(TIMER + k, 1);
    Ok(())
}

pub(crate) fn clear(r: &mut Wram<'_>, raw: u16) -> Result<()> {
    let k = slot(raw)?;
    for base in [
        X,
        Y,
        SUB_X,
        SUB_Y,
        DIRECTION,
        SPEED_X,
        SPEED_Y,
        RADIUS_X,
        RADIUS_Y,
        TYPE,
        DAMAGE,
        INSTRUCTION,
        TIMER,
        0xc7c,
        SPRITEMAP,
    ] {
        r.set_word(base + k, 0);
    }
    r.set_word(PRE + k, 0xb169);
    let count = if raw >= 10 { 0xcd2 } else { COUNT };
    let next = r.word(count).wrapping_sub(1);
    r.set_word(count, if next & 0x8000 != 0 { 0 } else { next });
    Ok(())
}

pub(crate) fn kill(r: &mut Wram<'_>, rom: &[u8], raw: u16) -> Result<()> {
    let k = slot(raw)?;
    if r.word(TYPE + k) & 0xf00 != 0 {
        return clear(r, raw);
    }
    let d = direction(r, k)?;
    let dx = [0, 1, 1, 1, 0, 0, -1, -1, -1, 0][d];
    let dy = [-1, -1, 0, 1, 1, 1, 1, 0, -1, -1][d];
    r.set_word(X + k, r.word(X + k).wrapping_add((dx * i32::from(r.word(RADIUS_X + k))) as u16));
    r.set_word(Y + k, r.word(Y + k).wrapping_add((dy * i32::from(r.word(RADIUS_Y + k))) as u16));
    let pointer = rw(rom, 0x93867b)?;
    r.set_word(TYPE + k, (r.word(TYPE + k) & 0xf0ff) | 0x700);
    r.set_word(INSTRUCTION + k, pointer);
    audio::queue(r, 1, 12, 6)?;
    r.set_word(TIMER + k, 1);
    r.set_word(DAMAGE + k, 8);
    r.set_word(PRE + k, 0xb169);
    Ok(())
}

pub(crate) fn initialize_speed(r: &mut Wram<'_>, raw: u16) -> Result<()> {
    let k = slot(raw)?;
    let d = direction(r, k)?;
    let speed = if matches!(d, 1 | 3 | 6 | 8) { 0x2ab } else { 0x400 };
    // These intentionally unaligned words preserve overlapping velocity views.
    let up = if r.byte(0xdb2) != 0 { (r.word(0xdb1) >> 2) | 0xc000 } else { 0 };
    let x = match d {
        1..=3 => r.word(0xdad).wrapping_add(speed),
        6..=8 => r.word(0xda9).wrapping_sub(speed),
        _ => 0,
    };
    let y = match d {
        0 | 1 | 8 | 9 => up.wrapping_sub(speed),
        3..=6 => r.word(0xdb5).wrapping_add(speed),
        _ => 0,
    };
    r.set_word(SUB_X + k, 0);
    r.set_word(SUB_Y + k, 0);
    r.set_word(SPEED_X + k, x);
    r.set_word(SPEED_Y + k, y);
    Ok(())
}

pub(crate) fn run_instructions(r: &mut Wram<'_>, rom: &[u8]) -> Result<()> {
    let raw = r.word(INDEX);
    let k = slot(raw)?;
    let timer = r.word(TIMER + k);
    // Even a pre-instruction that cleared the slot is followed by this decrement.
    r.set_word(TIMER + k, timer.wrapping_sub(1));
    if timer != 1 {
        return Ok(());
    }
    let mut pointer = r.word(INSTRUCTION + k);
    for _ in 0..1024 {
        let timer = bank_word(rom, pointer, 0)?;
        if timer & 0x8000 == 0 {
            let map = bank_word(rom, pointer, 2)?;
            let radius = bank_word(rom, pointer, 4)?;
            let next = pointer.checked_add(8).ok_or("projectile instruction wraps")?;
            r.set_word(TIMER + k, timer);
            r.set_word(SPRITEMAP + k, map);
            r.set_word(RADIUS_X + k, radius & 255);
            r.set_word(RADIUS_Y + k, radius >> 8);
            r.set_word(INSTRUCTION + k, next);
            return Ok(());
        }
        match timer {
            0x822f => return clear(r, raw),
            0x8239 => pointer = bank_word(rom, pointer, 2)?,
            _ => return Err(format!("unknown projectile instruction {timer:04x}")),
        }
    }
    Err("cyclic projectile instruction list".into())
}

pub(crate) fn instruction_value(r: &Wram<'_>, rom: &[u8], k: usize) -> Result<u16> {
    let ip = r.word(INSTRUCTION + k);
    let current = r.word(TIMER + k) == 1 && bank_word(rom, ip, 0)? & 0x8000 == 0;
    bank_word(rom, if current { ip } else { ip.checked_sub(8).ok_or("previous instruction underflow")? }, 6)
}

fn init_position(r: &mut Wram<'_>, rom: &[u8], k: usize) -> Result<bool> {
    let pose = r.word(0xa1c);
    if pose > 0xfc {
        return Err("Samus pose index".into());
    }
    let params = 0x91b629 + 8 * u32::from(pose);
    let override_dir = r.word(0xb5e);
    let mut d = if override_dir != 0 { override_dir as u8 } else { rb(rom, params + 3)? };
    if override_dir == 0 && d & 0xf0 != 0 {
        if d == 16 {
            let previous = r.word(0xa24);
            if previous > 0xfc {
                return Err("previous Samus pose index".into());
            }
            d = rb(rom, 0x91b629 + 8 * u32::from(previous) + 3)?;
        }
        if d & 0xf0 != 0 {
            r.set_word(COUNT, r.word(COUNT).wrapping_sub(1));
            return Ok(false);
        }
    }
    let index = usize::from(d & 15);
    if index >= 10 {
        return Err("projectile origin direction".into());
    }
    let offset = rb(rom, params + 4)?;
    let (xs, ys) = if matches!(pose, 0x75 | 0x76) || r.byte(0xa1f) == 1 {
        ([2i16, 15, 15, 13, 2, -5, -13, -13, -15, -2], [-8i16, -16, -2, 1, 13, 13, 1, -2, -16, -8])
    } else {
        ([2i16, 13, 11, 13, 2, -5, -14, -11, -19, -2], [-8i16, -13, 1, 4, 13, 13, 4, 1, -19, -8])
    };
    if override_dir != 0 {
        r.set_word(0xb5e, 0);
    }
    r.set_word(DIRECTION + k, u16::from(d));
    r.set_word(X + k, r.word(0xaf6).wrapping_add(xs[index] as u16));
    r.set_word(Y + k, r.word(0xafa).wrapping_add(ys[index] as u16).wrapping_sub(u16::from(offset)));
    Ok(true)
}

pub(crate) fn fire(r: &mut Wram<'_>, rom: &[u8], host: &mut impl Host) -> Result<()> {
    if r.word(COUNT).wrapping_sub(5) & 0x8000 == 0 || r.byte(COOLDOWN) != 0 {
        return failed_fire(r);
    }
    r.set_word(COOLDOWN, 1);
    r.set_word(COUNT, r.word(COUNT).wrapping_add(1));
    let k = (0..10).step_by(2).find(|k| r.word(DAMAGE + k) == 0).unwrap_or(8);
    if !init_position(r, rom, k)? {
        return failed_fire(r);
    }
    r.set_word(0x18ac, 10);
    r.set_word(0xc90 + k, 4);
    r.set_word(TYPE + k, 0x8000);
    audio::queue(r, 0, 11, 15)?;
    r.set_word(0xdc0, 0);
    initialize(r, rom, k)?;
    let automatic = r.word(0x9b2) & r.word(0x8f) == 0 && r.word(0x9b2) & r.word(0xe00) == 0;
    r.set_word(COOLDOWN, if automatic { 25 } else { 15 });
    r.set_word(SPEED_X + k, 0);
    r.set_word(SPEED_Y + k, 0);
    r.set_word(INDEX, k as u16);
    // CheckBeamCollByDir_7 probes one subpixel left before regular velocity setup.
    if direction(r, k)? == 7 {
        r.set_word(SPEED_X + k, u16::MAX);
    }
    collide_direction(r, rom, host, k as u16)?;
    let raw = r.word(INDEX);
    let k = slot(raw)?;
    if r.word(TYPE + k) & 0xf00 != 0 {
        return Ok(());
    }
    r.set_word(PRE + k, 0xaef3);
    initialize_speed(r, raw)
}

fn failed_fire(r: &mut Wram<'_>) -> Result<()> {
    if r.word(0xdc2).wrapping_sub(16) & 0x8000 == 0 {
        r.set_word(0xdc0, 0);
        audio::queue(r, 0, 2, 15)?;
    }
    Ok(())
}

fn collide_direction(r: &mut Wram<'_>, rom: &[u8], host: &mut impl Host, raw: u16) -> Result<()> {
    let d = direction(r, slot(raw)?)?;
    if matches!(d, 1 | 3 | 6 | 8) {
        if collision::collide(r, rom, host, raw, false)? == 0 {
            collision::collide(r, rom, host, raw, true)?;
        }
    } else {
        collision::collide(r, rom, host, raw, !matches!(d, 2 | 7))?;
    }
    Ok(())
}

fn flight(r: &mut Wram<'_>, rom: &[u8], host: &mut impl Host, raw: u16) -> Result<()> {
    let k = slot(raw)?;
    if r.word(DIRECTION + k) & 0xf0 != 0 {
        return clear(r, raw);
    }
    let timer = r.word(0xc90 + k);
    r.set_word(0xc90 + k, timer.wrapping_sub(1));
    if timer == 1 {
        r.set_word(0xc90 + k, 4);
        trails::spawn(r, rom, raw)?;
    }
    let raw = r.word(INDEX);
    let k = slot(raw)?;
    let d = direction(r, k)?;
    let dx = [0, 16, 16, 16, 0, 0, -16, -16, -16, 0][d];
    let dy = [-16, -16, 0, 16, 16, 16, 16, 0, -16, -16][d];
    r.set_word(SPEED_X + k, r.word(SPEED_X + k).wrapping_add(dx as u16));
    r.set_word(SPEED_Y + k, r.word(SPEED_Y + k).wrapping_add(dy as u16));
    collide_direction(r, rom, host, raw)?;
    let raw = r.word(INDEX);
    let k = slot(raw)?;
    let x = r.word(X + k).wrapping_sub(r.word(0x911));
    let y = r.word(Y + k).wrapping_sub(r.word(0x915));
    if x.wrapping_add(64) & 0x8000 != 0
        || x.wrapping_sub(320) & 0x8000 == 0
        || y.wrapping_add(64) & 0x8000 != 0
        || y.wrapping_sub(320) & 0x8000 == 0
    {
        clear(r, raw)?;
    }
    Ok(())
}

pub(crate) fn update(r: &mut Wram<'_>, rom: &[u8], host: &mut impl Host) -> Result<()> {
    let mut next = 18u16;
    r.set_word(INDEX, next);
    for _ in 0..1024 {
        if next & 0x8000 != 0 {
            return Ok(());
        }
        let k = slot(next)?;
        if r.word(INSTRUCTION + k) != 0 {
            match r.word(PRE + k) {
                0xaef3 => flight(r, rom, host, next)?,
                0xb169 => {}
                _ => return Err("changed projectile preinstruction".into()),
            }
            run_instructions(r, rom)?;
            next = r.word(INDEX);
            slot(next)?;
        }
        next = next.wrapping_sub(2);
        r.set_word(INDEX, next);
    }
    Err("cyclic projectile scheduler".into())
}
