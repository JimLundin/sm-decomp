//! Independent ownership/source checks for the native-content variant only.
//! Register directly under enemy_projectiles, separately from baseline carriers.
#![forbid(unsafe_code)]
use super::host::{Content as _, NativeRecord as _};
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;
use crate::native_rom::NativeRomPointer;
use crate::rom::RomImage;
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Arc, Mutex},
};
pub(crate) const CASES: u32 = 28;
/// Where a checked failure stopped execution: the record's own offset
/// arithmetic, or the ROM image read it resolved to.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Rejected {
    Offset,
    Read,
}
fn panics_with(operation: impl FnOnce(), expected: Rejected) {
    let location = Arc::new(Mutex::new(None));
    let observed = location.clone();
    let previous = Arc::new(std::panic::take_hook());
    let forward = previous.clone();
    std::panic::set_hook(Box::new(move |info| {
        *observed.lock().unwrap() = info.location().map(|at| at.file().replace('\\', "/"));
        forward(info);
    }));
    let result = catch_unwind(AssertUnwindSafe(operation));
    drop(std::panic::take_hook());
    std::panic::set_hook(
        Arc::try_unwrap(previous)
            .ok()
            .expect("released scoped panic hook"),
    );
    assert!(result.is_err(), "expected checked failure");
    let file = location.lock().unwrap().take().unwrap_or_default();
    let actual = if file.ends_with("enemy_projectiles/content.rs") {
        Some(Rejected::Offset)
    } else if file.ends_with("sm_data/src/cartridge.rs") {
        Some(Rejected::Read)
    } else {
        None
    };
    assert_eq!(actual, Some(expected), "checked failure raised at {file:?}");
}
fn check_record_boundary(id: u32) {
    use super::content::NativeRecord;
    let image = RomImage::from(vec![0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0]);
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
            Rejected::Read,
        ),
        10 => {
            let record = NativeRecord::new(image, 7);
            panics_with(
                || {
                    let _ = record.word_le(0);
                },
                Rejected::Read,
            );
        }
        11 => {
            let record = NativeRecord::new(image, 8);
            panics_with(
                || {
                    let _ = record.word_le(0);
                },
                Rejected::Read,
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
                Rejected::Read,
            );
        }
        16 => {
            let record = NativeRecord::new(RomImage::zeroed(0), 0);
            panics_with(
                || {
                    let _ = record.byte(0);
                },
                Rejected::Read,
            );
        }
        17 => {
            let record = NativeRecord::new(image, usize::MAX);
            panics_with(
                || {
                    let _ = record.byte(1);
                },
                Rejected::Offset,
            );
        }
        18 => panics_with(
            || {
                let _ = record.byte(-3);
            },
            Rejected::Offset,
        ),
        19 => panics_with(
            || {
                let _ = record.word_le(isize::MAX);
            },
            Rejected::Offset,
        ),
        20 => panics_with(
            || {
                let _ = record.word_le(isize::MIN);
            },
            Rejected::Offset,
        ),
        21 => {
            let record = NativeRecord::new(image, usize::MAX);
            panics_with(
                || {
                    let _ = record.word_le(1);
                },
                Rejected::Offset,
            );
        }
        22 => {
            let record = NativeRecord::new(image, usize::MAX);
            panics_with(
                || {
                    let _ = record.long_le(1);
                },
                Rejected::Offset,
            );
        }
        23 => {
            let record = NativeRecord::new(image, 0);
            // Bounds failure at lane0 precedes any later signed lane overflow.
            panics_with(
                || {
                    let _ = record.long_le(isize::MAX);
                },
                Rejected::Read,
            );
        }
        24 => panics_with(
            || {
                let _ = record.word_le(-2);
            },
            Rejected::Offset,
        ),
        25 => panics_with(
            || {
                let _ = record.long_le(-3);
            },
            Rejected::Offset,
        ),
        26 => panics_with(
            || {
                let _ = record.word_le(isize::MAX / 2);
            },
            Rejected::Read,
        ),
        27 => panics_with(
            || {
                let _ = record.word_le(isize::MIN / 2);
            },
            Rejected::Offset,
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
    let at = crate::rtl::lorom_offset(0x868000) as usize;
    let (mut first, mut second) = (vec![0; 0x40000], vec![0; 0x40000]);
    for (n, byte) in [0x12, 0x34, 0x56, 0x78].into_iter().enumerate() {
        first[at + n] = byte;
        second[at + n] = byte ^ 0xff;
    }
    let (first, second) = (RomImage::from(first), RomImage::from(second));
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
            drop(first);
            // The captured record keeps its allocation after the rebinding.
            assert_eq!(captured.word_le(0), 0x3412);
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
            let first_pointer = first.as_ptr() as *const u8;
            let second_pointer = second.as_ptr() as *const u8;
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
