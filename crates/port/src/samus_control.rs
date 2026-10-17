//! Samus input, momentum and animation control from pinned snesrev/sm.
//! Upstream license: crates/decomp/LICENSE.upstream.txt.
use crate::{Wram, audio, samus};

const JOYPAD1_LASTKEYS: usize = 0x8b;
const JOYPAD1_NEWKEYS: usize = 0x8f;
const NMI_FRAME_COUNTER_WORD: usize = 0x5b6;
const RANDOM_NUMBER: usize = 0x5e5;
const EQUIPPED_ITEMS: usize = 0x9a2;
const BUTTON_CONFIG_SHOOT_X: usize = 0x9b2;
const BUTTON_CONFIG_JUMP_A: usize = 0x9b4;
const BUTTON_CONFIG_RUN_B: usize = 0x9b6;
const BUTTON_CONFIG_ITEMCANCEL_Y: usize = 0x9b8;
const BUTTON_CONFIG_AIM_DOWN_L: usize = 0x9bc;
const BUTTON_CONFIG_AIM_UP_R: usize = 0x9be;
const SAMUS_HEALTH: usize = 0x9c2;
const GAME_TIME_FRAMES: usize = 0x9da;
const UNUSED_WORD_7E0A18: usize = 0xa18;
const SAMUS_POSE: usize = 0xa1c;
const SAMUS_POSE_X_DIR: usize = 0xa1e;
const SAMUS_MOVEMENT_TYPE: usize = 0xa1f;
const SAMUS_NEW_POSE: usize = 0xa28;
const SAMUS_NEW_POSE_TRANSITIONAL: usize = 0xa2c;
const SAMUS_MOMENTUM_ROUTINE_INDEX: usize = 0xa2e;
const SAMUS_HURT_SWITCH_INDEX: usize = 0xa32;
const ENABLE_HORIZ_SLOPE_COLL: usize = 0xa46;
const SAMUS_PERIODIC_SUBDAMAGE: usize = 0xa4e;
const SAMUS_PERIODIC_DAMAGE: usize = 0xa50;
const BOMB_JUMP_DIR: usize = 0xa56;
const SAMUS_MOVEMENT_HANDLER: usize = 0xa58;
const SAMUS_INPUT_HANDLER: usize = 0xa60;
const SAMUS_X_SPEED_DIVISOR: usize = 0xa66;
const SAMUS_SUIT_PALETTE_INDEX: usize = 0xa74;
const SAMUS_ANIM_FRAME_TIMER: usize = 0xa94;
const SAMUS_ANIM_FRAME: usize = 0xa96;
const SAMUS_ANIM_FRAME_BUFFER: usize = 0xa9c;
const UNUSEDWORD_7E0AA4: usize = 0xaa4;
const SPEED_ECHOES_INDEX: usize = 0xaae;
const SPEED_ECHO_XSPEED: usize = 0xac0;
const SPECIAL_SAMUS_PALETTE_FRAME: usize = 0xace;
const SPECIAL_SAMUS_PALETTE_TIMER: usize = 0xad0;
const LIQUID_PHYSICS_TYPE: usize = 0xad2;
const ATMOSPHERIC_GFX_ANIM_TIMER: usize = 0xad4;
const ATMOSPHERIC_GFX_X_POS: usize = 0xadc;
const ATMOSPHERIC_GFX_Y_POS: usize = 0xae4;
const ATMOSPHERIC_GFX_FRAME_AND_TYPE: usize = 0xaec;
const SAMUS_X_POS: usize = 0xaf6;
const SAMUS_Y_POS: usize = 0xafa;
const SAMUS_Y_RADIUS: usize = 0xb00;
const SAMUS_COLLISION_DIRECTION: usize = 0xb02;
const SAMUS_Y_SUBSPEED: usize = 0xb2c;
const SAMUS_Y_SPEED: usize = 0xb2e;
const SAMUS_Y_DIR: usize = 0xb36;
const SAMUS_HAS_MOMENTUM_FLAG: usize = 0xb3c;
const SPEED_BOOST_COUNTER: usize = 0xb3e;
const SAMUS_ECHOES_SOUND_FLAG: usize = 0xb40;
const SAMUS_X_EXTRA_RUN_SPEED: usize = 0xb42;
const SAMUS_X_EXTRA_RUN_SUBSPEED: usize = 0xb44;
const SAMUS_X_BASE_SPEED: usize = 0xb46;
const SAMUS_X_BASE_SUBSPEED: usize = 0xb48;
const SAMUS_X_ACCEL_MODE: usize = 0xb4a;
const INPUT_TO_POSE_CALC: usize = 0xdc6;
const SAMUS_COLLIDES_WITH_SOLID_ENEMY: usize = 0xdce;
const SAMUS_COLLISION_FLAG: usize = 0xdd0;
const ELEVATOR_STATUS: usize = 0xe18;
const SAMUS_KNOCKBACK_TIMER: usize = 0x18aa;
const FX_Y_POS: usize = 0x195e;
const LAVA_ACID_Y_POS: usize = 0x1962;
const FX_TYPE: usize = 0x196e;
const FX_LIQUID_OPTIONS: usize = 0x197e;
fn rom_bytes(rom: &[u8], address: u32, len: usize) -> Result<&[u8], &'static str> {
    let off = (((address >> 16) << 15) | (address & 0x7fff)) as usize & 0x3fffff;
    rom.get(off..off + len).ok_or("ROM table outside image")
}
fn word(rom: &[u8], address: u32) -> Result<u16, &'static str> {
    let b = rom_bytes(rom, address, 2)?;
    Ok(u16::from_le_bytes([b[0], b[1]]))
}
fn suit_palette(ram: &Wram<'_>) -> u16 {
    if ram.word(EQUIPPED_ITEMS) & 0x20 != 0 {
        0x9800
    } else if ram.word(EQUIPPED_ITEMS) & 1 != 0 {
        0x9520
    } else {
        0x9400
    }
}
fn copy_palette(ram: &mut Wram<'_>, rom: &[u8], pointer: u16, target: bool) -> Result<(), &'static str> {
    let bytes = rom_bytes(rom, 0x9b0000 | u32::from(pointer), 32)?;
    for (index, &byte) in bytes.iter().enumerate() {
        ram.set_byte((if target { 0xc380 } else { 0xc180 }) + index, byte);
    }
    Ok(())
}
fn cancel_boost(ram: &mut Wram<'_>, rom: &[u8]) -> Result<(), &'static str> {
    if ram.word(SAMUS_HAS_MOMENTUM_FLAG) != 0 {
        copy_palette(ram, rom, suit_palette(ram), false)?;
        for off in
            [SAMUS_HAS_MOMENTUM_FLAG, SPEED_BOOST_COUNTER, SPECIAL_SAMUS_PALETTE_FRAME, SPECIAL_SAMUS_PALETTE_TIMER]
        {
            ram.set_word(off, 0);
        }
    }
    if ram.word(SPEED_ECHOES_INDEX) & 0x8000 == 0 {
        ram.set_word(SPEED_ECHOES_INDEX, 0xffff);
        let speed = if ram.byte(SAMUS_POSE_X_DIR) == 4 { 0xfff8 } else { 8 };
        ram.set_word(SPEED_ECHO_XSPEED, speed);
        ram.set_word(0xac2, speed);
    }
    Ok(())
}
fn pose_field(ram: &Wram<'_>, rom: &[u8], field: u32) -> Result<u8, &'static str> {
    let pose = ram.word(SAMUS_POSE);
    if pose > 252 {
        return Err("invalid Samus pose");
    }
    Ok(rom_bytes(rom, 0x91b629 + u32::from(pose) * 8 + field, 1)?[0])
}
fn pose_momentum(ram: &mut Wram<'_>) -> Result<u32, &'static str> {
    const TABLE: [u16; 28] = [2, 1, 1, 0, 6, 2, 8, 2, 1, 6, 2, 2, 2, 6, 2, 2, 2, 6, 6, 6, 6, 2, 6, 2, 2, 2, 2, 2];
    let mode = *TABLE.get(usize::from(ram.byte(SAMUS_MOVEMENT_TYPE))).ok_or("invalid movement type")?;
    let moving = mode == 1 && (ram.word(SAMUS_X_BASE_SPEED) != 0 || ram.word(SAMUS_X_BASE_SUBSPEED) != 0);
    ram.set_word(SAMUS_MOMENTUM_ROUTINE_INDEX, if mode == 1 && !moving { 2 } else { mode });
    Ok(u32::from(moving))
}
fn cancel_grapple(ram: &mut Wram<'_>, rom: &[u8]) -> Result<(), &'static str> {
    let default = pose_field(ram, rom, 2)?;
    let moving = pose_momentum(ram)? != 0;
    ram.set_word(SAMUS_NEW_POSE, if moving || default == 255 { ram.word(SAMUS_POSE) } else { u16::from(default) });
    Ok(())
}
fn default_buttons(ram: &Wram<'_>, input: u16) -> u16 {
    let mut translated = input & 0x0f00;
    for (offset, mask) in [
        (BUTTON_CONFIG_SHOOT_X, 0x40),
        (BUTTON_CONFIG_JUMP_A, 0x80),
        (BUTTON_CONFIG_RUN_B, 0x8000),
        (BUTTON_CONFIG_ITEMCANCEL_Y, 0x4000),
    ] {
        if ram.word(offset) & input != 0 {
            translated |= mask;
        }
    }
    for (offset, mask) in [(BUTTON_CONFIG_AIM_UP_R, 0x10), (BUTTON_CONFIG_AIM_DOWN_L, 0x20)] {
        if ram.word(offset) & input != 0 && ram.word(offset) & 0x30 != 0 {
            translated |= mask;
        }
    }
    !translated
}
fn lookup_transition(ram: &mut Wram<'_>, rom: &[u8]) -> Result<(), &'static str> {
    if ram.word(JOYPAD1_LASTKEYS) != 0 {
        let pose = ram.word(SAMUS_POSE);
        if pose > 252 {
            return Err("invalid Samus pose");
        }
        let mut address = 0x910000 | u32::from(word(rom, 0x919ee2 + u32::from(pose) * 2)?);
        let new = default_buttons(ram, ram.word(JOYPAD1_NEWKEYS));
        let held = default_buttons(ram, ram.word(JOYPAD1_LASTKEYS));
        for index in 0..256 {
            let required_new = word(rom, address)?;
            if required_new == 0xffff {
                if index == 0 {
                    return Ok(());
                }
                ram.set_word(UNUSED_WORD_7E0A18, 0);
                return cancel_grapple(ram, rom);
            }
            if new & required_new == 0 && held & word(rom, address + 2)? == 0 {
                let target = word(rom, address + 4)?;
                if target != pose {
                    ram.set_word(SAMUS_NEW_POSE, target);
                    ram.set_word(BOMB_JUMP_DIR, 0);
                }
                return Ok(());
            }
            address += 6;
        }
        return Err("unterminated pose transition table");
    }
    ram.set_word(UNUSED_WORD_7E0A18, 0);
    cancel_grapple(ram, rom)
}

fn negative(v: u16) -> bool {
    v & 0x8000 != 0
}
fn boundary(ram: &mut Wram<'_>, rom: &[u8], top: bool) -> Result<u16, &'static str> {
    Ok(samus::dispatch_rom(ram, rom, if top { "Samus_GetTop_R20" } else { "Samus_GetBottom_R18" }, 0, 0)? as u16)
}
fn sfx(ram: &mut Wram<'_>, channel: usize, sound: u16, max: u16) -> Result<(), &'static str> {
    audio::queue_sfx(ram, channel, (sound << 8) | max)
}
fn bubbles(ram: &mut Wram<'_>, rom: &[u8]) -> Result<(), &'static str> {
    let top = boundary(ram, rom, true)?;
    if !negative(top.wrapping_sub(24).wrapping_sub(ram.word(FX_Y_POS)))
        && ram.word(NMI_FRAME_COUNTER_WORD) & 0x7f == 0
        && ram.word(0xaf0) == 0
    {
        ram.set_word(0xaf0, 0x500);
        ram.set_word(0xad8, 3);
        ram.set_word(0xae0, ram.word(SAMUS_X_POS));
        ram.set_word(0xae8, ram.word(SAMUS_Y_POS).wrapping_sub(ram.word(SAMUS_Y_RADIUS)).wrapping_add(6));
        let old = ram.word(RANDOM_NUMBER);
        let product = (old as u8 as u16) * 5;
        let carry = (product >> 8) + u16::from(((old >> 8) as u8).wrapping_mul(5)) + 1;
        let next = (((carry as u8 as u16) << 8) | (product & 255)).wrapping_add(carry >> 8).wrapping_add(17);
        ram.set_word(RANDOM_NUMBER, next);
        sfx(ram, 1, if next & 1 != 0 { 15 } else { 17 }, 6)?;
    }
    if [0, 0x9b].contains(&ram.word(SAMUS_POSE)) || ram.word(EQUIPPED_ITEMS) & 0x20 != 0 {
        ram.set_word(SAMUS_ANIM_FRAME_BUFFER, 0);
    }
    Ok(())
}
fn splash(ram: &mut Wram<'_>, rom: &[u8], bottom: u16) -> Result<(), &'static str> {
    const SPLASH: [bool; 28] = [
        true, false, false, false, true, true, false, false, false, false, false, false, false, false, true, true,
        true, true, false, false, false, true, false, false, false, false, false, false,
    ];
    if *SPLASH.get(usize::from(ram.byte(SAMUS_MOVEMENT_TYPE))).ok_or("invalid movement type")? {
        for (slot, delta) in [(0, 4i16), (1, -3)] {
            ram.set_word(ATMOSPHERIC_GFX_FRAME_AND_TYPE + slot * 2, 0x100);
            ram.set_word(ATMOSPHERIC_GFX_ANIM_TIMER + slot * 2, 3);
            ram.set_word(ATMOSPHERIC_GFX_X_POS + slot * 2, ram.word(SAMUS_X_POS).wrapping_add(delta as u16));
            ram.set_word(ATMOSPHERIC_GFX_Y_POS + slot * 2, bottom.wrapping_sub(4));
        }
    } else {
        ram.set_word(ATMOSPHERIC_GFX_FRAME_AND_TYPE, 0x300);
        ram.set_word(ATMOSPHERIC_GFX_ANIM_TIMER, 2);
        ram.set_word(ATMOSPHERIC_GFX_X_POS, ram.word(SAMUS_X_POS));
        ram.set_word(ATMOSPHERIC_GFX_Y_POS, ram.word(FX_Y_POS));
    }
    bubbles(ram, rom)
}
fn no_fx(ram: &mut Wram<'_>, rom: &[u8]) -> Result<(), &'static str> {
    let bottom = boundary(ram, rom, false)?;
    ram.set_word(SAMUS_ANIM_FRAME_BUFFER, ram.word(SAMUS_X_SPEED_DIVISOR));
    if ram.word(LIQUID_PHYSICS_TYPE) != 0 {
        let water = ram.word(LIQUID_PHYSICS_TYPE) & 1 != 0;
        ram.set_word(LIQUID_PHYSICS_TYPE, 0);
        if water {
            sfx(ram, 1, 14, 6)?;
            if ram.word(SAMUS_SUIT_PALETTE_INDEX) & 4 == 0 && [3, 20].contains(&ram.byte(SAMUS_MOVEMENT_TYPE)) {
                sfx(ram, 0, 0x30, 6)?;
            }
            splash(ram, rom, bottom)?;
        }
    }
    Ok(())
}
fn water_fx(ram: &mut Wram<'_>, rom: &[u8]) -> Result<(), &'static str> {
    let bottom = boundary(ram, rom, false)?;
    let water = ram.word(FX_Y_POS);
    if !negative(water) && negative(water.wrapping_sub(bottom)) && ram.word(FX_LIQUID_OPTIONS) & 4 == 0 {
        ram.set_word(SAMUS_ANIM_FRAME_BUFFER, 3);
        if ram.word(LIQUID_PHYSICS_TYPE) == 1 {
            bubbles(ram, rom)?;
        } else {
            ram.set_word(LIQUID_PHYSICS_TYPE, 1);
            sfx(ram, 1, 13, 6)?;
            splash(ram, rom, bottom)?;
        }
    } else {
        no_fx(ram, rom)?;
    }
    Ok(())
}
fn submerged_lava(ram: &mut Wram<'_>, rom: &[u8]) -> Result<(), &'static str> {
    let top = boundary(ram, rom, true)?;
    ram.set_word(SAMUS_ANIM_FRAME_BUFFER, 2);
    ram.set_word(LIQUID_PHYSICS_TYPE, 2);
    if negative(top.wrapping_sub(ram.word(LAVA_ACID_Y_POS))) && ram.word(ATMOSPHERIC_GFX_FRAME_AND_TYPE) & 0x400 == 0 {
        for slot in 0..4 {
            ram.set_word(ATMOSPHERIC_GFX_FRAME_AND_TYPE + slot * 2, 0x400);
            ram.set_word(ATMOSPHERIC_GFX_ANIM_TIMER + slot * 2, if slot == 0 || slot == 3 { 3 } else { 0x8002 });
            ram.set_word(ATMOSPHERIC_GFX_Y_POS + slot * 2, ram.word(LAVA_ACID_Y_POS));
            ram.set_word(ATMOSPHERIC_GFX_X_POS + slot * 2, ram.word(SAMUS_X_POS).wrapping_add([6, 0, 0, 0xfffa][slot]));
        }
        if ram.word(GAME_TIME_FRAMES) & 1 == 0 {
            sfx(ram, 1, 16, 6)?;
        }
    }
    if [0, 0x9b].contains(&ram.word(SAMUS_POSE)) || ram.word(EQUIPPED_ITEMS) & 0x20 != 0 {
        ram.set_word(SAMUS_ANIM_FRAME_BUFFER, 0);
    }
    Ok(())
}
fn lava_fx(ram: &mut Wram<'_>, rom: &[u8], acid: bool) -> Result<(), &'static str> {
    let bottom = boundary(ram, rom, false)?;
    let lava = ram.word(LAVA_ACID_Y_POS);
    if !negative(lava) && negative(lava.wrapping_sub(bottom)) {
        if !acid && ram.word(SPEED_BOOST_COUNTER) != 0 {
            cancel_boost(ram, rom)?;
            ram.set_word(SAMUS_X_EXTRA_RUN_SPEED, 0);
            ram.set_word(SAMUS_X_EXTRA_RUN_SUBSPEED, 0);
        }
        if !acid && ram.word(EQUIPPED_ITEMS) & 0x20 != 0 {
            ram.set_word(SAMUS_ANIM_FRAME_BUFFER, 0);
            ram.set_word(LIQUID_PHYSICS_TYPE, 2);
        } else {
            if ram.word(GAME_TIME_FRAMES) & 7 == 0 && !negative(ram.word(SAMUS_HEALTH).wrapping_sub(71)) {
                sfx(ram, 2, 0x2d, 3)?;
            }
            let damage = ((u32::from(ram.word(SAMUS_PERIODIC_DAMAGE)) << 16)
                | u32::from(ram.word(SAMUS_PERIODIC_SUBDAMAGE)))
            .wrapping_add(if acid { 0x18000 } else { 0x8000 });
            ram.set_word(SAMUS_PERIODIC_DAMAGE, (damage >> 16) as u16);
            ram.set_word(SAMUS_PERIODIC_SUBDAMAGE, damage as u16);
            submerged_lava(ram, rom)?;
        }
    } else {
        no_fx(ram, rom)?;
    }
    Ok(())
}

fn animation_command(ram: &mut Wram<'_>, rom: &[u8], address: u32, command: u8) -> Result<bool, &'static str> {
    let byte = |n: u32| -> Result<u16, &'static str> { Ok(u16::from(rom_bytes(rom, address + n, 1)?[0])) };
    let transition = |ram: &mut Wram<'_>, pose: u16| {
        ram.set_word(SAMUS_NEW_POSE_TRANSITIONAL, pose);
        ram.set_word(SAMUS_HURT_SWITCH_INDEX, 3);
    };
    match command & 15 {
        0..=5 => Ok(false),
        6 => {
            ram.set_word(
                SAMUS_ANIM_FRAME,
                if negative(ram.word(SAMUS_HEALTH).wrapping_sub(30)) {
                    ram.word(SAMUS_ANIM_FRAME).wrapping_add(1)
                } else {
                    0
                },
            );
            Ok(true)
        }
        7 => {
            ram.set_word(SAMUS_MOVEMENT_HANDLER, 0x94cb);
            ram.set_word(SAMUS_ANIM_FRAME, ram.word(SAMUS_ANIM_FRAME).wrapping_add(1));
            Ok(true)
        }
        8 => {
            if ram.word(SAMUS_INPUT_HANDLER) == 0xe91d {
                transition(ram, byte(1)?);
            } else if ![0x4b, 0x4c, 0x19, 0x1a].contains(&ram.word(SAMUS_NEW_POSE)) {
                ram.set_word(SAMUS_INPUT_HANDLER, 0xe926);
                transition(ram, byte(1)?);
            }
            Ok(false)
        }
        9 => {
            let equipped = word(rom, address + 1)? & ram.word(EQUIPPED_ITEMS) != 0;
            let moving = ram.word(SAMUS_Y_SPEED) != 0 || ram.word(SAMUS_Y_SUBSPEED) != 0;
            transition(
                ram,
                byte(match (equipped, moving) {
                    (true, true) => 6,
                    (true, false) => 5,
                    (false, true) => 4,
                    (false, false) => 3,
                })?,
            );
            Ok(false)
        }
        10 => {
            transition(ram, byte(if ram.word(SAMUS_Y_SPEED) != 0 || ram.word(SAMUS_Y_SUBSPEED) != 0 { 2 } else { 1 })?);
            Ok(false)
        }
        11 => {
            let top = boundary(ram, rom, true)?;
            let water = ram.word(FX_Y_POS);
            let lava = ram.word(LAVA_ACID_Y_POS);
            let submerged = ram.word(EQUIPPED_ITEMS) & 0x20 == 0
                && (if !negative(water) {
                    negative(water.wrapping_sub(top)) && ram.word(FX_LIQUID_OPTIONS) & 4 == 0
                } else {
                    !negative(lava) && negative(lava.wrapping_sub(top))
                });
            let (sound, skip) = if !submerged && ram.word(EQUIPPED_ITEMS) & 8 != 0 {
                (0x33, 21)
            } else if !submerged && ram.word(EQUIPPED_ITEMS) & 0x200 != 0 {
                (0x3e, 11)
            } else {
                (0x31, 1)
            };
            sfx(ram, 0, sound, 6)?;
            ram.set_word(SAMUS_ANIM_FRAME, ram.word(SAMUS_ANIM_FRAME).wrapping_add(skip));
            Ok(true)
        }
        12 => {
            transition(ram, byte(if word(rom, address + 1)? & ram.word(EQUIPPED_ITEMS) != 0 { 4 } else { 3 })?);
            Ok(false)
        }
        13 => {
            transition(ram, byte(1)?);
            Ok(false)
        }
        14 => {
            ram.set_word(SAMUS_ANIM_FRAME, ram.word(SAMUS_ANIM_FRAME).wrapping_sub(byte(1)?));
            Ok(true)
        }
        15 => {
            ram.set_word(SAMUS_ANIM_FRAME, 0);
            Ok(true)
        }
        _ => unreachable!(),
    }
}
fn speed_booster_delay(ram: &mut Wram<'_>, rom: &[u8], address: u32) -> Result<u8, &'static str> {
    let command = rom_bytes(rom, address, 1)?[0];
    if ram.word(SAMUS_HAS_MOMENTUM_FLAG) == 0
        || ram.word(BUTTON_CONFIG_RUN_B) & ram.word(JOYPAD1_LASTKEYS) == 0
        || ram.byte(SAMUS_MOVEMENT_TYPE) != 1
    {
        return Ok(command);
    }
    if ram.word(EQUIPPED_ITEMS) & 0x2000 == 0 {
        ram.set_word(SAMUS_ANIM_FRAME, 0);
        let pointer = word(rom, 0x91b5d1)?;
        ram.set_word(
            SAMUS_ANIM_FRAME_TIMER,
            ram.word(SAMUS_ANIM_FRAME_BUFFER)
                .wrapping_add(u16::from(rom_bytes(rom, 0x910000 | u32::from(pointer), 1)?[0])),
        );
        return Ok(0);
    }
    let mut counter = ram.word(SPEED_BOOST_COUNTER).wrapping_sub(1);
    ram.set_word(SPEED_BOOST_COUNTER, counter);
    if counter as u8 != 0 {
        return Ok(command);
    }
    if counter & 0x400 == 0 {
        counter = counter.wrapping_add(0x100);
        ram.set_word(SPEED_BOOST_COUNTER, counter);
        if counter & 0x400 != 0 {
            ram.set_word(SAMUS_ECHOES_SOUND_FLAG, 1);
            sfx(ram, 2, 3, 6)?;
        }
    }
    let stage = u32::from(counter >> 8);
    if stage > 4 {
        return Err("invalid speed booster stage");
    }
    ram.set_word(SPEED_BOOST_COUNTER, word(rom, 0x91b61f + stage * 2)? | (ram.word(SPEED_BOOST_COUNTER) & 0xff00));
    ram.set_word(SAMUS_ANIM_FRAME, 0);
    let pointer = word(rom, 0x91b5de + stage * 2)?;
    ram.set_word(
        SAMUS_ANIM_FRAME_TIMER,
        ram.word(SAMUS_ANIM_FRAME_BUFFER).wrapping_add(u16::from(rom_bytes(rom, 0x910000 | u32::from(pointer), 1)?[0])),
    );
    Ok(0)
}
fn handle_anim_delay(ram: &mut Wram<'_>, rom: &[u8]) -> Result<(), &'static str> {
    let pose = ram.word(SAMUS_POSE);
    if pose > 252 {
        return Err("invalid Samus pose");
    }
    let mut base = 0x910000 | u32::from(word(rom, 0x91b010 + u32::from(pose) * 2)?);
    let frame = ram.word(SAMUS_ANIM_FRAME);
    if frame > 255 {
        return Err("animation frame outside admitted stream");
    }
    if rom_bytes(rom, base + u32::from(frame), 1)?[0] & 0x80 != 0 {
        let command = speed_booster_delay(ram, rom, base + u32::from(frame))?;
        if animation_command(ram, rom, base + u32::from(ram.word(SAMUS_ANIM_FRAME)), command)? {
            ram.set_word(
                SAMUS_ANIM_FRAME_TIMER,
                ram.word(SAMUS_ANIM_FRAME_BUFFER)
                    .wrapping_add(u16::from(rom_bytes(rom, base + u32::from(ram.word(SAMUS_ANIM_FRAME)), 1)?[0])),
            );
        }
    } else {
        if ram.word(SAMUS_HAS_MOMENTUM_FLAG) != 0 && ram.byte(SAMUS_MOVEMENT_TYPE) == 1 {
            let pointer = if ram.word(EQUIPPED_ITEMS) & 0x2000 != 0 {
                let stage = u32::from(ram.word(SPEED_BOOST_COUNTER) >> 8);
                if stage > 4 {
                    return Err("invalid speed booster stage");
                }
                word(rom, 0x91b5de + stage * 2)?
            } else {
                word(rom, 0x91b5d1)?
            };
            base = 0x910000 | u32::from(pointer);
        }
        ram.set_word(
            SAMUS_ANIM_FRAME_TIMER,
            ram.word(SAMUS_ANIM_FRAME_BUFFER).wrapping_add(u16::from(rom_bytes(rom, base + u32::from(frame), 1)?[0])),
        );
    }
    Ok(())
}
fn animate(ram: &mut Wram<'_>, rom: &[u8]) -> Result<(), &'static str> {
    match (ram.word(FX_TYPE) & 15) >> 1 {
        0 => no_fx(ram, rom)?,
        1 => lava_fx(ram, rom, false)?,
        2 => lava_fx(ram, rom, true)?,
        3 => water_fx(ram, rom)?,
        _ => {}
    }
    if [0x4d, 0x4e].contains(&ram.word(SAMUS_POSE))
        && ram.word(SAMUS_Y_DIR) != 2
        && ram.word(SAMUS_ANIM_FRAME) == 1
        && ram.word(SAMUS_ANIM_FRAME_TIMER) == 1
    {
        ram.set_word(SAMUS_ANIM_FRAME_TIMER, 4);
    }
    let timer = ram.word(SAMUS_ANIM_FRAME_TIMER).wrapping_sub(1);
    ram.set_word(SAMUS_ANIM_FRAME_TIMER, timer);
    if timer == 0 || negative(timer) {
        ram.set_word(SAMUS_ANIM_FRAME, ram.word(SAMUS_ANIM_FRAME).wrapping_add(1));
        handle_anim_delay(ram, rom)?;
    }
    Ok(())
}

pub fn dispatch_rom(ram: &mut Wram<'_>, rom: &[u8], symbol: &str, a: i32, _: i32) -> Result<u32, &'static str> {
    match symbol {
        "Samus_ClearXSpeedIfColl" => {
            if ram.word(SAMUS_COLLISION_FLAG) != 0 {
                ram.set_word(
                    SAMUS_COLLIDES_WITH_SOLID_ENEMY,
                    if ram.word(SAMUS_COLLISION_DIRECTION) != 0 { 8 } else { 4 },
                );
                cancel_boost(ram, rom)?;
                for off in [
                    SAMUS_X_EXTRA_RUN_SPEED,
                    SAMUS_X_EXTRA_RUN_SUBSPEED,
                    SAMUS_X_BASE_SPEED,
                    SAMUS_X_BASE_SUBSPEED,
                    SAMUS_X_ACCEL_MODE,
                ] {
                    ram.set_word(off, 0);
                }
            } else {
                ram.set_word(INPUT_TO_POSE_CALC, 0);
                ram.set_word(SAMUS_COLLIDES_WITH_SOLID_ENEMY, 0);
            }
        }
        "Samus_MoveUp_SetPoseCalcInput" => {
            ram.set_word(INPUT_TO_POSE_CALC, if ram.word(SAMUS_COLLISION_FLAG) != 0 { 4 } else { 0 })
        }
        "Samus_MoveDown_SetPoseCalcInput" => {
            const FALL: [u16; 28] =
                [0, 0, 4, 4, 1, 0, 4, 2, 4, 4, 0, 0, 0, 0, 4, 4, 0, 3, 4, 4, 4, 0, 4, 4, 4, 4, 4, 4];
            const LAND: [u16; 28] =
                [4, 4, 0, 0, 4, 4, 0, 4, 1, 2, 0, 4, 4, 0, 4, 4, 4, 4, 3, 3, 0, 4, 4, 4, 4, 0, 4, 4];
            let mode = usize::from(ram.byte(SAMUS_MOVEMENT_TYPE));
            if mode >= 28 {
                return Err("invalid movement type");
            }
            if ram.word(SAMUS_COLLISION_FLAG) != 0 {
                ram.set_word(INPUT_TO_POSE_CALC, 1 | (LAND[mode] << 8));
            } else if ram.word(INPUT_TO_POSE_CALC) as u8 != 5 {
                ram.set_word(INPUT_TO_POSE_CALC, 2 | (FALL[mode] << 8));
            }
        }
        "SamusCrouchingEtcFunc" => {
            ram.set_word(ENABLE_HORIZ_SLOPE_COLL, 3);
            ram.set_word(UNUSEDWORD_7E0AA4, 0);
        }
        "Samus_Animate" => animate(ram, rom)?,
        "Samus_Animate_NoFx" => no_fx(ram, rom)?,
        "Samus_Animate_WaterFx" => water_fx(ram, rom)?,
        "Samus_Animate_LavaFx" | "Samus_Animate_AcidFx" => lava_fx(ram, rom, symbol == "Samus_Animate_AcidFx")?,
        "Samus_Animate_SubmergedLavaAcid" => submerged_lava(ram, rom)?,
        "Samus_SpawnWaterSplash" => splash(ram, rom, a as u16)?,
        "Samus_SpawnAirBubbles" => bubbles(ram, rom)?,
        "Samus_LookupTransitionTable" => lookup_transition(ram, rom)?,
        "Samus_Pose_CancelGrapple" => cancel_grapple(ram, rom)?,
        "Samus_Pose_Func2" => return pose_momentum(ram),
        "Samus_CancelSpeedBoost" => cancel_boost(ram, rom)?,
        "CopyToSamusSuitPalette" | "CopyToSamusSuitTargetPalette" => {
            copy_palette(ram, rom, a as u16, symbol == "CopyToSamusSuitTargetPalette")?
        }
        "Samus_LoadSuitPalette" | "Samus_LoadSuitTargetPalette" => {
            copy_palette(ram, rom, suit_palette(ram), symbol == "Samus_LoadSuitTargetPalette")?
        }
        _ if symbol.starts_with("Samus_Input_") => match symbol {
            "Samus_Input_07_Unused"
            | "Samus_Input_09_Unused"
            | "Samus_Input_0B_Unused"
            | "Samus_Input_0F_CrouchingEtcTransition" => {}
            "Samus_Input_00_Standing" => {
                if (ram.word(SAMUS_POSE) != 0 && ram.word(SAMUS_POSE) != 0x9b) || ram.word(ELEVATOR_STATUS) == 0 {
                    lookup_transition(ram, rom)?;
                }
            }
            "Samus_Input_0A_KnockbackOrCrystalFlashEnding" => {
                lookup_transition(ram, rom)?;
                if ram.byte(SAMUS_MOVEMENT_TYPE) != 10 {
                    samus::dispatch_rom(ram, rom, "Samus_InitJump", 0, 0)?;
                    ram.set_word(SAMUS_KNOCKBACK_TIMER, 0);
                }
            }
            "Samus_Input_05_Crouching" | "Samus_Input_15_RanIntoWall" => {
                return Err("frozen input pose dependency not translated");
            }
            "Samus_Input_01_Running"
            | "Samus_Input_02_NormalJumping"
            | "Samus_Input_03_SpinJumping"
            | "Samus_Input_04_MorphBallOnGround"
            | "Samus_Input_06_Falling"
            | "Samus_Input_08_MorphBallFalling"
            | "Samus_Input_0C_Unused"
            | "Samus_Input_0D_Unused"
            | "Samus_Input_0E_TurningAroundOnGround"
            | "Samus_Input_10_Moonwalking"
            | "Samus_Input_11_SpringBallOnGround"
            | "Samus_Input_12_SpringBallInAir"
            | "Samus_Input_13_SpringBallFalling"
            | "Samus_Input_14_WallJumping"
            | "Samus_Input_16_Grappling"
            | "Samus_Input_17_TurningAroundJumping"
            | "Samus_Input_18_TurningAroundFalling"
            | "Samus_Input_19_DamageBoost"
            | "Samus_Input_1A_GrabbedByDraygon"
            | "Samus_Input_1B_ShinesparkEtc" => lookup_transition(ram, rom)?,
            _ => return Err("unknown Samus control routine"),
        },
        _ => return Err("unknown Samus control routine"),
    }
    Ok(0)
}
