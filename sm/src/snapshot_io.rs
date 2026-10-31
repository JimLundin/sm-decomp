//! The hardware records of the save-state format, over sm_runtime's transfers.
#![forbid(unsafe_code)]
#[cfg(any(feature = "frontend", sm_verify_dll))]
pub(crate) use sm_runtime::snapshot::state_layout;
pub use sm_runtime::snapshot::{Block, SnapshotIo, State, TransferError};
use sm_runtime::snapshot::{Fields, SnapshotOutput};
use std::collections::TryReserveError;

// Hardware values are serialized field by field. Explicit padding fields retain
// the original byte format without interpreting a Rust allocation as a byte slice.
use crate::types::{
    Apu, ApuRegisters, BgLayer, Cpu, Dma, DmaChannel, DspChannel, DspState, Layer, Ppu, Snes, Spc,
    Timer, WindowLayer,
};

impl SnapshotOutput for crate::recording::RecordingBytes {
    fn append(&mut self, size: usize) -> Result<&mut [u8], TryReserveError> {
        let start = self.size;
        self.resize(start + size)?;
        Ok(&mut self.as_mut_slice()[start..])
    }
}

sm_runtime::state_record!(Cpu, 27, {
    a, x, y, sp, pc, dp, k, db, c, z, v, n, i, d, xf, mf, e, irqWanted, nmiWanted, waiting,
    stopped
} skip {
    memType, cyclesUsed, spBreakpoint, in_emu
});

sm_runtime::state_record!(Spc, 15, {
    a, x, y, sp, pc, c, z, v, n, i, h, p, b, stopped
} skip {
    apu, cyclesUsed
});

sm_runtime::state_record!(Dma, 224, {
    channel, hdmaTimer, snapshot_padding0, dmaTimer, dmaBusy, pad
} skip {
});

sm_runtime::state_record!(DspState, 3024, {
    ram, channel, dirPage, evenCycle, mute, reset, masterVolumeL, masterVolumeR,
    snapshot_padding0, noiseSample, noiseRate, noiseCounter, echoWrites, echoVolumeL,
    echoVolumeR, feedbackVolume, echoBufferAdr, echoDelay, echoRemain, echoBufferIndex,
    firBufferIndex, firValues, snapshot_padding1, firBufferL, firBufferR, sampleBuffer,
    sampleOffset, snapshot_padding2
} skip {
});

sm_runtime::state_record!(Ppu, 67305, {
    vram, vramPointer, vramIncrementOnHigh, snapshot_padding0, vramIncrement, vramRemapMode,
    snapshot_padding1, vramReadBuffer, cgram, cgramPointer, cgramSecondWrite, cgramBuffer,
    snapshot_padding2, oam, highOam, oamAdr, oamAdrWritten, oamInHigh, oamInHighWritten,
    oamSecondWrite, oamBuffer, objPriority, snapshot_padding3, objTileAdr1, objTileAdr2,
    objSize, objPixelBufferXX, objPriorityBufferXX, timeOver, rangeOver, objInterlace, bgLayer,
    scrollPrev, scrollPrev2, mosaicSize, mosaicStartLine, layer, m7matrix, m7prev, m7largeField,
    m7charFill, m7xFlip, m7yFlip, m7extBg, m7startX, m7startY, windowLayer, window1left,
    window1right, window2left, window2right, clipMode, preventMathMode, addSubscreen,
    subtractColor, halfColor, mathEnabled, fixedColorR, fixedColorG, fixedColorB, forcedBlank,
    brightness, mode, bg3priority, evenFrame, pseudoHires, overscan, frameOverscan, interlace,
    frameInterlace, directColor, snapshot_padding4, hCount, vCount, hCountSecond, vCountSecond,
    countersLatched, ppu1openBus, ppu2openBus
} skip {
    snes, pixelbuffer_placeholder, windowsel, extraLeftCur, extraRightCur, extraLeftRight,
    screenEnabled, screenWindowed, mosaicEnabled, lastBrightnessMult, lineHasSprites, bgBuffers,
    objBuffer, renderPitch, renderBuffer, brightnessMult, brightnessMultHalf, mosaicModulo
});

sm_runtime::state_record!(Snes, 54, {
    hPos, vPos, frames, cpuCyclesLeft, cpuMemOps, padpad, apuCatchupCycles, hIrqEnabled,
    vIrqEnabled, nmiEnabled, snapshot_padding0, hTimer, vTimer, inNmi, inIrq, inVblank,
    snapshot_padding1, portAutoReadX, autoJoyRead, snapshot_padding2, autoJoyTimer, ppuLatch,
    multiplyA, multiplyResult, divideA, divideResult, fastMem, openBus
} skip {
    cpu, apu, ppu, snes_ppu, my_ppu, cart, debug_cycles, debug_apu_cycles,
    disableRender, runningWhichVersion, ramAdr, padx
});

sm_runtime::state_record!(DmaChannel, 26, {
    bAdr, snapshot_padding0, aAdr, aBank, snapshot_padding1, size, indBank, snapshot_padding2,
    tableAdr, repCount, unusedByte, dmaActive, hdmaActive, mode, fixed, decrement, indirect,
    fromB, unusedBit, doTransfer, terminated, offIndex, snapshot_padding3
} skip {

});

sm_runtime::state_record!(DspChannel, 86, {
    pitch, pitchCounter, pitchModulation, snapshot_padding0, decodeBuffer, srcn,
    snapshot_padding1, decodeOffset, previousFlags, snapshot_padding2, old, older, useNoise,
    snapshot_padding3, adsrRates, rateCounter, adsrState, snapshot_padding4, sustainLevel,
    useGain, gainMode, directGain, snapshot_padding5, gainValue, gain, keyOn, keyOff, sampleOut,
    volumeL, volumeR, echoEnable, snapshot_padding6
} skip {

});

sm_runtime::state_record!(BgLayer, 12, {
    hScroll, vScroll, tilemapWider, tilemapHigher, tilemapAdr, tileAdr, bigTiles, mosaicEnabled
} skip {

});

sm_runtime::state_record!(Layer, 4, {
    mainScreenEnabled, subScreenEnabled, mainScreenWindowed, subScreenWindowed
} skip {

});

sm_runtime::state_record!(WindowLayer, 5, {
    window1enabled, window2enabled, window1inversed, window2inversed, maskLogic
} skip {

});

impl State for crate::apu_ram::ApuRam {
    const SIZE: usize = 65536;
    fn layout(layout: &mut String) {
        <[u8; Self::SIZE]>::layout(layout);
    }
    fn transfer(&mut self, fields: &mut Fields<'_>) {
        for offset in 0..Self::SIZE {
            let mut byte = self.read(offset as u16);
            byte.transfer(fields);
            self.write(offset as u16, byte);
        }
    }
}
sm_runtime::state_record!(Apu, 65576, { ram, registers } skip { spc, dsp });
sm_runtime::state_record!(ApuRegisters, 40, {
    romReadable, dspAdr, snapshot_padding0, cycles, inPorts, outPorts, timer, cpuCyclesLeft, pad
} skip { history });
sm_runtime::state_record!(Timer, 5, { cycles, divider, target, counter, enabled } skip { });
