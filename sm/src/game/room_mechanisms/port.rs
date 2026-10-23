//! Exclusive compatibility port. Foreign execution consumes and discards its owner.
#![forbid(unsafe_code)]
use super::state::{Column, RoomMechanisms};
use crate::game_state::Game;

use super::transport::Port;
pub(super) type RoomPort<'g> = Port<'g, RoomMechanisms>;
impl RoomPort<'_> {
    pub(super) fn field(&self, column: Column, slot: usize) -> u16 {
        self.read_le(column.address() + slot * 2)
    }
    pub(super) fn put_field(&mut self, column: Column, slot: usize, value: u16) {
        self.write_slice_le(column.address() + slot * 2, value);
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
