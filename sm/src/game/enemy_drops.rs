//! Synchronous enemy drop generation and checked definition access.
#![deny(unsafe_op_in_unsafe_fn)]
pub use crate::game::drop_rules::DropSource;
use crate::game::drop_rules::{PICKUPS, Spread, Weights};
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
    use crate::game::drop_rules::{REWARDS, Resource};
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
    dismiss(slot, game);
}

#[forbid(unsafe_code)]
fn dismiss(slot: usize, game: &mut Game) {
    word!(game.eproj_instr_list_ptr[slot] = 0xeca3);
    word!(game.eproj_instr_timers[slot] = 1);
    word!(game.eproj_properties[slot] = 0x3000);
    word!(game.eproj_pre_instr[slot] = 0xefdf);
}

#[forbid(unsafe_code)]
fn begin_pickup(slot: usize, kind: u16, game: &mut Game) {
    let code = kind.wrapping_mul(2);
    word!(game.eproj_E[slot] = code);
    let instruction = crate::rtl::rom_table_word!(game, 0x86ef04, (code >> 1) as isize);
    word!(game.eproj_instr_list_ptr[slot] = instruction);
    word!(game.eproj_instr_timers[slot] = 1);
    word!(game.eproj_F[slot] = 400);
}

pub unsafe fn initialize(index: u16, game: &mut Game) {
    let slot = usize::from(index >> 1);
    word!(game.eproj_x_pos[slot] = game.eproj_spawn_pt.x);
    word!(game.eproj_y_pos[slot] = game.eproj_spawn_pt.y);
    word!(game.eproj_gfx_idx[slot] = 0);
    word!(game.eproj_enemy_header_ptr[slot] = game.eproj_spawn_varE24);
    let kind = unsafe { select(index, game) };
    if kind != 0 && kind.wrapping_sub(6) & 0x8000 != 0 {
        begin_pickup(slot, kind, game);
        word!(game.eproj_killed_enemy_index[slot] = 0xffff);
    } else {
        dismiss(slot, game);
    }
}

pub unsafe fn convert(index: u16, game: &mut Game) -> u16 {
    let kind = unsafe { select(index, game) };
    let slot = usize::from(index >> 1);
    // Conversion tests the incoming index, unlike initialization's drop-code test.
    if index != 0 && kind.wrapping_sub(6) & 0x8000 != 0 {
        begin_pickup(slot, kind, game);
        word!(game.eproj_pre_instr[slot] = 0xefe0);
        word!(game.eproj_properties[slot] = word!(game.eproj_properties[slot]) & !0x4000);
        word!(game.eproj_instr_list_ptr[slot])
    } else {
        // Conversion writes its exit address last.
        word!(game.eproj_instr_timers[slot] = 1);
        word!(game.eproj_properties[slot] = 0x3000);
        word!(game.eproj_pre_instr[slot] = 0xefdf);
        word!(game.eproj_instr_list_ptr[slot] = 0xeca3);
        0xeca3
    }
}
