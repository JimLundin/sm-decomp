//! The PPU's register and memory ports and its reference pixel, in Rust. Ported from LakeSnes's PPU
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

const LAYERS: [[i32; 12]; 10] = [
    [4, 0, 1, 4, 0, 1, 4, 2, 3, 4, 2, 3],
    [4, 0, 1, 4, 0, 1, 4, 2, 4, 2, 5, 5],
    [4, 0, 4, 1, 4, 0, 4, 1, 5, 5, 5, 5],
    [4, 0, 4, 1, 4, 0, 4, 1, 5, 5, 5, 5],
    [4, 0, 4, 1, 4, 0, 4, 1, 5, 5, 5, 5],
    [4, 0, 4, 1, 4, 0, 4, 1, 5, 5, 5, 5],
    [4, 0, 4, 4, 0, 4, 5, 5, 5, 5, 5, 5],
    [4, 4, 4, 0, 4, 5, 5, 5, 5, 5, 5, 5],
    [2, 4, 0, 1, 4, 0, 1, 4, 4, 2, 5, 5],
    [4, 4, 1, 4, 0, 4, 1, 5, 5, 5, 5, 5],
];
const PRIORITIES: [[i32; 12]; 10] = [
    [3, 1, 1, 2, 0, 0, 1, 1, 1, 0, 0, 0],
    [3, 1, 1, 2, 0, 0, 1, 1, 0, 0, 5, 5],
    [3, 1, 2, 1, 1, 0, 0, 0, 5, 5, 5, 5],
    [3, 1, 2, 1, 1, 0, 0, 0, 5, 5, 5, 5],
    [3, 1, 2, 1, 1, 0, 0, 0, 5, 5, 5, 5],
    [3, 1, 2, 1, 1, 0, 0, 0, 5, 5, 5, 5],
    [3, 1, 2, 1, 0, 0, 5, 5, 5, 5, 5, 5],
    [3, 2, 1, 0, 0, 5, 5, 5, 5, 5, 5, 5],
    [1, 3, 1, 1, 2, 0, 0, 1, 0, 0, 5, 5],
    [3, 2, 1, 1, 0, 0, 0, 5, 5, 5, 5, 5],
];
const BIT_DEPTHS: [[i32; 4]; 10] = [
    [2, 2, 2, 2],
    [4, 4, 2, 5],
    [4, 4, 5, 5],
    [8, 4, 5, 5],
    [8, 2, 5, 5],
    [4, 2, 5, 5],
    [4, 5, 5, 5],
    [8, 5, 5, 5],
    [4, 4, 2, 5],
    [8, 7, 5, 5],
];
const LAYER_COUNTS: [i32; 10] = [12, 10, 8, 8, 8, 8, 6, 5, 10, 7];

fn window_state(ppu: &Ppu, layer: usize, x: i32) -> bool {
    let w = &ppu.windowLayer[layer];
    let inside = |left: u8, right: u8| x >= i32::from(left) && x <= i32::from(right);
    let test1 = inside(ppu.window1left, ppu.window1right) != w.window1inversed;
    let test2 = inside(ppu.window2left, ppu.window2right) != w.window2inversed;
    match (w.window1enabled, w.window2enabled) {
        (false, false) => false,
        (true, false) => test1,
        (false, true) => test2,
        (true, true) => match w.maskLogic {
            0 => test1 || test2,
            1 => test1 && test2,
            2 => test1 != test2,
            _ => test1 == test2,
        },
    }
}

fn offset_value(ppu: &Ppu, col: i32, row: i32) -> u16 {
    let bg = &ppu.bgLayer[2];
    let x = col * 8 + i32::from(bg.hScroll);
    let y = row * 8 + i32::from(bg.vScroll);
    let tile_bits = if bg.bigTiles { 4 } else { 3 };
    let high_bit = if bg.bigTiles { 0x200 } else { 0x100 };
    let mut map = bg
        .tilemapAdr
        .wrapping_add((((y >> tile_bits) & 0x1f) << 5 | ((x >> tile_bits) & 0x1f)) as u16);
    if x & high_bit != 0 && bg.tilemapWider {
        map = map.wrapping_add(0x400);
    }
    if y & high_bit != 0 && bg.tilemapHigher {
        map = map.wrapping_add(if bg.tilemapWider { 0x800 } else { 0x400 });
    }
    ppu.vram[usize::from(map & 0x7fff)]
}

