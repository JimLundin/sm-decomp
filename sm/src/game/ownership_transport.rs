//! Shared implementation of an exclusive, temporary encoded-state transport.
//! Registered privately under each owner; no game-wide byte or state API.
#![forbid(unsafe_code)]
use crate::game_state::Game;
pub(super) trait Codec: Sized {
    fn import(game: &Game) -> Self {
        Self::decode(&game.g_ram)
    }
    fn export(&self, game: &mut Game) {
        self.encode(&mut game.g_ram);
    }
    fn decode(ram: &[u8]) -> Self;
    fn encode(&self, ram: &mut [u8]);
    fn byte(&self, address: usize) -> Option<u8>;
    fn put_byte(&mut self, address: usize, value: u8) -> bool;
}
pub(super) struct Port<'g, S: Codec> {
    pub(super) state: S,
    game: Option<&'g mut Game>,
}
impl<'g, S: Codec> Port<'g, S> {
    pub(super) fn acquire(game: &'g mut Game) -> Self {
        Self {
            state: S::import(game),
            game: Some(game),
        }
    }
    pub(super) fn capture_rom(&self) -> Option<crate::rom::RomImage> {
        self.game.as_ref().unwrap().loaded_rom.clone()
    }
    pub(super) fn rom(&self) -> crate::rom::RomImage {
        self.capture_rom().expect("ROM must be loaded")
    }
    pub(super) fn byte(&self, address: usize) -> u8 {
        self.state
            .byte(address)
            .unwrap_or_else(|| self.game.as_ref().unwrap().g_ram[address])
    }
    pub(super) fn set_byte(&mut self, address: usize, value: u8) {
        if !self.state.put_byte(address, value) {
            self.game.as_mut().unwrap().g_ram[address] = value;
        }
    }
    pub(super) fn read_le(&self, address: usize) -> u16 {
        u16::from_le_bytes([self.byte(address), self.byte(address + 1)])
    }
    pub(super) fn read_native(&self, address: usize) -> u16 {
        u16::from_ne_bytes([self.byte(address), self.byte(address + 1)])
    }
    /// Validate the complete destination before either an owned or foreign byte changes.
    pub(super) fn write_slice(&mut self, address: usize, bytes: &[u8]) {
        let end = address
            .checked_add(bytes.len())
            .expect("owned slice extent");
        let _ = &self.game.as_ref().unwrap().g_ram[address..end];
        for (i, byte) in bytes.iter().copied().enumerate() {
            self.set_byte(address + i, byte);
        }
    }
    pub(super) fn write_slice_le(&mut self, address: usize, value: u16) {
        self.write_slice(address, &value.to_le_bytes());
    }
    pub(super) fn write_slice_native(&mut self, address: usize, value: u16) {
        self.write_slice(address, &value.to_ne_bytes());
    }
    /// Argument order preserves checked_word!'s value-before-address evaluation.
    /// Independent lanes preserve a low-byte prefix when the high byte fails.
    pub(super) fn write_word_le(&mut self, value: u16, address: usize) {
        let [low, high] = value.to_le_bytes();
        self.set_byte(address, low);
        self.set_byte(address + 1, high);
    }
    fn publish(&mut self) {
        if let Some(game) = self.game.as_deref_mut() {
            self.state.export(game);
        }
    }
    pub(super) fn release(mut self) -> &'g mut Game {
        self.publish();
        let game = self.game.take().unwrap();
        drop(self); // Destroy the owner, not only its Game borrow, before any callback.
        game
    }
    pub(super) fn foreign<R>(self, call: impl FnOnce(&mut Game) -> R) -> (Self, R) {
        let game = self.release();
        let result = call(game);
        (Self::acquire(game), result)
    }
}
impl<S: Codec> Drop for Port<'_, S> {
    fn drop(&mut self) {
        self.publish();
    }
}
