//! One installation path for the playable game and the comparison runner.
use sm_native_host::{Game, HookOutcome, ProjectileEntry, ProjectileServices};

struct RipperServices<'a>(&'a mut sm_native_host::RipperServices);
impl sm_ripper::Services for RipperServices<'_> {
    fn available(&self) -> bool {
        self.0.available()
    }
    fn move_horizontal(&mut self, ram: &mut [u8], slot: u16, amount: i32) -> Result<u8, String> {
        self.0.move_horizontal(ram, slot, amount)
    }
}

pub fn install_ripper(game: &mut Game) -> Result<(), String> {
    game.set_ripper_hook(|entry, ram, rom, services| {
        let entry = match entry {
            sm_native_host::RipperEntry::JetInit => sm_ripper::Entry::JetInit,
            sm_native_host::RipperEntry::JetMain => sm_ripper::Entry::JetMain,
            sm_native_host::RipperEntry::RipperInit => sm_ripper::Entry::RipperInit,
            sm_native_host::RipperEntry::RipperMain => sm_ripper::Entry::RipperMain,
        };
        sm_ripper::execute(entry, ram, rom, &mut RipperServices(services))
    })
}

struct WaverServices<'a>(&'a mut sm_native_host::WaverServices);
impl sm_waver::Services for WaverServices<'_> {
    fn available(&self) -> bool {
        self.0.available()
    }
    fn move_horizontal(&mut self, ram: &mut [u8], k: u16, amount: i32) -> Result<u8, String> {
        self.0.move_horizontal(ram, k, amount)
    }
    fn sine(&mut self, ram: &mut [u8], angle: u16, amplitude: u16) -> Result<u16, String> {
        self.0.sine(ram, angle, amplitude)
    }
    fn move_vertical(&mut self, ram: &mut [u8], k: u16, amount: i32) -> Result<u8, String> {
        self.0.move_vertical(ram, k, amount)
    }
}

pub fn install_waver(game: &mut Game) -> Result<(), String> {
    game.set_waver_hook(|entry, k, jp, ram, rom, services| {
        let entry = match entry {
            sm_native_host::WaverEntry::Init => sm_waver::Entry::Init,
            sm_native_host::WaverEntry::Main => sm_waver::Entry::Main,
            sm_native_host::WaverEntry::Instruction => sm_waver::Entry::Instruction,
            sm_native_host::WaverEntry::SelectAnimation => sm_waver::Entry::SelectAnimation,
        };
        Ok(sm_waver::execute(entry, k, jp, ram, rom, &mut WaverServices(services))?.map(|output| match output {
            sm_waver::Output::Done => sm_native_host::WaverOutput::Done,
            sm_waver::Output::Continue(jp) => sm_native_host::WaverOutput::Continue(jp),
        }))
    })
}

struct PlayerTerrain<'a>(&'a mut sm_native_host::PlayerMotionServices);
impl sm_player_motion::Terrain for PlayerTerrain<'_> {
    fn available(&self) -> bool {
        self.0.available()
    }
    fn move_vertical(&mut self, ram: &mut [u8], amount: i32) -> Result<i32, String> {
        self.0.move_vertical(ram, amount)
    }
}

pub fn install_player_motion(game: &mut Game) -> Result<(), String> {
    game.set_player_motion_hook(|entry, amount, ram, terrain| {
        let entry = match entry {
            sm_native_host::PlayerMotionEntry::Up => sm_player_motion::Entry::MoveUp(amount),
            sm_native_host::PlayerMotionEntry::Down => sm_player_motion::Entry::MoveDown(amount),
        };
        match sm_player_motion::execute(entry, ram, &mut PlayerTerrain(terrain))? {
            Some(sm_player_motion::Output::Done) => Ok(HookOutcome::Handled),
            None => Ok(HookOutcome::Refused),
            Some(_) => Err("player root returned a helper-only output".into()),
        }
    })
}

