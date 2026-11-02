//! Mother Brain's forced player movement and health/ammunition changes.
//! Positions, inventory and weapon controls retain their existing Session owners.
#![forbid(unsafe_code)]

use crate::effects::SoundEffects;
use crate::player_position::{PlayerPosition, PlayerWord};
use crate::session_access::SessionAccess;

fn negative(value: u16) -> bool {
    value & 0x8000 != 0
}

/// Add the original signed 8.8 displacement, touching the fractional high bytes
/// in current/previous order before reading and changing the whole coordinate.
fn move_fraction<P: PlayerPosition + ?Sized>(
    amount: u16,
    current: PlayerWord,
    previous: PlayerWord,
    game: &mut P,
) -> u16 {
    let fraction = amount.to_ne_bytes()[0];
    let whole = amount.to_be_bytes()[0];
    let current_fraction = game.player_word(current);
    let carry = u16::from(current_fraction.to_le_bytes()[1]) + u16::from(fraction);
    let high = carry.to_le_bytes()[0];
    game.set_player_word(
        current,
        u16::from_le_bytes([current_fraction.to_le_bytes()[0], high]),
    );
    let previous_fraction = game.player_word(previous);
    game.set_player_word(
        previous,
        u16::from_le_bytes([previous_fraction.to_le_bytes()[0], high]),
    );
    let signed_whole = i16::from(i8::from_ne_bytes([whole]));
    u16::from_ne_bytes(signed_whole.to_ne_bytes()).wrapping_add(carry >> 8)
}

pub fn move_samus_vertically_towards_ceiling_floor<P: PlayerPosition + ?Sized>(
    amount: u16,
    game: &mut P,
) -> u8 {
    let delta = move_fraction(
        amount,
        PlayerWord::YFraction,
        PlayerWord::PreviousYFraction,
        game,
    );
    let moved = game.player_y().wrapping_add(delta);
    let (position, clamped) = if negative(moved.wrapping_sub(48)) {
        (48, true)
    } else if negative(moved.wrapping_sub(192)) {
        (moved, false)
    } else {
        (192, true)
    };
    game.set_player_word(PlayerWord::Y, position);
    let stored = game.player_word(PlayerWord::Y);
    game.set_player_word(PlayerWord::PreviousY, stored);
    if clamped {
        game.set_player_word(PlayerWord::YFraction, 0);
        game.set_player_word(PlayerWord::PreviousYFraction, 0);
    }
    u8::from(clamped)
}

pub fn move_samus_horizontally_towards_wall<P: PlayerPosition + ?Sized>(
    amount: u16,
    game: &mut P,
) -> u8 {
    let delta = move_fraction(
        amount,
        PlayerWord::XFraction,
        PlayerWord::PreviousXFraction,
        game,
    );
    let moved = game.player_x().wrapping_add(delta);
    let clamped = !negative(moved.wrapping_sub(235));
    let position = if clamped { 235 } else { moved };
    game.set_player_word(PlayerWord::X, position);
    game.set_player_word(PlayerWord::PreviousX, position);
    if clamped {
        game.set_player_word(PlayerWord::XFraction, 0);
        game.set_player_word(PlayerWord::PreviousXFraction, 0);
    }
    u8::from(clamped)
}

fn spend_ammo<H: SessionAccess + ?Sized>(count: u16, hud_item: u16, game: &mut H) -> u16 {
    if negative(count.wrapping_sub(2)) {
        if game.session().projectiles.controls.selected_weapon == hud_item {
            game.session_mut().projectiles.controls.selected_weapon = 0;
        }
        game.session_mut().projectiles.controls.automatic_weapon = 0;
        0
    } else {
        count.wrapping_sub(1)
    }
}

pub fn drain_ammo_by_rainbow_beam<H: SessionAccess + ?Sized>(game: &mut H) {
    // Each count/counter query remains at its original place. No inventory copy
    // is held over a weapon-control change or another supply's query.
    let missiles = game.session().inventory.ammunition[0].current;
    if (game.session().actors.controls.phase_counter & 3) == 0 && missiles != 0 {
        let left = spend_ammo(missiles, 1, game);
        game.session_mut().inventory.ammunition[0].current = left;
    }
    let supers = game.session().inventory.ammunition[1].current;
    if (game.session().actors.controls.phase_counter & 3) == 0 && supers != 0 {
        let left = spend_ammo(supers, 2, game);
        game.session_mut().inventory.ammunition[1].current = left;
    }
    let power_bombs = game.session().inventory.ammunition[2].current;
    if power_bombs != 0 {
        let left = spend_ammo(power_bombs, 3, game);
        game.session_mut().inventory.ammunition[2].current = left;
    }
}

fn health_change_sound<H: SessionAccess + SoundEffects + ?Sized>(game: &mut H) {
    if !negative(game.session().inventory.health.current.wrapping_sub(81))
        && (game.session().actors.controls.phase_counter & 7) == 0
    {
        game.queue_sound(2, 0x2d, 3);
    }
}

fn lose_health<H: SessionAccess + ?Sized>(loss: u16, floor: u16, game: &mut H) {
    let health = game.session().inventory.health.current.wrapping_sub(loss);
    let floored = negative(health.wrapping_sub(floor + 1));
    game.session_mut().inventory.health.current = if floored { floor } else { health };
}

pub fn drain_health_by_baby_metroid<H: SessionAccess + ?Sized>(game: &mut H) {
    let varia = game.session().inventory.equipment.active & 1;
    lose_health(4 - (2 * varia), 1, game);
}

pub fn drain_health_by_rainbow_beam<H: SessionAccess + SoundEffects + ?Sized>(game: &mut H) {
    let varia = game.session().inventory.equipment.active & 1;
    lose_health(2 - varia, 0, game);
    health_change_sound(game);
}

pub fn heal_by_baby_metroid<H: SessionAccess + SoundEffects + ?Sized>(game: &mut H) -> bool {
    let health = game.session().inventory.health.current;
    let max = game.session().inventory.health.maximum;
    let healing = negative(health.wrapping_add(1).wrapping_sub(max));
    game.session_mut().inventory.health.current =
        if healing { health.wrapping_add(1) } else { max };
    health_change_sound(game);
    healing
}
