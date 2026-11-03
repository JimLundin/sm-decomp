//! The original game's indexed reads and stores that run past their tables.
//!
//! # Design
//!
//! Some original routines index a table with a value that can leave it: a
//! column search that runs into the routine's own code, an enemy index added
//! to Samus's position, a swoop index past the shared boss block, a block
//! index past the level data. The CPU then reads or stores whatever RAM lies
//! there. A typed index into the table's Session owner would panic or give a
//! different result, so these paths keep the original RAM address and resolve
//! it here, exactly as the address API each function wraps did (the Session's
//! projected bytes, which already own the neighbouring RAM).
//!
//! This is the one sanctioned home for those paths: every call into it is an
//! out-of-range path the original takes, tagged `LEGACY(original-overflow)` at
//! the call site and listed in docs/LEGACY.md. Code whose index is provably in
//! range uses its typed owner instead.
use crate::actors::host::ActorLease;
use crate::actors::session_slots::SlotAccess;
use crate::runtime_access::SessionRam;

// LEGACY(original-overflow): the slot word, as `SlotAccess` resolves it.
/// The word at RAM address `at`, through the host's slot access.
pub fn slot_word<H: SlotAccess + ?Sized>(h: &H, at: usize) -> u16 {
    h.slot_word(at)
}
// LEGACY(original-overflow): the slot word store, as `SlotAccess` resolves it.
/// Store `value` to the word at RAM address `at`, through the host's slot access.
pub fn set_slot_word<H: SlotAccess + ?Sized>(h: &mut H, at: usize, value: u16) {
    h.set_slot_word(at, value);
}
// LEGACY(original-overflow): the slot byte, as `SlotAccess` resolves it.
/// The byte at RAM address `at`, through the host's slot access.
pub fn slot_byte<H: SlotAccess + ?Sized>(h: &H, at: usize) -> u8 {
    h.slot_byte(at)
}
// LEGACY(original-overflow): the slot byte store, as `SlotAccess` resolves it.
/// Store `value` to the byte at RAM address `at`, through the host's slot access.
pub fn set_slot_byte<H: SlotAccess + ?Sized>(h: &mut H, at: usize, value: u8) {
    h.set_slot_byte(at, value);
}

// LEGACY(original-overflow): the projected byte, as `SessionRam` resolves it.
/// The byte at RAM address `at`, through the owned-Session view's projection.
pub fn ram_byte<H: SessionRam + ?Sized>(h: &H, at: usize) -> u8 {
    h.byte(at)
}
// LEGACY(original-overflow): the projected byte store, as `SessionRam` resolves it.
/// Store `value` to the byte at RAM address `at`, through the projection.
pub fn set_ram_byte<H: SessionRam + ?Sized>(h: &mut H, at: usize, value: u8) {
    h.set_byte(at, value);
}
// LEGACY(original-overflow): the projected little-endian word.
/// The little-endian word at RAM address `at`, through the projection.
pub fn ram_word<H: SessionRam + ?Sized>(h: &H, at: usize) -> u16 {
    h.word(at)
}
// LEGACY(original-overflow): the projected native-order word.
/// The native-order word at RAM address `at`, through the projection.
pub fn ram_native_word<H: SessionRam + ?Sized>(h: &H, at: usize) -> u16 {
    h.native_word(at)
}
// LEGACY(original-overflow): the projected slice store, extent checked first.
/// Store `bytes` from RAM address `at` in order, through the projection.
pub fn ram_write_slice<H: SessionRam + ?Sized>(h: &mut H, at: usize, bytes: &[u8]) {
    h.write_slice(at, bytes);
}
// LEGACY(original-overflow): the projected little-endian slice store.
/// Store `value` little-endian from RAM address `at` as one checked slice.
pub fn ram_write_slice_le<H: SessionRam + ?Sized>(h: &mut H, at: usize, value: u16) {
    h.write_slice_le(at, value);
}
// LEGACY(original-overflow): the projected little-endian word store, low byte first.
/// Store `value` little-endian at RAM address `at`, a byte at a time.
pub fn ram_write_word_le<H: SessionRam + ?Sized>(h: &mut H, at: usize, value: u16) {
    h.write_word_le(value, at);
}

// LEGACY(original-overflow): the actor lease's projected word.
/// The little-endian word at RAM address `at`, through the actor lease.
pub fn lease_word<H: ActorLease + ?Sized>(h: &H, at: usize) -> u16 {
    h.word(at)
}
// LEGACY(original-overflow): the actor lease's projected native-order word.
/// The native-order word at RAM address `at`, through the actor lease.
pub fn lease_native_word<H: ActorLease + ?Sized>(h: &H, at: usize) -> u16 {
    h.lease_native_word(at)
}
// LEGACY(original-overflow): the actor lease's projected native-order store.
/// Store `value` in native order at RAM address `at`, through the actor lease.
pub fn lease_write_native<H: ActorLease + ?Sized>(h: &mut H, at: usize, value: u16) {
    h.write_slice_native(at, value);
}
