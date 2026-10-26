//! Original 65816 sprite lifecycle and real OAM output. Component oracle, not a movie.
use super::{Game, MakeSnapshot, RestoreSnapshot, Snapshot};
use crate::initial_state::InitialState;
use crate::snes::cpu::{cpu_getFlags, cpu_runOpcode};
use crate::sprite_pool_case::{self as case, Operation};
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
// SnesInit installs a one-byte FixBugHook at the OAM emitter's original entry.
// Borrow that exact allocation and restore only its original-file byte for this
// capture. Snapshot restoration does not cover cartridge cells.
struct OriginalOamEntry {
    image: crate::rom::RomImage,
    at: usize,
    initialized: u8,
}
impl Drop for OriginalOamEntry {
    fn drop(&mut self) {
        self.image.write(self.at, self.initialized);
    }
}
unsafe fn original_oam_entry(game: &Game, original: &[u8]) -> Result<OriginalOamEntry, String> {
    unsafe {
        let image = game
            .loaded_rom
            .as_ref()
            .ok_or("sprite oracle missing loaded image")?
            .clone();
        let native = game
            .capture_native_rom_binding()
            .ok_or("sprite oracle missing native image")?;
        let cart = (*game.g_snes).cart;
        if cart.is_null()
            || image.as_mut_ptr() != (*cart).rom.as_mut_ptr()
            || image.as_mut_ptr() != native.as_mut_ptr()
        {
            return Err("sprite oracle requires shared initialized cartridge bindings".into());
        }
        let at = offset(0x818ab8);
        let original_byte = *original.get(at).ok_or("short original OAM entry")?;
        if at >= image.len() {
            return Err("short initialized OAM entry".into());
        }
        let initialized = image.read(at);
        // This is the third FixBugHook registration in PatchBugs(mode=1).
        // Unknown differences are never normalized away.
        if initialized != original_byte
            && !(initialized == 0
                && game.hookcnt >= 3
                && game.hook_fixbug_orgbyte[2] == original_byte)
        {
            return Err("unexpected initialized OAM entry; not the saved startup hook".into());
        }
        let restore = OriginalOamEntry {
            image,
            at,
            initialized,
        };
        restore.image.write(at, original_byte);
        Ok(restore)
    }
}
fn original_word(original: &[u8], address: u32) -> Result<u16, String> {
    let at = offset(address);
    let pair = original.get(at..at + 2).ok_or("short original content")?;
    Ok(u16::from_le_bytes([pair[0], pair[1]]))
}
fn guard(game: &Game, original: &[u8]) -> Result<(), String> {
    // Exact routine and OAM table windows; no patched/injected executable bytes.
    for (address, length) in [
        (0xb4bc26, 0x171),
        (0x818ab8, 0x69),
        (0x81839f, 0x400),
        (0xb4bdb4, 2),
    ] {
        if !matches_original(game, original, address, length) {
            return Err(format!("sprite oracle original mismatch {address:06x}"));
        }
    }
    let program = original_word(original, 0xb4bdb4)?;
    if program < 0x8000 || program > 0xffe5 {
        return Err("invalid original dud program".into());
    }
    if !matches_original(game, original, 0xb40000 | u32::from(program), 26) {
        return Err("sprite oracle program mismatch".into());
    }
    for frame in 0..6u32 {
        let cursor = 0xb40000 + u32::from(program) + frame * 4;
        if original_word(original, cursor)? != 4 {
            return Err("dud fixture requires six original four-tick frames".into());
        }
        let map = original_word(original, cursor + 2)?;
        if map < 0x8000 {
            return Err("dud original map outside high window".into());
        }
        let entries = original_word(original, 0xb40000 | u32::from(map))?;
        if entries == 0 || entries > 16 {
            return Err("dud original map must have bounded nonempty OAM".into());
        }
        let length = 2 + usize::from(entries) * 5;
        if usize::from(map) + length > 0x10000
            || !matches_original(game, original, 0xb40000 | u32::from(map), length)
        {
            return Err("sprite oracle map mismatch".into());
        }
    }
    if original_word(original, 0xb40000 + u32::from(program) + 24)? != 0xbd07 {
        return Err("dud original program must end in delete".into());
    }
    Ok(())
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
// Only executed ABI stores and pushes enter the exclusion set. No blanket DP/stack mask.
fn cpu_only_writes(pc: u32, sp: u16, out: &mut BTreeSet<usize>) {
    let direct = match pc {
        0xb4bc7b | 0xb4bce3 | 0xb4bd63 => Some(0x12),
        0xb4bd4e => Some(0x14),
        0xb4bd73 => Some(3),
        0xb4bd7c => Some(0),
        0x818ac0 | 0x818b18 => Some(0x18),
        0x818ad7 | 0x818aea => Some(0x16),
        _ => None,
    };
    if let Some(address) = direct {
        out.insert(address);
        out.insert(address + 1);
    }
    let bytes = match pc {
        0xb4bc26 | 0xb4bc27 | 0xb4bc2a | 0xb4bc70 | 0xb4bc82 | 0xb4bc83 | 0xb4bc86 | 0xb4bce5
        | 0xb4bd32 | 0xb4bd33 | 0xb4bd36 | 0xb4bd7e | 0x818ab8 => 2,
        0xb4bc28 | 0xb4bc29 | 0xb4bc84 | 0xb4bc85 | 0xb4bd34 | 0xb4bd35 => 1,
        0xb4bd89 => 3,
        _ => 0,
    };
    for i in 0..bytes {
        out.insert(usize::from(sp.wrapping_sub(i)));
    }
}
fn gameplay_write(operation: Operation, address: usize) -> bool {
    match operation {
        Operation::Create => (0xef78..0xf378).contains(&address),
        Operation::Update => {
            (0xef78..0xf378).contains(&address) || (0x1844..0x1846).contains(&address)
        }
        Operation::Draw => (0x370..0x592).contains(&address),
    }
}
unsafe fn run_phase(
    index: u16,
    operation: Operation,
    tick: u8,
    original: &[u8],
    game: &mut Game,
    output: &mut Vec<u8>,
) -> Result<(), String> {
    unsafe {
        let (entry, exit): (u16, u16) = match operation {
            Operation::Create => (0xbc26, 0xbc81),
            Operation::Update => (0xbc82, 0xbcef),
            Operation::Draw => (0xbd32, 0xbd96),
        };
        case::prepare(operation, game);
        let before = game.g_ram.to_vec();
        let cpu = &mut *game.g_cpu;
        cpu.a = 0xa55a;
        cpu.x = 0x1357;
        cpu.y = 0x2468;
        cpu.sp = 0x1fff;
        cpu.dp = 0;
        cpu.db = 0x7e;
        cpu.k = 0xb4;
        cpu.pc = entry;
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
        case::state(
            output,
            1,
            index,
            operation,
            tick,
            0xffff,
            cpu_header(game),
            game,
        );
        let mut excluded = BTreeSet::new();
        let mut emitter_calls = 0;
        for _ in 0..10_000 {
            let cpu = &*game.g_cpu;
            let pc = u32::from(cpu.k) << 16 | u32::from(cpu.pc);
            if pc == 0xb40000 | u32::from(exit) {
                if cpu.x != 0x1357
                    || cpu.y != 0x2468
                    || cpu.sp != 0x1fff
                    || cpu.dp != 0
                    || cpu.db != 0x7e
                    || cpu_getFlags(cpu) != flags
                {
                    return Err(format!("sprite phase{index} did not restore caller"));
                }
                for (address, (&prior, &after)) in before.iter().zip(game.g_ram.iter()).enumerate()
                {
                    if prior != after
                        && !excluded.contains(&address)
                        && !gameplay_write(operation, address)
                    {
                        return Err(format!(
                            "unaccounted sprite CPU write phase{index} address{address:05x}"
                        ));
                    }
                }
                let result = if operation == Operation::Create {
                    case::word(game, 0x12)
                } else {
                    0xffff
                };
                if operation == Operation::Create && result != 62 {
                    return Err("original allocation did not select slot31".into());
                }
                if operation == Operation::Draw {
                    let expect_live = tick < 24;
                    if (case::word(game, 0x590) != 0) != expect_live
                        || emitter_calls != usize::from(expect_live)
                    {
                        return Err(format!("missing/unexpected original OAM phase{index}"));
                    }
                }
                if operation == Operation::Update && (case::word(game, 0xefb6) != 0) != (tick < 24)
                {
                    return Err(format!("wrong original sprite retirement tick{tick}"));
                }
                case::state(
                    output,
                    2,
                    index,
                    operation,
                    tick,
                    result,
                    cpu_header(game),
                    game,
                );
                case::mask(output, index, &excluded);
                return Ok(());
            }
            if !matches_original(game, original, pc, 4) {
                return Err(format!("executed sprite code mismatch {pc:06x}"));
            }
            if pc == 0x818ab8 {
                emitter_calls += 1;
            }
            cpu_only_writes(pc, (*game.g_cpu).sp, &mut excluded);
            cpu_runOpcode(game.g_cpu, game);
            if (*(*game.g_snes).dma).dmaBusy {
                return Err("unexpected DMA in sprite component oracle".into());
            }
        }
        Err(format!("sprite phase{index} opcode budget exhausted"))
    }
}
/// Initialized exclusive emulator/audio. Exactly one component case; no movie fidelity claim.
pub unsafe fn capture(id: u32, original: &[u8], game: &mut Game) -> Result<Vec<u8>, String> {
    unsafe {
        if id != 0 || game.g_cpu.is_null() || game.g_snes.is_null() || game.g_spc_player.is_null() {
            return Err("invalid sprite oracle request".into());
        }
        let mut before = Box::new(Snapshot::INITIAL);
        MakeSnapshot(&mut before, &*game.g_snes, &*game.g_cpu, game.g_spc_player);
        let cpu_before = *game.g_cpu;
        let cycles_before = (*game.g_snes).cpuCyclesLeft;
        let memory_before = (*game.g_snes).cpuMemOps;
        let calling_before = game.g_calling_asm_from_c;
        let return_before = game.g_calling_asm_from_c_ret;
        let queue_before = crate::audio_snapshot::ApuQueueSnapshot::capture(game);
        let hooks_before = (
            game.hookmode,
            game.hookcnt,
            game.hookadr,
            game.hook_fixbug_orgbyte,
        );
        let fail_before = game.g_fail;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _original_entry = original_oam_entry(game, original)?;
            guard(game, original)?;
            case::seed(game);
            let mut output = b"PSOR1".to_vec();
            case::record(&mut output, 0, &id.to_le_bytes());
            for index in 0..case::PHASES {
                let (operation, tick) = case::phase(index).unwrap();
                run_phase(index, operation, tick, original, game, &mut output)?;
            }
            Ok(output)
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
        (
            game.hookmode,
            game.hookcnt,
            game.hookadr,
            game.hook_fixbug_orgbyte,
        ) = hooks_before;
        game.g_fail = fail_before;
        result.unwrap_or_else(|_| Err("sprite component oracle capture panicked".into()))
    }
}
