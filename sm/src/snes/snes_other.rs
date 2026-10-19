use crate::game_state::Game;
use crate::snes::cart::cart_load;
use crate::snes::dsp::dsp_getSamples;
use crate::snes::snes::snes_reset;
extern "C" {
    fn printf(__format: *const ::core::ffi::c_char, ...) -> i32;
    fn malloc(__size: size_t) -> *mut ::core::ffi::c_void;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memset(__s: *mut ::core::ffi::c_void, __c: i32, __n: size_t) -> *mut ::core::ffi::c_void;

}
pub use crate::types::{
    size_t, Apu, ApuHistory as C2RustUnnamed, BgLayer, Cart, Cpu, Dma, DmaChannel, Dsp, DspChannel,
    DspRegWriteHistory, Input, Layer, Ppu, PpuPixelPrioBufs, PpuZbufType, Snes, Spc, Timer,
    WindowLayer,
};
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CartHeader {
    pub headerVersion: u8,
    pub name: [::core::ffi::c_char; 22],
    pub speed: u8,
    pub type_0: u8,
    pub coprocessor: u8,
    pub chips: u8,
    pub romSize: u32,
    pub ramSize: u32,
    pub region: u8,
    pub maker: u8,
    pub version: u8,
    pub checksumComplement: u16,
    pub checksum: u16,
    pub makerCode: [::core::ffi::c_char; 3],
    pub gameCode: [::core::ffi::c_char; 5],
    pub flashSize: u32,
    pub exRamSize: u32,
    pub specialVersion: u8,
    pub exCoprocessor: u8,
    pub score: i16,
    pub pal: bool,
    pub cartType: u8,
}
pub const true_0: i32 = 1;
pub const false_0: i32 = 0;
pub unsafe fn snes_loadRom(
    snes: *mut Snes,
    mut data: *const u8,
    mut length: i32,
    game: &mut Game,
) -> bool {
    if length < 0x8000 as i32 {
        printf(
            b"Failed to load rom: rom to small (%d bytes)\n\0" as *const u8
                as *const ::core::ffi::c_char,
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
        &raw mut headers as *mut CartHeader as *mut ::core::ffi::c_void,
        0 as i32,
        ::core::mem::size_of::<[CartHeader; 4]>() as size_t,
    );
    let mut i: i32 = 0 as i32;
    while i < 4 as i32 {
        headers[i as usize].score = -(50 as i32) as i16;
        i += 1;
    }
    if length >= 0x8000 as i32 {
        readHeader(
            data,
            length,
            0x7fc0 as i32,
            (&raw mut headers as *mut CartHeader).offset(0) as *mut CartHeader,
        );
    }
    if length >= 0x8200 as i32 {
        readHeader(
            data,
            length,
            0x81c0 as i32,
            (&raw mut headers as *mut CartHeader).offset(1) as *mut CartHeader,
        );
    }
    if length >= 0x10000 as i32 {
        readHeader(
            data,
            length,
            0xffc0 as i32,
            (&raw mut headers as *mut CartHeader).offset(2) as *mut CartHeader,
        );
    }
    if length >= 0x10200 as i32 {
        readHeader(
            data,
            length,
            0x101c0 as i32,
            (&raw mut headers as *mut CartHeader).offset(3) as *mut CartHeader,
        );
    }
    let mut max: i32 = 0 as i32;
    let mut used: i32 = 0 as i32;
    let mut i_0: i32 = 0 as i32;
    while i_0 < 4 as i32 {
        if headers[i_0 as usize].score as i32 > max {
            max = headers[i_0 as usize].score as i32;
            used = i_0;
        }
        i_0 += 1;
    }
    if used & 1 as i32 != 0 {
        data = data.offset(0x200);
        length -= 0x200 as i32;
    }
    if headers[used as usize].cartType as i32 > 2 as i32 {
        printf(
            b"Failed to load rom: unsupported type (%d)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            headers[used as usize].cartType as i32,
        );
        return false_0 != 0;
    }
    let mut newLength: i32 = 0x8000 as i32;
    while (length > newLength) {
        newLength *= 2 as i32;
    }
    let newData: *mut u8 = malloc(newLength as size_t) as *mut u8;
    memcpy(
        newData as *mut ::core::ffi::c_void,
        data as *const ::core::ffi::c_void,
        length as size_t,
    );
    let mut test: i32 = 1 as i32;
    while length != newLength {
        if length & test != 0 {
            memcpy(
                newData.offset(length as isize) as *mut ::core::ffi::c_void,
                newData.offset(length as isize).offset(-(test as isize))
                    as *const ::core::ffi::c_void,
                test as size_t,
            );
            length += test;
        }
        test *= 2 as i32;
    }
    cart_load(
        (*snes).cart,
        headers[used as usize].cartType as i32,
        newData,
        newLength,
        (if headers[used as usize].chips as i32 > 0 as i32 {
            headers[used as usize].ramSize
        } else {
            0 as u32
        }) as i32,
    );
    snes_reset(snes, true_0 != 0, game);
    free(newData as *mut ::core::ffi::c_void);
    true_0 != 0
}
pub unsafe fn snes_setSamples(snes: *mut Snes, sampleData: *mut i16, samplesPerFrame: i32) {
    dsp_getSamples((*(*snes).apu).dsp, sampleData, samplesPerFrame);
}
unsafe fn readHeader(data: *const u8, length: i32, location: i32, header: *mut CartHeader) {
    let mut i: i32 = 0 as i32;
    while i < 21 as i32 {
        let ch: u8 = *data.offset((location + i) as isize);
        if ch as i32 >= 0x20 as i32 && (ch as i32) < 0x7f as i32 {
            (*header).name[i as usize] = ch as ::core::ffi::c_char;
        } else {
            (*header).name[i as usize] = '.' as i32 as ::core::ffi::c_char;
        }
        i += 1;
    }
    (*header).name[21 as i32 as usize] = 0 as ::core::ffi::c_char;
    (*header).speed = (*data.offset((location + 0x15 as i32) as isize) as i32 >> 4 as i32) as u8;
    (*header).type_0 = (*data.offset((location + 0x15 as i32) as isize) as i32 & 0xf as i32) as u8;
    (*header).coprocessor =
        (*data.offset((location + 0x16 as i32) as isize) as i32 >> 4 as i32) as u8;
    (*header).chips = (*data.offset((location + 0x16 as i32) as isize) as i32 & 0xf as i32) as u8;
    (*header).romSize =
        ((0x400 as i32) << *data.offset((location + 0x17 as i32) as isize) as i32) as u32;
    (*header).ramSize =
        ((0x400 as i32) << *data.offset((location + 0x18 as i32) as isize) as i32) as u32;
    (*header).region = *data.offset((location + 0x19 as i32) as isize);
    (*header).maker = *data.offset((location + 0x1a as i32) as isize);
    (*header).version = *data.offset((location + 0x1b as i32) as isize);
    (*header).checksumComplement =
        (((*data.offset((location + 0x1d as i32) as isize) as i32) << 8 as i32)
            + *data.offset((location + 0x1c as i32) as isize) as i32) as u16;
    (*header).checksum = (((*data.offset((location + 0x1f as i32) as isize) as i32) << 8 as i32)
        + *data.offset((location + 0x1e as i32) as isize) as i32) as u16;
    (*header).headerVersion = 1 as u8;
    if (*header).maker as i32 == 0x33 as i32 {
        (*header).headerVersion = 3 as u8;
        let mut i_0: i32 = 0 as i32;
        while i_0 < 2 as i32 {
            let ch_0: u8 = *data.offset((location - 0x10 as i32 + i_0) as isize);
            if ch_0 as i32 >= 0x20 as i32 && (ch_0 as i32) < 0x7f as i32 {
                (*header).makerCode[i_0 as usize] = ch_0 as ::core::ffi::c_char;
            } else {
                (*header).makerCode[i_0 as usize] = '.' as i32 as ::core::ffi::c_char;
            }
            i_0 += 1;
        }
        (*header).makerCode[2 as i32 as usize] = 0 as ::core::ffi::c_char;
        let mut i_1: i32 = 0 as i32;
        while i_1 < 4 as i32 {
            let ch_1: u8 = *data.offset((location - 0xe as i32 + i_1) as isize);
            if ch_1 as i32 >= 0x20 as i32 && (ch_1 as i32) < 0x7f as i32 {
                (*header).gameCode[i_1 as usize] = ch_1 as ::core::ffi::c_char;
            } else {
                (*header).gameCode[i_1 as usize] = '.' as i32 as ::core::ffi::c_char;
            }
            i_1 += 1;
        }
        (*header).gameCode[4 as i32 as usize] = 0 as ::core::ffi::c_char;
        (*header).flashSize =
            ((0x400 as i32) << *data.offset((location - 4 as i32) as isize) as i32) as u32;
        (*header).exRamSize =
            ((0x400 as i32) << *data.offset((location - 3 as i32) as isize) as i32) as u32;
        (*header).specialVersion = *data.offset((location - 2 as i32) as isize);
        (*header).exCoprocessor = *data.offset((location - 1 as i32) as isize);
    } else if *data.offset((location + 0x14 as i32) as isize) as i32 == 0 as i32 {
        (*header).headerVersion = 2 as u8;
        (*header).exCoprocessor = *data.offset((location - 1 as i32) as isize);
    }
    (*header).pal = (*header).region as i32 >= 0x2 as i32 && (*header).region as i32 <= 0xc as i32
        || (*header).region as i32 == 0x11 as i32;
    (*header).cartType = (if location < 0x9000 as i32 {
        1 as i32
    } else {
        2 as i32
    }) as u8;
    let mut score: i32 = 0 as i32;
    score += if (*header).speed as i32 == 2 as i32 || (*header).speed as i32 == 3 as i32 {
        5 as i32
    } else {
        -(4 as i32)
    };
    score += if (*header).type_0 as i32 <= 3 as i32 || (*header).type_0 as i32 == 5 as i32 {
        5 as i32
    } else {
        -(2 as i32)
    };
    score +=
        if (*header).coprocessor as i32 <= 5 as i32 || (*header).coprocessor as i32 >= 0xe as i32 {
            5 as i32
        } else {
            -(2 as i32)
        };
    score += if (*header).chips as i32 <= 6 as i32
        || (*header).chips as i32 == 9 as i32
        || (*header).chips as i32 == 0xa as i32
    {
        5 as i32
    } else {
        -(2 as i32)
    };
    score += if (*header).region as i32 <= 0x14 as i32 {
        5 as i32
    } else {
        -(2 as i32)
    };
    score += if (*header).checksum as i32 + (*header).checksumComplement as i32 == 0xffff as i32 {
        8 as i32
    } else {
        -(6 as i32)
    };
    let resetVector: u16 = (*data.offset((location + 0x3c as i32) as isize) as i32
        | (*data.offset((location + 0x3d as i32) as isize) as i32) << 8 as i32)
        as u16;
    score += if resetVector as i32 >= 0x8000 as i32 {
        8 as i32
    } else {
        -(20 as i32)
    };
    let opcodeLoc: i32 =
        location + 0x40 as i32 - 0x8000 as i32 + (resetVector as i32 & 0x7fff as i32);
    let mut opcode: u8 = 0xff as u8;
    if opcodeLoc < length {
        opcode = *data.offset(opcodeLoc as isize);
    } else {
        score -= 14 as i32;
    }
    if opcode as i32 == 0x78 as i32 || opcode as i32 == 0x18 as i32 {
        score += 6 as i32;
    }
    if opcode as i32 == 0x4c as i32 || opcode as i32 == 0x5c as i32 || opcode as i32 == 0x9c as i32
    {
        score += 3 as i32;
    }
    if opcode as i32 == 0 as i32 || opcode as i32 == 0xff as i32 || opcode as i32 == 0xdb as i32 {
        score -= 6 as i32;
    }
    (*header).score = score as i16;
}
