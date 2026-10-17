//! Complete bank90 camera frame boundary from pinned snesrev/sm.
//! Upstream license: crates/decomp/LICENSE.upstream.txt.
use crate::{WRAM_SIZE, Wram, room, samus};

const LAYER_X_FRACTION: usize = 0x090f;
const LAYER_X: usize = 0x0911;
const LAYER_Y_FRACTION: usize = 0x0913;
const LAYER_Y: usize = 0x0915;
const WIDTH: usize = 0x07a9;
const HEIGHT: usize = 0x07ab;
const UP_SCROLLER: usize = 0x07ad;
const DOWN_SCROLLER: usize = 0x07af;
const FINISHED_HOOK: usize = 0x07e9;
const CAMERA_DISTANCE_INDEX: usize = 0x0941;
const POSE_X_DIRECTION: usize = 0x0a1e;
const MOVEMENT_TYPE: usize = 0x0a1f;
const KNOCKBACK_DIRECTION: usize = 0x0a52;
const FROZEN: usize = 0x0a78;
const X: usize = 0x0af6;
const X_FRACTION: usize = 0x0af8;
const Y: usize = 0x0afa;
const Y_FRACTION: usize = 0x0afc;
const IDEAL_X: usize = 0x0b0a;
const IDEAL_Y: usize = 0x0b0e;
const PREVIOUS_X: usize = 0x0b10;
const PREVIOUS_X_FRACTION: usize = 0x0b12;
const PREVIOUS_Y: usize = 0x0b14;
const PREVIOUS_Y_FRACTION: usize = 0x0b16;
const Y_DIRECTION: usize = 0x0b36;
const X_ACCELERATION_MODE: usize = 0x0b4a;
const SLOW_GRAPPLE: usize = 0x0cf8;
const DISTANCE_X: usize = 0x0da2;
const DISTANCE_X_FRACTION: usize = 0x0da4;
const DISTANCE_Y: usize = 0x0da6;
const DISTANCE_Y_FRACTION: usize = 0x0da8;
const SCROLLS: usize = 0xcd20;

const INPUT_WORDS: [usize; 30] = [
    WIDTH,
    HEIGHT,
    UP_SCROLLER,
    DOWN_SCROLLER,
    FINISHED_HOOK,
    LAYER_X_FRACTION,
    LAYER_X,
    LAYER_Y_FRACTION,
    LAYER_Y,
    CAMERA_DISTANCE_INDEX,
    POSE_X_DIRECTION,
    KNOCKBACK_DIRECTION,
    FROZEN,
    X,
    X_FRACTION,
    Y,
    Y_FRACTION,
    IDEAL_X,
    IDEAL_Y,
    PREVIOUS_X,
    PREVIOUS_X_FRACTION,
    PREVIOUS_Y,
    PREVIOUS_Y_FRACTION,
    Y_DIRECTION,
    X_ACCELERATION_MODE,
    SLOW_GRAPPLE,
    DISTANCE_X,
    DISTANCE_X_FRACTION,
    DISTANCE_Y,
    DISTANCE_Y_FRACTION,
];
const OUTPUT_WORDS: [usize; 14] = [
    LAYER_X_FRACTION,
    LAYER_X,
    LAYER_Y_FRACTION,
    LAYER_Y,
    IDEAL_X,
    IDEAL_Y,
    PREVIOUS_X,
    PREVIOUS_X_FRACTION,
    PREVIOUS_Y,
    PREVIOUS_Y_FRACTION,
    DISTANCE_X,
    DISTANCE_X_FRACTION,
    DISTANCE_Y,
    DISTANCE_Y_FRACTION,
];

