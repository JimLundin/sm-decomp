//! Terrain and solid-enemy collision from pinned snesrev/sm banks94/A0/90.
//! Upstream license: crates/decomp/LICENSE.upstream.txt.
use crate::{Wram, audio, samus, samus_control};
const SAMUS_COLLISION_DIRECTION: usize = 0xb02;
const AREA_INDEX: usize = 0x79f;
const BOSS_BITS_FOR_AREA: usize = 0xd828;
const SAMUS_INVINCIBILITY_TIMER: usize = 0x18a8;
const SAMUS_PERIODIC_DAMAGE: usize = 0xa50;
const SAMUS_POSE_X_DIR: usize = 0xa1e;
const DOOR_LIST_POINTER: usize = 0x7b5;
const SAMUS_POSE: usize = 0xa1c;
const ENABLE_HORIZ_SLOPE_COLL: usize = 0xa46;
const INTERACTIVE_ENEMY_INDEXES_WRITE_PTR: usize = 0x17a6;
const INTERACTIVE_ENEMY_INDEXES: usize = 0x17ec;
const SAMUS_Y_DIR: usize = 0xb36;
const SAMUS_Y_SPEED: usize = 0xb2e;
const SAMUS_POS_ADJUSTED_BY_SLOPE_FLAG: usize = 0xdba;
const SAMUS_X_ACCEL_MODE: usize = 0xb4a;
const JOYPAD1_LASTKEYS: usize = 0x8b;
const KNOCKBACK_DIR: usize = 0xa52;
const BUTTON_CONFIG_JUMP_A: usize = 0x9b4;
const JOYPAD1_NEWKEYS: usize = 0x8f;
const COLLISION_DETECTION_INDEX: usize = 0x18a6;
const SAMUS_LAST_DIFFERENT_POSE_MOVEMENT_TYPE: usize = 0xa27;
const SAMUS_ANIM_FRAME: usize = 0xa96;
const ROOM_INDEX: usize = 0x79d;
const CINEMATIC_FUNCTION: usize = 0x1f51;
const FX_TYPE: usize = 0x196e;
const SPEED_BOOST_COUNTER: usize = 0xb3e;
const FX_Y_POS: usize = 0x195e;
const LAVA_ACID_Y_POS: usize = 0x1962;
const FX_LIQUID_OPTIONS: usize = 0x197e;
const BUTTON_CONFIG_SHOOT_X: usize = 0x9b2;
const ELEVATOR_STATUS: usize = 0xe18;
const SAMUS_ANIM_FRAME_TIMER: usize = 0xa94;
const BOSS_ID: usize = 0x179c;
const SAMUS_SHINE_TIMER: usize = 0xa68;
const FLARE_COUNTER: usize = 0xcd0;
const SAMUS_SUIT_PALETTE_INDEX: usize = 0xa74;
const EQUIPPED_ITEMS: usize = 0x9a2;
const LIQUID_PHYSICS_TYPE: usize = 0xad2;
const USED_FOR_BALL_BOUNCE_ON_LANDING: usize = 0xb20;
const INPUT_TO_POSE_CALC: usize = 0xdc6;
const SAMUS_VAR62: usize = 0xa62;
const KNOCKBACK_X_DIR: usize = 0xa54;
const BOMB_JUMP_DIR: usize = 0xa56;
const SAMUS_INPUT_HANDLER: usize = 0xa60;
const TIME_IS_FROZEN_FLAG: usize = 0xa78;
const SAMUS_MOVEMENT_TYPE: usize = 0xa1f;
const NMI_FRAME_COUNTER_WORD: usize = 0x5b6;
const FLAG_SAMUS_IN_QUICKSAND: usize = 0x1e71;
const CURRENT_SLOPE_BTS: usize = 0x1e77;
const SAMUS_KNOCKBACK_TIMER: usize = 0x18aa;
const DOOR_TRANSITION_FUNCTION: usize = 0x99c;
const ELEVATOR_FLAGS: usize = 0xe16;
const DOOR_DEF_PTR: usize = 0x78d;
const GAME_STATE: usize = 0x998;
const ENEMY_INDEX_COLLIDING_DIRS: usize = 0x182c;
const SAMUS_Y_SUBSPEED: usize = 0xb2c;
const SAMUS_X_BASE_SPEED: usize = 0xb46;
const SAMUS_X_BASE_SUBSPEED: usize = 0xb48;
const ENEMY_INDEX_TO_SHAKE: usize = 0xe1c;
const ATMOSPHERIC_GFX_X_POS: usize = 0xadc;
const ATMOSPHERIC_GFX_Y_POS: usize = 0xae4;
const ATMOSPHERIC_GFX_FRAME_AND_TYPE: usize = 0xaec;
const ATMOSPHERIC_GFX_ANIM_TIMER: usize = 0xad4;
const SAMUS_X_EXTRA_RUN_SPEED: usize = 0xb42;
const SAMUS_X_EXTRA_RUN_SUBSPEED: usize = 0xb44;
const SAMUS_CONTACT_DAMAGE_INDEX: usize = 0xa6e;
const UNUSED_WORD_7E0DFA: usize = 0xdfa;
const UNUSEDWORD_7E0AA4: usize = 0xaa4;
const SAMUS_MOVEMENT_HANDLER: usize = 0xa58;
const SAMUS_Y_ACCEL: usize = 0xb34;
const SAMUS_Y_SUBACCEL: usize = 0xb32;
const EXTRA_SAMUS_Y_DISPLACEMENT: usize = 0xb5c;
const EXTRA_SAMUS_Y_SUBDISPLACEMENT: usize = 0xb5a;
const SAMUS_TOTAL_X_SPEED: usize = 0xdbc;
const SAMUS_TOTAL_X_SUBSPEED: usize = 0xdbe;
const PROJECTILE_INIT_SPEED_SAMUS_MOVED_LEFT: usize = 0xdaa;
const PROJECTILE_INIT_SPEED_SAMUS_MOVED_LEFT_FRACT: usize = 0xdac;
const PROJECTILE_INIT_SPEED_SAMUS_MOVED_RIGHT: usize = 0xdae;
const PROJECTILE_INIT_SPEED_SAMUS_MOVED_RIGHT_FRACT: usize = 0xdb0;
const PROJECTILE_INIT_SPEED_SAMUS_MOVED_UP: usize = 0xdb2;
const PROJECTILE_INIT_SPEED_SAMUS_MOVED_UP_FRACT: usize = 0xdb4;
const PROJECTILE_INIT_SPEED_SAMUS_MOVED_DOWN: usize = 0xdb6;
const PROJECTILE_INIT_SPEED_SAMUS_MOVED_DOWN_FRACT: usize = 0xdb8;

