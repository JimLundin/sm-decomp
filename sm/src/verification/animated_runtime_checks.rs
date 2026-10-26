//! Actual animated runtime guard and alias checks.
#![forbid(unsafe_code)]
use crate::game::{
    animated_tiles::execution, music::QueueMusic_Delayed8, session_access::SessionAccess,
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
    let saved_ram = game.g_ram.to_vec();
    let saved_rom = game.loaded_rom.clone();
    let checked = catch_unwind(AssertUnwindSafe(|| {
        game.g_ram.fill(0);
        game.loaded_rom = Some(RomImage::zeroed(0x39000));
        let mut expected = game.g_ram.to_vec();
        put(&mut expected, 0x1f01, 1);
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                let mut runtime = Runtime::new(game);
                runtime.session_mut().visual.animated_tiles.slots[0].counter = 2;
                execution::command(&mut runtime, 8880340, 0, 0x9000);
            }))
            .is_err()
        );
        assert_eq!(game.g_ram.as_slice(), expected);
        // Id index12 aliases Counter0; the following countdown must read the imported zero.
        {
            let mut runtime = Runtime::new(game);
            runtime.session_mut().visual.animated_tiles.slots[0].counter = 0xabcd;
            execution::command(&mut runtime, 8880306, 12, 0);
            assert_eq!(runtime.session().visual.animated_tiles.slots[0].counter, 0);
        }
        assert_eq!(word(&game.g_ram, 0x1f01), 0);
        game.g_ram.fill(0);
        put(&mut game.g_ram, ram::music_queue_write_pos, 0x18da);
        put(&mut game.g_ram, ram::music_queue_read_pos, 1);
        expected = game.g_ram.to_vec();
        put(&mut expected, 0x1ef1, 0x8123);
        put(&mut expected, 0x1ef3, 0x3456);
        put(&mut expected, 0x1f03, 8);
        put(&mut expected, ram::music_queue_write_pos, 12);
        {
            let mut runtime = Runtime::new(game);
            runtime.session_mut().visual.animated_tiles.flags = 0x8123;
            runtime.session_mut().visual.animated_tiles.selected = 10;
            assert!(
                catch_unwind(AssertUnwindSafe(|| {
                    runtime.foreign(|game| {
                        assert_eq!(word(&game.g_ram, 0x1ef3), 10);
                        QueueMusic_Delayed8(0x3456, game);
                        panic!("intentional animation callback unwind");
                    });
                }))
                .is_err()
            );
            assert!(
                catch_unwind(AssertUnwindSafe(|| {
                    let _ = runtime.session();
                }))
                .is_err()
            );
        }
        assert_eq!(game.g_ram.as_slice(), expected);
        assert_eq!(
            crate::game::session_encoding::decode(
                &game.g_ram,
                std::sync::Arc::clone(game.world()),
            )
            .visual
            .animated_tiles
            .selected,
            0x3456
        );
    }));
    game.g_ram.copy_from_slice(&saved_ram);
    game.loaded_rom = saved_rom;
    checked.is_ok()
}
