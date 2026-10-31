//! Original-65816 alignment references; authored animation data is labelled.
//! No invocation of the legacy Rust alignment-sensitive routines.
use super::{Game, Snapshot, make_snapshot_with_player_owner, restore_snapshot};
use crate::initial_state::InitialState;
use crate::native_rom::NativeRomPointer;
use crate::snes::cpu::cpu_get_flags;

pub const CASES: u32 = 11;
pub const CAPACITY: usize = 2 * 1024 * 1024;
const STACK_TOP: u16 = 0x1fff;
// End-exclusive windows. Executed instructions are also checked individually.
const WINDOWS: &[(u32, u32)] = &[
    (0x91f433, 0x91f50c),
    (0x91f543, 0x91f5eb),
    (0x91f88c, 0x91f8ae),
    (0x91f8d3, 0x91f952),
    (0x91f9c2, 0x91f9f4),
    (0x91deba, 0x91dee6),
    (0x91dd5b, 0x91ddd7),
    (0x90cffa, 0x90d068),
    (0x9082dc, 0x908344),
    (0x90848b, 0x9084b6),
    (0x90852c, 0x9085e2),
    (0x91b010, 0x91b210),
    (0x91b629, 0x91be11),
    (0x9b9400, 0x9b9420),
];

// SnesInit's eighth FixBugHook changes one cell inside the speed helper.
// Restore the recognized original-file cell only for this capture; Drop also
// covers window rejection and unwinding, which Snapshot cannot do for ROM.
struct OriginalSpeedHook {
    image: crate::rom::RomImage,
    at: usize,
    initialized: u8,
}
impl Drop for OriginalSpeedHook {
    fn drop(&mut self) {
        self.image.write(self.at, self.initialized);
    }
}
fn original_speed_hook(game: &Game, original: &[u8]) -> Result<OriginalSpeedHook, String> {
    let image = game
        .loaded_rom
        .as_ref()
        .ok_or("missing oracle image")?
        .clone();
    let at = offset(0x9085aa);
    let original_byte = *original.get(at).ok_or("short original speed hook")?;
    if at >= image.len() {
        return Err("short initialized speed hook".into());
    }
    let initialized = image.read(at);
    if initialized != original_byte
        && !(initialized == 0
            && game.hookmode == 1
            && game.hookcnt >= 8
            && game.hook_fixbug_orgbyte[7] == original_byte)
    {
        return Err("unexpected speed hook; not the saved startup hook".into());
    }
    let restore = OriginalSpeedHook {
        image,
        at,
        initialized,
    };
    restore.image.write(at, original_byte);
    Ok(restore)
}

