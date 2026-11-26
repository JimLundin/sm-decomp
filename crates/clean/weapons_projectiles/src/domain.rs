//! Quantities and caller contracts for the uncharged power beam and the intro fight's missiles.
use core::num;
use enum_map::{Enum, EnumMap};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Spec: MissileExhaust. Number of stable exhaust positions.
pub const EXHAUST_POSITIONS: usize = 18;

/// Spec: HoldingShoot. Five stable slots, in ascending reservation order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum ShotSlot {
    /// First slot.
    Slot0,
    /// Second slot.
    Slot1,
    /// Third slot.
    Slot2,
    /// Fourth slot.
    Slot3,
    /// Last slot.
    Slot4,
}
impl ShotSlot {
    /// Spec: ShotContact. Updates and pictures visit this descending order.
    pub const DESCENDING: [Self; 5] = [
        Self::Slot4,
        Self::Slot3,
        Self::Slot2,
        Self::Slot1,
        Self::Slot0,
    ];
    /// Spec: BeamVisibility. Even slots show on the odd presentation phase.
    #[must_use]
    pub const fn even(self) -> bool {
        matches!(self, Self::Slot0 | Self::Slot2 | Self::Slot4)
    }
}
/// Spec: InheritedMotion. Facing changes vertical pictures, not motion.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum Heading {
    /// Down and left.
    DownLeft,
    /// Down and right.
    DownRight,
    /// Down, facing left.
    DownFacingLeft,
    /// Down, facing right.
    DownFacingRight,
    /// Left.
    Left,
    /// Right.
    Right,
    /// Up and left.
    UpLeft,
    /// Up and right.
    UpRight,
    /// Up, facing left.
    UpFacingLeft,
    /// Up, facing right.
    UpFacingRight,
}
impl Heading {
    /// Spec: FlyingBeam. Signed aimed axes; zero means that axis is not moved.
    #[must_use]
    pub const fn axes(self) -> Axes<i16> {
        match self {
            Self::DownLeft => Axes { x: -1, y: 1 },
            Self::DownRight => Axes { x: 1, y: 1 },
            Self::DownFacingLeft | Self::DownFacingRight => Axes { x: 0, y: 1 },
            Self::Left => Axes { x: -1, y: 0 },
            Self::Right => Axes { x: 1, y: 0 },
            Self::UpLeft => Axes { x: -1, y: -1 },
            Self::UpRight => Axes { x: 1, y: -1 },
            Self::UpFacingLeft | Self::UpFacingRight => Axes { x: 0, y: -1 },
        }
    }
}
/// Spec: FlyingBeam. A quantity for each named spatial axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Axes<T> {
    /// Horizontal component, increasing rightward.
    pub x: T,
    /// Vertical component, increasing downward.
    pub y: T,
}
/// Spec: ShotSpacing. Wrapping update word.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameWord(pub u16);
/// Spec: HoldingShoot. Independent wrapping count; all words are valid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShotCount(pub u16);
/// Spec: FlyingBeam. Wrapping position in 1/65536 pixel units.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixedPosition(pub u32);
/// Spec: MuzzleContact. Fraction retained in a free slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PixelFraction(pub u16);
/// Spec: FlyingBeam. Signed wrapping speed in 1/256 pixel units.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeamSpeed(pub i16);
/// Spec: ShotContact. Half-size in pixels, 0 through 255.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Radius(pub u8);
/// Spec: HoldingShoot. Nonnegative damage, at most 32767.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u16", into = "u16")]
pub struct Damage(pub(crate) u16);
/// Spec: BeamPictures. Imported holds range from 0 through 32767.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u16", into = "u16")]
pub struct FrameHold(pub(crate) u16);
const MAX_NONNEGATIVE: u16 = 32767;
macro_rules! nonnegative_word {
    ($name:ident) => {
        impl $name {
            /// Construct a value in the specified nonnegative word range.
            #[must_use]
            pub const fn new(value: u16) -> Option<Self> {
                if value <= MAX_NONNEGATIVE {
                    Some(Self(value))
                } else {
                    None
                }
            }
            /// The copied word value.
            #[must_use]
            pub const fn value(self) -> u16 {
                self.0
            }
        }
        impl TryFrom<u16> for $name {
            type Error = &'static str;
            fn try_from(value: u16) -> Result<Self, Self::Error> {
                Self::new(value).ok_or("value exceeds nonnegative word range")
            }
        }
        impl From<$name> for u16 {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}
nonnegative_word!(Damage);
nonnegative_word!(FrameHold);
/// Spec: BeamPictures. Caller-supplied clip identifier.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ClipId(pub String);
/// Spec: BeamVisibility. Caller-supplied picture identifier.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PictureId(pub String);
/// Spec: BeamTrail. Caller-supplied trail appearance identifier.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TrailAppearanceId(pub String);
/// Spec: BeamPictures. Frame index, including a clip's end index.
/// Index-to-clip relations, including restored indices, belong to the importer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClipFrame(usize);
impl ClipFrame {
    /// Accept an index up to and including the clip end.
    #[must_use]
    pub const fn new(index: usize, frame_count: usize) -> Option<Self> {
        if index <= frame_count {
            Some(Self(index))
        } else {
            None
        }
    }
    /// Index in the caller's clip.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
    pub(crate) const fn at(index: usize) -> Self {
        Self(index)
    }
}
/// Spec: BeamPictures. Imported frame values; None leaves the trail channel unchanged.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    /// Time to retain this displayed frame.
    pub hold: FrameHold,
    /// Picture displayed by this frame.
    pub picture: PictureId,
    /// Collision half-sizes copied into the body.
    pub radius: Axes<Radius>,
    /// Some replaces the retained appearance; None is an Inactive frame, which leaves it
    /// unchanged. This is not the same as a body whose retained appearance is None.
    pub trail_appearance: Option<TrailAppearanceId>,
}
/// Spec: BeamPictures. A clip either removes the shot or repeats a named frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClipEnding {
    /// End the shot.
    Remove,
    /// Display this existing frame again.
    RepeatFrom(ClipFrame),
}
/// Spec: BeamPictures. Nonempty frames and valid repeat relations are the importer's contract.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clip {
    /// Imported frames.
    pub frames: Vec<Frame>,
    /// What happens after the last hold.
    pub ending: ClipEnding,
}
/// Spec: BeamPictures. Imported clips by identifier, one library per weapon.
pub type ClipLibrary = BTreeMap<ClipId, Clip>;
/// Spec: HoldingShoot, BeamPictures. Imported beam content.
///
/// The importer supplies every heading's clip, resolves flying and impact ClipIds in clips,
/// validates picture/trail identifiers, nonempty clips and RepeatFrom targets strictly inside
/// their clip. Callers restoring bodies must validate clip references and next_frame at or
/// before that clip's end. The model does not validate these cross-field relations. Zero
/// damage, holds and radii are valid content, not missing-content sentinels.
#[derive(Clone, Debug)]
pub struct PowerBeam {
    /// Damage assigned at spawn.
    pub damage: Damage,
    /// Flying clip for every heading.
    pub flying_clips: EnumMap<Heading, ClipId>,
    /// Common burst clip.
    pub impact_clip: ClipId,
    /// Imported animation values.
    pub clips: ClipLibrary,
}
/// Spec: MissileExhaust. One stationary exhaust picture, held for a positive number of
/// presentation visits rather than gameplay updates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExhaustFrame {
    /// Presentation visits this picture stays.
    pub hold: num::NonZeroU16,
    /// Picture shown.
    pub picture: PictureId,
}
/// Spec: MissileExhaust. Where a puff appears relative to the missile and how it animates.
/// A nonempty frame list is the importer's contract; the puff disappears after its last hold.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExhaustAppearance {
    /// Displacement from the missile's whole centre.
    pub offset: Offset,
    /// Pictures shown in order, then the puff ends.
    pub frames: Vec<ExhaustFrame>,
}
/// Spec: MissileRelease, MissileBurst, MissileExhaust. Imported ordinary missile content.
///
/// The importer resolves both clip identifiers in `clips` and gives each flying frame's trail
/// appearance an exhaust entry. Flight repeats; impact ends with Remove.
#[derive(Clone, Debug)]
pub struct MissileContent {
    /// Damage assigned at release.
    pub damage: Damage,
    /// Leftward flight clip.
    pub flight_clip: ClipId,
    /// Stationary impact clip.
    pub impact_clip: ClipId,
    /// Imported animation values for both clips.
    pub clips: ClipLibrary,
    /// Exhaust puff for each flight trail appearance.
    pub exhaust: BTreeMap<TrailAppearanceId, ExhaustAppearance>,
}
/// Spec: BeamPictures, MissileTravel. All weapon content a shared slot table can hold.
#[derive(Clone, Debug)]
pub struct WeaponContent {
    /// Uncharged power beam.
    pub beam: PowerBeam,
    /// Ordinary missile.
    pub missile: MissileContent,
}
impl WeaponContent {
    /// Spec: BeamPictures. Each kind of shot plays clips from its own weapon's library.
    #[must_use]
    pub const fn clips(&self, kind: ShotKind) -> &ClipLibrary {
        match kind {
            ShotKind::Beam => &self.beam.clips,
            ShotKind::Missile(_) => &self.missile.clips,
        }
    }
}
/// Spec: CannonOrigin. Integer muzzle offset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Offset {
    /// Horizontal integer pixels; reduced only when forming the wrapping origin.
    pub x: i64,
    /// Vertical integer pixels; reduced only when forming the wrapping origin.
    pub y: i64,
}
/// Spec: CannonOrigin. Descriptor aim selection, before a possible pending override.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MuzzleAim {
    /// No muzzle aim.
    Forbidden,
    /// Accept only an Aimed preceding descriptor.
    PreviousAim,
    /// Use this heading.
    Aimed(Heading),
}
/// Spec: CannonOrigin. Pose-supplied aim, lift and complete offset maps.
#[derive(Clone, Debug)]
pub struct Muzzle {
    /// Aim eligibility supplied by the pose importer.
    pub aim: MuzzleAim,
    /// Unsigned lift subtracted from the vertical origin.
    pub lift: u64,
    /// Offsets used when the muzzle is stationary.
    pub stationary: EnumMap<Heading, Offset>,
    /// Offsets used when the muzzle is moving.
    pub moving: EnumMap<Heading, Offset>,
}
/// Spec: CannonOrigin. Samus's integer centre, distinct from a view origin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SamusCentre(pub Axes<u16>);
/// Spec: LeavingView, BeamVisibility. Integer view origin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScreenOrigin(pub Axes<u16>);
/// Spec: BeamVisibility. Already transformed integer screen coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScreenCentre(pub Axes<u16>);
/// Spec: InheritedMotion. Her preceding movement distances, in whole and fractional words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MovementDistance {
    /// Directional whole-pixel word, not a magnitude. Left/up inputs may encode negative
    /// travel (for example word(-2)); inheritance reads only its low byte, with wrapping
    /// arithmetic, rather than taking an absolute value or sign-extending the whole word.
    pub whole: u16,
    /// Fractional word in 1/65536 px; inheritance uses its high byte as 1/256 px speed units.
    pub fraction: PixelFraction,
}
/// Spec: InheritedMotion. Named directional distances and camera fractional vertical speed.
///
/// Rightward speed combines right whole with *left* fraction; leftward combines left whole
/// with *camera vertical* fraction. Downward combines down whole with *up* fraction. Upward
/// ignores fractions when the up whole low byte is zero; otherwise it combines up whole and
/// *right* fraction, shifts right two and ORs 49152 before adding the negative base. Down's
/// fraction never contributes. Cardinal base is 1024, diagonal 683; unaimed axes stay zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Inheritance {
    /// Leftward distance.
    pub left: MovementDistance,
    /// Rightward distance.
    pub right: MovementDistance,
    /// Upward distance.
    pub up: MovementDistance,
    /// Downward distance.
    pub down: MovementDistance,
    /// Camera's fractional vertical speed word.
    pub camera_vertical_fraction: PixelFraction,
}
impl Inheritance {
    /// Explicit zero preceding movement, not a content default.
    #[must_use]
    pub const fn stationary() -> Self {
        let zero = MovementDistance {
            whole: 0,
            fraction: PixelFraction(0),
        };
        Self {
            left: zero,
            right: zero,
            up: zero,
            down: zero,
            camera_vertical_fraction: PixelFraction(0),
        }
    }
}
/// Spec: ShotSpacing. Cannon inactivity does not freeze existing shots.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpdateMode {
    /// Update cannon before shots.
    CannonAndShots,
    /// Leave cannon unchanged and update existing shots.
    ShotsOnly,
}
/// Spec: HoldingShoot, CannonOrigin. Neighbour-supplied firing eligibility and muzzle inputs.
#[expect(
    clippy::struct_excessive_bools,
    reason = "Independent neighbouring actions and muzzle selection are specified inputs"
)]
#[derive(Clone, Debug)]
pub struct CannonInput {
    /// Held selected, pose-eligible shoot action.
    pub shoot: bool,
    /// New press this update.
    pub new_press: bool,
    /// New press on the preceding update.
    pub previous_new_press: bool,
    /// Current imported muzzle descriptor.
    pub muzzle: Muzzle,
    /// Previous imported descriptor's aim.
    pub previous_muzzle: MuzzleAim,
    /// Use the moving offset map.
    pub moving_muzzle: bool,
    /// Pose-change aim overriding any descriptor.
    pub pending_aim: Option<Heading>,
    /// Samus's integer centre.
    pub centre: SamusCentre,
}
/// Spec: MissileRelease. What Samus and inventory say about the ordinary missile cannon.
///
/// Fresh presses (this frame's and the retained previous frame's) come from the shared
/// [`CannonInput`]; its held `shoot` does not release a missile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MissileSupply {
    /// Her pose and state permit firing the cannon.
    pub eligible: bool,
    /// Inventory has ordinary missiles selected.
    pub selected: bool,
    /// Missiles remaining before this update.
    pub ammo: u16,
    /// The selection cancels itself after a shot.
    pub cancel_after_shot: bool,
}
/// Spec: ShotContact. Room extents in screens, not an implicit outside solid boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoomScreens(pub Axes<u16>);
/// Spec: ShotSpacing. Complete neighbouring frame inputs, with no raw pose numbers.
#[derive(Clone, Debug)]
pub struct UpdateInput {
    /// Which update producer is active.
    pub mode: UpdateMode,
    /// Whether time is stopped.
    pub time_stopped: bool,
    /// Cannon eligibility and origin inputs.
    pub cannon: CannonInput,
    /// Motion inherited on successful spawn.
    pub inheritance: Inheritance,
    /// View check origin.
    pub camera: ScreenOrigin,
    /// Room extents for contact.
    pub room: RoomScreens,
}
/// Spec: MissileTravel. Whether a flying missile is still awaiting its initial speed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissileLaunch {
    /// A fresh missile awaiting its initial speed.
    Pending,
    /// Already moving; each update accelerates it.
    Launched,
}
/// Spec: MissileTravel, MissileBurst. Which weapon a body is, kept through flight and impact.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShotKind {
    /// An uncharged power beam.
    Beam,
    /// An ordinary missile and its launch progress.
    Missile(MissileLaunch),
}
/// Spec: HoldingShoot. Every live body's copied values are caller-owned state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShotBody {
    /// Which weapon this body is.
    pub kind: ShotKind,
    /// Wrapping fixed-point centre.
    pub position: Axes<FixedPosition>,
    /// Aim retained for acceleration, contact and bursts.
    pub heading: Heading,
    /// Copied signed wrapping speeds.
    pub speed: Axes<BeamSpeed>,
    /// Copied collision half-sizes.
    pub radius: Axes<Radius>,
    /// Copied nonnegative damage.
    pub damage: Damage,
    /// Countdown retained on removal.
    pub trail: FrameWord,
    /// Current animation clip.
    pub clip: ClipId,
    /// Next frame, or the end index.
    pub next_frame: ClipFrame,
    /// Wrapping remaining hold.
    pub remaining_hold: FrameWord,
    /// Currently visible picture, if any.
    pub picture: Option<PictureId>,
    /// Retained meaningful trail appearance, if any.
    pub trail_appearance: Option<TrailAppearanceId>,
}
/// Spec: EndingShot. Free slots have no discarded body or hold.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShotState {
    /// Only countdown and reset fractions survive here.
    Free {
        /// Retained countdown.
        trail: FrameWord,
        /// Fractions supplied at room/elevator reset.
        retained_fraction: Axes<PixelFraction>,
    },
    /// A moving shot, optionally spent by a later actor event.
    Flying {
        /// Copied body values.
        body: ShotBody,
        /// Remove on the next active weapons update.
        consumed: bool,
    },
    /// A burst playing its clip.
    Impact {
        /// Copied body values.
        body: ShotBody,
    },
}
impl ShotState {
    /// Spec: BeamVisibility. A free slot has no body to draw.
    #[must_use]
    pub const fn body(&self) -> Option<&ShotBody> {
        match self {
            Self::Free { .. } => None,
            Self::Flying { body, .. } | Self::Impact { body } => Some(body),
        }
    }
    pub(crate) const fn body_mut(&mut self) -> Option<&mut ShotBody> {
        match self {
            Self::Free { .. } => None,
            Self::Flying { body, .. } | Self::Impact { body } => Some(body),
        }
    }
}
/// Spec: MissileExhaust, CinematicMissilePictures. A puff of missile exhaust in the room.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExhaustPuff {
    /// Whole-pixel room position, wrapping on each axis.
    pub position: Axes<u16>,
    /// Appearance whose pictures this puff plays.
    pub appearance: TrailAppearanceId,
    /// Picture now shown; None until the first presentation visit after placement.
    pub picture: Option<PictureId>,
    /// Next exhaust frame, or the end index.
    pub next_frame: ClipFrame,
    /// Wrapping presentation visits left on the shown picture.
    pub remaining_hold: FrameWord,
}
/// Spec: MissileExhaust. One stable exhaust position.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExhaustSlot {
    /// No puff here.
    Free,
    /// A puff ageing with each presentation visit.
    Puff(ExhaustPuff),
}
/// Spec: HoldingShoot, EndingShot, MissileExhaust. Complete serializable memory, independent
/// of occupancy.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    /// Wrapping cannon cooldown.
    pub cooldown: FrameWord,
    /// Independent wrapping count.
    pub count: ShotCount,
    /// Complete table in stable slot order.
    pub slots: EnumMap<ShotSlot, ShotState>,
    /// Missile exhaust positions, in stable ascending order.
    pub exhaust: [ExhaustSlot; EXHAUST_POSITIONS],
}
impl State {
    /// Room/elevator reset clears everything except caller-supplied slot fractions; the room
    /// starts with no exhaust.
    #[must_use]
    pub fn reset(fractions: EnumMap<ShotSlot, Axes<PixelFraction>>) -> Self {
        Self {
            cooldown: FrameWord(0),
            count: ShotCount(0),
            slots: EnumMap::from_fn(|slot| ShotState::Free {
                trail: FrameWord(0),
                retained_fraction: fractions[slot],
            }),
            exhaust: std::array::from_fn(|_| ExhaustSlot::Free),
        }
    }
    /// Spec: EndingShot. Late actor consumption changes Flying only, without removing it.
    pub fn consume(&mut self, slot: ShotSlot) {
        if let ShotState::Flying { consumed, .. } = &mut self.slots[slot] {
            *consumed = true;
        }
    }
}
/// Spec: ShotContact. Horizontal motion/contact precedes vertical motion/contact.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    /// Horizontal edge, visited top to bottom.
    Horizontal,
    /// Vertical edge, visited left to right.
    Vertical,
}
/// Spec: ShotContact. Complete geometry passed to World for an ordered edge span.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Contact {
    /// Stable shot slot.
    pub slot: ShotSlot,
    /// Retained heading.
    pub heading: Heading,
    /// Axis of this contact.
    pub axis: Axis,
    /// Integer centre after this axis's motion.
    pub centre: Axes<u16>,
    /// Copied collision half-sizes.
    pub radius: Axes<Radius>,
    /// Leading integer boundary.
    pub leading_boundary: u16,
    /// Perpendicular start pixel.
    pub perpendicular_start: u16,
    /// Number of cells visited, in order.
    pub cell_count: u16,
}
/// Spec: ShotContact. World's immediate answer and reaction for one cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContactResponse {
    /// Blocks this cell of the edge.
    Blocks,
    /// Lets this cell of the edge pass.
    Passes,
}
/// Spec: ShotContact. World maps geometric edge contacts into room content and reacts.
///
/// `cell` is a zero-based ordinal less than cell_count, not a content index or address.
/// Horizontal edges visit perpendicular cells top to bottom; vertical edges left to right,
/// starting with the 16-pixel cell containing perpendicular_start. leading_boundary locates
/// the edge along the moved axis; centre, radius and heading describe the shot making it.
/// Apply each cell's reaction immediately before returning Blocks or Passes. Every cell is
/// called even after Passes, so subsequent reactions see earlier changes.
///
/// The caller supplies room content, its geometry-to-content mapping and block responses.
/// ContactContinuation delegates limited-row/width aliases and contacts past the level to
/// World: preserve their responses and reaction order, rather than inventing solid outside
/// cells. The spec supplies no mapping algorithm for that continuation. Production support
/// needs the integrator's W2 capability check; implementing this trait alone proves nothing
/// about that capability. W1 exposes neither byte addresses nor partial-cell addresses.
pub trait World {
    /// Answer and react to zero-based ordinal `cell` in this ordered geometric span.
    fn contact_cell(&mut self, contact: &Contact, cell: u16) -> ContactResponse;
    /// Spec: MissileTravel. Answer and react to one horizontal point contact at a moved
    /// missile's integer centre; World owns its aliasing and reactions to shots.
    fn contact_point(&mut self, slot: ShotSlot, centre: Axes<u16>) -> ContactResponse;
}
/// Spec: BeamTrail, HoldingShoot, ShotContact. Requests returned instead of side effects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// Play the fire sound.
    FireSound(ShotSlot),
    /// Request incoming-shot protection for this duration.
    Protection(FrameWord),
    /// Tell the caller its pending aim was used.
    PendingAimConsumed,
    /// Play the impact sound.
    ImpactSound(ShotSlot),
    /// Trail position and retained appearance before motion.
    Trail {
        /// Stable producing slot.
        slot: ShotSlot,
        /// Pre-motion position.
        position: Axes<FixedPosition>,
        /// Retained picture before the clip update.
        picture: Option<PictureId>,
        /// Retained appearance before the clip update.
        appearance: Option<TrailAppearanceId>,
    },
    /// Spec: MissileRelease. Inventory spends one missile.
    MissileDebit,
    /// Spec: MissileRelease. Inventory clears the selection and its automatic cancellation.
    ClearSelection,
    /// Spec: MissileExhaust. A flying missile asked for a puff, before it moved; the model has
    /// already placed it when a position was free.
    PuffRequest {
        /// Stable producing slot.
        slot: ShotSlot,
        /// Whole centre before motion.
        centre: Axes<u16>,
        /// Previously displayed flight appearance.
        appearance: Option<TrailAppearanceId>,
    },
}
/// Spec: MissileTravel. A surviving flying missile the encounter may hit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MissileCandidate {
    /// Stable missile identity.
    pub slot: ShotSlot,
    /// Whole x after this update's motion.
    pub whole_x: u16,
}
/// Spec: MissileRelease, MissileTravel. One intro-fight weapons update's requests and the
/// missiles the encounter reads afterwards.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntroFightUpdate {
    /// Requests in the order they arose.
    pub events: Vec<Event>,
    /// Flying ordinary missiles in descending slot order.
    pub candidates: Vec<MissileCandidate>,
}
/// Spec: BeamVisibility. Presentation phase supplied by drawing, not remembered in State.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlickerPhase {
    /// Show odd Flying slots.
    Even,
    /// Show even Flying slots.
    Odd,
}
/// Spec: BeamVisibility. Normal subtraction or complete caller-transformed screen centres.
#[derive(Clone, Debug)]
pub enum DrawView {
    /// Subtract this view origin.
    Normal(ScreenOrigin),
    /// Use transformed centres directly; every live slot has a supplied value.
    Rotating(EnumMap<ShotSlot, ScreenCentre>),
}
/// Spec: BeamVisibility. Sprite encoding is the consumer's job.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PictureIntent {
    /// Stable producing slot.
    pub slot: ShotSlot,
    /// Retained visible picture.
    pub picture: PictureId,
    /// Wrapping integer screen pixels.
    pub screen: ScreenCentre,
}
/// Spec: BeamVisibility. Separately ordered producers for placement among other pictures.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PictureIntents {
    /// Flying pictures in descending slot order.
    pub flying: Vec<PictureIntent>,
    /// Impact pictures in descending slot order.
    pub impact: Vec<PictureIntent>,
}
/// Spec: CinematicMissilePictures. One drawn exhaust puff.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExhaustIntent {
    /// Stable exhaust position, 0 through 17.
    pub position: usize,
    /// Puff picture.
    pub picture: PictureId,
    /// Integer screen pixels.
    pub screen: ScreenCentre,
}
/// Spec: CinematicMissilePictures. One intro drawing opportunity: its missiles are drawn
/// first, then its exhaust.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntroPictures {
    /// Flying ordinary missiles in descending slot order.
    pub missiles: Vec<PictureIntent>,
    /// Surviving puffs in descending position order.
    pub exhaust: Vec<ExhaustIntent>,
}
