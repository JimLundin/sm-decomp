//! Verification only: actual 65816 selector-2 branch and ordered effects.
//! Captures contain ROM-derived RAM: keep them outside the repository.
use super::{Game, MakeSnapshot, RestoreSnapshot, Snapshot};
use crate::initial_state::InitialState;
use crate::snes::cpu::{cpu_getFlags, cpu_runOpcode};

// Primary sources: https://patrickjohnston.org/bank/91#D6F7 and #D8A5.
// Compare named executable/table windows with the user's local original ROM;
// no cartridge opcode bytes or captured output are embedded in source.
const ORIGINAL_WINDOWS: &[(u32, u32)] = &[
    (0x91d6f7, 0x91d998), // Dispatch, table, charge and misc branches.
    (0x91dd5b, 0x91ddd7), // Suit palette copy.
    (0x91deba, 0x91dee6), // Suit selection.
    (0x90f084, 0x90f0ae), // Samus command wrapper.
    (0x80902b, 0x8090a3), // Sound library 1 queue entries and shared body.
];

fn original_matches(game: &Game, original: &[u8], address: u32, length: usize) -> bool {
    let Some(rom) = game.loaded_rom.as_ref() else {
        return false;
    };
    let offset = crate::rtl::lorom_offset(address) as usize;
    offset.checked_add(length).is_some_and(|end| {
        end <= original.len()
            && end <= rom.len()
            && original[offset..end]
                .iter()
                .enumerate()
                .all(|(i, byte)| rom.read(offset + i) == *byte)
    })
}

#[derive(Clone, Copy)]
struct Case {
    flags: u16,
    hurt: u16,
    grapple: u16,
    movement: u16,
    flare: u16,
    shoot: u16,
    // (charge palette index, timer) injected at D712, never at routine entry.
    // These are explicit callback-transition experiments, not natural SNES paths.
    injected_transition: Option<(u16, u16)>,
}

fn cases() -> Vec<Case> {
    let base = Case {
        flags: 0,
        hurt: 0,
        grapple: 0xc4f0,
        movement: 0,
        flare: 0,
        shoot: 0,
        injected_transition: None,
    };
    let mut cases = Vec::new();
    for hurt in [0, 1, 2, 6, 7, 38, 58, 59, 60, 0x803a, 0x803b, 0xffff] {
        cases.push(Case { hurt, ..base });
    }
    for flags in [1, 2, 0x7fff] {
        cases.push(Case { flags, ..base });
    }
    for movement in [3, 0x14] {
        cases.push(Case {
            hurt: 39,
            movement,
            ..base
        });
    }
    for flare in [15, 16, 0x800f, 0x8010] {
        for shoot in [0, 0x40] {
            cases.push(Case {
                hurt: 39,
                flare,
                shoot,
                ..base
            });
        }
    }
    for grapple in [0xc855, 0xc856, 0x4855, 0x4856] {
        cases.push(Case {
            hurt: 39,
            grapple,
            ..base
        });
    }
    // IDs 0..28 retain the exact prior seed and byte-observation contract.
    // IDs 29..32 are injected callback transitions, in this fixed order.
    for index_timer in [(0, 2), (9, 2), (0, 1), (9, 1)] {
        cases.push(Case {
            injected_transition: Some(index_timer),
            ..base
        });
    }
    cases
}

fn put(game: &mut Game, address: usize, value: u16) {
    game.g_ram[address..address + 2].copy_from_slice(&value.to_le_bytes());
}

// Framing: tag u8, payload length u32 LE, payload.
fn record(output: &mut Vec<u8>, tag: u8, payload: &[u8]) {
    output.push(tag);
    output.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    output.extend_from_slice(payload);
}

unsafe fn observation(output: &mut Vec<u8>, tag: u8, game: &Game, full: bool) {
    unsafe {
        let cpu = &*game.g_cpu;
        let mut payload = Vec::new();
        payload.extend_from_slice(&(u32::from(cpu.k) << 16 | u32::from(cpu.pc)).to_le_bytes());
        payload.push(cpu_getFlags(cpu));
        for value in [cpu.a, cpu.x, cpu.y, cpu.sp, cpu.dp] {
            payload.extend_from_slice(&value.to_le_bytes());
        }
        payload.push(cpu.db);
        if full {
            payload.extend_from_slice(&game.g_ram);
        }
        record(output, tag, &payload);
    }
}

