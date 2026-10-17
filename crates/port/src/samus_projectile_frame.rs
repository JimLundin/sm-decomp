//! Complete scheduled Samus projectile update and render passes.
//! Native banks $90/$93/$81; pinned snesrev/sm MIT notice applies.
//! Projectile behavior is composed from the existing flight, bomb and special
//! attack modules. Native PLM/HDMA host operations remain explicit dependencies.
use crate::{WRAM_SIZE, Wram, power_bomb, projectile_flight, projectile_special, projectile_trails, projectiles};

type Result<T> = std::result::Result<T, &'static str>;
const INDEX: usize = 0xdde;
const INSTRUCTION: usize = 0xc40;
const PRE: usize = 0xc68;
const TYPE: usize = 0xc18;
const DIRECTION: usize = 0xc04;
const VARIABLE: usize = 0xc7c;
const SPRITEMAP: usize = 0xcb8;
const X: usize = 0xb64;
const Y: usize = 0xb78;
const CAMERA_X: usize = 0x911;
const CAMERA_Y: usize = 0x915;
const CERES: usize = 0x93f;
const FRAME: usize = 0x5b6;
const OAM: usize = 0x370;
const OAM_EXT: usize = 0x570;
const OAM_NEXT: usize = 0x590;
const ECHO_X: usize = 0xab0;
const ECHO_Y: usize = 0xab8;
const ECHO_SPEED: usize = 0xac0;
const SAMUS_TOP: usize = 0xac8;
const SAMUS_BOTTOM: usize = 0xaca;
const POSE: usize = 0xa1c;

/// Same host shape as the existing power-bomb verification adapter.
pub trait Host: projectile_special::Host {
    fn spawn_hdma(&mut self, ram: &mut Wram<'_>, script: u16) -> Result<u16>;
}
struct Bridge<'a, H>(&'a mut H);
impl<H: Host> projectile_flight::Host for Bridge<'_, H> {
    fn spawn_plm(&mut self, r: &mut Wram<'_>, header: u16) -> Result<u8> {
        self.0.spawn_plm(r, header)
    }
}
impl<H: Host> projectile_special::Host for Bridge<'_, H> {
    fn spawn_plm(&mut self, r: &mut Wram<'_>, header: u16) -> Result<u8> {
        self.0.spawn_plm(r, header)
    }
}
impl<H: Host> power_bomb::Host for Bridge<'_, H> {
    fn spawn_hdma(&mut self, r: &mut Wram<'_>, script: u16) -> Result<u16> {
        self.0.spawn_hdma(r, script)
    }
}
struct NoCallbacks;
impl projectile_special::Host for NoCallbacks {
    fn spawn_plm(&mut self, _: &mut Wram<'_>, _: u16) -> Result<u8> {
        Err("projectile frame needs a native PLM host")
    }
}
impl Host for NoCallbacks {
    fn spawn_hdma(&mut self, _: &mut Wram<'_>, _: u16) -> Result<u16> {
        Err("projectile frame needs a native HDMA host")
    }
}

fn slot(raw: u16) -> Result<usize> {
    if raw > 19 { Err("projectile frame slot outside ten-slot pool") } else { Ok(usize::from(raw & !1)) }
}
fn offset(bank: u8, pointer: u16) -> usize {
    ((usize::from(bank) & 0x7f) << 15) + usize::from(pointer & 0x7fff)
}
fn word(rom: &[u8], at: usize) -> Result<u16> {
    let bytes = rom.get(at..at + 2).ok_or("projectile frame ROM extent")?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}
fn copy(r: &Wram<'_>) -> Vec<u8> {
    (0..WRAM_SIZE).map(|i| r.byte(i)).collect()
}
fn publish(r: &mut Wram<'_>, bytes: Vec<u8>) {
    for (i, value) in bytes.into_iter().enumerate() {
        r.set_byte(i, value);
    }
}

