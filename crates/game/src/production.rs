//! Finite production recipes shared by the application and indexed replay.
//! Each advance carries complete clean state and publishes the complete model output.

use crate::state::owned::{CodecState, OwnedState};
use canonical::model::Model;
use canonical::{Canonical, Schema};
use std::sync::Arc;

mod animation;
pub mod appearance;
mod content;
mod entries;
mod immutable;
mod movement;
mod sender;
#[cfg(test)]
mod tests;
pub mod weapons;
pub mod world;
pub use animation::{AppearanceContent, ClipFault, FigureClip};
use content::Context;
pub use content::{MovementContent, PoseContent, ProductionContent, ProductionRules};
pub use entries::{BlocksInput, BlocksOutputs, HudInput, PassageState};
pub use immutable::{BlockRules, CameraRules, HudRules};
pub use sender::SenderInput;

/// A concrete clean entry supported by this engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    /// Movement and semantic pose.
    Movement,
    /// Admitted native audio transport.
    Audio,
    /// Quiet ordinary power flight with ordered single-box Ceres-door contact.
    WeaponsContact,
    /// Ordered block requests and running reactions.
    Blocks,
    /// Following camera and retained receipts.
    Camera,
    /// Semantic animation and sound requests.
    Appearance,
    /// Run inventory, events and remembered places.
    Progress,
    /// Status bar, selection and mini-map.
    Hud,
    /// A Ceres door's complete run.
    Ceres,
    /// Door passage and its carried scene.
    Doors,
    /// One CPU audio sender group, with independently admitted service timing.
    AudioSender,
}

/// Complete initial state, admitted once rather than reconstructed each cut.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProductionSeed {
    /// Full movement memory.
    Movement(samus_movement::Samus),
    /// Full committed and pending audio state; restoration never initializes it.
    Audio(Box<audio_transport::State>),
    /// Full ordinary power shot/trail/spacing state.
    WeaponsContact(Box<weapons_projectiles::State>),
    /// Full tiles, places and screens.
    Blocks(Box<world_blocks::State>),
    /// Full camera memory and receipt.
    Camera(world_camera::Camera),
    /// Full animation memory.
    Appearance(samus_appearance::State),
    /// Full run progress.
    Progress(Box<run_progress::Progress>),
    /// Full HUD memory.
    Hud(Box<interface_hud::HudState>),
    /// Full Ceres door.
    Ceres(boss_ceres_ridley::CeresDoor),
    /// Full passage and scene.
    Doors(Box<PassageState>),
    /// Full committed and pending CPU sender service.
    AudioSender(Box<audio_transport::SenderState>),
}

