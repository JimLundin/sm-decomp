//! Finite production recipes shared by the application and indexed replay.
//! Each advance carries complete clean state and publishes the complete model output.

use crate::state::owned::{CodecState, OwnedState};
use canonical::model::Model;
use canonical::{Canonical, Schema};
use std::sync::Arc;

mod animation;
pub mod appearance;
pub mod audio;
mod camera_semantic;
pub mod connected;
mod content;
mod entries;
pub mod frame;
mod immutable;
mod movement;
mod movement_ceres;
pub mod movement_touchdown;
pub mod movement_work;
pub use movement_ceres::{CeresContext, MovementCeresOutputs, MovementCeresState};
mod sender;
#[cfg(test)]
mod tests;
pub mod timing;
pub mod weapons;
pub mod world;
pub use animation::{AppearanceContent, ClipFault, FigureClip};
use content::Context;
pub use content::{MovementContent, PoseContent, ProductionContent, ProductionRules};
pub use entries::{BlocksInput, BlocksOutputs, HudInput, PassageState};
pub use frame::inputs::MovementInputs;
pub use immutable::{BlockRules, CameraRules, HudRules};
pub use sender::{SenderInput, Senders};

/// A concrete clean entry supported by this engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    /// Carried movement, appearance and camera through an ordinary opportunity.
    Connected,
    /// Retained CPU frame or its admitted bounded opening.
    Frame,
    /// Movement and semantic pose.
    Movement,
    /// Ordinary Movement carried through Contact and touchdown graphics.
    MovementTouchdown,
    /// Movement position consumed by one Ceres door.
    MovementCeres,
    /// Admitted native audio transport.
    Audio,
    /// Joint resident transport and three CPU senders from a common origin.
    ResidentAudio,
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
    /// Complete coupled owners and any actual pending semantic work.
    Connected(Box<connected::State>),
    /// Complete caller, selected owners, hardware and delivery continuation.
    Frame(Box<frame::State>),
    /// Bounded Frame acknowledgment and Music owner.
    FrameOpening(Box<frame::opening_prelude::Opening>),
    /// Full movement memory.
    Movement(samus_movement::Samus),
    /// Movement, preceding Figure and the current Contact queue.
    MovementTouchdown(Box<movement_touchdown::State>),
    /// Both complete owners admitted at the same boundary.
    MovementCeres(Box<MovementCeresState>),
    /// Full committed and pending audio state; restoration never initializes it.
    Audio(Box<audio_transport::State>),
    /// Loaded or restored resident audio.
    ResidentAudio(Box<audio::State>),
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
    /// Independently admitted unselected stimuli for one connected ordinary opportunity.
    Connected {
        /// Complete current inputs; selected feedback is derived internally.
        inputs: &'a connected::inputs::Inputs,
        /// A semantic yield or genuine opportunity completion.
        advance: connected::Advance,
    },
    /// A scoped ordinary opportunity with the current early map inputs.
    ConnectedMap {
        /// Complete current ordinary Movement and Appearance inputs.
        inputs: &'a connected::inputs::Inputs,
        /// Current early map premises.
        map: connected::early_map::Inputs<'a>,
        /// Stop at the map or continue the ordinary opportunity.
        advance: connected::Advance,
    },
    /// Pause budget for the actual retained frame caller.
    Frame {
        /// Intrinsic host pause budget.
        budget: data::timing::MasterTicks,
    },
    /// One independently admitted ordinary opportunity for connected owners.
    MovementCeres {
        /// External movement dependencies.
        frame: samus_movement::Frame<'a>,
        /// External collision geometry and reaction permissions.
        world: &'a dyn types::World,
        /// Unselected Ceres producers; proximity is derived from held movement.
        ceres: CeresContext,
    },
    /// Every changing movement dependency must be supplied by its producer or admitted source.
    Movement {
        /// Current admitted movement stimuli.
        inputs: MovementInputs,
        /// Read-only admitted collision geometry and reaction permissions.
        /// Queries must not mutate an external gameplay owner or publish outputs.
        world: &'a dyn types::World,
    },
    /// Ordinary Movement with its admitted Contact and graphics opportunities.
    MovementTouchdown(movement_touchdown::Input<'a>),
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
    /// Scoped ordinary power update, ordered contact and reached flying-picture insertion.
    WeaponsContactDrawing {
        /// The bounded update and contact opportunity.
        shot: weapons::Opportunity<'a>,
        /// Independently admitted inputs at the flying-picture insertion point.
        drawing: weapons::FlyingDraw<'a>,
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
    /// Operate one resident sender at an independently scheduled CPU opportunity.
    ResidentSender {
        /// The physical request line selected by the caller.
        group: audio_transport::SoundGroup,
        /// The production sender operation.
        input: SenderInput<'a>,
    },
    /// A required producer has no admitted value at this boundary.
    Missing(Scope),
}

/// Fresh complete outputs from the selected entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProductionOutputs {
    /// Complete new selected outputs and actual semantic progress.
    Connected(Box<connected::Outputs>),
    /// Actual frame progress and ordered fresh physical outputs.
    Frame(frame::Outputs),
    /// Real physical steps of the bounded Frame opening.
    FrameOpening(timing::Progress, Vec<timing::Event>),
    /// Every movement receipt, including requests and placements.
    Movement(samus_movement::Outcome),
    /// Movement and completed touchdown consumer publications.
    MovementTouchdown(Box<movement_touchdown::Outputs>),
    /// Complete outputs committed with both owners.
    MovementCeres(Box<MovementCeresOutputs>),
    /// Every ordered observation, staging and publication milestone.
    Audio(Vec<audio_transport::transport::Output>),
    /// Every weapons request and supplied contact receipt.
    WeaponsContact(weapons::WeaponsOutputs),
    /// Scoped weapons requests, ordered contact and exact flying-picture insertion.
    WeaponsContactDrawing(Box<weapons::ShotDrawingOutputs>),
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
    /// Complete connected owner/content relation is invalid.
    Connected(connected::Fault),
    /// Complete frame continuation failed validation.
    Frame(Box<frame::Fault>),
    /// State, content and rules must select the same entry.
    WrongScope,
    /// A selectable semantic pose has no body content.
    MissingPose(types::Pose),
    /// Duplicate keys disagree about their data.
    ConflictingPose(types::Pose),
    /// Carried touchdown Figure belongs to another movement pose.
    MovementTouchdownFigure,
    /// Audio content has no complete two-track arrangement.
    AudioTracks,
    /// The resident session violates an audio admission invariant.
    ResidentAudio(audio::Fault),
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

impl From<frame::Fault> for AdmissionFault {
    fn from(fault: frame::Fault) -> Self {
        Self::Frame(Box::new(fault))
    }
}

/// Missing selected production behaviour remains visible.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unsupported {
    /// A produced Appearance request has no selected receiver mapping.
    AppearanceSound(samus_appearance::PublishedSound),
    /// A current causal frame prerequisite has no admitted owner.
    Frame(frame::Dependency),
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
    /// The supplied touchdown opportunity is stale or disagrees with the carried figure.
    MovementTouchdownCurrent,
    /// The reached Movement sound has no admitted Contact mapping.
    MovementTouchdownSound(types::SoundRequest),
    /// A live, produced or muzzle envelope reaches an unsupported continuation.
    BeamContinuation,
    /// A possible clip radius can create an unsupported contact span.
    BeamSpan,
    /// The flying-picture insertion cannot consume the supplied drawing state.
    BeamDrawing,
    /// Queue offers must occur between CPU sender services.
    SenderService,
    /// A cue or physical reply belongs to another sender's request line.
    SenderCueGroup,
}

