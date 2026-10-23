use crate::c_library::{free, malloc, memcpy, memset, printf};
use crate::game_state::Game;
use crate::snes::cart::cart_load;
use crate::snes::snes::snes_reset;
pub use crate::types::{Snes, size_t};
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
    unsafe {
        if length < 0x8000 {
            printf(
                c"Failed to load rom: rom to small (%d bytes)\n".as_ptr(),
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
            0,
            size_of::<[CartHeader; 4]>() as size_t,
        );
        for i in 0i32..4 {
            headers[i as usize].score = -50i16;
        }
        if length >= 0x8000 {
            readHeader(data, length, 0x7fc0, (&raw mut headers as *mut CartHeader));
        }
        if length >= 0x8200 {
            readHeader(
                data,
                length,
                0x81c0,
                (&raw mut headers as *mut CartHeader).offset(1),
            );
        }
        if length >= 0x10000 {
            readHeader(
                data,
                length,
                0xffc0,
                (&raw mut headers as *mut CartHeader).offset(2),
            );
        }
        if length >= 0x10200 {
            readHeader(
                data,
                length,
                0x101c0,
                (&raw mut headers as *mut CartHeader).offset(3),
            );
        }
        let mut max: i32 = 0;
        let mut used: i32 = 0;
        for i_0 in 0i32..4 {
            if headers[i_0 as usize].score as i32 > max {
                max = headers[i_0 as usize].score as i32;
                used = i_0;
            }
        }
        if used & 1 != 0 {
            data = data.offset(0x200);
            length -= 0x200;
        }
        if headers[used as usize].cartType as i32 > 2 {
            printf(
                c"Failed to load rom: unsupported type (%d)\n".as_ptr(),
                headers[used as usize].cartType as i32,
            );
            return false_0 != 0;
        }
        let mut newLength: i32 = 0x8000;
        while (length > newLength) {
            newLength *= 2;
        }
        let newData = malloc(newLength as size_t) as *mut u8;
        memcpy(
            newData as *mut c_void,
            data as *const c_void,
            length as size_t,
        );
        let mut test: i32 = 1;
        while length != newLength {
            if length & test != 0 {
                memcpy(
                    newData.offset(length as isize) as *mut c_void,
                    newData.offset(length as isize).offset(-(test as isize)) as *const c_void,
                    test as size_t,
                );
                length += test;
            }
            test *= 2;
        }
        cart_load(
            (*snes).cart,
            headers[used as usize].cartType as i32,
            newData,
            newLength,
            (if headers[used as usize].chips as i32 > 0 {
                headers[used as usize].ramSize
            } else {
                0
            }) as i32,
        );
        snes_reset(snes, true_0 != 0, game);
        free(newData as *mut c_void);
        true_0 != 0
    }
}
unsafe fn readHeader(data: *const u8, length: i32, location: i32, header: *mut CartHeader) {
    unsafe {
        for i in 0i32..21 {
            let ch: u8 = *data.offset((location + i) as isize);
            if ch as i32 >= 0x20 && (ch as i32) < 0x7f {
                (*header).name[i as usize] = ch as c_char;
            } else {
                (*header).name[i as usize] = '.' as i32 as c_char;
            }
        }
        (*header).name[21i32 as usize] = 0 as c_char;
        (*header).speed = (*data.offset((location + 0x15) as isize) >> 4);
        (*header).type_0 = (*data.offset((location + 0x15) as isize) & 0xf);
        (*header).coprocessor = (*data.offset((location + 0x16) as isize) >> 4);
        (*header).chips = (*data.offset((location + 0x16) as isize) & 0xf);
        (*header).romSize = ((0x400i32) << *data.offset((location + 0x17) as isize) as i32) as u32;
        (*header).ramSize = ((0x400i32) << *data.offset((location + 0x18) as isize) as i32) as u32;
        (*header).region = *data.offset((location + 0x19) as isize);
        (*header).maker = *data.offset((location + 0x1a) as isize);
        (*header).version = *data.offset((location + 0x1b) as isize);
        (*header).checksumComplement = (((*data.offset((location + 0x1d) as isize) as i32) << 8)
            + *data.offset((location + 0x1c) as isize) as i32)
            as u16;
        (*header).checksum = (((*data.offset((location + 0x1f) as isize) as i32) << 8)
            + *data.offset((location + 0x1e) as isize) as i32) as u16;
        (*header).headerVersion = 1;
        if (*header).maker as i32 == 0x33 {
            (*header).headerVersion = 3;
            for i_0 in 0i32..2 {
                let ch_0: u8 = *data.offset((location - 0x10i32 + i_0) as isize);
                if ch_0 as i32 >= 0x20 && (ch_0 as i32) < 0x7f {
                    (*header).makerCode[i_0 as usize] = ch_0 as c_char;
                } else {
                    (*header).makerCode[i_0 as usize] = '.' as i32 as c_char;
                }
            }
            (*header).makerCode[2i32 as usize] = 0 as c_char;
            for i_1 in 0i32..4 {
                let ch_1: u8 = *data.offset((location - 0xei32 + i_1) as isize);
                if ch_1 as i32 >= 0x20 && (ch_1 as i32) < 0x7f {
                    (*header).gameCode[i_1 as usize] = ch_1 as c_char;
                } else {
                    (*header).gameCode[i_1 as usize] = '.' as i32 as c_char;
                }
            }
            (*header).gameCode[4i32 as usize] = 0 as c_char;
            (*header).flashSize =
                ((0x400i32) << *data.offset((location - 4i32) as isize) as i32) as u32;
            (*header).exRamSize =
                ((0x400i32) << *data.offset((location - 3i32) as isize) as i32) as u32;
            (*header).specialVersion = *data.offset((location - 2i32) as isize);
            (*header).exCoprocessor = *data.offset((location - 1i32) as isize);
        } else if *data.offset((location + 0x14) as isize) as i32 == 0 {
            (*header).headerVersion = 2;
            (*header).exCoprocessor = *data.offset((location - 1i32) as isize);
        }
        (*header).pal = (*header).region as i32 >= 0x2 && (*header).region as i32 <= 0xc
            || (*header).region as i32 == 0x11;
        (*header).cartType = (if location < 0x9000 { 1 } else { 2 }) as u8;
        let mut score: i32 = 0;
        score += if (*header).speed as i32 == 2 || (*header).speed as i32 == 3 {
            5
        } else {
            -4i32
        };
        score += if (*header).type_0 as i32 <= 3 || (*header).type_0 as i32 == 5 {
            5
        } else {
            -2i32
        };
        score += if (*header).coprocessor as i32 <= 5 || (*header).coprocessor as i32 >= 0xe {
            5
        } else {
            -2i32
        };
        score += if (*header).chips as i32 <= 6
            || (*header).chips as i32 == 9
            || (*header).chips as i32 == 0xa
        {
            5
        } else {
            -2i32
        };
        score += if (*header).region as i32 <= 0x14 {
            5
        } else {
            -2i32
        };
        score += if (*header).checksum as i32 + (*header).checksumComplement as i32 == 0xffff {
            8
        } else {
            -6i32
        };
        let resetVector: u16 = (*data.offset((location + 0x3c) as isize) as i32
            | (*data.offset((location + 0x3d) as isize) as i32) << 8)
            as u16;
        score += if resetVector as i32 >= 0x8000 {
            8
        } else {
            -20i32
        };
        let opcodeLoc: i32 = location + 0x40 - 0x8000i32 + (resetVector as i32 & 0x7fff);
        let mut opcode: u8 = 0xff;
        if opcodeLoc < length {
            opcode = *data.offset(opcodeLoc as isize);
        } else {
            score -= 14;
        }
        if opcode as i32 == 0x78 || opcode as i32 == 0x18 {
            score += 6;
        }
        if opcode as i32 == 0x4c || opcode as i32 == 0x5c || opcode as i32 == 0x9c {
            score += 3;
        }
        if opcode as i32 == 0 || opcode as i32 == 0xff || opcode as i32 == 0xdb {
            score -= 6;
        }
        (*header).score = score as i16;
    }
}