/// Independent stimuli for one selected entry.
pub enum AdvanceInput<'a> {
    /// Every changing movement dependency must be supplied by its producer or admitted source.
    Movement {
        /// The actual preceding frame context.
        frame: samus_movement::Frame<'a>,
        /// Read-only admitted collision geometry and reaction permissions.
        /// Queries must not mutate an external gameplay owner or publish outputs.
        world: &'a dyn types::World,
    },
    /// Native transport time and physical presentations.
    Audio {
        /// Absolute stop, including every milestone through it.
        stop: audio_transport::AudioInstant,
        /// Ordered presentations strictly after the held current time and through the stop.
        presentations: &'a [audio_transport::transport::Input],
    },
    /// Independently admitted ordinary power inputs and current actor opportunities.
    WeaponsContact {
        /// Actual controller, cannon, timing, inherited motion, camera and room inputs.
        input: &'a weapons_projectiles::UpdateInput,
        /// Complete checked immutable terrain; reached unsupported cells reject the candidate.
        world: &'a weapons::QuietBeamWorld,
        /// Every current door opportunity in the actual processing order.
        contacts: &'a [weapons::DoorOpportunity],
    },
    /// Independent ordered block requests or room placements.
    Blocks(BlocksInput<'a>),
    /// Actual movement placement, lead, room screens and time permission.
    Camera(&'a world_camera::Frame<'a>),
    /// Actual pre/post figures, movement events, liquid and sound queue facts.
    Appearance(&'a samus_appearance::Inputs),
    /// Actual ordered world/inventory reports.
    Progress(&'a [run_progress::Report]),
    /// One selected play/out-of-play/boss HUD entry.
    Hud(HudInput<'a>),
    /// Actual preceding position, stage, random word and shake memory.
    Ceres(&'a boss_ceres_ridley::Inputs),
    /// Actual passage work, sound, music and doorway inputs.
    Doors(&'a world_doors::Inputs),
    /// An actual queue offer or an admitted CPU sender service operation.
    AudioSender(SenderInput<'a>),
    /// A required producer has no admitted value at this boundary.
    Missing(Scope),
}

/// Fresh complete outputs from the selected entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProductionOutputs {
    /// Every movement receipt, including requests and placements.
    Movement(samus_movement::Outcome),
    /// Every ordered observation, staging and publication milestone.
    Audio(Vec<audio_transport::transport::Output>),
    /// Every weapons request and supplied contact receipt.
    WeaponsContact(weapons::WeaponsOutputs),
    /// Every block request answer, written cell and sound.
    Blocks(BlocksOutputs),
    /// Complete resulting camera and retained receipt.
    Camera(world_camera::Camera),
    /// The clean animation entry's sound request, if any.
    Appearance(Option<types::AnimationSound>),
    /// Progress changed only its held complete State and emits no requests.
    Progress,
    /// Every ordered HUD event from the selected entry.
    Hud(Vec<interface_hud::HudEvent>),
    /// Complete resulting door, sound and shake.
    Ceres(boss_ceres_ridley::Run),
    /// Complete resulting scene and every passage report.
    Doors(Box<world_doors::Exports>),
    /// Every CPU reply observation, request presentation and service completion.
    AudioSender(Vec<audio_transport::sender::SenderOutput>),
}

/// Why a selected start or checkpoint cannot be admitted.
#[derive(Debug, PartialEq, Eq)]
pub enum AdmissionFault {
    /// State, content and rules must select the same entry.
    WrongScope,
    /// A selectable semantic pose has no body content.
    MissingPose(types::Pose),
    /// Duplicate keys disagree about their data.
    ConflictingPose(types::Pose),
    /// Audio content has no complete two-track arrangement.
    AudioTracks,
    /// Invalid restored shot or content references.
    Weapons(crate::weapons_projectiles_bridge::Fault),
    /// A semantic figure has conflicting or missing clip content.
    Appearance(ClipFault),
    /// The complete sender queue/progress contains an invalid length or another group.
    SenderQueue,
    /// A checkpoint belongs to other content.
    ContentChanged,
    /// A checkpoint belongs to other rules.
    RulesChanged,
    /// The canonical schema or value cannot be decoded.
    Checkpoint(canonical::DecodeError),
    /// A clean state or context shape no longer matches its pin.
    StatePin(canonical::model::Unpinned),
}

/// Missing selected production behaviour remains visible.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unsupported {
    /// No independent value or producer for the selected entry.
    MissingInput(Scope),
    /// A selected pose requires unavailable semantic content.
    PoseContent(types::Pose),
    /// Screen-facing moving elevator requires the retained ride body producer.
    ElevatorBody,
    /// Geometry or room/screens does not describe a complete supported rectangle.
    BeamWorld,
    /// A reached terrain cell requires geometry or a reaction outside this quiet recipe.
    ReactiveBeamWorld,
    /// A live, produced or muzzle envelope reaches an unsupported continuation.
    BeamContinuation,
    /// A possible clip radius can create an unsupported contact span.
    BeamSpan,
    /// Queue offers must occur between CPU sender services.
    SenderService,
    /// A cue or physical reply belongs to another sender's request line.
    SenderCueGroup,
}

/// A failed advance leaves state and publications unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProductionFault {
    /// Input belongs to another admitted entry.
    WrongScope,
    /// Required production behaviour is unavailable.
    Unsupported(Unsupported),
    /// The clean movement model does not name this pose.
    Movement(samus_movement::UnknownPose),
    /// Ordinary shot/content references do not support this update.
    Weapons(crate::weapons_projectiles_bridge::Fault),
    /// A required animation figure has no imported clip.
    Appearance(ClipFault),
    /// A block request or placement lies outside the admitted room.
    Blocks(world_blocks::OutsideTheRoom),
    /// Absolute audio time moved backwards.
    TimeReversed,
    /// A presentation lies outside the requested interval.
    PresentationOutsideInterval,
    /// Presentations are not in nondecreasing order.
    PresentationsUnordered,
}

#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
struct Checkpoint {
    schema: Schema<CodecState>,
    state: CodecState,
}

/// Complete selected State lacks an approved durable codec.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveFault {
    /// Live behaviour is available, but this complete clean State cannot yet be encoded.
    Unsupported(Scope),
}

/// Canonical state bytes bound to an exact in-memory context.
/// Durable context encoding requires clean rule serialization support.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedProduction {
    state: Vec<u8>,
    context: Arc<Context>,
}

/// One admitted entry with immutable context and one writable clean state owner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Production {
    state: OwnedState,
    context: Arc<Context>,
}