fn pre_instruction(r: &mut Wram<'_>, rom: &[u8], h: &mut impl Host, raw: u16) -> Result<()> {
    let k = slot(raw)?;
    if raw & 1 != 0 {
        return Err("projectile scheduler needs even dependency slots");
    }
    let opcode = r.word(PRE + k);
    let flight = match opcode {
        0xaef3 => "ProjPreInstr_Beam_NoWaveBeam",
        0xaf4a => "ProjPreInstr_Dir0459",
        0xaf52 => "ProjPreInstr_Dir1368",
        0xaf60 => "ProjPreInstr_Dir27",
        0xaf68 => "ProjPreInstr_Missile",
        0xafc7 => "ProjPreInstr_Missile_Func0459",
        0xafcf => "ProjPreInstr_Missile_Func1368",
        0xafdd => "ProjPreInstr_Missile_Func27",
        0xafe5 => "ProjPreInstr_SuperMissile",
        0xb047 => "ProjPreInstr_SuperMissile_Func0459",
        0xb052 => "ProjPreInstr_SuperMissile_Func1368",
        0xb06a => "ProjPreInstr_SuperMissile_Func27",
        0xb0c3 => "ProjPreInstr_WavePlasmaEtc",
        0xb0e4 => "ProjPreInstr_BeamOrIceWave",
        0xb13b => "ProjPreInstr_WavePlasmaEtc_0459",
        0xb143 => "ProjPreInstr_WavePlasmaEtc_1368",
        0xb151 => "ProjPreInstr_WavePlasmaEtc_27",
        0xb159 => "ProjPreInstr_HyperBeam",
        _ => "",
    };
    if !flight.is_empty() {
        projectile_flight::dispatch_host(r, rom, &mut Bridge(h), flight, i32::from(raw), 0)?;
        return Ok(());
    }
    let special = match opcode {
        0xcf09 => "ProjPreInstr_IceSba",
        0xcf7a => "ProjPreInstr_IceSba2",
        0xd4d2 => "ProjPreInstr_SpeedEcho",
        0xd793 => "ProjPreInstr_PlasmaSba",
        0xd7e1 => "ProjPreInstr_PlasmaSbaFunc_0",
        0xd7fa => "ProjPreInstr_PlasmaSbaFunc_1",
        0xd813 => "ProjPreInstr_PlasmaSbaFunc_2",
        0xd8f7 => "ProjPreInstr_SpreadBomb",
        0xda08 => "ProjPreInstr_WaveSba",
        0xdb06 => "ProjPreInstr_SpazerSba",
        0xdc9c => "ProjPreInstr_EndOfSpazerSba",
        0xefd3 => "ProjPreInstr_UnknownProj8027",
        _ => "",
    };
    if !special.is_empty() {
        projectile_special::dispatch_host(r, rom, &mut Bridge(h), special, i32::from(raw), 0)?;
        return Ok(());
    }
    match opcode {
        0xb169 => Ok(()),
        0xb075 => projectiles::dispatch_rom(r, rom, "ProjPreInstr_Func1", i32::from(raw), 0).map(|_| ()),
        0xb099 => {
            if r.word(DIRECTION + k) & 0xf0 != 0 {
                projectiles::clear(r, raw)
            } else {
                projectiles::dispatch_rom(r, rom, "Bomb_Func2", 0, 0)?;
                // A normal bomb becomes type0x500 at explosion, but its fuse
                // stage is type0x300 and uses the same radius dependency as
                // native BlockFunc_A06A. Compose both through the full host.
                power_bomb::contact(r, rom, &mut Bridge(h), raw)
            }
        }
        0xb0ae => {
            power_bomb::dispatch_host(r, rom, &mut Bridge(h), "ProjPreInstr_PowerBomb", i32::from(raw), 0).map(|_| ())
        }
        0xb103 => {
            // Native Wave_Shared skips the timer/clear wrapper. Its directional
            // movement is the same dependency used after the hyper-beam guard.
            let direction = r.word(DIRECTION + k) & 15;
            if direction > 9 {
                return Err("projectile wave direction outside native table");
            }
            let dx = [0i16, 16, 16, 16, 0, 0, -16, -16, -16, 0][usize::from(direction)];
            let dy = [-16i16, -16, 0, 16, 16, 16, 16, 0, -16, -16][usize::from(direction)];
            r.set_word(0xbdc + k, r.word(0xbdc + k).wrapping_add(dx as u16));
            r.set_word(0xbf0 + k, r.word(0xbf0 + k).wrapping_add(dy as u16));
            let name = match direction {
                1 | 3 | 6 | 8 => "ProjPreInstr_WavePlasmaEtc_1368",
                2 | 7 => "ProjPreInstr_WavePlasmaEtc_27",
                _ => "ProjPreInstr_WavePlasmaEtc_0459",
            };
            projectile_flight::dispatch_host(r, rom, &mut Bridge(h), name, i32::from(direction * 2), 0)?;
            projectiles::dispatch_rom(r, rom, "DeleteProjectileIfFarOffScreen", 0, 0)?;
            Ok(())
        }
        _ => Err("unsupported projectile frame preinstruction"),
    }
}