fn offset(address: u32) -> usize {
    crate::rtl::lorom_offset(address) as usize
}
fn original_matches(game: &Game, original: &[u8], address: u32, length: usize) -> bool {
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
fn word(game: &Game, at: usize) -> u16 {
    u16::from_le_bytes([game.ram().read_byte(at), game.ram().read_byte(at + 1)])
}
fn overlaps_authored_data(id: u32, pc: u32) -> bool {
    if id < 7 {
        return false;
    }
    let stream = offset(if id == 10 { 0x91b401 } else { 0x91b400 });
    let at = offset(pc);
    [(offset(0x91b012), 2), (stream, 6)]
        .into_iter()
        .any(|(start, length)| at < start + length && start < at + 4)
}
fn put(game: &mut Game, at: usize, value: u16) {
    game.ram().copy_in(at, &value.to_le_bytes());
}
fn record(out: &mut Vec<u8>, tag: u8, payload: &[u8]) {
    out.push(tag);
    out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    out.extend_from_slice(payload);
}
unsafe fn observation(out: &mut Vec<u8>, tag: u8, game: &Game, full: bool) {
    unsafe {
        let cpu = &*game.g_cpu;
        let mut payload = (u32::from(cpu.k) << 16 | u32::from(cpu.pc))
            .to_le_bytes()
            .to_vec();
        payload.push(cpu_get_flags(cpu));
        for value in [cpu.a, cpu.x, cpu.y, cpu.sp, cpu.dp] {
            payload.extend_from_slice(&value.to_le_bytes());
        }
        payload.push(cpu.db);
        if full {
            payload.extend((0..game.ram().len()).map(|at| game.ram().read_byte(at)));
            payload.push(u8::from(game.g_fail));
        }
        record(out, tag, &payload);
    }
}
fn stack_bytes(id: u32) -> u16 {
    match id {
        1 | 2 => 13,
        4 => 9,
        5 | 6 => 3,
        _ => 7,
    }
}
fn checkpoints(id: u32) -> &'static [u32] {
    match id {
        0..=3 => &[0x91f433, 0x91f440, 0x91f446, 0x91f476, 0x91f4a0, 0x91f467],
        4 => &[0x91f468, 0x91f476, 0x91f47f, 0x91f485, 0x91f4a0, 0x91f4a1],
        5 | 6 => &[
            0x91f468, 0x91f476, 0x91f47f, 0x91f485, 0x91f490, 0x91f49a, 0x91f4a0, 0x91f4a1,
        ],
        _ => &[
            0x9082dc, 0x90848c, 0x90848e, 0x908495, 0x9084b4, 0x908313, 0x908323,
        ],
    }
}
fn effects(id: u32) -> &'static [u32] {
    match id {
        1 | 2 => &[0x91f4dc, 0x91deba, 0x91dd5b],
        0 | 3 => &[0x91f4dc],
        4 => &[0x91f543, 0x90cffa],
        5 | 6 => &[0x91f88c, 0x91f8d3],
        _ => &[0x90852c, 0x90848b],
    }
}
fn observed_effect(pc: u32) -> bool {
    matches!(
        pc,
        0x91f4dc
            | 0x91deba
            | 0x91dd5b
            | 0x91f543
            | 0x90cffa
            | 0x91f88c
            | 0x91f8d3
            | 0x90852c
            | 0x90848b
            | 0x80902b
            | 0x809049
            | 0x90bcbe
    )
}
// Actual word stores whose ordering defines this boundary. Record after opcode.
fn publication(pc: u32) -> Option<usize> {
    Some(match pc {
        0x91f443 | 0x91f482 | 0x91f49d => 0xa1e,
        0x91f56c | 0x91f8a1 | 0x91f8a9 | 0x91f923 => 0xa1c,
        0x91f592 => 0xafa,
        0x91f595 => 0xb14,
        0x90848e => 0x12,
        0x90849e | 0x9084ab => 0xa2c,
        0x9084b1 => 0xa32,
        p if (0x91dd67..=0x91ddd0).contains(&p) && (p - 0x91dd67) % 7 == 0 => {
            0xc180 + ((p - 0x91dd67) / 7 * 2) as usize
        }
        _ => return None,
    })
}
fn expected_publications(id: u32) -> Vec<u32> {
    match id {
        0 | 3 => vec![0x91f443],
        1 | 2 => core::iter::once(0x91f443)
            .chain((0..16).map(|i| 0x91dd67 + i * 7))
            .collect(),
        4 => vec![0x91f56c, 0x91f592, 0x91f595, 0x91f482],
        5 => vec![0x91f8a1, 0x91f482, 0x91f923, 0x91f49d],
        6 => vec![0x91f8a9, 0x91f482, 0x91f923, 0x91f49d],
        _ => vec![
            0x90848e,
            if id == 9 { 0x90849e } else { 0x9084ab },
            0x9084b1,
        ],
    }
}

