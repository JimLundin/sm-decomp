//! Current frame continuation and admitted immutable selected-owner context.

use super::borrowed_player_preparation::{
    PlayerContinuationCompletionsPin, PlayerContinuationOffersPin, PlayerContinuationOriginsPin,
    PlayerEntryCompletions, PlayerEntryCompletionsPin, PlayerEntryOffers, PlayerEntryOffersPin,
    PlayerEntryOrigins, PlayerEntryOriginsPin,
};
use super::{
    State,
    borrowed_receipts::{
        FrameContinuationAfterNmi, MusicCompletionReceipt, MusicCountdown, MusicQueue, MusicRequest,
    },
};
use crate::production::timing::{self, WorkId};
use crate::production::{ProductionFault, Unsupported};
use canonical::Canonical;
use canonical::model::Model;
use data::timing::MasterTicks;

/// Complete content for the selected movement, appearance and HUD producers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Content {
    /// Finite physical work selected by the current publication acknowledgment.
    pub acknowledgment: data::timing::Template,
    /// Finite physical work selected by the current music service disposition.
    pub music: data::timing::Template,
    /// Physical preparation through narrow bands Ready.
    pub haze: data::timing::Template,
    /// Qualified physical head for the live fade settling opportunity.
    pub settling_haze: Option<data::timing::Template>,
    /// Remaining Haze work after narrow Ready.
    pub haze_tail: data::timing::Template,
    /// Native transfer address of the actual completed bands.
    pub haze_bands_address: data::dma::BusAddress,
    /// Checked current drawing handler templates and immutable settings.
    pub drawing: Option<super::interrupts::DrawingProfiles>,
    /// Current layer-blend physical work.
    pub layer_blend: data::timing::Template,
    /// Actual immutable defaults offered to layer preparation.
    pub layer_defaults: super::layer_blend::LayerBlendDefaults,
    /// Complete physical Random preparation, requests, samples and tail.
    pub random: data::timing::Template,
    /// Qualified original-seed requests and checked immutable multiplier.
    pub random_requests: super::random::SeedRequests,
    /// Remaining current preparation profiles selected by full owner receipts.
    pub preparation: PreparationContent,
    /// Checked physical work and independently imported environment content.
    pub player_entry: PlayerEntryContent,
    /// Whole checked service bodies and separately retained tails.
    pub sender_physical: Option<super::sender_contract::PhysicalContent>,
    /// Checked physical item-control bindings and current switch permission.
    pub early_hud_controls: Option<super::hud_controls_inputs::Bindings>,
    /// Whole quiet early controls owner, selected by the actual HUD answer.
    pub early_hud_owner: Option<data::timing::Template>,
    /// Checked complete physical Inside owner and validation-only source predicates.
    pub inside_physical: Option<super::inside_contract::PhysicalContract>,
    /// Whole remaining player owner envelopes.
    pub player_continuation: Option<super::player_continuation::Content>,
    /// Qualified current facing and whole physical `MotionReset` work.
    pub motion_reset: Option<MotionResetContent>,
    /// Complete local Run predicates and immutable physical work.
    pub run: Option<super::movement_caller::RunContract>,
    /// Independently decoded unchanged room for actual ordinary movement queries.
    pub movement_room: Option<super::movement_caller::Room>,
    /// Whole ordinary right travel selected by actual query and live product.
    pub ordinary_right: Option<super::movement_pair_contract::PhysicalContract>,
    /// Whole compound support qualified by both actual queries and live products.
    pub compound_support: Option<super::movement_support_contract::PhysicalContract>,
    /// Whole ground pull with its actual identity-only local prefix.
    pub ground_pull: Option<super::movement_ground_contract::PhysicalContract>,
    /// Whole actual downward travel with its identity tail.
    pub ground_following_down: Option<super::movement_down_contract::PhysicalContract>,
    /// Whole current stride Contact and its actual Part2 graphical consumer.
    pub contact: Option<super::contact_contract::PhysicalContract>,
    /// Distinct `SoundOffer` and Poststep envelopes, qualified by their actual owners.
    pub post_down: Option<super::post_down_contract::Physical>,
    /// Actual current HUD display capability and producer/readiness/return work.
    pub hud_refresh: Option<super::hud_map_caller::Content>,
    /// Whole actual countdown opportunity after returned HUD display.
    pub countdown: Option<super::countdown_caller::Physical>,
    /// Complete independent health origin and finite neighbouring qualification.
    pub health: Option<super::health::Admission>,
    /// Authenticated original retained liquid history.
    pub liquid_history: Option<super::liquid_history::Admission>,
    /// Complete qualified ordinary Animation owner.
    pub animation: Option<super::appearance_caller::PhysicalContract>,
    /// Independently qualified inactive interruption continuation.
    pub interruption: Option<super::interruption_caller::Contract>,
    /// Complete late movement envelopes for the qualified actual pose choice.
    pub late_movement: Option<super::late_movement::Contract>,
    /// Whole physical rightward aim query and live arithmetic bindings.
    pub aim_right: Option<super::movement_pair_contract::PhysicalContract>,
    /// Shared no-growth body fitting and selected-clip envelope.
    pub pose_fit: Option<super::appearance_caller::PhysicalContract>,
    /// Whole quiet Health, Ceres pause and countdown-tail work.
    pub maintenance: Option<super::maintenance::Contract>,
    /// Source-qualified ordinary Samus colour work.
    pub samus_palette: Option<super::samus_palette::Contract>,
    /// Whole S6 empty-neighbour and actual Camera envelopes.
    pub s6_camera: Option<super::s6_camera::Contract>,
    /// Qualified live Drawing bridge and aggregate.
    pub s6_drawing: Option<super::s6_drawing::Contract>,
    /// Current applied background preparation.
    pub background: Option<super::background_caller::Contract>,
    /// Complete immutable finalization and native continuation.
    pub return_work: Option<super::frame_return::Contract>,
    /// Checked actual footstep output binding.
    pub sender_cue: Option<super::sender_stage::CueBinding>,
    /// Whole actual late HUD and checked transfer.
    pub late_hud: Option<super::late_hud::Contract>,
    /// Whole room and block preparation before play time.
    pub room_tail: Option<super::room_tail::Contract>,
    /// Checked no-carry clock opportunity envelope.
    pub play_time: Option<super::play_time::Contract>,
    /// Checked quiet room-quake opportunity envelope.
    pub room_quake: Option<super::room_quake::Contract>,
    /// Actual final quiet list and history contracts.
    pub frame_tail: Option<super::frame_tail::Contract>,
    /// Independently admitted initial scene scheduling for this expressly borrowed scope.
    pub scene_countdown: Option<super::countdown_scene::Admission>,
    /// Checked body and cannon content.
    pub movement: crate::production::MovementContent,
    /// Checked semantic figures and clips.
    pub appearance: crate::production::AppearanceContent,
    /// Actual HUD pictures.
    pub hud: Box<interface_hud::HudPictures>,
}

/// Physical work for the remaining distinct preparation owners.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparationContent {
    /// Actual builder reset work.
    pub builder: data::timing::Template,
    /// Actual current gameplay route work.
    pub gameplay: data::timing::Template,
    /// Actual full ordered actor eligibility work.
    pub actors: data::timing::Template,
    /// Actual current debug work.
    pub debug: data::timing::Template,
    /// Actual current palette work.
    pub palette: data::timing::Template,
}

/// Full physical entry owners; their effects remain unread until matching completion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerEntryContent {
    /// Full ordinary pose input.
    pub pose_input: data::timing::Template,
    /// Distinct finite pose work after releasing aim while continuing to run.
    pub recurring_pose_input: Option<data::timing::Template>,
    /// Full suit palette preparation.
    pub suit_palette: data::timing::Template,
    /// Full environment preparation.
    pub environment: data::timing::Template,
    /// Acceleration imported from checked reference content.
    pub reference_acceleration: types::Fixed,
}

/// Actual ordered S2 entry opportunity.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerEntryStage {
    /// Ordinary input, preserving the current playing pose.
    PoseInput,
    /// Suit palette.
    SuitPalette,
    /// Environment rules.
    Environment,
    /// Selected Inside queries are required next.
    Inside,
    /// Actual early HUD item controls follow the completed Inside owner.
    HudWeapons,
}

impl PlayerEntryStage {
    pub(super) const fn identity(self) -> WorkId {
        WorkId(match self {
            Self::PoseInput => 12,
            Self::SuitPalette => 13,
            Self::Environment => 14,
            Self::Inside => 15,
            Self::HudWeapons => 16,
        })
    }
}

impl PlayerEntryContent {
    const fn profile(&self, stage: PlayerEntryStage) -> Option<&data::timing::Template> {
        match stage {
            PlayerEntryStage::PoseInput => Some(&self.pose_input),
            PlayerEntryStage::SuitPalette => Some(&self.suit_palette),
            PlayerEntryStage::Environment => Some(&self.environment),
            PlayerEntryStage::Inside | PlayerEntryStage::HudWeapons => None,
        }
    }
}

impl PreparationStage {
    const fn identity(self) -> WorkId {
        WorkId(match self {
            Self::Builder => 7,
            Self::Gameplay => 8,
            Self::Actors => 9,
            Self::Debug => 10,
            Self::Palette => 11,
        })
    }
    const fn dependency(self) -> Dependency {
        match self {
            Self::Builder => Dependency::BuilderPreparation,
            Self::Gameplay => Dependency::GameplayPreparation,
            Self::Actors => Dependency::ActorPreparation,
            Self::Debug => Dependency::DebugPreparation,
            Self::Palette => Dependency::PalettePreparation,
        }
    }
}

impl PreparationContent {
    const fn profile(&self, stage: PreparationStage) -> &data::timing::Template {
        match stage {
            PreparationStage::Builder => &self.builder,
            PreparationStage::Gameplay => &self.gameplay,
            PreparationStage::Actors => &self.actors,
            PreparationStage::Debug => &self.debug,
            PreparationStage::Palette => &self.palette,
        }
    }
}

/// Immutable rules bound with the complete frame checkpoint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rules {
    /// Movement tuning.
    pub movement: samus_movement::Rules,
    /// Appearance tuning.
    pub appearance: samus_appearance::Rules,
    /// HUD tuning.
    pub hud: crate::production::HudRules,
}

/// Current explicitly borrowed preparation owners and unread external completion.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Prelude {
    /// Independently borrowed completion, unread until the actual music opportunity.
    pub music_completion: Option<MusicCompletionReceipt>,
    /// Current gameplay route offer, unread until actual dispatch.
    pub gameplay_completion: Option<super::borrowed_gameplay::GameplayDispatchOffer>,
    /// Actual full actor membership offer.
    pub actor_completion: Option<super::borrowed_preparation::ActorEligibilityReceipt>,
    /// Actual current debug no-action completion.
    pub debug_completion: Option<super::borrowed_preparation::DebugCompletionReceipt>,
    /// Actual current palette disposition.
    pub palette_completion: Option<super::borrowed_preparation::PaletteCompletionReceipt>,
    /// Independent current entry owners, refreshed against selected owners at consumption.
    pub player_entry_current: Option<Model<PlayerEntryOriginsPin>>,
    /// Separately borrowed entry effects.
    pub player_entry_offers: Option<Model<PlayerEntryOffersPin>>,
    /// Independently current unselected remaining player owners.
    pub player_continuation_current: Option<Model<PlayerContinuationOriginsPin>>,
    /// Separately qualified remaining player effects, unread before their actual opportunities.
    pub player_continuation_offers: Option<Model<PlayerContinuationOffersPin>>,
}

