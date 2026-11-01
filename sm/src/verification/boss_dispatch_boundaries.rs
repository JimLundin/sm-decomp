//! Constructed migration boundaries registered in the existing DLL actor checks.
#![forbid(unsafe_code)]
use crate::{game::session_runtime::Runtime, game_state::Game, rom::RomImage};
use std::panic::{AssertUnwindSafe, catch_unwind};

fn put(game: &mut Game, at: usize, value: u16) {
    game.ram().copy_in(at, &value.to_le_bytes());
}
fn word(game: &Game, at: usize) -> u16 {
    u16::from_le_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)])
}
fn fixture(length: usize) -> Box<Game> {
    let mut game = Game::new();
    let mut bytes = vec![0; length];
    if length > 0x110000 {
        for (address, value) in [(0xa0b7ee, 0x1234u16), (0xa0b86e, 0x5678)] {
            let at = crate::rtl::lorom_offset(address) as usize;
            bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
        }
    }
    let image = RomImage::from(bytes);
    game.loaded_rom = Some(image.clone());
    game.register_native_rom(image);
    put(&mut game, 0x5b6, 1); // Skip finish-return's random sprite effect.
    game
}

fn assert_parts(runtime: &Runtime<'_>, expected: (u16, u16)) {
    for part in [0x40, 0x80, 0xc0] {
        assert_eq!(
            (
                runtime.native_word(0xf7a + part),
                runtime.native_word(0xf7e + part)
            ),
            expected
        );
    }
}
fn alias_fixture(selector: u16, function: u16) -> Box<Game> {
    let mut game = fixture(0x200000);
    let k = usize::from(selector);
    for (at, value) in [
        (0xf7a, 0x80),
        (0xf7e, 0x250),
        (0xfa8 + k, function),
        (0xf7a + k, 0x110),
        (0xf7e + k, 0x240),
        (0xf7c + k, 0),
        (0xf80 + k, 0),
        (0xfac + k, 0x321),
        (0xfae + k, 0x456),
        (0x7814, 0),
        (0x7816, 16),
        (0x7818, 0xff03),
        (0x8010 + k, 1),
        (0x8012 + k, 0),
        (0x8014 + k, 2),
        (0x8016 + k, 0),
        (0x8806, 0),
    ] {
        put(&mut game, at, value);
    }
    if function == 0xeb9b {
        put(&mut game, 0xf7a, 0x80);
        put(&mut game, 0xf7e, 0x250);
    }
    put(&mut game, sm_game::ram::words::CUR_ENEMY_INDEX, selector);
    game
}

// Movies never install Spore function aliases in Draygon body/eye or select odd/full off-table actors.
pub(super) fn spore_aliases_reuse_actual_runtime() {
    for body in [true, false] {
        for selector in [0u16, 1, 0x8000, 0xffff] {
            for function in [0xeb1bu16, 0xeb52, 0xeb9b, 0xebee] {
                let mut game = alias_fixture(selector, function);
                let k = usize::from(selector);
                let mut runtime = Runtime::new(&mut game);
                runtime.actor_ai_legacy(if body { 10847996 } else { 10863750 });
                assert_eq!(runtime.word(sm_game::ram::words::CUR_ENEMY_INDEX), selector);
                match function {
                    0xeb1b => {
                        assert_eq!(runtime.native_word(0xf7e + k), 0x241);
                        assert_eq!(
                            (
                                runtime.word(0x7814),
                                runtime.word(0x7816),
                                runtime.word(0x7818)
                            ),
                            (192, 48, 1)
                        );
                    }
                    0xeb52 => {
                        assert_eq!(
                            (
                                runtime.native_word(0xf7a + k),
                                runtime.native_word(0xf7e + k)
                            ),
                            (0x321, 0x456)
                        );
                        assert_eq!(runtime.word(0x7814), 3);
                    }
                    0xeb9b => {
                        assert_eq!(runtime.word(0x8806), 192);
                        assert_eq!(
                            (
                                runtime.native_word(0x8010 + k),
                                runtime.native_word(0x8012 + k),
                                runtime.native_word(0x8014 + k),
                                runtime.native_word(0x8016 + k)
                            ),
                            (0, 0x1234, 0, 0x5678)
                        );
                    }
                    0xebee => {
                        assert_eq!(
                            (
                                runtime.native_word(0xf7a + k),
                                runtime.native_word(0xf7e + k)
                            ),
                            (0x111, 0x23e)
                        );
                        assert_eq!(runtime.session().compatibility_enemy_scratch, 0);
                    }
                    _ => unreachable!(),
                }
                if function != 0xeb9b {
                    let anchor = &runtime.session().enemy_projectiles.slots[14];
                    assert_eq!((anchor.x.whole, anchor.y.whole), (128, 560));
                }
                if body {
                    assert_parts(
                        &runtime,
                        (runtime.native_word(0xf7a), runtime.native_word(0xf7e)),
                    );
                }
                runtime.write_slice_le(0x1c37, 0xbeef);
                assert_eq!(runtime.word(0x1c37), 0xbeef);
                drop(runtime);
                assert_eq!(word(&game, 0x1c37), 0xbeef);
                assert!(!game.g_fail);
            }
        }
    }
}

// Movies never truncate the loaded projectile definition during physical Draygon eye dispatch.
pub(super) fn physical_eye_failure_preserves_allocation_prefix() {
    for body in [true, false] {
        let mut game = fixture(0x200000);
        put(&mut game, 0xfa8, 0x804b);
        put(&mut game, 0xf7a, 0x110);
        put(&mut game, 0xf7e, 0x240);
        let mut runtime = Runtime::new(&mut game);
        runtime.actor_ai_legacy(if body { 10847996 } else { 10863750 });
        assert_parts(&runtime, if body { (0x110, 0x240) } else { (0, 0) });
        drop(runtime);
        assert!(!game.g_fail);
    }
    let mut game = fixture(0x100);
    put(&mut game, sm_game::ram::words::CUR_ENEMY_INDEX, 0x40);
    put(&mut game, 0xfa8 + 0x40, 0xc48d);
    put(&mut game, 0xf7a + 0x40, 0x110);
    put(&mut game, 0xf7e + 0x40, 0x240);
    put(&mut game, 0xfb2 + 0x40, 0x1234); // Angle cache must not be updated after failure.
    put(&mut game, 0xf92 + 0x40, 0x5678);
    put(&mut game, 0x1a03 + 34, 0x9876); // Allocation's old pre-action survives missing word.
    let mut runtime = Runtime::new(&mut game);
    assert!(catch_unwind(AssertUnwindSafe(|| runtime.actor_ai_legacy(10863750))).is_err());
    assert!(!runtime.has_session_for_verification());
    assert!(catch_unwind(AssertUnwindSafe(|| drop(runtime))).is_ok());
    assert_eq!(
        (game.eproj_spawn_pt.x, game.eproj_spawn_pt.y),
        (0xf8, 0x220)
    );
    assert_eq!(word(&game, 0x1993), 0x18);
    assert_eq!(word(&game, 0x1997 + 34), 0xe509);
    assert_eq!(word(&game, 0x19bb + 34), 0);
    assert_eq!(word(&game, 0x1a03 + 34), 0x9876);
    assert_eq!(
        (word(&game, 0xfb2 + 0x40), word(&game, 0xf92 + 0x40)),
        (0x1234, 0x5678)
    );
    assert_eq!(
        (word(&game, 0xf7a + 0x80), word(&game, 0xf7e + 0x80)),
        (0, 0)
    );
    assert!(!game.g_fail);
}
