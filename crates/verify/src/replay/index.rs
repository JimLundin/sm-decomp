//! Strict case metadata. Cases name witnesses; rule identifiers resolve to clean catalogues.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub use super::native_packets::NativePacket;
use super::played::witnesses::Witnesses;
pub use super::save_files::SaveSchedule;

pub(super) const CURRENT_FORMAT: u16 = 2;

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
    /// The station scene after Ceres.
    StationExplosion,
    /// The shaft blackout.
    Blackout,
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
    /// The egg and baby in the intro.
    Discovery,
    /// The first Landing Site work interval and its direct DMA.
    LandingSite,
    /// The complete flight to Zebes.
    Zebes,
    /// Page, sprite and palette consumers selected alongside the flight.
    ZebesDrawing,
    /// The scheduler's published landing request and retained destination.
    ZebesScheduler,
    /// Static entry setup for the built room.
    RoomLoading,
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
    /// Retained shaft actors and stationary effects, including their picture.
    ShaftPicture,
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
    /// An uninterrupted production continuation played from an earlier recorded origin.
    PlayedContinuation {
        /// The only record admitted as initial state.
        seed: u64,
        /// The carried cut before the indexed comparisons begin.
        frame: u64,
    },
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
        #[serde(deserialize_with = "super::legacy_packets::sender_control")]
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

/// A constructed offer window over one sender queue before its next service.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SenderOfferControl {
    pub(super) group: audio_transport::SoundGroup,
    pub(super) seed: audio_transport::CueQueue,
    pub(super) cues: Vec<audio_transport::CueId>,
    pub(super) allowance: audio_transport::QueueAllowance,
    #[serde(with = "Permission")]
    pub(super) permission: audio_transport::OfferPermission,
    pub(super) expected: audio_transport::CueQueue,
}

#[derive(Deserialize, Serialize)]
#[serde(remote = "audio_transport::OfferPermission")]
pub(super) enum Permission {
    Enabled,
    Disabled,
    Demo,
    Explosion,
}

