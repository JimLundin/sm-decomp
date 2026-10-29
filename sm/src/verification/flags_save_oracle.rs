//! Untouched original CPU SaveToSram. Never calls the unaligned old Rust save body.
use super::{Game, MakeSnapshotWithPlayerOwner, RestoreSnapshot, Snapshot};
use crate::{flags_save_case as case, initial_state::InitialState, snes::cpu::cpu_getFlags};
use std::collections::BTreeSet;
fn offset(address: u32) -> usize {
    crate::rtl::lorom_offset(address) as usize
}
fn matches_original(game: &Game, original: &[u8], address: u32, length: usize) -> bool {
    let Some(image) = game.loaded_rom.as_ref() else {
        return false;
    };
    let start = offset(address);
    start.checked_add(length).is_some_and(|end| {
        end <= original.len()
            && end <= image.len()
            && original[start..end]
                .iter()
                .enumerate()
                .all(|(i, b)| image.read(start + i) == *b)
    })
}
pub(crate) unsafe fn guard(game: &Game, original: &[u8]) -> Result<(), String> {
    unsafe {
        let loaded = game.loaded_rom.as_ref().ok_or("missing loaded save ROM")?;
        let native = game
            .capture_native_rom_binding()
            .ok_or("missing native save ROM")?;
        if loaded.as_mut_ptr() != native.as_mut_ptr()
            || loaded.as_mut_ptr() != (*(*game.g_snes).cart).rom.as_mut_ptr()
        {
            return Err("save oracle requires shared initialized ROM bindings".into());
        }
        for (address, length) in [(0x818000, 0x85), (0x81812b, 0x1b9), (0x81834b, 0x54)] {
            if !matches_original(game, original, address, length) {
                return Err(format!(
                    "save oracle original window mismatch {address:06x}"
                ));
            }
        }
        // Each original packing list is read as words before its low byte is used.
        for area in 0..6usize {
            let count = original[offset(0x818131) + area] as usize;
            if count == 0 {
                return Err("original map packing count must be positive".into());
            }
            let at = offset(0x8182d6) + area * 2;
            let list = u16::from_le_bytes([original[at], original[at + 1]]);
            if list < 0x8000
                || usize::from(list) + count + 1 > 0x10000
                || !matches_original(game, original, 0x810000 | u32::from(list), count + 1)
            {
                return Err("original map packing list mismatch".into());
            }
        }
        Ok(())
    }
}
pub(crate) unsafe fn with_initialized_state<T>(
    game: &mut Game,
    body: impl FnOnce(&mut Game) -> Result<T, String>,
) -> Result<T, String> {
    unsafe {
        if game.g_cpu.is_null()
            || game.g_snes.is_null()
            || game.g_spc_player.is_null()
            || (*game.g_snes).cart.is_null()
            || (*(*game.g_snes).cart).ramSize != 8192
            || !game.VERIFY_ACTIVE
        {
            return Err(
                "save capture requires initialized verify-only CPU/audio/8192-byte SRAM".into(),
            );
        }
        let game_sram = game.g_sram.clone();
        let game_bytes = game_sram.bytes(0..8192);
        let cart_sram = (*(*game.g_snes).cart).ram.clone();
        let mut before = Box::new(Snapshot::INITIAL);
        {
            let snapshot_snes = game.g_snes;
            let snapshot_cpu = game.g_cpu;
            let snapshot_player = game.g_spc_player;
            let (backend, ram, _, _, _, owner) = game.ram_and_owned_snapshots();
            MakeSnapshotWithPlayerOwner(
                &mut before,
                &*snapshot_snes,
                &*snapshot_cpu,
                snapshot_player,
                owner,
                ram,
                backend,
            )
        };
        let cpu_before = *game.g_cpu;
        let cycles_before = (*game.g_snes).cpuCyclesLeft;
        let memory_before = (*game.g_snes).cpuMemOps;
        let calling_before = game.g_calling_asm_from_c;
        let return_before = game.g_calling_asm_from_c_ret;
        let fail_before = game.g_fail;
        let hooks_before = (
            game.hookmode,
            game.hookcnt,
            game.hookadr,
            game.hook_fixbug_orgbyte,
        );
        let queue_before = crate::audio_snapshot::ApuQueueSnapshot::capture(game);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // Both allocations are preflighted before this reversible identity probe.
            if game_bytes != cart_sram.bytes(0..8192) {
                return Err("save SRAM bindings disagree before capture".into());
            }
            let old = game_sram.read(0);
            game_sram.write(0, old ^ 0xff);
            let shared = cart_sram.read(0) == old ^ 0xff;
            game_sram.write(0, old);
            if !shared {
                return Err("save requires game/cartridge shared SRAM allocation".into());
            }
            body(game)
        }));
        (*(*game.g_snes).cart).ram = cart_sram;
        game.g_sram = game_sram;
        {
            let snapshot_snes = game.g_snes;
            let snapshot_cpu = game.g_cpu;
            let (backend, ram, _, _, _, _) = game.ram_and_owned_snapshots();
            RestoreSnapshot(
                &before,
                &mut *snapshot_snes,
                &mut *snapshot_cpu,
                ram,
                backend,
            )
        };
        game.g_sram.write_bytes(0, &game_bytes);
        before.audio.restore(game.g_snes);
        game.restore_native_player_snapshot(crate::game_state::PlayerSnapshotSource::Local(
            &before.player,
        ));
        *game.g_cpu = cpu_before;
        (*game.g_snes).cpuCyclesLeft = cycles_before;
        (*game.g_snes).cpuMemOps = memory_before;
        queue_before.restore(game);
        game.g_calling_asm_from_c = calling_before;
        game.g_calling_asm_from_c_ret = return_before;
        game.g_fail = fail_before;
        (
            game.hookmode,
            game.hookcnt,
            game.hookadr,
            game.hook_fixbug_orgbyte,
        ) = hooks_before;
        result.unwrap_or_else(|_| Err("save capture panicked".into()))
    }
}
unsafe fn cpu_header(game: &Game) -> [u8; 16] {
    unsafe {
        let cpu = &*game.g_cpu;
        let mut out = [0; 16];
        out[..4].copy_from_slice(&(u32::from(cpu.k) << 16 | u32::from(cpu.pc)).to_le_bytes());
        out[4] = cpu_getFlags(cpu);
        for (i, value) in [cpu.a, cpu.x, cpu.y, cpu.sp, cpu.dp]
            .into_iter()
            .enumerate()
        {
            out[5 + i * 2..7 + i * 2].copy_from_slice(&value.to_le_bytes());
        }
        out[15] = cpu.db;
        out
    }
}
fn add(set: &mut BTreeSet<usize>, address: usize, length: usize) {
    for at in address..address + length {
        set.insert(at);
    }
}
fn observe_writes(
    pc: u32,
    sp: u16,
    x: u16,
    y: u16,
    abi: &mut BTreeSet<usize>,
    ram: &mut BTreeSet<usize>,
    sram: &mut BTreeSet<usize>,
) -> Result<(), String> {
    let scratch = match pc {
        0x818011 => Some(0x12),
        0x81800b | 0x81805d => Some(0x14),
        0x818351 | 0x818393 => Some(0x1a),
        0x81835b | 0x81838f => Some(0x16),
        0x818364 | 0x81838c => Some(0),
        0x81836d => Some(0x18),
        0x818372 => Some(3),
        0x818377 => Some(5),
        _ => None,
    };
    if let Some(at) = scratch {
        add(abi, at, 2);
    }
    let push: u16 = match pc {
        0x818000 | 0x818003 | 0x81834b | 0x81834c | 0x81834d => 1,
        0x818004 | 0x818005 | 0x818006 | 0x81803a => 2,
        _ => 0,
    };
    for i in 0..push {
        abi.insert(usize::from(sp.wrapping_sub(i)));
    }
    match pc {
        0x818019 => add(ram, 0xd7c0 + usize::from(y), 2),
        0x81802e => add(ram, 0xcd52 + usize::from(x), 2),
        0x818386 => add(ram, 0xd91c + usize::from(x), 1),
        0x818040 => add(ram, 0xd916, 2),
        0x818046 => add(ram, 0xd918, 2),
        _ => (),
    }
    let save = match pc {
        0x818056 | 0x81806c => Some(usize::from(x)),
        0x818070 => Some(0x1ff0 + usize::from(x)),
        0x818077 => Some(8 + usize::from(x)),
        0x81807b => Some(0x1ff8 + usize::from(x)),
        _ => None,
    };
    if let Some(at) = save {
        if at + 2 > 8192 {
            return Err("original save wrote outside SRAM".into());
        }
        add(sram, at, 2);
    }
    Ok(())
}
pub unsafe fn capture(id: u32, original: &[u8], game: &mut Game) -> Result<Vec<u8>, String> {
    unsafe {
        with_initialized_state(game, |game| {
            guard(game, original)?;
            let slot = case::seed(id, game)?;
            let initial = (0..game.ram().len())
                .map(|at| game.ram().read_byte(at))
                .collect::<Vec<_>>();
            let initial_sram = game.g_sram.bytes(0..8192);
            let cpu = &mut *game.g_cpu;
            cpu.a = slot;
            cpu.x = 0x1357;
            cpu.y = 0x2468;
            cpu.sp = 0x1fff;
            cpu.dp = 0;
            cpu.db = 0xa5;
            cpu.k = 0x81;
            cpu.pc = 0x8000;
            cpu.c = true;
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
            let flags = cpu_getFlags(&*game.g_cpu);
            let mut output = b"FSOR1".to_vec();
            case::record(&mut output, 0, &id.to_le_bytes());
            let mut abi = BTreeSet::new();
            let mut ram = BTreeSet::new();
            let mut sram = BTreeSet::new();
            let mut stage = 0usize;
            for _ in 0..100_000 {
                let cpu = &*game.g_cpu;
                let pc = u32::from(cpu.k) << 16 | u32::from(cpu.pc);
                if stage < case::STAGES.len() && pc == 0x810000 | u32::from(case::STAGES[stage]) {
                    for (at, (&before, after)) in initial
                        .iter()
                        .zip((0..game.ram().len()).map(|at| game.ram().read_byte(at)))
                        .enumerate()
                    {
                        if before != after && !abi.contains(&at) && !ram.contains(&at) {
                            return Err(format!(
                                "unaccounted save RAM write stage{stage} address{at:05x}"
                            ));
                        }
                    }
                    let current_sram = game.g_sram.bytes(0..8192);
                    for (at, (&before, &after)) in
                        initial_sram.iter().zip(current_sram.iter()).enumerate()
                    {
                        if before != after && !sram.contains(&at) {
                            return Err(format!(
                                "unaccounted save SRAM write stage{stage} address{at:04x}"
                            ));
                        }
                    }
                    if {
                        let mut bytes = [0; 0x5e9 - (0x5e7)];
                        game.ram().copy_out(0x5e7, &mut bytes);
                        bytes
                    } != initial[0x5e7..0x5e9]
                    {
                        return Err("save changed gameplay scratch mask".into());
                    }
                    case::state(&mut output, stage as u8, cpu_header(game), game);
                    case::mask(&mut output, stage as u8, &abi);
                    stage += 1;
                    if stage == 7 {
                        if cpu.x != 0x1357
                            || cpu.y != 0x2468
                            || cpu.sp != 0x1fff
                            || cpu.dp != 0
                            || cpu.db != 0xa5
                            || cpu_getFlags(cpu) != flags
                        {
                            return Err("original save failed caller restoration".into());
                        }
                        return Ok(output);
                    }
                }
                if !matches_original(game, original, pc, 4) {
                    return Err(format!("executed save code differs {pc:06x}"));
                }
                observe_writes(pc, cpu.sp, cpu.x, cpu.y, &mut abi, &mut ram, &mut sram)?;
                game.execution().step_cpu();
                if game.dma_busy() {
                    return Err("unexpected DMA in original save capsule".into());
                }
            }
            Err("original save opcode budget exhausted".into())
        })
    }
}