impl Production {
    /// Admit a complete initial state and its matching imported context.
    ///
    /// # Errors
    /// Scope, content, and schema faults are refused before any model runs.
    pub fn admit(
        seed: ProductionSeed,
        content: ProductionContent,
        rules: ProductionRules,
    ) -> Result<Self, AdmissionFault> {
        validate_pins(match &seed {
            ProductionSeed::Movement(_) => Scope::Movement,
            ProductionSeed::Audio(_) => Scope::Audio,
            ProductionSeed::WeaponsContact(_) => Scope::WeaponsContact,
            ProductionSeed::Blocks(_) => Scope::Blocks,
            ProductionSeed::Camera(_) => Scope::Camera,
            ProductionSeed::Appearance(_) => Scope::Appearance,
            ProductionSeed::Progress(_) => Scope::Progress,
            ProductionSeed::Hud(_) => Scope::Hud,
            ProductionSeed::Ceres(_) => Scope::Ceres,
            ProductionSeed::Doors(_) => Scope::Doors,
            ProductionSeed::AudioSender(_) => Scope::AudioSender,
        })?;
        let state = match (seed, &content, &rules) {
            (
                ProductionSeed::Movement(samus),
                ProductionContent::Movement(content),
                ProductionRules::Movement(_),
            ) => {
                content.validate()?;
                if content.pose(samus.pose).is_none() {
                    return Err(AdmissionFault::MissingPose(samus.pose));
                }
                OwnedState::Movement(Model(samus))
            }
            (
                ProductionSeed::Audio(state),
                ProductionContent::Audio(content),
                ProductionRules::Audio,
            ) => {
                if content.tracks.len() != 2
                    || content.tracks.iter().any(|track| track.phrases.is_empty())
                {
                    return Err(AdmissionFault::AudioTracks);
                }
                OwnedState::Audio(Model(state))
            }
            (
                ProductionSeed::WeaponsContact(state),
                ProductionContent::WeaponsContact(content),
                ProductionRules::WeaponsContact(_),
            ) => {
                content
                    .validate_state(&state)
                    .map_err(AdmissionFault::Weapons)?;
                OwnedState::WeaponsContact(Model(state))
            }
            (seed, content, rules) => entries::admit(seed, content, rules)?,
        };
        Ok(Self {
            state,
            context: Arc::new(Context { content, rules }),
        })
    }

    /// The complete held state; only a successful advance can change it.
    #[must_use]
    pub const fn state(&self) -> &OwnedState {
        &self.state
    }

    /// Which concrete production entry was admitted.
    #[must_use]
    pub const fn scope(&self) -> Scope {
        match self.state {
            OwnedState::Movement(_) => Scope::Movement,
            OwnedState::Audio(_) => Scope::Audio,
            OwnedState::WeaponsContact(_) => Scope::WeaponsContact,
            OwnedState::Blocks(_) => Scope::Blocks,
            OwnedState::Camera(_) => Scope::Camera,
            OwnedState::Appearance(_) => Scope::Appearance,
            OwnedState::Progress(_) => Scope::Progress,
            OwnedState::Hud(_) => Scope::Hud,
            OwnedState::Ceres(_) => Scope::Ceres,
            OwnedState::Doors(_) => Scope::Doors,
            OwnedState::AudioSender(_) => Scope::AudioSender,
        }
    }

    /// Advance a candidate value and commit its state and complete outputs together.
    ///
    /// # Errors
    /// Wrong scope, missing inputs and invalid audio intervals do not advance state.
    pub fn advance(
        &mut self,
        input: AdvanceInput<'_>,
    ) -> Result<ProductionOutputs, ProductionFault> {
        let mut candidate = self.state.clone();
        let outputs = match (
            &mut candidate,
            &self.context.content,
            &self.context.rules,
            input,
        ) {
            (
                OwnedState::Movement(samus),
                ProductionContent::Movement(content),
                ProductionRules::Movement(rules),
                AdvanceInput::Movement { frame, world },
            ) => ProductionOutputs::Movement(movement::advance(
                &mut samus.0,
                content,
                rules,
                &frame,
                world,
            )?),
            (
                OwnedState::Audio(state),
                ProductionContent::Audio(content),
                ProductionRules::Audio,
                AdvanceInput::Audio {
                    stop,
                    presentations,
                },
            ) => {
                validate_audio_interval(&state.0.now, &stop, presentations)?;
                let elapsed = stop.difference(&state.0.now);
                ProductionOutputs::Audio(audio_transport::transport::advance(
                    &mut state.0,
                    content,
                    elapsed,
                    presentations,
                ))
            }
            (
                OwnedState::WeaponsContact(state),
                ProductionContent::WeaponsContact(content),
                ProductionRules::WeaponsContact(rules),
                AdvanceInput::WeaponsContact {
                    input,
                    world,
                    contacts,
                },
            ) => ProductionOutputs::WeaponsContact(weapons::advance(
                &mut state.0,
                input,
                content,
                *rules,
                world,
                contacts,
            )?),
            (_, _, _, AdvanceInput::Missing(scope)) if scope == self.scope() => {
                return Err(ProductionFault::Unsupported(Unsupported::MissingInput(
                    scope,
                )));
            }
            (state, content, rules, input) => entries::advance(state, content, rules, input)?,
        };
        self.state = candidate;
        Ok(outputs)
    }

