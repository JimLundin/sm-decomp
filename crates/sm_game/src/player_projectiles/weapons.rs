//! Beam input, creation and initial collision over ordinary projectile state.
#![forbid(unsafe_code)]
use super::host::{Host, PaletteEffect};
use super::motion::Flight;
use crate::{
    beam_definitions as definitions, projectile_initialization::Initialization,
    projectile_room::Axis,
};
use crate::{shot, shot_native};
#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Ordinary,
    Charged,
    Hyper,
}
pub fn clear_animation(h: &mut impl Host) {
    h.session_mut().projectiles.controls.clear_flare_animation();
}
pub fn update_input(h: &mut impl Host) {
    h.session_mut().projectiles.controls.previous_charge = h.session().projectiles.controls.charge;
    if h.session().inventory.hyper_beam != 0 || h.session().inventory.beams.active & 0x1000 == 0 {
        if h.inputs().shoot_button & h.inputs().held != 0 {
            fire(h, false);
        }
        return;
    }
    if h.session().projectiles.controls.pending_aim == 0 {
        if h.inputs().shoot_button & h.inputs().held != 0 {
            if h.session().projectiles.controls.charge.wrapping_sub(120) & 0x8000 != 0 {
                h.session_mut().projectiles.controls.charge =
                    h.session().projectiles.controls.charge.wrapping_add(1);
                if h.session().projectiles.controls.charge == 1 {
                    clear_animation(h);
                    fire(h, false);
                }
            } else if super::specials::fire(h) & 1 != 0 {
                h.session_mut().projectiles.controls.charge = 0;
                clear_animation(h);
                h.palette(PaletteEffect::Suit);
            }
            return;
        }
        if h.session().projectiles.controls.charge == 0 {
            return;
        }
    }
    let charged = h.session().projectiles.controls.charge.wrapping_sub(60) & 0x8000 == 0;
    h.session_mut().projectiles.controls.charge = 0;
    clear_animation(h);
    fire(h, charged);
}
fn fire(h: &mut impl Host, charged: bool) {
    let mode = if charged {
        Mode::Charged
    } else if h.session().inventory.hyper_beam != 0 {
        Mode::Hyper
    } else {
        Mode::Ordinary
    };
    if h.session().projectiles.count.wrapping_sub(5) & 0x8000 != 0
        && h.session().projectiles.controls.cooldown as u8 == 0
    {
        h.session_mut().projectiles.controls.cooldown = 1;
        h.session_mut().projectiles.count = h.session().projectiles.count.wrapping_add(1);
        let raw = (0..5)
            .map(|i| i * 2)
            .find(|&raw| shot!(h, raw, damage) == 0)
            .unwrap_or(8);
        if super::placement::place(h, raw) {
            h.session_mut().projectiles.controls.protection = 10;
            if mode != Mode::Hyper {
                shot!(h, raw, trail_time = 4; SliceLe);
            }
            let sound = match mode {
                Mode::Hyper => {
                    shot!(h, raw, kind = 0x9018; SliceLe);
                    definitions::CHARGED_SOUND[8]
                }
                Mode::Charged => {
                    let kind = h.session().inventory.beams.active & 0x100f | 0x8010;
                    shot!(h, raw, kind = kind; SliceLe);
                    definitions::CHARGED_SOUND[usize::from(kind & 15)]
                }
                Mode::Ordinary => {
                    let sound_index = h.session().inventory.beams.active & 15;
                    let kind = h.session().inventory.beams.active | 0x8000;
                    shot!(h, raw, kind = kind; SliceLe);
                    definitions::UNCHARGED_SOUND[usize::from(sound_index)]
                }
            };
            h.sound(1, 15, sound);
            h.session_mut().projectiles.controls.resume_sound = 0;
            super::animation::initialize(h, raw, Initialization::Projectile);
            h.probe("player_projectiles.fire.initialized");
            if mode == Mode::Hyper {
                shot!(h, raw, x_speed = 0; SliceLe);
                shot!(h, raw, y_speed = 0; SliceLe);
                h.session_mut().projectiles.selected = raw;
                initial_collision(h, raw, true);
                let selected = h.session().projectiles.selected;
                shot!(h, selected, damage = 1000; SliceLe);
                shot!(h, selected, motion = definitions::instruction(definitions::Motion::Hyper); SliceLe);
                super::motion::initialize_beam_speed(h, selected);
                h.session_mut().projectiles.controls.cooldown = 21;
                h.session_mut().projectiles.controls.glow = 0x8014;
                for (i, value) in [29, 5, 5].into_iter().enumerate() {
                    h.session_mut().projectiles.controls.flare_frames[i] = value;
                }
                for i in 0..3 {
                    h.session_mut().projectiles.controls.flare_timers[i] = 3;
                }
                h.session_mut().projectiles.controls.charge = 0x8000;
            } else {
                let wave = shot!(h, raw, kind) & 1 != 0;
                let manual = mode == Mode::Charged
                    || h.session().inventory.beams.active & 0x1000 != 0
                    || h.inputs().shoot_button & h.inputs().pressed != 0
                    || h.inputs().shoot_button & h.inputs().filtered_pressed != 0;
                let kind = shot!(h, raw, kind);
                let table: &[u8] = if manual {
                    &definitions::MANUAL_COOLDOWN
                } else {
                    &definitions::AUTO_COOLDOWN
                };
                h.session_mut().projectiles.controls.cooldown =
                    u16::from(table[usize::from(kind & 0x3f)]);
                shot!(h, raw, x_speed = 0; SliceLe);
                shot!(h, raw, y_speed = 0; SliceLe);
                h.session_mut().projectiles.selected = raw;
                initial_collision(h, raw, wave);
                let selected = if wave {
                    raw
                } else {
                    h.session().projectiles.selected
                };
                if wave || shot_native!(h, selected, kind) & super::kinds::TYPE_MASK == 0 {
                    let index = usize::from(shot!(h, selected, kind) & 15);
                    let motion =
                        definitions::motion(mode == Mode::Charged, index).unwrap_or_else(|| {
                            panic!("index out of bounds: the len is 12 but the index is {index}")
                        });
                    shot!(h, selected, motion = definitions::instruction(motion); SliceLe);
                    super::motion::initialize_beam_speed(h, selected);
                }
                if mode == Mode::Charged {
                    h.session_mut().projectiles.controls.glow = 4;
                }
            }
            return;
        }
    }
    if mode != Mode::Hyper
        && h.session()
            .projectiles
            .controls
            .previous_charge
            .wrapping_sub(16)
            & 0x8000
            == 0
    {
        h.session_mut().projectiles.controls.resume_sound = 0;
        h.sound(1, 15, 2);
    }
}
pub fn initial_collision(h: &mut impl Host, caller: u16, wave: bool) {
    let direction = shot!(h, caller, direction) & 15;
    if direction >= 10 {
        panic!("index out of bounds: the len is 10 but the index is {direction}");
    }
    let selected = h.session().projectiles.selected;
    let kind = if wave { Flight::Wave } else { Flight::NonWave };
    match direction {
        0 | 4 | 5 | 9 => {
            super::collision::scan(h, selected, kind, Axis::Y);
        }
        1 | 3 | 6 | 8 => {
            if super::collision::scan(h, selected, kind, Axis::X) & 1 == 0 {
                super::collision::scan(h, selected, kind, Axis::Y);
            }
        }
        2 => {
            super::collision::scan(h, selected, kind, Axis::X);
        }
        7 => {
            shot!(h, selected, x_speed = 0xffff; SliceNative);
            super::collision::scan(h, selected, kind, Axis::X);
        }
        _ => unreachable!(),
    }
    h.probe("player_projectiles.initial_collision.checked");
}
pub fn charged_instruction(index: usize) -> u16 {
    definitions::instruction(
        definitions::motion(true, index).unwrap_or_else(|| {
            panic!("index out of bounds: the len is 12 but the index is {index}")
        }),
    )
}
