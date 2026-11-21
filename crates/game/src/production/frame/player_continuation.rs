//! Ordered remaining player owners and their complete physical envelopes.

use super::borrowed_player_preparation::{
    ActorScene, OrdinaryActorContact, ProjectilesOrigin, SamusInteractionOrigin,
};
use crate::production::timing::WorkId;
use canonical::Canonical;
use data::timing::Template;

/// Carried projectile slots, clocks and Samus interaction modes.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct RecurringPlayer {
    /// Current physical projectile collection.
    pub projectiles: ProjectilesOrigin,
    /// Current interaction activity and enables.
    pub interaction: SamusInteractionOrigin,
    /// Current ordinary Ceres door actors.
    pub doors: super::actor_doors::Doors,
    /// Actual door state consumed by this interval's completed recurring poll.
    pub completed_poll_input: Option<super::actor_doors::Doors>,
    /// Current pre-escape actor scene.
    pub actor_scene: ActorScene,
    /// Current ordinary contact guards, independent of door waits and outputs.
    pub contact: OrdinaryActorContact,
}

/// Actual owner following completed Inside and early HUD controls.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    /// Projectile instruction owners.
    Projectiles,
    /// Current motion preparation and transient reset bindings.
    MotionReset,
    /// Current Samus interaction opportunity.
    Interaction,
    /// Full actual actor update.
    Actors,
    /// Sprite instruction owners.
    Sprites,
    /// Contact preparation.
    Contact,
    /// All S2 owners completed; actual ordinary Run is next.
    Run,
}

impl Stage {
    /// Matching finite physical operation identity.
    #[must_use]
    pub const fn identity(self) -> WorkId {
        WorkId(match self {
            Self::Projectiles => 17,
            Self::MotionReset => 18,
            Self::Interaction => 19,
            Self::Actors => 20,
            Self::Sprites => 21,
            Self::Contact => 22,
            Self::Run => 23,
        })
    }

    /// Next actual owner after the returned completion is published.
    #[must_use]
    pub const fn next(self) -> Option<Self> {
        Some(match self {
            Self::Projectiles => Self::MotionReset,
            Self::MotionReset => Self::Interaction,
            Self::Interaction => Self::Actors,
            Self::Actors => Self::Sprites,
            Self::Sprites => Self::Contact,
            Self::Contact => Self::Run,
            Self::Run => return None,
        })
    }
}

/// Whole physical envelopes; current semantic dependencies choose each one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Content {
    /// Whole projectile instruction scan.
    pub projectiles: Template,
    /// Whole current interaction owner.
    pub interaction: Template,
    /// Whole actor update, including all actual ordered members.
    pub actors: Template,
    /// Whole `ActorUpdate` when both carried doors poll without changing figure.
    pub recurring_actors: Option<Template>,
    /// Whole sprite instruction scan.
    pub sprites: Template,
    /// Whole contact preparation.
    pub contact: Template,
}

impl Content {
    /// Physical envelope for the current borrowed owner; other stages qualify separately.
    #[must_use]
    pub const fn template(&self, stage: Stage) -> Option<&Template> {
        match stage {
            Stage::Projectiles => Some(&self.projectiles),
            Stage::Interaction => Some(&self.interaction),
            Stage::Actors => Some(&self.actors),
            Stage::Sprites => Some(&self.sprites),
            Stage::Contact => Some(&self.contact),
            Stage::MotionReset | Stage::Run => None,
        }
    }
}
