//! Fireflea, the glowing bug of Norfair's dark rooms, which flies a circle or
//! a vertical line; killing one darkens the room (bank $A3:8C0F-$A3:8EA5).
#![forbid(unsafe_code)]
use crate::actors::host::{ActorHost, ActorRuntime, Death, Shot};
use crate::actors::layouts::part_view;
use crate::ram::RawRam as _;
use crate::ram::{slot, word};
use sm_data::cartridge::lorom_offset;
use sm_data::enemy_speeds::SpeedTable;

/// What the fireflea needs beyond the shared actor operations.
pub trait Host: ActorHost + ActorRuntime {
    /// `amplitude` times the cosine of `angle`.
    fn actor_cosine(&mut self, angle: u16, amplitude: u16) -> u16;
    /// `amplitude` times the sine of `angle`.
    fn actor_sine(&mut self, angle: u16, amplitude: u16) -> u16;
    /// The enemy death routine, with the normal death animation.
    fn die_normally(&mut self);
    /// Darken the room by one fireflea's light.
    fn darken_room(&mut self);
}

part_view! {
    /// The fireflea's view of its slot.
    Fireflea: EnemyLayout40 {
        words: [
            angle_subdelta, angle_delta, radius, angle, x_centre, y_centre, pattern, speed_and_radius,
        ],
        pad1: [u8; 26696],
        words: [
            min_y, max_y, speed_table_index, var_03, var_04, var_05, var_06, var_07, var_08, var_09,
            var_0A, var_0B, var_0C, var_0D, var_0E, var_0F, var_10, var_11, var_12, var_13, var_14,
            var_15, var_16, var_17, var_18, var_19, var_1A, var_1B, var_1C, var_1D, var_1E, var_1F,
        ],
    }
}

/// $A3:8C2F: the fireflea's instruction list; $A3:8D1D: its radii.
const INSTRUCTION_LIST: u16 = 0x8c2f;
const RADII: u32 = 0xa3_8d1d;
/// `pattern` bit 1 flies a circle, else a vertical line; bit 0 clear negates
/// the speed (4 more in the speed table index).
const CIRCLE: u16 = 2;
const POSITIVE: u16 = 1;
const NEGATE: u16 = 4;

