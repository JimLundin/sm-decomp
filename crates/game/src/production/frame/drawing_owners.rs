//! Typed current drawing neighbours and actual completed collection joins.

use super::{State, borrowed_player_preparation as preparation, camera_stage, rendering_stage};
use canonical::Canonical;

/// Membership meaning decoded at the origin edge; native IDs are not retained.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Membership {
    /// No member occupies this physical slot.
    Absent,
    /// A member requires a separately qualified drawing contribution.
    Present,
}

/// Remaining instruction time owned by a trail or atmospheric animation.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Remaining(pub u16);

/// Borrowed current collections not owned by the selected S2 collection updates.
/// Enemy-projectile views all use the same sole eighteen physical cells.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Collections {
    /// Current animation/collection dispatch settings.
    pub animated_tiles_enabled: bool,
    /// Current enemy-projectile dispatch setting.
    pub enemy_projectiles_enabled: bool,
    /// Current PLM dispatch setting.
    pub plm_enabled: bool,
    /// Sole enemy-projectile membership owner, shared by all four traversals.
    pub enemy_projectiles: [Membership; 18],
    /// Current PLM membership owner in physical slot order.
    pub plm: [Membership; 40],
    /// Current animated-tile membership owner.
    pub animated_tiles: [Membership; 6],
    /// Current atmospheric animation activity owner.
    pub atmospheric: [Membership; 4],
    /// Actual left trail instruction ticks, preserving physical identity.
    pub left_trails: [Remaining; 18],
    /// Actual right trail instruction ticks, preserving physical identity.
    pub right_trails: [Remaining; 18],
    /// Actual initial ordered layer queues, before completed Actor drawing dispositions.
    pub actor_layers: [Vec<u8>; 8],
}

/// Current cannon animation and eligibility, independent of selected Samus sprites.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cannon {
    /// Current weapon-charge elapsed ticks.
    pub charge_ticks: u16,
    /// Current weapon-charge sound resume activity.
    pub resume_charge_sound: bool,
    /// Actual open disposition.
    pub open: bool,
    /// Actual animation frame.
    pub frame: u16,
    /// Actual opening/closing activity.
    pub opening_closing: bool,
}

/// Scene settings that select the drawing path.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SceneDrawing {
    /// Current normal drawing opportunity; false requires another aggregate.
    pub normal: bool,
    /// Current debug rendering activity.
    pub debug: bool,
    /// Current ordinary room-rendering rotation activity.
    pub rotating: bool,
}

/// Speed echo activity consumed by sprite drawing.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpeedEchoes {
    /// Current speed-echo merge activity.
    pub merging: bool,
    /// Current speed-echo sound activity.
    pub sound_playing: bool,
}

/// Actor graphics requests and the current sprite interaction setting.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActorDrawing {
    /// Current actor background upload request.
    pub background_upload: bool,
    /// Current actor background graphics hook activity; native hook addresses are absent.
    pub background_hook: bool,
    /// Current sprite interaction setting.
    pub sprite_interactions_disabled: bool,
}

/// Concrete unchanged neighbouring ports admitted from checked origin ownership.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Origins {
    /// Actor graphics requests and the current sprite interaction setting.
    pub actors: ActorDrawing,
    /// Speed echo activity consumed by sprite drawing.
    pub echoes: SpeedEchoes,
    /// Scene settings that select the drawing path.
    pub scene: SceneDrawing,
    /// Sole current missing collection owners.
    pub collections: Collections,
    /// Current cannon owner.
    pub cannon: Cannon,
    /// Actual retained simulation frame counter.
    pub frame_counter: u16,
    /// Current simulation freeze activity.
    pub frozen: bool,
    /// Actual current invincibility timer, shared by both named source aliases.
    pub invincibility: Remaining,
    /// Actual current knockback timer.
    pub knockback: Remaining,
    /// Current screen-shake timer.
    pub shake: Remaining,
    /// Current equipped Hyper Beam disposition.
    pub hyper_beam: bool,
    /// Current world-event timebomb activity.
    pub timebomb: bool,
    /// Current power-bomb radius.
    pub power_bomb_radius: u16,
    /// Actual previous health owned by draw history.
    pub previous_health: u16,
    /// Current projectile population owned separately from instruction dispositions.
    pub projectile_population: u16,
    /// Actual configured jump button encoding, independent of physical source addresses.
    pub jump_buttons: u16,
}

