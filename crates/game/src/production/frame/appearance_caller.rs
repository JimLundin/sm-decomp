//! Appearance answers retained within their actual physical owner.

use super::{State, appearance_stage, inputs::FigurePin};
use crate::production::{AppearanceContent, appearance, timing::WorkId};
use canonical::{Canonical, model::Model};
use samus_appearance::{AnimationDecision, AppearanceRead, Figure, Inputs, Rules};

/// Whole appearance operations; source coordinates never choose one.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Actual stride contact before the animation update.
    Contact,
    /// Current ordinary animation through playing, before later pose fitting.
    Animation,
    /// Actual pose-selection read after the movement owner accepts its figure.
    Selection,
    /// Late initialization from that actual selection and the completed playing result.
    SelectedClip,
    /// Actual normal drawing from the prepared and current movement actions.
    Drawing,
}

impl Kind {
    /// The unique physical identity reserved for this caller operation.
    #[must_use]
    pub const fn identity(self) -> WorkId {
        WorkId(match self {
            Self::Contact => 1100,
            Self::Animation => 1101,
            Self::Selection => 1102,
            Self::SelectedClip => 1103,
            Self::Drawing => 1104,
        })
    }
}

/// Complete actual choice; snapshots are immutable receipts, not writable owners.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    /// Whole consuming operation.
    pub kind: Kind,
    /// Actual appearance owner before choosing.
    pub before: appearance::Work,
    /// Actual retained answer and any no-effect prefix decisions.
    pub chosen: appearance::Work,
    /// Complete clean result captured before the physical owner begins.
    pub result: appearance::Work,
    /// Actual currently playing figure, independent of future movement selection.
    pub playing: Model<FigurePin>,
    /// Playing publication belonging to this result, applied only at completion.
    pub playing_after: Model<FigurePin>,
}

/// An operation consumed only after its actual matching owner returned.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Completion {
    /// Actual returned physical identity.
    pub work: WorkId,
    /// Full actual choice consumed by that owner.
    pub receipt: Receipt,
}

/// Metadata for the sole mutable appearance work in frame state.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub enum Stage {
    /// No physical appearance owner is pending.
    #[default]
    Idle,
    /// Actual choice awaits full source-envelope qualification.
    Offered(Receipt),
    /// The qualified whole physical owner is charging.
    Charging {
        /// Actual physical work identity.
        work: WorkId,
        /// Retained complete choice used by the qualifier.
        receipt: Receipt,
    },
}

/// A readonly current-playing query captured within its consuming owner.
/// Capturing it is not a completed query and bills no additional work.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Query {
    owner: WorkId,
    before: appearance::Work,
    answered: appearance::Work,
    playing: Model<FigurePin>,
}

/// Caller order, current authority or physical identity does not match.
#[derive(Debug, PartialEq, Eq)]
pub enum Fault {
    /// Repeated choice, missing retained answer or premature completion.
    Order,
    /// Actual owner or checked content differs from the retained choice.
    Owner,
    /// The actual rule sequence requires a separately qualified physical family.
    Sequence,
    /// A different physical owner returned.
    Identity,
    /// Existing appearance admission or completion failed.
    Appearance(appearance_stage::StageFault),
    /// Actual drawing dependencies or checked sprite append failed.
    Rendering(super::rendering_stage::Fault),
}

/// Actual selected append retained inside the whole rendering aggregate.
/// Other aggregate owners require their own fresh qualification before the single bill.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct DrawingAggregate {
    current: super::drawing_owners::Current,
    movement: super::movement::Movement,
    camera: Model<crate::state::owned::CameraPin>,
    prepared: Option<Model<super::inputs::PreparedActionPin>>,
    contacts: super::rendering_stage::Contacts,
    rendering_before: super::rendering_stage::Stage,
    builder_before: super::borrowed::SpriteBuilderOrigin,
    builder_after: super::borrowed::SpriteBuilderOrigin,
    rendering_after: super::rendering_stage::Stage,
}

