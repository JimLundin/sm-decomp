//! Independent ownership/source checks for the native-content variant only.
//! Register directly under enemy_projectiles, separately from baseline carriers.
#![forbid(unsafe_code)]
use super::host::{Content as _, NativeRecord as _};
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::native_rom::NativeRomPointer;
use crate::rom::RomImage;
use std::panic::{AssertUnwindSafe, catch_unwind};
pub(crate) const CASES: u32 = 28;
fn panics_with(operation: impl FnOnce(), expected: &str) {
    let payload = catch_unwind(AssertUnwindSafe(operation)).expect_err("expected checked failure");
    let message = payload
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| payload.downcast_ref::<&str>().copied())
        .unwrap_or("");
    assert!(
        message.contains(expected),
        "expected {expected:?}, got {message:?}"
    );
}
fn check_record_boundary(id: u32) {
    use super::content::NativeRecord;
    let image = RomImage::zeroed(8);
    for (index, value) in [0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0]
        .into_iter()
        .enumerate()
    {
        image.write(index, value);
    }
    let record = NativeRecord::new(image.clone(), 2);
    match id {
        6 => {
            assert_eq!(record.byte(-2), 0x12);
            assert_eq!(record.byte(-1), 0x34);
            assert_eq!(record.byte(5), 0xf0);
        }
        7 => {
            // Word indexes are signed WORDS; the final complete word is legal.
            assert_eq!(record.word_le(-1), 0x3412);
            assert_eq!(record.word_le(2), 0xf0de);
        }
        8 => {
            // Long indexes are signed BYTES, including a negative valid start.
            assert_eq!(record.long_le(-2), 0x78563412);
            assert_eq!(record.long_le(2), 0xf0debc9a);
        }
        9 => panics_with(
            || {
                let _ = record.byte(6);
            },
            // LEGACY(message-text): asserts on panic or Die text; should assert the failure kind instead.
            "index out of bounds",
        ),
        10 => {
            let record = NativeRecord::new(image, 7);
            panics_with(
                || {
                    let _ = record.word_le(0);
                },
                // LEGACY(message-text): asserts on panic or Die text; should assert the failure kind instead.
                "index out of bounds",
            );
        }
        11 => {
            let record = NativeRecord::new(image, 8);
            panics_with(
                || {
                    let _ = record.word_le(0);
                },
                // LEGACY(message-text): asserts on panic or Die text; should assert the failure kind instead.
                "index out of bounds",
            );
        }
        12..=15 => {
            // Zero, one, two or three available bytes cannot form a long.
            let available = (id - 12) as usize;
            let record = NativeRecord::new(image, 8 - available);
            panics_with(
                || {
                    let _ = record.long_le(0);
                },
                // LEGACY(message-text): asserts on panic or Die text; should assert the failure kind instead.
                "index out of bounds",
            );
        }
        16 => {
            let record = NativeRecord::new(RomImage::zeroed(0), 0);
            panics_with(
                || {
                    let _ = record.byte(0);
                },
                // LEGACY(message-text): asserts on panic or Die text; should assert the failure kind instead.
                "index out of bounds",
            );
        }
        17 => {
            let record = NativeRecord::new(image, usize::MAX);
            panics_with(
                || {
                    let _ = record.byte(1);
                },
                "native ROM offset overflow",
            );
        }
        18 => panics_with(
            || {
                let _ = record.byte(-3);
            },
            "native ROM offset overflow",
        ),
        19 => panics_with(
            || {
                let _ = record.word_le(isize::MAX);
            },
            "native ROM word offset overflow",
        ),
        20 => panics_with(
            || {
                let _ = record.word_le(isize::MIN);
            },
            "native ROM word offset overflow",
        ),
        21 => {
            let record = NativeRecord::new(image, usize::MAX);
            panics_with(
                || {
                    let _ = record.word_le(1);
                },
                "native ROM offset overflow",
            );
        }
        22 => {
            let record = NativeRecord::new(image, usize::MAX);
            panics_with(
                || {
                    let _ = record.long_le(1);
                },
                "native ROM offset overflow",
            );
        }
        23 => {
            let record = NativeRecord::new(image, 0);
            // Bounds failure at lane0 precedes any later signed lane overflow.
            panics_with(
                || {
                    let _ = record.long_le(isize::MAX);
                },
                // LEGACY(message-text): asserts on panic or Die text; should assert the failure kind instead.
                "index out of bounds",
            );
        }
        24 => panics_with(
            || {
                let _ = record.word_le(-2);
            },
            "native ROM offset overflow",
        ),
        25 => panics_with(
            || {
                let _ = record.long_le(-3);
            },
            "native ROM offset overflow",
        ),
        26 => panics_with(
            || {
                let _ = record.word_le(isize::MAX / 2);
            },
            // LEGACY(message-text): asserts on panic or Die text; should assert the failure kind instead.
            "index out of bounds",
        ),
        27 => panics_with(
            || {
                let _ = record.word_le(isize::MIN / 2);
            },
            "native ROM offset overflow",
        ),
        _ => unreachable!(),
    }
}
pub(crate) fn check_case(id: u32) -> bool {
    if id >= CASES {
        return false;
    }
    if id >= 6 {
        check_record_boundary(id);
        return true;
    }
    let mut game = Game::new();
    let first = RomImage::zeroed(0x40000);
    let second = RomImage::zeroed(0x40000);
    let at = crate::rtl::lorom_offset(0x868000) as usize;
    for (n, byte) in [0x12, 0x34, 0x56, 0x78].into_iter().enumerate() {
        first.write(at + n, byte);
        second.write(at + n, byte ^ 0xff);
    }
    game.loaded_rom = Some(second.clone());
    match id {
        0 => {
            game.register_native_rom(first.clone());
            let record = Runtime::new(&mut game).native_record(0x86, 0x8000);
            assert_eq!(record.byte(0), 0x12);
            assert_eq!(record.word_le(0), 0x3412);
            assert_eq!(record.long_le(0), 0x78563412);
            assert_eq!(game.capture_native_rom().read(at), 0x12);
            assert_eq!(game.loaded_rom.as_ref().unwrap().read(at), 0xed);
        }
        1 => {
            game.register_native_rom(first.clone());
            let captured = Runtime::new(&mut game).native_record(0x86, 0x8000);
            game.register_native_rom(second.clone());
            first.write(at, 0x9a);
            drop(first);
            assert_eq!(captured.word_le(0), 0x349a);
            let current = Runtime::new(&mut game).native_record(0x86, 0x8000);
            assert_eq!(current.word_le(0), 0xcbed);
        }
        2 => {
            game.register_native_rom(first.clone());
            let saved = game.capture_native_rom_binding();
            game.register_native_rom(second);
            game.restore_native_rom_binding(saved);
            assert_eq!(game.capture_native_rom().read(at), 0x12);
            game.restore_native_rom_binding(None);
            assert!(game.capture_native_rom_binding().is_none());
            assert!(game.legacy_native_rom_ptr().is_null());
        }
        3 => {
            assert!(game.legacy_native_rom_ptr().is_null());
            assert!(game.capture_native_rom_binding().is_none());
            assert!(catch_unwind(AssertUnwindSafe(|| game.capture_native_rom())).is_err());
            assert!(!game.g_fail);
        }
        4 => {
            let first_pointer = first.as_mut_ptr() as *const u8;
            let second_pointer = second.as_mut_ptr() as *const u8;
            game.register_native_rom(first);
            assert_eq!(game.legacy_native_rom_ptr(), first_pointer);
            game.register_native_rom(second);
            assert_eq!(game.legacy_native_rom_ptr(), second_pointer);
            assert!(!game.g_fail);
        }
        5 => {
            // The low-window diagnostic must precede the missing-owner failure.
            assert!(
                catch_unwind(AssertUnwindSafe(|| {
                    Runtime::new(&mut game).native_record(0x86, 0x2000)
                }))
                .is_err()
            );
            assert!(game.g_fail);
        }
        _ => unreachable!(),
    }
    true
}
