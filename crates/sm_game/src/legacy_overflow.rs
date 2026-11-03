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
use crate::runtime_access::SessionRam;
use crate::session_access::SessionAccess;

/// The extended RAM's span, less the Botwoon death words its own cohort keeps.
fn extended_index(at: usize) -> Option<(usize, usize)> {
    match at {
        0x801e | 0x8020 | 0x803c | 0x8818 | 0x883e => None,
        0x7800..0x9000 if at.is_multiple_of(2) => {
            Some(((at - 0x7800) / 0x800, (at - 0x7800) % 0x800 / 2))
        }
        _ => None,
    }
}

// LEGACY(original-overflow): the bytes an out-of-range slot index reaches.
/// The RAM byte at `at` as the overflow paths below reach it: on a Session
/// lease, the actors' extended RAM ($7E:7800..$8FFF, less the Botwoon death
/// words) from `Actors::extended` and every other byte through the lease's
/// projection; on a host that still holds the RAM image, that image's byte.
pub trait SlotBytes {
    fn slot_byte(&self, at: usize) -> u8;
    fn set_slot_byte(&mut self, at: usize, value: u8);
}

impl<T: SessionAccess + SessionRam + ?Sized> SlotBytes for T {
    fn slot_byte(&self, at: usize) -> u8 {
        if let Some((block, index)) = extended_index(at & !1) {
            return self.session().actors.extended.blocks[block][index].to_le_bytes()[at & 1];
        }
        self.byte(at)
    }
    fn set_slot_byte(&mut self, at: usize, value: u8) {
        if let Some((block, index)) = extended_index(at & !1) {
            let word = &mut self.session_mut().actors.extended.blocks[block][index];
            let mut bytes = word.to_le_bytes();
            bytes[at & 1] = value;
            *word = u16::from_le_bytes(bytes);
            return;
        }
        self.set_byte(at, value);
    }
}

// LEGACY(original-overflow): the slot word, low byte then high.
/// The little-endian word at RAM address `at`, through the host's slot bytes.
pub fn slot_word<H: SlotBytes + ?Sized>(h: &H, at: usize) -> u16 {
    u16::from_le_bytes([h.slot_byte(at), h.slot_byte(at + 1)])
}
// LEGACY(original-overflow): the slot word store, low byte then high.
/// Store `value` little-endian at RAM address `at`, through the host's slot bytes.
pub fn set_slot_word<H: SlotBytes + ?Sized>(h: &mut H, at: usize, value: u16) {
    let [low, high] = value.to_le_bytes();
    h.set_slot_byte(at, low);
    h.set_slot_byte(at + 1, high);
}
// LEGACY(original-overflow): the slot byte.
/// The byte at RAM address `at`, through the host's slot bytes.
pub fn slot_byte<H: SlotBytes + ?Sized>(h: &H, at: usize) -> u8 {
    h.slot_byte(at)
}
// LEGACY(original-overflow): the slot byte store.
/// Store `value` to the byte at RAM address `at`, through the host's slot bytes.
pub fn set_slot_byte<H: SlotBytes + ?Sized>(h: &mut H, at: usize, value: u8) {
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