    /// Save complete clean state, pending work and the exact immutable context.
    ///
    /// # Errors
    /// A model without a complete codec reports its selected scope explicitly.
    pub fn save(&self) -> Result<SavedProduction, SaveFault> {
        let state = match &self.state {
            OwnedState::Movement(state) => CodecState::Movement(*state),
            OwnedState::Audio(state) => CodecState::Audio(state.clone()),
            OwnedState::WeaponsContact(state) => CodecState::WeaponsContact(state.clone()),
            OwnedState::Camera(state) => CodecState::Camera(*state),
            OwnedState::Appearance(state) => CodecState::Appearance(*state),
            OwnedState::AudioSender(state) => CodecState::AudioSender(state.clone()),
            OwnedState::Blocks(_)
            | OwnedState::Progress(_)
            | OwnedState::Hud(_)
            | OwnedState::Ceres(_)
            | OwnedState::Doors(_) => {
                return Err(SaveFault::Unsupported(self.scope()));
            }
        };
        Ok(SavedProduction {
            state: canonical::to_bytes(&Checkpoint {
                schema: Schema::new(),
                state,
            }),
            context: self.context.clone(),
        })
    }

    /// Restore directly; no initialization, model step or input replay occurs.
    ///
    /// # Errors
    /// A different schema, content or rule set is refused.
    pub fn restore(
        saved: &SavedProduction,
        content: ProductionContent,
        rules: ProductionRules,
    ) -> Result<Self, AdmissionFault> {
        let checkpoint: Checkpoint =
            canonical::from_bytes(&saved.state).map_err(AdmissionFault::Checkpoint)?;
        if saved.context.content != content {
            return Err(AdmissionFault::ContentChanged);
        }
        if saved.context.rules != rules {
            return Err(AdmissionFault::RulesChanged);
        }
        let seed = match checkpoint.state {
            CodecState::Movement(state) => ProductionSeed::Movement(state.0),
            CodecState::Audio(state) => ProductionSeed::Audio(state.0),
            CodecState::WeaponsContact(state) => ProductionSeed::WeaponsContact(state.0),
            CodecState::Camera(state) => ProductionSeed::Camera(state.0),
            CodecState::Appearance(state) => ProductionSeed::Appearance(state.0),
            CodecState::AudioSender(state) => ProductionSeed::AudioSender(state.0),
        };
        Self::admit(seed, content, rules)
    }
}

fn validate_pins(scope: Scope) -> Result<(), AdmissionFault> {
    match scope {
        Scope::Movement => canonical::model::pin_holds::<crate::state::owned::MovementPin>(),
        Scope::Audio => canonical::model::pin_holds::<crate::state::owned::AudioPin>(),
        Scope::WeaponsContact => canonical::model::pin_holds::<crate::state::owned::PowerPin>(),
        Scope::Camera => canonical::model::pin_holds::<crate::state::owned::CameraPin>(),
        Scope::Appearance => canonical::model::pin_holds::<crate::state::owned::AppearancePin>(),
        Scope::AudioSender => canonical::model::pin_holds::<crate::state::owned::SenderPin>(),
        Scope::Blocks | Scope::Progress | Scope::Hud | Scope::Ceres | Scope::Doors => Ok(()),
    }
    .map_err(AdmissionFault::StatePin)
}

fn validate_audio_interval(
    now: &audio_transport::AudioInstant,
    stop: &audio_transport::AudioInstant,
    inputs: &[audio_transport::transport::Input],
) -> Result<(), ProductionFault> {
    if stop < now {
        return Err(ProductionFault::TimeReversed);
    }
    if inputs
        .iter()
        .any(|input| input.at <= *now || input.at > *stop)
    {
        return Err(ProductionFault::PresentationOutsideInterval);
    }
    if inputs
        .iter()
        .zip(inputs.iter().skip(1))
        .any(|(first, second)| first.at > second.at)
    {
        return Err(ProductionFault::PresentationsUnordered);
    }
    Ok(())
}
