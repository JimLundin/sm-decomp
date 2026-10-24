//! Focused defined-old Sparks and lavaquake no-sample branches.
//! Child of the existing verification carrier; preserves all 1362 old fixtures.
use super::{OUTPUT_BYTES, bytes, put, seed, slot};
fn CallEprojPreInstr(ea: u32, k: u16, game: &mut Game) {
    super::super::with_execution(game, |ctx| super::super::pre_actions::dispatch(ea, k, ctx));
}
fn CallNorfairLavaquakeRocksFunc(ea: u32, k: u16, game: &mut Game) {
    super::super::with_execution(game, |ctx| {
        super::super::shared_pre::CallNorfairLavaquakeRocksFunc(ea, k, ctx)
    });
}
use crate::game_state::Game;
use crate::ram_map::{word_arrays as a, words as w};
use crate::rom::RomImage;
#[path = "enemy_native_runtime_cases.rs"]
pub(crate) mod native_runtime_cases;
#[path = "inventory_restore_cases.rs"]
pub(crate) mod restore_cases;
pub(crate) const CASES: u32 = 20 + restore_cases::CASES + native_runtime_cases::CASES;
pub(crate) fn capture_case(id: u32) -> Option<Vec<u8>> {
    if id >= CASES {
        return None;
    }
    if id >= 20 + restore_cases::CASES {
        return native_runtime_cases::capture_case(id - 20 - restore_cases::CASES);
    }
    if id >= 20 {
        return restore_cases::capture_case(id - 20);
    }
    let image = RomImage::zeroed(0x200000);
    let mut game = Game::new();
    seed(&mut game, &image);
    game.loaded_rom = Some(image.clone());
    game.register_native_rom(image.clone());
    let handle = [0u16, 1, 34, 0xffff][id as usize % 4];
    put(&mut game, w::eproj_index, handle);
    put(&mut game, w::layer1_x_pos, 0);
    put(&mut game, w::layer1_y_pos, 0);
    put(&mut game, w::room_width_in_blocks, 32);
    for block in 0..1024 {
        put(
            &mut game,
            0x10002 + block * 2,
            if id < 8 { 0x8000 } else { 0 },
        );
        game.g_ram[0x16402 + block] = 0;
    }
    for (field, value) in [
        (a::eproj_x_pos, 136),
        (a::eproj_y_pos, 136),
        (a::eproj_x_subpos, 0xffff),
        (a::eproj_y_subpos, 0xff00),
        (a::eproj_radius, 0x0303),
        (a::eproj_id, 0x8000),
        (a::eproj_x_vel, [0xffff, 0, 0x4000, 0xc000][id as usize % 4]),
        (
            a::eproj_y_vel,
            if id < 8 {
                (id / 4) as u16
            } else {
                [0, 3, 0x8000, 0xffff][id as usize % 4]
            },
        ),
        (a::eproj_E, [0x4000, 0x8000, 0xffff, 0][id as usize % 4]),
        (a::eproj_F, [0xffff, 0x7fff, 1, 0x8000][id as usize % 4]),
    ] {
        slot(&mut game, field, handle, value);
    }
    game.g_ram[0x5b5] = if id >= 12 { 0 } else { 1 };
    for index in 0..32 {
        put(&mut game, a::sprite_instr_list_ptrs + index * 2, 0);
    }
    bytes(
        &image,
        crate::rtl::lorom_offset(0xb4bda8) as usize + 48 * 2,
        &0x9000u16.to_le_bytes(),
    );
    bytes(
        &image,
        crate::rtl::lorom_offset(0xb49000) as usize,
        &7u16.to_le_bytes(),
    );
    {
        if id < 16 {
            CallEprojPreInstr(8844272, handle, &mut game);
        } else {
            // Func1 y_vel=0/1 changes phase without the old undefined u32 read.
            slot(&mut game, a::eproj_y_vel, handle, (id & 1) as u16);
            slot(&mut game, a::eproj_E, handle, 8829974u32 as u16);
            if id < 18 {
                CallNorfairLavaquakeRocksFunc(8829974, handle, &mut game);
            } else {
                slot(
                    &mut game,
                    a::eproj_x_pos,
                    handle,
                    if id == 18 { 128 } else { 300 },
                );
                CallEprojPreInstr(8829967, handle, &mut game);
            }
        }
    }
    assert!(!game.g_fail, "supplement case {id}");
    let mut output = game.g_ram.to_vec();
    for value in [
        game.eproj_spawn_rect.x,
        game.eproj_spawn_rect.y,
        game.eproj_spawn_rect.w,
        game.eproj_spawn_rect.h,
        game.eproj_spawn_r22,
        game.eproj_spawn_pt.x,
        game.eproj_spawn_pt.y,
        game.eproj_spawn_varE24,
    ] {
        output.extend_from_slice(&value.to_le_bytes());
    }
    output.extend_from_slice(&[0; 11]); // No return, panic or diagnostic.
    assert_eq!(output.len(), OUTPUT_BYTES);
    Some(output)
}
