//! Samus's position, as the player domain owns it.
//!
//! Code outside the player modules (rooms, scrolling, actors, cinematics, the
//! grapple) reads and moves Samus through this trait instead of naming the
//! compatibility RAM words. `sm` implements it over the projected words, with
//! the same reads and stores in the same order. A host that runs inside the
//! owned Session can implement it over `Session::player` instead.
#![forbid(unsafe_code)]

pub trait PlayerPosition {
    fn player_x(&self) -> u16;
    fn player_y(&self) -> u16;
    fn player_x_fraction(&self) -> u16;
    fn player_y_fraction(&self) -> u16;
    fn previous_player_x(&self) -> u16;
    fn previous_player_y(&self) -> u16;
    fn previous_player_x_fraction(&self) -> u16;
    fn previous_player_y_fraction(&self) -> u16;
    fn set_player_x(&mut self, x: u16);
    fn set_player_y(&mut self, y: u16);
    fn set_player_x_fraction(&mut self, fraction: u16);
    fn set_player_y_fraction(&mut self, fraction: u16);
    fn set_previous_player_x(&mut self, x: u16);
    fn set_previous_player_y(&mut self, y: u16);
    fn set_previous_player_x_fraction(&mut self, fraction: u16);
    fn set_previous_player_y_fraction(&mut self, fraction: u16);
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
