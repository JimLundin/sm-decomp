//! Draygon's horizontal approach, firing pass and exit share one glide update.
//! Extended motion words stay native compatibility state until the whole boss migrates.
#![forbid(unsafe_code)]
use crate::actors::draygon_host::{Host, abs16};
use crate::actors::layouts::EnemyLayout104 as Draygon;
use crate::enemy_script::EnemyScript;
use crate::ram::RawRam as _;
use crate::ram::{slot, word};

pub fn facing_animation<H: Host + ?Sized>(
    facing_key: i32,
    part_key: i32,
    instructions: [u16; 2],
    game: &mut H,
) {
    let instruction = instructions[usize::from(slot!(game, Draygon[facing_key].var_20) != 0)];
    slot!(game, Draygon[part_key].base; current_instruction = instruction, instruction_timer = 1);
}
pub fn shed_particle<H: Host + ?Sized>(k: u16, frame_mask: u16, game: &mut H) {
    if slot!(game, Draygon[k as i32].base.frame_counter) & frame_mask == 0 {
        let dx = if slot!(game, Draygon[0].var_20) != 0 {
            32
        } else {
            0u16.wrapping_sub(32)
        };
        let x = slot!(game, Draygon[0].base.x_pos).wrapping_add(dx);
        let y = slot!(game, Draygon[0].base.y_pos).wrapping_sub(16);
        game.create_sprite(x, y, 61, 0);
    }
}
pub fn turret<H: Host + ?Sized>(game: &mut H) {
    if word!(game.nmi_frame_counter_word) & 0x3f == 0 {
        let index = 2 * ((game.next_random() & 3) + 2);
        if slot!(game, Draygon[index as i32].var_40) == 0 {
            let x = crate::cartridge::rom_table_word!(game, 0xa587dc, index as isize);
            let y = crate::cartridge::rom_table_word!(game, 0xa587dc, index as isize + 1);
            game.set_projectile_spawn_point(x, y);
            game.spawn_projectile(3, index, 0x8e5e);
        }
    }
}
pub fn opening<H: Host + ?Sized>(first: bool, game: &mut H) {
    turret(game);
    let limit = if first { 256 } else { 1232 };
    if slot!(game, Draygon[0].var_B).wrapping_sub(limit) & 0x8000 != 0 {
        if first {
            if slot!(game, Draygon[0].var_B) == 0 {
                let at = 0xd0 + usize::from(word!(game.vram_write_queue_tail));
                // The native bank word deliberately overlaps the destination's low byte.
                game.write_native_words(&[
                    (at, 1536),
                    (at + 2, 0x9400),
                    (at + 4, 177),
                    (at + 5, 27904),
                ]);
                word!(game.vram_write_queue_tail += 7);
                game.draygon_spawn_trails();
            }
        } else {
            game.draygon_trajectory();
        }
        slot!(game, Draygon[0].var_B, |old| old.wrapping_add(1));
    } else {
        slot!(game, Draygon[0].var_A = if first { 0x878b } else { 0x87f4 });
        slot!(game, Draygon[0].var_B = 0);
        if first {
            let x = slot!(game, Draygon[0].base.x_pos);
            slot!(game, Draygon[0].var_00 = x);
            slot!(game, Draygon[0].var_02 = x.wrapping_add(672));
            slot!(game, Draygon[0].var_01 = slot!(game, Draygon[0].base.y_pos));
            slot!(game, Draygon[0].base.x_pos = slot!(game, Draygon[0].var_00));
            slot!(game, Draygon[0].base.y_pos = slot!(game, Draygon[0].var_01));
            slot!(game, Draygon[0].var_0F = 24);
        }
    }
}
/// Actor preparation and completion surround the existing synchronous HDMA install seam.
pub fn initialize<H: Host + ?Sized>(install_layers: impl FnOnce(&mut H), game: &mut H) {
    crate::word_transfers!(game, descending, 25; le(target_palettes, 168) <- rom(0xa5a217, 24));
    crate::word_transfers!(game, descending, 2048; le(tilemap_stuff, 2047) <- value(824));
    slot!(
        game,
        Draygon[word!(game.cur_enemy_index) as i32]
            .base
            .palette_index = 0xe00
    );
    // The native bank word also clears byte178F before the address word is stored.
    crate::word_transfers!(game, ascending, 1; native(0x178e) <- value(165), native(0x178c) <- value(0x9342));
    word!(game.flag_process_all_enemies = 0xffff);
    for (k, instruction) in [(0, 0x9889), (0x40, 0x9944), (0x80, 0x99fc), (0xc0, 0x9813)] {
        slot!(game, Draygon[k].base.current_instruction = instruction);
    }
    word!(game.enemy_bg2_tilemap_size = 1024);
    slot!(
        game,
        Draygon[word!(game.cur_enemy_index) as i32].var_A = 0x871b
    );
    install_layers(game);
    slot!(game, Draygon[0].var_45 = 1);
    game.disable_minimap_for_boss();
}
pub fn initialize_part<H: Host + ?Sized>(arms: bool, game: &mut H) {
    let k = word!(game.cur_enemy_index) as i32;
    let instruction = if arms { 0x97e7 } else { 0x99fc };
    slot!(game, Draygon[k].base; current_instruction = instruction, palette_index = 0xe00);
    if arms {
        slot!(game, Draygon[k].base.layer = 2);
    }
}
pub fn update<H: Host + ?Sized>(body: bool, dispatch: impl FnOnce(u32, u16, &mut H), game: &mut H) {
    let k = word!(game.cur_enemy_index) as i32;
    let state = 0xa50000 | u32::from(slot!(game, Draygon[k].var_A));
    dispatch(state, word!(game.cur_enemy_index), game);
    if !body {
        return;
    }
    let x = slot!(game, Draygon[0].base.x_pos);
    for k in [0x40, 0x80, 0xc0] {
        slot!(game, Draygon[k].base.x_pos = x);
    }
    let y = slot!(game, Draygon[0].base.y_pos);
    for k in [0x40, 0x80, 0xc0] {
        slot!(game, Draygon[k].base.y_pos = y);
    }
}
pub fn scroll_layer<H: Host + ?Sized>(game: &mut H) {
    let x = i32::from(slot!(game, Draygon[0].var_5E)) + i32::from(word!(game.layer1_x_pos))
        - i32::from(slot!(game, Draygon[0].base.x_pos))
        - 450;
    word!(game.reg_BG2HOFS = x as u16);
    let y = i32::from(slot!(game, Draygon[0].var_5F)) + i32::from(word!(game.layer1_y_pos))
        - i32::from(slot!(game, Draygon[0].base.y_pos))
        - 192;
    word!(game.reg_BG2VOFS = y as u16);
}
fn glide<H: Host + ?Sized>(right: bool, game: &mut H) {
    let phase = slot!(game, Draygon[0].var_0D);
    slot!(
        game,
        Draygon[0].base.y_pos = game.actor_cosine(phase, 32).wrapping_add(384)
    );
    slot!(game, Draygon[0].var_0D, |old| (old as u8).wrapping_add(1)
        as u16);
    displace(right, game);
}
pub fn displace<H: Host + ?Sized>(right: bool, game: &mut H) {
    let high = slot!(game, Draygon[0].var_D);
    let low = slot!(game, Draygon[0].var_E);
    let speed = (u32::from(high) << 16) | u32::from(low);
    let delta = if right {
        speed
    } else {
        (-(speed as i32)) as u32
    };
    slot!(game, Draygon[0].base(x_pos, x_subpos) += delta);
}
fn beyond_right(x: u16) -> bool {
    x as i16 >= 0 && x.wrapping_sub(672) & 0x8000 == 0
}
pub fn begin<H: Host + ?Sized>(right: bool, game: &mut H) {
    if right {
        turret(game);
    }
    slot!(
        game,
        Draygon[0].base.x_pos = if right {
            0xffb0
        } else {
            slot!(game, Draygon[0].var_02)
        }
    );
    slot!(game, Draygon[0].base.y_pos = 384);
    slot!(game, Draygon[0]; var_E = 0, var_D = 1, var_0D = 0);
    slot!(game, Draygon[0xc0].base; current_instruction = 0x9c06, instruction_timer = 1);
    slot!(game, Draygon[0].var_A = if right { 0x8b52 } else { 0x8cd4 });
    slot!(game, Draygon[0].base; current_instruction = if right { 0x97d1 } else { 0x97bb }, instruction_timer = 1);
    slot!(game, Draygon[0].var_20 = u16::from(right));
}
pub fn approach<H: Host + ?Sized>(right: bool, game: &mut H) {
    turret(game);
    game.draygon_body_bubble();
    let distance = abs16(slot!(game, Draygon[0].base.x_pos).wrapping_sub(word!(game.samus_x_pos)));
    if distance.wrapping_sub(208) & 0x8000 != 0 {
        slot!(game, Draygon[0]; var_A = if right { 0x8bae } else { 0x8d30 }, var_03 = 16);
    } else {
        glide(right, game);
    }
}
pub fn fire<H: Host + ?Sized>(right: bool, game: &mut H) {
    if right {
        turret(game);
    }
    if word!(game.samus_x_speed_divisor) == 0 {
        let mut leave = false;
        if word!(game.random_number) & 0xf == 0 {
            let remaining = slot!(game, Draygon[0].var_03).wrapping_sub(1);
            slot!(game, Draygon[0].var_03 = remaining);
            leave = remaining == 0;
            if !leave {
                slot!(game, Draygon[0].base; current_instruction = if right { 0x9c90 } else { 0x98fe }, instruction_timer = 1);
            }
        }
        if !leave {
            glide(right, game);
            // State 20 tests unsigned x_pos < 0, so its left-edge test never
            // succeeds. State 21 below intentionally uses the signed comparison.
            let x = slot!(game, Draygon[0].base.x_pos);
            if !right || !beyond_right(x) {
                return;
            }
        }
        if word!(game.samus_x_speed_divisor) == 0 {
            slot!(game, Draygon[0xc0].base; current_instruction = if right { 0x9bda } else { 0x97e7 }, instruction_timer = 1);
            slot!(game, Draygon[0].var_A = if right { 0x8c33 } else { 0x8db2 });
            return;
        }
    }
    slot!(game, Draygon[0].var_A = 0x8e19);
}
pub fn exit<H: Host + ?Sized>(right: bool, game: &mut H) {
    game.draygon_body_bubble();
    if !right && word!(game.samus_x_speed_divisor) != 0 {
        slot!(game, Draygon[0].var_A = 0x8e19);
        return;
    }
    glide(right, game);
    let x = slot!(game, Draygon[0].base.x_pos);
    let finished = if right {
        beyond_right(x)
    } else {
        (x as i16) < 0 && x.wrapping_add(80) & 0x8000 != 0
    };
    if finished {
        slot!(game, Draygon[0].var_A = if right { 0x89b3 } else { 0x87f4 });
        slot!(game, Draygon[0].base; x_pos = if right { 592 } else { 0xffb0 }, y_pos = 0xffb0);
        slot!(game, Draygon[0].var_B = slot!(game, Draygon[0].var_C));
    }
}

pub fn pose<H: Host + ?Sized>(script: EnemyScript, game: &mut H) -> EnemyScript {
    for (k, offset) in [(0, 0), (0x40, 2), (0x80, 4), (0xc0, 6)] {
        slot!(
            game,
            Draygon[k].base.current_instruction = script.word(offset)
        );
    }
    for k in [0, 0x40, 0x80, 0xc0] {
        slot!(game, Draygon[k].base.instruction_timer = 1);
    }
    script.offset(4)
}
pub fn layer_offsets<H: Host + ?Sized>(script: EnemyScript, game: &mut H) -> EnemyScript {
    slot!(game, Draygon[0].var_5E = script.word(0));
    slot!(game, Draygon[0].var_5F = script.word(2));
    script.offset(2)
}
pub fn hide_parts<H: Host + ?Sized>(script: EnemyScript, game: &mut H) -> EnemyScript {
    for k in [0x80, 0xc0] {
        slot!(game, Draygon[k].base; instruction_timer = 1, current_instruction = 0x97b9);
    }
    script
}
pub fn tangible<H: Host + ?Sized>(script: EnemyScript, game: &mut H) -> EnemyScript {
    slot!(game, Draygon[0].base.properties, |old| old | 0x400);
    script
}
