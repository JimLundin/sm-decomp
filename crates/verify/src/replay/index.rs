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
    /// The intro stage, scripts and their frame publications.
    Intro,
    /// Intro room installation, actor handoff and the first playable turn.
    IntroRooms,
    /// The Zebes flashback encounter.
    Flashback,
    /// The recorded intro pad and update authority.
    Playback,
    /// Complete cartridge persistence.
    SaveFiles,
    /// The existing movement and semantic pose owner.
    Movement,
    /// Audio transport, played one frame at a time.
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
    /// The sound queues' offer acceptance.
    AudioSender,
    /// Complete live health, including fractional energy and pending periodic loss.
    Health,
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
    /// One independently pinned native record supplies the sole scoped origin.
    NativeRecord {
        /// File under the native fixture directory.
        path: PathBuf,
        /// SHA-256 of the entire record.
        sha256: String,
        /// Original cut coordinate.
        frame: u64,
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Independent source of changing causal stimuli.
pub enum InputProvider {
    /// Movie pad, room seed and the played platform boundaries for a Ceres tail window.
    PlayedCeresInputs,
    /// The movie pad, platform completion and the Zebes encounter cue for scoped intro play.
    ScopedIntroInputs,
    /// A causal intro start and the borrowed work-completion boundaries.
    IntroRoomJoinInputs,
    /// Locked encounter phase, after-motion room facts and presentation opportunities.
    FlashbackInputs,
    /// Recorded scene setup and reached room actor opportunities.
    CinematicActorOpportunities,
    /// Ordered startup and save boundaries from pinned preceding cuts.
    SaveFileBoundaries,
    /// Authored demo controls and borrowed cinematic weapon opportunities.
    IntroMissileInputs,
    /// Independent passage phase and sampled button at each draw opportunity.
    PassageJumpDraws,
    /// Scoped discovery and delivery inputs, including the interrupted start.
    ScopedIntroDeliveryInputs,
    /// Locked current Movement stimuli and early mini-map premises.
    EarlyMapInputs,
    /// Independent scoped actor query/delivery, current equipment and active close.
    NativeHealthTailInputs,
    /// Authenticated actual-use snapshots for the connected ordinary family.
    ConnectedOrdinaryInputs,
    /// Independently admitted selector-time Frame and actor-selection-time geometry.
    NativeSearchInputs,
    /// Prior locked Movement state, actual solid actors and independent sender ingress.
    NativeTouchdownInputs,
    /// Causal neighbours from the preceding pinned cut.
    PrecedingCutMovementInputs,
    /// Independent prior facts plus observed external Ceres run scheduling.
    PrecedingCutCeresInputs,
    /// Independent prior facts while captured camera producers remain unchanged.
    PrecedingCutCameraInputs,
    /// Independent cannon/terrain facts and bounded watched actor opportunities.
    PrecedingCutWeaponsContactInputs,
    /// Independently watched ordinary weapon inputs and the flying-picture insertion context.
    ScopedShotDrawingInputs,
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
    /// Each frame's offers and audio cadence, from the native capture's port log.
    NativeFrameInputs,
    /// Power-on play using movie inputs.
    Movie,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Source of immutable admitted content.
pub enum ContentProvider {
    /// The draw count uses fixed rules and no imported content.
    PassageJump,
    /// Checked content imported from the player ROM.
    PlayerRomImport,
    /// Silent audio content: no clips or song, for frames that only queue offers.
    SenderGroup,
    /// ROM-selected audio for the independently admitted opening-room load.
    OpeningCeresAudio,
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
    /// Independent native early Map snapshots and carried cut record.
    NativeEarlyMap,
    /// Independently observed Health, Drawing reads and HUD gauge around a tail hit.
    NativeHealthTailObservations,
    /// Frozen complete source-derived movement work, with independent captured input packets.
    NativeCompleteMovementWork,
    /// Live floor, fit and consumer observations, with the independent Contact FIFO.
    NativeTouchdownObservations,
    /// Independent pinned oracle records.
    PinnedRecordCache,
    /// Independent queue and committed progress.
    SenderOfferQueue,
    /// Pinned slot state and independently watched exact shot insertions.
    PinnedShotDrawingObservations,
    /// Independent decoded native cuts.
    NativeDecodedCuts,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
/// A concrete supported compared field or output stream.
pub enum ObservationId {
    /// Ridley's carried body, held baby and retreat phase, with the station still calm.
    #[serde(rename = "ridley.retreat.body")]
    CeresRetreatBody,
    /// The boundary his move met, if any.
    #[serde(rename = "ridley.retreat.bound")]
    CeresRetreatBound,
    /// The pair of placements at the stopped cut.
    #[serde(rename = "ridley.backdrop.walls")]
    CeresBackdropWalls,
    /// Three independent current and native physical palette spans at publication.
    #[serde(rename = "ridley.backdrop.colours")]
    CeresBackdropColours,
    /// Complete carried flight tail, compared by segment and field.
    #[serde(rename = "ridley.tail.state")]
    CeresTailState,
    /// Seven tail places as produced for drawing.
    #[serde(rename = "ridley.tail.places")]
    CeresTailPlaces,
    /// The hit's side and initial push time.
    #[serde(rename = "ridley.tail.hit")]
    CeresTailHit,
    /// The tail's swish request.
    #[serde(rename = "ridley.tail.swish-request")]
    CeresTailSwish,
    /// The movement pose and push reaction at the cut.
    #[serde(rename = "movement.tail-reaction")]
    CeresTailKnockback,
    /// Energy, protection, pending loss and warning at the cut.
    #[serde(rename = "health.tail-reaction")]
    CeresTailHealth,
    /// Intro stage, scripts, dissolves, visibility and brightness.
    #[serde(rename = "intro.state")]
    IntroState,
    /// Intro encounter, layout, text, sound and palette publications.
    #[serde(rename = "intro.publications")]
    IntroPublications,
    /// Logical width, height, all backed words and their marks.
    #[serde(rename = "intro-room.grid")]
    IntroRoomGrid,
    /// Samus and demo at the room join.
    #[serde(rename = "intro-room.actor")]
    IntroRoomActor,
    /// The carried finished-work count.
    #[serde(rename = "intro-room.finished-frames")]
    IntroRoomClock,
    /// Pending scene start.
    #[serde(rename = "flashback.pending-start")]
    FlashbackStart,
    /// Carried background Y.
    #[serde(rename = "flashback.background-y")]
    FlashbackY,
    /// Complete boss State.
    #[serde(rename = "flashback.brain")]
    FlashbackBrain,
    /// Any scheduled pair and its remaining wait.
    #[serde(rename = "flashback.pair-schedule")]
    FlashbackPair,
    /// Each ring, with position, rank and clip.
    #[serde(rename = "flashback.rings")]
    FlashbackRings,
    /// Each burst, with position, rank and clip.
    #[serde(rename = "flashback.bursts")]
    FlashbackBursts,
    /// A fresh encounter turn.
    #[serde(rename = "flashback.turn")]
    FlashbackTurn,
    /// The weapon slot requested for conversion to impact.
    #[serde(rename = "flashback.missile-impact")]
    FlashbackImpact,
    /// The push direction and holds the ring sets.
    #[serde(rename = "flashback.ring-knockback")]
    FlashbackPush,
    /// The selection of colours written by the flash.
    #[serde(rename = "flashback.flash")]
    FlashbackFlash,
    /// The one-turn page-two cue.
    #[serde(rename = "flashback.page-two")]
    FlashbackCue,
    /// The request to hide Samus and her shots.
    #[serde(rename = "flashback.hide")]
    FlashbackHide,
    /// The flash write to the palette mirror.
    #[serde(rename = "flashback.palette-mirror")]
    FlashbackMirror,
    /// The flash row as observed in CGRAM.
    #[serde(rename = "flashback.cgram")]
    FlashbackCgram,
    /// Each visible encounter OAM piece and its high bits in order.
    #[serde(rename = "flashback.oam")]
    FlashbackOam,
    /// The signed Y published to the room background.
    #[serde(rename = "flashback.bg1-y")]
    FlashbackBg1Y,

    /// Current playback phase.
    #[serde(rename = "playback.phase")]
    PlaybackPhase,
    /// Issued opportunities within the active timeline.
    #[serde(rename = "playback.progress")]
    PlaybackProgress,
    /// Last retained held buttons.
    #[serde(rename = "playback.last.held")]
    PlaybackLastHeld,
    /// Last retained fresh presses.
    #[serde(rename = "playback.last.fresh")]
    PlaybackLastFresh,
    /// Current held buttons delivered to the actors.
    #[serde(rename = "playback.current.held")]
    PlaybackCurrentHeld,
    /// Current fresh presses delivered to the actors.
    #[serde(rename = "playback.current.fresh")]
    PlaybackCurrentFresh,
    /// Previous held buttons delivered to the actors.
    #[serde(rename = "playback.previous.held")]
    PlaybackPreviousHeld,
    /// Previous fresh presses delivered to the actors.
    #[serde(rename = "playback.previous.fresh")]
    PlaybackPreviousFresh,
    /// Scene setup handoff.
    #[serde(rename = "playback.start")]
    PlaybackStart,
    /// Terminal control handoff.
    #[serde(rename = "playback.finish")]
    PlaybackFinish,
    /// Work authorized on this room actor opportunity.
    #[serde(rename = "playback.authority")]
    PlaybackAuthority,
    /// The complete cartridge image at the selected cut.
    #[serde(rename = "save-files.battery")]
    Battery,
    /// Carried ammunition, selection and protection.
    #[serde(rename = "weapons.intro-stock")]
    IntroMissileStock,
    /// Cannon clock and shot count.
    #[serde(rename = "weapons.intro-spacing")]
    IntroMissileSpacing,
    /// Five weapon slots and retained trail holds.
    #[serde(rename = "weapons.intro-slots")]
    IntroMissileSlots,
    /// Live exhaust positions and clip holds.
    #[serde(rename = "weapons.intro-exhaust")]
    IntroMissileExhaust,
    /// Fresh ammunition debit.
    #[serde(rename = "weapons.intro-release")]
    IntroMissileRelease,
    /// Applied encounter request by weapon slot.
    #[serde(rename = "weapons.intro-impact")]
    IntroMissileImpact,
    /// Exhaust requests and their pre-motion centres.
    #[serde(rename = "weapons.intro-puffs")]
    IntroMissilePuffs,
    /// After-flight positions in encounter precedence.
    #[serde(rename = "weapons.intro-candidates")]
    IntroMissileCandidates,
    /// Missile point contact requests.
    #[serde(rename = "weapons.intro-point-reads")]
    IntroMissilePoints,
    /// Weapon launch and impact sound requests.
    #[serde(rename = "weapons.intro-sound")]
    IntroMissileSound,
    /// Ordered missile and exhaust OAM bytes.
    #[serde(rename = "weapons.intro-oam")]
    IntroMissileOam,
    /// Jump count and pending landing or end check.
    #[serde(rename = "movement.drawn-jump-held")]
    DrawnJumpHeld,
    /// The input sampled on the last history-writing draw.
    #[serde(rename = "movement.last-draw-jump")]
    LastDrawJump,
    /// Semantic body and cannon pose selected on landing.
    #[serde(rename = "movement.landing.pose")]
    LandingPose,
    /// The semantic body and cannon pose after ordinary movement.
    #[serde(rename = "movement.pose")]
    MovementPose,
    /// The horizontal base speed retained after landing.
    #[serde(rename = "movement.base-speed")]
    MovementBaseSpeed,
    /// The grounded upright floor response.
    #[serde(rename = "movement.landing.floor")]
    LandingFloor,

    /// Exact 15 mini-map cells in display order.
    #[serde(rename = "hud.map-window")]
    EarlyMapWindow,
    /// Complete finished ordinary Movement state.
    #[serde(rename = "movement.ordinary-completion")]
    MapCarryMovement,
    /// Complete finished ordinary Appearance state and Figure.
    #[serde(rename = "appearance.ordinary-completion")]
    MapCarryAppearance,
    /// Complete finished ordinary Camera state.
    #[serde(rename = "camera.ordinary-completion")]
    MapCarryCamera,
    /// Complete Health at the frame cut.
    #[serde(rename = "health.tail.state")]
    HealthTailState,
    /// The Warning request, before any receiver.
    #[serde(rename = "health.tail.warning_request")]
    HealthTailWarning,
    /// The protection Drawing reads for the flash after damage, before the frame spends one.
    #[serde(rename = "health.tail.protection_drawn")]
    HealthTailDrawnProtection,
    /// Drawing's energy history at the frame cut.
    #[serde(rename = "appearance.drawing.energy")]
    DrawingEnergy,
    /// The gauge's shown energy at the frame cut and whether the frame redrew it.
    #[serde(rename = "hud.gauge.refresh")]
    GaugeRefresh,
    /// All sixteen CPU-side gauge words.
    #[serde(rename = "hud.gauge.words")]
    GaugeWords,
    /// Complete pending sound queue, oldest first.
    #[serde(rename = "sender.queue")]
    SenderQueue,
    /// Complete committed sender progress.
    #[serde(rename = "sender.progress")]
    SenderProgress,
    /// Her downward travel this frame, after collision.
    #[serde(rename = "movement.touchdown.floor_clip")]
    TouchdownClip,
    /// Her position after her moves, before the pose change moves her.
    #[serde(rename = "movement.touchdown.body_fit")]
    TouchdownFit,
    /// Actual accepted floor response and its speed.
    #[serde(rename = "movement.touchdown.floor_response")]
    TouchdownResponse,
    /// Ordered offer that reached Contact, including its allowance.
    #[serde(rename = "sender.touchdown_offer")]
    TouchdownOffer,
    /// Ordered Appearance clear operation for the selected figure.
    #[serde(rename = "appearance.touchdown_graphics")]
    TouchdownGraphics,
    /// Complete retained movement owner.
    #[serde(rename = "movement.state.complete")]
    MovementStateComplete,
    /// Complete currently playing Figure.
    #[serde(rename = "appearance.playing")]
    AppearancePlaying,
    /// Complete ordered reaction-facing reaches.
    #[serde(rename = "movement.reached.complete")]
    CompleteReached,
    /// All starting inside cells, distinct from leading-edge tests.
    #[serde(rename = "movement.inside")]
    Inside,
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
    #[serde(rename = "audio.replies")]
    /// Reply bytes last published on each line at each cut.
    AudioReplies,
    #[serde(rename = "audio.request_lines")]
    /// Request bytes last presented on each group's line at each cut.
    AudioRequestLines,
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
    /// Reached power spawn and trail request sequence.
    #[serde(rename = "weapons.update-events")]
    WeaponsUpdateEvents,
    /// Ordered physical receiver and shot slots.
    #[serde(rename = "contact.receivers")]
    ContactReceivers,
    /// Selected flying picture intents.
    #[serde(rename = "weapons.flying-pictures")]
    WeaponsFlyingPictures,
    /// Exact low entry and high pair at the reached insertion.
    #[serde(rename = "weapons.oam-insertion")]
    WeaponsOamInsertion,
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
    screens_intro::catalogue()
        .iter()
        .any(|(id, _)| rule == format!("screens-intro/{id:?}"))
        || rule == game::production::played::intro_rooms::RULE
        || screens_flashback::catalogue()
            .iter()
            .any(|entry| rule == format!("screens-flashback/{:?}", entry.id))
        || screens_playback::catalogue()
            .iter()
            .any(|(id, _)| rule == format!("screens-playback/{id:?}"))
        || save_files::CATALOGUE
            .iter()
            .any(|entry| rule == format!("save-files/{:?}", entry.id))
        || samus_movement::CATALOGUE
            .iter()
            .any(|entry| rule == format!("samus-movement/{:?}", entry.id))
        || audio_transport::catalogue::catalogue()
            .iter()
            .any(|(id, _)| rule == format!("audio-transport/{id:?}"))
        || boss_ceres_ridley::catalogue()
            .iter()
            .any(|(id, _)| rule == format!("boss-ceres-ridley/{id:?}"))
        || world_doors::CATALOGUE
            .iter()
            .any(|entry| rule == format!("world-doors/{:?}", entry.id))
        || world_camera::CATALOGUE
            .iter()
            .any(|entry| rule == format!("world-camera/{:?}", entry.id))
        || interface_hud::CATALOGUE
            .iter()
            .any(|entry| rule == format!("interface-hud/{:?}", entry.id))
        || samus_appearance::catalogue()
            .iter()
            .any(|(id, _)| rule == format!("samus-appearance/{id:?}"))
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

macro_rules! flashback_observations {
    () => {
        ObservationId::FlashbackStart
            | ObservationId::FlashbackY
            | ObservationId::FlashbackBrain
            | ObservationId::FlashbackPair
            | ObservationId::FlashbackRings
            | ObservationId::FlashbackBursts
            | ObservationId::FlashbackTurn
            | ObservationId::FlashbackImpact
            | ObservationId::FlashbackPush
            | ObservationId::FlashbackFlash
            | ObservationId::FlashbackCue
            | ObservationId::FlashbackHide
            | ObservationId::FlashbackMirror
            | ObservationId::FlashbackCgram
            | ObservationId::FlashbackOam
            | ObservationId::FlashbackBg1Y
    };
}

macro_rules! weapons_observations {
    () => {
        ObservationId::IntroMissileStock
            | ObservationId::IntroMissileSpacing
            | ObservationId::IntroMissileSlots
            | ObservationId::IntroMissileExhaust
            | ObservationId::IntroMissileRelease
            | ObservationId::IntroMissileImpact
            | ObservationId::IntroMissilePuffs
            | ObservationId::IntroMissileCandidates
            | ObservationId::IntroMissilePoints
            | ObservationId::IntroMissileSound
            | ObservationId::IntroMissileOam
            | ObservationId::WeaponsState
            | ObservationId::WeaponsUpdateEvents
            | ObservationId::WeaponsFlyingPictures
            | ObservationId::WeaponsOamInsertion
    };
}

macro_rules! progress_observations {
    () => {
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
            | ObservationId::ProgressZebetites
    };
}

const fn observation_owner(field: ObservationId) -> Responsibility {
    use ObservationId as O;
    match field {
        O::IntroState | O::IntroPublications => Responsibility::Intro,
        O::IntroRoomGrid | O::IntroRoomActor | O::IntroRoomClock => Responsibility::IntroRooms,
        flashback_observations!() => Responsibility::Flashback,
        O::PlaybackPhase
        | O::PlaybackProgress
        | O::PlaybackLastHeld
        | O::PlaybackLastFresh
        | O::PlaybackCurrentHeld
        | O::PlaybackCurrentFresh
        | O::PlaybackPreviousHeld
        | O::PlaybackPreviousFresh
        | O::PlaybackStart
        | O::PlaybackFinish
        | O::PlaybackAuthority => Responsibility::Playback,
        O::Battery => Responsibility::SaveFiles,
        O::SenderQueue | O::SenderProgress | O::TouchdownOffer => Responsibility::AudioSender,
        O::HealthTailState
        | O::HealthTailWarning
        | O::HealthTailDrawnProtection
        | O::CeresTailHealth => Responsibility::Health,
        O::CeresTailKnockback
        | O::TouchdownClip
        | O::TouchdownFit
        | O::TouchdownResponse
        | O::CompleteReached
        | O::Inside
        | O::MovementStateComplete
        | O::DrawnJumpHeld
        | O::LastDrawJump
        | O::MovementPose
        | O::LandingPose
        | O::MovementBaseSpeed
        | O::LandingFloor
        | O::PositionX
        | O::PositionY
        | O::VerticalSpeed
        | O::TravelLeft
        | O::TravelRight
        | O::TravelUp
        | O::TravelDown
        | O::WallTest
        | O::Reached
        | O::MapCarryMovement => Responsibility::Movement,
        O::AudioSemantic | O::AudioReplies | O::AudioRequestLines => Responsibility::Audio,
        O::CeresPhase
        | O::CeresWait
        | O::CeresFigure
        | O::CeresShown
        | O::CeresSolid
        | O::CeresRetreatBody
        | O::CeresRetreatBound
        | O::CeresBackdropWalls
        | O::CeresBackdropColours
        | O::CeresTailState
        | O::CeresTailPlaces
        | O::CeresTailHit
        | O::CeresTailSwish => Responsibility::Ceres,
        O::CameraViewXPixel
        | O::CameraViewXFraction
        | O::CameraViewYPixel
        | O::CameraViewYFraction
        | O::CameraRememberedXPixel
        | O::CameraRememberedXFraction
        | O::CameraRememberedYPixel
        | O::CameraRememberedYFraction
        | O::CameraVerticalFraction
        | O::MapCarryCamera => Responsibility::Camera,
        O::GaugeRefresh
        | O::GaugeWords
        | O::EarlyMapWindow
        | O::HudGrid
        | O::HudExplored
        | O::HudEnergy
        | O::HudMissiles
        | O::HudSupers
        | O::HudBombs
        | O::HudHighlighted
        | O::HudSelected
        | O::HudOneShot
        | O::HudMapOff => Responsibility::Hud,
        weapons_observations!() => Responsibility::Weapons,
        O::ContactConsumedMarkers | O::ContactReceivers => Responsibility::Contact,
        O::DrawingEnergy
        | O::TouchdownGraphics
        | O::AppearancePlaying
        | O::AppearanceStep
        | O::AppearanceTimeLeft
        | O::AppearanceBoostStage
        | O::AppearanceLapsLeft
        | O::MapCarryAppearance => Responsibility::Appearance,
        progress_observations!() => Responsibility::Progress,
    }
}

fn rule_owner(rule: &str) -> Option<Responsibility> {
    if rule == "audio-transport/OfferAcceptance" {
        return Some(Responsibility::AudioSender);
    }
    [
        ("screens-intro/", Responsibility::Intro),
        ("world-intro-rooms/", Responsibility::IntroRooms),
        ("screens-flashback/", Responsibility::Flashback),
        ("samus-movement/", Responsibility::Movement),
        ("screens-playback/", Responsibility::Playback),
        ("audio-transport/", Responsibility::Audio),
        ("boss-ceres-ridley/", Responsibility::Ceres),
        ("world-camera/", Responsibility::Camera),
        ("world-doors/", Responsibility::Doors),
        ("weapons-projectiles/", Responsibility::Weapons),
        ("enemy-interactions/", Responsibility::Contact),
        ("interface-hud/", Responsibility::Hud),
        ("samus-appearance/", Responsibility::Appearance),
        ("run-progress/", Responsibility::Progress),
        ("save-files/", Responsibility::SaveFiles),
        ("samus-health/", Responsibility::Health),
    ]
    .iter()
    .find_map(|(prefix, owner)| rule.starts_with(prefix).then_some(*owner))
}

fn movement_witnessed_fields(
    rule: &str,
    provider: InputProvider,
) -> Option<&'static [ObservationId]> {
    use ObservationId as O;
    Some(match rule {
        "samus-movement/Jumping" if provider == InputProvider::PassageJumpDraws => {
            &[O::DrawnJumpHeld, O::LastDrawJump]
        }
        "samus-movement/BackwardLeap" | "samus-movement/WhatHerMovesMet" => {
            &[O::LandingPose, O::LandingFloor]
        }
        "samus-movement/MovesAsShePosed" | "samus-movement/PoseFitAfterMoves" => {
            &[O::PositionX, O::PositionY]
        }
        "samus-movement/PadChart" => &[O::MovementPose],
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
        "samus-movement/AcceptedFloorResponse" => &[O::TouchdownResponse, O::VerticalSpeed],
        "samus-movement/TouchdownRequests" => &[O::TouchdownOffer, O::TouchdownGraphics],
        "samus-movement/WallJump" => &[O::WallTest],
        "samus-movement/TouchedCells" if provider == InputProvider::NativeSearchInputs => {
            &[O::CompleteReached, O::Inside]
        }
        "samus-movement/TouchedCells" => &[O::Reached],
        _ => return None,
    })
}

fn witnessed_fields(rule: &str, provider: InputProvider) -> Option<&'static [ObservationId]> {
    use ObservationId as O;
    Some(match rule {
        game::production::played::intro_rooms::RULE => {
            &[O::IntroRoomGrid, O::IntroRoomActor, O::IntroRoomClock]
        }
        "save-files/ReadSlot"
        | "save-files/BootFiles"
        | "save-files/MenuFiles"
        | "save-files/ChooseFile"
        | "save-files/WriteFile" => &[O::Battery],
        "audio-transport/OfferAcceptance" => &[O::SenderQueue],
        "audio-transport/MusicalProgress"
        | "audio-transport/WholeArrangement"
        | "audio-transport/FinitePhrases" => &[O::AudioSemantic],
        "audio-transport/FrameCadence"
        | "audio-transport/PublishedReplies"
        | "audio-transport/StagedRequests" => &[O::AudioReplies],
        "audio-transport/RequestDelivery" | "audio-transport/RequestRelease" => {
            &[O::AudioRequestLines]
        }
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
        "weapons-projectiles/EndingShot" => &[O::WeaponsState, O::ContactConsumedMarkers],
        "weapons-projectiles/BeamTrail" => &[O::WeaponsState, O::WeaponsUpdateEvents],
        "weapons-projectiles/BeamVisibility" => &[O::WeaponsFlyingPictures, O::WeaponsOamInsertion],
        "enemy-interactions/ContactAllowed"
        | "enemy-interactions/BoxOverlap"
        | "enemy-interactions/FirstPowerShot" => &[O::ContactConsumedMarkers],
        "samus-health/PeriodicLoss" | "samus-health/TailHit" => &[O::HealthTailState],
        "samus-health/ProtectionTime" => &[O::HealthTailState, O::HealthTailDrawnProtection],
        "samus-health/LowEnergyWarning" => &[O::HealthTailState, O::HealthTailWarning],
        "interface-hud/EnergyGauge" => &[O::GaugeRefresh, O::GaugeWords],
        "interface-hud/HerBlink" | "interface-hud/MiniMapWindow"
            if provider == InputProvider::EarlyMapInputs =>
        {
            &[O::EarlyMapWindow]
        }
        "interface-hud/HerBlink" | "interface-hud/MiniMapWindow" => &[O::HudGrid],
        "interface-hud/Exploring" => &[O::HudExplored],
        "samus-appearance/ClipPlays" => &[O::AppearanceStep, O::AppearanceTimeLeft],
        "run-progress/EventsHappen" => &[O::ProgressPlanetAwake],
        _ => {
            return movement_witnessed_fields(rule, provider)
                .or_else(|| intro_fields(rule))
                .or_else(|| flashback_fields(rule))
                .or_else(|| playback_fields(rule))
                .or_else(|| intro_missile_fields(rule))
                .or_else(|| ceres_tail_fields(rule));
        }
    })
}

fn ceres_tail_fields(rule: &str) -> Option<&'static [ObservationId]> {
    use ObservationId as O;
    Some(match rule {
        "boss-ceres-ridley/FliesOutOfReach" => &[O::CeresRetreatBody, O::CeresRetreatBound],
        "boss-ceres-ridley/GetawayBackdrop" => &[
            O::CeresRetreatBody,
            O::CeresBackdropWalls,
            O::CeresBackdropColours,
        ],
        "boss-ceres-ridley/TailComesLoose"
        | "boss-ceres-ridley/TailFollows"
        | "boss-ceres-ridley/TailSwings"
        | "boss-ceres-ridley/TailWhips"
        | "boss-ceres-ridley/TailRecovers"
        | "boss-ceres-ridley/TailStretches" => &[O::CeresTailState],
        "boss-ceres-ridley/TailBends" => &[O::CeresTailState, O::CeresTailPlaces],
        "boss-ceres-ridley/TailSwishes" => &[O::CeresTailState, O::CeresTailSwish],
        "boss-ceres-ridley/TailHits" => &[
            O::CeresTailPlaces,
            O::CeresTailHit,
            O::CeresTailKnockback,
            O::CeresTailHealth,
        ],
        _ => return None,
    })
}

fn intro_fields(rule: &str) -> Option<&'static [ObservationId]> {
    use ObservationId as O;
    Some(match rule {
        "screens-intro/DiscoveryBegins"
        | "screens-intro/DiscoveryReturns"
        | "screens-intro/DeliveryBegins"
        | "screens-intro/BabyCrossesThePicture"
        | "screens-intro/PageFourBegins"
        | "screens-intro/DiscoveryAndDeliveryPagesEnd"
        | "screens-intro/ExaminationBegins"
        | "screens-intro/BattleBegins"
        | "screens-intro/BattleDissolves"
        | "screens-intro/PageTwoBegins"
        | "screens-intro/PageTwoEnds" => &[O::IntroState, O::IntroPublications],
        _ => return None,
    })
}

