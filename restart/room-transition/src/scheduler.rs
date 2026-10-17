//! The three roots and all fifteen phase bodies. State words remain canonical.
use crate::{
    CoroutineService, Entry, Host, Observation, Result, Routine, SyncService as S,
    memory::{Rom, Wram},
};

pub(crate) const PHASES: [u16; 15] = [
    0xe17d, 0xe19f, 0xe29e, 0xe2db, 0xe2f7, 0xe310, 0xe353, 0xe36e, 0xe38e, 0xe3c0, 0xe4a9, 0xe659, 0xe664, 0xe6a2,
    0xe737,
];
const ROUTINES: [Routine; 15] = [
    Routine::HandleElevator,
    Routine::Wait48Frames,
    Routine::WaitForSounds,
    Routine::FadeOut,
    Routine::LoadDoorHeader,
    Routine::AlignScroll,
    Routine::FixDoorsMovingUp,
    Routine::SetupNewRoom,
    Routine::SetupScrolling,
    Routine::PlaceSamusLoadTiles,
    Routine::LoadMoreThings,
    Routine::HandleAnimTiles,
    Routine::WaitForMusic,
    Routine::HandleTransition,
    Routine::FadeInAndFinish,
];

pub(crate) struct Coordinator<'a, H> {
    pub ram: Wram<'a>,
    pub rom: Rom<'a>,
    pub host: &'a mut H,
}

