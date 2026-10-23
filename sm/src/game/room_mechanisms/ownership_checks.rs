//! Focused regression checks for the owned compatibility boundary.
#![forbid(unsafe_code)]
use super::port::Execution;
use crate::{game_state::Game, rom::RomImage};
use std::panic::{AssertUnwindSafe, catch_unwind};

pub(super) fn run(game: &mut Game) -> bool {
    let original_ram = game.g_ram.to_vec();
    let original_rom = game.loaded_rom.clone();
    let result = catch_unwind(AssertUnwindSafe(|| {
        // Independent byte ranges include every slot, control, tile and custom lane.
        for (i, byte) in game.g_ram.iter_mut().enumerate() {
            *byte = (i.wrapping_mul(37) ^ (i >> 8)) as u8;
        }
        let mut expected = game.g_ram.to_vec();
        {
            let mut execution = Execution::new(game);
            for range in [0x1c23..0x1e6d, 0xc6c8..0xc8c8, 0xde1c..0xdf5c] {
                for address in range {
                    assert_eq!(execution.port().byte(address), expected[address]);
                    expected[address] ^= 0xa5;
                    execution.port_mut().put_byte(address, expected[address]);
                    assert_eq!(execution.port().byte(address), expected[address]);
                }
            }
            // Cross both outside edges and internal column boundaries, including
            // Counter[-1], raw odd selectors and the tilemap's two neighboring words.
            for address in [
                0x1c22, 0x1c36, 0x1c86, 0x1d75, 0x1e66, 0x1e6c, 0xc6c6, 0xc6c7, 0xc8c6, 0xc8c7,
                0xde1b, 0xde6b, 0xdf5b,
            ] {
                let bytes = [0x12, 0x34, 0x56, 0x78];
                execution.port_mut().put_slice(address, &bytes);
                expected[address..address + 4].copy_from_slice(&bytes);
                assert_eq!(execution.port().word(address + 1), 0x5634);
                assert_eq!(
                    execution.port().native_word(address),
                    u16::from_ne_bytes([0x12, 0x34])
                );
            }
        }
        assert_eq!(&game.g_ram[..], &expected[..]);

        let old_image = RomImage::zeroed(2);
        old_image.write(0, 0x31);
        game.loaded_rom = Some(old_image.clone());
        {
            let mut execution = Execution::new(game);
            execution.port_mut().put_word(0x1c37, 0xbeef);
            let returned = execution.foreign(|game| {
                assert_eq!(&game.g_ram[0x1c37..0x1c39], &[0xef, 0xbe]);
                game.g_ram[0x1c38] = 0x42;
                game.g_ram[0xc6c8] = 0x73;
                let replacement = RomImage::zeroed(3);
                replacement.write(0, 0x92);
                game.loaded_rom = Some(replacement);
                19
            });
            assert_eq!(returned, 19);
            assert_eq!(execution.port().word(0x1c37), 0x42ef);
            assert_eq!(execution.port().byte(0xc6c8), 0x73);
            assert_eq!(execution.port().rom_image().len(), 3);
            assert_eq!(execution.port().rom_image().read(0), 0x92);
            assert_eq!(old_image.read(0), 0x31);
        }

        // Drop the emptied Execution during unwinding: it must not restore its
        // pre-callback owner over the callback's newer bytes.
        let panic = catch_unwind(AssertUnwindSafe(|| {
            let mut execution = Execution::new(game);
            execution.port_mut().put_word(0x1d27, 0x1234);
            execution.foreign(|game| {
                assert_eq!(&game.g_ram[0x1d27..0x1d29], &[0x34, 0x12]);
                game.g_ram[0x1d27..0x1d29].copy_from_slice(&[0xcd, 0xab]);
                panic!("intentional mechanism callback unwind");
            });
        }));
        assert!(panic.is_err());
        assert_eq!(&game.g_ram[0x1d27..0x1d29], &[0xcd, 0xab]);

        let last = game.g_ram.len() - 1;
        game.g_ram[last] = 0x81;
        let mut execution = Execution::new(game);
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                execution.port_mut().put_slice(last, &[0x23, 0x45]);
            }))
            .is_err()
        );
        assert_eq!(execution.port().byte(last), 0x81);
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                execution.port_mut().put_word_prefix(last, 0x4523);
            }))
            .is_err()
        );
        assert_eq!(execution.port().byte(last), 0x23);
    }));
    game.g_ram.copy_from_slice(&original_ram);
    game.loaded_rom = original_rom;
    result.is_ok()
}
