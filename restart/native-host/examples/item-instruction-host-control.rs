//! Fixed actual-C body pairs and composition diagnostics, never a replay verdict.
//! K01/K02 real presentation are checked by the fixed full C/R live pair.
use libloading::Library;
use sha2::{Digest, Sha256};
use sm_item_instructions::{Entry, Output, Service, Services, execute, fixtures};
use sm_native_host::{
    Game, HookOutcome, ItemInstructionEntry as NativeEntry, ItemInstructionObservation,
    ItemInstructionOutcome as Outcome, ItemInstructionOutput as NativeOutput, ItemInstructionService as NativeService,
    ItemInstructionServices as NativeServices, RoomTransitionCoroutineService as Coroutine,
    RoomTransitionObservation as TransitionObservation, RoomTransitionOutcome as TransitionOutcome,
    RoomTransitionRoutine as Routine, prepare,
};
use std::{cell::RefCell, path::Path, rc::Rc};

pub(crate) const SEED_SHA: &str = "7adc5568e4239e479fcef325fef05769da8f6399e5909174d5f6f00c2f436459";
const SLOT: u16 = 78;

struct Bridge<'a>(&'a mut NativeServices);
impl Services for Bridge<'_> {
    fn available(&self, service: Service) -> bool {
        self.0.available(match service {
            Service::AddMissilesToHud => NativeService::AddMissilesToHud,
            Service::PlayRoomMusicAfter => NativeService::PlayRoomMusicAfter,
            Service::QueueMusicDelayed8 => NativeService::QueueMusicDelayed8,
            Service::ProcessPlmDrawInstruction => NativeService::ProcessPlmDrawInstruction,
            Service::CalculatePlmBlockCoords => NativeService::CalculatePlmBlockCoords,
            Service::DrawPlm => NativeService::DrawPlm,
            Service::AddSuperMissilesToHud => NativeService::AddSuperMissilesToHud,
            Service::AddPowerBombsToHud => NativeService::AddPowerBombsToHud,
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
pub(crate) fn install(game: &mut Game) -> Result<(), String> {
    game.set_item_instruction_hook(|entry, slot, operand, ram, rom, services| {
        Ok(match execute(Entry::try_from(entry as u32)?, slot, operand, ram, rom, &mut Bridge(services))? {
            Some(Output::PreinstructionDone) => Outcome::Handled(NativeOutput::PreinstructionDone),
            Some(Output::Stop) => Outcome::Handled(NativeOutput::Stop),
            Some(Output::Continue(offset)) => Outcome::Handled(NativeOutput::Continue(offset)),
            None => Outcome::Refused,
        })
    })
}
pub(crate) fn word(ram: &mut [u8], at: usize, value: u16) {
    ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
pub(crate) fn read(ram: &[u8], at: usize) -> u16 {
    u16::from_le_bytes(ram[at..at + 2].try_into().unwrap())
}
pub(crate) fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

type Instruction = unsafe extern "C" fn(u32, *const u8, u16) -> *const u8;
type Preinstruction = unsafe extern "C" fn(u32, u16);
#[derive(Clone, Copy)]
pub(crate) struct Native {
    commit: unsafe extern "C" fn(*const u8),
    pub(crate) begin: unsafe extern "C" fn(),
    instruction: Instruction,
    preinstruction: Preinstruction,
    pub(crate) plm: unsafe extern "C" fn() -> u8,
    pub(crate) root11: unsafe extern "C" fn() -> u8,
    projectile: unsafe extern "C" fn(u32) -> i32,
    projectile_done: unsafe extern "C" fn(u32),
    pub(crate) item_active: unsafe extern "C" fn() -> u32,
}
impl Native {
    pub(crate) unsafe fn load(library: &Library) -> Result<Self, String> {
        // The authenticated prepared library owns these private diagnostic symbols.
        unsafe {
            Ok(Self {
                commit: *library.get(b"sm_native_commit_wram\0").map_err(|e| e.to_string())?,
                begin: *library.get(b"sm_native_begin_step\0").map_err(|e| e.to_string())?,
                instruction: *library.get(b"CallPlmInstr\0").map_err(|e| e.to_string())?,
                preinstruction: *library.get(b"CallPlmPreInstr\0").map_err(|e| e.to_string())?,
                plm: *library.get(b"PlmHandler_Async\0").map_err(|e| e.to_string())?,
                root11: *library.get(b"sm_native_room_transition_11\0").map_err(|e| e.to_string())?,
                projectile: *library.get(b"sm_native_dispatch\0").map_err(|e| e.to_string())?,
                projectile_done: *library.get(b"sm_native_entry_done\0").map_err(|e| e.to_string())?,
                item_active: *library.get(b"sm_native_item_instruction_active\0").map_err(|e| e.to_string())?,
            })
        }
    }
    pub(crate) fn seed(self, ram: &[u8]) {
        assert_eq!(ram.len(), 0x20000);
        unsafe {
            (self.commit)(ram.as_ptr());
            (self.begin)();
        }
    }
    pub(crate) fn invoke(self, game: &Game, entry: Entry, slot: u16, operand: Option<u16>) -> Output {
        unsafe {
            if entry == Entry::GotoLinkIfTriggered {
                (self.preinstruction)(entry.address(), slot);
                return Output::PreinstructionDone;
            }
            let offset = usize::from(operand.expect("instruction operand"));
            let bank_start = game.rom().as_ptr().add(4 * 0x8000);
            let pointer = bank_start.add(offset - 0x8000);
            let result = (self.instruction)(entry.address(), pointer, slot) as usize;
            if result == 0 {
                Output::Stop
            } else if result < 0x10000 {
                assert!(result >= 0x8000);
                Output::Continue(result as u16)
            } else {
                let start = bank_start as usize;
                assert!((start..start + 0x8000).contains(&result));
                Output::Continue((result - start + 0x8000) as u16)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Record {
    pub(crate) observation: ItemInstructionObservation,
    pub(crate) ram: Vec<u8>,
}
pub(crate) type Records = Rc<RefCell<Vec<Record>>>;
pub(crate) fn observe(game: &mut Game) -> Result<Records, String> {
    let records = Rc::new(RefCell::new(Vec::new()));
    let output = records.clone();
    game.set_item_instruction_observer(move |observation, ram| {
        output.borrow_mut().push(Record { observation, ram: ram.to_vec() });
        Ok(())
    })?;
    Ok(records)
}
pub(crate) fn compare(id: &str, left: &[u8], right: &[u8]) {
    if left != right {
        let first = left.iter().zip(right).position(|(a, b)| a != b);
        panic!("{id} byte mismatch at {first:?}; C={} Rust={}", hash(left), hash(right));
    }
}
pub(crate) fn compare_records(id: &str, left: &[Record], right: &[Record]) {
    assert_eq!(left.len(), right.len(), "{id} observer count");
    for (at, (a, b)) in left.iter().zip(right).enumerate() {
        assert_eq!(a.observation, b.observation, "{id} observer {at}");
        compare(id, &a.ram, &b.ram);
    }
}
fn body_pairs(game: &mut Game, native: Native, seed: &[u8], records: &Records) -> Result<(), String> {
    let cases = fixtures::body_fixtures(seed, game.rom())?;
    assert_eq!(cases.len(), 25);
    for case in cases {
        let entry = case.entry.expect("body entry");
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
        assert_eq!(c_result, result, "{} return", case.id);
        assert_eq!(case.expected, fixtures::Expectation::Handled(result), "{} expected return", case.id);
        compare(case.id, &c_ram, game.wram());
        compare(case.id, &c_effects, game.effects());
        compare_records(case.id, &c_records, &records.borrow());
        let after = game.item_instruction_counters();
        let at = entry as usize;
        assert_eq!(after.scope_standalone.rust_completed[at] - before.scope_standalone.rust_completed[at], 1);
        assert_eq!(after.scope_standalone.c_entries, before.scope_standalone.c_entries);
        assert_eq!(after.scope_standalone.rust_refused, before.scope_standalone.rust_refused);
        println!(
            "body={} result={result:?} wram={} observer_records={} actual_c=true rust_completed=1",
            case.id,
            hash(game.wram()),
            c_records.len()
        );
    }
    Ok(())
}

pub(crate) struct Preflight {
    pub(crate) available: bool,
    pub(crate) calls: usize,
}
impl Services for Preflight {
    fn available(&self, _: Service) -> bool {
        self.available
    }
    fn add_missiles_to_hud(&mut self, _: &mut [u8]) -> Result<(), String> {
        self.calls += 1;
        Err("unexpected preflight service".into())
    }
    fn play_room_music_after(&mut self, _: &mut [u8], _: u16) -> Result<(), String> {
        self.calls += 1;
        Err("unexpected preflight service".into())
    }
    fn queue_music_delayed8(&mut self, _: &mut [u8], _: u16) -> Result<(), String> {
        self.calls += 1;
        Err("unexpected preflight service".into())
    }
    fn process_plm_draw_instruction(&mut self, _: &mut [u8], _: u16) -> Result<(), String> {
        self.calls += 1;
        Err("unexpected preflight service".into())
    }
    fn calculate_plm_block_coords(&mut self, _: &mut [u8], _: u16) -> Result<(), String> {
        self.calls += 1;
        Err("unexpected preflight service".into())
    }
    fn draw_plm(&mut self, _: &mut [u8], _: u16) -> Result<(), String> {
        self.calls += 1;
        Err("unexpected preflight service".into())
    }
    fn add_super_missiles_to_hud(&mut self, _: &mut [u8]) -> Result<(), String> {
        self.calls += 1;
        Err("unexpected preflight service".into())
    }
    fn add_power_bombs_to_hud(&mut self, _: &mut [u8]) -> Result<(), String> {
        self.calls += 1;
        Err("unexpected preflight service".into())
    }
}
fn refusals(game: &Game, seed: &[u8]) -> Result<(), String> {
    let cases = fixtures::refusal_fixtures(seed, game.rom())?;
    assert_eq!(cases.len(), 14);
    for mut case in cases {
        let before = case.ram.clone();
        let mut services = Preflight { available: case.services_available, calls: 0 };
        let result = Entry::try_from(case.raw_entry)
            .and_then(|entry| execute(entry, case.slot, case.operand, &mut case.ram, &case.rom, &mut services));
        match case.expected {
            fixtures::Expectation::Refused => assert_eq!(result?, None, "{}", case.id),
            fixtures::Expectation::Error => assert!(result.is_err(), "{} must fail", case.id),
            fixtures::Expectation::Handled(_) => panic!("refusal fixture marked handled"),
        }
        compare(case.id, &before, &case.ram);
        assert_eq!(services.calls, 0);
        println!("preflight={} unchanged=true services=0 unchecked_c=false", case.id);
    }
    Ok(())
}

fn scheduler_seed(seed: &[u8], ip: u16) -> Vec<u8> {
    let mut ram = seed.to_vec();
    for slot in (0..80).step_by(2) {
        word(&mut ram, 0x1c37 + slot, 0);
    }
    ram[0x611..=0x615].fill(0);
    word(&mut ram, 0x1c23, 0x8000);
    word(&mut ram, 0x1c37 + SLOT as usize, 0xef2f);
    word(&mut ram, 0x1dc7 + SLOT as usize, 0x22);
    word(&mut ram, 0x1cd7 + SLOT as usize, 0x86d0);
    word(&mut ram, 0x1d27 + SLOT as usize, ip);
    word(&mut ram, 0xde1c + SLOT as usize, 1);
    word(&mut ram, 0xdc8, 0);
    ram
}

fn k03(game: &mut Game, native: Native, seed: &[u8], records: &Records) -> Result<(), String> {
    let mut ram = scheduler_seed(seed, 0xe47c);
    ram[0xd874] |= 4;
    word(&mut ram, 0x998, 11);
    word(&mut ram, 0x99c, 0xe4a9);
    word(&mut ram, 0x943, 0); // No unrelated DrawTimer after transition root.
    ram[0x612] = 2; // Declared native LoadMoreThings coroutine resumption at service52.
    word(&mut ram, 0x9c6, 7);
    word(&mut ram, 0x9c8, 15);
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
        assert_eq!(read(game.wram(), 0x1d27 + SLOT as usize), 0xe4ac);
        assert_eq!(read(game.wram(), 0x1c37 + SLOT as usize), 0xef2f);
        assert_eq!(unsafe { (native.item_active)() }, 0);
        assert_eq!(unsafe { (native.plm)() }, 0); // Real one-tick empty draw then Delete.
        assert_eq!(read(game.wram(), 0x1c37 + SLOT as usize), 0);
        assert_eq!((read(game.wram(), 0x9c6), read(game.wram(), 0x9c8)), (7, 15));
        assert_eq!(read(game.wram(), 0xdc8), 0);
        let after = game.item_instruction_counters();
        assert_eq!(after.service_requests, before.service_requests);
        if rust {
            assert_eq!(
                after.scope_service52.rust_completed[Entry::GotoIfItemBitSet as usize]
                    - before.scope_service52.rust_completed[Entry::GotoIfItemBitSet as usize],
                1
            );
            assert_eq!(
                after.scope_standalone.rust_completed[Entry::Delete as usize]
                    - before.scope_standalone.rust_completed[Entry::Delete as usize],
                1
            );
            assert_eq!(after.scope_service52.c_entries, before.scope_service52.c_entries);
        }
        pair.push((game.wram().to_vec(), game.effects().to_vec(), records.borrow().clone()));
    }
    compare("K03", &pair[0].0, &pair[1].0);
    compare("K03 effects", &pair[0].1, &pair[1].1);
    compare_records("K03", &pair[0].2, &pair[1].2);
    println!("chain=K03 resumptions=2 actual_native_parent=true service52=true already_collected=true");
    Ok(())
}

fn transport(game: &mut Game, native: Native, seed: &[u8], records: &Records) -> Result<(), String> {
    let ram = scheduler_seed(seed, 0xe488);
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
            assert_eq!(read(ram, 0xdebc + SLOT as usize), 0xe49f, "real SetLinkReg result copied to parent");
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
        assert!(seen.iter().any(|record| record.observation.entry == Some(NativeEntry::SetLinkReg)
            && record.observation.context == 1
            && record.ram[0x1234] == 0xa1));
        pair.push((game.wram().to_vec(), game.effects().to_vec(), seen.clone()));
    }
    game.clear_room_transition_hook();
    compare("transport", &pair[0].0, &pair[1].0);
    compare("transport effects", &pair[0].1, &pair[1].1);
    compare_records("transport", &pair[0].2, &pair[1].2);
    println!("transport=parent_to_child_to_parent actual_service52=true sentinel=true");
    Ok(())
}

fn fallback(game: &mut Game, native: Native, seed: &[u8]) -> Result<(), String> {
    let mut ram = seed.to_vec();
    word(&mut ram, 0x1c37 + SLOT as usize, 0xef2b);
    game.clear_item_instruction_hook();
    native.seed(&ram);
    let c = native.invoke(game, Entry::SetTimer, SLOT, Some(0xe492));
    let c_ram = game.wram().to_vec();
    let c_effects = game.effects().to_vec();
    install(game)?;
    native.seed(&ram);
    let before = game.item_instruction_counters();
    let r = native.invoke(game, Entry::SetTimer, SLOT, Some(0xe492));
    assert_eq!(c, r);
    compare("foreign fallback", &c_ram, game.wram());
    compare("foreign fallback effects", &c_effects, game.effects());
    let after = game.item_instruction_counters();
    assert_eq!(after.other_standalone.rust_refused[4] - before.other_standalone.rust_refused[4], 1);
    assert_eq!(after.other_standalone.c_entries[4] - before.other_standalone.c_entries[4], 1);
    assert_eq!(after.scope_standalone.c_entries, before.scope_standalone.c_entries);
    println!("fallback=foreign_header unchanged_refusal=true actual_c=1");
    Ok(())
}

fn spawn_trigger(game: &mut Game, native: Native, seed: &[u8], records: &Records) -> Result<(), String> {
    // Pin578f90b/pristine LoROM84:EED3: actual EEAB setup, AAE3 script.
    if game.rom().get(0x26ed3..0x26ed7) != Some(&[0xab, 0xee, 0xe3, 0xaa][..]) {
        return Err("unsealed native EED3 projectile-trigger header".into());
    }
    let mut ram = scheduler_seed(seed, 0xdfb7);
    word(&mut ram, 0x1cd7 + SLOT as usize, 0xdf89);
    word(&mut ram, 0xdebc + SLOT as usize, 0xe484);
    word(&mut ram, 0x1d77 + SLOT as usize, 0);
    word(&mut ram, 0x0a78, 0); // time_is_frozen_flag.
    let block = read(&ram, 0x1c87 + SLOT as usize);
    assert!(block != 0 && block & 1 == 0);
    word(&mut ram, 0xdc4, block / 2);
    let mut pair = Vec::new();
    for rust in [false, true] {
        game.clear_item_instruction_hook();
        if rust {
            install(game)?;
        }
        let observations = records.clone();
        game.set_projectile_hook(move |_, ram, _, services| {
            let before = observations.borrow().len();
            let result = services.spawn_plm(ram, 0xeed3)?;
            assert_eq!(result, 0);
            assert_eq!(read(ram, 0x1d77 + SLOT as usize), 255, "native EEAB result copied back");
            assert_eq!(unsafe { (native.item_active)() }, 0);
            assert_eq!(observations.borrow().len(), before, "SpawnPLM setup cannot enter item instruction callback");
            Ok(HookOutcome::Handled)
        })?;
        native.seed(&ram);
        records.borrow_mut().clear();
        let before = game.item_instruction_counters();
        assert_eq!(unsafe { (native.projectile)(0) }, 1);
        unsafe {
            (native.projectile_done)(0);
        }
        assert_eq!(game.item_instruction_counters().total.entries, before.total.entries);
        assert_eq!(read(game.wram(), 0x1d77 + SLOT as usize), 255);
        assert_eq!(unsafe { (native.plm)() }, 0);
        assert_eq!(read(game.wram(), 0x1d27 + SLOT as usize), 0xdfcd, "real scheduler advanced into burst");
        if rust {
            let after = game.item_instruction_counters();
            assert_eq!(after.scope_standalone.rust_completed[12] - before.scope_standalone.rust_completed[12], 1);
            assert_eq!(after.scope_standalone.c_entries, before.scope_standalone.c_entries);
        }
        pair.push((game.wram().to_vec(), game.effects().to_vec(), records.borrow().clone()));
    }
    game.clear_projectile_hook();
    compare("SpawnPLM", &pair[0].0, &pair[1].0);
    compare("SpawnPLM effects", &pair[0].1, &pair[1].1);
    compare_records("SpawnPLM", &pair[0].2, &pair[1].2);
    println!(
        "composition=projectile_spawn_plm header=EED3 native_timer=00FF item_entries_inside_projectile=0 later_DF89=true"
    );
    Ok(())
}

fn fatal(game: &mut Game, native: Native, seed: &[u8], mode: &str) -> Result<(), String> {
    let cases = fixtures::body_fixtures(seed, game.rom())?;
    let case = cases
        .iter()
        .find(|c| c.id == if mode == "service-refuse" || mode == "wrong-ram" { "B12" } else { "B07" })
        .unwrap();
    native.seed(&case.ram);
    let raw_pointer = game.rom().as_ptr() as usize + 4 * 0x8000 + 0xe492 - 0x8000;
    let mode_owned = mode.to_owned();
    game.set_item_instruction_hook(move |_, _, _, ram, _, services| match mode_owned.as_str() {
        "dirty-refuse" => {
            ram[0] ^= 1;
            Ok(Outcome::Refused)
        }
        "service-refuse" => {
            services.add_missiles_to_hud(ram)?;
            Ok(Outcome::Refused)
        }
        "wrong-ram" => {
            let mut wrong = ram.to_vec();
            let _ = services.add_missiles_to_hud(&mut wrong);
            Ok(Outcome::Refused)
        }
        "wrong-tag" => Ok(Outcome::Handled(NativeOutput::PreinstructionDone)),
        "wrong-offset" => Ok(Outcome::Handled(NativeOutput::Continue(0x7fff))),
        "panic" => panic!("deliberate item hook panic"),
        "recursive" => {
            unsafe {
                (native.instruction)(Entry::SetTimer.address(), raw_pointer as *const u8, SLOT);
            }
            unreachable!()
        }
        "reverse-projectile" => {
            unsafe {
                (native.projectile)(0);
            }
            unreachable!()
        }
        _ => Err("unexpected fatal callback mode".into()),
    })?;
    if mode == "during-projectile" {
        game.set_projectile_hook(move |_, _, _, _| {
            unsafe {
                (native.instruction)(Entry::SetTimer.address(), raw_pointer as *const u8, SLOT);
            }
            unreachable!()
        })?;
        unsafe {
            (native.projectile)(0);
        }
    } else if mode == "outside-service52" {
        game.set_room_transition_hook(move |_, _, _, services| {
            services.observe(TransitionObservation::Enter { routine: Routine::LoadingNextRoomAsync, operand: 0 })?;
            unsafe {
                (native.instruction)(Entry::SetTimer.address(), raw_pointer as *const u8, SLOT);
            }
            unreachable!()
        })?;
        unsafe {
            (native.root11)();
        }
    } else if mode == "observer-failure" {
        game.set_item_instruction_observer(|_, _| Err("deliberate evidence failure".into()))?;
        native.invoke(game, case.entry.unwrap(), case.slot, case.operand);
    } else {
        native.invoke(game, case.entry.unwrap(), case.slot, case.operand);
    }
    Err(format!("fatal control {mode} unexpectedly returned"))
}

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 6 {
        return Err("usage: item-instruction-host-control ARCHIVE ROM OUTPUT SEED_WRAM MODE".into());
    }
    let prepared = prepare(Path::new(&args[1]), Path::new(&args[3]))?;
    println!("library={} reused={}", prepared.library.display(), prepared.reused);
    if args[5] == "prepare" {
        return Ok(());
    }
    let seed = std::fs::read(&args[4]).map_err(|e| e.to_string())?;
    if hash(&seed) != SEED_SHA {
        return Err("unsealed item fixture seed".into());
    }
    let mut game = Game::boot(&prepared.library, Path::new(&args[2]))?;
    let library = unsafe { Library::new(&prepared.library).map_err(|e| e.to_string())? };
    let native = unsafe { Native::load(&library)? };
    let records = observe(&mut game)?;
    match args[5].as_str() {
        "all" => {
            body_pairs(&mut game, native, &seed, &records)?;
            refusals(&game, &seed)?;
            k03(&mut game, native, &seed, &records)?;
            transport(&mut game, native, &seed, &records)?;
            spawn_trigger(&mut game, native, &seed, &records)?;
            fallback(&mut game, native, &seed)?;
            println!(
                "body_pairs=25 preflight_controls=14 compact_chains=1 K01_K02=fixed_live_pair pending_mocked=false"
            );
            Ok(())
        }
        "bodies" => body_pairs(&mut game, native, &seed, &records),
        "refusals" => refusals(&game, &seed),
        "k03" => k03(&mut game, native, &seed, &records),
        "transport" => transport(&mut game, native, &seed, &records),
        "fallback" => fallback(&mut game, native, &seed),
        "spawn-trigger" => spawn_trigger(&mut game, native, &seed, &records),
        mode @ ("dirty-refuse" | "service-refuse" | "wrong-ram" | "wrong-tag" | "wrong-offset" | "panic"
        | "recursive" | "reverse-projectile" | "during-projectile" | "outside-service52"
        | "observer-failure") => fatal(&mut game, native, &seed, mode),
        _ => Err("unknown item control mode".into()),
    }
}