fn offset_per_tile(ppu: &Ppu, layer: usize, lx: &mut i32, ly: &mut i32) {
    let (x, y) = (*lx, *ly);
    let scroll = i32::from(ppu.bgLayer[layer].hScroll);
    let column = if ppu.mode == 6 {
        ((x - (x & 0xf)) - ((scroll * 2) & 0xfff0)) >> 4
    } else {
        ((x - (x & 7)) - (scroll & 0xfff8)) >> 3
    };
    if column > 0 {
        let valid = if layer == 0 { 0x2000 } else { 0x4000 };
        let mut horizontal = offset_value(ppu, column - 1, 0);
        let mut vertical = 0;
        if ppu.mode == 4 {
            if horizontal & 0x8000 != 0 {
                vertical = horizontal;
                horizontal = 0;
            }
        } else {
            vertical = offset_value(ppu, column - 1, 1);
        }
        if horizontal & valid != 0 {
            let moved = i32::from(horizontal & 0x3f8) + column * 8;
            *lx = if ppu.mode == 6 {
                (moved * 2) | (x & 0xf)
            } else {
                moved | (x & 7)
            };
        }
        if vertical & valid != 0 {
            *ly = i32::from(vertical & 0x3ff) + (y - i32::from(ppu.bgLayer[layer].vScroll));
        }
    }
}

fn background_pixel(ppu: &Ppu, x: i32, y: i32, layer: usize, priority: bool) -> i32 {
    let bg = &ppu.bgLayer[layer];
    let wide = bg.bigTiles || ppu.mode == 5 || ppu.mode == 6;
    let (bits_x, high_x) = if wide { (4, 0x200) } else { (3, 0x100) };
    let (bits_y, high_y) = if bg.bigTiles { (4, 0x200) } else { (3, 0x100) };
    let mut map = bg
        .tilemapAdr
        .wrapping_add((((y >> bits_y) & 0x1f) << 5 | ((x >> bits_x) & 0x1f)) as u16);
    if x & high_x != 0 && bg.tilemapWider {
        map = map.wrapping_add(0x400);
    }
    if y & high_y != 0 && bg.tilemapHigher {
        map = map.wrapping_add(if bg.tilemapWider { 0x800 } else { 0x400 });
    }
    let tile = i32::from(ppu.vram[usize::from(map & 0x7fff)]);
    if (tile & 0x2000 != 0) != priority {
        return 0;
    }
    let mut palette = (tile & 0x1c00) >> 10;
    let row = if tile & 0x8000 != 0 {
        7 - (y & 7)
    } else {
        y & 7
    };
    let col = if tile & 0x4000 != 0 {
        x & 7
    } else {
        7 - (x & 7)
    };
    let mut number = tile & 0x3ff;
    if wide && ((x & 8 != 0) != (tile & 0x4000 != 0)) {
        number += 1;
    }
    if bg.bigTiles && ((y & 8 != 0) != (tile & 0x8000 != 0)) {
        number += 0x10;
    }
    let depth = BIT_DEPTHS[usize::from(ppu.mode)][layer];
    if ppu.mode == 0 {
        palette += 8 * layer as i32;
    }
    let plane = |offset: i32| {
        let base = i32::from(bg.tileAdr) + (number & 0x3ff) * 4 * depth + offset + row;
        i32::from(ppu.vram[(base & 0x7fff) as usize])
    };
    let mut size = 4;
    let first = plane(0);
    let mut pixel = ((first >> col) & 1) | (((first >> (8 + col)) & 1) << 1);
    if depth > 2 {
        size = 16;
        let second = plane(8);
        pixel |= ((second >> col) & 1) << 2 | ((second >> (8 + col)) & 1) << 3;
    }
    if depth > 4 {
        size = 256;
        let third = plane(16);
        pixel |= ((third >> col) & 1) << 4 | ((third >> (8 + col)) & 1) << 5;
        let fourth = plane(24);
        pixel |= ((fourth >> col) & 1) << 6 | ((fourth >> (8 + col)) & 1) << 7;
    }
    if pixel == 0 {
        0
    } else {
        size * palette + pixel
    }
}

