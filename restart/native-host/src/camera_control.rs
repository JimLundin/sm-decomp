//! Bounded native camera fixtures and ownership controls; never a live replay.
use crate::{CameraObservation as Observation, CameraRoutine as Routine, Game, HookOutcome, prepare};
use libloading::Library;
use sha2::{Digest, Sha256};
use std::path::Path;
fn get(ram: &[u8], at: usize) -> u16 {
    u16::from_le_bytes(ram[at..at + 2].try_into().unwrap())
}
fn put(ram: &mut [u8], at: usize, value: u16) {
    ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn enter(routine: Routine) -> Observation {
    Observation::Enter { routine, operand: 0 }
}
fn done(routine: Routine) -> Observation {
    Observation::Return { routine }
}
fn fixtures(input: &[u8]) -> Vec<(&'static str, Vec<u8>)> {
    let mut base = input.to_vec();
    for (at, value) in [
        (0x7a9, 4),
        (0x7ab, 4),
        (0x7ad, 112),
        (0x7af, 160),
        (0x7e9, 0),
        (0x90f, 0xf000),
        (0x911, 256),
        (0x913, 0xc000),
        (0x915, 256),
        (0x941, 0),
        (0xa52, 0),
        (0xa78, 0),
        (0xaf6, 400),
        (0xaf8, 0x1200),
        (0xafa, 400),
        (0xafc, 0x3400),
        (0xb10, 399),
        (0xb12, 0xf000),
        (0xb14, 399),
        (0xb16, 0xe000),
        (0xb36, 0),
        (0xb4a, 0),
        (0xcf8, 0),
        (0xda2, 1),
        (0xda4, 0),
        (0xda6, 1),
        (0xda8, 0),
    ] {
        put(&mut base, at, value);
    }
    base[0xa1e] = 4;
    base[0xa1f] = 0;
    // Well-formed bounded tables; these controls never ask C to evaluate an
    // unsupported dimension, target, table index or out-of-range scroll access.
    base[0xcd20..0xce20].fill(1);
    let mut result = Vec::new();
    for facing in [4, 8] {
        for y_direction in [0, 1] {
            for camera_index in [0, 1, 6, 7] {
                let mut ram = base.clone();
                ram[0xa1e] = facing;
                put(&mut ram, 0xb36, y_direction);
                put(&mut ram, 0x941, camera_index);
                result.push(("moving", ram));
            }
        }
    }
    let mut stationary = base.clone();
    put(&mut stationary, 0xb10, 400);
    put(&mut stationary, 0xb14, 400);
    result.push(("stationary", stationary.clone()));
    let mut slow_right = base.clone();
    put(&mut slow_right, 0xcf8, 1);
    put(&mut slow_right, 0xaf6, 512);
    put(&mut slow_right, 0xafa, 500);
    result.push(("slow-right-down", slow_right));
    let mut slow_left = base.clone();
    put(&mut slow_left, 0xcf8, 1);
    put(&mut slow_left, 0xaf6, 300);
    put(&mut slow_left, 0xafa, 300);
    result.push(("slow-left-up", slow_left));
    let mut signed_high = base.clone();
    put(&mut signed_high, 0xcf8, 1);
    put(&mut signed_high, 0xaf6, 0x8000);
    result.push(("slow-signed-high", signed_high));
    let mut frozen = stationary.clone();
    put(&mut frozen, 0xa78, 1);
    result.push(("frozen-autoscrollers", frozen));
    put(&mut stationary, 0x7e9, 0x9589);
    put(&mut stationary, 0x915, 300);
    result.push(("spore-finish", stationary));
    result
}
/// Run one complete camera control in a dedicated native process.
pub fn run(archive: &Path, rom: &Path, out: &Path, fixture: &Path, mode: &str) -> Result<(), String> {
    if !matches!(
        mode,
        "prepare"
            | "off"
            | "refuse"
            | "observer"
            | "dirty-refuse"
            | "panic"
            | "error"
            | "bad-operand"
            | "observe-refuse"
            | "error-after-observe"
            | "unbalanced"
            | "bad-parent"
            | "bad-return"
            | "recursive"
            | "register"
            | "scroll-out"
            | "projectile-out"
            | "ceres-out"
            | "coordinator-out"
            | "directional-out"
            | "scroll-in"
            | "projectile-in"
            | "ceres-in"
            | "coordinator-in"
            | "directional-in"
    ) {
        return Err(format!("unsupported camera control mode {mode}"));
    }
    let fixture = if mode == "prepare" { Vec::new() } else { crate::control_fixture::read(fixture, "camera")? };
    let mode = mode.to_owned();
    let prepared = prepare(archive, out)?;
    println!("library={} reused={}", prepared.library.display(), prepared.reused);
    if mode == "prepare" {
        return Ok(());
    }
    let mut game = Game::boot(&prepared.library, rom)?;
    // SAFETY: preparation authenticates the library implementing the exact ABI
    // signatures below. The retained Library keeps symbols alive through calls.
    // Commit synchronously copies a checked complete 128KiB allocation. Native
    // canonical views are copied before mutation; callbacks and native state
    // run on this dedicated child's sole thread. Fatal controls exit the child.
    unsafe {
        let library = Library::new(&prepared.library).map_err(|e| e.to_string())?;
        let commit: unsafe extern "C" fn(*const u8) =
            *library.get(b"sm_native_commit_wram\0").map_err(|e| format!("camera control symbol lookup: {e}"))?;
        let canonical: unsafe extern "C" fn() -> *const u8 =
            *library.get(b"sm_native_wram\0").map_err(|e| format!("camera control symbol lookup: {e}"))?;
        let begin: unsafe extern "C" fn() =
            *library.get(b"sm_native_begin_step\0").map_err(|e| format!("camera control symbol lookup: {e}"))?;
        let root: unsafe extern "C" fn() =
            *library.get(b"MainScrollingRoutine\0").map_err(|e| format!("camera control symbol lookup: {e}"))?;
        let scroll: unsafe extern "C" fn(u32) -> i32 =
            *library.get(b"sm_native_scroll_dispatch\0").map_err(|e| format!("camera control symbol lookup: {e}"))?;
        let projectile: unsafe extern "C" fn(u32) -> i32 =
            *library.get(b"sm_native_dispatch\0").map_err(|e| format!("camera control symbol lookup: {e}"))?;
        let ceres: unsafe extern "C" fn(u32, u16) =
            *library.get(b"sm_native_ceres_door_ai\0").map_err(|e| format!("camera control symbol lookup: {e}"))?;
        let coordinator: unsafe extern "C" fn() -> u8 = *library
            .get(b"sm_native_room_transition_11\0")
            .map_err(|e| format!("camera control symbol lookup: {e}"))?;
        let directional: unsafe extern "C" fn() =
            *library.get(b"Irq_FollowDoorTransition\0").map_err(|e| format!("camera control symbol lookup: {e}"))?;
        type RawHook = unsafe extern "C" fn(*mut std::ffi::c_void) -> i32;
        let set: unsafe extern "C" fn(Option<RawHook>, *mut std::ffi::c_void) =
            *library.get(b"sm_native_set_camera_hook\0").map_err(|e| format!("camera control symbol lookup: {e}"))?;
        let mut hash = Sha256::new();
        if mode == "off" || mode == "refuse" {
            if mode == "refuse" {
                game.set_camera_hook(|_, _| Ok(HookOutcome::Refused))?;
            }
            let mut cases = 0;
            for state in [8, 9] {
                for (name, mut ram) in fixtures(&fixture) {
                    put(&mut ram, 0x998, state);
                    commit(ram.as_ptr());
                    begin();
                    root();
                    for (previous, current) in [(0xb10, 0xaf6), (0xb12, 0xaf8), (0xb14, 0xafa), (0xb16, 0xafc)] {
                        assert_eq!(get(game.wram(), previous), get(&ram, current));
                    }
                    assert_eq!(get(game.wram(), 0x998), state);
                    if name == "spore-finish" {
                        assert_eq!(get(game.wram(), 0x915), 464);
                        assert_eq!(get(game.wram(), 0x913), 0xc000);
                    }
                    if name == "frozen-autoscrollers" || name == "slow-signed-high" {
                        assert_eq!(&game.wram()[0x90f..0x917], &ram[0x90f..0x917]);
                    }
                    let (effect_records, remainder) = game.effects().as_chunks::<16>();
                    assert!(remainder.is_empty(), "partial native camera effect record");
                    let events: Vec<[u32; 4]> = effect_records
                        .iter()
                        .map(|event| {
                            std::array::from_fn(|i| u32::from_le_bytes(event[i * 4..i * 4 + 4].try_into().unwrap()))
                        })
                        .collect();
                    assert_eq!(events[0], [39, state.into(), get(&ram, 0x79b).into(), get(&ram, 0xcf8).into()]);
                    assert!(
                        events.iter().all(|event| (39..=46).contains(&event[0])),
                        "pure camera has no gameplay/hardware effects"
                    );
                    hash.update(game.wram());
                    hash.update(game.effects());
                    cases += 1;
                }
            }
            let counts = game.camera_counters();
            assert_eq!(cases, 44, "complete camera fixture matrix");
            assert_eq!(counts.c_entries, cases);
            assert_eq!(counts.routine_c_entries, counts.routine_c_completed);
            assert!(counts.routine_c_entries.iter().all(|n| *n > 0));
            println!(
                "valid_c_cases={cases} state9_body_calls=22 all13_routines=true all_previous_words_published=true"
            );
        } else {
            let mut ram = fixtures(&fixture).remove(0).1;
            put(&mut ram, 0x998, 9);
            put(&mut ram, 0xe54, 0);
            put(&mut ram, 0xf78, 0xe23f);
            commit(ram.as_ptr());
            begin();
            let callback_mode = mode.clone();
            game.set_camera_hook(move |ram, observer| {
                match callback_mode.as_str() {
                    "dirty-refuse" => {
                        ram[0] ^= 1;
                        return Ok(HookOutcome::Refused);
                    }
                    "panic" => panic!("deliberate camera callback panic"),
                    "error" => return Err("deliberate camera callback error".into()),
                    "bad-operand" => {
                        observer.observe(Observation::Enter { routine: Routine::MainScrolling, operand: 1 })?;
                        unreachable!();
                    }
                    _ => {}
                }
                let before = std::slice::from_raw_parts(canonical(), 0x20000).to_vec();
                ram[0x1234] ^= 1;
                observer.observe(enter(Routine::MainScrolling))?;
                match callback_mode.as_str() {
                    "observe-refuse" => return Ok(HookOutcome::Refused),
                    "error-after-observe" => return Err("deliberate post-observation error".into()),
                    "unbalanced" => return Ok(HookOutcome::Handled),
                    "bad-parent" => {
                        observer.observe(enter(Routine::SporeSpawnFinish))?;
                        unreachable!();
                    }
                    "bad-return" => {
                        observer.observe(done(Routine::DistanceX))?;
                        unreachable!();
                    }
                    "recursive" => {
                        root();
                        unreachable!();
                    }
                    "register" => {
                        set(None, std::ptr::null_mut());
                        unreachable!();
                    }
                    "scroll-out" => {
                        scroll(0);
                        unreachable!();
                    }
                    "projectile-out" => {
                        projectile(0);
                        unreachable!();
                    }
                    "ceres-out" => {
                        ceres(0, 0);
                        unreachable!();
                    }
                    "coordinator-out" => {
                        coordinator();
                        unreachable!();
                    }
                    "directional-out" => {
                        directional();
                        unreachable!();
                    }
                    "observer" => {
                        observer.observe(enter(Routine::DistanceX))?;
                        assert_eq!(std::slice::from_raw_parts(canonical(), 0x20000), before);
                        observer.observe(done(Routine::DistanceX))?;
                    }
                    other => return Err(format!("unexpected camera mode {other}")),
                }
                observer.observe(done(Routine::MainScrolling))?;
                assert_eq!(
                    std::slice::from_raw_parts(canonical(), 0x20000),
                    before,
                    "observation does not synchronize private changes"
                );
                Ok(HookOutcome::Handled)
            })?;
            match mode.as_str() {
                "scroll-in" => {
                    game.set_scroll_hook(move |_, _, _| {
                        root();
                        unreachable!()
                    })?;
                    scroll(0);
                }
                "projectile-in" => {
                    game.set_projectile_hook(move |_, _, _, _| {
                        root();
                        unreachable!()
                    })?;
                    projectile(0);
                }
                "ceres-in" => {
                    game.set_ceres_door_hook(move |_, _, _, _, _, _| {
                        root();
                        unreachable!()
                    })?;
                    ceres(0, 0);
                }
                "coordinator-in" => {
                    game.set_room_transition_hook(move |_, _, _, _| {
                        root();
                        unreachable!()
                    })?;
                    coordinator();
                }
                "directional-in" => {
                    game.set_door_scrolling_hook(move |_, _, _| {
                        root();
                        unreachable!()
                    })?;
                    directional();
                }
                _ => root(),
            }
            assert_eq!(mode, "observer");
            assert_eq!(game.wram()[0x1234], ram[0x1234] ^ 1);
            let counts = game.camera_counters();
            assert_eq!(counts.rust_completed, 1);
            assert_eq!(counts.c_entries, 0);
            assert_eq!(counts.routine_rust_entries, counts.routine_rust_completed);
            assert_eq!(counts.routine_rust_entries[0], 1);
            assert_eq!(counts.routine_rust_entries[5], 1);
            hash.update(game.wram());
            hash.update(game.effects());
            println!("passive_observer_no_sync=true successful_full_state_commit=true visible_state9=true");
        }
        assert_eq!(game.state().host_steps, 0);
        println!("behavior_sha256={:x}", hash.finalize());
        println!("counters={:?}", game.camera_counters());
    }
    Ok(())
}
