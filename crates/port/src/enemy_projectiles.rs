//! Enemy projectile allocation and room-effect initialization, pinned bank86.
use crate::Wram;
/// Upstream projectile scratch globals live outside canonical WRAM.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProjectileHost {
    pub x: u16,
    pub y: u16,
    pub rect_x: u16,
    pub rect_y: u16,
    pub rect_width: u16,
    pub rect_height: u16,
    pub r22: u16,
    pub enemy_header: u16,
}
impl ProjectileHost {
    pub fn words(self) -> [u16; 8] {
        [self.x, self.y, self.rect_x, self.rect_y, self.rect_width, self.rect_height, self.r22, self.enemy_header]
    }
    pub fn from_words(v: [u16; 8]) -> Self {
        Self {
            x: v[0],
            y: v[1],
            rect_x: v[2],
            rect_y: v[3],
            rect_width: v[4],
            rect_height: v[5],
            r22: v[6],
            enemy_header: v[7],
        }
    }
}

fn word(rom: &[u8], p: u16) -> Result<u16, &'static str> {
    if p < 0x8000 || p == 0xffff {
        return Err("enemy projectile ROM extent");
    }
    let o = 0x30000 + usize::from(p & 0x7fff);
    let b = rom.get(o..o + 2).ok_or("missing enemy projectile ROM")?;
    Ok(u16::from_le_bytes([b[0], b[1]]))
}
pub fn enable(r: &mut Wram<'_>) {
    r.set_word(0x198d, r.word(0x198d) | 0x8000);
}
pub fn disable(r: &mut Wram<'_>) {
    r.set_word(0x198d, r.word(0x198d) & 0x7fff);
}
pub fn clear(r: &mut Wram<'_>) {
    for k in (0..=34).step_by(2) {
        r.set_word(0x1997 + k, 0);
    }
}
fn spawn(r: &mut Wram<'_>, rom: &[u8], definition: u16, parameter: u16, gfx: u16) -> Result<(), &'static str> {
    spawn_context(r, rom, definition, parameter, gfx, None)
}
fn spawn_context(
    r: &mut Wram<'_>,
    rom: &[u8],
    definition: u16,
    parameter: u16,
    gfx: u16,
    host: Option<&ProjectileHost>,
) -> Result<(), &'static str> {
    let free = (0..18).rev().map(|i| i * 2).find(|k| r.word(0x1997 + k) == 0);
    let Some(k) = free else {
        r.set_word(0x1993, parameter);
        return Ok(());
    };
    let init = word(rom, definition)?;
    if matches!(init, 0x947f | 0xe468 | 0xef29 | 0xdc8d | 0xa009 | 0xa098) {
        host.ok_or("projectile initializer requires host context")?;
        if init == 0xa009 {
            let enemy = r.word(0xe54);
            if enemy > 0x7c0 || enemy & 63 != 0 {
                return Err("invalid enemy slot");
            }
            let header = r.word(0xf78 + usize::from(enemy));
            let p = 0x100000 + usize::from(header & 0x7fff) + 6;
            rom.get(p..p + 2).ok_or("missing pirate laser properties")?;
        }
        if init == 0xef29 {
            for drop in 1..6 {
                word(rom, 0xef04 + drop * 2)?;
            }
            let header = host.unwrap().enemy_header;
            if header != 0 {
                let p = 0x100000 + usize::from(header & 0x7fff) + 0x3a;
                let value = rom.get(p..p + 2).ok_or("missing enemy drop definition")?;
                let pointer = u16::from_le_bytes([value[0], value[1]]);
                if pointer != 0 {
                    let p = 0x1a0000 + usize::from(pointer & 0x7fff);
                    rom.get(p..p + 6).ok_or("missing enemy drop chances")?;
                }
            }
        }
        if init == 0xe468 {
            if parameter >= 30 {
                return Err("dust script index");
            }
            word(
                rom,
                0xe42c_u16
                    .checked_add(parameter.checked_mul(2).ok_or("dust script extent")?)
                    .ok_or("dust script extent")?,
            )?;
        }
    } else {
        validate(r, rom, init, parameter)?;
    }
    let mut fields = [0; 4];
    for (i, v) in fields.iter_mut().enumerate() {
        *v = word(rom, definition.checked_add(2 + 2 * i as u16).ok_or("enemy projectile definition extent")?)?;
    }
    r.set_word(0x1993, parameter);
    r.set_word(0x19bb + k, gfx);
    r.set_word(0x1997 + k, definition);
    for (p, v) in [0x1a03, 0x1b47, 0x1bb3, 0x1bd7].into_iter().zip(fields) {
        r.set_word(p + k, v);
    }
    r.set_word(0x1b8f + k, 1);
    r.set_word(0x1b6b + k, 0x8000);
    for p in [0x1aff, 0x1b23, 0x19df, 0x1a27, 0x1a6f, 0x1bfb] {
        r.set_word(p + k, 0);
    }
    if let Some(host) = host.filter(|_| matches!(init, 0x947f | 0xe468 | 0xef29 | 0xdc8d | 0xa009 | 0xa098)) {
        r.set_word(0x1a4b + k, host.x);
        r.set_word(0x1a93 + k, host.y);
        match init {
            0x947f => {
                r.set_word(0x19bb + k, 2560);
                r.set_word(0x1aff + k, parameter);
                r.set_word(0x1ab7 + k, 0);
                r.set_word(0x1adb + k, 0);
            }
            0xa009 => {
                r.set_word(0x1b47 + k, if host.r22 != 0 { 0x9f7d } else { 0x9f41 });
                r.set_word(0x1a03 + k, 0xa05b);
                let enemy = 0xf78 + usize::from(r.word(0xe54));
                let p = 0x100000 + usize::from(r.word(enemy) & 0x7fff) + 6;
                r.set_word(0x1bd7 + k, u16::from_le_bytes([rom[p], rom[p + 1]]) | 0x1000);
                r.set_word(0x1aff + k, r.word(enemy + 0x3c));
                crate::audio::queue_sfx(r, 1, 0x6706)?;
            }
            0xa098 => {
                r.set_word(0x1a4b + k, host.rect_x.wrapping_add(host.rect_width));
                r.set_word(0x1a93 + k, host.rect_y.wrapping_add(host.rect_height));
                r.set_word(0x1b47 + k, if parameter != 0 { 0x9fe1 } else { 0x9fb9 });
                r.set_word(0x1a03 + k, 0xa05b);
                r.set_word(0x1aff + k, 2048);
                r.set_word(0x1b23 + k, 1);
            }
            0xdc8d => {
                r.set_word(0x1b23 + k, host.x);
                r.set_word(0x19bb + k, 512);
            }
            0xe468 => r.set_word(0x1b47 + k, word(rom, 0xe42c + parameter * 2)?),
            _ => {
                r.set_word(0x19bb + k, 0);
                r.set_word(0xf3c8 + k, host.enemy_header);
                let drop = crate::enemy_projectile_instructions::random_drop(r, rom, k as u16)?;
                if drop != 0 && (drop.wrapping_sub(6) as i16) < 0 {
                    r.set_word(0x1aff + k, drop * 2);
                    r.set_word(0x1b47 + k, word(rom, 0xef04 + drop * 2)?);
                    r.set_word(0x1b8f + k, 1);
                    r.set_word(0x1b23 + k, 400);
                    r.set_word(0xf410 + k, 0xffff);
                } else {
                    r.set_word(0x1b47 + k, 0xeca3);
                    r.set_word(0x1b8f + k, 1);
                    r.set_word(0x1bd7 + k, 0x3000);
                    r.set_word(0x1a03 + k, 0xefdf);
                }
            }
        }
    } else {
        initialize(r, rom, init, parameter, k)?;
    }
    Ok(())
}
pub fn spawn_room(r: &mut Wram<'_>, rom: &[u8], definition: u16, parameter: u16) -> Result<(), &'static str> {
    spawn(r, rom, definition, parameter, 0)
}
pub fn spawn_room_with_host(
    r: &mut Wram<'_>,
    rom: &[u8],
    definition: u16,
    parameter: u16,
    host: &mut ProjectileHost,
) -> Result<(), &'static str> {
    spawn_context(r, rom, definition, parameter, 0, Some(host))
}
pub fn spawn_enemy_drops(
    r: &mut Wram<'_>,
    rom: &[u8],
    host: &mut ProjectileHost,
    header: u16,
    k: u16,
    parameter: u16,
) -> Result<(), &'static str> {
    if k > 0x7c0 {
        return Err("enemy drop source extent");
    }
    host.enemy_header = header;
    let enemy = 0xf78 + usize::from(k);
    spawn_context(r, rom, 0xf337, parameter, r.word(enemy + 0x1e) | r.word(enemy + 0x20), Some(host))
}
pub fn dispatch_host(
    r: &mut Wram<'_>,
    rom: &[u8],
    host: &mut ProjectileHost,
    name: &str,
    a: i32,
    b: i32,
) -> Result<u32, &'static str> {
    if matches!(name, "SpacePirates_Instr_15" | "SpacePirates_Instr_FireLaserL" | "SpacePirates_Instr_FireLaserR") {
        pirate_instruction(r, rom, host, name, b as u16)
    } else if name == "SpawnEprojWithRoomGfx" {
        spawn_room_with_host(r, rom, a as u16, b as u16, host)?;
        Ok(0)
    } else if name == "EprojPreinstruction" {
        crate::enemy_projectile_instructions::preinstruction_with_host(r, rom, b as u16, a as u16, host)?;
        Ok(0)
    } else if name == "EprojRunAll" {
        crate::enemy_projectile_instructions::run_all_with_host(r, rom, 4096, host)?;
        Ok(0)
    } else if name == "EprojInstruction" {
        crate::enemy_projectile_instructions::dispatch_with_host(r, rom, host, name, a, b)
    } else {
        dispatch_rom(r, rom, name, a, b)
    }
}
pub fn spawn_with_gfx_host(
    r: &mut Wram<'_>,
    rom: &[u8],
    parameter: u16,
    enemy_slot: u16,
    definition: u16,
    host: &mut ProjectileHost,
) -> Result<(), &'static str> {
    if enemy_slot > 0x7c0 || enemy_slot & 63 != 0 {
        return Err("invalid enemy slot");
    }
    let enemy = 0xf78 + usize::from(enemy_slot);
    spawn_context(r, rom, definition, parameter, r.word(enemy + 0x1e) | r.word(enemy + 0x20), Some(host))
}
fn pirate_instruction(
    r: &mut Wram<'_>,
    rom: &[u8],
    host: &mut ProjectileHost,
    name: &str,
    pc: u16,
) -> Result<u32, &'static str> {
    if pc < 0x8000 || (name == "SpacePirates_Instr_15" && pc > 0xfff8) {
        return Err("invalid pirate instruction pointer");
    }
    let slot = r.word(0xe54);
    if slot > 0x7c0 || slot & 63 != 0 {
        return Err("invalid enemy slot");
    }
    let enemy = 0xf78 + usize::from(slot);
    if name == "SpacePirates_Instr_15" {
        let p = 0x190000 + usize::from(pc & 0x7fff);
        let data = rom.get(p..p + 6).ok_or("missing pirate claw operands")?;
        let values: [u16; 3] = std::array::from_fn(|i| u16::from_le_bytes([data[i * 2], data[i * 2 + 1]]));
        host.rect_x = r.word(enemy + 2);
        host.rect_y = r.word(enemy + 6);
        host.rect_width = values[1];
        host.rect_height = values[2];
        spawn_with_gfx_host(r, rom, values[0], slot, 0xa189, host)?;
        Ok(0xb20000 | u32::from(pc.wrapping_add(6)))
    } else {
        let right = name == "SpacePirates_Instr_FireLaserR";
        if !right {
            let p = 0x100000 + usize::from(r.word(enemy) & 0x7fff) + 6;
            let data = rom.get(p..p + 2).ok_or("missing pirate laser properties")?;
            r.set_word(0x1993, u16::from_le_bytes([data[0], data[1]]));
        }
        host.x = r.word(enemy + 2).wrapping_add(if right { 24 } else { (-24i16) as u16 });
        host.y = r.word(enemy + 6).wrapping_sub(16);
        host.r22 = u16::from(right);
        spawn_room_with_host(r, rom, 0xa17b, u16::from(right), host)?;
        Ok(0xb20000 | u32::from(pc))
    }
}
pub fn spawn_with_gfx(
    r: &mut Wram<'_>,
    rom: &[u8],
    parameter: u16,
    enemy_slot: u16,
    definition: u16,
) -> Result<(), &'static str> {
    if enemy_slot > 0x7c0 || enemy_slot & 63 != 0 {
        return Err("invalid enemy slot");
    }
    let p = 0xf78 + enemy_slot as usize;
    spawn(r, rom, definition, parameter, r.word(p + 0x1e) | r.word(p + 0x20))
}
pub fn dispatch_rom(r: &mut Wram<'_>, rom: &[u8], name: &str, a: i32, b: i32) -> Result<u32, &'static str> {
    match name {
        "SpawnEprojWithRoomGfx" => spawn_room(r, rom, a as u16, b as u16)?,
        "EnableEprojs" => enable(r),
        "DisableEprojs" => disable(r),
        "ClearEprojs" => clear(r),
        _ => return Err("unsupported enemy projectile routine"),
    };
    Ok(0)
}

fn validate(r: &Wram<'_>, rom: &[u8], init: u16, param: u16) -> Result<(), &'static str> {
    let i = param as usize / 2;
    let table = match init {
        0xef89 if param < 5 && r.word(0xe54) <= 0x7c0 && r.word(0xe54) & 63 == 0 => {
            word(rom, 0xefd5 + param * 2)?;
            None
        }
        0xec62 if r.word(0xe54) <= 0x7c0 && r.word(0xe54) & 63 == 0 => None,
        0xebc6 if r.word(0xe54) <= 0x7c0 && r.word(0xe54) & 63 == 0 => {
            let angle = usize::from(r.byte(0x60b));
            for phase in [64, 128] {
                let p = 0x1037ee + ((angle + phase) & 127) * 2;
                rom.get(p..p + 2).ok_or("missing projectile quarter circle ROM")?;
            }
            None
        }
        0xea31 if r.word(0xfa8) <= 0x7c0 && r.word(0xfa8) & 63 == 0 => {
            word(rom, if r.word(0xfa8) == 0 { 0xea21 } else { 0xea01 })?;
            None
        }
        0x8a39 if r.word(0xe54) <= 0x7c0 && r.word(0xe54) & 63 == 0 => {
            let velocity = r.word(0xfae + usize::from(r.word(0xe54)));
            word(rom, 0x8a75 + ((velocity >> 8) & 6))?;
            None
        }
        0x8acd | 0x8af1 | 0x8b15 | 0x8b39 if r.word(0xe54) <= 0x7c0 && r.word(0xe54) & 63 == 0 => None,
        0xe5d0 | 0xe5d5 | 0xe5da | 0xe6ad if r.word(0x1c27) <= 78 && r.word(0x1c27) & 1 == 0 && r.word(0x7a5) != 0 => {
            None
        }
        0xdb18 | 0xe000 if r.word(0xe54) <= 0x7c0 && r.word(0xe54) & 63 == 0 => None,
        0xded6 if r.word(0xe54) <= 0x7c0 && r.word(0xe54) & 63 == 0 => {
            let row = u16::from(r.byte(0xfb6 + usize::from(r.word(0xe54))));
            if row >= 8 {
                return Err("Nami/Fune fireball velocity table extent");
            }
            word(rom, 0xdeb6 + row * 4)?;
            word(rom, 0xdeb8 + row * 4)?;
            None
        }
        0xdca3 if param < 5 => None,
        0xdcd4 if param < 4 => None,
        0xbb30 if param < 3 && r.word(0xe54) <= 0x7c0 && r.word(0xe54) & 63 == 0 => Some((0xbb1e, usize::from(param))),
        0xbb92 if r.word(0xe54) <= 0x7c0 && r.word(0xe54) & 63 == 0 => {
            let enemy = usize::from(r.word(0xe54));
            let linked = enemy + usize::from(r.word(0x8014 + enemy));
            if linked > 0x7c0 || linked & 63 != 0 {
                return Err("invalid Nuclear Waffle linked enemy slot");
            }
            None
        }
        0xbe4f if param < 12 => {
            let direction = word(rom, 0xbee1 + param * 2)?;
            word(rom, 0xbeb9 + (direction & 255) * 2)?;
            word(rom, 0xbec9 + param * 2)?;
            None
        }
        0xbf59 if param <= 35 && r.byte(0x1a6f + usize::from(param & !1)) < 8 => None,
        0xbda2 | 0xbd9c if r.word(0xe54) <= 0x7c0 && r.word(0xe54) & 63 == 0 => None,
        0xbbdb if r.word(0xe54) <= 0x7c0 && r.word(0xe54) & 63 == 0 => None,
        0xb88e | 0xb8f8 if param <= 6 && param & 1 == 0 => None,
        0xb87a | 0xb8b5 | 0xb8e8 if param <= 34 && param & 1 == 0 => None,
        0xb93e | 0xb951 | 0xb964 => None,
        0xb4ef if r.word(0xe54) <= 0x7c0 && r.word(0xe54) & 63 == 0 => None,
        0xb328 if r.word(0xe54) <= 0x7c0 && r.word(0xe54) & 63 == 0 => {
            for p in (0xb376..0xb38a).step_by(2) {
                word(rom, p)?;
            }
            None
        }
        0xb1ce if r.word(0xe54) <= 0x7c0 && r.word(0xe54) & 63 == 0 => {
            word(rom, 0xb209)?;
            word(rom, 0xb20b)?;
            None
        }
        0xb001 if r.word(0xe54) <= 0x7c0 && r.word(0xe54) & 63 == 0 => {
            for p in (0xb02f..0xb043).step_by(2) {
                word(rom, p)?;
            }
            None
        }
        0xae15 if r.word(0xe54) <= 0x7c0 && r.word(0xe54) & 63 == 0 => None,
        0xabeb | 0xac7c if r.word(0xe54) <= 0x7c0 && r.word(0xe54) & 63 == 0 => {
            let p = if init == 0xabeb { 0xac08 } else { 0xac99 };
            for offset in (0..20).step_by(2) {
                word(rom, p + offset)?;
            }
            None
        }
        0xa81b | 0xa871 => {
            let enemy = r.word(0xe54);
            if enemy > 0x7c0 || enemy & 63 != 0 {
                return Err("invalid Torizo enemy slot");
            }
            let delta = if r.word(0xfb4 + usize::from(enemy)) & 0x8000 == 0 { 4 } else { 2 };
            if init == 0xa81b && param.wrapping_add(delta) >> 1 >= 6 {
                return Err("Torizo explosion table extent");
            }
            None
        }
        0xa5d3 | 0xa65d if r.word(0xe54) <= 0x7c0 && r.word(0xe54) & 63 == 0 => {
            if init == 0xa5d3 {
                for p in (0xa64d..0xa65d).step_by(2) {
                    word(rom, p)?;
                }
            }
            None
        }
        0x9eb2 if param >> 1 < 3 && r.word(0xe54) <= 0x7c0 && r.word(0xe54) & 63 == 0 => None,
        0x9dec | 0x9e46 | 0x9e4b => {
            let enemy = r.word(0xe54);
            if enemy > 0x7c0 || enemy & 63 != 0 {
                return Err("invalid Mini-Kraid enemy slot");
            }
            if init != 0x9dec && r.word(0x780c + usize::from(enemy)) >> 1 >= 3 {
                return Err("Mini-Kraid spike height extent");
            }
            None
        }
        0x9cd8 => None,
        0x9ca3 | 0x9d0c if r.word(0xe54) <= 0x7c0 && r.word(0xe54) & 63 == 0 => None,
        0x9824
            if match param >> 8 {
                0 => true,
                2 => (param as u8) < 16,
                4 => param & 15 < 9,
                6 => (param as u8) < 8,
                _ => false,
            } =>
        {
            None
        }
        0x993a if param < 8 => None,
        0x94a0 | 0x94b4 | 0x94c8 | 0x94dc if param <= 34 && param & 1 == 0 => {
            for p in [0x9408, 0x940a, 0x940c] {
                word(rom, p)?;
            }
            None
        }
        0x9023 if r.word(0xe54) <= 0x7c0 && r.word(0xe54) & 63 == 0 => None,
        0x9286 => None,
        0x934d if r.word(0xfb4) < 4 => None,
        0xb49d | 0xd6a5 => None,
        0xce6d if i < 18 => None,
        0xa764 if i < 16 => Some((0xa7ab, i)),
        0xb62d if i < 19 => None,
        0xb683 if i < 3 => None,
        0xcdc5 if i < 3 => {
            for p in (0xce41..0xce61).step_by(2) {
                word(rom, p)?;
            }
            None
        }
        0xd6c9 if i < 10 => Some((0xd760, i)),
        0xd774 if i < 6 => None,
        0xe4a6 if (param as u8) < 30 && param >> 8 < 5 => {
            for n in 0..4 {
                word(rom, 0xe47e + (param >> 8) * 8 + n * 2)?;
            }
            Some((0xe42c, param as u8 as usize))
        }
        _ => return Err("unsupported enemy projectile initializer or parameter"),
    };
    if let Some((p, i)) = table {
        word(rom, p + i as u16 * 2)?;
    }
    if !matches!(
        init,
        0x9dec
            | 0x9e46
            | 0x9e4b
            | 0x9eb2
            | 0xa5d3
            | 0xa65d
            | 0xa81b
            | 0xa871
            | 0xabeb
            | 0xac7c
            | 0xae15
            | 0xb001
            | 0xb328
            | 0xb4ef
            | 0xb88e
            | 0xb87a
            | 0xb93e
            | 0xb951
            | 0xb964
            | 0xbbdb
            | 0xbda2
            | 0xbd9c
            | 0xbe4f
            | 0xbf59
            | 0xbb30
            | 0xbb92
            | 0xdca3
            | 0xdcd4
            | 0xdb18
            | 0xe000
            | 0x8acd
            | 0x8a39
            | 0xea31
            | 0xec62
            | 0xef89
            | 0xebc6
            | 0x8af1
            | 0x8b15
            | 0x8b39
            | 0xded6
            | 0xb8b5
            | 0xb8e8
            | 0xb8f8
            | 0xb1ce
    ) && !matches!(
        init,
        0xb49d
            | 0xce6d
            | 0x9023
            | 0x9286
            | 0x934d
            | 0x94a0
            | 0x94b4
            | 0x94c8
            | 0x94dc
            | 0x993a
            | 0x9824
            | 0x9ca3
            | 0x9cd8
            | 0x9d0c
    ) && (r.word(0x1c27) > 78 || r.word(0x1c27) & 1 != 0)
    {
        return Err("invalid PLM slot");
    }
    Ok(())
}
pub(crate) fn random(r: &mut Wram<'_>) -> u16 {
    let n = r.word(0x5e5);
    let low = u16::from(n as u8) * 5;
    let high = (u16::from((n >> 8) as u8) * 5) as u8;
    let carry = (low >> 8) + u16::from(high) + 1;
    let n = ((low & 255) | u16::from(carry as u8) << 8).wrapping_add(carry >> 8).wrapping_add(17);
    r.set_word(0x5e5, n);
    n
}
fn initialize(r: &mut Wram<'_>, rom: &[u8], init: u16, param: u16, k: usize) -> Result<(), &'static str> {
    if init == 0xef89 {
        let enemy_slot = r.word(0xe54);
        let enemy = 0xf78 + usize::from(enemy_slot);
        r.set_word(0x1a4b + k, r.word(enemy + 2));
        r.set_word(0x1a93 + k, r.word(enemy + 6));
        r.set_word(0xf410 + k, enemy_slot | if r.word(enemy + 0x0e) & 0x4000 != 0 { 0x8000 } else { 0 });
        r.set_word(0xf3c8 + k, r.word(enemy));
        r.set_word(0x19bb + k, 0);
        r.set_word(0x1b47 + k, word(rom, 0xefd5 + param * 2)?);
        r.set_word(0x1b8f + k, 1);
        return Ok(());
    }
    if init == 0xec62 {
        let enemy_slot = r.word(0xe54);
        let enemy = 0xf78 + usize::from(enemy_slot);
        r.set_word(0x1a4b + k, r.word(enemy + 2));
        r.set_word(0x1a93 + k, r.word(enemy + 6));
        r.set_word(0x1b47 + k, if r.word(enemy + 0x3e) == 0 { 0xec56 } else { 0xec5c });
        let body = enemy_slot.wrapping_add(r.word(0x8808 + usize::from(enemy_slot)).wrapping_mul(2));
        r.set_word(0x8800 + usize::from(body), k as u16);
        return Ok(());
    }
    if init == 0xebc6 {
        let enemy = 0xf78 + usize::from(r.word(0xe54));
        r.set_word(0x1a4b + k, r.word(enemy + 2));
        r.set_word(0x1a93 + k, r.word(enemy + 6));
        r.set_word(0x1b47 + k, 0xebae);
        let angle = usize::from(r.byte(0x60b));
        r.set_word(0x97dc + k, angle as u16);
        for (phase, high, low) in [(64, 0x1ab7, 0x1aff), (128, 0x1adb, 0x1b23)] {
            let p = 0x1037ee + ((angle + phase) & 127) * 2;
            let bytes = rom.get(p..p + 2).ok_or("missing projectile quarter circle ROM")?;
            let amount = u32::from(u16::from_le_bytes([bytes[0], bytes[1]])) * u32::from(param);
            r.set_word(high + k, (amount >> 16) as u16);
            r.set_word(low + k, amount as u16);
        }
        return Ok(());
    }
    if init == 0xea31 {
        r.set_word(0x1a4b + k, r.word(0xf7a));
        r.set_word(0x1a93 + k, r.word(0xf7e));
        r.set_word(0x1adb + k, 0);
        let linked = usize::from(r.word(0xfa8));
        let direction = if linked == 0 { 48 } else { 16 };
        let script = word(rom, 0xe9f1 + direction)?;
        r.set_word(0x1b47 + k, script);
        r.set_word(0x1b23 + k, script);
        r.set_word(0x1aff + k, direction);
        r.set_word(0x1ab7 + k, 0xea98);
        r.set_word(0x7800 + linked, k as u16);
        r.set_word(0xf380 + k, 2);
        r.set_word(0x7820 + linked, 1);
        return Ok(());
    }
    if init == 0x8a39 {
        let random = random(r);
        let enemy = 0xf78 + usize::from(r.word(0xe54));
        let y = r.word(enemy + 6).wrapping_add(12);
        r.set_word(0x1a93 + k, y);
        r.set_word(0x19df + k, y.wrapping_add(72));
        let velocity = r.word(enemy + 0x36);
        r.set_word(0x1adb + k, velocity);
        r.set_word(0x1a4b + k, r.word(enemy + 2).wrapping_add(random & 31).wrapping_sub(16));
        r.set_word(0x1b47 + k, word(rom, 0x8a75 + ((velocity >> 8) & 6))?);
        return Ok(());
    }
    if matches!(init, 0x8acd | 0x8af1 | 0x8b15 | 0x8b39) {
        let enemy = 0xf78 + usize::from(r.word(0xe54));
        let left = matches!(init, 0x8b15 | 0x8b39);
        let upper = matches!(init, 0x8af1 | 0x8b39);
        r.set_word(0x1a93 + k, r.word(enemy + 6));
        r.set_word(0x1adb + k, if upper { 0xfbff } else { 0xfcff });
        r.set_word(0x1a4b + k, r.word(enemy + 2).wrapping_add(if left { 0xfffa } else { 6 }));
        let velocity: u16 = if upper { 96 } else { 320 };
        r.set_word(0x1ab7 + k, if left { velocity.wrapping_neg() } else { velocity });
        return Ok(());
    }
    if matches!(init, 0xe5d0 | 0xe5d5 | 0xe5da | 0xe6ad) {
        let plm = r.word(0x1c27);
        crate::plm_render::calculate_block_coords(r, plm)?;
        let (x, y) = (r.word(0x1c29), r.word(0x1c2b));
        if init == 0xe6ad {
            r.set_word(0x1a4b + k, x.wrapping_add(1).wrapping_mul(16));
            r.set_word(0x1a93 + k, y.wrapping_sub(2).wrapping_mul(16));
        } else {
            r.set_word(0x1aff + k, r.word(0x1c87 + usize::from(plm)));
            r.set_word(0x1a4b + k, x.wrapping_mul(16));
            let offset = match init {
                0xe5d5 => 64,
                0xe5da => 0xffc0,
                _ => 0,
            };
            r.set_word(0x1a93 + k, y.wrapping_mul(16).wrapping_add(offset));
        }
        return Ok(());
    }
    if matches!(init, 0xdb18 | 0xded6 | 0xe000) {
        let fireball = init == 0xded6;
        let lava = init == 0xe000;
        r.set_word(
            0x1b47 + k,
            if lava {
                if param == 0 { 0xdfd8 } else { 0xdfde }
            } else if fireball {
                if param == 0 { 0xde96 } else { 0xdea6 }
            } else {
                0xdb0c
            },
        );
        r.set_word(
            0x1aff + k,
            match (init, param == 0) {
                (0xe000, true) => 0xe050,
                (0xe000, false) => 0xe07a,
                (0xded6, true) => 0xdf40,
                (0xded6, false) => 0xdf6a,
                (_, true) => 0xdb62,
                (_, false) => 0xdb8c,
            },
        );
        let enemy = 0xf78 + usize::from(r.word(0xe54));
        for (dest, source) in [(0x1a4b, 2), (0x1a27, 4), (0x1a93, 6), (0x1a6f, 8)] {
            r.set_word(dest + k, r.word(enemy + source));
        }
        let offset = if fireball { u16::from(r.word(enemy + 0x3c) & 15 != 0) * 4 } else { 2 };
        r.set_word(0x1a93 + k, r.word(0x1a93 + k).wrapping_add(offset));
        let (vy, vx) = if fireball {
            let row = u16::from(r.byte(enemy + 0x3e));
            (word(rom, 0xdeb6 + row * 4)?, word(rom, 0xdeb8 + row * 4)?)
        } else {
            if lava { (0xfd00, 0x300) } else { (0xff00, 0x100) }
        };
        r.set_word(0x1adb + k, vy);
        r.set_word(0x1ab7 + k, vx);
        return Ok(());
    }
    if init == 0xdca3 {
        r.set_word(0x1a93 + k, r.word(0xf7e).wrapping_add([-64i16, -56, -48, -40, -32][usize::from(param)] as u16));
        r.set_word(0x1a4b + k, r.word(0xf7a));
        return Ok(());
    }
    if init == 0xdcd4 {
        r.set_word(0x1a4b + k, [32, 96, 160, 224][usize::from(param)]);
        r.set_word(0x1a93 + k, 520);
        return Ok(());
    }
    if init == 0xbb30 {
        let enemy = usize::from(r.word(0xe54));
        r.set_word(0x1a4b + k, r.word(0x8022 + enemy));
        r.set_word(0x1a93 + k, r.word(0x8024 + enemy));
        r.set_word(0x1b47 + k, word(rom, 0xbb1e + param * 2)?);
        return Ok(());
    }
    if init == 0xbb92 {
        let enemy = usize::from(r.word(0xe54));
        for (dest, source) in [(0x1a4b, 2), (0x1a27, 4), (0x1a93, 6), (0x1a6f, 8)] {
            r.set_word(dest + k, r.word(0xf78 + enemy + source));
        }
        let linked = enemy + usize::from(r.word(0x8014 + enemy));
        r.set_word(0x7800 + linked, k as u16);
        r.set_word(0xf380 + k, 1);
        return Ok(());
    }
    if init == 0xbe4f {
        let i = usize::from(param);
        let direction = word(rom, 0xbee1 + param * 2)? | 0x100;
        r.set_word(0x19bb + k, 1024);
        r.set_word(0x1a6f + k, direction);
        r.set_word(0x1b47 + k, word(rom, 0xbeb9 + (direction & 255) * 2)?);
        r.set_word(0x1a4b + k, [920, 840, 808, 728, 648, 616, 536, 456, 424, 344, 264, 232][i]);
        r.set_word(0x1a93 + k, [48, 64, 64, 48, 64, 64, 48, 64, 64, 48, 64, 64][i]);
        r.set_word(0x1a27 + k, word(rom, 0xbec9 + param * 2)?);
        let timer = u16::from(random(r) as u8).max(32);
        r.set_word(0x1ab7 + k, timer);
        let timer = u16::from(random(r) as u8).max(128);
        r.set_word(0x1adb + k, timer);
        return Ok(());
    }
    if init == 0xbf59 {
        r.set_word(0x1b23 + k, 0);
        r.set_word(0x19bb + k, 1024);
        let source = usize::from(param & !1);
        let direction = usize::from(r.byte(0x1a6f + source));
        r.set_word(0x1aff + k, direction as u16 * 2);
        let x: [i16; 8] = [-17, -12, 0, 12, 17, 12, 0, -12];
        let y: [i16; 8] = [-9, 3, 7, 3, -9, -19, -21, -19];
        let vx: [i16; 8] = [-704, -498, 0, 498, 704, 498, 0, -498];
        let vy: [i16; 8] = [0, 498, 704, 498, 0, -498, -704, -498];
        r.set_word(0x1ab7 + k, vx[direction] as u16);
        r.set_word(0x1adb + k, vy[direction] as u16);
        r.set_word(0x1a4b + k, r.word(0x1a4b + source).wrapping_add(x[direction] as u16));
        r.set_word(0x1a93 + k, r.word(0x1a93 + source).wrapping_add(y[direction] as u16));
        return Ok(());
    }
    if matches!(init, 0xbda2 | 0xbd9c) {
        if init == 0xbd9c {
            r.set_word(0x1aff + k, param);
        }
        let enemy = 0xf78 + usize::from(r.word(0xe54));
        let angle = usize::from(r.byte(enemy + 0x36));
        let octant = angle >> 5;
        let x: [i16; 8] = [0, 12, 16, 12, 0, -12, -16, -12];
        let y: [i16; 8] = [-16, -12, 0, 12, 16, 12, 0, -12];
        r.set_word(0x1a4b + k, r.word(enemy + 2));
        r.set_word(0x1a93 + k, r.word(enemy + 6));
        r.set_word(0x1ab7 + k, SINE[angle + 64] as u16);
        r.set_word(0x1adb + k, SINE[angle] as u16);
        r.set_word(0x1a4b + k, r.word(0x1a4b + k).wrapping_add(x[octant] as u16));
        r.set_word(0x1a93 + k, r.word(0x1a93 + k).wrapping_add(y[octant] as u16));
        return Ok(());
    }
    if init == 0xbbdb {
        r.set_word(0x1b47 + k, 0xbbd5);
        r.set_word(0x1aff + k, 0xbc16);
        r.set_word(0x1adb + k, param);
        r.set_word(0x1ab7 + k, r.word(0x1995));
        let enemy = 0xf78 + usize::from(r.word(0xe54));
        for (dest, source) in [(0x1a4b, 2), (0x1a27, 4), (0x1a93, 6), (0x1a6f, 8)] {
            r.set_word(dest + k, r.word(enemy + source));
        }
        return Ok(());
    }
    if init == 0xb87a {
        r.set_word(0x1a4b + k, r.word(0x1a4b + usize::from(param)));
        r.set_word(0x1a93 + k, r.word(0x195e).wrapping_sub(4));
        return Ok(());
    }
    if matches!(init, 0xb93e | 0xb951 | 0xb964) {
        let (x, y) = match init {
            0xb93e => (120, 184),
            0xb951 => (142, 85),
            _ => (132, 136),
        };
        r.set_word(0x1aff + k, x);
        r.set_word(0x1a4b + k, x);
        r.set_word(0x1b23 + k, y);
        r.set_word(0x1a93 + k, y);
        return Ok(());
    }
    if matches!(init, 0xb88e | 0xb8f8 | 0xb8b5 | 0xb8e8) {
        let (x, y) = if matches!(init, 0xb88e | 0xb8f8) {
            let i = usize::from(param >> 1);
            ([132, 122, 158, 104][i], [144, 81, 128, 114][i])
        } else {
            (r.word(0x1a4b + usize::from(param)), r.word(0x1a93 + usize::from(param)))
        };
        r.set_word(0x1a4b + k, x);
        r.set_word(0x1a93 + k, y);
        match init {
            0xb88e => {
                const COLORS: [u16; 16] = [
                    0x6bff, 0x033b, 0x0216, 0x0173, 0x7f5f, 0x7c1f, 0x5816, 0x300c, 0x7f5a, 0x7ec0, 0x6de0, 0x54e0,
                    0x6bfa, 0x3be0, 0x2680, 0x1580,
                ];
                for n in 0..4 {
                    r.set_word(0xc000 + 498 + n * 2, COLORS[usize::from(param) * 2 + n]);
                }
            }
            0xb8f8 => r.set_word(0x1adb + k, 0xfc00),
            0xb8b5 => {
                let angle = (random(r) & 63).wrapping_sub(32) as u8;
                r.set_word(0x1aff + k, u16::from(angle) * 2);
                r.set_word(0x1ab7 + k, SINE[usize::from(angle) + 64] as u16);
                r.set_word(0x1adb + k, (SINE[usize::from(angle)] as u16).wrapping_mul(4));
            }
            _ => {}
        }
        return Ok(());
    }
    if init == 0xb4ef {
        let enemy = usize::from(r.word(0xe54));
        let left = r.word(0xfa8 + enemy) & 0x8000 != 0;
        r.set_word(0x1a93 + k, r.word(0xf7e + enemy).wrapping_sub(28));
        r.set_word(0x1adb + k, 0xfc3f);
        r.set_word(0x1a4b + k, r.word(0xf7a + enemy).wrapping_add(if left { 0xfff4 } else { 12 }));
        r.set_word(0x1ab7 + k, if left { 0xfd40 } else { 704 });
        r.set_word(0x1b47 + k, if left { 0xb4bf } else { 0xb4cb });
        return Ok(());
    }
    if init == 0xb1ce {
        let enemy = r.word(0xe54);
        let p = 0xf78 + usize::from(enemy);
        let right = r.word(p + 0x3c) & 0x8000 != 0;
        r.set_word(0x1aff + k, enemy);
        r.set_word(0x1a4b + k, r.word(p + 2).wrapping_add(if right { 30 } else { 0xffe2 }));
        r.set_word(0x1a93 + k, r.word(p + 6).wrapping_sub(52));
        r.set_word(0x1b47 + k, word(rom, if right { 0xb20b } else { 0xb209 })?);
        return Ok(());
    }

    if init == 0xae15 {
        let offset = if random(r) & 1 != 0 { 0xfff4 } else { 20 };
        let enemy = usize::from(r.word(0xe54));
        let right = r.word(0xfb4 + enemy) & 0x8000 != 0;
        r.set_word(0x1a93 + k, r.word(0xf7e + enemy).wrapping_add(offset));
        r.set_word(0x1adb + k, 0);
        r.set_word(0x1a4b + k, r.word(0xf7a + enemy).wrapping_add(if right { 32 } else { 0xffe0 }));
        r.set_word(0x1ab7 + k, if right { 624 } else { 0xfd90 });
        r.set_word(0x1b47 + k, if right { 0xadd2 } else { 0xadbf });
        return Ok(());
    }
    if matches!(init, 0xabeb | 0xac7c | 0xb001 | 0xb328) {
        let enemy = usize::from(r.word(0xe54));
        if init == 0xb001 {
            r.set_word(0x1b23 + k, 66);
            r.set_word(0x1aff + k, r.word(0xfb4 + enemy));
        }
        let p = (match init {
            0xabeb => 0xac08,
            0xac7c => 0xac99,
            0xb328 => 0xb376,
            _ => 0xb02f,
        }) + if r.word(0xfb4 + enemy) & 0x8000 != 0 { 0 } else { 10 };
        r.set_word(0x1b47 + k, word(rom, p)?);
        r.set_word(0x1a4b + k, r.word(0xf7a + enemy).wrapping_add(word(rom, p + 2)?));
        let vx = word(rom, p + 4)?.wrapping_add(u16::from(random(r) as u8)).wrapping_sub(128);
        r.set_word(0x1ab7 + k, vx);
        r.set_word(0x1a93 + k, r.word(0xf7e + enemy).wrapping_add(word(rom, p + 6)?));
        let vy = word(rom, p + 8)?.wrapping_add(u16::from(random(r) as u8)).wrapping_sub(128);
        r.set_word(0x1adb + k, vy);
        if init == 0xb328 {
            let mut angle = (random(r) & 30) + 176;
            if r.word(0xfb4 + enemy) & 0x8000 == 0 {
                angle += 128;
            }
            let i = usize::from(angle >> 1);
            r.set_word(0x1ab7 + k, (SINE[i + 64] as u16).wrapping_mul(8));
            r.set_word(0x1adb + k, (SINE[i] as u16).wrapping_mul(8));
        }
        return Ok(());
    }
    if matches!(init, 0xa81b | 0xa871) {
        let enemy = usize::from(r.word(0xe54));
        let mut x = r.word(0xf7a + enemy);
        let mut y = r.word(0xf7e + enemy);
        if init == 0xa81b {
            let parameter = param.wrapping_add(if r.word(0xfb4 + enemy) & 0x8000 == 0 { 4 } else { 2 });
            r.set_word(0x1993, parameter);
            let i = usize::from(parameter >> 1);
            x = x.wrapping_add([0, 12, 0xfff4, 0, 16, 0xfff0][i]);
            y = y.wrapping_add([0xfff8, 0xfff8, 0xfff8, 0xffec, 0xffec, 0xffec][i]);
        }
        r.set_word(0x1a4b + k, x);
        r.set_word(0x1aff + k, x);
        r.set_word(0x1a93 + k, y);
        r.set_word(0x1b23 + k, y);
        return Ok(());
    }
    if matches!(init, 0xa5d3 | 0xa65d) {
        let enemy = usize::from(r.word(0xe54));
        let flags = r.word(0xfb4 + enemy);
        r.set_word(0x19bb + k, 0);
        let first = random(r);
        if init == 0xa5d3 {
            r.set_word(0x1b47 + k, word(rom, 0xa64d + ((first >> 1) & 14))?);
            let second = random(r);
            r.set_word(0x1a93 + k, r.word(0xf7e + enemy).wrapping_sub(5));
            let angle = if flags & 0x4000 != 0 {
                (second & 0x1fe) >> 1
            } else {
                (if flags & 0x8000 != 0 { 32u16 } else { 224 }).wrapping_add(second & 15).wrapping_sub(8)
            };
            r.set_word(0x1ab7 + k, SINE[usize::from(angle) + 64] as u16);
            r.set_word(0x1adb + k, SINE[usize::from(angle)] as u16);
            r.set_word(0x1a4b + k, r.word(0xf7a + enemy).wrapping_add(if flags & 0x8000 != 0 { 8 } else { 0xfff8 }));
        } else {
            r.set_word(0x1a93 + k, r.word(0xf7e + enemy).wrapping_add(first & 3).wrapping_sub(5));
            r.set_word(0x1adb + k, (first & 31) + 48);
            let second = random(r);
            let offset = if flags & 0x4000 != 0 {
                0
            } else if flags & 0x8000 != 0 {
                8
            } else {
                0xfff8
            };
            r.set_word(0x1a4b + k, r.word(0xf7a + enemy).wrapping_add(second & 3).wrapping_add(offset));
            r.set_word(0x1ab7 + k, 0);
        }
        return Ok(());
    }
    if init == 0x9eb2 {
        let enemy = usize::from(r.word(0xe54));
        let left = r.word(0xfae + enemy) & 0x8000 != 0;
        r.set_word(0x1a93 + k, r.word(0xf7e + enemy).wrapping_sub(12));
        r.set_word(0x1a4b + k, r.word(0xf7a + enemy).wrapping_add(if left { 0xfff0 } else { 16 }));
        r.set_word(0x1ab7 + k, if left { 0xfc00 } else { 1024 });
        r.set_word(0x1adb + k, [0xff00, 0, 256][usize::from(param >> 1)]);
        return Ok(());
    }
    if matches!(init, 0x9dec | 0x9e46 | 0x9e4b) {
        let enemy = usize::from(r.word(0xe54));
        let (x, y, vx, vy) = if init == 0x9dec {
            (
                r.word(0xf7a + enemy).wrapping_add(r.word(0x7804 + enemy)),
                r.word(0xf7e + enemy).wrapping_sub(16),
                r.word(0x7800 + enemy),
                r.word(0x7802 + enemy),
            )
        } else {
            let height = [0xfffe, 12, 24][usize::from(r.word(0x780c + enemy) >> 1)];
            (
                r.word(0xf7a + enemy),
                r.word(0xf7e + enemy).wrapping_add(height),
                if init == 0x9e46 { 0xfe00 } else { 0x200 },
                0,
            )
        };
        r.set_word(0x1a4b + k, x);
        r.set_word(0x1a93 + k, y);
        r.set_word(0x1ab7 + k, vx);
        r.set_word(0x1adb + k, vy);
        return Ok(());
    }
    if matches!(init, 0x9ca3 | 0x9cd8 | 0x9d0c) {
        let p = 0xf78 + usize::from(r.word(0xe54));
        let random = r.word(0x5e5);
        let (x, y, vx, vy) = match init {
            0x9ca3 => (r.word(p + 2).wrapping_add(16), r.word(p + 6).wrapping_sub(96), param, 0xfc00),
            0x9cd8 => (param, 312, 0, (random & 63) + 64),
            _ => {
                let offset = if random & 1 == 0 { !(random & 63) } else { random & 63 };
                (r.word(p + 2).wrapping_add(offset), 432, param, 0xfb00)
            }
        };
        r.set_word(0x1a4b + k, x);
        r.set_word(0x1a93 + k, y);
        r.set_word(0x1ab7 + k, vx);
        r.set_word(0x1adb + k, vy);
        r.set_word(0x19bb + k, 1536);
        if init == 0x9d0c {
            crate::audio::queue_sfx(r, 2, 0x1e06)?;
        }
        return Ok(());
    }
    if init == 0x9824 {
        let mode = param >> 8;
        let low = param as u8;
        r.set_word(0x1adb + k, 0);
        let (x, y, vx) = match mode {
            0 => {
                r.set_word(0x1b47 + k, 0x97b4);
                r.set_word(0x1bd7 + k, (r.word(0x1bd7 + k) & 0xfff) | 0x2000);
                (r.word(0xf7a), r.word(0xf7e).wrapping_add(32), 0)
            }
            2 => {
                const ANGLES: [u16; 16] = [16, 32, 48, 64, 80, 96, 112, 128, 240, 224, 208, 192, 176, 160, 144, 128];
                r.set_word(0x1aff + k, ANGLES[usize::from(low)]);
                r.set_word(0x1a03 + k, 0x9a45);
                (r.word(0xf7a), r.word(0xf7e).wrapping_add(32), if low < 8 { 2 } else { 0xfffe })
            }
            4 => {
                const POSITIONS: [u16; 9] = [48, 68, 88, 108, 128, 148, 168, 188, 208];
                r.set_word(0x1a03 + k, 0x9a94);
                (POSITIONS[usize::from(low & 15)], 40, u16::from(low & 0xf0) >> 1)
            }
            _ => {
                r.set_word(0x1aff + k, u16::from(low) * 32);
                r.set_word(0x1a03 + k, 0x9ada);
                (r.word(0xf7a), r.word(0xf7e).wrapping_add(16), 128)
            }
        };
        r.set_word(0x1a4b + k, x);
        r.set_word(0x1a93 + k, y);
        r.set_word(0x1ab7 + k, vx);
        return Ok(());
    }
    if init == 0x993a {
        let angle = param * 32;
        let (x, y) = phantoon_offset(angle as u8, 48);
        r.set_word(0x1aff + k, angle);
        r.set_word(0x1ab7 + k, 0);
        r.set_word(0x1adb + k, 0);
        r.set_word(0x1a4b + k, r.word(0xf7a).wrapping_add(x));
        r.set_word(0x1a93 + k, r.word(0xf7e).wrapping_add(y).wrapping_add(16));
        return Ok(());
    }
    if matches!(init, 0x94a0 | 0x94b4 | 0x94c8 | 0x94dc) {
        let (vx, vy, definition) = match init {
            0x94a0 => (0, 0xf200, 0x9688),
            0x94b4 => (0, 0x0e00, 0x9696),
            0x94c8 => (0x0e00, 0, 0x966c),
            _ => (0xf200, 0, 0x967a),
        };
        r.set_word(0x1ab7 + k, vx);
        r.set_word(0x1adb + k, vy);
        r.set_word(0x1b23 + k, definition);
        let source = usize::from(param);
        r.set_word(0x1a4b + k, r.word(0x1a4b + source));
        r.set_word(0x1a93 + k, r.word(0x1a93 + source));
        r.set_word(0x1aff + k, r.word(0x1aff + source));
        r.set_word(0x19bb + k, 2560);
        let offset = match r.word(0x79f) {
            2 => 2,
            5 => 4,
            _ => 0,
        };
        r.set_word(0x1bd7 + k, word(rom, 0x9408 + offset)?);
        return Ok(());
    }
    if matches!(init, 0x9023 | 0x9286 | 0x934d) {
        let (x, y, vx, vy) = match init {
            0x9023 => {
                let p = 0xf78 + usize::from(r.word(0xe54));
                r.set_word(0x1bfb + k, 0);
                (r.word(p + 2).wrapping_sub(32), r.word(p + 6).wrapping_sub(16), 0xfe00, 1)
            }
            0x9286 => (param, 187, 0, (r.word(0x5e5) & 63) + 64),
            0x934d => {
                let i = usize::from(r.word(0xfb4));
                r.set_word(0x1aff + k, 0);
                r.set_word(0x1b23 + k, 0);
                (
                    r.word(0xf7a).wrapping_sub(29),
                    r.word(0xf7e).wrapping_sub(35),
                    [0xfe00, 0xfe10, 0xfe44, 0xfe96][i],
                    [0, 0x88, 0xfc, 0x16a][i],
                )
            }
            _ => unreachable!(),
        };
        r.set_word(0x1a4b + k, x);
        r.set_word(0x1a93 + k, y);
        r.set_word(0x1ab7 + k, vx);
        r.set_word(0x1adb + k, vy);
        r.set_word(0x19bb + k, if init == 0x9286 { 1024 } else { 2560 });
        return Ok(());
    }
    if init == 0xce6d {
        let source = param as usize / 2 * 2;
        let x = r.word(0x1a4b + source).wrapping_add(random(r) & 31).wrapping_sub(16);
        r.set_word(0x1a4b + k, x);
        let y = r.word(0x1a93 + source).wrapping_add(random(r) & 31).wrapping_sub(16);
        r.set_word(0x1a93 + k, y);
        r.set_word(0x19bb + k, 1600);
        return Ok(());
    }
    if init == 0xb49d {
        r.set_word(0x1a4b + k, 272);
        r.set_word(0x1aff + k, 272);
        r.set_word(0x1a93 + k, 2184);
        r.set_word(0x1b23 + k, 2184);
        return Ok(());
    }
    let angle = if init == 0xcdc5 {
        let a = random(r).wrapping_mul(2) & 0x1fe;
        r.set_word(0x1aff + k, a);
        Some(a)
    } else {
        None
    };
    crate::plm_render::calculate_block_coords(r, r.word(0x1c27))?;
    let x = r.word(0x1c29).wrapping_mul(16);
    let y = r.word(0x1c2b).wrapping_mul(16);
    let i = param as usize / 2;
    let mut position = None;
    match init {
        0xa764 => {
            let dx = [8, 24, -8, 8, 24, -8, 8, 24, 8, -8, 24, 8, -8, 24, 8, -8];
            let dy = [-8, -8, 8, 8, 8, 24, 24, 24];
            position = Some((x.wrapping_add(dx[i] as u16), y.wrapping_add(dy[(param & 15) as usize / 2] as u16)));
            r.set_word(0x1b47 + k, word(rom, 0xa7ab + param / 2 * 2)?);
            r.set_word(0x1adb + k, 256);
            r.set_word(0x1b23 + k, 16);
            r.set_word(0x1bd7 + k, r.word(0x1bd7 + k) | 0x1000);
        }
        0xb62d => {
            let offsets =
                [-16, 16, -96, -64, -128, -32, -96, 64, -128, 32, 16, 16, 96, -64, 112, -64, 128, -64, 144, -64];
            position = Some((x.wrapping_add(8).wrapping_add(offsets[i] as u16), y.wrapping_add(offsets[i + 1] as u16)));
            r.set_word(0x1b23 + k, r.word(0x1dc7 + r.word(0x1c27) as usize));
        }
        0xb683 => {
            let vel = [-64, 512, 64, 512];
            position = Some((x.wrapping_sub(8), y.wrapping_add(16)));
            r.set_word(0x1ab7 + k, vel[i] as u16);
            r.set_word(0x1adb + k, vel[i + 1] as u16);
        }
        0xcdc5 => {
            let a = angle.unwrap();
            let v = (a / 2) as usize;
            r.set_word(0x1ab7 + k, SINE[v + 64] as u16);
            r.set_word(0x1adb + k, (SINE[v] as u16).wrapping_mul(4));
            r.set_word(0x1b47 + k, word(rom, 0xce41 + ((a >> 4) & 0x1e))?);
            r.set_word(0x19bb + k, 1600);
            let dx = [8, -40, -16];
            position = Some((
                x.wrapping_add(dx[i] as u16).wrapping_add((random(r) & 15).wrapping_sub(8)),
                y.wrapping_add(32).wrapping_add((random(r) & 15).wrapping_sub(8)),
            ));
        }
        0xd6a5 => position = Some((x.wrapping_add(96), y.wrapping_add(48))),
        0xd6c9 => {
            let dx = [-56, -64, -20, -40, -64, -48, -24, -40, 0, -8];
            let dy = [8, -12, -26, -24, -32, 28, 16, -8, -24, 16];
            let vx = [-384, -384, -160, -288, -288, -320, -96, -352, 0, -64];
            let vy = [320, -256, -416, -288, -288, 448, 576, -96, -288, 384];
            r.set_word(0x1b23 + k, x.wrapping_add(96).wrapping_add(dx[i] as u16));
            r.set_word(0x1a93 + k, y.wrapping_add(48).wrapping_add(dy[i] as u16));
            r.set_word(0x1b47 + k, word(rom, 0xd760 + i as u16 * 2)?);
            r.set_word(0x1ab7 + k, vx[i] as u16);
            r.set_word(0x1adb + k, vy[i] as u16);
        }
        0xd774 => {
            let dx = [40, 80, 104, 120, 152, 184];
            let dy = [80, 72, 84, 32, 64, 84];
            r.set_word(0x1b23 + k, x.wrapping_add(dx[i]));
            r.set_word(0x1a93 + k, y.wrapping_add(dy[i]));
            r.set_word(0x1adb + k, (-1280i16) as u16);
        }
        0xe4a6 => {
            let p = 0xe47e + (param >> 8) * 8;
            let n = r.word(0x5e5);
            let dx = word(rom, p + 4)?.wrapping_add(word(rom, p)? & n);
            let dy = word(rom, p + 6)?.wrapping_add(word(rom, p + 2)? & (n >> 8));
            r.set_word(0x1b47 + k, word(rom, 0xe42c + u16::from(param as u8) * 2)?);
            position = Some((x.wrapping_add(8).wrapping_add(dx), y.wrapping_add(8).wrapping_add(dy)));
            random(r);
        }
        _ => unreachable!(),
    }
    if let Some((x, y)) = position {
        r.set_word(0x1a4b + k, x);
        r.set_word(0x1a93 + k, y);
    }
    Ok(())
}
pub(crate) const SINE: [i16; 320] = [
    -256, -255, -255, -255, -254, -254, -253, -252, -251, -249, -248, -246, -244, -243, -241, -238, -236, -234, -231,
    -228, -225, -222, -219, -216, -212, -209, -205, -201, -197, -193, -189, -185, -181, -176, -171, -167, -162, -157,
    -152, -147, -142, -136, -131, -126, -120, -115, -109, -103, -97, -92, -86, -80, -74, -68, -62, -56, -49, -43, -37,
    -31, -25, -18, -12, -6, 0, 6, 12, 18, 25, 31, 37, 43, 49, 56, 62, 68, 74, 80, 86, 92, 97, 103, 109, 115, 120, 126,
    131, 136, 142, 147, 152, 157, 162, 167, 171, 176, 181, 185, 189, 193, 197, 201, 205, 209, 212, 216, 219, 222, 225,
    228, 231, 234, 236, 238, 241, 243, 244, 246, 248, 249, 251, 252, 253, 254, 254, 255, 255, 255, 256, 255, 255, 255,
    254, 254, 253, 252, 251, 249, 248, 246, 244, 243, 241, 238, 236, 234, 231, 228, 225, 222, 219, 216, 212, 209, 205,
    201, 197, 193, 189, 185, 181, 176, 171, 167, 162, 157, 152, 147, 142, 136, 131, 126, 120, 115, 109, 103, 97, 92,
    86, 80, 74, 68, 62, 56, 49, 43, 37, 31, 25, 18, 12, 6, 0, -6, -12, -18, -25, -31, -37, -43, -49, -56, -62, -68,
    -74, -80, -86, -92, -97, -103, -109, -115, -120, -126, -131, -136, -142, -147, -152, -157, -162, -167, -171, -176,
    -181, -185, -189, -193, -197, -201, -205, -209, -212, -216, -219, -222, -225, -228, -231, -234, -236, -238, -241,
    -243, -244, -246, -248, -249, -251, -252, -253, -254, -254, -255, -255, -255, -256, -255, -255, -255, -254, -254,
    -253, -252, -251, -249, -248, -246, -244, -243, -241, -238, -236, -234, -231, -228, -225, -222, -219, -216, -212,
    -209, -205, -201, -197, -193, -189, -185, -181, -176, -171, -167, -162, -157, -152, -147, -142, -136, -131, -126,
    -120, -115, -109, -103, -97, -92, -86, -80, -74, -68, -62, -56, -49, -43, -37, -31, -25, -18, -12, -6,
];

/// Original byte-width sine products; radius is truncated by hardware multiply.
pub(crate) fn phantoon_offset(angle: u8, radius: u16) -> (u16, u16) {
    let component = |angle: u8| {
        let negative = angle >= 128;
        let index = usize::from(if negative { angle.wrapping_add(128) } else { angle });
        let sine = SINE[index + 64] as u16;
        let radius = u16::from(radius as u8);
        let value = ((u16::from(sine as u8) * radius) >> 8) + u16::from((sine >> 8) as u8) * radius;
        if negative { value.wrapping_neg() } else { value }
    };
    (component(angle), component(angle.wrapping_sub(64)))
}