fn update(r: &mut Wram<'_>, rom: &[u8], host: &mut impl Host) -> Result<()> {
    let mut next = 18_i32;
    r.set_word(INDEX, 18);
    // Reload the native global after every active slot: dependencies can delete
    // another slot or allocate a linked missile while this pass is running.
    for _ in 0..1024 {
        if next < 0 {
            return Ok(());
        }
        let raw = next as u16;
        let k = slot(raw)?;
        if r.word(INSTRUCTION + k) != 0 {
            pre_instruction(r, rom, host, raw)?;
            projectiles::run_instructions(r, rom)?;
            next = i32::from(r.word(INDEX));
            if next > 18 || next & 1 != 0 {
                return Err("projectile dependency changed scheduler to invalid slot");
            }
        }
        next -= 2;
        r.set_word(INDEX, next as u16);
    }
    Err("cyclic projectile frame scheduler")
}

/// Whole native ten-slot update, with transactional publication on success.
/// Host failures abort ownership; callers must never retry the C frame afterward.
pub fn handle(r: &mut Wram<'_>, rom: &[u8], host: &mut impl Host) -> Result<()> {
    let mut bytes = copy(r);
    update(&mut Wram::new(&mut bytes)?, rom, host)?;
    publish(r, bytes);
    Ok(())
}