/// Requires an initialized, exclusive emulator and audio player. This captures
/// one case using actual CPU opcodes; it never calls the old Rust palette handler.
/// An error is a failed capture, never a passing comparison.
pub unsafe fn capture(id: u32, original: &[u8], game: &mut Game) -> Result<Vec<u8>, String> {
    unsafe {
        let case = *cases().get(id as usize).ok_or("unknown selector-2 case")?;
        if game.g_cpu.is_null() || game.g_snes.is_null() || game.g_spc_player.is_null() {
            return Err("selector-2 capture requires initialized emulator and audio".into());
        }
        for &(start, end) in ORIGINAL_WINDOWS {
            if !original_matches(game, original, start, (end - start) as usize) {
                return Err(format!(
                    "oracle code differs from local original at {start:06x}"
                ));
            }
        }
        let mut before = Box::new(Snapshot::INITIAL);
        MakeSnapshot(&mut before, &*game.g_snes, &*game.g_cpu, game.g_spc_player);
        let cpu_before = *game.g_cpu;
        let cycles_before = (*game.g_snes).cpuCyclesLeft;
        let memory_ops_before = (*game.g_snes).cpuMemOps;
        let calling_before = game.g_calling_asm_from_c;
        let return_before = game.g_calling_asm_from_c_ret;
        let queue_before = crate::audio_snapshot::ApuQueueSnapshot::capture(game);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            game.g_ram.fill(0);
            game.g_ram[0x1ffff] = 1;
            for (address, value) in [
                (0x0acc, 2),
                (0x0a4a, case.flags),
                (0x0a48, case.hurt),
                (0x0d32, case.grapple),
                (0x0a1f, case.movement),
                (0x0cd0, case.flare),
                (0x008b, case.shoot),
                (0x09b2, 0x40),
                (0x0a44, 0xe913),
                (0x0998, 8),
            ] {
                put(game, address, value);
            }
            let cpu = &mut *game.g_cpu;
            cpu.a = 0;
            cpu.x = 0;
            cpu.y = 0;
            cpu.sp = 0x1fff;
            cpu.dp = 0;
            cpu.db = 0x91;
            cpu.k = 0x91;
            cpu.pc = 0xd6f7;
            cpu.c = false;
            cpu.z = false;
            cpu.v = false;
            cpu.n = false;
            cpu.i = true;
            cpu.d = false;
            cpu.xf = false;
            cpu.mf = false;
            cpu.e = false;
            cpu.irqWanted = false;
            cpu.nmiWanted = false;
            cpu.waiting = false;
            cpu.stopped = false;
            cpu.spBreakpoint = 0;
            game.g_calling_asm_from_c = false;
            let mut output = b"SPOR1".to_vec();
            record(&mut output, 0, &id.to_le_bytes());
            let mut branch = false;
            let mut tail = false;
            let mut misc_calls = 0u8;
            let mut branch_pending = false;
            let mut injected = false;
            for _ in 0..20_000 {
                let pc = u32::from((*game.g_cpu).k) << 16 | u32::from((*game.g_cpu).pc);
                if branch_pending {
                    if pc != 0x91d717 && pc != 0x91d721 {
                        return Err(format!("unexpected branch destination {pc:06x}"));
                    }
                    observation(&mut output, 3, game, false);
                    branch_pending = false;
                }
                match pc {
                    0x91d6f7 => observation(&mut output, 1, game, true),
                    0x91d712 => {
                        if let Some((index, timer)) = case.injected_transition {
                            if injected {
                                return Err("duplicate injected callback transition".into());
                            }
                            // Charge processing and selector ASL/TAX have really
                            // executed. Preserve their CPU register/flag result;
                            // change only the three explicitly tagged RAM words.
                            if (*game.g_cpu).a != 4
                                || (*game.g_cpu).x != 4
                                || (*game.g_cpu).c
                                || (*game.g_cpu).mf
                                || (*game.g_cpu).xf
                            {
                                return Err(
                                    "injected transition has wrong dispatch CPU state".into()
                                );
                            }
                            let mut writes = pc.to_le_bytes().to_vec();
                            for (address, value) in
                                [(0x0a4au32, 0x8000), (0x0b62, index), (0x0ad0, timer)]
                            {
                                put(game, address as usize, value);
                                writes.extend_from_slice(&address.to_le_bytes());
                                writes.extend_from_slice(&value.to_le_bytes());
                            }
                            // Tag 9: injected-callback-transition, PC:u32 then
                            // exactly three (RAM-address:u32, value:u16) pairs.
                            record(&mut output, 9, &writes);
                            injected = true;
                        }
                    }
                    0x91d715 => {
                        if branch {
                            return Err("duplicate selector branch".into());
                        }
                        branch = true;
                        branch_pending = true;
                        observation(&mut output, 2, game, true);
                    }
                    0x91d721 => {
                        if tail {
                            return Err("duplicate final misc dispatch".into());
                        }
                        tail = true;
                        observation(&mut output, 4, game, true);
                    }
                    0x91d8a5 => {
                        misc_calls += 1;
                        observation(&mut output, 6, game, false);
                    }
                    0x91dd5b | 0x91deba | 0x80902b | 0x809049 | 0x90f084 => {
                        observation(&mut output, 7, game, false);
                    }
                    // After outer PLP, immediately before RTL: no artificial
                    // return address is necessary. C here is NOT the decision.
                    0x91d726 => {
                        if injected != case.injected_transition.is_some() {
                            return Err("missing injected callback transition".into());
                        }
                        if !branch || !tail || misc_calls != 2 {
                            return Err("missing selector-2 branch or misc call".into());
                        }
                        observation(&mut output, 5, game, true);
                        return Ok(output);
                    }
                    _ => {}
                }
                // Every executed instruction (maximum four bytes) must also
                // match the original, including nested helper paths.
                if !original_matches(game, original, pc, 4) {
                    return Err(format!("executed oracle code differs at {pc:06x}"));
                }
                cpu_runOpcode(game.g_cpu, game);
                // This routine has no DMA effects; unexpected DMA is a failed
                // setup, not an unbounded secondary loop.
                if (*(*game.g_snes).dma).dmaBusy {
                    return Err("unexpected DMA during palette capture".into());
                }
            }
            Err("selector-2 instruction budget exhausted".into())
        }));
        RestoreSnapshot(&before, &mut *game.g_snes, &mut *game.g_cpu);
        before.audio.restore(game.g_snes);
        before.player.restore(game.g_spc_player);
        *game.g_cpu = cpu_before;
        (*game.g_snes).cpuCyclesLeft = cycles_before;
        (*game.g_snes).cpuMemOps = memory_ops_before;
        queue_before.restore(game);
        game.g_calling_asm_from_c = calling_before;
        game.g_calling_asm_from_c_ret = return_before;
        result.unwrap_or_else(|_| Err("selector-2 capture panicked".into()))
    }
}
