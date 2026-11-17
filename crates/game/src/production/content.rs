//! Checked pose content and the exact context saved with production state.

use super::AdmissionFault;
use data::assets::samus::{key::shown_as, map::MAP};
use types::{FiringDirection, Pose, Radius};

/// Static content for one semantic pose.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PoseContent {
    /// Semantic lookup key.
    pub pose: Pose,
    /// Imported body half-sizes.
    pub radius: Radius,
    /// Optional imported ordinary transition firing direction.
    pub cannon_heading: Option<FiringDirection>,
}

/// Pose content with complete selectable coverage and no conflicting keys.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MovementContent {
    poses: Vec<PoseContent>,
}

impl MovementContent {
    /// Validate selectable pose coverage before movement can read the table.
    ///
    /// # Errors
    /// Missing selectable poses and conflicting duplicate keys are admission faults.
    pub fn new(poses: Vec<PoseContent>) -> Result<Self, AdmissionFault> {
        let content = Self { poses };
        content.validate()?;
        Ok(content)
    }

    pub(super) fn validate(&self) -> Result<(), AdmissionFault> {
        for (index, entry) in self.poses.iter().enumerate() {
            if self
                .poses
                .iter()
                .take(index)
                .any(|prior| prior.pose == entry.pose && prior != entry)
            {
                return Err(AdmissionFault::ConflictingPose(entry.pose));
            }
        }
        for entry in MAP {
            if self.pose(entry.key).is_none() {
                return Err(AdmissionFault::MissingPose(entry.key));
            }
        }
        Ok(())
    }

    /// Imported half-sizes for a selectable semantic pose.
    #[must_use]
    pub fn radius(&self, pose: Pose) -> Option<Radius> {
        self.pose(pose).map(|entry| entry.radius)
    }

    /// Optional ordinary transition direction for a selectable semantic pose.
    #[must_use]
    pub fn cannon_heading(&self, pose: Pose) -> Option<FiringDirection> {
        self.pose(pose).and_then(|entry| entry.cannon_heading)
    }

    pub(super) fn pose(&self, pose: Pose) -> Option<&PoseContent> {
        self.poses
            .iter()
            .find(|entry| entry.pose == pose)
            .or_else(|| self.poses.iter().find(|entry| entry.pose == shown_as(pose)))
    }
}

/// Content selected by the finite production recipe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProductionContent {
    /// Checked semantic pose/body/cannon content; changing geometry is an input.
    Movement(MovementContent),
    /// Checked movement content with fixed clean Ceres geometry.
    MovementCeres(MovementContent),
    /// Imported resident effects and arrangement.
    Audio(audio_transport::Content),
    /// Checked imported ordinary power assets.
    WeaponsContact(Box<crate::weapons_projectiles_bridge::PowerContent>),
    /// Blocks carry their imported reactions in the complete clean State.
    Blocks,
    /// Camera room/scroll constraints are explicit inputs.
    Camera,
    /// Exact imported semantic animation clips and paces.
    Appearance(super::AppearanceContent),
    /// Progress reports carry the typed world facts they record.
    Progress,
    /// Every named imported HUD picture.
    Hud(Box<interface_hud::HudPictures>),
    /// Ceres door geometry and figures are defined by its clean model.
    Ceres,
    /// Passage scene and doorway facts are carried State and explicit inputs.
    Doors,
    /// The one request group whose queue and physical reply line this sender owns.
    AudioSender(audio_transport::SoundGroup),
}

/// Rules selected by the same recipe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProductionRules {
    /// The existing movement model's complete rules.
    Movement(samus_movement::Rules),
    /// Complete parameters for both connected owners.
    MovementCeres {
        /// Movement tuning.
        movement: samus_movement::Rules,
        /// Ceres tuning.
        ceres: boss_ceres_ridley::Rules,
    },
    /// Transport has fixed catalogue rules.
    Audio,
    /// The existing power model's tuning parameters.
    WeaponsContact(weapons_projectiles::Rules),
    /// Complete immutable block rule tables.
    Blocks(super::BlockRules),
    /// Complete following-camera parameters.
    Camera(super::CameraRules),
    /// Complete animation tuning.
    Appearance(samus_appearance::Rules),
    /// Complete progress parameters.
    Progress(run_progress::Rules),
    /// Complete immutable HUD parameters.
    Hud(super::HudRules),
    /// Complete Ceres door parameters.
    Ceres(boss_ceres_ridley::Rules),
    /// Complete passage parameters.
    Doors(world_doors::Rules),
    /// The sender uses its fixed clean parameters and an admitted service profile.
    AudioSender,
}

/// Exact saved context; restoring with other content or rules is refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Context {
    pub content: ProductionContent,
    pub rules: ProductionRules,
}
