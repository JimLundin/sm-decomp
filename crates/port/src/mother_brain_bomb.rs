//! Mother Brain bomb allocation, bomb overlap, and decelerating bounce lifecycle.
use crate::{Wram, enemy_projectile_instructions, enemy_projectiles};
use enemy_projectiles::ProjectileHost;

const ACTIVE_BOMBS: usize = 0x784a;
const Y_ACCELERATION: [u16; 10] = [7, 0x10, 0x20, 0x40, 0x70, 0xb0, 0xf0, 0x130, 0x170, 0];

fn slot(k: u16) -> Result<usize, &'static str> {
    if k > 34 || k & 1 != 0 { Err("invalid Mother Brain bomb slot") } else { Ok(usize::from(k)) }
}

pub fn initialize(r: &mut Wram<'_>, k: u16, parameter: u16) -> Result<(), &'static str> {
    let k = slot(k)?;
    // The low byte carries the later fireball-explosion argument; the high byte
    // shares storage with horizontal subpixel movement and must survive here.
    r.set_byte(0x1a27 + k, parameter as u8);
    r.set_word(0x1adb + k, 256);
    r.set_word(0x1ab7 + k, 224);
    r.set_word(0x1a4b + k, r.word(0xfba).wrapping_add(12));
    r.set_word(0x1a93 + k, r.word(0xfbe).wrapping_add(16));
    r.set_word(0x19bb + k, 1024);
    r.set_word(0x1aff + k, 112);
    r.set_word(0x1b23 + k, 0);
    r.set_word(ACTIVE_BOMBS, r.word(ACTIVE_BOMBS).wrapping_add(1));
    Ok(())
}

fn overlaps_bomb(r: &Wram<'_>, k: usize) -> bool {
    if r.word(0xcd2) == 0 {
        return false;
    }
    let extent = r.word(0x1bb3 + k);
    (10..20).step_by(2).any(|bomb| {
        if r.word(0xc18 + bomb) & 0xf00 != 0x500 || r.word(0xc7c + bomb) != 0 {
            return false;
        }
        [(0xb64, 0xbb4, 0x1a4b, extent & 255), (0xb78, 0xbc8, 0x1a93, extent >> 8)].into_iter().all(
            |(position, radius, target, width)| {
                let delta = r.word(position + bomb).wrapping_sub(r.word(target + k));
                let distance = if delta & 0x8000 != 0 { delta.wrapping_neg() } else { delta };
                let radius = r.word(radius + bomb);
                distance < radius || distance.wrapping_sub(radius) < width
            },
        )
    })
}

pub fn collide_bombs(r: &mut Wram<'_>, rom: &[u8], k: u16, host: &mut ProjectileHost) -> Result<u8, &'static str> {
    let index = slot(k)?;
    if !overlaps_bomb(r, index) {
        return Ok(0);
    }
    r.set_word(ACTIVE_BOMBS, r.word(ACTIVE_BOMBS).wrapping_sub(1));
    r.set_word(0x1ab7 + index, 0);
    r.set_word(0x1adb + index, 0);
    r.set_word(0x1997 + index, 0);
    publish_position(r, index, host);
    enemy_projectiles::spawn_room_with_host(r, rom, 0xe509, 9, host)?;
    // Allocation can reuse this slot. C reads its position again afterward.
    publish_position(r, index, host);
    enemy_projectiles::spawn_enemy_drops(r, rom, host, 0xec3f, k, 0)?;
    Ok(1)
}

fn publish_position(r: &Wram<'_>, k: usize, host: &mut ProjectileHost) {
    host.x = r.word(0x1a4b + k);
    host.y = r.word(0x1a93 + k);
}

pub fn move_bomb(r: &mut Wram<'_>, k: u16, acceleration: u16) -> Result<u8, &'static str> {
    let index = slot(k)?;
    r.set_word(0x1adb + index, r.word(0x1adb + index).wrapping_add(acceleration));
    enemy_projectile_instructions::move_xy(r, k)?;
    if r.word(0x1a4b + index).wrapping_sub(240) as i16 >= 0 {
        r.set_word(0x1ab7 + index, r.word(0x1ab7 + index).wrapping_neg());
    }
    if (r.word(0x1a93 + index).wrapping_sub(208) as i16) < 0 {
        return Ok(0);
    }
    r.set_word(0x1a93 + index, 208);
    let speed = r.word(0x1aff + index);
    r.set_word(0x1ab7 + index, if r.word(0x1ab7 + index) & 0x8000 != 0 { speed.wrapping_neg() } else { speed });
    r.set_word(0x1adb + index, (-512i16) as u16);
    Ok(1)
}

pub fn update(r: &mut Wram<'_>, rom: &[u8], k: u16, host: &mut ProjectileHost) -> Result<(), &'static str> {
    let index = slot(k)?;
    if collide_bombs(r, rom, k, host)? != 0 {
        return Ok(());
    }
    let stage = r.word(0x1b23 + index);
    let acceleration = *Y_ACCELERATION.get(usize::from(stage >> 1)).ok_or("Mother Brain bomb acceleration extent")?;
    if stage != 0 && acceleration == 0 {
        r.set_word(0x1ab7 + index, 0);
        r.set_word(0x1adb + index, 0);
        r.set_word(ACTIVE_BOMBS, r.word(ACTIVE_BOMBS).wrapping_sub(1));
        r.set_word(0x1997 + index, 0);
        publish_position(r, index, host);
        enemy_projectiles::spawn_room_with_host(r, rom, 0x9650, u16::from(r.byte(0x1a27 + index)), host)?;
        publish_position(r, index, host);
        enemy_projectiles::spawn_room_with_host(r, rom, 0xe509, 3, host)?;
        crate::audio::queue_sfx(r, 2, 0x1306)?;
        return Ok(());
    }
    if stage == 0 {
        let velocity = r.word(0x1ab7 + index);
        let magnitude = if velocity & 0x8000 != 0 { velocity.wrapping_neg() } else { velocity };
        let damped = (magnitude.wrapping_sub(2) as i16).max(0) as u16;
        r.set_word(0x1ab7 + index, if velocity & 0x8000 != 0 { damped.wrapping_neg() } else { damped });
    }
    if move_bomb(r, k, acceleration)? != 0 {
        r.set_word(0x1b23 + index, stage.wrapping_add(2));
    }
    Ok(())
}
