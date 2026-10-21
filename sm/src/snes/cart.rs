use crate::c_library::{__assert_fail, free, malloc, memcpy, memset, printf};
use crate::game_state::Game;
use crate::host::Die;
use crate::rtl::words;
use crate::snes::cpu::DumpCpuHistory;
pub use crate::types::{
    size_t, Apu, ApuHistory as C2RustUnnamed, BgLayer, Cart, Cpu, Dma, DmaChannel, Dsp, DspChannel,
    DspRegWriteHistory, Input, Layer, Ppu, PpuPixelPrioBufs, PpuZbufType, SaveLoadFunc, Snes, Spc,
    Timer, WindowLayer,
};
use ::core::ffi::{c_char, c_void};
use ::core::mem::size_of;
use ::core::ptr::null_mut;
pub const NULL: *mut c_void = null_mut::<c_void>();
pub unsafe fn cart_init(snes: *mut Snes) -> *mut Cart {
    let cart = malloc(size_of::<Cart>() as size_t) as *mut Cart;
    crate::set_fields! {[(*cart)]; snes = snes, type_0 = 0, rom = null_mut::<u8>(), romSize = 0,
    ram = null_mut::<u8>(), ramSize = 0, };
    cart
}
pub unsafe fn cart_reset(cart: *mut Cart) {}
pub unsafe fn cart_saveload(
    cart: *mut Cart,
    func: Option<SaveLoadFunc>,
    ctx: *mut c_void,
    game: &mut Game,
) {
    func.expect("non-null function pointer")(
        ctx,
        (*cart).ram as *mut c_void,
        (*cart).ramSize as size_t,
        game,
    );
}
pub unsafe fn cart_load(cart: *mut Cart, type_0: i32, rom: *mut u8, romSize: i32, ramSize: i32) {
    (*cart).type_0 = type_0 as u8;
    if !(*cart).rom.is_null() {
        free((*cart).rom as *mut c_void);
    }
    if !(*cart).ram.is_null() {
        free((*cart).ram as *mut c_void);
    }
    (*cart).rom = malloc(romSize as size_t) as *mut u8;
    (*cart).romSize = romSize as u32;
    if ramSize > 0 {
        (*cart).ram = malloc(ramSize as size_t) as *mut u8;
        memset((*cart).ram as *mut c_void, 0, ramSize as size_t);
    } else {
        (*cart).ram = null_mut::<u8>();
    }
    (*cart).ramSize = ramSize as u32;
    memcpy(
        (*cart).rom as *mut c_void,
        rom as *const c_void,
        romSize as size_t,
    );
}
pub unsafe fn cart_read(cart: *mut Cart, bank: u8, adr: u16, game: &mut Game) -> u8 {
    match (*cart).type_0 {
        0 => {
            crate::native_assertion! {
                '_c2rust_label;
                b"0\0", b"src/snes/cart.c\0", 57,
                b"uint8_t cart_read(Cart *, uint8_t, uint16_t)\0"
            };
            return (*(*cart).snes).openBus;
        }
        1 => return cart_readLorom(cart, bank, adr, game),
        2 => return cart_readHirom(cart, bank, adr),
        _ => {}
    }
    crate::native_assertion! {
        '_c2rust_label_0;
        b"0\0", b"src/snes/cart.c\0", 62, b"uint8_t cart_read(Cart *, uint8_t, uint16_t)\0"
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
        c"While trying to read from 0x%x\n".as_ptr(),
        words!(i32, (bank, adr)),
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
    let mut direct_page = [0; 13];
    for (index, value) in direct_page.iter_mut().enumerate() {
        let address = cpu.dp.wrapping_add(0x44 + index as u16);
        *value = *snes.ram.add(usize::from(address));
    }
    eprintln!("Direct page $44..$50: {direct_page:02X?}");
    DumpCpuHistory(game);
    Die(c"The game crashed in cart_readLorom".as_ptr());
    (*(*cart).snes).openBus
}
unsafe fn cart_writeLorom(cart: *mut Cart, bank: u8, address: u16, value: u8) {
    if (matches!(bank, 0x70..=0x7d) || bank > 0xf0) && address < 0x8000 && (*cart).ramSize > 0 {
        let index =
            ((u32::from(bank & 0xf) << 15) | u32::from(address)) & (*cart).ramSize.wrapping_sub(1);
        *(*cart).ram.offset(index as isize) = value;
    }
}
unsafe fn cart_readHirom(cart: *mut Cart, mut bank: u8, adr: u16) -> u8 {
    bank = (bank & 0x7f);
    if (bank as i32) < 0x40 && adr as i32 >= 0x6000 && (adr as i32) < 0x8000 && (*cart).ramSize > 0
    {
        return *(*cart).ram.offset(
            (((bank as i32 & 0x3f) << 13 | adr as i32 & 0x1fff) as u32
                & (*cart).ramSize.wrapping_sub(1)) as isize,
        );
    }
    if adr as i32 >= 0x8000 || bank as i32 >= 0x40 {
        return *(*cart).rom.offset(
            (((bank as i32 & 0x3f) << 16 | adr as i32) as u32 & (*cart).romSize.wrapping_sub(1))
                as isize,
        );
    }
    crate::native_assertion! {
        '_c2rust_label;
        b"0\0", b"src/snes/cart.c\0", 109, b"uint8_t cart_readHirom(Cart *, uint8_t, uint16_t)\0"
    };
    (*(*cart).snes).openBus
}
unsafe fn cart_writeHirom(cart: *mut Cart, mut bank: u8, adr: u16, val: u8) {
    bank = (bank & 0x7f);
    if (bank as i32) < 0x40 && adr as i32 >= 0x6000 && (adr as i32) < 0x8000 && (*cart).ramSize > 0
    {
        *(*cart).ram.offset(
            (((bank as i32 & 0x3f) << 13 | adr as i32 & 0x1fff) as u32
                & (*cart).ramSize.wrapping_sub(1)) as isize,
        ) = val;
    }
}