fn mode7(r: &Wram<'_>, k: usize) -> (u16, u16) {
    fn product(a: u16, b: u16) -> u16 {
        ((i32::from(a as i16) * i32::from(b as i16)) >> 8) as u16
    }
    let dx = r.word(X + k).wrapping_sub(r.word(0x80));
    let dy = r.word(0x82).wrapping_sub(r.word(Y + k));
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

/// Native unsigned-Y bank81 emitter. Preserve X/extension/Y/tile store order,
/// unaligned or wrapped initial cursors and aliases into gameplay WRAM.
pub fn draw_spritemap(r: &mut Wram<'_>, rom: &[u8], bank: u8, pointer: u16, x: u16, y: u16) -> Result<()> {
    if pointer < 0x8000 {
        return Err("projectile spritemap below banked ROM");
    }
    let at = offset(bank, pointer);
    let count = usize::from(word(rom, at)?);
    let entries = rom.get(at + 2..at + 2 + count * 5).ok_or("projectile spritemap entries outside ROM")?;
    let mut cursor = usize::from(r.word(OAM_NEXT));
    for entry in entries.as_chunks::<5>().0 {
        let dx = u16::from_le_bytes([entry[0], entry[1]]);
        let screen_x = x.wrapping_add(dx);
        r.set_byte(OAM + cursor, screen_x as u8);
        let extension = ((screen_x & 0x100) >> 8) | (u16::from(dx & 0x8000 != 0) * 2);
        let address = OAM_EXT + (cursor >> 5) * 2;
        r.set_word(address, r.word(address) | (extension << (2 * ((cursor >> 2) & 7))));
        r.set_byte(OAM + cursor + 1, y.wrapping_add(u16::from(entry[2])) as u8);
        r.set_word(OAM + cursor + 2, u16::from_le_bytes([entry[3], entry[4]]));
        cursor = (cursor + 4) & 0x1ff;
    }
    r.set_word(OAM_NEXT, cursor as u16);
    Ok(())
}
fn projectile_map(r: &mut Wram<'_>, rom: &[u8], raw: u16, x: u16, y: u16) -> Result<()> {
    let pointer = r.word(SPRITEMAP + slot(raw)?);
    draw_spritemap(r, rom, 0x93, pointer, x, y)
}
fn samus_map(r: &mut Wram<'_>, rom: &[u8], a: u16, x: u16, y: u16) -> Result<()> {
    let pointer = word(rom, offset(0x92, 0x808d) + usize::from(a) * 2)?;
    if pointer != 0 {
        draw_spritemap(r, rom, 0x92, pointer, x, y)?;
    }
    Ok(())
}
fn echo(r: &mut Wram<'_>, rom: &[u8], raw: u16) -> Result<()> {
    if raw > 7 {
        return Err("projectile echo slot outside native four-entry pool");
    }
    let k = usize::from(raw & !1);
    let pose = r.word(POSE);
    if pose > 0xfc {
        return Err("projectile echo pose outside native table");
    }
    let height = *rom.get(offset(0x91, 0xb629) + usize::from(pose) * 8 + 4).ok_or("echo pose ROM extent")?;
    let y = r.word(ECHO_Y + k).wrapping_sub(u16::from(height)).wrapping_sub(r.word(CAMERA_Y));
    if y as i16 >= 0 && (y.wrapping_sub(248) as i16) < 0 {
        samus_map(r, rom, r.word(SAMUS_TOP), r.word(ECHO_X + k).wrapping_sub(r.word(CAMERA_X)), y)?;
        let bottom = r.word(SAMUS_BOTTOM);
        if bottom != 0 {
            samus_map(r, rom, bottom, r.word(ECHO_X + k).wrapping_sub(r.word(CAMERA_X)), y)?;
        }
    }
    Ok(())
}
fn crash_echoes(r: &mut Wram<'_>, rom: &[u8]) -> Result<()> {
    if r.word(FRAME) & 1 != 0 {
        if r.word(ECHO_SPEED + 6) != 0 {
            echo(r, rom, 6)?;
        }
        if r.word(ECHO_SPEED + 4) != 0 {
            echo(r, rom, 4)?;
        }
    }
    Ok(())
}
fn flicker_visible(r: &Wram<'_>, raw: u16, kind: u16) -> bool {
    if kind & 0xf10 != 0 {
        return kind & 0xf00 < 0x300;
    }
    if kind & 0xc != 0 {
        (raw & 2 != 0) == (r.word(FRAME) & 2 != 0)
    } else {
        (raw & 2 != 0) != (r.word(FRAME) & 1 != 0)
    }
}
fn player(r: &mut Wram<'_>, rom: &[u8], elevated: bool) -> Result<()> {
    let mut raw = 8;
    r.set_word(INDEX, raw);
    for _ in 0..1024 {
        let k = slot(raw)?;
        if r.word(INSTRUCTION + k) != 0 && (elevated || flicker_visible(r, raw, r.word(TYPE + k))) {
            let (x, y) = if elevated {
                (
                    r.word(X + k).wrapping_sub(r.word(CAMERA_X)),
                    r.word(Y + k).wrapping_sub(8).wrapping_sub(r.word(CAMERA_Y)),
                )
            } else if r.word(CERES) & 0x8000 != 0 {
                mode7(r, k)
            } else {
                (r.word(X + k).wrapping_sub(r.word(CAMERA_X)), r.word(Y + k).wrapping_sub(r.word(CAMERA_Y)))
            };
            if y & 0xff00 == 0 && r.word(SPRITEMAP + k) & 0x8000 != 0 {
                if !elevated && r.word(CERES) & 0x8000 != 0 {
                    // Native C assigns pt.y to the visibility temporary but
                    // passes the uninitialized r18 local to its OAM emitter.
                    return Err("Ceres player oracle leaves draw Y uninitialized");
                }
                projectile_map(r, rom, raw, x, y)?;
            }
            raw = r.word(INDEX);
        }
        raw = raw.wrapping_sub(2);
        r.set_word(INDEX, raw);
        if raw & 0x8000 != 0 {
            if !elevated {
                crash_echoes(r, rom)?;
            }
            return projectile_trails::handle(r, rom);
        }
    }
    Err("cyclic projectile render traversal")
}
fn explosions(r: &mut Wram<'_>, rom: &[u8]) -> Result<()> {
    let mut raw = 18;
    r.set_word(INDEX, raw);
    for _ in 0..1024 {
        let k = slot(raw)?;
        let kind = r.word(TYPE + k) & 0xf00;
        if r.word(INSTRUCTION + k) != 0 && kind >= 0x300 && (kind != 0x300 || r.word(VARIABLE + k) != 0) {
            let position = if kind == 0x300 || kind == 0x500 || r.word(CERES) & 0x8000 == 0 {
                let x = r.word(X + k).wrapping_sub(r.word(CAMERA_X));
                if (x.wrapping_sub(304) as i16) >= 0 || (x.wrapping_add(48) as i16) < 0 {
                    None
                } else {
                    let y = r.word(Y + k).wrapping_sub(r.word(CAMERA_Y));
                    Some((x, y))
                }
            } else {
                // The pinned C discards CalcExplosion_Mode7's return. An actual
                // whole-frame fixture also diverges after a prior initialized
                // slot, so compiler-dependent local reuse is not an oracle.
                return Err("Ceres explosion oracle discards Mode7 point");
            };
            if let Some((x, y)) = position {
                if y & 0xff00 == 0 {
                    projectile_map(r, rom, raw, x, y)?;
                }
                raw = r.word(INDEX);
            }
        }
        raw = raw.wrapping_sub(2);
        r.set_word(INDEX, raw);
        if raw & 0x8000 != 0 {
            return Ok(());
        }
    }
    Err("cyclic explosion render traversal")
}
fn render(r: &mut Wram<'_>, rom: &[u8], name: &str, a: i32, b: i32) -> Result<u32> {
    match name {
        "DrawPlayerExplosions2" => player(r, rom, false)?,
        "sub_9382FD" => player(r, rom, true)?,
        "DrawBombAndProjectileExplosions" => explosions(r, rom)?,
        "Samus_DrawShinesparkCrashEchoProjectiles" => crash_echoes(r, rom)?,
        "Samus_DrawEcho" => echo(r, rom, u16::try_from(a).map_err(|_| "echo argument exceeds native width")?)?,
        "HandleProjectileTrails" => projectile_trails::handle(r, rom)?,
        "PortDrawProjectileSpritemap" => projectile_map(
            r,
            rom,
            u16::try_from(a).map_err(|_| "projectile draw slot exceeds native width")?,
            b as u16,
            (b as u32 >> 16) as u16,
        )?,
        "CalcExplosion_Mode7" => {
            let raw = u16::try_from(a).map_err(|_| "Mode7 argument exceeds native width")?;
            let (x, y) = mode7(r, slot(raw)?);
            return Ok(u32::from(x) | u32::from(y) << 16);
        }
        _ => return Err("unknown Samus projectile frame routine"),
    }
    Ok(0)
}
/// Selection reads only a private copy. The default selector conservatively
/// retains C ownership for frames needing external PLM/HDMA callbacks.
pub fn supports_call(r: &Wram<'_>, rom: &[u8], name: &str, a: i32, b: i32) -> Result<bool> {
    let mut bytes = copy(r);
    let mut state = Wram::new(&mut bytes)?;
    Ok(if name == "HandleProjectile" {
        update(&mut state, rom, &mut NoCallbacks).is_ok()
    } else {
        render(&mut state, rom, name, a, b).is_ok()
    })
}
pub fn dispatch_rom(r: &mut Wram<'_>, rom: &[u8], name: &str, a: i32, b: i32) -> Result<u32> {
    if name == "HandleProjectile" {
        handle(r, rom, &mut NoCallbacks)?;
        return Ok(0);
    }
    let mut bytes = copy(r);
    let value = render(&mut Wram::new(&mut bytes)?, rom, name, a, b)?;
    publish(r, bytes);
    Ok(value)
}
pub fn dispatch_host(r: &mut Wram<'_>, rom: &[u8], h: &mut impl Host, name: &str, a: i32, b: i32) -> Result<u32> {
    if name == "HandleProjectile" {
        handle(r, rom, h)?;
        Ok(0)
    } else {
        dispatch_rom(r, rom, name, a, b)
    }
}
