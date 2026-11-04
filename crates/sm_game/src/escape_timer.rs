//! Escape countdown transitions and ordered sprite publication.
use crate::sprite_drawing::Clip;
use crate::sprite_output;

/// What the escape countdown runs on: its owner, the sequence and frame
/// display words it reads, and the spritemaps it draws.
pub trait Host:
    sprite_output::MapHost
    + crate::records::Records
    + crate::sequence_state::Access
    + crate::display::preparation::Access
    + crate::escape_countdown::Access
{
}
impl<T> Host for T where
    T: sprite_output::MapHost
        + crate::records::Records
        + crate::sequence_state::Access
        + crate::display::preparation::Access
        + crate::escape_countdown::Access
        + ?Sized
{
}

pub fn advance<H: Host + ?Sized>(game: &mut H) -> u8 {
    let states: [fn(&mut H) -> u8; 7] = [
        |_| 0,
        |game| prepare(1, game),
        |game| prepare(2, game),
        |game| prepare(3, game),
        |game| prepare(4, game),
        |game| prepare(5, game),
        tick,
    ];
    states[usize::from(crate::sequence_word!(game.TIMER_STATUS).to_le_bytes()[0])](game)
}
fn prepare<H: Host + ?Sized>(state: u8, game: &mut H) -> u8 {
    let count_down = game.countdown_apply(|timer, status| timer.prepare(status, state));
    if count_down { tick(game) } else { 0 }
}
fn tick<H: Host + ?Sized>(game: &mut H) -> u8 {
    // Preparation is already committed; no owner borrow crosses this live query.
    let frame = crate::frame_display_word!(game.NmiFrameWordPair);
    // Reacquire the selected current image if the query caused external adoption.
    game.countdown_apply(|timer, _| timer.tick(frame))
}
pub fn clear<H: Host + ?Sized>(game: &mut H) {
    game.countdown_apply(|timer, status| timer.clear(status));
}

pub fn draw<H: Host + ?Sized>(game: &mut H) {
    draw_sprite(0, 0xa060, game);
    for (at, offset) in [(2, 0xffe4u16), (1, 0xfffc), (0, 0x14)] {
        // The overlapping native word is read after the preceding sprite output.
        let value = game.countdown_native_word(at);
        for (shift, x) in [(4, offset), (0, offset.wrapping_add(8))] {
            let map =
                crate::menu_lists::timer_digit_spritemap(game, ((value >> shift) & 0xf) as isize);
            draw_sprite(x, map, game);
        }
    }
}

fn draw_sprite<H: Host + ?Sized>(offset: u16, map: u16, game: &mut H) {
    sprite_output::paletted(
        0x80,
        map,
        (
            offset.wrapping_add(u16::from(game.countdown_byte(4))),
            u16::from(game.countdown_byte(6)),
        ),
        0xa00,
        Clip::Bounded,
        game,
    );
}
