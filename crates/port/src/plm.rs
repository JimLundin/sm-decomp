//! Collision-related PLM setup, translated from MIT pinned snesrev/sm bank84.
use crate::Wram;
const HEADER: usize = 0x1c37;
const BLOCK: usize = 0x1c87;
const INSTR: usize = 0x1d27;
const VARIABLE: usize = 0x1e17;
const TIMER: usize = 0xde1c;
fn slot(j: u16) -> Result<usize, &'static str> {
    if j > 78 || j & 1 != 0 { Err("invalid PLM slot") } else { Ok(j as usize) }
}
fn block(r: &Wram, j: usize) -> Result<usize, &'static str> {
    let b = r.word(BLOCK + j) as usize;
    if b & 1 != 0 || b >= 0xc800 { Err("invalid PLM block") } else { Ok(b) }
}
fn tile(r: &Wram, b: usize) -> u16 {
    r.word(0x10002 + b)
}
fn set_tile(r: &mut Wram, b: usize, t: u16) {
    r.set_word(0x10002 + b, t);
}
fn write_type(r: &mut Wram, b: usize, t: u16) {
    r.set_byte(0x10003 + b, (t >> 8) as u8 | (r.byte(0x10003 + b) & 15));
    r.set_byte(0x16402 + b / 2, t as u8);
}
fn inc(r: &mut Wram, off: usize, n: u16) {
    r.set_word(off, r.word(off).wrapping_add(n));
}
fn delete(r: &mut Wram, j: usize) {
    r.set_word(HEADER + j, 0);
}
fn speed_breaks(r: &Wram, bombs: bool) -> bool {
    r.word(0xb3e) & 0xf00 == 0x400
        || (0xc9..=0xce).contains(&r.word(0xa1c))
        || (bombs && (0x81..=0x82).contains(&r.word(0xa1c)))
}
fn mark_broken(r: &mut Wram, j: usize, b: usize, graphic: u16, mask: u16) {
    let value = tile(r, b) & 0xf000 | graphic;
    r.set_word(VARIABLE + j, value);
    set_tile(r, b, value & mask);
}
/// Synchronous setup flags and optional movement amount mutation.
pub fn setup(r: &mut Wram, address: u16, j: u16, amount: &mut i32) -> Result<u8, &'static str> {
    let j = slot(j)?;
    let b = block(r, j)?;
    match address {
        0xb8dc => {
            r.set_word(0xcd20, 1);
            r.set_word(0xcd22, 0);
        }
        0xb96c => {}
        0xb5f8 | 0xb9c5 | 0xb9f1 => {
            let width = usize::from(r.word(0x7a5));
            let rows = if address == 0xb9c5 { 3 } else { 4 };
            if width == 0 || width > 255 || b + (rows - 1) * width * 2 >= 0x6400 {
                return Err("escape door column extent");
            }
            for row in 0..rows {
                let off = b + row * width * 2;
                if address == 0xb9f1 {
                    set_tile(r, off, tile(r, off) & 0xfff | 0x8000);
                } else {
                    write_type(r, off, if row == 0 { if address == 0xb5f8 { 0x9001 } else { 0xc04f } } else { 0xd0ff });
                }
            }
        }
        0xc6d8 | 0xc6dc => {
            let width = usize::from(r.word(0x7a5));
            if width == 0 || width > 255 {
                return Err("gate room width");
            }
            let indices: Vec<_> = (0..5)
                .map(|row| {
                    if address == 0xc6dc { (b / 2).checked_sub(row * width) } else { (b / 2).checked_add(row * width) }
                })
                .collect();
            if indices.iter().any(|index| index.is_none_or(|i| i >= 0x3200)) {
                return Err("gate column extent");
            }
            for index in indices {
                r.set_byte(0x16402 + index.unwrap(), 0x10);
            }
        }
        0xc7b1 => write_type(r, b, 0xc044),
        0xc7bb => {
            let projectile = r.word(0xdde);
            if projectile & 0x8000 == 0 && (projectile > 18 || projectile & 1 != 0) {
                return Err("door projectile index");
            }
            if projectile & 0x8000 == 0 && r.word(0xc18 + usize::from(projectile)) & 0xf00 == 0x300 {
                delete(r, j);
            } else {
                set_tile(r, b, tile(r, b) & 0xfff | 0x8000);
            }
        }
        0xc794 => {
            r.set_word(VARIABLE + j, (r.word(0x1dc7 + j) >> 8 & 0x7c) >> 1);
            r.set_word(0x1dc7 + j, r.word(0x1dc7 + j) & !0x7c00);
            write_type(r, b, 0xc044);
        }
        0xb7c3 => {
            r.set_word(0x1d77 + j, if r.word(0x79f) == 6 { 32 } else { 240 });
            r.set_word(0x1cd7 + j, 0xb7dd);
        }
        0xab28 => r.set_word(TIMER + j, 64),
        0xb04a => {
            if b + 112 > 0xc800 {
                return Err("room row extent");
            };
            for n in 0..56 {
                set_tile(r, b + n * 2, 255);
            }
        }
        0xb0dc | 0xb113 => {
            let y = r.word(0xafa);
            let radius = r.word(0xb00);
            let floor = address == 0xb0dc;
            let aligned = if floor {
                y.wrapping_add(radius).wrapping_sub(1) & 15 == 15
            } else {
                y.wrapping_sub(radius) & 15 == 0
            };
            if aligned {
                set_tile(r, b, tile(r, b) & 0x8fff);
                r.set_word(VARIABLE + j, r.word(0xaf6));
                r.set_word(0xdf0c + j, if floor { y.wrapping_sub(1) } else { y.wrapping_add(1) });
            } else {
                delete(r, j);
            }
        }
        0xb33a | 0xb345 | 0xb350 | 0xb35b => {
            write_type(
                r,
                b,
                match address {
                    0xb33a => 0x50ff,
                    0xb345 => 0x5001,
                    0xb350 => 0xd0ff,
                    _ => 0xd001,
                },
            );
            delete(r, j);
        }
        0xb371 | 0xb382 => {
            write_type(r, b, if address == 0xb371 { 0x3046 } else { 0xb046 });
            r.set_word(VARIABLE + j, 0);
            inc(r, INSTR + j, 4);
        }
        0xb393 => {
            let partner = (0..40)
                .rev()
                .map(|i| i * 2)
                .find(|&other| other != j && r.word(BLOCK + other) == b as u16)
                .ok_or("scroll PLM partner absent")?;
            r.set_word(BLOCK + j, 0);
            if r.word(VARIABLE + partner) & 0x8000 == 0 {
                r.set_word(VARIABLE + partner, 0x8000);
                inc(r, INSTR + partner, 2);
                r.set_word(TIMER + partner, 1);
            }
        }
        0xb3c1 => set_tile(r, b, tile(r, b) & 0x8fff),
        0xab27 | 0xb3cf | 0xb8ef | 0xbafa | 0xd18e | 0xd6f6 | 0xd6f7 | 0xb3d0 | 0xb54f => {}
        0xb3d2 => return Ok(1),
        0xb3d4 => set_tile(r, b, tile(r, b) & 0xfff),
        0xb3eb => {
            let bottom = r.word(0xafa).wrapping_add(r.word(0xb00)).wrapping_sub(1) & 15;
            if bottom == 7 || bottom == 15 {
                r.set_word(0xb4c, 16);
            }
        }
        0xb408 => {
            for off in [0xb3c, 0xb3e, 0xb40, 0xb44, 0xb42, 0xb46] {
                r.set_word(off, 0);
            }
            r.set_word(0xb48, r.word(0xb48) & 0x7fff);
            if r.word(0x1e73) == 0 {
                let gravity = r.word(0x9a2) & 0x20 != 0;
                match r.word(0xb36) & 3 {
                    0 | 3 => {
                        r.set_word(0xb2c, 0);
                        r.set_word(0xb2e, 0);
                        r.set_word(0xb5a, 0);
                        r.set_word(0xb5c, 0);
                        r.set_word(0xb5b, if gravity { 0x100 } else { 0x120 });
                    }
                    1 => {
                        let limit = if gravity { 0x380 } else { 0x280 };
                        if limit < r.word(0xb2d) {
                            r.set_word(0xb2c, 0);
                            r.set_word(0xb2e, 0);
                            r.set_word(0xb2d, limit);
                        }
                        r.set_word(0xb5a, 0);
                        r.set_word(0xb5c, 0);
                        r.set_word(0xb5b, 0x200);
                    }
                    _ => {
                        r.set_word(0xb5a, 0);
                        r.set_word(0xb5c, 0);
                        r.set_word(0xb5b, 0x200);
                        r.set_word(0xaf4, 0);
                    }
                }
            }
        }
        0xb497 | 0xb4a8 | 0xb4b6 => {
            if address == 0xb497 {
                r.set_word(0xaf4, 0)
            };
            r.set_word(
                0xb5a,
                match address {
                    0xb497 => 0x2000,
                    0xb4a8 => 0x4000,
                    _ => 0xc000,
                },
            );
            r.set_word(0xb5c, 1);
        }
        0xb4c4 => {
            if r.word(0xb02) & 2 != 0 {
                let dir = r.word(0xb36) & 3;
                if dir == 2 || ((dir == 0 || dir == 3) && r.word(0xb02) & 15 == 3) {
                    if r.word(0xa6e) == 1 {
                        *amount = 0;
                        return Ok(1);
                    }
                    if dir != 2 && (*amount >> 8) as u16 > 0x30 {
                        *amount = (*amount as u32 & 0xff0000ff | 0x3000) as i32;
                    }
                    inc(r, 0x1e71, 1);
                }
            }
        }
        0xb541 => {
            for off in [0xb2c, 0xb2e, 0xb32, 0xb34] {
                r.set_word(off, 0);
            }
        }
        0xb551 | 0xb56f => {
            let width = r.word(0x7a5) as usize;
            if width == 0 || width > 255 || b + width * 18 >= 0xc800 {
                return Err("wall room extent");
            };
            for row in 0..10 {
                let off = b + width * row * 2;
                set_tile(r, off, tile(r, off) & 0xfff | if address == 0xb56f { 0x8000 } else { 0 });
            }
        }
        0xb5ee => write_type(r, b, 0xb04d),
        0xcdea | 0xce83 => {
            if speed_breaks(r, address == 0xce83) {
                mark_broken(r, j, b, if address == 0xcdea { 0xb6 } else { 0x58 }, 0xfff);
            } else {
                delete(r, j);
                return Ok(1);
            }
        }
        0xce37 => {
            if r.word(0xb02) & 15 == 3 {
                mark_broken(r, j, b, 0xbc, 0x8fff);
                r.set_word(TIMER + j, 4);
            } else {
                delete(r, j);
            }
            return Ok(1);
        }
        0xce6b => mark_broken(r, j, b, 0x52, 0x8fff),
        0xceda | 0xcf2e | 0xcf67 | 0xcfa0 => {
            let p = r.word(0xdde) as usize;
            if p > 18 || p & 1 != 0 {
                return Err("projectile slot");
            };
            let kind = r.word(0xc18 + p) & 0xf00;
            if address == 0xcfa0 {
                if kind != 0x500 {
                    delete(r, j)
                }
            } else if kind == 0x500 {
                match address {
                    0xceda => {
                        inc(r, INSTR + j, 3);
                        mark_broken(r, j, b, 0x58, 0x8fff);
                    }
                    0xcf2e => r.set_word(INSTR + j, 0xc91c),
                    _ => r.set_word(INSTR + j, 0xc922),
                }
            } else if kind == if address == 0xcf67 { 0x200 } else { 0x300 } {
                mark_broken(
                    r,
                    j,
                    b,
                    if address == 0xcf67 {
                        0x9f
                    } else if address == 0xcf2e {
                        0x57
                    } else {
                        0x58
                    },
                    0x8fff,
                );
            } else {
                delete(r, j);
            }
        }
        0xcfb5 => {
            r.set_word(VARIABLE + j, tile(r, b));
            r.set_byte(0x16402 + b / 2, 0);
            return Ok(0x41);
        }
        0xcfcd => return Ok(0x41),
        0xcfd1 => return Ok(1),
        0xcfd5 => {
            inc(r, 0xa50, 1);
            return Ok(0x41);
        }
        0xd108 => set_tile(r, b, 0),
        0xd117 | 0xd12b => write_type(r, b, if address == 0xd117 { 0xc000 } else { 0xe000 }),
        0xd5f6 => {
            r.set_word(0x1dc7 + j, 0);
            write_type(r, b, 0x8044);
        }
        0xd616 => write_type(r, b, 0xb080),
        0xd67f | 0xd689 => write_type(r, b, if address == 0xd67f { 0x44 } else { 0x8044 }),
        _ => return Err("untranslated PLM setup"),
    }
    Ok(0)
}
fn rom_word(rom: &[u8], header: u16) -> Result<u16, &'static str> {
    if header < 0x8000 {
        return Err("PLM header below LoROM");
    };
    let off = (4 << 15) + (header as usize & 0x7fff);
    let bytes = rom.get(off..off + 2).ok_or("PLM ROM extent")?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}
