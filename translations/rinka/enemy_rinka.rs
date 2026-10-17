//! Complete Rinka actor, recovered C A2:B602 through A2:B9C7.
#![forbid(unsafe_code)]

const SIZE: usize = 0x20000;
const BASE: usize = 0xf78;
const CURRENT: usize = 0xe54;
const SPAWN: usize = 0x7000;
const GLOBAL_DEAD: usize = 0x783a;
const GLOBAL_COUNT: usize = 0x783c;
const CAMERA_X: usize = 0x911;
const CAMERA_Y: usize = 0x915;
const SAMUS_X: usize = 0xaf6;
const SAMUS_Y: usize = 0xafa;
const RANDOM: usize = 0xe44;
const X: usize = 2;
const Y: usize = 6;
const PROP: usize = 0xe;
const HEALTH: usize = 0x14;
const TIMER: usize = 0x18;
const INSTRUCTION: usize = 0x1a;
const INSTRUCTION_TIMER: usize = 0x1c;
const PALETTE: usize = 0x1e;
const FROZEN: usize = 0x26;
const A: usize = 0x30;
const B: usize = 0x32;
const C: usize = 0x34;
const D: usize = 0x36;
const F: usize = 0x3a;
const PARAM: usize = 0x3c;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry {
    Init,
    Init2(u16),
    Init3(u16),
    Position(u16),
    PositionIndex(u16),
    Call { address: u32, k: u16 },
    Main,
    Func3(u16),
    Func4(u16),
    Func5(u16),
    Moving(u16),
    Reset(u16),
    DecrementCount(u16),
    Tangibility(u16),
    ReleasePosition(u16),
    Outside(u16),
    OutsideSpawn { x: u16, y: u16 },
    Frozen(u16),
    Touch,
    Shot,
    Powerbomb(u16),
    AfterDamage(u16),
    InstructionB9A2 { k: u16, j: u16 },
    InstructionB9B3 { k: u16, jp: u16 },
    InstructionB9BD { k: u16, jp: u16 },
    InstructionB9C7 { k: u16, jp: u16 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Output {
    Done,
    Word(u16),
    Outside(bool),
    Instruction(u16),
}

/// Each variant names one existing synchronous service, not an actor replacement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Service {
    CalculateAngle {
        x: u16,
        y: u16,
    },
    MultiplySin {
        amount: u16,
        angle: u16,
    },
    MultiplyCos {
        amount: u16,
        angle: u16,
    },
    MoveEnemyWithVelocity,
    DeleteEnemyAndConnectedEnemies,
    NormalEnemyFrozenAI,
    NormalEnemyTouchAiSkipDeathAnimCurEnemy,
    NormalEnemyShotAiSkipDeathAnimCurEnemy,
    NormalEnemyPowerBombAiSkipDeathAnimCurEnemy,
    RinkasDeathAnimation(u16),
    /// Assignment to external eproj_spawn_pt precedes the following spawn call.
    /// This operation changes only that point and returns synchronously; unlike
    /// native calls it may not change WRAM or generate additional hardware effects.
    SetProjectileSpawnPoint {
        x: u16,
        y: u16,
    },
    SpawnEprojWithRoomGfx {
        definition: u16,
        parameter: u16,
    },
}

/// Immutable admission sees original root arguments, complete WRAM and immutable
/// ROM. It must cover all reachable services, including pre-service local writes,
/// full synchronous WRAM/external state copyback, effect order and changed globals
/// used after callbacks. Native generation/alias/reentry checks belong here.
/// No service may fall back after taking ownership. Err preserves prior mutations
/// and effects and is terminal; a caller must not replay the actor through C.
pub trait Services {
    fn admitted(&self, entry: Entry, ram: &[u8], rom: &[u8]) -> bool;
    /// Angle/trig return their recovered u16 result; other service results are ignored.
    fn call(&mut self, ram: &mut [u8], service: Service) -> Result<u16, String>;
}

#[derive(Clone, Copy)]
struct Position {
    x: u16,
    y: u16,
    overlay: u16,
}
type Positions = [Position; 11];

/// None is unchanged refusal. Whole conservative admission requires all eleven
/// A2:B75B triples and valid occupancy overlays even for entries not reading them.
pub fn execute(
    entry: Entry,
    ram: &mut [u8],
    rom: &[u8],
    services: &mut impl Services,
) -> Result<Option<Output>, String> {
    if ram.len() != SIZE || record(word(ram, CURRENT)).is_none() {
        return Ok(None);
    }
    let Some(positions) = positions(rom) else { return Ok(None) };
    if positions.iter().any(|p| occupancy(p.overlay).is_none()) {
        return Ok(None);
    }
    // Existing D values may become reachable after callback copyback. Admission
    // checks every canonical enemy slot; services must preserve this validity.
    if (0..0x800).step_by(64).any(|k| {
        let d = word(ram, BASE + k + D);
        d != 0 && occupancy(d).is_none()
    }) {
        return Ok(None);
    }
    if let Some(k) = argument_slot(entry) {
        if record(k).is_none() {
            return Ok(None);
        }
    }
    if let Entry::Call { address, .. } = entry {
        if !matches!(address, 0xa2b7df | 0xa2b844 | 0xa2b852 | 0xa2b85b) {
            return Ok(None);
        }
    }
    if matches!(entry, Entry::Main) {
        let e = BASE + usize::from(word(ram, CURRENT));
        if !(word(ram, e + PARAM) != 0 && word(ram, GLOBAL_DEAD) != 0)
            && !matches!(word(ram, e + A), 0xb7df | 0xb844 | 0xb852 | 0xb85b)
        {
            return Ok(None);
        }
    }
    if let Entry::InstructionB9A2 { j, .. } = entry {
        if rom_word(rom, j).is_none() {
            return Ok(None);
        }
    }
    if !services.admitted(entry, ram, rom) {
        return Ok(None);
    }
    run(entry, ram, rom, &positions, services).map(Some)
}

fn argument_slot(entry: Entry) -> Option<u16> {
    match entry {
        Entry::Init2(k)
        | Entry::Init3(k)
        | Entry::Position(k)
        | Entry::PositionIndex(k)
        | Entry::Call { k, .. }
        | Entry::Func3(k)
        | Entry::Func4(k)
        | Entry::Func5(k)
        | Entry::Moving(k)
        | Entry::Reset(k)
        | Entry::DecrementCount(k)
        | Entry::Tangibility(k)
        | Entry::ReleasePosition(k)
        | Entry::Outside(k)
        | Entry::Frozen(k)
        | Entry::Powerbomb(k)
        | Entry::AfterDamage(k)
        | Entry::InstructionB9B3 { k, .. }
        | Entry::InstructionB9BD { k, .. }
        | Entry::InstructionB9C7 { k, .. } => Some(k),
        // B9A2 ignores its k and reads slot zero.
        _ => None,
    }
}

fn run(entry: Entry, r: &mut [u8], rom: &[u8], p: &Positions, s: &mut impl Services) -> Result<Output, String> {
    match entry {
        Entry::Init => {
            let k = current(r)?;
            let e = record_checked(k)?;
            let special = word(r, e + PARAM) != 0;
            if special {
                position(r, p, k)?;
            }
            let properties = (word(r, e + PROP) & !0x6c00) | if special { 0x2c00 } else { 0x6400 };
            set(r, e + PROP, properties);
            set(r, e + PALETTE, 1024);
            // C rereads CURRENT for Init3 after positioning.
            let current_k = current(r)?;
            init3(r, current_k)?;
        }
        Entry::Init2(k) => init2(r, p, k)?,
        Entry::Init3(k) => init3(r, k)?,
        Entry::Position(k) => position(r, p, k)?,
        Entry::PositionIndex(k) => return Ok(Output::Word(position_index(r, p, k)?)),
        Entry::Call { address, k } => dispatch(r, p, s, address, k)?,
        Entry::Main => {
            let k = current(r)?;
            let e = record_checked(k)?;
            if word(r, e + PARAM) != 0 && word(r, GLOBAL_DEAD) != 0 {
                decrement(r, k)?;
                let current_k = current(r)?;
                release(r, current_k)?;
                s.call(r, Service::RinkasDeathAnimation(0))?;
            } else {
                let address = 0xa20000 | u32::from(word(r, e + A));
                let current_k = current(r)?;
                dispatch(r, p, s, address, current_k)?;
            }
        }
        Entry::Func3(k) => func3(r, s, k)?,
        Entry::Func4(k) => {
            let e = record_checked(k)?;
            let f = word(r, e + F).wrapping_sub(1);
            set(r, e + F, f);
            if f & 0x8000 != 0 {
                set(r, e + HEALTH, 10);
                init2(r, p, k)?;
            }
        }
        Entry::Func5(k) => {
            if outside(r, k)? {
                reset(r, p, s, k)?;
            }
        }
        Entry::Moving(k) => {
            s.call(r, Service::MoveEnemyWithVelocity)?;
            if outside(r, k)? {
                reset(r, p, s, k)?;
            }
        }
        Entry::Reset(k) => reset(r, p, s, k)?,
        Entry::DecrementCount(k) => decrement(r, k)?,
        Entry::Tangibility(k) => {
            let e = record_checked(k)?;
            let properties = if word(r, RANDOM) & 3 == word(r, e + PARAM) {
                word(r, e + PROP) & !0x400
            } else {
                word(r, e + PROP) | 0x400
            };
            set(r, e + PROP, properties);
        }
        Entry::ReleasePosition(k) => release(r, k)?,
        Entry::Outside(k) => return Ok(Output::Outside(outside(r, k)?)),
        Entry::OutsideSpawn { x, y } => return Ok(Output::Outside(outside_spawn(r, x, y))),
        Entry::Frozen(k) => {
            if outside(r, k)? {
                set(r, record_checked(k)? + FROZEN, 0);
            }
            s.call(r, Service::NormalEnemyFrozenAI)?;
            if word(r, GLOBAL_DEAD) != 0 {
                decrement(r, k)?;
                release(r, k)?;
                s.call(r, Service::RinkasDeathAnimation(0))?;
            }
        }
        Entry::Touch => {
            s.call(r, Service::NormalEnemyTouchAiSkipDeathAnimCurEnemy)?;
            let current_k = current(r)?;
            after_damage(r, s, current_k)?;
        }
        Entry::Shot => {
            s.call(r, Service::NormalEnemyShotAiSkipDeathAnimCurEnemy)?;
            let current_k = current(r)?;
            after_damage(r, s, current_k)?;
        }
        Entry::Powerbomb(k) => {
            if word(r, record_checked(k)? + PROP) & 0x100 == 0 {
                s.call(r, Service::NormalEnemyPowerBombAiSkipDeathAnimCurEnemy)?;
                after_damage(r, s, k)?;
            }
        }
        Entry::AfterDamage(k) => after_damage(r, s, k)?,
        Entry::InstructionB9A2 { j, .. } => {
            let result = if word(r, GLOBAL_COUNT).wrapping_sub(3) & 0x8000 != 0 {
                j.wrapping_add(2)
            } else {
                rom_word(rom, j).ok_or("Rinka instruction ROM unavailable after ownership; terminal")?
            };
            return Ok(Output::Instruction(result));
        }
        Entry::InstructionB9B3 { k, jp } | Entry::InstructionB9BD { k, jp } => {
            let e = record_checked(k)?;
            let bits = if matches!(entry, Entry::InstructionB9BD { .. }) { 0xd00 } else { 0x500 };
            let properties = word(r, e + PROP) | bits;
            set(r, e + PROP, properties);
            return Ok(Output::Instruction(jp));
        }
        Entry::InstructionB9C7 { k, jp } => {
            let e = record_checked(k)?;
            let properties = word(r, e + PROP) & !0x500;
            set(r, e + PROP, properties);
            set(r, e + A, 0xb7df);
            let count = word(r, GLOBAL_COUNT).wrapping_add(1);
            set(r, GLOBAL_COUNT, count);
            return Ok(Output::Instruction(jp));
        }
    }
    Ok(Output::Done)
}

fn dispatch(r: &mut [u8], p: &Positions, s: &mut impl Services, address: u32, k: u16) -> Result<(), String> {
    let entry = match address {
        0xa2b7df => Entry::Func3(k),
        0xa2b844 => Entry::Func4(k),
        0xa2b852 => Entry::Func5(k),
        0xa2b85b => Entry::Moving(k),
        _ => return Err("Rinka invalid dispatch after ownership; terminal".into()),
    };
    run(entry, r, &[], p, s)?;
    Ok(())
}

fn record(k: u16) -> Option<usize> {
    (k < 0x800 && k & 63 == 0).then_some(BASE + usize::from(k))
}
fn record_checked(k: u16) -> Result<usize, String> {
    record(k).ok_or_else(|| "Rinka invalid enemy index after ownership; terminal".into())
}
fn current(r: &[u8]) -> Result<u16, String> {
    let k = word(r, CURRENT);
    record_checked(k)?;
    Ok(k)
}
fn occupancy(offset: u16) -> Option<usize> {
    // Ram8000_Default is 64 bytes; [31].var_3F is +31*64+62.
    let address = 0x8000 + usize::from(offset) + 31 * 64 + 62;
    (offset & 1 == 0 && address + 2 <= SIZE).then_some(address)
}
fn occupancy_checked(offset: u16) -> Result<usize, String> {
    occupancy(offset).ok_or_else(|| "Rinka invalid occupancy overlay after ownership; terminal".into())
}
fn word(r: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([r[offset], r[offset + 1]])
}
fn set(r: &mut [u8], offset: usize, value: u16) {
    r[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}
fn rom_word(rom: &[u8], address: u16) -> Option<u16> {
    if address < 0x8000 {
        return None;
    }
    let offset = 0x22 * 0x8000 + usize::from(address & 0x7fff);
    let bytes = rom.get(offset..offset + 2)?;
    Some(u16::from_le_bytes([bytes[0], bytes[1]]))
}
fn positions(rom: &[u8]) -> Option<Positions> {
    let mut p = [Position { x: 0, y: 0, overlay: 0 }; 11];
    for (i, point) in p.iter_mut().enumerate() {
        let address = 0xb75b + i as u16 * 6;
        *point = Position {
            x: rom_word(rom, address)?,
            y: rom_word(rom, address + 2)?,
            overlay: rom_word(rom, address + 4)?,
        };
    }
    Some(p)
}

fn init2(r: &mut [u8], p: &Positions, k: u16) -> Result<(), String> {
    let e = record_checked(k)?;
    if word(r, e + PARAM) != 0 {
        position(r, p, k)?;
    }
    let spawn = SPAWN + usize::from(k);
    let x = word(r, spawn + 0x20);
    let y = word(r, spawn + 0x22);
    set(r, e + X, x);
    set(r, e + Y, y);
    init3(r, k)
}
fn init3(r: &mut [u8], k: u16) -> Result<(), String> {
    let e = record_checked(k)?;
    set(r, e + A, 0xb852);
    set(r, e + F, 26);
    set(r, e + B, 0);
    set(r, e + C, 0);
    let special = word(r, e + PARAM) != 0;
    if special && word(r, GLOBAL_DEAD) != 0 {
        let properties = word(r, e + PROP) | 0x200;
        set(r, e + PROP, properties);
    } else {
        set(r, e + INSTRUCTION, if special { 0xba0c } else { 0xb9e0 });
        set(r, e + INSTRUCTION_TIMER, 1);
        set(r, e + TIMER, 0);
    }
    Ok(())
}

fn position_index(r: &[u8], p: &Positions, k: u16) -> Result<u16, String> {
    record_checked(k)?;
    let spawn = SPAWN + usize::from(k);
    let found = p.iter().find(|point| point.x == word(r, spawn + 0x20) && point.y == word(r, spawn + 0x22));
    Ok(found.unwrap_or(&p[0]).overlay)
}
fn position(r: &mut [u8], p: &Positions, k: u16) -> Result<(), String> {
    let e = record_checked(k)?;
    let spawn = SPAWN + usize::from(k);
    let offscreen = outside_spawn(r, word(r, spawn + 0x20), word(r, spawn + 0x22));
    // Preserve C's short-circuit: lookup need not happen when offscreen.
    let overlay = if offscreen { None } else { Some(position_index(r, p, k)?) };
    if offscreen || word(r, occupancy_checked(overlay.ok_or("Rinka missing selected overlay; terminal")?)?) & 1 != 0 {
        let first = p.iter().find(|point| {
            !outside_spawn(r, point.x, point.y)
                && word(r, occupancy(point.overlay).expect("prevalidated ROM overlay")) & 1 == 0
        });
        let fallback =
            || p.iter().find(|point| word(r, occupancy(point.overlay).expect("prevalidated ROM overlay")) & 1 == 0);
        if let Some(point) = first.or_else(fallback) {
            set(r, spawn + 0x20, point.x);
            set(r, e + X, point.x);
            set(r, spawn + 0x22, point.y);
            set(r, e + Y, point.y);
            if first.is_some() {
                // C's preferred visible branch marks occupancy before saving D.
                set(r, occupancy_checked(point.overlay)?, u16::MAX);
                set(r, e + D, point.overlay);
            } else {
                // Its fallback branch saves D before marking occupancy.
                set(r, e + D, point.overlay);
                set(r, occupancy_checked(point.overlay)?, u16::MAX);
            }
        }
    } else {
        let offset = overlay.ok_or("Rinka missing selected overlay; terminal")?;
        set(r, e + D, offset);
        set(r, occupancy_checked(offset)?, u16::MAX);
    }
    Ok(())
}

fn func3(r: &mut [u8], s: &mut impl Services, k: u16) -> Result<(), String> {
    let e = record_checked(k)?;
    let f = word(r, e + F).wrapping_sub(1);
    set(r, e + F, f);
    if f & 0x8000 != 0 {
        set(r, e + A, 0xb85b);
        let properties =
            if word(r, e + PARAM) != 0 { word(r, e + PROP) & !0x400 } else { (word(r, e + PROP) & 0xf3ff) | 0x800 };
        set(r, e + PROP, properties);
        let x = word(r, SAMUS_X).wrapping_sub(word(r, e + X));
        let y = word(r, SAMUS_Y).wrapping_sub(word(r, e + Y));
        let angle = s.call(r, Service::CalculateAngle { x, y })?;
        let angle = 0u8.wrapping_sub(angle.wrapping_add(0x80) as u8) as u16;
        let sine = s.call(r, Service::MultiplySin { amount: 0x120, angle })?;
        set(r, e + B, sine);
        let cosine = s.call(r, Service::MultiplyCos { amount: 0x120, angle })?;
        set(r, e + C, cosine);
    }
    Ok(())
}

fn reset(r: &mut [u8], p: &Positions, s: &mut impl Services, k: u16) -> Result<(), String> {
    let e = record_checked(k)?;
    let special = word(r, e + PARAM) != 0;
    if special {
        release(r, k)?;
    }
    if special && word(r, GLOBAL_DEAD) != 0 {
        decrement(r, k)?;
        s.call(r, Service::DeleteEnemyAndConnectedEnemies)?;
    } else {
        decrement(r, k)?;
        init2(r, p, k)?;
    }
    Ok(())
}
fn decrement(r: &mut [u8], k: u16) -> Result<(), String> {
    let e = record_checked(k)?;
    if word(r, e + PARAM) != 0 && word(r, e + PROP) & 0x100 == 0 {
        let value = word(r, GLOBAL_COUNT).wrapping_sub(1) as i16;
        set(r, GLOBAL_COUNT, if value < 0 { 0 } else { value as u16 });
    }
    Ok(())
}
fn release(r: &mut [u8], k: u16) -> Result<(), String> {
    let e = record_checked(k)?;
    if word(r, e + PARAM) != 0 {
        let d = word(r, e + D);
        if d != 0 {
            set(r, occupancy_checked(d)?, 0);
            set(r, e + D, 0);
        }
    }
    Ok(())
}
fn outside(r: &[u8], k: u16) -> Result<bool, String> {
    let e = record_checked(k)?;
    let y = word(r, e + Y);
    let y_relative = y.wrapping_add(16).wrapping_sub(word(r, CAMERA_Y));
    let x = word(r, e + X);
    let x_relative = x.wrapping_add(16).wrapping_sub(word(r, CAMERA_X));
    Ok(!((y as i16) >= 0
        && (y_relative as i16) >= 0
        && y_relative.wrapping_sub(256) & 0x8000 != 0
        && (x as i16) >= 0
        && (x_relative as i16) >= 0
        && x_relative.wrapping_sub(288) & 0x8000 != 0))
}
fn outside_spawn(r: &[u8], x: u16, y: u16) -> bool {
    let dy = y.wrapping_sub(word(r, CAMERA_Y));
    let dx = x.wrapping_sub(word(r, CAMERA_X));
    // C tests y's sign twice (one identical redundant test is folded here);
    // it intentionally does not separately test x's sign.
    y & 0x8000 != 0
        || (dy as i16) < 0
        || dy.wrapping_sub(224) & 0x8000 == 0
        || (dx as i16) < 0
        || dx.wrapping_sub(256) & 0x8000 == 0
}
fn after_damage(r: &mut [u8], s: &mut impl Services, k: u16) -> Result<(), String> {
    let e = record_checked(k)?;
    if word(r, e + HEALTH) == 0 {
        decrement(r, k)?;
        release(r, k)?;
        if word(r, e + PARAM) != 0 {
            let properties = word(r, e + PROP) | 0x500;
            set(r, e + PROP, properties);
            let point = Service::SetProjectileSpawnPoint { x: word(r, e + X), y: word(r, e + Y) };
            s.call(r, point)?;
            s.call(r, Service::SpawnEprojWithRoomGfx { definition: 0xe509, parameter: 3 })?;
            set(r, e + A, 0xb844);
            set(r, e + F, 1);
        } else {
            s.call(r, Service::RinkasDeathAnimation(0))?;
        }
    }
    Ok(())
}