/// A failed advance leaves state and publications unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProductionFault {
    /// A connected opportunity lacks actual current owners or supported inputs.
    Connected(connected::Fault),
    /// Invalid or unavailable current frame hardware continuation.
    FrameHardware(timing::Fault),
    /// Invalid retained frame output ordering or delivery cursor.
    FramePublications(frame::PublicationFault),
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
struct Checkpoint<S> {
    schema: Schema<S>,
    state: S,
}

/// Complete selected State lacks an approved durable codec.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveFault {
    /// Live behaviour is available, but this complete clean State cannot yet be encoded.
    Unsupported(Scope),
}

/// Canonical state bytes bound to an exact in-memory context.
/// Completed retained frames also carry the production return evidence.
/// Durable context encoding requires clean rule serialization support.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedProduction {
    state: Vec<u8>,
    context: Arc<Context>,
    frame_returned: bool,
}

/// A saved retained frame whose production caller completed the native return.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedCompletedFrame(SavedProduction);

/// Why the current production state cannot be saved as a completed retained frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompletedFrameSaveFault {
    /// The selected entry is not the retained frame caller.
    WrongScope,
    /// Production has yet to complete the current frame's native return.
    Incomplete,
}

/// One admitted entry with immutable context and one writable clean state owner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Production {
    state: OwnedState,
    context: Arc<Context>,
    frame_returned: bool,
}

