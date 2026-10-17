use crate::host::Die;
use crate::snes::cpu::DumpCpuHistory;
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
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;

}
pub use crate::types::{
    __int16_t, __int32_t, __int8_t, __uint16_t, __uint32_t, __uint8_t, int16_t, int32_t, int8_t,
    size_t, uint16_t, uint32_t, uint8_t, Apu, ApuHistory as C2RustUnnamed, BgLayer, Cart, Cpu, Dma,
    DmaChannel, Dsp, DspChannel, DspRegWriteHistory, Input, Layer, Ppu, PpuPixelPrioBufs,
    PpuZbufType, SaveLoadFunc, Snes, Spc, Timer, WindowLayer,
};
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub unsafe fn cart_init(mut snes: *mut Snes) -> *mut Cart {
    let mut cart: *mut Cart = malloc(::core::mem::size_of::<Cart>() as size_t) as *mut Cart;
    (*cart).snes = snes;
    (*cart).type_0 = 0 as uint8_t;
    (*cart).rom = ::core::ptr::null_mut::<uint8_t>();
    (*cart).romSize = 0 as uint32_t;
    (*cart).ram = ::core::ptr::null_mut::<uint8_t>();
    (*cart).ramSize = 0 as uint32_t;
    return cart;
}
pub unsafe fn cart_free(mut cart: *mut Cart) {
    free(cart as *mut ::core::ffi::c_void);
}
pub unsafe fn cart_reset(mut cart: *mut Cart) {}
pub unsafe fn cart_saveload(
    mut cart: *mut Cart,
    mut func: Option<SaveLoadFunc>,
    mut ctx: *mut ::core::ffi::c_void,
) {
    func.expect("non-null function pointer")(
        ctx,
        (*cart).ram as *mut ::core::ffi::c_void,
        (*cart).ramSize as size_t,
    );
}
pub unsafe fn cart_load(
    mut cart: *mut Cart,
    mut type_0: ::core::ffi::c_int,
    mut rom: *mut uint8_t,
    mut romSize: ::core::ffi::c_int,
    mut ramSize: ::core::ffi::c_int,
) {
    (*cart).type_0 = type_0 as uint8_t;
    if !(*cart).rom.is_null() {
        free((*cart).rom as *mut ::core::ffi::c_void);
    }
    if !(*cart).ram.is_null() {
        free((*cart).ram as *mut ::core::ffi::c_void);
    }
    (*cart).rom = malloc(romSize as size_t) as *mut uint8_t;
    (*cart).romSize = romSize as uint32_t;
    if ramSize > 0 as ::core::ffi::c_int {
        (*cart).ram = malloc(ramSize as size_t) as *mut uint8_t;
        memset(
            (*cart).ram as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ramSize as size_t,
        );
    } else {
        (*cart).ram = ::core::ptr::null_mut::<uint8_t>();
    }
    (*cart).ramSize = ramSize as uint32_t;
    memcpy(
        (*cart).rom as *mut ::core::ffi::c_void,
        rom as *const ::core::ffi::c_void,
        romSize as size_t,
    );
}
pub unsafe fn cart_read(mut cart: *mut Cart, mut bank: uint8_t, mut adr: uint16_t) -> uint8_t {
    match (*cart).type_0 as ::core::ffi::c_int {
        0 => {
            '_c2rust_label: {
                __assert_fail(
                    b"0\0" as *const u8 as *const ::core::ffi::c_char,
                    b"src/snes/cart.c\0" as *const u8 as *const ::core::ffi::c_char,
                    57 as ::core::ffi::c_uint,
                    b"uint8_t cart_read(Cart *, uint8_t, uint16_t)\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            };
            return (*(*cart).snes).openBus;
        }
        1 => return cart_readLorom(cart, bank, adr),
        2 => return cart_readHirom(cart, bank, adr),
        _ => {}
    }
    '_c2rust_label_0: {
        __assert_fail(
            b"0\0" as *const u8 as *const ::core::ffi::c_char,
            b"src/snes/cart.c\0" as *const u8 as *const ::core::ffi::c_char,
            62 as ::core::ffi::c_uint,
            b"uint8_t cart_read(Cart *, uint8_t, uint16_t)\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    };
    return (*(*cart).snes).openBus;
}
pub unsafe fn cart_write(
    mut cart: *mut Cart,
    mut bank: uint8_t,
    mut adr: uint16_t,
    mut val: uint8_t,
) {
    match (*cart).type_0 as ::core::ffi::c_int {
        1 => {
            cart_writeLorom(cart, bank, adr, val);
        }
        2 => {
            cart_writeHirom(cart, bank, adr, val);
        }
        0 | _ => {}
    };
}
unsafe fn cart_readLorom(mut cart: *mut Cart, mut bank: uint8_t, mut adr: uint16_t) -> uint8_t {
    let original_bank = bank;
    if (bank as ::core::ffi::c_int >= 0x70 as ::core::ffi::c_int
        && (bank as ::core::ffi::c_int) < 0x7e as ::core::ffi::c_int
        || bank as ::core::ffi::c_int >= 0xf0 as ::core::ffi::c_int)
        && (adr as ::core::ffi::c_int) < 0x8000 as ::core::ffi::c_int
        && (*cart).ramSize > 0 as uint32_t
    {
        return *(*cart).ram.offset(
            (((bank as ::core::ffi::c_int & 0xf as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
                | adr as ::core::ffi::c_int) as uint32_t
                & (*cart).ramSize.wrapping_sub(1 as uint32_t)) as isize,
        );
    }
    bank = (bank as ::core::ffi::c_int & 0x7f as ::core::ffi::c_int) as uint8_t;
    if adr as ::core::ffi::c_int >= 0x8000 as ::core::ffi::c_int
        || bank as ::core::ffi::c_int >= 0x40 as ::core::ffi::c_int
    {
        return *(*cart).rom.offset(
            (((bank as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
                | adr as ::core::ffi::c_int & 0x7fff as ::core::ffi::c_int)
                as uint32_t
                & (*cart).romSize.wrapping_sub(1 as uint32_t)) as isize,
        );
    }
    printf(
        b"While trying to read from 0x%x\n\0" as *const u8 as *const ::core::ffi::c_char,
        (bank as ::core::ffi::c_int) << 16 as ::core::ffi::c_int | adr as ::core::ffi::c_int,
    );
    let snes = &*(*cart).snes;
    let cpu = &*snes.cpu;
    eprintln!(
        "Invalid LoROM read ${original_bank:02X}:{adr:04X}, frame {}: PC ${:02X}:{:04X} A ${:04X} X ${:04X} Y ${:04X} SP ${:04X} DP ${:04X} DB ${:02X}",
        crate::snes::snes::snes_frame_counter,
        cpu.k, cpu.pc, cpu.a, cpu.x, cpu.y, cpu.sp, cpu.dp, cpu.db,
    );
    eprintln!(
        "CPU flags: N={} V={} M={} X={} D={} I={} Z={} C={} E={}",
        cpu.n, cpu.v, cpu.mf, cpu.xf, cpu.d, cpu.i, cpu.z, cpu.c, cpu.e,
    );
    let mut direct_page = [0u8; 13];
    for (index, value) in direct_page.iter_mut().enumerate() {
        let address = cpu.dp.wrapping_add(0x44 + index as u16);
        *value = *snes.ram.add(usize::from(address));
    }
    eprintln!("Direct page $44..$50: {direct_page:02X?}");
    DumpCpuHistory();
    Die(b"The game crashed in cart_readLorom\0" as *const u8 as *const ::core::ffi::c_char);
    return (*(*cart).snes).openBus;
}
unsafe fn cart_writeLorom(
    mut cart: *mut Cart,
    mut bank: uint8_t,
    mut adr: uint16_t,
    mut val: uint8_t,
) {
    if (bank as ::core::ffi::c_int >= 0x70 as ::core::ffi::c_int
        && (bank as ::core::ffi::c_int) < 0x7e as ::core::ffi::c_int
        || bank as ::core::ffi::c_int > 0xf0 as ::core::ffi::c_int)
        && (adr as ::core::ffi::c_int) < 0x8000 as ::core::ffi::c_int
        && (*cart).ramSize > 0 as uint32_t
    {
        *(*cart).ram.offset(
            (((bank as ::core::ffi::c_int & 0xf as ::core::ffi::c_int) << 15 as ::core::ffi::c_int
                | adr as ::core::ffi::c_int) as uint32_t
                & (*cart).ramSize.wrapping_sub(1 as uint32_t)) as isize,
        ) = val;
    }
}
unsafe fn cart_readHirom(mut cart: *mut Cart, mut bank: uint8_t, mut adr: uint16_t) -> uint8_t {
    bank = (bank as ::core::ffi::c_int & 0x7f as ::core::ffi::c_int) as uint8_t;
    if (bank as ::core::ffi::c_int) < 0x40 as ::core::ffi::c_int
        && adr as ::core::ffi::c_int >= 0x6000 as ::core::ffi::c_int
        && (adr as ::core::ffi::c_int) < 0x8000 as ::core::ffi::c_int
        && (*cart).ramSize > 0 as uint32_t
    {
        return *(*cart).ram.offset(
            (((bank as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int) << 13 as ::core::ffi::c_int
                | adr as ::core::ffi::c_int & 0x1fff as ::core::ffi::c_int)
                as uint32_t
                & (*cart).ramSize.wrapping_sub(1 as uint32_t)) as isize,
        );
    }
    if adr as ::core::ffi::c_int >= 0x8000 as ::core::ffi::c_int
        || bank as ::core::ffi::c_int >= 0x40 as ::core::ffi::c_int
    {
        return *(*cart).rom.offset(
            (((bank as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int) << 16 as ::core::ffi::c_int
                | adr as ::core::ffi::c_int) as uint32_t
                & (*cart).romSize.wrapping_sub(1 as uint32_t)) as isize,
        );
    }
    '_c2rust_label: {
        __assert_fail(
            b"0\0" as *const u8 as *const ::core::ffi::c_char,
            b"src/snes/cart.c\0" as *const u8 as *const ::core::ffi::c_char,
            109 as ::core::ffi::c_uint,
            b"uint8_t cart_readHirom(Cart *, uint8_t, uint16_t)\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    };
    return (*(*cart).snes).openBus;
}
unsafe fn cart_writeHirom(
    mut cart: *mut Cart,
    mut bank: uint8_t,
    mut adr: uint16_t,
    mut val: uint8_t,
) {
    bank = (bank as ::core::ffi::c_int & 0x7f as ::core::ffi::c_int) as uint8_t;
    if (bank as ::core::ffi::c_int) < 0x40 as ::core::ffi::c_int
        && adr as ::core::ffi::c_int >= 0x6000 as ::core::ffi::c_int
        && (adr as ::core::ffi::c_int) < 0x8000 as ::core::ffi::c_int
        && (*cart).ramSize > 0 as uint32_t
    {
        *(*cart).ram.offset(
            (((bank as ::core::ffi::c_int & 0x3f as ::core::ffi::c_int) << 13 as ::core::ffi::c_int
                | adr as ::core::ffi::c_int & 0x1fff as ::core::ffi::c_int)
                as uint32_t
                & (*cart).ramSize.wrapping_sub(1 as uint32_t)) as isize,
        ) = val;
    }
}
