use crate::game_state::Game;
use crate::host::Die;
use crate::snes::cpu::DumpCpuHistory;
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
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: u32,
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
pub unsafe fn cart_init(snes: *mut Snes) -> *mut Cart {
    let cart: *mut Cart = malloc(::core::mem::size_of::<Cart>() as size_t) as *mut Cart;
    (*cart).snes = snes;
    (*cart).type_0 = 0 as uint8_t;
    (*cart).rom = ::core::ptr::null_mut::<uint8_t>();
    (*cart).romSize = 0 as uint32_t;
    (*cart).ram = ::core::ptr::null_mut::<uint8_t>();
    (*cart).ramSize = 0 as uint32_t;
    cart
}
pub unsafe fn cart_free(cart: *mut Cart) {
    free(cart as *mut ::core::ffi::c_void);
}
pub unsafe fn cart_reset(cart: *mut Cart) {}
pub unsafe fn cart_saveload(
    cart: *mut Cart,
    func: Option<SaveLoadFunc>,
    ctx: *mut ::core::ffi::c_void,
    game: &mut Game,
) {
    func.expect("non-null function pointer")(
        ctx,
        (*cart).ram as *mut ::core::ffi::c_void,
        (*cart).ramSize as size_t,
        game,
    );
}
pub unsafe fn cart_load(
    cart: *mut Cart,
    type_0: i32,
    rom: *mut uint8_t,
    romSize: i32,
    ramSize: i32,
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
    if ramSize > 0 as i32 {
        (*cart).ram = malloc(ramSize as size_t) as *mut uint8_t;
        memset(
            (*cart).ram as *mut ::core::ffi::c_void,
            0 as i32,
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
pub unsafe fn cart_read(cart: *mut Cart, bank: u8, adr: u16, game: &mut Game) -> u8 {
    match (*cart).type_0 {
        0 => {
            '_c2rust_label: {
                __assert_fail(
                    b"0\0" as *const u8 as *const ::core::ffi::c_char,
                    b"src/snes/cart.c\0" as *const u8 as *const ::core::ffi::c_char,
                    57 as u32,
                    b"uint8_t cart_read(Cart *, uint8_t, uint16_t)\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            };
            return (*(*cart).snes).openBus;
        }
        1 => return cart_readLorom(cart, bank, adr, game),
        2 => return cart_readHirom(cart, bank, adr),
        _ => {}
    }
    '_c2rust_label_0: {
        __assert_fail(
            b"0\0" as *const u8 as *const ::core::ffi::c_char,
            b"src/snes/cart.c\0" as *const u8 as *const ::core::ffi::c_char,
            62 as u32,
            b"uint8_t cart_read(Cart *, uint8_t, uint16_t)\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    };
    (*(*cart).snes).openBus
}
pub unsafe fn cart_write(cart: *mut Cart, bank: u8, adr: u16, val: u8) {
    match (*cart).type_0 {
        1 => cart_writeLorom(cart, bank, adr, val),
        2 => cart_writeHirom(cart, bank, adr, val),
        _ => {}
    };
}
unsafe fn cart_readLorom(cart: *mut Cart, mut bank: u8, adr: u16, game: &mut Game) -> u8 {
    let original_bank = bank;
    if (matches!(bank, 0x70..=0x7d) || bank >= 0xf0) && adr < 0x8000 && (*cart).ramSize > 0 {
        let index =
            ((u32::from(bank & 0xf) << 15) | u32::from(adr)) & (*cart).ramSize.wrapping_sub(1);
        return *(*cart).ram.offset(index as isize);
    }
    bank &= 0x7f;
    if adr >= 0x8000 || bank >= 0x40 {
        let index =
            ((u32::from(bank) << 15) | u32::from(adr & 0x7fff)) & (*cart).romSize.wrapping_sub(1);
        return *(*cart).rom.offset(index as isize);
    }
    printf(
        b"While trying to read from 0x%x\n\0" as *const u8 as *const ::core::ffi::c_char,
        (bank as i32) << 16 as i32 | adr as i32,
    );
    let snes = &*(*cart).snes;
    let cpu = &*snes.cpu;
    eprintln!(
        "Invalid LoROM read ${original_bank:02X}:{adr:04X}, frame {}: PC ${:02X}:{:04X} A ${:04X} X ${:04X} Y ${:04X} SP ${:04X} DP ${:04X} DB ${:02X}",
        game.snes_frame_counter,
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
    DumpCpuHistory(game);
    Die(b"The game crashed in cart_readLorom\0" as *const u8 as *const ::core::ffi::c_char);
    (*(*cart).snes).openBus
}
unsafe fn cart_writeLorom(cart: *mut Cart, bank: u8, address: u16, value: u8) {
    if (matches!(bank, 0x70..=0x7d) || bank > 0xf0) && address < 0x8000 && (*cart).ramSize > 0 {
        let index =
            ((u32::from(bank & 0xf) << 15) | u32::from(address)) & (*cart).ramSize.wrapping_sub(1);
        *(*cart).ram.offset(index as isize) = value;
    }
}
unsafe fn cart_readHirom(cart: *mut Cart, mut bank: uint8_t, adr: uint16_t) -> uint8_t {
    bank = (bank as i32 & 0x7f as i32) as uint8_t;
    if (bank as i32) < 0x40 as i32
        && adr as i32 >= 0x6000 as i32
        && (adr as i32) < 0x8000 as i32
        && (*cart).ramSize > 0 as uint32_t
    {
        return *(*cart).ram.offset(
            (((bank as i32 & 0x3f as i32) << 13 as i32 | adr as i32 & 0x1fff as i32) as uint32_t
                & (*cart).ramSize.wrapping_sub(1 as uint32_t)) as isize,
        );
    }
    if adr as i32 >= 0x8000 as i32 || bank as i32 >= 0x40 as i32 {
        return *(*cart).rom.offset(
            (((bank as i32 & 0x3f as i32) << 16 as i32 | adr as i32) as uint32_t
                & (*cart).romSize.wrapping_sub(1 as uint32_t)) as isize,
        );
    }
    '_c2rust_label: {
        __assert_fail(
            b"0\0" as *const u8 as *const ::core::ffi::c_char,
            b"src/snes/cart.c\0" as *const u8 as *const ::core::ffi::c_char,
            109 as u32,
            b"uint8_t cart_readHirom(Cart *, uint8_t, uint16_t)\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    };
    (*(*cart).snes).openBus
}
unsafe fn cart_writeHirom(cart: *mut Cart, mut bank: uint8_t, adr: uint16_t, val: uint8_t) {
    bank = (bank as i32 & 0x7f as i32) as uint8_t;
    if (bank as i32) < 0x40 as i32
        && adr as i32 >= 0x6000 as i32
        && (adr as i32) < 0x8000 as i32
        && (*cart).ramSize > 0 as uint32_t
    {
        *(*cart).ram.offset(
            (((bank as i32 & 0x3f as i32) << 13 as i32 | adr as i32 & 0x1fff as i32) as uint32_t
                & (*cart).ramSize.wrapping_sub(1 as uint32_t)) as isize,
        ) = val;
    }
}
