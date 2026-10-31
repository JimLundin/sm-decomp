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
pub(super) fn result_digests(game: &mut Game) -> [u64; sm_tools::trace::MACHINE.len()] {
    use core::hash::{Hash, Hasher};
    use sm_tools::trace::Digest;
    let pending = game.apu_queue.pending;
    let result = &mut *game.g_snapshot_mine;
    for range in crate::cpu_infra::NORMALIZED_RAM
        .map(|(_, range)| range)
        .into_iter()
        .chain([ORACLE_RAM])
    {
        result.ram[range].fill(0);
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
        Digest::of(&result.ram),
        Digest::of(&result.sram),
        hash(&|d| ppu.vram.hash(d)),
        hash(&|d| ppu.cgram.hash(d)),
        hash(&|d| (ppu.oam, ppu.highOam).hash(d)),
        registers,
        video.unwrap_or_default(),
        Digest::of(&pending),
    ]
}
