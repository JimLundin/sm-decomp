//! Finite Morph actual-C delta over the existing private item fixture harness.
//! Real message presentation remains the fixed full-route C/Rust acceptance pair.
#[allow(dead_code)]
#[path = "item-instruction-host-control.rs"]
mod item_control;

use item_control::{
    Native, Preflight, Records, SEED_SHA, compare, compare_records, hash, install, observe, read, word,
};
use libloading::Library;
use sm_item_instructions::{Entry, Output, fixtures};
use sm_native_host::{
    Game, ItemInstructionEntry as NativeEntry, ItemInstructionOutcome as Outcome, ItemInstructionPhase as Phase,
    RoomTransitionCoroutineService as Coroutine, RoomTransitionObservation as TransitionObservation,
    RoomTransitionOutcome as TransitionOutcome, RoomTransitionRoutine as Routine, prepare,
};
use std::path::Path;

const SLOT: u16 = 68;
const BODY_CASES: usize = 31;
const REFUSAL_CASES: usize = 20;

fn body_pairs(game: &mut Game, native: Native, seed: &[u8], records: &Records) -> Result<(), String> {
    let cases = fixtures::morph_body_fixtures(seed, game.rom())?;
    assert_eq!(cases.len(), BODY_CASES);
    for case in cases {
        let entry = case.entry.expect("Morph body entry");
        game.clear_item_instruction_hook();
        native.seed(&case.ram);
        records.borrow_mut().clear();
        let c_result = native.invoke(game, entry, case.slot, case.operand);
        let c_ram = game.wram().to_vec();
        let c_effects = game.effects().to_vec();
        let c_records = records.borrow().clone();
        install(game)?;
        native.seed(&case.ram);
        records.borrow_mut().clear();
        let before = game.item_instruction_counters();
        let result = native.invoke(game, entry, case.slot, case.operand);
        assert_eq!(result, c_result, "{} actual-C return", case.id);
        assert_eq!(case.expected, fixtures::Expectation::Handled(result), "{} expected return", case.id);
        compare(case.id, &c_ram, game.wram());
        compare(case.id, &c_effects, game.effects());
        compare_records(case.id, &c_records, &records.borrow());
        let seen = records.borrow();
        assert!(seen.iter().all(|r| r.observation.scope_id == 2 && r.observation.scope_identity));
        let service_calls: Vec<_> = seen
            .iter()
            .filter(|r| r.observation.phase == Phase::ServiceBegin)
            .map(|r| (r.observation.value >> 16, r.observation.value as u16))
            .collect();
        let expected_calls: Vec<_> = case.service_calls.iter().map(|(service, arg)| (*service as u32, *arg)).collect();
        assert_eq!(service_calls, expected_calls, "{} ordered native services", case.id);
        assert_eq!(seen.len(), 2 + 2 * service_calls.len(), "{} complete service images", case.id);
        let after = game.item_instruction_counters();
        let at = entry as usize;
        assert_eq!(after.morph_standalone.rust_completed[at] - before.morph_standalone.rust_completed[at], 1);
        assert_eq!(after.morph_standalone.c_entries, before.morph_standalone.c_entries);
        assert_eq!(after.morph_standalone.rust_refused, before.morph_standalone.rust_refused);
        assert_eq!(after.scope_standalone, before.scope_standalone, "Morph must not pollute missile counts");
        for id in 0..8 {
            let expected = service_calls.iter().filter(|(service, _)| *service == id as u32).count() as u64;
            assert_eq!(after.service_requests[id] - before.service_requests[id], expected);
            assert_eq!(after.service_completions[id] - before.service_completions[id], expected);
        }
        println!(
            "body={} scope=Morph result={result:?} wram={} observer_records={} actual_c=true rust_completed=1",
            case.id,
            hash(game.wram()),
            seen.len()
        );
    }
    Ok(())
}