/// PLM setup is synchronous and can mutate the current fixed-point displacement.
/// Adapters transfer an independent full WRAM snapshot; never alias a Rust slice
/// with the C host's canonical RAM during callback execution.
pub trait Host {
    fn spawn_plm(&mut self, ram: &mut Wram<'_>, header: u16, amount: &mut i32) -> Result<u8, &'static str>;
}
const SLOPE_HEIGHT: [u8; 512] = [
    8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 16, 16, 16, 16, 16, 16, 16, 16, 0, 0, 0, 0, 0, 0, 0, 0, 16, 16, 16,
    16, 16, 16, 16, 16, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 16, 15, 14, 13, 12, 11, 10, 9, 9, 10, 11, 12, 13, 14, 15, 16, 16, 14, 12, 10, 8, 6, 4, 2,
    2, 4, 6, 8, 10, 12, 14, 16, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 12, 12, 12, 12, 8, 8, 8, 8, 4, 4, 4, 4, 0, 0, 0, 0, 14, 14, 12, 12, 10, 10, 8, 8, 6, 6, 4, 4,
    2, 2, 0, 0, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20,
    20, 20, 20, 16, 16, 16, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 16, 16, 16, 16, 16, 16, 16, 16, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 0, 0, 0,
    0, 0, 0, 16, 16, 15, 15, 14, 14, 13, 13, 12, 12, 11, 11, 10, 10, 9, 9, 8, 8, 7, 7, 6, 6, 5, 5, 4, 4, 3, 3, 2, 2, 1,
    1, 16, 16, 16, 15, 15, 15, 14, 14, 14, 13, 13, 13, 12, 12, 12, 11, 11, 11, 10, 10, 10, 9, 9, 9, 8, 8, 8, 7, 7, 7,
    6, 6, 6, 5, 5, 5, 4, 4, 4, 3, 3, 3, 2, 2, 2, 1, 1, 1, 20, 20, 20, 20, 20, 20, 20, 20, 16, 14, 12, 10, 8, 6, 4, 2,
    16, 14, 12, 10, 8, 6, 4, 2, 0, 0, 0, 0, 0, 0, 0, 0, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 15, 12, 9, 6, 3,
    20, 20, 20, 20, 20, 20, 14, 11, 8, 5, 2, 0, 0, 0, 0, 0, 16, 13, 10, 7, 4, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
const SQUARE_MASK: [u8; 20] =
    [0, 0, 0x80, 0x80, 0, 0x80, 0, 0x80, 0, 0, 0, 0x80, 0, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80];
const SLOPE_SPEED: [u16; 64] = [
    0, 0x100, 0, 0x100, 0, 0x100, 0, 0x100, 0, 0x100, 0, 0x100, 0, 0x100, 0, 0x100, 0, 0x100, 0, 0x100, 0, 0x100, 0,
    0x100, 0, 0x100, 0, 0x100, 0x1000, 0xb0, 0x1000, 0xb0, 0, 0x100, 0, 0x100, 0x1000, 0xc0, 0, 0x100, 0x1000, 0xc0,
    0x1000, 0xc0, 0x800, 0xd8, 0x800, 0xd8, 0x600, 0xf0, 0x600, 0xf0, 0x600, 0xf0, 0x4000, 0x80, 0x4000, 0x80, 0x6000,
    0x50, 0x6000, 0x50, 0x6000, 0x50,
];
const X: usize = 0xaf6;
const XF: usize = 0xaf8;
const Y: usize = 0xafa;
const YF: usize = 0xafc;
const XR: usize = 0xafe;
const YR: usize = 0xb00;
const WIDTH: usize = 0x7a5;
const BLOCK: usize = 0xdc4;
const LEVEL: usize = 0x10002;
const BTS: usize = 0x16402;
const COLLISION: usize = 0xdd0;
fn pair(ram: &Wram<'_>, hi: usize, lo: usize) -> u32 {
    (u32::from(ram.word(hi)) << 16) | u32::from(ram.word(lo))
}
fn add(ram: &mut Wram<'_>, hi: usize, lo: usize, amount: i32) {
    let v = pair(ram, hi, lo).wrapping_add(amount as u32);
    ram.set_word(hi, (v >> 16) as u16);
    ram.set_word(lo, v as u16);
}
#[derive(Clone, Copy)]
struct Collision {
    amount: i32,
    position: u16,
    remaining: u16,
    span: u16,
    edge: u16,
}
fn solid(ram: &mut Wram<'_>, ci: &mut Collision, vertical: bool, square: bool) -> bool {
    let (pos, fraction, radius) = if vertical { (Y, YF, YR) } else { (X, XF, XR) };
    let mask = if square { 7 } else { 15 };
    let backwards = ci.amount < 0;
    let delta = if backwards {
        ram.word(radius).wrapping_add(ci.edge | mask).wrapping_add(1).wrapping_sub(ram.word(pos)) as i16
    } else {
        (ci.edge & !mask).wrapping_sub(ram.word(radius)).wrapping_sub(ram.word(pos)) as i16
    };
    ci.amount = i32::from(if backwards { delta.min(0) } else { delta.max(0) }) << 16;
    ram.set_word(fraction, if backwards { 0 } else { 0xffff });
    true
}
fn rom_word(rom: &[u8], address: u32) -> Result<u16, &'static str> {
    let o = (((address >> 16) << 15) | (address & 0x7fff)) as usize & 0x3fffff;
    let b = rom.get(o..o + 2).ok_or("ROM collision table outside image")?;
    Ok(u16::from_le_bytes([b[0], b[1]]))
}
fn slope(ram: &mut Wram<'_>, ci: &mut Collision, vertical: bool, bts: u8) -> Result<bool, &'static str> {
    let shape = usize::from(bts & 31);
    if shape < 5 {
        let slot = shape * 4 + (usize::from(bts >> 6) ^ usize::from((ci.edge & 8) >> if vertical { 2 } else { 3 }));
        let (pos, radius) = if vertical { (X, XR) } else { (Y, YR) };
        let other = slot ^ if vertical { 1 } else { 2 };
        let occupied = if ci.remaining == 0 {
            if ram.word(radius).wrapping_add(ram.word(pos)).wrapping_sub(1) & 8 == 0 {
                SQUARE_MASK[slot] != 0
            } else {
                SQUARE_MASK[slot] != 0 || SQUARE_MASK[other] != 0
            }
        } else {
            ((ci.remaining != ci.span || ram.word(pos).wrapping_sub(ram.word(radius)) & 8 == 0)
                && SQUARE_MASK[slot] != 0)
                || SQUARE_MASK[other] != 0
        };
        if occupied {
            let backwards = ci.amount < 0;
            solid(ram, ci, vertical, true);
            if vertical && !backwards {
                ram.set_word(SAMUS_POS_ADJUSTED_BY_SLOPE_FLAG, 1);
            }
            return Ok(true);
        }
        return Ok(false);
    }
    ram.set_word(CURRENT_SLOPE_BTS, u16::from(bts));
    if !vertical {
        if bts & 0x80 != 0 || pair(ram, SAMUS_Y_SPEED, SAMUS_Y_SUBSPEED) != 0 {
            return Ok(false);
        }
        let fraction = (ci.amount >> 8) as u16;
        ci.amount = if ci.amount >= 0 {
            u32::from(fraction).wrapping_mul(u32::from(SLOPE_SPEED[shape * 2 + 1])) as i32
        } else {
            u32::from(fraction.wrapping_neg()).wrapping_mul(u32::from(SLOPE_SPEED[shape * 2 + 1])).wrapping_neg() as i32
        };
        return Ok(false);
    }
    if ram.word(WIDTH) == 0 {
        return Err("invalid room width");
    }
    // SnesModulus takes an eight-bit divisor.
    if ram.word(X) >> 4 != ram.word(BLOCK) % u16::from(ram.word(WIDTH) as u8) {
        return Ok(false);
    }
    let down = ram.word(SAMUS_COLLISION_DIRECTION) & 1 != 0;
    if (bts & 0x80 != 0) == down {
        return Ok(false);
    }
    let x = if bts & 0x40 != 0 { ram.word(X) ^ 15 } else { ram.word(X) };
    let height = u16::from(SLOPE_HEIGHT[shape * 16 + usize::from(x & 15)] & 31);
    let inside = if down {
        ram.word(YR).wrapping_add(ci.position).wrapping_sub(1) & 15
    } else {
        ci.position.wrapping_sub(ram.word(YR)) & 15 ^ 15
    };
    let overlap = height.wrapping_sub(inside).wrapping_sub(1) as i16;
    if overlap <= 0 {
        let delta = ((ci.amount >> 16) as i16).wrapping_add(overlap).max(0);
        ci.amount = i32::from(delta) << 16;
        Ok(true)
    } else {
        Ok(false)
    }
}
fn spike(ram: &mut Wram<'_>, bts: u8) -> Result<(), &'static str> {
    if bts >= 16 {
        return Err("invalid spike BTS");
    }
    let area = ram.word(AREA_INDEX);
    if area > 7 {
        return Err("invalid area");
    }
    let damage = match bts {
        0 if area != 3 || ram.byte(BOSS_BITS_FOR_AREA + usize::from(area)) & 1 != 0 => 60,
        1 | 3 => 16,
        _ => 0,
    };
    if damage != 0 && ram.word(SAMUS_INVINCIBILITY_TIMER) == 0 {
        ram.set_word(SAMUS_INVINCIBILITY_TIMER, 60);
        ram.set_word(SAMUS_KNOCKBACK_TIMER, 10);
        ram.set_word(SAMUS_PERIODIC_DAMAGE, ram.word(SAMUS_PERIODIC_DAMAGE).wrapping_add(damage));
        ram.set_word(KNOCKBACK_X_DIR, u16::from((ram.byte(SAMUS_POSE_X_DIR) ^ 12) & 8 != 0));
    }
    Ok(())
}
fn reaction(
    ram: &mut Wram<'_>,
    rom: &[u8],
    host: &mut impl Host,
    ci: &mut Collision,
    vertical: bool,
    index: u16,
) -> Result<bool, &'static str> {
    ram.set_word(BLOCK, index);
    for _ in 0..256 {
        let index = ram.word(BLOCK);
        if usize::from(index) >= 0x3200 {
            return Err("tile outside canonical level storage");
        }
        let bts = ram.byte(BTS + usize::from(index));
        let kind = ram.word(LEVEL + usize::from(index) * 2) >> 12;
        match kind {
            0 | 2 | 4 | 6 => return Ok(false),
            1 => return slope(ram, ci, vertical, bts),
            10 => {
                spike(ram, bts)?;
                return Ok(solid(ram, ci, vertical, false));
            }
            9 => {
                ram.set_word(DOOR_TRANSITION_FUNCTION, 0xe17d);
                let header = rom_word(
                    rom,
                    0x8f0000 | u32::from(ram.word(DOOR_LIST_POINTER).wrapping_add(u16::from(bts & 0x7f) * 2)),
                )?;
                if rom_word(rom, 0x830000 | u32::from(header))? & 0x8000 == 0 {
                    if ram.word(SAMUS_POSE) < 9 {
                        ram.set_word(ELEVATOR_FLAGS, 1);
                    }
                    return Ok(solid(ram, ci, vertical, false));
                }
                ram.set_word(DOOR_DEF_PTR, header);
                ram.set_word(GAME_STATE, 9);
                return Ok(false);
            }
            8 | 12 | 14 => return Ok(solid(ram, ci, vertical, false)),
            5 | 13 => {
                if bts == 0 {
                    return Ok(false);
                }
                let delta = i16::from(bts as i8).wrapping_mul(if kind == 13 { ram.word(WIDTH) as i16 } else { 1 });
                ram.set_word(BLOCK, index.wrapping_add(delta as u16));
            }
            3 | 7 | 11 | 15 => {
                if (kind == 7 || kind == 15) && bts & 0x80 != 0 {
                    return Ok(if kind == 15 { solid(ram, ci, vertical, false) } else { false });
                }
                let pointer = if kind == 7 || kind == 15 {
                    rom_word(rom, 0x94936b + u32::from(bts) * 2)?
                } else if bts & 0x80 != 0 {
                    let area = ram.word(AREA_INDEX);
                    if area > 7 {
                        return Err("invalid area index");
                    }
                    let table = rom_word(rom, (if kind == 11 { 0x9492e9 } else { 0x9492d9 }) + u32::from(area) * 2)?;
                    rom_word(rom, 0x940000 | u32::from(table.wrapping_add(u16::from(bts & 0x7f) * 2)))?
                } else {
                    rom_word(rom, 0x949139 + u32::from(bts) * 2)?
                };
                let result = host.spawn_plm(ram, pointer, &mut ci.amount)? & 1;
                if kind == 11 || kind == 15 {
                    return Ok(result != 0 && solid(ram, ci, vertical, false));
                }
                return Ok(if kind == 3 && bts & 0x80 != 0 { result != 0 } else { false });
            }
            _ => return Err("collision reaction pending translation"),
        }
    }
    Err("unterminated block extension")
}
fn span(ram: &Wram<'_>, vertical: bool) -> u16 {
    let (pos, radius) = if vertical { (X, XR) } else { (Y, YR) };
    let start = ram.word(pos).wrapping_sub(ram.word(radius)) & 0xfff0;
    ram.word(radius).wrapping_add(ram.word(pos)).wrapping_sub(1).wrapping_sub(start) >> 4
}
fn traverse(
    ram: &mut Wram<'_>,
    rom: &[u8],
    host: &mut impl Host,
    amount: i32,
    vertical: bool,
    reverse: bool,
) -> Result<(bool, i32), &'static str> {
    let width = ram.word(WIDTH);
    if width == 0 || width > 255 {
        return Err("invalid room width");
    }
    let count = span(ram, vertical);
    if count > 31 {
        return Err("invalid collision span");
    }
    let (pos, frac, radius) = if vertical { (Y, YF, YR) } else { (X, XF, XR) };
    let next = (pair(ram, pos, frac).wrapping_add(amount as u32) >> 16) as u16;
    let edge = if amount < 0 {
        next.wrapping_sub(ram.word(radius))
    } else {
        ram.word(radius).wrapping_add(next).wrapping_sub(1)
    };
    let product = if vertical {
        u16::from((edge >> 4) as u8) * u16::from(width as u8)
    } else {
        u16::from((ram.word(Y).wrapping_sub(ram.word(YR)) >> 4) as u8) * u16::from(width as u8)
    };
    let column = if vertical {
        (ram.word(X).wrapping_sub(ram.word(XR)) >> 4).wrapping_add(if reverse { count } else { 0 })
    } else {
        edge >> 4
    };
    let mut index = product.wrapping_add(column);
    let mut ci = Collision { amount, position: next, remaining: if reverse { 0 } else { count }, span: count, edge };
    for n in 0..=count {
        if reaction(ram, rom, host, &mut ci, vertical, index)? {
            return Ok((true, ci.amount));
        }
        index = if vertical {
            if reverse { index.wrapping_sub(1) } else { index.wrapping_add(1) }
        } else {
            index.wrapping_add(width)
        };
        ci.remaining = if reverse { n + 1 } else { count.wrapping_sub(n + 1) };
    }
    Ok((false, ci.amount))
}
fn calculate_block(ram: &mut Wram<'_>, x: u16, y: u16) {
    if x >= 4096 || y >= 4096 {
        ram.set_word(BLOCK, 0xffff);
        return;
    }
    let product = u16::from((y >> 4) as u8) * u16::from(ram.word(WIDTH) as u8);
    ram.set_word(BLOCK, product.wrapping_add(x >> 4));
}
fn align_slope(ram: &mut Wram<'_>) -> Result<(), &'static str> {
    if ram.word(ENABLE_HORIZ_SLOPE_COLL) & 2 == 0 {
        return Ok(());
    }
    for top in [false, true] {
        let edge = if top {
            ram.word(Y).wrapping_sub(ram.word(YR))
        } else {
            ram.word(YR).wrapping_add(ram.word(Y)).wrapping_sub(1)
        };
        calculate_block(ram, ram.word(X), edge);
        let index = usize::from(ram.word(BLOCK));
        if index >= 0x3200 {
            return Err("tile outside canonical level storage");
        }
        let bts = ram.byte(BTS + index);
        if ram.word(LEVEL + index * 2) & 0xf000 == 0x1000 && bts & 31 >= 5 && (bts & 0x80 != 0) == top {
            let x = if bts & 0x40 != 0 { ram.word(X) ^ 15 } else { ram.word(X) };
            let height = u16::from(SLOPE_HEIGHT[usize::from(bts & 31) * 16 + usize::from(x & 15)] & 31);
            let overlap = height.wrapping_sub(if top { edge & 15 ^ 15 } else { edge & 15 }).wrapping_sub(1) as i16;
            if if top { overlap <= 0 } else { overlap < 0 } {
                ram.set_word(
                    Y,
                    if top {
                        ram.word(Y).wrapping_sub(overlap as u16)
                    } else {
                        ram.word(Y).wrapping_add(overlap as u16)
                    },
                );
                ram.set_word(SAMUS_POS_ADJUSTED_BY_SLOPE_FLAG, 1);
            }
        }
    }
    Ok(())
}
/// Canonical solid-enemy collision result: flag and unsigned-distance bits.
pub fn check_solid_enemy(ram: &mut Wram<'_>, amount: i32) -> Result<(bool, i32), &'static str> {
    if ram.word(INTERACTIVE_ENEMY_INDEXES_WRITE_PTR) == 0 {
        return Ok((false, amount));
    }
    let direction = ram.word(SAMUS_COLLISION_DIRECTION) & 3;
    let (hi, lo) = if direction < 2 { (X, XF) } else { (Y, YF) };
    let integer = (amount >> 16) as u16;
    let fraction = amount as u16;
    let mut future = ram.word(hi);
    if direction & 1 == 0 {
        future = future.wrapping_sub(integer);
        let mut equal = ram.word(lo) == fraction;
        if ram.word(lo) < fraction {
            equal = future == 1;
            future = future.wrapping_sub(1);
        }
        if !equal {
            future = future.wrapping_sub(1);
        }
    } else {
        future = future.wrapping_add(integer);
        let sum = u32::from(ram.word(lo)) + u32::from(fraction);
        let mut zero = sum == 0;
        if sum > 0xffff {
            zero = future == 0xffff;
            future = future.wrapping_add(1);
        }
        if !zero {
            future = future.wrapping_add(1);
        }
    }
    let px = if direction < 2 { future } else { ram.word(X) };
    let py = if direction >= 2 { future } else { ram.word(Y) };
    ram.set_word(COLLISION_DETECTION_INDEX, 0);
    for n in 0..32 {
        let index = ram.word(INTERACTIVE_ENEMY_INDEXES + n * 2);
        if index == 0xffff {
            return Ok((false, amount));
        }
        if index > 31 * 64 || !index.is_multiple_of(64) {
            return Err("invalid interactive enemy slot");
        }
        ram.set_word(COLLISION_DETECTION_INDEX, index);
        let e = 0xf78 + usize::from(index);
        if ram.word(e + 38) == 0 && ram.word(e + 14) & 0x8000 == 0 {
            continue;
        }
        let ex = ram.word(e + 2);
        let ey = ram.word(e + 6);
        let ew = ram.word(e + 10);
        let eh = ram.word(e + 12);
        let dx = (ex.wrapping_sub(px) as i16).wrapping_abs() as u16;
        let dy = (ey.wrapping_sub(py) as i16).wrapping_abs() as u16;
        if (dx < ew || dx.wrapping_sub(ew) < ram.word(XR)) && (dy < eh || dy.wrapping_sub(eh) < ram.word(YR)) {
            let delta = match direction {
                0 => ram.word(X).wrapping_sub(ram.word(XR)).wrapping_sub(ex.wrapping_add(ew)),
                1 => ex.wrapping_sub(ew).wrapping_sub(ram.word(XR).wrapping_add(ram.word(X))),
                2 => ram.word(Y).wrapping_sub(ram.word(YR)).wrapping_sub(ey.wrapping_add(eh)),
                _ => ey.wrapping_sub(eh).wrapping_sub(ram.word(YR).wrapping_add(ram.word(Y))),
            } as i16;
            if delta >= 0 {
                if delta == 0 {
                    ram.set_word(YF, 0);
                }
                ram.set_word(ENEMY_INDEX_COLLIDING_DIRS + usize::from(direction) * 2, index);
                return Ok((true, i32::from(delta) << 16));
            }
        }
    }
    Err("unterminated interactive enemy list")
}
fn movement(
    ram: &mut Wram<'_>,
    rom: &[u8],
    host: &mut impl Host,
    direction: u16,
    amount: i32,
) -> Result<(), &'static str> {
    if amount == i32::MIN {
        return Err("signed movement amount outside admitted domain");
    }
    if direction >= 2 {
        ram.set_word(SAMUS_COLLISION_DIRECTION, direction);
    }
    let distance = if direction & 1 == 0 { amount.wrapping_neg() } else { amount };
    let (hit, allowed) = check_solid_enemy(ram, distance)?;
    ram.set_word(COLLISION, if hit { 0xffff } else { 0 });
    if direction < 2 {
        if hit {
            samus_control::dispatch_rom(ram, rom, "Samus_ClearXSpeedIfColl", 0, 0)?;
            samus::dispatch_rom(
                ram,
                rom,
                if direction == 0 { "Samus_MoveLeft_NoColl" } else { "Samus_MoveRight_NoColl" },
                allowed,
                0,
            )?;
        } else {
            let delta = dispatch_host(
                ram,
                rom,
                host,
                "Samus_MoveRight_NoSolidColl",
                if direction == 0 { allowed.wrapping_neg() } else { allowed },
                0,
            )?;
            let (hi, lo) = if direction == 0 {
                (PROJECTILE_INIT_SPEED_SAMUS_MOVED_LEFT, PROJECTILE_INIT_SPEED_SAMUS_MOVED_LEFT_FRACT)
            } else {
                (PROJECTILE_INIT_SPEED_SAMUS_MOVED_RIGHT, PROJECTILE_INIT_SPEED_SAMUS_MOVED_RIGHT_FRACT)
            };
            ram.set_word(hi, (delta >> 16) as u16);
            ram.set_word(lo, delta as u16);
            if ram.word(SAMUS_COLLISION_DIRECTION) & 1 != direction {
                ram.set_word(COLLISION, 0);
            }
            samus_control::dispatch_rom(ram, rom, "Samus_ClearXSpeedIfColl", 0, 0)?;
        }
        align_slope(ram)?;
    } else {
        let routine = if direction == 2 { "Samus_MoveUp_SetPoseCalcInput" } else { "Samus_MoveDown_SetPoseCalcInput" };
        if hit {
            samus_control::dispatch_rom(ram, rom, routine, 0, 0)?;
            samus::dispatch_rom(
                ram,
                rom,
                if direction == 2 { "Samus_MoveUp_NoColl" } else { "Samus_MoveDown_NoColl" },
                allowed,
                0,
            )?;
        } else {
            let delta = dispatch_host(
                ram,
                rom,
                host,
                "Samus_MoveDown_NoSolidColl",
                if direction == 2 { allowed.wrapping_neg() } else { allowed },
                0,
            )?;
            let (hi, lo) = if direction == 2 {
                (PROJECTILE_INIT_SPEED_SAMUS_MOVED_UP, PROJECTILE_INIT_SPEED_SAMUS_MOVED_UP_FRACT)
            } else {
                (PROJECTILE_INIT_SPEED_SAMUS_MOVED_DOWN, PROJECTILE_INIT_SPEED_SAMUS_MOVED_DOWN_FRACT)
            };
            ram.set_word(hi, (delta >> 16) as u16);
            ram.set_word(lo, delta as u16);
            samus_control::dispatch_rom(ram, rom, routine, 0, 0)?;
        }
    }
    Ok(())
}
fn move_y(
    ram: &mut Wram<'_>,
    rom: &[u8],
    host: &mut impl Host,
    speed: bool,
    extra_only: bool,
) -> Result<(), &'static str> {
    let extra = pair(ram, EXTRA_SAMUS_Y_DISPLACEMENT, EXTRA_SAMUS_Y_SUBDISPLACEMENT) as i32;
    let amount = if speed {
        let old = pair(ram, SAMUS_Y_SPEED, SAMUS_Y_SUBSPEED);
        let accel = pair(ram, SAMUS_Y_ACCEL, SAMUS_Y_SUBACCEL);
        let down = ram.word(SAMUS_Y_DIR) == 2;
        let new = if down {
            if ram.word(SAMUS_Y_SPEED) != 5 { old.wrapping_add(accel) } else { old }
        } else {
            old.wrapping_sub(accel)
        };
        ram.set_word(SAMUS_Y_SPEED, (new >> 16) as u16);
        ram.set_word(SAMUS_Y_SUBSPEED, new as u16);
        (if down { old } else { old.wrapping_neg() }).wrapping_add(extra as u32) as i32
    } else {
        if extra == 0 && extra_only {
            return Ok(());
        }
        let amount = if extra == 0 && ram.word(SAMUS_POS_ADJUSTED_BY_SLOPE_FLAG) == 0 {
            pair(ram, SAMUS_TOTAL_X_SPEED, SAMUS_TOTAL_X_SUBSPEED) as i32
        } else {
            extra
        };
        if amount >= 0 { amount.wrapping_add(0x10000) } else { amount }
    };
    movement(ram, rom, host, if amount < 0 { 2 } else { 3 }, amount)
}
fn air_base_speed(ram: &mut Wram<'_>, rom: &[u8]) -> Result<(bool, i32), &'static str> {
    let pointer = samus::dispatch_rom(ram, rom, "Samus_DetermineSpeedTableEntryPtr_X", 0, 0)?;
    let mode = ram.word(SAMUS_X_ACCEL_MODE);
    let accel =
        (u32::from(rom_word(rom, 0x900000 | pointer)?) << 16) | u32::from(rom_word(rom, 0x900000 | pointer + 2)?);
    let tentative = pair(ram, SAMUS_X_BASE_SPEED, SAMUS_X_BASE_SUBSPEED).wrapping_add(accel);
    let max_hi = rom_word(rom, 0x900000 | pointer + 4)?;
    let max_lo = rom_word(rom, 0x900000 | pointer + 6)?;
    let hi = (tentative >> 16) as u16;
    let lo = tentative as u16;
    let clamped = mode & 1 == 0
        && (hi.wrapping_sub(max_hi) as i16) >= 0
        && (hi != max_hi || ((lo.wrapping_sub(max_lo) as i16) >= 0 && lo != max_lo));
    // NoDecel selects deceleration by bit0, unlike ordinary base-speed mode.
    ram.set_word(SAMUS_X_ACCEL_MODE, mode & 1);
    let result = samus::dispatch_rom(ram, rom, "Samus_CalcBaseSpeed_X", pointer as i32, 0);
    let stopped =
        ram.word(SAMUS_X_ACCEL_MODE) == 0 && mode & 1 != 0 && pair(ram, SAMUS_X_BASE_SPEED, SAMUS_X_BASE_SUBSPEED) == 0;
    ram.set_word(SAMUS_X_ACCEL_MODE, if stopped { 0 } else { mode });
    Ok((clamped, result? as i32))
}
fn clear_base(ram: &mut Wram<'_>) {
    ram.set_word(SAMUS_X_BASE_SPEED, 0);
    ram.set_word(SAMUS_X_BASE_SUBSPEED, 0);
    ram.set_word(COLLISION, 0);
}
fn fall(
    ram: &mut Wram<'_>,
    rom: &[u8],
    host: &mut impl Host,
    morphed: bool,
    bouncing: bool,
) -> Result<(), &'static str> {
    if !morphed {
        samus::dispatch_rom(ram, rom, "Samus_HandleExtraRunspeedX", 0, 0)?;
    }
    let (_, mut amount) = air_base_speed(ram, rom)?;
    let moving = ram.word(SAMUS_X_ACCEL_MODE) != 0 || ram.word(JOYPAD1_LASTKEYS) & 0x300 != 0;
    if !moving {
        clear_base(ram);
        amount = 0;
    }
    if moving || morphed {
        dispatch_host(ram, rom, host, "Samus_MoveX", amount, 0)?;
    }
    if bouncing && ram.word(KNOCKBACK_DIR) != 0 {
        return Ok(());
    }
    if bouncing && pair(ram, EXTRA_SAMUS_Y_DISPLACEMENT, EXTRA_SAMUS_Y_SUBDISPLACEMENT) != 0 {
        ram.set_word(SAMUS_Y_DIR, 2);
        ram.set_word(SAMUS_Y_SUBSPEED, 0);
        ram.set_word(SAMUS_Y_SPEED, 0);
        move_y(ram, rom, host, false, true)
    } else {
        samus::dispatch_rom(ram, rom, "Samus_CheckStartFalling", 0, 0)?;
        move_y(ram, rom, host, true, false)
    }
}
fn wall_jump(
    ram: &mut Wram<'_>,
    rom: &[u8],
    host: &mut impl Host,
    amount: i32,
    grapple: bool,
) -> Result<u32, &'static str> {
    if grapple {
        ram.set_word(ENEMY_INDEX_TO_SHAKE, 0xffff);
        let facing = ram.byte(SAMUS_POSE_X_DIR);
        if facing != 4 && facing != 8 {
            return Ok(0);
        }
        ram.set_word(SAMUS_COLLISION_DIRECTION, if facing == 4 { 0 } else { 1 });
        let (hit, delta) = check_solid_enemy(ram, amount)?;
        if !hit {
            dispatch_host(ram, rom, host, "WallJumpBlockCollDetect", delta, 0)?;
            return Ok(u32::from(
                ram.word(COLLISION) != 0 && ram.word(BUTTON_CONFIG_JUMP_A) & ram.word(JOYPAD1_NEWKEYS) != 0,
            ));
        }
        if ram.word(BUTTON_CONFIG_JUMP_A) & ram.word(JOYPAD1_NEWKEYS) == 0 {
            return Ok(0);
        }
        ram.set_word(ENEMY_INDEX_TO_SHAKE, ram.word(COLLISION_DETECTION_INDEX));
        return Ok(1);
    }
    if ![3, 20].contains(&ram.byte(SAMUS_LAST_DIFFERENT_POSE_MOVEMENT_TYPE)) {
        return Ok(0);
    }
    let screw = [0x81, 0x82].contains(&ram.word(SAMUS_POSE));
    let early = (ram.word(SAMUS_ANIM_FRAME).wrapping_sub(if screw { 27 } else { 11 }) as i16) < 0;
    let left = ram.word(JOYPAD1_LASTKEYS) & 0x200 != 0;
    let right = ram.word(JOYPAD1_LASTKEYS) & 0x100 != 0;
    if !left && !right {
        return Ok(0);
    }
    ram.set_word(SAMUS_COLLISION_DIRECTION, if left { 1 } else { 0 });
    let (hit, delta) = check_solid_enemy(ram, amount)?;
    if early {
        if !hit {
            dispatch_host(
                ram,
                rom,
                host,
                "WallJumpBlockCollDetect",
                if left { delta } else { delta.wrapping_neg() },
                0,
            )?;
            if ram.word(COLLISION) == 0 {
                return Ok(0);
            }
        }
        ram.set_word(SAMUS_ANIM_FRAME_TIMER, 1);
        ram.set_word(SAMUS_ANIM_FRAME, if screw { 26 } else { 10 });
        return Ok(0);
    }
    ram.set_word(ENEMY_INDEX_TO_SHAKE, 0xffff);
    if !hit {
        let distance = dispatch_host(
            ram,
            rom,
            host,
            "WallJumpBlockCollDetect",
            if left { delta } else { delta.wrapping_neg() },
            0,
        )? as i32;
        if ram.word(COLLISION) == 0 || ram.word(BUTTON_CONFIG_JUMP_A) & ram.word(JOYPAD1_NEWKEYS) == 0 {
            return Ok(0);
        }
        if (((if left { distance } else { delta.wrapping_neg() }) >> 16).wrapping_sub(8) as i16) < 0 {
            ram.set_word(INPUT_TO_POSE_CALC, 5);
            return Ok(1);
        }
    } else if ram.word(BUTTON_CONFIG_JUMP_A) & ram.word(JOYPAD1_NEWKEYS) != 0
        && ((delta >> 16).wrapping_sub(8) as i16) < 0
    {
        ram.set_word(INPUT_TO_POSE_CALC, 5);
        ram.set_word(ENEMY_INDEX_TO_SHAKE, ram.word(COLLISION_DETECTION_INDEX));
        return Ok(1);
    }
    Ok(0)
}
fn footstep(ram: &mut Wram<'_>, rom: &[u8], kind: u16) -> Result<(), &'static str> {
    let area = ram.word(AREA_INDEX);
    if area > 7 {
        return Err("invalid footstep area");
    }
    let splash = if kind == 2 {
        true
    } else if kind == 1 {
        false
    } else if area == 4 {
        true
    } else if area == 0 {
        const ROOMS: [u8; 16] = [1, 0, 0, 0, 0, 2, 0, 4, 0, 4, 4, 4, 4, 0, 4, 0];
        let room = ram.word(ROOM_INDEX);
        if ram.word(CINEMATIC_FUNCTION) != 0 || room >= 16 {
            false
        } else {
            let flags = ROOMS[usize::from(room)];
            if flags & 1 != 0 {
                ram.word(FX_TYPE) == 10
            } else if flags & 2 != 0 {
                ram.word(Y) >= 944
            } else {
                flags & 4 != 0
            }
        }
    } else {
        false
    };
    if !splash && ram.word(SPEED_BOOST_COUNTER) & 0xff00 != 0x400 {
        return Ok(());
    }
    let bottom = samus::dispatch_rom(ram, rom, "Samus_GetBottom_R18", 0, 0)? as u16;
    let water = ram.word(FX_Y_POS);
    let lava = ram.word(LAVA_ACID_Y_POS);
    if (water & 0x8000 != 0 && lava & 0x8000 == 0 && (lava.wrapping_sub(bottom) as i16) < 0)
        || (water & 0x8000 == 0 && (water.wrapping_sub(bottom) as i16) < 0 && ram.word(FX_LIQUID_OPTIONS) & 4 == 0)
    {
        return Ok(());
    }
    let left = ram.byte(SAMUS_POSE_X_DIR) == 4;
    for slot in 0..2 {
        let delta = if slot == 0 {
            if left { -12 } else { 12 }
        } else if left {
            8
        } else {
            -8
        };
        ram.set_word(ATMOSPHERIC_GFX_X_POS + slot * 2, ram.word(X).wrapping_add(delta as u16));
        ram.set_word(ATMOSPHERIC_GFX_Y_POS + slot * 2, ram.word(Y).wrapping_add(16));
        ram.set_word(ATMOSPHERIC_GFX_FRAME_AND_TYPE + slot * 2, if splash { 0x100 } else { 0x700 });
        ram.set_word(ATMOSPHERIC_GFX_ANIM_TIMER + slot * 2, if slot == 0 { 0x8002 } else { 3 });
    }
    Ok(())
}
fn stop_speed(ram: &mut Wram<'_>, rom: &[u8], base: bool) -> Result<(), &'static str> {
    samus_control::dispatch_rom(ram, rom, "Samus_CancelSpeedBoost", 0, 0)?;
    ram.set_word(SAMUS_X_EXTRA_RUN_SPEED, 0);
    ram.set_word(SAMUS_X_EXTRA_RUN_SUBSPEED, 0);
    if base {
        ram.set_word(SAMUS_X_BASE_SPEED, 0);
        ram.set_word(SAMUS_X_BASE_SUBSPEED, 0);
        ram.set_word(SAMUS_X_ACCEL_MODE, 0);
    }
    Ok(())
}
fn handler(ram: &mut Wram<'_>, rom: &[u8], host: &mut impl Host, mode: u8) -> Result<(), &'static str> {
    let call = |ram: &mut Wram<'_>, host: &mut _, name: &str| -> Result<(), &'static str> {
        dispatch_host(ram, rom, host, name, 0, 0)?;
        Ok(())
    };
    match mode {
        0 => {
            if ram.word(SAMUS_POSE) != 0 && ram.word(SAMUS_POSE) != 0x9b {
                if [1, 2].contains(&ram.word(SAMUS_POSE))
                    && ram.word(BUTTON_CONFIG_SHOOT_X) & ram.word(JOYPAD1_LASTKEYS) != 0
                {
                    ram.set_word(SAMUS_ANIM_FRAME_TIMER, 16);
                    ram.set_word(SAMUS_ANIM_FRAME, 0);
                }
                call(ram, host, "Samus_Move_NoBaseSpeed_X")?;
                move_y(ram, rom, host, false, false)?;
                stop_speed(ram, rom, true)?;
            } else {
                if ram.word(ELEVATOR_STATUS) != 0 {
                    ram.set_word(SAMUS_COLLISION_DIRECTION, 2);
                    dispatch_host(ram, rom, host, "Samus_MoveDown_NoSolidColl", 0x10000, 0)?;
                }
                ram.set_word(INPUT_TO_POSE_CALC, 0);
            }
        }
        1 => {
            call(ram, host, "Samus_HandleMovement_X")?;
            move_y(ram, rom, host, false, false)?;
            if ram.word(SAMUS_ANIM_FRAME_TIMER) == 1 {
                let frame = ram.word(SAMUS_ANIM_FRAME);
                if frame >= 10 {
                    return Err("invalid running animation frame");
                }
                if frame == 2 || frame == 7 {
                    footstep(ram, rom, 0)?;
                    if ram.word(CINEMATIC_FUNCTION) == 0
                        && ram.word(BOSS_ID) == 0
                        && ram.word(SAMUS_SHINE_TIMER) == 0
                        && ram.word(SPEED_BOOST_COUNTER) & 0x400 == 0
                    {
                        audio::queue_sfx(ram, 2, 0x0606)?;
                    }
                }
            }
        }
        2 | 20 | 25 => {
            if mode == 20 {
                if ram.word(SAMUS_ANIM_FRAME) < 23 {
                    if ram.word(SAMUS_ANIM_FRAME) >= 3 && ram.word(FLARE_COUNTER) >= 60 {
                        ram.set_word(SAMUS_CONTACT_DAMAGE_INDEX, 4);
                    }
                } else {
                    ram.set_word(SAMUS_CONTACT_DAMAGE_INDEX, 3);
                }
            }
            call(ram, host, "Samus_JumpingMovement")?;
        }
        3 => {
            let top = samus::dispatch_rom(ram, rom, "Samus_GetTop_R20", 0, 0)? as u16;
            let water = ram.word(FX_Y_POS);
            let lava = ram.word(LAVA_ACID_Y_POS);
            let submerged = ram.word(SAMUS_SUIT_PALETTE_INDEX) & 4 == 0
                && if water & 0x8000 != 0 {
                    lava & 0x8000 == 0 && (lava.wrapping_sub(top) as i16) < 0
                } else {
                    (water.wrapping_sub(top) as i16) < 0 && ram.word(FX_LIQUID_OPTIONS) & 4 == 0
                };
            if !submerged {
                if ram.word(EQUIPPED_ITEMS) & 0x200 != 0 && ram.word(SAMUS_Y_DIR) == 2 {
                    let velocity = ram.word(0xb2d);
                    let minimum = if ram.word(LIQUID_PHYSICS_TYPE) != 0 { 0x80 } else { 0x280 };
                    if (velocity.wrapping_sub(minimum) as i16) >= 0 && (velocity.wrapping_sub(0x500) as i16) < 0 {
                        ram.set_byte(UNUSED_WORD_7E0DFA, 1);
                        if ram.word(BUTTON_CONFIG_JUMP_A) & ram.word(JOYPAD1_NEWKEYS) != 0 {
                            samus::dispatch_rom(ram, rom, "Samus_InitJump", 0, 0)?;
                        }
                    }
                }
                if [0x81, 0x82].contains(&ram.word(SAMUS_POSE)) {
                    ram.set_word(SAMUS_CONTACT_DAMAGE_INDEX, 3);
                } else if (ram.word(FLARE_COUNTER).wrapping_sub(60) as i16) >= 0 {
                    ram.set_word(SAMUS_CONTACT_DAMAGE_INDEX, 4);
                }
            } else if ram.word(SAMUS_ANIM_FRAME_TIMER) == 1 {
                let frame = ram.word(SAMUS_ANIM_FRAME);
                let o = (((0x90a514u32 >> 16) << 15) | (0x90a514 & 0x7fff)) as usize & 0x3fffff;
                if *rom.get(o + usize::from(frame)).ok_or("underwater animation table outside image")? != 0 {
                    audio::queue_sfx(ram, 0, 0x2f06)?;
                }
            }
            call(ram, host, "Samus_SpinJumpMovement")?;
        }
        4 | 17 => {
            let pose = ram.word(SAMUS_POSE);
            let resting = ram.word(SAMUS_X_ACCEL_MODE) == 0
                && if ram.byte(SAMUS_POSE_X_DIR) == 4 {
                    pose == if mode == 4 { 0x41 } else { 0x7a }
                } else {
                    pose == if mode == 4 { 0x1d } else { 0x79 }
                };
            if resting {
                call(ram, host, "Samus_Move_NoBaseSpeed_X")?;
                if dispatch_host(ram, rom, host, "Samus_CheckAndMoveY", 0, 0)? & 1 == 0 {
                    move_y(ram, rom, host, false, false)?;
                    stop_speed(ram, rom, true)?;
                }
            } else {
                call(ram, host, "Samus_HandleMovement_X")?;
                if dispatch_host(ram, rom, host, "Samus_CheckAndMoveY", 0, 0)? & 1 == 0 {
                    move_y(ram, rom, host, false, false)?;
                }
            }
        }
        5 => {
            call(ram, host, "Samus_Move_NoBaseSpeed_X")?;
            move_y(ram, rom, host, false, false)?;
            for off in [
                SAMUS_X_EXTRA_RUN_SPEED,
                SAMUS_X_EXTRA_RUN_SUBSPEED,
                SAMUS_X_BASE_SPEED,
                SAMUS_X_BASE_SUBSPEED,
                SAMUS_X_ACCEL_MODE,
            ] {
                ram.set_word(off, 0);
            }
        }
        6 => {
            fall(ram, rom, host, false, false)?;
            if [0x29, 0x2a, 0x67, 0x68].contains(&ram.word(SAMUS_POSE))
                && (ram.word(SAMUS_Y_SPEED).wrapping_sub(5) as i16) >= 0
                && (ram.word(SAMUS_ANIM_FRAME).wrapping_sub(5) as i16) < 0
            {
                ram.set_word(SAMUS_ANIM_FRAME_TIMER, 8);
                ram.set_word(SAMUS_ANIM_FRAME, 5);
            }
        }
        8 | 19 => {
            if ram.word(JOYPAD1_LASTKEYS) & 0x300 == 0 && ram.word(SAMUS_X_ACCEL_MODE) == 0 {
                stop_speed(ram, rom, true)?;
            }
            fall(ram, rom, host, true, ram.word(USED_FOR_BALL_BOUNCE_ON_LANDING) != 0)?;
        }
        7 | 11 | 26 | 27 => ram.set_word(INPUT_TO_POSE_CALC, 0),
        9 | 12 | 13 => {}
        10 => {
            ram.set_word(INPUT_TO_POSE_CALC, 0);
            move_y(ram, rom, host, false, false)?;
        }
        14 => {
            call(ram, host, "Samus_HandleMovement_X")?;
            move_y(ram, rom, host, false, false)?;
            stop_speed(ram, rom, false)?;
            ram.set_word(INPUT_TO_POSE_CALC, 0);
        }
        15 => {
            let pose = ram.word(SAMUS_POSE);
            if pose >= 0xf1 || pose < 0xdb {
                if pose < 0xf1 {
                    let slot = pose.wrapping_sub(53);
                    if slot >= 12 {
                        return Err("invalid crouch transition pose");
                    }
                    if [4, 5, 10, 11].contains(&slot) {
                        ram.set_word(ENABLE_HORIZ_SLOPE_COLL, 3);
                        ram.set_word(UNUSEDWORD_7E0AA4, 0);
                    }
                }
                call(ram, host, "Samus_Move_NoBaseSpeed_X")?;
                if dispatch_host(ram, rom, host, "Samus_MoveY_Simple_", 0, 0)? & 1 == 0 {
                    move_y(ram, rom, host, false, false)?;
                }
            }
            if ram.word(INPUT_TO_POSE_CALC) == 1025 {
                ram.set_word(SAMUS_Y_SUBSPEED, 0);
                ram.set_word(SAMUS_Y_SPEED, 0);
                ram.set_word(SAMUS_Y_DIR, 0);
                ram.set_word(USED_FOR_BALL_BOUNCE_ON_LANDING, 0);
            }
            ram.set_word(INPUT_TO_POSE_CALC, 0);
        }
        16 => {
            call(ram, host, "Samus_HandleMovement_X")?;
            move_y(ram, rom, host, false, false)?;
        }
        18 => {
            if ram.word(USED_FOR_BALL_BOUNCE_ON_LANDING) != 0 {
                fall(ram, rom, host, true, true)?;
            } else {
                call(ram, host, "Samus_JumpingMovement")?;
            }
        }
        21 => {
            call(ram, host, "Samus_Move_NoBaseSpeed_X")?;
            move_y(ram, rom, host, false, false)?;
            stop_speed(ram, rom, true)?;
        }
        22 => {
            if ram.word(INPUT_TO_POSE_CALC) != 5 {
                ram.set_word(INPUT_TO_POSE_CALC, 0);
            }
        }
        23 | 24 => {
            call(ram, host, "Samus_HandleMovement_X")?;
            if dispatch_host(ram, rom, host, "Samus_CheckAndMoveY", 0, 0)? & 1 == 0 {
                move_y(ram, rom, host, false, false)?;
            }
            stop_speed(ram, rom, false)?;
            ram.set_word(INPUT_TO_POSE_CALC, 0);
        }
        _ => return Err("invalid movement type"),
    }
    Ok(())
}
fn bomb_x(ram: &mut Wram<'_>, rom: &[u8], host: &mut impl Host, falling: bool) -> Result<(), &'static str> {
    let pointer = if falling || ram.word(KNOCKBACK_DIR) != 0 {
        samus::dispatch_rom(ram, rom, "Samus_DetermineSpeedTableEntryPtr_X", 0, 0)?
    } else {
        0x9f25
    };
    let base = samus::dispatch_rom(ram, rom, "Samus_CalcBaseSpeed_X", pointer as i32, 0)?;
    let left = if falling {
        ram.word(SAMUS_VAR62) == 1
    } else if ram.word(KNOCKBACK_DIR) != 0 {
        ram.word(KNOCKBACK_X_DIR) == 0
    } else {
        ram.word(BOMB_JUMP_DIR) as u8 == 1
    };
    let amount = samus::dispatch_rom(
        ram,
        rom,
        if left { "Samus_CalcDisplacementMoveLeft" } else { "Samus_CalcDisplacementMoveRight" },
        base as i32,
        0,
    )? as i32;
    movement(ram, rom, host, if amount < 0 { 0 } else { 1 }, amount)
}
pub fn dispatch_host(
    ram: &mut Wram<'_>,
    rom: &[u8],
    host: &mut impl Host,
    symbol: &str,
    a: i32,
    _: i32,
) -> Result<u32, &'static str> {
    match symbol {
        "Samus_BombJumpRisingXMovement_" | "Samus_BombJumpFallingXMovement_" => {
            bomb_x(ram, rom, host, symbol == "Samus_BombJumpFallingXMovement_")?;
            Ok(0)
        }
        "Samus_BombJumpRisingYMovement_" => {
            if ram.word(SAMUS_Y_DIR) == 1 {
                if ram.word(SAMUS_Y_SPEED) & 0x8000 != 0 {
                    ram.set_word(SAMUS_Y_SUBSPEED, 0);
                    ram.set_word(SAMUS_Y_SPEED, 0);
                    ram.set_word(SAMUS_Y_DIR, 2);
                    if ram.word(BOMB_JUMP_DIR) as u8 != 2 {
                        ram.set_word(SAMUS_X_ACCEL_MODE, 2);
                    }
                } else if (ram.word(SAMUS_Y_SPEED).wrapping_sub(1) as i16) < 0
                    && ram.word(SAMUS_INPUT_HANDLER) != 0xe91d
                {
                    ram.set_word(SAMUS_INPUT_HANDLER, 0xe913);
                }
            }
            Ok(0)
        }
        "Samus_BombJumpFallingYMovement_" => {
            if (ram.word(SAMUS_Y_SPEED).wrapping_sub(5) as i16) < 0 {
                add(ram, SAMUS_Y_SPEED, SAMUS_Y_SUBSPEED, pair(ram, SAMUS_Y_ACCEL, SAMUS_Y_SUBACCEL) as i32);
            }
            movement(ram, rom, host, 3, pair(ram, SAMUS_Y_SPEED, SAMUS_Y_SUBSPEED) as i32)?;
            Ok(0)
        }
        "Samus_MoveHandler_ReleaseFromGrapple" => {
            if ram.word(SAMUS_Y_DIR) == 1 && ram.word(SAMUS_Y_SPEED) & 0x8000 != 0 {
                ram.set_word(SAMUS_Y_SUBSPEED, 0);
                ram.set_word(SAMUS_Y_SPEED, 0);
                ram.set_word(SAMUS_Y_DIR, 2);
                ram.set_word(SAMUS_MOVEMENT_HANDLER, 0xa337);
            }
            ram.set_word(SAMUS_X_ACCEL_MODE, 2);
            let pointer = samus::dispatch_rom(ram, rom, "Samus_DetermineGrappleSwingSpeed_X", 0, 0)?;
            let amount = samus::dispatch_rom(ram, rom, "Samus_CalcBaseSpeed_X", pointer as i32, 0)?;
            if ram.word(SAMUS_X_ACCEL_MODE) != 0 || ram.word(JOYPAD1_LASTKEYS) & 0x300 != 0 {
                dispatch_host(ram, rom, host, "Samus_MoveX", amount as i32, 0)?;
            } else {
                clear_base(ram);
            }
            move_y(ram, rom, host, true, false)?;
            if ram.word(INPUT_TO_POSE_CALC) != 0 {
                ram.set_word(SAMUS_MOVEMENT_HANDLER, 0xa337);
            }
            Ok(0)
        }
        "Samus_MovementHandler_Normal" => {
            if ram.word(TIME_IS_FROZEN_FLAG) == 0 {
                handler(ram, rom, host, ram.byte(SAMUS_MOVEMENT_TYPE))?;
                samus::dispatch_rom(ram, rom, "Samus_UpdateSpeedEchoPos", 0, 0)?;
            }
            Ok(0)
        }
        "Samus_FootstepGraphics"
        | "Samus_FootstepGraphics_Crateria"
        | "Samus_FootstepGraphics_Maridia"
        | "Samus_FootstepGraphics_1" => {
            footstep(
                ram,
                rom,
                match symbol {
                    "Samus_FootstepGraphics_Maridia" => 2,
                    "Samus_FootstepGraphics_1" => 1,
                    _ => 0,
                },
            )?;
            Ok(0)
        }
        "Samus_WallJumpCheck" | "Samus_GrappleWallJumpCheck" => {
            wall_jump(ram, rom, host, a, symbol == "Samus_GrappleWallJumpCheck")
        }
        "Samus_SpinJumpMovement" => {
            samus::dispatch_rom(ram, rom, "Samus_HandleExtraRunspeedX", 0, 0)?;
            if ram.word(SAMUS_Y_DIR) == 1
                && (ram.word(BUTTON_CONFIG_JUMP_A) & ram.word(JOYPAD1_LASTKEYS) == 0
                    || ram.word(SAMUS_Y_SPEED) & 0x8000 != 0)
            {
                ram.set_word(SAMUS_Y_SUBSPEED, 0);
                ram.set_word(SAMUS_Y_SPEED, 0);
                ram.set_word(SAMUS_Y_DIR, 2);
            }
            let (clamped, mut amount) = air_base_speed(ram, rom)?;
            let moving = clamped
                || ram.word(SAMUS_X_ACCEL_MODE) == 1
                || ram.word(JOYPAD1_LASTKEYS) & if ram.byte(SAMUS_POSE_X_DIR) == 4 { 0x200 } else { 0x100 } != 0;
            if moving {
                if ram.word(SAMUS_X_ACCEL_MODE) == 0 {
                    ram.set_word(SAMUS_X_ACCEL_MODE, 2);
                }
            } else {
                clear_base(ram);
                ram.set_word(SAMUS_X_ACCEL_MODE, 0);
                amount = 0;
            }
            dispatch_host(ram, rom, host, "Samus_MoveX", amount, 0)?;
            if wall_jump(ram, rom, host, 0x80000, false)? == 0 {
                move_y(ram, rom, host, true, false)?;
            }
            Ok(0)
        }
        "Samus_MoveY_WithSpeedCalc" => {
            move_y(ram, rom, host, true, false)?;
            Ok(0)
        }
        "Samus_Move_NoSpeedCalc_Y" | "Samus_MoveExtraY" => {
            move_y(ram, rom, host, false, symbol == "Samus_MoveExtraY")?;
            Ok(0)
        }
        "Samus_CheckAndMoveY" | "Samus_MoveY_Simple_" => {
            if ram.word(SAMUS_Y_DIR) == 0 {
                return Ok(0);
            }
            samus::dispatch_rom(ram, rom, "Samus_CheckStartFalling", 0, 0)?;
            move_y(ram, rom, host, true, false)?;
            Ok(1)
        }
        "Samus_FallingMovement" | "Samus_MorphedFallingMovement" | "Samus_MorphedBouncingMovement" => {
            fall(ram, rom, host, symbol != "Samus_FallingMovement", symbol == "Samus_MorphedBouncingMovement")?;
            Ok(0)
        }
        "Samus_JumpingMovement" => {
            samus::dispatch_rom(ram, rom, "Samus_HandleExtraRunspeedX", 0, 0)?;
            let pose = ram.word(SAMUS_POSE);
            if [0x4b, 0x4c].contains(&pose) || (0x55..0x5b).contains(&pose) {
                ram.set_word(SAMUS_X_ACCEL_MODE, 0);
                dispatch_host(ram, rom, host, "Samus_Move_NoBaseSpeed_X", 0, 0)?;
                move_y(ram, rom, host, false, true)?;
                return Ok(0);
            }
            if ram.word(SAMUS_Y_DIR) == 1
                && (ram.word(BUTTON_CONFIG_JUMP_A) & ram.word(JOYPAD1_LASTKEYS) == 0
                    || ram.word(SAMUS_Y_SPEED) & 0x8000 != 0)
            {
                ram.set_word(SAMUS_Y_SUBSPEED, 0);
                ram.set_word(SAMUS_Y_SPEED, 0);
                ram.set_word(SAMUS_Y_DIR, 2);
            }
            let (_, amount) = air_base_speed(ram, rom)?;
            if ram.byte(SAMUS_MOVEMENT_TYPE) == 20 && ram.word(SAMUS_X_ACCEL_MODE) == 0 {
                ram.set_word(SAMUS_X_ACCEL_MODE, 2);
            }
            if ram.word(SAMUS_X_ACCEL_MODE) != 0 || ram.word(JOYPAD1_LASTKEYS) & 0x300 != 0 {
                dispatch_host(ram, rom, host, "Samus_MoveX", amount, 0)?;
            } else {
                clear_base(ram);
            }
            move_y(ram, rom, host, true, false)?;
            Ok(0)
        }
        "Samus_HandleMovement_DrainedCrouching" => {
            move_y(ram, rom, host, true, false)?;
            if ram.word(COLLISION) != 0 {
                ram.set_word(SAMUS_MOVEMENT_HANDLER, 0xa337);
                ram.set_word(SAMUS_ANIM_FRAME_TIMER, 8);
                ram.set_word(SAMUS_ANIM_FRAME, 7);
                ram.set_word(SAMUS_Y_SUBSPEED, 0);
                ram.set_word(SAMUS_Y_SPEED, 0);
            }
            Ok(0)
        }
        "Samus_MoveLeft" | "Samus_MoveRight" | "Samus_MoveUp" | "Samus_MoveDown" => {
            movement(
                ram,
                rom,
                host,
                match symbol {
                    "Samus_MoveLeft" => 0,
                    "Samus_MoveRight" => 1,
                    "Samus_MoveUp" => 2,
                    _ => 3,
                },
                a,
            )?;
            Ok(0)
        }
        "Samus_MoveX" => {
            let accel = ram.word(SAMUS_X_ACCEL_MODE);
            let left = if accel == 0 || accel == 2 {
                ram.byte(SAMUS_POSE_X_DIR) == 4
            } else {
                ram.byte(SAMUS_POSE_X_DIR) == 8
            };
            let amount = samus::dispatch_rom(
                ram,
                rom,
                if left { "Samus_CalcDisplacementMoveLeft" } else { "Samus_CalcDisplacementMoveRight" },
                a,
                0,
            )? as i32;
            movement(ram, rom, host, if amount < 0 { 0 } else { 1 }, amount)?;
            Ok(0)
        }
        "Samus_Move_NoBaseSpeed_X" => dispatch_host(ram, rom, host, "Samus_MoveX", 0, 0),
        "Samus_HandleMovement_X" => {
            samus::dispatch_rom(ram, rom, "Samus_HandleExtraRunspeedX", 0, 0)?;
            let pointer = samus::dispatch_rom(ram, rom, "Samus_DetermineSpeedTableEntryPtr_X", 0, 0)?;
            let amount = samus::dispatch_rom(ram, rom, "Samus_CalcBaseSpeed_X", pointer as i32, 0)?;
            dispatch_host(ram, rom, host, "Samus_MoveX", amount as i32, 0)
        }
        "Samus_AlignYPosSlope" => {
            align_slope(ram)?;
            Ok(0)
        }
        "WallJumpBlockCollDetect" => {
            ram.set_word(SAMUS_COLLISION_DIRECTION, ram.word(SAMUS_COLLISION_DIRECTION) | 15);
            ram.set_word(FLAG_SAMUS_IN_QUICKSAND, 0);
            let (hit, amount) = traverse(ram, rom, host, a, false, false)?;
            ram.set_word(COLLISION, u16::from(hit));
            Ok(amount.wrapping_abs() as u32)
        }
        "Samus_CollDetectChangedPose" => {
            let mut detect = |ram: &mut Wram<'_>, amount: i32| -> Result<i32, &'static str> {
                ram.set_word(SAMUS_COLLISION_DIRECTION, ram.word(SAMUS_COLLISION_DIRECTION) | 15);
                ram.set_word(FLAG_SAMUS_IN_QUICKSAND, 0);
                let (hit, new) = traverse(ram, rom, host, amount, true, ram.word(NMI_FRAME_COUNTER_WORD) & 1 != 0)?;
                ram.set_word(COLLISION, u16::from(hit));
                Ok(new.wrapping_abs())
            };
            if ((a >> 16) as i16).wrapping_abs() as u16 & 0xfff8 == 0 {
                return Ok(detect(ram, a)? as u32);
            }
            let first = detect(ram, (((a >> 16) & 0xfff0) | 8) << 16)?;
            if ram.word(COLLISION) != 0 { Ok(first as u32) } else { Ok(detect(ram, a)? as u32) }
        }
        "Samus_MoveRight_NoSolidColl" => {
            let (hit, amount) = if a == 0 {
                (false, a)
            } else {
                ram.set_word(FLAG_SAMUS_IN_QUICKSAND, 0);
                traverse(ram, rom, host, a, false, false)?
            };
            ram.set_word(COLLISION, u16::from(hit));
            add(ram, X, XF, amount);
            Ok(amount as u32)
        }
        "Samus_MoveDown_NoSolidColl" => {
            let (hit, amount) = if a == 0 {
                (false, a)
            } else {
                ram.set_word(SAMUS_POS_ADJUSTED_BY_SLOPE_FLAG, 0);
                ram.set_word(FLAG_SAMUS_IN_QUICKSAND, 0);
                traverse(ram, rom, host, a, true, ram.word(NMI_FRAME_COUNTER_WORD) & 1 != 0)?
            };
            ram.set_word(COLLISION, u16::from(hit || ram.word(FLAG_SAMUS_IN_QUICKSAND) != 0));
            add(ram, Y, YF, amount);
            Ok(amount as u32)
        }
        _ if symbol.starts_with("Samus_Movement_") => {
            let number = symbol
                .strip_prefix("Samus_Movement_")
                .and_then(|v| v.split_once('_'))
                .and_then(|(n, _)| u8::from_str_radix(n, 16).ok())
                .ok_or("unknown collision routine")?;
            handler(ram, rom, host, number)?;
            Ok(0)
        }
        _ => Err("unknown collision routine"),
    }
}
