//! Strict case metadata. Each case names its witness, the rules it holds and the fields it compares.

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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// An existing selected production owner.
pub enum Responsibility {
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
    /// Movie pad, room seed and the played platform boundaries for a Ceres tail window.
    PlayedCeresInputs,
    /// One elevator arrival and T1 scheduling.
    Blackout,
    /// One admitted room and finished opportunities for the flyby’s owned producers.
    ScopedCeresFlyby,
    /// One admitted warning-ready room and the usual played room/passage inputs.
    ScopedCeresWarning,
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
    /// Locked encounter phase, after-motion room facts and presentation opportunities.
    #[serde(rename = "flashback-segments")]
    FlashbackInputs,
    /// Recorded scene setup and reached room actor opportunities.
    #[serde(rename = "cinematic-actor-segments")]
    CinematicActorOpportunities,
    /// Ordered startup and save boundaries from pinned preceding cuts.
    #[serde(rename = "save-file-schedule")]
    SaveFileBoundaries,
    /// Authored demo controls and borrowed cinematic weapon opportunities.
    IntroMissileInputs,
    /// Scoped discovery and delivery inputs, including the interrupted start.
    ScopedIntroDeliveryInputs,
    /// The intro requests, completion and resulting actor position in the discovery room.
    #[serde(rename = "discovery-segments")]
    DiscoveryInputs,
    /// Receiving T1 work opening and byte progress.
    LandingSiteTiming,
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
    /// Independent prior facts while captured camera producers remain unchanged.
    PrecedingCutCameraInputs,
    /// Independent cannon/terrain facts and bounded watched actor opportunities.
    PrecedingCutWeaponsContactInputs,
    /// Independently watched ordinary weapon inputs and the flying-picture insertion context.
    ScopedShotDrawingInputs,
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
/// Source of immutable admitted content.
pub enum ContentProvider {
    /// Checked content imported from the player ROM.
    PlayerRomImport,
    /// Silent audio content: no clips or song, for frames that only queue offers.
    SenderGroup,
    /// ROM-selected audio for the independently admitted opening-room load.
    OpeningCeresAudio,
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
    pub fields: Vec<String>,
    /// Compared output identities or occurrences.
    pub outputs: Vec<String>,
    #[serde(default)]
    /// Outputs compare only at this cut, or every cut when absent.
    pub output_cut: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(super) enum PlayedWindow {
    FightPicture { appends: [DoorAppend; 2] },
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
        let component = |part: &str| fixture_component(part) && !part.contains('\0');
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
    pub(super) fn door_slots(&self) -> Option<&[u8]> {
        self.door_slots.as_deref()
    }

    fn valid(&self) -> bool {
        let revision_valid = if self.provider == InputProvider::EarlyMapInputs {
            self.repository
                .as_deref()
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
            Self::PlayedCeresInputs
                | Self::ScopedCeresFlyby
                | Self::ScopedCeresFightPicture
                | Self::ScopedCeresWarning
                | Self::PlayedShaftSound
                | Self::ShaftPictures
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
    pub qualifications: Vec<String>,
    #[serde(default, deserialize_with = "super::borrowed::read")]
    /// Explicit external dependency declarations.
    pub borrowed: Vec<Borrowed>,
    #[serde(default)]
    /// Prove complete saved continuations through the horizon.
    pub checkpoints: bool,
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
        let declarations = doc.native_packet.unwrap_or_default();
        let mut kinds = std::collections::HashSet::new();
        for packet in &declarations {
            if !packet.valid() || !kinds.insert(std::mem::discriminant(packet)) {
                return Err("invalid or duplicate native packet");
            }
        }
        let shared_witnesses = doc
            .played_witnesses
            .map(|data| Witnesses::resolve(Some(data)))
            .transpose()?;
        for case in &mut doc.cases {
            if case.inputs.uses_played_witnesses() {
                case.played_witnesses =
                    Some(match (case.played_witnesses.take(), &shared_witnesses) {
                        (None, Some(shared)) => shared.clone(),
                        (declared, _) => Witnesses::resolve(declared)?,
                    });
            }
            if case.native_packet.is_none() {
                case.native_packet = declarations
                    .iter()
                    .find(|packet| packet.applies_to(case))
                    .cloned();
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
    /// Selected case absent from the index.
    UnknownCase(CaseId),
    /// Repeated selected case.
    DuplicateSelection(CaseId),
}

fn scope_valid(case: &Case) -> bool {
    let owners: BTreeSet<_> = case.responsibilities.iter().collect();
    owners.len() == case.responsibilities.len()
        && case.expectations.output_cut.is_none_or(|at| {
            case.first_cut().is_some_and(|first| at >= first)
                && at <= case.end.cut_inclusive
                && !case.expectations.outputs.is_empty()
        })
        && super::recorded::Witness::valid_for(case)
        && case
            .connected_packet
            .as_ref()
            .is_none_or(|packet| packet.admits(case))
        && case
            .save_steps
            .as_ref()
            .is_none_or(|schedule| schedule.valid(case))
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
                    InputProvider::PlayedShaftSound | InputProvider::ShaftPictures
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
                && case.health_packet().is_ok()
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
        } else {
            None
        }
    }

    /// Whether the case declares `flag`, after checking that its reader knows every name the
    /// case declares.
    ///
    /// # Errors
    /// A declared field or output missing from `known` is a data failure naming it, so a
    /// misspelled flag fails the case instead of reading as undeclared.
    pub(super) fn declares(
        &self,
        known: &[&str],
        flag: &str,
    ) -> Result<bool, super::result::Outcome> {
        let mut declared = self
            .expectations
            .fields
            .iter()
            .chain(&self.expectations.outputs);
        if let Some(unknown) = declared
            .clone()
            .find(|name| !known.contains(&name.as_str()))
        {
            return Err(super::data_failure(
                None,
                &format!("declared field {unknown} is unknown to its reader"),
            ));
        }
        Ok(declared.any(|name| name == flag))
    }

    pub(super) fn fixture_pair(&self) -> Result<&FixturePair, super::result::Outcome> {
        self.fixture
            .as_ref()
            .filter(|pair| pair.provider == self.inputs && pair.valid())
            .ok_or_else(|| super::data_failure(None, "scoped provider fixture"))
    }

    /// Validate the complete public execution recipe before source admission.
    ///
    /// # Errors
    /// Invalid starts, schedules, attachments and borrowed inputs are rejected.
    pub fn validate(&self) -> Result<(), IndexFault> {
        let case = self;
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
                    PlayedWindow::Flyby { .. } => case.inputs == InputProvider::ScopedCeresFlyby,
                })
            || (case.connected_packet.is_some()
                && case.inputs != InputProvider::ConnectedOrdinaryInputs)
            || (case.save_steps.is_some() && case.inputs != InputProvider::SaveFileBoundaries)
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
            Start::NativeSeed { .. } => Some(0),
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
        if self.format != CURRENT_FORMAT
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
    fn selections_take_every_case_and_refuse_empty_duplicate_or_unknown_ones() {
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
            assert!(!Index::try_from(doc).is_ok_and(|index| index.validate().is_ok()));
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
        case.end.cut_inclusive = case.connected_packet.as_ref().unwrap().first_end;
        assert!(case.validate().is_ok());
        case.row = "another-row".to_owned();
        assert!(case.validate().is_err());
        case.row = case.connected_packet.as_ref().unwrap().row.clone();
        case.connected_packet.as_mut().unwrap().directory = "../outside".to_owned();
        assert!(case.validate().is_err());
        case.connected_packet = None;
        assert!(case.validate().is_err());
    }

    #[test]
    fn unknown_keys_are_rejected() {
        assert!(matches!(
            Index::parse(&format!("extra = true\n{INDEX}")),
            Err(IndexFault::Syntax)
        ));
    }

    #[test]
    fn public_recipe_cannot_borrow_undeclared_inputs_or_start_at_its_end() {
        let index = Index::parse(INDEX).unwrap();
        let mut boundary = index
            .cases
            .iter()
            .find(|c| c.inputs == super::InputProvider::Unavailable)
            .unwrap()
            .clone();
        boundary.borrowed.push(super::Borrowed {
            dependency: "control".into(),
            provider: "missing-provider".into(),
            fields: vec!["invented-input".into()],
            phase: "before".into(),
            claim: "observed".into(),
        });
        assert!(matches!(
            boundary.validate(),
            Err(IndexFault::InvalidCase(_))
        ));
        let mut case = index.cases.first().unwrap().clone();
        case.start = super::Start::CacheCut {
            frame: case.end.cut_inclusive,
        };
        assert!(matches!(case.validate(), Err(IndexFault::InvalidCase(_))));
    }

    #[test]
    fn a_misspelled_flag_name_fails_the_reader_that_checks_it() {
        use crate::replay::{blackout, movement, played::indexed, result::Outcome};
        let index = Index::parse(INDEX).unwrap();
        for (id, known, flag) in [
            (
                "blackout/pending-room-publication",
                &blackout::NAMES[..],
                blackout::MEMORY,
            ),
            (
                "ridley/alternate-offscreen-retreat",
                &indexed::TAIL_NAMES[..],
                indexed::RETREAT_BODY,
            ),
            (
                "movement/escape-leap-landing",
                &movement::NAMES[..],
                movement::LANDING_FLOOR,
            ),
        ] {
            let mut case = index
                .cases
                .iter()
                .find(|case| case.id.0 == id)
                .unwrap()
                .clone();
            assert_eq!(case.declares(known, flag), Ok(true));
            let expectations = &mut case.expectations;
            for name in expectations
                .fields
                .iter_mut()
                .chain(&mut expectations.outputs)
            {
                if name == flag {
                    name.push('s');
                }
            }
            assert_eq!(
                case.declares(known, flag),
                Err(Outcome::DataFailure {
                    cut: None,
                    source: format!("declared field {flag}s is unknown to its reader"),
                })
            );
        }
    }
}
