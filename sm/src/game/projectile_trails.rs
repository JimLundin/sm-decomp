//! Projectile trail emission, placement, playback and checked OAM output.
use crate::game::projectile_animation::trail_frame;
use crate::game::sm_8b::CalcExplosion_Mode7;
use crate::game_state::Game;
use crate::ram_map::{checked_word as word, word_arrays as a};
use crate::rtl::{rom_bank_record, Unreachable};

#[derive(Clone, Copy)]
struct Side {
    timer: usize,
    script: usize,
    tile: usize,
    x: usize,
    y: usize,
}
const SIDES: [Side; 2] = [
    Side {
        timer: a::projectiletrail_left_instr_timer,
        script: a::projectiletrail_left_instr_list_ptr,
        tile: a::projectiletrail_left_tile_and_attribs,
        x: a::projectiletrail_left_x_pos,
        y: a::projectiletrail_left_y_pos,
    },
    Side {
        timer: a::projectiletrail_right_instr_timer,
        script: a::projectiletrail_right_instr_list_ptr,
        tile: a::projectiletrail_right_tile_and_attribs,
        x: a::projectiletrail_right_x_pos,
        y: a::projectiletrail_right_y_pos,
    },
];

pub unsafe fn emit(projectile: u16, game: &mut Game) {
    unsafe {
        let kind = word!(game.projectile_type[projectile >> 1]);
        let style = if kind & 0xf00 != 0 {
            let class = kind.to_ne_bytes()[1] & 15;
            if class >= 3 {
                return;
            }
            u16::from(class) + 31
        } else {
            kind & 63
        };
        let Some(pair) = (0..18)
            .rev()
            .find(|&i| word!(game.projectiletrail_left_instr_timer[i]) == 0)
        else {
            return;
        };
        word!(game.projectiletrail_left_instr_timer[pair] = 1);
        word!(game.projectiletrail_right_instr_timer[pair] = 1);
        word!(
            game.projectiletrail_left_instr_list_ptr[pair] =
                crate::rtl::rom_table_word!(game, 0x90b5bb, style as isize)
        );
        word!(
            game.projectiletrail_right_instr_list_ptr[pair] =
                crate::rtl::rom_table_word!(game, 0x90b609, style as isize)
        );
        place(word!(game.projectile_index), (2 * pair) as u16, game);
    }
}

pub unsafe fn place(projectile: u16, pair: u16, game: &mut Game) {
    unsafe {
        let marker = trail_frame(projectile, game);
        let index = usize::from(projectile >> 1);
        let (x, y) = if word!(game.ceres_status) & 0x8000 == 0 {
            (
                word!(game.projectile_x_pos[index]),
                word!(game.projectile_y_pos[index]),
            )
        } else {
            let point = CalcExplosion_Mode7(projectile, game);
            (
                word!(game.layer1_x_pos).wrapping_add(point.x),
                word!(game.layer1_y_pos).wrapping_add(point.y),
            )
        };
        let kind = word!(game.projectile_type[index]);
        let table = if kind & 0x20 != 0 {
            0x9ba4e3
        } else if kind & 0x10 != 0 {
            0x9ba4cb
        } else {
            0x9ba4b3
        };
        let first = crate::rtl::rom_table_word!(game, table, (kind & 15) as isize)
            .wrapping_add(2 * (word!(game.projectile_dir[index]) & 15));
        let second = rom_bank_record::<0x9b>(first, game)
            .word(0)
            .wrapping_add(marker.wrapping_mul(4));
        let data = rom_bank_record::<0x9b>(second, game);
        let index = usize::from(pair >> 1);
        let coordinate =
            |origin: u16, offset: u8| origin.wrapping_add(offset as i8 as u16).wrapping_sub(4);
        // Preserve the per-field read/write order, including prefixes on short reads.
        word!(game.projectiletrail_left_y_pos[index] = coordinate(y, data.byte(1)));
        word!(game.projectiletrail_left_x_pos[index] = coordinate(x, data.byte(0)));
        word!(game.projectiletrail_right_y_pos[index] = coordinate(y, data.byte(3)));
        word!(game.projectiletrail_right_x_pos[index] = coordinate(x, data.byte(2)));
    }
}

unsafe fn advance(pair: usize, side: usize, game: &mut Game) -> bool {
    unsafe {
        let fields = SIDES[side];
        let remaining = word!(game, fields.timer + 2 * pair;);
        if remaining == 0 {
            return false;
        }
        word!(game, fields.timer + 2 * pair; = remaining.wrapping_sub(1));
        if remaining != 1 {
            return true;
        }
        let mut cursor = i32::from(word!(game, fields.script + 2 * pair;));
        let duration = loop {
            let value = rom_bank_record::<0x90>(cursor as u16, game).word(0);
            if value & 0x8000 == 0 {
                break value;
            }
            match value {
                0xb525 => word!(game.projectiletrail_left_y_pos[pair] += 1),
                0xb587 => word!(game.projectiletrail_right_y_pos[pair] += 1),
                0xb5b3 => word!(game.projectiletrail_left_y_pos[pair] += u16::MAX),
                _ => {
                    Unreachable(game);
                }
            }
            cursor = if side == 0 {
                cursor + 2
            } else {
                i32::from((cursor as u16).wrapping_add(2))
            };
        };
        word!(game, fields.timer + 2 * pair; = duration);
        if duration == 0 {
            return false;
        }
        let tile = rom_bank_record::<0x90>(cursor as u16, game).word(2);
        word!(game, fields.tile + 2 * pair; = tile);
        word!(game, fields.script + 2 * pair; = (cursor + 4) as u16);
        true
    }
}

#[forbid(unsafe_code)]
fn native_word(game: &mut Game, address: usize, value: u16) {
    game.g_ram[address..address + 2].copy_from_slice(&value.to_ne_bytes());
}

#[forbid(unsafe_code)]
fn draw(pair: usize, side: usize, frozen: bool, game: &mut Game) {
    let saved = word!(game.oam_next_ptr);
    if (saved as i16).wrapping_sub(512) >= 0 {
        return;
    }
    let fields = SIDES[side];
    if frozen && word!(game, fields.timer + 2 * pair;) == 0 {
        return;
    }
    crate::verify_probe::hit(match (frozen, side) {
        (true, 0) => "trails.draw.frozen_left",
        (true, _) => "trails.draw.frozen_right",
        (false, 0) => "trails.draw.active_left",
        (false, _) => "trails.draw.active_right",
    });
    let x = word!(game, fields.x + 2 * pair;).wrapping_sub(word!(game.layer1_x_pos));
    if x & 0xff00 == 0 {
        let entry = 0x370 + usize::from(word!(game.oam_next_ptr));
        native_word(game, entry, x);
        // The first write can alias the live Y, camera, tile or cursor fields.
        let y = word!(game, fields.y + 2 * pair;).wrapping_sub(word!(game.layer1_y_pos));
        if y & 0xff00 == 0 {
            native_word(game, entry + 1, y);
            let tile = word!(game, fields.tile + 2 * pair;);
            native_word(game, entry + 2, tile);
            word!(game.oam_next_ptr = saved.wrapping_add(4));
        }
    }
}

pub unsafe fn update(game: &mut Game) {
    unsafe {
        crate::verify_probe::hit("trails.update");
        let frozen = word!(game.time_is_frozen_flag) != 0;
        for pair in (0..18).rev() {
            for side in 0..2 {
                if frozen || advance(pair, side, game) {
                    draw(pair, side, frozen, game);
                }
            }
        }
    }
}
