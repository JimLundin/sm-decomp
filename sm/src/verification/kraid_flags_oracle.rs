//! Original CPU Kraid read and complete palette/music/native-store caller.
use super::Game;
use crate::native_rom::NativeRomPointer;
use crate::{kraid_flags_case as case, snes::cpu::cpu_get_flags};
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
        let loaded = game.loaded_rom.as_ref().ok_or("missing Kraid loaded ROM")?;
        let native = game
            .capture_native_rom_binding()
            .ok_or("missing Kraid native ROM")?;
        if loaded.as_mut_ptr() != native.as_mut_ptr()
            || loaded.as_mut_ptr() != (*(*game.g_snes).cart).rom.as_mut_ptr()
        {
            return Err("Kraid requires shared original cartridge binding".into());
        }
        for (address, length) in [
            (0xa7a92c, 0x17),
            (0xa7c815, 0x2e),
            (0x82d96c, 0x19),
            (0x808fc1, 0x36),
        ] {
            if !matches_original(game, original, address, length) {
                return Err(format!("Kraid original window mismatch {address:06x}"));
            }
        }
        Ok(())
    }
}
unsafe fn header(game: &Game) -> [u8; 16] {
    unsafe {
        let c = &*game.g_cpu;
        let mut out = [0; 16];
        out[..4].copy_from_slice(&(u32::from(c.k) << 16 | u32::from(c.pc)).to_le_bytes());
        out[4] = cpu_get_flags(c);
        for (i, v) in [c.a, c.x, c.y, c.sp, c.dp].into_iter().enumerate() {
            out[5 + i * 2..7 + i * 2].copy_from_slice(&v.to_le_bytes());
        }
        out[15] = c.db;
        out
    }
}
fn add(set: &mut BTreeSet<usize>, at: usize, n: usize) {
    set.extend(at..at + n);
}
fn writes(pc: u32, sp: u16, x: u16, abi: &mut BTreeSet<usize>, ram: &mut BTreeSet<usize>) {
    let push: u16 = match pc {
        0xa7a92c | 0x808fc4 | 0x808fc5 | 0x808fce => 2,
        0xa7c815 | 0xa7c81e => 3,
        0x808fc1 => 1,
        _ => 0,
    };
    for i in 0..push {
        abi.insert(usize::from(sp.wrapping_sub(i)));
    }
    match pc {
        0x82d971 => add(ram, 0xc402, 2),
        0x82d97f => add(ram, 0xc400, 2),
        0x808fe1 => add(ram, 0x619 + usize::from(x), 2),
        0x808fe7 => add(ram, 0x629 + usize::from(x), 2),
        0x808ff0 => add(ram, 0x639, 2),
        0xa7c831 => add(ram, 0xd828 + usize::from(x), 2),
        0xa7c838 | 0xa7c83f => add(ram, 0xfa8, 2),
        _ => (),
    }
}
pub unsafe fn capture(id: u32, original: &[u8], game: &mut Game) -> Result<Vec<u8>, String> {
    unsafe {
        super::flags_save_oracle::with_initialized_state(game, |game| {
            guard(game, original)?;
            let fade = case::seed(id, game)?;
            let (_, area, bit) = case::parameters(id)?;
            let initial = (0..game.ram().len())
                .map(|at| game.ram().read_byte(at))
                .collect::<Vec<_>>();
            let saved_sram = game.g_sram.bytes(0..8192);
            let c = &mut *game.g_cpu;
            c.a = 0xa55a;
            c.x = 0x1357;
            c.y = 0x2468;
            c.sp = 0x1fff;
            c.dp = 0;
            c.db = 0x7e;
            c.k = 0xa7;
            c.pc = if fade { 0xc815 } else { 0xa92c };
            c.c = false;
            c.z = false;
            c.v = false;
            c.n = false;
            c.i = true;
            c.d = false;
            c.xf = false;
            c.mf = false;
            c.e = false;
            c.irqWanted = false;
            c.nmiWanted = false;
            c.waiting = false;
            c.stopped = false;
            c.spBreakpoint = 0;
            game.g_calling_asm_from_c = false;
            let final_pc = if fade {
                if bit == 0 { 0xa7c83b } else { 0xa7c842 }
            } else if bit == 0 {
                0xa7a942
            } else {
                0xa7a93d
            };
            let pcs = if fade {
                vec![0xa7c815, 0xa7c819, 0xa7c822, final_pc]
            } else {
                vec![0xa7a92c, final_pc]
            };
            let mut output = b"NKOR1".to_vec();
            case::record(&mut output, 0, &id.to_le_bytes());
            let mut stage = 0usize;
            let mut abi = BTreeSet::new();
            let mut ram = BTreeSet::new();
            let mut palette_calls = 0;
            let mut music_calls = 0;
            for _ in 0..2000 {
                let c = &*game.g_cpu;
                let pc = u32::from(c.k) << 16 | u32::from(c.pc);
                if pc == pcs[stage] {
                    for (at, (&before, after)) in initial
                        .iter()
                        .zip((0..game.ram().len()).map(|at| game.ram().read_byte(at)))
                        .enumerate()
                    {
                        if before != after && !abi.contains(&at) && !ram.contains(&at) {
                            return Err(format!(
                                "unaccounted Kraid write stage{stage} address{at:05x}"
                            ));
                        }
                    }
                    if game.g_sram.bytes(0..8192) != saved_sram {
                        return Err("Kraid unexpectedly modified SRAM".into());
                    }
                    if (0x5e7..0x5e9)
                        .map(|at| game.ram().read_byte(at))
                        .collect::<Vec<_>>()
                        .as_slice()
                        != &initial[0x5e7..0x5e9]
                    {
                        return Err("raw Kraid native access changed scratch".into());
                    }
                    let result = if !fade && stage == 1 { c.a } else { 0xffff };
                    case::state(&mut output, stage as u8, result, header(game), game);
                    case::mask(&mut output, stage as u8, &abi);
                    stage += 1;
                    if stage == pcs.len() {
                        if c.sp != 0x1fff || c.y != 0x2468 || c.dp != 0 || c.db != 0x7e {
                            return Err("Kraid caller stack/base register mismatch".into());
                        }
                        if !fade && (c.a != bit || c.x != 0x1357) {
                            return Err("Kraid actual return mismatch".into());
                        }
                        if fade
                            && (palette_calls != 1
                                || music_calls != 1
                                || c.x != area
                                || case::word(game, 0xc400) != 0
                                || case::word(game, 0xc402) != 12
                                || case::word(game, 0x619) != 3
                                || case::word(game, 0x629) != 8
                                || case::word(game, 0x639) != 2)
                        {
                            return Err(
                                "Kraid complete actual palette/music effects missing".into()
                            );
                        }
                        return Ok(output);
                    }
                }
                if !matches_original(game, original, pc, 4) {
                    return Err(format!("executed Kraid code differs {pc:06x}"));
                }
                if pc == 0x82d96c {
                    palette_calls += 1;
                }
                if pc == 0x808fc1 {
                    music_calls += 1;
                }
                writes(pc, c.sp, c.x, &mut abi, &mut ram);
                game.execution().step_cpu();
                if game.dma_busy() {
                    return Err("unexpected DMA in Kraid capsule".into());
                }
            }
            Err("Kraid original opcode budget exhausted".into())
        })
    }
}
