//! Draygon's mirrored sweep shares its path builder and every table consumer.
//! The native table remains authoritative and may alias other compatibility state.
#![forbid(unsafe_code)]
use crate::game::actors::draygon_flight::{displace, turret};
use crate::game::enemy_layouts::EnemyLayout104 as Draygon;
use crate::game::sprite_effect_runtime::draygon_body_bubble;
use crate::game_state::Game;
use crate::ram_map::{slot, word};
use crate::rtl::abs16;

fn write_height(cursor: u16, value: u16, game: &mut Game) {
    let at = 0x9002 + usize::from(cursor);
    let bytes = value.to_ne_bytes();
    game.ram().write_byte(at, bytes[0]);
    game.ram().write_byte(at + 1, bytes[1]);
}
fn build_path(game: &mut Game) {
    let (mut height, mut velocity, mut cursor) = (384u16, 0u16, 0u16);
    loop {
        velocity = velocity.wrapping_add(slot!(game, Draygon[0].var_0F));
        let next = height.wrapping_sub(u16::from(velocity.to_ne_bytes()[1]));
        if (next as i16).wrapping_sub(slot!(game, Draygon[0].var_01) as i16) < 0 {
            break;
        }
        height = next;
        write_height(cursor, next, game);
        cursor = cursor.wrapping_add(4);
        if cursor.wrapping_sub(2048) & 0x8000 == 0 {
            crate::game::animated_tiles::native::unreachable(game);
        }
    }
    let distance = abs16(slot!(game, Draygon[0].var_00).wrapping_sub(word!(game.samus_x_pos)));
    let speed = (u32::from(distance) << 16)
        .checked_div(u32::from(cursor >> 2))
        .unwrap_or(0);
    slot!(game, Draygon[0](var_D, var_E) = speed);
    write_height(cursor, slot!(game, Draygon[0].base.y_pos), game);
    slot!(game, Draygon[0]; var_B = cursor, var_C = cursor);
}
pub(crate) fn begin(right: bool, game: &mut Game) {
    if right {
        turret(game);
        build_path(game);
    } else {
        // The bank retains the original diagnostic before this zero-divisor setup.
        let distance = abs16(slot!(game, Draygon[0].var_02).wrapping_sub(word!(game.samus_x_pos)));
        let speed = (u32::from(distance) << 16).checked_div(0).unwrap_or(0);
        slot!(game, Draygon[0](var_D, var_E) = speed);
    }
    slot!(game, Draygon[0].var_A = if right { 0x88b1 } else { 0x8a00 });
    slot!(game, Draygon[0].base; current_instruction = if right { 0x97d1 } else { 0x97bb }, instruction_timer = 1);
    slot!(game, Draygon[0].var_20 = u16::from(right));
    if !right {
        slot!(game, Draygon[0].var_B = slot!(game, Draygon[0].var_C));
    }
}
pub(crate) fn turn(right: bool, game: &mut Game) {
    turret(game);
    let count = u32::from(slot!(game, Draygon[0].var_C) >> 2);
    let x = slot!(game, Draygon[0].base.x_pos) as i32;
    let distance = if right {
        672 - x
    } else {
        x - slot!(game, Draygon[0].var_00) as i32
    };
    let speed = ((distance << 16) as u32).checked_div(count).unwrap_or(0);
    if right {
        slot!(game, Draygon[0](var_D, var_E) = speed);
    } else {
        // The left turn writes high then low; other speed setters write low first.
        slot!(game, Draygon[0]; var_D = (speed >> 16) as u16, var_E = speed as u16);
    }
    slot!(game, Draygon[0].var_A = if right { 0x8951 } else { 0x8a90 });
}
pub(crate) fn advance(right: bool, returning: bool, game: &mut Game) {
    turret(game);
    if !right || !returning {
        draygon_body_bubble(game);
    }
    let cursor = slot!(game, Draygon[0].var_B);
    if cursor == 104 {
        let instruction = match (right, returning) {
            (true, false) => 0x9c06,
            (true, true) => 0x9bda,
            (false, false) => 0x9813,
            (false, true) => 0x97e7,
        };
        slot!(game, Draygon[0xc0].base; current_instruction = instruction, instruction_timer = 1);
    }
    let at = 0x9002 + usize::from(cursor);
    let height = u16::from_ne_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)]);
    slot!(
        game,
        Draygon[word!(game.cur_enemy_index) as i32].base.y_pos = height
    );
    // A broad selected slot can alias the body's cursor: reread after its Y store.
    let cursor = slot!(game, Draygon[0].var_B);
    let next = if returning {
        cursor.wrapping_add(4)
    } else {
        cursor.wrapping_sub(4)
    };
    slot!(game, Draygon[0].var_B = next);
    if !returning {
        if next == 0 {
            slot!(game, Draygon[0].var_A = if right { 0x8922 } else { 0x8a50 });
        } else {
            displace(right, game);
        }
    } else if next != slot!(game, Draygon[0].var_C) {
        displace(right, game);
    } else if right {
        slot!(game, Draygon[0].var_B = 0);
        slot!(
            game,
            Draygon[0].var_A = if word!(game.random_number) & 1 != 0 {
                0x89b3
            } else {
                0x8c8e
            }
        );
    } else if word!(game.random_number) & 1 != 0 {
        slot!(game, Draygon[0]; var_A = 0x87f4, var_B = 0);
        slot!(game, Draygon[0].base; x_pos = slot!(game, Draygon[0].var_00), y_pos = slot!(game, Draygon[0].var_01));
    } else {
        slot!(game, Draygon[0].var_A = 0x8b0a);
        slot!(game, Draygon[0].base.x_pos = slot!(game, Draygon[0].var_00));
    }
}
