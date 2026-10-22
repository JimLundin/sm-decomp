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

fn award(slot: usize, game: &mut Game) {
    use crate::game::drop_rules::{Resource, REWARDS};
    use crate::game::sm_91::{
        Samus_RestoreHealth, Samus_RestoreMissiles, Samus_RestorePowerBombs,
        Samus_RestoreSuperMissiles,
    };
    let (resource, amount, sound) =
        REWARDS[usize::from(word!(game.eproj_E[slot]) >> 1)].expect("non-null function pointer");
    match resource {
        Resource::Health => Samus_RestoreHealth(amount, game),
        Resource::Missiles => Samus_RestoreMissiles(amount, game),
        Resource::SuperMissiles => Samus_RestoreSuperMissiles(amount, game),
        Resource::PowerBombs => Samus_RestorePowerBombs(amount, game),
    }
    crate::game::sm_80::QueueSfx2_Max1(u16::from(sound), game);
}

/// Advance expiry, then attempt grapple collection before body contact.
pub unsafe fn collect(index: u16, game: &mut Game) {
    use crate::game::contact_rules::axis;
    use crate::ram_map::word_arrays as a;
    let slot = usize::from(index >> 1);
    let offset = slot * 2;
    let timer = a::eproj_F + offset;
    let remaining = u16::from_le_bytes([game.g_ram[timer], game.g_ram[timer + 1]]).wrapping_sub(1);
    game.g_ram[timer..timer + 2].copy_from_slice(&remaining.to_le_bytes());
    if remaining != 0 {
        // The query precedes the live lifetime and coordinate reads.
        let grapple = unsafe { crate::game::sm_90::CallSomeSamusCode(0xd, game) } != 0
            && word!(game.eproj_F[slot]).wrapping_sub(384) & 0x8000 != 0
            && ((word!(game.eproj_x_pos[slot]).wrapping_sub(word!(game.grapple_beam_end_x_pos))
                as i16)
                .wrapping_abs() as u16)
                < 16
            && ((word!(game.eproj_y_pos[slot]).wrapping_sub(word!(game.grapple_beam_end_y_pos))
                as i16)
                .wrapping_abs() as u16)
                < 16;
        if !grapple {
            let x_radius = u16::from(game.g_ram[a::eproj_radius + offset]);
            let y_radius = u16::from(game.g_ram[a::eproj_radius + offset + 1]);
            if !axis(
                game.samus_x_pos(),
                game.enemy_projectile_x_pos(index),
                game.samus_x_radius(),
                x_radius,
            ) || !axis(
                game.samus_y_pos(),
                game.enemy_projectile_y_pos(index),
                game.samus_y_radius(),
                y_radius,
            ) {
                return;
            }
        }
        award(slot, game);
    }
    // Award and sound complete before the original slot changes to its exit script.
    for (base, value) in [
        (a::eproj_instr_list_ptr, 0xeca3u16),
        (a::eproj_instr_timers, 1),
        (a::eproj_properties, 0x3000),
        (a::eproj_pre_instr, 0xefdf),
    ] {
        let at = base + offset;
        game.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
}