fn intro_missile_fields(rule: &str) -> Option<&'static [ObservationId]> {
    use ObservationId as O;
    Some(match rule {
        "weapons-projectiles/MissileRelease" => &[
            O::IntroMissileStock,
            O::IntroMissileSpacing,
            O::IntroMissileRelease,
            O::IntroMissileSound,
        ],
        "weapons-projectiles/MissileTravel" => &[
            O::IntroMissileSlots,
            O::IntroMissileCandidates,
            O::IntroMissilePoints,
        ],
        "weapons-projectiles/MissileBurst" => &[
            O::IntroMissileSlots,
            O::IntroMissileImpact,
            O::IntroMissileSound,
        ],
        "weapons-projectiles/MissileExhaust" => &[O::IntroMissileExhaust, O::IntroMissilePuffs],
        "weapons-projectiles/CinematicMissilePictures" => {
            &[O::IntroMissileOam, O::IntroMissileExhaust]
        }
        _ => return None,
    })
}

fn flashback_fields(rule: &str) -> Option<&'static [ObservationId]> {
    use ObservationId as O;
    Some(match rule {
        "screens-flashback/EncounterBeats" => &[O::FlashbackTurn],
        "screens-flashback/EncounterStarts" => &[
            O::FlashbackStart,
            O::FlashbackY,
            O::FlashbackBrain,
            O::FlashbackPair,
        ],
        "screens-flashback/BrainBreathes" => &[O::FlashbackBrain],
        "screens-flashback/BrainEndures" => &[O::FlashbackBrain, O::FlashbackImpact],
        "screens-flashback/BrainFlashes" => &[
            O::FlashbackBrain,
            O::FlashbackFlash,
            O::FlashbackMirror,
            O::FlashbackCgram,
        ],
        "screens-flashback/RingsAppear" => &[O::FlashbackPair, O::FlashbackRings],
        "screens-flashback/RingsSweep" => &[O::FlashbackRings, O::FlashbackPush],
        "screens-flashback/BurstsPulse" => &[O::FlashbackBursts],
        "screens-flashback/BackgroundBobs" => &[O::FlashbackY, O::FlashbackBg1Y],
        "screens-flashback/ReturnToPage" => &[O::FlashbackBrain, O::FlashbackCue, O::FlashbackHide],
        "screens-flashback/FightPictures" => &[O::FlashbackOam],
        _ => return None,
    })
}

