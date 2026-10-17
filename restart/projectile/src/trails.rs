//! Samus projectile trail scripts and OAM emission from pinned bank $90.
//!
//! This module owns allocation and placement plus the complete eighteen-slot
//! update/render boundary, including the native
//! partial word stores made before vertical clipping.
use crate::{Result, Wram, core, rb, rw};

const LEFT_TIMER: usize = 0xd658;
const RIGHT_TIMER: usize = 0xd67c;
const LEFT_LIST: usize = 0xd6a0;
const RIGHT_LIST: usize = 0xd6c4;
const LEFT_TILE: usize = 0xd6e8;
const RIGHT_TILE: usize = 0xd70c;
const LEFT_X: usize = 0xd730;
const RIGHT_X: usize = 0xd754;
const LEFT_Y: usize = 0xd778;
const RIGHT_Y: usize = 0xd79c;
const CAMERA_X: usize = 0x911;
const CAMERA_Y: usize = 0x915;
const FROZEN: usize = 0xa78;
const OAM: usize = 0x370;
const CURSOR: usize = 0x590;
const MOVE_LEFT_DOWN: u16 = 0xb525;
const MOVE_RIGHT_DOWN: u16 = 0xb587;
const MOVE_LEFT_UP: u16 = 0xb5b3;

#[derive(Clone, Copy)]
enum Side {
    Left,
    Right,
}
impl Side {
    fn timer(self) -> usize {
        match self {
            Self::Left => LEFT_TIMER,
            Self::Right => RIGHT_TIMER,
        }
    }
    fn list(self) -> usize {
        match self {
            Self::Left => LEFT_LIST,
            Self::Right => RIGHT_LIST,
        }
    }
    fn tile(self) -> usize {
        match self {
            Self::Left => LEFT_TILE,
            Self::Right => RIGHT_TILE,
        }
    }
    fn x(self) -> usize {
        match self {
            Self::Left => LEFT_X,
            Self::Right => RIGHT_X,
        }
    }
    fn y(self) -> usize {
        match self {
            Self::Left => LEFT_Y,
            Self::Right => RIGHT_Y,
        }
    }
}

fn word(rom: &[u8], pointer: u16, displacement: usize) -> Result<u16> {
    if pointer < 0x8000 {
        return Err("projectile trail script outside bank90 ROM window".into());
    }
    // C reads a record's following word through its existing host pointer. At
    // the bank end this is a physical contiguous read, rather than a second
    // banked pointer conversion. A later instruction-pointer advance wraps.
    let offset = 0x80000 + usize::from(pointer & 0x7fff) + displacement;
    let bytes = rom.get(offset..offset + 2).ok_or("projectile trail ROM extent")?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}

fn instruction(r: &mut Wram<'_>, k: usize, opcode: u16) -> Result<()> {
    let (address, delta) = match opcode {
        MOVE_LEFT_DOWN => (LEFT_Y + k, 1),
        MOVE_RIGHT_DOWN => (RIGHT_Y + k, 1),
        MOVE_LEFT_UP => (LEFT_Y + k, u16::MAX),
        _ => return Err("unsupported projectile trail instruction".into()),
    };
    r.set_word(address, r.word(address).wrapping_add(delta));
    Ok(())
}

fn advance(r: &mut Wram<'_>, rom: &[u8], side: Side, k: usize) -> Result<bool> {
    let timer = r.word(side.timer() + k);
    if timer == 0 {
        return Ok(false);
    }
    let remaining = timer.wrapping_sub(1);
    r.set_word(side.timer() + k, remaining);
    if remaining != 0 {
        return Ok(true);
    }
    let mut pointer = r.word(side.list() + k);
    // Native scripts contain only a few consecutive movement instructions.
    // The finite bound rejects malformed unchecked C loops as a fixture domain.
    for _ in 0..4096 {
        let token = word(rom, pointer, 0)?;
        if token & 0x8000 != 0 {
            instruction(r, k, token)?;
            pointer = pointer.wrapping_add(2);
            continue;
        }
        r.set_word(side.timer() + k, token);
        if token == 0 {
            return Ok(false);
        }
        r.set_word(side.tile() + k, word(rom, pointer, 2)?);
        r.set_word(side.list() + k, pointer.wrapping_add(4));
        return Ok(true);
    }
    Err("projectile trail script exceeds bounded instruction traversal".into())
}

