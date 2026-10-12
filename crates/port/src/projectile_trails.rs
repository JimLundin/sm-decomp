//! Samus projectile trail scripts and OAM emission from pinned bank $90.
//!
//! Trail allocation and placement belong to `projectile_flight`. This module
//! owns the complete eighteen-slot update/render boundary, including the native
//! partial word stores made before vertical clipping.
use crate::{WRAM_SIZE, Wram};

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

type Result<T> = std::result::Result<T, &'static str>;

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

fn slot(raw: u16) -> Result<usize> {
    // Direct native callbacks floor odd byte indices. Scheduled slots are even.
    if raw >= 36 {
        return Err("projectile trail slot outside eighteen-slot pool");
    }
    Ok(usize::from(raw & !1))
}

fn word(rom: &[u8], pointer: u16, displacement: usize) -> Result<u16> {
    if pointer < 0x8000 {
        return Err("projectile trail script outside bank90 ROM window");
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
        _ => return Err("unsupported projectile trail instruction"),
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
    Err("projectile trail script exceeds bounded instruction traversal")
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

fn frame(r: &mut Wram<'_>, rom: &[u8]) -> Result<()> {
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

fn active(r: &Wram<'_>) -> bool {
    (0..36).step_by(2).any(|k| r.word(LEFT_TIMER + k) != 0 || r.word(RIGHT_TIMER + k) != 0)
}

/// Execute an entire native trail frame. Unsupported scripts and invalid ROM
/// reads leave caller WRAM untouched, including timers and prior OAM emissions.
pub fn handle(r: &mut Wram<'_>, rom: &[u8]) -> Result<()> {
    if !active(r) {
        return Ok(());
    }
    if r.word(FROZEN) != 0 {
        // A frozen frame reads no ROM and has no fallible operations.
        return frame(r, rom);
    }
    let mut bytes: Vec<u8> = (0..WRAM_SIZE).map(|offset| r.byte(offset)).collect();
    frame(&mut Wram::new(&mut bytes)?, rom)?;
    for (offset, value) in bytes.into_iter().enumerate() {
        r.set_byte(offset, value);
    }
    Ok(())
}

/// Read-only ownership selection, including aliases introduced by earlier OAM
/// writes. A refusal occurs before the live bridge may choose a C execution.
pub fn supports_call(r: &Wram<'_>, rom: &[u8], symbol: &str, a: i32, _b: i32) -> Result<bool> {
    match symbol {
        "HandleProjectileTrails" => {
            if !active(r) || r.word(FROZEN) != 0 {
                return Ok(true);
            }
            let mut bytes: Vec<u8> = (0..WRAM_SIZE).map(|offset| r.byte(offset)).collect();
            Ok(frame(&mut Wram::new(&mut bytes)?, rom).is_ok())
        }
        "ProjInstr_MoveLeftProjectileTrailDown"
        | "ProjInstr_MoveRightProjectileTrailDown"
        | "ProjInstr_MoveLeftProjectileTrailUp" => Ok(u16::try_from(a).ok().and_then(|k| slot(k).ok()).is_some()),
        _ => Ok(false),
    }
}

pub fn dispatch_rom(r: &mut Wram<'_>, rom: &[u8], symbol: &str, a: i32, _b: i32) -> Result<u32> {
    let opcode = match symbol {
        "HandleProjectileTrails" => {
            handle(r, rom)?;
            return Ok(0);
        }
        "ProjInstr_MoveLeftProjectileTrailDown" => MOVE_LEFT_DOWN,
        "ProjInstr_MoveRightProjectileTrailDown" => MOVE_RIGHT_DOWN,
        "ProjInstr_MoveLeftProjectileTrailUp" => MOVE_LEFT_UP,
        _ => return Err("unknown projectile trail routine"),
    };
    let raw = u16::try_from(a).map_err(|_| "projectile trail slot argument width")?;
    instruction(r, slot(raw)?, opcode)?;
    Ok(0)
}