/// Exclusive initialized original-ROM emulator; root alone executes this capture.
pub unsafe fn capture(id: u32, original: &[u8], game: &mut Game) -> Result<Vec<u8>, String> {
    unsafe {
        if id >= CASES {
            return Err("unknown player transition oracle case".into());
        }
        if original.len() != 0x300000
            || game.g_cpu.is_null()
            || game.g_snes.is_null()
            || game.g_spc_player.is_null()
        {
            return Err(
                "player transition oracle requires original 3 MiB ROM and initialized host".into(),
            );
        }
        let image = game
            .loaded_rom
            .as_ref()
            .ok_or("ROM must be loaded")?
            .clone();
        // SnesInit normally binds all three to this one allocation. Never shift it.
        if image.as_mut_ptr() != (*(*game.g_snes).cart).rom.as_mut_ptr()
            || image.as_mut_ptr() != game.capture_native_rom().as_mut_ptr()
        {
            return Err("oracle image bindings differ".into());
        }
        let _original_speed_hook = original_speed_hook(game, original)?;
        for &(start, end) in WINDOWS {
            if !original_matches(game, original, start, (end - start) as usize) {
                return Err(format!("oracle original window differs at {start:06x}"));
            }
        }
        // Original pose IDs, not content substitutions. Check the local table.
        for (pose, facing, movement) in [
            (1, 8, 0),
            (2, 4, 0),
            (0x49, 8, 0x10),
            (0x4a, 4, 0x10),
            (0xc7, 8, 0x1b),
        ] {
            let at = offset(0x91b629) + pose * 8;
            if original[at] != facing || original[at + 1] != movement {
                return Err("unsupported original pose metadata".into());
            }
        }
        let mut before = Box::new(Snapshot::INITIAL);
        {
            let snapshot_snes = game.g_snes;
            let snapshot_cpu = game.g_cpu;
            let snapshot_player = game.g_spc_player;
            let (backend, ram, _, _, _, owner) = game.ram_and_owned_snapshots();
            make_snapshot_with_player_owner(
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
        let memory_ops_before = (*game.g_snes).cpuMemOps;
        let calling_before = game.g_calling_asm_from_c;
        let return_before = game.g_calling_asm_from_c_ret;
        let fail_before = game.g_fail;
        let queue_before = crate::audio_snapshot::ApuQueueSnapshot::capture(game);
        let mut restored_bytes = Vec::new();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            game.ram().fill(0..game.ram().len(), 0);
            game.ram().write_byte(0x1ffff, 1);
            game.g_fail = false;
            let (pose, facing_type, old_pose, old_type, items) = match id {
                0 => (1, 0x7654, 0, 0, 8),
                1 => (2, 0x7654, 0, 3, 8),
                2 => (2, 0x7654, 0, 0x14, 8),
                3 => (2, 0x7654, 0, 3, 0),
                4 => (0x4d, 0x0208, 0, 2, 0),
                5 => (0x49, 0x1008, 1, 0, 0),
                6 => (0x4a, 0x1004, 2, 0, 0),
                7 => (1, 8, 0, 0, 8),
                8 | 10 => (1, 8, 0, 0, 0x2000),
                _ => (1, 8, 0, 0, 0x20),
            };
            for (at, value) in [
                (0xa1c, pose),
                (0xa1e, facing_type),
                (0xa20, old_pose),
                (0x9a2, items),
                (0xa68, u16::from(id == 4)),
                (0xafa, 0x140),
                (0xb14, 0x130),
                (0xa2c, 0x3456),
                (0xa32, 0x4567),
                (0xa94, 0x5678),
                (0xa96, 0),
                (0xa9c, 0x6789),
            ] {
                put(game, at, value);
            }
            game.ram().write_byte(0xa23, old_type);
            game.ram().fill(0xc180..0xc1a0, 0x5a);
            // These seeds make the DP scratch exclusions independently visible.
            game.ram().copy_in(0, &[0xa5, 0x5a, 0x7e]);
            put(game, 0x12, 0x6b7c);
            let mut output = b"PTOR1".to_vec();
            let mut identity = id.to_le_bytes().to_vec();
            identity.push(u8::from(id >= 7)); // 0 original data; 1 authored data.
            record(&mut output, 0, &identity);
            if id >= 7 {
                // Real logical data placement, not native allocation realignment.
                let address: u32 = if id == 10 { 0x91b401 } else { 0x91b400 };
                let pointer = (address as u16).to_le_bytes();
                let stream = [0x8c, 0x08, 0x20, 0x11, 0x22, 0x5a];
                let mut authored = Vec::new();
                for (at, bytes) in [(0x91b012, pointer.as_slice()), (address, stream.as_slice())] {
                    authored.extend_from_slice(&at.to_le_bytes());
                    authored.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
                    authored.extend_from_slice(bytes);
                    for (i, value) in bytes.iter().enumerate() {
                        let at = offset(at) + i;
                        restored_bytes.push((at, image.read(at)));
                        image.write(at, *value);
                    }
                }
                record(&mut output, 8, &authored);
            }
            let mut exclusions = Vec::new();
            let mut exclude = |start: u32, end: u32, reason: u8| {
                exclusions.extend_from_slice(&start.to_le_bytes());
                exclusions.extend_from_slice(&end.to_le_bytes());
                exclusions.push(reason);
            };
            exclude(u32::from(STACK_TOP - stack_bytes(id) + 1), 0x2000, 1);
            if id >= 7 {
                exclude(0, 3, 2);
                exclude(0x12, 0x14, 2);
            }
            record(&mut output, 9, &exclusions);
            let cpu = &mut *game.g_cpu;
            cpu.a = 0;
            cpu.x = 0;
            cpu.y = 0;
            cpu.sp = STACK_TOP;
            cpu.dp = 0;
            cpu.db = 0x91;
            let entry = checkpoints(id)[0];
            cpu.k = (entry >> 16) as u8;
            cpu.pc = entry as u16;
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
            let mut points = Vec::new();
            let mut calls = Vec::new();
            let mut stores = Vec::new();
            let mut minimum_sp = STACK_TOP;
            let mut pending_branch = None;
            for _ in 0..20_000 {
                let pc = u32::from((*game.g_cpu).k) << 16 | u32::from((*game.g_cpu).pc);
                if let Some(expected) = pending_branch.take() {
                    if pc != expected {
                        return Err(format!("unexpected decision branch {pc:06x}"));
                    }
                    observation(&mut output, 3, game, false);
                }
                if checkpoints(id).contains(&pc) {
                    let index = points.len();
                    if checkpoints(id).get(index) != Some(&pc) {
                        return Err("duplicate/out-of-order phase".into());
                    }
                    points.push(pc);
                    let last = index + 1 == checkpoints(id).len();
                    observation(
                        &mut output,
                        if index == 0 {
                            1
                        } else if last {
                            5
                        } else {
                            2
                        },
                        game,
                        true,
                    );
                    if matches!(pc, 0x91f440 | 0x91f47f | 0x91f49a | 0x90848c)
                        && ((*game.g_cpu).mf
                            || (*game.g_cpu).xf
                            || (*game.g_cpu).db != 0x91
                            || (*game.g_cpu).dp != 0)
                    {
                        return Err("wrong word-read CPU widths/bank/direct page".into());
                    }
                    if pc == 0x91f476 {
                        if (*game.g_cpu).c != (id >= 4) {
                            return Err("wrong normalization callback carry".into());
                        }
                        pending_branch = Some(if (*game.g_cpu).c { 0x91f478 } else { 0x91f4a0 });
                    }
                    if pc == 0x90848c {
                        let address = if id == 10 { 0xb401 } else { 0xb400 };
                        if (*game.g_cpu).y != 1
                            || word(game, 0) != address
                            || game.ram().read_byte(2) != 0x91
                        {
                            return Err("wrong authored operand location".into());
                        }
                    }
                    if pc == 0x90848e && (*game.g_cpu).a != 0x2008 {
                        return Err("wrong little-endian instruction mask".into());
                    }
                    if pc == 0x908495 {
                        let matched = id != 9;
                        if (*game.g_cpu).z == matched {
                            return Err("wrong equipment mask result".into());
                        }
                        pending_branch = Some(if matched { 0x9084a3 } else { 0x908497 });
                    }
                    if pc == 0x908313 {
                        if (*game.g_cpu).c {
                            return Err("command12 must return carry clear".into());
                        }
                        pending_branch = Some(0x908321);
                    }
                    if last {
                        if calls != effects(id)
                            || stores != expected_publications(id)
                            || minimum_sp != STACK_TOP - stack_bytes(id)
                            || (*game.g_cpu).sp != STACK_TOP
                            || cpu_get_flags(&*game.g_cpu) != 0x05
                            || (*game.g_cpu).db != 0x91
                            || (*game.g_cpu).dp != 0
                            || game.g_fail
                        {
                            return Err(format!(
                                "missing effects/stores or unexpected stack footprint in case {id}"
                            ));
                        }
                        if id >= 7
                            && (word(game, 0xa2c) != if id == 9 { 0x11 } else { 0x22 }
                                || word(game, 0xa32) != 3
                                || word(game, 0xa94) != 0x5678
                                || word(game, 0xa96) != 0)
                        {
                            return Err("command12 final publications differ".into());
                        }
                        if output.len() > CAPACITY {
                            return Err("oracle output capacity exceeded".into());
                        }
                        return Ok(output);
                    }
                }
                if observed_effect(pc) {
                    calls.push(pc);
                    observation(&mut output, 7, game, true);
                }
                // Do not authorize authored data as executable code.
                if !matches!(pc >> 16, 0x80 | 0x90 | 0x91) || pc & 0x8000 == 0 {
                    return Err(format!(
                        "execution outside oracle LoROM code mapping at {pc:06x}"
                    ));
                }
                if overlaps_authored_data(id, pc) {
                    return Err(format!("instruction overlaps authored data at {pc:06x}"));
                }
                if !original_matches(game, original, pc, 4) {
                    return Err(format!("executed original instruction differs at {pc:06x}"));
                }
                let published = publication(pc).map(|at| (at, word(game, at)));
                game.execution().step_cpu();
                minimum_sp = minimum_sp.min((*game.g_cpu).sp);
                if let Some((at, before)) = published {
                    stores.push(pc);
                    let mut bytes = pc.to_le_bytes().to_vec();
                    bytes.extend_from_slice(&(at as u32).to_le_bytes());
                    bytes.extend_from_slice(&before.to_le_bytes());
                    bytes.extend_from_slice(&word(game, at).to_le_bytes());
                    record(&mut output, 4, &bytes);
                }
                if game.g_fail || game.dma_busy() || (*game.g_cpu).sp > STACK_TOP {
                    return Err("unexpected diagnostic/DMA/stack exit".into());
                }
            }
            Err("player transition instruction budget exhausted".into())
        }));
        for (at, byte) in restored_bytes {
            image.write(at, byte);
        }
        {
            let snapshot_snes = game.g_snes;
            let snapshot_cpu = game.g_cpu;
            let (backend, ram, _, _, _, _) = game.ram_and_owned_snapshots();
            restore_snapshot(
                &before,
                &mut *snapshot_snes,
                &mut *snapshot_cpu,
                ram,
                backend,
            )
        };
        before.audio.restore(game.g_snes);
        game.restore_native_player_snapshot(crate::game_state::PlayerSnapshotSource::Local(
            &before.player,
        ));
        *game.g_cpu = cpu_before;
        (*game.g_snes).cpuCyclesLeft = cycles_before;
        (*game.g_snes).cpuMemOps = memory_ops_before;
        queue_before.restore(game);
        game.g_calling_asm_from_c = calling_before;
        game.g_calling_asm_from_c_ret = return_before;
        game.g_fail = fail_before;
        result.unwrap_or_else(|_| Err("player transition oracle panicked".into()))
    }
}
