//! Scheduled door-transition coordinator from pinned recovered C bank 82.
//! Generic room, actor, hardware and IRQ operations remain named Host services.
mod background;
mod memory;
mod scheduler;

pub use memory::WRAM_SIZE;
type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Entry {
    HitDoorBlock = 0,
    LoadingNextRoom,
    LoadingNextRoomAsync,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The exact recovered-C coroutine byte: zero completes this invocation;
    /// nonzero commits the continuation and yields to the native scheduler.
    Handled(u8),
    Refused,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Routine {
    HitDoorBlock = 0,
    LoadingNextRoom,
    LoadingNextRoomAsync,
    Dispatch,
    HandleElevator,
    Wait48Frames,
    WaitForSounds,
    FadeOut,
    LoadDoorHeader,
    AlignScroll,
    FixDoorsMovingUp,
    SetupNewRoom,
    SetupScrolling,
    PlaceSamusLoadTiles,
    LoadMoreThings,
    HandleAnimTiles,
    WaitForMusic,
    HandleTransition,
    FadeInAndFinish,
    BackgroundFinish,
    BackgroundTransfer,
    BackgroundDecompress,
    BackgroundClearFx,
    BackgroundTransferAndSetBg3,
    BackgroundClearBg2,
    BackgroundClearKraidBg2,
    BackgroundDoorTransfer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Observation {
    Enter { routine: Routine, operand: u16 },
    Return { routine: Routine, result: u16 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyncService {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoroutineService {
    WaitForNmi,
    PlmHandler,
}

/// Every gameplay operation exchanges current WRAM synchronously; it may enter
/// installed child gameplay through the admitted native nesting contract.
/// Observation alone is passive: no RAM synchronization or gameplay dispatch.
/// Any failure after an observation/request is owned and prohibits C retry.
pub trait Host {
    fn sync(&mut self, ram: &mut [u8], service: SyncService) -> Result<()>;
    fn palette_fade(&mut self, ram: &mut [u8]) -> Result<u8>;
    fn has_queued_music(&mut self, ram: &mut [u8]) -> Result<u8>;
    fn call_samus_code(&mut self, ram: &mut [u8], code: u16) -> Result<u16>;
    fn coroutine(&mut self, ram: &mut [u8], service: CoroutineService) -> Result<u8>;
    fn observe(&mut self, observation: Observation) -> Result<()>;
}

/// Run one actual scheduled root. The native scheduler owns its state-0 latch;
/// it can resume this root even after the visible game-state word has changed.
/// No mutable continuation or borrowed state survives this function's return.
pub fn run(entry: Entry, ram: &mut [u8], rom: &[u8], host: &mut impl Host) -> Result<Outcome> {
    let ram = memory::Wram::new(ram)?;
    // The native loader mirrors the 3 MiB cartridge to a 4 MiB power-of-two
    // view. Require the complete image, while allowing that native expansion.
    if rom.len() < 0x300000 {
        return Err("room transition requires the complete native ROM view".into());
    }
    if entry != Entry::LoadingNextRoom {
        if !scheduler::PHASES.contains(&ram.word(0x99c)) {
            return Ok(Outcome::Refused);
        }
        if ram.word(0x99c) == 0xe4a9 && ram.0[0x612] > 2 {
            return Err("invalid room-transition continuation position".into());
        }
    }
    let mut coordinator = scheduler::Coordinator { ram, rom: memory::Rom(rom), host };
    Ok(Outcome::Handled(coordinator.root(entry)?))
}

#[cfg(test)]
mod tests;