impl DrawingAggregate {
    /// Retain the actual Draw read and checked append without publishing either early.
    /// The supplied premises must come from the complete fresh current collection join.
    /// This entry qualifies no objects, cannon, trails, tiles or history owner by itself.
    /// # Errors
    /// Incomplete actual Camera, stale pose/action, missing content or unsupported append.
    pub fn choose(
        stage: &mut Stage,
        frame: &mut State,
        builder: &super::borrowed::SpriteBuilderOrigin,
        content: &super::rendering_stage::Content,
        origins: &super::drawing_owners::Origins,
        completed: &super::drawing_owners::Completed<'_>,
    ) -> Result<Self, Fault> {
        let current = super::drawing_owners::Current::choose(origins, frame, completed)
            .map_err(|_| Fault::Owner)?;
        let premises = current.premises();
        let mut candidate = frame.clone();
        let mut next_stage = stage.clone();
        next_stage.drawing(&mut candidate)?;
        let receipt = next_stage.receipt().ok_or(Fault::Order)?;
        receipt.validate()?;
        let mut rendering_after = frame.rendering.clone();
        let mut builder_after = builder.clone();
        rendering_after
            .choose(
                &mut builder_after,
                content,
                &super::rendering_stage::Inputs {
                    movement: &frame.movement,
                    appearance: &receipt.result,
                    contacts: &frame.graphical_contacts,
                    figure: frame.playing_figure.0,
                    premises,
                },
            )
            .map_err(Fault::Rendering)?;
        let aggregate = Self {
            current,
            movement: frame.movement.clone(),
            camera: frame.camera,
            prepared: frame.prepared_action,
            contacts: frame.graphical_contacts.clone(),
            rendering_before: frame.rendering.clone(),
            builder_before: builder.clone(),
            builder_after,
            rendering_after,
        };
        frame.appearance = candidate.appearance;
        *stage = next_stage;
        Ok(aggregate)
    }

    /// Retained actual selected append for the full aggregate's typed current qualifier.
    #[must_use]
    pub const fn receipt(&self) -> Option<&super::rendering_stage::NormalDrawingReceipt> {
        self.rendering_after.receipt()
    }

    /// Consume the Draw read and retained selected append at the sole aggregate return.
    /// Finalise remains its actual later owner; this does not materialize future OAM.
    /// # Errors
    /// Different actual owner, changed current dependency or replaced builder backing.
    pub fn complete(
        &self,
        stage: &mut Stage,
        frame: &mut State,
        builder: &mut super::borrowed::SpriteBuilderOrigin,
        returned: WorkId,
        origins: &super::drawing_owners::Origins,
        completed: &super::drawing_owners::Completed<'_>,
    ) -> Result<(Completion, appearance::Outputs), Fault> {
        self.current
            .validate(origins, frame, completed)
            .map_err(|_| Fault::Owner)?;
        if returned != Kind::Drawing.identity()
            || frame.movement != self.movement
            || frame.camera != self.camera
            || frame.prepared_action != self.prepared
            || frame.graphical_contacts != self.contacts
            || frame.rendering != self.rendering_before
            || *builder != self.builder_before
        {
            return Err(Fault::Owner);
        }
        let mut candidate = frame.clone();
        let mut next_stage = stage.clone();
        let completion = next_stage.complete(&mut candidate, returned)?;
        let mut rendering = self.rendering_after.clone();
        rendering.bind(returned).map_err(Fault::Rendering)?;
        rendering.complete(returned).map_err(Fault::Rendering)?;
        frame.appearance = candidate.appearance;
        frame.playing_figure = candidate.playing_figure;
        frame.rendering = rendering;
        *builder = self.builder_after.clone();
        *stage = next_stage;
        Ok(completion)
    }
}