impl<H: Host> Coordinator<'_, H> {
    pub fn enter(&mut self, routine: Routine, operand: u16) -> Result<()> {
        self.host.observe(Observation::Enter { routine, operand })
    }
    pub fn leave(&mut self, routine: Routine, result: u16) -> Result<()> {
        self.host.observe(Observation::Return { routine, result })
    }
    pub fn sync(&mut self, service: S) -> Result<()> {
        self.host.sync(self.ram.0, service)
    }
    fn services(&mut self, services: &[S]) -> Result<()> {
        for &service in services {
            self.sync(service)?;
        }
        Ok(())
    }
    fn actor_frame(&mut self) -> Result<()> {
        self.services(&[
            S::DetermineWhichEnemiesToProcess,
            S::EnemyMain,
            S::DrawSamusEnemiesAndProjectiles,
            S::EnsureSamusDrawnEachFrame,
        ])
    }

    pub fn root(&mut self, entry: Entry) -> Result<u8> {
        let routine = match entry {
            Entry::HitDoorBlock => Routine::HitDoorBlock,
            Entry::LoadingNextRoom => Routine::LoadingNextRoom,
            Entry::LoadingNextRoomAsync => Routine::LoadingNextRoomAsync,
        };
        self.enter(routine, 0)?;
        let result = match entry {
            Entry::HitDoorBlock => self.dispatch()?,
            Entry::LoadingNextRoom => self.prepare_room()?,
            Entry::LoadingNextRoomAsync => {
                let pending = self.dispatch()?;
                if pending == 0 && self.ram.word(0x943) != 0 {
                    self.sync(S::DrawTimer)?;
                }
                pending
            }
        };
        self.leave(routine, u16::from(result))?;
        Ok(result)
    }

    fn dispatch(&mut self) -> Result<u8> {
        let word = self.ram.word(0x99c);
        self.enter(Routine::Dispatch, word)?;
        let result = self.phase(word)?;
        self.leave(Routine::Dispatch, u16::from(result))?;
        Ok(result)
    }

    fn phase(&mut self, word: u16) -> Result<u8> {
        let index = PHASES.iter().position(|&phase| phase == word).ok_or("unknown owned door-transition phase")?;
        let routine = ROUTINES[index];
        self.enter(routine, 0)?;
        let result = self.phase_body(word)?;
        self.leave(routine, u16::from(result))?;
        Ok(result)
    }

    fn phase_body(&mut self, phase: u16) -> Result<u8> {
        match phase {
            0xe17d => {
                if self.ram.word(0xe16) == 0 {
                    self.ram.add(0x998, 1);
                    return self.root(Entry::LoadingNextRoom);
                }
                self.host.call_samus_code(self.ram.0, 0)?;
                if self.ram.word(0x799) & 0x8000 != 0 {
                    self.ram.add(0x998, 1);
                    return self.root(Entry::LoadingNextRoom);
                }
                self.ram.set(0x92f, 48);
                self.ram.set(0x99c, 0xe19f);
                return self.phase(0xe19f);
            }
            0xe19f => {
                self.ram.add(0x92f, 0xffff);
                if self.ram.word(0x92f) & 0x8000 != 0 {
                    self.ram.add(0x998, 1);
                    return self.root(Entry::LoadingNextRoom);
                }
                self.actor_frame()?;
            }
            0xe29e => {
                self.actor_frame()?;
                if (0..3).all(|i| self.ram.0[0x646 + i].wrapping_sub(self.ram.0[0x643 + i]) & 15 == 0) {
                    self.ram.set(0x99c, 0xe2db);
                }
            }
            0xe2db => {
                if self.host.palette_fade(self.ram.0)? != 0 {
                    self.ram.set(0x99c, 0xe2f7);
                } else {
                    self.actor_frame()?;
                }
            }
            0xe2f7 => {
                self.services(&[S::LoadDoorHeader, S::Sub8882Ac])?;
                self.ram.set(0x18b0, self.ram.word(0x18b0) & !0x8000);
                self.ram.set(0xa7, 8);
                self.ram.set(0x99c, 0xe310);
            }
            0xe310 => {
                let axis = if self.ram.word(0x791) & 2 != 0 { 0x911 } else { 0x915 };
                let coordinate = self.ram.word(axis);
                if coordinate & 0xff != 0 {
                    self.ram.add(axis, if coordinate & 0x80 != 0 { 1 } else { 0xffff });
                    self.sync(S::CalculateLayer2PosAndScrollsWhenScrolling)?;
                    return Ok(0);
                }
                self.sync(S::CalculateLayer2PosAndScrollsWhenScrolling)?;
                self.ram.set(0x99c, 0xe353);
            }
            0xe353 => {
                if self.ram.word(0x791) & 3 == 3 {
                    self.sync(S::FixDoorsMovingUp)?;
                }
                self.ram.set(0x99c, 0xe36e);
            }
            0xe36e => {
                self.services(&[
                    S::SaveMapExploredIfElevator,
                    S::LoadRoomHeader,
                    S::LoadStateHeader,
                    S::LoadMapExploredIfElevator,
                    S::InitializeSpecialEffectsForNewRoom,
                    S::LoadLevelScrollAndCre,
                ])?;
                self.ram.set(0x99c, 0xe38e);
            }
            0xe38e => {
                self.ram.set(0xb5, 0);
                self.ram.set(0xb7, 0);
                self.ram.set(0x7e9, 0);
                if self.ram.word(0x791) & 3 == 2 {
                    self.ram.add(0xb3, 1);
                }
                if self.ram.word(0x791) & 3 != 3 {
                    self.ram.set(0x925, 0);
                }
                self.sync(S::DoorTransitionScrollingSetup)?;
                self.ram.set(0x99c, 0xe3c0);
            }
            0xe3c0 => self.place_samus_load_tiles()?,
            0xe4a9 => return self.load_more(),
            0xe659 => {
                self.sync(S::AnimtilesHandler)?;
                self.ram.set(0x99c, 0xe664);
            }
            0xe664 => {
                if self.host.has_queued_music(self.ram.0)? == 0 {
                    self.ram.set(0x99c, 0xe6a2);
                    self.sync(S::LoadNewMusicTrackIfChanged)?;
                }
            }
            0xe6a2 => self.finish_transition()?,
            0xe737 => {
                self.services(&[
                    S::AnimtilesHandler,
                    S::DetermineWhichEnemiesToProcess,
                    S::EnemyMain,
                    S::EprojRunAll,
                    S::DrawSamusEnemiesAndProjectiles,
                    S::EnsureSamusDrawnEachFrame,
                    S::QueueEnemyBg2TilemapTransfers,
                ])?;
                if self.host.palette_fade(self.ram.0)? != 0 {
                    self.ram.set(0x5f5, 0);
                    self.sync(S::PlaySpinJumpSoundIfSpinJumping)?;
                    self.ram.set(0x795, 0);
                    self.ram.set(0x797, 0);
                    self.ram.set(0x998, 8);
                }
            }
            _ => return Err("unknown owned door-transition phase".into()),
        }
        Ok(0)
    }

    fn prepare_room(&mut self) -> Result<u8> {
        self.ram.set(0x797, 1);
        self.ram.set(0x795, 1);
        self.ram.set(0x5f7, 0);
        self.ram.set(0x1e75, 0);
        self.actor_frame()?;
        self.sync(S::LoadDestinationRoomCreBitset)?;
        self.ram.0[0xc200..0xc400].fill(0);
        for index in [9, 10, 13, 14, 17, 18, 19, 29] {
            self.ram.set(0xc200 + 2 * index, self.ram.word(0xc000 + 2 * index));
        }
        if (self.ram.word(0x7b1) | self.ram.word(0x7b3)) & 1 == 0 {
            for index in [20, 21, 22, 23, 28] {
                self.ram.set(0xc200 + 2 * index, self.ram.word(0xc000 + 2 * index));
            }
            if self.ram.word(0x943) != 0 {
                for index in [209, 210, 212, 221] {
                    self.ram.set(0xc200 + 2 * index, self.ram.word(0xc000 + 2 * index));
                }
                self.sync(S::DrawTimer)?;
            }
        }
        self.services(&[S::ClearSoundsWhenGoingThroughDoor, S::QueueSfx2Max15 { sound: 0x71 }])?;
        self.ram.set(0x5f5, 0xffff);
        self.ram.set(0x99c, 0xe29e);
        self.ram.add(0x998, 1);
        Ok(0)
    }

    fn place_samus_load_tiles(&mut self) -> Result<()> {
        let x = self.ram.word(0x911).wrapping_add(u16::from(self.ram.0[0xaf6]));
        self.ram.set(0xaf6, x);
        self.ram.set(0xb10, x);
        let y = self.ram.word(0x915).wrapping_add(u16::from(self.ram.0[0xafa]));
        self.ram.set(0xafa, y);
        self.ram.set(0xb14, y);
        self.ram.set(0x931, 0);
        self.ram.set(0xa7, if self.ram.word(0x791) & 3 == 2 { 16 } else { 22 });
        self.sync(S::WaitUntilEndOfVblankAndEnableIrq)?;
        if self.ram.word(0x7b3) & 2 != 0 && self.ram.word(0x78d) != 0x947a {
            self.sync(S::DecompressToMem { source: 0xb98000, destination: 0x7000 })?;
        }
        self.sync(S::DecompressToMem { source: self.ram.long(0x7c3), destination: 0x2000 })?;
        self.sync(S::DecompressToMem { source: self.ram.long(0x7c6), destination: 0xc200 })?;
        self.services(&[
            S::CopyToVramNow { vram_dst: 0, source: 0x7e2000, size: 0x2000 },
            S::CopyToVramNow { vram_dst: 0x1000, source: 0x7e4000, size: 0x2000 },
            S::CopyToVramNow { vram_dst: 0x2000, source: 0x7e6000, size: 0x1000 },
        ])?;
        if self.ram.word(0x7b3) & 6 != 0 && self.ram.word(0x78d) != 0x947a {
            self.services(&[
                S::CopyToVramNow { vram_dst: 0x2800, source: 0x7e7000, size: 0x1000 },
                S::CopyToVramNow { vram_dst: 0x3000, source: 0x7e8000, size: 0x2000 },
                S::CopyToVramNow { vram_dst: 0x4000, source: 0x9ab200, size: 0x1000 },
            ])?;
        }
        if self.ram.word(0x791) & 3 == 3 {
            self.ram.set(0xa7, 16);
        }
        self.ram.set(0x99c, 0xe4a9);
        Ok(())
    }

    fn load_more(&mut self) -> Result<u8> {
        let resume = self.ram.0[0x612];
        match resume {
            0 => self.load_once()?,
            1 => {
                // The C case label is inside the while body: finish its already
                // pending wait before inspecting the newly observed IRQ flag.
                let pending = self.host.coroutine(self.ram.0, CoroutineService::WaitForNmi)?;
                if pending != 0 {
                    self.ram.0[0x612] = 1;
                    return Ok(pending);
                }
            }
            2 => return self.finish_plms(),
            _ => return Err("invalid owned transition continuation".into()),
        }
        while self.ram.word(0x931) & 0x8000 == 0 {
            let pending = self.host.coroutine(self.ram.0, CoroutineService::WaitForNmi)?;
            if pending != 0 {
                self.ram.0[0x612] = 1;
                return Ok(pending);
            }
        }
        // State 1 remains published here after a successful resumed wait.
        self.ram.set(0xc188, 15328);
        self.sync(S::SpawnBg3ScrollHdmaObject)?;
        self.ram.set(0x18b0, self.ram.word(0x18b0) | 0x8000);
        self.finish_plms()
    }

    fn load_once(&mut self) -> Result<()> {
        self.services(&[
            S::LoadEnemyGfxToVram,
            S::LoadRoomMusic,
            S::ClearEprojs,
            S::ClearAnimtiles,
            S::ClearPaletteFxObjects,
            S::ClearPlms,
            S::CreatePlmsExecuteDoorAsmRoomSetup,
            S::LoadFxHeader,
            S::SpawnDoorClosingPlm,
            S::UpdateBeamTilesAndPalette,
            S::LoadColorsForSpritesBeamsAndEnemies,
            S::LoadEnemies,
            S::InitializeEnemies,
            S::ResetProjectileData,
            S::SamusLoadSuitTargetPalette,
            S::ClearFxTilemap,
        ])?;
        let fx = self.ram.word(0x1964);
        if fx != 0 {
            self.sync(S::CopyToVramNow { vram_dst: 0x5be0, source: 0x8a0000 | u32::from(fx), size: 2112 })?;
        }
        let header = self.ram.word(0x7bb);
        let start = self.rom.word(header.wrapping_add(0x16))?;
        if start & 0x8000 != 0 {
            self.background(start)?;
        }
        Ok(())
    }

    fn finish_plms(&mut self) -> Result<u8> {
        let pending = self.host.coroutine(self.ram.0, CoroutineService::PlmHandler)?;
        if pending != 0 {
            self.ram.0[0x612] = 2;
            return Ok(pending);
        }
        let direction = self.ram.word(0x791);
        if direction & 2 == 0 {
            let x = self.ram.word(0xaf6);
            self.ram.set(0xaf6, if direction & 3 != 0 { x & 0xfff8 } else { x | 7 });
        }
        self.ram.set(0x99c, 0xe659);
        self.ram.0[0x612] = 0;
        Ok(0)
    }

    fn finish_transition(&mut self) -> Result<()> {
        let x = self.ram.word(0xaf6);
        if x & 0xf0 == 0x10 {
            self.ram.set(0xaf6, (x | 0xf).wrapping_add(8));
        } else if x & 0xf0 == 0xe0 {
            self.ram.set(0xaf6, (x & 0xfff0).wrapping_sub(8));
        }
        let y = self.ram.word(0xafa);
        if y & 0xf0 == 0x10 {
            self.ram.set(0xafa, (y | 0xf).wrapping_add(8));
        }
        self.ram.0[0x9c00..0xa000].fill(0);
        let irq = self.ram.word(0xa9);
        self.ram.set(0xa7, if irq == 0 { 4 } else { irq });
        self.sync(S::PointlessFunctionStupidToo)?;
        if self.ram.word(0xe16) != 0 {
            let code = if self.ram.word(0x799) & 0x8000 == 0 { 7 } else { 0 };
            self.host.call_samus_code(self.ram.0, code)?;
        }
        self.sync(S::SetLiquidPhysicsType)?;
        self.ram.set(0x99c, 0xe737);
        self.ram.set(0x51, self.ram.word(0x51) | 0x1f);
        Ok(())
    }
}
