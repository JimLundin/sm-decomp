//! Samus movement physics from pinned snesrev/sm src/sm_90.c.
//! Upstream license: crates/decomp/LICENSE.upstream.txt.
use crate::Wram;

const Y_POS: usize = 0xafa;
const POSE: usize = 0xa1c;
const EQUIPPED_ITEMS: usize = 0x9a2;
const WATER_Y: usize = 0x195e;
const LAVA_ACID_Y: usize = 0x1962;
const LIQUID_OPTIONS: usize = 0x197e;
const Y_SUBSPEED: usize = 0xb2c;
const Y_SPEED: usize = 0xb2e;
const EXTRA_RUN_SUBSPEED: usize = 0xb44;
const EXTRA_RUN_SPEED: usize = 0xb42;
const GRAPPLE_WALLJUMP_TIMER: usize = 0xa9e;
const CERES_FADE_TIMER: usize = 0xaa0;
const Y_DIRECTION: usize = 0xb36;
const MOVEMENT_TYPE: usize = 0xa1f;
const RUN_BUTTON: usize = 0x9b6;
const HELD_BUTTONS: usize = 0x8b;
const HAS_MOMENTUM: usize = 0xb3c;
const BOOST_COUNTER: usize = 0xb3e;
const PALETTE_TIMER: usize = 0xad0;
const PALETTE_FRAME: usize = 0xace;
const CONTACT_DAMAGE: usize = 0xa6e;
const BASE_SPEED: usize = 0xb46;
const BASE_SUBSPEED: usize = 0xb48;
const ACCEL_MODE: usize = 0xb4a;
const DECEL_MULTIPLIER: usize = 0xb4c;
const SPEED_DIVISOR: usize = 0xa66;
const TOTAL_X_SPEED: usize = 0xdbc;
const TOTAL_X_SUBSPEED: usize = 0xdbe;
const X_POS: usize = 0xaf6;
const X_SUBPOS: usize = 0xaf8;
const Y_SUBPOS: usize = 0xafc;
const MOVED_RIGHT: usize = 0xdae;
const MOVED_RIGHT_FRACTION: usize = 0xdb0;
const MOVED_LEFT: usize = 0xdaa;
const MOVED_LEFT_FRACTION: usize = 0xdac;
const MOVED_DOWN: usize = 0xdb6;
const MOVED_DOWN_FRACTION: usize = 0xdb8;
const MOVED_UP: usize = 0xdb2;
const MOVED_UP_FRACTION: usize = 0xdb4;
const PREV_X_POS: usize = 0xb10;
const PREV_X_SUBPOS: usize = 0xb12;
const DISTANCE_X: usize = 0xda2;
const DISTANCE_X_FRACTION: usize = 0xda4;
const PREV_Y_POS: usize = 0xb14;
const PREV_Y_SUBPOS: usize = 0xb16;
const DISTANCE_Y: usize = 0xda6;
const DISTANCE_Y_FRACTION: usize = 0xda8;
const EXTRA_X_DISPLACEMENT: usize = 0xb58;
const EXTRA_X_SUBDISPLACEMENT: usize = 0xb56;
const COLLISION_DIRECTION: usize = 0xb02;
const Y_ACCEL: usize = 0xb34;
const Y_SUBACCEL: usize = 0xb32;
const ANIM_TIMER: usize = 0xa94;
const ANIM_FRAME: usize = 0xa96;
const ECHO_CURSOR: usize = 0xaae;
const GAME_TIME_FRAMES: usize = 0x9da;
const ECHO_X: usize = 0xab0;
const ECHO_Y: usize = 0xab8;
const LIQUID_PHYSICS: usize = 0xad2;
const SPEED_TABLE: usize = 0xa6c;
const Y_RADIUS: usize = 0xb00;
const X_RADIUS: usize = 0xafe;
const PREV_POSE: usize = 0xa20;
const SUIT_PALETTE: usize = 0xa74;

