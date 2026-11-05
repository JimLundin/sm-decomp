//! The PPU's register and memory ports, in Rust. Ported from LakeSnes's PPU
//! (MIT, revision 28b8fbc15ad15afff003165ff4f2a8adb1c93d18), the behaviour the
//! emulated backend gives the same writes.
use crate::types::Ppu;

fn vram_remap(ppu: &Ppu) -> u16 {
    let adr = ppu.vramPointer;
    match ppu.vramRemapMode {
        0 => adr,
        1 => (adr & 0xff00) | ((adr & 0xe0) >> 5) | ((adr & 0x1f) << 3),
        2 => (adr & 0xfe00) | ((adr & 0x1c0) >> 6) | ((adr & 0x3f) << 3),
        _ => (adr & 0xfc00) | ((adr & 0x380) >> 7) | ((adr & 0x7f) << 3),
    }
}

fn vram_word(ppu: &Ppu) -> usize {
    usize::from(vram_remap(ppu) & 0x7fff)
}

pub(crate) unsafe fn sm_backend_ppu_read(ppu: *mut Ppu, adr: u8, h: u16, v: u16, bus: u8) -> u8 {
    let ppu = unsafe { &mut *ppu };
    match adr {
        0x04 | 0x14 | 0x24 | 0x05 | 0x15 | 0x25 | 0x06 | 0x16 | 0x26 | 0x08 | 0x18 | 0x28
        | 0x09 | 0x19 | 0x29 | 0x0a | 0x1a | 0x2a => ppu.ppu1openBus,
        0x34..=0x36 => {
            let result = i32::from(ppu.m7matrix[0]) * (i32::from(ppu.m7matrix[1]) >> 8);
            ppu.ppu1openBus = (result >> (8 * i32::from(adr - 0x34))) as u8;
            ppu.ppu1openBus
        }
        0x37 => {
            ppu.hCount = h / 4;
            ppu.vCount = v;
            ppu.countersLatched = true;
            bus
        }
        0x38 => {
            let ret;
            if ppu.oamInHigh {
                let at = usize::from(ppu.oamAdr & 0xf) << 1 | usize::from(ppu.oamSecondWrite);
                ret = ppu.highOam[at];
                if ppu.oamSecondWrite {
                    ppu.oamAdr = ppu.oamAdr.wrapping_add(1);
                    if ppu.oamAdr == 0 {
                        ppu.oamInHigh = false;
                    }
                }
            } else if !ppu.oamSecondWrite {
                ret = ppu.oam[usize::from(ppu.oamAdr)] as u8;
            } else {
                ret = (ppu.oam[usize::from(ppu.oamAdr)] >> 8) as u8;
                ppu.oamAdr = ppu.oamAdr.wrapping_add(1);
                if ppu.oamAdr == 0 {
                    ppu.oamInHigh = true;
                }
            }
            ppu.oamSecondWrite = !ppu.oamSecondWrite;
            ppu.ppu1openBus = ret;
            ret
        }
        0x39 | 0x3a => {
            let high = adr == 0x3a;
            let val = ppu.vramReadBuffer;
            if high == ppu.vramIncrementOnHigh {
                ppu.vramReadBuffer = ppu.vram[vram_word(ppu)];
                ppu.vramPointer = ppu.vramPointer.wrapping_add(ppu.vramIncrement);
            }
            let ret = if high { (val >> 8) as u8 } else { val as u8 };
            ppu.ppu1openBus = ret;
            ret
        }
        0x3b => {
            let ret;
            let word = ppu.cgram[usize::from(ppu.cgramPointer)];
            if !ppu.cgramSecondWrite {
                ret = word as u8;
            } else {
                ret = ((word >> 8) as u8 & 0x7f) | (ppu.ppu2openBus & 0x80);
                ppu.cgramPointer = ppu.cgramPointer.wrapping_add(1);
            }
            ppu.cgramSecondWrite = !ppu.cgramSecondWrite;
            ppu.ppu2openBus = ret;
            ret
        }
        0x3c => {
            let val = if ppu.hCountSecond {
                ((ppu.hCount >> 8) as u8 & 1) | (ppu.ppu2openBus & 0xfe)
            } else {
                ppu.hCount as u8
            };
            ppu.hCountSecond = !ppu.hCountSecond;
            ppu.ppu2openBus = val;
            val
        }
        0x3d => {
            let val = if ppu.vCountSecond {
                ((ppu.vCount >> 8) as u8 & 1) | (ppu.ppu2openBus & 0xfe)
            } else {
                ppu.vCount as u8
            };
            ppu.vCountSecond = !ppu.vCountSecond;
            ppu.ppu2openBus = val;
            val
        }
        0x3e => {
            let val = 1
                | (ppu.ppu1openBus & 0x10)
                | (u8::from(ppu.rangeOver) << 6)
                | (u8::from(ppu.timeOver) << 7);
            ppu.ppu1openBus = val;
            val
        }
        0x3f => {
            let val = 3
                | (ppu.ppu2openBus & 0x20)
                | (u8::from(ppu.countersLatched) << 6)
                | (u8::from(ppu.evenFrame) << 7);
            ppu.countersLatched = false;
            ppu.hCountSecond = false;
            ppu.vCountSecond = false;
            ppu.ppu2openBus = val;
            val
        }
        _ => bus,
    }
}