impl SenderOfferControl {
    fn valid(&self) -> bool {
        !self.cues.is_empty()
            && self
                .cues
                .iter()
                .chain(self.seed.cues())
                .chain(self.expected.cues())
                .all(|cue| cue.group() == self.group)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// The final declared comparison cut.
pub struct End {
    /// Last compared original cut.
    pub cut_inclusive: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Independent source of changing causal stimuli.
pub enum InputProvider {
    /// The scheduled NMI completion and post-service music reply.
    StationExplosion,
    /// Movie pad, room seed and the played platform boundaries for a Ceres tail window.
    PlayedCeresInputs,
    /// Ordinary fight draws at finished movie cuts through the played room.
    PlayedSamusPicture,
    /// Movie pad and platform opportunities for the return shaft's wall-jump dust.
    PlayedShaftDust,
    /// One elevator arrival and T1 scheduling.
    Blackout,
    /// One admitted room and finished opportunities for the flyby’s owned producers.
    ScopedCeresFlyby,
    /// One admitted warning-ready room and the usual played room/passage inputs.
    ScopedCeresWarning,
    /// The carried Ceres return through placement and elevator reach.
    CeresShaftActors,
    /// Played shaft actors with native CPU-side Ambient reply reads.
    PlayedShaftSound,
    /// One admitted ordinary fight and the played platform publication boundaries.
    ScopedCeresFightPicture,
    /// The movie pad, return-hall origin and the played platform boundaries.
    ShaftPictures,
    /// Carried Samus turn with a separately observed flyby pass and view edge.
    ScopedGetawayInputs,
    /// The movie pad, platform completion and the Zebes encounter cue for scoped intro play.
    ScopedIntroInputs,
    /// A causal intro start and the borrowed work-completion boundaries.
    #[serde(rename = "intro-room-join")]
    IntroRoomJoinInputs,
    /// Published witness with implicit recorded successors.
    #[serde(rename = "intro-room-join-inputs")]
    PublishedIntroRoomJoinInputs,
    /// Locked encounter phase, after-motion room facts and presentation opportunities.
    #[serde(rename = "flashback-segments")]
    FlashbackInputs,
    /// Published encounter witness with implicit source coordinates.
    #[serde(rename = "flashback-inputs")]
    PublishedFlashbackInputs,
    /// Recorded scene setup and reached room actor opportunities.
    #[serde(rename = "cinematic-actor-segments")]
    CinematicActorOpportunities,
    /// Published witness with implicit recorded successors.
    #[serde(rename = "cinematic-actor-opportunities")]
    PublishedCinematicActorOpportunities,
    /// Ordered startup and save boundaries from pinned preceding cuts.
    #[serde(rename = "save-file-schedule")]
    SaveFileBoundaries,
    /// The published format-1 spelling for the original save witness.
    #[serde(rename = "save-file-boundaries")]
    PublishedSaveFileBoundaries,
    /// Authored demo controls and borrowed cinematic weapon opportunities.
    IntroMissileInputs,
    /// Passage contents and block-script opportunities.
    ShaftSetupTurns,
    /// Independent passage phase and sampled button at each draw opportunity.
    PassageJumpDraws,
    /// Scoped discovery and delivery inputs, including the interrupted start.
    ScopedIntroDeliveryInputs,
    /// The intro requests, completion and resulting actor position in the discovery room.
    #[serde(rename = "discovery-segments")]
    DiscoveryInputs,
    /// Published encounter witness with implicit source coordinates.
    #[serde(rename = "discovery-inputs")]
    PublishedDiscoveryInputs,
    /// Station, flight and direct transfer work from one elevator origin.
    Departure,
    /// Receiving T1 work opening and byte progress.
    LandingSiteTiming,
    /// Recorded work completion after the station handoff.
    ZebesFlightInputs,
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
    /// Pinned oracle records qualified by the case's independent ordinary witness.
    WitnessedRecordCache,
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
    /// The station scene's stage observations.
    #[serde(rename = "explosion.stage")]
    ExplosionStage,
    /// The station scene's motion observations.
    #[serde(rename = "explosion.motion")]
    ExplosionMotion,
    /// The station scene's bursts observations.
    #[serde(rename = "explosion.bursts")]
    ExplosionBursts,
    /// The station scene's carry observations.
    #[serde(rename = "explosion.carry")]
    ExplosionCarry,
    /// The station scene's requests observations.
    #[serde(rename = "explosion.requests")]
    ExplosionRequests,
    /// The station scene's drawing observations.
    #[serde(rename = "explosion.drawing")]
    ExplosionDrawing,
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
    /// Phase, position, turn, picture counter and both beats.
    #[serde(rename = "ridley.flyby.state")]
    CeresFlybyState,
    /// Effective plane and physical transform, scroll, mode, and zoom.
    #[serde(rename = "ridley.flyby.plane")]
    CeresFlybyPlane,
    /// Current body shades and their published CGRAM span.
    #[serde(rename = "ridley.flyby.colours")]
    CeresFlybyColours,
    /// The low-byte tilemap updates for the baby and the wings.
    #[serde(rename = "ridley.flyby.tiles")]
    CeresFlybyTiles,
    /// The flyby sound offer on the second effects channel.
    #[serde(rename = "ridley.flyby.sound")]
    CeresFlybySound,
    /// The side-free request from Ridley to movement.
    #[serde(rename = "ridley.flyby.push")]
    CeresFlybyPush,
    /// The room’s door state, clock and quake at each flyby cut.
    #[serde(rename = "ridley.flyby.room")]
    CeresFlybyRoom,
    /// Warning phase, live progress, glow and Escape stage.
    #[serde(rename = "ridley.warning.state")]
    CeresWarningState,
    /// Local clocks keyed by shaft actor placement.
    #[serde(rename = "shaft-picture.clocks")]
    ShaftPictureClocks,
    /// Stable effect slots and their playback phases.
    #[serde(rename = "shaft-picture.effects")]
    ShaftPictureEffects,
    /// Selected native shadow and published OAM spans.
    #[serde(rename = "shaft-picture.oam")]
    ShaftPictureOam,
    /// Carried room art, positioned text and live and physically published colours.
    #[serde(rename = "ridley.warning.picture")]
    CeresWarningPicture,
    /// Delayed escape track and stroke offers.
    #[serde(rename = "ridley.warning.audio")]
    CeresWarningAudio,
    /// Defeat, doors, quake results and the clock picture through the first passage turns.
    #[serde(rename = "ridley.warning.handoff")]
    CeresWarningHandoff,
    /// Overlay phase, ordered mounted offsets, shaft rotation, shared RNG and lock.
    #[serde(rename = "shaft-actors.state")]
    CeresShaftState,
    /// Placement, burst, sound limit, lighting and elevator reach requests.
    #[serde(rename = "shaft-actors.events")]
    CeresShaftEvents,
    /// Overlay VRAM, first prepared and published OAM, and platform low bytes.
    #[serde(rename = "shaft-actors.picture")]
    CeresShaftPicture,
    /// The next ordinary ribs beat.
    #[serde(rename = "ridley.ribs.beat")]
    CeresRibsBeat,
    /// The eligible work runs left before the next reveal.
    #[serde(rename = "ridley.ribs.wait")]
    CeresRibsWait,
    /// The ordered pair of regular VRAM transfers serviced for a reveal.
    #[serde(rename = "ridley.ribs.uploads")]
    CeresRibsUploads,
    /// Every physical word of the upper ribs strip.
    #[serde(rename = "ridley.ribs.vram.upper")]
    CeresRibsUpper,
    /// Every physical word of the lower ribs strip.
    #[serde(rename = "ridley.ribs.vram.lower")]
    CeresRibsLower,
    /// The actual slot at which the attached door is appended.
    #[serde(rename = "ridley.ordinary.door-cursor")]
    CeresFightDoorCursor,
    /// The prepared ordinary fight picture.
    #[serde(rename = "ridley.ordinary.prepared-oam")]
    CeresFightPreparedOam,
    /// The physically published ordinary fight picture.
    #[serde(rename = "ridley.ordinary.physical-oam")]
    CeresFightPhysicalOam,
    /// The wrapping HUD frame count.
    #[serde(rename = "hud.frame-count")]
    HudFrameCount,
    /// The regular queue's descriptor and source words at an NMI cut.
    #[serde(rename = "hud.pending-transfer")]
    HudPendingTransfer,
    /// The descriptor and source words the NMI services.
    #[serde(rename = "hud.publication")]
    HudPublication,
    /// The full HUD window in physical VRAM.
    #[serde(rename = "hud.published-grid")]
    HudPublishedGrid,
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
    #[doc = "Blackout scene observation."]
    #[serde(rename = "blackout.scene")]
    BlackoutScene,
    #[doc = "Blackout room observation."]
    #[serde(rename = "blackout.room")]
    BlackoutRoom,
    #[doc = "Blackout battery observation."]
    #[serde(rename = "blackout.battery")]
    BlackoutBattery,
    #[doc = "Blackout timer-drawing observation."]
    #[serde(rename = "blackout.timer-drawing")]
    BlackoutTimerDrawing,
    #[doc = "Blackout requests observation."]
    #[serde(rename = "blackout.requests")]
    BlackoutRequests,
    #[doc = "Blackout memory observation."]
    #[serde(rename = "blackout.memory")]
    BlackoutMemory,
    /// Standing body's pending OAM slots.
    #[serde(rename = "standing-body.pending-oam")]
    StandingOamPending,
    /// Standing body's pending named strip transfers.
    #[serde(rename = "standing-body.pending-strips")]
    StandingStripsPending,
    /// Standing body's physical OAM slots.
    #[serde(rename = "standing-body.published-oam")]
    StandingOamPublished,
    /// Standing body's strip transfers serviced by video.
    #[serde(rename = "standing-body.publication")]
    StandingPublication,
    /// Physical first upper-body strip row.
    #[serde(rename = "standing-body.upper-first")]
    StandingUpperFirst,
    /// Physical second upper-body strip row.
    #[serde(rename = "standing-body.upper-second")]
    StandingUpperSecond,
    /// Physical first lower-body strip row.
    #[serde(rename = "standing-body.lower-first")]
    StandingLowerFirst,
    /// Physical second lower-body strip row.
    #[serde(rename = "standing-body.lower-second")]
    StandingLowerSecond,
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
    /// Carried wall-jump dust centre, look and clock.
    #[serde(rename = "shaft-dust.state")]
    ShaftDustState,
    /// Prepared wall-jump dust OAM slot and entry.
    #[serde(rename = "shaft-dust.prepared")]
    ShaftDustPrepared,
    /// Published wall-jump dust OAM slot and entry.
    #[serde(rename = "shaft-dust.published")]
    ShaftDustPublished,
    /// The native vent entry written after dust and body at the affected cut.
    #[serde(rename = "shaft-dust.following-vent")]
    ShaftDustFollowingVent,
    /// Shaft setup view.
    #[serde(rename = "shaft-entry.view")]
    ShaftView,
    /// Shaft setup affine-registers.
    #[serde(rename = "shaft-entry.affine-registers")]
    ShaftTransform,
    /// Shaft setup rotation.
    #[serde(rename = "shaft-entry.rotation")]
    ShaftRotation,
    /// Shaft setup cells.
    #[serde(rename = "shaft-entry.cells")]
    ShaftLevel,
    /// Shaft setup placement.
    #[serde(rename = "shaft-entry.placement")]
    ShaftPlacement,
    /// Shaft setup haze.
    #[serde(rename = "shaft-entry.haze")]
    ShaftHaze,

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
    /// Getaway pending or active on a side, or the ordinary movement phase.
    #[serde(rename = "movement.getaway-phase")]
    GetawayPhase,
    /// Latest resolved pose recorded by Movement.
    #[serde(rename = "movement.getaway-latest-pose")]
    GetawayLatestPose,
    /// Pose recorded before the latest resolved choice.
    #[serde(rename = "movement.getaway-older-pose")]
    GetawayOlderPose,
    /// Horizontal half-width used at the beginning of the shove opportunity.
    #[serde(rename = "movement.getaway-body-x")]
    GetawayBodyX,
    /// Vertical half-height used at the beginning of the shove opportunity.
    #[serde(rename = "movement.getaway-body-y")]
    GetawayBodyY,
    /// The grounded upright floor response.
    #[serde(rename = "movement.landing.floor")]
    LandingFloor,
    /// Native shell phase and hatch age.
    #[serde(rename = "discovery.egg")]
    DiscoveryEgg,
    /// The baby’s position and current motion memory.
    #[serde(rename = "discovery.baby")]
    DiscoveryBaby,
    /// Current pulse age, including an unseen first frame.
    #[serde(rename = "discovery.pulse")]
    DiscoveryPulse,
    /// All live shell fragments.
    #[serde(rename = "discovery.fragments")]
    DiscoveryFragments,
    /// All live droplets, including splashes.
    #[serde(rename = "discovery.droplets")]
    DiscoveryDroplets,
    /// The scene’s relative foreground order.
    #[serde(rename = "discovery.order")]
    DiscoveryOrder,
    /// Ordered shell and baby sound offers.
    #[serde(rename = "discovery.sounds")]
    DiscoverySounds,
    /// The page-three cue.
    #[serde(rename = "discovery.page-three")]
    DiscoveryCue,
    /// Whether the playable room got its last turn.
    #[serde(rename = "discovery.room-turn")]
    DiscoveryTurn,
    /// Visible discovery pieces and priority relative to the room.
    #[serde(rename = "discovery.oam")]
    DiscoveryOam,
    /// The scheduled scene, finished count, capacity, caption and destination.
    #[serde(rename = "departure.schedule")]
    DepartureSchedule,
    /// Complete health across the station blackout and flight refill.
    #[serde(rename = "departure.health")]
    DepartureHealth,
    /// The once-per-work RNG across interrupted scene and room setup.
    #[serde(rename = "departure.random")]
    DepartureRandom,
    /// The room, active and stored maps, tilemaps and whole run progress.
    #[serde(rename = "departure.room")]
    DepartureRoom,
    /// Every byte of settled VRAM, OAM and CGRAM, and the affine transform.
    #[serde(rename = "departure.picture")]
    DeparturePicture,
    /// Live and published registers and the accepted and committed loader transfer.
    #[serde(rename = "departure.loader")]
    DepartureLoader,
    /// Receiving loader random.
    #[serde(rename = "landing.random")]
    LandingRandom,
    /// Receiving loader carried.
    #[serde(rename = "landing.carried")]
    LandingCarried,
    /// Receiving loader map.
    #[serde(rename = "landing.map")]
    LandingMap,
    /// Receiving loader pipeline.
    #[serde(rename = "landing.pipeline")]
    LandingPipeline,
    /// Receiving loader vram.
    #[serde(rename = "landing.vram")]
    LandingVram,
    /// Receiving loader cgram.
    #[serde(rename = "landing.cgram")]
    LandingCgram,
    /// Receiving loader oam.
    #[serde(rename = "landing.oam")]
    LandingOam,
    /// Receiving loader display.live.
    #[serde(rename = "landing.display.live")]
    LandingLive,
    /// Receiving loader display.published.
    #[serde(rename = "landing.display.published")]
    LandingPublished,
    /// Receiving loader display.mirrors.
    #[serde(rename = "landing.display.mirrors")]
    LandingMirrors,
    /// Receiving loader dma.
    #[serde(rename = "landing.dma")]
    LandingDma,
    /// Receiving loader tilemaps.
    #[serde(rename = "landing.tilemaps")]
    LandingTilemaps,

    /// The flight stage and stage-local counters.
    #[serde(rename = "zebes.stage")]
    ZebesStage,
    /// Brightness, view, mosaic, pose and affine output.
    #[serde(rename = "zebes.picture")]
    ZebesPicture,
    /// The presence and motion of each placed picture.
    #[serde(rename = "zebes.motion")]
    ZebesMotion,
    /// Requests the flight publishes on this cut.
    #[serde(rename = "zebes.requests")]
    ZebesRequests,
    /// The energy held and restored by the receiving owner.
    #[serde(rename = "zebes.health")]
    ZebesHealth,
    /// Current game flow and retained Landing Site context.
    #[serde(rename = "zebes.schedule")]
    ZebesSchedule,
    /// Exact page and graphics transfers.
    #[serde(rename = "zebes.pages")]
    ZebesPages,
    /// Ordered visible sprite entries and high-table bits.
    #[serde(rename = "zebes.oam")]
    ZebesOam,
    /// Complete published BGR555 palette, including the engine accent and caption.
    #[serde(rename = "zebes.palette")]
    ZebesPalette,
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
    /// Room sound enable carried through passage.
    #[serde(rename = "sender.permission")]
    SenderPermission,
    /// The CPU's current request to the Ambient sound group.
    #[serde(rename = "sender.request")]
    SenderRequest,
    /// The shaft overlay's sound offer and its queue limit.
    #[serde(rename = "shaft-sound.offer")]
    ShaftSoundOffer,
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
    /// Whether the ordinary draw submitted body placements.
    #[serde(rename = "samus-oam.visible")]
    OrdinaryBodyVisible,
    /// Number of placements contributed by the ordinary body.
    #[serde(rename = "samus-oam.piece-count")]
    OrdinaryBodyCount,
    /// The body's entries in the prepared OAM batch.
    #[serde(rename = "samus-oam.prepared")]
    OrdinaryBodyPrepared,
    /// The body's entries in physical OAM after NMI.
    #[serde(rename = "samus-oam.physical")]
    OrdinaryBodyPhysical,
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

/// Authenticated source of independent, post-execution observations.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeProjection {
    /// Directory containing the projection's manifest.
    pub directory: String,
    /// SHA-256 of the complete member manifest.
    pub sha256: String,
    /// Reviewed member containing the expected observations.
    pub member: String,
}

pub(super) fn fixture_component(name: &str) -> bool {
    Path::new(name)
        .file_name()
        .is_some_and(|part| part == std::ffi::OsStr::new(name))
}

pub(super) fn pinned_directory(directory: &str, sha256: &str) -> bool {
    fixture_component(directory) && valid_hash(sha256)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Declared observations and their comparison schedule.
pub struct Expectations {
    /// Independent capture or import source.
    pub provider: ExpectedProvider,
    /// Packet and member used by the native movement-work comparison.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projection: Option<NativeProjection>,
    /// Native capture coordinates used by a played comparison.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) played: Option<PlayedWindow>,
    /// Compared semantic field identities or quantities.
    pub fields: Vec<ObservationId>,
    /// Compared output identities or occurrences.
    pub outputs: Vec<ObservationId>,
    #[serde(default)]
    /// Outputs compare only at this cut, or every cut when absent.
    pub output_cut: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(super) enum PlayedWindow {
    FightPicture { appends: [DoorAppend; 2] },
    RibsReveal { cut: u64 },
    ShaftActors { first: u64, picture_fields: usize },
    Flyby { first: u64, output_fields: usize },
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DoorAppend {
    pub cut: u64,
    pub cursor: u8,
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
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub claim: String,
}

/// A scope or provenance explanation carried in the replay map.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Qualification {
    /// Free-form explanation from the case author.
    Text(String),
    /// Source description from a published index.
    Published(Borrowed),
}

/// Authenticated input packet and its reviewed comparison interval.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConnectedPacket {
    /// Directory beneath the supplied native fixtures.
    pub directory: String,
    /// SHA-256 of the complete packet manifest.
    pub sha256: String,
    /// Oracle row covered by this packet.
    pub row: String,
    /// Cache cut at which the production owners are seeded.
    pub initial_cut: u64,
    /// Earliest reviewed comparison horizon.
    pub first_end: u64,
    /// Last cut supplied by the packet.
    pub last_cut: u64,
}

impl ConnectedPacket {
    fn admits(&self, case: &Case) -> bool {
        pinned_directory(&self.directory, &self.sha256)
            && self.row == case.row
            && case.start
                == (Start::CacheCut {
                    frame: self.initial_cut,
                })
            && self.initial_cut < self.first_end
            && (self.first_end..=self.last_cut).contains(&case.end.cut_inclusive)
    }
}

/// One authenticated TOML file under the local native fixture root.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureFile {
    /// Relative directory and member name, separated by a slash.
    pub path: String,
    /// SHA-256 of the file bytes.
    pub sha256: String,
}

impl FixtureFile {
    pub(super) fn valid(&self) -> bool {
        let component = |part: &str| fixture_component(part) && !part.contains(['\\', ':', '\0']);
        self.path
            .split_once('/')
            .is_some_and(|(directory, name)| component(directory) && component(name))
            && valid_hash(&self.sha256)
    }

    pub(super) fn read<T: serde::de::DeserializeOwned>(
        &self,
        root: &Path,
    ) -> Result<T, super::result::Outcome> {
        let failure = |reason| super::data_failure(None, &format!("{} {reason}", self.path));
        if !self.valid() {
            return Err(failure("fixture reference"));
        }
        let text =
            std::fs::read_to_string(root.join(&self.path)).map_err(|_| failure("fixture file"))?;
        if crate::lock::Sha256::of(text.as_bytes()).to_string() != self.sha256 {
            return Err(failure("fixture pin"));
        }
        toml::from_str(&text).map_err(|_| failure("fixture schema"))
    }
}

/// Input and independent observation files for a scoped provider.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FixturePair {
    /// Provider whose typed readers consume these files.
    pub provider: InputProvider,
    /// Capture repository revision recorded by early Map input fixtures.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    door_slots: Option<Vec<u8>>,
    /// File read for admission.
    pub inputs: FixtureFile,
    /// File read by the provider's comparison reader.
    pub expected: FixtureFile,
}

impl FixturePair {
    pub(super) fn repository(&self) -> Option<&str> {
        self.repository.as_deref().or_else(|| {
            super::legacy_packets::fixture(self.provider)?
                .repository
                .as_deref()
        })
    }

    pub(super) fn door_slots(&self) -> Option<&[u8]> {
        self.door_slots.as_deref().or_else(|| {
            super::legacy_packets::fixture(self.provider)?
                .door_slots
                .as_deref()
        })
    }

    fn valid(&self) -> bool {
        let revision_valid = if self.provider == InputProvider::EarlyMapInputs {
            self.repository()
                .is_some_and(|revision| valid_hex(revision, 40))
        } else {
            self.repository.is_none()
        };
        let slots_valid = if self.provider == InputProvider::ScopedShotDrawingInputs {
            self.door_slots()
                .is_some_and(super::shot_drawing::valid_slots)
        } else {
            self.door_slots.is_none()
        };
        self.provider.uses_fixture_pair()
            && slots_valid
            && revision_valid
            && self.inputs.path != self.expected.path
            && self.inputs.sha256 != self.expected.sha256
            && self.inputs.valid()
            && self.expected.valid()
    }
}

impl InputProvider {
    const fn uses_played_witnesses(self) -> bool {
        matches!(
            self,
            Self::ShaftSetupTurns
                | Self::PlayedSamusPicture
                | Self::PlayedCeresInputs
                | Self::ScopedCeresFlyby
                | Self::ScopedCeresFightPicture
                | Self::ScopedCeresWarning
                | Self::CeresShaftActors
                | Self::PlayedShaftSound
                | Self::ShaftPictures
                | Self::PlayedShaftDust
        )
    }

    const fn uses_fixture_pair(self) -> bool {
        matches!(
            self,
            Self::EarlyMapInputs | Self::NativeTouchdownInputs | Self::ScopedShotDrawingInputs
        )
    }
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
    /// Ordered requests with their independent battery observation coordinates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub save_steps: Option<SaveSchedule>,
    /// Locked record segments for the intro room and playback providers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) intro_join: Option<super::intro_sources::Join>,
    /// Reviewed fixture for connected ordinary inputs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connected_packet: Option<ConnectedPacket>,
    /// Input and observation files selected for this provider.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixture: Option<FixturePair>,
    /// Authenticated native capture consumed by this case.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_packet: Option<NativePacket>,
    /// Native trace coordinates for played comparisons.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) played_witnesses: Option<Witnesses>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) ordinary_witness: Option<super::recorded::Witness>,
    /// Declared independent comparison observations.
    pub expectations: Expectations,
    /// Explicit unproved dependencies and omitted outputs.
    pub limits: Vec<String>,
    /// Scope and provenance explanations accompanying this case.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub qualifications: Vec<Qualification>,
    #[serde(default, deserialize_with = "super::borrowed::read")]
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
#[serde(try_from = "IndexDocument")]
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

/// Shared fixtures are expanded so a selected case carries its entire execution recipe.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct IndexDocument {
    format: u16,
    engine_api: String,
    observation_schema: String,
    lock: PathBuf,
    #[serde(default)]
    fixture: Vec<FixturePair>,
    #[serde(default)]
    played_witnesses: Option<Witnesses>,
    #[serde(default)]
    native_packet: Option<Vec<NativePacket>>,
    #[serde(rename = "case")]
    cases: Vec<Case>,
}

impl TryFrom<IndexDocument> for Index {
    type Error = &'static str;

    fn try_from(mut doc: IndexDocument) -> Result<Self, Self::Error> {
        let mut by_provider = std::collections::BTreeMap::new();
        for pair in doc.fixture {
            if !pair.valid() || by_provider.insert(pair.provider, pair).is_some() {
                return Err("invalid or duplicate provider fixture");
            }
        }
        // Published format-1 documents predate both native packets and played windows.
        let legacy = doc.format == 1
            && doc.native_packet.is_none()
            && doc
                .cases
                .iter()
                .all(|case| case.native_packet.is_none() && case.expectations.played.is_none());
        let declarations = doc.native_packet.unwrap_or_default();
        let mut kinds = std::collections::HashSet::new();
        for packet in &declarations {
            if !packet.valid() || !kinds.insert(std::mem::discriminant(packet)) {
                return Err("invalid or duplicate native packet");
            }
        }
        let shared_witnesses = doc
            .played_witnesses
            .map(|data| Witnesses::resolve(doc.format, Some(data)))
            .transpose()?;
        for case in &mut doc.cases {
            if case.inputs == InputProvider::PublishedSaveFileBoundaries {
                if case.save_steps.is_some() {
                    return Err("published save witness already has a schedule");
                }
                case.save_steps = super::legacy_packets::save_schedule(&case.id.0).cloned();
                case.inputs = InputProvider::SaveFileBoundaries;
            }
            super::legacy_packets::read_intro_join(case)?;
            if case.inputs.uses_played_witnesses() {
                case.played_witnesses =
                    Some(match (case.played_witnesses.take(), &shared_witnesses) {
                        (None, Some(shared)) => shared.clone(),
                        (declared, _) => Witnesses::resolve(doc.format, declared)?,
                    });
            }
            if case.native_packet.is_none() {
                let shared = declarations.iter().find(|packet| packet.applies_to(case));
                let published = super::legacy_packets::native(case);
                if !legacy && shared.is_none() && published.is_some() {
                    return Err("missing native provider packet");
                }
                case.native_packet = shared.or(if legacy { published } else { None }).cloned();
            }
            if legacy && case.expectations.played.is_none() {
                case.expectations.played = super::legacy_packets::played(&case.id.0).cloned();
            }
            if case.fixture.is_none() {
                case.fixture = by_provider.get(&case.inputs).cloned();
            }
        }
        Ok(Self {
            format: doc.format,
            engine_api: doc.engine_api,
            observation_schema: doc.observation_schema,
            lock: doc.lock,
            cases: doc.cases,
        })
    }
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
    if let Some(name) = rule.strip_prefix("screens-explosion/") {
        let rules = enum_map::EnumMap::<screens_explosion::catalogue::BehaviourId, ()>::default();
        return rules.iter().any(|(id, ())| format!("{id:?}") == name);
    }
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
        || screens_blackout::catalogue::BehaviourId::ALL
            .iter()
            .any(|id| rule == format!("screens-blackout/{id:?}"))
        || screens_discovery::catalogue::catalogue()
            .iter()
            .any(|(id, _)| rule == format!("screens-discovery/{id:?}"))
        || screens_zebes::BehaviourId::ALL
            .iter()
            .any(|id| rule == format!("screens-zebes/{id:?}"))
        || game::production::shaft_entry::CATALOGUE.contains(&rule)
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
            enemy_interactions::BehaviourId::ShaftActorDraw,
            enemy_interactions::BehaviourId::ActorShake,
            enemy_interactions::BehaviourId::EffectSlots,
            enemy_interactions::BehaviourId::EffectClip,
            enemy_interactions::BehaviourId::PictureWindow,
            enemy_interactions::BehaviourId::ShaftPictureOrder,
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

macro_rules! explosion_observations {
    () => {
        O::ExplosionStage
            | O::ExplosionMotion
            | O::ExplosionBursts
            | O::ExplosionCarry
            | O::ExplosionRequests
            | O::ExplosionDrawing
    };
}

macro_rules! blackout_observations {
    () => {
        O::BlackoutScene
            | O::BlackoutRoom
            | O::BlackoutBattery
            | O::BlackoutRequests
            | O::BlackoutMemory
            | O::BlackoutTimerDrawing
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

macro_rules! discovery_observations {
    () => {
        ObservationId::DiscoveryEgg
            | ObservationId::DiscoveryBaby
            | ObservationId::DiscoveryPulse
            | ObservationId::DiscoveryFragments
            | ObservationId::DiscoveryDroplets
            | ObservationId::DiscoveryOrder
            | ObservationId::DiscoverySounds
            | ObservationId::DiscoveryCue
            | ObservationId::DiscoveryTurn
            | ObservationId::DiscoveryOam
    };
}

macro_rules! ceres_observations {
    () => {
        ObservationId::CeresPhase
            | ObservationId::CeresWait
            | ObservationId::CeresFigure
            | ObservationId::CeresShown
            | ObservationId::CeresSolid
            | ObservationId::CeresRetreatBody
            | ObservationId::CeresRetreatBound
            | ObservationId::CeresBackdropWalls
            | ObservationId::CeresBackdropColours
            | ObservationId::CeresTailState
            | ObservationId::CeresTailPlaces
            | ObservationId::CeresTailHit
            | ObservationId::CeresTailSwish
            | ObservationId::CeresFlybyState
            | ObservationId::CeresFlybyPlane
            | ObservationId::CeresFlybyColours
            | ObservationId::CeresFlybyTiles
            | ObservationId::CeresFlybySound
            | ObservationId::CeresFlybyPush
            | ObservationId::CeresFlybyRoom
            | ObservationId::CeresWarningState
            | ObservationId::CeresWarningPicture
            | ObservationId::CeresWarningAudio
            | ObservationId::CeresWarningHandoff
            | ObservationId::CeresShaftState
            | ObservationId::CeresShaftEvents
            | ObservationId::CeresShaftPicture
            | ObservationId::ShaftSoundOffer
            | ObservationId::CeresRibsBeat
            | ObservationId::CeresRibsWait
            | ObservationId::CeresRibsUploads
            | ObservationId::CeresRibsUpper
            | ObservationId::CeresRibsLower
            | ObservationId::CeresFightDoorCursor
            | ObservationId::CeresFightPreparedOam
            | ObservationId::CeresFightPhysicalOam
    };
}

macro_rules! landing_observations {
    () => {
        O::DepartureRandom
            | O::DepartureRoom
            | O::DepartureLoader
            | O::LandingRandom
            | O::LandingCarried
            | O::LandingMap
            | O::LandingPipeline
            | O::LandingVram
            | O::LandingCgram
            | O::LandingOam
            | O::LandingLive
            | O::LandingPublished
            | O::LandingMirrors
            | O::LandingDma
            | O::LandingTilemaps
    };
}

macro_rules! shaft_observations {
    () => {
        ObservationId::ShaftView
            | ObservationId::ShaftTransform
            | ObservationId::ShaftRotation
            | ObservationId::ShaftLevel
            | ObservationId::ShaftPlacement
            | ObservationId::ShaftHaze
    };
}

macro_rules! health_observations {
    () => {
        ObservationId::HealthTailState
            | ObservationId::HealthTailWarning
            | ObservationId::HealthTailDrawnProtection
            | ObservationId::CeresTailHealth
            | ObservationId::DepartureHealth
            | ObservationId::ZebesHealth
    };
}

macro_rules! appearance_observations {
    () => {
        O::StandingOamPending
            | O::ShaftDustState
            | O::ShaftDustPrepared
            | O::ShaftDustPublished
            | O::ShaftDustFollowingVent
            | O::StandingStripsPending
            | O::StandingOamPublished
            | O::StandingPublication
            | O::StandingUpperFirst
            | O::StandingUpperSecond
            | O::StandingLowerFirst
            | O::StandingLowerSecond
            | O::DrawingEnergy
            | O::TouchdownGraphics
            | O::OrdinaryBodyVisible
            | O::OrdinaryBodyCount
            | O::OrdinaryBodyPrepared
            | O::OrdinaryBodyPhysical
            | O::AppearancePlaying
            | O::AppearanceStep
            | O::AppearanceTimeLeft
            | O::AppearanceBoostStage
            | O::AppearanceLapsLeft
            | O::MapCarryAppearance
    };
}

macro_rules! hud_observations {
    () => {
        ObservationId::GaugeRefresh
            | ObservationId::GaugeWords
            | ObservationId::EarlyMapWindow
            | ObservationId::HudGrid
            | ObservationId::HudExplored
            | ObservationId::HudEnergy
            | ObservationId::HudMissiles
            | ObservationId::HudSupers
            | ObservationId::HudBombs
            | ObservationId::HudHighlighted
            | ObservationId::HudSelected
            | ObservationId::HudOneShot
            | ObservationId::HudMapOff
            | ObservationId::HudFrameCount
            | ObservationId::HudPendingTransfer
            | ObservationId::HudPublication
            | ObservationId::HudPublishedGrid
    };
}

const fn observation_owner(field: ObservationId) -> Responsibility {
    use ObservationId as O;
    match field {
        discovery_observations!() => Responsibility::Discovery,
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
        landing_observations!() => Responsibility::LandingSite,
        O::ZebesStage | O::ZebesPicture | O::ZebesMotion | O::ZebesRequests => {
            Responsibility::Zebes
        }
        O::DepartureSchedule | O::ZebesSchedule => Responsibility::ZebesScheduler,
        O::DeparturePicture | O::ZebesPages | O::ZebesOam | O::ZebesPalette => {
            Responsibility::ZebesDrawing
        }
        O::SenderQueue
        | O::SenderProgress
        | O::SenderPermission
        | O::SenderRequest
        | O::TouchdownOffer => Responsibility::AudioSender,
        shaft_observations!() => Responsibility::RoomLoading,
        health_observations!() => Responsibility::Health,
        explosion_observations!() => Responsibility::StationExplosion,
        blackout_observations!() => Responsibility::Blackout,
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
        | O::GetawayPhase
        | O::GetawayLatestPose
        | O::GetawayOlderPose
        | O::GetawayBodyX
        | O::GetawayBodyY
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
        ceres_observations!() => Responsibility::Ceres,
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
        hud_observations!() => Responsibility::Hud,
        weapons_observations!() => Responsibility::Weapons,
        O::ContactConsumedMarkers | O::ContactReceivers => Responsibility::Contact,
        O::ShaftPictureClocks | O::ShaftPictureEffects | O::ShaftPictureOam => {
            Responsibility::ShaftPicture
        }
        appearance_observations!() => Responsibility::Appearance,
        progress_observations!() => Responsibility::Progress,
    }
}

fn rule_owner(rule: &str, provider: InputProvider) -> Option<Responsibility> {
    if rule.starts_with("enemy-interactions/")
        && witnessed_fields(rule, InputProvider::ShaftPictures).is_some_and(|fields| {
            fields
                .iter()
                .all(|field| observation_owner(*field) == Responsibility::ShaftPicture)
        })
    {
        return Some(Responsibility::ShaftPicture);
    }
    if rule == "game-random/AdvanceRandom" {
        return Some(Responsibility::LandingSite);
    }
    if provider == InputProvider::PlayedShaftSound
        && matches!(
            rule,
            "audio-transport/RequestDelivery" | "audio-transport/RequestRelease"
        )
    {
        return Some(Responsibility::AudioSender);
    }
    if rule == "audio-transport/OfferAcceptance" {
        return Some(Responsibility::AudioSender);
    }
    [
        ("screens-intro/", Responsibility::Intro),
        ("world-intro-rooms/", Responsibility::IntroRooms),
        ("screens-flashback/", Responsibility::Flashback),
        ("screens-discovery/", Responsibility::Discovery),
        ("screens-zebes/", Responsibility::Zebes),
        ("samus-movement/", Responsibility::Movement),
        ("screens-explosion/", Responsibility::StationExplosion),
        ("screens-blackout/", Responsibility::Blackout),
        ("screens-playback/", Responsibility::Playback),
        ("audio-transport/", Responsibility::Audio),
        ("boss-ceres-ridley/", Responsibility::Ceres),
        ("world-camera/", Responsibility::Camera),
        ("world-shaft-entry/", Responsibility::RoomLoading),
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
    if provider == InputProvider::Departure
        || rule.starts_with("screens-zebes/")
        || rule.starts_with("screens-explosion/")
    {
        return cinematic_witnesses(rule, provider);
    }
    if rule.starts_with("screens-blackout/") {
        return super::blackout::witnessed_fields(rule);
    }
    Some(match rule {
        "screens-blackout/ElevatorStopsSamus"
        | "screens-blackout/PlatformWait"
        | "screens-blackout/DepartureHandoff" => &[
            O::BlackoutScene,
            O::BlackoutRoom,
            O::BlackoutRequests,
            O::BlackoutBattery,
            O::BlackoutTimerDrawing,
        ],
        "world-shaft-entry/ShaftViewStarts" => &[O::ShaftView, O::ShaftTransform, O::ShaftRotation],
        "world-shaft-entry/ShaftDoorSeals" => &[O::ShaftLevel, O::ShaftPlacement, O::ShaftHaze],
        "samus-movement/Jumping" if provider == InputProvider::PassageJumpDraws => {
            &[O::DrawnJumpHeld, O::LastDrawJump]
        }
        "samus-movement/BackwardLeap" | "samus-movement/WhatHerMovesMet" => {
            &[O::LandingPose, O::LandingFloor]
        }
        "game-random/AdvanceRandom" => &[O::LandingRandom],
        "samus-movement/MovesAsShePosed" | "samus-movement/PoseFitAfterMoves" => {
            &[O::PositionX, O::PositionY]
        }
        "samus-movement/PadChart" => &[O::MovementPose],
        "samus-movement/GetawayArms" => &[O::GetawayPhase],
        "samus-movement/GetawayStarts" => &[O::GetawayPhase, O::MovementPose, O::PositionY],
        "samus-movement/GetawayTravels" => &[O::GetawayPhase, O::PositionX, O::PositionY],
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
        "enemy-interactions/ActorShake" => &[O::ShaftPictureClocks, O::ShaftPictureOam],
        "enemy-interactions/EffectSlots" | "enemy-interactions/EffectClip" => {
            &[O::ShaftPictureEffects]
        }
        "enemy-interactions/ShaftActorDraw"
        | "enemy-interactions/PictureWindow"
        | "enemy-interactions/ShaftPictureOrder" => &[O::ShaftPictureOam],
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
        "boss-ceres-ridley/OverlayRumbles" if provider == InputProvider::PlayedShaftSound => {
            &[O::ShaftSoundOffer]
        }
        "audio-transport/RequestDelivery" | "audio-transport/RequestRelease"
            if provider == InputProvider::PlayedShaftSound =>
        {
            &[O::SenderProgress, O::SenderRequest]
        }
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
        super::blackout::standing::RULE => super::blackout::standing::OBSERVATIONS,
        "samus-appearance/ProtectionFlicker" => super::played::samus_picture::OUTPUTS,
        "samus-appearance/WallJumpDust" => &[O::ShaftDustState],
        "samus-appearance/WallJumpDustPicture" => &[O::ShaftDustPrepared, O::ShaftDustPublished],
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
        "boss-ceres-ridley/GetawayPass" => {
            &[O::CeresFlybyState, O::CeresFlybySound, O::CeresFlybyPush]
        }
        "boss-ceres-ridley/GetawayPicture" => {
            &[O::CeresFlybyPlane, O::CeresFlybyColours, O::CeresFlybyTiles]
        }
        "boss-ceres-ridley/WarningArt" | "boss-ceres-ridley/WarningGlow" => {
            &[O::CeresWarningState, O::CeresWarningPicture]
        }
        "boss-ceres-ridley/WarningHeading" | "boss-ceres-ridley/WarningTyping" => &[
            O::CeresWarningState,
            O::CeresWarningPicture,
            O::CeresWarningAudio,
        ],
        "boss-ceres-ridley/RibsBeat" => &[
            O::CeresRibsBeat,
            O::CeresRibsWait,
            O::CeresRibsUploads,
            O::CeresRibsUpper,
            O::CeresRibsLower,
        ],
        "boss-ceres-ridley/FightParts" | "boss-ceres-ridley/FightPictureOrder" => &[
            O::CeresFightDoorCursor,
            O::CeresFightPreparedOam,
            O::CeresFightPhysicalOam,
        ],
        "boss-ceres-ridley/WarningEnds" => &[O::CeresWarningState, O::CeresWarningHandoff],
        "boss-ceres-ridley/OverlayAppears" => &[
            O::CeresShaftState,
            O::CeresShaftEvents,
            O::CeresShaftPicture,
        ],
        "boss-ceres-ridley/OverlayRumbles" | "boss-ceres-ridley/ElevatorReached" => {
            &[O::CeresShaftState, O::CeresShaftEvents]
        }
        "boss-ceres-ridley/ShaftTilts" | "boss-ceres-ridley/VentDrawFollowsShaft" => {
            &[O::CeresShaftState]
        }
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
        "screens-discovery/EggHatches" => &[O::DiscoveryEgg, O::DiscoveryOam],
        "screens-discovery/ShellScatters" => {
            &[O::DiscoveryFragments, O::DiscoverySounds, O::DiscoveryOam]
        }
        "screens-discovery/BabyEmerges" => &[O::DiscoveryBaby],
        "screens-discovery/SlimeFalls" => {
            &[O::DiscoveryDroplets, O::DiscoverySounds, O::DiscoveryOam]
        }
        "screens-discovery/BabyPulses" => &[O::DiscoveryPulse, O::DiscoveryOam],
        "screens-discovery/BabyFollows" => {
            &[O::DiscoveryBaby, O::DiscoveryOrder, O::DiscoverySounds]
        }
        "screens-discovery/PageThreeApproaches" => {
            &[O::DiscoveryCue, O::DiscoveryTurn, O::DiscoveryOam]
        }
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

fn cinematic_witnesses(rule: &str, provider: InputProvider) -> Option<&'static [ObservationId]> {
    use ObservationId as O;
    if provider == InputProvider::Departure {
        return match rule {
            "screens-explosion/SceneBlacksOut" => Some(&[O::DepartureSchedule, O::DepartureHealth]),
            "screens-zebes/LandingSiteLoadBegins" => {
                Some(&[O::DepartureSchedule, O::DepartureHealth, O::DepartureLoader])
            }
            "game-random/AdvanceRandom" => Some(&[O::DepartureRandom]),
            _ => None,
        };
    }
    Some(match rule {
        "screens-zebes/GunshipAppears" | "screens-zebes/ShipApproachesPlanet" => {
            &[O::ZebesStage, O::ZebesPicture]
        }
        "screens-zebes/PlanetAppears" | "screens-zebes/PlanetAndStarsSlide" => {
            &[O::ZebesStage, O::ZebesMotion]
        }
        "screens-zebes/PlanetIsNamed" | "screens-zebes/LandingSiteLoadBegins" => {
            &[O::ZebesStage, O::ZebesRequests]
        }
        "screens-zebes/PlanetHolds" => &[O::ZebesStage],
        "screens-explosion/StationPictureArrives" => {
            &[O::ExplosionStage, O::ExplosionDrawing, O::ExplosionRequests]
        }
        "screens-explosion/StationAwaitsMusic" | "screens-explosion/SpaceHolds" => {
            &[O::ExplosionStage]
        }
        "screens-explosion/StationBrightens" | "screens-explosion/ShipDeparts" => {
            &[O::ExplosionStage, O::ExplosionMotion]
        }
        "screens-explosion/SpaceDrifts" => &[O::ExplosionMotion, O::ExplosionDrawing],
        "screens-explosion/StationBursts" => &[O::ExplosionBursts, O::ExplosionDrawing],
        "screens-explosion/StationBreaks" => &[
            O::ExplosionStage,
            O::ExplosionBursts,
            O::ExplosionRequests,
            O::ExplosionDrawing,
        ],
        "screens-explosion/SceneBlacksOut" => {
            &[O::ExplosionStage, O::ExplosionCarry, O::ExplosionRequests]
        }

        _ => return None,
    })
}

fn native_search_provider_valid(case: &Case, observations: &BTreeSet<ObservationId>) -> bool {
    use ObservationId as O;
    super::native_search::Packet::for_case(case).is_some()
        && *observations == BTreeSet::from([O::CompleteReached, O::Inside])
        && case.kind == CaseKind::Scoped
        && case.content == ContentProvider::PlayerRomImport
        && case.expectations.provider == ExpectedProvider::NativeCompleteMovementWork
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
        && case
            .connected_packet()
            .is_some_and(|packet| packet.admits(case))
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
        && case
            .sender_packet()
            .is_ok_and(|packet| packet.row == case.row)
        && case.content == ContentProvider::PlayerRomImport
        && case.inputs == InputProvider::NativeTouchdownInputs
        && case.expectations.provider == ExpectedProvider::NativeTouchdownObservations
        && matches!(case.start, Start::CacheCut { frame } if frame.checked_add(1) == Some(case.end.cut_inclusive))
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
        && case.cache_window().is_ok()
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
        && case.cache_window().is_ok_and(|window| {
            window.origin > 0 && window.origin.checked_add(2) == Some(case.end.cut_inclusive)
        })
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
    let Start::CacheCut { frame } = case.start else {
        return false;
    };
    recorded
        && case.content == ContentProvider::PlayerRomImport
        // A delivery start must include both the interrupted room turn and its resume.
        && (case.inputs == InputProvider::ScopedIntroInputs
            || (case.inputs == InputProvider::ScopedIntroDeliveryInputs
                && case.end.cut_inclusive.saturating_sub(frame) >= 2))
        && case.expectations.fields == [ObservationId::IntroState]
        && case.expectations.outputs == [ObservationId::IntroPublications]
        && case.expectations.output_cut.is_none()
        && !case.checkpoints
}

fn flashback_provider_valid(case: &Case, recorded: bool) -> bool {
    use ObservationId as O;

    recorded
        && case.intro_join.is_some()
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
        && case.intro_join.is_some()
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
        && matches!(case.start, Start::CacheCut { frame: 0 })
        && case
            .save_steps
            .as_ref()
            .is_some_and(|schedule| schedule.valid(case))
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

fn discovery_provider_valid(case: &Case, recorded: bool) -> bool {
    use ObservationId as O;
    recorded
        && case.intro_join.is_some()
        && case.inputs == InputProvider::DiscoveryInputs
        && case.content == ContentProvider::PlayerRomImport
        && case.expectations.output_cut.is_none()
        && case.checkpoints
        && !case.borrowed.is_empty()
        && !case.limits.is_empty()
        && case.expectations.fields
            == [
                O::DiscoveryEgg,
                O::DiscoveryBaby,
                O::DiscoveryPulse,
                O::DiscoveryFragments,
                O::DiscoveryDroplets,
                O::DiscoveryOrder,
            ]
        && case.expectations.outputs
            == [
                O::DiscoverySounds,
                O::DiscoveryCue,
                O::DiscoveryTurn,
                O::DiscoveryOam,
            ]
}

fn shaft_provider_valid(
    case: &Case,
    observations: &BTreeSet<ObservationId>,
    recorded: bool,
) -> bool {
    recorded
        && case.kind == CaseKind::Scoped
        && matches!(case.start, Start::CacheCut { .. })
        && case.first_cut().is_some_and(|first| {
            super::played::witnesses::selected(case)
                .and_then(|data| data.shaft())
                .is_ok_and(|data| data.entry.covers(&case.row, first, case.end.cut_inclusive))
        })
        && case.inputs == InputProvider::ShaftSetupTurns
        && case.content == ContentProvider::PlayerRomImport
        && case.checkpoints
        && observations.len() == 6
        && !case.borrowed.is_empty()
        && !case.limits.is_empty()
        && case.expectations.outputs.is_empty()
}

fn zebes_provider_valid(case: &Case) -> bool {
    use super::zebes::{INTEGRATED_FIELDS, INTEGRATED_OUTPUTS, SCOPED_FIELDS, SCOPED_OUTPUTS};
    use Responsibility::{Health, Zebes, ZebesDrawing, ZebesScheduler};

    let supported = match case.kind {
        CaseKind::Scoped => {
            case.responsibilities == [Zebes]
                && case.expectations.fields == SCOPED_FIELDS
                && case.expectations.outputs == SCOPED_OUTPUTS
        }
        CaseKind::Integrated => {
            case.responsibilities == [Zebes, ZebesDrawing, Health, ZebesScheduler]
                && case.expectations.fields == INTEGRATED_FIELDS
                && case.expectations.outputs == INTEGRATED_OUTPUTS
        }
        CaseKind::Movie => false,
    };
    supported
        && case.content == ContentProvider::PlayerRomImport
        && case.expectations.provider == ExpectedProvider::PinnedRecordCache
        && case.expectations.output_cut.is_none()
        && matches!(case.start, Start::CacheCut { .. })
}

fn station_provider_valid(case: &Case) -> bool {
    use ObservationId as O;
    case.kind == CaseKind::Scoped
        && case.responsibilities == [Responsibility::StationExplosion]
        && case.content == ContentProvider::PlayerRomImport
        && case.expectations.provider == ExpectedProvider::PinnedRecordCache
        && matches!(case.start, Start::CacheCut { .. })
        && case.expectations.output_cut.is_none()
        && case.expectations.fields
            == [
                O::ExplosionStage,
                O::ExplosionMotion,
                O::ExplosionBursts,
                O::ExplosionCarry,
            ]
        && case.expectations.outputs == [O::ExplosionRequests, O::ExplosionDrawing]
        && !case.borrowed.is_empty()
        && !case.limits.is_empty()
}

fn landing_provider_valid(case: &Case) -> bool {
    use ObservationId as O;
    case.kind == CaseKind::Scoped
        && case.cache_window().is_ok()
        && case.responsibilities == [Responsibility::LandingSite]
        && case.content == ContentProvider::PlayerRomImport
        && case.expectations.provider == ExpectedProvider::PinnedRecordCache
        && case.expectations.fields
            == [
                O::LandingRandom,
                O::LandingCarried,
                O::LandingMap,
                O::LandingPipeline,
            ]
        && case.expectations.outputs
            == [
                O::LandingVram,
                O::LandingCgram,
                O::LandingOam,
                O::LandingLive,
                O::LandingPublished,
                O::LandingMirrors,
                O::LandingDma,
                O::LandingTilemaps,
            ]
        && case.expectations.output_cut.is_none()
        && case.checkpoints
}

fn provider_valid(case: &Case, observations: &BTreeSet<ObservationId>) -> bool {
    if case.inputs == InputProvider::PlayedShaftSound {
        return super::played::shaft_sound::valid(case);
    }
    if matches!(
        case.inputs,
        InputProvider::PlayedCeresInputs
            | InputProvider::ScopedCeresFlyby
            | InputProvider::ScopedCeresWarning
            | InputProvider::CeresShaftActors
            | InputProvider::ScopedCeresFightPicture
    ) {
        return super::played::indexed::valid(case);
    }
    if case.inputs == InputProvider::PlayedSamusPicture {
        return super::played::samus_picture::valid(case);
    }
    if case.inputs == InputProvider::PlayedShaftDust {
        return super::played::shaft_dust::valid(case);
    }
    if case.inputs == InputProvider::StationExplosion {
        return station_provider_valid(case);
    }
    if case.inputs == InputProvider::Blackout {
        return super::blackout::valid(case);
    }
    if case.inputs == InputProvider::IntroRoomJoinInputs {
        return super::intro_rooms::valid(case);
    }
    if case.inputs == InputProvider::SaveFileBoundaries {
        return save_files_provider_valid(case);
    }
    if case.inputs == InputProvider::LandingSiteTiming {
        return landing_provider_valid(case);
    }
    if case.inputs == InputProvider::ZebesFlightInputs {
        return zebes_provider_valid(case);
    }
    if case.inputs == InputProvider::EarlyMapInputs {
        return early_map_provider_valid(case);
    }
    if case.inputs == InputProvider::Unavailable {
        return unavailable_provider_valid(case, observations);
    }
    responsibility_provider_valid(case, observations)
}

fn responsibility_provider_valid(case: &Case, observations: &BTreeSet<ObservationId>) -> bool {
    use ObservationId as O;
    let recorded = case.kind == CaseKind::Scoped
        && matches!(
            case.expectations.provider,
            ExpectedProvider::PinnedRecordCache | ExpectedProvider::WitnessedRecordCache
        )
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
        [Responsibility::Discovery] => discovery_provider_valid(case, recorded),
        [Responsibility::RoomLoading] => shaft_provider_valid(case, observations, recorded),
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
        && matches!(case.start, Start::NativeRecord { frame, .. } if frame < case.end.cut_inclusive)
        && case.health_packet().is_ok()
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
        if rule_owner(rule, case.inputs).is_none_or(|owner| !owners.contains(&owner)) {
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
    if !super::recorded::Witness::valid_for(case) {
        return false;
    }
    if case.inputs == InputProvider::ShaftPictures {
        return super::played::shaft_native::indexed::valid(case);
    }
    if case.inputs == InputProvider::ScopedGetawayInputs {
        return super::played::getaway::valid(case);
    }
    if case.inputs == InputProvider::Departure {
        return super::departure::valid(case);
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
        Start::PlayedContinuation { seed, frame } => {
            *seed < *frame
                && *frame < case.end.cut_inclusive
                && matches!(
                    case.inputs,
                    InputProvider::Departure
                        | InputProvider::PlayedShaftSound
                        | InputProvider::ShaftPictures
                        | InputProvider::PlayedShaftDust
                )
        }
        Start::CacheCut { frame } => {
            *frame < case.end.cut_inclusive
                || (*frame == case.end.cut_inclusive && case.first_cut() == Some(*frame))
        }
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
        Start::NativeSenderOffer { source_id } => case
            .sender_packet()
            .is_ok_and(|packet| *source_id < packet.windows && packet.row == case.row),
        Start::ConstructedSenderOffer { control } => {
            control.valid() && case.row == "constructed-sender-queue" && case.end.cut_inclusive == 0
        }
        Start::PowerOn => case.kind == CaseKind::Movie && case.borrowed.is_empty(),
    }
}

pub(super) struct Window<'a> {
    pub row: &'a str,
    pub origin: u64,
    pub cuts: std::ops::RangeInclusive<u64>,
}

impl<'a> Window<'a> {
    pub(super) fn after(case: &'a Case, origin: u64) -> Result<Self, super::result::Outcome> {
        let first = origin
            .checked_add(1)
            .filter(|cut| *cut <= case.end.cut_inclusive)
            .ok_or_else(|| super::data_failure(Some(origin), "empty replay window"))?;
        Ok(Self {
            row: &case.row,
            origin,
            cuts: first..=case.end.cut_inclusive,
        })
    }
}

impl Case {
    pub(super) fn cache_window(&self) -> Result<Window<'_>, super::result::Outcome> {
        let Start::CacheCut { frame } = self.start else {
            return Err(super::data_failure(None, "cache window origin"));
        };
        Window::after(self, frame)
    }

    pub(super) fn ordinary_witness(&self) -> Option<&super::recorded::Witness> {
        if self.expectations.provider == ExpectedProvider::WitnessedRecordCache {
            self.ordinary_witness.as_ref()
        } else if self.expectations.provider == ExpectedProvider::PinnedRecordCache
            && super::recorded::Witness::required(self)
        {
            super::legacy_packets::ordinary_witness(&self.id.0)
        } else {
            None
        }
    }

    pub(super) fn fixture_pair(&self) -> Result<&FixturePair, super::result::Outcome> {
        self.fixture
            .as_ref()
            .or_else(|| super::legacy_packets::fixture(self.inputs))
            .filter(|pair| pair.provider == self.inputs && pair.valid())
            .ok_or_else(|| super::data_failure(None, "scoped provider fixture"))
    }

    pub(super) fn connected_packet(&self) -> Option<&ConnectedPacket> {
        self.connected_packet
            .as_ref()
            .or_else(|| super::legacy_packets::connected(&self.id.0))
    }

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
        if case.inputs.uses_played_witnesses() != case.played_witnesses.is_some()
            || case
                .played_witnesses
                .as_ref()
                .is_some_and(|data| !data.valid())
        {
            return Err(IndexFault::InvalidCase(case.id.clone()));
        }
        if (case.fixture.is_some() || case.inputs.uses_fixture_pair())
            && case.fixture_pair().is_err()
        {
            return Err(IndexFault::InvalidCase(case.id.clone()));
        }
        if case
            .native_packet
            .as_ref()
            .is_some_and(|packet| !packet.applies_to(case) || !packet.valid())
        {
            return Err(IndexFault::InvalidCase(case.id.clone()));
        }
        if case.inputs == InputProvider::NativeFrameInputs && case.audio_packet().is_err() {
            return Err(IndexFault::InvalidCase(case.id.clone()));
        }
        if case
            .intro_join
            .as_ref()
            .is_some_and(|join| !join.valid(case))
        {
            return Err(IndexFault::InvalidCase(case.id.clone()));
        }
        if (case.expectations.projection.is_some()
            && case.inputs != InputProvider::NativeSearchInputs)
            || case
                .expectations
                .played
                .as_ref()
                .is_some_and(|window| !match window {
                    PlayedWindow::FightPicture { .. } => {
                        case.inputs == InputProvider::ScopedCeresFightPicture
                    }
                    PlayedWindow::RibsReveal { .. } => {
                        case.inputs == InputProvider::PlayedCeresInputs
                            && super::played::ridley_ribs::is_case(case)
                    }
                    PlayedWindow::ShaftActors { .. } => {
                        case.inputs == InputProvider::CeresShaftActors
                    }
                    PlayedWindow::Flyby { .. } => case.inputs == InputProvider::ScopedCeresFlyby,
                })
            || (case.connected_packet.is_some()
                && case.inputs != InputProvider::ConnectedOrdinaryInputs)
            || (case.save_steps.is_some() && case.inputs != InputProvider::SaveFileBoundaries)
            || case.inputs == InputProvider::PublishedSaveFileBoundaries
            || case.id.0.is_empty()
            || case.row.is_empty()
            || (case.rules.is_empty() && case.inputs != InputProvider::Unavailable)
            || case.profile != "reference"
            || case.responsibilities.is_empty()
            || (observations.is_empty() && case.inputs != InputProvider::Unavailable)
            || unique.len() != observations.len()
            || !start_valid(case)
            || !scope_valid(case)
            || !super::borrowed::valid(case)
            || case.borrowed.iter().any(|borrowed| {
                borrowed.dependency.is_empty()
                    || borrowed.provider.is_empty()
                    || borrowed.fields.is_empty()
                    || borrowed.phase.is_empty()
            })
        {
            return Err(IndexFault::InvalidCase(case.id.clone()));
        }
        Ok(())
    }

    /// First original comparison coordinate in this uninterrupted case.
    #[must_use]
    pub const fn first_cut(&self) -> Option<u64> {
        if let Start::CacheCut { frame } = self.start
            && matches!(
                self.inputs,
                InputProvider::FlashbackInputs | InputProvider::IntroMissileInputs
            )
        {
            // The selected setup cut is itself a comparison.
            return Some(frame);
        }
        if matches!(self.inputs, InputProvider::CeresShaftActors) {
            return match self.expectations.played {
                Some(PlayedWindow::ShaftActors { first, .. }) => Some(first),
                _ => None,
            };
        }
        match self.start {
            Start::CacheCut { .. } if matches!(self.inputs, InputProvider::ScopedCeresFlyby) => {
                match self.expectations.played {
                    Some(PlayedWindow::Flyby { first, .. }) => Some(first),
                    _ => None,
                }
            }
            Start::CacheCut { frame }
            | Start::NativeRecord { frame, .. }
            | Start::PlayedContinuation { frame, .. } => frame.checked_add(1),
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
        if !matches!(self.format, 1 | CURRENT_FORMAT)
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
    valid_hex(hash, 64)
}

fn valid_hex(hash: &str, width: usize) -> bool {
    hash.len() == width
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
    fn constructed_sender_queues_and_offers_belong_to_the_selected_group() {
        let index = Index::parse(INDEX).unwrap();
        let case = index
            .cases
            .iter()
            .find(|case| case.id.0 == "sender-offer/ordered-pair")
            .unwrap();
        let super::Start::ConstructedSenderOffer { control } = &case.start else {
            panic!()
        };
        let mut invalid = control.clone();
        invalid.cues.clear();
        assert!(!invalid.valid());
        invalid.cues.push(audio_transport::CueId::Beam);
        assert!(!invalid.valid());
        for expected in [false, true] {
            let mut invalid = control.clone();
            if expected {
                invalid.expected.push(audio_transport::CueId::Beam);
            } else {
                invalid.seed.push(audio_transport::CueId::Beam);
            }
            assert!(!invalid.valid());
        }
    }

    #[test]
    fn native_packets_remain_attached_to_cases_extracted_from_the_index() {
        let index = Index::parse(INDEX).unwrap();
        let sender = index
            .cases
            .iter()
            .find(|case| matches!(case.start, super::Start::NativeSenderOffer { .. }))
            .unwrap();
        let standalone: super::Case = toml::from_str(&toml::to_string(sender).unwrap()).unwrap();
        assert!(standalone.native_packet.is_some() && standalone.validate().is_ok());
        let mut constructed = index
            .cases
            .iter()
            .find(|case| matches!(case.start, super::Start::ConstructedSenderOffer { .. }))
            .unwrap()
            .clone();
        assert!(constructed.native_packet.is_none());
        constructed.native_packet = standalone.native_packet;
        assert!(constructed.validate().is_err());

        let mut document: super::IndexDocument = toml::from_str(INDEX).unwrap();
        let packets = document.native_packet.as_mut().unwrap();
        packets.push(packets.first().unwrap().clone());
        assert!(Index::try_from(document).is_err());
    }

    #[test]
    fn current_indexes_require_each_consumed_native_declaration() {
        for kind in ["sender", "health", "audio"] {
            let mut doc: super::IndexDocument = toml::from_str(INDEX).unwrap();
            doc.native_packet.as_mut().unwrap().retain(|packet| {
                !matches!(
                    (kind, packet),
                    ("sender", super::NativePacket::Sender(_))
                        | ("health", super::NativePacket::Health(_))
                        | ("audio", super::NativePacket::Audio(_))
                )
            });
            assert!(Index::try_from(doc).is_err());
        }
    }

    #[test]
    fn witnessed_records_require_an_explicit_source_after_renaming() {
        let index = Index::parse(INDEX).unwrap();
        for mut case in index.cases.into_iter().filter(|case| {
            case.expectations.provider == super::ExpectedProvider::WitnessedRecordCache
        }) {
            case.id.0.push_str("-reissued");
            assert!(case.validate().is_ok());
            case.ordinary_witness = None;
            assert!(case.validate().is_err());
            case.expectations.provider = super::ExpectedProvider::PinnedRecordCache;
            assert!(case.validate().is_err());
        }
    }

    #[test]
    fn shared_fixture_pairs_travel_with_selected_cases() {
        let index = Index::parse(INDEX).unwrap();
        let mut touchdown = index
            .cases
            .iter()
            .filter(|case| case.inputs == super::InputProvider::NativeTouchdownInputs);
        let mut first = touchdown.next().unwrap().clone();
        let second = touchdown.next().unwrap();
        let pair = first.fixture.as_ref().unwrap();
        assert_eq!(
            pair.inputs.path,
            second.fixture.as_ref().unwrap().inputs.path
        );
        let mut wrong = first.clone();
        wrong.fixture = index
            .cases
            .iter()
            .find(|case| case.inputs == super::InputProvider::EarlyMapInputs)
            .unwrap()
            .fixture
            .clone();
        assert!(wrong.validate().is_err());

        let standalone: super::Case = toml::from_str(&toml::to_string(&first).unwrap()).unwrap();
        assert!(standalone.validate().is_ok());
        assert_eq!(
            standalone.fixture.unwrap().expected.sha256,
            pair.expected.sha256
        );
        for path in [
            "",
            "expected.toml",
            "../expected.toml",
            "fixture/..",
            "/outside/file",
            "dir/sub/file",
            "dir/..\\file",
            "C:/file",
        ] {
            first.fixture.as_mut().unwrap().expected.path = path.to_owned();
            assert!(first.validate().is_err(), "{path}");
        }
    }

    #[test]
    fn shared_fixtures_have_one_valid_declaration_per_provider() {
        let mut doc: super::IndexDocument = toml::from_str(INDEX).unwrap();
        doc.fixture.push(doc.fixture.first().unwrap().clone());
        assert!(Index::try_from(doc).is_err());
        let mut doc: super::IndexDocument = toml::from_str(INDEX).unwrap();
        doc.fixture.first_mut().unwrap().inputs.sha256 = "uncounted".to_owned();
        assert!(Index::try_from(doc).is_err());
    }

    #[test]
    fn connected_packets_bind_the_reviewed_scope_independently_of_the_case_name() {
        let index = Index::parse(INDEX).unwrap();
        let mut case = index
            .cases
            .iter()
            .find(|case| case.connected_packet.is_some())
            .unwrap()
            .clone();
        case.id = CaseId("connected-ordinary/another-prefix".to_owned());
        case.end.cut_inclusive = case.connected_packet().unwrap().first_end;
        assert!(case.validate().is_ok());
        case.row = "another-row".to_owned();
        assert!(case.validate().is_err());
        case.row = case.connected_packet().unwrap().row.clone();
        case.connected_packet.as_mut().unwrap().directory = "../outside".to_owned();
        assert!(case.validate().is_err());
        case.connected_packet = None;
        assert!(case.validate().is_err());
    }

    #[test]
    fn earlier_connected_cases_keep_their_published_packet_and_horizons() {
        let index = Index::parse(INDEX).unwrap();
        for (name, directory, start, end) in [
            (
                "connected-ordinary/carried-running-camera",
                "connected-ordinary-v1",
                6,
                15,
            ),
            (
                "connected-ordinary/run-release-floor-aim",
                "connected-run-slope-v1",
                126,
                142,
            ),
        ] {
            let mut case = index
                .cases
                .iter()
                .find(|case| case.id.0 == name)
                .unwrap()
                .clone();
            case.connected_packet = None;
            case.row = "chain-any-0003-DFD7".to_owned();
            case.start = super::Start::CacheCut { frame: start };
            case.end.cut_inclusive = end;
            assert_eq!(case.connected_packet().unwrap().directory, directory);
            assert!(case.validate().is_ok());
            case.end.cut_inclusive = end.saturating_add(1);
            assert!(case.validate().is_err());
        }
    }

    #[test]
    fn malformed_fields_unknown_keys_and_fictitious_rules_are_rejected() {
        assert!(matches!(
            Index::parse(&INDEX.replace("movement.travel.Left", "movement.travel.Unknown")),
            Err(IndexFault::Syntax)
        ));
        assert!(matches!(
            Index::parse(&format!("extra = true\n{INDEX}")),
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