fn affine_pixel(ppu: &Ppu, x: i32, layer: usize, priority: bool) -> i32 {
    let rx = i32::from(if ppu.m7xFlip {
        (255 - x) as u8
    } else {
        x as u8
    });
    let mut x_pos = ppu
        .m7startX
        .wrapping_add(i32::from(ppu.m7matrix[0]).wrapping_mul(rx))
        >> 8;
    let mut y_pos = ppu
        .m7startY
        .wrapping_add(i32::from(ppu.m7matrix[2]).wrapping_mul(rx))
        >> 8;
    let mut outside = x_pos < 0 || x_pos >= 1024 || y_pos < 0 || y_pos >= 1024;
    x_pos &= 0x3ff;
    y_pos &= 0x3ff;
    if !ppu.m7largeField {
        outside = false;
    }
    let tile = if outside {
        0
    } else {
        ppu.vram[((y_pos >> 3) * 128 + (x_pos >> 3)) as usize] & 0xff
    };
    let pixel = if outside && !ppu.m7charFill {
        0
    } else {
        ppu.vram[(i32::from(tile) * 64 + (y_pos & 7) * 8 + (x_pos & 7)) as usize] >> 8
    } as i32;
    if layer == 1 {
        if (pixel & 0x80 != 0) != priority {
            return 0;
        }
        return pixel & 0x7f;
    }
    pixel
}

/// One layer's pixel on the mosaic and extended-background lines, which the
/// line renderer does not draw: the layer order per mode, sprites from the
/// packed sprite word, then the colour.
pub(crate) unsafe fn sm_backend_ppu_pixel(
    ppu: *mut Ppu,
    x: i32,
    y: i32,
    sub: bool,
    sprite: u16,
    color: *mut i32,
) -> i32 {
    let ppu = unsafe { &*ppu };
    let tag = i32::from(sprite >> 12);
    let sprite_priority = if tag & 3 == 2 { tag >> 2 } else { 0xff };
    let sprite_pixel = i32::from(sprite & 0xff);
    let mode = if ppu.mode == 7 && ppu.m7extBg {
        9
    } else if ppu.mode == 1 && ppu.bg3priority {
        8
    } else {
        usize::from(ppu.mode)
    };
    let mut layer = 5;
    let mut pixel = 0;
    for i in 0..LAYER_COUNTS[mode] as usize {
        let current = LAYERS[mode][i] as usize;
        let priority = PRIORITIES[mode][i];
        let routed = &ppu.layer[current];
        let (enabled, windowed) = if sub {
            (routed.subScreenEnabled, routed.subScreenWindowed)
        } else {
            (routed.mainScreenEnabled, routed.mainScreenWindowed)
        };
        if enabled && (!windowed || !window_state(ppu, current, x)) {
            if current < 4 {
                let bg = &ppu.bgLayer[current];
                let (mut lx, mut ly) = (x, y);
                if bg.mosaicEnabled && ppu.mosaicSize > 1 {
                    lx -= lx % i32::from(ppu.mosaicSize);
                    ly -= (ly - i32::from(ppu.mosaicStartLine)) % i32::from(ppu.mosaicSize);
                }
                if ppu.mode == 7 {
                    pixel = affine_pixel(ppu, lx, current, priority != 0);
                } else {
                    lx += i32::from(bg.hScroll);
                    if ppu.mode == 5 || ppu.mode == 6 {
                        lx *= 2;
                        lx += if sub || bg.mosaicEnabled { 0 } else { 1 };
                        if ppu.interlace {
                            ly *= 2;
                            ly += if ppu.evenFrame || bg.mosaicEnabled {
                                0
                            } else {
                                1
                            };
                        }
                    }
                    ly += i32::from(bg.vScroll);
                    if ppu.mode == 2 || ppu.mode == 4 || ppu.mode == 6 {
                        offset_per_tile(ppu, current, &mut lx, &mut ly);
                    }
                    pixel = background_pixel(ppu, lx & 0x3ff, ly & 0x3ff, current, priority != 0);
                }
            } else {
                pixel = 0;
                if sprite_priority == priority {
                    pixel = sprite_pixel;
                }
            }
        }
        if pixel > 0 {
            layer = current as i32;
            break;
        }
    }
    let rgb = if ppu.directColor && layer < 4 && BIT_DEPTHS[mode][layer as usize] == 8 {
        [
            ((pixel & 7) << 2) | ((pixel & 0x100) >> 7),
            ((pixel & 0x38) >> 1) | ((pixel & 0x200) >> 8),
            ((pixel & 0xc0) >> 3) | ((pixel & 0x400) >> 8),
        ]
    } else {
        let word = i32::from(ppu.cgram[(pixel & 0xff) as usize]);
        [word & 0x1f, (word >> 5) & 0x1f, (word >> 10) & 0x1f]
    };
    unsafe { core::ptr::copy_nonoverlapping(rgb.as_ptr(), color, 3) };
    if layer == 4 && pixel < 0xc0 { 6 } else { layer }
}
