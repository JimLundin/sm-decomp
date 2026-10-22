//! Synchronous enemy drop generation and checked definition access.
#![deny(unsafe_op_in_unsafe_fn)]
pub use crate::game::drop_rules::DropSource;
use crate::game::drop_rules::{Spread, Weights, PICKUPS};
use crate::game::sm_80::NextRandom;
use crate::game::sm_86::SpawnEprojWithGfx;
use crate::game_state::Game;
use crate::ram_map::checked_word as word;
use crate::rtl::rom_record;
use crate::types::EnemyDef;
use core::mem::offset_of;

pub unsafe fn spawn(definition: u16, index: u16, parameter: u16, game: &mut Game) {
    game.eproj_spawn_varE24 = definition;
    unsafe {
        SpawnEprojWithGfx(parameter, index, 0xf337, game);
    }
}

pub unsafe fn scatter(source: DropSource, index: u16, parameter: u16, game: &mut Game) {
    let (definition, spread) = source.pattern();
    let count = match spread {
        Spread::Nearby(count) => count,
        Spread::Area(..) => 16,
    };
    for _ in 0..count {
        match spread {
            Spread::Nearby(_) => {
                // X precedes RNG; Y and the random high byte are live afterward.
                let x = game.special_death_item_drop_x_origin_pos();
                let random = unsafe { NextRandom(game) };
                game.eproj_spawn_pt.x = x.wrapping_add(random & 0x1f).wrapping_sub(16);
                game.eproj_spawn_pt.y = game
                    .special_death_item_drop_y_origin_pos()
                    .wrapping_add((game.random_number() & 0x1f00) >> 8)
                    .wrapping_sub(16);
            }
            Spread::Area(x, y, mask) => {
                game.eproj_spawn_pt.x = (unsafe { NextRandom(game) } & mask).wrapping_add(x);
                game.eproj_spawn_pt.y = ((game.random_number() & 0x3f00) >> 8).wrapping_add(y);
            }
        }
        unsafe {
            spawn(definition, index, parameter, game);
        }
    }
}

pub unsafe fn select(index: u16, game: &mut Game) -> u16 {
    let definition = word!(game.eproj_enemy_header_ptr[index >> 1]);
    if definition == 0 {
        return 6;
    }
    let address = unsafe { rom_record(0xa0, definition, game) }
        .word(offset_of!(EnemyDef, item_drop_chances_ptr));
    if address == 0 {
        return 6;
    }
    let random = loop {
        let value = unsafe { NextRandom(game) } as u8;
        if value != 0 {
            break value;
        }
    };
    let health = game
        .samus_reserve_health()
        .wrapping_add(game.samus_health());
    if health < 30 {
        word!(game.health_drop_bias_flag = 1);
    } else if health >= 50 {
        word!(game.health_drop_bias_flag = 0);
    }
    let table = unsafe { rom_record(0xb4, address, game) };
    let weights = Weights::new(
        word!(game.health_drop_bias_flag) as u8 != 0,
        [
            game.samus_health() != game.samus_max_health()
                || game.samus_reserve_health() != game.samus_max_reserve_health(),
            game.samus_missiles() != game.samus_max_missiles(),
            game.samus_super_missiles() != game.samus_max_super_missiles(),
            game.samus_power_bombs() != game.samus_max_power_bombs(),
        ],
        |offset| table.byte(offset),
    );
    let mut cumulative = 0u16;
    for (slot, pickup) in PICKUPS.into_iter().enumerate() {
        if weights.enabled(slot) {
            // Each selected row diagnoses and captures the then-current ROM separately.
            let value =
                unsafe { rom_record(0xb4, address.wrapping_add(slot as u16), game) }.byte(0);
            cumulative = cumulative.wrapping_add(weights.contribution(slot, value));
            if cumulative >= u16::from(random) {
                return pickup;
            }
        }
    }
    6
}