fn playback_fields(rule: &str) -> Option<&'static [ObservationId]> {
    use ObservationId as O;
    Some(match rule {
        "screens-playback/DemoReceivesScene" => {
            &[O::PlaybackPhase, O::PlaybackProgress, O::PlaybackStart]
        }
        "screens-playback/BattleLocksOnLeft" | "screens-playback/DiscoveryReleasesSamus" => {
            &[O::PlaybackPhase, O::PlaybackFinish, O::PlaybackAuthority]
        }
        "screens-playback/DemoPlaysPad" => &[
            O::PlaybackCurrentHeld,
            O::PlaybackCurrentFresh,
            O::PlaybackPreviousHeld,
            O::PlaybackPreviousFresh,
        ],
        "screens-playback/SamusReachesEgg" => &[
            O::PlaybackPhase,
            O::PlaybackProgress,
            O::PlaybackCurrentHeld,
        ],
        _ => return None,
    })
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
            *observations == BTreeSet::from([O::CompleteReached, O::Inside])
        }
        Start::NativeOpportunity {
            sha256,
            video_frame: 9233,
            source_start_index: 424,
            source_completion_index: 4052,
            ..
        } if sha256 == super::native_search::TOUCH_PACKET_PIN => {
            *observations == BTreeSet::from([O::CompleteReached, O::Inside])
        }
        Start::NativeOpportunity { .. }
        | Start::NativeSeed { .. }
        | Start::NativeRecord { .. }
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
    use ObservationId as O;
    if case.inputs == InputProvider::PassageJumpDraws {
        return recorded
            && case.start
                == Start::CacheCut {
                    frame: super::jump_history::START,
                }
            && case.row == super::jump_history::ROW
            && case.end.cut_inclusive == super::jump_history::END
            && case.content == ContentProvider::PassageJump
            && case.expectations.fields == [O::DrawnJumpHeld, O::LastDrawJump]
            && case.expectations.outputs.is_empty()
            && case.checkpoints
            && !case.borrowed.is_empty()
            && !case.limits.is_empty();
    }
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

