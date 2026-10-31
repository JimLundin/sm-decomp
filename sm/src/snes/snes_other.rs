use crate::c_library::{memcpy, memset, printf};
use crate::game_state::Game;
use crate::snes::cart::cart_load;
pub use crate::types::Snes;
use ::core::ffi::{c_char, c_void};
use ::core::mem::size_of;
crate::typed_struct! {
    #[derive(Copy, Clone)]
    #[repr(C)]
    pub struct CartHeader {
        pub header_version: u8;
        pub name: [c_char; 22];
        pub speed, type_0, coprocessor, chips: u8;
        pub rom_size, ramSize: u32;
        pub region, maker, version: u8;
        pub checksum_complement, checksum: u16;
        pub maker_code: [c_char; 3];
        pub game_code: [c_char; 5];
        pub flash_size, ex_ram_size: u32;
        pub special_version, ex_coprocessor: u8;
        pub score: i16;
        pub pal: bool;
        pub cart_type: u8;
    }
}
pub unsafe fn snes_load_rom(mut data: *const u8, mut length: i32, game: &mut Game) -> bool {
    unsafe {
        if length < 0x8000 {
            printf(
                c"Failed to load rom: rom to small (%d bytes)\n".as_ptr(),
                length,
            );
            return false;
        }
        let mut headers: [CartHeader; 4] = [CartHeader {
            header_version: 0,
            name: [0; 22],
            speed: 0,
            type_0: 0,
            coprocessor: 0,
            chips: 0,
            rom_size: 0,
            ramSize: 0,
            region: 0,
            maker: 0,
            version: 0,
            checksum_complement: 0,
            checksum: 0,
            maker_code: [0; 3],
            game_code: [0; 5],
            flash_size: 0,
            ex_ram_size: 0,
            special_version: 0,
            ex_coprocessor: 0,
            score: 0,
            pal: false,
            cart_type: 0,
        }; 4];
        memset(
            &raw mut headers as *mut CartHeader as *mut c_void,
            0,
            size_of::<[CartHeader; 4]>() as usize,
        );
        for i in 0i32..4 {
            headers[i as usize].score = -50i16;
        }
        if length >= 0x8000 {
            read_header(data, length, 0x7fc0, &raw mut headers as *mut CartHeader);
        }
        if length >= 0x8200 {
            read_header(
                data,
                length,
                0x81c0,
                (&raw mut headers as *mut CartHeader).offset(1),
            );
        }
        if length >= 0x10000 {
            read_header(
                data,
                length,
                0xffc0,
                (&raw mut headers as *mut CartHeader).offset(2),
            );
        }
        if length >= 0x10200 {
            read_header(
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
        if headers[used as usize].cart_type as i32 > 2 {
            printf(
                c"Failed to load rom: unsupported type (%d)\n".as_ptr(),
                headers[used as usize].cart_type as i32,
            );
            return false;
        }
        let mut new_length: i32 = 0x8000;
        while length > new_length {
            new_length *= 2;
        }
        let image = crate::snes::cart::CartRom::zeroed(new_length as usize);
        let new_data = image.as_mut_ptr();
        memcpy(
            new_data as *mut c_void,
            data as *const c_void,
            length as usize,
        );
        let mut test: i32 = 1;
        while length != new_length {
            if length & test != 0 {
                memcpy(
                    new_data.offset(length as isize) as *mut c_void,
                    new_data.offset(length as isize).offset(-(test as isize)) as *const c_void,
                    test as usize,
                );
                length += test;
            }
            test *= 2;
        }
        let mut execution = game.execution();
        cart_load(
            (*execution.backend.views().0).cart,
            headers[used as usize].cart_type as i32,
            image,
            (if headers[used as usize].chips as i32 > 0 {
                headers[used as usize].ramSize
            } else {
                0
            }) as i32,
        );
        execution.reset(true);
        true
    }
}
unsafe fn read_header(data: *const u8, length: i32, location: i32, header: *mut CartHeader) {
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
        (*header).speed = *data.offset((location + 0x15) as isize) >> 4;
        (*header).type_0 = *data.offset((location + 0x15) as isize) & 0xf;
        (*header).coprocessor = *data.offset((location + 0x16) as isize) >> 4;
        (*header).chips = *data.offset((location + 0x16) as isize) & 0xf;
        (*header).rom_size = ((0x400i32) << *data.offset((location + 0x17) as isize) as i32) as u32;
        (*header).ramSize = ((0x400i32) << *data.offset((location + 0x18) as isize) as i32) as u32;
        (*header).region = *data.offset((location + 0x19) as isize);
        (*header).maker = *data.offset((location + 0x1a) as isize);
        (*header).version = *data.offset((location + 0x1b) as isize);
        (*header).checksum_complement = (((*data.offset((location + 0x1d) as isize) as i32) << 8)
            + *data.offset((location + 0x1c) as isize) as i32)
            as u16;
        (*header).checksum = (((*data.offset((location + 0x1f) as isize) as i32) << 8)
            + *data.offset((location + 0x1e) as isize) as i32) as u16;
        (*header).header_version = 1;
        if (*header).maker as i32 == 0x33 {
            (*header).header_version = 3;
            for i_0 in 0i32..2 {
                let ch_0: u8 = *data.offset((location - 0x10i32 + i_0) as isize);
                if ch_0 as i32 >= 0x20 && (ch_0 as i32) < 0x7f {
                    (*header).maker_code[i_0 as usize] = ch_0 as c_char;
                } else {
                    (*header).maker_code[i_0 as usize] = '.' as i32 as c_char;
                }
            }
            (*header).maker_code[2i32 as usize] = 0 as c_char;
            for i_1 in 0i32..4 {
                let ch_1: u8 = *data.offset((location - 0xei32 + i_1) as isize);
                if ch_1 as i32 >= 0x20 && (ch_1 as i32) < 0x7f {
                    (*header).game_code[i_1 as usize] = ch_1 as c_char;
                } else {
                    (*header).game_code[i_1 as usize] = '.' as i32 as c_char;
                }
            }
            (*header).game_code[4i32 as usize] = 0 as c_char;
            (*header).flash_size =
                ((0x400i32) << *data.offset((location - 4i32) as isize) as i32) as u32;
            (*header).ex_ram_size =
                ((0x400i32) << *data.offset((location - 3i32) as isize) as i32) as u32;
            (*header).special_version = *data.offset((location - 2i32) as isize);
            (*header).ex_coprocessor = *data.offset((location - 1i32) as isize);
        } else if *data.offset((location + 0x14) as isize) as i32 == 0 {
            (*header).header_version = 2;
            (*header).ex_coprocessor = *data.offset((location - 1i32) as isize);
        }
        (*header).pal = (*header).region as i32 >= 0x2 && (*header).region as i32 <= 0xc
            || (*header).region as i32 == 0x11;
        (*header).cart_type = (if location < 0x9000 { 1 } else { 2 }) as u8;
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
        score += if (*header).checksum as i32 + (*header).checksum_complement as i32 == 0xffff {
            8
        } else {
            -6i32
        };
        let reset_vector: u16 = (*data.offset((location + 0x3c) as isize) as i32
            | (*data.offset((location + 0x3d) as isize) as i32) << 8)
            as u16;
        score += if reset_vector as i32 >= 0x8000 {
            8
        } else {
            -20i32
        };
        let opcode_loc: i32 = location + 0x40 - 0x8000i32 + (reset_vector as i32 & 0x7fff);
        let mut opcode: u8 = 0xff;
        if opcode_loc < length {
            opcode = *data.offset(opcode_loc as isize);
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
