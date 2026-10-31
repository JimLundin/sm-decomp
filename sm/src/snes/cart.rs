use crate::c_library::printf;
use crate::execution::{DmaRomSource, Execution};
use crate::host::Die;
use crate::rom::RomImage;
use crate::rtl::words;
pub use crate::types::{Cart, Snes};
use ::core::ffi::c_void;
use ::core::ptr::null_mut;
pub const NULL: *mut c_void = null_mut::<c_void>();
pub unsafe fn cart_reset(_cart: *mut Cart) {}
// The variable cartridge extent remains an external count, with no wire prefix.
macro_rules! cart_snapshot {
    ($cart:ident, $io:ident; $size:ident : $ty:ty) => {
        #[cfg(any(feature = "frontend", sm_verify_dll))]
        pub(crate) fn snapshot_layout(layout: &mut String) {
            let _: fn(&Cart) -> &$ty = |cart| &cart.$size;
            layout.push_str(concat!(
                "cart-ram{external-count:cart.",
                stringify!($size),
                ":",
                stringify!($ty),
                ";bytes:count*u8;no-count-prefix;}"
            ));
        }
        pub unsafe fn cart_saveload(
            $cart: *mut Cart,
            $io: &mut crate::snapshot_io::SnapshotIo<'_>,
        ) {
            unsafe {
                let ram = &(*$cart).ram;
                let mut bytes = ram.bytes(0..(*$cart).$size as usize);
                crate::rtl::snapshot_result($io.transfer(&mut bytes));
                ram.write_bytes(0, &bytes);
            }
        }
    };
}
cart_snapshot! { cart, io; ramSize: u32 }
// Adopt the actual prepared allocation; cartridge and native views share its lifetime.
pub unsafe fn cart_load(cart: *mut Cart, type_0: i32, rom: RomImage, ramSize: i32) {
    unsafe {
        (*cart).type_0 = type_0 as u8;
        (*cart).rom = RomImage::default();
        (*cart).ram = crate::save_ram::SaveRam::default();
        (*cart).rom = rom;
        (*cart).romSize = (*cart).rom.len() as u32;
        (*cart).ram = crate::save_ram::SaveRam::zeroed(ramSize.max(0) as usize);
        (*cart).ramSize = ramSize as u32;
    }
}
pub unsafe fn cart_write(cart: *mut Cart, bank: u8, adr: u16, val: u8) {
    unsafe {
        match (*cart).type_0 {
            1 => cart_writeLorom(cart, bank, adr, val),
            2 => cart_writeHirom(cart, bank, adr, val),
            _ => {}
        };
    }
}
unsafe fn cart_writeLorom(cart: *mut Cart, bank: u8, address: u16, value: u8) {
    unsafe {
        if (matches!(bank, 0x70..=0x7d) || bank > 0xf0) && address < 0x8000 && (*cart).ramSize > 0 {
            let index = ((u32::from(bank & 0xf) << 15) | u32::from(address))
                & (*cart).ramSize.wrapping_sub(1);
            (*cart).ram.write(index as usize, value);
        }
    }
}
unsafe fn cart_writeHirom(cart: *mut Cart, mut bank: u8, adr: u16, val: u8) {
    unsafe {
        bank = bank & 0x7f;
        if (bank as i32) < 0x40
            && adr as i32 >= 0x6000
            && (adr as i32) < 0x8000
            && (*cart).ramSize > 0
        {
            (*cart).ram.write(
                (((bank as i32 & 0x3f) << 13 | adr as i32 & 0x1fff) as u32
                    & (*cart).ramSize.wrapping_sub(1)) as usize,
                val,
            );
        }
    }
}

// Cartridge mapping is shared; invalid-read diagnostics keep their actual host
// and device context and the original output/failure order.
unsafe fn read_lorom_memory(
    cart: *mut Cart,
    mut bank: u8,
    adr: u16,
    source: Option<&DmaRomSource<'_>>,
) -> Option<u8> {
    unsafe {
        if (matches!(bank, 0x70..=0x7d) || bank >= 0xf0) && adr < 0x8000 && (*cart).ramSize > 0 {
            let index =
                ((u32::from(bank & 0xf) << 15) | u32::from(adr)) & (*cart).ramSize.wrapping_sub(1);
            return Some((*cart).ram.read(index as usize));
        }
        bank &= 0x7f;
        if adr >= 0x8000 || bank >= 0x40 {
            let index = ((u32::from(bank) << 15) | u32::from(adr & 0x7fff))
                & (*cart).romSize.wrapping_sub(1);
            if let Some(value) = source.and_then(|source| source.byte(index as usize)) {
                return Some(value);
            }
            return Some((*cart).rom.read(index as usize));
        }
        None
    }
}

