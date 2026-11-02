//! Palette transitions over the Session-owned current and target colours and
//! fade cursor, with ordered byte stores on the selected actor image.

use crate::visual_state::PaletteFade;
pub trait Host:
    crate::actors::boss_fields_owned::State + crate::enemy_projectiles::host::Graphics
{
}
impl<T: crate::actors::boss_fields_owned::State + crate::enemy_projectiles::host::Graphics> Host
    for T
{
}
fn read_word<H: Host>(host: &H, at: usize) -> u16 {
    u16::from_le_bytes([
        crate::actors::boss_fields_owned::read_byte(host, at),
        crate::actors::boss_fields_owned::read_byte(host, at + 1),
    ])
}
fn store_word<H: Host>(host: &mut H, at: usize, value: u16) {
    for (lane, byte) in value.to_le_bytes().into_iter().enumerate() {
        crate::actors::boss_fields_owned::store_byte(host, at + lane, byte);
    }
}

/// The current colours, then the targets, then the fade's step, duration and
/// cursor words, as `session_encoding` lays them out.
const COLORS: usize = 0xc000;
const TARGETS: usize = 0xc200;
const FADE: [usize; 3] = [0xc400, 0xc402, 0xc404];

fn read_fade<H: Host>(game: &H) -> PaletteFade {
    let [step, duration, cursor] = FADE.map(|at| read_word(game, at));
    PaletteFade {
        step,
        duration,
        cursor,
    }
}

/// Store the fade's words; each fade stores its duration, so every word it
/// leaves unchanged is stored with its own value.
fn store_fade<H: Host>(fade: PaletteFade, game: &mut H) {
    for (at, value) in FADE
        .into_iter()
        .zip([fade.step, fade.duration, fade.cursor])
    {
        store_word(game, at, value);
    }
}

/// Start a fade of `duration` steps; a step past its end restarts it and reports done.
fn begin(fade: &mut PaletteFade, duration: u16) -> bool {
    fade.duration = duration;
    if fade.step <= duration.wrapping_add(1) {
        true
    } else {
        fade.step = 0;
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

fn blend<H: Host>(fade: &PaletteFade, index: usize, step: u16, native: bool, game: &mut H) {
    let current = read_word(game, COLORS + 2 * index);
    let target = read_word(game, TARGETS + 2 * index);
    // Native fade loops leave identical colors, including their high bit, untouched.
    if !native || current != target {
        let color = transition(step, current, target, fade.duration);
        let bytes = if native {
            color.to_ne_bytes()
        } else {
            color.to_le_bytes()
        };
        crate::enemy_projectiles::host::Graphics::set_palette_buffer(
            game,
            isize::try_from(index).expect("fade palette index"),
            u16::from_le_bytes(bytes),
        );
    }
}

/// Run `steps` over the fade read from the projection and store it back.
fn fading<H: Host>(game: &mut H, steps: impl FnOnce(&mut PaletteFade, &mut H) -> u8) -> u8 {
    let mut fade = read_fade(game);
    let done = steps(&mut fade, game);
    store_fade(fade, game);
    done
}

pub fn all<H: Host>(duration: u16, game: &mut H) -> u8 {
    fading(game, |fade, game| {
        if !begin(fade, duration) {
            return 1;
        }
        for offset in (0u16..512).step_by(2) {
            fade.cursor = offset;
            blend(fade, usize::from(offset >> 1), fade.step, true, game);
        }
        fade.step = fade.step.wrapping_add(1);
        0
    })
}

pub fn background6<H: Host>(game: &mut H) -> u8 {
    fading(game, |fade, game| {
        if !begin(fade, 12) {
            return 1;
        }
        for index in 96..112 {
            blend(fade, index, fade.step, false, game);
        }
        fade.step = fade.step.wrapping_add(1);
        0
    })
}

pub fn selected<H: Host>(mut mask: u16, game: &mut H) -> u8 {
    fading(game, |fade, game| {
        if !begin(fade, 12) {
            return 1;
        }
        fade.cursor = 0;
        while mask != 0 {
            if mask & 1 == 0 {
                fade.cursor = fade.cursor.wrapping_add(32);
            } else {
                loop {
                    blend(fade, usize::from(fade.cursor >> 1), fade.step, true, game);
                    fade.cursor = fade.cursor.wrapping_add(2);
                    if fade.cursor & 31 == 0 {
                        break;
                    }
                }
            }
            mask >>= 1;
        }
        fade.step = fade.step.wrapping_add(1);
        0
    })
}

pub fn file_select<H: Host>(game: &mut H) -> u8 {
    fading(game, |fade, game| {
        fade.duration = 15;
        // This comparison is signed and wraps; unlike the other fades, step zero advances.
        if 15i16.wrapping_sub(fade.step as i16) < 0 {
            fade.step = 0;
            return 1;
        }
        for index in 0..32 {
            blend(fade, index, fade.step.wrapping_add(1), false, game);
        }
        fade.step = fade.step.wrapping_add(1);
        0
    })
}