fn refusals(game: &Game, seed: &[u8]) -> Result<(), String> {
    let cases = fixtures::morph_refusal_fixtures(seed, game.rom())?;
    assert_eq!(cases.len(), REFUSAL_CASES);
    for mut case in cases {
        let before = case.ram.clone();
        let mut services = Preflight { available: case.services_available, calls: 0 };
        let result = case.run(&mut services);
        match case.expected {
            fixtures::Expectation::Refused => assert_eq!(result?, None, "{}", case.id),
            fixtures::Expectation::Error => assert!(result.is_err(), "{} must fail preflight", case.id),
            fixtures::Expectation::Handled(_) => panic!("Morph refusal marked handled"),
        }
        compare(case.id, &before, &case.ram);
        assert_eq!(services.calls, 0);
        println!("preflight={} scope=Morph unchanged=true services=0 unchecked_c=false", case.id);
    }
    Ok(())
}

fn scheduler_seed(mut ram: Vec<u8>, ip: u16) -> Vec<u8> {
    for slot in (0..80).step_by(2) {
        word(&mut ram, 0x1c37 + slot, 0);
    }
    ram[0x611..=0x615].fill(0);
    word(&mut ram, 0x1c23, 0x8000);
    word(&mut ram, 0x1c37 + SLOT as usize, 0xef23);
    word(&mut ram, 0x1dc7 + SLOT as usize, 0x1a);
    word(&mut ram, 0x1cd7 + SLOT as usize, 0x86d0);
    word(&mut ram, 0x1d27 + SLOT as usize, ip);
    word(&mut ram, 0xde1c + SLOT as usize, 1);
    word(&mut ram, 0xdc8, 0);
    ram
}

fn collected_service52(game: &mut Game, native: Native, seed: &[u8], records: &Records) -> Result<(), String> {
    let cases = fixtures::morph_body_fixtures(seed, game.rom())?;
    let input = cases.into_iter().find(|c| c.id == "M01").ok_or("missing M01")?;
    let mut ram = scheduler_seed(input.ram, 0xe3ef);
    ram[0xd873] |= 4;
    word(&mut ram, 0x998, 11);
    word(&mut ram, 0x99c, 0xe4a9);
    word(&mut ram, 0x943, 0);
    ram[0x612] = 2; // Actual native LoadMoreThings resumption at PlmHandler service52.
    word(&mut ram, 0x9a2, 0x102);
    word(&mut ram, 0x9a4, 0x208);
    word(&mut ram, 0x330, 0);
    word(&mut ram, 0x1c2d, 0);
    let mut pair = Vec::new();
    for rust in [false, true] {
        game.clear_room_transition_hook();
        game.clear_item_instruction_hook();
        if rust {
            install(game)?;
        }
        native.seed(&ram);
        records.borrow_mut().clear();
        let before = game.item_instruction_counters();
        assert_eq!(unsafe { (native.root11)() }, 0);
        assert_eq!(read(game.wram(), 0x1d27 + SLOT as usize), 0xdfad);
        assert_eq!(read(game.wram(), 0x1c37 + SLOT as usize), 0xef23);
        assert_eq!(read(game.wram(), 0x1c2d), 2, "gfx loaded before collected branch");
        assert_eq!(read(game.wram(), 0x1c2f), 0xe3f1);
        assert_eq!(read(game.wram(), 0xdf0c + SLOT as usize), 0);
        assert_eq!(&game.wram()[0xd0..0xd7], &[0, 1, 0, 0x87, 0x89, 0, 0x3e]);
        for i in 0..8 {
            assert_eq!(read(game.wram(), 0xa470 + 2 * i), 0x3e0 + i as u16);
        }
        assert_eq!(unsafe { (native.item_active)() }, 0);
        assert_eq!(unsafe { (native.plm)() }, 0);
        assert_eq!(read(game.wram(), 0x1c37 + SLOT as usize), 0);
        assert_eq!((read(game.wram(), 0x9a2), read(game.wram(), 0x9a4)), (0x102, 0x208));
        assert_eq!(read(game.wram(), 0xdc8), 0);
        let after = game.item_instruction_counters();
        assert_eq!(after.service_requests, before.service_requests, "no pickup/music or frame Common");
        if rust {
            for entry in [Entry::LoadItemPlmGfx, Entry::GotoIfItemBitSet, Entry::Goto] {
                assert_eq!(
                    after.morph_service52.rust_completed[entry as usize]
                        - before.morph_service52.rust_completed[entry as usize],
                    1
                );
            }
            assert_eq!(after.morph_standalone.rust_completed[0] - before.morph_standalone.rust_completed[0], 1);
            assert_eq!(after.morph_service52.c_entries, before.morph_service52.c_entries);
            assert_eq!(after.morph_standalone.c_entries, before.morph_standalone.c_entries);
        }
        pair.push((game.wram().to_vec(), game.effects().to_vec(), records.borrow().clone()));
    }
    compare("Morph collected service52", &pair[0].0, &pair[1].0);
    compare("Morph collected effects", &pair[0].1, &pair[1].1);
    compare_records("Morph collected service52", &pair[0].2, &pair[1].2);
    println!(
        "chain=Morph_collected resumptions=2 actual_native_parent=true service52=true gfx_loaded=true pickup=false"
    );
    Ok(())
}

