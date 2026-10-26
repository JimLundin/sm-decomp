//! Bounded dependencies for the actual shared sprite core; no Game/RAM owner.
#![forbid(unsafe_code)]
use crate::game::{
    session_access::SessionAccess,
    sprite_effect_state::{LegacySpriteWordIndex, SpriteDraw},
    visual_host::Content,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpriteMember {
    Instruction,
    Timer,
    Appearance,
    X,
    XFraction,
    Y,
    YFraction,
    DisableFlags,
}
/// Compatibility indices are widened words, already shifted by the caller.
/// Projection must dispatch to the sole active, tail or other aggregate owner.
pub trait Aliases {
    fn sprite_word(&self, member: SpriteMember, word_index: LegacySpriteWordIndex) -> u16;
    fn set_sprite_word_le_slice(
        &mut self,
        member: SpriteMember,
        word_index: LegacySpriteWordIndex,
        value: u16,
    );
    fn sprite_word_native(&self, member: SpriteMember, word_index: LegacySpriteWordIndex) -> u16;
    fn set_sprite_word_native_slice(
        &mut self,
        member: SpriteMember,
        word_index: LegacySpriteWordIndex,
        value: u16,
    );
}
pub trait Effects {
    /// Synchronous current OAM backend; no Session borrow survives this boundary.
    fn submit_sprite(&mut self, request: SpriteDraw);
    fn invalid_sprite_instruction(&mut self);
}
/// Reuse visual Content's loaded/native allocation-capture and record contracts.
/// Content::record may diagnose and replace the live owner/image; Content::table
/// captures without that diagnostic. No separate sprite content cache is added.
/// Live native actor observations. Captured actor index survives sprite writes;
/// individual reads occur where the old controller reread its actor pointer.
pub trait ControllerInputs {
    fn attachment_x(&self, enemy: u16) -> u16;
    fn attachment_y(&self, enemy: u16) -> u16;
    fn attachment_frozen_timer(&self, enemy: u16) -> u16;
}
pub trait Host: SessionAccess + Content + Aliases + Effects + ControllerInputs {
    #[cfg(sm_verify_dll)]
    fn observe_draygon(&self, _kind: u8, _handle: u16, _value: u16) {}
}
