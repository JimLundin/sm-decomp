//! Room, X-ray and Power Bomb layer/window composition.
use crate::game::constants::bank_8f::{addr_kRoom_a66a, addr_kRoom_cefb};
use crate::game::prelude::*;

// W12SEL, W34SEL, WOBJSEL, TM, TS, TMW, TSW, CGWSEL and CGADSUB shadows.
fn apply(settings: [u8; 9], game: &mut Game) {
    for (address, value) in [0x60, 0x61, 0x62, 0x69, 0x6b, 0x6c, 0x6d, 0x6e, 0x71]
        .into_iter()
        .zip(settings)
    {
        game.g_ram[address] = value;
    }
}

// Ordered changes after the common room defaults, indexed by the configuration's low byte / 2.
const ROOM_SETTINGS: [&[(usize, u8)]; 27] = [
    &[],
    &[],
    &[(0x69, 17), (0x6b, 4)],
    &[(0x6b, 20), (0x71, 39)],
    &[(0x6b, 20), (0x71, 34)],
    &[(0x71, 50)],
    &[(0x6e, 0), (0x71, 162)],
    &[],
    &[(0x61, 2), (0x62, 32), (0x6d, 4)],
    &[(0x61, 2), (0x62, 32), (0x6d, 4)],
    &[(0x71, 179)],
    &[(0x69, 17), (0x6b, 6), (0x71, 177)],
    &[],
    &[(0x69, 21), (0x6b, 2), (0x71, 53)],
    &[(0x69, 21), (0x6b, 2), (0x71, 85)],
    &[],
    &[],
    &[(0x71, 179)],
    &[
        (0x60, 0),
        (0x61, 2),
        (0x62, 32),
        (0x69, 19),
        (0x6b, 4),
        (0x6c, 19),
        (0x6d, 4),
        (0x6e, 16),
        (0x71, 51),
    ],
    &[(0x71, 119)],
    &[(0x6e, 0), (0x71, 179)],
    &[(0x6e, 0), (0x71, 179)],
    &[(0x6e, 0)],
    &[(0x71, 179)],
    &[],
    &[(0x6b, 68), (0x71, 178)],
    &[],
];

pub unsafe fn update(game: &mut Game) {
    unsafe {
        let configuration = word!(game.fx_layer_blending_config_c);
        let mut bomb_mode = 0;
        if configuration as u8 != 0 {
            apply([0, 0, 0, 19, 4, 0, 0, 2, 51], game);
            let mode = (configuration as u8 >> 1) as usize;
            // Index after initialization: an invalid mode leaves the original defaults written.
            for &(address, value) in ROOM_SETTINGS[mode] {
                game.g_ram[address] = value;
            }
            bomb_mode = match mode {
                11 | 13 => 4,
                12 | 15 | 24 => {
                    if game.g_ram[0x84] & 0x30 == 0x30 {
                        game.g_ram[0x71] = 36;
                        game.g_ram[0x6b] = 19;
                        game.g_ram[0x69] = 4;
                    }
                    2
                }
                20 | 21 if configuration & 0x8000 == 0 => {
                    game.g_ram[0x74] = if mode == 20 { 37 } else { 38 };
                    game.g_ram[0x75] = if mode == 20 { 64 } else { 66 };
                    game.g_ram[0x76] = 0x80;
                    0
                }
                26 => 6,
                _ => 0,
            };
        }
        if configuration & 0x8000 != 0 {
            if word!(game.room_ptr) == addr_kRoom_a66a as u16 {
                bomb_mode = 6;
            }
            let special = bomb_mode == 4;
            // Power Bombs write the color-math settings before the screen/window masks.
            game.g_ram[0x60] = if special { 0x80 } else { 0 };
            game.g_ram[0x61] = 8;
            game.g_ram[0x62] = 0x80;
            game.g_ram[0x6e] = 2;
            game.g_ram[0x71] = if bomb_mode == 6 { 49 } else { 55 };
            game.g_ram[0x6c] = 0;
            game.g_ram[0x6d] = if special { 6 } else { 4 };
            game.g_ram[0x69] = if special { 17 } else { 19 };
            game.g_ram[0x6b] = if special { 6 } else { 4 };
        } else if configuration & 0x4000 != 0 {
            apply(
                [200, 8, 128, 19, 4, 3, 4, 34, game.g_ram[0x71] & 0x80 | 0x73],
                game,
            );
        } else if configuration & 0x2000 != 0 {
            apply(
                [0, 8, 128, 19, 4, 3, 4, 34, game.g_ram[0x71] & 0x80 | 0x61],
                game,
            );
            if word!(game.room_ptr) == addr_kRoom_cefb as u16 {
                game.g_ram[0x69] = 17;
            }
        } else if configuration & 0x1000 != 0 {
            apply([0, 8, 128, 19, 0, 3, 4, 32, 179], game);
        }
    }
}
