//! Real maintained enemy controller fixtures; no replacement controller logic.
#![forbid(unsafe_code)]
use super::sprite_effect_cases::{Operation, put, word};
use crate::{game_state::Game, ram_map::words as w, rom::RomImage, types::EnemyBase};
use core::mem::offset_of;
use sm_game::actors::layouts::{EnemyLayout40 as Metroid, EnemyLayout104 as Boss};
fn field(g: &mut Game, offset: usize, value: u16) {
    put(g, 0xf78 + offset, value);
}
pub(crate) fn seed(id: u32, g: &mut Game, image: &RomImage) -> Operation {
    put(g, w::CUR_ENEMY_INDEX, 0);
    field(g, offset_of!(EnemyBase, x_pos), 0x234);
    field(g, offset_of!(EnemyBase, y_pos), 0x345);
    field(g, offset_of!(EnemyBase, vram_tiles_index), 0x155);
    field(g, offset_of!(EnemyBase, palette_index), 0xa00);
    field(g, offset_of!(Metroid, var_00), 62);
    field(g, offset_of!(Metroid, var_01), 60);
    match id {
        80..=83 => {
            if id >= 81 {
                for slot in 0..32 {
                    put(g, 0xef78 + 2 * slot, 0x9000);
                }
            }
            if id == 81 {
                put(g, 0xef78 + 62, 0);
            }
            if id == 83 {
                put(g, 0xef78 + 10, 0);
                put(g, 0xef78 + 42, 0);
            }
            Operation::MetroidInit
        }
        84..=87 => {
            field(
                g,
                offset_of!(EnemyBase, flash_timer),
                if id & 1 == 0 { 2 } else { 0 },
            );
            if id >= 86 {
                field(g, offset_of!(Metroid, var_00), 0x80);
                field(g, offset_of!(Metroid, var_01), 0x40);
            }
            put(g, 0xf0f8, 0xf35d);
            put(g, 0xf0b8, 0x8c3a);
            Operation::MetroidHurt
        }
        88..=91 => {
            field(g, offset_of!(Metroid, var_f), 2); // Actual Func3 follows Samus without collision traversal.
            put(g, w::SAMUS_X_POS, 0x112);
            put(g, w::SAMUS_Y_POS, 0x220);
            let handles = [(62, 60), (0x40, 0x80), (0xffff, 0xfffd), (3, 5)][(id - 88) as usize];
            field(g, offset_of!(Metroid, var_00), handles.0);
            field(g, offset_of!(Metroid, var_01), handles.1);
            Operation::MetroidMain
        }
        92..=95 => {
            field(
                g,
                offset_of!(EnemyBase, frozen_timer),
                [0, 1, 90, 200][(id - 92) as usize],
            );
            field(g, offset_of!(EnemyBase, ai_handler_bits), 0x14);
            field(
                g,
                offset_of!(Metroid, var_e),
                if id & 1 == 0 { 0 } else { 1 },
            );
            put(g, w::EQUIPPED_BEAMS, if id & 1 == 0 { 0 } else { 2 });
            if id == 95 {
                field(g, offset_of!(Metroid, var_00), 0xffff);
                field(g, offset_of!(Metroid, var_01), 0xfffd);
            }
            Operation::MetroidFrozen
        }
        96..=103 => {
            field(g, offset_of!(Boss, var_45), 0xa00);
            field(
                g,
                offset_of!(Boss, var_46),
                [62, 0x40, 0x80, 0xffff][((id - 96) % 4) as usize],
            );
            field(
                g,
                offset_of!(EnemyBase, frozen_timer),
                [0, 1, 2, 89, 90, 91, 0xffff, 88][(id - 96) as usize],
            );
            Operation::MawAppearance
        }
        104..=107 => {
            for i in 0..12 {
                word(
                    image,
                    0xa5a1af + 2 * i,
                    if i & 1 == 0 { 0x30 } else { 0x40 },
                );
                word(
                    image,
                    0xa5a1df + 2 * i,
                    if id & 1 == 0 { 0x40 } else { 0xc0 },
                );
            }
            for slot in 26..32 {
                put(g, 0xf0f8 + 2 * slot, 0x1234);
                put(
                    g,
                    0xf178 + 2 * slot,
                    if id & 1 == 0 { 0xfff0 } else { 0x10 },
                );
                put(g, 0xf1f8 + 2 * slot, 0x5678);
                put(
                    g,
                    0xf278 + 2 * slot,
                    if id & 1 == 0 { 0xfff0 } else { 0x10 },
                );
            }
            Operation::DraygonMotion(if id >= 106 { 2 } else { 0 })
        }
        108 => {
            for slot in 0..32 {
                put(g, 0xef78 + 2 * slot, 0x7777);
            }
            for i in 0..12 {
                word(image, 0xa5a1c7 + 2 * i, 0x100 + 17 * i as u16);
            }
            Operation::DraygonStart
        }
        109..=110 => {
            if id == 110 {
                for slot in 0..32 {
                    put(g, 0xef78 + 2 * slot, 0x9000);
                }
            }
            Operation::DraygonFour
        }
        _ => unreachable!(),
    }
}