fn rom_word(rom: &[u8], address: u32) -> Result<u16, &'static str> {
    let offset = (((address >> 16) << 15) | (address & 0x7fff)) as usize & 0x3fffff;
    let bytes = rom.get(offset..offset + 2).ok_or("ROM table outside image")?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
}
fn radius(rom: &[u8], pose: u16) -> Result<u16, &'static str> {
    if pose > 252 {
        return Err("invalid Samus pose");
    }
    Ok(rom_word(rom, 0x91b629 + u32::from(pose) * 8 + 6)? & 255)
}
fn bottom(ram: &Wram<'_>, rom: &[u8]) -> Result<u16, &'static str> {
    Ok(ram.word(Y_POS).wrapping_add(radius(rom, ram.word(POSE))?).wrapping_sub(1))
}

fn negative(value: u16) -> bool {
    value & 0x8000 != 0
}
/// Gameplay liquid tests prioritize a valid water surface over lava/acid.
fn jump_medium(ram: &Wram<'_>, rom: &[u8]) -> Result<usize, &'static str> {
    if ram.word(EQUIPPED_ITEMS) & 0x20 != 0 {
        return Ok(0);
    }
    let feet = bottom(ram, rom)?;
    let water = ram.word(WATER_Y);
    let lava = ram.word(LAVA_ACID_Y);
    if !negative(water) {
        Ok(if negative(water.wrapping_sub(feet)) && ram.word(LIQUID_OPTIONS) & 4 == 0 { 1 } else { 0 })
    } else {
        Ok(if !negative(lava) && negative(lava.wrapping_sub(feet)) { 2 } else { 0 })
    }
}
fn init_jump(ram: &mut Wram<'_>, rom: &[u8], kind: &str) -> Result<(), &'static str> {
    let medium = jump_medium(ram, rom)?;
    let high = ram.word(EQUIPPED_ITEMS) & 0x100 != 0;
    let (speeds, fractions): ([u16; 3], [u16; 3]) = match kind {
        "Samus_InitJump" if high => ([6, 2, 3], [0, 0x8000, 0x8000]),
        "Samus_InitJump" => ([4, 1, 2], [0xe000, 0xc000, 0xc000]),
        "Samus_InitWallJump" if high => ([5, 0, 3], [0x8000; 3]),
        "Samus_InitWallJump" => ([4, 0, 2], [0xa000, 0x4000, 0xa000]),
        "Samus_InitBombJump" => ([2, 0, 0], [0xc000, 0x1000, 0x1000]),
        _ => ([5, 2, 2], [0; 3]),
    };
    let boosted = ram.word(EQUIPPED_ITEMS) & 0x2000 != 0 && (kind == "Samus_InitJump" || kind == "Samus_InitWallJump");
    // Upstream adds fractions separately, deliberately discarding carry.
    ram.set_word(Y_SUBSPEED, fractions[medium].wrapping_add(if boosted { ram.word(EXTRA_RUN_SUBSPEED) } else { 0 }));
    ram.set_word(Y_SPEED, speeds[medium].wrapping_add(if boosted { ram.word(EXTRA_RUN_SPEED) >> 1 } else { 0 }));
    ram.set_word(GRAPPLE_WALLJUMP_TIMER, 0);
    ram.set_word(CERES_FADE_TIMER, 0);
    ram.set_word(Y_DIRECTION, 1);
    Ok(())
}

fn pair(ram: &Wram<'_>, hi: usize, lo: usize) -> u32 {
    (u32::from(ram.word(hi)) << 16) | u32::from(ram.word(lo))
}
fn set_pair(ram: &mut Wram<'_>, hi: usize, lo: usize, value: u32) {
    ram.set_word(hi, (value >> 16) as u16);
    ram.set_word(lo, value as u16);
}
fn extra_run_speed(ram: &mut Wram<'_>, rom: &[u8]) -> Result<(), &'static str> {
    let running = jump_medium(ram, rom)? == 0
        && ram.byte(MOVEMENT_TYPE) == 1
        && ram.word(RUN_BUTTON) & ram.word(HELD_BUTTONS) != 0;
    if !running {
        if ram.word(HAS_MOMENTUM) == 0 {
            set_pair(ram, EXTRA_RUN_SPEED, EXTRA_RUN_SUBSPEED, 0);
        }
    } else {
        let boosted = ram.word(EQUIPPED_ITEMS) & 0x2000 != 0;
        if ram.word(HAS_MOMENTUM) == 0 {
            let counter = if boosted { rom_word(rom, 0x91b61f)? } else { 0 };
            ram.set_word(HAS_MOMENTUM, 1);
            ram.set_word(BOOST_COUNTER, counter);
            if boosted {
                ram.set_word(PALETTE_TIMER, 1);
                ram.set_word(PALETTE_FRAME, 0);
            }
        }
        let limit = if boosted { 7 } else { 2 };
        // Signed comparisons of the two words are independent in upstream.
        if !negative(ram.word(EXTRA_RUN_SPEED).wrapping_sub(limit)) && !negative(ram.word(EXTRA_RUN_SUBSPEED)) {
            set_pair(ram, EXTRA_RUN_SPEED, EXTRA_RUN_SUBSPEED, u32::from(limit) << 16);
        } else {
            let next = pair(ram, EXTRA_RUN_SPEED, EXTRA_RUN_SUBSPEED).wrapping_add(0x1000);
            set_pair(ram, EXTRA_RUN_SPEED, EXTRA_RUN_SUBSPEED, next);
        }
    }
    if ram.word(BOOST_COUNTER) & 0xff00 == 0x400 {
        ram.set_word(CONTACT_DAMAGE, 1);
    }
    Ok(())
}

fn base_speed(ram: &mut Wram<'_>, rom: &[u8], pointer: u16) -> Result<u32, &'static str> {
    // Only known table records are admitted to the unchecked C oracle.
    let valid = [(0x9f25u16, 3u16), (0x9f49, 28), (0xa08d, 28), (0xa1dd, 28)]
        .iter()
        .any(|&(base, count)| pointer >= base && (pointer - base).is_multiple_of(12) && (pointer - base) / 12 < count);
    if !valid {
        return Err("invalid Samus speed table entry");
    }
    let words: Vec<u16> =
        (0..6).map(|n| rom_word(rom, 0x900000 | u32::from(pointer + 2 * n))).collect::<Result<_, _>>()?;
    let old = pair(ram, BASE_SPEED, BASE_SUBSPEED);
    let mut value;
    if ram.word(ACCEL_MODE) != 0 {
        let multiplier = ram.byte(DECEL_MULTIPLIER);
        let delta = if multiplier != 0 {
            let whole = u32::from(multiplier) * u32::from(words[4] as u8);
            let fraction = u32::from(multiplier) * u32::from(words[5] >> 8);
            ((whole >> 8) << 16) | fraction
        } else {
            (u32::from(words[4]) << 16) | u32::from(words[5])
        };
        value = old.wrapping_sub(delta);
        if value & 0x80000000 != 0 {
            value = 0;
            ram.set_word(ACCEL_MODE, 0);
        }
    } else {
        value = old.wrapping_add((u32::from(words[0]) << 16) | u32::from(words[1]));
        let hi = (value >> 16) as u16;
        let lo = value as u16;
        if !negative(hi.wrapping_sub(words[2]))
            && (hi != words[2] || (!negative(lo.wrapping_sub(words[3])) && lo != words[3]))
        {
            value = (u32::from(words[2]) << 16) | u32::from(words[3]);
        }
    }
    set_pair(ram, BASE_SPEED, BASE_SUBSPEED, value);
    Ok(value)
}

fn total_speed(ram: &mut Wram<'_>, amount: u32) -> u32 {
    let speed = amount.wrapping_add(pair(ram, EXTRA_RUN_SPEED, EXTRA_RUN_SUBSPEED)) >> ram.word(SPEED_DIVISOR).min(4);
    set_pair(ram, TOTAL_X_SPEED, TOTAL_X_SUBSPEED, speed);
    speed
}
fn clamp_speed(amount: u32) -> u32 {
    let integer = (amount >> 16) as i16;
    // Preserve the original fractional word, even at either signed clamp.
    let clamped = if integer < 0 {
        if negative((integer as u16).wrapping_add(15)) { -15 } else { integer }
    } else if !negative((integer as u16).wrapping_sub(16)) {
        15
    } else {
        integer
    };
    ((clamped as u16 as u32) << 16) | (amount & 0xffff)
}
fn move_without_collision(ram: &mut Wram<'_>, symbol: &str, amount: i32) -> Result<(), &'static str> {
    if amount == i32::MIN {
        return Err("upstream signed-negation domain exceeded");
    }
    let (hi, lo, out_hi, out_lo, negative) = match symbol {
        "Samus_MoveRight_NoColl" => (X_POS, X_SUBPOS, MOVED_RIGHT, MOVED_RIGHT_FRACTION, false),
        "Samus_MoveLeft_NoColl" => (X_POS, X_SUBPOS, MOVED_LEFT, MOVED_LEFT_FRACTION, true),
        "Samus_MoveDown_NoColl" => (Y_POS, Y_SUBPOS, MOVED_DOWN, MOVED_DOWN_FRACTION, false),
        _ => (Y_POS, Y_SUBPOS, MOVED_UP, MOVED_UP_FRACTION, true),
    };
    let delta = if negative { (amount as u32).wrapping_neg() } else { amount as u32 };
    let next = pair(ram, hi, lo).wrapping_add(delta);
    set_pair(ram, hi, lo, next);
    set_pair(ram, out_hi, out_lo, delta);
    Ok(())
}
fn distance_moved(ram: &mut Wram<'_>, horizontal: bool) {
    let (hi, lo, prev_hi, prev_lo, out_hi, out_lo) = if horizontal {
        (X_POS, X_SUBPOS, PREV_X_POS, PREV_X_SUBPOS, DISTANCE_X, DISTANCE_X_FRACTION)
    } else {
        (Y_POS, Y_SUBPOS, PREV_Y_POS, PREV_Y_SUBPOS, DISTANCE_Y, DISTANCE_Y_FRACTION)
    };
    let current = pair(ram, hi, lo);
    let previous = pair(ram, prev_hi, prev_lo);
    let difference = if !negative(ram.word(hi).wrapping_sub(ram.word(prev_hi))) {
        current.wrapping_sub(previous)
    } else {
        previous.wrapping_sub(current)
    };
    set_pair(ram, out_hi, out_lo, difference.wrapping_add(0x10000));
}

pub fn dispatch_rom(ram: &mut Wram<'_>, rom: &[u8], symbol: &str, a: i32, _: i32) -> Result<u32, &'static str> {
    match symbol {
        "Samus_CalcSpeed_X" => return Ok(total_speed(ram, a as u32)),
        "Samus_CalcDisplacementMoveLeft" | "Samus_CalcDisplacementMoveRight" => {
            let speed = total_speed(ram, a as u32);
            let extra = pair(ram, EXTRA_X_DISPLACEMENT, EXTRA_X_SUBDISPLACEMENT);
            let left = symbol == "Samus_CalcDisplacementMoveLeft";
            ram.set_word(COLLISION_DIRECTION, if left { 0 } else { 1 });
            return Ok(clamp_speed(if left { extra.wrapping_sub(speed) } else { extra.wrapping_add(speed) }));
        }
        "Samus_MoveRight_NoColl" | "Samus_MoveLeft_NoColl" | "Samus_MoveDown_NoColl" | "Samus_MoveUp_NoColl" => {
            move_without_collision(ram, symbol, a)?
        }
        "Samus_CalcDistanceMoved_X" | "Samus_CalcDistanceMoved_Y" => distance_moved(ram, symbol.ends_with("_X")),
        "Samus_CheckStartFalling" => {
            if ram.word(Y_DIRECTION) == 1 && negative(ram.word(Y_SPEED)) {
                set_pair(ram, Y_SPEED, Y_SUBSPEED, 0);
                ram.set_word(Y_DIRECTION, 2);
            }
        }
        "Samus_Func9" => {
            if negative(ram.word(Y_SPEED).wrapping_sub(5)) {
                let speed = pair(ram, Y_SPEED, Y_SUBSPEED).wrapping_add(pair(ram, Y_ACCEL, Y_SUBACCEL));
                set_pair(ram, Y_SPEED, Y_SUBSPEED, speed);
            }
            if [0x29, 0x2a, 0x67, 0x68].contains(&ram.word(POSE)) && !negative(ram.word(Y_SPEED).wrapping_sub(5)) {
                ram.set_word(ANIM_TIMER, 16);
                ram.set_word(ANIM_FRAME, 4);
            }
        }
        "Samus_UpdateSpeedEchoPos" => {
            let index = ram.word(ECHO_CURSOR);
            if ram.word(BOOST_COUNTER) & 0xff00 == 0x400 && !negative(index) && ram.word(GAME_TIME_FRAMES) & 3 == 0 {
                if index != 0 && index != 2 {
                    return Err("invalid speed echo cursor");
                }
                ram.set_word(ECHO_X + usize::from(index), ram.word(X_POS));
                ram.set_word(ECHO_Y + usize::from(index), ram.word(Y_POS));
                ram.set_word(ECHO_CURSOR, if index == 0 { 2 } else { 0 });
            }
        }
        "Samus_CalcBaseSpeed_X" => return base_speed(ram, rom, a as u16),
        "Samus_DetermineSpeedTableEntryPtr_X" => {
            let medium = jump_medium(ram, rom)?;
            if medium != 0 {
                ram.set_word(SPEED_TABLE, if medium == 1 { 0xa08d } else { 0xa1dd });
            }
            return Ok(u32::from(ram.word(SPEED_TABLE).wrapping_add(12 * u16::from(ram.byte(MOVEMENT_TYPE)))));
        }
        "Samus_DetermineGrappleSwingSpeed_X" => return Ok([0x9f31, 0x9f3d, 0x9f49][jump_medium(ram, rom)?]),
        "Samus_HandleExtraRunspeedX" => extra_run_speed(ram, rom)?,
        "SetLiquidPhysicsType" => {
            let feet = bottom(ram, rom)?;
            let medium = match (ram.word(0x196e) & 15) >> 1 {
                1 | 2 if !negative(ram.word(LAVA_ACID_Y)) && negative(ram.word(LAVA_ACID_Y).wrapping_sub(feet)) => 2,
                3 if !negative(ram.word(WATER_Y))
                    && negative(ram.word(WATER_Y).wrapping_sub(feet))
                    && ram.word(LIQUID_OPTIONS) & 4 == 0 =>
                {
                    1
                }
                _ => 0,
            };
            ram.set_word(LIQUID_PHYSICS, medium);
        }
        "Samus_InitJump" | "Samus_InitWallJump" | "Samus_InitBombJump" | "Samus_SetSpeedForKnockback_Y" => {
            init_jump(ram, rom, symbol)?
        }
        "Samus_DetermineAccel_Y" => {
            let medium = jump_medium(ram, rom)?;
            ram.set_word(Y_SUBACCEL, [0x1c00, 0x800, 0x900][medium]);
            ram.set_word(Y_ACCEL, 0);
        }
        "Samus_GetBottom_R18" => return bottom(ram, rom).map(u32::from),
        "Samus_GetTop_R20" => return Ok(u32::from(ram.word(Y_POS).wrapping_sub(radius(rom, ram.word(POSE))?))),
        "Samus_SetRadius" => {
            let r = radius(rom, ram.word(POSE))?;
            ram.set_word(Y_RADIUS, r);
            ram.set_word(X_RADIUS, 5);
        }
        "Samus_AlignBottomWithPrevPose" => {
            let delta = radius(rom, ram.word(PREV_POSE))?.wrapping_sub(radius(rom, ram.word(POSE))?);
            for off in [Y_POS, PREV_Y_POS] {
                ram.set_word(off, ram.word(off).wrapping_add(delta));
            }
        }
        "Samus_UpdateSuitPaletteIndex" => {
            let items = ram.word(EQUIPPED_ITEMS);
            ram.set_word(
                SUIT_PALETTE,
                if items & 0x20 != 0 {
                    4
                } else if items & 1 != 0 {
                    2
                } else {
                    0
                },
            );
        }
        _ => return Err("unknown Samus routine"),
    }
    Ok(0)
}
