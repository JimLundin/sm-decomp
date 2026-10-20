use crate::c_library::{free, malloc, memcpy, memset, printf};
use crate::game_state::Game;
use crate::snes::cart::cart_load;
use crate::snes::dsp::dsp_getSamples;
use crate::snes::snes::snes_reset;
pub use crate::types::{
    size_t, Apu, ApuHistory as C2RustUnnamed, BgLayer, Cart, Cpu, Dma, DmaChannel, Dsp, DspChannel,
    DspRegWriteHistory, Input, Layer, Ppu, PpuPixelPrioBufs, PpuZbufType, Snes, Spc, Timer,
    WindowLayer,
};
use ::core::ffi::{c_char, c_void};
use ::core::mem::size_of;
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct CartHeader {
        pub headerVersion: u8;
        pub name: [c_char; 22];
        pub speed, type_0, coprocessor, chips: u8;
        pub romSize, ramSize: u32;
        pub region, maker, version: u8;
        pub checksumComplement, checksum: u16;
        pub makerCode: [c_char; 3];
        pub gameCode: [c_char; 5];
        pub flashSize, exRamSize: u32;
        pub specialVersion, exCoprocessor: u8;
        pub score: i16;
        pub pal: bool;
        pub cartType: u8;
    }
}
pub const true_0: i32 = 1;
pub const false_0: i32 = 0;
pub unsafe fn snes_loadRom(
    snes: *mut Snes,
    mut data: *const u8,
    mut length: i32,
    game: &mut Game,
) -> bool {
    if length < 0x8000i32 {
        printf(
            b"Failed to load rom: rom to small (%d bytes)\n\0" as *const u8 as *const c_char,
            length,
        );
        return false_0 != 0;
    }
    let mut headers: [CartHeader; 4] = [CartHeader {
        headerVersion: 0,
        name: [0; 22],
        speed: 0,
        type_0: 0,
        coprocessor: 0,
        chips: 0,
        romSize: 0,
        ramSize: 0,
        region: 0,
        maker: 0,
        version: 0,
        checksumComplement: 0,
        checksum: 0,
        makerCode: [0; 3],
        gameCode: [0; 5],
        flashSize: 0,
        exRamSize: 0,
        specialVersion: 0,
        exCoprocessor: 0,
        score: 0,
        pal: false,
        cartType: 0,
    }; 4];
    memset(
        &raw mut headers as *mut CartHeader as *mut c_void,
        0i32,
        size_of::<[CartHeader; 4]>() as size_t,
    );
    let mut i: i32 = 0i32;
    while i < 4i32 {
        headers[i as usize].score = -50i16;
        i += 1;
    }
    if length >= 0x8000i32 {
        readHeader(
            data,
            length,
            0x7fc0i32,
            (&raw mut headers as *mut CartHeader).offset(0),
        );
    }
    if length >= 0x8200i32 {
        readHeader(
            data,
            length,
            0x81c0i32,
            (&raw mut headers as *mut CartHeader).offset(1),
        );
    }
    if length >= 0x10000i32 {
        readHeader(
            data,
            length,
            0xffc0i32,
            (&raw mut headers as *mut CartHeader).offset(2),
        );
    }
    if length >= 0x10200i32 {
        readHeader(
            data,
            length,
            0x101c0i32,
            (&raw mut headers as *mut CartHeader).offset(3),
        );
    }
    let mut max: i32 = 0i32;
    let mut used: i32 = 0i32;
    let mut i_0: i32 = 0i32;
    while i_0 < 4i32 {
        if headers[i_0 as usize].score as i32 > max {
            max = headers[i_0 as usize].score as i32;
            used = i_0;
        }
        i_0 += 1;
    }
    if used & 1i32 != 0 {
        data = data.offset(0x200);
        length -= 0x200i32;
    }
    if headers[used as usize].cartType as i32 > 2i32 {
        printf(
            b"Failed to load rom: unsupported type (%d)\n\0" as *const u8 as *const c_char,
            headers[used as usize].cartType as i32,
        );
        return false_0 != 0;
    }
    let mut newLength: i32 = 0x8000i32;
    while (length > newLength) {
        newLength *= 2i32;
    }
    let newData = malloc(newLength as size_t) as *mut u8;
    memcpy(
        newData as *mut c_void,
        data as *const c_void,
        length as size_t,
    );
    let mut test: i32 = 1i32;
    while length != newLength {
        if length & test != 0 {
            memcpy(
                newData.offset(length as isize) as *mut c_void,
                newData.offset(length as isize).offset(-(test as isize)) as *const c_void,
                test as size_t,
            );
            length += test;
        }
        test *= 2i32;
    }
    cart_load(
        (*snes).cart,
        headers[used as usize].cartType as i32,
        newData,
        newLength,
        (if headers[used as usize].chips as i32 > 0i32 {
            headers[used as usize].ramSize
        } else {
            0u32
        }) as i32,
    );
    snes_reset(snes, true_0 != 0, game);
    free(newData as *mut c_void);
    true_0 != 0
}
pub unsafe fn snes_setSamples(snes: *mut Snes, sampleData: *mut i16, samplesPerFrame: i32) {
    dsp_getSamples((*(*snes).apu).dsp, sampleData, samplesPerFrame);
}
unsafe fn readHeader(data: *const u8, length: i32, location: i32, header: *mut CartHeader) {
    let mut i: i32 = 0i32;
    while i < 21i32 {
        let ch: u8 = *data.offset((location + i) as isize);
        if ch as i32 >= 0x20i32 && (ch as i32) < 0x7fi32 {
            (*header).name[i as usize] = ch as c_char;
        } else {
            (*header).name[i as usize] = '.' as i32 as c_char;
        }
        i += 1;
    }
    (*header).name[21i32 as usize] = 0 as c_char;
    (*header).speed = (*data.offset((location + 0x15i32) as isize) >> 4);
    (*header).type_0 = (*data.offset((location + 0x15i32) as isize) & 0xf);
    (*header).coprocessor = (*data.offset((location + 0x16i32) as isize) >> 4);
    (*header).chips = (*data.offset((location + 0x16i32) as isize) & 0xf);
    (*header).romSize = ((0x400i32) << *data.offset((location + 0x17i32) as isize) as i32) as u32;
    (*header).ramSize = ((0x400i32) << *data.offset((location + 0x18i32) as isize) as i32) as u32;
    (*header).region = *data.offset((location + 0x19i32) as isize);
    (*header).maker = *data.offset((location + 0x1ai32) as isize);
    (*header).version = *data.offset((location + 0x1bi32) as isize);
    (*header).checksumComplement = (((*data.offset((location + 0x1di32) as isize) as i32) << 8i32)
        + *data.offset((location + 0x1ci32) as isize) as i32)
        as u16;
    (*header).checksum = (((*data.offset((location + 0x1fi32) as isize) as i32) << 8i32)
        + *data.offset((location + 0x1ei32) as isize) as i32) as u16;
    (*header).headerVersion = 1u8;
    if (*header).maker as i32 == 0x33i32 {
        (*header).headerVersion = 3u8;
        let mut i_0: i32 = 0i32;
        while i_0 < 2i32 {
            let ch_0: u8 = *data.offset((location - 0x10i32 + i_0) as isize);
            if ch_0 as i32 >= 0x20i32 && (ch_0 as i32) < 0x7fi32 {
                (*header).makerCode[i_0 as usize] = ch_0 as c_char;
            } else {
                (*header).makerCode[i_0 as usize] = '.' as i32 as c_char;
            }
            i_0 += 1;
        }
        (*header).makerCode[2i32 as usize] = 0 as c_char;
        let mut i_1: i32 = 0i32;
        while i_1 < 4i32 {
            let ch_1: u8 = *data.offset((location - 0xei32 + i_1) as isize);
            if ch_1 as i32 >= 0x20i32 && (ch_1 as i32) < 0x7fi32 {
                (*header).gameCode[i_1 as usize] = ch_1 as c_char;
            } else {
                (*header).gameCode[i_1 as usize] = '.' as i32 as c_char;
            }
            i_1 += 1;
        }
        (*header).gameCode[4i32 as usize] = 0 as c_char;
        (*header).flashSize =
            ((0x400i32) << *data.offset((location - 4i32) as isize) as i32) as u32;
        (*header).exRamSize =
            ((0x400i32) << *data.offset((location - 3i32) as isize) as i32) as u32;
        (*header).specialVersion = *data.offset((location - 2i32) as isize);
        (*header).exCoprocessor = *data.offset((location - 1i32) as isize);
    } else if *data.offset((location + 0x14i32) as isize) as i32 == 0i32 {
        (*header).headerVersion = 2u8;
        (*header).exCoprocessor = *data.offset((location - 1i32) as isize);
    }
    (*header).pal = (*header).region as i32 >= 0x2i32 && (*header).region as i32 <= 0xci32
        || (*header).region as i32 == 0x11i32;
    (*header).cartType = (if location < 0x9000i32 { 1i32 } else { 2i32 }) as u8;
    let mut score: i32 = 0i32;
    score += if (*header).speed as i32 == 2i32 || (*header).speed as i32 == 3i32 {
        5i32
    } else {
        -4i32
    };
    score += if (*header).type_0 as i32 <= 3i32 || (*header).type_0 as i32 == 5i32 {
        5i32
    } else {
        -2i32
    };
    score += if (*header).coprocessor as i32 <= 5i32 || (*header).coprocessor as i32 >= 0xei32 {
        5i32
    } else {
        -2i32
    };
    score += if (*header).chips as i32 <= 6i32
        || (*header).chips as i32 == 9i32
        || (*header).chips as i32 == 0xai32
    {
        5i32
    } else {
        -2i32
    };
    score += if (*header).region as i32 <= 0x14i32 {
        5i32
    } else {
        -2i32
    };
    score += if (*header).checksum as i32 + (*header).checksumComplement as i32 == 0xffffi32 {
        8i32
    } else {
        -6i32
    };
    let resetVector: u16 = (*data.offset((location + 0x3ci32) as isize) as i32
        | (*data.offset((location + 0x3di32) as isize) as i32) << 8i32)
        as u16;
    score += if resetVector as i32 >= 0x8000i32 {
        8i32
    } else {
        -20i32
    };
    let opcodeLoc: i32 = location + 0x40i32 - 0x8000i32 + (resetVector as i32 & 0x7fffi32);
    let mut opcode: u8 = 0xffu8;
    if opcodeLoc < length {
        opcode = *data.offset(opcodeLoc as isize);
    } else {
        score -= 14i32;
    }
    if opcode as i32 == 0x78i32 || opcode as i32 == 0x18i32 {
        score += 6i32;
    }
    if opcode as i32 == 0x4ci32 || opcode as i32 == 0x5ci32 || opcode as i32 == 0x9ci32 {
        score += 3i32;
    }
    if opcode as i32 == 0i32 || opcode as i32 == 0xffi32 || opcode as i32 == 0xdbi32 {
        score -= 6i32;
    }
    (*header).score = score as i16;
}
