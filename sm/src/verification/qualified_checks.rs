//! Candidate-only independent byte expectations. Never execute these old UB paths.
//! Child of candidate carrier; distinct from differential baseline outcomes.
use super::{put, seed, slot};
use crate::game::constants::bank_86::{ADDR_K_EPROJ_INIT_AB07_TILE0, ADDR_K_EPROJ_INIT_AB07_TILE1};
use crate::game_state::Game;
use crate::ram_map::{word_arrays as a, words as w};
use crate::rom::RomImage;
use core::mem::offset_of;
use sm_game::actors::layouts::EnemyLayout104 as Layout;
pub(crate) const CASES: u32 = 24;
fn write(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
pub(crate) fn check_case(id: u32) -> bool {
    if id >= CASES {
        return false;
    }
    let mut image = vec![0; 0x200000];
    let mut game = Game::new();
    seed(&mut game, &mut image);
    let image = RomImage::from(image);
    game.loaded_rom = Some(image.clone());
    game.register_native_rom(image.clone());
    let expected;
    if id < 8 {
        let enemy = if id & 1 == 0 { 0 } else { 64 };
        let cursor = [1u16, 3, 0x3ff, 0xffff][id as usize / 2];
        put(&mut game, w::CUR_ENEMY_INDEX, enemy);
        put(
            &mut game,
            0xf78 + enemy as usize + offset_of!(Layout, var_b),
            cursor,
        );
        put(
            &mut game,
            0xf78 + enemy as usize + offset_of!(Layout, base.x_pos),
            0x1234,
        );
        put(
            &mut game,
            0xf78 + enemy as usize + offset_of!(Layout, base.y_pos),
            0xabcd,
        );
        let mut bytes = (0..game.ram().len())
            .map(|at| game.ram().read_byte(at))
            .collect::<Vec<_>>();
        // Literal byte expectation includes odd cursor and linear ffff extent.
        bytes[0x9000 + cursor as usize..0x9004 + cursor as usize]
            .copy_from_slice(&[0x34, 0x12, 0xcd, 0xab]);
        expected = bytes;
        super::super::record_botwoon_head(&mut game);
    } else if id < 16 {
        let n = (id - 8) as usize;
        let target = [0u16, 1, 32, 34][n % 4];
        let source = [0u16, 34, 1, 0xffff][n % 4];
        put(&mut game, w::EPROJ_INIT_PARAM_1, source);
        put(&mut game, w::FX_Y_POS, if n < 4 { 2 } else { 0xffff });
        let at = a::EPROJ_X_POS + usize::from(source & !1);
        let value = u16::from_le_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)]);
        let mut bytes = (0..game.ram().len())
            .map(|at| game.ram().read_byte(at))
            .collect::<Vec<_>>();
        write(&mut bytes, a::EPROJ_X_POS + usize::from(target & !1), value);
        write(
            &mut bytes,
            a::EPROJ_Y_POS + usize::from(target & !1),
            if n < 4 { 0xfffe } else { 0xfffb },
        );
        expected = bytes;
        super::super::with_execution(&mut game, |ctx| {
            super::super::setup::dispatch(8829050, target, ctx)
        });
    } else if id < 20 {
        let target = [0u16, 1, 32, 34][(id - 16) as usize];
        slot(&mut game, a::EPROJ_E, target, 2);
        slot(&mut game, a::EPROJ_ID, 2, 0);
        let mut bytes = (0..game.ram().len())
            .map(|at| game.ram().read_byte(at))
            .collect::<Vec<_>>();
        write(&mut bytes, a::EPROJ_ID + usize::from(target & !1), 0);
        expected = bytes;
        super::super::with_execution(&mut game, |ctx| {
            super::super::pre_actions::dispatch(8830482, target, ctx)
        });
    } else {
        let tail = [0u16, 1, 0xfffe, 0xffff][(id - 20) as usize];
        put(&mut game, w::VRAM_WRITE_QUEUE_TAIL, tail);
        put(&mut game, w::SAMUS_X_POS, 0x1234);
        put(&mut game, w::SAMUS_Y_POS, 16);
        let mut bytes = (0..game.ram().len())
            .map(|at| game.ram().read_byte(at))
            .collect::<Vec<_>>();
        for (position, source, destination) in [
            (tail, ADDR_K_EPROJ_INIT_AB07_TILE0 as u16, 28160),
            (
                tail.wrapping_add(7),
                ADDR_K_EPROJ_INIT_AB07_TILE1 as u16,
                28416,
            ),
        ] {
            let at = 0xd0 + usize::from(position);
            write(&mut bytes, at, 64);
            write(&mut bytes, at + 2, source);
            // The old bank-word store overlaps the destination's low byte.
            write(&mut bytes, at + 4, 134);
            write(&mut bytes, at + 5, destination);
        }
        write(&mut bytes, w::VRAM_WRITE_QUEUE_TAIL, tail.wrapping_add(14));
        write(&mut bytes, a::EPROJ_X_POS + 34, 0x1234);
        write(&mut bytes, a::EPROJ_Y_POS + 34, 0xffec);
        expected = bytes;
        super::super::with_execution(&mut game, |ctx| {
            super::super::setup::dispatch(8825405, 34, ctx)
        });
    }
    assert_eq!(
        (0..game.ram().len())
            .map(|at| game.ram().read_byte(at))
            .collect::<Vec<_>>()
            .as_slice(),
        expected.as_slice(),
        "qualified enemy-projectile case {id}"
    );
    assert!(!game.g_fail);
    true
}
