//! Actual even-area Kraid read/store callers, including palette and music prefixes.
//! The same source is used for predecessor and candidate; no odd typed read here.
use crate::{game::sm_a7, game_state::Game};
use std::panic::{AssertUnwindSafe, catch_unwind};
pub(crate) const CASES: u32 = 8;
fn put(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn native_put(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_ne_bytes());
}
fn record(output: &mut Vec<u8>, tag: u8, returned: u16, game: &Game) {
    output.push(tag);
    output.extend_from_slice(&returned.to_le_bytes());
    output.extend_from_slice(&game.g_ram);
    output.push(u8::from(game.g_fail));
}
pub(crate) fn capture_case(id: u32) -> Option<Vec<u8>> {
    if id >= CASES {
        return None;
    }
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut game = Game::new();
        for (i, b) in game.g_ram.iter_mut().enumerate() {
            *b = (i.wrapping_mul(37) ^ (i >> 8) ^ 0x5a) as u8;
        }
        let area = [0u16, 6, 65534, 0][id as usize / 2];
        let dead = id as u16 & 1;
        let boss_at = 0xd828 + usize::from(area);
        put(&mut game.g_ram, 0x79f, area);
        native_put(&mut game.g_ram, boss_at, 0xa500 | dead);
        put(&mut game.g_ram, 0x5e7, 0xa534);
        put(&mut game.g_ram, 0x998, 8);
        put(&mut game.g_ram, 0x639, 0);
        put(&mut game.g_ram, 0x63b, 6);
        put(&mut game.g_ram, 0xc400, if id >= 6 { 13 } else { 14 });
        put(&mut game.g_ram, 0xc402, 0xbeef);
        put(&mut game.g_ram, 0xfa8, 0x1234);
        for index in 0..16 {
            put(&mut game.g_ram, 0xc0c0 + 2 * index, (index as u16) * 0x421);
            put(
                &mut game.g_ram,
                0xc2c0 + 2 * index,
                0x7fff - (index as u16) * 0x421,
            );
        }
        game.g_fail = false;
        game.VERIFY_ACTIVE = true;
        let mut expected = game.g_ram.to_vec();
        let mut output = b"KRDF1".to_vec();
        output.extend_from_slice(&id.to_le_bytes());
        record(&mut output, 0, 0, &game);
        let result = unsafe { sm_a7::Kraid_CheckIfDead(&mut game) };
        assert_eq!(result, dead, "flags.kraid.read.initial");
        assert_eq!(
            &game.g_ram[..],
            &expected[..],
            "flags.kraid.read.scratch_free"
        );
        record(&mut output, 1, result, &game);
        if id >= 6 {
            unsafe {
                sm_a7::Kraid_FadeInBg_FadeInBp6(&mut game);
            }
            // Last interpolation step copies actual target colours and returns
            // before music, native boss read/store and enemy transition.
            let targets = expected[0xc2c0..0xc2e0].to_vec();
            expected[0xc0c0..0xc0e0].copy_from_slice(&targets);
            put(&mut expected, 0xc400, 14);
            put(&mut expected, 0xc402, 12);
            assert_eq!(
                &game.g_ram[..],
                &expected[..],
                "flags.kraid.palette_before_completion"
            );
            assert!(!game.g_fail, "flags.kraid.palette.diagnostic");
            record(&mut output, 2, 0, &game);
        }
        unsafe {
            sm_a7::Kraid_FadeInBg_FadeInBp6(&mut game);
        }
        put(&mut expected, 0xc400, 0);
        put(&mut expected, 0xc402, 12);
        put(&mut expected, 0x619, 3);
        put(&mut expected, 0x629, 8);
        put(&mut expected, 0x639, 2);
        native_put(&mut expected, boss_at, 0xa501);
        put(
            &mut expected,
            0xfa8,
            if dead != 0 { 0xc851 } else { 0xc843 },
        );
        assert_eq!(
            &game.g_ram[..],
            &expected[..],
            "flags.kraid.completion_music_boss_transition"
        );
        assert!(!game.g_fail, "flags.kraid.completion.diagnostic");
        record(&mut output, 3, 0, &game);
        let result = unsafe { sm_a7::Kraid_CheckIfDead(&mut game) };
        assert_eq!(result, 1, "flags.kraid.read.after_completion");
        assert_eq!(
            &game.g_ram[..],
            &expected[..],
            "flags.kraid.final_whole_ram"
        );
        record(&mut output, 4, result, &game);
        output
    }));
    match result {
        Ok(bytes) => Some(bytes),
        Err(_) => {
            eprintln!("Kraid defined case {id} failed");
            None
        }
    }
}
