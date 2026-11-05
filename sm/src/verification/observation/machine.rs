//! The machine streams of a single-side run: its frame result captured as
//! the lockstep captures each side's, and one digest per stream of what
//! `verify_snapshots_eq` and the APU queue check compare.

use crate::cpu_infra::Snapshot;
use crate::game_state::Game;

// LEGACY(verifier): snesrev's bug-fix counter, the oracle's patch selector in RAM the
// original never uses. Only an oracle frame writes it, so lockstep's shared RAM
// hides it; free run masks it too (docs/LEGACY.md).
const ORACLE_RAM: core::ops::Range<usize> = 0x00c2..0x00c4;

/// A single-side run's frame result, captured as the lockstep captures each side's.
pub(super) fn capture_result(game: &mut Game) -> &Snapshot {
    let snes = game.g_snes;
    let cpu = game.g_cpu;
    let (backend, ram, mine, _, _, owner) = game.ram_and_owned_snapshots();
    // SAFETY: a single-side run calls this between frames, while the live
    // backend owns the machine these pointers name, as the lockstep's capture.
    unsafe {
        crate::cpu_infra::make_snapshot_with_player_owner(mine, &*snes, &*cpu, owner, ram, backend)
    };
    &game.g_snapshot_mine
}

/// The captured result's machine streams (`sm_tools::trace::MACHINE`): what
/// `verify_snapshots_eq` and the APU queue check compare, with the normalized
/// RAM masked.
pub(super) fn result_digests(game: &mut Game) -> [u64; sm_data::digest::MACHINE.len()] {
    use core::hash::{Hash, Hasher};
    use sm_data::digest::Digest;
    let pending = game.apu_queue.pending;
    let result = &mut *game.g_snapshot_mine;
    diagnostic::tick(result);
    for range in crate::cpu_infra::LOCK_SESSION_MASK
        .into_iter()
        .chain([ORACLE_RAM])
    {
        result.ram[range].fill(0);
    }
    // LEGACY(verifier): the lock's `wram` stream zeroes the frozen `LOCK_WRAM_MASK`, a
    // superset of `NORMALIZED_RAM`. Its ranges are zeroed for the digest and then
    // restored, so the Session streams' decode keeps `LOCK_SESSION_MASK`.
    let mask = crate::cpu_infra::LOCK_WRAM_MASK;
    let saved: Vec<u8> = mask
        .iter()
        .flat_map(|range| result.ram[range.clone()].iter().copied())
        .collect();
    for range in mask.clone() {
        result.ram[range].fill(0);
    }
    let wram = Digest::of(&result.ram);
    let mut rest = saved.as_slice();
    for range in mask {
        let (bytes, tail) = rest.split_at(range.len());
        result.ram[range].copy_from_slice(bytes);
        rest = tail;
    }
    let ppu = &result.ppu;
    let hash = |value: &dyn Fn(&mut Digest)| {
        let mut digest = Digest::default();
        value(&mut digest);
        digest.finish()
    };
    let registers = hash(&|d| {
        (ppu.rangeOver, ppu.timeOver, ppu.evenFrame).hash(d);
        (ppu.brightness, ppu.forcedBlank).hash(d);
        let geometry = (
            ppu.interlace,
            ppu.objInterlace,
            ppu.overscan,
            ppu.pseudoHires,
        );
        (geometry, ppu.frameInterlace, ppu.frameOverscan).hash(d);
        (
            ppu.clipMode,
            ppu.preventMathMode,
            ppu.addSubscreen,
            ppu.directColor,
        )
            .hash(d);
        (ppu.subtractColor, ppu.halfColor, ppu.mathEnabled).hash(d);
        (ppu.fixedColorR, ppu.fixedColorG, ppu.fixedColorB).hash(d);
        (
            ppu.window1left,
            ppu.window1right,
            ppu.window2left,
            ppu.window2right,
        )
            .hash(d);
        ppu.windowsel.hash(d);
        for w in &ppu.windowLayer {
            let windows = (w.window1enabled, w.window2enabled, w.window1inversed);
            (windows, w.window2inversed, w.maskLogic).hash(d);
        }
        (ppu.screenEnabled, ppu.screenWindowed).hash(d);
        for l in &ppu.layer {
            let main = (l.mainScreenEnabled, l.mainScreenWindowed);
            (main, l.subScreenEnabled, l.subScreenWindowed).hash(d);
        }
        for bg in &ppu.bgLayer {
            (bg.tilemapWider, bg.tilemapHigher, bg.tilemapAdr, bg.tileAdr).hash(d);
            (bg.hScroll, bg.vScroll, bg.bigTiles, bg.mosaicEnabled).hash(d);
        }
        (
            ppu.mode,
            ppu.bg3priority,
            ppu.mosaicSize,
            ppu.mosaicStartLine,
        )
            .hash(d);
        (ppu.m7largeField, ppu.m7charFill, ppu.m7xFlip, ppu.m7yFlip).hash(d);
        (ppu.m7extBg, ppu.m7matrix).hash(d);
        (ppu.hCount, ppu.vCount, ppu.hCountSecond, ppu.vCountSecond).hash(d);
        (ppu.countersLatched, ppu.ppu1openBus, ppu.ppu2openBus).hash(d);
        (ppu.cgramPointer, ppu.cgramSecondWrite, ppu.cgramBuffer).hash(d);
        (ppu.vramPointer, ppu.vramIncrement, ppu.vramIncrementOnHigh).hash(d);
        (ppu.vramRemapMode, ppu.vramReadBuffer).hash(d);
        (ppu.scrollPrev, ppu.scrollPrev2, ppu.m7prev).hash(d);
        (
            ppu.oamAdr,
            ppu.oamAdrWritten,
            ppu.oamInHigh,
            ppu.oamInHighWritten,
        )
            .hash(d);
        (ppu.oamSecondWrite, ppu.oamBuffer, ppu.objPriority).hash(d);
        (ppu.objTileAdr1, ppu.objTileAdr2, ppu.objSize).hash(d);
        ppu.objPixelBufferXX[..19].hash(d);
    });
    // SAFETY: both headless hosts register an initialized 256x240 BGRA buffer
    // with pitch 1024 for the frame's PPU, as verify_snapshots_eq reads it.
    let video = (!ppu.renderBuffer.is_null() && ppu.renderPitch == 1024).then(|| unsafe {
        Digest::of(core::slice::from_raw_parts(ppu.renderBuffer, 256 * 240 * 4))
    });
    [
        wram,
        Digest::of(&result.sram),
        hash(&|d| ppu.vram.hash(d)),
        hash(&|d| ppu.cgram.hash(d)),
        hash(&|d| (ppu.oam, ppu.highOam).hash(d)),
        registers,
        video.unwrap_or_default(),
        Digest::of(&pending),
    ]
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_lock_session_mask_holds_every_normalized_range() {
        let mask = crate::cpu_infra::LOCK_SESSION_MASK;
        for (_, range) in crate::cpu_infra::NORMALIZED_RAM {
            for address in range {
                assert!(
                    mask.iter().any(|frozen| frozen.contains(&address)),
                    "{address:#x}"
                );
            }
        }
    }

    #[test]
    fn the_lock_wram_mask_holds_every_normalized_range() {
        let mask = crate::cpu_infra::LOCK_WRAM_MASK;
        for (_, range) in crate::cpu_infra::NORMALIZED_RAM {
            for address in range {
                assert!(
                    mask.iter().any(|frozen| frozen.contains(&address)),
                    "{address:#x}"
                );
            }
        }
    }
}