/// Completed preparation history and still-current borrowed owners.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Preparation {
    /// Current available inputs for later opportunities.
    pub prelude: PreludeSource,
    /// Completed prior acknowledgment.
    pub publication: FrameContinuationAfterNmi,
    /// Completed Music.
    pub music: MusicCompletionReceipt,
    /// Completed Haze.
    pub haze: super::haze::Prepared,
    /// Completed layers.
    pub layers: super::layer_blend::LayerBlendReceipt,
    /// Actual completed builder reset, absent until its physical work completes.
    pub builder: Option<super::borrowed_preparation::BuilderCompletionReceipt>,
    /// Actual completed scene route.
    pub gameplay: Option<super::borrowed_gameplay::GameplayDispatchReceipt>,
    /// Actual completed ordered memberships.
    pub actors: Option<super::borrowed_preparation::ActorEligibilityReceipt>,
    /// Actual completed debug disposition.
    pub debug: Option<super::borrowed_preparation::DebugCompletionReceipt>,
    /// Actual completed palette disposition.
    pub palette: Option<super::borrowed_preparation::PaletteCompletionReceipt>,
}

impl Preparation {
    fn entry_origins(&self) -> Option<&PlayerEntryOrigins> {
        Some(&self.prelude.origin()?.player_entry_current.as_ref()?.0)
    }

    fn entry_offers(&self) -> Option<&PlayerEntryOffers> {
        Some(&self.prelude.origin()?.player_entry_offers.as_ref()?.0)
    }
}

/// Current remaining preparation opportunity, independent of source coordinates.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreparationStage {
    /// Reset the current builder.
    Builder,
    /// Read the current gameplay controller route.
    Gameplay,
    /// Complete current ordered actor eligibility.
    Actors,
    /// Complete current debug controls.
    Debug,
    /// Complete current palette slots.
    Palette,
}

/// Actual current pose history captured when qualified motion preparation completes.
/// The snapshot is an immutable receipt, never the writable Movement authority.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct MotionResetCompletion {
    /// Actual completed physical owner identity.
    pub work: WorkId,
    /// Actual prior pose, facing and action, before prospective pose placement.
    pub previous: canonical::model::Model<crate::state::owned::MovementPin>,
    /// Actual completed discharge of transient new-clip-start preparation.
    pub clip_start: super::motion_reset_contract::ClipStartPreparation,
}

/// Validation-only current facing and complete motion preparation envelope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MotionResetContent {
    /// Current source-qualified facing, checked against the actual movement owner.
    pub facing: super::motion_reset_contract::CurrentFacing,
    /// Every exposure of the complete physical owner.
    pub whole: data::timing::Template,
}

/// Prelude inputs available at the current opportunity.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub enum PreludeSource {
    /// Current neighbour effects at the admitted start.
    Origin(Box<Prelude>),
    /// Current neighbours must come from the retained game.
    Recurring,
}

impl PreludeSource {
    /// Scoped origin inputs that remain available in the first opportunity.
    #[must_use]
    pub const fn origin(&self) -> Option<&Prelude> {
        match self {
            Self::Origin(prelude) => Some(prelude),
            Self::Recurring => None,
        }
    }

    /// Scoped origin inputs still owned by the active first opportunity.
    #[must_use]
    pub fn origin_mut(&mut self) -> Option<&mut Prelude> {
        match self {
            Self::Origin(prelude) => Some(prelude),
            Self::Recurring => None,
        }
    }
}

/// Actual software continuation; hardware pending interrupts have a separate owner.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub enum Caller {
    /// The current frame-publication owner must acknowledge its prior software request.
    /// Origin inputs alone do not establish acknowledgment or prepared producer results.
    AwaitingAcknowledgment {
        /// Current unselected producer origins, still unread by preparation.
        prelude: PreludeSource,
        /// Current actual software owner receipt, absent when unavailable.
        receipt: Option<FrameContinuationAfterNmi>,
    },
    /// The acknowledged caller's physical work is retained by the hardware owner.
    Acknowledgment {
        /// Still-needed preparation origins.
        prelude: PreludeSource,
        /// Consumed owner receipt, never reconsidered while charging.
        receipt: FrameContinuationAfterNmi,
    },
    /// Prior publication continuation completed; current music work must be admitted next.
    Music {
        /// Still-needed preparation origins.
        prelude: PreludeSource,
        /// Completed prior owner receipt.
        publication: FrameContinuationAfterNmi,
    },
    /// The selected current music service is physically charging before borrowed completion.
    MusicRunning {
        /// Current unselected owners and actual dependency chosen for this service.
        prelude: PreludeSource,
        /// Completed prior publication acknowledgment.
        publication: FrameContinuationAfterNmi,
    },
    /// Music completed; the current Haze opportunity can be selected.
    Haze {
        /// Current preparation owners after completed music work.
        prelude: PreludeSource,
        /// Completed prior acknowledgment.
        publication: FrameContinuationAfterNmi,
        /// Actual consumed music completion, retained independently from unread inputs.
        music: MusicCompletionReceipt,
    },
    /// Actual Haze preparation is physically charging.
    HazeRunning {
        /// Current available neighbour inputs.
        prelude: PreludeSource,
        /// Completed publication acknowledgment.
        publication: FrameContinuationAfterNmi,
        /// Completed Music service.
        music: MusicCompletionReceipt,
        /// Private selected result while the head charges.
        work: super::haze::Work,
    },
    /// Bands are Ready; the owner tail remains active physical work.
    HazeTail {
        /// Current available neighbour inputs.
        prelude: PreludeSource,
        /// Prior acknowledgment.
        publication: FrameContinuationAfterNmi,
        /// Prior Music service.
        music: MusicCompletionReceipt,
        /// Current display preparation, with transport still charging.
        prepared: super::haze::Prepared,
    },
    /// Haze fully completed; layer preparation is the next actual prerequisite.
    LayerBlend {
        /// Current owners after full Haze completion.
        prelude: PreludeSource,
        /// Prior acknowledgment.
        publication: FrameContinuationAfterNmi,
        /// Prior Music service.
        music: MusicCompletionReceipt,
        /// Consumed Haze outputs, retained for later prepared video.
        haze: super::haze::Prepared,
    },
    /// Current layer operation is physically charging.
    LayerBlendRunning {
        /// Remaining current owners.
        prelude: PreludeSource,
        /// Completed acknowledgment.
        publication: FrameContinuationAfterNmi,
        /// Completed Music.
        music: MusicCompletionReceipt,
        /// Completed Haze.
        haze: super::haze::Prepared,
        /// Current prepared effects held until the physical completion.
        layers: super::layer_blend::LayerBlendReceipt,
    },
    /// Layers completed; current Random hardware work is needed next.
    Random {
        /// Remaining current owners.
        prelude: PreludeSource,
        /// Completed acknowledgment.
        publication: FrameContinuationAfterNmi,
        /// Completed Music.
        music: MusicCompletionReceipt,
        /// Completed Haze.
        haze: super::haze::Prepared,
        /// Actual completed prepared layer settings.
        layers: super::layer_blend::LayerBlendReceipt,
    },
    /// The current Random owner is charging with actual device samples retained.
    RandomRunning {
        /// Current remaining owners.
        prelude: PreludeSource,
        /// Completed acknowledgment.
        publication: FrameContinuationAfterNmi,
        /// Completed Music.
        music: MusicCompletionReceipt,
        /// Completed Haze.
        haze: super::haze::Prepared,
        /// Completed prepared layers.
        layers: super::layer_blend::LayerBlendReceipt,
        /// Seed used to bind the outstanding physical requests.
        requested: super::random::RandomSeed,
        /// Actual live result bytes, preserved through pauses and interrupts.
        samples: super::random::Samples,
    },
    /// Random completed; builder reset is the next actual owner opportunity.
    Preparation {
        /// Current owners and prior completed preparation.
        prepared: Preparation,
        /// Current actual software opportunity.
        stage: PreparationStage,
        /// Its physical work has begun and remains retained in hardware.
        charging: bool,
    },
    /// All S1 preparation completed; selected player observations are needed next.
    Player {
        /// Actual completed preparation and current builder/memberships.
        prepared: Preparation,
        /// Current full entry operation.
        stage: PlayerEntryStage,
        /// The operation is retained in hardware.
        charging: bool,
        /// Actual completed entry receipts.
        completed: Model<PlayerEntryCompletionsPin>,
        /// Matching actual hardware identities, in completed owner order.
        completion_work: Vec<WorkId>,
    },
    /// Inside and early controls completed; remaining S2 preparation is required.
    PlayerContinuation {
        /// Actual completed S1 owners and still-current external owners.
        prepared: Box<Preparation>,
        /// Actual first three completed player owners.
        completed: Model<PlayerEntryCompletionsPin>,
        /// Matching first three actual hardware identities.
        completion_work: Vec<WorkId>,
        /// Full actual early controls answer and its physical completion identity.
        controls: super::hud_stage::ControlsCompletion,
        /// Actual next remaining player opportunity.
        stage: super::player_continuation::Stage,
        /// Its full physical envelope remains retained in hardware.
        charging: bool,
        /// Actual completed remaining owners in their causal order.
        remaining: Model<PlayerContinuationCompletionsPin>,
        /// Actual physical identities returned by completed remaining owners.
        remaining_work: Vec<WorkId>,
        /// Actual motion preparation, absent before its physical owner completes.
        motion_reset: Option<super::MotionResetCompletion>,
        /// Actual current-playing query and local Run decisions retained through billing.
        run: super::movement_caller::Run,
        /// Actual paired ordinary move, absent until its current opportunity.
        ordinary: Option<super::OrdinaryWork>,
        /// Single physical owner for both actual support answers.
        support: super::movement_support_contract::Stage,
        /// Retained ground pull after the complete support owner.
        ground: super::movement_ground_contract::Stage,
        /// Actual downward request, answer and identity tail.
        down: super::movement_down_contract::Stage,
        /// Actual distinct Contact, sound offer and poststep owners after Down.
        post_down: super::post_down_caller::Stage,
    },
}

/// A current causal prerequisite absent from the admitted frame continuation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dependency {
    /// The preceding software publication has no valid acknowledgment.
    FramePublicationAcknowledgment,
    /// Current music disposition does not yet include admitted actual owner effects.
    MusicPreparation,
    /// The current Haze owner has no independently admitted preparation/completion effects.
    HazePreparation,
    /// Current completed Haze blend requires its next borrowed producer.
    LayerBlendPreparation,
    /// Missing shared Random sequence, invalid timing, or inconsistent physical samples.
    RandomPreparation,
    /// Complete current builder reset and its physical work are unavailable.
    BuilderPreparation,
    /// The current gameplay controller has no admitted route.
    GameplayPreparation,
    /// Current full actor eligibility completion is unavailable.
    ActorPreparation,
    /// Current debug completion is unavailable.
    DebugPreparation,
    /// Current palette completion is unavailable.
    PalettePreparation,
    /// Current selected player inputs are not yet supplied at their actual opportunity.
    PlayerPreparation,
    /// Entry preparation completed; current selected Inside work is required.
    InsidePreparation,
    /// Actual early HUD item controls require their next qualified physical owner.
    HudWeaponsPreparation,
    /// Fresh display inputs or full HUD producer/return ownership is unavailable.
    HudDisplayPreparation,
    /// Actual current scene choice or its bounded preservation is unavailable.
    CountdownSceneScheduling,
    /// A selected countdown still requires its actual clean producer.
    CountdownPreparation,
    /// Current animation has no integrated completion following countdown.
    AnimationPreparation,
    /// Current health or its finite actor/interrupt/protection source join is unavailable.
    HealthCurrentInputs,
    /// Retained liquid origin or actual preservation receipts are unavailable.
    AnimationLiquidHistory,
    /// Actual interruption opportunity following Animation is unavailable.
    InterruptionPreparation,
    /// Late ordinary movement has no qualified current physical continuation.
    AimPreparation,
    /// The actual current pose fit and selected clip are unavailable.
    PoseFitPreparation,
    /// Ordinary current palette lacks its actual model and current producers.
    SamusPalettePreparation,
    /// Health maintenance lacks a fresh owner or a qualified physical return.
    HealthMaintenance,
    /// The actual Camera call or its qualified predecessors are unavailable.
    CameraPreparation,
    /// The bounded current owners preceding Camera are unavailable.
    CameraNeighbourOwners,
    /// The actual complete movement centre has no qualified Camera envelope.
    CameraCurrentCentre,
    /// The actual previous centre or placement has no qualified Camera envelope.
    CameraPreviousCentre,
    /// The Camera room and scroll permissions failed checked construction.
    CameraRoom,
    /// The actual Camera dependencies or physical profile are unqualified.
    CameraPhysical,
    /// The Camera owner could not consume its live arithmetic samples.
    CameraLiveSample,
    /// The current held Camera receipt disagrees with its producer.
    CameraHeldInput,
    /// The current live Camera bill differs from the qualified physical profile.
    CameraHeldProfile,
    /// The qualified live Drawing aggregate lacks a current dependency.
    DrawingPreparation,
    /// Full current Drawing controller premises differ.
    DrawingCurrentControls,
    /// Actual current collection, Health or Drawing premises are missing.
    DrawingCurrentOwners,
    /// Actual clean Drawing Appearance work could not be retained.
    DrawingAppearance,
    /// Selected Draw results could not enter the renderer and current builder.
    DrawingRendering,
    /// Actual Drawing returned and the late HUD owner is unavailable.
    LateHudPreparation,
    /// Actual Background arithmetic lacks its current owners.
    BackgroundPreparation,
    /// Required actual producer for OAM finalization or native return is missing.
    NativeReturnPreparation,
    /// The actual three ordered Sender services lack their current work.
    FrameSenderPreparation,
    /// Actual retained play timer service requires a live qualified model.
    PlayTimePreparation,
    /// Actual room and layer ownership did not qualify post-background work.
    RoomPreparation,
    /// Actual room-shake phase service requires a live qualified model.
    RoomShakePreparation,
    /// Current hurt timers, actual actor list or final held history failed qualification.
    FinalFrameHistory,
    /// Remaining player preparation requires its actual current dependencies.
    PlayerContinuationPreparation,
    /// Recurring sprite-object owners require independent qualification.
    SpriteObjectsPreparation,
    /// An interrupt requires a current admitted handler owner.
    InterruptOwner,
}

