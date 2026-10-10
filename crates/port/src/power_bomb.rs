//! Power bomb gameplay lifecycle; bank88 HDMA execution remains an explicit host dependency.
//! Translation of pinned snesrev/sm, under its MIT license.
use crate::{Wram, audio, projectile_special, projectiles};
const PROJECTILE_INDEX: usize = 0xdde;
const FUSE: usize = 0xc7c;
const INSTRUCTION: usize = 0xc40;
const EXPLOSION_X: usize = 0xce2;
const EXPLOSION_Y: usize = 0xce4;
const HDMA_ENABLE: usize = 0x18b0;
const POWER_BOMB_FLAG: usize = 0xcee;
const RADIUS: usize = 0xcea;
const ROOM_WIDTH: usize = 0x7a5;
const ROOM_HEIGHT: usize = 0x7a7;
const WIDTH_SCROLLS: usize = 0x7a9;
const HEIGHT_SCROLLS: usize = 0x7ab;
const PROJECTILE_X: usize = 0xb64;
const PROJECTILE_Y: usize = 0xb78;
const PROJECTILE_TYPE: usize = 0xc18;
const PROJECTILE_DIR: usize = 0xc04;
const BLOCK_INDEX: usize = 0xdc4;
const TIME_FROZEN: usize = 0xa78;
const EXPLOSION_STATUS: usize = 0x592;
type Result<T> = std::result::Result<T, &'static str>;
pub trait Host: projectile_special::Host {
    /// Allocate the original bank88 object (DMAP=0x40, BBAD=0x28/0x29).
    fn spawn_hdma(&mut self, ram: &mut Wram<'_>, script: u16) -> Result<u16>;
}
fn slot(k: u16) -> Result<usize> {
    if k > 18 || k & 1 != 0 { Err("projectile slot") } else { Ok(k as usize) }
}
pub fn spawn(r: &mut Wram<'_>, h: &mut impl Host) -> Result<()> {
    if r.word(TIME_FROZEN) != 0 {
        r.set_word(EXPLOSION_STATUS, 0x4000);
    } else {
        audio::dispatch(r, "QueueSfx1_Max15", 1, 0)?;
        r.set_word(EXPLOSION_STATUS, 0x8000);
        h.spawn_hdma(r, 0x8ace)?;
        h.spawn_hdma(r, 0x8b80)?;
    }
    Ok(())
}
pub fn fuse(r: &mut Wram<'_>, h: &mut impl Host) -> Result<()> {
    let k = r.word(PROJECTILE_INDEX);
    let p = slot(k)?;
    let timer = r.word(FUSE + p);
    if timer != 0 {
        let next = timer.wrapping_sub(1);
        r.set_word(FUSE + p, next);
        if next != 0 {
            if next == 15 {
                r.set_word(INSTRUCTION + p, r.word(INSTRUCTION + p).wrapping_add(28));
            }
        } else {
            r.set_word(EXPLOSION_X, r.word(PROJECTILE_X + p));
            r.set_word(EXPLOSION_Y, r.word(PROJECTILE_Y + p));
            r.set_word(HDMA_ENABLE, r.word(HDMA_ENABLE) | 0x8000);
            spawn(r, h)?;
            r.set_word(FUSE + p, 65535);
        }
    } else if r.word(POWER_BOMB_FLAG) == 0 {
        projectiles::dispatch_rom(r, &[], "ClearProjectile", k as i32, 0)?;
    }
    Ok(())
}
pub fn radius(r: &mut Wram<'_>, rom: &[u8], h: &mut impl Host, k: u16) -> Result<()> {
    let p = slot(k)?;
    let timer = r.word(FUSE + p);
    if timer != 0 {
        if (timer as i16) < 0 {
            r.set_word(FUSE + p, 0);
        }
        return Ok(());
    }
    let w = r.word(ROOM_WIDTH);
    let height = r.word(ROOM_HEIGHT);
    if w == 0 || w > 255 || height == 0 || height > 255 {
        return Err("room geometry");
    }
    let rx = r.word(RADIUS) >> 8;
    let ry = 3 * rx / 4;
    let x = r.word(EXPLOSION_X);
    let y = r.word(EXPLOSION_Y);
    let left = if (x.wrapping_sub(rx) as i16) < 0 { 0 } else { x.wrapping_sub(rx) >> 4 };
    let top = if (y.wrapping_sub(ry) as i16) < 0 { 0 } else { y.wrapping_sub(ry) >> 4 };
    let right = (x.wrapping_add(rx) >> 4).min(w - 1);
    let bottom = (y.wrapping_add(ry) >> 4).min(height - 1);
    if left > right || top > bottom || bottom * w + right >= 0x3200 {
        return Err("power bomb bounds");
    }
    // Original order repeats corners, including every point four times at radius zero.
    for (start, count, step) in [
        (top * w + left, right - left, 1),
        (top * w + left, bottom - top, w),
        (bottom * w + left, right - left, 1),
        (top * w + right, bottom - top, w),
    ] {
        for n in 0..=count {
            projectile_special::bomb_reaction(r, rom, h, 2 * (start + n * step))?;
        }
    }
    Ok(())
}
pub fn contact(r: &mut Wram<'_>, rom: &[u8], h: &mut impl Host, k: u16) -> Result<()> {
    let p = slot(k)?;
    if r.word(PROJECTILE_TYPE + p) & 0xf00 == 0x500 {
        return projectile_special::bomb_contact(r, rom, h, k);
    }
    let t = r.word(PROJECTILE_TYPE + p);
    if (t & 0xf00 != 0 && (t >> 8) & 15 > 8) || (t & 0xf00 == 0 && t & 15 > 11) {
        return Err("power bomb type");
    }
    let x = r.word(PROJECTILE_X + p);
    let y = r.word(PROJECTILE_Y + p);
    if (x as i16) < 0 || x >> 8 >= r.word(WIDTH_SCROLLS) || (y as i16) < 0 || y >> 8 >= r.word(HEIGHT_SCROLLS) {
        return Ok(());
    }
    r.set_word(
        BLOCK_INDEX,
        if x < 4096 && y < 4096 { (x >> 4) + (y >> 4) * (r.word(ROOM_WIDTH) as u8 as u16) } else { 65535 },
    );
    radius(r, rom, h, k)
}
pub fn dispatch_host(r: &mut Wram<'_>, rom: &[u8], h: &mut impl Host, s: &str, a: i32, _: i32) -> Result<u32> {
    match s {
        "PowerBomb_Func3" => fuse(r, h)?,
        "SpawnPowerBombExplosion" => spawn(r, h)?,
        "BlockFunc_A06A" => radius(r, rom, h, a as u16)?,
        "BombOrPowerBomb_Func1" => contact(r, rom, h, a as u16)?,
        "ProjPreInstr_PowerBomb" => {
            let p = slot(a as u16)?;
            if r.word(PROJECTILE_DIR + p) & 0xf0 != 0 {
                projectiles::dispatch_rom(r, rom, "ClearProjectile", a, 0)?;
            } else {
                fuse(r, h)?;
                contact(r, rom, h, a as u16)?;
            }
        }
        _ => return Err("unknown power bomb routine"),
    };
    Ok(0)
}
