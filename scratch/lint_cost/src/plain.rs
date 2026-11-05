//! Samus movement and slope landing, written without the K5/K8 restriction lints:
//! plain `+`/`-`, `as` and indexing. Same behaviour as `strict.rs`.

pub const WALK_ACCEL: u32 = 0x3000;
pub const RUN_ACCEL: u32 = 0x4000;
pub const WALK_MAX: u32 = 0x2_0000;
pub const RUN_MAX: u32 = 0x3_0000;
pub const DECEL: u32 = 0x2000;
pub const GRAVITY: u32 = 0x1C00;
pub const TERMINAL: u32 = 0x5_0000;
pub const JUMP: u32 = 0x4_E000;
pub const HALF_HEIGHT: u16 = 0x14;

pub const SLOPES: [[u8; 16]; 4] = [
    [15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0],
    [15, 15, 14, 14, 13, 13, 12, 12, 11, 11, 10, 10, 9, 9, 8, 8],
    [7, 7, 6, 6, 5, 5, 4, 4, 3, 3, 2, 2, 1, 1, 0, 0],
    [8; 16],
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Samus {
    pub x: u32,
    pub y: u32,
    pub x_speed: u32,
    pub facing_left: bool,
    pub y_speed: u32,
    pub moving_up: bool,
    pub on_ground: bool,
}

pub struct Level {
    pub width: usize,
    pub blocks: Vec<u16>,
    pub bts: Vec<u8>,
}

/// `held`: bit 0 left, bit 1 right, bit 2 run, bit 3 jump.
pub fn step(samus: &mut Samus, level: &Level, held: u8) {
    let left = held & 1 != 0;
    let right = held & 2 != 0;
    let run = held & 4 != 0;
    let jump = held & 8 != 0;

    if left || right {
        let want_left = left && !right;
        if want_left != samus.facing_left {
            samus.facing_left = want_left;
            samus.x_speed = 0;
        }
        let (accel, max) = if run { (RUN_ACCEL, RUN_MAX) } else { (WALK_ACCEL, WALK_MAX) };
        samus.x_speed = (samus.x_speed + accel).min(max);
    } else {
        samus.x_speed = samus.x_speed.saturating_sub(DECEL);
    }
    if samus.facing_left {
        samus.x = samus.x.wrapping_sub(samus.x_speed);
    } else {
        samus.x = samus.x.wrapping_add(samus.x_speed);
    }

    if jump && samus.on_ground {
        samus.on_ground = false;
        samus.moving_up = true;
        samus.y_speed = JUMP;
    }
    if !samus.on_ground {
        if samus.moving_up {
            if samus.y_speed <= GRAVITY {
                samus.y_speed = 0;
                samus.moving_up = false;
            } else {
                samus.y_speed -= GRAVITY;
                samus.y = samus.y.wrapping_sub(samus.y_speed);
            }
        } else {
            samus.y_speed = (samus.y_speed + GRAVITY).min(TERMINAL);
            samus.y = samus.y.wrapping_add(samus.y_speed);
        }
    }
    land_on_slope(samus, level);
}

fn land_on_slope(samus: &mut Samus, level: &Level) {
    let x = (samus.x >> 16) as u16;
    let bottom = ((samus.y >> 16) as u16).wrapping_add(HALF_HEIGHT);
    let column = (x >> 4) as usize;
    let row = (bottom >> 4) as usize;
    if column >= level.width {
        return;
    }
    let index = row * level.width + column;
    if index >= level.blocks.len() {
        return;
    }
    if level.blocks[index] >> 12 != 1 {
        return;
    }
    let bts = level.bts[index];
    let shape = (bts & 0x1F) as usize;
    if shape >= SLOPES.len() {
        return;
    }
    let mut in_block = (x & 0xF) as usize;
    if bts & 0x40 != 0 {
        in_block ^= 0xF;
    }
    let floor = (bottom & !0xF) + SLOPES[shape][in_block] as u16;
    if bottom >= floor && !samus.moving_up {
        let lift = bottom - floor;
        samus.y = (samus.y & 0xFFFF) | ((((samus.y >> 16) as u16).wrapping_sub(lift) as u32) << 16);
        samus.y_speed = 0;
        samus.on_ground = true;
    }
}
