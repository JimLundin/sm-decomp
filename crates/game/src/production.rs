//! Finite production recipes shared by the application and indexed replay.
//! Each advance carries complete clean state and publishes the complete model output.

use self::audio_checkpoint::PortableAudio;
use self::played::discovery;
use self::played::flashback;
use self::played::intro_rooms;
use crate::state::owned::{CodecState, OwnedState};
use canonical::model::Model;
use canonical::{Canonical, Schema};
use std::sync::Arc;

mod animation;
pub mod appearance;
pub mod audio;
mod audio_checkpoint;
pub mod blackout;
mod camera_semantic;
pub mod connected;
mod content;
mod entries;
pub mod health_tail;
pub mod hud;
mod immutable;
pub mod intro_missile_scope;
pub mod jump_history;
mod movement;
pub mod movement_touchdown;
pub mod playback;
pub mod played;
pub mod ridley;
pub mod room_loading;
pub mod save_files;
pub mod sender;
pub mod shaft;
#[cfg(test)]
mod tests;
pub mod weapons;
pub mod world;
pub use animation::{AppearanceContent, ClipFault, FigureClip, pulse_colours};
use content::Context;
pub use content::{MovementContent, PoseContent, ProductionContent, ProductionRules};
pub use entries::{BlocksInput, BlocksOutputs, HudInput, PassageState};
pub use immutable::{BlockRules, CameraRules, HudRules};
pub use movement::MovementInputs;

/// A concrete clean entry supported by this engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    /// The shaft blackout and its suspended departure.
    Blackout,
    /// The complete intro with a selected encounter input.
    Intro,
    /// The intro room start and its ordinary first turn.
    IntroRooms,
    /// Recorded intro pad and Samus update authority.
    Playback,
    /// Cartridge persistence.
    SaveFiles,
    /// Carried movement, appearance and camera through an ordinary opportunity.
    Connected,
    /// The Zebes flashback encounter at its cinematic actor turn.
    Flashback,
    /// The intro discovery encounter at its room turn.
    Discovery,
    /// Scoped Health and its late Drawing and HUD consumers.
    HealthTail,
    /// Movement and semantic pose.
    Movement,
    /// Jump history on draws while movement is paused.
    JumpHistory,
    /// Ordinary Movement carried through Contact and touchdown graphics.
    MovementTouchdown,
    /// Audio transport, one frame at a time.
    Audio,
    /// Quiet ordinary power flight with ordered single-box Ceres-door contact.
    WeaponsContact,
    /// Ordinary missiles in the intro fight.
    IntroMissiles,
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
}

/// Complete initial state, admitted once rather than reconstructed each cut.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProductionSeed {
    /// The shaft blackout and its suspended departure.
    Blackout(Box<blackout::State>),
    /// The complete intro and its displayed palette.
    Intro(Box<played::intro::State>),
    /// The intro room join from one complete origin.
    IntroRooms(Box<intro_rooms::Scoped>),
    /// Complete recorded pad phase.
    Playback(playback::ScenePhase),
    /// Complete cartridge, working profile and pending work.
    SaveFiles(Box<save_files::State>),
    /// Complete coupled owners and any actual pending semantic work.
    Connected(Box<connected::State>),
    /// Complete flashback encounter State, including any pending start.
    Flashback(Box<flashback::EncounterState>),
    /// Complete discovery encounter State, including every particle and the baby.
    Discovery(Box<discovery::DiscoveryState>),
    /// Complete current Health, Drawing history and gauge from a single origin.
    HealthTail(Box<health_tail::State>),
    /// Full movement memory.
    Movement(samus_movement::Samus),
    /// The carried jump count, pending check and last draw sample.
    JumpHistory(samus_movement::DrawnJumpHistory),
    /// Movement, preceding Figure and the current audio transport.
    MovementTouchdown(Box<movement_touchdown::State>),
    /// Complete audio transport State; restoration never initializes it.
    Audio(Box<audio_transport::State>),
    /// Full ordinary power shot/trail/spacing state.
    WeaponsContact(Box<weapons_projectiles::State>),
    /// Complete cinematic weapon State and pending sprite publication.
    IntroMissiles(Box<intro_missile_scope::State>),
    /// Full tiles, places and screens.
    Blocks(Box<world_blocks::State>),
    /// Full camera memory and receipt.
    Camera(world_camera::Camera),
    /// Her whole appearance State.
    Appearance(samus_appearance::AppearanceState),
    /// Full run progress.
    Progress(Box<run_progress::Progress>),
    /// Full HUD memory.
    Hud(Box<interface_hud::HudState>),
    /// Full Ceres door.
    Ceres(boss_ceres_ridley::CeresDoor),
    /// Full passage and scene.
    Doors(Box<PassageState>),
}

