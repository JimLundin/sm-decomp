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
    Actors(ActorUpdateCompletion, Option<super::actor_doors::Doors>),
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
    let prelude = prepared.prelude.origin().ok_or(fault)?;
    Ok((
        &prelude.player_continuation_current.as_ref().ok_or(fault)?.0,
        &prelude.player_continuation_offers.as_ref().ok_or(fault)?.0,
    ))
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
            || !self.has_remaining_player_source(prepared)
        {
            return Err(fault);
        }
        Ok(*stage)
    }

    pub(super) fn has_remaining_player_source(&self, prepared: &super::Preparation) -> bool {
        if prepared.prelude.origin().is_some() {
            remaining_sources(prepared).is_ok()
        } else {
            self.recurring_player.is_some()
        }
    }

    pub(super) fn current_projectiles(
        &self,
        prepared: &super::Preparation,
        selected: &CompletedInsideAndHudWeapons<'_>,
        actual: Option<&ProjectilesCompletion>,
    ) -> Result<ProjectilesCompletion, ProductionFault> {
        let fault = missing();
        let Caller::PlayerContinuation { completed, .. } = &self.caller else {
            return Err(ProductionFault::WrongScope);
        };
        let pose = &completed.0.pose_input.as_ref().ok_or(fault)?.dependency;
        if pose.held.0 & pose.bindings.shoot.0 != 0
            && self.hud.state().selection == interface_hud::Selection::None
            && self
                .appearance
                .cannon
                .as_ref()
                .is_some_and(|cannon| cannon.0.may_follow())
        {
            return Err(ProductionFault::Unsupported(Unsupported::Frame(
                Dependency::WeaponPrePreparation,
            )));
        }
        let expected = if prepared.prelude.origin().is_some() {
            let (current, offers) = remaining_sources(prepared)?;
            offers
                .projectiles
                .clone()
                .complete(&current.projectiles, selected)
        } else {
            self.recurring_player
                .as_ref()
                .ok_or(fault)?
                .projectiles
                .complete_quiet(selected)
        }
        .map_err(|_| fault)?;
        if actual.is_some_and(|actual| actual != &expected) {
            return Err(fault);
        }
        Ok(expected)
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
        let inside_hud = CompletedInsideAndHudWeapons {
            inside: self.inside_completed.as_ref().ok_or(fault)?,
            controls,
        };
        let expected_projectiles =
            self.current_projectiles(prepared, &inside_hud, remaining.0.projectiles.as_ref())?;
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
        let projectiles = remaining.0.projectiles.as_ref().ok_or(fault)?;
        let expected_interaction = if prepared.prelude.origin().is_some() {
            let (current, offers) = remaining_sources(prepared)?;
            offers
                .interaction
                .clone()
                .complete(&current.interaction, &selected, projectiles)
        } else {
            self.recurring_player
                .as_ref()
                .ok_or(fault)?
                .interaction
                .complete_quiet(&selected, projectiles)
        }
        .map_err(|_| fault)?;
        if remaining
            .0
            .interaction
            .as_ref()
            .is_some_and(|actual| !actual.eq(&expected_interaction))
        {
            return Err(fault);
        }
        if stage == Stage::Interaction {
            return Ok(Completion::Interaction(expected_interaction));
        }
        if prepared.prelude.origin().is_none() {
            return self.remaining_recurring_actors(
                content,
                stage,
                prepared,
                &selected,
                &expected_interaction,
                &remaining.0,
            );
        }
        self.remaining_origin_candidate(content, stage, prepared, &selected, &remaining.0)
    }

    fn remaining_recurring_actors(
        &self,
        content: &Content,
        stage: Stage,
        prepared: &super::Preparation,
        selected: &CompletedPlayerSelection<'_>,
        interaction: &SamusInteractionCompletion,
        remaining: &super::borrowed_player_preparation::PlayerContinuationCompletions,
    ) -> Result<Completion, ProductionFault> {
        let fault = missing();
        let owners = self.recurring_player.as_ref().ok_or(fault)?;
        let actual = remaining.actors.as_ref();
        let input = match (stage, actual, owners.completed_door_input) {
            (Stage::Actors, None, None) => owners.doors,
            (Stage::Sprites | Stage::Contact, Some(_), Some(input)) => input,
            _ => return Err(fault),
        };
        let (next, result) =
            self.complete_recurring_actor(content, prepared, selected, interaction, input)?;
        if let Some(actual) = actual {
            let health_holds = match (&self.health, &content.health) {
                (Some(owner), Some(admission)) => owner.holds_recurring_actor(admission, actual),
                (None, None) => true,
                _ => false,
            };
            if &result != actual || next != owners.doors || !health_holds {
                return Err(fault);
            }
            let sprites = owners.sprites.complete_quiet(actual).map_err(|_| {
                ProductionFault::Unsupported(Unsupported::Frame(
                    Dependency::SpriteObjectsPreparation,
                ))
            })?;
            if stage == Stage::Sprites {
                return Ok(Completion::Sprites(sprites));
            }
            if remaining.sprites.as_ref() != Some(&sprites) {
                return Err(fault);
            }
            return owners
                .contact_reset
                .complete_reset(actual, &sprites)
                .map(Completion::Contact)
                .map_err(|_| fault);
        }
        Ok(Completion::Actors(result, Some(next)))
    }

    fn complete_recurring_actor(
        &self,
        content: &Content,
        prepared: &super::Preparation,
        selected: &CompletedPlayerSelection<'_>,
        interaction: &SamusInteractionCompletion,
        doors: super::actor_doors::Doors,
    ) -> Result<(super::actor_doors::Doors, ActorUpdateCompletion), ProductionFault> {
        use super::borrowed_player_preparation::ActorUnselectedOrigin;
        use samus_movement::suspended::MovementDecision;
        let fault = missing();
        let owners = self.recurring_player.as_ref().ok_or(fault)?;
        let membership = prepared.actors.as_ref().ok_or(fault)?;
        if self
            .actor_eligibility
            .as_ref()
            .and_then(super::borrowed_preparation::ActorEligibilityReceipt::marked_offscreen)
            .as_ref()
            != Some(membership)
        {
            return Err(fault);
        }
        let scene = prepared.gameplay.as_ref().ok_or(fault)?.scene;
        Self::recurring_actor_profile(content, doors)?;
        if self.scene != Some(scene)
            || scene != super::borrowed_gameplay::SceneController::MainGameplay
            || self
                .room_quake
                .current()
                .is_none_or(|quake| quake.is_active())
            || owners.contact.activity != interaction.terminal.activity
            || owners.contact.sprite_interaction != interaction.terminal.sprite_interaction
        {
            return Err(fault);
        }
        let samus = &selected.motion_reset.previous.0;
        let radius = content.movement.radius(samus.pose).ok_or(fault)?;
        let inside = selected.inside_hud.inside;
        let Some(MovementDecision::InsideRequest(bounds)) = inside.receipt.decisions.0.first()
        else {
            return Err(fault);
        };
        let her = types::Pair {
            x: samus.position.x.pixel(),
            y: samus.position.y.pixel(),
        };
        if bounds.at != samus.position
            || bounds.r != radius
            || inside.retained.state().pose != samus.pose
            || !doors.contact_misses(her.x, radius.x)
        {
            return Err(fault);
        }
        let camera = self.camera.0.position();
        let views = doors
            .views(types::Pair {
                x: camera.x.pixel().0,
                y: camera.y.pixel().0,
            })
            .ok_or(fault)?;
        let current = ActorUnselectedOrigin {
            actors: doors.current_actors(),
            clocks: owners.projectiles.clocks,
            scene: owners.actor_scene,
            shake_requested: false,
            contact: owners.contact,
        };
        doors
            .run(
                &current,
                views,
                membership,
                her,
                self.current_random().ok_or(fault)?,
            )
            .ok_or(fault)
    }

    fn remaining_origin_candidate(
        &self,
        content: &Content,
        stage: Stage,
        prepared: &super::Preparation,
        selected: &CompletedPlayerSelection<'_>,
        remaining: &super::borrowed_player_preparation::PlayerContinuationCompletions,
    ) -> Result<Completion, ProductionFault> {
        let fault = missing();
        let (current, offers) = remaining_sources(prepared)?;
        let input = match (&self.recurring_player, &remaining.actors) {
            (Some(owners), None) if owners.completed_door_input.is_none() => Some(owners.doors),
            (Some(owners), Some(_)) => Some(owners.completed_door_input.ok_or(fault)?),
            (None, _) => None,
            _ => return Err(fault),
        };
        let complete_actors = || {
            self.remaining_actors(
                content,
                prepared,
                selected,
                remaining.interaction.as_ref().ok_or(fault)?,
                input,
            )
        };
        if let Some(actual) = &remaining.actors {
            let (expected, next) = complete_actors()?;
            if actual != &expected
                || next != self.recurring_player.as_ref().map(|owner| owner.doors)
            {
                return Err(fault);
            }
        }
        let complete_sprites = || {
            offers
                .sprites
                .clone()
                .complete(&current.sprites, remaining.actors.as_ref().ok_or(fault)?)
                .map_err(|_| fault)
        };
        if let Some(actual) = &remaining.sprites
            && !actual.eq(&complete_sprites()?)
        {
            return Err(fault);
        }
        match stage {
            Stage::Actors => {
                complete_actors().map(|(result, doors)| Completion::Actors(result, doors))
            }
            Stage::Sprites => complete_sprites().map(Completion::Sprites),
            Stage::Contact => offers
                .contact
                .clone()
                .complete(
                    current.contact,
                    remaining.actors.as_ref().ok_or(fault)?,
                    remaining.sprites.as_ref().ok_or(fault)?,
                )
                .map(Completion::Contact)
                .map_err(|_| fault),
            Stage::Projectiles | Stage::MotionReset | Stage::Interaction | Stage::Run => {
                Err(ProductionFault::WrongScope)
            }
        }
    }

    fn remaining_actors(
        &self,
        content: &Content,
        prepared: &super::Preparation,
        selected: &CompletedPlayerSelection<'_>,
        interaction: &SamusInteractionCompletion,
        doors: Option<super::actor_doors::Doors>,
    ) -> Result<(ActorUpdateCompletion, Option<super::actor_doors::Doors>), ProductionFault> {
        let fault = missing();
        let (current, offers) = remaining_sources(prepared)?;
        let current = &current.actors;
        let offer = &offers.actors;
        let position = self.camera.0.position();
        let view = ActorView {
            samus: &selected.motion_reset.previous.0,
            content: &content.movement,
            camera: types::Pair {
                x: position.x.pixel().0,
                y: position.y.pixel().0,
            },
        };
        let membership = prepared.actors.as_ref().ok_or(fault)?;
        if let Some(doors) = doors {
            if doors.path() != Some(super::actor_doors::Path::Waiting)
                || doors.current_actors() != current.actors
            {
                return Err(fault);
            }
            offer
                .validate_dependencies(current, selected, view, membership, interaction)
                .map_err(|_| fault)?;
            let [x, y] = [view.samus.position.x.pixel(), view.samus.position.y.pixel()];
            let (next, completion) = doors
                .run(
                    current,
                    offer.required.views,
                    membership,
                    types::Pair { x, y },
                    self.current_random().ok_or(fault)?,
                )
                .ok_or(fault)?;
            return Ok((completion, Some(next)));
        }
        offer
            .clone()
            .complete(current, selected, view, membership, interaction)
            .map(|result| (result, None))
            .map_err(|_| fault)
    }

    /// Bind only the full no-device-effect envelope after semantic admission succeeds.
    fn recurring_actor_profile(
        content: &Content,
        doors: super::actor_doors::Doors,
    ) -> Result<&data::timing::Template, ProductionFault> {
        let profiles = content.player_continuation.as_ref().ok_or_else(missing)?;
        match doors.path().ok_or_else(missing)? {
            super::actor_doors::Path::Waiting => profiles.recurring_waiting_actors.as_ref(),
            super::actor_doors::Path::Polling => profiles.recurring_actors.as_ref(),
        }
        .ok_or_else(missing)
    }

    fn remaining_profile(
        &self,
        content: &Content,
        stage: Stage,
    ) -> Result<Profile, ProductionFault> {
        let profiles = content.player_continuation.as_ref().ok_or_else(missing)?;
        let template = if stage == Stage::Actors
            && matches!(&self.caller, Caller::PlayerContinuation { prepared, .. }
                if prepared.prelude.origin().is_none())
        {
            let doors = self.recurring_player.as_ref().ok_or_else(missing)?.doors;
            Self::recurring_actor_profile(content, doors)?
        } else {
            profiles
                .template(stage)
                .ok_or(ProductionFault::WrongScope)?
        };
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
        let profile = self.remaining_profile(content, stage)?;
        self.timing
            .begin(stage.identity(), profile)
            .map_err(ProductionFault::FrameHardware)?;
        let Caller::PlayerContinuation { charging, .. } = &mut self.caller else {
            return Err(ProductionFault::WrongScope);
        };
        *charging = true;
        Ok(())
    }

    pub(crate) fn validate_committed_actors(&self, content: &Content) -> Result<(), super::Fault> {
        let Caller::PlayerContinuation {
            prepared,
            controls,
            motion_reset,
            remaining,
            ..
        } = &self.caller
        else {
            return Ok(());
        };
        let Some(actual) = remaining.0.actors.as_ref() else {
            return Ok(());
        };
        let original = prepared.prelude.origin().is_some();
        if original && self.recurring_player.is_none() {
            return Ok(());
        }
        let fault = || super::Fault::InvalidPlayerEntry;
        let owners = self.recurring_player.as_ref().ok_or_else(fault)?;
        let selected = CompletedPlayerSelection {
            inside_hud: CompletedInsideAndHudWeapons {
                inside: self.inside_completed.as_ref().ok_or_else(fault)?,
                controls,
            },
            motion_reset: motion_reset.as_ref().ok_or_else(fault)?,
        };
        let interaction = remaining.0.interaction.as_ref().ok_or_else(fault)?;
        let input = owners.completed_door_input.ok_or_else(fault)?;
        let (next, result) = if original {
            let (result, next) = self
                .remaining_actors(content, prepared, &selected, interaction, Some(input))
                .map_err(|_| fault())?;
            (next.ok_or_else(fault)?, result)
        } else {
            self.complete_recurring_actor(content, prepared, &selected, interaction, input)
                .map_err(|_| fault())?
        };
        let health_holds = match (&self.health, &content.health) {
            (Some(owner), Some(admission)) if original => {
                owner.validate_origin(admission).is_ok() && owner.actor() == Some(actual)
            }
            (Some(owner), Some(admission)) => owner.holds_recurring_actor(admission, actual),
            (None, None) => true,
            _ => false,
        };
        if &result != actual || next != owners.doors || !health_holds {
            return Err(fault());
        }
        if original {
            return Ok(());
        }
        if let Some(sprites) = remaining.0.sprites.as_ref() {
            if &owners.sprites.complete_quiet(actual).map_err(|_| fault())? != sprites {
                return Err(fault());
            }
            if let Some(contact) = remaining.0.contact.as_ref()
                && (contact.execution_counter != owners.contact_reset.execution_counter
                    || &contact
                        .dependency
                        .complete_reset(actual, sprites)
                        .map_err(|_| fault())?
                        != contact)
            {
                return Err(fault());
            }
        }
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
        match self.remaining_candidate(content) {
            Ok(_) => {}
            Err(ProductionFault::Unsupported(Unsupported::Frame(
                Dependency::SpriteObjectsPreparation,
            ))) if !charging => return Ok(()),
            Err(_) => return Err(super::Fault::InvalidPlayerEntry),
        }
        let profile = self
            .remaining_profile(content, stage)
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
        let _ = self.remaining_profile(content, *stage)?;
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
            Completion::Actors(result, next) => {
                if let Some(next) = next {
                    let owners = self.recurring_player.as_mut().ok_or_else(missing)?;
                    owners.completed_door_input = Some(owners.doors);
                    owners.doors = next;
                }
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
            Completion::Contact(result) => {
                if let Some(owners) = &mut self.recurring_player {
                    owners.contact_reset.execution_counter = result.execution_counter;
                }
                remaining.0.contact = Some(result);
            }
        }
        remaining_work.push(work);
        *stage = stage.next().ok_or(ProductionFault::WrongScope)?;
        *charging = false;
        Ok(())
    }
}
