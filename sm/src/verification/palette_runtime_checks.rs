//! Actual compatibility-guard checks; no replacement guard or fake RAM owner.
#![forbid(unsafe_code)]
use crate::game::{
    music::QueueMusic_Delayed8, palette_effects::execution, session_access::SessionAccess,
    session_runtime::Runtime,
};
use crate::{game_state::Game, ram_map::words as ram, rom::RomImage};
use std::panic::{AssertUnwindSafe, catch_unwind};

fn word(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}
fn put(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
pub(super) fn run(game: &mut Game) -> bool {
    let original_ram = game.g_ram.to_vec();
    let original_rom = game.loaded_rom.clone();
    let result = catch_unwind(AssertUnwindSafe(|| {
        game.g_ram.fill(0);
        game.loaded_rom = Some(RomImage::zeroed(0x69000));
        let mut expected = game.g_ram.to_vec();
        put(&mut expected, 0x1edd, 1);
        put(&mut expected, 0xc00a, 0x1234);
        let failed = catch_unwind(AssertUnwindSafe(|| {
            let mut runtime = Runtime::new(game);
            runtime.session_mut().visual.palette_effects.slots[0].counter = 2;
            runtime.session_mut().visual.colors[5] = 0x1234;
            execution::command(&mut runtime, 9291321, 0, 0x9000);
        }));
        assert!(failed.is_err());
        assert_eq!(game.g_ram.as_slice(), expected);

        game.g_ram.fill(0);
        expected.fill(0);
        put(&mut expected, ram::flag_for_palette_fx_objects, 0x8000);
        let failed = catch_unwind(AssertUnwindSafe(|| {
            let mut runtime = Runtime::new(game);
            execution::set_enabled(runtime.session_mut(), true);
            execution::command(&mut runtime, 9291358, 0, 0x9000);
        }));
        assert!(failed.is_err());
        assert_eq!(game.g_ram.as_slice(), expected);

        // These newly owned damage/color fields must also survive core failure.
        game.g_ram.fill(0);
        expected.fill(0);
        put(&mut expected, ram::samus_in_heat_palfx_index, 1);
        put(&mut expected, ram::samus_in_heat_palettefx_prev_index, 1);
        put(&mut expected, 0x1ecd, 1);
        expected[ram::samus_periodic_damage..ram::samus_periodic_damage + 2]
            .copy_from_slice(&2u16.to_ne_bytes());
        let failed = catch_unwind(AssertUnwindSafe(|| {
            let mut runtime = Runtime::new(game);
            runtime.session_mut().visual.palette_effects.heat = 1;
            runtime.session_mut().player.periodic_damage_fraction = 0xc000;
            runtime.session_mut().player.periodic_damage = 1;
            execution::pre(&mut runtime, 9298809, 0);
        }));
        assert!(failed.is_err());
        assert_eq!(game.g_ram.as_slice(), expected);

        // A real music callback writes this owner's Selected and ID7 through
        // the existing oversized queue index. Its later unwind must not let
        // Drop restore the old model over those callback writes.
        game.g_ram.fill(0);
        put(&mut game.g_ram, ram::music_queue_write_pos, 0x1862);
        put(&mut game.g_ram, ram::music_queue_read_pos, 1);
        expected = game.g_ram.to_vec();
        put(&mut expected, ram::flag_for_palette_fx_objects, 0x8123);
        put(&mut expected, 0xc00a, 0x4567);
        put(&mut expected, ram::palettefx_index, 0x2345);
        put(&mut expected, 0x1e8b, 8);
        put(&mut expected, ram::music_queue_write_pos, 4);
        {
            let mut runtime = Runtime::new(game);
            runtime.session_mut().visual.palette_effects.flags = 0x8123;
            runtime.session_mut().visual.palette_effects.selected = 14;
            runtime.session_mut().visual.colors[5] = 0x4567;
            let failed = catch_unwind(AssertUnwindSafe(|| {
                runtime.foreign(|game| {
                    assert_eq!(word(&game.g_ram, ram::palettefx_index), 14);
                    assert_eq!(word(&game.g_ram, 0xc00a), 0x4567);
                    QueueMusic_Delayed8(0x2345, game);
                    panic!("intentional palette callback unwind");
                });
            }));
            assert!(failed.is_err());
            assert!(
                catch_unwind(AssertUnwindSafe(|| {
                    let _ = runtime.session();
                }))
                .is_err()
            );
        }
        assert_eq!(game.g_ram.as_slice(), expected);
    }));
    game.g_ram.copy_from_slice(&original_ram);
    game.loaded_rom = original_rom;
    result.is_ok()
}