struct ItemServices<'a>(&'a mut sm_native_host::ItemInstructionServices);
impl sm_item_instructions::Services for ItemServices<'_> {
    fn available(&self, service: sm_item_instructions::Service) -> bool {
        use sm_item_instructions::Service as S;
        use sm_native_host::ItemInstructionService as N;
        self.0.available(match service {
            S::AddMissilesToHud => N::AddMissilesToHud,
            S::PlayRoomMusicAfter => N::PlayRoomMusicAfter,
            S::QueueMusicDelayed8 => N::QueueMusicDelayed8,
            S::ProcessPlmDrawInstruction => N::ProcessPlmDrawInstruction,
            S::CalculatePlmBlockCoords => N::CalculatePlmBlockCoords,
            S::DrawPlm => N::DrawPlm,
            S::AddSuperMissilesToHud => N::AddSuperMissilesToHud,
            S::AddPowerBombsToHud => N::AddPowerBombsToHud,
        })
    }
    fn add_missiles_to_hud(&mut self, ram: &mut [u8]) -> Result<(), String> {
        self.0.add_missiles_to_hud(ram)
    }
    fn play_room_music_after(&mut self, ram: &mut [u8], frames: u16) -> Result<(), String> {
        self.0.play_room_music_after(ram, frames)
    }
    fn queue_music_delayed8(&mut self, ram: &mut [u8], track: u16) -> Result<(), String> {
        self.0.queue_music_delayed8(ram, track)
    }
    fn process_plm_draw_instruction(&mut self, ram: &mut [u8], slot: u16) -> Result<(), String> {
        self.0.process_plm_draw_instruction(ram, slot)
    }
    fn calculate_plm_block_coords(&mut self, ram: &mut [u8], slot: u16) -> Result<(), String> {
        self.0.calculate_plm_block_coords(ram, slot)
    }
    fn draw_plm(&mut self, ram: &mut [u8], slot: u16) -> Result<(), String> {
        self.0.draw_plm(ram, slot)
    }
    fn add_super_missiles_to_hud(&mut self, ram: &mut [u8]) -> Result<(), String> {
        self.0.add_super_missiles_to_hud(ram)
    }
    fn add_power_bombs_to_hud(&mut self, ram: &mut [u8]) -> Result<(), String> {
        self.0.add_power_bombs_to_hud(ram)
    }
}

pub fn install_item_instructions(game: &mut Game) -> Result<(), String> {
    install_selected_item_instructions(game, true, true, true)
}

pub fn install_selected_item_instructions(
    game: &mut Game,
    missile: bool,
    morph: bool,
    tanks: bool,
) -> Result<(), String> {
    use sm_item_instructions::{Entry, Output};
    use sm_native_host::{ItemInstructionOutcome as Outcome, ItemInstructionOutput as NativeOutput};
    game.set_item_instruction_hook(move |entry, slot, operand, ram, rom, services| {
        if ram.len() == 0x20000 && slot <= 78 && slot & 1 == 0 {
            let k = usize::from(slot);
            let header = u16::from_le_bytes([ram[0x1c37 + k], ram[0x1c38 + k]]);
            let argument = u16::from_le_bytes([ram[0x1dc7 + k], ram[0x1dc8 + k]]);
            if (!missile && header == 0xef2f && argument == 0x22)
                || (!morph && header == 0xef23 && argument == 0x1a)
                || (!tanks && matches!(header, 0xeed7 | 0xeedb | 0xeedf | 0xeee3 | 0xef27) && argument <= 0x1ff)
            {
                return Ok(Outcome::Refused);
            }
        }
        Ok(
            match sm_item_instructions::execute(
                Entry::try_from(entry as u32)?,
                slot,
                operand,
                ram,
                rom,
                &mut ItemServices(services),
            )? {
                Some(Output::PreinstructionDone) => Outcome::Handled(NativeOutput::PreinstructionDone),
                Some(Output::Stop) => Outcome::Handled(NativeOutput::Stop),
                Some(Output::Continue(offset)) => Outcome::Handled(NativeOutput::Continue(offset)),
                None => Outcome::Refused,
            },
        )
    })
}