fn touchdown_provider_valid(case: &Case) -> bool {
    use ObservationId as O;
    case.kind == CaseKind::Scoped
        && case.row == super::sender_offer::SOURCE_ROW
        && case.content == ContentProvider::PlayerRomImport
        && case.inputs == InputProvider::NativeTouchdownInputs
        && case.expectations.provider == ExpectedProvider::NativeTouchdownObservations
        && super::movement_touchdown::WINDOWS
            .iter()
            .any(|(before, after, _)| {
                case.start == Start::CacheCut { frame: *before } && case.end.cut_inclusive == *after
            })
        && case.expectations.fields
            == [
                O::PositionX,
                O::PositionY,
                O::VerticalSpeed,
                O::TouchdownClip,
                O::TouchdownFit,
                O::TouchdownResponse,
                O::SenderQueue,
                O::SenderProgress,
            ]
        && case.expectations.outputs == [O::TouchdownOffer, O::TouchdownGraphics]
        && case.expectations.output_cut.is_none()
        && case.checkpoints
        && !case.borrowed.is_empty()
        && !case.limits.is_empty()
}

fn weapons_provider_valid(case: &Case, recorded: bool) -> bool {
    use ObservationId as O;
    if case.content != ContentProvider::PlayerRomImport
        || case.expectations.fields != [O::WeaponsState]
        || case.expectations.output_cut.is_some()
    {
        return false;
    }
    if case.inputs == InputProvider::PrecedingCutWeaponsContactInputs {
        return recorded && case.expectations.outputs == [O::ContactConsumedMarkers];
    }
    case.inputs == InputProvider::ScopedShotDrawingInputs
        && case.kind == CaseKind::Scoped
        && case.expectations.provider == ExpectedProvider::PinnedShotDrawingObservations
        && case.row == super::shot_drawing::ROW
        && matches!(
            case.start,
            Start::CacheCut {
                frame: super::shot_drawing::START
            }
        )
        && case.end.cut_inclusive == super::shot_drawing::END
        && case.expectations.outputs
            == [
                O::ContactConsumedMarkers,
                O::WeaponsUpdateEvents,
                O::ContactReceivers,
                O::WeaponsFlyingPictures,
                O::WeaponsOamInsertion,
            ]
}