/// Determine availability from immutable header data before any state mutation.
pub fn supports_header(rom: &[u8], header: u16) -> Result<bool, &'static str> {
    Ok(matches!(
        rom_word(rom, header)?,
        0xc794
            | 0xc7b1
            | 0xc7bb
            | 0xc6d8
            | 0xc6dc
            | 0xb5f8
            | 0xb8dc
            | 0xb96c
            | 0xb9c5
            | 0xb9f1
            | 0xb7c3
            | 0xab27
            | 0xb3cf
            | 0xb8ef
            | 0xbafa
            | 0xd18e
            | 0xd6f6
            | 0xd6f7
            | 0xab28
            | 0xb04a
            | 0xb0dc
            | 0xb113
            | 0xb33a
            | 0xb345
            | 0xb350
            | 0xb35b
            | 0xb371
            | 0xb382
            | 0xb393
            | 0xb3c1
            | 0xb3d0
            | 0xb3d2
            | 0xb3d4
            | 0xb3eb
            | 0xb408
            | 0xb497
            | 0xb4a8
            | 0xb4b6
            | 0xb4c4
            | 0xb541
            | 0xb54f
            | 0xb551
            | 0xb56f
            | 0xb5ee
            | 0xcdea
            | 0xce37
            | 0xce6b
            | 0xce83
            | 0xceda
            | 0xcf2e
            | 0xcf67
            | 0xcfa0
            | 0xcfb5
            | 0xcfcd
            | 0xcfd1
            | 0xcfd5
            | 0xd108
            | 0xd117
            | 0xd12b
            | 0xd5f6
            | 0xd616
            | 0xd67f
            | 0xd689
    ))
}
/// Allocate and synchronously set up a collision PLM. Errors leave RAM unchanged.
pub fn spawn(r: &mut Wram, rom: &[u8], header: u16, amount: &mut i32) -> Result<u8, &'static str> {
    let Some(j) = (0..40).rev().map(|i| i * 2).find(|&j| r.word(HEADER + j) == 0) else { return Ok(0) };
    if !supports_header(rom, header)? {
        return Err("untranslated PLM setup");
    };
    let setup_addr = rom_word(rom, header)?;
    let instructions = rom_word(rom, header.checked_add(2).ok_or("PLM header extent")?)?;
    let mut transaction: Vec<_> = (0..crate::WRAM_SIZE).map(|i| r.byte(i)).collect();
    let mut state = Wram::new(&mut transaction)?;
    let mut displacement = *amount;
    state.set_word(BLOCK + j, state.word(0xdc4).wrapping_mul(2));
    state.set_word(HEADER + j, header);
    state.set_word(0x1cd7 + j, 0x853d);
    state.set_word(INSTR + j, instructions);
    state.set_word(TIMER + j, 1);
    state.set_word(0xde6c + j, 0x8da0);
    for off in [0x1d77, 0x1dc7, 0xdf0c, VARIABLE] {
        state.set_word(off + j, 0);
    }
    state.set_word(0x1c27, j as u16);
    let flags = setup(&mut state, setup_addr, j as u16, &mut displacement)?;
    for (i, value) in transaction.into_iter().enumerate() {
        r.set_byte(i, value);
    }
    *amount = displacement;
    Ok(flags)
}
/// Read a room entry from bank8F or the door-closing scratch entry at WRAM 0x12.
pub fn room_entry(r: &Wram, rom: &[u8], entry: u16) -> Result<(u16, u8, u8, u16), &'static str> {
    if entry == 0x12 {
        return Ok((r.word(0x12), r.byte(0x14), r.byte(0x15), r.word(0x16)));
    }
    if !(0x8000..=0xfffa).contains(&entry) {
        return Err("room PLM entry extent");
    }
    let offset = (15 << 15) + usize::from(entry & 0x7fff);
    let b = rom.get(offset..offset + 6).ok_or("room PLM ROM extent")?;
    Ok((u16::from_le_bytes([b[0], b[1]]), b[2], b[3], u16::from_le_bytes([b[4], b[5]])))
}
/// Collision displacement callbacks are not valid room/hardcoded setup context.
pub fn supports_positioned_header(rom: &[u8], header: u16) -> Result<bool, &'static str> {
    Ok(supports_header(rom, header)? && !matches!(rom_word(rom, header)?, 0xb4b6 | 0xb4c4 | 0xb541 | 0xb551 | 0xb56f))
}
/// Room allocation preserves its distinct preinstruction and room argument.
pub fn spawn_room(r: &mut Wram, rom: &[u8], entry: u16) -> Result<(), &'static str> {
    if !(0..40).any(|i| r.word(HEADER + i * 2) == 0) {
        return Ok(());
    }
    let (header, x, y, argument) = room_entry(r, rom, entry)?;
    spawn_positioned(r, rom, header, x, y, argument, 0x84e6)
}
/// Hardcoded allocation uses byte coordinates and the original empty preinstruction.
pub fn spawn_hardcoded(r: &mut Wram, rom: &[u8], x: u8, y: u8, header: u16) -> Result<(), &'static str> {
    spawn_positioned(r, rom, header, x, y, 0, 0x8469)
}
fn spawn_positioned(
    r: &mut Wram,
    rom: &[u8],
    header: u16,
    x: u8,
    y: u8,
    argument: u16,
    preinstruction: u16,
) -> Result<(), &'static str> {
    let Some(j) = (0..40).rev().map(|i| i * 2).find(|&j| r.word(HEADER + j) == 0) else {
        return Ok(());
    };
    if !supports_positioned_header(rom, header)? {
        return Err("untranslated PLM setup");
    }
    let setup_address = rom_word(rom, header)?;
    let instructions = rom_word(rom, header.checked_add(2).ok_or("PLM header extent")?)?;
    let mut transaction: Vec<_> = (0..crate::WRAM_SIZE).map(|i| r.byte(i)).collect();
    let mut state = Wram::new(&mut transaction)?;
    let block = (u16::from(y) * u16::from(state.word(0x7a5) as u8) + u16::from(x)).wrapping_mul(2);
    state.set_word(BLOCK + j, block);
    state.set_word(HEADER + j, header);
    state.set_word(0x1dc7 + j, argument);
    state.set_word(VARIABLE + j, 0);
    state.set_word(0x1cd7 + j, preinstruction);
    state.set_word(INSTR + j, instructions);
    state.set_word(TIMER + j, 1);
    state.set_word(0xde6c + j, 0x8da0);
    state.set_word(0x1d77 + j, 0);
    state.set_word(0xdf0c + j, 0);
    state.set_word(0x1c27, j as u16);
    setup(&mut state, setup_address, j as u16, &mut 0)?;
    for (i, value) in transaction.into_iter().enumerate() {
        r.set_byte(i, value);
    }
    Ok(())
}
pub fn dispatch_rom(r: &mut Wram, _rom: &[u8], name: &str, a: i32, _b: i32) -> Result<u32, &'static str> {
    let address = match name {
        "PlmSetup_C794_GreyDoor" => 0xc794,
        "PlmSetup_Door_Colored" => 0xc7b1,
        "PlmSetup_Door_Blue" => 0xc7bb,
        "PlmSetup_C826_DownwardsOpenGate" => 0xc6d8,
        "PlmSetup_C82E_UpwardsOpenGate" => 0xc6dc,
        "PlmSetup_MotherBrainRoomEscapeDoor" => 0xb5f8,
        "PlmSetup_ShaktoolsRoom" => 0xb8dc,
        "PlmSetup_B974" => 0xb96c,
        "PlmSetup_B9ED_CrittersEscapeBlock" => 0xb9c5,
        "sub_84B9F1" => 0xb9f1,
        "PlmSetup_CrumbleBotwoonWall" => 0xab28,
        "PlmSetup_SetrupWreckedShipEntrance" => 0xb04a,
        "PlmSetup_BTS_Brinstar_0x80_Floorplant" => 0xb0dc,
        "PlmSetup_BTS_Brinstar_0x81_Ceilingplant" => 0xb113,
        "PlmSetup_B638_Rightwards_Extension" => 0xb33a,
        "PlmSetup_B63F_Leftwards_Extension" => 0xb345,
        "PlmSetup_B643_Downwards_Extension" => 0xb350,
        "PlmSetup_B647_Upwards_Extension" => 0xb35b,
        "PlmSetup_B703_ScrollPLM" => 0xb371,
        "PlmSetup_B707_SolidScrollPLM" => 0xb382,
        "PlmSetup_B6FF_ScrollBlockTouch" => 0xb393,
        "PlmSetup_DeactivatePlm" => 0xb3c1,
        "PlmSetup_ReturnCarryClear" => 0xb3d0,
        "PlmSetup_ReturnCarrySet" => 0xb3d2,
        "PlmSetup_D094_EnemyBreakableBlock" => 0xb3d4,
        "PlmSetup_B70F_IcePhysics" => 0xb3eb,
        "PlmSetup_QuicksandSurface" => 0xb408,
        "PlmSetup_B71F_SubmergingQuicksand" => 0xb497,
        "PlmSetup_B723_SandfallsSlow" => 0xb4a8,
        "PlmSetup_B727_SandFallsFast" => 0xb4b6,
        "PlmSetup_QuicksandSurfaceB" => 0xb4c4,
        "PlmSetup_B737_SubmergingQuicksand" => 0xb541,
        "PlmSetup_B73B_B73F_SandFalls" => 0xb54f,
        "PlmSetup_ClearShitroidInvisibleWall" => 0xb551,
        "PlmSetup_B767_ClearShitroidInvisibleWall" => 0xb56f,
        "PlmSetup_B76F_SaveStation" => 0xb5ee,
        "PlmSetup_RespawningSpeedBoostBlock" => 0xcdea,
        "PlmSetup_RespawningCrumbleBlock" => 0xce37,
        "PlmSetup_RespawningShotBlock" => 0xce6b,
        "PlmSetup_RespawningBombBlock" => 0xce83,
        "PlmSetup_RespawningBombBlock2" => 0xceda,
        "PlmSetup_RespawningPowerBombBlock" => 0xcf2e,
        "PlmSetup_D08C_SuperMissileBlockRespawning" => 0xcf67,
        "PlmSetup_D08C_CrumbleBlock" => 0xcfa0,
        "PlmSetup_D0DC_BreakableGrappleBlock" => 0xcfb5,
        "PlmSetup_D0D8_SetVFlag" => 0xcfcd,
        "PlmSetup_D0D8_ClearVflag" => 0xcfd1,
        "PlmSetup_D0E8_GiveSamusDamage" => 0xcfd5,
        "PlmSetup_D113_LowerNorfairChozoRoomPlug" => 0xd108,
        "PlmSetup_D127" => 0xd117,
        "PlmSetup_D138" => 0xd12b,
        "PlmSetup_MotherBrainGlass" => 0xd5f6,
        "PlmSetup_MakeBllockChozoHandTrigger" => 0xd616,
        "PlmSetup_D700_MakePlmAirBlock_Unused" => 0xd67f,
        "PlmSetup_D704_AlteranateLowerNorfairChozoHand_Unused" => 0xd689,
        _ => return Err("untranslated PLM routine"),
    };
    let mut amount = 0;
    Ok(setup(r, address, a as u16, &mut amount)?.into())
}
