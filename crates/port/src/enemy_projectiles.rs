//! Enemy projectile allocation and room-effect initialization, pinned bank86.
use crate::Wram;
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
    let free = (0..18).rev().map(|i| i * 2).find(|k| r.word(0x1997 + k) == 0);
    let Some(k) = free else {
        r.set_word(0x1993, parameter);
        return Ok(());
    };
    let init = word(rom, definition)?;
    validate(r, rom, init, parameter)?;
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
    initialize(r, rom, init, parameter, k)?;
    Ok(())
}
pub fn spawn_room(r: &mut Wram<'_>, rom: &[u8], definition: u16, parameter: u16) -> Result<(), &'static str> {
    spawn(r, rom, definition, parameter, 0)
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
    if !matches!(init, 0x9dec | 0x9e46 | 0x9e4b | 0x9eb2)
        && !matches!(
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
        )
        && (r.word(0x1c27) > 78 || r.word(0x1c27) & 1 != 0)
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