fn transport(game: &mut Game, native: Native, seed: &[u8], records: &Records) -> Result<(), String> {
    let cases = fixtures::morph_body_fixtures(seed, game.rom())?;
    let input = cases.into_iter().find(|c| c.id == "M05").ok_or("missing M05")?;
    let ram = scheduler_seed(input.ram, 0xe407);
    let mut pair = Vec::new();
    for rust in [false, true] {
        game.clear_item_instruction_hook();
        if rust {
            install(game)?;
        }
        game.set_room_transition_hook(|_, ram, _, services| {
            services.observe(TransitionObservation::Enter { routine: Routine::LoadingNextRoomAsync, operand: 0 })?;
            ram[0x1234] = 0xa1;
            let result = services.coroutine(ram, Coroutine::PlmHandler)?;
            assert_eq!(result, 0);
            assert_eq!(read(ram, 0xde1c + SLOT as usize), 4);
            assert_eq!(read(ram, 0x1d27 + SLOT as usize), 0xe409);
            assert_eq!(read(ram, 0x1298c), 0xb08e, "Process result copied through child and parent");
            assert_eq!((read(ram, 0x1c29), read(ram, 0x1c2b)), (69, 41));
            assert_eq!(read(ram, 0x1c25), 8, "Draw tilemap writes copied back");
            assert_eq!(read(ram, 0x330), 14, "Draw VRAM queue copied back");
            assert_eq!(ram[0x1234], 0xa1);
            ram[0x1235] = 0xc2;
            services.observe(TransitionObservation::Return {
                routine: Routine::LoadingNextRoomAsync,
                result: result.into(),
            })?;
            Ok(TransitionOutcome::Handled(result))
        })?;
        native.seed(&ram);
        records.borrow_mut().clear();
        assert_eq!(unsafe { (native.root11)() }, 0);
        assert_eq!(game.wram()[0x1235], 0xc2);
        let seen = records.borrow();
        assert!(seen.iter().any(|r| r.observation.entry == Some(NativeEntry::DrawItemFrame0)
            && r.observation.scope_id == 2
            && r.observation.context == 1
            && r.ram[0x1234] == 0xa1));
        let services: Vec<_> =
            seen.iter().filter(|r| r.observation.phase == Phase::ServiceBegin).map(|r| r.observation.value).collect();
        assert_eq!(services, vec![3 << 16 | u32::from(SLOT), 4 << 16 | u32::from(SLOT), 5 << 16 | u32::from(SLOT)]);
        pair.push((game.wram().to_vec(), game.effects().to_vec(), seen.clone()));
    }
    game.clear_room_transition_hook();
    compare("Morph transport", &pair[0].0, &pair[1].0);
    compare("Morph transport effects", &pair[0].1, &pair[1].1);
    compare_records("Morph transport", &pair[0].2, &pair[1].2);
    println!("transport=Morph_parent_child_three_draw_services actual_service52=true copyback=true");
    Ok(())
}

