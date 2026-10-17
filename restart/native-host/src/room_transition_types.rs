//! Fixed types shared by the scheduled transition interface and its evidence.
#[cfg(feature = "legacy-evidence")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
#[repr(u32)]
pub enum RoomTransitionEntry {
    HitDoorBlock = 0,
    LoadingNextRoom = 1,
    LoadingNextRoomAsync = 2,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
#[repr(u32)]
pub enum RoomTransitionRoutine {
    HitDoorBlock = 0,
    LoadingNextRoom = 1,
    LoadingNextRoomAsync = 2,
    Dispatch = 3,
    HandleElevator = 4,
    Wait48Frames = 5,
    WaitForSounds = 6,
    FadeOut = 7,
    LoadDoorHeader = 8,
    AlignScroll = 9,
    FixDoorsMovingUp = 10,
    SetupNewRoom = 11,
    SetupScrolling = 12,
    PlaceSamusLoadTiles = 13,
    LoadMoreThings = 14,
    HandleAnimTiles = 15,
    WaitForMusic = 16,
    HandleTransition = 17,
    FadeInAndFinish = 18,
    BackgroundFinish = 19,
    BackgroundTransfer = 20,
    BackgroundDecompress = 21,
    BackgroundClearFx = 22,
    BackgroundTransferAndSetBg3 = 23,
    BackgroundClearBg2 = 24,
    BackgroundClearKraidBg2 = 25,
    BackgroundDoorTransfer = 26,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomTransitionOutcome {
    Handled(u8),
    Refused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomTransitionObservation {
    Enter { routine: RoomTransitionRoutine, operand: u16 },
    Return { routine: RoomTransitionRoutine, result: u16 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomTransitionCoroutineService {
    WaitForNmi,
    PlmHandler,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomTransitionSyncService {
    DetermineWhichEnemiesToProcess,
    EnemyMain,
    DrawSamusEnemiesAndProjectiles,
    EnsureSamusDrawnEachFrame,
    LoadDestinationRoomCreBitset,
    DrawTimer,
    ClearSoundsWhenGoingThroughDoor,
    QueueSfx2Max15 { sound: u16 },
    LoadDoorHeader,
    Sub8882Ac,
    CalculateLayer2PosAndScrollsWhenScrolling,
    FixDoorsMovingUp,
    SaveMapExploredIfElevator,
    LoadRoomHeader,
    LoadStateHeader,
    LoadMapExploredIfElevator,
    InitializeSpecialEffectsForNewRoom,
    LoadLevelScrollAndCre,
    DoorTransitionScrollingSetup,
    WaitUntilEndOfVblankAndEnableIrq,
    DecompressToMem { source: u32, destination: u16 },
    CopyToVramNow { vram_dst: u16, source: u32, size: u16 },
    LoadEnemyGfxToVram,
    LoadRoomMusic,
    ClearEprojs,
    ClearAnimtiles,
    ClearPaletteFxObjects,
    ClearPlms,
    CreatePlmsExecuteDoorAsmRoomSetup,
    LoadFxHeader,
    SpawnDoorClosingPlm,
    UpdateBeamTilesAndPalette,
    LoadColorsForSpritesBeamsAndEnemies,
    LoadEnemies,
    InitializeEnemies,
    ResetProjectileData,
    SamusLoadSuitTargetPalette,
    ClearFxTilemap,
    SpawnBg3ScrollHdmaObject,
    ClearBg2Tilemap,
    ClearKraidBg2Tilemap,
    AnimtilesHandler,
    LoadNewMusicTrackIfChanged,
    PointlessFunctionStupidToo,
    SetLiquidPhysicsType,
    EprojRunAll,
    QueueEnemyBg2TilemapTransfers,
    PlaySpinJumpSoundIfSpinJumping,
}
impl TryFrom<u32> for RoomTransitionRoutine {
    type Error = String;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        use RoomTransitionRoutine::*;
        match value {
            0 => Ok(HitDoorBlock),
            1 => Ok(LoadingNextRoom),
            2 => Ok(LoadingNextRoomAsync),
            3 => Ok(Dispatch),
            4 => Ok(HandleElevator),
            5 => Ok(Wait48Frames),
            6 => Ok(WaitForSounds),
            7 => Ok(FadeOut),
            8 => Ok(LoadDoorHeader),
            9 => Ok(AlignScroll),
            10 => Ok(FixDoorsMovingUp),
            11 => Ok(SetupNewRoom),
            12 => Ok(SetupScrolling),
            13 => Ok(PlaceSamusLoadTiles),
            14 => Ok(LoadMoreThings),
            15 => Ok(HandleAnimTiles),
            16 => Ok(WaitForMusic),
            17 => Ok(HandleTransition),
            18 => Ok(FadeInAndFinish),
            19 => Ok(BackgroundFinish),
            20 => Ok(BackgroundTransfer),
            21 => Ok(BackgroundDecompress),
            22 => Ok(BackgroundClearFx),
            23 => Ok(BackgroundTransferAndSetBg3),
            24 => Ok(BackgroundClearBg2),
            25 => Ok(BackgroundClearKraidBg2),
            26 => Ok(BackgroundDoorTransfer),
            _ => Err("invalid room-transition routine ID".into()),
        }
    }
}

impl RoomTransitionSyncService {
    pub(crate) fn encode(self) -> (u32, [u32; 3]) {
        use RoomTransitionSyncService::*;
        match self {
            DetermineWhichEnemiesToProcess => (0, [0; 3]),
            EnemyMain => (1, [0; 3]),
            DrawSamusEnemiesAndProjectiles => (2, [0; 3]),
            EnsureSamusDrawnEachFrame => (3, [0; 3]),
            LoadDestinationRoomCreBitset => (4, [0; 3]),
            DrawTimer => (5, [0; 3]),
            ClearSoundsWhenGoingThroughDoor => (6, [0; 3]),
            QueueSfx2Max15 { sound } => (7, [sound.into(), 0, 0]),
            LoadDoorHeader => (8, [0; 3]),
            Sub8882Ac => (9, [0; 3]),
            CalculateLayer2PosAndScrollsWhenScrolling => (10, [0; 3]),
            FixDoorsMovingUp => (11, [0; 3]),
            SaveMapExploredIfElevator => (12, [0; 3]),
            LoadRoomHeader => (13, [0; 3]),
            LoadStateHeader => (14, [0; 3]),
            LoadMapExploredIfElevator => (15, [0; 3]),
            InitializeSpecialEffectsForNewRoom => (16, [0; 3]),
            LoadLevelScrollAndCre => (17, [0; 3]),
            DoorTransitionScrollingSetup => (18, [0; 3]),
            WaitUntilEndOfVblankAndEnableIrq => (19, [0; 3]),
            DecompressToMem { source, destination } => (20, [source, destination.into(), 0]),
            CopyToVramNow { vram_dst, source, size } => (21, [vram_dst.into(), source, size.into()]),
            LoadEnemyGfxToVram => (22, [0; 3]),
            LoadRoomMusic => (23, [0; 3]),
            ClearEprojs => (24, [0; 3]),
            ClearAnimtiles => (25, [0; 3]),
            ClearPaletteFxObjects => (26, [0; 3]),
            ClearPlms => (27, [0; 3]),
            CreatePlmsExecuteDoorAsmRoomSetup => (28, [0; 3]),
            LoadFxHeader => (29, [0; 3]),
            SpawnDoorClosingPlm => (30, [0; 3]),
            UpdateBeamTilesAndPalette => (31, [0; 3]),
            LoadColorsForSpritesBeamsAndEnemies => (32, [0; 3]),
            LoadEnemies => (33, [0; 3]),
            InitializeEnemies => (34, [0; 3]),
            ResetProjectileData => (35, [0; 3]),
            SamusLoadSuitTargetPalette => (36, [0; 3]),
            ClearFxTilemap => (37, [0; 3]),
            SpawnBg3ScrollHdmaObject => (38, [0; 3]),
            ClearBg2Tilemap => (39, [0; 3]),
            ClearKraidBg2Tilemap => (40, [0; 3]),
            AnimtilesHandler => (41, [0; 3]),
            LoadNewMusicTrackIfChanged => (42, [0; 3]),
            PointlessFunctionStupidToo => (43, [0; 3]),
            SetLiquidPhysicsType => (44, [0; 3]),
            EprojRunAll => (45, [0; 3]),
            QueueEnemyBg2TilemapTransfers => (46, [0; 3]),
            PlaySpinJumpSoundIfSpinJumping => (47, [0; 3]),
        }
    }
}

/// Actual root/routine/service work; ownership is kept outside behavior effects.
/// Child arrays count Rust callback attempts/handled completions inside a named
/// parent service; columns are projectile, scrolling, then Ceres door.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
pub struct RoomTransitionCounters {
    pub c_entries: [u64; 3],
    pub rust_entries: [u64; 3],
    pub rust_completed: [u64; 3],
    pub rust_refused: [u64; 3],
    pub root_pending: [u64; 3],
    pub root_resumed: [u64; 3],
    pub routine_c_entries: [u64; 27],
    pub routine_c_completed: [u64; 27],
    pub routine_rust_entries: [u64; 27],
    pub routine_rust_completed: [u64; 27],
    #[cfg_attr(feature = "legacy-evidence", serde(with = "fixed_array"))]
    pub service_requests: [u64; 53],
    #[cfg_attr(feature = "legacy-evidence", serde(with = "fixed_array"))]
    pub service_completions: [u64; 53],
    #[cfg_attr(feature = "legacy-evidence", serde(with = "fixed_array"))]
    pub foreign_requests: [u64; 53],
    #[cfg_attr(feature = "legacy-evidence", serde(with = "fixed_array"))]
    pub foreign_completions: [u64; 53],
    #[cfg_attr(feature = "legacy-evidence", serde(with = "fixed_array"))]
    pub service_pending: [u64; 53],
    #[cfg_attr(feature = "legacy-evidence", serde(with = "fixed_array"))]
    pub child_entries: [[u64; 3]; 53],
    #[cfg_attr(feature = "legacy-evidence", serde(with = "fixed_array"))]
    pub child_completions: [[u64; 3]; 53],
}
impl Default for RoomTransitionCounters {
    fn default() -> Self {
        Self {
            c_entries: [0; 3],
            rust_entries: [0; 3],
            rust_completed: [0; 3],
            rust_refused: [0; 3],
            root_pending: [0; 3],
            root_resumed: [0; 3],
            routine_c_entries: [0; 27],
            routine_c_completed: [0; 27],
            routine_rust_entries: [0; 27],
            routine_rust_completed: [0; 27],
            service_requests: [0; 53],
            service_completions: [0; 53],
            foreign_requests: [0; 53],
            foreign_completions: [0; 53],
            service_pending: [0; 53],
            child_entries: [[0; 3]; 53],
            child_completions: [[0; 3]; 53],
        }
    }
}

#[cfg(feature = "legacy-evidence")]
mod fixed_array {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    pub fn serialize<T: Serialize, S: Serializer, const N: usize>(v: &[T; N], s: S) -> Result<S::Ok, S::Error> {
        v.as_slice().serialize(s)
    }
    pub fn deserialize<'de, T: Deserialize<'de>, D: Deserializer<'de>, const N: usize>(
        d: D,
    ) -> Result<[T; N], D::Error> {
        Vec::<T>::deserialize(d)?
            .try_into()
            .map_err(|v: Vec<T>| serde::de::Error::custom(format!("expected {N} counters, got {}", v.len())))
    }
}

#[cfg(all(test, feature = "legacy-evidence"))]
mod tests {
    use super::*;

    #[test]
    fn counter_transport_retains_last_service_and_child_and_rejects_short_arrays() {
        let mut counts = RoomTransitionCounters::default();
        counts.foreign_completions[52] = 17;
        counts.child_completions[52][2] = 29;
        let mut encoded = serde_json::to_value(counts).unwrap();
        assert_eq!(serde_json::from_value::<RoomTransitionCounters>(encoded.clone()).unwrap(), counts);
        encoded["service_requests"].as_array_mut().unwrap().pop();
        assert!(serde_json::from_value::<RoomTransitionCounters>(encoded).is_err());
    }
}
