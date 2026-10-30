//! Independent turret byte expectations; never execute old unaligned clears.
//! Child of candidate carrier. Predicate cases cross the actual legacy interval.
use super::{put, seed, slot};
use crate::game_state::Game;
use crate::ram_map::{word_arrays as a, words as w};
use crate::rom::RomImage;

pub(crate) const CASES: u32 = 16;

fn write(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}

pub(crate) fn check_case(id: u32) -> bool {
    if id >= CASES {
        return false;
    }
    let image = RomImage::zeroed(0x200000);
    let mut game = Game::new();
    seed(&mut game, &image);
    game.loaded_rom = Some(image.clone());
    game.register_native_rom(image);
    let handles = [0u16, 1, 34, 0xffff];
    let handle = handles[id as usize % 4];
    let offset = usize::from(handle & !1);
    put(&mut game, w::LAYER1_X_POS, 0);
    put(&mut game, w::LAYER1_Y_POS, 0);
    if id < 8 {
        // Negative Y rejects before countdown or dust spawning. The clear starts
        // at 0x1997 + k, including odd k, and leaves both neighboring bytes alone.
        slot(&mut game, a::EPROJ_Y_POS, handle, 0xffff);
        put(&mut game, 0x783a, if id < 4 { 1 } else { 0 });
        let at = 0x1997 + usize::from(handle);
        game.ram().copy_in(at - 1, &[0x12, 0x34, 0x56, 0x78]);
        let mut expected = (0..game.ram().len())
            .map(|at| game.ram().read_byte(at))
            .collect::<Vec<_>>();
        if id < 4 {
            expected[at] = 0;
            expected[at + 1] = 0;
        }
        super::super::with_execution(&mut game, |ctx| {
            super::super::pre_actions::dispatch(8830943, handle, ctx);
        });
        assert_eq!(
            (0..game.ram().len())
                .map(|at| game.ram().read_byte(at))
                .collect::<Vec<_>>()
                .as_slice(),
            expected.as_slice(),
            "turret clear {id}"
        );
    } else {
        // Distinct selected/argument slots expose accidental use of k for clear.
        // Ridley reads terrain at 0x10002 + 2*(2*16+2). Both velocities are zero
        // so all motion bytes must remain unchanged, including odd handles.
        let selected = [35u16, 0xffff, 1, 0][id as usize % 4];
        put(&mut game, w::EPROJ_INDEX, selected);
        put(&mut game, w::ROOM_WIDTH_IN_BLOCKS, 16);
        put(&mut game, 0x10046, if id < 12 { 0x1000 } else { 0x0fff });
        for (field, value) in [
            (a::EPROJ_X_POS, 32),
            (a::EPROJ_Y_POS, 32),
            (a::EPROJ_X_VEL, 0),
            (a::EPROJ_Y_VEL, 0),
            (a::EPROJ_PROPERTIES, 0x1234),
        ] {
            slot(&mut game, field, handle, value);
        }
        let at = 0x1997 + usize::from(selected & !1);
        game.ram().copy_in(at - 1, &[0x12, 0x34, 0x56, 0x78]);
        let mut expected = (0..game.ram().len())
            .map(|at| game.ram().read_byte(at))
            .collect::<Vec<_>>();
        write(&mut expected, a::EPROJ_PROPERTIES + offset, 0x9234);
        if id < 12 {
            expected[at] = 0;
            expected[at + 1] = 0;
        }
        super::super::with_execution(&mut game, |ctx| {
            super::super::pre_actions::dispatch(8831200, handle, ctx);
        });
        assert_eq!(
            (0..game.ram().len())
                .map(|at| game.ram().read_byte(at))
                .collect::<Vec<_>>()
                .as_slice(),
            expected.as_slice(),
            "turret bullet {id}"
        );
    }
    assert!(!game.g_fail);
    true
}