unsafe fn read_hirom_memory(
    cart: *mut Cart,
    bank: u8,
    adr: u16,
    source: Option<&DmaRomSource<'_>>,
) -> Option<u8> {
    unsafe {
        let bank = bank & 0x7f;
        if (bank as i32) < 0x40
            && adr as i32 >= 0x6000
            && (adr as i32) < 0x8000
            && (*cart).ramSize > 0
        {
            return Some((*cart).ram.read(
                (((bank as i32 & 0x3f) << 13 | adr as i32 & 0x1fff) as u32
                    & (*cart).ramSize.wrapping_sub(1)) as usize,
            ));
        }
        if adr as i32 >= 0x8000 || bank as i32 >= 0x40 {
            let index = (((bank as i32 & 0x3f) << 16 | adr as i32) as u32
                & (*cart).romSize.wrapping_sub(1)) as usize;
            if let Some(value) = source.and_then(|source| source.byte(index)) {
                return Some(value);
            }
            return Some((*cart).rom.read(index));
        }
        None
    }
}

impl Execution<'_> {
    pub(crate) unsafe fn read_cart(&mut self, bank: u8, adr: u16) -> u8 {
        unsafe { self.read_cart_with_dma_source(bank, adr, None) }
    }

    pub(crate) unsafe fn read_cart_with_dma_source(
        &mut self,
        bank: u8,
        adr: u16,
        source: Option<&DmaRomSource<'_>>,
    ) -> u8 {
        unsafe {
            let cart = self.cart();
            match (*cart).type_0 {
                0 => {
                    crate::native_assertion! {
                        '_c2rust_label;
                        b"0\0", b"src/snes/cart.c\0", 57,
                        b"uint8_t cart_read(Cart *, uint8_t, uint16_t)\0"
                    };
                }
                1 => {
                    if let Some(value) = read_lorom_memory(cart, bank, adr, source) {
                        return value;
                    }
                    return self.invalid_lorom_read(bank, adr);
                }
                2 => {
                    if let Some(value) = read_hirom_memory(cart, bank, adr, source) {
                        return value;
                    }
                    crate::native_assertion! {
                        '_c2rust_label;
                        b"0\0", b"src/snes/cart.c\0", 109,
                        b"uint8_t cart_readHirom(Cart *, uint8_t, uint16_t)\0"
                    };
                }
                _ => {}
            }
            crate::native_assertion! {
                '_c2rust_label_0;
                b"0\0", b"src/snes/cart.c\0", 62,
                b"uint8_t cart_read(Cart *, uint8_t, uint16_t)\0"
            };
        }
    }

    pub(crate) unsafe fn write_cart(&mut self, bank: u8, adr: u16, value: u8) {
        unsafe { cart_write(self.cart(), bank, adr, value) }
    }

    unsafe fn invalid_lorom_read(&mut self, original_bank: u8, adr: u16) -> u8 {
        unsafe {
            // Unmapped addresses return the last value on the CPU data bus.
            if crate::oracle_mode::models_open_bus() {
                return (*self.snes()).openBus;
            }
            let bank = original_bank & 0x7f;
            printf(
                c"While trying to read from 0x%x\n".as_ptr(),
                words!(i32, (bank, adr)),
            );
            let selected_cpu = self.selected_cpu();
            let frame = *self.host.snes_frame_counter;
            let (k, pc, a, x, y, sp, dp, db) = {
                let cpu = &*selected_cpu;
                (cpu.k, cpu.pc, cpu.a, cpu.x, cpu.y, cpu.sp, cpu.dp, cpu.db)
            };
            eprintln!(
                "Invalid LoROM read ${original_bank:02X}:{adr:04X}, frame {}: PC ${:02X}:{:04X} A ${:04X} X ${:04X} Y ${:04X} SP ${:04X} DP ${:04X} DB ${:02X}",
                frame, k, pc, a, x, y, sp, dp, db,
            );
            let (n, v, mf, xf, d, i, z, c, e) = {
                let cpu = &*selected_cpu;
                (
                    cpu.n, cpu.v, cpu.mf, cpu.xf, cpu.d, cpu.i, cpu.z, cpu.c, cpu.e,
                )
            };
            eprintln!(
                "CPU flags: N={} V={} M={} X={} D={} I={} Z={} C={} E={}",
                n, v, mf, xf, d, i, z, c, e,
            );
            let mut direct_page = [0; 13];
            for (index, value) in direct_page.iter_mut().enumerate() {
                let address = (*selected_cpu).dp.wrapping_add(0x44 + index as u16);
                *value = self.ram().read_byte(usize::from(address));
            }
            eprintln!("Direct page $44..$50: {direct_page:02X?}");
            self.dump_cpu_history();
            Die(c"The game crashed in cart_readLorom".as_ptr());
            (*self.snes()).openBus
        }
    }
}
