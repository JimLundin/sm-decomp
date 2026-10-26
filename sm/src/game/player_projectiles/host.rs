//! Player-projectile effects and finite aliases over the shared Session.
#![forbid(unsafe_code)]
use super::access::{ShotLane, Store};
use crate::game::session_access::SessionAccess;
use crate::rom::{RomImage, RomRecord};

#[derive(Clone, Copy)]
pub enum PlayerStep {
    Radius,
    Input,
    SuitIndex,
    Acceleration,
    InsideBlock,
    Grapple,
    Xray,
    RestoreControl,
}
#[derive(Clone, Copy)]
pub enum PaletteEffect {
    Suit,
    SuitTarget,
    Spawn(u16),
}
#[derive(Clone, Copy)]
pub enum Drawing {
    Projectile { slot: u16, x: u16, y: u16 },
    Beam { sprite: u16, x: u16, y: u16 },
    CrashEchoes,
    GrappleSegment { x: u16, y: u16, tile: u16 },
    GrappleEnd,
}
pub trait Host: SessionAccess {
    fn loaded_image(&self) -> Option<RomImage>;
    fn native_image(&self) -> RomImage;
    // Loaded low-window diagnostic runs before acquiring the fresh allocation.
    fn record(&mut self, bank: u8, cursor: u16) -> RomRecord;
    // Native low-window diagnostic likewise precedes acquisition; the same
    // record value can retain either independently selected allocation.
    fn native_record(&mut self, bank: u8, cursor: u16) -> RomRecord;
    fn sound(&mut self, bank: u8, limit: u8, sound: u16);
    fn player_step(&mut self, step: PlayerStep);
    fn palette(&mut self, effect: PaletteEffect) -> u8;
    fn enable_hdma(&mut self);
    fn spawn_power_bomb(&mut self);
    fn spawn_mechanism(&mut self, header: u16);
    fn draw(&mut self, drawing: Drawing);
    // The real ordered OAM word stores stay in this output edge; draw inputs
    // must be reread after each overlapping native store. Core chooses pair,
    // half, and frozen policy, and owns the complete trail interpreter.
    fn draw_trail(&mut self, pair: usize, right: bool, frozen: bool);
    fn invalid_damage(&mut self);
    fn unknown(&mut self) -> bool;
    fn room_level(&self, cell: u16) -> u16;
    fn room_bts(&self, cell: u16) -> u8;
    // Exceptional lanes only: normal access expands to ordinary struct fields.
    fn alias_shot_read(&self, lane: ShotLane, raw: u16) -> u16;
    fn alias_shot_write(&mut self, lane: ShotLane, raw: u16, value: u16, store: Store);
    fn variable_byte(&self, raw: u16, high: bool) -> u8;
    // Echo overlap and unusual outside selectors require narrowly typed lanes,
    // never an address or arbitrary byte method exposed to gameplay.
    fn echo_x(&self, index: usize) -> u16;
    fn set_echo_x(&mut self, index: usize, value: u16);
    fn set_echo_y(&mut self, index: usize, value: u16);
    fn set_echo_index_native(&mut self, raw: u16, value: u16);
}
