//! Proposed visual integration seams; no fake implementation or gameplay stubs.
#![forbid(unsafe_code)]
use crate::{
    session_access::SessionAccess,
    visual_state::{PaletteBuffer, Upload},
};
/// Every displacement below is bytes. Reads remain live within a captured allocation.
pub trait Record {
    fn byte(&self, displacement: usize) -> u8;
    fn native_word(&self, displacement: usize) -> u16;
    /// Whole source span is checked before any destination write (legacy memcpy path).
    fn bytes<const N: usize>(&self, displacement: usize) -> [u8; N];
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Loaded,
    Native,
}
pub trait Content {
    type Record: Record;
    /// Logical pointer: low-window diagnostic before fresh source capture.
    fn record(&mut self, source: Source, bank: u8, address: u16) -> Self::Record;
    /// Legacy native pointers retain their diagnostic before fresh image capture.
    fn legacy_native_record(&mut self, bank: u8, address: u16) -> Self::Record {
        self.record(Source::Native, bank, address)
    }
    /// Fixed table: no low-window diagnostic; index arithmetic happens AFTER capture.
    /// The returned record's byte offsets are linear, never bank-wrapped.
    fn table(&self, source: Source, address: u32) -> Self::Record;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaletteMember {
    Definition,
    ColorOffset,
    PreAction,
    Program,
    Delay,
    Counter,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationMember {
    Definition,
    Counter,
    Program,
    Delay,
    Source,
}
pub trait Aliases {
    fn palette_word(&self, member: PaletteMember, handle: u16) -> u16;
    fn set_palette_word_slice(&mut self, member: PaletteMember, handle: u16, value: u16);
    fn set_palette_counter_byte(&mut self, handle: u16, value: u8);
    fn palette_flag_native(&self, displacement: u16) -> u16;
    fn animation_word(&self, member: AnimationMember, handle: u16) -> u16;
    fn set_animation_word_prefix(&mut self, member: AnimationMember, handle: u16, value: u16);
    fn set_animation_counter_byte(&mut self, handle: u16, value: u8);
    /// Palette effects: rounded u16 displacement and whole LE slice store.
    fn set_palette_color_slice(&mut self, displacement: u16, value: u16);
    /// Animation: widened word index, ordered low/high bytes, no hidden u16 wrap.
    fn set_color_prefix(&mut self, buffer: PaletteBuffer, index: usize, value: u16);
}
pub trait Effects {
    /// Native drawing uses the shared Samus producer; G supplies its owned IDs/content.
    fn draw_samus_sprite(&mut self, index: u16, x: u16, y: u16);
    /// Live frame and ordered OAM writes; kind was captured before timer callbacks.
    fn draw_atmospheric_tile(&mut self, slot: usize, kind: u16);
    fn sound(&mut self, channel: u8, limit: u8, sound: u16);
    fn music_delayed8(&mut self, track: u16);
    /// Existing SamusCode_1C sound selection; synchronous external interval.
    fn resume_movement_sound(&mut self);
    /// Synchronous backend request; caller acknowledges captured slot only on success.
    fn upload(&mut self, request: Upload);
    fn invalid_palette_instruction(&mut self);
    fn invalid_animation_instruction(&mut self) -> u16;
    /// Existing observer, after in-range write and before probe; production is inert.
    fn color_written(&mut self, index: u16, value: u16);
    /// Verifier observer of a Samus palette phase; production has none.
    #[cfg(sm_verify_dll)]
    fn samus_palette_event(&mut self, _kind: &'static str, _argument: u16) {}
    /// Verifier injection point before the Samus palette dispatch.
    #[cfg(sm_verify_dll)]
    fn samus_palette_inject(&mut self, _name: &'static str) {}
    /// Verifier transition trace: whether palette publications are recorded.
    #[cfg(sm_verify_dll)]
    fn palette_publications_observed(&self) -> bool {
        false
    }
    #[cfg(sm_verify_dll)]
    fn palette_publication(&mut self, _kind: u16, _at: usize, _before: u16, _after: u16) {}
    /// Verifier movement case: speed-boost cancellation inputs.
    #[cfg(sm_verify_dll)]
    fn observe_speed_boost_cancel(&mut self, _values: [u16; 3]) {}
}
/// Actual-core composition, not RAM transport. Implementations must use this same owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatueProjectile {
    EyeGlow,
    Soul,
}
pub trait Composition {
    fn spawn_palette(&mut self, definition: u16);
    fn spawn_statue_projectile(&mut self, kind: StatueProjectile, parameter: u16);
}
pub trait Host:
    SessionAccess + Content + Aliases + Effects + Composition + crate::gameplay_flag_host::Host
{
}
impl Record for sm_data::cartridge::RomRecord {
    fn byte(&self, displacement: usize) -> u8 {
        sm_data::cartridge::RomRecord::byte(self, displacement)
    }
    fn native_word(&self, displacement: usize) -> u16 {
        sm_data::cartridge::RomRecord::word(self, displacement)
    }
    fn bytes<const N: usize>(&self, displacement: usize) -> [u8; N] {
        let start = self
            .base
            .checked_add(displacement)
            .expect("ROM copy offset overflow");
        let end = start.checked_add(N).expect("ROM copy extent overflow");
        assert!(end <= self.image.len(), "ROM copy source extent");
        core::array::from_fn(|index| self.image.read(start + index))
    }
}

/// The palette-effect and animated-tile aliases over a session lease's RAM
/// projection, at the original RAM addresses.
pub mod session_aliases {
    use super::{AnimationMember, PaletteMember};
    use crate::runtime_access::SessionRam;
    use crate::visual_state::PaletteBuffer;

    fn palette_address(member: PaletteMember, handle: u16) -> usize {
        let base = match member {
            PaletteMember::Definition => 0x1e7d,
            PaletteMember::ColorOffset => 0x1e8d,
            PaletteMember::PreAction => 0x1ead,
            PaletteMember::Program => 0x1ebd,
            PaletteMember::Delay => 0x1ecd,
            PaletteMember::Counter => 0x1edd,
        };
        base + usize::from(handle & !1)
    }
    fn animation_address(member: AnimationMember, handle: u16) -> usize {
        let base = match member {
            AnimationMember::Definition => 0x1ef5,
            AnimationMember::Counter => 0x1f01,
            AnimationMember::Program => 0x1f0d,
            AnimationMember::Delay => 0x1f19,
            AnimationMember::Source => 0x1f25,
        };
        base + usize::from(handle & !1)
    }
    pub fn palette_word(h: &(impl SessionRam + ?Sized), member: PaletteMember, handle: u16) -> u16 {
        h.word(palette_address(member, handle))
    }
    pub fn set_palette_word_slice(
        h: &mut (impl SessionRam + ?Sized),
        member: PaletteMember,
        handle: u16,
        value: u16,
    ) {
        h.write_slice_le(palette_address(member, handle), value);
    }
    pub fn set_palette_counter_byte(h: &mut (impl SessionRam + ?Sized), handle: u16, value: u8) {
        h.set_byte(0x1edd + usize::from(handle), value);
    }
    pub fn palette_flag_native(h: &(impl SessionRam + ?Sized), displacement: u16) -> u16 {
        h.native_word(0x1e79 + usize::from(displacement))
    }
    pub fn animation_word(
        h: &(impl SessionRam + ?Sized),
        member: AnimationMember,
        handle: u16,
    ) -> u16 {
        h.word(animation_address(member, handle))
    }
    pub fn set_animation_word_prefix(
        h: &mut (impl SessionRam + ?Sized),
        member: AnimationMember,
        handle: u16,
        value: u16,
    ) {
        h.write_word_le(value, animation_address(member, handle));
    }
    pub fn set_animation_counter_byte(h: &mut (impl SessionRam + ?Sized), handle: u16, value: u8) {
        h.set_byte(0x1f01 + usize::from(handle), value);
    }
    pub fn set_palette_color_slice(
        h: &mut (impl SessionRam + ?Sized),
        displacement: u16,
        value: u16,
    ) {
        h.write_slice_le(0xc000 + usize::from(displacement & !1), value);
    }
    pub fn set_color_prefix(
        h: &mut (impl SessionRam + ?Sized),
        buffer: PaletteBuffer,
        index: usize,
        value: u16,
    ) {
        let base: usize = match buffer {
            PaletteBuffer::Current => 0xc000,
            PaletteBuffer::Target => 0xc200,
        };
        let at = base
            .checked_add(index.checked_mul(2).expect("palette index overflow"))
            .expect("palette address overflow");
        h.write_word_le(value, at);
    }
}
