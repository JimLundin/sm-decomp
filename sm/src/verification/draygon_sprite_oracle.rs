//! Actual 65816 Draygon intro-dance observations; never invokes old Rust Func_46.
//! Original ROM and captured RAM remain local, outside the repository.
use super::{Game, MakeSnapshot, RestoreSnapshot, Snapshot};
use crate::initial_state::InitialState;
use crate::snes::cpu::{cpu_getFlags, cpu_runOpcode};

const WINDOWS: &[(u32, u32)] = &[
    (0xa5a13e, 0xa5a19f), // Complete dance routine, including delete branch.
    (0xa0afea, 0xa0affd), // Actual sign-extension helper.
    (0xa5a19f, 0xa5a1a7), // Four used latency words.
    (0xa5ce07, 0xa60000), // Data window used by these non-bank-wrapping cases.
];
const HANDLES: [u16; 4] = [62, 60, 58, 56];

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
                .all(|(i, byte)| image.read(start + i) == *byte)
    })
}
fn word(bytes: &[u8], at: usize) -> Option<u16> {
    let bytes = bytes.get(at..at.checked_add(2)?)?;
    Some(u16::from_le_bytes([bytes[0], bytes[1]]))
}
fn put(game: &mut Game, at: usize, value: u16) {
    game.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
#[derive(Clone, Copy)]
enum Action {
    Skip,
    Delete,
    Move(u8, u8),
}
fn actions(original: &[u8], cursor: u16) -> Option<[Action; 4]> {
    let mut actions = [Action::Skip; 4];
    for (i, handle) in HANDLES.into_iter().enumerate() {
        let delay = word(original, offset(0xa5a19f) + usize::from(handle - 56))?;
        let index = cursor.wrapping_add(delay);
        if index & 0x8000 != 0 {
            continue;
        }
        // Both 16-bit indexed data reads remain inside bank A5. This avoids
        // conflating SNES bank wrap with the old Rust wide native pointer.
        if index > 0x31f6 {
            return None;
        }
        let at = offset(0xa5ce07) + usize::from(index);
        let pair = word(original, at)?;
        actions[i] = if pair == 0x8080 {
            Action::Delete
        } else {
            Action::Move(pair as u8, (pair >> 8) as u8)
        };
    }
    Some(actions)
}
fn choose_cursor(id: u32, original: &[u8]) -> Result<u16, String> {
    if id > 4 {
        return Err("unknown Draygon sprite oracle case".into());
    }
    if id == 0 || id == 4 {
        let cursor = if id == 0 { 0 } else { 0xffff };
        let selected = actions(original, cursor).ok_or("invalid latency table")?;
        if !selected.iter().all(|a| matches!(a, Action::Skip)) {
            return Err("expected all-negative latency setup".into());
        }
        return Ok(cursor);
    }
    // Select from actual local data, not an embedded cartridge fixture. The
    // selected cursor is in the entry RAM record, so candidate capture reuses it.
    for cursor in (0..=u16::MAX).step_by(4) {
        let Some(selected) = actions(original, cursor) else {
            continue;
        };
        let moves = selected
            .iter()
            .filter(|a| matches!(a, Action::Move(..)))
            .count();
        let skips = selected
            .iter()
            .filter(|a| matches!(a, Action::Skip))
            .count();
        let deletes = selected
            .iter()
            .filter(|a| matches!(a, Action::Delete))
            .count();
        let negative = selected
            .iter()
            .any(|a| matches!(a, Action::Move(x, y) if x & 0x80 != 0 || y & 0x80 != 0));
        let positive = selected.iter().any(
            |a| matches!(a, Action::Move(x, y) if (1..=0x7f).contains(x) || (1..=0x7f).contains(y)),
        );
        let selected_case = match id {
            1 => moves == 4 && negative && positive,
            2 => deletes != 0,
            3 => moves != 0 && skips != 0,
            _ => false,
        };
        if selected_case {
            return Ok(cursor);
        }
    }
    Err(format!("local original has no suitable Draygon case {id}"))
}
fn record(output: &mut Vec<u8>, tag: u8, payload: &[u8]) {
    output.push(tag);
    output.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    output.extend_from_slice(payload);
}
unsafe fn observe(output: &mut Vec<u8>, tag: u8, game: &Game, full_ram: bool) {
    unsafe {
        let cpu = &*game.g_cpu;
        let mut payload = (u32::from(cpu.k) << 16 | u32::from(cpu.pc))
            .to_le_bytes()
            .to_vec();
        payload.push(cpu_getFlags(cpu));
        for value in [cpu.a, cpu.x, cpu.y, cpu.sp, cpu.dp] {
            payload.extend_from_slice(&value.to_le_bytes());
        }
        payload.push(cpu.db);
        if full_ram {
            payload.extend_from_slice(&game.g_ram);
        }
        record(output, tag, &payload);
    }
}
/// Initialized exclusive emulator/audio required. IDs 0..4 are five focused
/// actual-CPU cases; Result::Err never means verification passed.
pub unsafe fn capture(id: u32, original: &[u8], game: &mut Game) -> Result<Vec<u8>, String> {
    unsafe {
        if game.g_cpu.is_null() || game.g_snes.is_null() || game.g_spc_player.is_null() {
            return Err("Draygon capture requires initialized emulator/audio".into());
        }
        for &(start, end) in WINDOWS {
            if !matches_original(game, original, start, (end - start) as usize) {
                return Err(format!(
                    "patched/mismatched Draygon oracle window {start:06x}"
                ));
            }
        }
        let cursor = choose_cursor(id, original)?;
        let mut before = Box::new(Snapshot::INITIAL);
        MakeSnapshot(&mut before, &*game.g_snes, &*game.g_cpu, game.g_spc_player);
        let cpu_before = *game.g_cpu;
        let cycles_before = (*game.g_snes).cpuCyclesLeft;
        let memory_before = (*game.g_snes).cpuMemOps;
        let calling_before = game.g_calling_asm_from_c;
        let return_before = game.g_calling_asm_from_c_ret;
        let queue_before = crate::audio_snapshot::ApuQueueSnapshot::capture(game);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            game.g_ram.fill(0);
            game.g_ram[0x1ffff] = 1;
            put(game, 0x880c, cursor);
            put(game, 0xe32, 0xbeef);
            for (i, handle) in HANDLES.into_iter().enumerate() {
                let slot_offset = usize::from(handle);
                // Include an inactive slot: this controller updates coordinates
                // independently of the pool's program/disable state.
                put(
                    game,
                    0xef78 + slot_offset,
                    if i == 3 { 0 } else { 0x9000 + handle },
                );
                put(game, 0xf0f8 + slot_offset, [0, 0xffff, 0x7fff, 0x8000][i]);
                put(game, 0xf1f8 + slot_offset, [0xffff, 0, 0x8000, 0x7fff][i]);
                for base in [0xeff8, 0xf078, 0xf178, 0xf278, 0xf2f8] {
                    put(game, base + slot_offset, 0x5a01 + handle);
                }
            }
            let cpu = &mut *game.g_cpu;
            cpu.a = 0;
            cpu.x = 0;
            cpu.y = 0x2468;
            cpu.sp = 0x1fff;
            cpu.dp = 0;
            cpu.db = 0xa5;
            cpu.k = 0xa5;
            cpu.pc = 0xa13e;
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
            let mut output = b"DSOR1".to_vec();
            record(&mut output, 0, &id.to_le_bytes());
            let mut entered = 0usize;
            let mut completed = 0usize;
            for _ in 0..10_000 {
                let pc = u32::from((*game.g_cpu).k) << 16 | u32::from((*game.g_cpu).pc);
                match pc {
                    0xa5a13e => observe(&mut output, 1, game, true),
                    0xa5a142 => {
                        if entered >= 4
                            || (*game.g_cpu).x != HANDLES[entered]
                            || entered != completed
                        {
                            return Err("wrong Draygon loop order".into());
                        }
                        entered += 1;
                        observe(&mut output, 2, game, false);
                    }
                    0xa5a153 | 0xa5a159 | 0xa5a15b | 0xa5a196 | 0xa0afea => {
                        observe(&mut output, 3, game, false);
                    }
                    0xa5a181 => {
                        if completed >= 4
                            || (*game.g_cpu).x != HANDLES[completed]
                            || entered != completed + 1
                        {
                            return Err("wrong Draygon completion order".into());
                        }
                        completed += 1;
                        observe(&mut output, 4, game, true);
                    }
                    0xa5a195 => {
                        if entered != 4
                            || completed != 4
                            || (*game.g_cpu).y != 0x2468
                            || (*game.g_cpu).sp != 0x1fff
                        {
                            return Err("incomplete Draygon oracle execution".into());
                        }
                        observe(&mut output, 5, game, true);
                        return Ok(output);
                    }
                    _ => {}
                }
                if !matches_original(game, original, pc, 4) {
                    return Err(format!("executed Draygon code differs {pc:06x}"));
                }
                cpu_runOpcode(game.g_cpu, game);
                if (*(*game.g_snes).dma).dmaBusy {
                    return Err("unexpected DMA in Draygon capture".into());
                }
            }
            Err("Draygon instruction budget exhausted".into())
        }));
        RestoreSnapshot(&before, &mut *game.g_snes, &mut *game.g_cpu);
        before.audio.restore(game.g_snes);
        before.player.restore(game.g_spc_player);
        *game.g_cpu = cpu_before;
        (*game.g_snes).cpuCyclesLeft = cycles_before;
        (*game.g_snes).cpuMemOps = memory_before;
        queue_before.restore(game);
        game.g_calling_asm_from_c = calling_before;
        game.g_calling_asm_from_c_ret = return_before;
        result.unwrap_or_else(|_| Err("Draygon oracle capture panicked".into()))
    }
}