impl Production {
    /// Admit a complete initial state and its matching imported context.
    ///
    /// # Errors
    /// Scope mismatches, shape-pin failures and invalid state or content are refused.
    /// Owner-specific faults identify the rejected continuation or content relation.
    pub fn admit(
        seed: ProductionSeed,
        content: ProductionContent,
        rules: ProductionRules,
    ) -> Result<Self, AdmissionFault> {
        validate_pins(match &seed {
            ProductionSeed::Connected(_) => Scope::Connected,
            ProductionSeed::Frame(_) | ProductionSeed::FrameOpening(_) => Scope::Frame,
            ProductionSeed::Movement(_) => Scope::Movement,
            ProductionSeed::MovementTouchdown(_) => Scope::MovementTouchdown,
            ProductionSeed::MovementCeres(_) => Scope::MovementCeres,
            ProductionSeed::Audio(_) => Scope::Audio,
            ProductionSeed::ResidentAudio(_) => Scope::ResidentAudio,
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
                ProductionSeed::Connected(state),
                ProductionContent::Connected(content),
                ProductionRules::Connected(_),
            ) => {
                content.movement.validate()?;
                state.validate(content).map_err(AdmissionFault::Connected)?;
                OwnedState::Connected(state)
            }
            (
                ProductionSeed::Frame(state),
                ProductionContent::Frame(content),
                ProductionRules::Frame(rules),
            ) => {
                state.validate()?;
                content.movement.validate()?;
                content
                    .random_requests
                    .multiplier
                    .checked()
                    .map_err(|_| frame::Fault::InvalidRandomWork)?;
                state.current_clip(&content.appearance)?;
                state.validate_committed_actors(content)?;
                state.validate_carried_health(content)?;
                state.validate_grounded_checkpoint(content, rules)?;
                state.validate_hud_checkpoint(content, rules)?;
                state.validate_scene_checkpoint(content)?;
                state.validate_countdown_checkpoint(content, rules)?;
                state.validate_animation_checkpoint(content, rules)?;
                OwnedState::Frame(state)
            }
            (
                ProductionSeed::MovementCeres(state),
                ProductionContent::MovementCeres(content),
                ProductionRules::MovementCeres { .. },
            ) => {
                content.validate()?;
                if content.pose(state.movement.0.pose).is_none() {
                    return Err(AdmissionFault::MissingPose(state.movement.0.pose));
                }
                OwnedState::MovementCeres(state)
            }
            (
                ProductionSeed::MovementTouchdown(state),
                ProductionContent::MovementTouchdown(content),
                ProductionRules::MovementTouchdown(_),
            ) => {
                state.validate(content)?;
                OwnedState::MovementTouchdown(state)
            }
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
            frame_returned: false,
        })
    }

    /// The complete held state, including any scheduled controller changes.
    #[must_use]
    pub const fn state(&self) -> &OwnedState {
        &self.state
    }

    /// Which concrete production entry was admitted.
    #[must_use]
    pub const fn scope(&self) -> Scope {
        match self.state {
            OwnedState::Connected(_) => Scope::Connected,
            OwnedState::Frame(_) | OwnedState::FrameOpening(_) => Scope::Frame,
            OwnedState::Movement(_) => Scope::Movement,
            OwnedState::MovementTouchdown(_) => Scope::MovementTouchdown,
            OwnedState::MovementCeres(_) => Scope::MovementCeres,
            OwnedState::Audio(_) => Scope::Audio,
            OwnedState::ResidentAudio(_) => Scope::ResidentAudio,
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

    /// Schedule physical first-port changes for the retained frame.
    ///
    /// Use absolute hardware times at or after the current time, in strictly increasing
    /// order across pending changes. Entering line 241 samples the latest change through
    /// that crossing and holds it for serial polling. A crossing already processed by
    /// an earlier advance is final. Unsampled changes survive pauses and checkpoints.
    ///
    /// # Errors
    /// A wrong scope, past time or unordered batch leaves the queue unchanged.
    pub fn schedule_frame_pad_changes(
        &mut self,
        changes: &[timing::PadChange],
    ) -> Result<(), ProductionFault> {
        let OwnedState::Frame(state) = &mut self.state else {
            return Err(ProductionFault::WrongScope);
        };
        state
            .timing
            .schedule_pad_changes(changes)
            .map_err(ProductionFault::FrameHardware)?;
        if !changes.is_empty() {
            self.frame_returned = false;
        }
        Ok(())
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
                OwnedState::Connected(state),
                ProductionContent::Connected(content),
                ProductionRules::Connected(rules),
                AdvanceInput::Connected { inputs, advance },
            ) => ProductionOutputs::Connected(Box::new(
                state
                    .advance(inputs, content, rules, advance)
                    .map_err(ProductionFault::Connected)?,
            )),
            (
                OwnedState::Connected(state),
                ProductionContent::Connected(content),
                ProductionRules::Connected(rules),
                AdvanceInput::ConnectedMap {
                    inputs,
                    map,
                    advance,
                },
            ) => ProductionOutputs::Connected(Box::new(
                state
                    .advance_with_map(inputs, content, rules, advance, Some(map))
                    .map_err(ProductionFault::Connected)?,
            )),
            (
                OwnedState::Frame(state),
                ProductionContent::Frame(content),
                ProductionRules::Frame(rules),
                AdvanceInput::Frame { budget },
            ) => {
                // The outer candidate already makes this caller's changes atomic.
                ProductionOutputs::Frame(state.advance_caller(content, rules, budget)?)
            }
            (
                OwnedState::MovementCeres(state),
                ProductionContent::MovementCeres(content),
                ProductionRules::MovementCeres { movement, ceres },
                AdvanceInput::MovementCeres {
                    frame,
                    world,
                    ceres: neighbours,
                },
            ) => ProductionOutputs::MovementCeres(Box::new(movement_ceres::advance(
                state, content, movement, ceres, &frame, world, neighbours,
            )?)),
            (
                OwnedState::MovementTouchdown(state),
                ProductionContent::MovementTouchdown(content),
                ProductionRules::MovementTouchdown(rules),
                AdvanceInput::MovementTouchdown(input),
            ) => ProductionOutputs::MovementTouchdown(Box::new(movement_touchdown::advance(
                state, content, rules, input,
            )?)),
            (
                OwnedState::Movement(samus),
                ProductionContent::Movement(content),
                ProductionRules::Movement(rules),
                AdvanceInput::Movement { inputs, world },
            ) => ProductionOutputs::Movement(movement::advance_scoped(
                &mut samus.0,
                content,
                rules,
                &inputs,
                world,
            )?),
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
        self.frame_returned = matches!(&outputs, ProductionOutputs::Frame(output)
            if output.progress == frame::Progress::Completed);
        self.state = candidate;
        Ok(outputs)
    }

    /// Read the admitted transport shared by the loaded and independently scoped entries.
    #[must_use]
    pub fn transport(&self) -> Option<&audio_transport::State> {
        match &self.state {
            OwnedState::Audio(state) => Some(&state.0),
            OwnedState::ResidentAudio(state) => Some(state.transport()),
            OwnedState::Connected(_)
            | OwnedState::Frame(_)
            | OwnedState::FrameOpening(_)
            | OwnedState::Movement(_)
            | OwnedState::MovementTouchdown(_)
            | OwnedState::MovementCeres(_)
            | OwnedState::WeaponsContact(_)
            | OwnedState::Blocks(_)
            | OwnedState::Camera(_)
            | OwnedState::Appearance(_)
            | OwnedState::Progress(_)
            | OwnedState::Hud(_)
            | OwnedState::Ceres(_)
            | OwnedState::Doors(_)
            | OwnedState::AudioSender(_) => None,
        }
    }

    /// Save the selected state and pending work with its immutable context in memory.
    /// A suspended Frame can be saved before the requested NMI return.
    ///
    /// # Errors
    /// A model without a complete codec reports its selected scope explicitly.
    pub fn save(&self) -> Result<SavedProduction, SaveFault> {
        let state = match &self.state {
            OwnedState::Connected(state) => CodecState::Connected(state.clone()),
            OwnedState::Frame(state) => CodecState::Frame(state.clone()),
            OwnedState::FrameOpening(state) => return Ok(self.encode_save(state.clone())),
            OwnedState::Movement(state) => CodecState::Movement(*state),
            OwnedState::Audio(state) => CodecState::Audio(state.clone()),
            OwnedState::ResidentAudio(state) => return Ok(self.encode_save(state.clone())),
            OwnedState::MovementTouchdown(state) => return Ok(self.encode_save(state.clone())),
            OwnedState::WeaponsContact(state) => CodecState::WeaponsContact(state.clone()),
            OwnedState::Camera(state) => CodecState::Camera(*state),
            OwnedState::Appearance(state) => CodecState::Appearance(*state),
            OwnedState::AudioSender(state) => CodecState::AudioSender(state.clone()),
            OwnedState::MovementCeres(_)
            | OwnedState::Blocks(_)
            | OwnedState::Progress(_)
            | OwnedState::Hud(_)
            | OwnedState::Ceres(_)
            | OwnedState::Doors(_) => {
                return Err(SaveFault::Unsupported(self.scope()));
            }
        };
        Ok(self.encode_save(state))
    }

    /// Save the current completed retained frame and its selected content, rules and timing.
    /// A failed advance preserves any prior committed return. Advancing partway into the next
    /// frame or scheduling new controller input clears eligibility until another native return.
    ///
    /// # Errors
    /// Returns [`CompletedFrameSaveFault::WrongScope`] for other entries and
    /// [`CompletedFrameSaveFault::Incomplete`] before a completed return.
    pub fn save_completed_frame(&self) -> Result<SavedCompletedFrame, CompletedFrameSaveFault> {
        let OwnedState::Frame(state) = &self.state else {
            return Err(CompletedFrameSaveFault::WrongScope);
        };
        if !self.frame_returned {
            return Err(CompletedFrameSaveFault::Incomplete);
        }
        Ok(SavedCompletedFrame(
            self.encode_save(CodecState::Frame(state.clone())),
        ))
    }

    fn encode_save<S: Canonical>(&self, state: S) -> SavedProduction {
        SavedProduction {
            state: canonical::to_bytes(&Checkpoint {
                schema: Schema::new(),
                state,
            }),
            context: self.context.clone(),
            frame_returned: self.frame_returned,
        }
    }

    /// Continue from a saved native return using the same imported content and selected rules.
    /// The saved hardware clock, pending controller input and publication cursor resume directly.
    ///
    /// # Errors
    /// Returns the same schema, content, rules and state admission errors as [`Self::restore`].
    pub fn restore_completed_frame(
        saved: &SavedCompletedFrame,
        content: ProductionContent,
        rules: ProductionRules,
    ) -> Result<Self, AdmissionFault> {
        Self::restore(&saved.0, content, rules)
    }

    /// Restore directly; no initialization, model step or input replay occurs.
    ///
    /// # Errors
    /// Checkpoint decoding, content and rule equality, and admission must all succeed.
    pub fn restore(
        saved: &SavedProduction,
        content: ProductionContent,
        rules: ProductionRules,
    ) -> Result<Self, AdmissionFault> {
        if saved.context.content != content {
            return Err(AdmissionFault::ContentChanged);
        }
        if saved.context.rules != rules {
            return Err(AdmissionFault::RulesChanged);
        }
        let seed = if matches!(&content, ProductionContent::ResidentAudio(_)) {
            let checkpoint: Checkpoint<Box<audio::State>> =
                canonical::from_bytes(&saved.state).map_err(AdmissionFault::Checkpoint)?;
            ProductionSeed::ResidentAudio(checkpoint.state)
        } else if matches!(&content, ProductionContent::MovementTouchdown(_)) {
            let checkpoint: Checkpoint<Box<movement_touchdown::State>> =
                canonical::from_bytes(&saved.state).map_err(AdmissionFault::Checkpoint)?;
            ProductionSeed::MovementTouchdown(checkpoint.state)
        } else if matches!(&content, ProductionContent::FrameOpening(_)) {
            let checkpoint: Checkpoint<Box<frame::opening_prelude::Opening>> =
                canonical::from_bytes(&saved.state).map_err(AdmissionFault::Checkpoint)?;
            ProductionSeed::FrameOpening(checkpoint.state)
        } else {
            let checkpoint: Checkpoint<CodecState> =
                canonical::from_bytes(&saved.state).map_err(AdmissionFault::Checkpoint)?;
            match checkpoint.state {
                CodecState::Connected(state) => ProductionSeed::Connected(state),
                CodecState::Frame(state) => ProductionSeed::Frame(state),
                CodecState::Movement(state) => ProductionSeed::Movement(state.0),
                CodecState::Audio(state) => ProductionSeed::Audio(state.0),
                CodecState::WeaponsContact(state) => ProductionSeed::WeaponsContact(state.0),
                CodecState::Camera(state) => ProductionSeed::Camera(state.0),
                CodecState::Appearance(state) => ProductionSeed::Appearance(state.0),
                CodecState::AudioSender(state) => ProductionSeed::AudioSender(state.0),
            }
        };
        let mut restored = Self::admit(seed, content, rules)?;
        restored.frame_returned = saved.frame_returned;
        Ok(restored)
    }
}