/// Production completion, independent of expected wall duration or callback count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Progress {
    /// A retained operation remains active.
    Advanced,
    /// The actual caller completed its frame and native return.
    Completed,
}

/// Fresh physical output delta committed with the complete continuation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outputs {
    /// Actual caller completion.
    pub progress: Progress,
    /// Every newly delivered physical event in its production order.
    pub events: Vec<crate::production::timing::Event>,
}

const ACKNOWLEDGMENT: WorkId = WorkId(1);
const MUSIC: WorkId = WorkId(2);
const HAZE: WorkId = WorkId(3);
const HAZE_TAIL: WorkId = WorkId(4);
const DRAWING: WorkId = WorkId(100);
const LAYERS: WorkId = WorkId(5);
const RANDOM: WorkId = WorkId(6);

impl State {
    fn motion_reset_profile(
        &self,
        content: &Content,
    ) -> Result<data::timing::Profile, ProductionFault> {
        let fault = ProductionFault::Unsupported(Unsupported::Frame(
            Dependency::PlayerContinuationPreparation,
        ));
        self.validate_remaining_player(content).map_err(|_| fault)?;
        let Caller::PlayerContinuation {
            prepared,
            completed,
            controls,
            remaining,
            stage: super::player_continuation::Stage::MotionReset,
            motion_reset: None,
            run: super::movement_caller::Run::Idle,
            ordinary: None,
            support: super::movement_support_contract::Stage::Idle,
            ground: super::movement_ground_contract::Stage::Idle,
            down: super::movement_down_contract::Stage::Idle,
            ..
        } = &self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        let inside = self.inside_completed.as_ref().ok_or(fault)?;
        let selected =
            super::borrowed_player_preparation::CompletedInsideAndHudWeapons { inside, controls };
        self.current_projectiles(prepared, &selected, remaining.0.projectiles.as_ref())?;
        if remaining.0.projectiles.is_none() || self.prepared_action.is_some() {
            return Err(fault);
        }
        let source = content.motion_reset.as_ref().ok_or(fault)?;
        super::motion_reset_contract::qualify(
            super::motion_reset_contract::Current {
                movement: &self.movement,
                inside,
                hud: &self.hud,
                controls,
                bindings: content.early_hud_controls.as_ref().ok_or(fault)?,
                control_inputs: super::hud_controls_inputs::CurrentControls {
                    gameplay: prepared.gameplay.as_ref().ok_or(fault)?,
                    scene: self.scene.ok_or(fault)?,
                    pose_input: completed.0.pose_input.as_ref().ok_or(fault)?,
                    pose: self.movement.state().pose,
                    inventory: self.hud_inventory.as_ref().ok_or(fault)?,
                },
            },
            source.facing,
            &source.whole,
        )
        .map_err(|_| fault)
    }

    fn begin_motion_reset(&mut self, content: &Content) -> Result<(), ProductionFault> {
        if !matches!(
            self.caller,
            Caller::PlayerContinuation {
                stage: super::player_continuation::Stage::MotionReset,
                charging: false,
                ..
            }
        ) {
            return Err(ProductionFault::WrongScope);
        }
        let profile = self.motion_reset_profile(content)?;
        self.timing
            .begin(WorkId(18), profile)
            .map_err(ProductionFault::FrameHardware)?;
        let Caller::PlayerContinuation { charging, .. } = &mut self.caller else {
            return Err(ProductionFault::WrongScope);
        };
        *charging = true;
        Ok(())
    }

    fn validate_motion_reset(&self, content: &Content) -> Result<(), ProductionFault> {
        let profile = self.motion_reset_profile(content)?;
        let retained: Vec<_> = self
            .timing
            .work
            .iter()
            .filter(|work| work.kind == timing::WorkKind::Semantic(WorkId(18)))
            .collect();
        if !matches!(retained.as_slice(), [work] if work.profile == profile) {
            return Err(ProductionFault::Unsupported(Unsupported::Frame(
                Dependency::PlayerContinuationPreparation,
            )));
        }
        Ok(())
    }

    fn finish_motion_reset(
        &mut self,
        work: WorkId,
        content: &Content,
    ) -> Result<(), ProductionFault> {
        if work != WorkId(18)
            || !matches!(
                self.caller,
                Caller::PlayerContinuation {
                    stage: super::player_continuation::Stage::MotionReset,
                    charging: true,
                    ..
                }
            )
        {
            return Err(ProductionFault::WrongScope);
        }
        let _ = self.motion_reset_profile(content)?;
        let completion = self.complete_motion_reset(work).map_err(|_| {
            ProductionFault::Unsupported(Unsupported::Frame(
                Dependency::PlayerContinuationPreparation,
            ))
        })?;
        let Caller::PlayerContinuation {
            motion_reset,
            remaining_work,
            stage,
            charging,
            ..
        } = &mut self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        *motion_reset = Some(completion);
        remaining_work.push(work);
        *stage = super::player_continuation::Stage::Interaction;
        *charging = false;
        Ok(())
    }

    fn begin_inside(&mut self, content: &Content, rules: &Rules) -> Result<(), ProductionFault> {
        let fault = ProductionFault::Unsupported(Unsupported::Frame(Dependency::InsidePreparation));
        let Caller::Player {
            prepared,
            stage: PlayerEntryStage::Inside,
            charging: false,
            completed,
            completion_work,
        } = &self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        let prepared = prepared.clone();
        let completed = completed.clone();
        let completion_work = completion_work.clone();
        let pose = completed.0.pose_input.as_ref().ok_or(fault)?;
        let environment = completed.0.environment.as_ref().ok_or(fault)?;
        if content.movement.radius(self.movement.state().pose) != Some(pose.terminal.radius)
            || self.inside_pending.is_some()
            || self.inside_completed.is_some()
        {
            return Err(fault);
        }
        let room = self.inside_world.as_ref().ok_or(fault)?.0.clone();
        if !self.publications.original_interval() {
            let ahead = self
                .current_animation(&content.appearance)
                .map_err(|_| fault)?;
            let premises = self.inside_premises.ok_or(fault)?;
            self.inside_inputs = Some(Model(premises.at_inside(
                pose,
                environment,
                self.playing_figure.0.sprite,
                ahead,
            )));
        }
        let inputs = self.inside_inputs.as_ref().ok_or(fault)?.0;
        let contract = content.inside_physical.as_ref().ok_or(fault)?;
        let radius = pose.terminal.radius;
        // The checked movement adapter replaces both closures from immutable content.
        let body = |_| radius;
        let heading = |_| None;
        let frame = inputs.frame(&body, &heading);
        let mut movement_rules = rules.movement.clone();
        movement_rules.gravity = environment.terminal.acceleration;
        self.movement.begin().map_err(|_| fault)?;
        let receipt = room
            .choose(
                &mut self.movement_stage,
                &mut self.movement,
                &content.movement,
                &movement_rules,
                &frame,
            )
            .map_err(|_| fault)?
            .clone();
        let requests = super::inside_contract::Requests::from_receipt(&receipt, room.width())
            .map_err(|_| fault)?;
        let profile = contract
            .physical(
                &receipt,
                &self.movement,
                &content.movement,
                &room,
                pose,
                self.inside_geometry(),
            )
            .map_err(|_| fault)?;
        let identity = PlayerEntryStage::Inside.identity();
        self.timing
            .begin(identity, profile)
            .map_err(ProductionFault::FrameHardware)?;
        self.movement_stage.bind(identity).map_err(|_| fault)?;
        self.inside_pending = Some(super::inside_contract::Pending::new(identity, requests));
        self.caller = Caller::Player {
            prepared,
            stage: PlayerEntryStage::Inside,
            charging: true,
            completed,
            completion_work,
        };
        Ok(())
    }

    fn complete_inside(
        &mut self,
        identity: WorkId,
        content: &Content,
        rules: &Rules,
    ) -> Result<(), ProductionFault> {
        let fault = ProductionFault::Unsupported(Unsupported::Frame(Dependency::InsidePreparation));
        let Caller::Player {
            prepared,
            stage: PlayerEntryStage::Inside,
            charging: true,
            completed,
            completion_work,
        } = &self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        if identity != PlayerEntryStage::Inside.identity() || self.inside_completed.is_some() {
            return Err(fault);
        }
        self.inside_pending
            .as_ref()
            .ok_or(fault)?
            .complete(identity)
            .map_err(|_| fault)?;
        let prepared = prepared.clone();
        let completed = completed.clone();
        let completion_work = completion_work.clone();
        let environment = completed.0.environment.as_ref().ok_or(fault)?;
        let room = self.inside_world.as_ref().ok_or(fault)?.0.clone();
        let inputs = self.inside_inputs.as_ref().ok_or(fault)?.0;
        let radius = content
            .movement
            .radius(self.movement.state().pose)
            .ok_or(fault)?;
        let pose = completed.0.pose_input.as_ref().ok_or(fault)?;
        content
            .inside_physical
            .as_ref()
            .ok_or(fault)?
            .physical(
                self.movement_stage.receipt().ok_or(fault)?,
                &self.movement,
                &content.movement,
                &room,
                pose,
                self.inside_geometry(),
            )
            .map_err(|_| fault)?;
        let body = |_| radius;
        let heading = |_| None;
        let frame = inputs.frame(&body, &heading);
        let mut movement_rules = rules.movement.clone();
        movement_rules.gravity = environment.terminal.acceleration;
        let completion = room
            .complete(
                &mut self.movement_stage,
                identity,
                &mut self.movement,
                &content.movement,
                &movement_rules,
                &frame,
            )
            .map_err(|_| fault)?;
        self.inside_pending = None;
        self.inside_completed = Some(completion);
        self.caller = Caller::Player {
            prepared,
            stage: PlayerEntryStage::HudWeapons,
            charging: false,
            completed,
            completion_work,
        };
        Ok(())
    }

    pub(super) const fn inside_geometry(&self) -> super::inside_contract::GeometryMatch {
        use super::inside_contract::GeometryMatch;
        if self.publications.original_interval() {
            GeometryMatch::Origin
        } else {
            GeometryMatch::Recurring
        }
    }

