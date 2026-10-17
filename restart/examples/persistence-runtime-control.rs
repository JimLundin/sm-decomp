//! Six ordinary Save/Load invocations exercising the public runtime installer.
//! Root alone runs this control against an authenticated merged native library.
use libloading::Library;
use sha2::{Digest, Sha256};
use sm_binary_evidence::ToEvidence;
use sm_native_host::{
    Game, PersistenceCounters, PersistenceEntry, PersistenceObservation, PersistenceOutput, PersistencePhase,
};
use sm_restart::{gameplay, persistence_runtime};
use std::{
    cell::RefCell,
    fs,
    path::{Path, PathBuf},
    rc::Rc,
};

type Result<T> = std::result::Result<T, String>;
const WRAM: usize = 0x20000;
const SRAM: usize = 0x2000;
#[derive(Clone, Copy)]
struct Native {
    commit: unsafe extern "C" fn(*const u8, *const u8),
    begin: unsafe extern "C" fn(),
    save: unsafe extern "C" fn(u16),
    load: unsafe extern "C" fn(u16) -> u8,
}
impl Native {
    unsafe fn load(library: &Library) -> Result<Self> {
        // SAFETY: Game authenticated the library; these are the retained exact
        // ordinary Save/Load signatures and existing diagnostic state exports.
        unsafe {
            Ok(Self {
                commit: *library.get(b"sm_native_commit_persistence\0").map_err(|e| e.to_string())?,
                begin: *library.get(b"sm_native_begin_step\0").map_err(|e| e.to_string())?,
                save: *library.get(b"SaveToSram\0").map_err(|e| e.to_string())?,
                load: *library.get(b"LoadFromSram\0").map_err(|e| e.to_string())?,
            })
        }
    }
    fn seed(self, ram: &[u8], sram: &[u8]) {
        assert_eq!(ram.len(), WRAM);
        assert_eq!(sram.len(), SRAM);
        // SAFETY: fixed complete buffers, synchronous copy, all owners idle.
        unsafe {
            (self.commit)(ram.as_ptr(), sram.as_ptr());
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Record {
    event: PersistenceObservation,
    ram: Vec<u8>,
    sram: Vec<u8>,
}
type Records = Rc<RefCell<Vec<Record>>>;
#[derive(Clone, Debug, PartialEq, Eq)]
struct Snapshot {
    output: PersistenceOutput,
    ram: Vec<u8>,
    sram: Vec<u8>,
    effects: Vec<u8>,
    records: Vec<Record>,
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn read(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    let path = path.as_ref();
    fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))
}
fn write(path: impl AsRef<Path>, bytes: impl AsRef<[u8]>) -> Result<()> {
    let path = path.as_ref();
    fs::write(path, bytes).map_err(|e| format!("write {}: {e}", path.display()))
}
fn observations(records: &[Record]) -> Vec<u8> {
    // Same encoding as accepted24: count, four u32 fields, full WRAM + SRAM.
    let mut bytes = (records.len() as u32).to_le_bytes().to_vec();
    for record in records {
        let output = match record.event.output {
            None => u32::MAX,
            Some(PersistenceOutput::Done) => 0,
            Some(PersistenceOutput::Loaded(value)) => 0x10000 | u32::from(value),
        };
        for value in [record.event.phase as u32, record.event.entry as u32, u32::from(record.event.argument), output] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.extend_from_slice(&record.ram);
        bytes.extend_from_slice(&record.sram);
    }
    bytes
}
fn count_delta(before: &PersistenceCounters, after: &PersistenceCounters) -> PersistenceCounters {
    let subtract = |a: [u64; 4], b: [u64; 4]| std::array::from_fn(|i| a[i] - b[i]);
    PersistenceCounters {
        entries: subtract(after.entries, before.entries),
        c_entries: subtract(after.c_entries, before.c_entries),
        rust_entries: subtract(after.rust_entries, before.rust_entries),
        rust_refused: subtract(after.rust_refused, before.rust_refused),
        rust_completed: subtract(after.rust_completed, before.rust_completed),
        write_requests: after.write_requests - before.write_requests,
        write_completions: after.write_completions - before.write_completions,
    }
}
#[derive(Clone, Copy)]
enum Owner {
    Disabled,
    Rust,
    Refused,
}
fn invoke(
    game: &mut Game,
    native: Native,
    records: &Records,
    directory: &Path,
    entry: PersistenceEntry,
    owner: Owner,
) -> Result<Snapshot> {
    fs::create_dir(directory).map_err(|e| e.to_string())?;
    records.borrow_mut().clear();
    let input_ram = game.wram().to_vec();
    let input_sram = game.sram().to_vec();
    write(directory.join("input.wram"), &input_ram)?;
    write(directory.join("input.sram"), &input_sram)?;
    let before = game.persistence_counters();
    // SAFETY: no Game borrow/callback is active; ordinary wrapped roots execute
    // synchronously at admitted slot0. No diagnostic dispatcher substitutes them.
    let output = unsafe {
        (native.begin)();
        match entry {
            PersistenceEntry::Save => {
                (native.save)(0);
                PersistenceOutput::Done
            }
            PersistenceEntry::Load => PersistenceOutput::Loaded((native.load)(0)),
            _ => unreachable!("composition only calls ordinary Save/Load"),
        }
    };
    let snapshot = Snapshot {
        output,
        ram: game.wram().to_vec(),
        sram: game.sram().to_vec(),
        effects: game.effects().to_vec(),
        records: records.borrow().clone(),
    };
    let delta = count_delta(&before, &game.persistence_counters());
    write(directory.join("final.wram"), &snapshot.ram)?;
    write(directory.join("final.sram"), &snapshot.sram)?;
    write(directory.join("effects.bin"), &snapshot.effects)?;
    write(directory.join("observations.bin"), observations(&snapshot.records))?;
    write(directory.join("counters.sme"), sm_binary_evidence::encode(&delta.to_evidence()?)?)?;
    write(directory.join("output.txt"), format!("{output:?}\n"))?;
    let i = entry as usize;
    let mut expected = PersistenceCounters::default();
    expected.entries[i] = 1;
    match owner {
        Owner::Disabled => expected.c_entries[i] = 1,
        Owner::Rust => {
            expected.rust_entries[i] = 1;
            expected.rust_completed[i] = 1;
        }
        Owner::Refused => {
            expected.rust_entries[i] = 1;
            expected.rust_refused[i] = 1;
            expected.c_entries[i] = 1;
        }
    }
    let save = entry == PersistenceEntry::Save;
    expected.write_requests = u64::from(save);
    expected.write_completions = u64::from(save);
    assert_eq!(delta, expected, "typed ownership counters");
    let phases: Vec<_> = snapshot.records.iter().map(|r| r.event.phase).collect();
    assert_eq!(
        phases,
        if save {
            vec![PersistencePhase::Begin, PersistencePhase::DiskBegin, PersistencePhase::DiskEnd, PersistencePhase::End]
        } else {
            vec![PersistencePhase::Begin, PersistencePhase::End]
        }
    );
    assert!(snapshot.records.iter().all(|r| r.event.entry == entry && r.event.argument == 0));
    assert_eq!(snapshot.records[0].ram, input_ram);
    assert_eq!(snapshot.records[0].sram, input_sram);
    let last = snapshot.records.last().unwrap();
    assert_eq!(last.event.output, Some(output));
    assert_eq!(last.ram, snapshot.ram);
    assert_eq!(last.sram, snapshot.sram);
    if save {
        assert_eq!(snapshot.records[1].ram, snapshot.ram);
        assert_eq!(snapshot.records[1].sram, snapshot.sram);
        assert_eq!(snapshot.records[1].ram, snapshot.records[2].ram);
        assert_eq!(snapshot.records[1].sram, snapshot.records[2].sram);
        assert_eq!(read("saves/sm.srm")?, snapshot.sram);
        assert_eq!(read("saves/sm.srm.bak")?, vec![0xa5; SRAM]);
    } else {
        assert_eq!(output, PersistenceOutput::Loaded(0));
        assert_eq!(snapshot.sram, input_sram);
        assert_eq!(read("saves/sm.srm")?, snapshot.sram);
        assert_eq!(read("saves/sm.srm.bak")?, vec![0xa5; SRAM]);
    }
    Ok(snapshot)
}
fn fresh_cwd(path: &Path) -> Result<()> {
    fs::create_dir(path).map_err(|e| format!("fresh directory {}: {e}", path.display()))?;
    std::env::set_current_dir(path).map_err(|e| e.to_string())?;
    fs::create_dir("saves").map_err(|e| e.to_string())?;
    write("saves/sm.srm", vec![0xa5; SRAM])?;
    write("saves/sm.srm.bak", vec![0x5a; SRAM])
}
fn install_siblings(game: &mut Game) -> Result<()> {
    gameplay::install_player_motion(game)?;
    gameplay::install_projectiles(game, false)?;
    gameplay::install_scrolling(game, false)?;
    gameplay::install_ceres_doors(game, false)?;
    gameplay::install_room_transitions(game, false)?;
    gameplay::install_door_scrolling(game, false)?;
    gameplay::install_camera(game, false)?;
    gameplay::install_item_instructions(game)
}
fn sibling_counters(game: &Game) -> String {
    format!(
        "{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n",
        game.player_motion_counters(),
        game.projectile_counters(),
        game.scroll_counters(),
        game.ceres_door_counters(),
        game.room_transition_counters(),
        game.door_scrolling_counters(),
        game.camera_counters(),
        game.item_instruction_counters()
    )
}
fn absolute_existing(path: &str) -> Result<PathBuf> {
    fs::canonicalize(path).map_err(|e| format!("{path}: {e}"))
}
fn run() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 4 {
        return Err("usage: persistence-runtime-control LIBRARY ROM REFERENCE24_SAVE0_C_DIR FRESH_OUTPUT".into());
    }
    let library_path = absolute_existing(&args[0])?;
    let rom = absolute_existing(&args[1])?;
    let reference = absolute_existing(&args[2])?;
    let old_cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let out = old_cwd.join(&args[3]);
    fs::create_dir(&out).map_err(|e| format!("fresh output {}: {e}", out.display()))?;
    let out = fs::canonicalize(out).map_err(|e| e.to_string())?;
    write(out.join("argv.txt"), format!("{:?}\n", std::env::args().collect::<Vec<_>>()))?;
    write(
        out.join("inputs.txt"),
        format!(
            "library={}\nlibrary_sha256={}\nrom={}\nrom_sha256={}\nreference={}\n",
            library_path.display(),
            sm_native_host::authenticate_library(&library_path)?,
            rom.display(),
            hash(&read(&rom)?),
            reference.display()
        ),
    )?;
    let ram = read(reference.join("input.wram"))?;
    let sram = read(reference.join("input.sram"))?;
    assert_eq!(ram.len(), WRAM);
    assert_eq!(sram.len(), SRAM);
    assert_eq!(&ram[0x79f..0x7a1], &[0, 0], "reference must be save-slot-0-area-0");
    std::env::set_current_dir(&out).map_err(|e| e.to_string())?;
    let mut game = Game::boot(&library_path, &rom)?;
    // SAFETY: load a second handle to the same already-authenticated process-
    // lifetime library; retain it through every diagnostic invocation.
    let library = unsafe { Library::new(&library_path) }.map_err(|e| e.to_string())?;
    let native = unsafe { Native::load(&library)? };
    install_siblings(&mut game)?;
    let siblings = sibling_counters(&game);
    let records = Records::default();
    let captured = records.clone();
    game.set_persistence_observer(move |event, ram, sram| {
        captured.borrow_mut().push(Record { event, ram: ram.to_vec(), sram: sram.to_vec() });
        Ok(())
    })?;
    let before = game.persistence_counters();
    let mut admitted = Vec::new();
    for (name, enabled) in [("disabled", false), ("installed", true)] {
        fresh_cwd(&out.join(name))?;
        if enabled {
            persistence_runtime::install(&mut game)?;
        } else {
            game.clear_persistence_hook();
        }
        native.seed(&ram, &sram);
        let save = invoke(
            &mut game,
            native,
            &records,
            Path::new("save"),
            PersistenceEntry::Save,
            if enabled { Owner::Rust } else { Owner::Disabled },
        )?;
        for (file, actual) in [("final.wram", &save.ram), ("final.sram", &save.sram), ("effects.bin", &save.effects)] {
            assert_eq!(*actual, read(reference.join(file))?, "accepted24 Save expectation {file}");
        }
        assert_eq!(observations(&save.records), read(reference.join("observations.bin"))?, "accepted24 service images");
        let mut changed = save.ram.clone();
        changed[0x9a2..0xa02].fill(0);
        changed[0x78b..0x78d].fill(0);
        changed[0x79f..0x7a1].fill(0);
        native.seed(&changed, &save.sram);
        let load = invoke(
            &mut game,
            native,
            &records,
            Path::new("load"),
            PersistenceEntry::Load,
            if enabled { Owner::Rust } else { Owner::Disabled },
        )?;
        assert_eq!(&load.ram[0x9a2..0xa02], &ram[0x9a2..0xa02]);
        assert_eq!(&load.ram[0x78b..0x78d], &ram[0x78b..0x78d]);
        admitted.push((save, load));
    }
    assert_eq!(admitted[0], admitted[1], "disabled versus production-installed full state and effects");
    let mut area6 = ram.clone();
    area6[0x79f..0x7a1].copy_from_slice(&6_u16.to_le_bytes());
    let mut refused = Vec::new();
    for (name, enabled) in [("area6-c", false), ("area6-installed", true)] {
        fresh_cwd(&out.join(name))?;
        if enabled {
            persistence_runtime::install(&mut game)?;
        } else {
            game.clear_persistence_hook();
        }
        native.seed(&area6, &sram);
        refused.push(invoke(
            &mut game,
            native,
            &records,
            Path::new("save"),
            PersistenceEntry::Save,
            if enabled { Owner::Refused } else { Owner::Disabled },
        )?);
    }
    assert_eq!(refused[0], refused[1], "safe area6 refusal retains original C behavior");
    assert_eq!(sibling_counters(&game), siblings, "codec must not enter another installed owner");
    write(out.join("sibling-counters.txt"), &siblings)?;
    let delta = count_delta(&before, &game.persistence_counters());
    write(out.join("counters.sme"), sm_binary_evidence::encode(&delta.to_evidence()?)?)?;
    write(
        out.join("result.toml"),
        "success = true\nordinary_invocations = 6\nadmitted_save_load_pairs = 1\nsafe_area6_pair = 1\nproduction_installer = true\nother_owners_registered = 8\nother_owners_exercised = false\nlive_gameplay = false\nfile_reopen_continue = false\n",
    )?;
    std::env::set_current_dir(old_cwd).map_err(|e| e.to_string())?;
    println!(
        "ordinary_invocations=6 admitted_save_load_pair=pass area6_refusal_pair=pass production_installer=true disk_writes=4 live=false reopen=false"
    );
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("persistence runtime control: {error}");
        std::process::exit(1);
    }
}
