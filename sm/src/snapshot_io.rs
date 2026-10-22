//! Checked, ordered byte transfers for the existing save-state format.
#![forbid(unsafe_code)]
use crate::recording::RecordingBytes;
use std::collections::TryReserveError;

pub enum SnapshotIo<'a> {
    Save(&'a mut RecordingBytes),
    Load(&'a [u8]),
}

pub(crate) enum Block<'a> {
    Save(&'a mut [u8]),
    Load(&'a [u8]),
}

#[derive(Debug)]
pub enum TransferError {
    Truncated,
    Allocation(TryReserveError),
}

impl SnapshotIo<'_> {
    pub fn remaining(&self) -> usize {
        match self {
            Self::Load(bytes) => bytes.len(),
            Self::Save(_) => 0,
        }
    }

    pub fn transfer(&mut self, state: &mut [u8]) -> Result<(), TransferError> {
        self.block(state.len(), |block| match block {
            Block::Save(output) => output.copy_from_slice(state),
            Block::Load(input) => state.copy_from_slice(input),
        })
    }

    pub(crate) fn block(
        &mut self,
        size: usize,
        transfer: impl FnOnce(Block<'_>),
    ) -> Result<(), TransferError> {
        match self {
            Self::Save(output) => {
                let start = output.size;
                output
                    .resize(start + size)
                    .map_err(TransferError::Allocation)?;
                transfer(Block::Save(&mut output.as_mut_slice()[start..]));
            }
            Self::Load(input) => {
                let (head, tail) = input
                    .split_at_checked(size)
                    .ok_or(TransferError::Truncated)?;
                transfer(Block::Load(head));
                *input = tail;
            }
        }
        Ok(())
    }
}

// Hardware values are serialized field by field. Explicit padding fields retain
// the original byte format without interpreting a Rust allocation as a byte slice.
use crate::types::{
    Apu, ApuRegisters, BgLayer, Cpu, Dma, DmaChannel, DspChannel, DspState, Layer, Ppu, Snes, Spc,
    Timer, WindowLayer,
};

pub(crate) struct Fields<'a> {
    block: Block<'a>,
    position: usize,
}

impl Fields<'_> {
    fn bytes(&mut self, bytes: &mut [u8]) {
        let end = self.position + bytes.len();
        match &mut self.block {
            Block::Save(output) => output[self.position..end].copy_from_slice(bytes),
            Block::Load(input) => bytes.copy_from_slice(&input[self.position..end]),
        }
        self.position = end;
    }
}

pub(crate) trait State {
    const SIZE: usize;
    fn transfer(&mut self, fields: &mut Fields<'_>);
}

impl SnapshotIo<'_> {
    pub(crate) fn state<T: State>(&mut self, state: &mut T) -> Result<(), TransferError> {
        self.block(T::SIZE, |block| {
            let mut fields = Fields { block, position: 0 };
            state.transfer(&mut fields);
            assert_eq!(fields.position, T::SIZE);
        })
    }
}

macro_rules! scalar_state {
    ($($ty:ty),* $(,)?) => {$(
        impl State for $ty {
            const SIZE: usize = core::mem::size_of::<Self>();
            fn transfer(&mut self, fields: &mut Fields<'_>) {
                let mut bytes = self.to_ne_bytes();
                fields.bytes(&mut bytes);
                *self = Self::from_ne_bytes(bytes);
            }
        }
    )*};
}
scalar_state!(u8, i8, u16, i16, u32, i32, f64);

impl State for bool {
    const SIZE: usize = 1;
    fn transfer(&mut self, fields: &mut Fields<'_>) {
        let mut byte = u8::from(*self);
        byte.transfer(fields);
        *self = byte != 0;
    }
}

impl<T: State, const N: usize> State for [T; N] {
    const SIZE: usize = T::SIZE * N;
    fn transfer(&mut self, fields: &mut Fields<'_>) {
        for value in self {
            value.transfer(fields);
        }
    }
}

macro_rules! state_record {
    ($ty:ty, $size:literal, { $($field:ident),* } skip { $($skip:ident),* }) => {
        impl State for $ty {
            const SIZE: usize = $size;
            fn transfer(&mut self, fields: &mut Fields<'_>) {
                let Self { $($field,)* $($skip: _,)* } = self;
                $($field.transfer(fields);)*
            }
        }
    };
}