impl Receipt {
    fn sequence(&self) -> Result<(), Fault> {
        let mut steps = self
            .chosen
            .model
            .0
            .steps
            .iter()
            .skip(self.before.model.0.steps.len());
        match self.kind {
            Kind::Animation => {
                for expected in [
                    AnimationDecision::DashBegan,
                    AnimationDecision::StandingRestart,
                    AnimationDecision::WallJumpReady,
                    AnimationDecision::FastFall,
                    AnimationDecision::RisingHold,
                    AnimationDecision::ClipPlaying,
                ] {
                    if !matches!(steps.next(), Some(samus_appearance::WorkStep::Animation(rule)) if *rule == expected)
                    {
                        return Err(Fault::Sequence);
                    }
                }
            }
            Kind::SelectedClip => {
                if !matches!(
                    steps.next(),
                    Some(samus_appearance::WorkStep::Appearance(
                        AppearanceRead::Selection { .. }
                    ))
                ) || !matches!(
                    steps.next(),
                    Some(samus_appearance::WorkStep::Animation(
                        AnimationDecision::SelectedClip
                    ))
                ) {
                    return Err(Fault::Sequence);
                }
            }
            Kind::Contact => {
                if !matches!(
                    steps.next(),
                    Some(samus_appearance::WorkStep::Appearance(
                        AppearanceRead::Contact { .. }
                    ))
                ) {
                    return Err(Fault::Sequence);
                }
            }
            Kind::Selection => {
                if !matches!(
                    steps.next(),
                    Some(samus_appearance::WorkStep::Appearance(
                        AppearanceRead::Selection { .. }
                    ))
                ) {
                    return Err(Fault::Sequence);
                }
            }
            Kind::Drawing => {
                if !matches!(
                    steps.next(),
                    Some(samus_appearance::WorkStep::Appearance(
                        AppearanceRead::Draw { .. }
                    ))
                ) {
                    return Err(Fault::Sequence);
                }
            }
        }
        if steps.next().is_some() {
            return Err(Fault::Sequence);
        }
        Ok(())
    }

    /// Validate complete retained before/choice/result identity without rerunning a read.
    /// # Errors
    /// Invalid work, changed prefix, mismatched answer or replaced result publication.
    pub fn validate(&self) -> Result<(), Fault> {
        self.sequence()?;
        self.before.validate().map_err(|_| Fault::Owner)?;
        self.chosen.validate().map_err(|_| Fault::Owner)?;
        self.result.validate().map_err(|_| Fault::Owner)?;
        let chosen = &self.chosen.model.0;
        let result = &self.result.model.0;
        let before = &self.before.model.0;
        if chosen.steps.get(..before.steps.len()) != Some(before.steps.as_slice())
            || chosen.completed.get(..before.completed.len()) != Some(before.completed.as_slice())
            || chosen.consumed < before.consumed
            || chosen.state != self.before.model.0.state
            || chosen.sounds != self.before.model.0.sounds
            || chosen.graphics != self.before.model.0.graphics
            || result.pending.is_some()
            || chosen.consumed.checked_add(1) != Some(result.consumed)
            || result.steps != chosen.steps
            || result.animation != chosen.animation
            || result.ahead != chosen.ahead
            || result.completed.len() != chosen.completed.len().saturating_add(1)
            || result.completed.get(..chosen.completed.len()) != Some(chosen.completed.as_slice())
        {
            return Err(Fault::Owner);
        }
        let (sound, graphic) = if let Some(answer) = &chosen.pending {
            let Some(samus_appearance::Receipt::Animation(completed)) = result.completed.last()
            else {
                return Err(Fault::Owner);
            };
            if completed.as_ref() != answer || result.state != answer.after {
                return Err(Fault::Owner);
            }
            let playing_after = if self.kind == Kind::SelectedClip {
                answer.inputs.figure_after
            } else {
                self.playing.0
            };
            if self.playing_after.0 != playing_after {
                return Err(Fault::Owner);
            }
            (
                answer
                    .sound
                    .map(samus_appearance::PublishedSound::Animation),
                None,
            )
        } else {
            let Some(samus_appearance::WorkStep::Appearance(input)) =
                chosen.steps.get(chosen.consumed)
            else {
                return Err(Fault::Owner);
            };
            let Some(samus_appearance::Receipt::Appearance {
                input: completed,
                state,
                output,
            }) = result.completed.last()
            else {
                return Err(Fault::Owner);
            };
            if completed != input
                || *state != chosen.state
                || result.state != chosen.state
                || self.playing_after != self.playing
            {
                return Err(Fault::Owner);
            }
            (
                output
                    .sound
                    .map(samus_appearance::PublishedSound::Appearance),
                output.contact,
            )
        };
        if !result
            .sounds
            .iter()
            .copied()
            .eq(chosen.sounds.iter().copied().chain(sound))
            || !result
                .graphics
                .iter()
                .copied()
                .eq(chosen.graphics.iter().copied().chain(graphic))
        {
            return Err(Fault::Owner);
        }
        Ok(())
    }
}