fn fatal(game: &mut Game, native: Native, seed: &[u8], mode: &str) -> Result<(), String> {
    let cases = fixtures::morph_body_fixtures(seed, game.rom())?;
    let case = cases.into_iter().find(|c| c.id == "M05").ok_or("missing M05")?;
    native.seed(&case.ram);
    let mode_owned = mode.to_owned();
    game.set_item_instruction_hook(move |_, slot, _, ram, _, services| {
        // Give the transport the same valid draw record selected by Frame0.
        word(ram, 0xde6c + usize::from(slot), 0xa30f);
        match mode_owned.as_str() {
            "draw-service-refuse" => {
                services.process_plm_draw_instruction(ram, slot)?;
                Ok(Outcome::Refused)
            }
            "draw-service-error" => {
                services.process_plm_draw_instruction(ram, slot)?;
                services.calculate_plm_block_coords(ram, 1)?;
                Err("bad slot unexpectedly accepted".into())
            }
            "draw-wrong-ram" => {
                let mut wrong = ram.to_vec();
                let _ = services.process_plm_draw_instruction(&mut wrong, slot);
                Ok(Outcome::Refused)
            }
            "draw-bad-slot" => {
                services.process_plm_draw_instruction(ram, 80)?;
                Err("bad slot unexpectedly accepted".into())
            }
            _ => Err("unknown Morph fatal callback".into()),
        }
    })?;
    assert_eq!(native.invoke(game, Entry::DrawItemFrame0, case.slot, case.operand), Output::Stop);
    Err(format!("fatal Morph control {mode} unexpectedly returned"))
}

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 6 {
        return Err("usage: morph-instruction-host-control ARCHIVE ROM OUTPUT SEED_WRAM MODE".into());
    }
    let prepared = prepare(Path::new(&args[1]), Path::new(&args[3]))?;
    println!("library={} reused={}", prepared.library.display(), prepared.reused);
    if args[5] == "prepare" {
        return Ok(());
    }
    let seed = std::fs::read(&args[4]).map_err(|e| e.to_string())?;
    if hash(&seed) != SEED_SHA {
        return Err("unsealed shared item fixture seed".into());
    }
    let mut game = Game::boot(&prepared.library, Path::new(&args[2]))?;
    let library = unsafe { Library::new(&prepared.library).map_err(|e| e.to_string())? };
    let native = unsafe { Native::load(&library)? };
    let records = observe(&mut game)?;
    match args[5].as_str() {
        "all" => {
            body_pairs(&mut game, native, &seed, &records)?;
            refusals(&game, &seed)?;
            collected_service52(&mut game, native, &seed, &records)?;
            transport(&mut game, native, &seed, &records)?;
            println!(
                "Morph_body_pairs={BODY_CASES} preflight_controls={REFUSAL_CASES} compact_collected_chain=1 transaction_control=1 live_presentation=fixed_route_pair"
            );
            Ok(())
        }
        "bodies" => body_pairs(&mut game, native, &seed, &records),
        "refusals" => refusals(&game, &seed),
        "collected-service52" => collected_service52(&mut game, native, &seed, &records),
        "transport" => transport(&mut game, native, &seed, &records),
        mode @ ("draw-service-refuse" | "draw-service-error" | "draw-wrong-ram" | "draw-bad-slot") => {
            fatal(&mut game, native, &seed, mode)
        }
        _ => Err("unknown Morph control mode".into()),
    }
}
