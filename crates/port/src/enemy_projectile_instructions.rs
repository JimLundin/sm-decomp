//! Bank86 enemy-projectile script control and canonical 8.8 velocity movement.
use crate::Wram;
fn slot(k: u16) -> Result<usize, &'static str> {
    if k > 34 || k & 1 != 0 { Err("invalid enemy projectile slot") } else { Ok(k as usize) }
}
fn byte(rom: &[u8], p: u16) -> Result<u8, &'static str> {
    if p < 0x8000 {
        return Err("enemy projectile ROM extent");
    }
    rom.get(0x30000 + usize::from(p & 0x7fff)).copied().ok_or("missing enemy projectile ROM")
}
fn word(rom: &[u8], p: u16) -> Result<u16, &'static str> {
    Ok(u16::from_le_bytes([byte(rom, p)?, byte(rom, p.checked_add(1).ok_or("enemy projectile ROM extent")?)?]))
}
fn move_axis(r: &mut Wram<'_>, k: usize, pos: usize, sub: usize, vel: u16) -> u16 {
    let carry = u16::from(r.byte(sub + k + 1)) + u16::from(vel as u8);
    r.set_byte(sub + k + 1, carry as u8);
    let result = r.word(pos + k).wrapping_add((vel >> 8) as u8 as i8 as u16).wrapping_add(carry >> 8);
    r.set_word(pos + k, result);
    result
}
pub fn move_x(r: &mut Wram<'_>, k: u16) -> Result<u16, &'static str> {
    let k = slot(k)?;
    Ok(move_axis(r, k, 0x1a4b, 0x1a27, r.word(0x1ab7 + k)))
}
pub fn move_y(r: &mut Wram<'_>, k: u16) -> Result<u16, &'static str> {
    let k = slot(k)?;
    Ok(move_axis(r, k, 0x1a93, 0x1a6f, r.word(0x1adb + k)))
}
pub fn move_xy(r: &mut Wram<'_>, k: u16) -> Result<u16, &'static str> {
    move_x(r, k)?;
    move_y(r, k)
}
/// Execute one opcode. Zero stops this frame; other returns are bank86 continuations.
pub fn instruction(r: &mut Wram<'_>, rom: &[u8], opcode: u16, pc: u16, k: u16) -> Result<u16, &'static str> {
    if matches!(opcode, 0xab8a | 0x9270 | 0x8c68 | 0xdc61 | 0xdc77) {
        return Err("projectile instruction requires host context");
    }
    instruction_context(r, rom, opcode, pc, k, None)
}
pub fn instruction_with_host(
    r: &mut Wram<'_>,
    rom: &[u8],
    opcode: u16,
    pc: u16,
    k: u16,
    host: &mut crate::enemy_projectiles::ProjectileHost,
) -> Result<u16, &'static str> {
    instruction_context(r, rom, opcode, pc, k, Some(host))
}
fn instruction_context(
    r: &mut Wram<'_>,
    rom: &[u8],
    opcode: u16,
    pc: u16,
    k: u16,
    host: Option<&mut crate::enemy_projectiles::ProjectileHost>,
) -> Result<u16, &'static str> {
    let k = slot(k)?;
    match opcode {
        0x8171 => {
            let address = u32::from(word(rom, pc)?) | (u32::from(byte(rom, pc.wrapping_add(2))?) << 16);
            if address != 0x86c7fb {
                return Err("unsupported projectile function call");
            }
            crate::enemy_projectiles::spawn_room(r, rom, 0xcb75, r.word(0x1aff + k))?;
            Ok(pc.wrapping_add(3))
        }
        0xcaee => {
            r.set_word(0x786a, 0);
            Ok(pc)
        }
        0xb3b8 => {
            let enemy = usize::from(r.word(0x1b23 + k));
            if enemy > 0x7c0 || enemy & 63 != 0 {
                return Err("Kraid projectile enemy state extent");
            }
            if r.word(0x780c + enemy) & 0x8000 == 0 { word(rom, pc) } else { Ok(pc.wrapping_add(2)) }
        }
        0x82a5 => {
            let x = r.word(0xaf6).wrapping_sub(r.word(0x1a4b + k));
            let y = r.word(0xafa).wrapping_sub(r.word(0x1a93 + k));
            let angle = crate::enemy::dispatch_rom(r, rom, "CalculateAngleFromXY", i32::from(x), i32::from(y))? as u16;
            let destination = slot(r.word(0x1991))?;
            r.set_word(0x1aff + destination, angle.wrapping_mul(2));
            r.set_word(0x1ab7 + destination, crate::enemy_projectiles::SINE[usize::from(angle) + 64] as u16);
            r.set_word(0x1adb + destination, crate::enemy_projectiles::SINE[usize::from(angle)] as u16);
            Ok(pc)
        }
        0x8cf6 | 0x8cfd => {
            r.set_word(0x1a03 + k, if opcode == 0x8cf6 { 0x8dff } else { 0x8dca });
            Ok(pc)
        }
        0x9475 => {
            r.set_word(0x1bd7 + k, r.word(0x1bd7 + k) | 0x2000);
            Ok(pc)
        }
        0xaf92 | 0xc8d0 => {
            r.set_word(0x1a93 + k, r.word(0x1a93 + k).wrapping_add(if opcode == 0xaf92 { 0xfffc } else { 12 }));
            Ok(pc)
        }
        0xb436 => {
            r.set_word(0x1a4b + k, r.word(0x1aff + k));
            r.set_word(0x1a93 + k, r.word(0x1b23 + k));
            Ok(pc)
        }
        0xd1b6 => {
            r.set_word(0x1bfb + k, (r.word(0x5e5) & 0x1f) + 1);
            r.set_word(0x1a03 + k, 0xd0ca);
            Ok(pc)
        }
        0xd15c => {
            preinstruction(r, rom, 0xd0b3, k as u16)?;
            preinstruction(r, rom, 0xd1e4, k as u16)?;
            let mut magnitude = (r.word(0x5e5) & 0x300) + 2048;
            r.set_word(0x1adb + k, magnitude.wrapping_neg());
            let delta = r.word(0xf7a + usize::from(r.word(0x1b23 + k))).wrapping_sub(r.word(0x1a4b + k));
            let absolute = if delta & 0x8000 != 0 { delta.wrapping_neg() } else { delta };
            if (absolute.wrapping_sub(48) as i16) >= 0 {
                magnitude = if delta & 0x8000 != 0 { 0xffff } else { 0 };
            }
            r.set_word(0x1ab7 + k, if magnitude & 0x100 != 0 { (-512_i16) as u16 } else { 512 });
            r.set_word(0x1a03 + k, 0xd0ec);
            Ok(pc)
        }
        0xd1c7 => {
            r.set_word(0x19bb + k, 0);
            Ok(pc)
        }
        0xdc5a => {
            r.set_word(0x1bd7 + k, 0x3000);
            Ok(pc)
        }
        0xee8b | 0xee97 | 0xeea3 => {
            let sound = match opcode {
                0xee8b => 9,
                0xee97 => 0x24,
                _ => 0xb,
            };
            crate::audio::queue_sfx(r, 1, (sound << 8) | 1)?;
            Ok(pc)
        }
        0xece3 | 0xed17 => {
            let radius = if opcode == 0xece3 { 32 } else { 16 };
            let mask = radius * 2 - 1;
            let random = crate::enemy_projectiles::random(r);
            let x = r.word(0x1a4b + k).wrapping_add(random & mask).wrapping_sub(radius);
            let y = r.word(0x1a93 + k).wrapping_add((random >> 8) & mask).wrapping_sub(radius);
            crate::sprite_objects::create(r, rom, x, y, word(rom, pc)?, 0)?;
            Ok(pc.wrapping_add(2))
        }
        0xdc77 => {
            let host = host.ok_or("projectile instruction requires host context")?;
            host.x = r.word(0x1a4b + k);
            host.y = r.word(0x1a93 + k);
            crate::enemy_projectiles::spawn_room_with_host(r, rom, 0xde7a, 0, host)?;
            Ok(pc)
        }
        0xab8a | 0x9270 | 0x8c68 | 0xdc61 => {
            let host = host.ok_or("projectile instruction requires host context")?;
            let header = match opcode {
                0x9270 => 0xddbf,
                0x8c68 => 0xde7f,
                0xdc61 => 0xdf7f,
                _ => word(rom, pc.wrapping_add(if r.word(0x79f) != 0 { 2 } else { 0 }))?,
            };
            host.x = r.word(0x1a4b + k);
            host.y = r.word(0x1a93 + k);
            crate::enemy_projectiles::spawn_enemy_drops(r, rom, host, header, k as u16, 0)?;
            Ok(if opcode == 0xab8a { pc.wrapping_add(4) } else { pc })
        }
        0x8d99 => {
            dispatch_rom(r, rom, "EprojPowerBombCollision", k as i32, 0)?;
            let divisor = r.word(0xa66);
            if (divisor.wrapping_sub(5) as i16) < 0 {
                let next = divisor.wrapping_add(1);
                r.set_word(0xa66, next);
                r.set_word(0x1b23 + k, next);
                r.set_word(0x1aff + k, 256);
                r.set_word(0x1a03 + k, 0x8dca);
                r.set_word(0x1bd7 + k, (r.word(0x1bd7 + k) & 0x5fff) | 0x2000);
                r.set_word(0x18a8, 0);
                r.set_word(0x18aa, 0);
            }
            Ok(pc)
        }
        0xc173 => word(rom, pc.wrapping_add(r.word(0x1aff + k))),
        0xc1b4 => {
            r.set_word(0x19bb + k, 0);
            Ok(pc)
        }
        0xb7ea | 0xb818 => {
            crate::enemy_projectiles::spawn_room(r, rom, if opcode == 0xb7ea { 0xba78 } else { 0xba86 }, k as u16)?;
            Ok(pc)
        }
        0xb269 | 0xb272 => {
            let angle = crate::enemy::dispatch_rom(r, rom, "CalculateAngleOfSamusFromEproj", k as i32, 0)? as usize;
            let angle = if opcode == 0xb269 { angle & 127 } else { angle | 128 };
            r.set_word(0x1ab7 + k, (crate::enemy_projectiles::SINE[angle + 64] as u16).wrapping_mul(4));
            r.set_word(0x1adb + k, (crate::enemy_projectiles::SINE[angle] as u16).wrapping_mul(4));
            Ok(pc)
        }
        0xb13e => Ok(if r.word(0x1aff + k) & 0x8000 != 0 { 0xb166 } else { 0xb14b }),
        0xad92 => {
            add_hilo(r, k, 0x1a4b, 0x1a27, r.word(0x1ab7 + k));
            word(rom, pc.wrapping_add(if r.word(0x1ab7 + k) & 0x8000 == 0 { 2 } else { 0 }))
        }
        0xa3be | 0xaf36 => {
            r.set_word(0x1a4b + k, r.word(0x1aff + k));
            r.set_word(0x1a93 + k, r.word(0x1b23 + k));
            Ok(pc)
        }
        0xa456 => {
            if crate::enemy_projectiles::random(r) & 0xc000 == 0xc000 {
                word(rom, pc)
            } else {
                Ok(pc.wrapping_add(2))
            }
        }
        0xeeaf => {
            let drop = random_drop(r, rom, k as u16)?;
            if k != 0 && (drop.wrapping_sub(6) as i16) < 0 {
                let script = word(rom, 0xef04 + drop * 2)?;
                r.set_word(0x1aff + k, drop * 2);
                r.set_word(0x1b47 + k, script);
                r.set_word(0x1b8f + k, 1);
                r.set_word(0x1b23 + k, 400);
                r.set_word(0x1a03 + k, 0xefe0);
                r.set_word(0x1bd7 + k, r.word(0x1bd7 + k) & !0x4000);
                Ok(script)
            } else {
                r.set_word(0x1b8f + k, 1);
                r.set_word(0x1bd7 + k, 0x3000);
                r.set_word(0x1a03 + k, 0xefdf);
                r.set_word(0x1b47 + k, 0xeca3);
                Ok(0xeca3)
            }
        }
        0xa050 => {
            r.set_word(0x1a03 + k, word(rom, pc)?);
            Ok(pc)
        }
        0x95ba | 0x95ed => {
            let definitions = if opcode == 0x95ba { [0x966c, 0x967a] } else { [0x9688, 0x9696] };
            for definition in definitions {
                r.set_word(0x1995, 0);
                crate::enemy_projectiles::spawn_room(r, rom, definition, k as u16)?;
            }
            Ok(pc)
        }
        0x9620 => {
            let count = r.byte(0x1aff + k).wrapping_sub(1);
            r.set_byte(0x1aff + k, count);
            if count & 0x80 == 0 {
                crate::enemy_projectiles::spawn_room(r, rom, r.word(0x1b23 + k), k as u16)?;
            }
            Ok(pc)
        }
        0x8154 => {
            r.set_word(0x1997 + k, 0);
            Ok(0)
        }
        0x8159 => {
            r.set_word(0x1b47 + k, pc.wrapping_sub(2));
            Ok(0)
        }
        0x8161 => {
            r.set_word(0x1a03 + k, word(rom, pc)?);
            Ok(pc.wrapping_add(2))
        }
        0x816a => {
            r.set_word(0x1a03 + k, 0x8170);
            Ok(pc)
        }
        0x81ab => word(rom, pc),
        0x81b0 => Ok(pc.wrapping_add(byte(rom, pc)? as i8 as u16)),
        0x81c6 | 0x81ce => {
            let old = r.word(0x19df + k);
            r.set_word(0x19df + k, old.wrapping_sub(1));
            if old == 1 {
                Ok(pc.wrapping_add(if opcode == 0x81c6 { 2 } else { 1 }))
            } else {
                instruction(r, rom, if opcode == 0x81c6 { 0x81ab } else { 0x81b0 }, pc, k as u16)
            }
        }
        0x81d5 => {
            r.set_word(0x19df + k, word(rom, pc)?);
            Ok(pc.wrapping_add(2))
        }
        0x81df => {
            let mut parameters = [0; 4];
            for (i, value) in parameters.iter_mut().enumerate() {
                *value = byte(rom, pc.checked_add(i as u16).ok_or("random radius script extent")?)?;
            }
            for axis in parameters.as_chunks::<2>().0 {
                if !(0..=255u16).any(|n| ((n as u8 & axis[0]).wrapping_sub(axis[1]) as i8) >= 0) {
                    return Err("impossible enemy projectile random radius");
                }
            }
            let direction = crate::enemy_projectiles::random(r);
            for (axis, position, sign) in [(0, 0x1a4b, 0x8000), (2, 0x1a93, 0x4000)] {
                let mut accepted = None;
                for _ in 0..65536 {
                    let value = (crate::enemy_projectiles::random(r) as u8 & parameters[axis])
                        .wrapping_sub(parameters[axis + 1]);
                    if value & 0x80 == 0 {
                        accepted = Some(u16::from(value));
                        break;
                    }
                }
                let value = accepted.ok_or("enemy projectile random radius budget")?;
                let delta = if direction & sign != 0 { value.wrapping_neg() } else { value };
                r.set_word(position + k, r.word(position + k).wrapping_add(delta));
            }
            Ok(pc.checked_add(4).ok_or("random radius continuation extent")?)
        }
        0x8230 | 0x823c => {
            let mask = word(rom, pc)?;
            let old = r.word(0x1bd7 + k);
            r.set_word(0x1bd7 + k, if opcode == 0x8230 { old | mask } else { old & mask });
            Ok(pc.wrapping_add(2))
        }
        0x8248 | 0x8252 | 0x825c | 0x8266 | 0x8270 | 0x827a | 0x8284 | 0x828e => {
            let mask = match opcode {
                0x8248 | 0x8252 => 0x8000,
                0x825c | 0x8266 => 0x2000,
                0x8270 | 0x827a => 0x4000,
                _ => 0x1000,
            };
            let old = r.word(0x1bd7 + k);
            r.set_word(
                0x1bd7 + k,
                if [0x8248, 0x825c, 0x8270, 0x8284].contains(&opcode) { old | mask } else { old & !mask },
            );
            Ok(pc)
        }
        0x8298 => {
            r.set_word(0x1bb3 + k, word(rom, pc)?);
            Ok(pc.wrapping_add(2))
        }
        0x82a1 => {
            r.set_word(0x1bb3 + k, 0);
            Ok(pc)
        }
        0x82d5 => {
            let source = word(rom, pc)?;
            let destination = word(rom, pc.wrapping_add(2))? as usize & !1;
            let count = usize::from(byte(rom, pc.wrapping_add(4))?) + 1;
            if destination + count * 2 > 512 {
                return Err("enemy projectile palette extent");
            };
            let colors = (0..count)
                .map(|i| word(rom, source.checked_add(i as u16 * 2).ok_or("enemy projectile palette source extent")?))
                .collect::<Result<Vec<_>, _>>()?;
            for (i, c) in colors.into_iter().enumerate() {
                r.set_word(0xc000 + destination + i * 2, c)
            }
            Ok(pc.wrapping_add(5))
        }
        0x82fd => {
            crate::audio::queue_music(r, u16::from(byte(rom, pc)?), 8, true)?;
            Ok(pc.wrapping_add(1))
        }
        0x8309 | 0x8312 | 0x831b | 0x8324 | 0x832d | 0x8336 | 0x833f | 0x8348 | 0x8351 | 0x835a | 0x8363 | 0x836c
        | 0x8375 | 0x837e | 0x8387 => {
            let n = (opcode - 0x8309) / 9;
            let quota = [6, 15, 3, 9, 1][usize::from(n / 3)];
            crate::audio::queue_sfx(r, usize::from(n % 3), (u16::from(byte(rom, pc)?) << 8) | quota)?;
            Ok(pc.wrapping_add(1))
        }
        0xb7f5 => {
            r.set_word(0x183e, 1);
            r.set_word(0x1840, r.word(0x1840) | 32);
            Ok(pc)
        }
        0xb841 => {
            r.set_word(0x1a93 + k, r.word(0x1a93 + k).wrapping_add(word(rom, pc)?));
            Ok(pc.wrapping_add(2))
        }
        0xd5e1 | 0xd69a => {
            crate::enemy_projectiles::random(r);
            r.set_word(0x1ab7 + k, r.word(0x5e6));
            if opcode == 0xd5e1 {
                r.set_word(0x1adb + k, 192)
            }
            Ok(pc)
        }
        0xd5f2 | 0xd62a => {
            let odd = r.word(0x5b6) & 1 != 0;
            let f = r.word(0x1b23 + k);
            let (x, map, next) = if opcode == 0xd5f2 {
                (
                    if odd { f } else { 256u16.wrapping_sub(f) },
                    word(rom, pc.wrapping_add(if odd { 0 } else { 2 }))?,
                    pc.wrapping_add(4),
                )
            } else {
                (if odd { f } else { 0xee00 }, word(rom, pc)?, pc.wrapping_add(2))
            };
            r.set_word(0x1a4b + k, x);
            r.set_word(0x1b6b + k, map);
            r.set_word(0x1b47 + k, next);
            r.set_word(0x1b8f + k, 1);
            Ok(0)
        }
        0xe533 => {
            r.set_word(0x1adb + k, word(rom, pc)?);
            Ok(pc.wrapping_add(2))
        }
        _ => Err("unsupported enemy projectile instruction"),
    }
}
fn add_hilo(r: &mut Wram<'_>, k: usize, hi: usize, lo: usize, velocity: u16) {
    let value = (u32::from(r.word(hi + k)) << 16) | u32::from(r.word(lo + k));
    let result = value.wrapping_add(((velocity as i16 as i32) << 8) as u32);
    r.set_word(hi + k, (result >> 16) as u16);
    r.set_word(lo + k, result as u16);
}
fn delete_below(r: &mut Wram<'_>, k: usize) {
    if r.word(0x1a93 + k).wrapping_sub(r.word(0x915)) as i16 >= 288 {
        r.set_word(0x1997 + k, 0)
    }
}
fn lavaquake_outside(r: &Wram<'_>, k: usize) -> bool {
    [(0x1a4b, 0x911), (0x1a93, 0x915)].into_iter().any(|(pos, camera)| {
        (r.word(pos + k).wrapping_sub(r.word(camera)) as i16) < 0
            || (r.word(camera).wrapping_add(256).wrapping_sub(r.word(pos + k)) as i16) < 0
    })
}
fn outside_screen(r: &Wram<'_>, k: usize) -> bool {
    [(0x1a4b, 0x911), (0x1a93, 0x915)].into_iter().any(|(pos, camera)| {
        (r.word(pos + k).wrapping_sub(r.word(camera)) as i16) < 0
            || (r.word(pos + k).wrapping_sub(r.word(camera).wrapping_add(256)) as i16) >= 0
    })
}
fn lavaquake_arc(r: &mut Wram<'_>, rom: &[u8], k: usize, state: u16) -> Result<(), &'static str> {
    let rising = state == 0xbc16;
    let speed = r.word(0x1adb + k).wrapping_add(if rising { 0xfffe } else { 2 });
    if rising && (speed as i16) < 0 {
        r.set_word(0x1adb + k, 0);
        r.set_word(0x1aff + k, 0xbc8f);
        return Ok(());
    }
    let speed = if !rising && (speed.wrapping_sub(64) as i16) >= 0 { 64 } else { speed };
    let mut steps = [(0u32, 0u16); 2];
    for (i, step) in steps.iter_mut().enumerate() {
        let row = if rising { (i32::from(speed) + 1 - i as i32).max(0) } else { i32::from(speed) - 1 + i as i32 };
        if !(0..95).contains(&row) {
            return Err("lavaquake quadratic speed extent");
        }
        // A0:CBC7 contains paired 16.16 downward/upward speeds per row.
        let p = 0x104bc7 + row as usize * 8 + usize::from(rising) * 4;
        let bytes = rom.get(p..p + 4).ok_or("missing lavaquake quadratic ROM")?;
        let amount = u32::from_le_bytes(bytes.try_into().unwrap());
        *step = (amount, (amount >> 16) as u16);
    }
    r.set_word(0x1adb + k, speed);
    for (amount, scratch) in steps {
        let position = (u32::from(r.word(0x1a93 + k)) << 16) | u32::from(r.word(0x1a6f + k));
        let position = position.wrapping_add(amount);
        r.set_word(0x1a93 + k, (position >> 16) as u16);
        r.set_word(0x1a6f + k, position as u16);
        r.set_word(0x1b23 + k, scratch);
    }
    move_x(r, k as u16)?;
    Ok(())
}
/// Dispatch supported movement and room-effect preinstructions. Collision remains explicit.
pub fn preinstruction(r: &mut Wram<'_>, rom: &[u8], address: u16, k: u16) -> Result<(), &'static str> {
    if matches!(address, 0x940e | 0xbfdf) {
        return Err("projectile preinstruction requires host context");
    }
    preinstruction_context(r, rom, address, k, None)
}
pub fn preinstruction_with_host(
    r: &mut Wram<'_>,
    rom: &[u8],
    address: u16,
    k: u16,
    host: &mut crate::enemy_projectiles::ProjectileHost,
) -> Result<(), &'static str> {
    preinstruction_context(r, rom, address, k, Some(host))
}
fn preinstruction_context(
    r: &mut Wram<'_>,
    rom: &[u8],
    address: u16,
    k: u16,
    host: Option<&mut crate::enemy_projectiles::ProjectileHost>,
) -> Result<(), &'static str> {
    let k = slot(k)?;
    match address {
        0xdcc3 => {
            r.set_word(0x1a4b + k, r.word(0xf7a));
            r.set_word(0x1a93 + k, r.word(0xf7e).wrapping_sub(96));
        }
        0xd9db => {
            let direction = r.word(0x1aff + k) >> 1;
            let (x, y) = match direction {
                0 | 3 => (Some(0x1adb), None),
                1 => (None, Some(0x1adb)),
                2 | 5 => (Some(0x1ab7), None),
                4 => (None, Some(0x1ab7)),
                6 => (Some(0x1adb), Some(0x1adb)),
                7 => (Some(0x1ab7), Some(0x1adb)),
                8 => (Some(0x1adb), Some(0x1ab7)),
                9 => (Some(0x1ab7), Some(0x1ab7)),
                _ => return Err("invalid plant spike direction"),
            };
            if let Some(velocity) = x {
                add_hilo(r, k, 0x1a4b, 0x1a27, r.word(velocity + k));
            }
            if let Some(velocity) = y {
                add_hilo(r, k, 0x1a93, 0x1a6f, r.word(velocity + k));
            }
            let x = r.word(0x1a4b + k);
            let y = r.word(0x1a93 + k);
            let camera_x = r.word(0x911);
            let camera_y = r.word(0x915);
            if (x.wrapping_sub(camera_x) as i16) < 0
                || (camera_x.wrapping_add(256).wrapping_sub(x) as i16) < 0
                || (y.wrapping_sub(camera_y) as i16) < 0
                || (camera_y.wrapping_add(256).wrapping_sub(y) as i16) < 0
            {
                r.set_word(0x1997 + k, 0);
            }
        }
        0xd263 => {
            const X: [i16; 8] = [0, 32, 32, 32, 0, -32, -32, -32];
            const Y: [i16; 8] = [-32, -32, 0, 32, 32, 32, 0, -32];
            let direction = usize::from(r.word(0x1aff + k));
            let x = *X.get(direction).ok_or("invalid floater spike direction")?;
            r.set_word(0x1ab7 + k, r.word(0x1ab7 + k).wrapping_add(x as u16));
            let hit = if crate::enemy_projectile_collision::horizontal(r, rom, k as u16)? & 1 != 0 {
                true
            } else {
                r.set_word(0x1adb + k, r.word(0x1adb + k).wrapping_add(Y[direction] as u16));
                crate::enemy_projectile_collision::vertical(r, rom, k as u16)? & 1 != 0
            };
            if hit {
                r.set_word(0x1b47 + k, 0xd218);
                r.set_word(0x1b8f + k, 1);
            }
        }
        0xd0eb => {}
        0xd0b3 => {
            let timer = r.word(0x1aff + k);
            if timer != 0 {
                r.set_word(0x1aff + k, timer - 1);
                if timer == 1 {
                    crate::audio::queue_sfx(r, 1, 0x6c06)?;
                }
            }
        }
        0xd1e4 => {
            let enemy = r.word(0x1b23 + k);
            if enemy > 0x7c0 || enemy & 63 != 0 {
                return Err("invalid Kago bug enemy slot");
            }
            let delta = r.word(0xf7a + usize::from(enemy)).wrapping_sub(r.word(0x1a4b + k));
            let absolute = if delta & 0x8000 != 0 { delta.wrapping_neg() } else { delta };
            if (absolute.wrapping_sub(23) as i16) >= 0 {
                r.set_word(0x1bd7 + k, r.word(0x1bd7 + k) | 0x8000);
            }
        }
        0xd0ca | 0xd0ec | 0xd128 => {
            preinstruction(r, rom, 0xd0b3, k as u16)?;
            preinstruction(r, rom, 0xd1e4, k as u16)?;
            if address == 0xd0ca {
                let timer = r.word(0x1bfb + k);
                if timer != 0 {
                    r.set_word(0x1bfb + k, timer - 1);
                } else {
                    r.set_word(0x1b47 + k, 0xd052);
                    r.set_word(0x1b8f + k, 1);
                    r.set_word(0x1a03 + k, 0xd0eb);
                }
            } else {
                let horizontal = crate::enemy_projectile_collision::horizontal(r, rom, k as u16)?;
                let mut transition = false;
                if horizontal & 1 != 0 {
                    r.set_word(0x1ab7 + k, 0);
                    if address == 0xd0ec {
                        r.set_word(0x1adb + k, 256);
                        transition = true;
                    }
                } else if crate::enemy_projectile_collision::vertical(r, rom, k as u16)? & 1 != 0 {
                    if address == 0xd0ec {
                        r.set_word(0x1adb + k, 256);
                    }
                    transition = true;
                } else {
                    let velocity = r.word(0x1adb + k).wrapping_add(224);
                    r.set_word(0x1adb + k, velocity);
                    transition = address == 0xd0ec && (velocity as i16) >= 0;
                }
                if transition {
                    r.set_word(0x1a03 + k, if address == 0xd0ec { 0xd128 } else { 0xd0eb });
                    r.set_word(0x1b47 + k, if address == 0xd0ec { 0xd04a } else { 0xd03c });
                    r.set_word(0x1b8f + k, 1);
                }
            }
        }
        0xcfd5 | 0xcfe6 => {
            r.set_word(0x1a03 + k, 0xcff8);
            r.set_word(
                0x1a4b + k,
                r.word(0x1a4b + k).wrapping_add(if address == 0xcfd5 { (-19_i16) as u16 } else { 19 }),
            );
        }
        0xcff8 => {
            if crate::enemy_projectile_collision::vertical(r, rom, k as u16)? & 1 != 0 {
                r.set_word(0x1b47 + k, 0xcf56);
                r.set_word(0x1b8f + k, 1);
            } else if crate::enemy_projectile_collision::horizontal(r, rom, k as u16)? & 1 != 0 {
                r.set_word(0x1ab7 + k, 0);
            } else {
                let velocity = r.word(0x1adb + k).wrapping_add(16);
                r.set_word(0x1adb + k, if (velocity.wrapping_sub(512) as i16) >= 0 { 512 } else { velocity });
            }
        }
        0xcafa => {
            r.set_word(0x1ab7 + k, 0);
            r.set_word(0x1adb + k, 0);
            r.set_word(0x1a4b + k, 128);
            r.set_word(0x1a93 + k, 192);
        }
        0xcbe7 | 0xcbea | 0xcc08 => {
            let host = host.ok_or("projectile preinstruction requires host context")?;
            let state = if address == 0xcbe7 { r.word(0x1aff + k) } else { address };
            if !matches!(state, 0xcbea | 0xcc08) {
                return Err("unsupported falling Mother Brain tube state");
            }
            if state == 0xcbea {
                host.x = r.word(0x1a4b + k);
                host.y = r.word(0x1a93 + k).wrapping_add(8);
                crate::enemy_projectiles::spawn_room_with_host(r, rom, 0xe509, 9, host)?;
                r.set_word(0x1aff + k, 0xcc08);
            }
            r.set_word(0x1adb + k, r.word(0x1adb + k).wrapping_add(6));
            let y = move_y(r, k as u16)?;
            if (y.wrapping_sub(208) as i16) >= 0 {
                r.set_word(0x1997 + k, 0);
                host.x = r.word(0x1a4b + k);
                host.y = y;
                crate::enemy_projectiles::spawn_room_with_host(r, rom, 0xe509, 12, host)?;
            }
        }
        0xc9d2 => {
            let host = host.ok_or("projectile preinstruction requires host context")?;
            let velocity = r.word(0x1ab7 + k);
            let absolute = if velocity & 0x8000 != 0 { velocity.wrapping_neg() } else { velocity };
            let slowed = absolute.wrapping_sub(16);
            let slowed = if (slowed as i16) < 0 { 0 } else { slowed };
            r.set_word(0x1ab7 + k, if velocity & 0x8000 != 0 { slowed.wrapping_neg() } else { slowed });
            r.set_word(0x1adb + k, r.word(0x1adb + k).wrapping_add(32));
            move_xy(r, k as u16)?;
            let timer = r.word(0x1aff + k).wrapping_sub(1);
            r.set_word(0x1aff + k, timer);
            if timer & 0x8000 != 0 {
                r.set_word(0x1997 + k, 0);
                r.set_word(0x1a93 + k, r.word(0x1a93 + k).wrapping_sub(4));
                host.x = r.word(0x1a4b + k);
                host.y = r.word(0x1a93 + k);
                crate::enemy_projectiles::spawn_room_with_host(r, rom, 0xe509, 9, host)?;
            }
        }
        0xc914 | 0xc94c => {
            let (x, y) = if address == 0xc914 { (0xf7a, 0xf7e) } else { (0xaf6, 0xafa) };
            r.set_word(0x1a4b + k, r.word(x).wrapping_add(r.word(0x1ab7 + k)));
            r.set_word(0x1a93 + k, r.word(y).wrapping_add(r.word(0x1adb + k)));
        }
        0xc84d => {
            const OFFSETS: [(u16, u16); 6] = [(6, 20), (14, 18), (8, 23), (10, 19), (11, 25), (12, 18)];
            let &(x, y) = OFFSETS.get(usize::from(r.word(0x1aff + k))).ok_or("invalid Mother Brain drool offset")?;
            r.set_word(0x1a4b + k, r.word(0xfba).wrapping_add(x));
            r.set_word(0x1a93 + k, r.word(0xfbe).wrapping_add(y));
            r.set_word(0x1ab7 + k, 0);
            r.set_word(0x1adb + k, 0);
        }
        0xc886 => {
            r.set_word(0x1adb + k, r.word(0x1adb + k).wrapping_add(12));
            if move_y(r, k as u16)? >= 0xd7 {
                r.set_word(0x1a93 + k, r.word(0x1a93 + k).wrapping_sub(4));
                r.set_word(0x1b47 + k, 0xc8e1);
                r.set_word(0x1b8f + k, 1);
            }
        }
        0xc814 => {
            r.set_word(0x1a4b + k, r.word(0xfba));
            r.set_word(0x1a93 + k, r.word(0xfbe));
        }
        0xa3a3 => r.set_word(0x923, 0),
        0xa328 => {
            let timer = r.word(0x1aff + k);
            let expired = if timer == 0 {
                true
            } else {
                let remaining = timer.wrapping_sub(1);
                r.set_word(0x1aff + k, remaining);
                (remaining as i16) <= 0
            };
            if expired {
                let height = r.word(0xafa);
                r.set_word(0x1a93 + k, height.wrapping_add(28));
                let next_height = height.wrapping_add(1);
                r.set_word(0xafa, next_height);
                if (next_height.wrapping_sub(73) as i16) >= 0 {
                    r.set_word(0xafa, 72);
                    r.set_word(0x1b8f + k, 1);
                    r.set_word(0x1b47 + k, 0xa28b);
                    // SamusCode_0E releases the elevator's Samus frame handlers.
                    r.set_word(0xa42, 0xe695);
                    r.set_word(0xa44, 0xe8aa);
                    // CallSomeSamusCode clears pending transitions for an odd result.
                    for address in [0xa28, 0xa2a, 0xa2c] {
                        r.set_word(address, 0xffff);
                    }
                    for address in [0xa2e, 0xa30, 0xa32] {
                        r.set_word(address, 0);
                    }
                }
            }
        }
        0xa364 => {
            if r.word(0xafa) == 72 {
                r.set_word(0x1b8f + k, 1);
                r.set_word(0x1b47 + k, 0xa28b);
            }
        }
        0xbfdf => {
            let host = host.ok_or("projectile preinstruction requires host context")?;
            let x = r.word(0x1a4b + k);
            let y = r.word(0x1a93 + k);
            let sx = x.wrapping_add(4).wrapping_sub(r.word(0x911));
            let sy = y.wrapping_add(16).wrapping_sub(r.word(0x915));
            let outside = (x as i16) < 0
                || (y as i16) < 0
                || (sx as i16) < 0
                || (sy as i16) < 0
                || (sx.wrapping_sub(264) as i16) >= 0
                || (sy.wrapping_sub(256) as i16) >= 0;
            if r.word(0x783a) != 0 {
                r.set_word(0x1997 + k, 0);
                if !outside {
                    host.x = x;
                    host.y = y;
                    crate::enemy_projectiles::spawn_room_with_host(r, rom, 0xe509, 12, host)?;
                }
            } else if !outside {
                let timer = r.word(0x1ab7 + k);
                r.set_word(0x1ab7 + k, timer.wrapping_sub(1));
                if timer == 1 {
                    dispatch_rom(r, rom, "Eproj_SetXvelRandom", k as i32, 0)?;
                    dispatch_rom(r, rom, "Eproj_MotherBrainRoomTurretBullets_Func2", k as i32, 0)?;
                    r.set_word(0x1b47 + k, word(rom, 0xc040 + u16::from(r.byte(0x1a6f + k)) * 2)?);
                    r.set_word(0x1b8f + k, 1);
                }
                let timer = r.word(0x1adb + k);
                r.set_word(0x1adb + k, timer.wrapping_sub(1));
                if timer == 1 {
                    dispatch_rom(r, rom, "Eproj_SetYvelRandom", k as i32, 0)?;
                    crate::enemy_projectiles::spawn_room_with_host(r, rom, 0xc18c, k as u16, host)?;
                }
            }
        }
        0x940e => {
            let host = host.ok_or("projectile preinstruction requires host context")?;
            let explosion = if crate::enemy_projectile_collision::horizontal(r, rom, k as u16)? & 1 != 0 {
                Some(0x965e)
            } else if crate::enemy_projectile_collision::vertical(r, rom, k as u16)? & 1 != 0 {
                Some(0x9650)
            } else {
                None
            };
            if let Some(definition) = explosion {
                r.set_word(0x1997 + k, 0);
                if r.word(0x1b23 + k) == 0 {
                    host.x = r.word(0x1a4b + k);
                    host.y = r.word(0x1a93 + k);
                    crate::enemy_projectiles::spawn_room_with_host(r, rom, definition, 3, host)?;
                    crate::audio::queue_sfx(r, 1, 0x2b06)?;
                }
            }
        }
        0xec05 => {
            dispatch_rom(r, rom, "Eproj_FuncE73E_MoveXY", k as i32, 0)?;
            if lavaquake_outside(r, k) {
                r.set_word(0x1997 + k, 0);
            }
        }
        0xea80 => {
            let mut state = r.word(0x1ab7 + k);
            if r.word(0x8020) != 0 && state == 0xea98 {
                state = 0xeaf4;
                r.set_word(0x1ab7 + k, state);
            }
            match state {
                0xea98 => {
                    let direction = r.word(0x1aff + k) & !1;
                    if direction >= 64 {
                        return Err("Botwoon body script extent");
                    }
                    let script = word(rom, 0xe9f1 + direction)?;
                    if script != r.word(0x1b23 + k) {
                        r.set_word(0x1b47 + k, script);
                        r.set_word(0x1b23 + k, script);
                        r.set_word(0x1b8f + k, 1);
                    }
                }
                0xeaf4 | 0xeb04 => {
                    if state == 0xeaf4 {
                        r.set_word(0x1aff + k, k as u16 * 4 + 96);
                        r.set_word(0x1ab7 + k, 0xeb04);
                    }
                    let timer = r.word(0x1aff + k).wrapping_add(1);
                    r.set_word(0x1aff + k, timer);
                    if (timer.wrapping_sub(256) as i16) >= 0 {
                        r.set_word(0x1ab7 + k, 0xeb1f);
                    }
                    r.set_word(0x1b8f + k, 0);
                }
                0xeb1f => {
                    let row = usize::from(r.word(0x1adb + k) >> 8);
                    if row >= 95 {
                        return Err("Botwoon falling speed extent");
                    }
                    let p = 0x104bc7 + row * 8;
                    let bytes = rom.get(p..p + 4).ok_or("missing Botwoon falling ROM")?;
                    let amount = u32::from_le_bytes(bytes.try_into().unwrap());
                    let position = (u32::from(r.word(0x1a93 + k)) << 16) | u32::from(r.word(0x1a6f + k));
                    let next = position.wrapping_add(amount);
                    r.set_word(0x1a93 + k, (next >> 16) as u16);
                    r.set_word(0x1a6f + k, next as u16);
                    if (r.word(0x1a93 + k).wrapping_sub(200) as i16) < 0 {
                        r.set_word(0x1adb + k, r.word(0x1adb + k).wrapping_add(192));
                        r.set_word(0x1b8f + k, 0);
                    } else {
                        r.set_word(0x1a93 + k, 200);
                        r.set_word(0x1ab7 + k, 0xeb93);
                        r.set_word(0x1b47 + k, 0xe208);
                        r.set_word(0x19bb + k, 2560);
                        r.set_word(0x1b8f + k, 1);
                        crate::audio::queue_sfx(r, 1, 0x2406)?;
                        if k == 10 {
                            r.set_word(0x883e, 1);
                        }
                        return Ok(());
                    }
                }
                0xeb93 => return Ok(()),
                _ => return Err("unsupported Botwoon body state"),
            }
            let mut gfx = r.word(0x19bb + k) | 0xe00;
            if r.word(0xf9c) != 0 && r.word(0xe44) & 2 != 0 {
                gfx &= 0xf1ff;
            }
            r.set_word(0x19bb + k, gfx);
        }
        0x8dca => {
            dispatch_rom(r, rom, "EprojPowerBombCollision", k as i32, 0)?;
            let release = if r.word(0xa6e) != 0 {
                true
            } else {
                r.set_word(0x1a4b + k, r.word(0xaf6));
                r.set_word(0x1a93 + k, r.word(0xafa).wrapping_add(r.word(0x1b23 + k).wrapping_mul(4)).wrapping_sub(12));
                let timer = r.word(0x1aff + k).wrapping_sub(1);
                r.set_word(0x1aff + k, timer);
                timer == 0
            };
            if release {
                r.set_word(0x1997 + k, 0);
                let divisor = r.word(0xa66).wrapping_sub(1);
                r.set_word(0xa66, if divisor & 0x8000 != 0 { 0 } else { divisor });
            }
        }
        0x8dff | 0x8e0f => {
            dispatch_rom(r, rom, "EprojPowerBombCollision", k as i32, 0)?;
            dispatch_rom(r, rom, "Eproj_FuncE73E_MoveXY", k as i32, 0)?;
            let abs = |v: u16| if v & 0x8000 != 0 { v.wrapping_neg() } else { v };
            let near = address == 0x8e0f
                && (abs(r.word(0xaf6).wrapping_sub(r.word(0x1a4b + k))).wrapping_sub(16) as i16) < 0
                && (abs(r.word(0xafa).wrapping_sub(r.word(0x1a93 + k))).wrapping_sub(20) as i16) < 0;
            if near {
                r.set_word(0x1b47 + k, 0x8c38);
                r.set_word(0x1b8f + k, 1);
            } else if r.word(0x1a4b + k) >= 512 || r.word(0x1a93 + k) >= 512 {
                r.set_word(0x1997 + k, 0);
            }
        }
        0x8a7d => {
            move_y(r, k as u16)?;
            if r.word(0x1a93 + k) >= r.word(0x19df + k) {
                r.set_word(0x1997 + k, 0);
            }
        }
        0x8b5d => {
            move_xy(r, k as u16)?;
            r.set_word(0x1adb + k, r.word(0x1adb + k).wrapping_add(80));
            if outside_screen(r, k) {
                r.set_word(0x1997 + k, 0);
            }
        }
        0xe605 => {
            let velocity = r.word(0x1adb + k);
            let distance = if velocity & 0x8000 != 0 { velocity.wrapping_neg() } else { velocity };
            let timer = r.word(0x19df + k).wrapping_add(distance);
            if timer >= 0x1000 {
                r.set_word(0x1b8f + k, 1);
                r.set_word(0x1b47 + k, r.word(0x1b47 + k).wrapping_add(2));
            }
            r.set_word(0x19df + k, if timer >= 0x1000 { 0 } else { timer });
            r.set_word(0x1a6f + k, r.word(0x1a6f + k).wrapping_add(velocity << 8));
            r.set_word(0x1a93 + k, r.word(0x1a93 + k).wrapping_add((velocity >> 8) as u8 as i8 as u16));
        }
        0xdb5b | 0xdf39 | 0xe049 => {
            let state = r.word(0x1aff + k);
            let velocity = match (address, state) {
                (0xdb5b, 0xdb62) | (0xdf39, 0xdf40) | (0xe049, 0xe050) => r.word(0x1adb + k),
                (0xdb5b, 0xdb8c) | (0xdf39, 0xdf6a) | (0xe049, 0xe07a) => r.word(0x1ab7 + k),
                _ => return Err("unsupported horizontal enemy projectile state"),
            };
            add_hilo(r, k, 0x1a4b, 0x1a27, velocity);
            if lavaquake_outside(r, k) {
                r.set_word(0x1997 + k, 0);
            }
        }
        0xdb62 | 0xdb8c | 0xe050 | 0xe07a => {
            let velocity = r.word(if matches!(address, 0xdb62 | 0xe050) { 0x1adb } else { 0x1ab7 } + k);
            add_hilo(r, k, 0x1a4b, 0x1a27, velocity);
        }
        0xdcee => {
            let phase = r.byte(0x1aff + k);
            let mut dx = byte(rom, 0xdd6c + u16::from(phase))? as i8 as i16 as u16;
            if r.word(0x1b23 + k) & 0x80 != 0 {
                dx = dx.wrapping_neg();
            }
            let dy = byte(rom, 0xdd6d + u16::from(phase))? as i8 as i16 as u16;
            r.set_word(0x1a4b + k, r.word(0x1a4b + k).wrapping_add(dx));
            let y = r.word(0x1a93 + k).wrapping_add(dy.wrapping_mul(2));
            r.set_word(0x1a93 + k, y);
            if (y.wrapping_sub(768) as i16) >= 0 {
                r.set_word(0x1997 + k, 0);
            }
            r.set_word(0x1aff + k, u16::from(phase.wrapping_add(2)));
        }
        0xdd46 => {
            if r.word(0x9000) == 0 {
                if r.word(0x1b23 + k) == 0 {
                    r.set_word(0x1b47 + k, 0xdc06);
                    r.set_word(0x1b8f + k, 1);
                    let timer = crate::enemy_projectiles::random(r) & 0x1ff;
                    r.set_word(0x1b23 + k, timer);
                }
                r.set_word(0x1b23 + k, r.word(0x1b23 + k).wrapping_sub(1));
            }
        }
        0xc0e0 => {
            r.set_word(0x1bd7 + k, r.word(0x1bd7 + k) ^ 0x8000);
            move_xy(r, k as u16)?;
            let product = u16::from((r.word(0x1a93 + k) >> 4) as u8) * u16::from(r.word(0x7a5) as u8);
            let block = usize::from(product) + usize::from(r.word(0x1a4b + k) >> 4);
            if block >= 0x3200 {
                return Err("turret bullet terrain extent");
            }
            if r.word(0x10002 + block * 2) & 0xf000 != 0 {
                let active = slot(r.word(0x1991))?;
                r.set_word(0x1997 + active, 0);
            }
        }
        0xbe03 => {
            if crate::enemy_projectile_collision::horizontal(r, rom, k as u16)? != 0
                || crate::enemy_projectile_collision::vertical(r, rom, k as u16)? != 0
            {
                r.set_word(0x1997 + k, 0);
            }
        }
        0xbe12 => {
            // C indexes by E >> 1, including odd byte-offset aliases.
            let linked = usize::from(r.word(0x1aff + k) & !1);
            if linked > 34 {
                return Err("invalid linked Shaktool projectile slot");
            }
            if r.word(0x1997 + linked) != 0 {
                crate::enemy_projectile_collision::horizontal(r, rom, k as u16)?;
                crate::enemy_projectile_collision::vertical(r, rom, k as u16)?;
            } else {
                r.set_word(0x1997 + k, 0);
            }
        }
        0xbc0f => {
            let state = r.word(0x1aff + k);
            if !matches!(state, 0xbc16 | 0xbc8f) {
                return Err("unsupported lavaquake rock state");
            }
            lavaquake_arc(r, rom, k, state)?;
            if lavaquake_outside(r, k) {
                r.set_word(0x1997 + k, 0);
            }
        }
        0xb977 => r.set_word(0x1a93 + k, r.word(0x195e).wrapping_sub(4)),
        0xb982 => {
            move_x(r, k as u16)?;
            let before = r.word(0x195e).wrapping_sub(r.word(0x1a93 + k));
            move_y(r, k as u16)?;
            let after = r.word(0x195e).wrapping_sub(r.word(0x1a93 + k));
            if (before ^ after) & 0x8000 != 0 {
                crate::enemy_projectiles::spawn_room(r, rom, 0xba5c, k as u16)?;
            }
            if r.word(0x1a93 + k) & 0xff00 == 0x100 {
                r.set_word(0x1b47 + k, 0xb79f);
                r.set_word(0x1b8f + k, 1);
            } else {
                r.set_word(0x1adb + k, r.word(0x1adb + k).wrapping_add(16));
            }
        }
        0xb9fd => {
            move_y(r, k as u16)?;
            if r.word(0x1a93 + k) & 0x100 != 0 {
                r.set_word(0x1b47 + k, 0xb79f);
                r.set_word(0x1b8f + k, 1);
            }
            r.set_word(0x1adb + k, r.word(0x1adb + k).wrapping_sub(128));
        }
        0xba37 | 0xba42 => {
            if address == 0xba37 && r.word(0x1e6f) == 0 {
                r.set_word(0x1e6d, r.word(0x1e6d) | 0x8000);
            }
            r.set_word(0x1a4b + k, r.word(0x1aff + k));
            r.set_word(0x1a93 + k, r.word(0x1b23 + k).wrapping_add(r.word(0x915)).wrapping_sub(r.word(0x9e00)));
        }
        0xb38a => {
            if crate::enemy_projectile_collision::horizontal(r, rom, k as u16)? != 0 {
                r.set_word(0x1b47 + k, 0xb3cd);
                r.set_word(0x1b8f + k, 1);
            } else if crate::enemy_projectile_collision::vertical(r, rom, k as u16)? != 0 {
                r.set_word(0x1a93 + k, ((r.word(0x1a93 + k) & 0xfff0) | 8).wrapping_sub(2));
                r.set_word(0x1b47 + k, 0xb3e5);
                r.set_word(0x1b8f + k, 1);
            }
        }
        0xb20d => {
            let enemy = r.word(0x1aff + k);
            if enemy > 0x7c0 || enemy & 63 != 0 {
                return Err("invalid tracked Torizo enemy slot");
            }
            let p = 0xf78 + usize::from(enemy);
            r.set_word(
                0x1a4b + k,
                r.word(p + 2).wrapping_add(if r.word(p + 0x3c) & 0x8000 != 0 { 32 } else { 0xffe0 }),
            );
            r.set_word(0x1a93 + k, r.word(p + 6).wrapping_sub(52));
        }
        0xb237 => {
            let hit = if crate::enemy_projectile_collision::horizontal(r, rom, k as u16)? != 0 {
                true
            } else {
                crate::enemy_projectile_collision::vertical(r, rom, k as u16)? != 0 && r.word(0x1adb + k) & 0x8000 == 0
            };
            if hit {
                r.set_word(0x1b47 + k, 0xb2ef);
                r.set_word(0x1b8f + k, 1);
            } else {
                let vy = r.word(0x1adb + k).wrapping_add(16);
                r.set_word(0x1adb + k, vy);
                if vy & 0xf000 == 0x1000 {
                    r.set_word(0x1997 + k, 0);
                }
            }
        }
        0xb043 => {
            let timer = r.word(0x1b23 + k).wrapping_sub(1);
            r.set_word(0x1b23 + k, timer);
            if timer & 0x8000 != 0 {
                r.set_word(0x1b47 + k, r.word(0x1b47 + k).wrapping_add(2));
                r.set_word(0x1b8f + k, 1);
                r.set_word(0x1ab7 + k, if r.word(0x1aff + k) & 0x8000 != 0 { 256 } else { 0xff00 });
            } else {
                if crate::enemy_projectile_collision::horizontal(r, rom, k as u16)? != 0 {
                    r.set_word(0x1ab7 + k, r.word(0x1ab7 + k).wrapping_neg());
                    r.set_word(0x1aff + k, r.word(0x1aff + k) ^ 0x8000);
                }
                if crate::enemy_projectile_collision::vertical(r, rom, k as u16)? != 0
                    && r.word(0x1adb + k) & 0x8000 == 0
                {
                    let vx = r.word(0x1ab7 + k);
                    r.set_word(0x1ab7 + k, vx.wrapping_add(if vx & 0x8000 != 0 { 32 } else { 0xffe0 }));
                    r.set_word(0x1adb + k, r.word(0x1adb + k).wrapping_neg());
                }
                let vy = r.word(0x1adb + k).wrapping_add(48);
                r.set_word(0x1adb + k, vy);
                if vy & 0xf000 == 0x1000 {
                    r.set_word(0x1997 + k, 0);
                }
            }
        }
        0xb0b9 => {
            if crate::enemy_projectile_collision::horizontal(r, rom, k as u16)? != 0 {
                r.set_word(0x1a03 + k, 0xb0dd);
                r.set_word(0x1adb + k, 0);
            } else {
                r.set_word(
                    0x1ab7 + k,
                    r.word(0x1ab7 + k).wrapping_add(if r.word(0x1aff + k) & 0x8000 != 0 { 48 } else { 0xffd0 }),
                );
            }
        }
        0xb0dd => {
            if crate::enemy_projectile_collision::vertical(r, rom, k as u16)? != 0 {
                r.set_word(0x1b47 + k, if r.word(0x1aff + k) & 0x8000 != 0 { 0xb1a8 } else { 0xb190 });
                r.set_word(0x1b8f + k, 1);
            } else {
                r.set_word(0x1adb + k, r.word(0x1adb + k).wrapping_add(48));
            }
        }
        0xae6c => {
            if crate::enemy_projectile_collision::horizontal(r, rom, k as u16)? != 0 {
                r.set_word(0x1b47 + k, 0xade5);
                r.set_word(0x1b8f + k, 1);
                r.set_word(0x1aff + k, r.word(0x1a4b + k));
                r.set_word(0x1b23 + k, r.word(0x1a93 + k));
            } else {
                let old = r.word(0x1ab7 + k);
                let next = old.wrapping_add(if old & 0x8000 != 0 { 0xfff0 } else { 16 });
                r.set_word(0x1ab7 + k, next);
                if next & 0xf000 == 0x1000 {
                    r.set_word(0x1997 + k, 0);
                }
            }
        }
        0xacad => {
            if crate::enemy_projectile_collision::horizontal(r, rom, k as u16)? != 0 {
                r.set_word(0x1b47 + k, 0xab25);
                r.set_word(0x1b8f + k, 1);
            } else {
                let hit = crate::enemy_projectile_collision::vertical(r, rom, k as u16)? != 0;
                if r.word(0x1adb + k) & 0x8000 != 0 || !hit {
                    let next = r.word(0x1adb + k).wrapping_add(18);
                    r.set_word(0x1adb + k, next);
                    if next & 0xf000 == 0x1000 {
                        r.set_word(0x1997 + k, 0);
                    }
                } else {
                    r.set_word(0x1a93 + k, ((r.word(0x1a93 + k) & 0xfff0) | 8).wrapping_sub(2));
                    r.set_word(0x1b47 + k, 0xab41);
                    r.set_word(0x1b8f + k, 1);
                }
            }
        }
        0xacfa => {
            if crate::enemy_projectile_collision::horizontal(r, rom, k as u16)? != 0 {
                r.set_word(0x1ab7 + k, r.word(0x1ab7 + k).wrapping_neg());
            }
            let hit = crate::enemy_projectile_collision::vertical(r, rom, k as u16)? != 0;
            let mut settled = false;
            if hit && r.word(0x1adb + k) & 0x8000 == 0 {
                let vx = r.word(0x1ab7 + k);
                r.set_word(0x1ab7 + k, if vx & 0x8000 == 0 { vx.wrapping_sub(64) } else { vx.wrapping_add(64) });
                let vy = (r.word(0x1adb + k) >> 1).wrapping_neg();
                r.set_word(0x1adb + k, vy);
                settled = vy & 0xff80 == 0xff80;
            }
            if settled {
                r.set_word(0x1a93 + k, ((r.word(0x1a93 + k) & 0xfff0) | 8).wrapping_sub(2));
                r.set_word(0x1b47 + k, 0xab41);
                r.set_word(0x1b8f + k, 1);
            } else {
                r.set_word(0x1adb + k, r.word(0x1adb + k).wrapping_add(24));
            }
        }
        0xa887 => {
            if crate::enemy_projectile_collision::horizontal(r, rom, k as u16)? != 0 {
                r.set_word(0x1b47 + k, 0xa48a);
                r.set_word(0x1b8f + k, 1);
            } else {
                let old = r.word(0x1ab7 + k) as i16;
                let next = if old >= 0 { old.wrapping_sub(4) } else { old.wrapping_add(4) };
                let crossed = if old >= 0 { next < 0 } else { next >= 0 };
                r.set_word(0x1ab7 + k, if crossed { 3 } else { next as u16 });
                let hit = crate::enemy_projectile_collision::vertical(r, rom, k as u16)? != 0;
                if r.word(0x1adb + k) & 0x8000 != 0 || !hit {
                    let next = r.word(0x1adb + k).wrapping_add(16);
                    r.set_word(0x1adb + k, next);
                    if next & 0xf000 == 0x1000 {
                        r.set_word(0x1997 + k, 0);
                    }
                } else {
                    r.set_word(0x1a93 + k, r.word(0x1a93 + k).wrapping_sub(3));
                    r.set_word(0x1b47 + k, 0xa48e);
                    r.set_word(0x1b8f + k, 1);
                }
            }
        }
        0xa05b => {}
        0xa05c | 0xa07a | 0xa0d1 | 0xa124 => {
            let left = matches!(address, 0xa05c | 0xa0d1);
            let delta = if matches!(address, 0xa05c | 0xa07a) {
                if r.word(0x1aff + k) & 0x8000 == 0 { 4 } else { 2 }
            } else {
                let speed = r.word(0x1aff + k);
                let outward = r.word(0x1b23 + k) != 0;
                let next = if outward { speed.wrapping_sub(32) } else { speed.wrapping_add(32) };
                r.set_word(0x1aff + k, next);
                if outward && next == 0 {
                    r.set_word(0x1b23 + k, 0);
                }
                r.set_word(0x1a93 + k, r.word(0x1a93 + k).wrapping_add(1));
                let delta = speed >> 8;
                if outward { delta } else { delta.wrapping_neg() }
            };
            let x = r.word(0x1a4b + k).wrapping_add(if left { delta.wrapping_neg() } else { delta });
            r.set_word(0x1a4b + k, x);
            let y = r.word(0x1a93 + k);
            let cx = r.word(0x911);
            let cy = r.word(0x915);
            if (x.wrapping_sub(cx) as i16) < 0
                || x.wrapping_sub(cx.wrapping_add(256)) as i16 >= 0
                || (y.wrapping_sub(cy) as i16) < 0
                || y.wrapping_sub(cy.wrapping_add(256)) as i16 >= 0
            {
                r.set_word(0x1997 + k, 0);
            }
        }
        0x9eff => {
            if crate::enemy_projectile_collision::vertical(r, rom, k as u16)? != 0
                || crate::enemy_projectile_collision::horizontal(r, rom, k as u16)? != 0
            {
                r.set_word(0x1997 + k, 0);
            } else {
                let old = r.word(0x1ab7 + k);
                let next = if old & 0x8000 == 0 {
                    let value = old.wrapping_sub(64);
                    if (value.wrapping_sub(512) as i16) < 0 { 512 } else { value }
                } else {
                    let value = old.wrapping_add(64);
                    if value.wrapping_add(512) as i16 >= 0 { 0xfe00 } else { value }
                };
                r.set_word(0x1ab7 + k, next);
            }
        }
        0x9e1e => {
            if crate::enemy_projectile_collision::horizontal(r, rom, k as u16)? != 0
                || crate::enemy_projectile_collision::vertical(r, rom, k as u16)? != 0
            {
                r.set_word(0x1997 + k, 0);
            } else {
                let old = r.word(0x1adb + k);
                let next = old.wrapping_add(64);
                r.set_word(0x1adb + k, if next as i16 >= 0 && old.wrapping_sub(960) as i16 >= 0 { 1024 } else { next });
            }
        }
        0x9e83 => {
            if crate::enemy_projectile_collision::horizontal(r, rom, k as u16)? != 0 {
                r.set_word(0x1997 + k, 0);
            }
        }
        0x9d56 => {
            if crate::enemy_projectile_collision::horizontal(r, rom, k as u16)? != 0
                || crate::enemy_projectile_collision::vertical(r, rom, k as u16)? != 0
            {
                r.set_word(0x1997 + k, 0);
            } else {
                let vx = r.word(0x1ab7 + k).wrapping_add(8);
                r.set_word(0x1ab7 + k, if vx.wrapping_sub(256) as i16 >= 0 { 0xff00 } else { vx });
                r.set_word(0x1adb + k, r.word(0x1adb + k).wrapping_add(64));
            }
        }
        0x9d89 => {
            if crate::enemy_projectile_collision::vertical(r, rom, k as u16)? != 0 {
                r.set_word(0x1997 + k, 0);
            } else {
                r.set_word(0x1adb + k, r.word(0x1adb + k).wrapping_add(24) & 0x3fff);
            }
        }
        0x9da5 => {
            r.set_word(0x19bb + k, 0);
        }
        0x9a45 | 0x9ada => {
            let step = if address == 0x9a45 { 4 } else { 2 };
            let radius = r.word(0x1adb + k).wrapping_add(step);
            r.set_word(0x1adb + k, radius);
            let angle =
                r.word(0x1aff + k).wrapping_add(if address == 0x9a45 { r.word(0x1ab7 + k) & 255 } else { 2 }) as u8;
            r.set_word(0x1aff + k, u16::from(angle));
            let (dx, dy) = crate::enemy_projectiles::phantoon_offset(angle, radius);
            let x = r.word(0xf7a).wrapping_add(dx);
            r.set_word(0x1a4b + k, x);
            let mut outside = (x as i16) < 0 || x.wrapping_sub(256) as i16 >= 0;
            if !outside {
                let y = r.word(0xf7e).wrapping_add(dy).wrapping_add(16);
                r.set_word(0x1a93 + k, y);
                outside = (y as i16) < 0 || y.wrapping_sub(256) as i16 >= 0;
            }
            if outside {
                r.set_word(0x1b47 + k, 0x97f8);
                r.set_word(0x1b8f + k, 1);
            }
        }
        0x9a94 => {
            let old = r.word(0x1ab7 + k);
            if old != 0 {
                let next = old.wrapping_sub(1);
                r.set_word(0x1ab7 + k, next);
                if old != 1 && next & 0x8000 == 0 {
                    return Ok(());
                }
                crate::audio::queue_sfx(r, 2, 0x1d06)?;
            }
            r.set_word(0x1adb + k, r.word(0x1adb + k).wrapping_add(16));
            if crate::enemy_projectile_collision::vertical(r, rom, k as u16)? != 0 {
                r.set_word(0x1b47 + k, 0x97ac);
                r.set_word(0x1b8f + k, 1);
                r.set_word(0x1a93 + k, r.word(0x1a93 + k).wrapping_add(8));
                r.set_word(0x1a03 + k, 0x9a44);
                crate::audio::queue_sfx(r, 2, 0x1d06)?;
            }
        }
        0x9b29 => {
            if r.word(0xfaa) != 0 {
                r.set_word(0x1a03 + k, 0x9b41);
                r.set_word(0x1ab7 + k, 180);
                r.set_word(0x1adb + k, 48);
            }
        }
        0x9b41 => {
            let delay = r.word(0x1ab7 + k);
            if delay != 0 {
                r.set_word(0x1ab7 + k, delay - 1);
            } else if r.word(0x5b6) & 1 != 0 {
                let radius = r.word(0x1adb + k).wrapping_sub(1);
                r.set_word(0x1adb + k, radius);
                if radius == 0 {
                    r.set_word(0x1a4b + k, r.word(0xf7a));
                    r.set_word(0x1a93 + k, r.word(0xf7e).wrapping_add(16));
                    r.set_word(0x1b8f + k, 1);
                    r.set_word(0x1b47 + k, 0x97f8);
                    return Ok(());
                }
            }
            let angle = r.word(0x1aff + k).wrapping_add(1) as u8;
            r.set_word(0x1aff + k, u16::from(angle));
            let (x, y) = crate::enemy_projectiles::phantoon_offset(angle, r.word(0x1adb + k));
            r.set_word(0x1a4b + k, r.word(0xf7a).wrapping_add(x));
            r.set_word(0x1a93 + k, r.word(0xf7e).wrapping_add(y).wrapping_add(16));
        }
        0x9a44 => {}
        0x9981 => {
            r.set_word(0x1adb + k, r.word(0x1adb + k).wrapping_add(16));
            if crate::enemy_projectile_collision::vertical(r, rom, k as u16)? != 0 {
                r.set_word(0x1bd7 + k, (r.word(0x1bd7 + k) & 0x0fff) | 0x8000);
                r.set_word(0x1a03 + k, 0x99bf);
                r.set_word(0x1b47 + k, 0x976c);
                r.set_word(0x1b8f + k, 1);
                r.set_word(0x1aff + k, 8);
                r.set_word(0x1a93 + k, r.word(0x1a93 + k).wrapping_add(8));
            }
        }
        0x99bf => {
            let old = r.word(0x1aff + k);
            let next = old.wrapping_sub(1);
            r.set_word(0x1aff + k, next);
            if old == 1 || next & 0x8000 != 0 {
                r.set_word(0x1a03 + k, 0x9a01);
                r.set_word(0x1b47 + k, 0x9772);
                r.set_word(0x1b8f + k, 1);
                r.set_word(0x1a93 + k, r.word(0x1a93 + k).wrapping_sub(8));
                r.set_word(0x1adb + k, 0xfd00);
                r.set_word(0x1aff + k, 0);
                r.set_word(0x1ab7 + k, if r.word(0x5b6) & 1 != 0 { 0xff80 } else { 128 });
            }
        }
        0x9a01 => {
            let count = r.word(0x1aff + k);
            if count > 2 {
                return Err("Phantoon fireball bounce extent");
            }
            r.set_word(0x1adb + k, r.word(0x1adb + k).wrapping_add(16));
            let vertical = crate::enemy_projectile_collision::vertical(r, rom, k as u16)? != 0;
            if vertical {
                let next = count + 1;
                r.set_word(0x1aff + k, next);
                if next < 3 {
                    r.set_word(0x1adb + k, [0xfd00, 0xfe00, 0xff00][usize::from(next)]);
                    return Ok(());
                }
            }
            if vertical || crate::enemy_projectile_collision::horizontal(r, rom, k as u16)? != 0 {
                r.set_word(0x1b47 + k, 0x9782);
                r.set_word(0x1b8f + k, 1);
                r.set_word(0x1a03 + k, 0x9a44);
            }
        }
        0x950d | 0x9522 => {
            let hit = if address == 0x950d {
                move_x(r, k as u16)?;
                crate::enemy_projectile_collision::vertical(r, rom, k as u16)?
            } else {
                move_y(r, k as u16)?;
                crate::enemy_projectile_collision::horizontal(r, rom, k as u16)?
            };
            if hit != 0 {
                r.set_word(0x1b47 + k, 0x9574);
                r.set_word(0x1b8f + k, 1);
            }
        }
        0x9537 | 0x9540 | 0x9549 => {
            let hit = if address == 0x9537 {
                crate::enemy_projectile_collision::horizontal(r, rom, k as u16)?
            } else {
                crate::enemy_projectile_collision::vertical(r, rom, k as u16)?
            };
            if hit != 0 {
                r.set_word(0x1997 + k, 0);
            }
        }
        0x92ba => {
            if crate::enemy_projectile_collision::vertical(r, rom, k as u16)? != 0 {
                r.set_word(0x1997 + k, 0);
            } else {
                r.set_word(0x1adb + k, r.word(0x1adb + k).wrapping_add(24) & 0x3fff);
            }
        }
        0x9392 => {
            let timer = r.word(0x1aff + k);
            if timer < 8 {
                r.set_word(0x1aff + k, timer + 1);
            } else {
                move_xy(r, k as u16)?;
                if crate::enemy_projectile_collision::vertical(r, rom, k as u16)? != 0 {
                    r.set_word(0x1b47 + k, 0x9574);
                    r.set_word(0x1aff + k, timer.wrapping_add(1));
                    r.set_word(0x1b8f + k, 1);
                    r.set_word(0x1ab7 + k, 0);
                    r.set_word(0x1adb + k, 0);
                    r.set_word(0x19bb + k, 2560);
                    crate::audio::queue_sfx(r, 1, 0x2b06)?;
                }
            }
        }
        0xefe0 => pickup(r, k)?,
        0x8170 | 0x84fb | 0xdd44 | 0xefdf => {}
        0x90b3 | 0xd3bf => {
            if address == 0xd3bf {
                r.set_word(0x19bb + k, 0);
            }
            if crate::enemy_projectile_collision::horizontal(r, rom, k as u16)? != 0
                || crate::enemy_projectile_collision::vertical(r, rom, k as u16)? != 0
            {
                r.set_word(0x1997 + k, 0);
            }
        }
        0xb6b9 => {
            let angle = usize::from(r.word(0x1aff + k) >> 1);
            if angle >= 256 {
                return Err("eye projectile angle extent");
            }
            let hit = crate::enemy_projectile_collision::horizontal(r, rom, k as u16)? != 0
                || crate::enemy_projectile_collision::vertical(r, rom, k as u16)? != 0;
            let opened = if hit {
                false
            } else {
                r.set_word(
                    0x1ab7 + k,
                    r.word(0x1ab7 + k).wrapping_add((crate::enemy_projectiles::SINE[angle + 64] >> 4) as u16),
                );
                r.set_word(
                    0x1adb + k,
                    r.word(0x1adb + k).wrapping_add((crate::enemy_projectiles::SINE[angle] >> 4) as u16),
                );
                let t = crate::system::dispatch(r, "PrepareBitAccess", r.word(0x1b23 + k), 0)? as usize;
                r.word(0x5e7) & u16::from(r.byte(0xd8b0 + t)) != 0
            };
            if hit || opened {
                r.set_word(0x1b47 + k, 0xb5f3);
                r.set_word(0x1b8f + k, 1);
            }
        }
        0xb714 => {
            crate::enemy_projectile_collision::horizontal(r, rom, k as u16)?;
            let hit = crate::enemy_projectile_collision::vertical(r, rom, k as u16)? != 0;
            if r.word(0x1adb + k) & 0x8000 != 0 || !hit {
                r.set_word(0x1adb + k, r.word(0x1adb + k).wrapping_add(12));
            } else {
                r.set_word(0x1a93 + k, r.word(0x1a93 + k).wrapping_sub(4));
                r.set_word(0x1b47 + k, 0xb61d);
                r.set_word(0x1b8f + k, 1);
            }
        }
        0xe4fe => {
            if outside_screen(r, k) {
                r.set_word(0x1997 + k, 0)
            }
        }
        0xd7bf => {
            if r.word(0x1a4b + k) != 0xee00 {
                r.set_word(0x1aff + k, r.word(0x1a4b + k))
            }
            r.set_word(0x1a4b + k, if r.word(0x5b6) & 1 != 0 { 0xee00 } else { r.word(0x1aff + k) })
        }
        0xd7de => add_hilo(r, k, 0x1a93, 0x1a6f, 192),
        0xd7fd => {
            add_hilo(r, k, 0x1b23, 0x1aff, r.word(0x1ab7 + k));
            add_hilo(r, k, 0x1a93, 0x1a6f, r.word(0x1adb + k));
            delete_below(r, k)
        }
        0xce9b => {
            add_hilo(r, k, 0x1a4b, 0x1a27, r.word(0x1ab7 + k));
            add_hilo(r, k, 0x1a93, 0x1a6f, r.word(0x1adb + k));
            if r.word(0x1a93 + k) & 0xff00 != 0 {
                r.set_word(0x1997 + k, 0)
            } else {
                r.set_word(0x1adb + k, r.word(0x1adb + k).wrapping_add(32));
                if crate::enemy_projectiles::random(r) & 0x420 == 0 {
                    crate::enemy_projectiles::spawn_room(r, rom, 0xcf0a, k as u16)?
                }
            }
        }
        0xb535 => {
            add_hilo(r, k, 0x1a4b, 0x1a27, r.word(0x1ab7 + k));
            add_hilo(r, k, 0x1a93, 0x1a6f, r.word(0x1adb + k));
            let old = r.word(0x1adb + k);
            let new = old.wrapping_add(32);
            r.set_word(0x1adb + k, new);
            if old as i16 >= 0 {
                delete_below(r, k)
            } else if new as i16 >= 0 {
                r.set_word(0x1b47 + k, if r.word(0x1ab7 + k) & 0x8000 == 0 { 0xb4e3 } else { 0xb4d7 });
                r.set_word(0x1b8f + k, 1)
            }
        }
        0xd83d | 0xd89f => {
            let active = slot(r.word(0x1991))?;
            let angle = usize::from((r.word(0x1ab7 + k) & 0x17e | 0x80) >> 1);
            let velocity = (crate::enemy_projectiles::SINE[angle + 64] >> 2) as u16;
            add_hilo(r, active, 0x1b23, 0x1aff, velocity);
            r.set_word(0x1ab7 + active, r.word(0x1ab7 + active).wrapping_add(if address == 0xd83d { 2 } else { 4 }));
            add_hilo(r, active, 0x1a93, 0x1a6f, r.word(0x1adb + active));
            if address == 0xd83d {
                delete_below(r, active)
            } else {
                r.set_word(0x1a4b + active, r.word(0x1b23 + active))
            }
        }
        0xd8df => {
            add_hilo(r, k, 0x1a93, 0x1a6f, r.word(0x1adb + k));
            r.set_word(0x1a4b + k, r.word(0x1b23 + k))
        }
        _ => return Err("unsupported enemy projectile preinstruction"),
    };
    let _ = rom;
    Ok(())
}
/// Descending canonical scheduler, bounded to reject cyclic malformed scripts.
pub fn run_all(r: &mut Wram<'_>, rom: &[u8], budget: usize) -> Result<(), &'static str> {
    if r.word(0x198d) & 0x8000 != 0
        && (0..36).step_by(2).any(|k| r.word(0x1997 + k) != 0 && matches!(r.word(0x1a03 + k), 0x940e | 0xbfdf))
    {
        return Err("projectile frame requires host context");
    }
    run_all_context(r, rom, budget, None)
}
pub fn run_all_with_host(
    r: &mut Wram<'_>,
    rom: &[u8],
    budget: usize,
    host: &mut crate::enemy_projectiles::ProjectileHost,
) -> Result<(), &'static str> {
    run_all_context(r, rom, budget, Some(host))
}
fn run_all_context(
    r: &mut Wram<'_>,
    rom: &[u8],
    mut budget: usize,
    mut host: Option<&mut crate::enemy_projectiles::ProjectileHost>,
) -> Result<(), &'static str> {
    if r.word(0x198d) & 0x8000 == 0 {
        return Ok(());
    }
    let mut index = 34i32;
    while index >= 0 {
        let k = slot(index as u16)?;
        r.set_word(0x1991, index as u16);
        if r.word(0x1997 + k) != 0 {
            preinstruction_context(r, rom, r.word(0x1a03 + k), k as u16, host.as_deref_mut())?;
            let active = slot(r.word(0x1991))?;
            let old = r.word(0x1b8f + active);
            r.set_word(0x1b8f + active, old.wrapping_sub(1));
            if old == 1 {
                let mut pc = r.word(0x1b47 + active);
                loop {
                    if budget == 0 {
                        return Err("enemy projectile instruction budget");
                    };
                    budget -= 1;
                    let opcode = word(rom, pc)?;
                    if opcode & 0x8000 == 0 {
                        let map = word(rom, pc.wrapping_add(2))?;
                        r.set_word(0x1b8f + active, opcode);
                        r.set_word(0x1b6b + active, map);
                        r.set_word(0x1b47 + active, pc.wrapping_add(4));
                        break;
                    }
                    pc = instruction_context(r, rom, opcode, pc.wrapping_add(2), active as u16, host.as_deref_mut())?;
                    if pc == 0 {
                        break;
                    }
                }
            }
            index = i32::from(r.word(0x1991));
        }
        index -= 2;
    }
    Ok(())
}
fn pickup(r: &mut Wram<'_>, k: usize) -> Result<(), &'static str> {
    let lifetime = r.word(0x1b23 + k).wrapping_sub(1);
    r.set_word(0x1b23 + k, lifetime);
    let abs = |value: u16| if value & 0x8000 != 0 { value.wrapping_neg() } else { value };
    let grapple = r.word(0xd32) != 0xc4f0
        && (lifetime.wrapping_sub(384) as i16) < 0
        && (abs(r.word(0x1a4b + k).wrapping_sub(r.word(0xd08))).wrapping_sub(16) as i16) < 0
        && (abs(r.word(0x1a93 + k).wrapping_sub(r.word(0xd0c))).wrapping_sub(16) as i16) < 0;
    let dx = abs(r.word(0xaf6).wrapping_sub(r.word(0x1a4b + k)));
    let dy = abs(r.word(0xafa).wrapping_sub(r.word(0x1a93 + k)));
    let collision = (dx < r.word(0xafe) || dx.wrapping_sub(r.word(0xafe)) < u16::from(r.byte(0x1bb3 + k)))
        && (dy < r.word(0xb00) || dy.wrapping_sub(r.word(0xb00)) < u16::from(r.byte(0x1bb4 + k)));
    if lifetime != 0 && (grapple || collision) {
        let kind = r.word(0x1aff + k) >> 1;
        let sound = match kind {
            1 | 2 => {
                let amount = if kind == 1 { 5 } else { 20 };
                let health = r.word(0x9c2).wrapping_add(amount);
                r.set_word(0x9c2, health);
                let maximum = r.word(0x9c4);
                if (health.wrapping_sub(maximum) as i16) >= 0 {
                    let reserve = r.word(0x9d6).wrapping_add(health).wrapping_sub(maximum);
                    let reserve_max = r.word(0x9d4);
                    let reserve = if (reserve.wrapping_sub(reserve_max) as i16) >= 0 { reserve_max } else { reserve };
                    r.set_word(0x9d6, reserve);
                    if reserve != 0 && r.word(0x9c0) == 0 {
                        r.set_word(0x9c0, 1);
                    }
                    r.set_word(0x9c2, maximum);
                }
                kind
            }
            3 | 5 => {
                let (current, maximum) = if kind == 3 { (0x9ce, 0x9d0) } else { (0x9ca, 0x9cc) };
                let value = r.word(current).wrapping_add(1);
                let maximum = r.word(maximum);
                r.set_word(
                    current,
                    if (value.wrapping_sub(maximum) as i16) >= 0 && value != maximum { maximum } else { value },
                );
                if kind == 3 { 5 } else { 4 }
            }
            4 => {
                let value = r.word(0x9c6).wrapping_add(2);
                r.set_word(0x9c6, value);
                let maximum = r.word(0x9c8);
                if (value.wrapping_sub(maximum) as i16) >= 0 {
                    let reserve = r.word(0x9d8).wrapping_add(value).wrapping_sub(maximum);
                    let limit = if (maximum.wrapping_sub(99) as i16) < 0 { maximum } else { 99 };
                    r.set_word(0x9d8, if (reserve.wrapping_sub(limit) as i16) >= 0 { limit } else { reserve });
                    r.set_word(0x9c6, maximum);
                }
                3
            }
            _ => return Err("invalid projectile pickup callback"),
        };
        crate::audio::queue_sfx(r, 1, sound << 8 | 1)?;
    }
    if lifetime == 0 || grapple || collision {
        r.set_word(0x1b47 + k, 0xeca3);
        r.set_word(0x1b8f + k, 1);
        r.set_word(0x1bd7 + k, 0x3000);
        r.set_word(0x1a03 + k, 0xefdf);
    }
    Ok(())
}

// The historical get_EnemyDef_A2 helper actually addresses bank A0.
pub(crate) fn random_drop(r: &mut Wram<'_>, rom: &[u8], k: u16) -> Result<u16, &'static str> {
    let k = slot(k)?;
    let header = r.word(0xf3c8 + k);
    if header == 0 {
        return Ok(6);
    }
    let address = 0x100000 + usize::from(header & 0x7fff) + 0x3a;
    let pointer = rom.get(address..address + 2).ok_or("missing enemy drop definition")?;
    let pointer = u16::from_le_bytes([pointer[0], pointer[1]]);
    if pointer == 0 {
        return Ok(6);
    }
    let address = 0x1a0000 + usize::from(pointer & 0x7fff);
    let weights = rom.get(address..address + 6).ok_or("missing enemy drop chances")?;
    let random = loop {
        let value = crate::enemy_projectiles::random(r) as u8;
        if value != 0 {
            break u16::from(value);
        }
    };
    let health = r.word(0x9c2).wrapping_add(r.word(0x9d6));
    if health < 30 {
        r.set_word(0xe1a, 1);
    } else if health >= 50 {
        r.set_word(0xe1a, 0);
    }
    let mut available = 255u8;
    let (total, mut mask) = if r.byte(0xe1a) != 0 {
        (weights[0].wrapping_add(weights[1]), 3u8)
    } else {
        let mut total = weights[3];
        let mut mask = 8u8;
        if r.word(0x9c2) != r.word(0x9c4) || r.word(0x9d6) != r.word(0x9d4) {
            total = total.wrapping_add(weights[0]).wrapping_add(weights[1]);
            mask |= 3;
        }
        if r.word(0x9c6) != r.word(0x9c8) {
            total = total.wrapping_add(weights[2]);
            mask |= 4;
        }
        if r.word(0x9ca) != r.word(0x9cc) {
            available = available.wrapping_sub(weights[4]);
            mask |= 16;
        }
        if r.word(0x9ce) != r.word(0x9d0) {
            available = available.wrapping_sub(weights[5]);
            mask |= 32;
        }
        (total, mask)
    };
    let drops = [1, 2, 4, 6, 5, 3];
    let mut cumulative = 0u16;
    for i in 0..4 {
        if total == 0 {
            // C discards the entire lower nibble when its denominator is zero.
            mask >>= 4;
            break;
        }
        let selected = mask & 1 != 0;
        mask >>= 1;
        if selected {
            let next = cumulative.wrapping_add(u16::from(available) * u16::from(weights[i]) / u16::from(total));
            if next >= random {
                return Ok(drops[i]);
            }
            cumulative = next;
        }
    }
    for i in 4..6 {
        let selected = mask & 1 != 0;
        mask >>= 1;
        if selected {
            let next = cumulative.wrapping_add(u16::from(weights[i]));
            if next >= random {
                return Ok(drops[i]);
            }
            cumulative = next;
        }
    }
    Ok(6)
}

pub fn dispatch_with_host(
    r: &mut Wram<'_>,
    rom: &[u8],
    host: &mut crate::enemy_projectiles::ProjectileHost,
    name: &str,
    a: i32,
    b: i32,
) -> Result<u32, &'static str> {
    if name == "EprojInstruction" {
        let pc = b as u16;
        let result = instruction_with_host(r, rom, word(rom, pc)?, pc.wrapping_add(2), a as u16, host)?;
        Ok(if result == 0 { 0 } else { 0x860000 | u32::from(result) })
    } else {
        dispatch_rom(r, rom, name, a, b)
    }
}
pub fn dispatch_rom(r: &mut Wram<'_>, rom: &[u8], name: &str, a: i32, b: i32) -> Result<u32, &'static str> {
    match name {
        "RandomDropRoutine" => Ok(u32::from(random_drop(r, rom, a as u16)?)),
        "EprojPowerBombCollision" => {
            let k = slot(a as u16)?;
            let radius = r.word(0xcea) >> 8;
            let ry = (radius + (radius & 1) + (r.word(0xcea) >> 9)) >> 1;
            let abs = |v: u16| if v & 0x8000 != 0 { v.wrapping_neg() } else { v };
            if radius != 0
                && abs(r.word(0xce2).wrapping_sub(r.word(0x1a4b + k))) < radius
                && abs(r.word(0xce4).wrapping_sub(r.word(0x1a93 + k))) < ry
            {
                r.set_word(0x1997 + k, 0);
                r.set_word(0xa66, 0);
            }
            Ok(0)
        }
        "Eproj_FuncE722" => {
            let k = slot(a as u16)?;
            Ok(u32::from(r.word(0x1a4b + k) >= 512 || r.word(0x1a93 + k) >= 512))
        }
        "Eproj_FuncE73E_MoveXY" => {
            let k = slot(a as u16)?;
            let angle = r.word(0x97dc + k);
            for (pos, sub, high, low, phase) in
                [(0x1a4b, 0x1a27, 0x1ab7, 0x1aff, 64), (0x1a93, 0x1a6f, 0x1adb, 0x1b23, 128)]
            {
                let amount = (u32::from(r.word(high + k)) << 16) | u32::from(r.word(low + k));
                let amount = if angle.wrapping_add(phase) & 128 != 0 { amount.wrapping_neg() } else { amount };
                let position = (u32::from(r.word(pos + k)) << 16) | u32::from(r.word(sub + k));
                let next = position.wrapping_add(amount);
                r.set_word(pos + k, (next >> 16) as u16);
                r.set_word(sub + k, next as u16);
            }
            Ok(0)
        }
        "Eproj_SetXvelRandom" | "Eproj_SetYvelRandom" => {
            let k = slot(a as u16)?;
            let x = name == "Eproj_SetXvelRandom";
            let timer = u16::from(crate::enemy_projectiles::random(r) as u8).max(if x { 32 } else { 128 });
            r.set_word(if x { 0x1ab7 } else { 0x1adb } + k, timer);
            Ok(0)
        }
        "Eproj_MotherBrainRoomTurretBullets_Func2" => {
            let k = slot(a as u16)?;
            let direction = r.byte(0x1a6f + k);
            let step = r.byte(0x1a70 + k);
            let next = direction.wrapping_add(step) & 7;
            let allowed = byte(rom, r.word(0x1a27 + k).wrapping_add(u16::from(next)))?;
            if allowed != 0 {
                r.set_byte(0x1a6f + k, next);
            } else {
                let step = step.wrapping_neg();
                r.set_byte(0x1a70 + k, step);
                r.set_byte(0x1a6f + k, direction.wrapping_add(step));
            }
            Ok(0)
        }
        "EprojInstr_SetPreInstrA" => {
            let result = instruction(r, rom, 0x8cf6, b as u16, a as u16)?;
            Ok(if result == 0 { 0 } else { 0x860000 | u32::from(result) })
        }
        "EprojInstr_SetPreInstrB" => {
            let result = instruction(r, rom, 0x8cfd, b as u16, a as u16)?;
            Ok(if result == 0 { 0 } else { 0x860000 | u32::from(result) })
        }
        "EprojInstr_DisableCollisionsWithSamus" => {
            let result = instruction(r, rom, 0x9475, b as u16, a as u16)?;
            Ok(if result == 0 { 0 } else { 0x860000 | u32::from(result) })
        }
        "EprojInstr_MoveY_Minus4" => {
            let result = instruction(r, rom, 0xaf92, b as u16, a as u16)?;
            Ok(if result == 0 { 0 } else { 0x860000 | u32::from(result) })
        }
        "EprojInstr_ResetXYpos2" => {
            let result = instruction(r, rom, 0xb436, b as u16, a as u16)?;
            Ok(if result == 0 { 0 } else { 0x860000 | u32::from(result) })
        }
        "EprojInstr_Add12ToY" => {
            let result = instruction(r, rom, 0xc8d0, b as u16, a as u16)?;
            Ok(if result == 0 { 0 } else { 0x860000 | u32::from(result) })
        }
        "EprojInstr_D1B6" => {
            let result = instruction(r, rom, 0xd1b6, b as u16, a as u16)?;
            Ok(if result == 0 { 0 } else { 0x860000 | u32::from(result) })
        }
        "EprojInstr_CalculateDirectionTowardsSamus" => {
            let result = instruction(r, rom, 0x82a5, b as u16, a as u16)?;
            Ok(if result == 0 { 0 } else { 0x860000 | u32::from(result) })
        }
        "EprojInstr_MotherBrainPurpleBreathIsActive" => {
            let result = instruction(r, rom, 0xcaee, b as u16, a as u16)?;
            Ok(if result == 0 { 0 } else { 0x860000 | u32::from(result) })
        }
        "EprojInstr_GotoIfFunc1" => {
            let result = instruction(r, rom, 0xb3b8, b as u16, a as u16)?;
            Ok(if result == 0 { 0 } else { 0x860000 | u32::from(result) })
        }
        "SpawnMotherBrainDeathBeam" => {
            let k = slot(a as u16)?;
            crate::enemy_projectiles::spawn_room(r, rom, 0xcb75, r.word(0x1aff + k))?;
            Ok(0)
        }
        "EprojInstr_CallFunc" => {
            let result = instruction(r, rom, 0x8171, b as u16, a as u16)?;
            Ok(if result == 0 { 0 } else { 0x860000 | u32::from(result) })
        }
        "EprojPreInstr_MotherBrainRainbowBeam" => {
            preinstruction(r, rom, 0xc814, a as u16)?;
            Ok(0)
        }
        "EprojPreInstr_C84D" => {
            preinstruction(r, rom, 0xc84d, a as u16)?;
            Ok(0)
        }
        "EprojPreInstr_TimeBombSetJapaneseText" => {
            preinstruction(r, rom, 0xcafa, a as u16)?;
            Ok(0)
        }
        "sub_86DCC3" => {
            preinstruction(r, rom, 0xdcc3, a as u16)?;
            Ok(0)
        }
        "sub_86CFD5" | "sub_86CFE6" | "sub_86CFF8" => {
            preinstruction(
                r,
                rom,
                match name {
                    "sub_86CFD5" => 0xcfd5,
                    "sub_86CFE6" => 0xcfe6,
                    _ => 0xcff8,
                },
                a as u16,
            )?;
            Ok(0)
        }
        "EprojPreInit_MotherBrainsDeathExplosion_0" | "EprojPreInstr_MotherBrainsRainbowBeamExplosion" => {
            preinstruction(
                r,
                rom,
                if name == "EprojPreInit_MotherBrainsDeathExplosion_0" { 0xc914 } else { 0xc94c },
                a as u16,
            )?;
            Ok(0)
        }
        "EprojInstruction" => {
            let pc = b as u16;
            let result = instruction(r, rom, word(rom, pc)?, pc.wrapping_add(2), a as u16)?;
            Ok(if result == 0 { 0 } else { 0x860000 | u32::from(result) })
        }
        "EprojInstr_Delete"
        | "EprojInstr_SetPreInstr_"
        | "EprojInstr_ClearPreInstr"
        | "EprojInstr_Goto"
        | "EprojInstr_ECE3"
        | "EprojInstr_ED17"
        | "EprojInstr_D1C7"
        | "EprojInstr_DC5A"
        | "EprojInstr_A3BE"
        | "EprojInstr_GotoWithProbability25"
        | "EprojInstr_QueueSfx2_9"
        | "EprojInstr_QueueSfx2_24"
        | "EprojInstr_QueueSfx2_B" => {
            let opcode = match name {
                "EprojInstr_Delete" => 0x8154,
                "EprojInstr_SetPreInstr_" => 0x8161,
                "EprojInstr_ClearPreInstr" => 0x816a,
                "EprojInstr_ECE3" => 0xece3,
                "EprojInstr_ED17" => 0xed17,
                "EprojInstr_D1C7" => 0xd1c7,
                "EprojInstr_DC5A" => 0xdc5a,
                "EprojInstr_A3BE" => 0xa3be,
                "EprojInstr_GotoWithProbability25" => 0xa456,
                "EprojInstr_QueueSfx2_9" => 0xee8b,
                "EprojInstr_QueueSfx2_24" => 0xee97,
                "EprojInstr_QueueSfx2_B" => 0xeea3,
                _ => 0x81ab,
            };
            let result = instruction(r, rom, opcode, b as u16, a as u16)?;
            Ok(if result == 0 { 0 } else { 0x860000 | u32::from(result) })
        }
        "EprojPreInstr_CeresElevatorPlatform" | "EprojPreInstr_CeresElevatorPad" => {
            let address = if name == "EprojPreInstr_CeresElevatorPad" { 0xa328 } else { 0xa364 };
            preinstruction(r, rom, address, a as u16)?;
            Ok(0)
        }
        "EprojPreinstruction" => {
            preinstruction(r, rom, b as u16, a as u16)?;
            Ok(0)
        }
        "EprojRunAll" => {
            run_all(r, rom, 4096)?;
            Ok(0)
        }
        "SpawnEprojWithRoomGfx" => {
            crate::enemy_projectiles::spawn_room(r, rom, a as u16, b as u16)?;
            Ok(0)
        }
        "MoveEprojWithVelocity" => Ok(u32::from(move_xy(r, a as u16)?)),
        "MoveEprojWithVelocityX" => Ok(u32::from(move_x(r, a as u16)?)),
        "MoveEprojWithVelocityY" => Ok(u32::from(move_y(r, a as u16)?)),
        _ => Err("unsupported enemy projectile instruction routine"),
    }
}
