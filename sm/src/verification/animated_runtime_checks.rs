//! Actual animated runtime guard and alias checks.
#![forbid(unsafe_code)]
use crate::{game_state::Game, ram_map::words as ram, rom::RomImage};
use std::panic::{AssertUnwindSafe, catch_unwind};
use {
    crate::game::{music::queue_music_delayed_8, session_runtime::Runtime},
    sm_game::animated_tiles::execution,
};
fn word(bytes: &impl Fn(usize) -> u8, at: usize) -> u16 {
    u16::from_le_bytes([bytes(at), bytes(at + 1)])
}
fn put(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
pub(super) fn run(game: &mut Game) -> bool {
    let saved_ram = (0..game.ram().len())
        .map(|at| game.ram().read_byte(at))
        .collect::<Vec<_>>();
    let saved_rom = game.loaded_rom.clone();
    let checked = catch_unwind(AssertUnwindSafe(|| {
        game.ram().fill(0..game.ram().len(), 0);
        game.loaded_rom = Some(RomImage::zeroed(0x39000));
        let mut expected = (0..game.ram().len())
            .map(|at| game.ram().read_byte(at))
            .collect::<Vec<_>>();
        put(&mut expected, 0x1f01, 1);
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                let mut runtime = Runtime::new(game);
                runtime
                    .session_mut()
                    .presentation
                    .visual_mut()
                    .animated_tiles
                    .slots[0]
                    .counter = 2;
                execution::command(&mut runtime, 8880340, 0, 0x9000);
            }))
            .is_err()
        );
        assert_eq!(
            (0..game.ram().len())
                .map(|at| game.ram().read_byte(at))
                .collect::<Vec<_>>()
                .as_slice(),
            expected
        );
        // Id index12 aliases Counter0; the following countdown must read the imported zero.
        {
            let mut runtime = Runtime::new(game);
            runtime
                .session_mut()
                .presentation
                .visual_mut()
                .animated_tiles
                .slots[0]
                .counter = 0xabcd;
            execution::command(&mut runtime, 8880306, 12, 0);
            assert_eq!(
                runtime.session().presentation.visual().animated_tiles.slots[0].counter,
                0
            );
        }
        assert_eq!(word(&|at| game.ram().read_byte(at), 0x1f01), 0);
        game.ram().fill(0..game.ram().len(), 0);
        game.ram()
            .copy_in(ram::MUSIC_QUEUE_WRITE_POS, &(0x18dau16).to_le_bytes());
        game.ram()
            .copy_in(ram::MUSIC_QUEUE_READ_POS, &(1u16).to_le_bytes());
        expected = (0..game.ram().len())
            .map(|at| game.ram().read_byte(at))
            .collect::<Vec<_>>();
        put(&mut expected, 0x1ef1, 0x8123);
        put(&mut expected, 0x1ef3, 0x3456);
        put(&mut expected, 0x1f03, 8);
        put(&mut expected, ram::MUSIC_QUEUE_WRITE_POS, 12);
        {
            let mut runtime = Runtime::new(game);
            runtime
                .session_mut()
                .presentation
                .visual_mut()
                .animated_tiles
                .flags = 0x8123;
            runtime
                .session_mut()
                .presentation
                .visual_mut()
                .animated_tiles
                .selected = 10;
            assert!(
                catch_unwind(AssertUnwindSafe(|| {
                    runtime.foreign(|game| {
                        assert_eq!(word(&|at| game.ram().read_byte(at), 0x1ef3), 10);
                        queue_music_delayed_8(0x3456, game);
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
        assert_eq!(
            (0..game.ram().len())
                .map(|at| game.ram().read_byte(at))
                .collect::<Vec<_>>()
                .as_slice(),
            expected
        );
        assert_eq!(
            crate::game::session_encoding::decode(
                &|at| game.ram().read_byte(at),
                game.ram().len(),
                std::sync::Arc::clone(game.world()),
            )
            .presentation
            .visual()
            .animated_tiles
            .selected,
            0x3456
        );
    }));
    [(); 0x20000].copy_from_slice(&vec![(); saved_ram.len()]);
    game.ram().copy_in(0, &saved_ram);
    game.loaded_rom = saved_rom;
    checked.is_ok()
}