/// Validation-only current conditions for independently qualified physical families.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Requirement {
    /// Current and next checked instructions show; the actual stride uses its plain pace.
    OrdinaryAnimation {
        /// Imported current-only step equality, never a playhead initializer.
        step: samus_appearance::Step,
    },
    /// Actual selected Running figure keeps the already completed playing result.
    RunningInitialization,
}

/// Complete immutable envelope; a shared `PoseFit` envelope must be billed only once.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhysicalContract {
    requirement: Requirement,
    template: data::timing::Template,
}

impl PhysicalContract {
    /// Import independently accepted current conditions and the complete owner template.
    #[must_use]
    pub const fn new(requirement: Requirement, template: data::timing::Template) -> Self {
        Self {
            requirement,
            template,
        }
    }

    /// Qualify full actual current inputs, retained rule/result and immutable content.
    /// `PoseFit` callers conjoin this with their actual movement/body qualifier before one bill.
    /// # Errors
    /// Changed owners, unsupported family, wrong result or unresolved physical effects.
    pub fn qualify(
        &self,
        stage: &Stage,
        frame: &State,
        movement: &crate::production::MovementContent,
        content: &AppearanceContent,
    ) -> Result<data::timing::Profile, Fault> {
        stage.validate(frame)?;
        let receipt = stage.receipt().ok_or(Fault::Order)?;
        let before = &receipt.before.model.0.state;
        let chosen = &receipt.chosen.model.0;
        let answer = chosen.pending.as_ref().ok_or(Fault::Sequence)?;
        let data = chosen.animation.as_ref().ok_or(Fault::Owner)?;
        let current_content = content
            .animation_inputs(data.inputs, data.selected_from, data.rules.clone())
            .map_err(|_| Fault::Owner)?;
        if current_content != *data
            || data.inputs.figure != receipt.playing.0
            || frame
                .sender_current
                .is_none_or(|current| current.bus != super::sender_contract::BusRegime::Fast)
        {
            return Err(Fault::Owner);
        }
        match self.requirement {
            Requirement::OrdinaryAnimation { step } => {
                let next = step.next().ok_or(Fault::Sequence)?;
                let expected_hold = answer
                    .paces
                    .plain
                    .hold(next)
                    .or_else(|| answer.clip.hold(next))
                    .ok_or(Fault::Owner)?;
                if receipt.kind != Kind::Animation
                    || answer.rule != AnimationDecision::ClipPlaying
                    || answer.inputs.figure != receipt.playing.0
                    || answer.inputs.figure.pose.action != types::Action::Running
                    || before.playhead.step != step
                    || before.playhead.time_left != samus_appearance::Frames::LAST
                    || before.stage != types::BoostStage::Stage0
                    || answer.inputs.dash != samus_appearance::Dash::On
                    || answer.inputs.items[types::Item::SpeedBooster]
                    || answer.inputs.liquid != samus_appearance::Liquid::NONE
                    || !matches!(
                        answer.clip.at(step),
                        Some(samus_appearance::Instruction::Show(_))
                    )
                    || !matches!(
                        answer.clip.at(next),
                        Some(samus_appearance::Instruction::Show(_))
                    )
                    || answer.answer != samus_appearance::Answer::Advance
                    || answer.sound.is_some()
                    || answer.after.playhead.step != next
                    || answer.after.playhead.time_left != expected_hold.frames()
                    || answer.after.stage != before.stage
                    || answer.after.laps_left != before.laps_left
                {
                    return Err(Fault::Sequence);
                }
            }
            Requirement::RunningInitialization => {
                let previous = movement
                    .radius(data.selected_from.pose)
                    .ok_or(Fault::Owner)?;
                let selected = movement
                    .radius(answer.inputs.figure_after.pose)
                    .ok_or(Fault::Owner)?;
                if receipt.kind != Kind::SelectedClip
                    || answer.rule != AnimationDecision::SelectedClip
                    || answer.inputs.figure != data.selected_from
                    || data.selected_from.pose.action != types::Action::Running
                    || answer.inputs.figure_after.pose.action != types::Action::Running
                    || answer.inputs.figure_after.pose != frame.movement.state().pose
                    || previous.y.0 < selected.y.0
                    || answer.answer != samus_appearance::Answer::KeepPlayhead
                    || answer.after != *before
                    || answer.sound.is_some()
                {
                    return Err(Fault::Sequence);
                }
            }
        }
        if self
            .template
            .runs
            .iter()
            .any(|run| !matches!(run.effect, data::timing::TemplateEffect::None))
        {
            return Err(Fault::Sequence);
        }
        self.template.bind(&[]).map_err(|_| Fault::Sequence)
    }
}