/// A run's per-Tick digests and one Tick's memory, for locating where two hosts
/// part: `SM_DIGEST_LOG` names a text file, `SM_DUMP_TICK` and `SM_DUMP_OUT` a Tick.
mod diagnostic {
    use crate::cpu_infra::Snapshot;
    use std::cell::Cell;
    use std::io::Write;

    thread_local! {
        static TICK: Cell<u64> = const { Cell::new(0) };
    }

    pub(super) fn tick(result: &Snapshot) {
        let log = std::env::var_os("SM_DIGEST_LOG");
        let dump = std::env::var("SM_DUMP_TICK").ok();
        if log.is_none() && dump.is_none() {
            return;
        }
        let tick = TICK.replace(TICK.get() + 1);
        let ppu = &result.ppu;
        let words = |words: &[u16]| -> Vec<u8> {
            words.iter().flat_map(|word| word.to_le_bytes()).collect()
        };
        let (vram, cgram, oam) = (words(&ppu.vram), words(&ppu.cgram), words(&ppu.oam));
        if let Some(path) = log {
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .expect("digest log");
            use sm_data::digest::Digest;
            writeln!(
                file,
                "{tick} {:016x} {:016x} {:016x} {:016x} {:016x}",
                Digest::of(&result.ram),
                Digest::of(&vram),
                Digest::of(&cgram),
                Digest::of(&oam),
                Digest::of(&ppu.highOam)
            )
            .expect("digest log");
        }
        if dump.as_deref().and_then(|at| at.parse().ok()) == Some(tick) {
            let mut bytes = result.ram.to_vec();
            bytes.extend_from_slice(&vram);
            bytes.extend_from_slice(&cgram);
            bytes.extend_from_slice(&oam);
            std::fs::write(std::env::var("SM_DUMP_OUT").expect("SM_DUMP_OUT"), bytes)
                .expect("dump");
        }
    }
}