struct Services<'a>(&'a mut ProjectileServices);
impl sm_projectile::Host for Services<'_> {
    fn spawn_plm(&mut self, ram: &mut [u8], header: u16) -> Result<u8, String> {
        self.0.spawn_plm(ram, header)
    }
}

pub fn install_projectiles(game: &mut Game, deliberate_fault: bool) -> Result<(), String> {
    install_projectile_hook(game, deliberate_fault, false)
}

/// The escape comparison uses the same gameplay implementation and admission.
/// Its fault control targets an update because the saved route contains no Fire.
pub fn install_escape_projectiles(game: &mut Game, deliberate_fault: bool) -> Result<(), String> {
    install_projectile_hook(game, false, deliberate_fault)
}

fn install_projectile_hook(game: &mut Game, fire_fault: bool, escape_fault: bool) -> Result<(), String> {
    let mut escape_fault_pending = escape_fault;
    game.set_projectile_hook(move |entry, ram, rom, services| {
        let entry = match entry {
            ProjectileEntry::FireUnchargedBeam => sm_projectile::Entry::FireUnchargedBeam,
            ProjectileEntry::HandleProjectile => sm_projectile::Entry::HandleProjectile,
            ProjectileEntry::DrawPlayerExplosions2 => sm_projectile::Entry::DrawPlayerExplosions2,
            ProjectileEntry::DrawBombAndProjectileExplosions => sm_projectile::Entry::DrawBombAndProjectileExplosions,
        };
        let outcome = sm_projectile::run(entry, ram, rom, &mut Services(services))?;
        if fire_fault
            && matches!(entry, sm_projectile::Entry::FireUnchargedBeam)
            && matches!(outcome, sm_projectile::Outcome::Handled)
        {
            // Wrong projectile X after real firing: a control of the actual
            // candidate path, not corruption of the saved reference or comparator.
            if let Some(slot) = (0..5).find(|i| u16::from_le_bytes([ram[0xc40 + i * 2], ram[0xc41 + i * 2]]) != 0) {
                ram[0xb64 + slot * 2] = ram[0xb64 + slot * 2].wrapping_add(1);
            }
        }
        if escape_fault_pending
            && matches!(entry, sm_projectile::Entry::HandleProjectile)
            && matches!(outcome, sm_projectile::Outcome::Handled)
            && u16::from_le_bytes([ram[0x93f], ram[0x940]]) & 0x8000 != 0
        {
            // An erroneous extra X update, including for an inactive slot,
            // violates the scheduler's unchanged-state contract. Unlike its
            // scratch index, this output survives downstream drawing.
            let wrong_x = u16::from_le_bytes([ram[0xb64], ram[0xb65]]).wrapping_add(1);
            ram[0xb64..0xb66].copy_from_slice(&wrong_x.to_le_bytes());
            escape_fault_pending = false;
        }
        Ok(match outcome {
            sm_projectile::Outcome::Handled => HookOutcome::Handled,
            sm_projectile::Outcome::Refused => HookOutcome::Refused,
        })
    })
}

struct ScrollServices<'a>(&'a mut sm_native_host::ScrollServices);
impl sm_room_scroll::Host for ScrollServices<'_> {
    fn flush_vram(&mut self, ram: &mut [u8]) -> Result<(), String> {
        self.0.flush_vram(ram)
    }
}

