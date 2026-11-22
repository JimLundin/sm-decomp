//! Normal Drawing after returned Health and Camera, with one live sprite builder.

use super::{
    Caller, Content, Dependency, State, appearance_caller, drawing_owners,
    frame_history::{EnergyChange, EnergyRead},
    rendering_stage::{NormalDrawingReceipt, ShownStep},
};
use crate::production::{ProductionFault, Unsupported, timing};
use canonical::Canonical;
use data::timing::{Profile, Template, TemplateEffect};
use std::borrow::Cow;

pub(super) const BRIDGE: timing::WorkId = timing::WorkId(1322);
pub(super) const DRAWING: timing::WorkId = appearance_caller::Kind::Drawing.identity();

pub use super::running_draw_timing::{DrawingPortions, PieceProfiles, RecurringProfiles};

/// Complete physical bridge and selected Draw aggregate with checked sprite content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Contract {
    bridge: Profile,
    drawing: Profile,
    decreased_energy: Option<DecreasedEnergy>,
    rendering: super::rendering_stage::Content,
    adjacent: Option<Adjacent>,
    recurring: Option<RecurringProfiles>,
    mixed_actor: Option<Profile>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DecreasedEnergy {
    profile: Profile,
    previous: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Adjacent {
    drawing: Profile,
    figure: samus_appearance::Figure,
    step: ShownStep,
}

impl Adjacent {
    fn accepts_picture(&self, receipt: &NormalDrawingReceipt) -> bool {
        receipt.figure.0 == self.figure
            && receipt.shown_step == self.step
            && receipt.first_byte == 0
            && receipt.pieces == 9
    }
}

impl Contract {
    /// Bind the admitted no-device-output envelope and actual rendering content.
    /// # Errors
    /// Device effects or unsupported physical work need another Drawing contract.
    pub fn new(
        bridge: &Template,
        drawing: &Template,
        rendering: super::rendering_stage::Content,
    ) -> Result<Self, ProductionFault> {
        Ok(Self {
            bridge: Self::bind(bridge)?,
            drawing: Self::bind(drawing)?,
            decreased_energy: None,
            rendering,
            adjacent: None,
            recurring: None,
            mixed_actor: None,
        })
    }

    /// Admit the distinct physical Drawing arm reached with decreased live energy.
    /// # Errors
    /// The exposure template includes unowned device effects or is invalid.
    pub fn with_decreased_energy(
        mut self,
        drawing: &Template,
        previous: u16,
    ) -> Result<Self, ProductionFault> {
        self.decreased_energy = Some(DecreasedEnergy {
            profile: Self::bind(drawing)?,
            previous,
        });
        Ok(self)
    }

    fn bind(template: &Template) -> Result<Profile, ProductionFault> {
        if template
            .runs
            .iter()
            .any(|run| run.effect != TemplateEffect::None)
        {
            return Err(missing());
        }
        template.bind(&[]).map_err(|_| missing())
    }

    /// Bind the whole checked Drawing envelope for an exact cannon-free picture.
    /// # Errors
    /// Rejects device effects, invalid work and missing imported pose bindings.
    pub fn admit_adjacent(
        &mut self,
        drawing: &Template,
        figure: samus_appearance::Figure,
        step: ShownStep,
    ) -> Result<(), ProductionFault> {
        self.adjacent = Some(Adjacent {
            drawing: Self::bind(drawing)?,
            figure,
            step,
        });
        Ok(())
    }

    /// Admit recurring physical appends selected by the actual running maps and piece sizes.
    pub fn admit_recurring(&mut self, profiles: RecurringProfiles) {
        self.recurring = Some(profiles);
    }

    /// Admit the single visible actor layer splice and both imported current door figures.
    /// # Errors
    /// The splice contains an incompatible hardware effect or invalid physical profile.
    pub fn admit_mixed_actor(
        &mut self,
        profile: &Template,
        art: super::rendering_stage::DoorArt,
    ) -> Result<(), ProductionFault> {
        self.mixed_actor = Some(Self::bind(profile)?);
        self.rendering.admit_door(art);
        Ok(())
    }

    fn drawing_profile(
        &self,
        state: &State,
        receipt: Option<&NormalDrawingReceipt>,
        read: EnergyRead,
    ) -> Result<Cow<'_, Profile>, ProductionFault> {
        if state.publications.original_interval() {
            let alternate = self.decreased_energy.as_ref();
            if alternate.is_some_and(|arm| read.previous != arm.previous) {
                return Err(missing());
            }
            return match read.change {
                EnergyChange::Held => Ok(Cow::Borrowed(&self.drawing)),
                EnergyChange::Decreased => alternate
                    .map(|arm| Cow::Borrowed(&arm.profile))
                    .ok_or_else(missing),
                EnergyChange::Increased => Err(missing()),
            };
        }
        if read.change != EnergyChange::Held {
            return Err(missing());
        }
        let (samus, outcome) = state.movement.finished().ok_or_else(missing)?;
        let figure = state.playing_figure.0;
        let receipt = receipt.ok_or_else(missing)?;
        if samus.pose != figure.pose
            || outcome.sprite != figure.sprite
            || receipt.figure.0 != figure
        {
            return Err(missing());
        }
        if let Some(recurring) = &self.recurring {
            return recurring
                .profile(&self.rendering, receipt, self.mixed_actor.as_ref())
                .map(Cow::Owned);
        }
        self.adjacent
            .as_ref()
            .filter(|alternate| alternate.accepts_picture(receipt))
            .map(|alternate| Cow::Borrowed(&alternate.drawing))
            .ok_or_else(missing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use canonical::model::Model;
    use types::{Action, Aim, Body, Facing, Gun, Legs, Pose, Sprite};

    #[test]
    fn adjacent_drawing_requires_the_qualified_step_of_the_same_figure() {
        let figure = samus_appearance::Figure {
            pose: Pose {
                body: Body::Upright,
                action: Action::Running,
                facing: Facing::Right,
                aim: Aim::Forward,
                gun: Gun::Stowed,
                legs: Legs::Striding,
            },
            sprite: Sprite::Plain,
        };
        let qualified = Adjacent {
            drawing: Profile { runs: Vec::new() },
            figure,
            step: ShownStep(1),
        };
        let mut receipt = NormalDrawingReceipt {
            figure: Model(figure),
            shown_step: qualified.step,
            centre: [0, 0],
            premises: super::super::rendering_stage::Premises {
                camera: [0, 0],
                shake: [0, 0],
                normal_visible: true,
                no_additional_selected_pieces: true,
                unselected_draws_empty: true,
            },
            first_byte: 0,
            pieces: 9,
        };
        assert!(qualified.accepts_picture(&receipt));
        receipt.shown_step = ShownStep(2);
        assert!(!qualified.accepts_picture(&receipt));
    }
}

#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
struct Pending {
    selected: appearance_caller::Stage,
    aggregate: appearance_caller::DrawingAggregate,
    builder: super::borrowed::SpriteBuilderOrigin,
    energy: EnergyRead,
}

#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
enum Progress {
    #[default]
    Pending,
    Bridge,
    DrawingReady,
    Charging(Box<Pending>),
    Complete {
        builder: Box<super::borrowed::SpriteBuilderOrigin>,
        appearance: Box<appearance_caller::Completion>,
    },
    CompleteAfterEnergyDecrease {
        builder: Box<super::borrowed::SpriteBuilderOrigin>,
        appearance: Box<appearance_caller::Completion>,
    },
}

/// Retained Draw read, selected append and actual builder publication.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub struct Stage(Progress);

impl Stage {
    pub(super) fn selected_result_holds(
        &self,
        selected: &appearance_caller::Completion,
        state: &State,
    ) -> bool {
        match &self.0 {
            Progress::Complete { appearance, .. }
            | Progress::CompleteAfterEnergyDecrease { appearance, .. } => {
                appearance.work == DRAWING
                    && appearance.receipt.validate().is_ok()
                    && appearance.receipt.before == selected.receipt.result
                    && appearance.receipt.playing == selected.receipt.playing_after
                    && state.appearance == appearance.receipt.result
                    && state.playing_figure == appearance.receipt.playing_after
            }
            Progress::Pending
            | Progress::Bridge
            | Progress::DrawingReady
            | Progress::Charging(_) => {
                state.appearance == selected.receipt.result
                    && state.playing_figure == selected.receipt.playing_after
            }
        }
    }

    /// The actual complete rendered builder, awaiting later finalization.
    #[must_use]
    pub const fn builder(&self) -> Option<&super::borrowed::SpriteBuilderOrigin> {
        match &self.0 {
            Progress::Complete { builder, .. }
            | Progress::CompleteAfterEnergyDecrease { builder, .. } => Some(builder),
            Progress::Pending
            | Progress::Bridge
            | Progress::DrawingReady
            | Progress::Charging(_) => None,
        }
    }

    pub(super) const fn deferred_hurt_flash(&self) -> bool {
        matches!(self.0, Progress::CompleteAfterEnergyDecrease { .. })
    }
}

const fn missing() -> ProductionFault {
    ProductionFault::Unsupported(Unsupported::Frame(Dependency::DrawingPreparation))
}

impl State {
    fn current_drawing_energy(&self) -> Option<EnergyRead> {
        let health = self.health.as_ref()?.state();
        let history = self.frame_history.as_ref()?;
        Some(EnergyRead::compare(health, history.previous_health()))
    }

    /// The Drawing comparison while the selected physical arm is in flight.
    #[must_use]
    pub fn drawing_energy_read(&self) -> Option<EnergyRead> {
        let Progress::Charging(pending) = &self.s6_drawing.0 else {
            return None;
        };
        Some(pending.energy)
    }

    pub(crate) fn validate_drawing_health_checkpoint(
        &self,
        content: &Content,
    ) -> Result<(), super::Fault> {
        use super::Fault::InvalidPlayerEntry;
        if let Progress::Charging(pending) = &self.s6_drawing.0 {
            if self.current_drawing_energy() != Some(pending.energy) {
                return Err(InvalidPlayerEntry);
            }
            let contract = content.s6_drawing.as_ref().ok_or(InvalidPlayerEntry)?;
            let inputs = self.drawing_inputs().map_err(|_| InvalidPlayerEntry)?;
            let (selected, aggregate) = self
                .select_drawing(contract, &inputs)
                .map_err(|_| InvalidPlayerEntry)?;
            if aggregate != pending.aggregate
                || selected != pending.selected
                || *inputs.builder != pending.builder
            {
                return Err(InvalidPlayerEntry);
            }
            let profile = contract
                .drawing_profile(self, aggregate.receipt(), pending.energy)
                .map_err(|_| InvalidPlayerEntry)?;
            self.validate_drawing_profile(DRAWING, profile.as_ref())
                .map_err(|_| InvalidPlayerEntry)?;
        }
        let alternate = content
            .s6_drawing
            .as_ref()
            .and_then(|contract| contract.decreased_energy.as_ref());
        let deferred = self.s6_drawing.deferred_hurt_flash();
        let Some(alternate) = alternate.filter(|_| self.publications.original_interval()) else {
            return if deferred {
                Err(InvalidPlayerEntry)
            } else {
                Ok(())
            };
        };
        let history = self.frame_history.as_ref().ok_or(InvalidPlayerEntry)?;
        let complete = self.s6_drawing.builder().is_some();
        let (expected, decreased) = if complete {
            let health = self.health.as_ref().ok_or(InvalidPlayerEntry)?.state();
            let read = EnergyRead::compare(health, alternate.previous);
            if self.maintenance.energy() != Some(read.current) {
                return Err(InvalidPlayerEntry);
            }
            (read.current, read.change == EnergyChange::Decreased)
        } else {
            (alternate.previous, false)
        };
        if history.drawn() != complete
            || history.previous_health() != expected
            || deferred != decreased
        {
            return Err(InvalidPlayerEntry);
        }
        Ok(())
    }

    /// A completed builder must be its retained draw repeated on the prepared builder.
    /// Finalization and the native return read this builder, so a restore cannot
    /// supply other sprite bytes.
    pub(super) fn validate_drawn_builder(&self, content: &Content) -> Result<(), super::Fault> {
        use super::Fault::InvalidPlayerEntry;
        let Some(drawn) = self.s6_drawing.builder() else {
            return Ok(());
        };
        let Caller::PlayerContinuation {
            prepared,
            remaining,
            ..
        } = &self.caller
        else {
            return Err(InvalidPlayerEntry);
        };
        let contract = content.s6_drawing.as_ref().ok_or(InvalidPlayerEntry)?;
        let receipt = self.rendering.receipt().ok_or(InvalidPlayerEntry)?;
        let actors = remaining.0.actors.as_ref().ok_or(InvalidPlayerEntry)?;
        let mut expected = prepared
            .builder
            .as_ref()
            .ok_or(InvalidPlayerEntry)?
            .terminal
            .clone();
        contract
            .rendering
            .redraw(self, actors, receipt, &mut expected)
            .map_err(|_| InvalidPlayerEntry)?;
        if expected != *drawn {
            return Err(InvalidPlayerEntry);
        }
        Ok(())
    }

    fn drawing_inputs(&self) -> Result<DrawingInputs<'_>, ProductionFault> {
        let Caller::PlayerContinuation {
            prepared,
            completed,
            remaining,
            controls,
            motion_reset: Some(motion_reset),
            ..
        } = &self.caller
        else {
            return Err(missing());
        };
        let origins = self.drawing_origins.as_ref().ok_or_else(missing)?;
        let energy = self.maintenance.energy().ok_or_else(missing)?;
        let health = self.health.as_ref().ok_or_else(missing)?.state();
        let builder = &prepared.builder.as_ref().ok_or_else(missing)?.terminal;
        let pose = completed.0.pose_input.as_ref().ok_or_else(missing)?;
        let actors = remaining.0.actors.as_ref().ok_or_else(missing)?;
        let camera = self.s6_camera.following().ok_or_else(missing)?;
        let appeared = &self.appearance;
        if energy != health.energy.whole.get()
            || energy == 0
            || origins.scene.debug
            || origins.invincibility.0 != actors.dependency.contact.samus_invincibility
            || actors.dependency.contact.damage_mode
                != super::borrowed_player_preparation::ContactDamageDisposition::Ordinary
            || origins.jump_buttons != pose.dependency.bindings.jump.0
            || appeared
                .validate_cannon_completion(interface_hud::Selection::None, &controls.answer.0)
                .is_err()
            || builder.cursor != 0
        {
            let dependency = if origins.jump_buttons == pose.dependency.bindings.jump.0 {
                Dependency::DrawingCurrentOwners
            } else {
                Dependency::DrawingCurrentControls
            };
            return Err(ProductionFault::Unsupported(Unsupported::Frame(dependency)));
        }
        Ok(DrawingInputs {
            origins,
            builder,
            energy,
            health,
            completed: drawing_owners::Completed {
                membership: prepared.actors.as_ref().ok_or_else(missing)?,
                projectiles: remaining.0.projectiles.as_ref().ok_or_else(missing)?,
                sprites: remaining.0.sprites.as_ref().ok_or_else(missing)?,
                actors,
                camera,
                motion_reset,
                controls,
            },
        })
    }

    fn select_drawing(
        &self,
        contract: &Contract,
        inputs: &DrawingInputs<'_>,
    ) -> Result<
        (
            appearance_caller::Stage,
            appearance_caller::DrawingAggregate,
        ),
        appearance_caller::Fault,
    > {
        let mut candidate = self.clone();
        let mut selected = appearance_caller::Stage::Idle;
        let aggregate = appearance_caller::DrawingAggregate::choose(
            &mut selected,
            &mut candidate,
            inputs.builder,
            &contract.rendering,
            inputs.origins,
            &inputs.completed,
        )?;
        selected.bind(DRAWING)?;
        Ok((selected, aggregate))
    }

    pub(super) fn advance_live_drawing(
        &mut self,
        content: &Content,
    ) -> Result<(), ProductionFault> {
        let contract = content.s6_drawing.as_ref().ok_or_else(missing)?;
        let inputs = self.drawing_inputs()?;
        match &self.s6_drawing.0 {
            Progress::Pending => {
                self.timing
                    .begin(BRIDGE, contract.bridge.clone())
                    .map_err(ProductionFault::FrameHardware)?;
                self.s6_drawing.0 = Progress::Bridge;
            }
            Progress::Bridge => self.validate_drawing_profile(BRIDGE, &contract.bridge)?,
            Progress::DrawingReady => {
                let (selected, aggregate) =
                    self.select_drawing(contract, &inputs).map_err(|fault| {
                        let dependency = match fault {
                            appearance_caller::Fault::Owner => Dependency::DrawingCurrentOwners,
                            appearance_caller::Fault::Rendering(_) => Dependency::DrawingRendering,
                            appearance_caller::Fault::Order
                            | appearance_caller::Fault::Sequence
                            | appearance_caller::Fault::Identity
                            | appearance_caller::Fault::Appearance(_) => {
                                Dependency::DrawingAppearance
                            }
                        };
                        ProductionFault::Unsupported(Unsupported::Frame(dependency))
                    })?;
                let energy = self.current_drawing_energy().ok_or_else(missing)?;
                let pending = Pending {
                    selected,
                    aggregate,
                    builder: inputs.builder.clone(),
                    energy,
                };
                self.timing
                    .begin(
                        DRAWING,
                        contract
                            .drawing_profile(self, pending.aggregate.receipt(), pending.energy)?
                            .into_owned(),
                    )
                    .map_err(ProductionFault::FrameHardware)?;
                self.s6_drawing.0 = Progress::Charging(Box::new(pending));
            }
            Progress::Charging(pending) => {
                if pending.energy.current != inputs.energy
                    || self.current_drawing_energy() != Some(pending.energy)
                    || pending.builder != *inputs.builder
                {
                    return Err(missing());
                }
                self.validate_drawing_profile(
                    DRAWING,
                    contract
                        .drawing_profile(self, pending.aggregate.receipt(), pending.energy)?
                        .as_ref(),
                )?;
            }
            Progress::Complete { .. } | Progress::CompleteAfterEnergyDecrease { .. } => {
                return Err(missing());
            }
        }
        Ok(())
    }

    fn validate_drawing_profile(
        &self,
        identity: timing::WorkId,
        profile: &Profile,
    ) -> Result<(), ProductionFault> {
        let mut active = self
            .timing
            .work
            .iter()
            .filter(|work| work.kind == timing::WorkKind::Semantic(identity));
        if !active.next().is_some_and(|work| &work.profile == profile) || active.next().is_some() {
            return Err(missing());
        }
        Ok(())
    }

    pub(super) fn complete_drawing_work(
        &mut self,
        work: timing::WorkId,
    ) -> Result<(), ProductionFault> {
        let inputs = self.drawing_inputs()?;
        match (&self.s6_drawing.0, work) {
            (Progress::Bridge, BRIDGE) => self.s6_drawing.0 = Progress::DrawingReady,
            (Progress::Charging(pending), DRAWING) => {
                if pending.energy.current != inputs.energy
                    || self.current_drawing_energy() != Some(pending.energy)
                    || pending.builder != *inputs.builder
                {
                    return Err(missing());
                }
                let mut candidate = self.clone();
                let mut stage = pending.selected.clone();
                let mut builder = pending.builder.clone();
                let selected = stage.receipt().ok_or_else(missing)?;
                if candidate.appearance != selected.before {
                    return Err(missing());
                }
                candidate.appearance = selected.chosen.clone();
                let (appearance, delta) = pending
                    .aggregate
                    .complete(
                        &mut stage,
                        &mut candidate,
                        &mut builder,
                        work,
                        inputs.origins,
                        &inputs.completed,
                    )
                    .map_err(|_| missing())?;
                if !delta.sounds.is_empty() || !delta.graphics.is_empty() {
                    return Err(missing());
                }
                let energy_decreased = pending.energy.change == EnergyChange::Decreased;
                let mut history = self.frame_history.clone().ok_or_else(missing)?;
                if !history.complete_drawing(self, inputs.health) {
                    return Err(missing());
                }
                self.frame_history = Some(history);
                self.appearance = candidate.appearance;
                self.rendering = candidate.rendering;
                let builder = Box::new(builder);
                let appearance = Box::new(appearance);
                self.s6_drawing.0 = if energy_decreased {
                    Progress::CompleteAfterEnergyDecrease {
                        builder,
                        appearance,
                    }
                } else {
                    Progress::Complete {
                        builder,
                        appearance,
                    }
                };
            }
            _ => return Err(missing()),
        }
        Ok(())
    }
}

struct DrawingInputs<'a> {
    origins: &'a drawing_owners::Origins,
    builder: &'a super::borrowed::SpriteBuilderOrigin,
    energy: u16,
    health: &'a samus_health::State,
    completed: drawing_owners::Completed<'a>,
}