impl Query {
    /// Capture current-playing quantities at the actual consuming opportunity.
    /// Early `FrameStart` belongs within the existing whole Run owner.
    /// # Errors
    /// Missing current clip or invalid actual appearance work.
    pub fn choose(
        state: &State,
        content: &AppearanceContent,
        owner: WorkId,
    ) -> Result<Self, Fault> {
        state.appearance.validate().map_err(|_| Fault::Owner)?;
        let ahead = state.current_animation(content).map_err(|_| Fault::Owner)?;
        let mut answered = state.appearance.clone();
        answered.model.0.ahead = Some(ahead);
        Ok(Self {
            owner,
            before: state.appearance.clone(),
            answered,
            playing: state.playing_figure,
        })
    }

    /// Complete only the actual owner that consumed these current-playing quantities.
    /// This operation neither advances animation nor starts independent physical work.
    /// # Errors
    /// Different returned owner or changed playing/appearance authority.
    pub fn complete(
        &self,
        state: &mut State,
        returned: WorkId,
    ) -> Result<samus_appearance::Ahead, Fault> {
        if returned != self.owner {
            return Err(Fault::Identity);
        }
        if state.appearance != self.before || state.playing_figure != self.playing {
            return Err(Fault::Owner);
        }
        let answer = self.answered.model.0.ahead.ok_or(Fault::Owner)?;
        state.appearance.model.0.ahead = Some(answer);
        Ok(answer)
    }
}

impl Stage {
    const fn idle(&self) -> Result<(), Fault> {
        if !matches!(self, Self::Idle) {
            return Err(Fault::Order);
        }
        Ok(())
    }

    fn retain(&mut self, state: &mut State, candidate: State, kind: Kind) -> Result<(), Fault> {
        let mut result = candidate.appearance.clone();
        let playing_after = if kind == Kind::SelectedClip {
            let answer = candidate
                .appearance
                .model
                .0
                .pending
                .as_ref()
                .ok_or(Fault::Order)?;
            Model(answer.inputs.figure_after)
        } else {
            state.playing_figure
        };
        if !result.complete() {
            return Err(Fault::Order);
        }
        result.validate().map_err(|_| Fault::Owner)?;
        let receipt = Receipt {
            kind,
            before: state.appearance.clone(),
            chosen: candidate.appearance.clone(),
            result,
            playing: state.playing_figure,
            playing_after,
        };
        state.appearance = candidate.appearance;
        *self = Self::Offered(receipt);
        Ok(())
    }

