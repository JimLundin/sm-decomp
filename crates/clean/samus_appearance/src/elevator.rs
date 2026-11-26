//! The standing body's contribution to a locked elevator room picture. Video owns publication.

use std::collections::BTreeMap;

use enum_map::{Enum, EnumMap, enum_map};
use types::{Action, Aim, Body, Facing, Gun, Legs, Pair, Pose, Side};

use crate::clip::Step;

/// The side and displayed clip step of an admitted elevator standing pose.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StandingLook {
    /// The side her standing pose faces.
    pub side: Side,
    /// The step her clip currently displays.
    pub step: Step,
}

impl StandingLook {
    /// Spec: `ElevatorStandingPicture`. Admit only an upright, still, side-facing pose
    /// with forward aim, stowed cannon and legs together. Keep its displayed step.
    ///
    /// The caller admits only the ordinary Power suit without active damage flicker.
    #[must_use]
    pub const fn from_pose(pose: Pose, step: Step) -> Option<Self> {
        if !matches!(
            (pose.body, pose.action, pose.aim, pose.gun, pose.legs),
            (
                Body::Upright,
                Action::Still,
                Aim::Forward,
                Gun::Stowed,
                Legs::Together
            )
        ) {
            return None;
        }
        match pose.facing {
            Facing::Left => Some(Self {
                side: Side::Left,
                step,
            }),
            Facing::Right => Some(Self {
                side: Side::Right,
                step,
            }),
            Facing::Screen => None,
        }
    }
}

/// Signed 8.8 terms from the shaft view carried into drawing, before the late tilt step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShaftView {
    /// The diagonal scale term.
    pub diagonal: i16,
    /// The signed cross-axis term.
    pub cross: i16,
}

/// Whole-pixel room geometry at the drawing opportunity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ElevatorProjection {
    /// Samus's unprojected centre in the room.
    pub centre: Pair<i32>,
    /// The shaft's pivot in the room.
    pub pivot: Pair<i32>,
    /// The room layer origin at drawing.
    pub camera_origin: Pair<i32>,
    /// The shaft view at the start of the room turn.
    pub carried_view: ShaftView,
}

/// Signed vertical displacement of the pose's art, positive downward.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArtDisplacement(pub i32);

/// Whole pixels in screen space.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScreenPoint(pub Pair<i64>);

const VIEW_UNIT: i64 = 256;

impl ElevatorProjection {
    /// Spec: `ElevatorStandingPicture`. Floor each product separately, then subtract the
    /// camera and apply the pose's art displacement. This does not move her room centre.
    #[must_use]
    pub fn origin(self, displacement: ArtDisplacement) -> ScreenPoint {
        // Widen before computing: the 32-bit pixel differences times the 16-bit view
        // terms, and all sums below, fit in 64 bits without wrapping.
        let pivot_x = i64::from(self.pivot.x);
        let pivot_y = i64::from(self.pivot.y);
        let right = i64::from(self.centre.x).wrapping_sub(pivot_x);
        let up = pivot_y.wrapping_sub(i64::from(self.centre.y));
        let diagonal = i64::from(self.carried_view.diagonal);
        let cross = i64::from(self.carried_view.cross);
        let project = |term: i64, pixels| term.wrapping_mul(pixels).div_euclid(VIEW_UNIT);
        ScreenPoint(Pair {
            x: pivot_x
                .wrapping_add(project(diagonal, right))
                .wrapping_add(project(cross, up))
                .wrapping_sub(i64::from(self.camera_origin.x)),
            y: pivot_y
                .wrapping_sub(project(cross.wrapping_neg(), right))
                .wrapping_sub(project(diagonal, up))
                .wrapping_sub(i64::from(self.camera_origin.y))
                .wrapping_add(i64::from(displacement.0)),
        })
    }
}

/// The two sizes of object pieces in the standing art.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectSize {
    /// A small object piece.
    Small,
    /// A large object piece.
    Large,
}

/// Object properties preserved from imported art to screen placement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectStyle {
    /// The tile used by this piece.
    pub tile: u16,
    /// The object's palette selection.
    pub palette: u8,
    /// The object's size.
    pub size: ObjectSize,
    /// The object's draw priority.
    pub priority: u8,
    /// Whether the piece is flipped on each axis.
    pub flips: Pair<bool>,
}

/// A piece anchored by an art offset or a screen position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BodyObject<P> {
    /// The piece's art offset or screen position, as determined by `P`.
    pub position: P,
    /// The piece's unchanged drawing properties.
    pub style: ObjectStyle,
}

/// An imported piece offset from the body's art origin.
pub type BodyPiece = BodyObject<Pair<i32>>;
/// A piece placed on screen for video.
pub type BodyPlacement = BodyObject<ScreenPoint>;

