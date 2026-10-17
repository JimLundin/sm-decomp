//! Isolated host-boundary controls over recorded native boot input. Service
//! modes deliberately alter gameplay and are never a gameplay acceptance run.
use sha2::{Digest, Sha256};
use sm_native_host::{Game, HookOutcome};
use std::path::Path;

#[path = "pilot-host-control/escape.rs"]
mod escape;

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 5 {
        return Err("usage: pilot-host-control LIBRARY ROM NATIVE_INPUTS off|refuse|dirty-refuse|panic|service-clear|service-set|service-refuse".into());
    }
    let mode = args[4].clone();
    let mut game = Game::boot(Path::new(&args[1]), Path::new(&args[2]))?;
    if mode == "escape-update" {
        return escape::run(&mut game, Path::new(&args[1]));
    }
    if mode != "off" && mode != "off-full" {
        let mut service_done = false;
        let mode = mode.clone();
        game.set_projectile_hook(move |_, ram, _, services| match mode.as_str() {
            "refuse" => Ok(HookOutcome::Refused),
            "dirty-refuse" => {
                ram[0] ^= 1;
                Ok(HookOutcome::Refused)
            }
            "panic" => panic!("deliberate projectile callback panic"),
            "service-clear" | "service-set" | "service-refuse" if !service_done => {
                let header = if mode == "service-set" { 0xb637 } else { 0xb633 };
                let result = services.spawn_plm(ram, header)?;
                println!("actual SpawnPLM header={header:04x} result={result}");
                assert_eq!(result, u8::from(mode == "service-set"));
                let allocated = u16::from_le_bytes([ram[0x1c27], ram[0x1c28]]) as usize;
                assert!(allocated < 80 && allocated.is_multiple_of(2));
                assert_eq!(u16::from_le_bytes([ram[0x1c37 + allocated], ram[0x1c38 + allocated]]), header);
                service_done = true;
                Ok(if mode == "service-refuse" { HookOutcome::Refused } else { HookOutcome::Handled })
            }
            "service-clear" | "service-set" | "service-refuse" => Ok(HookOutcome::Refused),
            _ => Err("unknown control mode".into()),
        })?;
    }
    let inputs = std::fs::read_to_string(&args[3]).map_err(|e| e.to_string())?;
    let mut saw_service_trace = false;
    let mut maximum_effects = (0, 0);
    let input_count = inputs.lines().count();
    for (index, line) in inputs.lines().enumerate() {
        if index >= 20000 {
            return Err("control did not reach a scheduled projectile entry in bound".into());
        }
        let buttons = u16::from_str_radix(line.trim(), 16).map_err(|e| e.to_string())?;
        game.host_step(buttons)?;
        if game.effects().len() > maximum_effects.1 {
            maximum_effects = (index + 1, game.effects().len());
        }
        let counters = game.projectile_counters();
        if counters.foreign_calls == 1 && !saw_service_trace {
            let header = if mode == "service-set" { 0xb637 } else { 0xb633 };
            let words: Vec<[u32; 4]> = game
                .effects()
                .as_chunks::<16>()
                .0
                .iter()
                .map(|record| std::array::from_fn(|i| u32::from_le_bytes(record[i * 4..i * 4 + 4].try_into().unwrap())))
                .collect();
            let enter = words.iter().position(|r| r[0] == 3 && r[1] == header).expect("actual PLM enter trace");
            let exit =
                words.iter().enumerate().skip(enter + 1).find(|(_, r)| r[0] == 4).expect("actual PLM return trace");
            assert_eq!(exit.1[1], u32::from(mode == "service-set"));
            println!("ordered SpawnPLM trace enter_record={enter} return_record={}", exit.0);
            saw_service_trace = true;
        }
        if (mode == "off-full" && index + 1 == input_count)
            || (mode != "off-full"
                && counters.c_entries.iter().sum::<u64>() + counters.rust_completed.iter().sum::<u64>() > 4)
        {
            let mut hash = Sha256::new();
            hash.update(game.wram());
            hash.update(game.observation_bytes());
            hash.update(game.effects());
            println!("state={:?}", game.state());
            println!("behavior_sha256={:x}", hash.finalize());
            println!("external_bytes={} effects_bytes={}", game.observation_bytes().len(), game.effects().len());
            println!("counters={counters:?}");
            println!("maximum_effects_step={} bytes={}", maximum_effects.0, maximum_effects.1);
            if mode.starts_with("service-") {
                assert_eq!(counters.foreign_calls, 1);
                assert!(saw_service_trace);
            }
            return Ok(());
        }
    }
    Err("native inputs exhausted before scheduled projectile control".into())
}