    /// Capture actual stride quantities before animation advances.
    /// # Errors
    /// Pending work, wrong movement figure or missing checked current clip.
    pub fn contact(
        &mut self,
        state: &mut State,
        content: &AppearanceContent,
        figure: Figure,
        suppressors: types::Set<samus_appearance::ContactSuppressor>,
    ) -> Result<(), Fault> {
        self.idle()?;
        let mut candidate = state.clone();
        appearance_stage::choose_contact(&mut candidate, content, figure, suppressors)
            .map_err(Fault::Appearance)?;
        self.retain(state, candidate, Kind::Contact)?;
        Ok(())
    }

    /// Capture the actual accepted figure; movement need not be globally finished yet.
    /// # Errors
    /// Unaccepted current pose, missing immutable content or unfinished work.
    pub fn selection(
        &mut self,
        state: &mut State,
        content: &AppearanceContent,
        accepted: Figure,
        items: types::Set<types::Item>,
        submerged: bool,
        cinematic: samus_appearance::CinematicActivity,
    ) -> Result<(), Fault> {
        self.idle()?;
        if accepted.pose != state.movement.state().pose || content.get(accepted).is_none() {
            return Err(Fault::Owner);
        }
        let mut candidate = state.clone();
        let read = AppearanceRead::Selection {
            from: state.playing_figure.0,
            selected: accepted,
            items,
            submerged,
            cinematic,
        };
        appearance_stage::choose_read(&mut candidate.appearance, read)
            .map_err(Fault::Appearance)?;
        self.retain(state, candidate, Kind::Selection)?;
        Ok(())
    }

    /// Retain the whole ordinary animation owner, leaving playing as its final pending answer.
    /// Only a no-effect prefix is admitted before this bounded family's final rule.
    /// Accepted selection is deliberately refreshed later, at its own opportunity.
    /// # Errors
    /// Future figures, effective prefix rules or invalid pending animation.
    pub fn animation(
        &mut self,
        state: &mut State,
        content: &AppearanceContent,
        inputs: Inputs,
        rules: Rules,
    ) -> Result<(), Fault> {
        self.idle()?;
        if inputs.figure != state.playing_figure.0 || inputs.figure_after != inputs.figure {
            return Err(Fault::Owner);
        }
        let data = content
            .animation_inputs(inputs, state.playing_figure.0, rules)
            .map_err(|_| Fault::Owner)?;
        let mut candidate = state.clone();
        for rule in [
            AnimationDecision::DashBegan,
            AnimationDecision::StandingRestart,
            AnimationDecision::WallJumpReady,
            AnimationDecision::FastFall,
            AnimationDecision::RisingHold,
        ] {
            appearance_stage::choose_animation(&mut candidate.appearance, data.clone(), rule)
                .map_err(Fault::Appearance)?;
            if let Some(answer) = &candidate.appearance.model.0.pending {
                if answer.before != answer.after || answer.sound.is_some() {
                    return Err(Fault::Sequence);
                }
                if !candidate.appearance.complete() {
                    return Err(Fault::Order);
                }
            }
        }
        appearance_stage::choose_animation(
            &mut candidate.appearance,
            data,
            AnimationDecision::ClipPlaying,
        )
        .map_err(Fault::Appearance)?;
        if candidate.appearance.model.0.pending.is_none() {
            return Err(Fault::Sequence);
        }
        self.retain(state, candidate, Kind::Animation)?;
        Ok(())
    }

