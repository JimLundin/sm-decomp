//! Defined-old drop/contact cases; child of frozen core carrier, image >= 0x200000.
use super::{bytes, definition, put, slot};
mod drops {
    pub(super) use super::super::super::DropSource;
    use crate::game_state::Game;
    pub(super) fn select(k: u16, game: &mut Game) -> u16 {
        super::super::super::with_execution(game, |ctx| super::super::super::drops::select(k, ctx))
    }
    pub(super) fn initialize(k: u16, game: &mut Game) {
        super::super::super::with_execution(game, |ctx| {
            super::super::super::drops::initialize(k, ctx)
        });
    }
    pub(super) fn convert(k: u16, game: &mut Game) -> u16 {
        super::super::super::with_execution(game, |ctx| super::super::super::drops::convert(k, ctx))
    }
    pub(super) fn collect(k: u16, game: &mut Game) {
        super::super::super::with_execution(game, |ctx| {
            super::super::super::drops::collect(k, ctx)
        });
    }
    pub(super) use super::super::super::scatter_drop as scatter;
}
mod projectile_contact {
    pub(super) use super::super::super::{enemy_projectiles_hit_samus, shoot_enemy_projectiles};
}
use crate::game_state::Game;
use crate::ram_map::{word_arrays as a, words as w};
use crate::rom::RomImage;
use crate::types::{EnemyDef, EprojDef};
use core::mem::offset_of;