pub fn install_scrolling(game: &mut Game, deliberate_fault: bool) -> Result<(), String> {
    game.set_scroll_hook(move |entry, ram, services| {
        let entry = match entry {
            sm_native_host::ScrollEntry::DisplayViewablePartOfRoom => sm_room_scroll::Entry::DisplayViewablePartOfRoom,
            sm_native_host::ScrollEntry::UpdateScrollVarsUpdateMap => sm_room_scroll::Entry::UpdateScrollVarsUpdateMap,
            sm_native_host::ScrollEntry::CalculateLayer2PosAndScrollsWhenScrolling => {
                sm_room_scroll::Entry::CalculateLayer2PosAndScrollsWhenScrolling
            }
        };
        let outcome = sm_room_scroll::run(entry, ram, &mut ScrollServices(services))?;
        let word = |at| u16::from_le_bytes([ram[at], ram[at + 1]]);
        if deliberate_fault
            && matches!(entry, sm_room_scroll::Entry::CalculateLayer2PosAndScrollsWhenScrolling)
            && matches!(outcome, sm_room_scroll::Outcome::Handled)
            && word(0x0998) == 8
            && word(0x079f) == 6
            && word(0x079b) == 0xdf45
            && word(0x0915) > 0
        {
            // A real one-pixel scroll-register error once the camera descends.
            let wrong = word(0x00b3).wrapping_add(1);
            ram[0x00b3..0x00b5].copy_from_slice(&wrong.to_le_bytes());
        }
        Ok(match outcome {
            sm_room_scroll::Outcome::Handled => HookOutcome::Handled,
            sm_room_scroll::Outcome::Refused => HookOutcome::Refused,
        })
    })
}

struct DoorServices<'a>(&'a mut sm_native_host::CeresDoorServices);
impl sm_ceres_door::Host for DoorServices<'_> {
    fn spawn_eproj_with_room_gfx(
        &mut self,
        ram: &mut [u8],
        header: u16,
        parameter: u16,
        point: [u16; 2],
    ) -> Result<(), String> {
        self.0.spawn_eproj_with_room_gfx(ram, header, parameter, point)
    }
}

pub fn install_ceres_doors(game: &mut Game, deliberate_fault: bool) -> Result<(), String> {
    use sm_ceres_door::{Entry, Outcome, Output};
    use sm_native_host::{
        CeresDoorEntry as NativeEntry, CeresDoorOutcome as NativeOutcome, CeresDoorOutput as NativeOutput,
    };
    game.set_ceres_door_hook(move |entry, index, instruction, ram, rom, services| {
        let entry = match entry {
            NativeEntry::Init => Entry::Init,
            NativeEntry::Main => Entry::Main,
            NativeEntry::Instr1 => Entry::Instr1,
            NativeEntry::Instr2 => Entry::Instr2,
            NativeEntry::Instr3 => Entry::Instr3,
            NativeEntry::Instr4 => Entry::Instr4,
            NativeEntry::Instr5 => Entry::Instr5,
            NativeEntry::Instr6 => Entry::Instr6,
            NativeEntry::Instr7 => Entry::Instr7,
            NativeEntry::Instr8 => Entry::Instr8,
            NativeEntry::SetTangible => Entry::SetTangible,
            NativeEntry::SetInvisible => Entry::SetInvisible,
            NativeEntry::SolidSetup => Entry::SolidSetup,
        };
        let result = sm_ceres_door::run(entry, index, instruction, ram, rom, &mut DoorServices(services))?;
        if deliberate_fault
            && matches!(entry, Entry::Instr6)
            && let Outcome::Handled(Output::InstructionOffset(next)) = &result
            && *next == instruction.wrapping_add(2)
        {
            // Change the executed proximity branch to the closed-loop target.
            // The immutable C trace and comparator are untouched.
            let at = ((0xa6 & 0x7f) as usize) * 0x8000 + (instruction as usize & 0x7fff);
            let closed = u16::from_le_bytes([rom[at], rom[at + 1]]);
            return Ok(NativeOutcome::Handled(NativeOutput::InstructionOffset(closed)));
        }
        Ok(match result {
            Outcome::Refused => NativeOutcome::Refused,
            Outcome::Handled(output) => NativeOutcome::Handled(match output {
                Output::Done => NativeOutput::Done,
                Output::InstructionOffset(offset) => NativeOutput::InstructionOffset(offset),
                Output::SetupStatus(status) => NativeOutput::SetupStatus(status),
            }),
        })
    })
}