    /// Decide late initialization from the actual accepted figure and already played state.
    /// No preparation flag resets the persistent playhead.
    /// # Errors
    /// Wrong current/accepted figure, missing content or unfinished previous owner.
    pub fn selected_clip(
        &mut self,
        state: &mut State,
        content: &AppearanceContent,
        inputs: Inputs,
        selected_from: Figure,
        rules: Rules,
        cinematic: samus_appearance::CinematicActivity,
    ) -> Result<(), Fault> {
        self.idle()?;
        if inputs.figure != state.playing_figure.0
            || inputs.figure_after.pose != state.movement.state().pose
        {
            return Err(Fault::Owner);
        }
        let data = content
            .animation_inputs(inputs, selected_from, rules)
            .map_err(|_| Fault::Owner)?;
        let mut candidate = state.clone();
        let read = AppearanceRead::Selection {
            from: selected_from,
            selected: inputs.figure_after,
            items: inputs.items,
            submerged: inputs.liquid.submerged,
            cinematic,
        };
        appearance_stage::choose_read(&mut candidate.appearance, read)
            .map_err(Fault::Appearance)?;
        if !candidate.appearance.complete() {
            return Err(Fault::Order);
        }
        let Some(samus_appearance::Receipt::Appearance { output, .. }) =
            candidate.appearance.model.0.completed.last()
        else {
            return Err(Fault::Sequence);
        };
        if output.contact.is_some() || output.sound.is_some() {
            return Err(Fault::Sequence);
        }
        appearance_stage::choose_animation(
            &mut candidate.appearance,
            data,
            AnimationDecision::SelectedClip,
        )
        .map_err(Fault::Appearance)?;
        self.retain(state, candidate, Kind::SelectedClip)?;
        Ok(())
    }

    /// Capture drawing from its independently prepared action and actual current action.
    /// # Errors
    /// Missing preparation or unfinished appearance owner.
    pub fn drawing(&mut self, state: &mut State) -> Result<(), Fault> {
        self.idle()?;
        let mut candidate = state.clone();
        appearance_stage::choose_drawing(&mut candidate).map_err(Fault::Appearance)?;
        self.retain(state, candidate, Kind::Drawing)?;
        Ok(())
    }

    /// Full current receipt supplied to the game-owned physical qualifier.
    #[must_use]
    pub const fn receipt(&self) -> Option<&Receipt> {
        match self {
            Self::Idle => None,
            Self::Offered(receipt) | Self::Charging { receipt, .. } => Some(receipt),
        }
    }

    /// Revalidate a held actual answer without rerunning any query or rule.
    /// # Errors
    /// Replaced owner, figure or work, or an invalid retained answer.
    pub fn validate(&self, state: &State) -> Result<(), Fault> {
        let receipt = self.receipt().ok_or(Fault::Order)?;
        receipt.validate()?;
        state.appearance.validate().map_err(|_| Fault::Owner)?;
        if state.appearance != receipt.chosen || state.playing_figure != receipt.playing {
            return Err(Fault::Owner);
        }
        Ok(())
    }

    /// Bind only the actual qualified whole owner, never an arbitrary identity.
    /// # Errors
    /// Missing offer or an identity belonging to another operation.
    pub fn bind(&mut self, work: WorkId) -> Result<(), Fault> {
        let Self::Offered(receipt) = self else {
            return Err(Fault::Order);
        };
        if work != receipt.kind.identity() {
            return Err(Fault::Identity);
        }
        *self = Self::Charging {
            work,
            receipt: receipt.clone(),
        };
        Ok(())
    }

    /// Consume the retained answer after the actual matching physical owner returned.
    /// Parent commits the returned delta with actual graphics and Sender consumers.
    /// # Errors
    /// Early/different completion or replaced held work.
    pub fn complete(
        &mut self,
        state: &mut State,
        returned: WorkId,
    ) -> Result<(Completion, appearance::Outputs), Fault> {
        self.validate(state)?;
        let Self::Charging { work, receipt } = self else {
            return Err(Fault::Order);
        };
        if returned != *work {
            return Err(Fault::Identity);
        }
        let mut candidate = state.clone();
        candidate.appearance = receipt.result.clone();
        candidate.playing_figure = receipt.playing_after;
        let outputs = candidate.appearance.deliver().map_err(|_| Fault::Owner)?;
        let completion = Completion {
            work: *work,
            receipt: receipt.clone(),
        };
        state.appearance = candidate.appearance;
        state.playing_figure = candidate.playing_figure;
        *self = Self::Idle;
        Ok((completion, outputs))
    }
}