pub(super) const CASES: u32 = 100;
fn rom_word(image: &RomImage, address: u32, offset: usize, value: u16) {
    bytes(
        image,
        crate::rtl::lorom_offset(address) as usize + offset,
        &value.to_le_bytes(),
    );
}
pub(super) unsafe fn execute(id: u32, game: &mut Game, image: &RomImage) -> Option<usize> {
    assert!(id < CASES);
    let variant = id as usize % 16;
    let index = [0, 1, 32, 34][variant % 4];
    for (field, value) in [
        (
            w::samus_health,
            [1, 29, 30, 49, 50, 99, 100, 0xffff][variant % 8],
        ),
        (w::samus_max_health, 99),
        (w::samus_reserve_health, 0),
        (w::samus_max_reserve_health, 100),
        (w::health_drop_bias_flag, variant as u16 & 1),
        (w::samus_missiles, 0),
        (w::samus_max_missiles, 10),
        (w::samus_super_missiles, 0),
        (w::samus_max_super_missiles, 5),
        (w::samus_power_bombs, 0),
        (w::samus_max_power_bombs, 5),
        (
            w::random_number,
            (variant as u16).wrapping_mul(0x1731).wrapping_add(1),
        ),
        (w::samus_x_pos, 128),
        (w::samus_y_pos, 128),
        (w::samus_x_radius, 8),
        (w::samus_y_radius, 16),
        (w::time_is_frozen_flag, 0),
        (w::grapple_beam_function, 0xc4f0),
        (w::grapple_beam_end_x_pos, 128),
        (w::grapple_beam_end_y_pos, 128),
    ] {
        put(game, field, value);
    }
    if id < 48 {
        let header = if variant == 0 { 0 } else { 0x8000 };
        slot(game, a::eproj_enemy_header_ptr, index, header);
        game.eproj_spawn_varE24 = header;
        rom_word(
            image,
            0xa08000,
            offset_of!(EnemyDef, item_drop_chances_ptr),
            if variant == 1 { 0 } else { 0x9000 },
        );
        let table: [u8; 6] = match variant % 4 {
            0 => [64, 64, 32, 32, 32, 31],
            1 => [0, 0, 0, 255, 0, 0],
            2 => [255, 0, 0, 0, 0, 0],
            _ => [0, 0, 0, 0, 128, 127],
        };
        bytes(image, crate::rtl::lorom_offset(0xb49000) as usize, &table);
        for kind in 0..7 {
            rom_word(image, 0x86ef04, kind * 2, 0x9100 + kind as u16 * 4);
        }
        {
            return match id / 16 {
                0 => Some(drops::select(index, game) as usize),
                1 => {
                    drops::initialize(index, game);
                    None
                }
                _ => Some(drops::convert(index, game) as usize),
            };
        }
    }
    if id < 64 {
        let n = (id - 48) as usize;
        slot(game, a::eproj_E, index, 2 * (1 + n as u16 % 5));
        slot(game, a::eproj_F, index, [1, 2, 384, 385][n % 4]);
        slot(game, a::eproj_radius, index, 0x0808);
        slot(
            game,
            a::eproj_x_pos,
            index,
            if n < 10 { 128 } else { 0xfff0 },
        );
        slot(game, a::eproj_y_pos, index, 128);
        if n >= 12 {
            put(game, w::grapple_beam_function, 0);
            put(game, w::grapple_beam_end_x_pos, 0xfff0);
        }
        {
            drops::collect(index, game)
        };
        return None;
    }
    if id < 76 {
        use drops::DropSource::*;
        let source = [
            MiniKraid,
            LowerNorfairPirate,
            Metroid,
            Ridley,
            Crocomire,
            Phantoon,
            Botwoon,
            Kraid,
            BombTorizo,
            GoldenTorizo,
            SporeSpawn,
            Draygon,
        ]
        .into_iter()
        .nth((id - 64) as usize)
        .unwrap();
        put(game, w::special_death_item_drop_x_origin_pos, 0xfff0);
        put(game, w::special_death_item_drop_y_origin_pos, 0x8000);
        // Successful allocations use the actual pickup initializer; zero chance
        // pointers in synthetic enemy records choose its explicit dismiss path.
        definition(image, 0xf337, 8843049u32 as u16);
        for handle in (0..36).step_by(2) {
            slot(game, a::eproj_id, handle, 0);
        }
        {
            game.ram().copy_in(0xe20, &0x4321u16.to_le_bytes());
            drops::scatter(source, 0, game)
        };
        return None;
    }
    let n = (id - 76) as usize;
    for handle in (0..36).step_by(2) {
        slot(game, a::eproj_id, handle, 0);
    }
    slot(game, a::eproj_id, 34, 0x8000);
    slot(game, a::eproj_x_pos, 34, 128);
    slot(game, a::eproj_y_pos, 34, 128);
    slot(game, a::eproj_radius, 34, 0x0808);
    rom_word(
        image,
        0x868000,
        offset_of!(EprojDef, hit_instruction_list),
        if n & 1 == 0 { 0x9876 } else { 0 },
    );
    rom_word(
        image,
        0x868000,
        offset_of!(EprojDef, shot_instruction_list),
        0x9234,
    );
    if n < 12 {
        put(game, w::samus_health, 999);
        put(game, w::equipped_items, [0, 1, 0x20][n % 3]);
        put(
            game,
            w::samus_invincibility_timer,
            if n == 10 { 1 } else { 0 },
        );
        put(
            game,
            w::samus_contact_damage_index,
            if n == 11 { 1 } else { 0 },
        );
        put(game, w::samus_x_pos, [128, 112, 144][n % 3]);
        slot(
            game,
            a::eproj_properties,
            34,
            5 | if n & 2 == 0 { 0 } else { 0x4000 },
        );
        if n == 8 {
            slot(game, a::eproj_properties, 34, 0x2005);
        }
        if n == 9 {
            slot(game, a::eproj_radius, 34, 0);
        }
        {
            projectile_contact::enemy_projectiles_hit_samus(game)
        };
    } else {
        put(game, w::projectile_counter, if n == 23 { 0 } else { 5 });
        slot(game, a::eproj_properties, 34, 0x8005);
        slot(game, a::eproj_flags, 34, ((n - 12) % 3) as u16);
        for shot in 0..5 {
            put(
                game,
                a::projectile_type + shot * 2,
                [1, 8, 0x300, 0x500, 0x700][shot],
            );
            put(
                game,
                a::projectile_x_pos + shot * 2,
                if n & 1 == 0 { 128 } else { 160 },
            );
            put(game, a::projectile_y_pos + shot * 2, 128);
        }
        {
            projectile_contact::shoot_enemy_projectiles(game)
        };
    }
    None
}
