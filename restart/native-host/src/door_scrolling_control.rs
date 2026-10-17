//! Injected direct native controls, with zero host steps and no live-route claim.
use crate::{
    DoorScrollingEntry as Entry, DoorScrollingObservation as Observation, DoorScrollingRoutine as Routine, Game,
    HookOutcome as Outcome, RoomTransitionObservation as ParentObservation, RoomTransitionOutcome as ParentOutcome,
    RoomTransitionRoutine as ParentRoutine, RoomTransitionSyncService as ParentService, prepare,
};
use libloading::Library;
use sha2::{Digest, Sha256};
use std::{cell::RefCell, path::Path, rc::Rc};
fn get(ram: &[u8], at: usize) -> u16 {
    u16::from_le_bytes(ram[at..at + 2].try_into().unwrap())
}
fn put(ram: &mut [u8], at: usize, value: u16) {
    ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn enter(routine: Routine, ram: &[u8]) -> Observation {
    Observation::Enter { routine, counter: get(ram, 0x925) }
}
fn done(routine: Routine, ram: &[u8]) -> Observation {
    Observation::Return { routine, result: 0, counter: get(ram, 0x925) }
}
/// Run one injected control in a dedicated native process.
pub fn run(archive: &Path, rom: &Path, out: &Path, fixture: &Path, mode: &str) -> Result<(), String> {
    if !matches!(
        mode,
        "prepare"
            | "off"
            | "refuse"
            | "c-right"
            | "c-vertical"
            | "services"
            | "child"
            | "coordinator"
            | "dirty-refuse"
            | "panic"
            | "error"
            | "wrong-ram-refuse"
            | "observe-refuse"
            | "unbalanced"
            | "bad-observer"
            | "recursive"
            | "child-outside-service"
            | "request-refuse"
            | "sibling"
            | "parent-outside-service"
    ) {
        return Err(format!("unsupported door-scrolling control mode {mode}"));
    }
    let fixture = if mode == "prepare" { Vec::new() } else { crate::control_fixture::read(fixture, "door-scrolling")? };
    let mode = mode.to_owned();
    let prepared = prepare(archive, out)?;
    println!("library={} reused={}", prepared.library.display(), prepared.reused);
    if mode == "prepare" {
        return Ok(());
    }
    let mut game = Game::boot(&prepared.library, rom)?;
    // SAFETY: preparation authenticates the library defining these exact ABI
    // signatures. Library remains alive through every diagnostic call. Commit
    // synchronously copies a complete bounded128KiB allocation; child services
    // exchange distinct private copies with ordered synchronous copyback.
    // This dedicated child has one thread using process-global native state;
    // terminal ownership controls exit the child and do not resume C fallback.
    unsafe {
        let library = Library::new(&prepared.library).map_err(|e| e.to_string())?;
        let commit: unsafe extern "C" fn(*const u8) = *library
            .get(b"sm_native_commit_wram\0")
            .map_err(|e| format!("door-scrolling control symbol lookup: {e}"))?;
        let begin: unsafe extern "C" fn() = *library
            .get(b"sm_native_begin_step\0")
            .map_err(|e| format!("door-scrolling control symbol lookup: {e}"))?;
        let roots: [unsafe extern "C" fn(); 3] = [
            *library.get(b"FixDoorsMovingUp\0").map_err(|e| format!("door-scrolling control symbol lookup: {e}"))?,
            *library
                .get(b"DoorTransitionScrollingSetup\0")
                .map_err(|e| format!("door-scrolling control symbol lookup: {e}"))?,
            *library
                .get(b"Irq_FollowDoorTransition\0")
                .map_err(|e| format!("door-scrolling control symbol lookup: {e}"))?,
        ];
        let parent: unsafe extern "C" fn() -> u8 = *library
            .get(b"sm_native_room_transition_11\0")
            .map_err(|e| format!("door-scrolling control symbol lookup: {e}"))?;
        let scroll: unsafe extern "C" fn(u32) -> i32 = *library
            .get(b"sm_native_scroll_dispatch\0")
            .map_err(|e| format!("door-scrolling control symbol lookup: {e}"))?;
        let projectile: unsafe extern "C" fn(u32) -> i32 =
            *library.get(b"sm_native_dispatch\0").map_err(|e| format!("door-scrolling control symbol lookup: {e}"))?;
        let mut hash = Sha256::new();
        if mode == "off" || mode == "refuse" {
            if mode == "refuse" {
                game.set_door_scrolling_hook(|_, _, _| Ok(Outcome::Refused))?;
            }
            let mut cases = 0;
            for (entry, root) in roots.iter().enumerate() {
                for direction in 0..4 {
                    for counter in [0, 1, 4, 5, 55, 56, 57, 62, 63, 64, 65535] {
                        let mut ram = fixture.clone();
                        put(&mut ram, 0x791, direction);
                        put(&mut ram, 0x925, counter);
                        put(&mut ram, 0x911, 64);
                        put(&mut ram, 0x915, 256);
                        put(&mut ram, 0x917, 64);
                        put(&mut ram, 0x919, 256);
                        put(&mut ram, 0x927, if direction == 0 { 256 } else { 0 });
                        put(&mut ram, 0x929, if direction == 3 { 0 } else { 256 });
                        commit(ram.as_ptr());
                        begin();
                        root();
                        hash.update(game.wram());
                        hash.update(game.effects());
                        cases += 1;
                        assert_eq!(game.door_scrolling_counters().c_entries[entry], (cases - entry * 44) as u64);
                    }
                }
            }
            assert_eq!(cases, 132, "complete door-scrolling fixture matrix");
            let counts = game.door_scrolling_counters();
            assert_eq!(counts.c_entries, [44; 3]);
            assert_eq!(counts.routine_c_entries, counts.routine_c_completed);
            assert!(counts.routine_c_entries.iter().all(|n| *n > 0));
            assert_eq!(counts.service_requests, counts.service_completions);
            assert_eq!(counts.foreign_requests, [0; 4]);
            println!("direct_c_cases={cases} all13_routines=true real_scroll_service=true");
        } else if mode == "c-right" || mode == "c-vertical" {
            let seen = Rc::new(RefCell::new(Vec::new()));
            let child_seen = seen.clone();
            let vertical = mode == "c-vertical";
            game.set_scroll_hook(move |entry, ram, _| {
                assert_eq!(entry, crate::ScrollEntry::UpdateScrollVarsUpdateMap);
                child_seen.borrow_mut().push(get(ram, 0x925));
                if vertical {
                    assert_eq!(get(ram, 0x915), if get(ram, 0x791) == 2 { 241 } else { 240 });
                    for at in [0x915, 0x919, 0xb3, 0xb7] {
                        put(ram, at, 0x7777);
                    }
                    put(ram, 0x911, 0x3333);
                    put(ram, 0xb1, 0x4444);
                    put(ram, 0x8f7, 0x5555);
                } else {
                    put(ram, 0x917, 0x3333);
                    put(ram, 0x919, 0x4444);
                    put(ram, 0xb1, 0x5555);
                    put(ram, 0xb3, 0x6666);
                    put(ram, 0xb5, 0x7777);
                    put(ram, 0xb7, 0x8888);
                }
                put(ram, 0x925, 0x1111);
                ram[0x1234] = 0xb7;
                Ok(Outcome::Handled)
            })?;
            let directions: &[u16] = if vertical { &[2, 3] } else { &[0] };
            for direction in directions {
                let mut ram = fixture.clone();
                put(&mut ram, 0x791, *direction);
                put(&mut ram, 0x925, if vertical { 0 } else { 63 });
                put(&mut ram, 0x911, 64);
                put(&mut ram, 0x915, 256);
                put(&mut ram, 0x917, 96);
                put(&mut ram, 0x919, 272);
                put(&mut ram, 0x927, 0x123);
                put(&mut ram, 0x929, 0x234);
                put(&mut ram, 0x931, 0x12);
                commit(ram.as_ptr());
                begin();
                roots[2]();
                let out = game.wram();
                if vertical {
                    assert_eq!(get(out, 0x925), 1, "saved counter survives child mutation");
                    for at in [0x915, 0x919, 0xb3, 0xb7] {
                        assert_eq!(get(out, at), get(&ram, at));
                    }
                    assert_eq!(get(out, 0x911), 0x3333);
                    assert_eq!(get(out, 0xb1), 0x4444);
                    assert_eq!(get(out, 0x8f7), 0x5555, "child block work remains");
                    assert_eq!(get(out, 0x931), 0x12);
                } else {
                    // Second child sees64 before deliberately changing the counter.
                    assert_eq!(&*seen.borrow(), &[63, 64]);
                    assert_eq!(get(out, 0x925), 0x1111);
                    assert_eq!(get(out, 0x911), 0x123);
                    assert_eq!(get(out, 0x915), 0x234);
                    assert_eq!(get(out, 0x917), 0x3333);
                    assert_eq!(get(out, 0x919), 0x4444);
                    for (at, value) in [(0xb1, 0x5555), (0xb3, 0x6666), (0xb5, 0x7777), (0xb7, 0x8888)] {
                        assert_eq!(get(out, at), value);
                    }
                    assert_eq!(get(out, 0x931), 0x8012);
                }
                assert_eq!(out[0x1234], 0xb7);
                hash.update(out);
                hash.update(game.effects());
            }
            println!(
                "actual_c_parent_injected_rust_child=true selective_state_and_saved_locals=true child_counters={:?}",
                seen.borrow()
            );
        } else {
            let mut ram = fixture.clone();
            put(&mut ram, 0x791, 0);
            put(&mut ram, 0x925, 63);
            put(&mut ram, 0x911, 64);
            put(&mut ram, 0x915, 256);
            ram[0x91b] = 1;
            ram[0x91c] = 1;
            commit(ram.as_ptr());
            begin();
            let child = matches!(mode.as_str(), "child" | "coordinator" | "sibling");
            if child {
                let sibling = mode == "sibling";
                game.set_scroll_hook(move |_, ram, _| {
                    assert_eq!(ram[0x1234], 0xa1);
                    if sibling {
                        projectile(0);
                        unreachable!();
                    }
                    ram[0x1234] = 0xb7;
                    Ok(Outcome::Handled)
                })?;
            }
            if mode == "coordinator" || mode == "parent-outside-service" {
                let outside = mode == "parent-outside-service";
                game.set_room_transition_hook(move |_, ram, _, services| {
                    services.observe(ParentObservation::Enter {
                        routine: ParentRoutine::LoadingNextRoomAsync,
                        operand: 0,
                    })?;
                    ram[0x1233] = 0xd1;
                    if outside {
                        roots[1]();
                        unreachable!();
                    }
                    services.sync(ram, ParentService::DoorTransitionScrollingSetup)?;
                    assert_eq!(&ram[0x1233..0x1236], &[0xd1, 0xb7, 0xc2]);
                    ram[0x1236] = 0xe3;
                    services.observe(ParentObservation::Return {
                        routine: ParentRoutine::LoadingNextRoomAsync,
                        result: 0,
                    })?;
                    Ok(ParentOutcome::Handled(0))
                })?;
            }
            let callback_mode = mode.clone();
            game.set_door_scrolling_hook(move |entry, ram, services| {
                let routine = match entry {
                    Entry::FixDoorsMovingUp => Routine::FixDoorsMovingUp,
                    Entry::ScrollingSetup => Routine::ScrollingSetup,
                    Entry::FollowTransition => Routine::FollowTransition,
                };
                match callback_mode.as_str() {
                    "dirty-refuse" => {
                        ram[0] ^= 1;
                        return Ok(Outcome::Refused);
                    }
                    "panic" => panic!("deliberate callback panic"),
                    "error" => return Err("deliberate callback error".into()),
                    "wrong-ram-refuse" => {
                        assert!(services.calculate_blocks(&mut ram.to_vec()).is_err());
                        return Ok(Outcome::Refused);
                    }
                    _ => {}
                }
                services.observe(enter(routine, ram))?;
                match callback_mode.as_str() {
                    "observe-refuse" => return Ok(Outcome::Refused),
                    "unbalanced" => return Ok(Outcome::Handled),
                    "bad-observer" => {
                        services.observe(done(Routine::MoveRight, ram))?;
                        unreachable!();
                    }
                    "recursive" => {
                        roots[2]();
                        unreachable!();
                    }
                    "child-outside-service" => {
                        scroll(1);
                        unreachable!();
                    }
                    "request-refuse" => {
                        services.calculate_blocks(ram)?;
                        return Ok(Outcome::Refused);
                    }
                    "services" => {
                        let before = [get(ram, 0x917), get(ram, 0x919)];
                        assert_eq!(services.calculate_layer2_x(ram)?, 1);
                        assert_eq!(services.calculate_layer2_y(ram)?, 1);
                        assert_eq!([get(ram, 0x917), get(ram, 0x919)], before);
                        services.calculate_blocks(ram)?;
                        assert_eq!(get(ram, 0x8f7), get(ram, 0x911) >> 4);
                        services.update_scroll_vars(ram)?;
                    }
                    "child" | "coordinator" | "sibling" => {
                        if callback_mode == "coordinator" {
                            assert_eq!(ram[0x1233], 0xd1);
                        }
                        ram[0x1234] = 0xa1;
                        services.update_scroll_vars(ram)?;
                        assert_eq!(ram[0x1234], 0xb7);
                        ram[0x1235] = 0xc2;
                    }
                    other => return Err(format!("unknown mode {other}")),
                }
                services.observe(done(routine, ram))?;
                Ok(Outcome::Handled)
            })?;
            if mode == "coordinator" || mode == "parent-outside-service" {
                assert_eq!(parent(), 0);
            } else {
                roots[2]();
            }
            let counts = game.door_scrolling_counters();
            if mode == "services" {
                assert_eq!(counts.foreign_requests, [1; 4]);
                assert_eq!(counts.foreign_requests, counts.foreign_completions);
                println!("four_native_services=true preserved_math_return_bytes=true");
            } else {
                assert_eq!(counts.child_entries, [0, 0, 0, 1]);
                assert_eq!(counts.child_entries, counts.child_completions);
                assert_eq!(&game.wram()[0x1234..0x1236], &[0xb7, 0xc2]);
                if mode == "coordinator" {
                    assert_eq!(counts.coordinator_entries, [0, 1, 0]);
                    assert_eq!(game.wram()[0x1236], 0xe3);
                    assert_eq!(game.room_transition_counters().child_entries[18], [0; 3]);
                    println!("three_distinct_wram_copies=true no_double_coordinator_child_attribution=true");
                }
                println!("synchronous_rust_child_copyback=true");
            }
            hash.update(game.wram());
            hash.update(game.effects());
        }
        assert_eq!(game.state().host_steps, 0);
        println!("behavior_sha256={:x}", hash.finalize());
        println!("counters={:?}", game.door_scrolling_counters());
    }
    Ok(())
}