fn draw(r: &mut Wram<'_>, side: Side, k: usize) {
    let cursor = r.word(CURSOR);
    if cursor.wrapping_sub(512) as i16 >= 0 {
        return;
    }
    let x = r.word(side.x() + k).wrapping_sub(r.word(CAMERA_X));
    if x & 0xff00 != 0 {
        return;
    }
    let address = OAM + usize::from(cursor);
    // The original first store writes both X and Y. If Y is then clipped,
    // that transient zero high byte remains and the cursor stays unchanged.
    r.set_word(address, x);
    let y = r.word(side.y() + k).wrapping_sub(r.word(CAMERA_Y));
    if y & 0xff00 != 0 {
        return;
    }
    // Preserve the native overlapping word stores and reads between them.
    // Wrapped cursors can reach WRAM aliases, including later trail fields.
    r.set_word(address + 1, y);
    let tile = r.word(side.tile() + k);
    r.set_word(address + 2, tile);
    r.set_word(CURSOR, cursor.wrapping_add(4));
}

pub(crate) fn frame(r: &mut Wram<'_>, rom: &[u8]) -> Result<()> {
    let frozen = r.word(FROZEN) != 0;
    for k in (0..36).step_by(2).rev() {
        for side in [Side::Left, Side::Right] {
            let active = if frozen { r.word(side.timer() + k) != 0 } else { advance(r, rom, side, k)? };
            if active {
                draw(r, side, k);
            }
        }
    }
    Ok(())
}

/// CalcExplosion_Mode7 ($8B:8AD9), returning screen coordinates without writes.
/// Each signed product is shifted and truncated before wrapping the sums. The
/// native Y equation uses M7A again, regardless of the value of M7D.
pub(crate) fn mode7(r: &Wram<'_>, k: usize) -> (u16, u16) {
    fn product(a: u16, b: u16) -> u16 {
        ((i32::from(a as i16) * i32::from(b as i16)) >> 8) as u16
    }
    let dx = r.word(core::X + k).wrapping_sub(r.word(0x80));
    let dy = r.word(0x82).wrapping_sub(r.word(core::Y + k));
    let x = product(dx, r.word(0x78))
        .wrapping_add(product(r.word(0x7a), dy))
        .wrapping_add(r.word(0x80))
        .wrapping_sub(r.word(CAMERA_X));
    let y = r
        .word(0x82)
        .wrapping_sub(product(r.word(0x7c), dx).wrapping_add(product(r.word(0x78), dy)))
        .wrapping_sub(r.word(CAMERA_Y));
    (x, y)
}

/// Native allocation order is the highest free left-trail slot first.
pub(crate) fn spawn(r: &mut Wram<'_>, rom: &[u8], raw: u16) -> Result<()> {
    let k = core::slot(raw)?;
    if r.word(core::TYPE + k) & 0xf00 != 0 {
        return Ok(());
    }
    let Some(j) = (0..36).step_by(2).rev().find(|j| r.word(LEFT_TIMER + j) == 0) else {
        return Ok(());
    };
    r.set_word(LEFT_TIMER + j, 1);
    r.set_word(RIGHT_TIMER + j, 1);
    r.set_word(LEFT_LIST + j, rw(rom, 0x90b5bb)?);
    r.set_word(RIGHT_LIST + j, rw(rom, 0x90b609)?);
    // Native placement consumes the global index rather than the argument.
    let k = core::slot(r.word(core::INDEX))?;
    let direction = core::direction(r, k)?;
    let value = core::instruction_value(r, rom, k)?;
    let list = rw(rom, 0x9ba4b3)?;
    let direction_ptr = list.checked_add(2 * direction as u16).ok_or("trail direction wraps")?;
    let base = rw(rom, 0x9b0000 | u32::from(direction_ptr))?;
    let frame = base.checked_add(value.checked_mul(4).ok_or("trail frame multiply")?).ok_or("trail frame extent")?;
    let mut delta = [0i8; 4];
    for (i, v) in delta.iter_mut().enumerate() {
        *v = rb(rom, 0x9b0000 | u32::from(frame.checked_add(i as u16).ok_or("trail bytes")?))? as i8;
    }
    let (x, y) = if r.word(0x93f) & 0x8000 == 0 {
        (r.word(core::X + k), r.word(core::Y + k))
    } else {
        let (x, y) = mode7(r, k);
        (x.wrapping_add(r.word(CAMERA_X)), y.wrapping_add(r.word(CAMERA_Y)))
    };
    r.set_word(LEFT_Y + j, y.wrapping_add(delta[1] as u16).wrapping_sub(4));
    r.set_word(LEFT_X + j, x.wrapping_add(delta[0] as u16).wrapping_sub(4));
    r.set_word(RIGHT_Y + j, y.wrapping_add(delta[3] as u16).wrapping_sub(4));
    r.set_word(RIGHT_X + j, x.wrapping_add(delta[2] as u16).wrapping_sub(4));
    Ok(())
}