/// Complete selected producers consumed at the actual drawing opportunity.
pub struct Completed<'a> {
    /// Actual completed S1 membership retained by the caller and consumed by Actor20.
    pub membership: &'a super::borrowed_preparation::ActorEligibilityReceipt,
    /// Completed S2 projectile instruction owner.
    pub projectiles: &'a preparation::ProjectilesCompletion,
    /// Completed S2 sprite instruction owner.
    pub sprites: &'a preparation::SpriteObjectsCompletion,
    /// Completed Actor owner, including actual drawing dispositions.
    pub actors: &'a preparation::ActorUpdateCompletion,
    /// Completed Camera owner; Offered/Charging cameras cannot be read.
    pub camera: &'a camera_stage::Stage,
    /// Actual completed `MotionReset`, retaining its previous action authority.
    pub motion_reset: &'a super::MotionResetCompletion,
    /// Actual completed early HUD control observation.
    pub controls: &'a super::hud_stage::ControlsCompletion,
}

/// Immutable complete current join retained by the whole drawing operation.
/// The existing continuation pin stores actual receipts, never writable collection owners.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Current {
    origins: Origins,
    membership: super::borrowed_preparation::ActorEligibilityReceipt,
    collections: canonical::model::Model<preparation::PlayerContinuationCompletionsPin>,
    motion_reset: super::MotionResetCompletion,
    controls: super::hud_stage::ControlsCompletion,
    camera: camera_stage::Stage,
    premises: rendering_stage::Premises,
}

impl Current {
    /// Qualify every actual current drawing neighbour before retaining its receipt.
    /// # Errors
    /// Incomplete selected producers or a contribution outside the admitted family.
    pub fn choose(
        origins: &Origins,
        state: &State,
        completed: &Completed<'_>,
    ) -> Result<Self, Fault> {
        let premises = origins.premises(state, completed)?;
        Ok(Self {
            origins: origins.clone(),
            membership: completed.membership.clone(),
            collections: canonical::model::Model(Box::new(
                preparation::PlayerContinuationCompletions {
                    projectiles: Some(completed.projectiles.clone()),
                    actors: Some(completed.actors.clone()),
                    sprites: Some(completed.sprites.clone()),
                    interaction: None,
                    contact: None,
                },
            )),
            motion_reset: completed.motion_reset.clone(),
            controls: completed.controls.clone(),
            camera: completed.camera.clone(),
            premises,
        })
    }

    /// Actual qualified rendering premises belonging to this complete join.
    #[must_use]
    pub const fn premises(&self) -> rendering_stage::Premises {
        self.premises
    }

    /// Requalify the complete fresh join at the sole drawing return.
    /// # Errors
    /// A current owner, completion, alias or physical disposition changed.
    pub fn validate(
        &self,
        origins: &Origins,
        state: &State,
        completed: &Completed<'_>,
    ) -> Result<(), Fault> {
        if Self::choose(origins, state, completed)? != *self {
            return Err(Fault::Producers);
        }
        Ok(())
    }
}

/// Missing current authority or a nonempty drawing contribution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Required actual producers are incomplete or their identities changed.
    Producers,
    /// A current neighbour requires another qualified aggregate family.
    Unsupported,
}

