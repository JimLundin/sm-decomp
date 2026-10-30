//! The one place sm_game's typed views touch the compatibility host's bytes.
//!
//! A view (`camera`, `flow`, `menu_state`, `inventory`, the liquid words, the
//! enemy projectile table) gives code outside a lease typed access to
//! Session-owned state. Outside a lease the host's projected bytes are that
//! state: the owned Session's projection while one is installed, the byte
//! image otherwise. Views read and store through these helpers, so the byte
//! layout stays with `session_encoding` and the byte access stays here.
//!
//! [`byte`]/[`set_byte`] and the word helpers go through the projection
//! (`CompatibilityRam`). The `image_*` helpers address the work RAM image
//! directly, for a view that must not admit the owned Session (the enemy
//! projectile table when no lease can open).
#![forbid(unsafe_code)]
use crate::ram::{CompatibilityRam, RawRam as _, WorkRamAccess};

/// Byte `at` as the host's projection holds it.
pub fn byte<H: CompatibilityRam + ?Sized>(h: &H, at: usize) -> u8 {
    h.compatibility_byte(at)
}

/// Store byte `at` through the host's projection.
pub fn set_byte<H: CompatibilityRam + ?Sized>(h: &mut H, at: usize, value: u8) {
    h.compatibility_write_byte(at, value);
}

/// The little-endian word at `at`, low byte then high.
pub fn word<H: CompatibilityRam + ?Sized>(h: &H, at: usize) -> u16 {
    u16::from_le_bytes([byte(h, at), byte(h, at + 1)])
}

/// Store `value` as the little-endian word at `at`, low byte then high, as a
/// named word store does.
pub fn set_word<H: CompatibilityRam + ?Sized>(h: &mut H, at: usize, value: u16) {
    for (lane, value) in value.to_le_bytes().into_iter().enumerate() {
        set_byte(h, at + lane, value);
    }
}

/// The projection as a byte reader, for `session_encoding`'s part decoders.
pub fn reader<H: CompatibilityRam + ?Sized>(h: &H) -> impl Fn(usize) -> u8 + '_ {
    move |at| byte(h, at)
}

/// The little-endian word at `at` of the work RAM image.
pub fn image_word<H: WorkRamAccess + ?Sized>(h: &H, at: usize) -> u16 {
    let ram = h.ram();
    u16::from_le_bytes([ram.read_byte(at), ram.read_byte(at + 1)])
}

/// Store `value` as the little-endian word at `at` of the work RAM image,
/// low byte then high.
pub fn set_image_word<H: WorkRamAccess + ?Sized>(h: &H, at: usize, value: u16) {
    let [low, high] = value.to_le_bytes();
    let ram = h.ram();
    ram.write_byte(at, low);
    ram.write_byte(at + 1, high);
}

/// Copy `bytes` into the work RAM image at `at`.
pub fn copy_into_image<H: WorkRamAccess + ?Sized>(h: &H, at: usize, bytes: &[u8]) {
    h.ram().copy_in(at, bytes);
}

/// A host over a Session's projection, as the compat game's is while one is
/// installed, for the views' tests.
#[cfg(test)]
pub(crate) struct SessionProjection(pub crate::session::Session);

#[cfg(test)]
impl CompatibilityRam for SessionProjection {
    fn compatibility_byte(&self, at: usize) -> u8 {
        crate::session_encoding::projected_byte(&self.0, at).expect("view byte is projected")
    }
    fn compatibility_write_byte(&mut self, at: usize, value: u8) {
        assert!(crate::session_encoding::write_projected_byte(
            &mut self.0,
            at,
            value
        ));
    }
}
