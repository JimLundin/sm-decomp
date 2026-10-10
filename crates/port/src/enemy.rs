//! Generic enemy geometry and state from pinned snesrev/sm `src/sm_a0.c`.
//! Upstream MIT copyright/license: crates/decomp/LICENSE.upstream.txt.
use crate::Wram;
const ENEMIES: usize = 0xf78;
const CURRENT_ENEMY: usize = 0xe54;
const X: usize = 2;
const X_SUB: usize = 4;
const Y: usize = 6;
const Y_SUB: usize = 8;
const WIDTH: usize = 10;
const HEIGHT: usize = 12;
const SAMUS_X: usize = 0xaf6;
const SAMUS_Y: usize = 0xafa;
const SAMUS_WIDTH: usize = 0xafe;
const SAMUS_HEIGHT: usize = 0xb00;
fn record(index: u16) -> Result<usize, &'static str> {
    if index >= 0x800 || index & 0x3f != 0 {
        return Err("enemy offset must select a 0x40-byte record below 0x800");
    }
    Ok(ENEMIES + usize::from(index))
}
fn abs16(value: u16) -> u16 {
    if value & 0x8000 != 0 { value.wrapping_neg() } else { value }
}
fn negative(value: u16) -> bool {
    value & 0x8000 != 0
}
fn angle_from_xy(mut x: u16, mut y: u16) -> u16 {
    let quadrant = usize::from(negative(x)) * 2 + usize::from(negative(y));
    x = abs16(x);
    y = abs16(y);
    let horizontal = y < x;
    let (numerator, denominator) = if horizontal { (y.wrapping_shl(8), x as u8) } else { (x.wrapping_shl(8), y as u8) };
    let quotient = if denominator == 0 { 65535 } else { numerator / u16::from(denominator) };
    let step = (quotient >> 3) as u8;
    u16::from(if horizontal {
        match quadrant {
            0 => 64u8.wrapping_add(step),
            1 => 64u8.wrapping_sub(step),
            2 => 192u8.wrapping_sub(step),
            _ => step.wrapping_sub(64),
        }
    } else {
        match quadrant {
            0 => 128u8.wrapping_sub(step),
            1 => step,
            2 => 128u8.wrapping_add(step),
            _ => 0u8.wrapping_sub(step),
        }
    })
}
fn mult8(a: u16, b: u16) -> u16 {
    u16::from(a as u8) * u16::from(b as u8)
}
fn block_probe(ram: &Wram<'_>, enemy: u16, amount: i32, horizontal: bool) -> Result<u32, &'static str> {
    let e = record(enemy)?;
    let x = ram.word(e + X);
    let y = ram.word(e + Y);
    let width = ram.word(e + WIDTH);
    let height = ram.word(e + HEIGHT);
    let room_width = ram.word(0x7a5);
    let perpendicular_min = if horizontal { y.wrapping_sub(height) } else { x.wrapping_sub(width) };
    let perpendicular_max =
        if horizontal { height.wrapping_add(y).wrapping_sub(1) } else { width.wrapping_add(x).wrapping_sub(1) };
    let mut span = perpendicular_max.wrapping_sub(perpendicular_min & 0xfff0) >> 4;
    let position = if horizontal {
        u32::from(x) << 16 | u32::from(ram.word(e + X_SUB))
    } else {
        u32::from(y) << 16 | u32::from(ram.word(e + Y_SUB))
    };
    let integer = (position.wrapping_add(amount as u32) >> 16) as u16;
    let radius = if horizontal { width } else { height };
    let edge = if amount < 0 { integer.wrapping_sub(radius) } else { radius.wrapping_add(integer).wrapping_sub(1) };
    let first = if horizontal {
        mult8(perpendicular_min >> 4, room_width).wrapping_add(edge >> 4)
    } else {
        mult8(edge >> 4, room_width).wrapping_add(perpendicular_min >> 4)
    };
    // Horizontal C stores the byte index in u16; vertical C uses an int loop index.
    let mut byte_index = if horizontal { usize::from(first.wrapping_mul(2)) } else { usize::from(first) * 2 };
    loop {
        if byte_index / 2 >= 0x3200 {
            return Err("probe crosses level-data allocation");
        }
        if ram.word(0x10002 + byte_index) & 0x8000 != 0 {
            return Ok(1);
        }
        span = span.wrapping_sub(1);
        if negative(span) {
            return Ok(0);
        }
        byte_index = if horizontal {
            usize::from((byte_index as u16).wrapping_add(room_width.wrapping_mul(2)))
        } else {
            byte_index + 2
        };
    }
}
pub fn dispatch_rom(ram: &mut Wram<'_>, rom: &[u8], symbol: &str, a: i32, b: i32) -> Result<u32, &'static str> {
    let a = u16::try_from(a).map_err(|_| "first argument must be u16")?;
    if symbol == "EnemyFunc_BBBF" || symbol == "EnemyFunc_BC76" {
        return block_probe(ram, a, b, symbol == "EnemyFunc_BBBF");
    }
    let b = u16::try_from(b).map_err(|_| "second argument must be u16")?;
    match symbol {
        "CalculateBlockContainingPixelPos" => {
            ram.set_word(0xdc4, mult8(b >> 4, ram.word(0x7a5)).wrapping_add(a >> 4));
            Ok(0)
        }
        "DecrementSamusTimers" => {
            for offset in [0x18a8, 0x18aa, 0x18ac] {
                let timer = ram.word(offset);
                if timer != 0 {
                    ram.set_word(offset, timer - 1);
                }
            }
            ram.set_word(0x17ac, 65535);
            ram.set_word(0x17ec, 65535);
            Ok(0)
        }
        "EnemyFunc_B7A1" => {
            for (current, previous) in [(SAMUS_Y, 0xb14), (SAMUS_X, 0xb10)] {
                let position = ram.word(current);
                let delta = position.wrapping_sub(ram.word(previous));
                if !negative(abs16(delta).wrapping_sub(12)) {
                    ram.set_word(previous, position.wrapping_add(if negative(delta) { 0xfff4 } else { 12 }));
                }
            }
            Ok(0)
        }
        "NormalEnemyFrozenAI" => {
            let e = record(ram.word(CURRENT_ENEMY))?;
            ram.set_word(e + 0x24, 0);
            let timer = ram.word(e + 0x26);
            if timer != 0 {
                ram.set_word(e + 0x26, timer.wrapping_sub(1));
            }
            if timer == 0 || ram.word(0x9a6) & 2 == 0 {
                let bits = ram.word(e + 0x12) & !4;
                ram.set_word(e + 0x12, bits);
                ram.set_word(e + 0x26, bits);
            }
            Ok(0)
        }
        "SwitchEnemyAiToMainAi" => {
            let e = record(ram.word(CURRENT_ENEMY))?;
            for field in [0x12, 0x28, 0x26, 0x2a] {
                ram.set_word(e + field, 0);
            }
            Ok(0)
        }
        "SineMult8bit" | "CosineMult8bit" | "SineMult8bitFull" | "CosineMult8bitFull" => {
            let angle = a.wrapping_add(if symbol.starts_with("Cosine") { 64 } else { 128 }) as u8;
            let sine = *rom.get(0x103143 + usize::from(angle & 0x7f)).ok_or("ROM missing sine table")?;
            let product = u16::from(sine) * u16::from(b as u8);
            let mut hi = product >> 8;
            let mut lo = product << 8;
            if angle & 0x80 != 0 {
                hi = hi.wrapping_neg();
                lo = lo.wrapping_neg();
            }
            Ok(if symbol.ends_with("Full") { u32::from(hi) << 16 | u32::from(lo) } else { u32::from(hi) })
        }
        "SignExtend8" => Ok(u32::from(if a & 0x80 != 0 { a | 0xff00 } else { a })),
        "Mult32" => Ok(u32::from(a.wrapping_mul(32))),
        "Abs16" => Ok(u32::from(abs16(a))),
        "SubtractThenAbs16" => Ok(u32::from(abs16(b.wrapping_sub(a)))),
        "CalculateAngleFromXY" => Ok(u32::from(angle_from_xy(a, b))),
        "CalculateAngleOfSamusFromEnemy" => {
            let e = record(a)?;
            Ok(u32::from(angle_from_xy(
                ram.word(SAMUS_X).wrapping_sub(ram.word(e + X)),
                ram.word(SAMUS_Y).wrapping_sub(ram.word(e + Y)),
            )))
        }
        "CalculateAngleOfEnemyXfromEnemyY" => {
            let first = record(a)?;
            let second = record(b)?;
            Ok(u32::from(angle_from_xy(
                ram.word(first + X).wrapping_sub(ram.word(second + X)),
                ram.word(first + Y).wrapping_sub(ram.word(second + Y)),
            )))
        }
        "CalculateAngleOfSamusFromEproj" => {
            if a >= 36 || a & 1 != 0 {
                return Err("enemy projectile offset must be even and below 36");
            }
            Ok(u32::from(angle_from_xy(
                ram.word(SAMUS_X).wrapping_sub(ram.word(0x1a4b + usize::from(a))),
                ram.word(SAMUS_Y).wrapping_sub(ram.word(0x1a93 + usize::from(a))),
            )))
        }
        "CheckIfEnemyTouchesSamus" | "EnemyFunc_AC67" => {
            let e = record(a)?;
            let dx = abs16(ram.word(SAMUS_X).wrapping_sub(ram.word(e + X)));
            let outside = dx.wrapping_sub(ram.word(SAMUS_WIDTH));
            if dx >= ram.word(SAMUS_WIDTH)
                && outside >= ram.word(e + WIDTH)
                && (symbol == "CheckIfEnemyTouchesSamus" || outside >= 8)
            {
                return Ok(0);
            }
            let sy = ram.word(SAMUS_Y);
            let ey = ram.word(e + Y);
            let radius = ram.word(SAMUS_HEIGHT);
            let contact = if symbol == "CheckIfEnemyTouchesSamus" {
                let above = negative(sy.wrapping_add(3).wrapping_sub(ey));
                let gap = ey.wrapping_sub(sy.wrapping_add(3));
                above && (gap < radius || gap.wrapping_sub(radius) <= ram.word(e + HEIGHT))
            } else {
                let gap = abs16(sy.wrapping_sub(ey));
                gap < radius || gap.wrapping_sub(radius) < ram.word(e + HEIGHT)
            };
            Ok(if contact { 65535 } else { 0 })
        }
        "CheckIfEnemyIsOnScreen" | "EnemyFunc_ADA3" | "EnemyWithNormalSpritesIsOffScreen" | "IsEnemyLeavingScreen" => {
            let e = record(if symbol == "IsEnemyLeavingScreen" { a } else { ram.word(CURRENT_ENEMY) })?;
            let x = ram.word(e + X);
            let y = ram.word(e + Y);
            let camera_x = ram.word(0x911);
            let camera_y = ram.word(0x915);
            if symbol == "IsEnemyLeavingScreen" {
                let edge = ram.word(e + WIDTH).wrapping_add(x).wrapping_sub(camera_x);
                return Ok(u32::from(
                    negative(x)
                        || negative(edge)
                        || !negative(edge.wrapping_sub(256).wrapping_sub(ram.word(e + WIDTH))),
                ));
            }
            let (x_margin, top, bottom) = if symbol == "EnemyFunc_ADA3" {
                (a, a, a)
            } else if symbol == "EnemyWithNormalSpritesIsOffScreen" {
                (ram.word(e + WIDTH), 8, 0xfff8)
            } else {
                (0, 0, 0)
            };
            Ok(u32::from(
                negative(x_margin.wrapping_add(x).wrapping_sub(camera_x))
                    || negative(x_margin.wrapping_add(camera_x).wrapping_add(256).wrapping_sub(x))
                    || negative(top.wrapping_add(y).wrapping_sub(camera_y))
                    || negative(bottom.wrapping_add(camera_y).wrapping_add(256).wrapping_sub(y)),
            ))
        }
        "DetermineDirectionOfSamusFromEnemy" => {
            let e = record(ram.word(CURRENT_ENEMY))?;
            let dx = ram.word(SAMUS_X).wrapping_sub(ram.word(e + X));
            let dy = ram.word(SAMUS_Y).wrapping_sub(ram.word(e + Y));
            Ok(if negative(abs16(dy).wrapping_sub(32)) {
                if negative(dx) { 7 } else { 2 }
            } else if negative(abs16(dx).wrapping_sub(32)) {
                if negative(dy) { 0 } else { 4 }
            } else if negative(dx) {
                if negative(dy) { 8 } else { 6 }
            } else if negative(dy) {
                1
            } else {
                3
            })
        }
        "GetSamusEnemyDelta_X"
        | "GetSamusEnemyDelta_Y"
        | "IsSamusWithinEnemy_X"
        | "IsSamusWithinEnemy_Y"
        | "CompareDistToSamus_X"
        | "CompareDistToSamus_Y" => {
            let e = record(a)?;
            let horizontal = symbol.ends_with("_X");
            let delta = ram
                .word(if horizontal { SAMUS_X } else { SAMUS_Y })
                .wrapping_sub(ram.word(e + if horizontal { X } else { Y }));
            Ok(if symbol.starts_with("Get") {
                u32::from(delta)
            } else if symbol.starts_with("Is") {
                u32::from(negative(abs16(delta).wrapping_sub(b)))
            } else {
                u32::from(abs16(delta) >= b)
            })
        }
        _ => Err("unknown enemy routine"),
    }
}