fn current(game: &(impl Host + ?Sized)) -> i32 {
    i32::from(word!(game.cur_enemy_index))
}
fn angle<const INITIAL: bool>(flea: i32, game: &(impl Host + ?Sized)) -> u16 {
    if INITIAL {
        slot!(game, Fireflea[flea].angle)
    } else {
        u16::from(slot!(byte game, Fireflea[flea].angle[1]))
    }
}
/// Set initial circling fireflea position ($A3:8DAE), or circle on.
fn circle<const INITIAL: bool>(flea: i32, game: &mut (impl Host + ?Sized)) {
    // The centre is read before the angle and radius, as the original did.
    let centre = slot!(game, Fireflea[flea].x_centre);
    let a = angle::<INITIAL>(flea, game);
    let radius = slot!(game, Fireflea[flea].radius);
    let x = centre.wrapping_add(game.actor_cosine(a, radius));
    slot!(game, Fireflea[flea].base.x_pos = x);
    let centre = slot!(game, Fireflea[flea].y_centre);
    let a = angle::<INITIAL>(flea, game);
    let radius = slot!(game, Fireflea[flea].radius);
    let y = centre.wrapping_add(game.actor_sine(a, radius));
    slot!(game, Fireflea[flea].base.y_pos = y);
}
/// Set fireflea speed ($A3:8D75).
fn set_speed(flea: i32, game: &mut (impl Host + ?Sized)) {
    let mut index = 8 * u16::from(slot!(byte game, Fireflea[flea].speed_and_radius[0]));
    if slot!(game, Fireflea[flea].pattern) & POSITIVE == 0 {
        index += NEGATE;
    }
    slot!(game, Fireflea[flea].speed_table_index = index);
    let index = (index >> 1) as isize;
    let high = game.enemy_speed(SpeedTable::Linear, index);
    slot!(game, Fireflea[flea].angle_delta = high);
    let low = game.enemy_speed(SpeedTable::Linear, index + 1);
    slot!(game, Fireflea[flea].angle_subdelta = low);
}
/// Set fireflea radius ($A3:8D9C).
fn set_radius(flea: i32, game: &mut (impl Host + ?Sized)) {
    let image = game.native_image();
    let index = usize::from(slot!(byte game, Fireflea[flea].speed_and_radius[1]));
    let radius = image.read(lorom_offset(RADII) as usize + 2 * index);
    slot!(game, Fireflea[flea].radius = u16::from(radius));
}
/// Initialisation AI - fireflea ($A3:8D2D).
pub fn initialize(game: &mut (impl Host + ?Sized)) {
    let flea = current(game);
    slot!(
        game,
        Fireflea[flea].base.current_instruction = INSTRUCTION_LIST
    );
    let circling = slot!(game, Fireflea[flea].pattern) & CIRCLE != 0;
    if circling {
        // Set fireflea centre ($A3:8D5D) and initial angle ($A3:8D6A).
        let flea = current(game);
        slot!(
            game,
            Fireflea[flea].x_centre = slot!(game, Fireflea[flea].base.x_pos)
        );
        slot!(
            game,
            Fireflea[flea].y_centre = slot!(game, Fireflea[flea].base.y_pos)
        );
        let flea = current(game);
        let angle = u16::from(slot!(byte game, Fireflea[flea].pattern[1]));
        slot!(game, Fireflea[flea].angle = angle << 8);
    }
    set_speed(current(game), game);
    set_radius(current(game), game);
    if circling {
        circle::<true>(current(game), game);
    } else {
        // Set fireflea Y extrema ($A3:8DD7).
        let flea = current(game);
        let low =
            slot!(game, Fireflea[flea].base.y_pos).wrapping_sub(slot!(game, Fireflea[flea].radius));
        slot!(game, Fireflea[flea].min_y = low);
        let high =
            slot!(game, Fireflea[flea].radius).wrapping_add(slot!(game, Fireflea[flea].base.y_pos));
        slot!(game, Fireflea[flea].max_y = high);
    }
}
/// Main AI - fireflea ($A3:8DEE): circle, or fly up and down the line. Was
/// `advance`.
pub fn main_ai(game: &mut (impl Host + ?Sized)) {
    let flea = current(game);
    if slot!(game, Fireflea[flea].pattern) & CIRCLE != 0 {
        circle::<false>(flea, game);
        // The angle delta times 100h: the bytes from the subdelta's high byte.
        let step = u16::from_ne_bytes([
            slot!(byte game, Fireflea[flea].angle_subdelta[1]),
            slot!(byte game, Fireflea[flea].angle_subdelta[2]),
        ]);
        slot!(game, Fireflea[flea].angle, |old| old.wrapping_add(step));
    } else {
        let index = (slot!(game, Fireflea[flea].speed_table_index) >> 1) as isize;
        let step = (u32::from(game.enemy_speed(SpeedTable::Linear, index)) << 16)
            | u32::from(game.enemy_speed(SpeedTable::Linear, index + 1));
        slot!(game, Fireflea[flea].base(y_pos, y_subpos) += step);
        if (slot!(game, Fireflea[flea].base.y_pos) as i16)
            .wrapping_sub(slot!(game, Fireflea[flea].min_y) as i16)
            < 0
            || (slot!(game, Fireflea[flea].base.y_pos) as i16)
                .wrapping_sub(slot!(game, Fireflea[flea].max_y) as i16)
                >= 0
        {
            slot!(game, Fireflea[flea].speed_table_index, |old| old ^ NEGATE);
        }
    }
}
/// Enemy touch - fireflea ($A3:8E6B).
pub fn touch(game: &mut (impl Host + ?Sized)) {
    game.contact_reaction(Death::Finalize);
    game.die_normally();
    game.darken_room();
}
/// Fireflea shared shot reaction ($A3:8E8D): darken the room on death.
fn darken_if_dead(game: &mut (impl Host + ?Sized)) {
    let flea = current(game);
    if slot!(game, Fireflea[flea].base.health) == 0 {
        game.darken_room();
    }
}
/// Power bomb reaction - fireflea ($A3:8E83).
pub fn power_bomb(game: &mut (impl Host + ?Sized)) {
    game.power_bomb_reaction(Death::Finalize);
    darken_if_dead(game);
}
/// Enemy shot - fireflea ($A3:8E89).
pub fn shot(game: &mut (impl Host + ?Sized)) {
    game.shot_reaction(Shot::Finalize);
    darken_if_dead(game);
}
