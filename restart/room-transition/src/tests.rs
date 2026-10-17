use super::*;
use SyncService as S;
use std::collections::VecDeque;

fn word(ram: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([ram[at], ram[at + 1]])
}
fn set(ram: &mut [u8], at: usize, value: u16) {
    ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn long(ram: &mut [u8], at: usize, value: u32) {
    ram[at..at + 3].copy_from_slice(&value.to_le_bytes()[..3]);
}
fn rom_bytes(rom: &mut [u8], at: u16, data: &[u8]) {
    let start = 0x78000 + usize::from(at & 0x7fff);
    rom[start..start + data.len()].copy_from_slice(data);
}
fn state(phase: u16) -> (Vec<u8>, Vec<u8>) {
    let mut ram = vec![0; WRAM_SIZE];
    set(&mut ram, 0x998, 11);
    set(&mut ram, 0x99c, phase);
    set(&mut ram, 0x7bb, 0x9000);
    // snes_loadRom expands the disk image to this native cartridge extent.
    (ram, vec![0; 0x400000])
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Call {
    Sync(S),
    Fade(u8),
    Music(u8),
    Samus(u16),
    Coroutine(CoroutineService, u8),
}
#[derive(Default)]
struct Probe {
    calls: Vec<Call>,
    observations: Vec<Observation>,
    stack: Vec<Routine>,
    waits: VecDeque<u8>,
    plms: VecDeque<u8>,
    fades: VecDeque<u8>,
    music: VecDeque<u8>,
    // Continuation/IRQ/completion values seen at each actual coroutine request.
    continuation_inputs: Vec<(CoroutineService, u8, u16, u8)>,
    effect: Option<fn(&mut [u8], S)>,
    fail: Option<S>,
}
impl Probe {
    fn syncs(&self) -> Vec<S> {
        self.calls.iter().filter_map(|call| if let Call::Sync(s) = call { Some(*s) } else { None }).collect()
    }
    fn entered(&self, routine: Routine) -> usize {
        self.observations
            .iter()
            .filter(|event| matches!(event, Observation::Enter { routine: r, .. } if *r == routine))
            .count()
    }
}
impl Host for Probe {
    fn sync(&mut self, ram: &mut [u8], service: S) -> Result<()> {
        self.calls.push(Call::Sync(service));
        if self.fail == Some(service) {
            return Err("deliberate native service failure".into());
        }
        if let Some(effect) = self.effect {
            effect(ram, service);
        }
        Ok(())
    }
    fn palette_fade(&mut self, _ram: &mut [u8]) -> Result<u8> {
        let value = self.fades.pop_front().expect("unexpected palette fade");
        self.calls.push(Call::Fade(value));
        Ok(value)
    }
    fn has_queued_music(&mut self, _ram: &mut [u8]) -> Result<u8> {
        let value = self.music.pop_front().expect("unexpected music query");
        self.calls.push(Call::Music(value));
        Ok(value)
    }
    fn call_samus_code(&mut self, _ram: &mut [u8], code: u16) -> Result<u16> {
        self.calls.push(Call::Samus(code));
        Ok(0xabcd)
    }
    fn coroutine(&mut self, ram: &mut [u8], service: CoroutineService) -> Result<u8> {
        self.continuation_inputs.push((service, ram[0x612], word(ram, 0x931), ram[0x782]));
        let value = match service {
            CoroutineService::WaitForNmi => {
                let result = self.waits.pop_front().expect("unexpected NMI wait");
                ram[0x782] = u8::from(result != 0);
                result
            }
            CoroutineService::PlmHandler => self.plms.pop_front().unwrap_or(0),
        };
        self.calls.push(Call::Coroutine(service, value));
        Ok(value)
    }
    fn observe(&mut self, observation: Observation) -> Result<()> {
        match observation {
            Observation::Enter { routine, .. } => {
                assert!(!self.stack.contains(&routine), "unexpected routine recursion");
                self.stack.push(routine);
            }
            Observation::Return { routine, .. } => assert_eq!(self.stack.pop(), Some(routine)),
        }
        self.observations.push(observation);
        Ok(())
    }
}
fn invoke(entry: Entry, ram: &mut [u8], rom: &[u8], host: &mut Probe) -> u8 {
    let Outcome::Handled(value) = run(entry, ram, rom, host).unwrap() else { panic!("fixture refused") };
    assert!(host.stack.is_empty(), "pending calls must also unwind observations");
    value
}
fn async_step(ram: &mut [u8], rom: &[u8], host: &mut Probe) -> u8 {
    invoke(Entry::LoadingNextRoomAsync, ram, rom, host)
}

#[test]
fn state_nine_calls_state_ten_in_same_invocation_with_distinct_palette_branches() {
    for previous_cre in [0, 1] {
        let (mut ram, rom) = state(0xe17d);
        set(&mut ram, 0x998, 9);
        set(&mut ram, 0x7b1, previous_cre);
        set(&mut ram, 0x943, 1);
        for index in 0..256 {
            set(&mut ram, 0xc000 + 2 * index, 0x1000 + index as u16);
        }
        ram[0xc200..0xc400].fill(0xaa);
        let mut host = Probe::default();
        assert_eq!(invoke(Entry::HitDoorBlock, &mut ram, &rom, &mut host), 0);
        assert_eq!(word(&ram, 0x998), 11);
        assert_eq!(word(&ram, 0x99c), 0xe29e);
        assert_eq!(word(&ram, 0x5f5), 0xffff);
        assert_eq!(word(&ram, 0x795), 1);
        assert_eq!(word(&ram, 0x797), 1);
        assert_eq!(word(&ram, 0xc212), 0x1009);
        assert_eq!(word(&ram, 0xc214), 0x100a);
        assert_eq!(word(&ram, 0xc200), 0);
        assert_eq!(word(&ram, 0xc228), if previous_cre == 0 { 0x1014 } else { 0 });
        assert_eq!(word(&ram, 0xc3a2), if previous_cre == 0 { 0x10d1 } else { 0 });
        let mut expected = vec![
            S::DetermineWhichEnemiesToProcess,
            S::EnemyMain,
            S::DrawSamusEnemiesAndProjectiles,
            S::EnsureSamusDrawnEachFrame,
            S::LoadDestinationRoomCreBitset,
        ];
        if previous_cre == 0 {
            expected.push(S::DrawTimer);
        }
        expected.extend([S::ClearSoundsWhenGoingThroughDoor, S::QueueSfx2Max15 { sound: 0x71 }]);
        assert_eq!(host.syncs(), expected);
        assert_eq!(
            host.observations,
            [
                Observation::Enter { routine: Routine::HitDoorBlock, operand: 0 },
                Observation::Enter { routine: Routine::Dispatch, operand: 0xe17d },
                Observation::Enter { routine: Routine::HandleElevator, operand: 0 },
                Observation::Enter { routine: Routine::LoadingNextRoom, operand: 0 },
                Observation::Return { routine: Routine::LoadingNextRoom, result: 0 },
                Observation::Return { routine: Routine::HandleElevator, result: 0 },
                Observation::Return { routine: Routine::Dispatch, result: 0 },
                Observation::Return { routine: Routine::HitDoorBlock, result: 0 },
            ]
        );
    }
}

#[test]
fn downward_elevator_enters_delay_immediately_and_predecrements_to_negative() {
    let (mut ram, rom) = state(0xe17d);
    set(&mut ram, 0x998, 9);
    set(&mut ram, 0xe16, 1);
    let mut host = Probe::default();
    invoke(Entry::HitDoorBlock, &mut ram, &rom, &mut host);
    assert_eq!(word(&ram, 0x92f), 47);
    assert_eq!(word(&ram, 0x99c), 0xe19f);
    assert_eq!(host.calls[0], Call::Samus(0));
    assert_eq!(host.entered(Routine::Wait48Frames), 1);
    for _ in 0..47 {
        invoke(Entry::HitDoorBlock, &mut ram, &rom, &mut host);
    }
    assert_eq!(word(&ram, 0x92f), 0);
    assert_eq!(word(&ram, 0x998), 9);
    invoke(Entry::HitDoorBlock, &mut ram, &rom, &mut host);
    assert_eq!(word(&ram, 0x92f), 0xffff);
    assert_eq!(word(&ram, 0x998), 11);
    assert_eq!(host.entered(Routine::LoadingNextRoom), 1);
    assert_eq!(host.entered(Routine::Wait48Frames), 49);
}

#[test]
fn upward_elevator_skips_delay_and_signed_delay_bit_is_preserved() {
    let (mut ram, rom) = state(0xe17d);
    set(&mut ram, 0x998, 9);
    set(&mut ram, 0xe16, 1);
    set(&mut ram, 0x799, 0x8000);
    let mut host = Probe::default();
    invoke(Entry::HitDoorBlock, &mut ram, &rom, &mut host);
    assert_eq!(word(&ram, 0x998), 11);
    assert_eq!(host.entered(Routine::Wait48Frames), 0);
    assert_eq!(host.calls[0], Call::Samus(0));
    set(&mut ram, 0x998, 9);
    set(&mut ram, 0x99c, 0xe19f);
    set(&mut ram, 0x92f, 0x8001);
    invoke(Entry::HitDoorBlock, &mut ram, &rom, &mut host);
    assert_eq!(word(&ram, 0x92f), 0x8000);
    assert_eq!(word(&ram, 0x998), 11);
}

#[test]
fn pending_nmi_then_repeated_plm_resumes_without_reinitializing() {
    let (mut ram, rom) = state(0xe4a9);
    ram[0x611] = 21; // The surrounding native scheduler owns this latch.
    ram[0x613] = 0x55;
    set(&mut ram, 0xaf6, 0x82);
    set(&mut ram, 0x943, 1);
    let mut host = Probe {
        waits: [0xfe, 0].into(),
        plms: [7, 0x80, 0].into(),
        effect: Some(|ram, service| {
            if service == S::SpawnBg3ScrollHdmaObject {
                assert_eq!(ram[0x612], 1, "successful wait must not clear state 1");
                assert_eq!(ram[0x782], 0, "pending wait cleanup precedes HDMA setup");
                assert_eq!(word(ram, 0xc188), 15328);
            }
        }),
        ..Probe::default()
    };
    assert_eq!(async_step(&mut ram, &rom, &mut host), 0xfe);
    assert_eq!(ram[0x612], 1);
    assert_eq!(word(&ram, 0x99c), 0xe4a9);
    assert_eq!(host.syncs().iter().filter(|&&s| s == S::LoadEnemyGfxToVram).count(), 1);
    assert!(!host.syncs().contains(&S::DrawTimer));
    // IRQ finishes between scheduled calls, while the native wait is outstanding.
    set(&mut ram, 0x931, 0x8000);
    assert_eq!(async_step(&mut ram, &rom, &mut host), 7);
    assert_eq!(host.continuation_inputs[1], (CoroutineService::WaitForNmi, 1, 0x8000, 1));
    assert_eq!(ram[0x612], 2);
    assert_eq!(word(&ram, 0x18b0), 0x8000);
    assert_eq!(word(&ram, 0xaf6), 0x82);
    assert_eq!(async_step(&mut ram, &rom, &mut host), 0x80);
    assert_eq!(ram[0x612], 2);
    assert_eq!(async_step(&mut ram, &rom, &mut host), 0);
    assert_eq!(ram[0x612], 0);
    assert_eq!(ram[0x611], 21);
    assert_eq!(ram[0x613], 0x55);
    assert_eq!(word(&ram, 0xaf6), 0x87);
    assert_eq!(word(&ram, 0x99c), 0xe659);
    assert_eq!(host.syncs().iter().filter(|&&s| s == S::LoadEnemyGfxToVram).count(), 1);
    assert_eq!(host.syncs().iter().filter(|&&s| s == S::SpawnBg3ScrollHdmaObject).count(), 1);
    assert_eq!(host.syncs().iter().filter(|&&s| s == S::DrawTimer).count(), 1);
    assert_eq!(host.continuation_inputs.iter().filter(|v| v.0 == CoroutineService::WaitForNmi).count(), 2);
    assert!(host.observations.contains(&Observation::Return { routine: Routine::LoadMoreThings, result: 0xfe }));
}

#[test]
fn resumed_wait_with_unfinished_irq_can_wait_again_in_same_invocation() {
    let (mut ram, rom) = state(0xe4a9);
    ram[0x612] = 1;
    ram[0x782] = 1;
    let mut host = Probe { waits: [0, 3].into(), ..Probe::default() };
    assert_eq!(async_step(&mut ram, &rom, &mut host), 3);
    assert_eq!(ram[0x612], 1);
    assert_eq!(
        host.continuation_inputs,
        [(CoroutineService::WaitForNmi, 1, 0, 1), (CoroutineService::WaitForNmi, 1, 0, 0),]
    );
    assert!(host.syncs().is_empty());
}

fn install_background(rom: &mut [u8], data: &[u8]) {
    rom_bytes(rom, 0x9016, &0xe4a5u16.to_le_bytes());
    rom_bytes(rom, 0xe4a5, data);
}

#[test]
fn nonempty_background_decompresses_before_two_immediate_vram_transfers() {
    let (mut ram, mut rom) = state(0xe4a9);
    set(&mut ram, 0x931, 0x8000);
    install_background(
        &mut rom,
        &[
            4, 0, 0x2a, 0xc2, 0xba, 0, 0x40, 2, 0, 0, 0x40, 0x7e, 0, 0x48, 0, 8, 2, 0, 0, 0x40, 0x7e, 0, 0x4c, 0, 8, 0,
            0,
        ],
    );
    let mut host = Probe {
        effect: Some(|ram, service| match service {
            S::DecompressToMem { source: 0xbac22a, destination: 0x4000 } => set(ram, 0x4000, 0xbeef),
            S::CopyToVramNow { source: 0x7e4000, .. } => assert_eq!(word(ram, 0x4000), 0xbeef),
            _ => {}
        }),
        ..Probe::default()
    };
    assert_eq!(async_step(&mut ram, &rom, &mut host), 0);
    let graphics: Vec<_> =
        host.syncs().into_iter().filter(|s| matches!(s, S::DecompressToMem { .. } | S::CopyToVramNow { .. })).collect();
    assert_eq!(
        graphics,
        [
            S::DecompressToMem { source: 0xbac22a, destination: 0x4000 },
            S::CopyToVramNow { vram_dst: 0x4800, source: 0x7e4000, size: 0x800 },
            S::CopyToVramNow { vram_dst: 0x4c00, source: 0x7e4000, size: 0x800 },
        ]
    );
    let background: Vec<_> = host
        .observations
        .iter()
        .copied()
        .filter(|v| match v {
            Observation::Enter { routine, .. } | Observation::Return { routine, .. } => *routine as u32 >= 19,
        })
        .collect();
    assert_eq!(
        background,
        [
            Observation::Enter { routine: Routine::BackgroundDecompress, operand: 0xe4a7 },
            Observation::Return { routine: Routine::BackgroundDecompress, result: 0xe4ac },
            Observation::Enter { routine: Routine::BackgroundTransfer, operand: 0xe4ae },
            Observation::Return { routine: Routine::BackgroundTransfer, result: 0xe4b5 },
            Observation::Enter { routine: Routine::BackgroundTransfer, operand: 0xe4b7 },
            Observation::Return { routine: Routine::BackgroundTransfer, result: 0xe4be },
            Observation::Enter { routine: Routine::BackgroundFinish, operand: 0xe4c0 },
            Observation::Return { routine: Routine::BackgroundFinish, result: 0 },
        ]
    );
    assert!(host.continuation_inputs.iter().all(|v| v.0 == CoroutineService::PlmHandler));
}

#[test]
fn remaining_background_commands_preserve_nested_transfer_and_door_skip() {
    for matching_door in [false, true] {
        let (mut ram, mut rom) = state(0xe4a9);
        set(&mut ram, 0x931, 0x8000);
        set(&mut ram, 0x78d, if matching_door { 0xab4c } else { 0xab64 });
        install_background(
            &mut rom,
            &[
                6, 0, 10, 0, 12, 0, 8, 0, 0, 0x40, 0x7e, 0, 0x48, 2, 0, 14, 0, 0x4c, 0xab, 0, 0x50, 0x7e, 0, 0x4c, 4,
                0, 0, 0,
            ],
        );
        let mut host = Probe::default();
        async_step(&mut ram, &rom, &mut host);
        assert_eq!(ram[0x5e], 2);
        assert_eq!(host.entered(Routine::BackgroundClearFx), 1);
        assert_eq!(host.entered(Routine::BackgroundClearBg2), 1);
        assert_eq!(host.entered(Routine::BackgroundClearKraidBg2), 1);
        assert_eq!(host.entered(Routine::BackgroundTransferAndSetBg3), 1);
        assert_eq!(host.entered(Routine::BackgroundDoorTransfer), 1);
        assert_eq!(host.entered(Routine::BackgroundTransfer), if matching_door { 2 } else { 1 });
        let first_nested = host
            .observations
            .iter()
            .position(|v| matches!(v, Observation::Enter { routine: Routine::BackgroundTransferAndSetBg3, .. }))
            .unwrap();
        assert!(matches!(
            host.observations[first_nested + 1],
            Observation::Enter { routine: Routine::BackgroundTransfer, .. }
        ));
        assert!(matches!(
            host.observations[first_nested + 2],
            Observation::Return { routine: Routine::BackgroundTransfer, .. }
        ));
        assert!(matches!(
            host.observations[first_nested + 3],
            Observation::Return { routine: Routine::BackgroundTransferAndSetBg3, .. }
        ));
    }
}

#[test]
fn fades_preserve_raw_query_condition_and_only_finish_restores_gameplay() {
    let (mut ram, rom) = state(0xe2db);
    let mut host = Probe {
        fades: [0, 2, 0, 0x80].into(),
        effect: Some(|ram, service| {
            if service == S::PlaySpinJumpSoundIfSpinJumping {
                assert_eq!(word(ram, 0x5f5), 0);
                assert_eq!(word(ram, 0x998), 11);
                assert_eq!(word(ram, 0x795), 1);
            }
        }),
        ..Probe::default()
    };
    async_step(&mut ram, &rom, &mut host);
    assert_eq!(word(&ram, 0x99c), 0xe2db);
    assert_eq!(host.syncs().len(), 4);
    host.calls.clear();
    async_step(&mut ram, &rom, &mut host);
    assert_eq!(word(&ram, 0x99c), 0xe2f7);
    assert!(host.syncs().is_empty());
    set(&mut ram, 0x99c, 0xe737);
    set(&mut ram, 0x795, 1);
    set(&mut ram, 0x797, 1);
    set(&mut ram, 0x5f5, 0xffff);
    async_step(&mut ram, &rom, &mut host);
    assert_eq!(word(&ram, 0x998), 11);
    assert_eq!(word(&ram, 0x5f5), 0xffff);
    async_step(&mut ram, &rom, &mut host);
    assert_eq!(word(&ram, 0x998), 8);
    assert_eq!(word(&ram, 0x795), 0);
    assert_eq!(word(&ram, 0x797), 0);
    assert_eq!(word(&ram, 0x99c), 0xe737);
}

#[test]
fn alignment_steps_wrap_and_publish_next_phase_only_on_following_call() {
    for direction in 0..4 {
        for (initial, aligned) in [(0x101, 0x100), (0xff, 0x100), (0xffff, 0)] {
            let (mut ram, rom) = state(0xe310);
            set(&mut ram, 0x791, direction);
            let axis = if direction & 2 == 0 { 0x915 } else { 0x911 };
            set(&mut ram, axis, initial);
            let mut host = Probe::default();
            async_step(&mut ram, &rom, &mut host);
            assert_eq!(word(&ram, axis), aligned);
            assert_eq!(word(&ram, 0x99c), 0xe310);
            async_step(&mut ram, &rom, &mut host);
            assert_eq!(word(&ram, 0x99c), 0xe353);
            assert_eq!(host.syncs(), [S::CalculateLayer2PosAndScrollsWhenScrolling; 2]);
            async_step(&mut ram, &rom, &mut host);
            assert_eq!(word(&ram, 0x99c), 0xe36e);
            assert_eq!(host.syncs().contains(&S::FixDoorsMovingUp), direction == 3);
        }
    }
}

#[test]
fn header_and_setup_phases_keep_services_ordered_and_state_visible() {
    let (mut ram, rom) = state(0xe2f7);
    set(&mut ram, 0x18b0, 0xa123);
    let mut host = Probe {
        effect: Some(|ram, service| {
            if service == S::LoadDoorHeader {
                set(ram, 0x79b, 0xdf8d);
            }
            if service == S::Sub8882Ac {
                assert_eq!(word(ram, 0x79b), 0xdf8d);
            }
            if service == S::LoadRoomHeader {
                set(ram, 0x7bb, 0xdf9f);
            }
            if service == S::LoadStateHeader {
                assert_eq!(word(ram, 0x7bb), 0xdf9f);
            }
        }),
        ..Probe::default()
    };
    async_step(&mut ram, &rom, &mut host);
    assert_eq!(word(&ram, 0x18b0), 0x2123);
    assert_eq!(word(&ram, 0xa7), 8);
    assert_eq!(word(&ram, 0x99c), 0xe310);
    set(&mut ram, 0x99c, 0xe36e);
    host.calls.clear();
    async_step(&mut ram, &rom, &mut host);
    assert_eq!(
        host.syncs(),
        [
            S::SaveMapExploredIfElevator,
            S::LoadRoomHeader,
            S::LoadStateHeader,
            S::LoadMapExploredIfElevator,
            S::InitializeSpecialEffectsForNewRoom,
            S::LoadLevelScrollAndCre
        ]
    );
    assert_eq!(word(&ram, 0x99c), 0xe38e);
    for direction in 0..4 {
        set(&mut ram, 0x99c, 0xe38e);
        set(&mut ram, 0x791, direction);
        set(&mut ram, 0xb3, 0xffff);
        set(&mut ram, 0x925, 0x99);
        set(&mut ram, 0xb5, 0xffff);
        set(&mut ram, 0xb7, 0xffff);
        set(&mut ram, 0x7e9, 0xffff);
        async_step(&mut ram, &rom, &mut host);
        assert_eq!(word(&ram, 0xb3), if direction == 2 { 0 } else { 0xffff });
        assert_eq!(word(&ram, 0x925), if direction == 3 { 0x99 } else { 0 });
        assert_eq!(word(&ram, 0xb5) | word(&ram, 0xb7) | word(&ram, 0x7e9), 0);
    }
}

#[test]
fn tile_loading_preserves_rebase_copy_order_and_special_door_exception() {
    for exception in [false, true] {
        let (mut ram, rom) = state(0xe3c0);
        set(&mut ram, 0xaf6, 0x1220);
        set(&mut ram, 0xafa, 0x3408);
        set(&mut ram, 0x911, 0xfff0);
        set(&mut ram, 0x915, 0x100);
        set(&mut ram, 0x791, 3);
        set(&mut ram, 0x78d, if exception { 0x947a } else { 0xab4c });
        set(&mut ram, 0x7b3, 6);
        long(&mut ram, 0x7c3, 0xbac000);
        long(&mut ram, 0x7c6, 0x8fe000);
        let mut host = Probe {
            effect: Some(|ram, service| {
                if service == S::WaitUntilEndOfVblankAndEnableIrq {
                    assert_eq!(word(ram, 0xaf6), 0x10);
                    assert_eq!(word(ram, 0xb10), 0x10);
                    assert_eq!(word(ram, 0xafa), 0x108);
                    assert_eq!(word(ram, 0xa7), 22);
                }
            }),
            ..Probe::default()
        };
        async_step(&mut ram, &rom, &mut host);
        let mut expected = vec![S::WaitUntilEndOfVblankAndEnableIrq];
        if !exception {
            expected.push(S::DecompressToMem { source: 0xb98000, destination: 0x7000 });
        }
        expected.extend([
            S::DecompressToMem { source: 0xbac000, destination: 0x2000 },
            S::DecompressToMem { source: 0x8fe000, destination: 0xc200 },
            S::CopyToVramNow { vram_dst: 0, source: 0x7e2000, size: 0x2000 },
            S::CopyToVramNow { vram_dst: 0x1000, source: 0x7e4000, size: 0x2000 },
            S::CopyToVramNow { vram_dst: 0x2000, source: 0x7e6000, size: 0x1000 },
        ]);
        if !exception {
            expected.extend([
                S::CopyToVramNow { vram_dst: 0x2800, source: 0x7e7000, size: 0x1000 },
                S::CopyToVramNow { vram_dst: 0x3000, source: 0x7e8000, size: 0x2000 },
                S::CopyToVramNow { vram_dst: 0x4000, source: 0x9ab200, size: 0x1000 },
            ]);
        }
        assert_eq!(host.syncs(), expected);
        assert_eq!(word(&ram, 0xa7), 16);
        assert_eq!(word(&ram, 0x99c), 0xe4a9);
    }
}

#[test]
fn final_adjustment_restores_irq_and_clears_only_declared_hdma_buffers() {
    for (direction, x, expected_x, irq, code) in [(0, 0x11, 0x27, 0, 7), (0x8000, 0xe7, 0xd8, 12, 0)] {
        let (mut ram, rom) = state(0xe6a2);
        set(&mut ram, 0xaf6, x);
        set(&mut ram, 0xafa, 0x11);
        set(&mut ram, 0xa9, irq);
        set(&mut ram, 0xe16, 1);
        set(&mut ram, 0x799, direction);
        set(&mut ram, 0x51, 0xa500);
        ram[0x9bff..0xa001].fill(0xaa);
        let mut host = Probe::default();
        async_step(&mut ram, &rom, &mut host);
        assert_eq!(word(&ram, 0xaf6), expected_x);
        assert_eq!(word(&ram, 0xafa), 0x27);
        assert_eq!(word(&ram, 0xa7), if irq == 0 { 4 } else { irq });
        assert!(ram[0x9c00..0xa000].iter().all(|&b| b == 0));
        assert_eq!(ram[0x9bff], 0xaa);
        assert_eq!(ram[0xa000], 0xaa);
        assert_eq!(word(&ram, 0x51), 0xa51f);
        assert_eq!(
            host.calls,
            [Call::Sync(S::PointlessFunctionStupidToo), Call::Samus(code), Call::Sync(S::SetLiquidPhysicsType)]
        );
        assert_eq!(word(&ram, 0x99c), 0xe737);
        assert_eq!(word(&ram, 0x998), 11);
    }
}

#[test]
fn sound_wait_is_modulo_sixteen_and_music_advances_only_after_actual_empty_query() {
    let (mut ram, rom) = state(0xe29e);
    ram[0x643] = 16;
    let mut host = Probe::default();
    async_step(&mut ram, &rom, &mut host);
    assert_eq!(word(&ram, 0x99c), 0xe2db);
    set(&mut ram, 0x99c, 0xe659);
    async_step(&mut ram, &rom, &mut host);
    assert_eq!(word(&ram, 0x99c), 0xe664);
    host.calls.clear();
    host.music = [0xfe, 0].into();
    host.effect = Some(|ram, service| {
        if service == S::LoadNewMusicTrackIfChanged {
            assert_eq!(word(ram, 0x99c), 0xe6a2);
        }
    });
    async_step(&mut ram, &rom, &mut host);
    assert_eq!(word(&ram, 0x99c), 0xe664);
    assert!(host.syncs().is_empty());
    async_step(&mut ram, &rom, &mut host);
    assert_eq!(host.calls, [Call::Music(0xfe), Call::Music(0), Call::Sync(S::LoadNewMusicTrackIfChanged)]);
}

#[test]
fn complete_image_and_native_mirror_are_admitted_but_truncation_emits_nothing() {
    let (ram, rom) = state(0xe29e);
    for extent in [0x300000, 0x400000] {
        let mut actual = ram.clone();
        actual[0x643] = 16;
        let mut host = Probe::default();
        assert_eq!(async_step(&mut actual, &rom[..extent], &mut host), 0);
        assert_eq!(word(&actual, 0x99c), 0xe2db);
        assert!(!host.observations.is_empty());
    }
    let mut actual = ram.clone();
    let mut host = Probe::default();
    assert!(run(Entry::LoadingNextRoomAsync, &mut actual, &rom[..0x2fffff], &mut host).is_err());
    assert_eq!(actual, ram);
    assert!(host.calls.is_empty() && host.observations.is_empty());
}

#[test]
fn refused_or_invalid_admission_emits_nothing_and_owned_service_failure_never_refuses() {
    let (mut ram, rom) = state(0xbeef);
    let before = ram.clone();
    let mut host = Probe::default();
    assert_eq!(run(Entry::HitDoorBlock, &mut ram, &rom, &mut host).unwrap(), Outcome::Refused);
    assert_eq!(ram, before);
    assert!(host.observations.is_empty() && host.calls.is_empty());
    set(&mut ram, 0x99c, 0xe4a9);
    ram[0x612] = 3;
    let before = ram.clone();
    assert!(run(Entry::LoadingNextRoomAsync, &mut ram, &rom, &mut host).is_err());
    assert_eq!(ram, before);
    assert!(host.observations.is_empty() && host.calls.is_empty());
    assert!(run(Entry::LoadingNextRoom, &mut ram, &[], &mut host).is_err());
    assert!(host.observations.is_empty());
    host.fail = Some(S::EnemyMain);
    assert_eq!(
        run(Entry::LoadingNextRoom, &mut ram, &rom, &mut host).unwrap_err(),
        "deliberate native service failure"
    );
    assert_eq!(word(&ram, 0x797), 1);
    assert_eq!(host.observations.len(), 1);
    assert_eq!(host.calls, [Call::Sync(S::DetermineWhichEnemiesToProcess), Call::Sync(S::EnemyMain)]);
}