fn add_distance(ram: &mut Wram<'_>, x: bool, negative: bool) {
    let (position, fraction, distance, distance_fraction) = if x {
        (LAYER_X, LAYER_X_FRACTION, DISTANCE_X, DISTANCE_X_FRACTION)
    } else {
        (LAYER_Y, LAYER_Y_FRACTION, DISTANCE_Y, DISTANCE_Y_FRACTION)
    };
    let old = (u32::from(ram.word(position)) << 16) | u32::from(ram.word(fraction));
    let amount = (u32::from(ram.word(distance)) << 16) | u32::from(ram.word(distance_fraction));
    let next = if negative { old.wrapping_sub(amount) } else { old.wrapping_add(amount) };
    ram.set_word(position, (next >> 16) as u16);
    ram.set_word(fraction, next as u16);
}

fn handle_x(ram: &mut Wram<'_>) -> Result<(), &'static str> {
    if ram.word(PREVIOUS_X) == ram.word(X) {
        room::dispatch_rom(ram, &[], "HandleAutoscrolling_X", 0, 0)?;
        return Ok(());
    }
    const LEFT: [u16; 4] = [0xa0, 0x50, 0x20, 0xe0];
    const RIGHT: [u16; 4] = [0x60, 0x40, 0x20, 0xe0];
    let right =
        (ram.word(KNOCKBACK_DIRECTION) != 0 || ram.byte(MOVEMENT_TYPE) == 16 || ram.word(X_ACCELERATION_MODE) == 1)
            ^ (ram.byte(POSE_X_DIRECTION) != 4);
    let distance = (if right { RIGHT } else { LEFT })
        .get(usize::from(ram.word(CAMERA_DISTANCE_INDEX) >> 1))
        .copied()
        .ok_or("camera distance index outside four native entries")?;
    let ideal = ram.word(X).wrapping_sub(distance);
    ram.set_word(IDEAL_X, ideal);
    if ideal != ram.word(LAYER_X) {
        let negative = ideal.wrapping_sub(ram.word(LAYER_X)) & 0x8000 != 0;
        add_distance(ram, true, negative);
        room::dispatch_rom(
            ram,
            &[],
            if negative {
                "HandleScrollingWhenTriggeringScrollLeft"
            } else {
                "HandleScrollingWhenTriggeringScrollRight"
            },
            0,
            0,
        )?;
    }
    Ok(())
}

fn handle_y(ram: &mut Wram<'_>) -> Result<(), &'static str> {
    if ram.word(PREVIOUS_Y) == ram.word(Y) {
        room::dispatch_rom(ram, &[], "HandleAutoscrolling_Y", 0, 0)?;
        return Ok(());
    }
    let scroller = if ram.word(Y_DIRECTION) == 1 { DOWN_SCROLLER } else { UP_SCROLLER };
    let ideal = ram.word(Y).wrapping_sub(ram.word(scroller));
    ram.set_word(IDEAL_Y, ideal);
    if ideal != ram.word(LAYER_Y) {
        let negative = ideal.wrapping_sub(ram.word(LAYER_Y)) & 0x8000 != 0;
        add_distance(ram, false, negative);
        room::dispatch_rom(
            ram,
            &[],
            if negative { "HandleScrollingWhenTriggeringScrollUp" } else { "HandleScrollingWhenTriggeringScrollDown" },
            0,
            0,
        )?;
    }
    Ok(())
}

fn spore_spawn_hook(ram: &mut Wram<'_>) {
    if ram.word(LAYER_Y) <= 0x01d0 {
        ram.set_word(LAYER_Y, 464);
    }
}

fn main_scrolling(ram: &mut Wram<'_>) -> Result<(), &'static str> {
    if ram.word(SLOW_GRAPPLE) != 0 {
        // The signed-high X branch skips both manual axes in the native routine.
        if ram.word(X) & 0x8000 == 0 {
            let relative = ram.word(X).wrapping_sub(ram.word(LAYER_X));
            if ram.word(X) < ram.word(LAYER_X) || relative < 0x60 {
                ram.set_word(LAYER_X, ram.word(LAYER_X).wrapping_sub(3));
            } else if relative >= 0xa0 {
                ram.set_word(LAYER_X, ram.word(LAYER_X).wrapping_add(3));
            }
            if ram.word(Y) & 0x8000 == 0 {
                let relative = ram.word(Y).wrapping_sub(ram.word(LAYER_Y));
                if ram.word(Y) < ram.word(LAYER_Y) || relative < 0x70 {
                    ram.set_word(LAYER_Y, ram.word(LAYER_Y).wrapping_sub(3));
                } else if relative >= 0x90 {
                    ram.set_word(LAYER_Y, ram.word(LAYER_Y).wrapping_add(3));
                }
            }
        }
        room::dispatch_rom(ram, &[], "HandleAutoscrolling_X", 0, 0)?;
        room::dispatch_rom(ram, &[], "HandleAutoscrolling_Y", 0, 0)?;
    } else {
        samus::dispatch_rom(ram, &[], "Samus_CalcDistanceMoved_X", 0, 0)?;
        handle_x(ram)?;
        samus::dispatch_rom(ram, &[], "Samus_CalcDistanceMoved_Y", 0, 0)?;
        handle_y(ram)?;
    }
    if ram.word(FINISHED_HOOK) != 0 {
        spore_spawn_hook(ram);
    }
    for (previous, current) in
        [(PREVIOUS_X, X), (PREVIOUS_X_FRACTION, X_FRACTION), (PREVIOUS_Y, Y), (PREVIOUS_Y_FRACTION, Y_FRACTION)]
    {
        ram.set_word(previous, ram.word(current));
    }
    Ok(())
}

fn prepare(ram: &Wram<'_>, symbol: &str) -> Result<[u16; 14], &'static str> {
    if !matches!(
        symbol,
        "MainScrollingRoutine"
            | "Samus_HandleScroll_X"
            | "Samus_HandleScroll_Y"
            | "Samus_ScrollFinishedHook_SporeSpawnFight"
    ) {
        return Err("unsupported camera frame routine");
    }
    if symbol == "MainScrollingRoutine" && !matches!(ram.word(FINISHED_HOOK), 0 | 0x9589) {
        return Err("unsupported scrolling finished hook");
    }
    if symbol != "Samus_ScrollFinishedHook_SporeSpawnFight" {
        let width = ram.word(WIDTH);
        let height = ram.word(HEIGHT);
        if !(1..=255).contains(&width) || !(1..=255).contains(&height) || u32::from(width) * u32::from(height) > 256 {
            return Err("room extent outside native camera scroll allocation");
        }
    }
    // Room helpers can reset fractions before rejecting a later scroll lookup.
    // Execute the complete closure privately and publish only after it succeeds.
    let mut bytes = vec![0; WRAM_SIZE];
    let mut private = Wram::new(&mut bytes)?;
    for offset in INPUT_WORDS {
        private.set_word(offset, ram.word(offset));
    }
    for index in 0..256 {
        private.set_byte(SCROLLS + index, ram.byte(SCROLLS + index));
    }
    match symbol {
        "MainScrollingRoutine" => main_scrolling(&mut private)?,
        "Samus_HandleScroll_X" => handle_x(&mut private)?,
        "Samus_HandleScroll_Y" => handle_y(&mut private)?,
        "Samus_ScrollFinishedHook_SporeSpawnFight" => spore_spawn_hook(&mut private),
        _ => unreachable!(),
    }
    Ok(OUTPUT_WORDS.map(|offset| private.word(offset)))
}

/// Select a complete native camera closure without changing canonical WRAM.
pub fn supports(ram: &Wram<'_>, symbol: &str) -> bool {
    prepare(ram, symbol).is_ok()
}

/// A refusal leaves the entire caller state intact, including distance words.
pub fn dispatch_rom(ram: &mut Wram<'_>, _rom: &[u8], symbol: &str, _a: i32, _b: i32) -> Result<u32, &'static str> {
    let update = prepare(ram, symbol)?;
    for (offset, value) in OUTPUT_WORDS.into_iter().zip(update) {
        ram.set_word(offset, value);
    }
    Ok(0)
}
