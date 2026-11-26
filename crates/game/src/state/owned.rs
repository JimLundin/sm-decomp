//! Complete clean model values carried by the production entries.

use canonical::Canonical;
use canonical::model::{Model, Pinned};

/// The complete movement state schema.
pub struct MovementPin;
impl Pinned for MovementPin {
    type State = samus_movement::Samus;
    const NAME: &'static str = "production-movement";
    const SHAPE: u64 = 0xCB92_9AE7_D54A_32B2;
}

/// The complete transport state schema: senders, request lines, groups and song.
pub struct AudioPin;
impl Pinned for AudioPin {
    type State = Box<audio_transport::State>;
    const NAME: &'static str = "production-audio";
    const SHAPE: u64 = 0xF5B0_435C_9274_A3C7;
    const TRACE_U8_SAMPLES: &'static [(&'static str, u8)] = &[("MusicalPitch", 19)];
}

/// The full ordinary power state, including fractions, trails and spacing.
pub struct PowerPin;
impl Pinned for PowerPin {
    type State = Box<weapons_projectiles::State>;
    const NAME: &'static str = "weapons-projectiles";
    const SHAPE: u64 = 0x04F9_034D_ACF0_8659;
}

/// The complete clean following-camera state and retained height receipt.
pub struct CameraPin;
impl Pinned for CameraPin {
    type State = world_camera::Camera;
    const NAME: &'static str = "world-camera";
    const SHAPE: u64 = 0x1ED5_DF03_BD0A_7BD4;
}

/// Her complete appearance State: her animation, kept action, cannon delay and colour sequences.
pub struct AppearancePin;
impl Pinned for AppearancePin {
    type State = samus_appearance::AppearanceState;
    const NAME: &'static str = "samus-appearance";
    const SHAPE: u64 = 0xB2DA_FEFF_DE5F_DB9F;
}

/// Complete Zebes flashback encounter State.
pub struct FlashbackPin;
impl Pinned for FlashbackPin {
    type State = Box<types::flashback::EncounterState>;
    const NAME: &'static str = "screens-flashback";
    const SHAPE: u64 = 0x384F_FECC_395A_C5CA;
}

/// The complete carried phase and last sample of the recorded pad.
pub struct PlaybackPin;
impl Pinned for PlaybackPin {
    type State = types::playback::ScenePhase;
    const NAME: &'static str = "screens-playback";
    const SHAPE: u64 = 0x8098_1B49_53DC_58B7;
}

/// One writable owner for every selected model quantity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OwnedState {
    /// Complete intro State and the displayed palette.
    Intro(Box<crate::production::played::intro::State>),
    /// The intro room join and its first-turn actors.
    IntroRooms(Box<crate::production::played::intro_rooms::Scoped>),
    /// The recorded pad and its current position in the scene.
    Playback(Model<PlaybackPin>),
    /// The whole cartridge and its pending operation.
    SaveFiles(Box<crate::production::save_files::State>),
    /// Complete connected current owners and semantic continuation.
    Connected(Box<crate::production::connected::State>),
    /// Every actor, pending timer and background offset of the flashback encounter.
    Flashback(Model<FlashbackPin>),
    /// Complete Health and qualified late consumer continuation.
    HealthTail(Box<crate::production::health_tail::State>),
    /// Motion, semantic pose and movement memory.
    Movement(Model<MovementPin>),
    /// Current Movement, Figure and audio transport at an ordinary opportunity.
    MovementTouchdown(Box<crate::production::movement_touchdown::State>),
    /// The complete audio transport between frames.
    Audio(Model<AudioPin>),
    /// Current power shots and late contact consumption.
    WeaponsContact(Model<PowerPin>),
    /// Complete cinematic weapon State and pending drawing.
    IntroMissiles(Box<crate::production::intro_missile_scope::State>),
    /// Complete room tiles, running places and scroll screens.
    Blocks(Box<world_blocks::State>),
    /// Following view, remembered centre and height receipt.
    Camera(Model<CameraPin>),
    /// Her animation, kept action, cannon delay and colour sequences.
    Appearance(Model<AppearancePin>),
    /// Complete inventory, events and remembered locations/maps/stations.
    Progress(Box<run_progress::Progress>),
    /// Complete HUD grid, shown values, selection and explored mini-map.
    Hud(Box<interface_hud::HudState>),
    /// A complete Ceres door phase, wait and immutable placement.
    Ceres(boss_ceres_ridley::CeresDoor),
    /// Complete retained passage and scene.
    Doors(Box<crate::production::PassageState>),
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
}
