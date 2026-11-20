//! Admission and completion of the five whole borrowed player owners.

use super::borrowed_player_preparation::{
    ActorUpdateCompletion, ActorView, CompletedInsideAndHudWeapons, CompletedPlayerSelection,
    ContactResetCompletion, ProjectilesCompletion, SamusInteractionCompletion,
    SpriteObjectsCompletion,
};
use super::player_continuation::Stage;
use super::{Caller, Content, Dependency, State};
use crate::production::timing::{self, WorkId};
use crate::production::{ProductionFault, Unsupported};
use data::timing::{Profile, TemplateEffect};
use types::Parts;

/// Private candidate products; only actual completion installs them in the caller.
enum Completion {
    Projectiles(ProjectilesCompletion),
    Interaction(SamusInteractionCompletion),
    Actors(ActorUpdateCompletion),
    Sprites(SpriteObjectsCompletion),
    Contact(ContactResetCompletion),
}

const fn missing() -> ProductionFault {
    ProductionFault::Unsupported(Unsupported::Frame(
        Dependency::PlayerContinuationPreparation,
    ))
}

fn remaining_sources(
    prepared: &super::Preparation,
) -> Result<
    (
        &super::borrowed_player_preparation::PlayerContinuationOrigins,
        &super::borrowed_player_preparation::PlayerContinuationOffers,
    ),
    ProductionFault,
> {
    let fault = missing();
    let prelude = &prepared.prelude;
    Ok((
        &prelude.player_continuation_current.as_ref().ok_or(fault)?.0,
        &prelude.player_continuation_offers.as_ref().ok_or(fault)?.0,
    ))
}

fn remaining_projectiles(
    offer: &super::borrowed_player_preparation::DeferredProjectilesReceipt,
    current: &super::borrowed_player_preparation::ProjectilesOrigin,
    inside_hud: &CompletedInsideAndHudWeapons<'_>,
    actual: Option<&ProjectilesCompletion>,
) -> Result<ProjectilesCompletion, ProductionFault> {
    let fault = missing();
    let expected_projectiles = offer
        .clone()
        .complete(current, inside_hud)
        .map_err(|_| fault)?;
    if actual.is_some_and(|actual| !actual.eq(&expected_projectiles)) {
        return Err(fault);
    }
    Ok(expected_projectiles)
}

impl State {
    /// Guard the complete retained causal prefix without beginning any owner work.
    pub(super) fn remaining_history(&self) -> Result<Stage, ProductionFault> {
        let fault = missing();
        let Caller::PlayerContinuation {
            prepared,
            completed,
            completion_work,
            controls,
            stage,
            remaining,
            remaining_work,
            motion_reset,
            ..
        } = &self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        let count = match stage {
            Stage::Projectiles => 0,
            Stage::MotionReset => 1,
            Stage::Interaction => 2,
            Stage::Actors => 3,
            Stage::Sprites => 4,
            Stage::Contact => 5,
            Stage::Run => 6,
        };
        let expected_work = [
            WorkId(17),
            WorkId(18),
            WorkId(19),
            WorkId(20),
            WorkId(21),
            WorkId(22),
        ];
        let expected = (count >= 1, count >= 3, count >= 4, count >= 5, count >= 6);
        let actual = (
            remaining.0.projectiles.is_some(),
            remaining.0.interaction.is_some(),
            remaining.0.actors.is_some(),
            remaining.0.sprites.is_some(),
            remaining.0.contact.is_some(),
        );
        if completion_work.as_slice() != [WorkId(12), WorkId(13), WorkId(14)]
            || completed.0.pose_input.is_none()
            || completed.0.suit_palette.is_none()
            || completed.0.environment.is_none()
            || self
                .inside_completed
                .as_ref()
                .is_none_or(|inside| inside.work != WorkId(15))
            || controls.work != WorkId(16)
            || !matches!(self.hud.controls(), Some(interface_hud::Observation::Completed(answer)) if answer.eq(&controls.answer.0))
            || Some(remaining_work.as_slice()) != expected_work.get(..count)
            || actual != expected
            || motion_reset.is_some() != (count >= 2)
            || motion_reset
                .as_ref()
                .is_some_and(|motion| motion.work != WorkId(18))
            || prepared.gameplay.is_none()
            || prepared.actors.is_none()
            || prepared.prelude.player_continuation_current.is_none()
            || prepared.prelude.player_continuation_offers.is_none()
        {
            return Err(fault);
        }
        Ok(*stage)
    }

