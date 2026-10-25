//! Ceres and Tourian escape countdowns, movement and sprite output.
#![forbid(unsafe_code)]
use crate::game::{projectile_presentation_ram as sprite_output, sprite_drawing::Clip};
use crate::game_state::Game;
use crate::ram_map::word;

pub(crate) fn advance(game: &mut Game) -> u8 {
    const STATES: [fn(&mut Game) -> u8; 7] = [
        |_| 0,
        |game| start(0x100, game),
        |game| start(0x300, game),
        |game| delay(0x10, false, game),
        |game| delay(0x60, true, game),
        move_into_place,
        tick,
    ];
    STATES[word!(game.timer_status) as u8 as usize](game)
}

fn set_digits(minutes: u16, game: &mut Game) {
    // These native words overlap at the seconds byte; retain both ordered writes.
    game.g_ram[0x945..0x947].copy_from_slice(&0u16.to_ne_bytes());
    game.g_ram[0x946..0x948].copy_from_slice(&minutes.to_ne_bytes());
}

pub(crate) fn clear(game: &mut Game) {
    word! { game, 0x8000; timer_x_pos, timer_y_pos }
    set_digits(0, game);
    word!(game.timer_status = 0);
}

fn start(minutes: u16, game: &mut Game) -> u8 {
    clear(game);
    set_digits(minutes, game);
    word!(game.timer_status = 0x8003);
    0
}

fn delay(limit: u8, count_down: bool, game: &mut Game) -> u8 {
    game.g_ram[0x948] = (word!(game.timer_x_pos) as u8).wrapping_add(1);
    if word!(game.timer_x_pos) as u8 >= limit {
        if count_down {
            game.g_ram[0x948] = 0;
        }
        game.g_ram[0x943] = (word!(game.timer_status) as u8).wrapping_add(1);
    }
    if count_down { tick(game) } else { 0 }
}

fn move_into_place(game: &mut Game) -> u8 {
    let x = word!(game.timer_x_pos).wrapping_add(224);
    let x_ready = x >= 0xdc00;
    word!(game.timer_x_pos = if x_ready { 0xdc00 } else { x });
    let y = word!(game.timer_y_pos).wrapping_sub(193);
    let y_ready = y < 0x3000;
    word!(game.timer_y_pos = if y_ready { 0x3000 } else { y });
    if x_ready && y_ready {
        word!(game.timer_status += 1);
    }
    tick(game)
}

fn decrement_decimal(number: &mut u8, value: u8) -> bool {
    let mut result = (*number as i32 & 0xf) + (!(value as i32) & 0xf) + 1;
    if result < 0x10 {
        result = result - 0x6 & (if result - 0x6 < 0 { 0xf } else { 0x1f });
    }
    result = (*number as i32 & 0xf0) + (!(value as i32) & 0xf0) + result;
    if result < 0x100 {
        result -= 0x60;
    }
    *number = result as u8;
    result < 0x100
}

fn tick(game: &mut Game) -> u8 {
    const CENTISECOND_DECREMENTS: [u8; 128] = [
        1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 1, 2, 2, 1, 2, 2,
        1, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 1, 2, 2, 1,
        2, 2, 1, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 1, 2,
        2, 1, 2, 2, 1, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2,
        2, 1, 2, 2, 2, 1, 2, 2,
    ];
    let amount = CENTISECOND_DECREMENTS[usize::from(word!(game.nmi_frame_counter_word) & 0x7f)];
    if decrement_decimal(&mut game.g_ram[0x945], amount)
        && decrement_decimal(&mut game.g_ram[0x946], 1)
    {
        if decrement_decimal(&mut game.g_ram[0x947], 1) {
            game.g_ram[0x945] = 0;
            game.g_ram[0x946] = 0;
            game.g_ram[0x947] = 0;
        } else {
            game.g_ram[0x946] = 0x59;
        }
    }
    ((game.g_ram[0x945] | game.g_ram[0x946] | game.g_ram[0x947]) == 0) as u8
}

pub(crate) fn draw(game: &mut Game) {
    draw_sprite(0, 0xa060, game);
    for (at, offset) in [(0x947, 0xffe4u16), (0x946, 0xfffc), (0x945, 0x14)] {
        // The overlapping native word is read after the preceding sprite output.
        let value = u16::from_ne_bytes([game.g_ram[at], game.g_ram[at + 1]]);
        for (shift, x) in [(4, offset), (0, offset.wrapping_add(8))] {
            let map =
                crate::rtl::rom_table_word!(game, 0x809fd4, ((value >> shift) & 0xf) as isize);
            draw_sprite(x, map, game);
        }
    }
}

fn draw_sprite(offset: u16, map: u16, game: &mut Game) {
    sprite_output::paletted(
        0x80,
        map,
        (
            offset.wrapping_add(u16::from(game.g_ram[0x949])),
            u16::from(game.g_ram[0x94b]),
        ),
        0xa00,
        Clip::Bounded,
        game,
    );
}
