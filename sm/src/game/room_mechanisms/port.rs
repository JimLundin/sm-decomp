//! Exclusive compatibility port. Foreign execution consumes and discards its owner.
#![forbid(unsafe_code)]
use super::state::{Column, RoomMechanisms};
use crate::game_state::Game;

pub(super) struct RoomPort<'g> {
    state: RoomMechanisms,
    game: Option<&'g mut Game>,
}
impl<'g> RoomPort<'g> {
    pub(super) fn acquire(game: &'g mut Game) -> Self {
        Self {
            state: RoomMechanisms::decode(&game.g_ram),
            game: Some(game),
        }
    }
    pub(super) fn rom_image(&self) -> crate::rom::RomImage {
        self.game
            .as_ref()
            .unwrap()
            .loaded_rom
            .as_ref()
            .expect("ROM must be loaded")
            .clone()
    }
    fn publish(&mut self) {
        if let Some(game) = self.game.as_deref_mut() {
            self.state.encode(&mut game.g_ram);
        }
    }
    pub(super) fn foreign<R>(mut self, call: impl FnOnce(&mut Game) -> R) -> (Self, R) {
        self.publish();
        let game = self.game.take().expect("released mechanism port");
        drop(self);
        // A callback panic finds no owner able to republish pre-callback state.
        let returned = call(game);
        (Self::acquire(game), returned)
    }
    pub(super) fn byte(&self, address: usize) -> u8 {
        self.state
            .byte(address)
            .unwrap_or_else(|| self.game.as_ref().unwrap().g_ram[address])
    }
    pub(super) fn put_byte(&mut self, address: usize, value: u8) {
        if !self.state.put_byte(address, value) {
            self.game.as_mut().unwrap().g_ram[address] = value;
        }
    }
    pub(super) fn word(&self, address: usize) -> u16 {
        u16::from_le_bytes([self.byte(address), self.byte(address + 1)])
    }
    pub(super) fn native_word(&self, address: usize) -> u16 {
        u16::from_ne_bytes([self.byte(address), self.byte(address + 1)])
    }
    /// Checked slice writes validate their entire destination before any byte.
    pub(super) fn put_slice(&mut self, address: usize, bytes: &[u8]) {
        let end = address
            .checked_add(bytes.len())
            .expect("mechanism slice extent");
        let _ = &self.game.as_ref().unwrap().g_ram[address..end];
        for (i, value) in bytes.iter().copied().enumerate() {
            self.put_byte(address + i, value);
        }
    }
    pub(super) fn put_word(&mut self, address: usize, value: u16) {
        self.put_slice(address, &value.to_le_bytes());
    }
    pub(super) fn put_native_word(&mut self, address: usize, value: u16) {
        self.put_slice(address, &value.to_ne_bytes());
    }
    /// Preserve call sites whose second byte can fail after the low byte commits.
    pub(super) fn put_word_prefix(&mut self, address: usize, value: u16) {
        let [low, high] = value.to_le_bytes();
        self.put_byte(address, low);
        self.put_byte(address + 1, high);
    }
    pub(super) fn field(&self, column: Column, slot: usize) -> u16 {
        self.word(column.address() + slot * 2)
    }
    pub(super) fn put_field(&mut self, column: Column, slot: usize, value: u16) {
        self.put_word(column.address() + slot * 2, value);
    }
}
impl Drop for RoomPort<'_> {
    fn drop(&mut self) {
        self.publish();
    }
}

/// A synchronous rule call temporarily removes this port when entering a
/// foreign operation. The retained rule stack can keep captured scalar inputs,
/// but neither a live typed owner nor an exclusive Game borrow survives there.
pub(super) struct Execution<'g> {
    port: Option<RoomPort<'g>>,
}
impl<'g> Execution<'g> {
    pub(super) fn new(game: &'g mut Game) -> Self {
        Self {
            port: Some(RoomPort::acquire(game)),
        }
    }
    pub(super) fn port(&self) -> &RoomPort<'g> {
        self.port.as_ref().expect("mechanism foreign interval")
    }
    pub(super) fn port_mut(&mut self) -> &mut RoomPort<'g> {
        self.port.as_mut().expect("mechanism foreign interval")
    }
    pub(super) fn foreign<R>(&mut self, call: impl FnOnce(&mut Game) -> R) -> R {
        let port = self.port.take().expect("mechanism foreign interval");
        let (port, returned) = port.foreign(call);
        self.port = Some(port);
        returned
    }
}