fn audio_provider_valid(case: &Case, observations: &BTreeSet<ObservationId>) -> bool {
    use ObservationId as O;
    case.kind == CaseKind::Scoped
        && case.content == ContentProvider::OpeningCeresAudio
        && matches!(case.start, Start::NativeSeed { .. })
        && case.inputs == InputProvider::NativeFrameInputs
        && case.expectations.provider == ExpectedProvider::NativeDecodedCuts
        && *observations
            == BTreeSet::from([O::AudioSemantic, O::AudioReplies, O::AudioRequestLines])
        && case.expectations.output_cut.is_none()
}

fn early_map_provider_valid(case: &Case) -> bool {
    use ObservationId as O;
    case.kind == CaseKind::Scoped
        && case.id.0 == "connected-ordinary/early-mini-map"
        && case.row == "chain-any-0003-DFD7"
        && matches!(case.start, Start::CacheCut { frame: 78 })
        && case.end.cut_inclusive == 80
        && case.content == ContentProvider::PlayerRomImport
        && case.expectations.provider == ExpectedProvider::NativeEarlyMap
        && case.responsibilities
            == [
                Responsibility::Movement,
                Responsibility::Appearance,
                Responsibility::Camera,
                Responsibility::Hud,
            ]
        && case.checkpoints
        && case.expectations.fields
            == [
                O::HudExplored,
                O::EarlyMapWindow,
                O::PositionX,
                O::PositionY,
                O::MapCarryMovement,
                O::MapCarryAppearance,
                O::MapCarryCamera,
            ]
        && case.expectations.outputs.is_empty()
        && !case.borrowed.is_empty()
        && !case.limits.is_empty()
}

