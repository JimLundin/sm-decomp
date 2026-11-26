//! The discovery's content, as the importer provides it: the pictures, the fragments' and
//! droplets' paths, and the sounds. Relations between its fields are the importer's to check.

use core::{iter, num};

use types::QueueAllowance;
use types::discovery::{Offset, RoomPoint};

/// One image from the sheet installed with the room.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawingRef(pub u16);

/// A palette among the colours the intro installs with the room.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PaletteRef(pub u16);

/// A placed sprite's picture: its drawing and its palette.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpriteRef {
    /// The drawing.
    pub drawing: DrawingRef,
    /// The palette.
    pub palette: PaletteRef,
}

/// A sound resource, for audio to play.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SoundRef(pub u16);

/// An offer of a sound to audio: the sound and the most sounds that may wait before it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SoundOffer {
    /// The sound.
    pub sound: SoundRef,
    /// The most waiting sounds audio may hold for it to join.
    pub allowance: QueueAllowance,
}

/// One picture and its duration in complete frames.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shown {
    /// The picture.
    pub sprite: SpriteRef,
    /// The finished frames it holds, at least one.
    pub hold: num::NonZeroU16,
}

/// A nonempty sequence of pictures, each held for its finished frames. An age counts finished
/// frames from the first picture, which is age zero.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Animation {
    /// The first picture.
    first: Shown,
    /// The pictures after it.
    rest: Vec<Shown>,
}

impl Animation {
    /// The animation of `first`, then `rest`.
    #[must_use]
    pub const fn new(first: Shown, rest: Vec<Shown>) -> Self {
        Self { first, rest }
    }

    /// Its pictures in order.
    fn pictures(&self) -> impl Iterator<Item = &Shown> {
        iter::once(&self.first).chain(&self.rest)
    }

    /// Its whole length in finished frames: the sum of its holds.
    #[must_use]
    pub fn length(&self) -> num::NonZeroU32 {
        self.rest
            .iter()
            .fold(num::NonZeroU32::from(self.first.hold), |sum, shown| {
                sum.saturating_add(u32::from(shown.hold.get()))
            })
    }

    /// Whether `age` falls within it, before its last hold ends.
    #[must_use]
    pub fn holds(&self, age: u16) -> bool {
        u32::from(age) < self.length().get()
    }

    /// The picture whose hold includes `age`; none once its last hold has ended.
    #[must_use]
    pub fn picture_at(&self, age: u16) -> Option<SpriteRef> {
        let mut start = 0_u32;
        self.pictures().find_map(|shown| {
            start = start.saturating_add(u32::from(shown.hold.get()));
            (u32::from(age) < start).then_some(shown.sprite)
        })
    }

    /// Read as a repeating cycle: the age after `age`, back to zero once the cycle is complete.
    /// The importer keeps a cycle within 65,535 frames, so the age always fits; a longer one
    /// stays at the last age it can count.
    #[must_use]
    pub fn looped_after(&self, age: u16) -> u16 {
        let next = u32::from(age).saturating_add(1) % self.length();
        u16::try_from(next).unwrap_or(u16::MAX)
    }
}

/// A nonempty series of signed vertical displacements, one for each finished frame of flight.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Track {
    /// The displacements before the last.
    leading: Vec<Offset>,
    /// The last displacement.
    last: Offset,
}

impl Track {
    /// The track of these displacements, in flight order, if there is at least one.
    #[must_use]
    pub fn new(mut steps: Vec<Offset>) -> Option<Self> {
        let last = steps.pop()?;
        Some(Self {
            leading: steps,
            last,
        })
    }

    /// The displacement of flight frame `step`, counted from zero; none once the track is
    /// exhausted.
    #[must_use]
    pub fn at(&self, step: u16) -> Option<Offset> {
        let step = usize::from(step);
        self.leading
            .get(step)
            .copied()
            .or_else(|| (step == self.leading.len()).then_some(self.last))
    }

    /// The displacement of flight frame `step`, or the last one past the track's end.
    #[must_use]
    pub fn at_or_last(&self, step: u16) -> Offset {
        self.at(step).unwrap_or(self.last)
    }
}

/// Spec: `ShellScatters`. One shell fragment: its picture, where it starts, its drift and its
/// flight.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FragmentContent {
    /// Its picture.
    pub sprite: SpriteRef,
    /// Where the scatter places it.
    pub anchor: RoomPoint,
    /// Its horizontal move each finished frame.
    pub drift: Offset,
    /// Its vertical moves, ending at the latest frame on which it can still be drawn.
    pub track: Track,
}

/// Spec: `SlimeFalls`. One droplet: its picture, its drift and its fall. It starts at the
/// baby's whole-pixel place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DropletContent {
    /// Its picture while it falls.
    pub sprite: SpriteRef,
    /// Its horizontal move each finished frame.
    pub drift: Offset,
    /// Its vertical moves; past the end it keeps the last one until impact.
    pub track: Track,
}

/// Spec: `EggHatches`, `ShellScatters`, `BabyEmerges`, `PageThreeApproaches`. The egg's pictures
/// and the hatch frames that free its contents.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShellPictures {
    /// The closed egg.
    pub closed: SpriteRef,
    /// The hatch, rocking and opening.
    pub hatch: Animation,
    /// The spent shell, after the hatch.
    pub spent: SpriteRef,
    /// The hatch age on which the shell scatters its fragments.
    pub scatter_at: u16,
    /// The hatch age on which the baby comes free.
    pub release_at: u16,
}

/// The discovery's whole content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscoveryScene {
    /// Where the egg and the baby are placed.
    pub home: RoomPoint,
    /// The egg's pictures.
    pub shell: ShellPictures,
    /// The baby's repeating pulse.
    pub baby: Animation,
    /// The fragment cohort, in foreground precedence.
    pub fragments: Vec<FragmentContent>,
    /// The droplet cohort, in foreground precedence.
    pub droplets: Vec<DropletContent>,
    /// A splash's pictures after a droplet's impact.
    pub splash: Animation,
    /// The sound of the shell breaking.
    pub hatch_sound: SoundOffer,
    /// The baby's cry.
    pub cry_sound: SoundOffer,
}
