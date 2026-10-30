//! Closed room-entry effects supplied by the compatibility runtime.
use super::{
    api::EntryWait,
    content::{
        AssetSource, BackgroundDestination, EntryContent, ObjectCursor, SetupAction,
        VramDestination,
    },
};

pub trait EntryHost:
    crate::visual_host::Host
    + crate::enemy_projectiles::host::Host
    + crate::player_projectiles::host::Host
    + EntryContent
    + EntryEffects
{
}
pub trait EntryEffects {
    fn read_external(&mut self, field: ExternalField) -> u16;
    fn write_external(&mut self, field: ExternalField, value: u16);
    fn read_camera(&mut self, field: CameraField) -> u16;
    fn write_camera(&mut self, field: CameraField, value: u16);
    fn call(&mut self, effect: LeafEffect);
    fn camera_call(&mut self, effect: CameraEffect) -> u8;
    fn palette_fade(&mut self) -> u8;
    fn wait(&mut self, cause: EntryWait) -> u8;
    fn sfx_empty(&mut self, queue: SfxQueue) -> bool;
    fn queued_music(&mut self) -> u8;
    fn samus_command(&mut self, command: u16);
    fn setup(&mut self, action: SetupAction);
    fn spawn_room_object(&mut self, cursor: &ObjectCursor);
    fn spawn_initial_hardcoded_object(&mut self);
    fn close_door(&mut self);
    fn write_scroll_byte(&mut self, cell: u8, value: u8);
    fn write_scroll_pair(&mut self, byte_offset: u16, value: [u8; 2]);
    fn device_word(&mut self, register: DeviceWord, value: u16);
    fn device_byte(&mut self, register: DeviceByte, value: u8);
    fn device_vram_address(&mut self, destination: VramDestination);
    fn decompress(&mut self, source: AssetSource, destination: InstallBuffer);
    fn decompress_vram(&mut self, source: AssetSource, destination: VramDecodeTarget);
    fn copy_vram(&mut self, destination: VramDestination, source: AssetSource, bytes: u16);
    fn clear_buffer(&mut self, operation: BufferClear);
    fn decode_level_and_copy_planes(&mut self, source: AssetSource);
    fn invalid_stage(&mut self, full_address: u32) -> u8;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExternalField {
    GameMode,
    DebugMinimap,
    TimerStatus,
    PreviousPlayerX,
    PreviousPlayerY,
    MusicDataIndex,
    MusicTrackIndex,
    HdmaEnabled,
    NextIrqHandler,
    FxTilemap,
    BackgroundCopyByte,
    DisplayAndObjectSelect,
    LoadingLowByte,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CameraField {
    Layer1X,
    Layer1Y,
    Layer2X,
    Layer2Y,
    Bg1XOffset,
    Bg1YOffset,
    Bg2XScroll,
    Bg2YScroll,
    Bg1Hofs,
    Bg1Vofs,
    Bg2Hofs,
    Bg2Vofs,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CameraEffect {
    CalculateLayer2X,
    CalculateLayer2Y,
    CalculateLayer2AndScrolls,
    CalculateBgScrolls,
    DisplayVisibleRoom,
    FixDoorMovingUp,
    SetupDoorScroll,
    StepDoorScrollRight,
    StepDoorScrollLeft,
    StepDoorScrollDown,
    StepDoorScrollUp,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SfxQueue {
    One,
    Two,
    Three,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeafEffect {
    ResetSoundQueues,
    ClearTransitionSounds,
    QueueDoorSound,
    SelectEnemies,
    RunEnemies,
    DrawScene,
    EnsureSamusDrawn,
    QueueEnemyTiles,
    DrawTimer,
    SpinJumpSound,
    SetLiquidPhysics,
    StopHdmaEffects,
    InitializeRoomSpecialEffects,
    LoadFxHeader,
    InitializeEnemies,
    LoadEnemyGraphics,
    LoadRoomMusic,
    UpdateMusicTrack,
    LoadChangedMusic,
    UpdateBeamTilesAndPalette,
    LoadSpritePalettes,
    LoadRoomAssets,
    LoadSuitTargetPalette,
    SpawnBg3ScrollHdma,
    SaveExploredMap,
    LoadExploredMap,
    MarkElevatorsUsed,
    EnableNmi,
    DisableNmi,
    EnableIrq,
    DisableIrq,
    WaitVblankClearHdma,
    WaitVblankEnableIrq,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceWord {
    VramIncrement,
    Dma1Mode,
    Dma1SourceLow,
    Dma1SourceBank,
    Dma1Count,
    DmaEnable,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceByte {
    VramIncrement,
    DmaEnable,
}
#[derive(Clone, Debug, PartialEq)]
pub enum InstallBuffer {
    Tiles,
    CreTiles,
    Palette,
    TileTablePrimary,
    TileTableAfterCre,
    Background(BackgroundDestination),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BufferClear {
    InitialLevel,
    DoorLevel,
    InitialFx,
    InitialBg2,
    DoorFx,
    DoorBg2,
    DoorDualBg,
    TransitionXray,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VramSlot {
    MainTiles,
    TilesSecond,
    TilesThird,
    CreFirst,
    CreSecond,
    BackgroundFirst,
    BackgroundSecond,
    Fx,
    FxTilemap,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VramDecodeTarget {
    MainTiles,
    Cre,
}
pub use crate::room_entry::AssetSlot;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetRead {
    ThreeLittleEndian,
    FourNativeMasked,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StagingSource {
    TilesFirst,
    TilesSecond,
    TilesThird,
    CreFirst,
    CreSecond,
    Background,
}
