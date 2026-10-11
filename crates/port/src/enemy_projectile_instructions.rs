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
    let k = slot(k)?;
    match opcode {
        0xa456 => {
            if crate::enemy_projectiles::random(r) & 0xc000 == 0xc000 {
                word(rom, pc)
            } else {
                Ok(pc.wrapping_add(2))
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
/// Dispatch supported movement and room-effect preinstructions. Collision remains explicit.
pub fn preinstruction(r: &mut Wram<'_>, rom: &[u8], address: u16, k: u16) -> Result<(), &'static str> {
    let k = slot(k)?;
    match address {
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
        0x8170 => {}
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
            let x = r.word(0x1a4b + k);
            let y = r.word(0x1a93 + k);
            let cx = r.word(0x911);
            let cy = r.word(0x915);
            if (x.wrapping_sub(cx) as i16) < 0
                || x.wrapping_sub(cx.wrapping_add(256)) as i16 >= 0
                || (y.wrapping_sub(cy) as i16) < 0
                || y.wrapping_sub(cy.wrapping_add(256)) as i16 >= 0
            {
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
pub fn run_all(r: &mut Wram<'_>, rom: &[u8], mut budget: usize) -> Result<(), &'static str> {
    if r.word(0x198d) & 0x8000 == 0 {
        return Ok(());
    }
    let mut index = 34i32;
    while index >= 0 {
        let k = slot(index as u16)?;
        r.set_word(0x1991, index as u16);
        if r.word(0x1997 + k) != 0 {
            preinstruction(r, rom, r.word(0x1a03 + k), k as u16)?;
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
                    pc = instruction(r, rom, opcode, pc.wrapping_add(2), active as u16)?;
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
pub fn dispatch_rom(r: &mut Wram<'_>, rom: &[u8], name: &str, a: i32, b: i32) -> Result<u32, &'static str> {
    match name {
        "EprojInstruction" => {
            let pc = b as u16;
            let result = instruction(r, rom, word(rom, pc)?, pc.wrapping_add(2), a as u16)?;
            Ok(if result == 0 { 0 } else { 0x860000 | u32::from(result) })
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
