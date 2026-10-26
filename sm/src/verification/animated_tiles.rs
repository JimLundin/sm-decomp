//! Synthetic actual-code observations; no cartridge-derived fixtures.
use crate::ram_map::{word_arrays as a, words as w};
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
    g.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn slot(g: &mut Game, column: usize, k: u16, value: u16) {
    put(g, column + usize::from(k & !1), value);
}
fn rw(image: &RomImage, bank: u8, cursor: u16, offset: usize, value: u16) {
    let at =
        crate::rtl::lorom_offset((u32::from(bank) << 16) | u32::from(cursor)) as usize + offset;
    for (i, b) in value.to_ne_bytes().into_iter().enumerate() {
        image.write(at + i, b);
    }
}
fn seed(g: &mut Game, image: &RomImage) {
    for at in 0x1ef1..0x1f51 {
        g.g_ram[at] = (at * 37) as u8;
    }
    for k in (0..12).step_by(2) {
        for (column, value) in [
            (a::animtiles_ids, 0),
            (a::animtiles_timers, 2),
            (a::animtiles_instr_list_ptrs, 0x9000),
            (a::animtiles_instr_timers, 1),
            (a::animtiles_src_ptr, 0),
            (a::animtiles_sizes, 4),
            (a::animtiles_vram_ptr, 0x100),
        ] {
            slot(g, column, k, value);
        }
    }
    put(g, w::animtiles_enable_flag, 0x9234);
    put(g, w::animtiles_object_index, 0);
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
fn operands(image: &RomImage, cursor: u16, cmd: usize, v: u32) {
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
fn execute(id: u32, g: &mut Game, image: &RomImage, trace: &mut Vec<u8>) -> u16 {
    match id {
        0..=15 => {
            for k in (0..12).step_by(2) {
                slot(g, a::animtiles_ids, k, 0x8123 + k);
            }
            match id {
                0 => clear(g),
                1 => {
                    enabled(false, g);
                    trace.extend_from_slice(&g.g_ram);
                    enabled(true, g);
                }
                2 => {
                    g.loaded_rom = None;
                    spawn(0xffff, g);
                }
                3..=8 => {
                    slot(g, a::animtiles_ids, (id - 3) as u16 * 2, 0);
                    spawn(0x8000, g);
                }
                9 => {
                    slot(g, a::animtiles_ids, 2, 0);
                    slot(g, a::animtiles_ids, 8, 0);
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
                    let short = RomImage::zeroed(at + len);
                    for i in 0..len {
                        short.write(at + i, [0, 0x90, 4, 0, 0x23][i]);
                    }
                    g.loaded_rom = Some(short);
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
            slot(g, a::animtiles_ids, k, 0x8123);
            slot(
                g,
                a::animtiles_timers,
                k,
                [0, 1, 2, 0xffff][(v % 4) as usize],
            );
            put(g, w::animtiles_object_index, k);
            put(
                g,
                w::tourian_entrance_statue_animstate,
                if v & 1 != 0 { 0x8000 } else { 0 },
            );
            put(g, 0xd828, if v & 1 != 0 { 0xffff } else { 0 });
            put(g, 0xd820, if v & 1 != 0 { 0xffff } else { 0 });
            operands(image, cursor, cmd, v);
            return command(COMMANDS[cmd], k, cursor, g);
        }
        304..=327 => {
            let n = id - 304;
            let k = [0, 1, 10, 0xffff][(n % 4) as usize];
            put(g, w::animtiles_object_index, k);
            slot(g, a::animtiles_ids, k, 0x8000);
            slot(g, a::animtiles_instr_list_ptrs, k, 0x9000);
            slot(
                g,
                a::animtiles_instr_timers,
                k,
                [0, 1, 2, 0xffff][(n / 4 % 4) as usize],
            );
            if n == 15 {
                // Actual lifecycle producer feeds the actual NMI consumer.
                for (offset, value) in [(0, 0x9000), (2, 4), (4, 0x123)] {
                    rw(image, 0x87, 0x8000, offset, value);
                }
                clear(g);
                spawn(0x8000, g);
                advance(g);
                for i in 0..4 {
                    image.write(
                        crate::rtl::lorom_offset(0x879234) as usize + i,
                        0x80 + i as u8,
                    );
                }
            }
            if n >= 16 {
                slot(g, a::animtiles_instr_timers, k, 1);
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
            }
            process(g);
        }
        328..=351 => {
            let n = id - 328;
            let target = [
                w::animtiles_enable_flag,
                w::animtiles_object_index,
                a::animtiles_ids,
                a::animtiles_timers,
                a::animtiles_instr_list_ptrs,
                a::animtiles_instr_timers,
                a::animtiles_src_ptr,
                a::animtiles_sizes,
                a::animtiles_vram_ptr,
                w::animtiles_instruction,
                0x1f4b,
                0x1ef0,
            ][(n % 12) as usize];
            put(
                g,
                w::music_queue_write_pos,
                (target - a::music_queue_track) as u16,
            );
            put(g, w::music_queue_read_pos, 1);
            if n < 12 {
                return command(COMMANDS[6], 1, 0x9000, g);
            }
            slot(g, a::animtiles_ids, 10, 0x8000);
            rw(image, 0x87, 0x9000, 0, COMMANDS[6] as u16);
            rw(image, 0x87, 0x9000, 2, 0x0300);
            rw(image, 0x87, 0x9000, 4, 0x3400);
            rw(image, 0x87, 0x9000, 6, 0x0092);
            advance(g);
        }
        352..=375 => {
            let n = id - 352;
            let cmd = [1, 3, 4, 5, 10, 21, 22, 23][(n / 3) as usize];
            let len = (n % 3) as usize;
            let at = crate::rtl::lorom_offset(0x879000) as usize;
            let short = RomImage::zeroed(at + len);
            for i in 0..len {
                short.write(at + i, 1);
            }
            g.loaded_rom = Some(short);
            slot(g, a::animtiles_timers, 1, if n < 6 { 1 } else { 2 });
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
                        a::animtiles_instr_timers,
                        1,
                        [0, 1, 2, 0xffff][n as usize],
                    );
                    put(g, w::animtiles_object_index, 1);
                    process(g);
                }
                4..=7 => {
                    slot(g, a::animtiles_timers, 1, if n & 1 == 0 { 1 } else { 2 });
                    return command(COMMANDS[if n < 6 { 3 } else { 4 }], 1, 0xffff, g);
                }
                8..=11 => {
                    put(
                        g,
                        w::animtiles_enable_flag,
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
                a::animtiles_timers,
                1,
                if n == 2 || n == 4 { 1 } else { 2 },
            );
            rw(image, 0x87, 0x9000, 0, 1);
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
            return command(COMMANDS[5], k, 0x9000, g);
        }
        437 => {
            // Selected12 maps Program to Delay0, Delay to Source0, Source to Size0.
            put(g, w::animtiles_object_index, 12);
            slot(g, a::animtiles_instr_list_ptrs, 12, 0x9000);
            slot(g, a::animtiles_instr_timers, 12, 1);
            process(g);
        }
        _ => unreachable!(),
    }
    0
}
fn dma_case(n: u32, g: &mut Game, image: &RomImage, trace: &mut Vec<u8>) -> u16 {
    use crate::initial_state::InitialState;
    let mut snes = Box::new(crate::types::Snes::INITIAL);
    let mut ppu = Box::new(crate::types::Ppu::INITIAL);
    let mut dma = Box::new(crate::types::Dma::INITIAL);
    let mut cart = Box::new(crate::types::Cart {
        snes: &mut *snes,
        type_0: 1,
        rom: image.clone(),
        romSize: image.len() as u32,
        ram: Default::default(),
        ramSize: 0,
    });
    snes.ppu = &mut *ppu;
    snes.dma = &mut *dma;
    snes.cart = &mut *cart;
    snes.ram = g.g_ram.as_mut_ptr();
    ppu.snes = &mut *snes;
    dma.snes = &mut *snes;
    g.g_snes = &mut *snes;
    ppu.vramIncrement = 32;
    ppu.vramRemapMode = (n % 4) as u8;
    ppu.vramIncrementOnHigh = false;
    for (i, v) in ppu.vram.iter_mut().enumerate() {
        *v = (i as u16).wrapping_mul(37);
    }
    for at in 0..0x100 {
        image.write(
            crate::rtl::lorom_offset(0x878000) as usize + at,
            (at * 11) as u8,
        );
    }
    for k in (0..12).step_by(2) {
        slot(g, a::animtiles_ids, k, 0x8000 + k);
        slot(g, a::animtiles_src_ptr, k, 0x8000 + k * 8);
        slot(
            g,
            a::animtiles_sizes,
            k,
            [1, 2, 3, 4, 7, 8][usize::from(k / 2)],
        );
        slot(g, a::animtiles_vram_ptr, k, 0x123);
    }
    if n < 4 {
        put(
            g,
            w::animtiles_enable_flag,
            [0, 0x80, 0x7fff, 0x8000][n as usize],
        );
    }
    if n == 4 {
        clear(g);
    }
    if n == 5 {
        for k in (0..12).step_by(2) {
            slot(g, a::animtiles_src_ptr, k, 0);
        }
    }
    if n == 6 {
        slot(g, a::animtiles_ids, 10, 0);
    }
    if n == 7 {
        slot(g, a::animtiles_src_ptr, 10, 0);
    }
    if (8..16).contains(&n) {
        clear(g);
        slot(g, a::animtiles_ids, 10, 0x8000);
        slot(
            g,
            a::animtiles_src_ptr,
            10,
            if n < 12 { 0x1ef1 } else { 0xffff },
        );
        slot(g, a::animtiles_sizes, 10, if n < 12 { 90 } else { 2 });
        slot(g, a::animtiles_vram_ptr, 10, 0x7fff);
    }
    if n == 15 {
        // Actual lifecycle producer feeds the actual NMI consumer.
        for (offset, value) in [(0, 0x9000), (2, 4), (4, 0x123)] {
            rw(image, 0x87, 0x8000, offset, value);
        }
        clear(g);
        spawn(0x8000, g);
        advance(g);
        for i in 0..4 {
            image.write(
                crate::rtl::lorom_offset(0x879234) as usize + i,
                0x80 + i as u8,
            );
        }
    }
    if n >= 16 {
        // Short cart allocation forces a Rust panic during real DMA after
        // zero, one, or two completed bytes. Size zero must still start DMA.
        clear(g);
        slot(g, a::animtiles_ids, 10, 0x8000);
        let at = crate::rtl::lorom_offset(0x878000) as usize;
        cart.rom = RomImage::zeroed(at + (n % 3) as usize);
        slot(g, a::animtiles_sizes, 10, if n >= 20 { 0 } else { 8 });
        slot(g, a::animtiles_src_ptr, 10, 0x8000);
    }
    let result = catch_unwind(AssertUnwindSafe(|| upload(g)));
    let mut hardware = crate::recording::RecordingBytes::default();
    let mut io = crate::snapshot_io::SnapshotIo::Save(&mut hardware);
    io.state(&mut *ppu).unwrap();
    io.state(&mut *dma).unwrap();
    io.state(&mut *snes).unwrap();
    trace.extend_from_slice(hardware.as_slice());
    g.g_snes = core::ptr::null_mut();
    if let Err(error) = result {
        std::panic::resume_unwind(error);
    }
    0
}
pub(crate) fn capture_case(id: u32) -> Option<Vec<u8>> {
    if id >= CASES {
        return None;
    }
    let mut g = Game::new();
    let image = RomImage::zeroed(0x80000);
    seed(&mut g, &image);
    g.loaded_rom = Some(image.clone());
    g.register_native_rom(image.clone());
    let mut trace = Vec::new();
    let result = catch_unwind(AssertUnwindSafe(|| execute(id, &mut g, &image, &mut trace)));
    let panicked = result.is_err();
    let mut bytes = g.g_ram.to_vec();
    bytes.extend_from_slice(&result.unwrap_or(0).to_le_bytes());
    bytes.push(u8::from(panicked));
    bytes.push(u8::from(g.g_fail));
    bytes.extend(trace);
    Some(bytes)
}
