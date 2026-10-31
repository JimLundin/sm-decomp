//! Synthetic actual-code observations; no cartridge-derived fixtures.
use crate::ram_map::{word_arrays as a, words as w};
use crate::snes::cart::CartRom;
use crate::{game_state::Game, rom::RomImage};
use std::panic::{AssertUnwindSafe, catch_unwind};
const COMMANDS: [u32; 24] = [
    8880306, 8880311, 8880316, 8880340, 8880348, 8880355, 8880368, 8880380, 8880392, 8880404,
    8880416, 8880435, 8880447, 8880464, 8880570, 8880899, 8880928, 8880943, 8880958, 8880969,
    8880978, 8880987, 8881010, 8881023,
];
pub(crate) const CASES: u32 = 438;
// Entry selection only; frozen case inputs and observations remain unchanged.
fn enabled(value: bool, g: &mut Game) {
    crate::game::animated_tiles::set_enabled(value, g)
}
fn clear(g: &mut Game) {
    crate::game::animated_tiles::clear(g)
}
fn spawn(def: u16, g: &mut Game) {
    crate::game::animated_tiles::spawn(def, g)
}
fn advance(g: &mut Game) {
    crate::game::animated_tiles::advance(g)
}
fn process(g: &mut Game) {
    crate::game::animated_tiles::checks::process_one(g)
}
fn command(ea: u32, k: u16, j: u16, g: &mut Game) -> u16 {
    crate::game::animated_tiles::checks::command(ea, k, j, g)
}
fn upload(g: &mut Game) {
    crate::game::animated_tiles::upload_pending(g)
}
fn put(g: &mut Game, at: usize, value: u16) {
    let _ = &[(); 0x20000][at..at + 2];
    g.ram().copy_in(at, &value.to_le_bytes());
}
fn slot(g: &mut Game, column: usize, k: u16, value: u16) {
    put(g, column + usize::from(k & !1), value);
}
fn rw(image: &mut [u8], bank: u8, cursor: u16, offset: usize, value: u16) {
    let at =
        crate::rtl::lorom_offset((u32::from(bank) << 16) | u32::from(cursor)) as usize + offset;
    image[at..at + 2].copy_from_slice(&value.to_ne_bytes());
}
/// Register an image of the case's synthetic bytes as both loaded and native ROM.
fn install(g: &mut Game, bytes: &[u8]) {
    let image = RomImage::from(bytes);
    g.loaded_rom = Some(image.clone());
    g.register_native_rom(image);
}
fn seed(g: &mut Game, image: &mut [u8]) {
    for at in 0x1ef1..0x1f51 {
        let value = (at * 37) as u8;
        g.ram().write_byte(at, value);
    }
    for k in (0..12).step_by(2) {
        for (column, value) in [
            (a::ANIMTILES_IDS, 0),
            (a::ANIMTILES_TIMERS, 2),
            (a::ANIMTILES_INSTR_LIST_PTRS, 0x9000),
            (a::ANIMTILES_INSTR_TIMERS, 1),
            (a::ANIMTILES_SRC_PTR, 0),
            (a::ANIMTILES_SIZES, 4),
            (a::ANIMTILES_VRAM_PTR, 0x100),
        ] {
            slot(g, column, k, value);
        }
    }
    put(g, w::ANIMTILES_ENABLE_FLAG, 0x9234);
    put(g, w::ANIMTILES_OBJECT_INDEX, 0);
    for (off, value) in [(0, 0x9000), (2, 4), (4, 0x123)] {
        rw(image, 0x87, 0x8000, off, value);
    }
    rw(image, 0x87, 0x9000, 0, 3);
    rw(image, 0x87, 0x9000, 2, 0x9234);
    rw(image, 0x8d, 0x8000, 0, 9291397u32 as u16);
    rw(image, 0x8d, 0x8000, 2, 0x9000);
    for (def, init) in [(0xba6a, 8829070u32), (0xba94, 8829176)] {
        rw(
            image,
            0x86,
            def,
            core::mem::offset_of!(crate::types::EprojDef, init_code_ptr),
            init as u16,
        );
    }
}
fn operands(image: &mut [u8], cursor: u16, cmd: usize, v: u32) {
    let value = match cmd {
        16 | 17 => (v % 4) as u16 * 2,
        22 => 0x8000,
        21 | 23 => [0, 1, 0x1ef1, 0xfffe][(v % 4) as usize],
        1 | 3 | 18 => 0x9234,
        2 | 4 => [0x007f, 0x0080, 0x00ff, 0][(v % 4) as usize],
        _ => 1,
    };
    rw(image, 0x87, cursor, 0, value);
    rw(image, 0x87, cursor, 2, 0x9256);
}
// Cases that author more bytes install an image of them before the engine reads it.
fn execute(id: u32, g: &mut Game, image: &mut [u8], trace: &mut Vec<u8>) -> u16 {
    match id {
        0..=15 => {
            for k in (0..12).step_by(2) {
                slot(g, a::ANIMTILES_IDS, k, 0x8123 + k);
            }
            match id {
                0 => clear(g),
                1 => {
                    enabled(false, g);
                    trace.extend((0..g.ram().len()).map(|at| g.ram().read_byte(at)));
                    enabled(true, g);
                }
                2 => {
                    g.loaded_rom = None;
                    spawn(0xffff, g);
                }
                3..=8 => {
                    slot(g, a::ANIMTILES_IDS, (id - 3) as u16 * 2, 0);
                    spawn(0x8000, g);
                }
                9 => {
                    slot(g, a::ANIMTILES_IDS, 2, 0);
                    slot(g, a::ANIMTILES_IDS, 8, 0);
                    spawn(0x8000, g);
                }
                10 => {
                    clear(g);
                    g.loaded_rom = None;
                    spawn(0x8000, g);
                }
                11..=15 => {
                    clear(g);
                    let len = (id - 11) as usize;
                    let at = crate::rtl::lorom_offset(0x878000) as usize;
                    let mut short = vec![0; at + len];
                    short[at..].copy_from_slice(&[0, 0x90, 4, 0, 0x23][..len]);
                    g.loaded_rom = Some(RomImage::from(short));
                    spawn(0x8000, g);
                }
                _ => unreachable!(),
            }
        }
        16..=303 => {
            let n = id - 16;
            let cmd = (n / 12) as usize;
            let v = n % 12;
            let k = [0, 1, 10, 0xffff][(v % 4) as usize];
            let cursor = if v >= 8 { 0xfffe } else { 0x9000 };
            slot(g, a::ANIMTILES_IDS, k, 0x8123);
            slot(
                g,
                a::ANIMTILES_TIMERS,
                k,
                [0, 1, 2, 0xffff][(v % 4) as usize],
            );
            put(g, w::ANIMTILES_OBJECT_INDEX, k);
            put(
                g,
                w::TOURIAN_ENTRANCE_STATUE_ANIMSTATE,
                if v & 1 != 0 { 0x8000 } else { 0 },
            );
            put(g, 0xd828, if v & 1 != 0 { 0xffff } else { 0 });
            put(g, 0xd820, if v & 1 != 0 { 0xffff } else { 0 });
            operands(image, cursor, cmd, v);
            install(g, image);
            return command(COMMANDS[cmd], k, cursor, g);
        }
        304..=327 => {
            let n = id - 304;
            let k = [0, 1, 10, 0xffff][(n % 4) as usize];
            put(g, w::ANIMTILES_OBJECT_INDEX, k);
            slot(g, a::ANIMTILES_IDS, k, 0x8000);
            slot(g, a::ANIMTILES_INSTR_LIST_PTRS, k, 0x9000);
            slot(
                g,
                a::ANIMTILES_INSTR_TIMERS,
                k,
                [0, 1, 2, 0xffff][(n / 4 % 4) as usize],
            );
            if n == 15 {
                // Actual lifecycle producer feeds the actual NMI consumer.
                for (offset, value) in [(0, 0x9000), (2, 4), (4, 0x123)] {
                    rw(image, 0x87, 0x8000, offset, value);
                }
                install(g, image);
                clear(g);
                spawn(0x8000, g);
                advance(g);
                let at = crate::rtl::lorom_offset(0x879234) as usize;
                image[at..at + 4].copy_from_slice(&[0x80, 0x81, 0x82, 0x83]);
                install(g, image);
            }
            if n >= 16 {
                slot(g, a::ANIMTILES_INSTR_TIMERS, k, 1);
                rw(
                    image,
                    0x87,
                    0x9000,
                    0,
                    COMMANDS[if n < 20 { 0 } else { 5 }] as u16,
                );
                rw(image, 0x87, 0x9000, 2, 0x0307);
                rw(image, 0x87, 0x9000, 4, 0x3400);
                rw(image, 0x87, 0x9000, 6, 0x0092);
                install(g, image);
            }
            process(g);
        }
        328..=351 => {
            let n = id - 328;
            let target = [
                w::ANIMTILES_ENABLE_FLAG,
                w::ANIMTILES_OBJECT_INDEX,
                a::ANIMTILES_IDS,
                a::ANIMTILES_TIMERS,
                a::ANIMTILES_INSTR_LIST_PTRS,
                a::ANIMTILES_INSTR_TIMERS,
                a::ANIMTILES_SRC_PTR,
                a::ANIMTILES_SIZES,
                a::ANIMTILES_VRAM_PTR,
                w::ANIMTILES_INSTRUCTION,
                0x1f4b,
                0x1ef0,
            ][(n % 12) as usize];
            put(
                g,
                w::MUSIC_QUEUE_WRITE_POS,
                (target - a::MUSIC_QUEUE_TRACK) as u16,
            );
            put(g, w::MUSIC_QUEUE_READ_POS, 1);
            if n < 12 {
                return command(COMMANDS[6], 1, 0x9000, g);
            }
            slot(g, a::ANIMTILES_IDS, 10, 0x8000);
            rw(image, 0x87, 0x9000, 0, COMMANDS[6] as u16);
            rw(image, 0x87, 0x9000, 2, 0x0300);
            rw(image, 0x87, 0x9000, 4, 0x3400);
            rw(image, 0x87, 0x9000, 6, 0x0092);
            install(g, image);
            advance(g);
        }
        352..=375 => {
            let n = id - 352;
            let cmd = [1, 3, 4, 5, 10, 21, 22, 23][(n / 3) as usize];
            let len = (n % 3) as usize;
            let at = crate::rtl::lorom_offset(0x879000) as usize;
            let mut short = vec![0; at + len];
            short[at..].fill(1);
            g.loaded_rom = Some(RomImage::from(short));
            slot(g, a::ANIMTILES_TIMERS, 1, if n < 6 { 1 } else { 2 });
            return command(COMMANDS[cmd], 1, 0x9000, g);
        }
        376..=399 => return dma_case(id - 376, g, image, trace),
        400..=415 => {
            let n = id - 400;
            g.loaded_rom = None;
            match n {
                0..=3 => {
                    slot(
                        g,
                        a::ANIMTILES_INSTR_TIMERS,
                        1,
                        [0, 1, 2, 0xffff][n as usize],
                    );
                    put(g, w::ANIMTILES_OBJECT_INDEX, 1);
                    process(g);
                }
                4..=7 => {
                    slot(g, a::ANIMTILES_TIMERS, 1, if n & 1 == 0 { 1 } else { 2 });
                    return command(COMMANDS[if n < 6 { 3 } else { 4 }], 1, 0xffff, g);
                }
                8..=11 => {
                    put(
                        g,
                        w::ANIMTILES_ENABLE_FLAG,
                        [0, 0x80, 0x7fff, 0x8000][(n - 8) as usize],
                    );
                    advance(g);
                }
                _ => {
                    return command(COMMANDS[[0, 14, 18, 5][(n - 12) as usize]], 1, 0xffff, g);
                }
            }
        }
        416..=423 => {
            let n = id - 416;
            let cmd = [1, 2, 3, 3, 4, 4, 5, 19][n as usize];
            slot(
                g,
                a::ANIMTILES_TIMERS,
                1,
                if n == 2 || n == 4 { 1 } else { 2 },
            );
            rw(image, 0x87, 0x9000, 0, 1);
            install(g, image);
            return command(COMMANDS[cmd], 1, 0x1000, g);
        }
        424..=431 => {
            // Out-of-range Id aliases every following owned column, then the gap.
            let k = [12, 24, 36, 48, 60, 72, 84, 86][(id - 424) as usize];
            return command(COMMANDS[0], k, 0x9000, g);
        }
        432..=436 => {
            // Counter-byte aliases retain their exact odd/even byte lane.
            let k = [12, 13, 72, 73, 74][(id - 432) as usize];
            rw(image, 0x87, 0x9000, 0, 0xa5);
            install(g, image);
            return command(COMMANDS[5], k, 0x9000, g);
        }
        437 => {
            // Selected12 maps Program to Delay0, Delay to Source0, Source to Size0.
            put(g, w::ANIMTILES_OBJECT_INDEX, 12);
            slot(g, a::ANIMTILES_INSTR_LIST_PTRS, 12, 0x9000);
            slot(g, a::ANIMTILES_INSTR_TIMERS, 12, 1);
            process(g);
        }
        _ => unreachable!(),
    }
    0
}
fn dma_case(n: u32, g: &mut Game, image: &mut [u8], trace: &mut Vec<u8>) -> u16 {
    let previous_cpu = g.g_cpu;
    g.ensure_backend();
    let snes = g.g_snes;
    let (ppu, cart) = unsafe { ((*snes).ppu, (*snes).cart) };
    unsafe {
        (*cart).type_0 = 1;
        (*cart).rom_size = image.len() as u32;
        let ppu = &mut *ppu;
        ppu.vramIncrement = 32;
        ppu.vramRemapMode = (n % 4) as u8;
        ppu.vramIncrementOnHigh = false;
        for (i, v) in ppu.vram.iter_mut().enumerate() {
            *v = (i as u16).wrapping_mul(37);
        }
    }
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        // The cartridge serves the image's final bytes to the upload's DMA.
        let mut short_cart = None;
        let start = crate::rtl::lorom_offset(0x878000) as usize;
        for (at, byte) in image[start..start + 0x100].iter_mut().enumerate() {
            *byte = (at * 11) as u8;
        }
        install(g, image);
        for k in (0..12).step_by(2) {
            slot(g, a::ANIMTILES_IDS, k, 0x8000 + k);
            slot(g, a::ANIMTILES_SRC_PTR, k, 0x8000 + k * 8);
            slot(
                g,
                a::ANIMTILES_SIZES,
                k,
                [1, 2, 3, 4, 7, 8][usize::from(k / 2)],
            );
            slot(g, a::ANIMTILES_VRAM_PTR, k, 0x123);
        }
        if n < 4 {
            put(
                g,
                w::ANIMTILES_ENABLE_FLAG,
                [0, 0x80, 0x7fff, 0x8000][n as usize],
            );
        }
        if n == 4 {
            clear(g);
        }
        if n == 5 {
            for k in (0..12).step_by(2) {
                slot(g, a::ANIMTILES_SRC_PTR, k, 0);
            }
        }
        if n == 6 {
            slot(g, a::ANIMTILES_IDS, 10, 0);
        }
        if n == 7 {
            slot(g, a::ANIMTILES_SRC_PTR, 10, 0);
        }
        if (8..16).contains(&n) {
            clear(g);
            slot(g, a::ANIMTILES_IDS, 10, 0x8000);
            slot(
                g,
                a::ANIMTILES_SRC_PTR,
                10,
                if n < 12 { 0x1ef1 } else { 0xffff },
            );
            slot(g, a::ANIMTILES_SIZES, 10, if n < 12 { 90 } else { 2 });
            slot(g, a::ANIMTILES_VRAM_PTR, 10, 0x7fff);
        }
        if n == 15 {
            // Actual lifecycle producer feeds the actual NMI consumer.
            for (offset, value) in [(0, 0x9000), (2, 4), (4, 0x123)] {
                rw(image, 0x87, 0x8000, offset, value);
            }
            install(g, image);
            clear(g);
            spawn(0x8000, g);
            advance(g);
            let at = crate::rtl::lorom_offset(0x879234) as usize;
            image[at..at + 4].copy_from_slice(&[0x80, 0x81, 0x82, 0x83]);
            install(g, image);
        }
        if n >= 16 {
            // Short cart allocation forces a Rust panic during real DMA after
            // zero, one, or two completed bytes. Size zero must still start DMA.
            clear(g);
            slot(g, a::ANIMTILES_IDS, 10, 0x8000);
            let at = crate::rtl::lorom_offset(0x878000) as usize;
            short_cart = Some(at + (n % 3) as usize);
            slot(g, a::ANIMTILES_SIZES, 10, if n >= 20 { 0 } else { 8 });
            slot(g, a::ANIMTILES_SRC_PTR, 10, 0x8000);
        }
        let cart_rom = short_cart.map_or_else(|| CartRom::from(image.to_vec()), CartRom::zeroed);
        unsafe {
            (*cart).rom = cart_rom;
        }
        let result = catch_unwind(AssertUnwindSafe(|| upload(g)));
        let mut hardware = crate::recording::RecordingBytes::default();
        let mut io = crate::snapshot_io::SnapshotIo::Save(&mut hardware);
        unsafe {
            io.state(&mut *ppu).unwrap();
            io.state(&mut g.dma_state()).unwrap();
            io.state(&mut *snes).unwrap();
        }
        trace.extend_from_slice(hardware.as_slice());
        g.g_snes = core::ptr::null_mut();
        if let Err(error) = result {
            std::panic::resume_unwind(error);
        }
        0
    }));
    g.g_cpu = previous_cpu;
    let _hardware = unsafe { g.take_backend_for_shutdown() };
    match outcome {
        Ok(value) => value,
        Err(error) => std::panic::resume_unwind(error),
    }
}
pub(crate) fn capture_case(id: u32) -> Option<Vec<u8>> {
    if id >= CASES {
        return None;
    }
    let mut g = Game::new();
    let mut rom = vec![0; 0x80000];
    seed(&mut g, &mut rom);
    install(&mut g, &rom);
    let mut trace = Vec::new();
    let result = catch_unwind(AssertUnwindSafe(|| {
        execute(id, &mut g, &mut rom, &mut trace)
    }));
    let panicked = result.is_err();
    let mut bytes = (0..g.ram().len())
        .map(|at| g.ram().read_byte(at))
        .collect::<Vec<_>>();
    bytes.extend_from_slice(&result.unwrap_or(0).to_le_bytes());
    bytes.push(u8::from(panicked));
    bytes.push(u8::from(g.g_fail));
    bytes.extend(trace);
    Some(bytes)
}