    /// Validate fresh actual owners and compute an unpublished candidate result.
    fn remaining_candidate(&self, content: &Content) -> Result<Completion, ProductionFault> {
        let stage = self.remaining_history()?;
        let fault = missing();
        let Caller::PlayerContinuation {
            prepared,
            controls,
            remaining,
            motion_reset,
            ..
        } = &self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        let (current, offers) = remaining_sources(prepared)?;
        let inside_hud = CompletedInsideAndHudWeapons {
            inside: self.inside_completed.as_ref().ok_or(fault)?,
            controls,
        };
        let expected_projectiles = remaining_projectiles(
            &offers.projectiles,
            &current.projectiles,
            &inside_hud,
            remaining.0.projectiles.as_ref(),
        )?;
        if stage == Stage::Projectiles {
            return Ok(Completion::Projectiles(expected_projectiles));
        }
        if matches!(stage, Stage::MotionReset | Stage::Run) {
            return Err(ProductionFault::WrongScope);
        }
        let selected = CompletedPlayerSelection {
            inside_hud,
            motion_reset: motion_reset.as_ref().ok_or(fault)?,
        };
        if !self.movement.state().eq(&selected.motion_reset.previous.0) {
            return Err(fault);
        }
        let expected_interaction = offers
            .interaction
            .clone()
            .complete(
                &current.interaction,
                &selected,
                remaining.0.projectiles.as_ref().ok_or(fault)?,
            )
            .map_err(|_| fault)?;
        if remaining
            .0
            .interaction
            .as_ref()
            .is_some_and(|actual| !actual.eq(&expected_interaction))
        {
            return Err(fault);
        }
        let complete_actors = || {
            self.remaining_actors(
                content,
                prepared,
                &selected,
                &current.actors,
                &offers.actors,
                remaining.0.interaction.as_ref().ok_or(fault)?,
            )
        };
        if let Some(actual) = &remaining.0.actors
            && !actual.eq(&complete_actors()?)
        {
            return Err(fault);
        }
        let complete_sprites = || {
            offers
                .sprites
                .clone()
                .complete(&current.sprites, remaining.0.actors.as_ref().ok_or(fault)?)
                .map_err(|_| fault)
        };
        if let Some(actual) = &remaining.0.sprites
            && !actual.eq(&complete_sprites()?)
        {
            return Err(fault);
        }
        match stage {
            Stage::Interaction => Ok(Completion::Interaction(expected_interaction)),
            Stage::Actors => complete_actors().map(Completion::Actors),
            Stage::Sprites => complete_sprites().map(Completion::Sprites),
            Stage::Contact => offers
                .contact
                .clone()
                .complete(
                    current.contact,
                    remaining.0.actors.as_ref().ok_or(fault)?,
                    remaining.0.sprites.as_ref().ok_or(fault)?,
                )
                .map(Completion::Contact)
                .map_err(|_| fault),
            Stage::Projectiles | Stage::MotionReset | Stage::Run => {
                Err(ProductionFault::WrongScope)
            }
        }
    }

    fn remaining_actors(
        &self,
        content: &Content,
        prepared: &super::Preparation,
        selected: &CompletedPlayerSelection<'_>,
        current: &super::borrowed_player_preparation::ActorUnselectedOrigin,
        offer: &super::borrowed_player_preparation::DeferredActorUpdateReceipt,
        interaction: &SamusInteractionCompletion,
    ) -> Result<ActorUpdateCompletion, ProductionFault> {
        let fault = missing();
        let position = self.camera.0.position();
        let view = ActorView {
            samus: self.movement.state(),
            content: &content.movement,
            camera: types::Pair {
                x: position.x.pixel().0,
                y: position.y.pixel().0,
            },
        };
        offer
            .clone()
            .complete(
                current,
                selected,
                view,
                prepared.actors.as_ref().ok_or(fault)?,
                interaction,
            )
            .map_err(|_| fault)
    }

