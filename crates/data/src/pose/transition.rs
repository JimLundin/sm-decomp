//! Samus's pose transitions from input: the bank $91 transition table, decoded per pose.
//!
//! Bank $91 log, `$91:9EE2` "Pose transition table": one bank $91 pointer a pose, to a list
//! of six-byte entries (the newly pressed controls, the held controls, the new pose) ended
//! by $FFFF. The input handler (`$91:8000`, through `$91:81A9` "determine new pose from
//! input") takes the first entry whose pressed controls were all newly pressed and whose
//! held controls are all held (<https://patrickjohnston.org/bank/91#f9EE2>).

use canonical::Canonical;

use super::{POSE_COUNT, PoseId};
use crate::rom::{AddressError, Lorom, RomAddress};

/// One of Samus's controls, as the pose transition table numbers them: the d-pad keeps its
/// pad bits and each action has the bit of its default button.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Control {
    /// Up on the d-pad.
    Up,
    /// Down on the d-pad.
    Down,
    /// Left on the d-pad.
    Left,
    /// Right on the d-pad.
    Right,
    /// Aim up (R by default).
    AimUp,
    /// Aim down (L by default).
    AimDown,
    /// Shoot (X by default).
    Shoot,
    /// Jump (A by default).
    Jump,
    /// Run (B by default).
    Run,
    /// Item cancel (Y by default).
    ItemCancel,
}

impl Control {
    /// Every control.
    pub const ALL: [Self; 10] = [
        Self::Up,
        Self::Down,
        Self::Left,
        Self::Right,
        Self::AimUp,
        Self::AimDown,
        Self::Shoot,
        Self::Jump,
        Self::Run,
        Self::ItemCancel,
    ];

    /// The control's bit in the table's words.
    #[must_use]
    pub const fn bit(self) -> u16 {
        match self {
            Self::Up => 0x0800,
            Self::Down => 0x0400,
            Self::Left => 0x0200,
            Self::Right => 0x0100,
            Self::AimUp => 0x0010,
            Self::AimDown => 0x0020,
            Self::Shoot => 0x0040,
            Self::Jump => 0x0080,
            Self::Run => 0x8000,
            Self::ItemCancel => 0x4000,
        }
    }
}

/// A set of [`Control`]s, kept as the table's word so it goes back out unchanged; bits no
/// control has (start, select and the low nybble) are kept too.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Controls(pub u16);

impl Controls {
    /// No control.
    pub const NONE: Self = Self(0);

    /// The set holding `controls`.
    #[must_use]
    pub fn of(controls: &[Control]) -> Self {
        Self(
            controls
                .iter()
                .fold(0, |bits, control| bits | control.bit()),
        )
    }

    /// Whether every control of `other` is in this set.
    #[must_use]
    pub const fn contains_all(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Whether `control` is in this set.
    #[must_use]
    pub const fn contains(self, control: Control) -> bool {
        (self.0 & control.bit()) != 0
    }
}

/// One entry: the controls it needs and the pose it gives.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Transition {
    /// Controls that must all be newly pressed this frame.
    pub pressed: Controls,
    /// Controls that must all be held.
    pub held: Controls,
    /// The pose to take.
    pub pose: PoseId,
}

impl Transition {
    /// Whether the controls newly `pressed` and `held` this frame satisfy the entry.
    #[must_use]
    pub const fn matches(&self, pressed: Controls, held: Controls) -> bool {
        pressed.contains_all(self.pressed) && held.contains_all(self.held)
    }
}

/// The table's first pointer (bank $91 log).
pub const POSE_TRANSITIONS: RomAddress = RomAddress::new(0x91, 0x9EE2);

/// The bank every list is in.
const LIST_BANK: u8 = 0x91;

/// The word that ends a list.
const END: u16 = 0xFFFF;

/// Bytes an entry.
const ENTRY_BYTES: usize = 6;

/// Every pose's transitions, indexed by [`PoseId`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PoseTransitions {
    lists: Vec<Vec<Transition>>,
}

impl PoseTransitions {
    /// The table in `rom`, a list for each of the [`POSE_COUNT`] poses.
    ///
    /// # Errors
    ///
    /// [`AddressError`] when a pointer or a list is outside the image.
    pub fn load(rom: Lorom<'_>) -> Result<Self, AddressError> {
        let pointers = rom.read(POSE_TRANSITIONS, POSE_COUNT.wrapping_mul(2))?;
        let (pointers, _) = pointers.as_chunks::<2>();
        let lists = pointers
            .iter()
            .map(|&pointer| {
                list(rom.from(RomAddress::new(LIST_BANK, u16::from_le_bytes(pointer)))?)
            })
            .collect::<Result<_, _>>()?;
        Ok(Self { lists })
    }

    /// The table from each pose's list.
    #[must_use]
    pub const fn new(lists: Vec<Vec<Transition>>) -> Self {
        Self { lists }
    }

    /// The transitions of `pose`, in the table's order; empty past the table and for the
    /// poses whose list is empty.
    #[must_use]
    pub fn of(&self, pose: PoseId) -> &[Transition] {
        self.lists
            .get(usize::from(pose.0))
            .map_or(&[], Vec::as_slice)
    }
}

/// One list, from its first entry to the end word; a list running past the image is an
/// error.
fn list(bytes: &[u8]) -> Result<Vec<Transition>, AddressError> {
    let (entries, rest) = bytes.as_chunks::<ENTRY_BYTES>();
    let mut list = Vec::new();
    for &[p0, p1, h0, h1, n0, n1] in entries {
        let pressed = u16::from_le_bytes([p0, p1]);
        if pressed == END {
            return Ok(list);
        }
        list.push(Transition {
            pressed: Controls(pressed),
            held: Controls(u16::from_le_bytes([h0, h1])),
            pose: PoseId(u16::from_le_bytes([n0, n1])),
        });
    }
    // An end word in a trailing partial entry still ends the list.
    match rest {
        [0xFF, 0xFF, ..] => Ok(list),
        _ => Err(AddressError::PastEnd(POSE_TRANSITIONS)),
    }
}

#[cfg(test)]
mod tests;