impl BodyPiece {
    fn place(self, origin: ScreenPoint) -> BodyPlacement {
        BodyPlacement {
            position: ScreenPoint(Pair {
                x: origin.0.x.wrapping_add(i64::from(self.position.x)),
                y: origin.0.y.wrapping_add(i64::from(self.position.y)),
            }),
            style: self.style,
        }
    }
}

/// Each strip has its own reserved destination in the body's object tiles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum BodyRow {
    /// First upper-body strip.
    UpperFirst,
    /// Second upper-body strip.
    UpperSecond,
    /// First lower-body strip.
    LowerFirst,
    /// Second lower-body strip.
    LowerSecond,
}

/// Art for one half of a standing step: ordered pieces and their imported tile strips.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BodyHalf<T> {
    /// Imported pieces in drawing order.
    pub pieces: Vec<BodyPiece>,
    /// Tiles for the first reserved row of this half.
    pub first_strip: T,
    /// Tiles for the second reserved row, when supplied.
    pub second_strip: Option<T>,
}

/// The upper and lower art for one displayed standing step.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StandingStepArt<T> {
    /// Upper-body art, drawn first.
    pub upper: BodyHalf<T>,
    /// Lower-body art, drawn after the upper pieces.
    pub lower: BodyHalf<T>,
}

/// Imported art for a standing pose, indexed by its displayed step.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StandingPoseArt<T> {
    /// Vertical displacement shared by the pose's steps.
    pub displacement: ArtDisplacement,
    /// Imported pictures and strips by displayed clip step.
    pub steps: BTreeMap<Step, StandingStepArt<T>>,
}

/// Imported standing art for the two side-facing poses. `T` is the importer's tile-strip type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElevatorArt<T> {
    /// The left-facing standing art.
    pub left: StandingPoseArt<T>,
    /// The right-facing standing art.
    pub right: StandingPoseArt<T>,
}

/// Only a new locked room drawing submits body work. Resuming an interrupted cut does not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElevatorDrawing {
    /// A new locked elevator room drawing.
    Locked {
        /// The current pose supplied by movement.
        pose: Pose,
        /// The clip step currently displayed, without advancing the clip.
        step: Step,
        /// The geometry and carried view for this drawing.
        projection: ElevatorProjection,
    },
    /// No new room drawing, even if video is about to publish earlier work.
    NoNewDrawing,
}

/// Body work for video, between the countdown and the room actors.
///
/// Video keeps prepared work across interrupted cuts and installs it at its next requested
/// publication. An absent strip leaves only its own reserved row unchanged.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElevatorBody<T> {
    /// Screen placements in drawing order, upper then lower.
    pub placements: Vec<BodyPlacement>,
    /// The first reserved row of each body half receives graphics, and the second does only
    /// when a strip is supplied.
    pub strips: EnumMap<BodyRow, Option<T>>,
}

/// Why a new locked drawing could not submit body work.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ElevatorPictureError {
    /// The supplied pose is outside the supported standing poses.
    UnsupportedPose,
    /// No imported art exists for this exact facing and displayed step.
    MissingArt(StandingLook),
}

/// Spec: `ElevatorStandingPicture`. Admit the current pose, then place the upper pieces before
/// the lower pieces from the same projected origin, using the exact facing and displayed step.
///
/// A cut without a new drawing returns `Ok(None)` and leaves previous video work intact.
///
/// # Errors
/// Returns [`ElevatorPictureError::UnsupportedPose`] for a new drawing outside the standing
/// pose rule, or [`ElevatorPictureError::MissingArt`] if the admitted look has no imported art.
pub fn elevator_standing_picture<T>(
    drawing: ElevatorDrawing,
    art: &ElevatorArt<T>,
) -> Result<Option<ElevatorBody<&T>>, ElevatorPictureError> {
    let ElevatorDrawing::Locked {
        pose,
        step,
        projection,
    } = drawing
    else {
        return Ok(None);
    };
    let look = StandingLook::from_pose(pose, step).ok_or(ElevatorPictureError::UnsupportedPose)?;
    let pose_art = match look.side {
        Side::Left => &art.left,
        Side::Right => &art.right,
    };
    let step = pose_art
        .steps
        .get(&look.step)
        .ok_or(ElevatorPictureError::MissingArt(look))?;
    let origin = projection.origin(pose_art.displacement);
    Ok(Some(ElevatorBody {
        placements: step
            .upper
            .pieces
            .iter()
            .chain(&step.lower.pieces)
            .map(|piece| piece.place(origin))
            .collect(),
        strips: enum_map! {
            BodyRow::UpperFirst => Some(&step.upper.first_strip),
            BodyRow::UpperSecond => step.upper.second_strip.as_ref(),
            BodyRow::LowerFirst => Some(&step.lower.first_strip),
            BodyRow::LowerSecond => step.lower.second_strip.as_ref(),
        },
    }))
}
