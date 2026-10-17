use crate::snes::cart::cart_load;
use crate::snes::dsp::dsp_getSamples;
use crate::snes::snes::snes_reset;
extern "C" {
    fn printf(__format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
    fn malloc(__size: size_t) -> *mut ::core::ffi::c_void;
    fn free(__ptr: *mut ::core::ffi::c_void);
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn memset(
        __s: *mut ::core::ffi::c_void,
        __c: ::core::ffi::c_int,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;

}
pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16_t, int32_t, int8_t,
    size_t, uint16_t, uint32_t, uint8_t, Apu, ApuHistory as C2RustUnnamed, BgLayer, Cart, Cpu, Dma,
    DmaChannel, Dsp, DspChannel, DspRegWriteHistory, Input, Layer, Ppu, PpuPixelPrioBufs,
    PpuZbufType, Snes, Spc, Timer, WindowLayer,
};
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CartHeader {
    pub headerVersion: uint8_t,
    pub name: [::core::ffi::c_char; 22],
    pub speed: uint8_t,
    pub type_0: uint8_t,
    pub coprocessor: uint8_t,
    pub chips: uint8_t,
    pub romSize: uint32_t,
    pub ramSize: uint32_t,
    pub region: uint8_t,
    pub maker: uint8_t,
    pub version: uint8_t,
    pub checksumComplement: uint16_t,
    pub checksum: uint16_t,
    pub makerCode: [::core::ffi::c_char; 3],
    pub gameCode: [::core::ffi::c_char; 5],
    pub flashSize: uint32_t,
    pub exRamSize: uint32_t,
    pub specialVersion: uint8_t,
    pub exCoprocessor: uint8_t,
    pub score: int16_t,
    pub pal: bool,
    pub cartType: uint8_t,
}
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
#[no_mangle]
pub unsafe extern "C" fn snes_loadRom(
    mut snes: *mut Snes,
    mut data: *const uint8_t,
    mut length: ::core::ffi::c_int,
) -> bool {
    if length < 0x8000 as ::core::ffi::c_int {
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
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<[CartHeader; 4]>() as size_t,
    );
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 4 as ::core::ffi::c_int {
        headers[i as usize].score = -(50 as ::core::ffi::c_int) as int16_t;
        i += 1;
    }
    if length >= 0x8000 as ::core::ffi::c_int {
        readHeader(
            data,
            length,
            0x7fc0 as ::core::ffi::c_int,
            (&raw mut headers as *mut CartHeader).offset(0 as ::core::ffi::c_int as isize)
                as *mut CartHeader,
        );
    }
    if length >= 0x8200 as ::core::ffi::c_int {
        readHeader(
            data,
            length,
            0x81c0 as ::core::ffi::c_int,
            (&raw mut headers as *mut CartHeader).offset(1 as ::core::ffi::c_int as isize)
                as *mut CartHeader,
        );
    }
    if length >= 0x10000 as ::core::ffi::c_int {
        readHeader(
            data,
            length,
            0xffc0 as ::core::ffi::c_int,
            (&raw mut headers as *mut CartHeader).offset(2 as ::core::ffi::c_int as isize)
                as *mut CartHeader,
        );
    }
    if length >= 0x10200 as ::core::ffi::c_int {
        readHeader(
            data,
            length,
            0x101c0 as ::core::ffi::c_int,
            (&raw mut headers as *mut CartHeader).offset(3 as ::core::ffi::c_int as isize)
                as *mut CartHeader,
        );
    }
    let mut max: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut used: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_0 < 4 as ::core::ffi::c_int {
        if headers[i_0 as usize].score as ::core::ffi::c_int > max {
            max = headers[i_0 as usize].score as ::core::ffi::c_int;
            used = i_0;
        }
        i_0 += 1;
    }
    if used & 1 as ::core::ffi::c_int != 0 {
        data = data.offset(0x200 as ::core::ffi::c_int as isize);
        length -= 0x200 as ::core::ffi::c_int;
    }
    if headers[used as usize].cartType as ::core::ffi::c_int > 2 as ::core::ffi::c_int {
        printf(
            b"Failed to load rom: unsupported type (%d)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            headers[used as usize].cartType as ::core::ffi::c_int,
        );
        return false_0 != 0;
    }
    let mut newLength: ::core::ffi::c_int = 0x8000 as ::core::ffi::c_int;
    while !(length <= newLength) {
        newLength *= 2 as ::core::ffi::c_int;
    }
    let mut newData: *mut uint8_t = malloc(newLength as size_t) as *mut uint8_t;
    memcpy(
        newData as *mut ::core::ffi::c_void,
        data as *const ::core::ffi::c_void,
        length as size_t,
    );
    let mut test: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
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
        test *= 2 as ::core::ffi::c_int;
    }
    cart_load(
        (*snes).cart,
        headers[used as usize].cartType as ::core::ffi::c_int,
        newData,
        newLength,
        (if headers[used as usize].chips as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
            headers[used as usize].ramSize
        } else {
            0 as uint32_t
        }) as ::core::ffi::c_int,
    );
    snes_reset(snes, true_0 != 0);
    free(newData as *mut ::core::ffi::c_void);
    return true_0 != 0;
}
#[no_mangle]
pub unsafe extern "C" fn snes_setSamples(
    mut snes: *mut Snes,
    mut sampleData: *mut int16_t,
    mut samplesPerFrame: ::core::ffi::c_int,
) {
    dsp_getSamples((*(*snes).apu).dsp, sampleData, samplesPerFrame);
}
unsafe extern "C" fn readHeader(
    mut data: *const uint8_t,
    mut length: ::core::ffi::c_int,
    mut location: ::core::ffi::c_int,
    mut header: *mut CartHeader,
) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < 21 as ::core::ffi::c_int {
        let mut ch: uint8_t = *data.offset((location + i) as isize);
        if ch as ::core::ffi::c_int >= 0x20 as ::core::ffi::c_int
            && (ch as ::core::ffi::c_int) < 0x7f as ::core::ffi::c_int
        {
            (*header).name[i as usize] = ch as ::core::ffi::c_char;
        } else {
            (*header).name[i as usize] = '.' as i32 as ::core::ffi::c_char;
        }
        i += 1;
    }
    (*header).name[21 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_char;
    (*header).speed = (*data.offset((location + 0x15 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int) as uint8_t;
    (*header).type_0 = (*data.offset((location + 0x15 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int) as uint8_t;
    (*header).coprocessor = (*data.offset((location + 0x16 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        >> 4 as ::core::ffi::c_int) as uint8_t;
    (*header).chips = (*data.offset((location + 0x16 as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        & 0xf as ::core::ffi::c_int) as uint8_t;
    (*header).romSize = ((0x400 as ::core::ffi::c_int)
        << *data.offset((location + 0x17 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int)
        as uint32_t;
    (*header).ramSize = ((0x400 as ::core::ffi::c_int)
        << *data.offset((location + 0x18 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int)
        as uint32_t;
    (*header).region = *data.offset((location + 0x19 as ::core::ffi::c_int) as isize);
    (*header).maker = *data.offset((location + 0x1a as ::core::ffi::c_int) as isize);
    (*header).version = *data.offset((location + 0x1b as ::core::ffi::c_int) as isize);
    (*header).checksumComplement = (((*data.offset((location + 0x1d as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int)
        + *data.offset((location + 0x1c as ::core::ffi::c_int) as isize) as ::core::ffi::c_int)
        as uint16_t;
    (*header).checksum = (((*data.offset((location + 0x1f as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int)
        << 8 as ::core::ffi::c_int)
        + *data.offset((location + 0x1e as ::core::ffi::c_int) as isize) as ::core::ffi::c_int)
        as uint16_t;
    (*header).headerVersion = 1 as uint8_t;
    if (*header).maker as ::core::ffi::c_int == 0x33 as ::core::ffi::c_int {
        (*header).headerVersion = 3 as uint8_t;
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < 2 as ::core::ffi::c_int {
            let mut ch_0: uint8_t =
                *data.offset((location - 0x10 as ::core::ffi::c_int + i_0) as isize);
            if ch_0 as ::core::ffi::c_int >= 0x20 as ::core::ffi::c_int
                && (ch_0 as ::core::ffi::c_int) < 0x7f as ::core::ffi::c_int
            {
                (*header).makerCode[i_0 as usize] = ch_0 as ::core::ffi::c_char;
            } else {
                (*header).makerCode[i_0 as usize] = '.' as i32 as ::core::ffi::c_char;
            }
            i_0 += 1;
        }
        (*header).makerCode[2 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_char;
        let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_1 < 4 as ::core::ffi::c_int {
            let mut ch_1: uint8_t =
                *data.offset((location - 0xe as ::core::ffi::c_int + i_1) as isize);
            if ch_1 as ::core::ffi::c_int >= 0x20 as ::core::ffi::c_int
                && (ch_1 as ::core::ffi::c_int) < 0x7f as ::core::ffi::c_int
            {
                (*header).gameCode[i_1 as usize] = ch_1 as ::core::ffi::c_char;
            } else {
                (*header).gameCode[i_1 as usize] = '.' as i32 as ::core::ffi::c_char;
            }
            i_1 += 1;
        }
        (*header).gameCode[4 as ::core::ffi::c_int as usize] = 0 as ::core::ffi::c_char;
        (*header).flashSize = ((0x400 as ::core::ffi::c_int)
            << *data.offset((location - 4 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int)
            as uint32_t;
        (*header).exRamSize = ((0x400 as ::core::ffi::c_int)
            << *data.offset((location - 3 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int)
            as uint32_t;
        (*header).specialVersion = *data.offset((location - 2 as ::core::ffi::c_int) as isize);
        (*header).exCoprocessor = *data.offset((location - 1 as ::core::ffi::c_int) as isize);
    } else if *data.offset((location + 0x14 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
        == 0 as ::core::ffi::c_int
    {
        (*header).headerVersion = 2 as uint8_t;
        (*header).exCoprocessor = *data.offset((location - 1 as ::core::ffi::c_int) as isize);
    }
    (*header).pal = (*header).region as ::core::ffi::c_int >= 0x2 as ::core::ffi::c_int
        && (*header).region as ::core::ffi::c_int <= 0xc as ::core::ffi::c_int
        || (*header).region as ::core::ffi::c_int == 0x11 as ::core::ffi::c_int;
    (*header).cartType = (if location < 0x9000 as ::core::ffi::c_int {
        1 as ::core::ffi::c_int
    } else {
        2 as ::core::ffi::c_int
    }) as uint8_t;
    let mut score: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    score += if (*header).speed as ::core::ffi::c_int == 2 as ::core::ffi::c_int
        || (*header).speed as ::core::ffi::c_int == 3 as ::core::ffi::c_int
    {
        5 as ::core::ffi::c_int
    } else {
        -(4 as ::core::ffi::c_int)
    };
    score += if (*header).type_0 as ::core::ffi::c_int <= 3 as ::core::ffi::c_int
        || (*header).type_0 as ::core::ffi::c_int == 5 as ::core::ffi::c_int
    {
        5 as ::core::ffi::c_int
    } else {
        -(2 as ::core::ffi::c_int)
    };
    score += if (*header).coprocessor as ::core::ffi::c_int <= 5 as ::core::ffi::c_int
        || (*header).coprocessor as ::core::ffi::c_int >= 0xe as ::core::ffi::c_int
    {
        5 as ::core::ffi::c_int
    } else {
        -(2 as ::core::ffi::c_int)
    };
    score += if (*header).chips as ::core::ffi::c_int <= 6 as ::core::ffi::c_int
        || (*header).chips as ::core::ffi::c_int == 9 as ::core::ffi::c_int
        || (*header).chips as ::core::ffi::c_int == 0xa as ::core::ffi::c_int
    {
        5 as ::core::ffi::c_int
    } else {
        -(2 as ::core::ffi::c_int)
    };
    score += if (*header).region as ::core::ffi::c_int <= 0x14 as ::core::ffi::c_int {
        5 as ::core::ffi::c_int
    } else {
        -(2 as ::core::ffi::c_int)
    };
    score += if (*header).checksum as ::core::ffi::c_int
        + (*header).checksumComplement as ::core::ffi::c_int
        == 0xffff as ::core::ffi::c_int
    {
        8 as ::core::ffi::c_int
    } else {
        -(6 as ::core::ffi::c_int)
    };
    let mut resetVector: uint16_t = (*data.offset((location + 0x3c as ::core::ffi::c_int) as isize)
        as ::core::ffi::c_int
        | (*data.offset((location + 0x3d as ::core::ffi::c_int) as isize) as ::core::ffi::c_int)
            << 8 as ::core::ffi::c_int) as uint16_t;
    score += if resetVector as ::core::ffi::c_int >= 0x8000 as ::core::ffi::c_int {
        8 as ::core::ffi::c_int
    } else {
        -(20 as ::core::ffi::c_int)
    };
    let mut opcodeLoc: ::core::ffi::c_int = location + 0x40 as ::core::ffi::c_int
        - 0x8000 as ::core::ffi::c_int
        + (resetVector as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int);
    let mut opcode: uint8_t = 0xff as uint8_t;
    if opcodeLoc < length {
        opcode = *data.offset(opcodeLoc as isize);
    } else {
        score -= 14 as ::core::ffi::c_int;
    }
    if opcode as ::core::ffi::c_int == 0x78 as ::core::ffi::c_int
        || opcode as ::core::ffi::c_int == 0x18 as ::core::ffi::c_int
    {
        score += 6 as ::core::ffi::c_int;
    }
    if opcode as ::core::ffi::c_int == 0x4c as ::core::ffi::c_int
        || opcode as ::core::ffi::c_int == 0x5c as ::core::ffi::c_int
        || opcode as ::core::ffi::c_int == 0x9c as ::core::ffi::c_int
    {
        score += 3 as ::core::ffi::c_int;
    }
    if opcode as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        || opcode as ::core::ffi::c_int == 0xff as ::core::ffi::c_int
        || opcode as ::core::ffi::c_int == 0xdb as ::core::ffi::c_int
    {
        score -= 6 as ::core::ffi::c_int;
    }
    (*header).score = score as int16_t;
}
