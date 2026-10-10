//! Samus projectile data, animation, lifetime and inherited velocity.
//! Translated from pinned snesrev/sm banks $90/$93. Upstream MIT notice applies.
//! Terrain-impact flight and power-bomb HDMA remain external gameplay systems.
use crate::{Wram, audio};

const X: usize = 0xb64;
const Y: usize = 0xb78;
const SUB_X: usize = 0xb8c;
const SUB_Y: usize = 0xba0;
const RADIUS_X: usize = 0xbb4;
const RADIUS_Y: usize = 0xbc8;
const SPEED_X: usize = 0xbdc;
const SPEED_Y: usize = 0xbf0;
const DIRECTION: usize = 0xc04;
const TYPE: usize = 0xc18;
const DAMAGE: usize = 0xc2c;
const INSTRUCTION: usize = 0xc40;
const TIMER: usize = 0xc54;
const PRE_INSTRUCTION: usize = 0xc68;
const VARIABLE: usize = 0xc7c;
const SPRITEMAP: usize = 0xcb8;
const COOLDOWN: usize = 0xccc;
const COUNT: usize = 0xcce;
const BOMB_COUNT: usize = 0xcd2;
const INDEX: usize = 0xdde;
const EMPTY: u16 = 0xb169;
const CINEMATIC_FUNCTION: usize = 0x1f51;
const EARTHQUAKE_TYPE: usize = 0x183e;
const EARTHQUAKE_TIMER: usize = 0x1840;
const INHERITED_RIGHT_PACKED: usize = 0xdad;
const INHERITED_LEFT_PACKED: usize = 0xda9;
const INHERITED_DOWN_PACKED: usize = 0xdb5;
const SAMUS_MOVED_UP: usize = 0xdb2;
const INHERITED_UP_PACKED: usize = 0xdb1;
const SAMUS_POSE: usize = 0xa1c;
const PREVIOUS_POSE: usize = 0xa24;
const DIRECTION_OVERRIDE: usize = 0xb5e;
const SAMUS_MOVEMENT_TYPE: usize = 0xa1f;
const SAMUS_X: usize = 0xaf6;
const SAMUS_Y: usize = 0xafa;
const CAMERA_X: usize = 0x911;
const CAMERA_Y: usize = 0x915;
const HUD_ITEM: usize = 0x9d2;
type Result<T> = std::result::Result<T, &'static str>;
fn slot(k: u16) -> Result<usize> {
    if k > 18 || k & 1 != 0 {
        return Err("projectile slot must be even and at most 18");
    }
    Ok(usize::from(k))
}
fn rom_byte(rom: &[u8], address: u32) -> Result<u8> {
    if address & 0xffff < 0x8000 {
        return Err("projectile ROM pointer below LoROM window");
    }
    let offset = (((address >> 16) & 0x7f) * 0x8000 + (address & 0x7fff)) as usize;
    rom.get(offset).copied().ok_or("projectile ROM extent")
}
fn rom_word(rom: &[u8], address: u32) -> Result<u16> {
    if address & 0xffff == 0xffff {
        return Err("projectile word crosses bank");
    }
    Ok(u16::from_le_bytes([rom_byte(rom, address)?, rom_byte(rom, address + 1)?]))
}
fn bank_word(rom: &[u8], pointer: u16, delta: u16) -> Result<u16> {
    let address = pointer.checked_add(delta).ok_or("projectile data crosses bank")?;
    rom_word(rom, 0x930000 | u32::from(address))
}
fn direction(ram: &Wram<'_>, k: usize) -> Result<usize> {
    let d = usize::from(ram.word(DIRECTION + k) & 15);
    if d >= 10 {
        return Err("invalid ten-direction projectile index");
    }
    Ok(d)
}
fn damage_instruction(
    ram: &mut Wram<'_>,
    rom: &[u8],
    k: usize,
    pointer: u16,
    direction: usize,
    radii: bool,
) -> Result<()> {
    let damage = bank_word(rom, pointer, 0)?;
    if damage & 0x8000 != 0 {
        return Err("upstream invalid projectile damage crash");
    }
    let instruction = bank_word(rom, pointer, 2 + 2 * direction as u16)?;
    let radius = if radii {
        Some((
            rom_byte(rom, 0x930000 | u32::from(instruction.checked_add(4).ok_or("instruction extent")?))?,
            rom_byte(rom, 0x930000 | u32::from(instruction.checked_add(5).ok_or("instruction extent")?))?,
        ))
    } else {
        None
    };
    ram.set_word(DAMAGE + k, damage);
    ram.set_word(INSTRUCTION + k, instruction);
    if let Some((x, y)) = radius {
        ram.set_word(RADIUS_X + k, u16::from(x));
        ram.set_word(RADIUS_Y + k, u16::from(y));
    }
    ram.set_word(TIMER + k, 1);
    Ok(())
}
pub fn initialize(ram: &mut Wram<'_>, rom: &[u8], k: u16) -> Result<()> {
    let k = slot(k)?;
    let t = ram.word(TYPE + k);
    let d = direction(ram, k)?;
    let (table, index) = if t & 0xf00 != 0 {
        (0x9383f1, (t >> 8) & 15)
    } else if t & 16 != 0 {
        (0x9383d9, t & 15)
    } else {
        (0x9383c1, t & 15)
    };
    if (table == 0x9383f1 && index > 8) || (table != 0x9383f1 && index > 11) {
        return Err("projectile data table index");
    }
    damage_instruction(ram, rom, k, rom_word(rom, table + 2 * u32::from(index))?, d, true)
}
pub fn clear(ram: &mut Wram<'_>, k: u16) -> Result<()> {
    let offset = slot(k)?;
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
        VARIABLE,
        SPRITEMAP,
    ] {
        ram.set_word(base + offset, 0);
    }
    ram.set_word(PRE_INSTRUCTION + offset, EMPTY);
    let count = if k >= 10 { BOMB_COUNT } else { COUNT };
    let next = ram.word(count).wrapping_sub(1);
    ram.set_word(count, if next & 0x8000 != 0 { 0 } else { next });
    Ok(())
}
fn kill_offset(ram: &mut Wram<'_>, k: usize, d: usize) {
    let dx = [0, 1, 1, 1, 0, 0, -1, -1, -1, 0][d];
    let dy = [-1, -1, 0, 1, 1, 1, 1, 0, -1, -1][d];
    ram.set_word(X + k, ram.word(X + k).wrapping_add((dx * i32::from(ram.word(RADIUS_X + k))) as u16));
    ram.set_word(Y + k, ram.word(Y + k).wrapping_add((dy * i32::from(ram.word(RADIUS_Y + k))) as u16));
}
fn kill_inner(ram: &mut Wram<'_>, rom: &[u8], k: usize) -> Result<()> {
    let t = ram.word(TYPE + k);
    let pointer = if t & 0xf00 != 0 {
        if t & 0x200 != 0 { rom_word(rom, 0x938693)? } else { rom_word(rom, 0x93867f)? }
    } else {
        rom_word(rom, 0x93867b)?
    };
    if t & 0xf00 != 0 {
        if ram.word(CINEMATIC_FUNCTION) == 0 {
            audio::dispatch(ram, "QueueSfx2_Max6", 7, 0)?;
        }
        ram.set_word(TYPE + k, (t & 0xf0ff) | 0x800);
        if t & 0x200 != 0 {
            ram.set_word(EARTHQUAKE_TYPE, 20);
            ram.set_word(EARTHQUAKE_TIMER, 30);
        }
        if ram.word(COOLDOWN).wrapping_sub(21) & 0x8000 == 0 {
            ram.set_word(COOLDOWN, 20);
        }
    } else {
        audio::dispatch(ram, "QueueSfx2_Max6", 12, 0)?;
        ram.set_word(TYPE + k, (t & 0xf0ff) | 0x700);
    }
    ram.set_word(INSTRUCTION + k, pointer);
    ram.set_word(TIMER + k, 1);
    ram.set_word(DAMAGE + k, 8);
    Ok(())
}
pub fn kill(ram: &mut Wram<'_>, rom: &[u8], k: u16) -> Result<()> {
    let offset = slot(k)?;
    let t = ram.word(TYPE + offset);
    if t & 0xf00 != 0 {
        if ((t >> 8) & 15).wrapping_sub(3) & 0x8000 == 0 {
            return clear(ram, k);
        }
    } else {
        let d = direction(ram, offset)?;
        kill_offset(ram, offset, d);
    }
    kill_inner(ram, rom, offset)?;
    ram.set_word(PRE_INSTRUCTION + offset, EMPTY);
    Ok(())
}
pub fn initialize_speed(ram: &mut Wram<'_>, k: u16, speed: u16) -> Result<()> {
    let k = slot(k)?;
    let d = direction(ram, k)?;
    // These intentionally unaligned words preserve the original overlapping views.
    let right = ram.word(INHERITED_RIGHT_PACKED);
    let left = ram.word(INHERITED_LEFT_PACKED);
    let down = ram.word(INHERITED_DOWN_PACKED);
    let up = if ram.byte(SAMUS_MOVED_UP) != 0 { (ram.word(INHERITED_UP_PACKED) >> 2) | 0xc000 } else { 0 };
    let x = match d {
        1..=3 => right.wrapping_add(speed),
        6..=8 => left.wrapping_sub(speed),
        _ => 0,
    };
    let y = match d {
        0 | 1 | 8 | 9 => up.wrapping_sub(speed),
        3..=6 => down.wrapping_add(speed),
        _ => 0,
    };
    ram.set_word(SUB_X + k, 0);
    ram.set_word(SUB_Y + k, 0);
    ram.set_word(SPEED_X + k, x);
    ram.set_word(SPEED_Y + k, y);
    Ok(())
}
/// Execute the complete $93 instruction language (delete/goto/data), bounded to
/// reject malformed cyclic bytecode before handing an unchecked fixture to C.
pub fn run_instructions(ram: &mut Wram<'_>, rom: &[u8]) -> Result<()> {
    let raw = ram.word(INDEX);
    let k = slot(raw)?;
    let timer = ram.word(TIMER + k);
    if timer != 1 {
        ram.set_word(TIMER + k, timer.wrapping_sub(1));
        return Ok(());
    }
    let mut pointer = ram.word(INSTRUCTION + k);
    for _ in 0..1024 {
        let timer = bank_word(rom, pointer, 0)?;
        if timer & 0x8000 == 0 {
            let spritemap = bank_word(rom, pointer, 2)?;
            let packed = bank_word(rom, pointer, 4)?;
            let next = pointer.checked_add(8).ok_or("instruction pointer wraps")?;
            ram.set_word(TIMER + k, timer);
            ram.set_word(SPRITEMAP + k, spritemap);
            ram.set_word(RADIUS_X + k, packed & 255);
            ram.set_word(RADIUS_Y + k, packed >> 8);
            ram.set_word(INSTRUCTION + k, next);
            return Ok(());
        }
        match timer {
            0x822f => {
                ram.set_word(TIMER + k, 0);
                return clear(ram, raw);
            }
            0x8239 => pointer = bank_word(rom, pointer, 2)?,
            _ => return Err("unknown projectile instruction"),
        }
    }
    Err("cyclic projectile bytecode")
}
fn instruction_value(ram: &Wram<'_>, rom: &[u8], k: usize) -> Result<u16> {
    let ip = ram.word(INSTRUCTION + k);
    let current = ram.word(TIMER + k) == 1 && bank_word(rom, ip, 0)? & 0x8000 == 0;
    bank_word(rom, if current { ip } else { ip.checked_sub(8).ok_or("previous projectile instruction underflow")? }, 6)
}
pub fn init_position(ram: &mut Wram<'_>, rom: &[u8], k: u16) -> Result<u8> {
    let k = slot(k)?;
    let pose = ram.word(SAMUS_POSE);
    if pose > 0xfc {
        return Err("Samus pose index");
    }
    let params = 0x91b629 + 8 * u32::from(pose);
    let override_dir = ram.word(DIRECTION_OVERRIDE);
    let mut d = if override_dir != 0 { override_dir as u8 } else { rom_byte(rom, params + 3)? };
    if override_dir == 0 && d & 0xf0 != 0 {
        let previous = ram.word(PREVIOUS_POSE);
        if previous > 0xfc {
            return Err("previous Samus pose index");
        }
        if d == 16 {
            d = rom_byte(rom, 0x91b629 + 8 * u32::from(previous) + 3)?;
        }
        if d & 0xf0 != 0 {
            ram.set_word(COUNT, ram.word(COUNT).wrapping_sub(1));
            return Ok(1);
        }
    }
    let index = usize::from(d & 15);
    if index >= 10 {
        return Err("projectile origin direction");
    }
    let offset = rom_byte(rom, params + 4)?;
    let (xs, ys) = if pose == 0x75 || pose == 0x76 || ram.byte(SAMUS_MOVEMENT_TYPE) == 1 {
        ([2i16, 15, 15, 13, 2, -5, -13, -13, -15, -2], [-8i16, -16, -2, 1, 13, 13, 1, -2, -16, -8])
    } else {
        ([2i16, 13, 11, 13, 2, -5, -14, -11, -19, -2], [-8i16, -13, 1, 4, 13, 13, 4, 1, -19, -8])
    };
    if override_dir != 0 {
        ram.set_word(DIRECTION_OVERRIDE, 0);
    }
    ram.set_word(DIRECTION + k, u16::from(d));
    ram.set_word(X + k, ram.word(SAMUS_X).wrapping_add(xs[index] as u16));
    ram.set_word(Y + k, ram.word(SAMUS_Y).wrapping_add(ys[index] as u16).wrapping_sub(u16::from(offset)));
    Ok(0)
}
pub fn missile_link(ram: &mut Wram<'_>, rom: &[u8]) -> Result<()> {
    let parent = slot(ram.word(INDEX))?;
    let Some(k) = (0..10).step_by(2).find(|k| ram.word(DAMAGE + k) == 0) else {
        return Ok(());
    };
    ram.set_word(TYPE + k, ram.word(TYPE + k) | 0x8200);
    ram.set_word(X + k, ram.word(X + parent));
    ram.set_word(Y + k, ram.word(Y + parent));
    ram.set_word(DIRECTION + k, ram.word(DIRECTION + parent));
    init_position(ram, rom, k as u16)?;
    let kind = (ram.word(TYPE + k) >> 8) & 15;
    if kind > 3 {
        return Err("super-missile link table index");
    }
    let pointer = rom_word(rom, 0x93842b + 2 * u32::from(kind))?;
    damage_instruction(ram, rom, k, pointer, 0, false)?;
    ram.set_word(PRE_INSTRUCTION + k, 0xb075);
    ram.set_word(VARIABLE + parent, k as u16 + (ram.word(VARIABLE + parent) & 0xff00));
    ram.set_word(COUNT, ram.word(COUNT).wrapping_add(1));
    Ok(())
}
pub fn missile_accelerate(ram: &mut Wram<'_>, rom: &[u8], k: u16) -> Result<()> {
    let offset = slot(k)?;
    let d = direction(ram, offset)?;
    let v = ram.word(VARIABLE + offset);
    if v & 0xff00 != 0 {
        let table = if ram.word(TYPE + offset) & 0x200 != 0 { 0x90c32b } else { 0x90c303 };
        let x = rom_word(rom, table + 4 * d as u32)?;
        let y = rom_word(rom, table + 4 * d as u32 + 2)?;
        ram.set_word(SPEED_X + offset, ram.word(SPEED_X + offset).wrapping_add(x));
        ram.set_word(SPEED_Y + offset, ram.word(SPEED_Y + offset).wrapping_add(y));
    } else {
        let next = v.wrapping_add(0x100);
        ram.set_word(VARIABLE + offset, next);
        if next & 0xff00 != 0 {
            initialize_speed(ram, k, next)?;
            if ram.word(TYPE + offset) & 0x200 != 0 {
                missile_link(ram, rom)?;
            }
        }
    }
    Ok(())
}
pub fn reflect(ram: &mut Wram<'_>, rom: &[u8], k: u16) -> Result<()> {
    let offset = slot(k)?;
    let t = ram.word(TYPE + offset);
    if t & 0x100 != 0 {
        initialize(ram, rom, k)?;
        ram.set_word(PRE_INSTRUCTION + offset, 0xaf68);
        ram.set_word(VARIABLE + offset, 240);
    } else if t & 0x200 != 0 {
        let linked = ram.byte(VARIABLE + offset);
        slot(u16::from(linked))?;
        clear(ram, u16::from(linked))?;
        initialize(ram, rom, k)?;
        ram.set_word(PRE_INSTRUCTION + offset, 0xafe5);
        ram.set_word(VARIABLE + offset, 240);
    } else {
        if t & 15 > 11 {
            return Err("charged beam pre-instruction index");
        }
        let d = direction(ram, offset)?;
        initialize_speed(ram, k, if matches!(d, 1 | 3 | 6 | 8) { 0x2ab } else { 0x400 })?;
        initialize(ram, rom, k)?;
        ram.set_word(PRE_INSTRUCTION + offset, if ram.word(TYPE + offset) & 1 != 0 { 0xb0c3 } else { 0xaef3 });
    }
    Ok(())
}
pub fn dispatch_rom(ram: &mut Wram<'_>, rom: &[u8], symbol: &str, a: i32, b: i32) -> Result<u32> {
    let k = a as u16;
    match symbol {
        "Missile_Func1" => missile_accelerate(ram, rom, k)?,
        "Missile_Func2" => missile_link(ram, rom)?,
        "ProjectileReflection" => reflect(ram, rom, k)?,
        "ProjPreInstr_Func1" => {
            let offset = slot(k)?;
            if ram.word(DIRECTION + offset) & 0xf0 != 0 {
                clear(ram, k)?;
                for i in (0..10).step_by(2).rev() {
                    if ram.word(TYPE + i) & 0xfff == 512 {
                        clear(ram, i as u16)?;
                    }
                }
            }
        }
        "Projectile_Func4" => {
            let offset = slot(k)?;
            if ram.byte(VARIABLE + offset + 1) != 0 && ram.word(TYPE + offset) & 0xf00 == 2048 {
                clear(ram, u16::from(ram.byte(VARIABLE + offset)))?;
            }
        }
        "InitializeProjectile" => {
            initialize(ram, rom, k)?;
        }
        "InitializeInstrForMissile"
        | "InitializeInstrForSuperMissile"
        | "InitializeSbaProjectile"
        | "InitializeShinesparkEchoOrSpazerSba" => {
            let offset = slot(k)?;
            let t = ram.word(TYPE + offset);
            let (table, index, d) = match symbol {
                "InitializeInstrForMissile" => (0x9383f1, (t >> 8) & 15, 0),
                "InitializeInstrForSuperMissile" => (0x93842b, (t >> 8) & 15, 0),
                "InitializeSbaProjectile" => (0x938413, t & 15, 0),
                _ => (0x938403, (t as u8).checked_sub(34).ok_or("SBA type index")? as u16, direction(ram, offset)?),
            };
            let limit = match table {
                0x9383f1 => 8,
                0x93842b => 3,
                0x938413 => 11,
                _ => 7,
            };
            if index > limit {
                return Err("projectile initialization table index");
            }
            damage_instruction(ram, rom, offset, rom_word(rom, table + 2 * u32::from(index))?, d, false)?;
        }
        "InitializeBombExplosion" => {
            let offset = slot(k)?;
            let pointer = rom_word(rom, 0x938683)?;
            ram.set_word(INSTRUCTION + offset, pointer);
            ram.set_word(TIMER + offset, 1);
        }
        "ClearProjectile" => clear(ram, k)?,
        "KillProjectile" => kill(ram, rom, k)?,
        "KillProjectileInner" => kill_inner(ram, rom, slot(k)?)?,
        "RunProjectileInstructions" => run_instructions(ram, rom)?,
        "Proj93Instr_Delete" => {
            clear(ram, k)?;
            return Ok(0);
        }
        "Proj93Instr_Goto" => return Ok(u32::from(bank_word(rom, b as u16, 0)?)),
        "ProjectileInsts_GetValue" => return Ok(u32::from(instruction_value(ram, rom, slot(k)?)?)),
        "InitializeProjectileSpeed" => initialize_speed(ram, k, b as u16)?,
        "InitializeProjectileSpeedOfType" => initialize_speed(ram, k, 0)?,
        "SetInitialProjectileSpeed" => {
            let offset = slot(k)?;
            if ram.word(TYPE + offset) & 15 > 11 {
                return Err("beam speed type index");
            }
            let d = direction(ram, offset)?;
            initialize_speed(ram, k, if matches!(d, 1 | 3 | 6 | 8) { 0x2ab } else { 0x400 })?;
        }
        "InitProjectilePositionDirection" => return Ok(u32::from(init_position(ram, rom, k)?)),
        "DeleteProjectileIfFarOffScreen" => {
            let raw = ram.word(INDEX);
            let offset = slot(raw)?;
            let x = ram.word(X + offset).wrapping_sub(ram.word(CAMERA_X));
            let y = ram.word(Y + offset).wrapping_sub(ram.word(CAMERA_Y));
            if x.wrapping_add(64) & 0x8000 == 0
                && x.wrapping_sub(320) & 0x8000 != 0
                && y.wrapping_add(64) & 0x8000 == 0
                && y.wrapping_sub(320) & 0x8000 != 0
            {
                return Ok(0);
            }
            clear(ram, raw)?;
            return Ok(1);
        }
        "Bomb_Func2" => {
            let raw = ram.word(INDEX);
            let offset = slot(raw)?;
            let v = ram.word(VARIABLE + offset);
            if v != 0 {
                let next = v.wrapping_sub(1);
                if next == 0 {
                    let pointer = rom_word(rom, 0x938683)?;
                    audio::dispatch(ram, "QueueSfx2_Max6", 8, 0)?;
                    ram.set_word(INSTRUCTION + offset, pointer);
                    ram.set_word(TIMER + offset, 1);
                } else if next == 15 {
                    ram.set_word(INSTRUCTION + offset, ram.word(INSTRUCTION + offset).wrapping_add(28));
                }
                ram.set_word(VARIABLE + offset, next);
            }
        }
        "Samus_CanFireBeam" | "Samus_CanFireSuperMissile" => {
            let limit = if symbol == "Samus_CanFireSuperMissile" && ram.word(HUD_ITEM) == 2 { 4 } else { 5 };
            if ram.word(COUNT).wrapping_sub(limit) & 0x8000 == 0 || ram.byte(COOLDOWN) != 0 {
                return Ok(0);
            }
            ram.set_word(COOLDOWN, 1);
            ram.set_word(COUNT, ram.word(COUNT).wrapping_add(1));
            return Ok(1);
        }
        name if name.starts_with("KillProjectileFunc_") => {
            let offset = slot(k)?;
            let d = match name {
                "KillProjectileFunc_0" => 0,
                "KillProjectileFunc_1" => 1,
                "KillProjectileFunc_2" => 2,
                "KillProjectileFunc_3" => 3,
                "KillProjectileFunc_4" => 4,
                "KillProjectileFunc_6" => 6,
                "KillProjectileFunc_7" => 7,
                "KillProjectileFunc_8" => 8,
                _ => return Err("unknown kill direction"),
            };
            kill_offset(ram, offset, d);
        }
        _ => return Err("unknown projectile routine"),
    }
    Ok(0)
}