/// Independent stimuli for one selected entry.
pub enum AdvanceInput<'a> {
    /// The shaft blackout and its suspended departure.
    Blackout(blackout::Cut),
    /// An intro work-frame boundary.
    Intro(played::intro::FrameEnd),
    /// A causal intro room opportunity.
    IntroRooms(intro_rooms::Opportunity),
    /// The next authorized scene phase and any completed drawing opportunity.
    Flashback(flashback::Cut<'a>),
    /// Scene start and/or the actual room actor opportunity at this cut.
    Playback(playback::Cut),
    /// The current cartridge operation.
    SaveFiles(save_files::Input<'a>),
    /// Cinematic weapon work, encounter and presentation opportunities.
    IntroMissiles(intro_missile_scope::Input<'a>),
    /// A history-writing draw button sample or an interval without a draw.
    JumpHistory(Option<samus_movement::JumpButton>),
    /// The next authorized scene phase and any completed drawing opportunity.
    Discovery(discovery::Frame),
    /// One whole scoped Health frame and its Drawing and HUD consumers.
    HealthTail(&'a health_tail::Frame),
    /// Independently admitted unselected stimuli for one connected ordinary frame.
    Connected {
        /// Complete current inputs; selected feedback is derived internally.
        inputs: &'a connected::inputs::Inputs,
    },
    /// A scoped ordinary frame with the current early map inputs.
    ConnectedMap {
        /// Complete current ordinary Movement and Appearance inputs.
        inputs: &'a connected::inputs::Inputs,
        /// Current early map premises.
        map: connected::early_map::Inputs<'a>,
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
    /// One frame of audio transport: its queue offers and its borrowed cadence.
    Audio {
        /// The frame's offers, in the order their producers made them.
        offers: &'a [audio_transport::Offer],
        /// The frame's effect turns around its sender service, and its music turns.
        cadence: audio_transport::frame::FrameCadence,
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
    /// One frame of her appearance: its facts, once each, and its parts.
    Appearance(&'a samus_appearance::Frame),
    /// Actual ordered world/inventory reports.
    Progress(&'a [run_progress::Report]),
    /// One selected play/out-of-play/boss HUD entry.
    Hud(HudInput<'a>),
    /// Actual preceding position, stage, random word and shake memory.
    Ceres(&'a boss_ceres_ridley::Inputs),
    /// Actual passage work, sound, music and doorway inputs.
    Doors(&'a world_doors::Inputs),
    /// A required producer has no admitted value at this boundary.
    Missing(Scope),
}

/// Fresh complete outputs from the selected entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProductionOutputs {
    /// The shaft blackout and its suspended departure.
    Blackout(Box<blackout::Output>),
    /// The intro's completed frame publications, if work finished.
    Intro(Option<Box<screens_intro::Report>>),
    /// The first turn when it completed.
    IntroRooms(Option<Box<intro_rooms::FirstTurn>>),
    /// Requests to the surrounding owners and the completed sprite publication.
    Flashback(flashback::Publication),
    /// Recorded pad and immediate update authority publications.
    Playback(playback::CutReport),
    /// The full battery image and operation receipt.
    SaveFiles(Box<save_files::Output>),
    /// Complete scoped cinematic weapon requests and publication.
    IntroMissiles(Box<intro_missile_scope::Output>),
    /// No external request accompanies the carried jump history.
    JumpHistory,
    /// Requests to the surrounding owners and the completed sprite publication.
    Discovery(discovery::Publication),
    /// One scoped Health frame's requests and gauge redraw.
    HealthTail(health_tail::Output),
    /// Complete new selected outputs and actual semantic progress.
    Connected(Box<connected::Outputs>),
    /// Every movement receipt, including requests and placements.
    Movement(samus_movement::Outcome),
    /// Movement and completed touchdown consumer publications.
    MovementTouchdown(Box<movement_touchdown::Outputs>),
    /// The frame played; its published replies are read back from the held State.
    Audio,
    /// Every weapons request and supplied contact receipt.
    WeaponsContact(weapons::WeaponsOutputs),
    /// Scoped weapons requests, ordered contact and exact flying-picture insertion.
    WeaponsContactDrawing(Box<weapons::ShotDrawingOutputs>),
    /// Every block request answer, written cell and sound.
    Blocks(BlocksOutputs),
    /// Complete resulting camera and retained receipt.
    Camera(world_camera::Camera),
    /// Her appearance frame's complete outputs: sounds, contact, cannon and dry colours.
    Appearance(samus_appearance::FrameOutputs),
    /// Progress changed only its held complete State and emits no requests.
    Progress,
    /// A frame of play's outputs; a step outside play or a boss blanking the map has none.
    Hud(Option<interface_hud::FrameOutputs>),
    /// Complete resulting door, sound and shake.
    Ceres(boss_ceres_ridley::Run),
    /// Complete resulting scene and every passage report.
    Doors(Box<world_doors::Exports>),
}

/// Why a selected start or checkpoint cannot be admitted.
#[derive(Debug, PartialEq, Eq)]
pub enum AdmissionFault {
    /// The shaft blackout and its suspended departure.
    Blackout(blackout::Fault),
    /// The saved playback progress or pad is outside its content.
    Playback,
    /// Complete connected owner/content relation is invalid.
    Connected(connected::Fault),
    /// State, content and rules must select the same entry.
    WrongScope,
    /// A selectable semantic pose has no body content.
    MissingPose(types::Pose),
    /// Duplicate keys disagree about their data.
    ConflictingPose(types::Pose),
    /// Carried touchdown Figure belongs to another movement pose.
    MovementTouchdownFigure,
    /// Carried scoped Health belongs to a different authenticated origin.
    HealthTailOrigin,
    /// Audio content or a saved transport State violates an admission invariant.
    Audio(audio::Fault),
    /// Invalid restored shot or content references.
    Weapons(crate::weapons_projectiles_bridge::Fault),
    /// A semantic figure has conflicting or missing clip content.
    Appearance(ClipFault),
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
    /// The shaft blackout and its suspended departure.
    Blackout(blackout::Fault),
    /// A required continuation beyond the intro scene provided here.
    Intro(played::intro::Fault),
    /// The intro room join requires a producer or content outside its scope.
    IntroRooms(intro_rooms::Fault),
    /// The cartridge cannot accept this request in its current state.
    SaveFiles(::save_files::Refusal),
    /// A weapon impact cue needs its qualified sender mapping.
    WeaponImpactSound,
    /// The scoped Health frame is outside the admitted clock, equipment or quiet HUD.
    HealthTailFrame,
    /// A frame of play the HUD leaves out.
    Hud(interface_hud::UnsupportedFrame),
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
}

/// A failed advance leaves state and publications unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProductionFault {
    /// A connected opportunity lacks actual current owners or supported inputs.
    Connected(connected::Fault),
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
    /// Scope mismatches, shape-pin failures and invalid state or content are refused.
    /// Owner-specific faults identify the rejected continuation or content relation.
    pub fn admit(
        seed: ProductionSeed,
        content: ProductionContent,
        rules: ProductionRules,
    ) -> Result<Self, AdmissionFault> {
        validate_pins(match &seed {
            ProductionSeed::Blackout(_) => Scope::Blackout,
            ProductionSeed::Intro(_) => Scope::Intro,
            ProductionSeed::Playback(_) => Scope::Playback,
            ProductionSeed::SaveFiles(_) => Scope::SaveFiles,
            ProductionSeed::Connected(_) => Scope::Connected,
            ProductionSeed::IntroRooms(_) => Scope::IntroRooms,
            ProductionSeed::Flashback(_) => Scope::Flashback,
            ProductionSeed::Discovery(_) => Scope::Discovery,
            ProductionSeed::HealthTail(_) => Scope::HealthTail,
            ProductionSeed::Movement(_) => Scope::Movement,
            ProductionSeed::JumpHistory(_) => Scope::JumpHistory,
            ProductionSeed::MovementTouchdown(_) => Scope::MovementTouchdown,
            ProductionSeed::Audio(_) => Scope::Audio,
            ProductionSeed::WeaponsContact(_) => Scope::WeaponsContact,
            ProductionSeed::IntroMissiles(_) => Scope::IntroMissiles,
            ProductionSeed::Blocks(_) => Scope::Blocks,
            ProductionSeed::Camera(_) => Scope::Camera,
            ProductionSeed::Appearance(_) => Scope::Appearance,
            ProductionSeed::Progress(_) => Scope::Progress,
            ProductionSeed::Hud(_) => Scope::Hud,
            ProductionSeed::Ceres(_) => Scope::Ceres,
            ProductionSeed::Doors(_) => Scope::Doors,
        })?;
        let state = match (seed, &content, &rules) {
            (
                ProductionSeed::Flashback(state),
                ProductionContent::Flashback(_),
                ProductionRules::Flashback(_),
            ) => OwnedState::Flashback(Model(state)),
            (
                ProductionSeed::Playback(phase),
                ProductionContent::Playback(content),
                ProductionRules::Playback,
            ) => {
                if !playback::valid_phase(phase, content) {
                    return Err(AdmissionFault::Playback);
                }
                OwnedState::Playback(Model(phase))
            }
            (
                ProductionSeed::SaveFiles(state),
                ProductionContent::SaveFiles(_),
                ProductionRules::SaveFiles,
            ) => OwnedState::SaveFiles(state),
            (
                ProductionSeed::IntroMissiles(state),
                ProductionContent::IntroMissiles(_),
                ProductionRules::IntroMissiles(_),
            ) => OwnedState::IntroMissiles(state),
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
                ProductionSeed::MovementTouchdown(state),
                ProductionContent::MovementTouchdown(content, audio),
                ProductionRules::MovementTouchdown(_),
            ) => {
                state.validate(content, audio)?;
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
        })
    }

    /// The exact battery image held by a cartridge scope.
    #[must_use]
    pub fn battery(&self) -> Option<types::save_files::BatteryImage> {
        match (&self.state, &self.context.content) {
            (OwnedState::Blackout(state), ProductionContent::Blackout(content)) => Some(
                save_files::image(&content.format, &state.owners.0.files.cartridge),
            ),
            (OwnedState::SaveFiles(state), ProductionContent::SaveFiles(format)) => {
                Some(save_files::image(format, &state.cartridge))
            }
            _ => None,
        }
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
            OwnedState::Blackout(_) => Scope::Blackout,
            OwnedState::Intro(_) => Scope::Intro,
            OwnedState::Playback(_) => Scope::Playback,
            OwnedState::SaveFiles(_) => Scope::SaveFiles,
            OwnedState::Connected(_) => Scope::Connected,
            OwnedState::IntroRooms(_) => Scope::IntroRooms,
            OwnedState::Flashback(_) => Scope::Flashback,
            OwnedState::Discovery(_) => Scope::Discovery,
            OwnedState::HealthTail(_) => Scope::HealthTail,
            OwnedState::Movement(_) => Scope::Movement,
            OwnedState::JumpHistory(_) => Scope::JumpHistory,
            OwnedState::MovementTouchdown(_) => Scope::MovementTouchdown,
            OwnedState::Audio(_) => Scope::Audio,
            OwnedState::WeaponsContact(_) => Scope::WeaponsContact,
            OwnedState::IntroMissiles(_) => Scope::IntroMissiles,
            OwnedState::Blocks(_) => Scope::Blocks,
            OwnedState::Camera(_) => Scope::Camera,
            OwnedState::Appearance(_) => Scope::Appearance,
            OwnedState::Progress(_) => Scope::Progress,
            OwnedState::Hud(_) => Scope::Hud,
            OwnedState::Ceres(_) => Scope::Ceres,
            OwnedState::Doors(_) => Scope::Doors,
        }
    }

    /// Advance a candidate value and commit its state and complete outputs together.
    ///
    /// # Errors
    /// Wrong scope and missing inputs do not advance state.
    pub fn advance(
        &mut self,
        input: AdvanceInput<'_>,
    ) -> Result<ProductionOutputs, ProductionFault> {
        let (candidate, outputs) = self.advance_candidate(self.state.clone(), input)?;
        self.state = candidate;
        Ok(outputs)
    }

    fn advance_candidate(
        &self,
        mut state: OwnedState,
        input: AdvanceInput<'_>,
    ) -> Result<(OwnedState, ProductionOutputs), ProductionFault> {
        let Context { content, rules } = self.context.as_ref();
        let outputs = match (&mut state, content, rules, input) {
            (
                OwnedState::IntroRooms(state),
                ProductionContent::IntroRooms(content),
                ProductionRules::IntroRooms(rules),
                AdvanceInput::IntroRooms(input),
            ) => ProductionOutputs::IntroRooms(
                state
                    .advance(input, content, rules)
                    .map_err(|fault| ProductionFault::Unsupported(Unsupported::IntroRooms(fault)))?
                    .map(Box::new),
            ),

            (
                OwnedState::Flashback(state),
                ProductionContent::Flashback(art),
                ProductionRules::Flashback(rules),
                AdvanceInput::Flashback(cut),
            ) => ProductionOutputs::Flashback(flashback::advance(&mut state.0, art, *rules, cut)),
            (
                OwnedState::Playback(phase),
                ProductionContent::Playback(content),
                ProductionRules::Playback,
                AdvanceInput::Playback(cut),
            ) => ProductionOutputs::Playback(playback::advance(&mut phase.0, content, cut)),
            (
                OwnedState::SaveFiles(state),
                ProductionContent::SaveFiles(format),
                ProductionRules::SaveFiles,
                AdvanceInput::SaveFiles(input),
            ) => ProductionOutputs::SaveFiles(Box::new(state.advance(format, input).map_err(
                |refusal| ProductionFault::Unsupported(Unsupported::SaveFiles(refusal)),
            )?)),
            (
                OwnedState::Connected(state),
                ProductionContent::Connected(content),
                ProductionRules::Connected(rules),
                AdvanceInput::Connected { inputs },
            ) => ProductionOutputs::Connected(Box::new(
                state
                    .advance(inputs, content, rules)
                    .map_err(ProductionFault::Connected)?,
            )),
            (
                OwnedState::Connected(state),
                ProductionContent::Connected(content),
                ProductionRules::Connected(rules),
                AdvanceInput::ConnectedMap { inputs, map },
            ) => ProductionOutputs::Connected(Box::new(
                state
                    .advance_with_map(inputs, content, rules, Some(map))
                    .map_err(ProductionFault::Connected)?,
            )),
            (
                OwnedState::MovementTouchdown(state),
                ProductionContent::MovementTouchdown(content, audio),
                ProductionRules::MovementTouchdown(rules),
                AdvanceInput::MovementTouchdown(input),
            ) => ProductionOutputs::MovementTouchdown(Box::new(movement_touchdown::advance(
                state, content, audio, rules, &input,
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
            (state, content, rules, input) => entries::advance_intro(state, content, rules, input)?,
        };
        Ok((state, outputs))
    }

    /// Read the admitted audio transport, alone or carried with Movement's touchdown.
    #[must_use]
    pub fn transport(&self) -> Option<&audio_transport::State> {
        match &self.state {
            OwnedState::Audio(state) => Some(&state.0),
            OwnedState::MovementTouchdown(state) => Some(&state.audio.0),
            OwnedState::Intro(_)
            | OwnedState::Connected(_)
            | OwnedState::IntroRooms(_)
            | OwnedState::Flashback(_)
            | OwnedState::Playback(_)
            | OwnedState::SaveFiles(_)
            | OwnedState::Discovery(_)
            | OwnedState::Blackout(_)
            | OwnedState::HealthTail(_)
            | OwnedState::Movement(_)
            | OwnedState::JumpHistory(_)
            | OwnedState::WeaponsContact(_)
            | OwnedState::IntroMissiles(_)
            | OwnedState::Blocks(_)
            | OwnedState::Camera(_)
            | OwnedState::Appearance(_)
            | OwnedState::Progress(_)
            | OwnedState::Hud(_)
            | OwnedState::Ceres(_)
            | OwnedState::Doors(_) => None,
        }
    }

    /// Save the selected state and pending work with its immutable context in memory.
    ///
    /// # Errors
    /// A model without a complete codec reports its selected scope explicitly.
    pub fn save(&self) -> Result<SavedProduction, SaveFault> {
        let state = match &self.state {
            OwnedState::Playback(phase) => return Ok(self.encode_save(*phase)),
            OwnedState::SaveFiles(state) => {
                return Ok(self.encode_save(Model::<save_files::SaveFilesPin>(state.clone())));
            }
            OwnedState::Blackout(state) => return Ok(self.encode_save(state.clone())),
            OwnedState::Connected(state) => CodecState::Connected(state.clone()),
            OwnedState::IntroRooms(state) => return Ok(self.encode_save(state.clone())),
            OwnedState::Flashback(state) => return Ok(self.encode_save(state.clone())),
            OwnedState::Discovery(state) => return Ok(self.encode_save(state.clone())),
            OwnedState::HealthTail(state) => return Ok(self.encode_save(state.clone())),
            OwnedState::Movement(state) => CodecState::Movement(*state),
            OwnedState::JumpHistory(state) => return Ok(self.encode_save(*state)),
            OwnedState::Audio(state) => return Ok(self.encode_save(PortableAudio(state.clone()))),
            OwnedState::MovementTouchdown(state) => return Ok(self.encode_save(state.clone())),
            OwnedState::WeaponsContact(state) => CodecState::WeaponsContact(state.clone()),
            OwnedState::IntroMissiles(state) => return Ok(self.encode_save(state.clone())),
            OwnedState::Camera(state) => CodecState::Camera(*state),
            OwnedState::Appearance(state) => CodecState::Appearance(*state),
            OwnedState::Intro(_)
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

    fn encode_save<S: Canonical>(&self, state: S) -> SavedProduction {
        SavedProduction {
            state: canonical::to_bytes(&Checkpoint {
                schema: Schema::new(),
                state,
            }),
            context: self.context.clone(),
        }
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
        let seed = if matches!(&content, ProductionContent::Blackout(_)) {
            let checkpoint: Checkpoint<Box<blackout::State>> =
                canonical::from_bytes(&saved.state).map_err(AdmissionFault::Checkpoint)?;
            ProductionSeed::Blackout(checkpoint.state)
        } else if matches!(&content, ProductionContent::SaveFiles(_)) {
            let checkpoint: Checkpoint<Model<save_files::SaveFilesPin>> =
                canonical::from_bytes(&saved.state).map_err(AdmissionFault::Checkpoint)?;
            ProductionSeed::SaveFiles(checkpoint.state.0)
        } else if matches!(&content, ProductionContent::Flashback(_)) {
            let checkpoint: Checkpoint<Model<crate::state::owned::FlashbackPin>> =
                canonical::from_bytes(&saved.state).map_err(AdmissionFault::Checkpoint)?;
            ProductionSeed::Flashback(checkpoint.state.0)
        } else if matches!(&content, ProductionContent::Playback(_)) {
            let checkpoint: Checkpoint<Model<crate::state::owned::PlaybackPin>> =
                canonical::from_bytes(&saved.state).map_err(AdmissionFault::Checkpoint)?;
            ProductionSeed::Playback(checkpoint.state.0)
        } else if matches!(&content, ProductionContent::IntroMissiles(_)) {
            let checkpoint: Checkpoint<Box<intro_missile_scope::State>> =
                canonical::from_bytes(&saved.state).map_err(AdmissionFault::Checkpoint)?;
            ProductionSeed::IntroMissiles(checkpoint.state)
        } else if matches!(&content, ProductionContent::JumpHistory) {
            let checkpoint: Checkpoint<Model<jump_history::Pin>> =
                canonical::from_bytes(&saved.state).map_err(AdmissionFault::Checkpoint)?;
            ProductionSeed::JumpHistory(checkpoint.state.0)
        } else if matches!(&content, ProductionContent::Discovery(_)) {
            let checkpoint: Checkpoint<Model<crate::state::owned::DiscoveryPin>> =
                canonical::from_bytes(&saved.state).map_err(AdmissionFault::Checkpoint)?;
            ProductionSeed::Discovery(checkpoint.state.0)
        } else if matches!(&content, ProductionContent::Audio(_)) {
            let checkpoint: Checkpoint<PortableAudio> =
                canonical::from_bytes(&saved.state).map_err(AdmissionFault::Checkpoint)?;
            let PortableAudio(state) = checkpoint.state;
            ProductionSeed::Audio(state.0)
        } else if matches!(&content, ProductionContent::MovementTouchdown(..)) {
            let checkpoint: Checkpoint<Box<movement_touchdown::State>> =
                canonical::from_bytes(&saved.state).map_err(AdmissionFault::Checkpoint)?;
            ProductionSeed::MovementTouchdown(checkpoint.state)
        } else if matches!(&content, ProductionContent::IntroRooms(_)) {
            let checkpoint: Checkpoint<Box<intro_rooms::Scoped>> =
                canonical::from_bytes(&saved.state).map_err(AdmissionFault::Checkpoint)?;
            ProductionSeed::IntroRooms(checkpoint.state)
        } else if matches!(&content, ProductionContent::HealthTail(_)) {
            let checkpoint: Checkpoint<Box<health_tail::State>> =
                canonical::from_bytes(&saved.state).map_err(AdmissionFault::Checkpoint)?;
            ProductionSeed::HealthTail(checkpoint.state)
        } else {
            let checkpoint: Checkpoint<CodecState> =
                canonical::from_bytes(&saved.state).map_err(AdmissionFault::Checkpoint)?;
            match checkpoint.state {
                CodecState::Connected(state) => ProductionSeed::Connected(state),
                CodecState::Movement(state) => ProductionSeed::Movement(state.0),
                CodecState::Audio(_) => return Err(AdmissionFault::WrongScope),
                CodecState::WeaponsContact(state) => ProductionSeed::WeaponsContact(state.0),
                CodecState::Camera(state) => ProductionSeed::Camera(state.0),
                CodecState::Appearance(state) => ProductionSeed::Appearance(state.0),
            }
        };
        Self::admit(seed, content, rules)
    }
}

fn validate_pins(scope: Scope) -> Result<(), AdmissionFault> {
    match scope {
        Scope::Flashback => canonical::model::pin_holds::<crate::state::owned::FlashbackPin>(),
        Scope::Playback => canonical::model::pin_holds::<crate::state::owned::PlaybackPin>(),
        Scope::Discovery => canonical::model::pin_holds::<crate::state::owned::DiscoveryPin>(),
        Scope::Connected => {
            use canonical::model::pin_holds;
            pin_holds::<connected::inputs::InputsPin>()
                .and_then(|()| pin_holds::<connected::PlayingPin>())
                .and_then(|()| pin_holds::<crate::state::owned::AppearancePin>())
                .and_then(|()| pin_holds::<hud::HudPin>())
                .and_then(|()| pin_holds::<crate::state::owned::CameraPin>())
                .and_then(|()| pin_holds::<crate::state::owned::MovementPin>())
        }
        Scope::HealthTail => canonical::model::pin_holds::<health_tail::HealthPin>()
            .and_then(|()| canonical::model::pin_holds::<hud::HudPin>()),
        Scope::JumpHistory => canonical::model::pin_holds::<jump_history::Pin>(),
        Scope::Movement => canonical::model::pin_holds::<crate::state::owned::MovementPin>(),
        Scope::MovementTouchdown => {
            canonical::model::pin_holds::<crate::state::owned::MovementPin>()
                .and_then(|()| canonical::model::pin_holds::<movement::FigurePin>())
                .and_then(|()| canonical::model::pin_holds::<crate::state::owned::AudioPin>())
        }
        Scope::Audio => canonical::model::pin_holds::<crate::state::owned::AudioPin>(),
        Scope::WeaponsContact | Scope::IntroMissiles => {
            canonical::model::pin_holds::<crate::state::owned::PowerPin>()
        }
        Scope::Camera => canonical::model::pin_holds::<crate::state::owned::CameraPin>(),
        Scope::Appearance => canonical::model::pin_holds::<crate::state::owned::AppearancePin>(),
        Scope::IntroRooms => intro_rooms::validate_pins(),
        Scope::Intro
        | Scope::Blackout
        | Scope::SaveFiles
        | Scope::Blocks
        | Scope::Progress
        | Scope::Hud
        | Scope::Ceres
        | Scope::Doors => Ok(()),
    }
    .map_err(AdmissionFault::StatePin)
}