pub(crate) unsafe fn sm_backend_ppu_write(ppu: *mut Ppu, adr: u8, val: u8) {
    let ppu = unsafe { &mut *ppu };
    let bit = |mask: u8| val & mask != 0;
    let high = u16::from(val) << 8;
    match adr {
        0x00 => {
            ppu.brightness = val & 0xf;
            ppu.forcedBlank = bit(0x80);
        }
        0x01 => {
            ppu.objSize = val >> 5;
            ppu.objTileAdr1 = u16::from(val & 7) << 13;
            ppu.objTileAdr2 = ppu
                .objTileAdr1
                .wrapping_add((u16::from(val & 0x18) + 8) << 9);
        }
        0x02 => {
            ppu.oamAdr = val;
            ppu.oamAdrWritten = ppu.oamAdr;
            ppu.oamInHigh = ppu.oamInHighWritten;
            ppu.oamSecondWrite = false;
        }
        0x03 => {
            ppu.objPriority = bit(0x80);
            ppu.oamInHigh = bit(1);
            ppu.oamInHighWritten = ppu.oamInHigh;
            ppu.oamAdr = ppu.oamAdrWritten;
            ppu.oamSecondWrite = false;
        }
        0x04 => {
            if ppu.oamInHigh {
                let at = usize::from(ppu.oamAdr & 0xf) << 1 | usize::from(ppu.oamSecondWrite);
                ppu.highOam[at] = val;
                if ppu.oamSecondWrite {
                    ppu.oamAdr = ppu.oamAdr.wrapping_add(1);
                    if ppu.oamAdr == 0 {
                        ppu.oamInHigh = false;
                    }
                }
            } else if !ppu.oamSecondWrite {
                ppu.oamBuffer = val;
            } else {
                ppu.oam[usize::from(ppu.oamAdr)] = high | u16::from(ppu.oamBuffer);
                ppu.oamAdr = ppu.oamAdr.wrapping_add(1);
                if ppu.oamAdr == 0 {
                    ppu.oamInHigh = true;
                }
            }
            ppu.oamSecondWrite = !ppu.oamSecondWrite;
        }
        0x05 => {
            ppu.mode = val & 7;
            ppu.bg3priority = bit(8);
            for (i, layer) in ppu.bgLayer.iter_mut().enumerate() {
                layer.bigTiles = bit(0x10 << i);
            }
        }
        0x06 => {
            for (i, layer) in ppu.bgLayer.iter_mut().enumerate() {
                layer.mosaicEnabled = bit(1 << i);
            }
            ppu.mosaicSize = (val >> 4) + 1;
            // The mosaic restarts at line zero, independently of the beam.
            ppu.mosaicStartLine = 0;
        }
        0x07..=0x0a => {
            let layer = &mut ppu.bgLayer[usize::from(adr - 7)];
            layer.tilemapWider = bit(1);
            layer.tilemapHigher = bit(2);
            layer.tilemapAdr = u16::from(val & 0xfc) << 8;
        }
        0x0b => {
            ppu.bgLayer[0].tileAdr = u16::from(val & 0xf) << 12;
            ppu.bgLayer[1].tileAdr = u16::from(val & 0xf0) << 8;
        }
        0x0c => {
            ppu.bgLayer[2].tileAdr = u16::from(val & 0xf) << 12;
            ppu.bgLayer[3].tileAdr = u16::from(val & 0xf0) << 8;
        }
        0x0d | 0x0f | 0x11 | 0x13 => {
            if adr == 0x0d {
                ppu.m7matrix[6] = ((high | u16::from(ppu.m7prev)) & 0x1fff) as i16;
                ppu.m7prev = val;
            }
            ppu.bgLayer[usize::from(adr - 0xd) / 2].hScroll =
                (high | u16::from(ppu.scrollPrev & 0xf8) | u16::from(ppu.scrollPrev2 & 7)) & 0x3ff;
            ppu.scrollPrev = val;
            ppu.scrollPrev2 = val;
        }
        0x0e | 0x10 | 0x12 | 0x14 => {
            if adr == 0x0e {
                ppu.m7matrix[7] = ((high | u16::from(ppu.m7prev)) & 0x1fff) as i16;
                ppu.m7prev = val;
            }
            ppu.bgLayer[usize::from(adr - 0xe) / 2].vScroll =
                (high | u16::from(ppu.scrollPrev)) & 0x3ff;
            ppu.scrollPrev = val;
        }
        0x15 => {
            ppu.vramIncrement = match val & 3 {
                0 => 1,
                1 => 32,
                _ => 128,
            };
            ppu.vramRemapMode = (val & 0xc) >> 2;
            ppu.vramIncrementOnHigh = bit(0x80);
        }
        0x16 => {
            ppu.vramPointer = (ppu.vramPointer & 0xff00) | u16::from(val);
            ppu.vramReadBuffer = ppu.vram[vram_word(ppu)];
        }
        0x17 => {
            ppu.vramPointer = (ppu.vramPointer & 0x00ff) | high;
            ppu.vramReadBuffer = ppu.vram[vram_word(ppu)];
        }
        0x18 => {
            let at = vram_word(ppu);
            ppu.vram[at] = (ppu.vram[at] & 0xff00) | u16::from(val);
            if !ppu.vramIncrementOnHigh {
                ppu.vramPointer = ppu.vramPointer.wrapping_add(ppu.vramIncrement);
            }
        }
        0x19 => {
            let at = vram_word(ppu);
            ppu.vram[at] = (ppu.vram[at] & 0x00ff) | high;
            if ppu.vramIncrementOnHigh {
                ppu.vramPointer = ppu.vramPointer.wrapping_add(ppu.vramIncrement);
            }
        }
        0x1a => {
            ppu.m7largeField = bit(0x80);
            ppu.m7charFill = bit(0x40);
            ppu.m7yFlip = bit(2);
            ppu.m7xFlip = bit(1);
        }
        0x1b..=0x1e => {
            ppu.m7matrix[usize::from(adr - 0x1b)] = (high | u16::from(ppu.m7prev)) as i16;
            ppu.m7prev = val;
        }
        0x1f | 0x20 => {
            ppu.m7matrix[usize::from(adr - 0x1b)] =
                ((high | u16::from(ppu.m7prev)) & 0x1fff) as i16;
            ppu.m7prev = val;
        }
        0x21 => {
            ppu.cgramPointer = val;
            ppu.cgramSecondWrite = false;
        }
        0x22 => {
            if !ppu.cgramSecondWrite {
                ppu.cgramBuffer = val;
            } else {
                ppu.cgram[usize::from(ppu.cgramPointer)] = high | u16::from(ppu.cgramBuffer);
                ppu.cgramPointer = ppu.cgramPointer.wrapping_add(1);
            }
            ppu.cgramSecondWrite = !ppu.cgramSecondWrite;
        }
        0x23..=0x25 => {
            let first = usize::from(adr - 0x23) * 2;
            let (low, upper) = (&mut ppu.windowLayer[first..first + 2]).split_at_mut(1);
            low[0].window1inversed = bit(1);
            low[0].window1enabled = bit(2);
            low[0].window2inversed = bit(4);
            low[0].window2enabled = bit(8);
            upper[0].window1inversed = bit(0x10);
            upper[0].window1enabled = bit(0x20);
            upper[0].window2inversed = bit(0x40);
            upper[0].window2enabled = bit(0x80);
        }
        0x26 => ppu.window1left = val,
        0x27 => ppu.window1right = val,
        0x28 => ppu.window2left = val,
        0x29 => ppu.window2right = val,
        0x2a => {
            for i in 0..4 {
                ppu.windowLayer[i].maskLogic = (val >> (2 * i)) & 3;
            }
        }
        0x2b => {
            ppu.windowLayer[4].maskLogic = val & 3;
            ppu.windowLayer[5].maskLogic = (val >> 2) & 3;
        }
        0x2c => (0..5).for_each(|i| ppu.layer[i].mainScreenEnabled = bit(1 << i)),
        0x2d => (0..5).for_each(|i| ppu.layer[i].subScreenEnabled = bit(1 << i)),
        0x2e => (0..5).for_each(|i| ppu.layer[i].mainScreenWindowed = bit(1 << i)),
        0x2f => (0..5).for_each(|i| ppu.layer[i].subScreenWindowed = bit(1 << i)),
        0x30 => {
            ppu.directColor = bit(1);
            ppu.addSubscreen = bit(2);
            ppu.preventMathMode = (val & 0x30) >> 4;
            ppu.clipMode = (val & 0xc0) >> 6;
        }
        0x31 => {
            ppu.subtractColor = bit(0x80);
            ppu.halfColor = bit(0x40);
            (0..6).for_each(|i| ppu.mathEnabled[i] = bit(1 << i));
        }
        0x32 => {
            if bit(0x80) {
                ppu.fixedColorB = val & 0x1f;
            }
            if bit(0x40) {
                ppu.fixedColorG = val & 0x1f;
            }
            if bit(0x20) {
                ppu.fixedColorR = val & 0x1f;
            }
        }
        0x33 => {
            ppu.interlace = bit(1);
            ppu.objInterlace = bit(2);
            ppu.overscan = bit(4);
            ppu.pseudoHires = bit(8);
            ppu.m7extBg = bit(0x40);
        }
        _ => {}
    }
}