// Transition services execute synchronously against canonical native state.
// The gameplay crate owns continuation control; the adapter preserves raw results.
struct TransitionServices<'a> {
    native: &'a mut sm_native_host::RoomTransitionServices,
    false_wait_completion: &'a mut bool,
}
impl sm_room_transition::Host for TransitionServices<'_> {
    fn sync(&mut self, ram: &mut [u8], service: sm_room_transition::SyncService) -> Result<(), String> {
        use sm_native_host::RoomTransitionSyncService as N;
        use sm_room_transition::SyncService as S;
        self.native.sync(
            ram,
            match service {
                S::DetermineWhichEnemiesToProcess => N::DetermineWhichEnemiesToProcess,
                S::EnemyMain => N::EnemyMain,
                S::DrawSamusEnemiesAndProjectiles => N::DrawSamusEnemiesAndProjectiles,
                S::EnsureSamusDrawnEachFrame => N::EnsureSamusDrawnEachFrame,
                S::LoadDestinationRoomCreBitset => N::LoadDestinationRoomCreBitset,
                S::DrawTimer => N::DrawTimer,
                S::ClearSoundsWhenGoingThroughDoor => N::ClearSoundsWhenGoingThroughDoor,
                S::QueueSfx2Max15 { sound } => N::QueueSfx2Max15 { sound },
                S::LoadDoorHeader => N::LoadDoorHeader,
                S::Sub8882Ac => N::Sub8882Ac,
                S::CalculateLayer2PosAndScrollsWhenScrolling => N::CalculateLayer2PosAndScrollsWhenScrolling,
                S::FixDoorsMovingUp => N::FixDoorsMovingUp,
                S::SaveMapExploredIfElevator => N::SaveMapExploredIfElevator,
                S::LoadRoomHeader => N::LoadRoomHeader,
                S::LoadStateHeader => N::LoadStateHeader,
                S::LoadMapExploredIfElevator => N::LoadMapExploredIfElevator,
                S::InitializeSpecialEffectsForNewRoom => N::InitializeSpecialEffectsForNewRoom,
                S::LoadLevelScrollAndCre => N::LoadLevelScrollAndCre,
                S::DoorTransitionScrollingSetup => N::DoorTransitionScrollingSetup,
                S::WaitUntilEndOfVblankAndEnableIrq => N::WaitUntilEndOfVblankAndEnableIrq,
                S::DecompressToMem { source, destination } => N::DecompressToMem { source, destination },
                S::CopyToVramNow { vram_dst, source, size } => N::CopyToVramNow { vram_dst, source, size },
                S::LoadEnemyGfxToVram => N::LoadEnemyGfxToVram,
                S::LoadRoomMusic => N::LoadRoomMusic,
                S::ClearEprojs => N::ClearEprojs,
                S::ClearAnimtiles => N::ClearAnimtiles,
                S::ClearPaletteFxObjects => N::ClearPaletteFxObjects,
                S::ClearPlms => N::ClearPlms,
                S::CreatePlmsExecuteDoorAsmRoomSetup => N::CreatePlmsExecuteDoorAsmRoomSetup,
                S::LoadFxHeader => N::LoadFxHeader,
                S::SpawnDoorClosingPlm => N::SpawnDoorClosingPlm,
                S::UpdateBeamTilesAndPalette => N::UpdateBeamTilesAndPalette,
                S::LoadColorsForSpritesBeamsAndEnemies => N::LoadColorsForSpritesBeamsAndEnemies,
                S::LoadEnemies => N::LoadEnemies,
                S::InitializeEnemies => N::InitializeEnemies,
                S::ResetProjectileData => N::ResetProjectileData,
                S::SamusLoadSuitTargetPalette => N::SamusLoadSuitTargetPalette,
                S::ClearFxTilemap => N::ClearFxTilemap,
                S::SpawnBg3ScrollHdmaObject => N::SpawnBg3ScrollHdmaObject,
                S::ClearBg2Tilemap => N::ClearBg2Tilemap,
                S::ClearKraidBg2Tilemap => N::ClearKraidBg2Tilemap,
                S::AnimtilesHandler => N::AnimtilesHandler,
                S::LoadNewMusicTrackIfChanged => N::LoadNewMusicTrackIfChanged,
                S::PointlessFunctionStupidToo => N::PointlessFunctionStupidToo,
                S::SetLiquidPhysicsType => N::SetLiquidPhysicsType,
                S::EprojRunAll => N::EprojRunAll,
                S::QueueEnemyBg2TilemapTransfers => N::QueueEnemyBg2TilemapTransfers,
                S::PlaySpinJumpSoundIfSpinJumping => N::PlaySpinJumpSoundIfSpinJumping,
            },
        )
    }
    fn palette_fade(&mut self, ram: &mut [u8]) -> Result<u8, String> {
        self.native.palette_fade(ram)
    }
    fn has_queued_music(&mut self, ram: &mut [u8]) -> Result<u8, String> {
        self.native.has_queued_music(ram)
    }
    fn call_samus_code(&mut self, ram: &mut [u8], code: u16) -> Result<u16, String> {
        self.native.call_samus_code(ram, code)
    }
    fn coroutine(&mut self, ram: &mut [u8], service: sm_room_transition::CoroutineService) -> Result<u8, String> {
        use sm_native_host::RoomTransitionCoroutineService as N;
        use sm_room_transition::CoroutineService as S;
        let waiting_for_nmi = matches!(service, S::WaitForNmi);
        let result = self.native.coroutine(
            ram,
            match service {
                S::WaitForNmi => N::WaitForNmi,
                S::PlmHandler => N::PlmHandler,
            },
        )?;
        if waiting_for_nmi && result != 0 && *self.false_wait_completion {
            // Falsely complete one real pending wait. The actual Rust control
            // flow continues; neither the C trace nor comparator is changed.
            *self.false_wait_completion = false;
            return Ok(0);
        }
        Ok(result)
    }
    fn observe(&mut self, observation: sm_room_transition::Observation) -> Result<(), String> {
        use sm_native_host::{RoomTransitionObservation as N, RoomTransitionRoutine as R};
        use sm_room_transition::Observation as S;
        self.native.observe(match observation {
            S::Enter { routine, operand } => N::Enter { routine: R::try_from(routine as u32)?, operand },
            S::Return { routine, result } => N::Return { routine: R::try_from(routine as u32)?, result },
        })
    }
}

