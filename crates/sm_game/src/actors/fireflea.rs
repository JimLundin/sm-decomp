//! Fireflea, the glowing bug of Norfair's dark rooms, which flies a circle or
//! a vertical line; killing one darkens the room (bank $A3:8C0F-$A3:8EA5).
use crate::actors::boss_lease::BossState;
use crate::actors::host::{Death, Shot};
use crate::actors::slot_view::{SlotState, slot_view};
use sm_data::cartridge::lorom_offset;
use sm_data::enemy_speeds::SpeedTable;

/// What the fireflea needs beyond the shared actor operations; its slot is
/// the Session's.
pub trait Host: BossState + SlotState {
    /// The enemy death routine, with the normal death animation.
    fn die_normally(&mut self);
    /// Darken the room by one fireflea's light.
    fn darken_room(&mut self);
}

slot_view! {
    /// The fireflea's view of its slot.
    Fireflea, FirefleaEdit {
        record: [
            angle_subdelta = var_a, angle_delta = var_b, radius = var_c, angle = var_d,
            x_centre = var_e, y_centre = var_f, pattern = parameter_1,
            speed_and_radius = parameter_2,
        ],
        extended: [min_y = var_00, max_y = var_01, speed_table_index = var_02],
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

fn current(game: &(impl Host + ?Sized)) -> u16 {
    game.current_enemy()
}
fn angle<const INITIAL: bool>(flea: u16, game: &(impl Host + ?Sized)) -> u16 {
    let angle = Fireflea::of(game, flea).angle();
    if INITIAL { angle } else { angle >> 8 }
}
/// Set initial circling fireflea position ($A3:8DAE), or circle on.
fn circle<const INITIAL: bool>(flea: u16, game: &mut (impl Host + ?Sized)) {
    // The centre is read before the angle and radius, as the original did.
    let centre = Fireflea::of(game, flea).x_centre();
    let a = angle::<INITIAL>(flea, game);
    let radius = Fireflea::of(game, flea).radius();
    let x = centre.wrapping_add(game.scaled_cosine(a, radius));
    FirefleaEdit::of(game, flea).actor().x.whole = x;
    let centre = Fireflea::of(game, flea).y_centre();
    let a = angle::<INITIAL>(flea, game);
    let radius = Fireflea::of(game, flea).radius();
    let y = centre.wrapping_add(game.scaled_sine(a, radius));
    FirefleaEdit::of(game, flea).actor().y.whole = y;
}
/// Set fireflea speed ($A3:8D75).
fn set_speed(flea: u16, game: &mut (impl Host + ?Sized)) {
    let fireflea = Fireflea::of(game, flea);
    let mut index = 8 * (fireflea.speed_and_radius() & 0xff);
    if fireflea.pattern() & POSITIVE == 0 {
        index += NEGATE;
    }
    *FirefleaEdit::of(game, flea).speed_table_index() = index;
    let index = (index >> 1) as isize;
    let high = game.enemy_speed(SpeedTable::Linear, index);
    *FirefleaEdit::of(game, flea).angle_delta() = high;
    let low = game.enemy_speed(SpeedTable::Linear, index + 1);
    *FirefleaEdit::of(game, flea).angle_subdelta() = low;
}
/// Set fireflea radius ($A3:8D9C).
fn set_radius(flea: u16, game: &mut (impl Host + ?Sized)) {
    let image = game.native_image();
    let index = usize::from(Fireflea::of(game, flea).speed_and_radius() >> 8);
    let radius = image.read(lorom_offset(RADII) as usize + 2 * index);
    *FirefleaEdit::of(game, flea).radius() = u16::from(radius);
}
/// Initialisation AI - fireflea ($A3:8D2D).
pub fn initialize(game: &mut (impl Host + ?Sized)) {
    let flea = current(game);
    let circling = {
        let mut fireflea = FirefleaEdit::of(game, flea);
        fireflea.actor().instruction = INSTRUCTION_LIST;
        *fireflea.pattern() & CIRCLE != 0
    };
    if circling {
        // Set fireflea centre ($A3:8D5D) and initial angle ($A3:8D6A).
        let flea = current(game);
        let mut fireflea = FirefleaEdit::of(game, flea);
        let (x, y) = (fireflea.actor().x.whole, fireflea.actor().y.whole);
        *fireflea.x_centre() = x;
        *fireflea.y_centre() = y;
        let angle = *fireflea.pattern() >> 8;
        *fireflea.angle() = angle << 8;
    }
    set_speed(current(game), game);
    set_radius(current(game), game);
    if circling {
        circle::<true>(current(game), game);
    } else {
        // Set fireflea Y extrema ($A3:8DD7).
        let flea = current(game);
        let mut fireflea = FirefleaEdit::of(game, flea);
        let (y, radius) = (fireflea.actor().y.whole, *fireflea.radius());
        *fireflea.min_y() = y.wrapping_sub(radius);
        *fireflea.max_y() = radius.wrapping_add(y);
    }
}
/// Main AI - fireflea ($A3:8DEE): circle, or fly up and down the line. Was
/// `advance`.
pub fn main_ai(game: &mut (impl Host + ?Sized)) {
    let flea = current(game);
    if Fireflea::of(game, flea).pattern() & CIRCLE != 0 {
        circle::<false>(flea, game);
        // The angle delta times 100h: the bytes from the subdelta's high byte.
        let mut fireflea = FirefleaEdit::of(game, flea);
        let step = (*fireflea.angle_subdelta() >> 8) | (*fireflea.angle_delta() << 8);
        let angle = fireflea.angle();
        *angle = angle.wrapping_add(step);
    } else {
        let index = (Fireflea::of(game, flea).speed_table_index() >> 1) as isize;
        let step = (u32::from(game.enemy_speed(SpeedTable::Linear, index)) << 16)
            | u32::from(game.enemy_speed(SpeedTable::Linear, index + 1));
        let mut fireflea = FirefleaEdit::of(game, flea);
        fireflea.actor().y.advance(step.cast_signed());
        let y = fireflea.actor().y.whole as i16;
        if y.wrapping_sub(*fireflea.min_y() as i16) < 0
            || y.wrapping_sub(*fireflea.max_y() as i16) >= 0
        {
            *fireflea.speed_table_index() ^= NEGATE;
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
    if Fireflea::of(game, flea).actor().health == 0 {
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
