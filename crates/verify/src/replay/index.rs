//! Strict case metadata. Cases name witnesses; rule identifiers resolve to clean catalogues.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(transparent)]
/// Stable committed witness identity.
pub struct CaseId(#[doc = "Committed witness identity."] pub String);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Dependency claim made by a witness.
pub enum CaseKind {
    /// Selected owners with declared external neighbours.
    Scoped,
    /// Selected owners and their production dependency closure.
    Integrated,
    /// Power-on play using movie inputs.
    Movie,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// An existing selected production owner.
pub enum Responsibility {
    /// The complete retained frame caller and physical time owner.
    Frame,
    /// The existing movement and semantic pose owner.
    Movement,
    /// The existing native audio transport owner.
    Audio,
    /// The complete following camera.
    Camera,
    /// The complete Ceres door.
    Ceres,
    /// The complete shot owner.
    Weapons,
    /// Ordered contact consuming produced shots.
    Contact,
    /// The complete status bar and mini-map owner.
    Hud,
    /// The complete animation playhead and boost memory.
    Appearance,
    /// The complete inventory, events and remembered places.
    Progress,
    /// The complete room blocks and running placements.
    Blocks,
    /// The complete passage and scene.
    Doors,
    /// The complete CPU audio sender queue and pending service.
    AudioSender,
    /// Complete live health, including fractional energy and pending periodic loss.
    Health,
    /// Complete play-clock state after the admitted gameplay opportunity.
    GameTime,
    /// The carried shared random sequence.
    GameRandom,
    /// Complete room quake timer and phase after its quiet opportunity.
    WorldQuake,
    /// The current Ceres backdrop haze and its band publication.
    WorldHaze,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "source", rename_all = "kebab-case", deny_unknown_fields)]
/// The sole source of initial selected state.
pub enum Start {
    /// Direct scoped opportunity within one video frame, rather than an NMI-cut start.
    NativeOpportunity {
        /// Frozen independent source/input packet directory.
        path: PathBuf,
        /// Frozen admission manifest SHA-256.
        sha256: String,
        /// Video frame containing this opportunity.
        video_frame: u64,
        /// Independently qualified pre-step instruction coordinate.
        source_start_index: u64,
        /// Independent complete opportunity return coordinate.
        source_completion_index: u64,
    },
    /// One independent recorded sender offer.
    NativeSenderOffer {
        /// Identity in the frozen ingress sequence.
        source_id: usize,
    },
    /// One independent constructed sender control.
    ConstructedSenderOffer {
        /// Starting queue and ordered offers.
        control: SenderOfferControl,
    },
    /// Initial selected state from one pinned record cut.
    CacheCut {
        /// The original record cut used only for initial selected state.
        frame: u64,
    },
    /// One independently decoded hash-pinned native start.
    NativeSeed {
        /// Independently decoded native fixture path.
        path: PathBuf,
        /// Exact fixture SHA-256.
        sha256: String,
    },
    /// Complete independently captured continuation after an external intervention.
    NativeCheckpoint {
        /// Original native expectation fixture.
        path: PathBuf,
        /// Exact expectation fixture pin.
        sha256: String,
        /// Complete independently decoded seed file.
        seed_path: PathBuf,
        /// Exact independent seed pin.
        seed_sha256: String,
        /// Original saved cut coordinate.
        frame: u64,
    },
    /// Power-on start without borrowed record state.
    PowerOn,
}

/// Constructed controls for queue behavior absent from the recording.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SenderOfferControl {
    /// Two different cues append in offer order.
    OrderedPair,
    /// A second offer of an already pending cue can enter.
    RepeatAtFive,
    /// Occupancy equal to the allowance denies the offer.
    DenyAtSix,
    /// A reduced allowance preserves the existing queue.
    DenySmallerLimit,
    /// Disabled sound denies the offer.
    DenyDisabled,
    /// Demo playback denies the offer.
    DenyDemo,
    /// Explosion suppression denies the offer.
    DenyExplosion,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// The final declared comparison cut.
pub struct End {
    /// Last compared original cut.
    pub cut_inclusive: u64,
    /// Pinned return schedule for a native frame case spanning multiple cuts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_returns: Option<NativeReturns>,
}

/// Source supplement for consecutive native frame returns.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeReturns {
    /// First compared return in the native fixture coordinates.
    pub first_cut: u64,
    /// Supplement manifest path relative to the native fixture directory.
    pub path: PathBuf,
    /// Exact supplement manifest SHA-256.
    pub sha256: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Independent source of changing causal stimuli.
pub enum InputProvider {
    /// Authenticated actual-use snapshots for the connected ordinary family.
    ConnectedOrdinaryInputs,
    /// Qualified current owner inputs and explicitly borrowed unselected completions.
    NativeFrameOwners,
    /// Independently admitted selector-time Frame and actor-selection-time geometry.
    NativeSearchInputs,
    /// Causal neighbours from the preceding pinned cut.
    PrecedingCutMovementInputs,
    /// Independent prior facts plus observed external Ceres run scheduling.
    PrecedingCutCeresInputs,
    /// Reviewed ordinary opportunities and external neighbours for the position connection.
    CertifiedMovementCeresInputs,
    /// Independent prior facts while captured camera producers remain unchanged.
    PrecedingCutCameraInputs,
    /// Independent cannon/terrain facts and bounded watched actor opportunities.
    PrecedingCutWeaponsContactInputs,
    /// Independent stable status/map facts and inert-input representatives.
    PrecedingCutHudInputs,
    /// Independent stable standing-figure neighbours.
    PrecedingCutAppearanceInputs,
    /// Observed visits to independently identified fixed event setters.
    ObservedFixedProgressEvents,
    /// A declared capability boundary with no admitted changing-input provider.
    Unavailable,
    /// Independently observed or constructed queue offers.
    SenderOffers,
    /// Physical port presentations and measured clock stops.
    NativePhysicalPresentations,
    /// Power-on play using movie inputs.
    Movie,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Source of immutable admitted content.
pub enum ContentProvider {
    /// Checked content imported from the player ROM.
    PlayerRomImport,
    /// Resident content in the independent pinned fixture.
    NativeResidentContent,
    /// Fixed group selected by the sender offer case.
    SenderGroup,
    /// Static figures and geometry in the existing Ceres model.
    CeresReference,
    /// Independently observed room scroll kinds and dimensions.
    RecordedScrollRun,
    /// Checked imported HUD pictures and area maps.
    HudReference,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Oracle source used only for comparison.
pub enum ExpectedProvider {
    /// Independent frozen ordered physical callbacks and completed-return wall time.
    NativeFrameCallbacks,
    /// Frozen complete source-derived movement work, with independent captured input packets.
    NativeCompleteMovementWork,
    /// Independent pinned oracle records.
    PinnedRecordCache,
    /// Independent queue and committed progress.
    SenderOfferQueue,
    /// Independent decoded native cuts.
    NativeDecodedCuts,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
/// A concrete supported compared field or output stream.
pub enum ObservationId {
    /// Complete pending sound queue, oldest first.
    #[serde(rename = "sender.queue")]
    SenderQueue,
    /// Complete committed sender progress.
    #[serde(rename = "sender.progress")]
    SenderProgress,
    /// Complete retained movement owner.
    #[serde(rename = "movement.state.complete")]
    MovementStateComplete,
    /// Complete currently playing Figure.
    #[serde(rename = "appearance.playing")]
    AppearancePlaying,
    /// Exact completed-return wall time from the retained physical owner.
    #[serde(rename = "frame.time")]
    FrameTime,
    /// The committed shared random number.
    #[serde(rename = "frame.random")]
    FrameRandom,
    /// Complete ordered physical callback stream, including equal-time multiplicity.
    #[serde(rename = "frame.outputs")]
    FrameOutputs,
    /// Whole hours carried by the play clock.
    #[serde(rename = "frame.play_time.hours")]
    PlayTimeHours,
    /// Minutes within the play-clock hour.
    #[serde(rename = "frame.play_time.minutes")]
    PlayTimeMinutes,
    /// Seconds within the play-clock minute.
    #[serde(rename = "frame.play_time.seconds")]
    PlayTimeSeconds,
    /// Frames within the play-clock second.
    #[serde(rename = "frame.play_time.frames")]
    PlayTimeFrames,
    /// Complete current room-shake timer.
    #[serde(rename = "frame.room_shake.timer")]
    RoomShakeTimer,
    /// Complete current room-shake phase.
    #[serde(rename = "frame.room_shake.phase")]
    RoomShakePhase,
    /// Whole energy after Maintenance.
    #[serde(rename = "frame.health.energy.whole")]
    HealthEnergyWhole,
    /// Fractional energy after Maintenance.
    #[serde(rename = "frame.health.energy.fraction")]
    HealthEnergyFraction,
    /// Whole pending energy loss.
    #[serde(rename = "frame.health.pending_loss.whole")]
    HealthPendingWhole,
    /// Fractional pending energy loss.
    #[serde(rename = "frame.health.pending_loss.fraction")]
    HealthPendingFraction,
    /// Current requested low-energy warning.
    #[serde(rename = "frame.health.warning")]
    HealthWarning,
    /// Every ordered Motion, Support and nested `BodyFit` receipt.
    #[serde(rename = "movement.work")]
    MovementWork,
    /// Every ordered leading-edge test, including classified horizontal context.
    #[serde(rename = "movement.collision_tests")]
    MovementTests,
    /// Complete ordered reaction-facing reaches.
    #[serde(rename = "movement.reached.complete")]
    CompleteReached,
    /// All starting inside cells, distinct from leading-edge tests.
    #[serde(rename = "movement.inside")]
    Inside,
    /// Explicit absent ordinary-movement admission reason.
    #[serde(rename = "movement.skip")]
    MovementSkip,
    /// Complete independently selected first request and its considered candidates.
    #[serde(rename = "movement.motion.0.request")]
    FirstMotionSearch,
    /// Ordered considered identities, geometry and outcomes in the selected request.
    #[serde(rename = "movement.motion.0.enemy_search.candidates")]
    FirstMotionCandidates,
    /// Actual ordinary-right terrain work for the independently unique first request.
    #[serde(rename = "movement.motion.0.ordinary_terrain_tests")]
    FirstMotionTerrain,
    #[serde(rename = "movement.position.x")]
    /// Fixed-point horizontal position.
    PositionX,
    #[serde(rename = "movement.position.y")]
    /// Fixed-point vertical position.
    PositionY,
    #[serde(rename = "movement.vertical_speed")]
    /// Observed vertical speed magnitude.
    VerticalSpeed,
    #[serde(rename = "movement.travel.Left")]
    /// Latest accepted signed left travel.
    TravelLeft,
    #[serde(rename = "movement.travel.Right")]
    /// Latest accepted signed right travel.
    TravelRight,
    #[serde(rename = "movement.travel.Up")]
    /// Latest accepted signed upward travel.
    TravelUp,
    #[serde(rename = "movement.travel.Down")]
    /// Latest accepted signed downward travel.
    TravelDown,
    #[serde(rename = "movement.wall_test")]
    /// Performed near-wall test outcome.
    WallTest,
    #[serde(rename = "movement.reached")]
    /// Ordered reached-cell receipt.
    Reached,
    #[serde(rename = "audio.semantic")]
    /// Every admitted native semantic quantity.
    AudioSemantic,
    #[serde(rename = "audio.publications")]
    /// Ordered physical lines, bytes and instants.
    AudioPublications,
    #[serde(rename = "audio.replies")]
    /// Physical reply bytes at each cut.
    AudioReplies,
    #[serde(rename = "audio.pending_music")]
    /// Whether music work is pending.
    AudioPendingMusic,
    /// Exact independently decoded ceres.phase.
    #[serde(rename = "ceres.phase")]
    CeresPhase,
    /// Exact independently decoded ceres.wait.
    #[serde(rename = "ceres.wait")]
    CeresWait,
    /// Exact independently decoded ceres.figure.
    #[serde(rename = "ceres.figure")]
    CeresFigure,
    /// Exact independently decoded ceres.shown.
    #[serde(rename = "ceres.shown")]
    CeresShown,
    /// Exact independently decoded ceres.solid.
    #[serde(rename = "ceres.solid")]
    CeresSolid,
    /// Exact independently decoded camera.view.x.pixel.
    #[serde(rename = "camera.view.x.pixel")]
    CameraViewXPixel,
    /// Exact independently decoded camera.view.x.fraction.
    #[serde(rename = "camera.view.x.fraction")]
    CameraViewXFraction,
    /// Exact independently decoded camera.view.y.pixel.
    #[serde(rename = "camera.view.y.pixel")]
    CameraViewYPixel,
    /// Exact independently decoded camera.view.y.fraction.
    #[serde(rename = "camera.view.y.fraction")]
    CameraViewYFraction,
    /// Exact independently decoded camera.remembered.x.pixel.
    #[serde(rename = "camera.remembered.x.pixel")]
    CameraRememberedXPixel,
    /// Exact independently decoded camera.remembered.x.fraction.
    #[serde(rename = "camera.remembered.x.fraction")]
    CameraRememberedXFraction,
    /// Exact independently decoded camera.remembered.y.pixel.
    #[serde(rename = "camera.remembered.y.pixel")]
    CameraRememberedYPixel,
    /// Exact independently decoded camera.remembered.y.fraction.
    #[serde(rename = "camera.remembered.y.fraction")]
    CameraRememberedYFraction,
    /// Exact independently decoded camera.vertical-fraction.
    #[serde(rename = "camera.vertical-fraction")]
    CameraVerticalFraction,
    /// Exact independently decoded weapons.state.
    #[serde(rename = "weapons.state")]
    WeaponsState,
    /// Exact independently decoded contact.consumed-markers.
    #[serde(rename = "contact.consumed-markers")]
    ContactConsumedMarkers,
    /// Exact independently decoded hud.grid.
    #[serde(rename = "hud.grid")]
    HudGrid,
    /// Exact independently decoded hud.explored.
    #[serde(rename = "hud.explored")]
    HudExplored,
    /// Exact independently decoded hud.energy.
    #[serde(rename = "hud.energy")]
    HudEnergy,
    /// Exact independently decoded hud.missiles.
    #[serde(rename = "hud.missiles")]
    HudMissiles,
    /// Exact independently decoded hud.supers.
    #[serde(rename = "hud.supers")]
    HudSupers,
    /// Exact independently decoded hud.bombs.
    #[serde(rename = "hud.bombs")]
    HudBombs,
    /// Exact independently decoded hud.highlighted.
    #[serde(rename = "hud.highlighted")]
    HudHighlighted,
    /// Exact independently decoded hud.selected.
    #[serde(rename = "hud.selected")]
    HudSelected,
    /// Exact independently decoded hud.one-shot.
    #[serde(rename = "hud.one-shot")]
    HudOneShot,
    /// Exact independently decoded hud.map-off.
    #[serde(rename = "hud.map-off")]
    HudMapOff,
    /// Independently decoded appearance.step.
    #[serde(rename = "appearance.step")]
    AppearanceStep,
    /// Independently decoded appearance.time-left.
    #[serde(rename = "appearance.time-left")]
    AppearanceTimeLeft,
    /// Independently decoded appearance.boost-stage.
    #[serde(rename = "appearance.boost-stage")]
    AppearanceBoostStage,
    /// Independently decoded appearance.laps-left.
    #[serde(rename = "appearance.laps-left")]
    AppearanceLapsLeft,
    /// Independently decoded progress.events.PlanetAwake.
    #[serde(rename = "progress.events.PlanetAwake")]
    ProgressPlanetAwake,
    /// Independently decoded progress.events.MotherBrainTankBroken.
    #[serde(rename = "progress.events.MotherBrainTankBroken")]
    ProgressMotherBrainTankBroken,
    /// Independently decoded progress.events.PhantoonStatueGrey.
    #[serde(rename = "progress.events.PhantoonStatueGrey")]
    ProgressPhantoonStatueGrey,
    /// Independently decoded progress.events.RidleyStatueGrey.
    #[serde(rename = "progress.events.RidleyStatueGrey")]
    ProgressRidleyStatueGrey,
    /// Independently decoded progress.events.DraygonStatueGrey.
    #[serde(rename = "progress.events.DraygonStatueGrey")]
    ProgressDraygonStatueGrey,
    /// Independently decoded progress.events.KraidStatueGrey.
    #[serde(rename = "progress.events.KraidStatueGrey")]
    ProgressKraidStatueGrey,
    /// Independently decoded progress.events.TourianOpen.
    #[serde(rename = "progress.events.TourianOpen")]
    ProgressTourianOpen,
    /// Independently decoded progress.events.GlassTunnelBroken.
    #[serde(rename = "progress.events.GlassTunnelBroken")]
    ProgressGlassTunnelBroken,
    /// Independently decoded progress.events.AcidReceded.
    #[serde(rename = "progress.events.AcidReceded")]
    ProgressAcidReceded,
    /// Independently decoded progress.events.ShaktoolPath.
    #[serde(rename = "progress.events.ShaktoolPath")]
    ProgressShaktoolPath,
    /// Independently decoded progress.events.TimeBombSet.
    #[serde(rename = "progress.events.TimeBombSet")]
    ProgressTimeBombSet,
    /// Independently decoded progress.events.AnimalsFreed.
    #[serde(rename = "progress.events.AnimalsFreed")]
    ProgressAnimalsFreed,
    /// Independently decoded progress.events.MetroidRoom1Empty.
    #[serde(rename = "progress.events.MetroidRoom1Empty")]
    ProgressMetroidRoom1Empty,
    /// Independently decoded progress.events.MetroidRoom2Empty.
    #[serde(rename = "progress.events.MetroidRoom2Empty")]
    ProgressMetroidRoom2Empty,
    /// Independently decoded progress.events.MetroidRoom3Empty.
    #[serde(rename = "progress.events.MetroidRoom3Empty")]
    ProgressMetroidRoom3Empty,
    /// Independently decoded progress.events.MetroidRoom4Empty.
    #[serde(rename = "progress.events.MetroidRoom4Empty")]
    ProgressMetroidRoom4Empty,
    /// Independently decoded progress.events.TourianRoomsUnset.
    #[serde(rename = "progress.events.TourianRoomsUnset")]
    ProgressTourianRoomsUnset,
    /// Independently decoded progress.events.LavaQuakeOutrun.
    #[serde(rename = "progress.events.LavaQuakeOutrun")]
    ProgressLavaQuakeOutrun,
    /// Independently decoded progress.zebetites.
    #[serde(rename = "progress.zebetites")]
    ProgressZebetites,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Declared observations and their comparison schedule.
pub struct Expectations {
    /// Independent capture or import source.
    pub provider: ExpectedProvider,
    /// Compared semantic field identities or quantities.
    pub fields: Vec<ObservationId>,
    /// Compared output identities or occurrences.
    pub outputs: Vec<ObservationId>,
    #[serde(default)]
    /// Outputs compare only at this cut, or every cut when absent.
    pub output_cut: Option<u64>,
    /// Independent Random draw observations, when the case compares them.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_random: Option<NativeRandomObservations>,
}

/// Expected-only source for the native Random draws.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeRandomObservations {
    /// File under the replay native fixture directory.
    pub path: PathBuf,
    /// Exact expected observation file digest.
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// An external dependency and its claim limit.
pub struct Borrowed {
    /// Missing or externally admitted causal dependency.
    pub dependency: String,
    /// Independent capture or import source.
    pub provider: String,
    /// Compared semantic field identities or quantities.
    pub fields: Vec<String>,
    /// Named causal capture phase.
    pub phase: String,
    /// Limit imposed by an external producer.
    pub claim: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// One witness replayed continuously from its admitted start.
pub struct Case {
    /// Committed witness identity.
    pub id: CaseId,
    /// Dependency claim.
    pub kind: CaseKind,
    /// Original oracle row identity.
    pub row: String,
    /// Once-only selected-state source.
    pub start: Start,
    /// Comparison horizon.
    pub end: End,
    /// Existing clean catalogue rules witnessed.
    pub rules: Vec<String>,
    /// Admitted rules profile.
    pub profile: String,
    /// Concrete selected production owners.
    pub responsibilities: Vec<Responsibility>,
    /// Immutable content provider.
    pub content: ContentProvider,
    /// Independent causal stimulus provider.
    pub inputs: InputProvider,
    /// Declared independent comparison observations.
    pub expectations: Expectations,
    /// Explicit unproved dependencies and omitted outputs.
    pub limits: Vec<String>,
    #[serde(default)]
    /// Explicit external dependency declarations.
    pub borrowed: Vec<Borrowed>,
    #[serde(default)]
    /// Prove complete saved continuations through the horizon.
    pub checkpoints: bool,
    #[serde(default)]
    /// Explicit original actor slot for a scoped Ceres door; never selected by expected state.
    pub actor_slot: Option<u16>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// The complete strict TOML case catalogue.
pub struct Index {
    /// Strict metadata format version.
    pub format: u16,
    /// Required production API identity.
    pub engine_api: String,
    /// Required observation schema identity.
    pub observation_schema: String,
    /// Pinned oracle lock path.
    pub lock: PathBuf,
    #[serde(rename = "case")]
    /// Complete committed case list.
    pub cases: Vec<Case>,
}

/// Validated declarations and their identity from one immutable byte snapshot.
#[derive(Debug)]
pub struct IndexSnapshot {
    index: Index,
    sha256: String,
}

impl IndexSnapshot {
    /// Read and validate an index and hash the exact bytes parsed.
    ///
    /// # Errors
    /// Missing or malformed indices are rejected.
    pub fn read(path: &Path) -> Result<Self, IndexFault> {
        let text = std::fs::read_to_string(path).map_err(|_| IndexFault::Missing)?;
        Self::parse(&text)
    }

    /// Validate index text and hash its exact UTF-8 bytes.
    ///
    /// # Errors
    /// Unknown keys, schemas, identities and empty observations are rejected.
    pub fn parse(text: &str) -> Result<Self, IndexFault> {
        let index = Index::parse(text)?;
        let sha256 = crate::lock::Sha256::of(text.as_bytes()).to_string();
        Ok(Self { index, sha256 })
    }

    /// Validated declarations covered by this identity.
    #[must_use]
    pub const fn index(&self) -> &Index {
        &self.index
    }

    /// Consume the snapshot and retain its declarations.
    #[must_use]
    pub fn into_index(self) -> Index {
        self.index
    }

    /// SHA-256 of the exact index bytes.
    #[must_use]
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// A missing or invalid index or selection.
pub enum IndexFault {
    /// Index file unavailable.
    Missing,
    /// Strict TOML schema did not decode.
    Syntax,
    /// Unsupported API or observation schema.
    Schema,
    /// Index or selection contains no cases.
    Empty,
    /// Repeated case identity.
    DuplicateCase(CaseId),
    /// Case violates its metadata contract.
    InvalidCase(CaseId),
    /// Rule absent from existing clean catalogues.
    UnknownRule(String),
    /// Selected case absent from the index.
    UnknownCase(CaseId),
    /// Repeated selected case.
    DuplicateSelection(CaseId),
}

fn rule_exists(rule: &str) -> bool {
    samus_movement::CATALOGUE
        .iter()
        .any(|entry| rule == format!("samus-movement/{:?}", entry.id))
        || audio_transport::catalogue::catalogue()
            .iter()
            .any(|(id, _)| rule == format!("audio-transport/{id:?}"))
        || boss_ceres_ridley::catalogue()
            .iter()
            .any(|(id, _)| rule == format!("boss-ceres-ridley/{id:?}"))
        || world_camera::CATALOGUE
            .iter()
            .any(|entry| rule == format!("world-camera/{:?}", entry.id))
        || interface_hud::CATALOGUE
            .iter()
            .any(|entry| rule == format!("interface-hud/{:?}", entry.id))
        || samus_appearance::CATALOGUE
            .iter()
            .any(|entry| rule == format!("samus-appearance/{:?}", entry.id))
        || run_progress::catalogue()
            .iter()
            .any(|(id, _)| rule == format!("run-progress/{id:?}"))
        || world_haze::catalogue()
            .iter()
            .any(|(id, _)| rule == format!("world-haze/{id:?}"))
        || world_quake::CATALOGUE
            .iter()
            .any(|entry| rule == format!("world-quake/{:?}", entry.id))
        || game_random::CATALOGUE
            .iter()
            .any(|entry| rule == format!("game-random/{:?}", entry.id))
        || game_time::BEHAVIOURS
            .iter()
            .any(|entry| rule == format!("game-time/{:?}", entry.id))
        || samus_health::BehaviourId::ALL
            .iter()
            .any(|id| rule == format!("samus-health/{id:?}"))
        || weapons_projectiles::catalogue()
            .iter()
            .any(|(id, _)| rule == format!("weapons-projectiles/{id:?}"))
        || [
            enemy_interactions::BehaviourId::ContactAllowed,
            enemy_interactions::BehaviourId::BoxOverlap,
            enemy_interactions::BehaviourId::FirstPowerShot,
        ]
        .iter()
        .any(|id| rule == format!("enemy-interactions/{id:?}"))
}

const fn observation_owner(field: ObservationId) -> Responsibility {
    match field {
        ObservationId::SenderQueue | ObservationId::SenderProgress => Responsibility::AudioSender,
        ObservationId::FrameTime | ObservationId::FrameOutputs => Responsibility::Frame,
        ObservationId::FrameRandom => Responsibility::GameRandom,
        ObservationId::PlayTimeHours
        | ObservationId::PlayTimeMinutes
        | ObservationId::PlayTimeSeconds
        | ObservationId::PlayTimeFrames => Responsibility::GameTime,
        ObservationId::RoomShakeTimer | ObservationId::RoomShakePhase => Responsibility::WorldQuake,
        ObservationId::HealthEnergyWhole
        | ObservationId::HealthEnergyFraction
        | ObservationId::HealthPendingWhole
        | ObservationId::HealthPendingFraction
        | ObservationId::HealthWarning => Responsibility::Health,
        ObservationId::MovementWork
        | ObservationId::MovementTests
        | ObservationId::CompleteReached
        | ObservationId::Inside
        | ObservationId::MovementSkip
        | ObservationId::FirstMotionSearch
        | ObservationId::FirstMotionCandidates
        | ObservationId::FirstMotionTerrain
        | ObservationId::MovementStateComplete
        | ObservationId::PositionX
        | ObservationId::PositionY
        | ObservationId::VerticalSpeed
        | ObservationId::TravelLeft
        | ObservationId::TravelRight
        | ObservationId::TravelUp
        | ObservationId::TravelDown
        | ObservationId::WallTest
        | ObservationId::Reached => Responsibility::Movement,
        ObservationId::AudioSemantic
        | ObservationId::AudioPublications
        | ObservationId::AudioReplies
        | ObservationId::AudioPendingMusic => Responsibility::Audio,
        ObservationId::CeresPhase
        | ObservationId::CeresWait
        | ObservationId::CeresFigure
        | ObservationId::CeresShown
        | ObservationId::CeresSolid => Responsibility::Ceres,
        ObservationId::CameraViewXPixel
        | ObservationId::CameraViewXFraction
        | ObservationId::CameraViewYPixel
        | ObservationId::CameraViewYFraction
        | ObservationId::CameraRememberedXPixel
        | ObservationId::CameraRememberedXFraction
        | ObservationId::CameraRememberedYPixel
        | ObservationId::CameraRememberedYFraction
        | ObservationId::CameraVerticalFraction => Responsibility::Camera,
        ObservationId::HudGrid
        | ObservationId::HudExplored
        | ObservationId::HudEnergy
        | ObservationId::HudMissiles
        | ObservationId::HudSupers
        | ObservationId::HudBombs
        | ObservationId::HudHighlighted
        | ObservationId::HudSelected
        | ObservationId::HudOneShot
        | ObservationId::HudMapOff => Responsibility::Hud,
        ObservationId::WeaponsState => Responsibility::Weapons,
        ObservationId::ContactConsumedMarkers => Responsibility::Contact,
        ObservationId::AppearancePlaying
        | ObservationId::AppearanceStep
        | ObservationId::AppearanceTimeLeft
        | ObservationId::AppearanceBoostStage
        | ObservationId::AppearanceLapsLeft => Responsibility::Appearance,
        ObservationId::ProgressPlanetAwake
        | ObservationId::ProgressMotherBrainTankBroken
        | ObservationId::ProgressPhantoonStatueGrey
        | ObservationId::ProgressRidleyStatueGrey
        | ObservationId::ProgressDraygonStatueGrey
        | ObservationId::ProgressKraidStatueGrey
        | ObservationId::ProgressTourianOpen
        | ObservationId::ProgressGlassTunnelBroken
        | ObservationId::ProgressAcidReceded
        | ObservationId::ProgressShaktoolPath
        | ObservationId::ProgressTimeBombSet
        | ObservationId::ProgressAnimalsFreed
        | ObservationId::ProgressMetroidRoom1Empty
        | ObservationId::ProgressMetroidRoom2Empty
        | ObservationId::ProgressMetroidRoom3Empty
        | ObservationId::ProgressMetroidRoom4Empty
        | ObservationId::ProgressTourianRoomsUnset
        | ObservationId::ProgressLavaQuakeOutrun
        | ObservationId::ProgressZebetites => Responsibility::Progress,
    }
}

fn rule_owner(rule: &str) -> Option<Responsibility> {
    if rule == "audio-transport/OfferAcceptance" {
        return Some(Responsibility::AudioSender);
    }
    [
        ("samus-movement/", Responsibility::Movement),
        ("audio-transport/", Responsibility::Audio),
        ("boss-ceres-ridley/", Responsibility::Ceres),
        ("world-camera/", Responsibility::Camera),
        ("weapons-projectiles/", Responsibility::Weapons),
        ("enemy-interactions/", Responsibility::Contact),
        ("interface-hud/", Responsibility::Hud),
        ("samus-appearance/", Responsibility::Appearance),
        ("run-progress/", Responsibility::Progress),
        ("samus-health/", Responsibility::Health),
        ("game-time/", Responsibility::GameTime),
        ("game-random/", Responsibility::GameRandom),
        ("world-quake/", Responsibility::WorldQuake),
        ("world-haze/", Responsibility::WorldHaze),
    ]
    .iter()
    .find_map(|(prefix, owner)| rule.starts_with(prefix).then_some(*owner))
}

fn witnessed_fields(rule: &str) -> Option<&'static [ObservationId]> {
    use ObservationId as O;
    Some(match rule {
        "samus-movement/MotionAttempts" => &[O::FirstMotionSearch],
        "samus-movement/ReachedEnemyCandidates" => &[O::FirstMotionCandidates],
        "samus-movement/PerformedCollisionTests" => &[O::MovementTests],
        "samus-movement/PerformedSupportChecks" | "samus-movement/PerformedBodyFit" => {
            &[O::MovementWork]
        }
        "samus-movement/MovesAsShePosed" => &[O::PositionX, O::PositionY],
        "samus-movement/Dash" => &[O::MovementStateComplete],
        "samus-movement/AimingAndShooting"
        | "samus-movement/SlopesCarry"
        | "samus-movement/SlopesSlowTheWalk"
        | "samus-movement/FollowingTheGround" => {
            &[O::MovementStateComplete, O::PositionX, O::PositionY]
        }
        "samus-movement/TravelledDistances" => {
            &[O::TravelLeft, O::TravelRight, O::TravelUp, O::TravelDown]
        }
        "samus-movement/WallJump" => &[O::WallTest],
        "samus-movement/TouchedCells" => &[O::Reached],
        "audio-transport/OfferAcceptance" => &[O::SenderQueue],
        "audio-transport/MusicalProgress"
        | "audio-transport/WholeArrangement"
        | "audio-transport/FinitePhrases" => &[O::AudioSemantic],
        "audio-transport/VisibleCompletion" => &[O::AudioPublications, O::AudioPendingMusic],
        "audio-transport/ReplyCadence" => &[O::AudioPublications, O::AudioReplies],
        "boss-ceres-ridley/Waiting" | "boss-ceres-ridley/Polling" => &[O::CeresPhase, O::CeresWait],
        "boss-ceres-ridley/SheIsNear" | "boss-ceres-ridley/Closing" => {
            &[O::CeresPhase, O::CeresFigure]
        }
        "boss-ceres-ridley/Shown" => &[O::CeresShown],
        "boss-ceres-ridley/Solid" => &[O::CeresSolid],
        "world-camera/FollowingHer"
        | "world-camera/Remembering"
        | "world-camera/RememberedHeightFraction" => &[
            O::CameraViewXPixel,
            O::CameraViewXFraction,
            O::CameraViewYPixel,
            O::CameraViewYFraction,
            O::CameraRememberedXPixel,
            O::CameraRememberedXFraction,
            O::CameraRememberedYPixel,
            O::CameraRememberedYFraction,
            O::CameraVerticalFraction,
        ],
        "weapons-projectiles/ShotSpacing"
        | "weapons-projectiles/HoldingShoot"
        | "weapons-projectiles/CannonOrigin"
        | "weapons-projectiles/InheritedMotion"
        | "weapons-projectiles/FlyingBeam" => &[O::WeaponsState],
        "enemy-interactions/ContactAllowed"
        | "enemy-interactions/BoxOverlap"
        | "enemy-interactions/FirstPowerShot" => &[O::ContactConsumedMarkers],
        "interface-hud/HerBlink" | "interface-hud/MiniMapWindow" => &[O::HudGrid],
        "interface-hud/Exploring" => &[O::HudExplored],
        "samus-appearance/ClipPlays" => &[O::AppearanceStep, O::AppearanceTimeLeft],
        "run-progress/EventsHappen" => &[O::ProgressPlanetAwake],
        _ => return None,
    })
}

fn connected_provider_valid(case: &Case, observations: &BTreeSet<ObservationId>) -> bool {
    use ObservationId as O;

    case.kind == CaseKind::Integrated
        && case.expectations.provider == ExpectedProvider::PinnedRecordCache
        && case.content == ContentProvider::PlayerRomImport
        && case.inputs == InputProvider::CertifiedMovementCeresInputs
        && case.row == "chain-any-0003-DFD7"
        && matches!(case.start, Start::CacheCut { frame: 6 })
        && case.end.cut_inclusive == 10
        && case.actor_slot == Some(0)
        && !case.checkpoints
        && !case.borrowed.is_empty()
        && case.expectations.outputs.is_empty()
        && *observations
            == BTreeSet::from([
                O::PositionX,
                O::PositionY,
                O::CeresPhase,
                O::CeresWait,
                O::CeresFigure,
                O::CeresShown,
                O::CeresSolid,
            ])
}

fn native_search_provider_valid(case: &Case, observations: &BTreeSet<ObservationId>) -> bool {
    use ObservationId as O;
    let source = match &case.start {
        Start::NativeOpportunity {
            sha256,
            video_frame: 9233,
            source_start_index: 424,
            source_completion_index: 4283,
            ..
        } if sha256 == super::native_search::PACKET_PIN => {
            *observations
                == BTreeSet::from([
                    O::FirstMotionSearch,
                    O::FirstMotionCandidates,
                    O::MovementWork,
                    O::MovementTests,
                    O::CompleteReached,
                    O::Inside,
                    O::MovementSkip,
                ])
        }
        Start::NativeOpportunity {
            sha256,
            video_frame: 9233,
            source_start_index: 424,
            source_completion_index: 4052,
            ..
        } if sha256 == super::native_search::TOUCH_PACKET_PIN => {
            *observations
                == BTreeSet::from([
                    O::FirstMotionTerrain,
                    O::FirstMotionSearch,
                    O::FirstMotionCandidates,
                    O::MovementWork,
                    O::MovementTests,
                    O::CompleteReached,
                    O::Inside,
                    O::MovementSkip,
                ])
                && case.expectations.fields.first() == Some(&O::FirstMotionTerrain)
        }
        Start::NativeOpportunity { .. }
        | Start::NativeSeed { .. }
        | Start::NativeCheckpoint { .. }
        | Start::CacheCut { .. }
        | Start::NativeSenderOffer { .. }
        | Start::ConstructedSenderOffer { .. }
        | Start::PowerOn => false,
    };
    source
        && case.kind == CaseKind::Scoped
        && case.content == ContentProvider::PlayerRomImport
        && case.expectations.provider == ExpectedProvider::NativeCompleteMovementWork
        && case.end.cut_inclusive == super::native_search::COMPLETION
        && !case.limits.is_empty()
        && !case.borrowed.is_empty()
}

fn movement_provider_valid(
    case: &Case,
    observations: &BTreeSet<ObservationId>,
    recorded: bool,
) -> bool {
    if case.inputs == InputProvider::NativeSearchInputs {
        return native_search_provider_valid(case, observations);
    }
    recorded
        && case.content == ContentProvider::PlayerRomImport
        && case.inputs == InputProvider::PrecedingCutMovementInputs
}

fn ordinary_provider_valid(case: &Case, observations: &BTreeSet<ObservationId>) -> bool {
    use ObservationId as O;

    case.kind == CaseKind::Integrated
        && case.expectations.provider == ExpectedProvider::PinnedRecordCache
        && match case.id.0.as_str() {
            "connected-ordinary/carried-running-camera" => {
                matches!(case.start, Start::CacheCut { frame: 6 })
                    && (10..=15).contains(&case.end.cut_inclusive)
            }
            "connected-ordinary/run-release-floor-aim" => {
                matches!(case.start, Start::CacheCut { frame: 126 })
                    && case.end.cut_inclusive == 142
            }
            _ => false,
        }
        && case.row == "chain-any-0003-DFD7"
        && case.content == ContentProvider::PlayerRomImport
        && case.inputs == InputProvider::ConnectedOrdinaryInputs
        && observations.len() == 21
        && case.expectations.fields.len() == 17
        && case.expectations.outputs == [O::TravelLeft, O::TravelRight, O::TravelUp, O::TravelDown]
}

fn sender_offer_provider_valid(case: &Case) -> bool {
    use ObservationId as O;

    case.responsibilities == [Responsibility::AudioSender]
        && case.kind == CaseKind::Scoped
        && case.content == ContentProvider::SenderGroup
        && case.inputs == InputProvider::SenderOffers
        && case.expectations.provider == ExpectedProvider::SenderOfferQueue
        && matches!(
            case.start,
            Start::NativeSenderOffer { .. } | Start::ConstructedSenderOffer { .. }
        )
        && case.expectations.fields == [O::SenderQueue, O::SenderProgress]
        && case.expectations.outputs.is_empty()
        && !case.borrowed.is_empty()
        && !case.limits.is_empty()
}

fn provider_valid(case: &Case, observations: &BTreeSet<ObservationId>) -> bool {
    use ObservationId as O;
    if case.inputs == InputProvider::Unavailable {
        return case.kind == CaseKind::Scoped
            && matches!(
                case.responsibilities.as_slice(),
                [Responsibility::Blocks | Responsibility::Doors | Responsibility::AudioSender]
            )
            && case.rules.is_empty()
            && observations.is_empty()
            && !case.checkpoints
            && !case.limits.is_empty();
    }
    let recorded = case.kind == CaseKind::Scoped
        && case.expectations.provider == ExpectedProvider::PinnedRecordCache
        && matches!(case.start, Start::CacheCut { .. });
    match case.responsibilities.as_slice() {
        [
            Responsibility::Movement,
            Responsibility::Appearance,
            Responsibility::Camera,
        ] => ordinary_provider_valid(case, observations),
        [Responsibility::Movement, Responsibility::Ceres] => {
            connected_provider_valid(case, observations)
        }
        [Responsibility::Movement] => movement_provider_valid(case, observations, recorded),
        [Responsibility::Audio] => {
            case.kind == CaseKind::Scoped
                && case.content == ContentProvider::NativeResidentContent
                && case.inputs == InputProvider::NativePhysicalPresentations
                && case.expectations.provider == ExpectedProvider::NativeDecodedCuts
                && matches!(
                    case.start,
                    Start::NativeSeed { .. } | Start::NativeCheckpoint { .. }
                )
                && *observations
                    == BTreeSet::from([
                        O::AudioSemantic,
                        O::AudioPublications,
                        O::AudioReplies,
                        O::AudioPendingMusic,
                    ])
                && case.expectations.output_cut.is_none()
        }
        [Responsibility::Ceres] => {
            recorded
                && case.content == ContentProvider::CeresReference
                && case.inputs == InputProvider::PrecedingCutCeresInputs
                && case.actor_slot.is_some_and(|slot| slot < 32)
                && case.expectations.outputs.is_empty()
                && *observations
                    == BTreeSet::from([
                        O::CeresPhase,
                        O::CeresWait,
                        O::CeresFigure,
                        O::CeresShown,
                        O::CeresSolid,
                    ])
        }
        [Responsibility::Camera] => {
            recorded
                && case.content == ContentProvider::RecordedScrollRun
                && case.inputs == InputProvider::PrecedingCutCameraInputs
                && case.row == "item-morph-ball"
                && matches!(case.start, Start::CacheCut { frame: 1215 })
                && case.end.cut_inclusive == 1216
                && case.expectations.outputs.is_empty()
                && observations.len() == 9
        }
        [Responsibility::Weapons, Responsibility::Contact] => {
            recorded
                && case.content == ContentProvider::PlayerRomImport
                && case.inputs == InputProvider::PrecedingCutWeaponsContactInputs
                && case.expectations.fields == [O::WeaponsState]
                && case.expectations.outputs == [O::ContactConsumedMarkers]
                && case.expectations.output_cut.is_none()
        }
        [Responsibility::Hud] => {
            recorded
                && case.content == ContentProvider::HudReference
                && case.inputs == InputProvider::PrecedingCutHudInputs
                && case.expectations.outputs.is_empty()
                && observations.len() == 10
                && case.expectations.output_cut.is_none()
        }
        [Responsibility::Appearance] => {
            recorded
                && case.content == ContentProvider::PlayerRomImport
                && case.inputs == InputProvider::PrecedingCutAppearanceInputs
                && case.expectations.outputs.is_empty()
                && observations.len() == 4
        }
        [Responsibility::Progress] => {
            recorded
                && case.content == ContentProvider::PlayerRomImport
                && case.inputs == InputProvider::ObservedFixedProgressEvents
                && case.expectations.outputs.is_empty()
                && observations.len() == 19
        }
        _ => case.kind != CaseKind::Scoped && case.borrowed.is_empty(),
    }
}

const FINAL_OWNER_FIELDS: [ObservationId; 7] = [
    ObservationId::FrameTime,
    ObservationId::PlayTimeHours,
    ObservationId::PlayTimeMinutes,
    ObservationId::PlayTimeSeconds,
    ObservationId::PlayTimeFrames,
    ObservationId::RoomShakeTimer,
    ObservationId::RoomShakePhase,
];

const FINAL_HEALTH_FIELDS: [ObservationId; 5] = [
    ObservationId::HealthEnergyWhole,
    ObservationId::HealthEnergyFraction,
    ObservationId::HealthPendingWhole,
    ObservationId::HealthPendingFraction,
    ObservationId::HealthWarning,
];

pub(super) const PLAY_TIME_RULE: &str = "game-time/PlayTimeAdvance";
pub(super) const ROOM_QUAKE_RULE: &str = "world-quake/QuietQuakePhase";

impl Case {
    fn native_frame_fields(&self) -> &[ObservationId] {
        self.expectations
            .fields
            .strip_suffix(&[ObservationId::FrameRandom])
            .unwrap_or(&self.expectations.fields)
    }

    pub(super) fn compares_final_frame_health(&self) -> bool {
        self.native_frame_fields()
            .strip_prefix(FINAL_OWNER_FIELDS.as_slice())
            == Some(FINAL_HEALTH_FIELDS.as_slice())
    }

    pub(super) fn compares_final_frame_owners(&self) -> bool {
        (self.native_frame_fields() == FINAL_OWNER_FIELDS || self.compares_final_frame_health())
            && self.responsibilities.contains(&Responsibility::GameTime)
            && self.responsibilities.contains(&Responsibility::WorldQuake)
            && [PLAY_TIME_RULE, ROOM_QUAKE_RULE]
                .into_iter()
                .all(|id| self.rules.iter().any(|rule| rule == id))
    }
}

fn scope_valid(case: &Case) -> bool {
    let owners: BTreeSet<_> = case.responsibilities.iter().copied().collect();
    if owners.len() != case.responsibilities.len()
        || (case.actor_slot.is_some()
            && case.responsibilities != [Responsibility::Ceres]
            && case.responsibilities != [Responsibility::Movement, Responsibility::Ceres])
    {
        return false;
    }
    let observations: BTreeSet<_> = case
        .expectations
        .fields
        .iter()
        .chain(&case.expectations.outputs)
        .copied()
        .collect();
    if observations
        .iter()
        .any(|field| !owners.contains(&observation_owner(*field)))
    {
        return false;
    }
    if case.inputs == InputProvider::NativeFrameOwners {
        let mut frame_owners = owners.clone();
        frame_owners.remove(&Responsibility::Health);
        let haze = frame_owners.remove(&Responsibility::WorldHaze);
        let random = frame_owners.remove(&Responsibility::GameRandom);
        let compares_final_owners = case.compares_final_frame_owners();
        if compares_final_owners {
            frame_owners.remove(&Responsibility::GameTime);
            frame_owners.remove(&Responsibility::WorldQuake);
        }
        return case.kind == CaseKind::Integrated
            && case.content == ContentProvider::PlayerRomImport
            && haze
                == case
                    .rules
                    .iter()
                    .any(|rule| rule.starts_with("world-haze/"))
            && random == case.expectations.native_random.is_some()
            && case.expectations.provider == ExpectedProvider::NativeFrameCallbacks
            && matches!(case.start, Start::NativeSeed { .. })
            && frame_owners
                == BTreeSet::from([
                    Responsibility::Frame,
                    Responsibility::Movement,
                    Responsibility::Hud,
                    Responsibility::Appearance,
                    Responsibility::AudioSender,
                ])
            && (compares_final_owners || case.native_frame_fields() == [ObservationId::FrameTime])
            && case.expectations.outputs == [ObservationId::FrameOutputs]
            && case.expectations.output_cut.is_none()
            && !case.borrowed.is_empty()
            && case.rules.iter().all(|rule| {
                let owner = match rule.as_str() {
                    "audio-transport/OfferAcceptance"
                    | "audio-transport/RequestDelivery"
                    | "audio-transport/RequestRelease"
                    | "audio-transport/SenderTiming" => Some(Responsibility::AudioSender),
                    _ => rule_owner(rule),
                };
                owner.is_some_and(|owner| owners.contains(&owner))
            });
    }
    for rule in &case.rules {
        if rule_owner(rule).is_none_or(|owner| !owners.contains(&owner)) {
            return false;
        }
        if witnessed_fields(rule)
            .is_none_or(|fields| fields.iter().any(|field| !observations.contains(field)))
        {
            return false;
        }
    }
    if let Some(at) = case.expectations.output_cut
        && (case.first_cut().is_none_or(|first| at < first)
            || at > case.end.cut_inclusive
            || case.expectations.outputs.is_empty())
    {
        return false;
    }
    if (observations.contains(&ObservationId::WallTest)
        || observations.contains(&ObservationId::Reached))
        && (case.row != "item-morph-ball" || case.expectations.output_cut != Some(817))
    {
        return false;
    }
    sender_offer_provider_valid(case) || provider_valid(case, &observations)
}

impl Case {
    /// Validate the complete public execution recipe before source admission.
    ///
    /// # Errors
    /// Invalid schedules, unsupported provider combinations and unknown rules are rejected.
    pub fn validate(&self) -> Result<(), IndexFault> {
        let case = self;
        for rule in &case.rules {
            if !rule_exists(rule) {
                return Err(IndexFault::UnknownRule(rule.clone()));
            }
        }
        let observations: Vec<_> = case
            .expectations
            .fields
            .iter()
            .chain(&case.expectations.outputs)
            .collect();
        let unique: BTreeSet<_> = observations.iter().copied().collect();
        let start_valid = match &case.start {
            Start::NativeOpportunity {
                path,
                sha256,
                video_frame,
                source_start_index,
                source_completion_index,
            } => {
                !path.as_os_str().is_empty()
                    && valid_hash(sha256)
                    && *video_frame == case.end.cut_inclusive
                    && source_start_index < source_completion_index
            }
            Start::CacheCut { frame } => *frame < case.end.cut_inclusive,
            Start::NativeSeed { path, sha256 } => {
                !path.as_os_str().is_empty() && valid_hash(sha256)
            }
            Start::NativeCheckpoint {
                path,
                sha256,
                seed_path,
                seed_sha256,
                frame,
            } => {
                !path.as_os_str().is_empty()
                    && !seed_path.as_os_str().is_empty()
                    && valid_hash(sha256)
                    && valid_hash(seed_sha256)
                    && *frame < case.end.cut_inclusive
            }
            Start::NativeSenderOffer { source_id } => {
                *source_id < super::sender_offer::NATIVE_WINDOWS
                    && case.row == super::sender_offer::SOURCE_ROW
            }
            Start::ConstructedSenderOffer { .. } => {
                case.row == "constructed-sender-queue" && case.end.cut_inclusive == 0
            }
            Start::PowerOn => case.kind == CaseKind::Movie && case.borrowed.is_empty(),
        };
        if case.id.0.is_empty()
            || case.row.is_empty()
            || (case.rules.is_empty() && case.inputs != InputProvider::Unavailable)
            || case.profile != "reference"
            || case.responsibilities.is_empty()
            || (observations.is_empty() && case.inputs != InputProvider::Unavailable)
            || unique.len() != observations.len()
            || case.expectations.native_random.is_some()
                != case
                    .expectations
                    .fields
                    .contains(&ObservationId::FrameRandom)
            || case
                .expectations
                .native_random
                .as_ref()
                .is_some_and(|source| {
                    case.inputs != InputProvider::NativeFrameOwners
                        || source.path.as_os_str().is_empty()
                        || !valid_hash(&source.sha256)
                })
            || !start_valid
            || case.end.native_returns.as_ref().is_some_and(|returns| {
                case.inputs != InputProvider::NativeFrameOwners
                    || returns.first_cut == 0
                    || returns.first_cut >= case.end.cut_inclusive
                    || returns.path.as_os_str().is_empty()
                    || !valid_hash(&returns.sha256)
            })
            || !scope_valid(case)
            || case.borrowed.iter().any(|borrowed| {
                borrowed.dependency.is_empty()
                    || borrowed.provider.is_empty()
                    || borrowed.fields.is_empty()
                    || borrowed.phase.is_empty()
                    || borrowed.claim.is_empty()
            })
        {
            return Err(IndexFault::InvalidCase(case.id.clone()));
        }
        Ok(())
    }

    /// First original comparison coordinate in this uninterrupted case.
    #[must_use]
    pub const fn first_cut(&self) -> Option<u64> {
        match self.start {
            Start::CacheCut { frame } | Start::NativeCheckpoint { frame, .. } => {
                frame.checked_add(1)
            }
            Start::NativeOpportunity { video_frame, .. } => Some(video_frame),
            Start::NativeSenderOffer { .. } | Start::ConstructedSenderOffer { .. } => {
                Some(self.end.cut_inclusive)
            }
            Start::NativeSeed { .. } if matches!(self.inputs, InputProvider::NativeFrameOwners) => {
                Some(match &self.end.native_returns {
                    Some(returns) => returns.first_cut,
                    None => self.end.cut_inclusive,
                })
            }
            Start::NativeSeed { .. } | Start::PowerOn => Some(0),
        }
    }
}

impl Index {
    /// Read and validate a strict index file.
    ///
    /// # Errors
    /// Missing or malformed indices are rejected.
    pub fn read(path: &Path) -> Result<Self, IndexFault> {
        let text = std::fs::read_to_string(path).map_err(|_| IndexFault::Missing)?;
        Self::parse(&text)
    }

    /// Decode and validate complete case metadata.
    ///
    /// # Errors
    /// Unknown keys, schemas, identities and empty observations are rejected.
    pub fn parse(text: &str) -> Result<Self, IndexFault> {
        let index: Self = toml::from_str(text).map_err(|_| IndexFault::Syntax)?;
        index.validate()?;
        Ok(index)
    }

    /// Validate a publicly constructed index with the same contracts as a read index.
    ///
    /// # Errors
    /// Invalid schemas, empty or duplicate case collections and invalid cases are rejected.
    pub fn validate(&self) -> Result<(), IndexFault> {
        if self.format != 1
            || self.engine_api != "production-v1"
            || self.observation_schema != "semantic-v1"
        {
            return Err(IndexFault::Schema);
        }
        if self.cases.is_empty() {
            return Err(IndexFault::Empty);
        }
        let mut ids = BTreeSet::new();
        for case in &self.cases {
            if !ids.insert(&case.id) {
                return Err(IndexFault::DuplicateCase(case.id.clone()));
            }
            case.validate()?;
        }
        Ok(())
    }

    /// Resolve filters without empty or duplicate success.
    ///
    /// # Errors
    /// Unknown identities, duplicate selections and empty results are rejected.
    pub fn select(
        &self,
        ids: &[CaseId],
        rule: Option<&str>,
        kind: Option<CaseKind>,
    ) -> Result<Vec<&Case>, IndexFault> {
        let mut unique = BTreeSet::new();
        for id in ids {
            if !unique.insert(id) {
                return Err(IndexFault::DuplicateSelection(id.clone()));
            }
            if !self.cases.iter().any(|case| &case.id == id) {
                return Err(IndexFault::UnknownCase(id.clone()));
            }
        }
        if let Some(rule) = rule
            && !rule_exists(rule)
        {
            return Err(IndexFault::UnknownRule(rule.to_owned()));
        }
        let selected: Vec<_> = self
            .cases
            .iter()
            .filter(|case| ids.is_empty() || ids.contains(&case.id))
            .filter(|case| rule.is_none_or(|rule| case.rules.iter().any(|named| named == rule)))
            .filter(|case| kind.is_none_or(|kind| case.kind == kind))
            .collect();
        if selected.is_empty() {
            Err(IndexFault::Empty)
        } else {
            Ok(selected)
        }
    }
}

/// Whether a hash is exactly 64 lowercase hexadecimal digits.
#[must_use]
pub fn valid_hash(hash: &str) -> bool {
    hash.len() == 64
        && hash
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::{CaseId, Index, IndexFault};
    const INDEX: &str = include_str!("../../../../scenarios/replay/index.toml");
    #[test]
    fn committed_cases_resolve_real_rules_and_empty_selections_fail() {
        let index = Index::parse(INDEX).unwrap();
        assert_eq!(
            index.select(&[], None, None).unwrap().len(),
            index.cases.len()
        );
        assert!(matches!(
            index.select(&[], Some("audio-transport/RequestDelivery"), None),
            Err(IndexFault::Empty)
        ));
        let id = CaseId("audio/native-control".to_owned());
        assert!(matches!(
            index.select(&[id.clone(), id], None, None),
            Err(IndexFault::DuplicateSelection(_))
        ));
        assert!(matches!(
            index.select(&[CaseId("missing/case".to_owned())], None, None),
            Err(IndexFault::UnknownCase(_))
        ));
    }
    #[test]
    fn malformed_fields_unknown_keys_and_fictitious_rules_are_rejected() {
        assert!(matches!(
            Index::parse(&INDEX.replace("movement.travel.Left", "movement.travel.Unknown")),
            Err(IndexFault::Syntax)
        ));
        assert!(matches!(
            Index::parse(&INDEX.replace("format = 1", "format = 1\nextra = true")),
            Err(IndexFault::Syntax)
        ));
        assert!(matches!(
            Index::parse(&INDEX.replace("TravelledDistances", "ImaginaryRule")),
            Err(IndexFault::UnknownRule(_))
        ));
    }

    #[test]
    fn public_recipe_cannot_add_unwitnessed_rules_or_relabel_scope() {
        let index = Index::parse(INDEX).unwrap();
        let mut case = index.cases.first().unwrap().clone();
        case.rules.push("samus-movement/SpaceJump".to_owned());
        assert!(case.validate().is_err());
        let mut case = index.cases.first().unwrap().clone();
        case.kind = super::CaseKind::Integrated;
        assert!(matches!(case.validate(), Err(IndexFault::InvalidCase(_))));
        case.kind = super::CaseKind::Scoped;
        case.start = super::Start::CacheCut {
            frame: case.end.cut_inclusive,
        };
        assert!(matches!(case.validate(), Err(IndexFault::InvalidCase(_))));
    }
}
