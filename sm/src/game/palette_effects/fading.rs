//! Palette transitions over the authoritative current/target colors and fade cursor.
#![forbid(unsafe_code)]
use crate::game_state::Game;
use crate::ram_map::word;

fn begin(duration: u16, game: &mut Game) -> bool {
    word!(game.palette_change_denom = duration);
    if word!(game.palette_change_num) <= duration.wrapping_add(1) {
        true
    } else {
        word!(game.palette_change_num = 0);
        false
    }
}

fn transition(step: u16, from: u16, to: u16, duration: u16) -> u16 {
    [0, 5, 10].into_iter().fold(0, |color, shift| {
        let from = (from >> shift) & 31;
        let to = (to >> shift) & 31;
        let component = if step == 0 {
            from
        } else if step.wrapping_sub(1) == duration {
            to
        } else {
            let remaining = duration.wrapping_sub(step).wrapping_add(1) as u8;
            let distance = from.abs_diff(to) << 8;
            let delta = if remaining == 0 {
                0xffff
            } else {
                distance / u16::from(remaining)
            };
            (if to < from {
                (from << 8).wrapping_sub(delta)
            } else {
                (from << 8).wrapping_add(delta)
            }) >> 8
        };
        color | component << shift
    })
}

fn blend(index: usize, step: u16, native: bool, game: &mut Game) {
    let current = word!(game.palette_buffer[index]);
    let target = word!(game.target_palettes[index]);
    // Native fade loops leave identical colors, including their high bit, untouched.
    if !native || current != target {
        let color = transition(step, current, target, word!(game.palette_change_denom));
        let bytes = if native {
            color.to_ne_bytes()
        } else {
            color.to_le_bytes()
        };
        let at = 0xc000 + index * 2;
        game.g_ram[at..at + 2].copy_from_slice(&bytes);
    }
}

pub(crate) fn all(duration: u16, game: &mut Game) -> u8 {
    if !begin(duration, game) {
        return 1;
    }
    for offset in (0u16..512).step_by(2) {
        word!(game.g_word_7EC404 = offset);
        blend(
            usize::from(offset >> 1),
            word!(game.palette_change_num),
            true,
            game,
        );
    }
    word!(game.palette_change_num += 1);
    0
}

pub(crate) fn background6(game: &mut Game) -> u8 {
    if !begin(12, game) {
        return 1;
    }
    for index in 96..112 {
        blend(index, word!(game.palette_change_num), false, game);
    }
    word!(game.palette_change_num += 1);
    0
}

pub(crate) fn selected(mut mask: u16, game: &mut Game) -> u8 {
    if !begin(12, game) {
        return 1;
    }
    word!(game.g_word_7EC404 = 0);
    while mask != 0 {
        if mask & 1 == 0 {
            word!(game.g_word_7EC404 += 32);
        } else {
            loop {
                let index = usize::from(word!(game.g_word_7EC404) >> 1);
                blend(index, word!(game.palette_change_num), true, game);
                word!(game.g_word_7EC404 += 2);
                if word!(game.g_word_7EC404) & 31 == 0 {
                    break;
                }
            }
        }
        mask >>= 1;
    }
    word!(game.palette_change_num += 1);
    0
}

pub(crate) fn file_select(game: &mut Game) -> u8 {
    word!(game.palette_change_denom = 15);
    // This comparison is signed and wraps; unlike the other fades, step zero advances.
    if 15i16.wrapping_sub(word!(game.palette_change_num) as i16) < 0 {
        word!(game.palette_change_num = 0);
        return 1;
    }
    for index in 0..32 {
        blend(
            index,
            word!(game.palette_change_num).wrapping_add(1),
            false,
            game,
        );
    }
    word!(game.palette_change_num += 1);
    0
}
