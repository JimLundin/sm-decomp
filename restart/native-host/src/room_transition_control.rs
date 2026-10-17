//! Direct native fixtures with injected WRAM; not a gameplay replay or verdict.
use crate::{
    Game, HookOutcome, RoomTransitionCoroutineService as Coroutine, RoomTransitionEntry as Entry,
    RoomTransitionObservation as Observation, RoomTransitionOutcome as Outcome, RoomTransitionRoutine as Routine,
    RoomTransitionSyncService as Sync, prepare,
};
use libloading::Library;
use sha2::{Digest, Sha256};
use std::path::Path;

fn word(ram: &mut [u8], at: usize, value: u16) {
    ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn read_word(ram: &[u8], at: usize) -> u16 {
    u16::from_le_bytes(ram[at..at + 2].try_into().unwrap())
}
fn enter(routine: Routine, operand: u16) -> Observation {
    Observation::Enter { routine, operand }
}
fn done(routine: Routine, result: u16) -> Observation {
    Observation::Return { routine, result }
}

/// Run one complete injected control in a dedicated native child.
pub fn run(archive: &Path, rom: &Path, out: &Path, fixture: &Path, mode: &str) -> Result<(), String> {
    let mode = mode.to_owned();
    if ![
        "prepare",
        "off",
        "refuse",
        "pending-off",
        "pending-refuse",
        "raw-pending",
        "service",
        "child",
        "dirty-refuse",
        "panic",
        "observe-refuse",
        "wrong-ram-refuse",
        "bad-service",
        "bad-observer",
        "recursive",
        "child-outside-service",
        "sibling",
    ]
    .contains(&mode.as_str())
    {
        return Err(format!("unsupported room-transition control mode {mode}"));
    }
    let mut ram =
        if mode == "prepare" { Vec::new() } else { crate::control_fixture::read(fixture, "room-transition")? };
    let prepared = prepare(archive, out)?;
    println!("library={} reused={}", prepared.library.display(), prepared.reused);
    if mode == "prepare" {
        return Ok(());
    }
    let mut game = Game::boot(&prepared.library, rom)?;
    // SAFETY: preparation authenticates the library defining these diagnostic
    // ABI signatures. Library is retained through all calls; commit synchronously
    // copies a complete bounded128KiB allocation. Services exchange distinct
    // private WRAM copies with ordered copyback. All native state and callbacks
    // run on this dedicated child's sole thread; terminal cases exit the child.
    unsafe {
        let library = Library::new(&prepared.library).map_err(|e| e.to_string())?;
        let commit: unsafe extern "C" fn(*const u8) = *library
            .get(b"sm_native_commit_wram\0")
            .map_err(|e| format!("room-transition control symbol lookup: {e}"))?;
        let begin: unsafe extern "C" fn() = *library
            .get(b"sm_native_begin_step\0")
            .map_err(|e| format!("room-transition control symbol lookup: {e}"))?;
        let root9: unsafe extern "C" fn() -> u8 = *library
            .get(b"sm_native_room_transition_9\0")
            .map_err(|e| format!("room-transition control symbol lookup: {e}"))?;
        let root11: unsafe extern "C" fn() -> u8 = *library
            .get(b"sm_native_room_transition_11\0")
            .map_err(|e| format!("room-transition control symbol lookup: {e}"))?;
        let scroll: unsafe extern "C" fn(u32) -> i32 = *library
            .get(b"sm_native_scroll_dispatch\0")
            .map_err(|e| format!("room-transition control symbol lookup: {e}"))?;
        let ceres: unsafe extern "C" fn(u32, u16) = *library
            .get(b"sm_native_ceres_door_ai\0")
            .map_err(|e| format!("room-transition control symbol lookup: {e}"))?;
        word(&mut ram, 0x998, 9);
        word(&mut ram, 0x99c, 0xe17d);
        word(&mut ram, 0x78d, 0xab64);
        word(&mut ram, 0xe16, 0);
        ram[0x612] = 0;
        let entry9 = matches!(mode.as_str(), "off" | "refuse");
        if !entry9 {
            word(&mut ram, 0x998, 11);
            word(&mut ram, 0x99c, 0xe4a9);
            word(&mut ram, 0x931, 0);
            ram[0x612] = 1;
            ram[0x782] = 0;
        }
        commit(ram.as_ptr());
        begin();
        if mode == "child" || mode == "sibling" {
            let sibling = mode == "sibling";
            game.set_scroll_hook(move |_, ram, _| {
                assert_eq!(ram[0x1234], 0xa1, "child sees parent writes before service");
                if sibling {
                    ceres(1, 0);
                    unreachable!();
                }
                ram[0x1234] = 0xb7;
                Ok(HookOutcome::Handled)
            })?;
        }
        if !matches!(mode.as_str(), "off" | "pending-off") {
            let mode = mode.clone();
            game.set_room_transition_hook(move |entry, ram, _, services| {
                if mode == "refuse" || mode == "pending-refuse" {
                    return Ok(Outcome::Refused);
                }
                assert_eq!(entry, Entry::LoadingNextRoomAsync);
                if mode == "dirty-refuse" {
                    ram[0] ^= 1;
                    return Ok(Outcome::Refused);
                }
                if mode == "panic" {
                    panic!("deliberate room-transition callback panic");
                }
                if mode == "wrong-ram-refuse" {
                    let mut unrelated = ram.to_vec();
                    assert!(services.coroutine(&mut unrelated, Coroutine::WaitForNmi).is_err());
                    return Ok(Outcome::Refused);
                }
                services.observe(enter(Routine::LoadingNextRoomAsync, 0))?;
                if mode == "observe-refuse" {
                    return Ok(Outcome::Refused);
                }
                if mode == "recursive" {
                    root9();
                    unreachable!();
                }
                if mode == "child-outside-service" {
                    scroll(2);
                    unreachable!();
                }
                if mode == "bad-observer" {
                    services.observe(done(Routine::HitDoorBlock, 0))?;
                    unreachable!();
                }
                if mode == "bad-service" {
                    services.sync(ram, Sync::DecompressToMem { source: 0x1000000, destination: 0x2000 })?;
                    unreachable!();
                }
                let result = if mode == "service" {
                    let result = services.coroutine(ram, Coroutine::WaitForNmi)?;
                    assert_eq!(result, 1);
                    assert_eq!(ram[0x782], 1, "native pending state copied back synchronously");
                    result
                } else if mode == "child" || mode == "sibling" {
                    ram[0x1234] = 0xa1;
                    services.sync(ram, Sync::CalculateLayer2PosAndScrollsWhenScrolling)?;
                    assert_eq!(ram[0x1234], 0xb7, "child commit copied back into parent");
                    ram[0x1235] = 0xc2;
                    0
                } else {
                    assert_eq!(mode, "raw-pending");
                    ram[0x612] = 1;
                    ram[0x1234] = 0xd3;
                    0x7f
                };
                services.observe(done(Routine::LoadingNextRoomAsync, result.into()))?;
                Ok(Outcome::Handled(result))
            })?;
        }
        let first = if entry9 { root9() } else { root11() };
        if entry9 {
            assert_eq!(first, 0);
            assert_eq!(read_word(game.wram(), 0x998), 11);
            assert_eq!(read_word(game.wram(), 0x99c), 0xe29e);
            let counts = game.room_transition_counters();
            assert_eq!(counts.c_entries, [1, 0, 0]);
            for id in [0, 1, 3, 4] {
                assert_eq!(counts.routine_c_entries[id], 1);
            }
            assert_eq!(counts.routine_c_entries.iter().sum::<u64>(), 4);
            assert_eq!(counts.routine_c_completed, counts.routine_c_entries);
            for id in [0, 1, 2, 3, 4, 6, 7] {
                assert_eq!(counts.service_requests[id], 1);
            }
            println!("same_invocation_state9_to_state11=true private_state10_calls=1 scheduled_state10_calls=0");
        } else if mode == "pending-off" || mode == "pending-refuse" {
            assert_eq!(first, 1);
            assert_eq!(game.wram()[0x612], 1);
            assert_eq!(game.wram()[0x782], 1);
            let counts = game.room_transition_counters();
            assert_eq!(counts.service_requests[51], 1);
            assert_eq!(counts.service_pending[51], 1);
            assert_eq!(counts.service_requests.iter().sum::<u64>(), 1, "one-time loading is skipped on resume");
            ram.copy_from_slice(game.wram());
            word(&mut ram, 0x931, 0x8000);
            commit(ram.as_ptr());
            assert_eq!(root11(), 0);
            assert_eq!(game.wram()[0x782], 0, "pending NMI resumes despite completed IRQ flag");
            assert_eq!(game.wram()[0x612], 0);
            assert_eq!(read_word(game.wram(), 0x99c), 0xe659);
            let counts = game.room_transition_counters();
            assert_eq!(counts.service_requests[51], 2);
            assert_eq!(counts.service_requests[38], 1);
            assert_eq!(counts.service_requests[52], 1);
            assert_eq!(counts.root_pending, [0, 0, 1]);
            assert_eq!(counts.root_resumed, [0, 0, 2]);
            println!("pending_then_irq_completed_resume=true raw_returns=[1,0]");
        } else if mode == "raw-pending" {
            assert_eq!(first, 0x7f);
            assert_eq!(game.wram()[0x1234], 0xd3);
            assert_eq!(game.room_transition_counters().root_pending, [0, 0, 1]);
            println!("raw_pending_byte=127 continuation_committed=true");
        } else if mode == "child" {
            assert_eq!(&game.wram()[0x1234..0x1236], &[0xb7, 0xc2]);
            let counts = game.room_transition_counters();
            assert_eq!(counts.child_entries[10], [0, 1, 0]);
            assert_eq!(counts.child_completions[10], [0, 1, 0]);
            assert_eq!(counts.foreign_requests[10], 1);
            assert_eq!(counts.foreign_completions[10], 1);
            println!("parent_service_rust_child_copyback=true");
        } else {
            assert_eq!(mode, "service");
            assert_eq!(first, 1);
            assert_eq!(game.room_transition_counters().foreign_completions[51], 1);
            println!("synchronous_native_pending_copyback=true");
        }
        assert_eq!(game.state().host_steps, 0, "injected room control must not advance host steps");
        let mut hash = Sha256::new();
        hash.update(game.wram());
        hash.update(game.effects());
        println!("behavior_sha256={:x}", hash.finalize());
        println!("counters={:?}", game.room_transition_counters());
    }
    Ok(())
}
