//! Room/door ROM data and camera scrolling, pinned MIT snesrev/sm banks80/82/8F.
//! Hardware transition scheduling and tilemap upload remain explicit dependencies.
use crate::{Wram, audio, system};
const ROOM: usize = 0x79b;
const AREA: usize = 0x79f;
const ROOM_STATE: usize = 0x7bb;
const DOOR: usize = 0x78d;
const LX: usize = 0x911;
const LY: usize = 0x915;
const L2X: usize = 0x917;
const L2Y: usize = 0x919;
const SCROLLS: usize = 0xcd20;
fn data(rom: &[u8], bank: u8, p: u16, len: usize) -> Result<&[u8], &'static str> {
    if p < 0x8000 || usize::from(p) + len > 0x10000 {
        return Err("room ROM pointer crosses bank boundary");
    }
    let o = usize::from(bank & 127) * 32768 + usize::from(p & 32767);
    rom.get(o..o + len).ok_or("missing room ROM data")
}
fn word(rom: &[u8], bank: u8, p: u16) -> Result<u16, &'static str> {
    let b = data(rom, bank, p, 2)?;
    Ok(u16::from_le_bytes([b[0], b[1]]))
}
fn read_word(b: &[u8], p: usize) -> u16 {
    u16::from_le_bytes([b[p], b[p + 1]])
}
fn copy(r: &mut Wram<'_>, dest: usize, bytes: &[u8]) {
    for (i, b) in bytes.iter().enumerate() {
        r.set_byte(dest + i, *b);
    }
}
fn select(r: &mut Wram<'_>, rom: &[u8], symbol: &str, p: u16) -> Result<u16, &'static str> {
    if symbol == "RoomDefStateSelect_Finish" {
        data(rom, 0x8f, p, 26)?;
        r.set_word(ROOM_STATE, p);
        return Ok(0);
    }
    let len =
        if symbol == "RoomDefStateSelect_IsEventSet" || symbol == "RoomDefStateSelect_IsBossDead" { 3 } else { 2 };
    let b = data(rom, 0x8f, p, len)?;
    let pass = match symbol {
        "RoomDefStateSelect_TourianBoss01" => system::dispatch(r, "CheckBossBitForCurArea", 1, 0)? != 0,
        "RoomDefStateSelect_IsEventSet" => system::dispatch(r, "CheckEventHappened", u16::from(b[0]), 0)? != 0,
        "RoomDefStateSelect_IsBossDead" => system::dispatch(r, "CheckBossBitForCurArea", u16::from(b[0]), 0)? != 0,
        "RoomDefStateSelect_MorphBallMissiles" => r.word(0x9a4) & 4 != 0 && r.word(0x9c8) != 0,
        "RoomDefStateSelect_PowerBombs" => r.word(0x9d0) != 0,
        _ => return Err("unsupported room state selector"),
    };
    if pass {
        let dest = read_word(b, if len == 3 { 1 } else { 0 });
        select(r, rom, "RoomDefStateSelect_Finish", dest)
    } else {
        Ok(p + len as u16)
    }
}
fn select_room(r: &mut Wram<'_>, rom: &[u8], room: u16) -> Result<(), &'static str> {
    data(rom, 0x8f, room, 13)?;
    let mut p = room + 11;
    let mut seen = std::collections::BTreeSet::new();
    loop {
        if !seen.insert(p) {
            return Err("cyclic room state selector");
        }
        let code = word(rom, 0x8f, p)?;
        let symbol = match code {
            0xe5e6 => "RoomDefStateSelect_Finish",
            0xe5ff => "RoomDefStateSelect_TourianBoss01",
            0xe612 => "RoomDefStateSelect_IsEventSet",
            0xe629 => "RoomDefStateSelect_IsBossDead",
            0xe652 => "RoomDefStateSelect_MorphBallMissiles",
            0xe669 => "RoomDefStateSelect_PowerBombs",
            _ => return Err("unsupported room state bytecode"),
        };
        p = select(r, rom, symbol, p.checked_add(2).ok_or("room selector wraps bank")?)?;
        if p == 0 {
            return Ok(());
        }
    }
}
fn header(r: &mut Wram<'_>, rom: &[u8]) -> Result<(), &'static str> {
    let pointer = r.word(ROOM);
    let b = data(rom, 0x8f, pointer, 11)?;
    for (dest, index) in [(0x79d, 0), (AREA, 1), (0x7a1, 2), (0x7a3, 3), (0x7a9, 4), (0x7ab, 5), (0x7ad, 6), (0x7af, 7)]
    {
        r.set_word(dest, u16::from(b[index]));
    }
    r.set_word(0x7a5, u16::from(b[4]) * 16);
    r.set_word(0x7a7, u16::from(b[5]) * 16);
    r.set_word(0x7b5, read_word(b, 9));
    select_room(r, rom, pointer)?;
    r.set_word(0x7b9, 2 * u16::from(r.word(0x7a5) as u8) * u16::from(r.word(0x7a7) as u8));
    Ok(())
}
fn state_header(r: &mut Wram<'_>, rom: &[u8]) -> Result<(), &'static str> {
    let b = data(rom, 0x8f, r.word(ROOM_STATE), 26)?;
    if b[3] >= 29 {
        return Err("graphics set outside29 tileset pointers");
    }
    let tileset = word(rom, 0x8f, 0xe7a7 + u16::from(b[3]) * 2)?;
    let ts = data(rom, 0x8f, tileset, 9)?;
    copy(r, 0x7c0, &ts[0..3]);
    copy(r, 0x7c3, &ts[3..6]);
    copy(r, 0x7c6, &ts[6..9]);
    copy(r, 0x7bd, &b[..3]);
    for (dest, index) in [(0x7cb, 4), (0x7c9, 5)] {
        r.set_word(dest, u16::from(b[index]));
    }
    for (dest, index) in [(0x7cd, 6), (0x7cf, 8), (0x7d1, 10), (0x91b, 12), (0x7df, 18)] {
        r.set_word(dest, read_word(b, index));
    }
    Ok(())
}
fn layer2(r: &mut Wram<'_>, x: bool) -> u32 {
    let (pos, target, ratio) = if x { (LX, L2X, 0x91b) } else { (LY, L2Y, 0x91c) };
    let factor = r.byte(ratio);
    if factor == 1 {
        return 1;
    }
    let p = r.word(pos);
    let value = if factor == 0 {
        p
    } else {
        let t = u16::from(factor & 254);
        t * u16::from((p >> 8) as u8) + ((t * u16::from(p as u8)) >> 8)
    };
    r.set_word(target, value);
    0
}
fn bg_scrolls(r: &mut Wram<'_>) {
    for (dst, offset, pos) in [(0xb1, 0x91d, LX), (0xb3, 0x91f, LY), (0xb5, 0x921, L2X), (0xb7, 0x923, L2Y)] {
        r.set_word(dst, r.word(offset).wrapping_add(r.word(pos)));
    }
}
fn blocks(r: &mut Wram<'_>) {
    for (dst, reg) in [(0x907, 0xb1), (0x909, 0xb3), (0x90b, 0xb5), (0x90d, 0xb7)] {
        r.set_word(dst, r.word(reg) >> 4);
    }
    for (dst, pos) in [(0x8f7, LX), (0x8f9, LY), (0x8fb, L2X), (0x8fd, L2Y)] {
        r.set_word(dst, ((r.word(pos) as i16) >> 4) as u16);
    }
}
fn scroll(r: &Wram<'_>, index: u16) -> Result<u8, &'static str> {
    if index >= 256 {
        return Err("camera scroll lookup outside256-byte allocation");
    }
    Ok(r.byte(SCROLLS + usize::from(index)))
}
fn product(a: u16, b: u16) -> u16 {
    u16::from(a as u8) * u16::from(b as u8)
}
fn neg(v: u16) -> bool {
    v & 0x8000 != 0
}
fn swap(v: u16) -> u16 {
    v.rotate_left(8)
}
fn horizontal(r: &mut Wram<'_>, right: bool) -> Result<(), &'static str> {
    let old = r.word(LX);
    let ideal = r.word(0xb0a);
    let mut pos = old;
    if neg(if right { ideal.wrapping_sub(pos) } else { pos.wrapping_sub(ideal) }) {
        pos = ideal;
        r.set_word(0x90f, 0);
    }
    let width = r.word(0x7a9);
    let base = product(r.word(LY).wrapping_add(128) >> 8, width);
    if right {
        let max = swap(width.wrapping_sub(1));
        if max < pos {
            pos = max;
        } else if scroll(r, base.wrapping_add(1).wrapping_add(pos >> 8))? == 0 {
            let edge = pos & 0xff00;
            let moved = old.wrapping_sub(r.word(0xda2)).wrapping_sub(2);
            pos = if neg(moved.wrapping_sub(edge)) { edge } else { moved };
        }
    } else if neg(pos) {
        pos = 0;
    } else if scroll(r, base.wrapping_add(pos >> 8))? == 0 {
        let edge = (pos & 0xff00).wrapping_add(256);
        let moved = old.wrapping_add(r.word(0xda2)).wrapping_add(2);
        pos = if moved >= edge { edge } else { moved };
    }
    r.set_word(LX, pos);
    Ok(())
}
fn vertical(r: &mut Wram<'_>, down: bool) -> Result<(), &'static str> {
    let old = r.word(LY);
    let ideal = r.word(0xb0e);
    let mut pos = old;
    let width = r.word(0x7a9);
    let row = product(pos >> 8, width).wrapping_add(r.word(LX).wrapping_add(128) >> 8);
    let margin = if down && scroll(r, row)? != 1 { 31 } else { 0 };
    if neg(if down { ideal.wrapping_sub(pos) } else { pos.wrapping_sub(ideal) }) {
        pos = ideal;
        r.set_word(LY - 2, 0);
    }
    if down {
        let max = margin + swap(r.word(0x7ab).wrapping_sub(1));
        let mut bound = max;
        let clamp = if max < pos {
            true
        } else if scroll(r, width.wrapping_add(row))? == 0 {
            bound = margin + (pos & 0xff00);
            bound < pos
        } else {
            false
        };
        if clamp {
            let moved = old.wrapping_sub(r.word(0xda6)).wrapping_sub(2);
            pos = if neg(moved.wrapping_sub(bound)) { bound } else { moved };
        }
    } else if neg(pos) {
        pos = 0;
    } else {
        let row = product(pos >> 8, width).wrapping_add(r.word(LX).wrapping_add(128) >> 8);
        if scroll(r, row)? == 0 {
            let edge = (pos & 0xff00).wrapping_add(256);
            let moved = old.wrapping_add(r.word(0xda6)).wrapping_add(2);
            pos = if moved >= edge { edge } else { moved };
        }
    }
    r.set_word(LY, pos);
    Ok(())
}
fn automatic_x(r: &mut Wram<'_>) -> Result<(), &'static str> {
    if r.word(0xa78) != 0 {
        return Ok(());
    }
    let old = r.word(LX);
    let width = r.word(0x7a9);
    let mut pos = if neg(old) { 0 } else { old };
    let max = swap(width.wrapping_sub(1));
    if max < pos {
        pos = max;
    }
    let base = product(r.word(LY).wrapping_add(128) >> 8, width);
    let index = base.wrapping_add(pos >> 8);
    if scroll(r, index)? != 0 {
        if scroll(r, index.wrapping_add(1))? != 0 {
            r.set_word(LX, pos);
            return Ok(());
        }
        let boundary = pos & 0xff00;
        let moved = old.wrapping_sub(r.word(0xda2)).wrapping_sub(2);
        pos = if neg(moved.wrapping_sub(boundary)) {
            boundary
        } else if scroll(r, base.wrapping_add(moved >> 8))? != 0 {
            moved
        } else {
            (moved & 0xff00).wrapping_add(256)
        };
    } else {
        let boundary = (pos & 0xff00).wrapping_add(256);
        let moved = old.wrapping_add(r.word(0xda2)).wrapping_add(2);
        pos = if moved >= boundary {
            boundary
        } else if scroll(r, base.wrapping_add(u16::from(((moved >> 8) as u8).wrapping_add(1))))? != 0 {
            moved
        } else {
            moved & 0xff00
        };
    }
    r.set_word(LX, pos);
    Ok(())
}
fn automatic_y(r: &mut Wram<'_>) -> Result<(), &'static str> {
    if r.word(0xa78) != 0 {
        return Ok(());
    }
    let old = r.word(LY);
    let width = r.word(0x7a9);
    let x = r.word(LX).wrapping_add(128) >> 8;
    let initial = product(old >> 8, width).wrapping_add(x);
    let margin = if scroll(r, initial)? != 1 { 31 } else { 0 };
    let mut pos = if neg(old) { 0 } else { old };
    let max = margin + swap(r.word(0x7ab).wrapping_sub(1));
    if max < pos {
        pos = max;
    }
    let index = product(pos >> 8, width).wrapping_add(x);
    if scroll(r, index)? == 0 {
        let boundary = (pos & 0xff00).wrapping_add(256);
        let moved = old.wrapping_add(r.word(0xda6)).wrapping_add(2);
        pos = if moved >= boundary {
            boundary
        } else if scroll(r, product((moved >> 8).wrapping_add(1), width).wrapping_add(x))? != 0 {
            moved
        } else {
            moved & 0xff00
        };
    } else if scroll(r, width.wrapping_add(index))? == 0 {
        let boundary = margin + (pos & 0xff00);
        if boundary < pos {
            let moved = old.wrapping_sub(r.word(0xda6)).wrapping_sub(2);
            pos = if neg(moved.wrapping_sub(boundary)) {
                boundary
            } else if scroll(r, product(moved >> 8, width).wrapping_add(x))? != 0 {
                moved
            } else {
                (moved & 0xff00).wrapping_add(256)
            };
        }
    }
    r.set_word(LY, pos);
    Ok(())
}

/// Native constant door scroll writes, including unused recovered callbacks.
fn door_scroll_writes(symbol: &str) -> Option<&'static [(usize, u8)]> {
    Some(match symbol {
        "DoorCode_Scroll6_Green" => &[(6, 2)],
        "DoorCode_Scroll0_Blue" => &[(0, 1)],
        "DoorCode_Scroll13_Blue" => &[(19, 1)],
        "DoorCode_Scroll_4Blue_8Green" => &[(4, 0), (8, 2)],
        "DoorCode_Scroll_8toB_Red" => &[(8, 0), (9, 0), (10, 0), (11, 0)],
        "DoorCode_Scroll_LotsRed" => &[(2, 0), (3, 0), (4, 0), (5, 0), (11, 0), (12, 0), (13, 0), (17, 0)],
        "DoorCode_Scroll_1_4_Green" => &[(1, 2), (4, 2)],
        "DoorCode_Scroll_2_Blue" => &[(2, 1)],
        "DoorCode_Scroll_17_Blue" => &[(23, 1)],
        "DoorCode_Scroll_4_Blue" => &[(4, 1)],
        "DoorCode_Scroll_6_Green" => &[(6, 2)],
        "DoorCode_Scroll_3_Green" => &[(3, 2)],
        "DoorCode_SetScroll_0" => &[(24, 2), (28, 2)],
        "DoorCode_SetScroll_1" => &[(5, 1), (6, 1)],
        "DoorCode_SetScroll_2" => &[(29, 1)],
        "DoorCode_SetScroll_3" => &[(2, 2), (3, 2)],
        "DoorCode_SetScroll_4" => &[(0, 0), (1, 2)],
        "DoorCode_SetScroll_5" => &[(11, 2)],
        "DoorCode_SetScroll_6" => &[(28, 0), (29, 1)],
        "DoorCode_SetScroll_7" => &[(4, 0)],
        "DoorCode_SetScroll_8" => &[(32, 2), (36, 2), (37, 2)],
        "DoorCode_SetScroll_9" => &[(2, 1)],
        "DoorCode_SetScroll_10" => &[(0, 2)],
        "DoorCode_SetScroll_11" => &[(6, 2), (7, 2)],
        "DoorCode_SetScroll_12" => &[(1, 1), (2, 0)],
        "DoorCode_SetScroll_13" => &[(1, 1), (3, 0)],
        "DoorCode_SetScroll_14" => &[(0, 0), (4, 1)],
        "DoorCode_SetScroll_15" => &[(2, 1), (3, 1)],
        "DoorCode_SetScroll_16" => &[(0, 2), (1, 2)],
        "DoorCode_SetScroll_17" => &[(1, 2)],
        "DoorCode_SetScroll_18" => &[(15, 2), (18, 2)],
        "DoorCode_SetScroll_19" => &[(6, 2)],
        "DoorCode_SetScroll_20" => &[(0, 2), (1, 1)],
        "DoorCode_SetScroll_21" => &[(2, 2)],
        "DoorCode_SetScroll_22" => &[(3, 0), (4, 0), (6, 1), (7, 1), (8, 1)],
        "DoorCode_SetScroll_23" => &[(1, 1), (2, 1), (3, 1), (4, 2), (6, 0)],
        "DoorCode_SetScroll_24" => &[(0, 1), (1, 1)],
        "DoorCode_SetScroll_25" => &[(1, 0), (0, 1)],
        "DoorCode_SetScroll_26" => &[(10, 2)],
        "DoorCode_SetScroll_27" => &[(0, 1), (2, 0)],
        "DoorCode_SetScroll_28" => &[(0, 2), (2, 2)],
        "DoorCode_SetScroll_29" => &[(6, 1), (7, 1), (8, 0)],
        "DoorCode_SetScroll_30" => &[(2, 0), (3, 1)],
        "DoorCode_SetScroll_31" => &[(7, 2)],
        "DoorCode_SetScroll_32" => &[(1, 0), (2, 1)],
        "DoorCode_SetScroll_33" => &[(3, 0), (0, 1)],
        "DoorCode_SetScroll_34" => &[(1, 1), (4, 0)],
        "DoorCode_SetScroll_35" => &[(0, 1), (1, 0), (2, 0), (3, 0)],
        "DoorCode_SetScroll_36" => &[(0, 2)],
        "DoorCode_SetScroll_37" => &[(0, 1), (1, 1), (4, 0)],
        "DoorCode_SetScroll_38" => &[(0, 1), (3, 0)],
        "DoorCode_SetScroll_39" => &[(0, 1)],
        "DoorCode_SetScroll_40" => &[(0, 1), (1, 0)],
        "DoorCode_SetScroll_41" => &[(24, 1)],
        "DoorCode_SetScroll_42" => &[(3, 0), (2, 1)],
        "DoorCode_SetScroll_43" => &[(14, 0)],
        "DoorCode_SetScroll_44" => &[(1, 1)],
        "DoorCode_SetScroll_45" => &[(0, 2)],
        "DoorCode_SetScroll_46" => &[(3, 0), (4, 1)],
        "DoorCode_SetScroll_47" => &[(41, 1)],
        "DoorCode_SetScroll_48" => &[(40, 2), (46, 2)],
        "DoorCode_SetScroll_49" => &[(6, 0), (7, 0), (8, 0), (9, 0), (10, 0), (11, 0)],
        "DoorCode_SetScroll_50" => &[(10, 0), (11, 1)],
        "UNUSED_DoorCode_SetScroll_51" => &[(17, 0), (16, 1)],
        "UNUSED_DoorCode_SetScroll_52" => &[(4, 2)],
        "DoorCode_SetScroll_53" => &[(0, 0), (4, 1)],
        "DoorCode_SetScroll_54" => &[(0, 0), (1, 1)],
        "DoorCode_SetScroll_55" => &[(9, 0), (10, 1)],
        "DoorCode_SetScroll_56" => &[(0, 0), (2, 0), (1, 1)],
        "DoorCode_SetScroll_57" => &[(1, 1)],
        "DoorCode_SetScroll_58" => &[(6, 1)],
        "DoorCode_SetScroll_59" => &[(4, 0)],
        "DoorCode_SetScroll_60" => &[(1, 0)],
        "DoorCode_SetScroll_61" => &[(4, 0), (7, 0)],
        "DoorCode_SetScroll_62" => &[(2, 0), (1, 1)],
        "DoorCode_SetScroll_63" => &[(0, 2), (2, 2)],
        "DoorCode_SetScroll_64" => &[(0, 2), (1, 2)],
        "DoorCode_SetScroll_65" => &[(24, 1), (25, 0)],
        _ => return None,
    })
}

pub fn random_explosion(r: &mut Wram<'_>, rom: &[u8], x: u16, y: u16) -> Result<(), &'static str> {
    let selector = usize::from(crate::enemy_projectiles::random(r) & 15);
    let sounds = [0x24, 0, 0, 0x25, 0, 0, 0, 0];
    if selector < 8 && sounds[selector] != 0 {
        crate::audio::queue_sfx(r, 1, (sounds[selector] << 8) | 6)?;
    }
    let sprites = [3, 3, 9, 12, 12, 18, 18, 21];
    crate::sprite_objects::create(r, rom, x, y, sprites[selector & 7], 0)?;
    Ok(())
}

pub fn dispatch_rom(r: &mut Wram<'_>, rom: &[u8], symbol: &str, a: i32, b: i32) -> Result<u32, &'static str> {
    if let Some(writes) = door_scroll_writes(symbol) {
        for &(index, value) in writes {
            r.set_byte(SCROLLS + index, value);
        }
        return Ok(0);
    }
    let a = u16::try_from(a).map_err(|_| "invalid room argument")?;
    match symbol {
        "RoomCode_8FC8C8" => crate::enemy_projectiles::spawn_room(r, rom, 0xa3b0, 0)?,
        "RoomCode_ScrollRightDachora" => {
            let x = r.word(LX);
            if r.byte(SCROLLS + 11) == 2 && r.word(LY) < 0x500 && x < 0x380 {
                r.set_word(LX, x.wrapping_add(3));
            }
        }
        "RoomCode_SetupShaktoolRoomPlm" => {
            crate::plm::spawn_hardcoded(r, rom, 0, 0, 0xb8eb)?;
        }
        "RoomCode_SetPauseCodeForDraygon" => {
            r.set_word(0x601, 0xc8f6);
            r.set_byte(0x603, 0x8f);
            r.set_word(0x604, 0xc8fc);
            r.set_byte(0x606, 0x8f);
        }
        "PauseHook_DraygonRoom" => r.set_word(0xa7, 4),
        "UnpauseHook_DraygonRoom" => {
            if r.word(0x18b6) == 8 {
                r.set_word(0xa7, 12);
            }
        }
        "RoomCode_CrocomireShaking" => {
            if r.word(0xf86) & 0x200 == 0 {
                if r.word(0xfa8) == 64 {
                    r.set_word(0xb3, r.word(0xfee).wrapping_add(r.word(0x91f)).wrapping_add(r.word(LY)));
                } else if r.word(0xfaa) & 0x400 != 0 {
                    let timer = r.word(0xfee).wrapping_sub(1);
                    r.set_word(0xfee, timer);
                    let displacement = if neg(timer.wrapping_add(7)) {
                        timer.wrapping_sub(timer.wrapping_add(7).wrapping_mul(2))
                    } else {
                        timer
                    };
                    r.set_word(0xb3, r.word(0xb3).wrapping_add(displacement));
                    r.set_word(0xb7, displacement.wrapping_sub(48));
                } else if r.word(0xfac) == 34 && r.word(0xfae) != 0 {
                    let timer = r.word(0xfae).wrapping_sub(1);
                    r.set_word(0xfae, timer);
                    r.set_word(LX, r.word(LX).wrapping_add(if timer & 1 != 0 { 0xfffc } else { 4 }));
                }
            }
        }
        "RoomCode_RidleyRoomShaking" => {
            let timer = r.word(0x10a8);
            if timer != 0 {
                if timer > 8 {
                    return Err("Ridley room shake table extent");
                }
                let index = usize::from(timer - 1);
                let x: [u16; 8] = [0, 2, 2, 2, 0, 0xfffe, 0xfffe, 0xfffe];
                let y: [u16; 8] = [0xfffe, 0xfffe, 0, 2, 2, 2, 0, 0xfffe];
                r.set_word(0x10a8, timer - 1);
                for (address, displacement) in [(0xb1, x[index]), (0xb5, x[index]), (0xb3, y[index]), (0xb7, y[index])]
                {
                    r.set_word(address, r.word(address).wrapping_add(displacement));
                }
            }
        }
        "RoomCode_ShakeScreenHorizDiag" | "RoomCode_ShakeScreenHorizDiagStrong" => {
            let strong = symbol == "RoomCode_ShakeScreenHorizDiagStrong";
            let timer = r.word(0x7e1);
            let next_kind = if timer != 0 {
                r.set_word(0x7e1, timer.wrapping_sub(1));
                (timer == 1).then_some(if strong { 21 } else { 18 })
            } else if crate::enemy_projectiles::random(r) < if strong { 0x180 } else { 0x200 } {
                r.set_word(0x7e1, 42);
                Some(if strong { 26 } else { 23 })
            } else {
                None
            };
            if let Some(kind) = next_kind {
                r.set_word(if strong { 0x7e3 } else { 0x183e }, kind);
            }
            if strong {
                r.set_word(0x183e, r.word(0x7e3));
            }
            dispatch_rom(r, rom, "RoomCode_GenRandomExplodes_4th", 0, 0)?;
        }
        "RoomCode_SetCollectedMap" => {
            let area = usize::from(r.word(0x79f));
            if area >= 8 {
                return Err("room map area extent");
            }
            let value = r.word(0xd908 + area) | 1;
            r.set_word(0xd908 + area, value);
            r.set_word(0x789, value);
        }
        "RoomCode_SetZebesTimebombEvent" => {
            system::dispatch(r, "SetEventHappened", 14, 0)?;
            r.set_word(0x183e, 18);
            r.set_word(0x1840, 0xffff);
        }
        "RoomCode_SetLightHorizRoomShake"
        | "RoomCode_SetMediumHorizRoomShake"
        | "RoomCode_Escape4_SetMediumHorizRoomShake" => {
            if symbol == "RoomCode_Escape4_SetMediumHorizRoomShake" {
                crate::plm::spawn_hardcoded(r, rom, 16, 16, 0xb968)?;
            }
            let kind = if symbol == "RoomCode_SetLightHorizRoomShake" { 18 } else { 21 };
            r.set_word(0x183e, kind);
            if symbol != "RoomCode_SetMediumHorizRoomShake" {
                r.set_word(0x7e3, kind);
                r.set_word(0x7e1, 0);
            }
            r.set_word(0x1840, 0xffff);
        }
        "RoomSetup_AfterSavingAnimals" => {
            crate::plm::spawn_hardcoded(r, rom, 0x3d, 0x0b, 0xbb30)?;
            r.set_word(0x183e, 24);
            r.set_word(0x1840, 0xffff);
        }
        "RoomSetup_AutoDestroyWallAfterEscape" => {
            crate::plm::spawn_hardcoded(r, rom, 0x10, 0x87, 0xb964)?;
        }
        "RoomSetup_TurnWallIntoShotblocks" => {
            crate::plm::spawn_hardcoded(r, rom, 0x0f, 0x0a, 0xb9ed)?;
        }
        "DoorCode_StartWreckedShipTreadmillWest" | "DoorCode_StartWreckedSkipTreadmill_East" => {
            let west = symbol == "DoorCode_StartWreckedShipTreadmillWest";
            crate::animated_tiles::spawn(r, rom, if west { 0x8275 } else { 0x827b })?;
            crate::plm::spawn_hardcoded(r, rom, 4, 9, if west { 0xb64b } else { 0xb64f })?;
        }
        "RoomSetup_RunStatueUnlockAnims" => {
            for header in [0x8558, 0x854c, 0x855e, 0x8552] {
                crate::animated_tiles::spawn(r, rom, header)?;
            }
        }
        "RoomCode_GenRandomExplodes" => random_explosion(r, rom, a, b as u16)?,
        "RoomCode_GenRandomExplodes_Nonblank" | "RoomCode_GenRandomExplodes_4th" | "RoomCode_ExplodeShakes" => {
            let nonblank = symbol != "RoomCode_GenRandomExplodes_4th";
            let frame_mask = if nonblank { 1 } else { 3 };
            if r.word(0xa78) == 0 && r.word(0x5b6) & frame_mask == 0 {
                let random = crate::enemy_projectiles::random(r);
                let x = r.word(LX).wrapping_add(u16::from(random as u8));
                let y = r.word(LY).wrapping_add(random >> 8);
                let visible = if nonblank {
                    let row = u16::from((y >> 4) as u8) * u16::from(r.word(0x7a5) as u8);
                    // C promotes the sum to int before indexing level_data.
                    let address = 0x10002 + (usize::from(row) + usize::from(x >> 4)) * 2;
                    if address + 2 > crate::WRAM_SIZE {
                        return Err("room explosion level data extent");
                    }
                    r.word(address) & 0x3ff != 255
                } else {
                    true
                };
                if visible {
                    random_explosion(r, rom, x, y)?;
                }
            }
            if symbol == "RoomCode_ExplodeShakes" {
                r.set_word(0x1840, r.word(0x1840) | 0x8000);
            }
        }
        "HandleRoomDefStateSelect" => select_room(r, rom, a)?,
        s if s.starts_with("RoomDefStateSelect_") => return Ok(u32::from(select(r, rom, s, a)?)),
        "LoadRoomHeader" => header(r, rom)?,
        "LoadStateHeader" => state_header(r, rom)?,
        "LoaadDesinationRoomCreBitset" => {
            let d = data(rom, 0x83, r.word(DOOR), 12)?;
            let b = data(rom, 0x8f, read_word(d, 0), 11)?;
            r.set_word(0x7b1, r.word(0x7b3));
            r.set_word(0x7b3, u16::from(b[8]));
        }
        "LoadDoorHeader" => {
            let d = data(rom, 0x83, r.word(DOOR), 12)?;
            r.set_word(ROOM, read_word(d, 0));
            r.set_word(0x793, read_word(d, 2));
            r.set_word(0xe16, read_word(d, 2) & 128);
            r.set_word(0x791, u16::from(d[3]));
            r.set_word(0x927, u16::from(d[6]) << 8);
            r.set_word(0x929, u16::from(d[7]) << 8);
            let distance = read_word(d, 8);
            let distance = if neg(distance) { if d[3] & 2 != 0 { 384 } else { 200 } } else { distance };
            let speed = u32::from(distance) << 8;
            r.set_word(0x92b, speed as u16);
            r.set_word(0x92d, (speed >> 16) as u16);
        }
        "CalculateLayer2Xpos" => return Ok(layer2(r, true)),
        "CalculateLayer2Ypos" => return Ok(layer2(r, false)),
        "CalculateBgScrolls" => bg_scrolls(r),
        "CalculateBgScrollAndLayerPositionBlocks" => blocks(r),
        "UpdatePreviousLayerBlocks" => {
            for (dst, src) in [(0x8ff, 0x8f7), (0x901, 0x8f9), (0x903, 0x8fb), (0x905, 0x8fd)] {
                r.set_word(dst, r.word(src));
            }
        }
        "UpdateBgScrollOffsets" => {
            for (dst, reg, pos) in [(0x91d, 0xb1, LX), (0x91f, 0xb3, LY), (0x921, 0xb5, LX), (0x923, 0xb7, LY)] {
                r.set_word(dst, r.word(reg).wrapping_sub(r.word(pos)));
            }
        }
        "HandleScrollingWhenTriggeringScrollRight" => horizontal(r, true)?,
        "HandleScrollingWhenTriggeringScrollLeft" => horizontal(r, false)?,
        "HandleScrollingWhenTriggeringScrollDown" => vertical(r, true)?,
        "HandleScrollingWhenTriggeringScrollUp" => vertical(r, false)?,
        "HandleAutoscrolling_X" => automatic_x(r)?,
        "HandleAutoscrolling_Y" => automatic_y(r)?,
        "LoadRoomMusic" => {
            if r.word(0x998) < 40 && r.word(0x7cb) != 0 && r.word(0x7cb) != r.word(0x7f3) {
                audio::dispatch(r, "QueueMusic_Delayed8", 0, 0)?;
                audio::dispatch(r, "QueueMusic_Delayed8", r.word(0x7cb) | 0xff00, 0)?;
            }
        }
        "UpdateMusicTrackIndex" | "LoadNewMusicTrackIfChanged" => {
            if r.word(0x998) < 40
                && r.word(0x7c9) != 0
                && (r.word(0x7cb) << 8 | r.word(0x7c9)) != (r.word(0x7f3) << 8 | r.word(0x7f5))
            {
                if symbol == "UpdateMusicTrackIndex" {
                    r.set_word(0x7f5, r.word(0x7c9));
                } else {
                    audio::dispatch(r, "QueueMusic_DelayedY", r.word(0x7c9), 6)?;
                }
            }
        }
        "PlayRoomMusicTrackAfterAFrames" => {
            if r.word(0x998) < 40 {
                audio::dispatch(r, "QueueMusic_DelayedY", 0, a)?;
                audio::dispatch(r, "QueueMusic_Delayed8", r.word(0x7f5), 0)?;
            }
        }
        _ => return Err("unowned room/hardware transition routine"),
    }
    Ok(0)
}
