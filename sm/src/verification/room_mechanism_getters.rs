//! Shared Session scalar imports and actual enemy setup coordinate helpers.
#![forbid(unsafe_code)]
use crate::game::{enemy_projectiles::coordinates, session::Session, session_runtime::Runtime};
use crate::game_state::Game;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};

/// Captures deterministic observations while checking fresh reads and read-only
/// behavior. No native ROM is needed for these shared-state observations.
pub(crate) fn capture(game: &mut Game) -> Vec<u8> {
    let saved = game.g_ram.to_vec();
    let result = catch_unwind(AssertUnwindSafe(|| {
        for (index, byte) in game.g_ram.iter_mut().enumerate() {
            *byte = (index.wrapping_mul(37) ^ (index >> 8)) as u8;
        }
        let mut expected = game.g_ram.to_vec();
        let mut output = Vec::new();
        let scalar_queries: [(usize, fn(&Session) -> u16); 3] = [
            (0x1c27, |session| session.mechanisms.selected),
            (0x1c29, |session| session.mechanisms.block_x),
            (0x1c2b, |session| session.mechanisms.block_y),
        ];
        for (address, query) in scalar_queries {
            for bytes in [[0x35, 0xc2], [0xa7, 0x19]] {
                game.g_ram[address..address + 2].copy_from_slice(&bytes);
                expected[address..address + 2].copy_from_slice(&bytes);
                let actual = query(Runtime::new(game).session());
                assert_eq!(actual, u16::from_le_bytes(bytes), "scalar live read");
                output.extend_from_slice(&actual.to_le_bytes());
                assert_eq!(&game.g_ram[..], expected.as_slice(), "getter changed RAM");
            }
        }
        // Explicit physical targets, independent of the implementation's index
        // expression: normal edge, odd floor, next column, broad and terrain alias.
        for (handle, block, argument) in [
            (0, 0x1c87, 0x1dc7),
            (1, 0x1c87, 0x1dc7),
            (78, 0x1cd5, 0x1e15),
            (79, 0x1cd5, 0x1e15),
            (80, 0x1cd7, 0x1e17),
            (81, 0x1cd7, 0x1e17),
            (0x1234, 0x2ebb, 0x2ffb),
            (0xe37a, 0x10001, 0x10141),
            (0xe37b, 0x10001, 0x10141),
            (0xe37c, 0x10003, 0x10143),
            (0xfffe, 0x11c85, 0x11dc5),
            (0xffff, 0x11c85, 0x11dc5),
        ] {
            for bytes in [[0x12, 0xb4], [0xe3, 0x56]] {
                game.g_ram[block..block + 2].copy_from_slice(&bytes);
                expected[block..block + 2].copy_from_slice(&bytes);
                let actual = coordinates::block_index(handle, &Runtime::new(game));
                assert_eq!(actual, u16::from_le_bytes(bytes), "block alias/live read");
                output.extend_from_slice(&actual.to_le_bytes());
                assert_eq!(
                    &game.g_ram[..],
                    expected.as_slice(),
                    "block getter changed RAM"
                );
                let native = [bytes[1], bytes[0]];
                game.g_ram[argument..argument + 2].copy_from_slice(&native);
                expected[argument..argument + 2].copy_from_slice(&native);
                let actual = coordinates::projectile_argument(handle, &Runtime::new(game));
                assert_eq!(
                    actual,
                    u16::from_ne_bytes(native),
                    "argument alias/native/live read"
                );
                output.extend_from_slice(&actual.to_le_bytes());
                assert_eq!(
                    &game.g_ram[..],
                    expected.as_slice(),
                    "argument getter changed RAM"
                );
            }
        }
        output.extend_from_slice(&game.g_ram[..]);
        output
    }));
    game.g_ram.copy_from_slice(&saved);
    match result {
        Ok(output) => output,
        Err(payload) => resume_unwind(payload),
    }
}
