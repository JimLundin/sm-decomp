//! One checked byte-lane projection; no canonical or broad algorithm fallback.
#![forbid(unsafe_code)]
use super::state::*;
use super::transport::{Codec, Port};
use crate::ram_map::{word_arrays as arrays, words as ram};
// Compatibility layout is private. Each physical owned word has exactly ONE
// representation, including the preinstruction/bomb-function overlap.
const SHOTS: usize = 0xb64;
const SHOT_BYTES: usize = 18 * 20;

fn shot_word(shot: &mut Shot, column: usize) -> &mut u16 {
    match column {
        0 => &mut shot.x,
        1 => &mut shot.y,
        2 => &mut shot.x_fraction,
        3 => &mut shot.y_fraction,
        4 => &mut shot.animation.x_radius,
        5 => &mut shot.animation.y_radius,
        6 => &mut shot.x_speed,
        7 => &mut shot.y_speed,
        8 => &mut shot.direction,
        9 => &mut shot.kind.0,
        10 => &mut shot.damage,
        11 => &mut shot.animation.cursor.0,
        12 => &mut shot.animation.remaining,
        13 => &mut shot.motion.0,
        14 => &mut shot.variable,
        15 => &mut shot.trail_time,
        16 => &mut shot.auxiliary,
        17 => &mut shot.animation.sprite.0,
        _ => unreachable!(),
    }
}
fn shot_value(shot: &Shot, column: usize) -> u16 {
    match column {
        0 => shot.x,
        1 => shot.y,
        2 => shot.x_fraction,
        3 => shot.y_fraction,
        4 => shot.animation.x_radius,
        5 => shot.animation.y_radius,
        6 => shot.x_speed,
        7 => shot.y_speed,
        8 => shot.direction,
        9 => shot.kind.0,
        10 => shot.damage,
        11 => shot.animation.cursor.0,
        12 => shot.animation.remaining,
        13 => shot.motion.0,
        14 => shot.variable,
        15 => shot.trail_time,
        16 => shot.auxiliary,
        17 => shot.animation.sprite.0,
        _ => unreachable!(),
    }
}
#[derive(Clone, Copy)]
enum Control {
    Count,
    BombCount,
    Selected,
    Cooldown,
    SpreadCharge,
    OrbitStep,
    Charge,
    PreviousCharge,
    Glow,
    ChargePalette,
    ResumeSound,
    EchoSound,
    PendingAim,
    Protection,
    SelectedWeapon,
    AutomaticWeapon,
    WeaponChanged,
    FlareFrame0,
    FlareFrame1,
    FlareFrame2,
    FlareTimer0,
    FlareTimer1,
    FlareTimer2,
}
const CONTROLS: [(usize, Control); 23] = [
    (ram::projectile_counter, Control::Count),
    (ram::bomb_counter, Control::BombCount),
    (ram::projectile_index, Control::Selected),
    (ram::cooldown_timer, Control::Cooldown),
    (
        ram::bomb_spread_charge_timeout_counter,
        Control::SpreadCharge,
    ),
    (ram::used_for_sba_attacksB60, Control::OrbitStep),
    (ram::flare_counter, Control::Charge),
    (ram::prev_beam_charge_counter, Control::PreviousCharge),
    (ram::charged_shot_glow_timer, Control::Glow),
    (ram::samus_charge_palette_index, Control::ChargePalette),
    (ram::play_resume_charging_beam_sfx, Control::ResumeSound),
    (ram::samus_echoes_sound_flag, Control::EchoSound),
    (
        ram::new_projectile_direction_changed_pose,
        Control::PendingAim,
    ),
    (ram::projectile_invincibility_timer, Control::Protection),
    (ram::hud_item_index, Control::SelectedWeapon),
    (
        ram::samus_auto_cancel_hud_item_index,
        Control::AutomaticWeapon,
    ),
    (ram::hud_item_changed_this_frame, Control::WeaponChanged),
    (ram::flare_animation_frame + 0, Control::FlareFrame0),
    (ram::flare_animation_frame + 2, Control::FlareFrame1),
    (ram::flare_animation_frame + 4, Control::FlareFrame2),
    (ram::flare_animation_timer + 0, Control::FlareTimer0),
    (ram::flare_animation_timer + 2, Control::FlareTimer1),
    (ram::flare_animation_timer + 4, Control::FlareTimer2),
];
const TRAILS: [usize; 10] = [
    arrays::projectiletrail_left_instr_timer,
    arrays::projectiletrail_right_instr_timer,
    arrays::projectiletrail_left_instr_list_ptr,
    arrays::projectiletrail_right_instr_list_ptr,
    arrays::projectiletrail_left_tile_and_attribs,
    arrays::projectiletrail_right_tile_and_attribs,
    arrays::projectiletrail_left_x_pos,
    arrays::projectiletrail_right_x_pos,
    arrays::projectiletrail_left_y_pos,
    arrays::projectiletrail_right_y_pos,
];
#[derive(Clone, Copy)]
enum Field {
    Shot { slot: usize, column: usize },
    Control(Control),
    Trail { pair: usize, column: usize },
    Launch { direction: usize, low: bool },
}
fn field(address: usize) -> Option<(Field, usize)> {
    if (SHOTS..SHOTS + SHOT_BYTES).contains(&address) {
        let offset = address - SHOTS;
        return Some((
            Field::Shot {
                slot: offset % 20 / 2,
                column: offset / 20,
            },
            offset % 2,
        ));
    }
    for &(base, control) in &CONTROLS {
        if address == base || address == base + 1 {
            return Some((Field::Control(control), address - base));
        }
    }
    for (column, &base) in TRAILS.iter().enumerate() {
        if (base..base + 36).contains(&address) {
            return Some((
                Field::Trail {
                    pair: (address - base) / 2,
                    column,
                },
                (address - base) % 2,
            ));
        }
    }
    if (0xdaa..0xdba).contains(&address) {
        let offset = address - 0xdaa;
        return Some((
            Field::Launch {
                direction: offset / 4,
                low: offset % 4 >= 2,
            },
            offset % 2,
        ));
    }
    None
}
impl PlayerProjectiles {
    fn word(&self, field: Field) -> u16 {
        match field {
            Field::Shot { slot, column } => shot_value(&self.shots[slot], column),
            Field::Control(control) => match control {
                Control::Count => self.count,
                Control::BombCount => self.bomb_count,
                Control::Selected => self.selected,
                Control::Cooldown => self.controls.cooldown,
                Control::SpreadCharge => self.controls.spread_charge,
                Control::OrbitStep => self.controls.orbit_step,
                Control::Charge => self.controls.charge,
                Control::PreviousCharge => self.controls.previous_charge,
                Control::Glow => self.controls.glow,
                Control::ChargePalette => self.controls.charge_palette,
                Control::ResumeSound => self.controls.resume_sound,
                Control::EchoSound => self.controls.echo_sound,
                Control::PendingAim => self.controls.pending_aim,
                Control::Protection => self.controls.protection,
                Control::SelectedWeapon => self.controls.selected_weapon,
                Control::AutomaticWeapon => self.controls.automatic_weapon,
                Control::WeaponChanged => self.controls.weapon_changed,
                Control::FlareFrame0 => self.controls.flare_frames[0],
                Control::FlareFrame1 => self.controls.flare_frames[1],
                Control::FlareFrame2 => self.controls.flare_frames[2],
                Control::FlareTimer0 => self.controls.flare_timers[0],
                Control::FlareTimer1 => self.controls.flare_timers[1],
                Control::FlareTimer2 => self.controls.flare_timers[2],
            },
            Field::Trail { pair, column } => {
                let side = if column % 2 == 0 {
                    &self.trails.pairs[pair].left
                } else {
                    &self.trails.pairs[pair].right
                };
                match column / 2 {
                    0 => side.remaining,
                    1 => side.cursor.0,
                    2 => side.tile,
                    3 => side.position.x,
                    4 => side.position.y,
                    _ => unreachable!(),
                }
            }
            Field::Launch { direction, low } => {
                let value = self.launch.displacements[direction] as u32;
                if low {
                    value as u16
                } else {
                    (value >> 16) as u16
                }
            }
        }
    }
    fn set_word(&mut self, field: Field, value: u16) {
        match field {
            Field::Shot { slot, column } => *shot_word(&mut self.shots[slot], column) = value,
            Field::Control(control) => {
                *match control {
                    Control::Count => &mut self.count,
                    Control::BombCount => &mut self.bomb_count,
                    Control::Selected => &mut self.selected,
                    Control::Cooldown => &mut self.controls.cooldown,
                    Control::SpreadCharge => &mut self.controls.spread_charge,
                    Control::OrbitStep => &mut self.controls.orbit_step,
                    Control::Charge => &mut self.controls.charge,
                    Control::PreviousCharge => &mut self.controls.previous_charge,
                    Control::Glow => &mut self.controls.glow,
                    Control::ChargePalette => &mut self.controls.charge_palette,
                    Control::ResumeSound => &mut self.controls.resume_sound,
                    Control::EchoSound => &mut self.controls.echo_sound,
                    Control::PendingAim => &mut self.controls.pending_aim,
                    Control::Protection => &mut self.controls.protection,
                    Control::SelectedWeapon => &mut self.controls.selected_weapon,
                    Control::AutomaticWeapon => &mut self.controls.automatic_weapon,
                    Control::WeaponChanged => &mut self.controls.weapon_changed,
                    Control::FlareFrame0 => &mut self.controls.flare_frames[0],
                    Control::FlareFrame1 => &mut self.controls.flare_frames[1],
                    Control::FlareFrame2 => &mut self.controls.flare_frames[2],
                    Control::FlareTimer0 => &mut self.controls.flare_timers[0],
                    Control::FlareTimer1 => &mut self.controls.flare_timers[1],
                    Control::FlareTimer2 => &mut self.controls.flare_timers[2],
                } = value
            }
            Field::Trail { pair, column } => {
                let side = if column % 2 == 0 {
                    &mut self.trails.pairs[pair].left
                } else {
                    &mut self.trails.pairs[pair].right
                };
                *match column / 2 {
                    0 => &mut side.remaining,
                    1 => &mut side.cursor.0,
                    2 => &mut side.tile,
                    3 => &mut side.position.x,
                    4 => &mut side.position.y,
                    _ => unreachable!(),
                } = value;
            }
            Field::Launch { direction, low } => {
                let old = self.launch.displacements[direction] as u32;
                self.launch.displacements[direction] = if low {
                    (old & 0xffff0000) | u32::from(value)
                } else {
                    (old & 0xffff) | (u32::from(value) << 16)
                } as i32;
            }
        }
    }
}
fn owned_words() -> impl Iterator<Item = usize> {
    (SHOTS..SHOTS + SHOT_BYTES)
        .step_by(2)
        .chain(CONTROLS.iter().map(|&(address, _)| address))
        .chain(
            TRAILS
                .into_iter()
                .flat_map(|base| (base..base + 36).step_by(2)),
        )
        .chain((0xdaa..0xdba).step_by(2))
}

