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
use crate::ram::{CompatibilityRam, RawRam as _, WorkRamAccess};

/// Byte `at` as the host's projection holds it.
pub fn byte<H: CompatibilityRam + ?Sized>(h: &H, at: usize) -> u8 {
    #[cfg(sm_verify_dll)]
    crate::session_cohort::check_not_held(at);
    h.compatibility_byte(at)
}

/// Store byte `at` through the host's projection.
pub fn set_byte<H: CompatibilityRam + ?Sized>(h: &mut H, at: usize, value: u8) {
    #[cfg(sm_verify_dll)]
    crate::session_cohort::check_not_held(at);
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
impl crate::display_scroll::Access for SessionProjection {
    fn scroll_word(&self, word: crate::display_scroll::Word) -> u16 {
        self.0
            .presentation
            .owner()
            .expect("provided Display")
            .state
            .controls
            .scroll_words[usize::from(word)]
    }
    fn set_scroll_word(&mut self, word: crate::display_scroll::Word, value: u16) {
        self.0
            .presentation
            .owner_mut()
            .expect("provided Display")
            .state
            .controls
            .scroll_words[usize::from(word)] = value;
    }
    fn physical_scroll_byte(&self, _: crate::display_scroll::Word, _: bool) -> u8 {
        panic!("physical image was not supplied")
    }
    fn write_physical_scroll_byte(&mut self, _: crate::display_scroll::Word, _: bool, _: u8) {
        panic!("physical image was not supplied")
    }
}
#[cfg(test)]

impl crate::sequence_state::Access for SessionProjection {
    fn set_cinematic_function(&mut self, value: u16) {
        for (part, byte) in value.to_le_bytes().into_iter().enumerate() {
            self.compatibility_write_byte(0x1f51 + part, byte);
        }
    }
    fn set_sequence_word(&mut self, word: crate::session_words::Word, value: u16) {
        for (part, byte) in value.to_le_bytes().into_iter().enumerate() {
            self.compatibility_write_byte((word as usize) + part, byte);
        }
    }
    fn sequence_flow(&self) -> crate::flow::Flow {
        self.0.flow
    }
    fn sequence_words(&self) -> crate::session_words::SessionWords {
        crate::session_words::SessionWords::from_session(&self.0)
    }
    fn commit_flow(&mut self, before: &crate::flow::Flow, after: crate::flow::Flow) {
        after.apply_changed(before, &mut self.0.flow);
    }
    fn commit_sequence_words(
        &mut self,
        before: &crate::session_words::SessionWords,
        after: crate::session_words::SessionWords,
    ) {
        after.apply_changed(before, &mut self.0);
    }
}

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
