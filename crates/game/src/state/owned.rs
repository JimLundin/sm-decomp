//! Complete clean model values carried by the production entries.

use canonical::Canonical;
use canonical::model::{Model, Pinned};

/// The complete movement state schema.
pub struct MovementPin;
impl Pinned for MovementPin {
    type State = samus_movement::Samus;
    const NAME: &'static str = "production-movement";
    const SHAPE: u64 = 0x873B_3B46_1A4F_1747;
}

/// The complete transport state schema, including unfinished work.
pub struct AudioPin;
impl Pinned for AudioPin {
    type State = Box<audio_transport::State>;
    const NAME: &'static str = "production-audio";
    const SHAPE: u64 = 0x8296_0EDF_19A5_1FC5;
    const TRACE_U8_SAMPLES: &'static [(&'static str, u8)] = &[("MusicalPitch", 19)];
}

/// The full ordinary power state, including fractions, trails and spacing.
pub struct PowerPin;
impl Pinned for PowerPin {
    type State = Box<weapons_projectiles::State>;
    const NAME: &'static str = "weapons-projectiles";
    const SHAPE: u64 = 0x98F6_AF73_265D_2556;
}

/// The complete clean following-camera state and retained height receipt.
pub struct CameraPin;
impl Pinned for CameraPin {
    type State = world_camera::Camera;
    const NAME: &'static str = "world-camera";
    const SHAPE: u64 = 0x1ED5_DF03_BD0A_7BD4;
}

/// The complete clean animation state, including its playhead and boost laps.
pub struct AppearancePin;
impl Pinned for AppearancePin {
    type State = samus_appearance::State;
    const NAME: &'static str = "samus-appearance";
    const SHAPE: u64 = 0x347B_D040_BE28_1439;
}

/// The complete pending and committed CPU sender service schema.
pub struct SenderPin;
impl Pinned for SenderPin {
    type State = Box<audio_transport::SenderState>;
    const NAME: &'static str = "production-audio-sender";
    const SHAPE: u64 = 0xB596_22C8_9169_DF68;
}

/// One writable owner for every selected model quantity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OwnedState {
    /// Complete connected current owners and semantic continuation.
    Connected(Box<crate::production::connected::State>),
    /// Complete retained frame caller, selected producers and physical continuation.
    Frame(Box<crate::production::frame::State>),
    /// Bounded Frame acknowledgment and Music continuation.
    FrameOpening(Box<crate::production::frame::opening_prelude::Opening>),
    /// Complete Health and qualified late consumer continuation.
    HealthTail(Box<crate::production::health_tail::State>),
    /// Motion, semantic pose and movement memory.
    Movement(Model<MovementPin>),
    /// Current Movement, Figure and Contact sender at an ordinary opportunity.
    MovementTouchdown(Box<crate::production::movement_touchdown::State>),
    /// Complete owners connected by held movement position.
    MovementCeres(Box<crate::production::MovementCeresState>),
    /// Committed and pending transport quantities.
    Audio(Model<AudioPin>),
    /// Resident transport, all CPU senders and the measured clock relationship.
    ResidentAudio(Box<crate::production::audio::State>),
    /// Current power shots and late contact consumption.
    WeaponsContact(Model<PowerPin>),
    /// Current ordinary weapon, door, Movement, Camera and drawing owners.
    ShotOpportunity(Box<crate::production::frame::shot_opportunity::Continuation>),
    /// Complete room tiles, running places and scroll screens.
    Blocks(Box<world_blocks::State>),
    /// Following view, remembered centre and height receipt.
    Camera(Model<CameraPin>),
    /// The animation playhead, boost stage and laps.
    Appearance(Model<AppearancePin>),
    /// Complete inventory, events and remembered locations/maps/stations.
    Progress(Box<run_progress::Progress>),
    /// Complete HUD grid, shown values, selection and explored mini-map.
    Hud(Box<interface_hud::HudState>),
    /// A complete Ceres door phase, wait and immutable placement.
    Ceres(boss_ceres_ridley::CeresDoor),
    /// Complete retained passage and scene.
    Doors(Box<crate::production::PassageState>),
    /// First passage entry with its audio caller.
    PassageAudio(Box<crate::production::passage_audio::State>),
    /// One sender group's committed and pending CPU service.
    AudioSender(Model<SenderPin>),
}

/// Codec-only values created at the save edge; these never own live gameplay.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub(crate) enum CodecState {
    Connected(Box<crate::production::connected::State>),
    Movement(Model<MovementPin>),
    // Keep the shared codec shape and its other scope tags stable. Standalone Audio has its own codec.
    Audio(Model<AudioPin>),
    WeaponsContact(Model<PowerPin>),
    Camera(Model<CameraPin>),
    Appearance(Model<AppearancePin>),
    AudioSender(Model<SenderPin>),
    Frame(Box<crate::production::frame::State>),
}
