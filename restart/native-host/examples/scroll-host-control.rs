//! Isolated native-boundary controls. Service modes inject test descriptors and
//! deliberately alter gameplay; these runs are not gameplay acceptance evidence.
use sha2::{Digest, Sha256};
use sm_native_host::{Game, HookOutcome};
use std::path::Path;

fn word(ram: &mut [u8], at: usize, value: u16) {
    ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
}

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 5 {
        return Err("usage: scroll-host-control LIBRARY ROM INPUTS off|refuse|dirty-refuse|panic|service|service-refuse|wrong-ram-refuse".into());
    }
    let mode = args[4].clone();
    if !["off", "refuse", "dirty-refuse", "panic", "service", "service-refuse", "wrong-ram-refuse"]
        .contains(&mode.as_str())
    {
        return Err("unknown control mode".into());
    }
    let mut game = Game::boot(Path::new(&args[1]), Path::new(&args[2]))?;
    if mode != "off" {
        let mut service_done = false;
        let mode = mode.clone();
        game.set_scroll_hook(move |_, ram, services| match mode.as_str() {
            "refuse" => Ok(HookOutcome::Refused),
            "dirty-refuse" => {
                ram[0] ^= 1;
                Ok(HookOutcome::Refused)
            }
            "panic" => panic!("deliberate scroll callback panic"),
            "wrong-ram-refuse" => {
                let mut unrelated = ram.to_vec();
                assert!(services.flush_vram(&mut unrelated).is_err());
                Ok(HookOutcome::Refused)
            }
            "service" | "service-refuse" if !service_done => {
                // One two-byte queue transfer and four two-half row/column
                // transfers of two bytes each: exactly 18 real VRAM DMA bytes.
                word(ram, 0x330, 7);
                ram[0xd0..0xd7].copy_from_slice(&[2, 0, 0, 0, 0x7e, 0x34, 0x12]);
                for descriptor in [0x956, 0x964, 0x972, 0x980] {
                    ram[descriptor..descriptor + 14].fill(0);
                    word(ram, descriptor, 2);
                    word(ram, descriptor + 4, 0x2000);
                    word(ram, descriptor + 12, 0xab01);
                }
                services.flush_vram(ram)?;
                assert_eq!(&ram[0x330..0x332], &[0, 0]);
                for enable in [0x962, 0x970, 0x97e, 0x98c] {
                    assert_eq!(&ram[enable..enable + 2], &[0, 0xab], "flush must copy back cleared low enable byte");
                }
                println!("synchronous VRAM queue and four row/column enables copied back");
                service_done = true;
                Ok(if mode == "service-refuse" { HookOutcome::Refused } else { HookOutcome::Handled })
            }
            "service" | "service-refuse" => Ok(HookOutcome::Refused),
            _ => unreachable!(),
        })?;
    }
    let inputs = std::fs::read_to_string(&args[3]).map_err(|e| e.to_string())?;
    for (index, line) in inputs.lines().take(20000).enumerate() {
        game.host_step(u16::from_str_radix(line.trim(), 16).map_err(|e| e.to_string())?)?;
        let counters = game.scroll_counters();
        if counters.c_entries.iter().sum::<u64>() + counters.rust_completed.iter().sum::<u64>() == 0 {
            continue;
        }
        if mode == "service" {
            assert_eq!(counters.foreign_requests, 1);
            assert_eq!(counters.foreign_completions, 1);
            assert!(counters.consumed_columns.iter().all(|&count| count > 0));
            assert!(counters.consumed_rows.iter().all(|&count| count > 0));
            let records: Vec<[u32; 4]> = game
                .effects()
                .as_chunks::<16>()
                .0
                .iter()
                .map(|r| std::array::from_fn(|i| u32::from_le_bytes(r[i * 4..i * 4 + 4].try_into().unwrap())))
                .collect();
            let enter = records
                .iter()
                .position(|r| r[0] == 12 && r[1] != 0 && r[2] == 7 && r[3] == 15)
                .expect("actual flush entry");
            let end = records[enter + 1..].iter().position(|r| r[0] == 13).unwrap() + enter + 1;
            assert_eq!(records[end], [13, records[enter][1], 18, 0]);
            assert_eq!(records[enter + 1..end].iter().filter(|r| r[0] == 1 && r[1] == 0x420b && r[2] == 2).count(), 9);
            println!("ordered actual flush: 9 DMA requests, 18 VRAM bytes");
        }
        let mut hash = Sha256::new();
        hash.update(game.wram());
        hash.update(game.observation_bytes());
        hash.update(game.effects());
        println!("step={} behavior_sha256={:x}", index + 1, hash.finalize());
        println!("counters={counters:?}");
        return Ok(());
    }
    Err("inputs exhausted before scheduled scroll entry".into())
}
