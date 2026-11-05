//! Dense content IDs and the engine-wide quantities the portable records share (D15).
//!
//! An ID is a dense index into the loaded content, fast to compare and compact in the State.
//! It is only meaningful against the content it was made with, which is why the Checkpoint
//! carries the content fingerprint. Persisted and exchanged data uses stable keys instead.

/// A room in the loaded content.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RoomId(pub u16);

/// A doorway: one side of a door, the place a room is entered by.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DoorwayId(pub u16);

/// An enemy species.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpeciesId(pub u16);

/// An instruction list in one family's program table. The family is the label's type
/// parameter in `game`, so one family's ID can't name another's program.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProgramId(pub u16);

/// A scanline of the picture, 0 at the top.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Line(pub u16);

/// CPU time, in master clock cycles (D9).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Cycles(pub u32);