pub fn install_room_transitions(game: &mut Game, deliberate_fault: bool) -> Result<(), String> {
    use sm_native_host::{RoomTransitionEntry as N, RoomTransitionOutcome as NO};
    use sm_room_transition::{Entry as S, Outcome as SO};
    let mut false_wait_completion = deliberate_fault;
    game.set_room_transition_hook(move |entry, ram, rom, services| {
        let entry = match entry {
            N::HitDoorBlock => S::HitDoorBlock,
            N::LoadingNextRoom => S::LoadingNextRoom,
            N::LoadingNextRoomAsync => S::LoadingNextRoomAsync,
        };
        let mut host = TransitionServices { native: services, false_wait_completion: &mut false_wait_completion };
        Ok(match sm_room_transition::run(entry, ram, rom, &mut host)? {
            SO::Handled(result) => NO::Handled(result),
            SO::Refused => NO::Refused,
        })
    })
}

struct DirectionalServices<'a>(&'a mut sm_native_host::DoorScrollingServices);
impl sm_door_scrolling::Host for DirectionalServices<'_> {
    fn calculate_layer2_x(&mut self, ram: &mut [u8]) -> Result<u8, String> {
        self.0.calculate_layer2_x(ram)
    }
    fn calculate_layer2_y(&mut self, ram: &mut [u8]) -> Result<u8, String> {
        self.0.calculate_layer2_y(ram)
    }
    fn calculate_blocks(&mut self, ram: &mut [u8]) -> Result<(), String> {
        self.0.calculate_blocks(ram)
    }
    fn update_scroll_vars(&mut self, ram: &mut [u8]) -> Result<(), String> {
        self.0.update_scroll_vars(ram)
    }
    fn observe(&mut self, observation: sm_door_scrolling::Observation) -> Result<(), String> {
        use sm_door_scrolling::Observation as S;
        use sm_native_host::{DoorScrollingObservation as N, DoorScrollingRoutine as R};
        self.0.observe(match observation {
            S::Enter { routine, counter } => N::Enter { routine: R::try_from(routine as u32)?, counter },
            S::Return { routine, result, counter } => {
                N::Return { routine: R::try_from(routine as u32)?, result, counter }
            }
        })
    }
}

