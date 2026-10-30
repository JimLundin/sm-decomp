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