fn validate_pins(scope: Scope) -> Result<(), AdmissionFault> {
    match scope {
        Scope::Connected => {
            use canonical::model::pin_holds;
            pin_holds::<connected::inputs::InputsPin>()
                .and_then(|()| pin_holds::<connected::AheadPin>())
                .and_then(|()| pin_holds::<connected::PlayingPin>())
                .and_then(|()| pin_holds::<frame::hud::HudPin>())
                .and_then(|()| pin_holds::<frame::hud::WorkPin>())
                .and_then(|()| pin_holds::<movement_work::WorkPin>())
                .and_then(|()| pin_holds::<movement_work::ReceiptsPin>())
                .and_then(|()| pin_holds::<appearance::WorkPin>())
                .and_then(|()| pin_holds::<crate::state::owned::CameraPin>())
                .and_then(|()| pin_holds::<crate::state::owned::MovementPin>())
        }
        Scope::Frame => frame::validate_pins(),
        Scope::Movement | Scope::MovementCeres => {
            canonical::model::pin_holds::<crate::state::owned::MovementPin>()
        }
        Scope::MovementTouchdown => {
            canonical::model::pin_holds::<crate::state::owned::MovementPin>()
                .and_then(|()| canonical::model::pin_holds::<frame::inputs::FigurePin>())
                .and_then(|()| canonical::model::pin_holds::<crate::state::owned::SenderPin>())
        }
        Scope::Audio => canonical::model::pin_holds::<crate::state::owned::AudioPin>(),
        Scope::ResidentAudio => canonical::model::pin_holds::<crate::state::owned::AudioPin>()
            .and_then(|()| canonical::model::pin_holds::<crate::state::owned::SenderPin>()),
        Scope::WeaponsContact => canonical::model::pin_holds::<crate::state::owned::PowerPin>(),
        Scope::Camera => canonical::model::pin_holds::<crate::state::owned::CameraPin>(),
        Scope::Appearance => canonical::model::pin_holds::<crate::state::owned::AppearancePin>(),
        Scope::AudioSender => canonical::model::pin_holds::<crate::state::owned::SenderPin>(),
        Scope::Blocks | Scope::Progress | Scope::Hud | Scope::Ceres | Scope::Doors => Ok(()),
    }
    .map_err(AdmissionFault::StatePin)
}

fn advance_transport(
    state: &mut audio_transport::State,
    content: &audio_transport::Content,
    stop: &audio_transport::AudioInstant,
    presentations: &[audio_transport::transport::Input],
) -> Result<ProductionOutputs, ProductionFault> {
    validate_audio_interval(&state.now, stop, presentations)?;
    let elapsed = stop.difference(&state.now);
    Ok(ProductionOutputs::Audio(
        audio_transport::transport::advance(state, content, elapsed, presentations),
    ))
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