impl Origins {
    /// Join fresh typed borrowed owners with actual selected collection results.
    /// Callers retain these sole ports across all preceding owners and update actual writers.
    /// # Errors
    /// Changed physical slot identity, missing Camera/HUD/preparation, or active contribution.
    pub fn premises(
        &self,
        state: &State,
        completed: &Completed<'_>,
    ) -> Result<rendering_stage::Premises, Fault> {
        use preparation::InstructionDisposition;
        let camera = completed
            .camera
            .viewport(&state.camera)
            .ok_or(Fault::Producers)?;
        if state.movement.finished().is_none()
            || completed.projectiles.terminal.slots != completed.projectiles.dependency.slots
            || completed.sprites.terminal.slots != completed.sprites.dependency.slots
            || completed.actors.solid_boxes().is_err()
            || completed.motion_reset.work != crate::production::timing::WorkId(18)
            || state.prepared_action.as_ref().map(|action| action.0)
                != Some(completed.motion_reset.previous.0.pose.action)
            || !matches!(state.hud.controls(), Some(interface_hud::Observation::Completed(answer)) if *answer == completed.controls.answer.0)
        {
            return Err(Fault::Producers);
        }
        for (index, slot) in completed.projectiles.terminal.slots.iter().enumerate() {
            if usize::from(slot.identity.0) != index {
                return Err(Fault::Producers);
            }
            if slot.instruction != InstructionDisposition::Empty {
                return Err(Fault::Unsupported);
            }
        }
        for (index, slot) in completed.sprites.terminal.slots.iter().enumerate() {
            if usize::from(slot.identity.0) != index {
                return Err(Fault::Producers);
            }
            if slot.instruction != InstructionDisposition::Empty {
                return Err(Fault::Unsupported);
            }
        }
        let shake = if let Some(quake) = state.room_quake.current() {
            use types::Parts;

            if quake.remaining.0 != self.shake.0 {
                return Err(Fault::Producers);
            }
            let displacement = world_quake::displacement(&quake).map_err(|_| Fault::Unsupported)?;
            [
                displacement.x.pixel().0.cast_signed(),
                displacement.y.pixel().0.cast_signed(),
            ]
        } else if self.shake.0 == 0 {
            [0, 0]
        } else {
            return Err(Fault::Unsupported);
        };
        let cannon = state.appearance.cannon.as_ref().ok_or(Fault::Producers)?;
        self.normal_visibility(cannon.0)?;
        // The optional layer entry belongs to the completed ActorUpdate, before Camera.
        // Its coordinates are projected only against the later returned Camera.
        let actors = &completed.actors.actors;
        if completed.membership.active.len() != actors.len()
            || completed.membership.interactive != completed.actors.interactive
        {
            return Err(Fault::Producers);
        }
        for (actor, identity) in completed
            .actors
            .actors
            .iter()
            .zip(&completed.membership.active)
        {
            if &actor.identity != identity {
                return Err(Fault::Producers);
            }
        }
        let mut shown = actors
            .iter()
            .filter(|actor| actor.view == preparation::ActorViewDisposition::WithinPaddedView);
        if shown.next().map(|actor| actor.identity) != completed.actors.visible_layer_two
            || shown.next().is_some()
        {
            return Err(Fault::Producers);
        }
        Ok(rendering_stage::Premises {
            camera,
            shake,
            normal_visible: true,
            no_additional_selected_pieces: true,
            unselected_draws_empty: true,
        })
    }

    fn normal_visibility(&self, readiness: samus_appearance::CannonDelay) -> Result<(), Fault> {
        if !self.scene.normal
            || self.scene.rotating
            || self.scene.debug
            || self.frozen
            // The qualified stowed-cannon work includes the ready selection check.
            || !readiness.may_follow()
            || self.cannon.open
            || self.cannon.opening_closing
            || self.cannon.frame != 0
            || self.cannon.charge_ticks != 0
            || self.invincibility.0 != 0
            || self.knockback.0 != 0
            || self.hyper_beam
            || self.echoes.merging
            || self.echoes.sound_playing
            || self.actors.background_upload
            || self.actors.background_hook
            || self.power_bomb_radius != 0
            || self.cannon.resume_charge_sound
            || self.projectile_population != 0
            || self.actors.sprite_interactions_disabled
            || self
                .collections
                .enemy_projectiles
                .iter()
                .chain(&self.collections.plm)
                .chain(&self.collections.animated_tiles)
                .chain(&self.collections.atmospheric)
                .any(|member| *member != Membership::Absent)
            || self
                .collections
                .left_trails
                .iter()
                .chain(&self.collections.right_trails)
                .any(|ticks| ticks.0 != 0)
            || self
                .collections
                .actor_layers
                .iter()
                .any(|layer| !layer.is_empty())
        {
            return Err(Fault::Unsupported);
        }
        Ok(())
    }
}