// Both the codec and transport stay inside the projectile integration namespace.
pub(super) type Transaction<'g> = Port<'g, PlayerProjectiles>;
impl Codec for PlayerProjectiles {
    fn decode(ram: &[u8]) -> Self {
        let mut state = Self::default();
        for address in owned_words() {
            let (field, _) = field(address).unwrap();
            state.set_word(field, u16::from_le_bytes([ram[address], ram[address + 1]]));
        }
        state
    }
    fn encode(&self, ram: &mut [u8]) {
        for address in owned_words() {
            let (field, _) = field(address).unwrap();
            ram[address..address + 2].copy_from_slice(&self.word(field).to_le_bytes());
        }
    }
    fn byte(&self, address: usize) -> Option<u8> {
        let (field, lane) = field(address)?;
        Some(self.word(field).to_le_bytes()[lane])
    }
    fn put_byte(&mut self, address: usize, value: u8) -> bool {
        let Some((field, lane)) = field(address) else {
            return false;
        };
        let mut bytes = self.word(field).to_le_bytes();
        bytes[lane] = value;
        self.set_word(field, u16::from_le_bytes(bytes));
        true
    }
}

// Match checked_word!'s original value-before-address and per-byte assignment.
macro_rules! word {
    ($g:ident.$field:ident) => {
        $g.read_le(crate::ram_map::words::$field)
    };
    ($g:ident.$field:ident[$index:expr]) => {
        $g.read_le(
            crate::ram_map::word_arrays::$field
                .checked_add_signed(($index as isize).checked_mul(2).unwrap())
                .unwrap(),
        )
    };
    ($g:ident.$field:ident = $value:expr) => {
        $g.write_word_le($value, crate::ram_map::words::$field)
    };
    ($g:ident.$field:ident[$index:expr] = $value:expr) => {
        $g.write_word_le(
            $value,
            crate::ram_map::word_arrays::$field
                .checked_add_signed(($index as isize).checked_mul(2).unwrap())
                .unwrap(),
        )
    };
    ($g:ident, $offset:expr;) => {
        $g.read_le($offset)
    };
    ($g:ident, $offset:expr; = $value:expr) => {
        $g.write_word_le($value, $offset)
    };
    ($g:ident.$field:ident[$index:expr] += $value:expr) => {{
        let offset = crate::ram_map::word_arrays::$field
            .checked_add_signed(($index as isize).checked_mul(2).unwrap())
            .unwrap();
        let prior = $g.read_le(offset);
        $g.write_word_le(prior.wrapping_add($value), offset);
    }};
}
