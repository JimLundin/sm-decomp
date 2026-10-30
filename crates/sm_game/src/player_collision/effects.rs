//! Declaration-only finite dependencies for player collision.
//! State is the existing Session owner, not a second body/terrain or field port.
#![forbid(unsafe_code)]

use crate::{actors::access::ActorAccess, room_entry::api::EntryPoll, visual_host::Record};

/// A compatible mechanism header, not an address into mutable RAM.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MechanismHeader(pub u16);

/// Captures retain the selected allocation and read live bytes within it. They
/// do not borrow the host, allowing a captured record to survive an effect.
/// A later source lookup must capture anew where the original did so.
pub trait Content {
    type Record: Record;

    /// Actual native pose table91:B629; offsets are linear pose*8+field, not
    /// wrapping bank offsets. Preserve native vs byte reads at each call site.
    fn pose_table(&mut self) -> Self::Record;
    /// Capture the loaded walking-transition allocation at91:EB74 BEFORE the
    /// native pose-table capture and direction-shots-fired index byte read.
    /// The early solid path indexes CURRENT pose; the later blocked path indexes
    /// PENDING pose. Read a native-endian word at twice that byte in this capture.
    fn walked_pose_table(&mut self) -> Self::Record;
    /// Resolve the original local/area-specific94 tables. The adapter preserves
    /// low-window diagnostic/capture ordering before returning a header.
    fn special_header(&mut self, behavior: u8, solid: bool) -> MechanismHeader;
    fn bomb_header(&mut self, behavior: u8) -> MechanismHeader;
}

/// Only exceptional reads from these three already-defined imported windows.
/// Normal terrain/list entries are direct reads from Session. No arbitrary RAM
/// address, mutable byte view or independent storage is exposed here.
pub trait ImportedCollisionReads {
    /// Byte offset relative to the level-cell window, with original widening.
    /// Includes the checked failure after cursorFFFF publication; never clamp.
    fn imported_level_word(&self, byte_offset: usize) -> u16;
    fn imported_behavior_byte(&self, index: usize) -> u8;
    /// Byte offset relative to the actor interactive-list window, LE semantics.
    /// The terminator and unbounded legacy tail remain observable.
    fn imported_interactive_word(&self, byte_offset: usize) -> u16;
}

pub trait Effects {
    /// Verifier and unit-test observer of each launch-motion record, called
    /// before the record is stored; production is inert.
    fn record_launch(
        &mut self,
        _direction: crate::launch_motion::MovementDirection,
        _displacement: i32,
    ) {
    }
    /// Execute existing mechanism creation/setup synchronously with the SAME
    /// local displacement. Preserve all live Session mutations and the full
    /// result byte; the operation masks bits only where the original did so.
    fn apply_mechanism(&mut self, header: MechanismHeader, displacement: &mut i32) -> u8;
    /// Existing room_entry::begin(CurrentCollision), including its live state
    /// publications. Continuations belong to room entry; retain no collision
    /// scan across a frame. Rejected admission is handled as solid terrain.
    fn enter_door(&mut self) -> EntryPoll;
    /// Reuse samus_palette::cancel_speed_boost, including palette/echo effects.
    fn cancel_speed_boost(&mut self);
    /// Existing current-area boss-bit query used only by the spike rule.
    fn boss_defeated(&mut self) -> bool;
    /// Two actual ordered physical reads at7F6402+block and+1. Export/relinquish
    /// Session before backend execution; import and refresh inputs afterward,
    /// including unwind handling. Do not replace this with a cached terrain word.
    fn vertical_slope_bts(&mut self, block: u16) -> u16;
    /// Preserve the diagnostic at invalid clearance dispatch, after all prior
    /// state publications. This is not permission to sanitize unknown flags.
    fn invalid_clearance_flags(&mut self, flags: u16);
}

/// ActorAccess includes the existing SessionAccess. Narrow imported reads avoid
/// exposing actors' private writable collection/staging interface to collision.
pub trait Host: ActorAccess + Content + ImportedCollisionReads + Effects {}
impl<T: ActorAccess + Content + ImportedCollisionReads + Effects> Host for T {}
