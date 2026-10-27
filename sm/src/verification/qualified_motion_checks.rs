//! Independent byte expectations for the formerly unaligned A0CBC7 u32 load.
//! Candidate only; raw and loaded images deliberately contain different data.
use super::{bytes, put, seed, slot};
use crate::game_state::Game;
use crate::ram_map::{word_arrays as a, words as w};
use crate::rom::RomImage;
use crate::types::EnemyData;
use core::mem::offset_of;
pub(crate) const CASES: u32 = 4;
fn write(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
pub(crate) fn check_case(id: u32) -> bool {
    if id >= CASES {
        return false;
    }
    let image = RomImage::zeroed(0x200000);
    let raw = RomImage::zeroed(0x200000);
    let mut game = Game::new();
    seed(&mut game, &image);
    game.loaded_rom = Some(image.clone());
    game.register_native_rom(raw.clone());
    let handle = if id == 3 { 10 } else { 34 };
    let offset = usize::from(handle);
    put(&mut game, w::game_state, 40);
    put(
        &mut game,
        w::random_enemy_counter,
        if id == 1 { 2 } else { 0 },
    );
    put(&mut game, 0xf78 + offset_of!(EnemyData, flash_timer), 1);
    slot(
        &mut game,
        a::eproj_y_pos,
        handle,
        if id < 2 { 100 } else { 199 },
    );
    slot(&mut game, a::eproj_y_subpos, handle, 0xffff);
    slot(&mut game, a::eproj_y_vel, handle, 0x1234);
    slot(&mut game, a::eproj_gfx_idx, handle, 0x8135);
    // High velocity byte 0x12 selects record 36, i.e. CBC7 + 144 bytes.
    // 0001:FFFF + 0000:0002 has a fraction carry; here the position starts
    // at 100:FFFF or 199:FFFF, hence the expected 102:0001 / 201:0001.
    bytes(
        &raw,
        crate::rtl::lorom_offset(0xa0cbc7) as usize + 144,
        &[2, 0, 1, 0],
    );
    let mut expected = (0..game.ram().len())
        .map(|at| game.ram().read_byte(at))
        .collect::<Vec<_>>();
    write(&mut expected, a::eproj_y_subpos + offset, 1);
    if id < 2 {
        write(&mut expected, a::eproj_y_pos + offset, 102);
        write(&mut expected, a::eproj_y_vel + offset, 0x12f4);
        write(&mut expected, a::eproj_instr_timers + offset, 0);
        write(
            &mut expected,
            a::eproj_gfx_idx + offset,
            if id == 1 { 0x8135 } else { 0x8f35 },
        );
    } else {
        write(&mut expected, a::eproj_y_pos + offset, 200);
        write(&mut expected, a::eproj_x_vel + offset, 0xeb93);
        write(&mut expected, a::eproj_instr_list_ptr + offset, 0xe208);
        write(&mut expected, a::eproj_gfx_idx + offset, 2560);
        write(&mut expected, a::eproj_instr_timers + offset, 1);
        expected[0x654] = 6; // Queue capacity is published even with sound gated.
        if id == 3 {
            write(&mut expected, 0x883e, 1);
        }
    }
    super::super::with_execution(&mut game, |ctx| {
        super::super::shared_pre::CallBotwoonEprojFunc(8842015, handle, ctx);
    });
    assert_eq!(
        (0..game.ram().len())
            .map(|at| game.ram().read_byte(at))
            .collect::<Vec<_>>()
            .as_slice(),
        expected.as_slice(),
        "qualified u32 case {id}"
    );
    assert!(!game.g_fail);
    true
}