    pub(super) fn validate_player_entry(&self) -> Result<(), super::Fault> {
        let pose_charging = matches!(
            self.caller,
            Caller::Player {
                stage: PlayerEntryStage::PoseInput,
                charging: true,
                ..
            },
        );
        if pose_charging
            != self
                .pose_input_configuration
                .is_some_and(super::pose_input::Configuration::charging)
        {
            return Err(super::Fault::InvalidPlayerEntry);
        }
        if matches!(self.caller, Caller::PlayerContinuation { .. }) {
            return self.validate_continuation_entry();
        }
        let Caller::Player {
            prepared,
            stage,
            charging,
            completed,
            completion_work,
        } = &self.caller
        else {
            return Ok(());
        };
        let expected = match stage {
            PlayerEntryStage::PoseInput => [false, false, false],
            PlayerEntryStage::SuitPalette => [true, false, false],
            PlayerEntryStage::Environment => [true, true, false],
            PlayerEntryStage::Inside | PlayerEntryStage::HudWeapons => [true, true, true],
        };
        let actual = [
            completed.0.pose_input.is_some(),
            completed.0.suit_palette.is_some(),
            completed.0.environment.is_some(),
        ];
        let active = self
            .timing
            .work
            .iter()
            .any(|work| work.kind == timing::WorkKind::Semantic(stage.identity()));
        let expected_work = [WorkId(12), WorkId(13), WorkId(14)];
        let completed_count = expected.into_iter().filter(|done| *done).count();
        if Some(completion_work.as_slice()) != expected_work.get(..completed_count)
            || expected != actual
            || *charging != active
            || prepared.gameplay.is_none()
            || (!matches!(
                stage,
                PlayerEntryStage::Inside | PlayerEntryStage::HudWeapons
            ) && prepared
                .prelude
                .origin()
                .is_some_and(|origin| origin.player_entry_current.is_some())
                && !self.entry_current_matches(prepared, *stage))
        {
            return Err(super::Fault::InvalidPlayerEntry);
        }
        if *stage == PlayerEntryStage::HudWeapons
            && (self
                .inside_completed
                .as_ref()
                .is_none_or(|receipt| receipt.work != WorkId(15))
                || (*charging
                    && !matches!(
                        self.hud.controls(),
                        Some(interface_hud::Observation::Chosen(_))
                    )))
        {
            return Err(super::Fault::InvalidPlayerEntry);
        }
        Ok(())
    }

    pub(super) fn validate_prelude(&self) -> Result<(), super::Fault> {
        let (source, expected) = match &self.caller {
            Caller::AwaitingAcknowledgment { prelude, .. }
            | Caller::Music { prelude, .. }
            | Caller::Haze { prelude, .. }
            | Caller::LayerBlend { prelude, .. }
            | Caller::Random { prelude, .. } => (prelude, None),
            Caller::Acknowledgment { prelude, .. } => (prelude, Some(ACKNOWLEDGMENT)),
            Caller::MusicRunning { prelude, .. } => (prelude, Some(MUSIC)),
            Caller::HazeRunning { prelude, .. } => (prelude, Some(HAZE)),
            Caller::HazeTail { prelude, .. } => (prelude, Some(HAZE_TAIL)),
            Caller::LayerBlendRunning { prelude, .. } => (prelude, Some(LAYERS)),
            Caller::RandomRunning { .. }
            | Caller::Preparation { .. }
            | Caller::Player { .. }
            | Caller::PlayerContinuation { .. } => return Ok(()),
        };
        let original = matches!(source, PreludeSource::Origin(_));
        if original != self.publications.original_interval() {
            return Err(super::Fault::InvalidPlayerEntry);
        }
        let work = self.timing.work.as_slice();
        if !original && !expected.map_or(work.is_empty(), |identity| {
            matches!(work, [work] if work.kind == timing::WorkKind::Semantic(identity))
        }) {
            return Err(super::Fault::InvalidPlayerEntry);
        }
        Ok(())
    }