fn intro_provider_valid(case: &Case, recorded: bool) -> bool {
    recorded
        && case.content == ContentProvider::PlayerRomImport
        && ((case.inputs == InputProvider::ScopedIntroInputs
            && case.row == "any-intro-0568"
            && matches!(case.start, Start::CacheCut { frame: 1650 })
            && case.end.cut_inclusive == 2948)
            || (case.inputs == InputProvider::ScopedIntroDeliveryInputs
                && case.row == "any-intro-2219"
                && matches!(case.start, Start::CacheCut { frame: 1297 })
                && case.end.cut_inclusive == 3794))
        && case.expectations.fields == [ObservationId::IntroState]
        && case.expectations.outputs == [ObservationId::IntroPublications]
        && case.expectations.output_cut.is_none()
        && !case.checkpoints
}

fn flashback_provider_valid(case: &Case, recorded: bool) -> bool {
    use ObservationId as O;

    recorded
        && case.start == Start::CacheCut { frame: 0 }
        && case.row == super::flashback::ROW
        && case.end.cut_inclusive == super::flashback::END
        && case.content == ContentProvider::PlayerRomImport
        && case.inputs == InputProvider::FlashbackInputs
        && case.expectations.output_cut.is_none()
        && case.expectations.fields
            == [
                O::FlashbackStart,
                O::FlashbackY,
                O::FlashbackBrain,
                O::FlashbackPair,
                O::FlashbackRings,
                O::FlashbackBursts,
            ]
        && case.expectations.outputs
            == [
                O::FlashbackTurn,
                O::FlashbackImpact,
                O::FlashbackPush,
                O::FlashbackFlash,
                O::FlashbackCue,
                O::FlashbackHide,
                O::FlashbackMirror,
                O::FlashbackCgram,
                O::FlashbackOam,
                O::FlashbackBg1Y,
            ]
        && case.checkpoints
        && !case.borrowed.is_empty()
        && !case.limits.is_empty()
}

