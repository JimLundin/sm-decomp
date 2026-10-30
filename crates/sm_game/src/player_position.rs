//! Samus's position and damage timers, as the player domain owns them.
//!
//! Code outside the player modules (rooms, scrolling, actors, cinematics, the
//! grapple) reads and moves Samus through this trait instead of naming the
//! compatibility RAM words. A host implements one word read and one word store:
//! the compat game over the projected words, a lease over `Session::player`
//! through `PlayerWord::get`, with stores through its own byte path.

/// Samus's words in `Session::player`, each with the RAM word it projects to.
macro_rules! player_words {
    ($($word:ident => $at:literal, $($field:ident).+;)+) => {
        /// A Samus word that `PlayerPosition` reads or stores.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum PlayerWord { $($word,)+ }
        impl PlayerWord {
            /// Every word, in RAM order.
            pub const ALL: [Self; 10] = [$(Self::$word,)+];
            /// The RAM word this field projects to.
            pub const fn address(self) -> usize { match self { $(Self::$word => $at,)+ } }
            /// The owned `Session::player` field.
            pub fn get(self, player: &crate::session::Player) -> u16 {
                match self { $(Self::$word => player.$($field).+,)+ }
            }
        }
    };
}
player_words! {
    X => 0xaf6, x;
    XFraction => 0xaf8, x_fraction;
    Y => 0xafa, y;
    YFraction => 0xafc, y_fraction;
    PreviousX => 0xb10, previous_position.x.whole;
    PreviousXFraction => 0xb12, previous_position.x.fraction;
    PreviousY => 0xb14, previous_position.y.whole;
    PreviousYFraction => 0xb16, previous_position.y.fraction;
    Invincibility => 0x18a8, invincibility;
    Knockback => 0x18aa, knockback_timer;
}

/// Samus's position and damage timers ($18A8 invincibility, $18AA knockback),
/// read and stored a word at a time.
pub trait PlayerPosition {
    fn player_word(&self, word: PlayerWord) -> u16;
    fn set_player_word(&mut self, word: PlayerWord, value: u16);

    fn player_x(&self) -> u16 {
        self.player_word(PlayerWord::X)
    }
    fn player_y(&self) -> u16 {
        self.player_word(PlayerWord::Y)
    }
    fn player_x_fraction(&self) -> u16 {
        self.player_word(PlayerWord::XFraction)
    }
    fn player_y_fraction(&self) -> u16 {
        self.player_word(PlayerWord::YFraction)
    }
    fn previous_player_x(&self) -> u16 {
        self.player_word(PlayerWord::PreviousX)
    }
    fn previous_player_y(&self) -> u16 {
        self.player_word(PlayerWord::PreviousY)
    }
    fn invincibility_timer(&self) -> u16 {
        self.player_word(PlayerWord::Invincibility)
    }
    fn knockback_timer(&self) -> u16 {
        self.player_word(PlayerWord::Knockback)
    }
    fn set_player_x(&mut self, x: u16) {
        self.set_player_word(PlayerWord::X, x)
    }
    fn set_player_y(&mut self, y: u16) {
        self.set_player_word(PlayerWord::Y, y)
    }
    fn set_player_x_fraction(&mut self, fraction: u16) {
        self.set_player_word(PlayerWord::XFraction, fraction)
    }
    fn set_previous_player_x(&mut self, x: u16) {
        self.set_player_word(PlayerWord::PreviousX, x)
    }
    fn set_previous_player_y(&mut self, y: u16) {
        self.set_player_word(PlayerWord::PreviousY, y)
    }
    fn set_previous_player_x_fraction(&mut self, fraction: u16) {
        self.set_player_word(PlayerWord::PreviousXFraction, fraction)
    }
    fn set_previous_player_y_fraction(&mut self, fraction: u16) {
        self.set_player_word(PlayerWord::PreviousYFraction, fraction)
    }
    fn set_invincibility_timer(&mut self, frames: u16) {
        self.set_player_word(PlayerWord::Invincibility, frames)
    }
    fn set_knockback_timer(&mut self, frames: u16) {
        self.set_player_word(PlayerWord::Knockback, frames)
    }
}

/// Samus's facing and movement type, and the grapple angle bytes, as the
/// original raw work-RAM byte reads. `sm` implements them for `Game` over raw
/// RAM, exactly as the call sites read them; inside a lease they are the
/// Session-owned `player.movement_and_facing` and grapple fields.
pub trait PlayerFacts {
    /// The facing byte ($0A1E): the low byte of `movement_and_facing`.
    fn facing(&self) -> u8;
    /// The movement type ($0A1F): the high byte of `movement_and_facing`.
    fn movement_type(&self) -> u8;
    /// The grapple beam's end angle, high byte ($0CFB).
    fn grapple_angle(&self) -> u8;
    /// The grapple beam's mirrored end angle, high byte ($0CFD).
    fn grapple_mirrored_angle(&self) -> u8;
}

/// Keep Samus's previous position within 12 px of her position, Y then X, as
/// the game's previous-position correction does.
pub fn correct_previous_position<P: PlayerPosition + ?Sized>(game: &mut P) {
    let position = game.player_y();
    if let Some(previous) = corrected(position, game.previous_player_y()) {
        game.set_previous_player_y(previous);
    }
    let position = game.player_x();
    if let Some(previous) = corrected(position, game.previous_player_x()) {
        game.set_previous_player_x(previous);
    }
}

fn corrected(position: u16, previous: u16) -> Option<u16> {
    let delta = position.wrapping_sub(previous);
    if (delta as i16).unsigned_abs().wrapping_sub(12) & 0x8000 != 0 {
        return None;
    }
    let correction = if delta & 0x8000 == 0 {
        12
    } else {
        (-12i16) as u16
    };
    Some(position.wrapping_add(correction))
}
