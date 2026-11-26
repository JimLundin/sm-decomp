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
    /// The imported intro scene.
    Intro(Box<screens_intro::IntroContent>),
    /// The intro's room collision and Samus's ordinary body and appearance.
    IntroRooms(Box<super::played::intro_rooms::ScopedContent>),
    /// Checked encounter clip maps and normal sprite palette.
    Flashback(Box<data::assets::flashback::Scene>),
    /// The imported recorded pad holds.
    Playback(super::playback::Content),
    /// Checked battery format and packed map projection.
    SaveFiles(Box<crate::exact::save_files::Format>),
    /// Checked cinematic missile and power weapon assets.
    IntroMissiles(Box<super::played::intro_missiles::Content>),
    /// Checked shared connected movement/appearance content.
    Connected(Box<super::connected::Content>),
    /// Checked current gauge content.
    HealthTail(Box<super::health_tail::Content>),
    /// Checked semantic pose/body/cannon content; changing geometry is an input.
    Movement(MovementContent),
    /// Checked body and cannon content shared by the live touchdown caller, and the audio
    /// content its Contact offer is played with.
    MovementTouchdown(MovementContent, Box<super::audio::Content>),
    /// Checked effect clips, song graph and the resources they name.
    Audio(Box<super::audio::Content>),
    /// Checked imported ordinary power assets.
    WeaponsContact(Box<crate::weapons_projectiles_bridge::PowerContent>),
    /// Blocks carry their imported reactions in the complete clean State.
    Blocks,
    /// Camera room/scroll constraints are explicit inputs.
    Camera,
    /// Exact imported semantic animation clips and paces.
    Appearance(Box<super::AppearanceContent>),
    /// Progress reports carry the typed world facts they record.
    Progress,
    /// Every named imported HUD picture.
    Hud(Box<interface_hud::HudPictures>),
    /// Ceres door geometry and figures are defined by its clean model.
    Ceres,
    /// Passage scene and doorway facts are carried State and explicit inputs.
    Doors,
}

/// Rules selected by the same recipe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProductionRules {
    /// Rules of the intro scene.
    Intro(screens_intro::Rules),
    /// The ordinary room turn's movement and appearance rules.
    IntroRooms(Box<super::played::intro_rooms::ScopedRules>),
    /// Parameters of the Zebes flashback encounter.
    Flashback(super::played::flashback::Rules),
    /// The playback catalogue rules are fixed.
    Playback,
    /// Cartridge rules are fixed.
    SaveFiles,
    /// Complete weapon rule parameters in the cinematic.
    IntroMissiles(weapons_projectiles::Rules),
    /// Complete parameters for every connected selected owner.
    Connected(Box<super::connected::Rules>),
    /// Scoped current Health rules.
    HealthTail(samus_health::Rules),
    /// The existing movement model's complete rules.
    Movement(samus_movement::Rules),
    /// Existing Movement rules; the Contact owner uses fixed transport rules.
    MovementTouchdown(samus_movement::Rules),
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
}

/// Exact saved context; restoring with other content or rules is refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Context {
    pub content: ProductionContent,
    pub rules: ProductionRules,
}

#[cfg(test)]
#[path = "tests/missing_selected_pose.rs"]
mod tests;