fn intro_missile_provider_valid(case: &Case, recorded: bool) -> bool {
    use ObservationId as O;

    recorded
        && case.start == Start::CacheCut { frame: 0 }
        && case.row == super::intro_missiles::ROW
        && case.end.cut_inclusive == super::intro_missiles::END
        && case.content == ContentProvider::PlayerRomImport
        && case.inputs == InputProvider::IntroMissileInputs
        && case.expectations.output_cut.is_none()
        && case.expectations.fields
            == [
                O::IntroMissileStock,
                O::IntroMissileSpacing,
                O::IntroMissileSlots,
                O::IntroMissileExhaust,
            ]
        && case.expectations.outputs
            == [
                O::IntroMissileRelease,
                O::IntroMissileImpact,
                O::IntroMissilePuffs,
                O::IntroMissileCandidates,
                O::IntroMissilePoints,
                O::IntroMissileSound,
                O::IntroMissileOam,
            ]
        && case.checkpoints
        && !case.borrowed.is_empty()
        && !case.limits.is_empty()
}

fn playback_provider_valid(case: &Case, recorded: bool) -> bool {
    use ObservationId as O;
    recorded
        && case.row == super::playback::ORIGIN_ROW
        && case.start
            == (Start::CacheCut {
                frame: super::playback::ORIGIN,
            })
        && case.end.cut_inclusive == super::playback::END
        && case.content == ContentProvider::PlayerRomImport
        && case.inputs == InputProvider::CinematicActorOpportunities
        && case.expectations.fields
            == [
                O::PlaybackPhase,
                O::PlaybackProgress,
                O::PlaybackLastHeld,
                O::PlaybackLastFresh,
            ]
        && case.expectations.outputs
            == [
                O::PlaybackStart,
                O::PlaybackCurrentHeld,
                O::PlaybackCurrentFresh,
                O::PlaybackPreviousHeld,
                O::PlaybackPreviousFresh,
                O::PlaybackFinish,
                O::PlaybackAuthority,
            ]
        && case.expectations.output_cut.is_none()
        && case.checkpoints
}

fn save_files_provider_valid(case: &Case) -> bool {
    case.kind == CaseKind::Scoped
        && case.row == "any-power-on"
        && matches!(case.start, Start::CacheCut { frame: 0 })
        && case.end.cut_inclusive == super::save_files::END
        && case.responsibilities == [Responsibility::SaveFiles]
        && case.content == ContentProvider::PlayerRomImport
        && case.expectations.provider == ExpectedProvider::PinnedRecordCache
        && case.expectations.fields == [ObservationId::Battery]
        && case.expectations.outputs.is_empty()
        && case.checkpoints
}