state_record!(Cpu, 27, {
    a, x, y, sp, pc, dp, k, db, c, z, v, n, i, d, xf, mf, e, irqWanted, nmiWanted, waiting,
    stopped
} skip {
    mem, memType, cyclesUsed, spBreakpoint, in_emu
});

state_record!(Spc, 15, {
    a, x, y, sp, pc, c, z, v, n, i, h, p, b, stopped
} skip {
    apu, cyclesUsed
});

state_record!(Dma, 224, {
    channel, hdmaTimer, snapshot_padding0, dmaTimer, dmaBusy, pad
} skip {
    snes
});

state_record!(DspState, 3024, {
    ram, channel, dirPage, evenCycle, mute, reset, masterVolumeL, masterVolumeR,
    snapshot_padding0, noiseSample, noiseRate, noiseCounter, echoWrites, echoVolumeL,
    echoVolumeR, feedbackVolume, echoBufferAdr, echoDelay, echoRemain, echoBufferIndex,
    firBufferIndex, firValues, snapshot_padding1, firBufferL, firBufferR, sampleBuffer,
    sampleOffset, snapshot_padding2
} skip {
    apu_ram
});

state_record!(Ppu, 67305, {
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

state_record!(Snes, 54, {
    hPos, vPos, frames, cpuCyclesLeft, cpuMemOps, padpad, apuCatchupCycles, hIrqEnabled,
    vIrqEnabled, nmiEnabled, snapshot_padding0, hTimer, vTimer, inNmi, inIrq, inVblank,
    snapshot_padding1, portAutoReadX, autoJoyRead, snapshot_padding2, autoJoyTimer, ppuLatch,
    multiplyA, multiplyResult, divideA, divideResult, fastMem, openBus
} skip {
    cpu, apu, ppu, snes_ppu, my_ppu, dma, cart, input1, input2, debug_cycles, debug_apu_cycles,
    disableRender, runningWhichVersion, ramAdr, ram, padx
});

state_record!(DmaChannel, 26, {
    bAdr, snapshot_padding0, aAdr, aBank, snapshot_padding1, size, indBank, snapshot_padding2,
    tableAdr, repCount, unusedByte, dmaActive, hdmaActive, mode, fixed, decrement, indirect,
    fromB, unusedBit, doTransfer, terminated, offIndex, snapshot_padding3
} skip {

});

state_record!(DspChannel, 86, {
    pitch, pitchCounter, pitchModulation, snapshot_padding0, decodeBuffer, srcn,
    snapshot_padding1, decodeOffset, previousFlags, snapshot_padding2, old, older, useNoise,
    snapshot_padding3, adsrRates, rateCounter, adsrState, snapshot_padding4, sustainLevel,
    useGain, gainMode, directGain, snapshot_padding5, gainValue, gain, keyOn, keyOff, sampleOut,
    volumeL, volumeR, echoEnable, snapshot_padding6
} skip {

});

state_record!(BgLayer, 12, {
    hScroll, vScroll, tilemapWider, tilemapHigher, tilemapAdr, tileAdr, bigTiles, mosaicEnabled
} skip {

});

state_record!(Layer, 4, {
    mainScreenEnabled, subScreenEnabled, mainScreenWindowed, subScreenWindowed
} skip {

});

state_record!(WindowLayer, 5, {
    window1enabled, window2enabled, window1inversed, window2inversed, maskLogic
} skip {

});

impl State for crate::apu_ram::ApuRam {
    const SIZE: usize = 65536;
    fn transfer(&mut self, fields: &mut Fields<'_>) {
        for offset in 0..65536 {
            let mut byte = self.read(offset as u16);
            byte.transfer(fields);
            self.write(offset as u16, byte);
        }
    }
}
state_record!(Apu, 65576, { ram, registers } skip { spc, dsp });
state_record!(ApuRegisters, 40, {
    romReadable, dspAdr, snapshot_padding0, cycles, inPorts, outPorts, timer, cpuCyclesLeft, pad
} skip { history });
state_record!(Timer, 5, { cycles, divider, target, counter, enabled } skip { });
