//! Direct actual-C boundary fixtures, not a gameplay replay or acceptance run.
//! Each fatal control must run in its own supervised process/private directory.
use libloading::Library;
use sha2::{Digest, Sha256};
use sm_native_host::{CeresDoorEntry, CeresDoorOutcome as Outcome, CeresDoorOutput as Output, Game, prepare};
use std::path::Path;

fn word(ram: &mut [u8], at: usize, value: u16) {
    ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn read_word(ram: &[u8], at: usize) -> u16 {
    u16::from_le_bytes(ram[at..at + 2].try_into().unwrap())
}

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 5 {
        return Err("usage: ceres-door-host-control ARCHIVE ROM OUTPUT prepare|off|refuse|dirty-refuse|panic|bad-output|service|service-refuse|wrong-ram-refuse|bad-service-refuse|escape-off|nested-projectile|nested-scroll".into());
    }
    let mode = args[4].clone();
    if ![
        "prepare",
        "off",
        "refuse",
        "dirty-refuse",
        "panic",
        "bad-output",
        "service",
        "service-refuse",
        "wrong-ram-refuse",
        "bad-service-refuse",
        "escape-off",
        "nested-projectile",
        "nested-scroll",
    ]
    .contains(&mode.as_str())
    {
        return Err("unknown control mode".into());
    }
    let prepared = prepare(Path::new(&args[1]), Path::new(&args[3]))?;
    println!("library={} reused={}", prepared.library.display(), prepared.reused);
    if mode == "prepare" {
        return Ok(());
    }
    let mut game = Game::boot(&prepared.library, Path::new(&args[2]))?;
    // These diagnostic-only bindings deliberately inject fixture state. The
    // production Game API exposes no arbitrary mutable canonical native state.
    unsafe {
        let library = Library::new(&prepared.library).map_err(|e| e.to_string())?;
        let commit: unsafe extern "C" fn(*const u8) = *library.get(b"sm_native_commit_wram\0").unwrap();
        let begin: unsafe extern "C" fn() = *library.get(b"sm_native_begin_step\0").unwrap();
        let ai: unsafe extern "C" fn(u32) = *library.get(b"CallEnemyAi\0").unwrap();
        let instr: unsafe extern "C" fn(u32, u16, *const u16) -> *const u16 =
            *library.get(b"CallEnemyInstr\0").unwrap();
        let setup: unsafe extern "C" fn(u32, u16) -> u8 = *library.get(b"CallPlmHeaderFunc\0").unwrap();
        let projectile: unsafe extern "C" fn(u32) -> i32 = *library.get(b"sm_native_dispatch\0").unwrap();
        let scroll: unsafe extern "C" fn(u32) -> i32 = *library.get(b"sm_native_scroll_dispatch\0").unwrap();
        let mut ram = vec![0; 0x20000];
        word(&mut ram, 0xf78, 0xe23f);
        word(&mut ram, 0xf7a, 100);
        word(&mut ram, 0xf7e, 200);
        word(&mut ram, 0xfa8, 0xf76b);
        word(&mut ram, 0xaf6, 100);
        word(&mut ram, 0xafa, 200);
        word(&mut ram, 0x7a5, 16);
        word(&mut ram, 0x1c37, 0xba48);
        let service = matches!(
            mode.as_str(),
            "service" | "service-refuse" | "wrong-ram-refuse" | "bad-service-refuse" | "escape-off"
        );
        if service {
            // C Func6 decrements D=2/E=0/F=0 to 1/4/3 and selects offset(2,12).
            // Seed zero chooses puff parameter12. The service control checks
            // only the actual foreign operation and its external spawn point.
            word(&mut ram, 0xfa8, 0xf7dc);
            word(&mut ram, 0xfae, 2);
        }
        commit(ram.as_ptr());
        begin();
        if !matches!(mode.as_str(), "off" | "escape-off") {
            let mode = mode.clone();
            game.set_ceres_door_hook(move |entry, index, instruction, ram, _, services| {
                assert_eq!(index, 0);
                if matches!(entry, CeresDoorEntry::Init | CeresDoorEntry::Main | CeresDoorEntry::SolidSetup) {
                    assert_eq!(instruction, 0);
                } else {
                    assert_eq!(instruction, 0xf5a2);
                }
                match mode.as_str() {
                    "refuse" => Ok(Outcome::Refused),
                    "dirty-refuse" => {
                        ram[0] ^= 1;
                        Ok(Outcome::Refused)
                    }
                    "panic" => panic!("deliberate Ceres callback panic"),
                    "bad-output" => Ok(Outcome::Handled(Output::InstructionOffset(0xf5a2))),
                    "nested-projectile" => {
                        projectile(0);
                        unreachable!()
                    }
                    "nested-scroll" => {
                        scroll(0);
                        unreachable!()
                    }
                    "wrong-ram-refuse" => {
                        let mut unrelated = ram.to_vec();
                        assert!(services.spawn_eproj_with_room_gfx(&mut unrelated, 0xe509, 12, [102, 212]).is_err());
                        Ok(Outcome::Refused)
                    }
                    "bad-service-refuse" => {
                        assert!(services.spawn_eproj_with_room_gfx(ram, 0xe508, 12, [102, 212]).is_err());
                        Ok(Outcome::Refused)
                    }
                    "service" | "service-refuse" => {
                        assert_eq!(entry, CeresDoorEntry::Main);
                        services.spawn_eproj_with_room_gfx(ram, 0xe509, 12, [102, 212])?;
                        let slot = (0..18)
                            .find(|&slot| read_word(ram, 0x1997 + slot * 2) == 0xe509)
                            .expect("actual C E509 allocation copied back");
                        assert_eq!(read_word(ram, 0x1a4b + slot * 2), 102);
                        assert_eq!(read_word(ram, 0x1a93 + slot * 2), 212);
                        Ok(if mode == "service" { Outcome::Handled(Output::Done) } else { Outcome::Refused })
                    }
                    _ => unreachable!(),
                }
            })?;
        }
        if service {
            ai(0xa6f765);
            let point: *const [u16; 2] = *library.get(b"eproj_spawn_pt\0").unwrap();
            assert_eq!(*point, [102, 212], "external point is published synchronously");
            let events: Vec<_> = game
                .effects()
                .as_chunks::<16>()
                .0
                .iter()
                .map(|record| {
                    std::array::from_fn::<_, 4, _>(|i| u32::from_le_bytes(record[i * 4..i * 4 + 4].try_into().unwrap()))
                })
                .filter(|record| record[0] == 16 || record[0] == 17)
                .collect();
            assert_eq!(events, [[16, 0xe509, 12, 102 | 212 << 16], [17, 0xe509, 12, 102 | 212 << 16]]);
            let counts = game.ceres_door_counters();
            if mode == "service" {
                assert_eq!(counts.foreign_requests, 1);
                assert_eq!(counts.foreign_completions, 1);
                assert_eq!(counts.rust_completed[1], 1);
                assert_eq!(counts.c_entries, [0; 13]);
            } else {
                assert_eq!(counts.foreign_requests, 0);
                assert_eq!(counts.foreign_completions, 0);
                assert_eq!(counts.c_entries[1], 1);
            }
            let capture: unsafe extern "C" fn(*const u32, *const i16) = *library.get(b"sm_native_capture\0").unwrap();
            capture(game.pixels().as_ptr(), game.audio().as_ptr());
            let observation = game.observation_bytes();
            assert_eq!(&observation[..8], b"SMNOBS02");
            assert_eq!(observation.len(), 543791);
            assert_eq!(u64::from_le_bytes(observation[104..112].try_into().unwrap()), 102);
            assert_eq!(u64::from_le_bytes(observation[112..120].try_into().unwrap()), 212);
            println!("observation_bytes={} external_point=[102,212]", observation.len());
            println!("actual_spawn_effects={events:?}");
        } else {
            ai(0xa6f6c5);
            ai(0xa6f765);
            let pointer = game.rom().as_ptr().add(0x1375a2).cast::<u16>();
            for address in [0xf695, 0xf6b3, 0xf69f, 0xf66a, 0xf6b0, 0xf63e, 0xf6bd, 0xf678, 0xf68b, 0xf6a6] {
                let result = instr(0xa60000 | address, 0, pointer) as usize;
                assert!((0x8000..0x10000).contains(&result) && result & 1 == 0);
            }
            assert_eq!(setup(0x84b9f1, 0), 0);
            // Direct Instr5->Instr2 and Main->Steam5/Instr2 must not inflate
            // scheduled callback counts; non-door steam must bypass the hook.
            ram.copy_from_slice(game.wram());
            word(&mut ram, 0xfa8, 0xf7a5);
            word(&mut ram, 0x93f, 1);
            word(&mut ram, 0xfb8, 0xe1ff);
            commit(ram.as_ptr());
            ai(0xa6f765);
            let _ = instr(0xa6f68b, 64, pointer);
            let _ = instr(0xa6f6a6, 64, pointer);
            let counts = game.ceres_door_counters();
            let expected = [1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
            assert_eq!(counts.c_entries, expected);
            assert_eq!(counts.proximity_successes[0], 1);
            assert_eq!(counts.proximity_successes.iter().sum::<u64>(), 1);
            assert_eq!(counts.solid_setups, 1);
            if mode == "refuse" {
                assert_eq!(counts.rust_entries, expected);
                assert_eq!(counts.rust_refused, expected);
            }
            for at in [0x10002, 0x10022, 0x10042, 0x10062] {
                assert_eq!(read_word(game.wram(), at) & 0xf000, 0x8000);
            }
        }
        let mut hash = Sha256::new();
        hash.update(game.wram());
        hash.update(game.effects());
        println!("behavior_sha256={:x}", hash.finalize());
        println!("counters={:?}", game.ceres_door_counters());
    }
    Ok(())
}