pub fn install_door_scrolling(game: &mut Game, deliberate_fault: bool) -> Result<(), String> {
    use sm_door_scrolling::{Entry as S, Outcome};
    use sm_native_host::DoorScrollingEntry as N;
    let mut fault_pending = deliberate_fault;
    game.set_door_scrolling_hook(move |entry, ram, services| {
        let entry = match entry {
            N::FixDoorsMovingUp => S::FixDoorsMovingUp,
            N::ScrollingSetup => S::ScrollingSetup,
            N::FollowTransition => S::FollowTransition,
        };
        let counter_before = u16::from_le_bytes([ram[0x925], ram[0x926]]);
        let result = sm_door_scrolling::run(entry, ram, &mut DirectionalServices(services))?;
        if fault_pending
            && matches!(entry, S::FollowTransition)
            && counter_before == 63
            && matches!(result, Outcome::Handled)
        {
            // Lose the completion bit after a real terminal movement. This
            // changes actual gameplay output; reference and comparator stay fixed.
            ram[0x932] &= 0x7f;
            fault_pending = false;
        }
        Ok(match result {
            Outcome::Handled => HookOutcome::Handled,
            Outcome::Refused => HookOutcome::Refused,
        })
    })
}

struct CameraObserver<'a>(&'a mut sm_native_host::CameraObserver);
impl sm_camera::Observer for CameraObserver<'_> {
    fn observe(&mut self, observation: sm_camera::Observation) -> Result<(), String> {
        use sm_camera::Observation as S;
        use sm_native_host::{CameraObservation as N, CameraRoutine as R};
        self.0.observe(match observation {
            S::Enter { routine, operand } => N::Enter { routine: R::try_from(routine as u32)?, operand },
            S::Return { routine } => N::Return { routine: R::try_from(routine as u32)? },
        })
    }
}

pub fn install_camera(game: &mut Game, deliberate_fault: bool) -> Result<(), String> {
    let mut fault_pending = deliberate_fault;
    game.set_camera_hook(move |ram, observer| {
        let before_y = u16::from_le_bytes([ram[0x915], ram[0x916]]);
        let outcome = sm_camera::run(ram, &mut CameraObserver(observer))?;
        let after_y = u16::from_le_bytes([ram[0x915], ram[0x916]]);
        if fault_pending && matches!(outcome, sm_camera::Outcome::Handled) && after_y != before_y {
            // One real camera update gains an incorrect pixel. This changes
            // gameplay output, leaving the saved reference/comparator intact.
            ram[0x915..0x917].copy_from_slice(&after_y.wrapping_add(1).to_le_bytes());
            fault_pending = false;
        }
        Ok(match outcome {
            sm_camera::Outcome::Handled => HookOutcome::Handled,
            sm_camera::Outcome::Refused => HookOutcome::Refused,
        })
    })
}