    /// Bind only the full no-device-effect envelope after semantic admission succeeds.
    fn remaining_profile(content: &Content, stage: Stage) -> Result<Profile, ProductionFault> {
        let template = content
            .player_continuation
            .as_ref()
            .ok_or_else(missing)?
            .template(stage)
            .ok_or(ProductionFault::WrongScope)?;
        if template
            .runs
            .iter()
            .any(|run| run.effect != TemplateEffect::None)
        {
            return Err(ProductionFault::FrameHardware(
                timing::Fault::InvalidProfile,
            ));
        }
        template
            .bind(&[])
            .map_err(|_| ProductionFault::FrameHardware(timing::Fault::InvalidProfile))
    }

    /// Admit one complete borrowed opportunity before beginning its physical bill.
    pub(super) fn begin_remaining_player(
        &mut self,
        content: &Content,
    ) -> Result<(), ProductionFault> {
        let Caller::PlayerContinuation {
            charging: false, ..
        } = self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        let stage = self.remaining_history()?;
        let _ = self.remaining_candidate(content)?;
        let profile = Self::remaining_profile(content, stage)?;
        self.timing
            .begin(stage.identity(), profile)
            .map_err(ProductionFault::FrameHardware)?;
        let Caller::PlayerContinuation { charging, .. } = &mut self.caller else {
            return Err(ProductionFault::WrongScope);
        };
        *charging = true;
        Ok(())
    }

    /// Recheck the current whole owner and retained bill across pauses and restoration.
    pub(super) fn validate_remaining_player(&self, content: &Content) -> Result<(), super::Fault> {
        if !matches!(self.caller, Caller::PlayerContinuation { .. }) {
            return Ok(());
        }
        let stage = self
            .remaining_history()
            .map_err(|_| super::Fault::InvalidPlayerEntry)?;
        if matches!(stage, Stage::MotionReset | Stage::Run) {
            return Ok(());
        }
        let Caller::PlayerContinuation { charging, .. } = &self.caller else {
            return Err(super::Fault::InvalidPlayerEntry);
        };
        self.remaining_candidate(content)
            .map_err(|_| super::Fault::InvalidPlayerEntry)?;
        let profile = Self::remaining_profile(content, stage)
            .map_err(|_| super::Fault::InvalidPlayerEntry)?;
        let retained: Vec<_> = self
            .timing
            .work
            .iter()
            .filter(|work| work.kind == timing::WorkKind::Semantic(stage.identity()))
            .collect();
        if retained.len() != usize::from(*charging)
            || retained.first().is_some_and(|work| work.profile != profile)
        {
            return Err(super::Fault::InvalidPlayerEntry);
        }
        Ok(())
    }

    /// Publish exactly one borrowed result only at its actual returned work identity.
    pub(super) fn complete_remaining_player(
        &mut self,
        work: WorkId,
        content: &Content,
    ) -> Result<(), ProductionFault> {
        let Caller::PlayerContinuation {
            charging: true,
            stage,
            ..
        } = &self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        if work != stage.identity() {
            return Err(missing());
        }
        let candidate = self.remaining_candidate(content)?;
        let _ = Self::remaining_profile(content, *stage)?;
        let Caller::PlayerContinuation {
            remaining,
            remaining_work,
            charging,
            stage,
            ..
        } = &mut self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        match candidate {
            Completion::Projectiles(result) => remaining.0.projectiles = Some(result),
            Completion::Interaction(result) => remaining.0.interaction = Some(result),
            Completion::Actors(result) => {
                if let Some(owner) = &mut self.health {
                    owner.actor_completed(&result).map_err(|_| {
                        ProductionFault::Unsupported(crate::production::Unsupported::Frame(
                            super::Dependency::HealthCurrentInputs,
                        ))
                    })?;
                }
                remaining.0.actors = Some(result);
            }
            Completion::Sprites(result) => remaining.0.sprites = Some(result),
            Completion::Contact(result) => remaining.0.contact = Some(result),
        }
        remaining_work.push(work);
        *stage = stage.next().ok_or(ProductionFault::WrongScope)?;
        *charging = false;
        Ok(())
    }
}