fn unavailable_provider_valid(case: &Case, observations: &BTreeSet<ObservationId>) -> bool {
    case.kind == CaseKind::Scoped
        && matches!(
            case.responsibilities.as_slice(),
            [Responsibility::Blocks
                | Responsibility::Doors
                | Responsibility::AudioSender
                | Responsibility::Appearance]
        )
        && case.rules.is_empty()
        && observations.is_empty()
        && !case.checkpoints
        && !case.limits.is_empty()
}

fn provider_valid(case: &Case, observations: &BTreeSet<ObservationId>) -> bool {
    use ObservationId as O;
    if case.inputs == InputProvider::PlayedCeresInputs {
        return super::played::indexed::valid(case);
    }
    if case.inputs == InputProvider::IntroRoomJoinInputs {
        return super::intro_rooms::valid(case);
    }
    if case.inputs == InputProvider::SaveFileBoundaries {
        return save_files_provider_valid(case);
    }
    if case.inputs == InputProvider::EarlyMapInputs {
        return early_map_provider_valid(case);
    }
    if case.inputs == InputProvider::Unavailable {
        return unavailable_provider_valid(case, observations);
    }
    let recorded = case.kind == CaseKind::Scoped
        && case.expectations.provider == ExpectedProvider::PinnedRecordCache
        && matches!(case.start, Start::CacheCut { .. });
    match case.responsibilities.as_slice() {
        [
            Responsibility::Health,
            Responsibility::Appearance,
            Responsibility::Hud,
        ] => health_tail_provider_valid(case),
        [
            Responsibility::Movement,
            Responsibility::Appearance,
            Responsibility::Camera,
        ] => ordinary_provider_valid(case, observations),
        [
            Responsibility::Movement,
            Responsibility::Appearance,
            Responsibility::AudioSender,
        ] => touchdown_provider_valid(case),
        [Responsibility::Intro] => intro_provider_valid(case, recorded),
        [Responsibility::Flashback] => flashback_provider_valid(case, recorded),
        [Responsibility::Weapons] => intro_missile_provider_valid(case, recorded),
        [Responsibility::Movement] => movement_provider_valid(case, observations, recorded),
        [Responsibility::Audio] => audio_provider_valid(case, observations),
        [Responsibility::Playback] => playback_provider_valid(case, recorded),
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
            weapons_provider_valid(case, recorded)
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

fn health_tail_provider_valid(case: &Case) -> bool {
    use ObservationId as O;

    case.kind == CaseKind::Scoped
        && case.row == super::health_tail::ROW
        && matches!(
            case.start,
            Start::NativeRecord {
                frame: super::health_tail::ORIGIN,
                ..
            }
        )
        && case.end.cut_inclusive == super::health_tail::CUTS[1]
        && case.inputs == InputProvider::NativeHealthTailInputs
        && case.expectations.provider == ExpectedProvider::NativeHealthTailObservations
        && case.content == ContentProvider::HudReference
        && !case.borrowed.is_empty()
        && !case.limits.is_empty()
        && case.checkpoints
        && case.expectations.output_cut.is_none()
        && case.expectations.fields == [O::HealthTailState, O::DrawingEnergy, O::GaugeRefresh]
        && case.expectations.outputs
            == [
                O::HealthTailWarning,
                O::HealthTailDrawnProtection,
                O::GaugeWords,
            ]
}

fn scope_valid(case: &Case) -> bool {
    let owners: BTreeSet<_> = case.responsibilities.iter().copied().collect();
    if owners.len() != case.responsibilities.len()
        || (case.actor_slot.is_some() && case.responsibilities != [Responsibility::Ceres])
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
    for rule in &case.rules {
        if rule_owner(rule).is_none_or(|owner| !owners.contains(&owner)) {
            return false;
        }
        if witnessed_fields(rule, case.inputs)
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

fn start_valid(case: &Case) -> bool {
    match &case.start {
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
        Start::NativeRecord {
            path,
            sha256,
            frame,
        } => {
            case.inputs == InputProvider::NativeHealthTailInputs
                && !path.as_os_str().is_empty()
                && valid_hash(sha256)
                && *frame < case.end.cut_inclusive
        }
        Start::NativeSeed { path, sha256 } => !path.as_os_str().is_empty() && valid_hash(sha256),
        Start::NativeSenderOffer { source_id } => {
            *source_id < super::sender_offer::NATIVE_WINDOWS
                && case.row == super::sender_offer::SOURCE_ROW
        }
        Start::ConstructedSenderOffer { .. } => {
            case.row == "constructed-sender-queue" && case.end.cut_inclusive == 0
        }
        Start::PowerOn => case.kind == CaseKind::Movie && case.borrowed.is_empty(),
    }
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
        if case.id.0.is_empty()
            || case.row.is_empty()
            || (case.rules.is_empty() && case.inputs != InputProvider::Unavailable)
            || case.profile != "reference"
            || case.responsibilities.is_empty()
            || (observations.is_empty() && case.inputs != InputProvider::Unavailable)
            || unique.len() != observations.len()
            || !start_valid(case)
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
        if matches!(self.inputs, InputProvider::FlashbackInputs) {
            // The lookup's anchor is the pending-start cut. Setup itself is an opportunity
            // driven by the predecessor's press.
            return Some(0);
        }
        if matches!(self.inputs, InputProvider::IntroMissileInputs) {
            // Compare the admitted setup before the first actor turn.
            return Some(0);
        }
        match self.start {
            Start::CacheCut { frame } | Start::NativeRecord { frame, .. } => frame.checked_add(1),
            Start::NativeOpportunity { video_frame, .. } => Some(video_frame),
            Start::NativeSenderOffer { .. } | Start::ConstructedSenderOffer { .. } => {
                Some(self.end.cut_inclusive)
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
            index.select(&[], Some("audio-transport/EffectResources"), None),
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