    pub(super) fn validate_random_work(&self) -> Result<(), super::Fault> {
        let mut random_work = self
            .timing
            .work
            .iter()
            .filter(|work| work.kind == timing::WorkKind::Semantic(RANDOM));
        let work = random_work.next();
        let Caller::RandomRunning { requested, .. } = &self.caller else {
            return if work.is_none() {
                Ok(())
            } else {
                Err(super::Fault::InvalidRandomWork)
            };
        };
        let work = work.ok_or(super::Fault::InvalidRandomWork)?;
        if random_work.next().is_some()
            || !self
                .random
                .as_ref()
                .is_some_and(|sequence| sequence.current() == *requested)
        {
            return Err(super::Fault::InvalidRandomWork);
        }
        super::random::validate_profile(*requested, &work.profile)
            .map_err(|_| super::Fault::InvalidRandomWork)
    }
    fn validate_continuation_entry(&self) -> Result<(), super::Fault> {
        let Caller::PlayerContinuation {
            prepared,
            completed,
            completion_work,
            controls,
            stage,
            charging,
            remaining,
            remaining_work,
            motion_reset,
            ..
        } = &self.caller
        else {
            return Err(super::Fault::InvalidPlayerEntry);
        };
        let count = match stage {
            super::player_continuation::Stage::Projectiles => 0,
            super::player_continuation::Stage::MotionReset => 1,
            super::player_continuation::Stage::Interaction => 2,
            super::player_continuation::Stage::Actors => 3,
            super::player_continuation::Stage::Sprites => 4,
            super::player_continuation::Stage::Contact => 5,
            super::player_continuation::Stage::Run => 6,
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
            || Some(remaining_work.as_slice()) != expected_work.get(..count)
            || actual != expected
            || motion_reset.is_some() != (count >= 2)
            || motion_reset
                .as_ref()
                .is_some_and(|completion| completion.work != WorkId(18))
            || prepared
                .prelude
                .origin()
                .is_some_and(|origin| origin.player_continuation_current.is_some())
                != prepared
                    .prelude
                    .origin()
                    .is_some_and(|origin| origin.player_continuation_offers.is_some())
            || ((count != 0 || *charging) && !self.has_remaining_player_source(prepared))
            || completed.0.pose_input.is_none()
            || completed.0.suit_palette.is_none()
            || completed.0.environment.is_none()
            || prepared.gameplay.is_none()
            || self
                .inside_completed
                .as_ref()
                .is_none_or(|receipt| receipt.work != WorkId(15))
            || controls.work != WorkId(16)
            || self
                .appearance
                .validate_cannon_completion(interface_hud::Selection::None, &controls.answer.0)
                .is_err()
            || controls.answer.0.selection != interface_hud::Selection::None
            || controls.answer.0.switched.is_some()
            || controls.answer.0.inputs.select != interface_hud::SelectButton::NotPressed
            || controls.answer.0.inputs.cancel != interface_hud::CancelButton::NotPressed
            || !controls.answer.0.inputs.can_switch
            || self.hud.controls()
                != Some(&interface_hud::Observation::Completed(
                    controls.answer.0.clone(),
                ))
            || self
                .timing
                .work
                .iter()
                .any(|work| matches!(work.kind, timing::WorkKind::Semantic(WorkId(12..=16))))
            || !self.continuation_motion_holds()
        {
            return Err(super::Fault::InvalidPlayerEntry);
        }
        Ok(())
    }

    fn continuation_motion_holds(&self) -> bool {
        use super::{
            OrdinaryWork, movement_caller::Run, movement_down_contract::Stage as Down,
            movement_ground_contract::Stage as Ground, movement_support_contract::Stage as Support,
            player_continuation::Stage as Entry, post_down_caller::Stage as PostDown,
        };
        let Caller::PlayerContinuation {
            stage,
            charging,
            run,
            ordinary,
            support,
            ground,
            down,
            post_down,
            ..
        } = &self.caller
        else {
            return false;
        };
        let charging = *charging;
        let ground_charging = matches!(ground, Ground::Charging(_));
        let support_charging = matches!(support, Support::Charging { .. });
        let down_charging = matches!(down, Down::Charging { .. });
        let post_identity = post_down.charging_identity();
        let post_charging = post_identity.is_some();
        let ordinary_charging = ordinary.as_ref().is_some_and(OrdinaryWork::is_charging);
        let awaiting_result = ordinary_charging
            || support_charging
            || ground_charging
            || down_charging
            || post_charging;
        let run_holds =
            Self::continuation_run_holds(*stage, charging, run, ordinary.as_ref(), awaiting_result);
        let support_holds = match support {
            Support::Idle => true,
            Support::Charging { .. } => charging,
            Support::Complete(completion) => {
                (!charging || down_charging || post_charging || ground_charging)
                    && completion.work == WorkId(1001)
            }
            Support::Offered(_) => false,
        };
        let ground_holds = match ground {
            Ground::Idle => true,
            Ground::Charging(_) => charging,
            Ground::Complete(completion) => {
                (!charging || down_charging || post_charging) && completion.work == WorkId(1003)
            }
            Ground::Offered(_) => false,
        };
        let down_holds = match down {
            Down::Idle => true,
            Down::Charging { .. } => charging,
            Down::Complete(completion) => {
                (!charging || post_charging) && completion.work == WorkId(1004)
            }
            Down::Offered(_) => false,
        };
        let ordinary_holds = ordinary.as_ref().is_none_or(|work| {
            matches!(stage, Entry::Run)
                && matches!(run, Run::Complete(_))
                && (work.is_charging() == charging
                    || support_charging
                    || down_charging
                    || post_charging
                    || ground_charging)
        });
        let identity = post_identity.unwrap_or_else(|| {
            if down_charging {
                WorkId(1004)
            } else if ground_charging {
                WorkId(1003)
            } else if support_charging {
                WorkId(1001)
            } else if ordinary_charging {
                WorkId(1000)
            } else {
                stage.identity()
            }
        });
        run.validate().is_ok()
            && ordinary_holds
            && support_holds
            && ground_holds
            && down_holds
            && (matches!(down, Down::Idle) || matches!(ground, Ground::Complete(_)))
            && self.animation_prefix_holds(post_down)
            && (matches!(post_down, PostDown::Idle) || matches!(down, Down::Complete(_)))
            && (matches!(ground, Ground::Idle) || matches!(support, Support::Complete(_)))
            && (matches!(support, Support::Idle)
                || ordinary.as_ref().is_some_and(OrdinaryWork::is_complete))
            && run_holds
            && self.continuation_hardware_holds(charging.then_some(identity))
    }

    fn continuation_hardware_holds(&self, expected: Option<WorkId>) -> bool {
        let active = self.timing.work.iter().filter_map(|work| match work.kind {
            timing::WorkKind::Semantic(
                identity @ WorkId(17..=23 | 1000..=1001 | 1003..=1004 | 1100 | 1105..=1106),
            ) => Some(identity),
            timing::WorkKind::Semantic(_)
            | timing::WorkKind::Wait(_)
            | timing::WorkKind::Native { .. }
            | timing::WorkKind::Handler { .. } => None,
        });
        active.eq(expected)
    }

    fn continuation_run_holds(
        stage: super::player_continuation::Stage,
        charging: bool,
        run: &super::movement_caller::Run,
        ordinary: Option<&super::OrdinaryWork>,
        awaiting_result: bool,
    ) -> bool {
        use super::{OrdinaryWork, movement_caller::Run, player_continuation::Stage as Entry};
        match (stage, charging, run) {
            (Entry::Run, true, Run::Complete(_)) if awaiting_result => true,
            (Entry::Run, false, Run::Complete(_)) => ordinary.is_none_or(OrdinaryWork::is_complete),
            (Entry::Run, true, Run::Charging { .. })
            | (_, false, Run::Idle)
            | (
                Entry::Projectiles
                | Entry::MotionReset
                | Entry::Interaction
                | Entry::Actors
                | Entry::Sprites
                | Entry::Contact,
                true,
                Run::Idle,
            ) => ordinary.is_none(),
            _ => false,
        }
    }

    fn entry_current_matches(&self, prepared: &Preparation, stage: PlayerEntryStage) -> bool {
        use types::Parts;
        let Some(current) = prepared.entry_origins() else {
            let Some(environment) = self.entry_environment.as_ref() else {
                return false;
            };
            return match stage {
                PlayerEntryStage::SuitPalette => environment.supports_power_suit(),
                PlayerEntryStage::Environment => environment.supports_dry_environment(),
                PlayerEntryStage::PoseInput
                | PlayerEntryStage::Inside
                | PlayerEntryStage::HudWeapons => false,
            };
        };
        let samus = self.movement.state();
        match stage {
            PlayerEntryStage::PoseInput => {
                current.pose_input.pose == samus.pose
                    && current.pose_input.action == samus.pose.action
            }
            PlayerEntryStage::SuitPalette => {
                current.suit_palette.equipment == current.environment.equipment
            }
            PlayerEntryStage::Environment => {
                current.environment.pose == samus.pose
                    && current.environment.centre_y == samus.position.y.pixel().0
            }
            PlayerEntryStage::Inside | PlayerEntryStage::HudWeapons => false,
        }
    }

    fn current_pose_input(
        &self,
        prepared: &Preparation,
        content: &Content,
    ) -> Option<super::pose_input::Decision> {
        let route = prepared.gameplay.as_ref()?;
        let words = if self.publications.original_interval() {
            let current = &prepared.entry_origins()?.pose_input;
            [current.held.0, current.pressed.0]
        } else {
            self.frame_history.as_ref()?.controller()
        };
        self.pose_input_configuration?.decide(
            self.movement.state().pose,
            words,
            route,
            &content.movement,
        )
    }

    fn begin_player_entry(&mut self, content: &Content) -> Result<(), ProductionFault> {
        let Caller::Player {
            prepared,
            stage,
            charging: false,
            completed,
            completion_work,
        } = &self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        let dependency = if matches!(
            stage,
            PlayerEntryStage::Inside | PlayerEntryStage::HudWeapons
        ) {
            Dependency::InsidePreparation
        } else {
            Dependency::PlayerPreparation
        };
        let fault = ProductionFault::Unsupported(Unsupported::Frame(dependency));
        let entry = &content.player_entry;
        let pose_input = (*stage == PlayerEntryStage::PoseInput)
            .then(|| self.current_pose_input(prepared, content).ok_or(fault))
            .transpose()?;
        let template = if let Some(decision) = &pose_input {
            match decision.path {
                super::pose_input::Path::AimDown => &entry.pose_input,
                super::pose_input::Path::ReleaseAim => {
                    entry.recurring_pose_input.as_ref().ok_or(fault)?
                }
            }
        } else {
            if !self.entry_current_matches(prepared, *stage)
                || (prepared.prelude.origin().is_some() && prepared.entry_offers().is_none())
            {
                return Err(fault);
            }
            entry.profile(*stage).ok_or(fault)?
        };
        let profile = template
            .bind(&[])
            .map_err(|_| ProductionFault::FrameHardware(timing::Fault::InvalidProfile))?;
        self.timing
            .begin(stage.identity(), profile)
            .map_err(ProductionFault::FrameHardware)?;
        if let Some(decision) = &pose_input {
            self.pose_input_configuration
                .as_mut()
                .ok_or(fault)?
                .begin(decision)
                .ok_or(fault)?;
        }
        self.caller = Caller::Player {
            prepared: prepared.clone(),
            stage: *stage,
            charging: true,
            completed: completed.clone(),
            completion_work: completion_work.clone(),
        };
        Ok(())
    }

    fn complete_player_entry(
        &mut self,
        identity: WorkId,
        content: &Content,
    ) -> Result<(), ProductionFault> {
        let fault = ProductionFault::Unsupported(Unsupported::Frame(Dependency::PlayerPreparation));
        let Caller::Player {
            prepared,
            stage,
            charging: true,
            completed,
            completion_work,
        } = &self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        if identity != stage.identity()
            || (*stage != PlayerEntryStage::PoseInput
                && !self.entry_current_matches(prepared, *stage))
        {
            return Err(fault);
        }
        let route = prepared.gameplay.as_ref().ok_or(fault)?;
        let live = || self.entry_environment.as_ref().ok_or(fault);
        let mut completed = completed.clone();
        let mut completion_work = completion_work.clone();
        let next = match stage {
            PlayerEntryStage::PoseInput => {
                if completed.0.pose_input.is_some() {
                    return Err(fault);
                }
                let decision = self.current_pose_input(prepared, content).ok_or(fault)?;
                completed.0.pose_input = Some(
                    self.pose_input_configuration
                        .as_mut()
                        .ok_or(fault)?
                        .finish(decision)
                        .ok_or(fault)?,
                );
                PlayerEntryStage::SuitPalette
            }
            PlayerEntryStage::SuitPalette => {
                if completed.0.pose_input.is_none() || completed.0.suit_palette.is_some() {
                    return Err(fault);
                }
                let suit = if let Some(current) = prepared.entry_origins() {
                    prepared
                        .entry_offers()
                        .ok_or(fault)?
                        .suit_palette
                        .clone()
                        .complete(&current.suit_palette, route)
                } else {
                    live()?.suit(route)
                };
                completed.0.suit_palette = Some(suit.map_err(|_| fault)?);
                PlayerEntryStage::Environment
            }
            PlayerEntryStage::Environment => {
                if completed.0.suit_palette.is_none() || completed.0.environment.is_some() {
                    return Err(fault);
                }
                let acceleration = content.player_entry.reference_acceleration;
                let environment = if let Some(current) = prepared.entry_origins() {
                    prepared
                        .entry_offers()
                        .ok_or(fault)?
                        .environment
                        .clone()
                        .complete(&current.environment, acceleration, route)
                } else {
                    use types::Parts;
                    let samus = self.movement.state();
                    live()?.environment(samus.pose, samus.position.y.pixel().0, acceleration, route)
                };
                completed.0.environment = Some(environment.map_err(|_| fault)?);
                PlayerEntryStage::Inside
            }
            PlayerEntryStage::Inside | PlayerEntryStage::HudWeapons => return Err(fault),
        };
        if identity == PlayerEntryStage::Environment.identity()
            && let Some(history) = &mut self.liquid_history
        {
            history.environment(&completed).map_err(|_| {
                ProductionFault::Unsupported(Unsupported::Frame(Dependency::AnimationLiquidHistory))
            })?;
        }
        completion_work.push(identity);
        self.caller = Caller::Player {
            prepared: prepared.clone(),
            stage: next,
            charging: false,
            completed,
            completion_work,
        };
        Ok(())
    }

    fn begin_preparation(&mut self, content: &PreparationContent) -> Result<(), ProductionFault> {
        let Caller::Preparation {
            prepared,
            stage,
            charging: false,
        } = &self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        let ready = match (stage, prepared.prelude.origin()) {
            (PreparationStage::Builder, _) => self.sprite_builder.is_some(),
            (PreparationStage::Gameplay, origin) => {
                self.scene == Some(super::borrowed_gameplay::SceneController::MainGameplay)
                    && prepared.builder.is_some()
                    && origin.is_none_or(|prelude| {
                        prelude
                            .gameplay_completion
                            .as_ref()
                            .is_some_and(|offer| Some(offer.dependency) == self.scene)
                    })
            }
            (PreparationStage::Actors, origin) => {
                prepared.gameplay.is_some()
                    && self.actor_eligibility.as_ref().is_some_and(|current| {
                        current.supports_marked_offscreen()
                            && origin.is_none_or(|prelude| {
                                prelude
                                    .actor_completion
                                    .as_ref()
                                    .is_some_and(|offer| &offer.dependency == current)
                            })
                    })
            }
            (PreparationStage::Debug, origin) => self.debug.is_some_and(|current| {
                current.supports_quiet()
                    && origin.is_none_or(|prelude| {
                        prelude
                            .debug_completion
                            .as_ref()
                            .is_some_and(|offer| offer.dependency == current)
                    })
            }),
            (PreparationStage::Palette, origin) => self.palette_objects.is_some_and(|current| {
                current.supports_empty_scan()
                    && origin.is_none_or(|prelude| {
                        prelude
                            .palette_completion
                            .as_ref()
                            .is_some_and(|offer| offer.dependency == current)
                    })
            }),
        };
        if !ready {
            return Err(ProductionFault::Unsupported(Unsupported::Frame(
                stage.dependency(),
            )));
        }
        let profile = content
            .profile(*stage)
            .bind(&[])
            .map_err(|_| ProductionFault::FrameHardware(timing::Fault::InvalidProfile))?;
        self.timing
            .begin(stage.identity(), profile)
            .map_err(ProductionFault::FrameHardware)?;
        self.caller = Caller::Preparation {
            prepared: prepared.clone(),
            stage: *stage,
            charging: true,
        };
        Ok(())
    }

    fn complete_preparation(&mut self, identity: WorkId) -> Result<(), ProductionFault> {
        let Caller::Preparation {
            prepared,
            stage,
            charging: true,
        } = &self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        if identity != stage.identity() {
            return Err(ProductionFault::FrameHardware(timing::Fault::InvalidState));
        }
        let mut prepared = prepared.clone();
        let fault = ProductionFault::Unsupported(Unsupported::Frame(stage.dependency()));
        let next = match stage {
            PreparationStage::Builder => {
                let builder = self.sprite_builder.take().ok_or(fault)?;
                prepared.builder =
                    Some(super::borrowed_preparation::BuilderCompletionReceipt::reset(builder));
                Some(PreparationStage::Gameplay)
            }
            PreparationStage::Gameplay => {
                let current = self.scene.ok_or(fault)?;
                let builder = prepared.builder.as_ref().ok_or(fault)?;
                let route = match prepared.prelude.origin_mut() {
                    Some(prelude) => prelude
                        .gameplay_completion
                        .take()
                        .ok_or(fault)?
                        .complete(current, builder),
                    None => super::borrowed_gameplay::GameplayDispatchReceipt::dispatch(
                        current, builder,
                    ),
                };
                prepared.gameplay = Some(route.map_err(|_| fault)?);
                Some(PreparationStage::Actors)
            }
            PreparationStage::Actors => {
                use super::borrowed_preparation::ActorEligibilityReceipt;
                let current = self.actor_eligibility.as_ref().ok_or(fault)?;
                let actors = ActorEligibilityReceipt::marked_offscreen(current).ok_or(fault)?;
                if let Some(prelude) = prepared.prelude.origin_mut()
                    && prelude.actor_completion.take().as_ref() != Some(&actors)
                {
                    return Err(fault);
                }
                prepared.actors = Some(actors);
                Some(PreparationStage::Debug)
            }
            PreparationStage::Debug => {
                let current = self.debug.ok_or(fault)?;
                let debug = current.complete_quiet().ok_or(fault)?;
                if let Some(prelude) = prepared.prelude.origin_mut()
                    && prelude.debug_completion.take().as_ref() != Some(&debug)
                {
                    return Err(fault);
                }
                prepared.debug = Some(debug);
                Some(PreparationStage::Palette)
            }
            PreparationStage::Palette => {
                let current = self.palette_objects.ok_or(fault)?;
                let palette = current.complete_empty_scan().ok_or(fault)?;
                if let Some(prelude) = prepared.prelude.origin_mut()
                    && prelude.palette_completion.take().as_ref() != Some(&palette)
                {
                    return Err(fault);
                }
                prepared.palette = Some(palette);
                None
            }
        };
        self.caller = match next {
            Some(stage) => Caller::Preparation {
                prepared,
                stage,
                charging: false,
            },
            None => Caller::Player {
                prepared,
                stage: PlayerEntryStage::PoseInput,
                charging: false,
                completed: Model(Box::new(PlayerEntryCompletions::default())),
                completion_work: Vec::new(),
            },
        };
        Ok(())
    }

    /// Advance the actual caller using only a host pause budget.
    ///
    /// # Errors
    /// An absent current owner receipt remains unsupported before any dependent effect.
    pub fn advance(
        &mut self,
        content: &Content,
        rules: &Rules,
        budget: MasterTicks,
    ) -> Result<Outputs, ProductionFault> {
        let fault = ProductionFault::Unsupported(Unsupported::Frame(Dependency::RandomPreparation));
        self.validate_random_work().map_err(|_| fault)?;
        if matches!(self.caller, Caller::RandomRunning { .. }) {
            content
                .random_requests
                .multiplier
                .checked()
                .map_err(|_| fault)?;
        }
        let mut candidate = self.clone();
        let outputs = candidate.advance_caller(content, rules, budget)?;
        *self = candidate;
        Ok(outputs)
    }

    /// Advance a private candidate. The caller must discard it on error.
    pub(in crate::production) fn advance_caller(
        &mut self,
        content: &Content,
        rules: &Rules,
        budget: MasterTicks,
    ) -> Result<Outputs, ProductionFault> {
        self.begin_next_opportunity()?;
        self.validate_cannon_history()?;
        let (gameplay, native_admitted) = self.gameplay_publications()?;
        self.validate_scene_preservation(content, gameplay, native_admitted)?;
        let health = self.validate_health_preservation(content, gameplay);
        if matches!(self.room_tail, super::room_tail::Stage::Completed(_))
            && !self.play_time.is_complete()
            && content.play_time.is_some()
        {
            health.map_err(|_| super::play_time::missing())?;
        } else {
            health?;
        }
        if let Some(outputs) = self.start_current_owner(content, rules, budget)? {
            return Ok(outputs);
        }
        let active = self.timing.work.last().and_then(|work| {
            if let timing::WorkKind::Semantic(identity) = work.kind {
                Some(identity)
            } else {
                None
            }
        });
        let sender_active = self
            .sender_progress
            .active()
            .filter(|identity| active == Some(*identity));
        let (progress, events) = self
            .timing
            .step(budget, &self.transfer_regions())
            .map_err(ProductionFault::FrameHardware)?;
        if let Some(identity) = sender_active {
            self.process_frame_sender(content, identity, progress, &events)?;
        }
        self.deliver_hardware_samples(active, &events)?;
        self.publications
            .record(events)
            .map_err(ProductionFault::FramePublications)?;
        self.finish_current_work(progress, content, rules, sender_active)?;
        Ok(Outputs {
            progress: Progress::Advanced,
            events: self
                .publications
                .deliver()
                .map_err(ProductionFault::FramePublications)?,
        })
    }

    fn start_current_owner(
        &mut self,
        content: &Content,
        rules: &Rules,
        budget: MasterTicks,
    ) -> Result<Option<Outputs>, ProductionFault> {
        match &self.caller {
            Caller::Preparation {
                charging: false, ..
            } => self.begin_preparation(&content.preparation)?,
            Caller::Player {
                stage: PlayerEntryStage::Inside,
                charging: false,
                ..
            } => self.begin_inside(content, rules)?,
            Caller::Player {
                stage: PlayerEntryStage::HudWeapons,
                charging: false,
                ..
            } => self.begin_early_hud(content)?,
            Caller::Player {
                stage: PlayerEntryStage::HudWeapons,
                charging: true,
                ..
            } => self.validate_early_hud_current(content)?,
            Caller::Player {
                charging: false, ..
            } => self.begin_player_entry(content)?,
            Caller::Preparation { charging: true, .. } | Caller::Player { charging: true, .. } => {}
            Caller::AwaitingAcknowledgment { .. }
            | Caller::Acknowledgment { .. }
            | Caller::Music { .. }
            | Caller::MusicRunning { .. }
            | Caller::Haze { .. }
            | Caller::HazeRunning { .. }
            | Caller::HazeTail { .. }
            | Caller::LayerBlend { .. }
            | Caller::LayerBlendRunning { .. }
            | Caller::Random { .. }
            | Caller::RandomRunning { .. } => self.start_prelude(content)?,
            Caller::PlayerContinuation { .. } => {
                if self.start_player_motion(content, rules)?
                    || self.start_player_contact(content, rules)?
                {
                    return Ok(None);
                }
                match &self.caller {
                    Caller::PlayerContinuation {
                        post_down: super::post_down_caller::Stage::Complete(_),
                        charging: false,
                        ..
                    } => {
                        return self.start_frame_tail(content, rules, budget);
                    }
                    Caller::PlayerContinuation {
                        stage: super::player_continuation::Stage::Run,
                        ..
                    } => {
                        return Err(ProductionFault::Unsupported(Unsupported::Frame(
                            Dependency::PlayerContinuationPreparation,
                        )));
                    }
                    Caller::PlayerContinuation {
                        charging: false, ..
                    } => self.begin_remaining_player(content)?,
                    Caller::PlayerContinuation { charging: true, .. } => {
                        self.validate_remaining_player(content).map_err(|_| {
                            ProductionFault::Unsupported(Unsupported::Frame(
                                Dependency::PlayerContinuationPreparation,
                            ))
                        })?;
                    }

                    Caller::AwaitingAcknowledgment { .. }
                    | Caller::Acknowledgment { .. }
                    | Caller::Music { .. }
                    | Caller::MusicRunning { .. }
                    | Caller::Haze { .. }
                    | Caller::HazeRunning { .. }
                    | Caller::HazeTail { .. }
                    | Caller::LayerBlend { .. }
                    | Caller::LayerBlendRunning { .. }
                    | Caller::Random { .. }
                    | Caller::RandomRunning { .. }
                    | Caller::Preparation { .. }
                    | Caller::Player { .. } => return Err(ProductionFault::WrongScope),
                }
            }
        }
        Ok(None)
    }

    fn start_prelude(&mut self, content: &Content) -> Result<(), ProductionFault> {
        match &self.caller {
            Caller::AwaitingAcknowledgment { receipt: None, .. } => {
                return Err(ProductionFault::Unsupported(Unsupported::Frame(
                    Dependency::FramePublicationAcknowledgment,
                )));
            }
            Caller::AwaitingAcknowledgment {
                prelude,
                receipt: Some(receipt),
            } => {
                let profile = content
                    .acknowledgment
                    .bind(&[])
                    .map_err(|_| ProductionFault::FrameHardware(timing::Fault::InvalidProfile))?;
                self.timing
                    .begin(ACKNOWLEDGMENT, profile)
                    .map_err(ProductionFault::FrameHardware)?;
                self.caller = Caller::Acknowledgment {
                    prelude: prelude.clone(),
                    receipt: *receipt,
                };
            }
            Caller::Acknowledgment { .. }
            | Caller::MusicRunning { .. }
            | Caller::HazeRunning { .. }
            | Caller::HazeTail { .. }
            | Caller::LayerBlendRunning { .. }
            | Caller::RandomRunning { .. }
            | Caller::Preparation { .. }
            | Caller::Player { .. }
            | Caller::PlayerContinuation { .. } => {}
            Caller::Music {
                prelude,
                publication,
            } => {
                self.music_result(prelude)?;
                if content
                    .music
                    .runs
                    .iter()
                    .any(|run| run.effect != data::timing::TemplateEffect::None)
                {
                    return Err(ProductionFault::Unsupported(Unsupported::Frame(
                        Dependency::MusicPreparation,
                    )));
                }
                let profile = content
                    .music
                    .bind(&[])
                    .map_err(|_| ProductionFault::FrameHardware(timing::Fault::InvalidProfile))?;
                self.timing
                    .begin(MUSIC, profile)
                    .map_err(ProductionFault::FrameHardware)?;
                self.caller = Caller::MusicRunning {
                    prelude: prelude.clone(),
                    publication: *publication,
                };
            }
            Caller::Haze { .. } => self.start_haze(content)?,
            Caller::LayerBlend {
                prelude,
                publication,
                music,
                haze,
            } => {
                let layers = content.layer_defaults.prepare(haze.blend);
                super::layer_blend::encode(layers.terminal).map_err(|_| {
                    ProductionFault::Unsupported(Unsupported::Frame(
                        Dependency::LayerBlendPreparation,
                    ))
                })?;
                let profile = content
                    .layer_blend
                    .bind(&[])
                    .map_err(|_| ProductionFault::FrameHardware(timing::Fault::InvalidProfile))?;
                self.timing
                    .begin(LAYERS, profile)
                    .map_err(ProductionFault::FrameHardware)?;
                self.caller = Caller::LayerBlendRunning {
                    prelude: prelude.clone(),
                    publication: *publication,
                    music: music.clone(),
                    haze: haze.clone(),
                    layers,
                };
            }
            Caller::Random { .. } => self.start_random(content)?,
        }
        Ok(())
    }
    fn start_haze(&mut self, content: &Content) -> Result<(), ProductionFault> {
        let Caller::Haze {
            prelude,
            publication,
            music,
        } = &self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        let (work, profile) = self
            .haze
            .as_ref()
            .ok_or_else(super::haze::missing)?
            .choose(&content.haze, content.settling_haze.as_ref())?;
        self.timing
            .begin(HAZE, profile)
            .map_err(ProductionFault::FrameHardware)?;
        self.caller = Caller::HazeRunning {
            prelude: prelude.clone(),
            publication: *publication,
            music: music.clone(),
            work,
        };
        Ok(())
    }

    fn start_random(&mut self, content: &Content) -> Result<(), ProductionFault> {
        let Caller::Random {
            prelude,
            publication,
            music,
            haze,
            layers,
        } = &self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        let fault = ProductionFault::Unsupported(Unsupported::Frame(Dependency::RandomPreparation));
        let current = self.random.as_ref().ok_or(fault)?.current();
        let profile = content
            .random_requests
            .bind(current, &content.random)
            .map_err(|_| fault)?;
        self.timing
            .begin(RANDOM, profile)
            .map_err(ProductionFault::FrameHardware)?;
        self.caller = Caller::RandomRunning {
            prelude: prelude.clone(),
            publication: *publication,
            music: music.clone(),
            haze: haze.clone(),
            layers: layers.clone(),
            requested: current,
            samples: super::random::Samples::default(),
        };
        Ok(())
    }

    fn start_player_motion(
        &mut self,
        content: &Content,
        rules: &Rules,
    ) -> Result<bool, ProductionFault> {
        match &self.caller {
            Caller::PlayerContinuation {
                stage: super::player_continuation::Stage::MotionReset,
                charging: false,
                ..
            } => self.begin_motion_reset(content)?,
            Caller::PlayerContinuation {
                stage: super::player_continuation::Stage::MotionReset,
                charging: true,
                ..
            } => self.validate_motion_reset(content)?,
            Caller::PlayerContinuation {
                stage: super::player_continuation::Stage::Run,
                run: super::movement_caller::Run::Idle,
                charging: false,
                ..
            } => self.begin_run(content, rules)?,
            Caller::PlayerContinuation {
                stage: super::player_continuation::Stage::Run,
                run: super::movement_caller::Run::Charging { .. },
                charging: true,
                ..
            } => self.validate_run(content, rules)?,
            Caller::PlayerContinuation {
                stage: super::player_continuation::Stage::Run,
                run: super::movement_caller::Run::Complete(_),
                ordinary: None,
                charging: false,
                ..
            } => self.begin_ordinary(content, rules)?,
            Caller::PlayerContinuation {
                stage: super::player_continuation::Stage::Run,
                ordinary: Some(ordinary),
                charging: true,
                ..
            } if ordinary.is_charging() => self.validate_ordinary(content, rules)?,
            Caller::PlayerContinuation {
                ordinary: Some(ordinary),
                support: super::movement_support_contract::Stage::Idle,
                ground: super::movement_ground_contract::Stage::Idle,
                down: super::movement_down_contract::Stage::Idle,
                charging: false,
                ..
            } if ordinary.is_complete() => self.begin_support(content, rules)?,
            Caller::PlayerContinuation {
                support: super::movement_support_contract::Stage::Charging { .. },
                charging: true,
                ..
            } => self.validate_support(content, rules)?,
            Caller::PlayerContinuation {
                support: super::movement_support_contract::Stage::Complete(_),
                ground: super::movement_ground_contract::Stage::Idle,
                down: super::movement_down_contract::Stage::Idle,
                charging: false,
                ..
            } => self.begin_ground(content, rules)?,
            Caller::PlayerContinuation {
                ground: super::movement_ground_contract::Stage::Charging(_),
                charging: true,
                ..
            } => self.validate_ground(content, rules)?,
            Caller::PlayerContinuation {
                ground: super::movement_ground_contract::Stage::Complete(_),
                down: super::movement_down_contract::Stage::Idle,
                charging: false,
                ..
            } => self.begin_down(content, rules)?,
            Caller::PlayerContinuation {
                down: super::movement_down_contract::Stage::Charging { .. },
                charging: true,
                ..
            } => self.validate_down(content, rules)?,
            Caller::AwaitingAcknowledgment { .. }
            | Caller::Acknowledgment { .. }
            | Caller::Music { .. }
            | Caller::MusicRunning { .. }
            | Caller::Haze { .. }
            | Caller::HazeRunning { .. }
            | Caller::HazeTail { .. }
            | Caller::LayerBlend { .. }
            | Caller::LayerBlendRunning { .. }
            | Caller::Random { .. }
            | Caller::RandomRunning { .. }
            | Caller::Preparation { .. }
            | Caller::Player { .. }
            | Caller::PlayerContinuation { .. } => return Ok(false),
        }
        Ok(true)
    }

    fn start_player_contact(
        &mut self,
        content: &Content,
        rules: &Rules,
    ) -> Result<bool, ProductionFault> {
        match &self.caller {
            Caller::PlayerContinuation {
                down: super::movement_down_contract::Stage::Complete(_),
                post_down: super::post_down_caller::Stage::Idle,
                charging: false,
                ..
            } => self.begin_stride_contact(content, rules)?,
            Caller::PlayerContinuation {
                post_down: super::post_down_caller::Stage::Contact { .. },
                charging: true,
                ..
            } => self.validate_stride_contact(content)?,
            Caller::PlayerContinuation {
                post_down: super::post_down_caller::Stage::ContactComplete(contact),
                charging: false,
                ..
            } => {
                if contact.quiet() {
                    self.begin_poststep(content, rules)?;
                } else {
                    self.begin_footstep_offer(content)?;
                }
            }
            Caller::PlayerContinuation {
                post_down: super::post_down_caller::Stage::SoundOffer { .. },
                charging: true,
                ..
            } => self.validate_footstep_offer(content)?,
            Caller::PlayerContinuation {
                post_down: super::post_down_caller::Stage::SoundComplete(_),
                charging: false,
                ..
            } => self.begin_poststep(content, rules)?,
            Caller::PlayerContinuation {
                post_down: super::post_down_caller::Stage::Poststep { .. },
                charging: true,
                ..
            } => self.validate_poststep(content)?,
            Caller::AwaitingAcknowledgment { .. }
            | Caller::Acknowledgment { .. }
            | Caller::Music { .. }
            | Caller::MusicRunning { .. }
            | Caller::Haze { .. }
            | Caller::HazeRunning { .. }
            | Caller::HazeTail { .. }
            | Caller::LayerBlend { .. }
            | Caller::LayerBlendRunning { .. }
            | Caller::Random { .. }
            | Caller::RandomRunning { .. }
            | Caller::Preparation { .. }
            | Caller::Player { .. }
            | Caller::PlayerContinuation { .. } => return Ok(false),
        }
        Ok(true)
    }

    fn start_frame_tail(
        &mut self,
        content: &Content,
        rules: &Rules,
        budget: MasterTicks,
    ) -> Result<Option<Outputs>, ProductionFault> {
        if !matches!(
            self.countdown,
            super::countdown_caller::Stage::Complete { .. }
        ) {
            self.advance_hud_map(content, rules)?;
        } else if !matches!(self.animation, super::animation_caller::Stage::Complete(_)) {
            self.advance_animation(content, rules)?;
        } else if !matches!(
            self.interruption,
            super::interruption_caller::Stage::Complete(_)
        ) {
            self.advance_interruption(content)?;
        } else if !matches!(self.late_movement, super::late_movement::Stage::Complete(_)) {
            self.advance_late_movement(content, rules)?;
        } else if self
            .samus_palette
            .as_ref()
            .and_then(super::samus_palette::Palette::completed)
            .is_none()
        {
            self.advance_palette(content)?;
        } else if self.maintenance.energy().is_none() {
            self.advance_maintenance(content)?;
        } else if self.s6_camera.following().is_none() {
            self.advance_before_drawing(content)?;
        } else if self.advance_drawing_tail(content, rules)? {
        } else if !matches!(self.room_tail, super::room_tail::Stage::Completed(_)) {
            self.advance_room_tail(content)?;
        } else if !self.play_time.is_complete() {
            self.advance_play_time(content)?;
        } else if !self.room_quake.is_complete() {
            self.advance_room_quake(content)?;
        } else {
            return self.advance_after_shake(content, budget);
        }
        Ok(None)
    }

    fn deliver_hardware_samples(
        &mut self,
        active: Option<WorkId>,
        events: &[timing::Event],
    ) -> Result<(), ProductionFault> {
        self.sample_drawing_tail(active, events)?;
        if active == Some(super::s6_camera::CAMERA) {
            self.sample_current_camera(events)?;
        }
        if active == Some(super::late_movement::AIM_RIGHT) {
            self.sample_late_aim(events)?;
        }
        if active == Some(RANDOM)
            && let Caller::RandomRunning { samples, .. } = &mut self.caller
        {
            for event in events {
                if let timing::EventKind::Sample(
                    device @ (data::timing::Sample::ProductLow | data::timing::Sample::ProductHigh),
                    timing::SampleValue::Byte(value),
                ) = &event.kind
                {
                    samples.accept(*device, *value).map_err(|_| {
                        ProductionFault::Unsupported(Unsupported::Frame(
                            Dependency::RandomPreparation,
                        ))
                    })?;
                }
            }
        }
        if active == Some(PlayerEntryStage::Inside.identity()) {
            let pending = self
                .inside_pending
                .as_mut()
                .ok_or(ProductionFault::Unsupported(Unsupported::Frame(
                    Dependency::InsidePreparation,
                )))?;
            for event in events {
                if let timing::EventKind::Sample(
                    device @ (data::timing::Sample::ProductLow | data::timing::Sample::ProductHigh),
                    timing::SampleValue::Byte(value),
                ) = &event.kind
                {
                    pending
                        .sample(PlayerEntryStage::Inside.identity(), *device, *value)
                        .map_err(|_| {
                            ProductionFault::Unsupported(Unsupported::Frame(
                                Dependency::InsidePreparation,
                            ))
                        })?;
                }
            }
        }
        self.deliver_movement_samples(active, events)
    }

    fn deliver_movement_samples(
        &mut self,
        active: Option<WorkId>,
        events: &[timing::Event],
    ) -> Result<(), ProductionFault> {
        if let Some(identity @ WorkId(1000)) = active {
            let Caller::PlayerContinuation {
                ordinary: Some(ordinary),
                ..
            } = &mut self.caller
            else {
                return Err(ProductionFault::WrongScope);
            };
            for event in events {
                if let timing::EventKind::Sample(
                    device @ (data::timing::Sample::ProductLow | data::timing::Sample::ProductHigh),
                    timing::SampleValue::Byte(value),
                ) = event.kind
                {
                    ordinary.sample(identity, device, value)?;
                }
            }
        }
        if active == Some(WorkId(1001)) {
            let Caller::PlayerContinuation { support, .. } = &mut self.caller else {
                return Err(ProductionFault::WrongScope);
            };
            for event in events {
                if let timing::EventKind::Sample(
                    device @ (data::timing::Sample::ProductLow | data::timing::Sample::ProductHigh),
                    timing::SampleValue::Byte(value),
                ) = event.kind
                {
                    support.sample(WorkId(1001), device, value).map_err(|_| {
                        ProductionFault::Unsupported(Unsupported::Frame(
                            Dependency::PlayerContinuationPreparation,
                        ))
                    })?;
                }
            }
        }
        if active == Some(WorkId(1004))
            && let Caller::PlayerContinuation { down, .. } = &mut self.caller
        {
            for event in events {
                if let timing::EventKind::Sample(
                    device @ (data::timing::Sample::ProductLow | data::timing::Sample::ProductHigh),
                    timing::SampleValue::Byte(value),
                ) = event.kind
                {
                    down.sample(WorkId(1004), device, value).map_err(|_| {
                        ProductionFault::Unsupported(Unsupported::Frame(
                            Dependency::PlayerContinuationPreparation,
                        ))
                    })?;
                }
            }
        }
        Ok(())
    }

    fn finish_current_work(
        &mut self,
        progress: timing::Progress,
        content: &Content,
        rules: &Rules,
        sender_active: Option<WorkId>,
    ) -> Result<(), ProductionFault> {
        if let timing::Progress::Completed(identity) = progress
            && self.finish_striding_work(identity, content, rules)?
        {
            return Ok(());
        }
        match progress {
            timing::Progress::Completed(ACKNOWLEDGMENT) => self.complete_acknowledgment()?,
            timing::Progress::Completed(MUSIC) => self.complete_music()?,
            timing::Progress::Completed(HAZE) => self.complete_haze(content)?,
            timing::Progress::Completed(HAZE_TAIL) => self.complete_haze_tail()?,
            timing::Progress::Completed(LAYERS) => self.complete_layers()?,
            timing::Progress::Completed(RANDOM) => self.complete_random(content.random_requests)?,
            timing::Progress::Completed(identity)
                if self.finish_player_work(identity, content, rules)? => {}
            timing::Progress::Completed(identity @ WorkId(1200..=1201)) => {
                self.complete_hud_map(identity, content, rules)?;
            }
            timing::Progress::Completed(identity @ WorkId(1101)) => {
                self.complete_animation(identity)?;
            }
            timing::Progress::Completed(identity)
                if identity == super::interruption_caller::WORK =>
            {
                self.complete_interruption(content)?;
            }
            timing::Progress::Completed(
                identity @ (super::late_movement::CALLER
                | super::late_movement::ADMISSION
                | super::late_movement::AIM_RIGHT
                | super::pose_fit_caller::WORK),
            ) => {
                self.complete_late_work(identity, content, rules)?;
            }
            timing::Progress::Completed(identity) if identity == super::maintenance::WORK => {
                self.complete_maintenance(content)?;
            }
            timing::Progress::Completed(
                identity @ (super::s6_camera::BEFORE_CAMERA | super::s6_camera::CAMERA),
            ) => {
                self.complete_camera_work(identity, content)?;
            }
            timing::Progress::Completed(identity) if identity == super::samus_palette::WORK => {
                self.complete_palette(content)?;
            }
            timing::Progress::Completed(identity) if identity == super::frame_return::FINALIZE => {
                self.complete_finalization(content)?;
            }
            timing::Progress::Completed(identity) if sender_active == Some(identity) => {}
            timing::Progress::Completed(identity)
                if self.complete_drawing_tail(identity, content)? => {}
            timing::Progress::Completed(super::room_quake::WORK) => {
                self.complete_room_quake(content)?;
            }
            timing::Progress::Completed(super::play_time::WORK) => {
                self.complete_play_time(content)?;
            }
            timing::Progress::Completed(
                identity @ (super::room_tail::BLOCKS | super::room_tail::ROOM),
            ) => {
                self.complete_room_tail(identity, content)?;
            }
            timing::Progress::Completed(
                identity @ (super::frame_tail::HURT | super::frame_tail::FINAL_HISTORY),
            ) => {
                self.complete_frame_tail(identity)?;
            }
            timing::Progress::Completed(identity @ WorkId(1300)) => {
                self.complete_countdown(identity, content, rules)?;
            }
            timing::Progress::Completed(_) => {
                return Err(ProductionFault::FrameHardware(timing::Fault::InvalidState));
            }
            boundary @ (timing::Progress::Advanced
            | timing::Progress::Interrupt(_)
            | timing::Progress::HandlerEntered(_)
            | timing::Progress::HandlerReturned(_)
            | timing::Progress::HandlerPortionCompleted(_)
            | timing::Progress::HandlerNeedsWork(_)
            | timing::Progress::Idle) => self.finish_timing_boundary(boundary, content)?,
        }
        Ok(())
    }

    fn finish_timing_boundary(
        &mut self,
        progress: timing::Progress,
        content: &Content,
    ) -> Result<(), ProductionFault> {
        match progress {
            timing::Progress::Advanced => {}
            timing::Progress::Interrupt(data::timing::Interrupt::Irq) => {
                self.enter_drawing_interrupt(content)?;
            }
            timing::Progress::HandlerEntered(identity) if identity == DRAWING => {}
            timing::Progress::HandlerReturned(identity) if identity == DRAWING => {
                self.complete_drawing_interrupt(identity)?;
            }
            timing::Progress::Interrupt(data::timing::Interrupt::Nmi) => {
                return Err(ProductionFault::Unsupported(Unsupported::Frame(
                    Dependency::InterruptOwner,
                )));
            }
            timing::Progress::Completed(_)
            | timing::Progress::HandlerEntered(_)
            | timing::Progress::HandlerReturned(_)
            | timing::Progress::HandlerPortionCompleted(_)
            | timing::Progress::HandlerNeedsWork(_)
            | timing::Progress::Idle => {
                return Err(ProductionFault::FrameHardware(timing::Fault::InvalidState));
            }
        }
        Ok(())
    }

    fn finish_striding_work(
        &mut self,
        identity: WorkId,
        content: &Content,
        rules: &Rules,
    ) -> Result<bool, ProductionFault> {
        match identity {
            work if work == WorkId(1105)
                && matches!(
                    &self.caller,
                    Caller::PlayerContinuation {
                        post_down: super::post_down_caller::Stage::SoundOffer { .. },
                        charging: true,
                        ..
                    }
                ) =>
            {
                self.complete_footstep_offer(identity, content)?;
            }
            work if work == WorkId(1106)
                && matches!(
                    &self.caller,
                    Caller::PlayerContinuation {
                        post_down: super::post_down_caller::Stage::Poststep { .. },
                        charging: true,
                        ..
                    }
                ) =>
            {
                self.complete_poststep(identity, content)?;
            }
            work if work == super::appearance_caller::Kind::Contact.identity()
                && matches!(
                    &self.caller,
                    Caller::PlayerContinuation {
                        post_down: super::post_down_caller::Stage::Contact { .. },
                        charging: true,
                        ..
                    }
                ) =>
            {
                self.complete_stride_contact(identity, content)?;
            }
            work if work == WorkId(1004)
                && matches!(
                    &self.caller,
                    Caller::PlayerContinuation {
                        down: super::movement_down_contract::Stage::Charging { .. },
                        charging: true,
                        ..
                    }
                ) =>
            {
                self.complete_down(identity, content, rules)?;
            }
            work if work == WorkId(1003)
                && matches!(
                    &self.caller,
                    Caller::PlayerContinuation {
                        ground: super::movement_ground_contract::Stage::Charging(_),
                        charging: true,
                        ..
                    }
                ) =>
            {
                self.complete_ground(identity, content, rules)?;
            }
            work if work == WorkId(1001)
                && matches!(
                    &self.caller,
                    Caller::PlayerContinuation {
                        support: super::movement_support_contract::Stage::Charging { .. },
                        charging: true,
                        ..
                    }
                ) =>
            {
                self.complete_support(identity, content, rules)?;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn finish_player_work(
        &mut self,
        identity: WorkId,
        content: &Content,
        rules: &Rules,
    ) -> Result<bool, ProductionFault> {
        match identity {
            work if matches!(&self.caller, Caller::Preparation { stage, charging: true, .. } if stage.identity() == work) =>
            {
                self.complete_preparation(identity)?;
            }
            work if matches!(&self.caller, Caller::Player { stage: PlayerEntryStage::Inside, charging: true, .. } if work == PlayerEntryStage::Inside.identity()) =>
            {
                self.complete_inside(identity, content, rules)?;
            }
            work if matches!(&self.caller, Caller::Player { stage: PlayerEntryStage::HudWeapons, charging: true, .. } if work == PlayerEntryStage::HudWeapons.identity()) =>
            {
                self.complete_early_hud(identity, content)?;
            }
            work if matches!(&self.caller, Caller::Player { stage, charging: true, .. } if stage.identity() == work) =>
            {
                self.complete_player_entry(identity, content)?;
            }
            work if matches!(&self.caller, Caller::PlayerContinuation { stage: super::player_continuation::Stage::MotionReset, charging: true, .. } if work == WorkId(18)) =>
            {
                self.finish_motion_reset(identity, content)?;
            }
            WorkId(1000) if matches!(&self.caller, Caller::PlayerContinuation { ordinary: Some(ordinary), charging: true, .. } if ordinary.is_charging()) =>
            {
                self.complete_ordinary(identity, content, rules)?;
            }
            work if matches!(&self.caller, Caller::PlayerContinuation { stage: super::player_continuation::Stage::Run, charging: true, .. } if work == WorkId(23)) =>
            {
                self.complete_run(identity, content, rules)?;
            }
            work if matches!(&self.caller, Caller::PlayerContinuation { stage, charging: true, .. } if stage.identity() == work) =>
            {
                self.complete_remaining_player(identity, content)?;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
    fn complete_acknowledgment(&mut self) -> Result<(), ProductionFault> {
        let Caller::Acknowledgment { prelude, receipt } = &self.caller else {
            return Err(ProductionFault::WrongScope);
        };
        self.caller = Caller::Music {
            prelude: prelude.clone(),
            publication: *receipt,
        };
        Ok(())
    }

    fn music_result(
        &self,
        prelude: &PreludeSource,
    ) -> Result<MusicCompletionReceipt, ProductionFault> {
        let fault = ProductionFault::Unsupported(Unsupported::Frame(Dependency::MusicPreparation));
        let current = self.music.ok_or(fault)?;
        let result = match (current.countdown, current.queue) {
            (MusicCountdown::Ready, MusicQueue::Empty) => MusicCompletionReceipt {
                dependency: current,
                terminal: current,
                request: MusicRequest::None,
            },
        };
        if let PreludeSource::Origin(source) = prelude
            && source.music_completion.as_ref() != Some(&result)
        {
            return Err(fault);
        }
        Ok(result)
    }

    fn complete_music(&mut self) -> Result<(), ProductionFault> {
        let Caller::MusicRunning {
            prelude,
            publication,
        } = &self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        let music = self.music_result(prelude)?;
        let mut prelude = prelude.clone();
        if let PreludeSource::Origin(prelude) = &mut prelude {
            prelude.music_completion = None;
        }
        self.music = Some(music.terminal);
        self.caller = Caller::Haze {
            prelude,
            publication: *publication,
            music,
        };
        Ok(())
    }

    fn complete_haze(&mut self, content: &Content) -> Result<(), ProductionFault> {
        let Caller::HazeRunning {
            prelude,
            publication,
            music,
            work,
        } = &self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        let prepared = self
            .haze
            .as_mut()
            .ok_or_else(super::haze::missing)?
            .complete_head(work, &mut self.content, content.haze_bands_address)?;
        let profile = super::haze::quiet_profile(&content.haze_tail)?;
        self.timing
            .begin(HAZE_TAIL, profile)
            .map_err(ProductionFault::FrameHardware)?;
        self.caller = Caller::HazeTail {
            prelude: prelude.clone(),
            publication: *publication,
            music: music.clone(),
            prepared,
        };
        Ok(())
    }

    fn complete_haze_tail(&mut self) -> Result<(), ProductionFault> {
        let Caller::HazeTail {
            prelude,
            publication,
            music,
            prepared,
        } = &self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        self.haze
            .as_mut()
            .ok_or_else(super::haze::missing)?
            .complete_tail()?;
        self.caller = Caller::LayerBlend {
            prelude: prelude.clone(),
            publication: *publication,
            music: music.clone(),
            haze: prepared.clone(),
        };
        Ok(())
    }
    fn complete_layers(&mut self) -> Result<(), ProductionFault> {
        let Caller::LayerBlendRunning {
            prelude,
            publication,
            music,
            haze,
            layers,
        } = &self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        self.caller = Caller::Random {
            prelude: prelude.clone(),
            publication: *publication,
            music: music.clone(),
            haze: haze.clone(),
            layers: layers.clone(),
        };
        Ok(())
    }

    fn complete_random(
        &mut self,
        requests: super::random::SeedRequests,
    ) -> Result<(), ProductionFault> {
        let Caller::RandomRunning {
            prelude,
            publication,
            music,
            haze,
            layers,
            requested,
            samples,
        } = &self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        let fault = ProductionFault::Unsupported(Unsupported::Frame(Dependency::RandomPreparation));
        let sequence = self.random.as_mut().ok_or(fault)?;
        if sequence.current() != *requested {
            return Err(fault);
        }
        requests
            .validate_samples(*requested, *samples)
            .map_err(|_| fault)?;
        sequence.draw();
        self.caller = Caller::Preparation {
            prepared: Preparation {
                prelude: prelude.clone(),
                publication: *publication,
                music: music.clone(),
                haze: haze.clone(),
                layers: layers.clone(),
                builder: None,
                gameplay: None,
                actors: None,
                debug: None,
                palette: None,
            },
            stage: PreparationStage::Builder,
            charging: false,
        };
        Ok(())
    }

    fn enter_drawing_interrupt(&mut self, content: &Content) -> Result<(), ProductionFault> {
        let original = self.publications.original_interval();
        let missing = ProductionFault::Unsupported(Unsupported::Frame(Dependency::InterruptOwner));
        let drawing = self.drawing.as_mut().ok_or(missing)?;
        let profiles = content.drawing.as_ref().ok_or(missing)?;
        if drawing.active.is_some() {
            return Err(ProductionFault::FrameHardware(timing::Fault::InvalidState));
        }
        let profile = if original {
            let receipt = match drawing.current.request {
                super::interrupts::DrawingRequest::Begin => &mut drawing.begin,
                super::interrupts::DrawingRequest::End => &mut drawing.end,
            };
            let profile = profiles
                .bind(drawing.current, receipt.as_ref())
                .map_err(|_| missing)?;
            receipt.take();
            profile
        } else {
            drawing.bind_recurring(profiles).map_err(|_| missing)?
        };
        if let Some(owner) = &mut self.scene_countdown {
            owner.0.hud_irq(drawing.current.request);
        }
        if let Some(history) = &mut self.liquid_history {
            history.hud_irq(drawing.current.request);
        }
        if let Some(owner) = &mut self.health {
            owner.hud_irq(drawing.current.request);
        }
        drawing.active = Some(drawing.current.request);
        let events = self
            .timing
            .enter_interrupt(DRAWING, profile)
            .map_err(ProductionFault::FrameHardware)?;
        self.publications
            .record(events)
            .map_err(ProductionFault::FramePublications)?;
        Ok(())
    }

    fn complete_drawing_interrupt(&mut self, identity: WorkId) -> Result<(), ProductionFault> {
        let original = self.publications.original_interval();
        let drawing = self
            .drawing
            .as_mut()
            .ok_or(ProductionFault::FrameHardware(timing::Fault::InvalidState))?;
        let request = drawing
            .active
            .take()
            .ok_or(ProductionFault::FrameHardware(timing::Fault::InvalidState))?;
        if let Some(history) = &mut self.liquid_history {
            history.hud_irq_returned(request, identity).map_err(|_| {
                ProductionFault::Unsupported(Unsupported::Frame(Dependency::AnimationLiquidHistory))
            })?;
        }
        if !original {
            return drawing.complete_recurring(request).map_err(|_| {
                ProductionFault::Unsupported(Unsupported::Frame(Dependency::InterruptOwner))
            });
        }
        if request == super::interrupts::DrawingRequest::Begin {
            drawing.current.request =
                drawing
                    .after_begin
                    .take()
                    .ok_or(ProductionFault::Unsupported(Unsupported::Frame(
                        Dependency::InterruptOwner,
                    )))?;
        } else {
            drawing.current.request =
                drawing
                    .after_end
                    .take()
                    .ok_or(ProductionFault::Unsupported(Unsupported::Frame(
                        Dependency::InterruptOwner,
                    )))?;
        }
        Ok(())
    }
}
