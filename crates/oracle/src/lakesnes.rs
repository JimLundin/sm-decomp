//! The LakeSnes backend: upstream angelo-wf/LakeSnes, unmodified, behind
//! `native/bridge.c`.

use crate::record::{
    ApuAccess, ApuAccessKind, Beam, CpuRegisters, Cut, FrameRecord, Hit, MAX_LINES, Nmi, NmiKind,
    Pad, PpuRegisters, RegisterWrite, SpcState, WIDTH, WriteSource,
};
use crate::{Oracle, OracleError, OracleId, Start, Tiers};
use core::ffi::c_int;
use core::hash::{Hash, Hasher};
use data::digest::Digest;

/// The backend's identity in records and locks.
pub const IDENTITY: OracleId = OracleId {
    emulator: "lakesnes",
    version: "angelo-wf/LakeSnes 9db90b86e46a377609305e298dd92d71cd1d4c8a",
    patches: &[],
    fixes: &[
        "0001-apu-clock-real-master-rate",
        "0002-apu-clock-bsnes-v085-exact",
        "0004-hdma-overhead-only-while-a-channel-runs",
    ],
};

mod ffi {
    use core::ffi::c_int;

    #[repr(C)]
    pub struct Machine {
        _opaque: [u8; 0],
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct Nmi {
        pub interrupted_pc: u32,
        pub polled: u8,
        pub returned: u8,
        pub stack: u16,
    }

    #[repr(C)]
    pub struct Facts {
        pub video_frame: u32,
        pub nmi_count: u32,
        pub nmis: [Nmi; 8],
        pub polls: u32,
        pub pad_latched: u16,
        pub ended_at_rti: u8,
        pub overscan: u8,
        pub lines_rendered: u32,
        pub wait_v: u16,
        pub wait_h: u16,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct Write {
        pub v: u16,
        pub h: u16,
        pub adr: u8,
        pub value: u8,
        pub source: u8,
        pub reserved: u8,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct Hit {
        pub pc: u32,
        pub v: u16,
        pub h: u16,
        pub peek: [u8; 4],
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct ApuAccess {
        pub kind: u8,
        pub port: u8,
        pub value: u8,
        pub reserved: u8,
        pub apu_cycle: u32,
    }

    #[repr(C)]
    #[derive(Default)]
    pub struct Cpu {
        pub a: u16,
        pub x: u16,
        pub y: u16,
        pub sp: u16,
        pub pc: u16,
        pub dp: u16,
        pub k: u8,
        pub db: u8,
        pub p: u8,
        pub e: u8,
    }

    #[repr(C)]
    #[derive(Default)]
    pub struct Spc {
        pub a: u8,
        pub x: u8,
        pub y: u8,
        pub sp: u8,
        pub pc: u16,
        pub psw: u8,
        pub stopped: u8,
        pub in_ports: [u8; 4],
        pub out_ports: [u8; 4],
    }

    #[repr(C)]
    #[derive(Default, Clone, Copy)]
    pub struct Bg {
        pub h_scroll: u16,
        pub v_scroll: u16,
        pub tilemap_adr: u16,
        pub tile_adr: u16,
        pub tilemap_wider: u8,
        pub tilemap_higher: u8,
        pub big_tiles: u8,
        pub mosaic_enabled: u8,
    }

    #[repr(C)]
    #[derive(Default)]
    pub struct Ppu {
        pub bg: [Bg; 4],
        pub m7matrix: [i16; 8],
        pub vram_pointer: u16,
        pub vram_increment: u16,
        pub vram_read_buffer: u16,
        pub obj_tile_adr1: u16,
        pub obj_tile_adr2: u16,
        pub h_count: u16,
        pub v_count: u16,
        pub layer_main: [u8; 5],
        pub layer_sub: [u8; 5],
        pub layer_main_windowed: [u8; 5],
        pub layer_sub_windowed: [u8; 5],
        pub window_enabled1: [u8; 6],
        pub window_enabled2: [u8; 6],
        pub window_inversed1: [u8; 6],
        pub window_inversed2: [u8; 6],
        pub mask_logic: [u8; 6],
        pub math_enabled: [u8; 6],
        pub vram_increment_on_high: u8,
        pub vram_remap_mode: u8,
        pub cgram_pointer: u8,
        pub cgram_second_write: u8,
        pub cgram_buffer: u8,
        pub oam_adr: u8,
        pub oam_adr_written: u8,
        pub oam_in_high: u8,
        pub oam_in_high_written: u8,
        pub oam_second_write: u8,
        pub oam_buffer: u8,
        pub obj_priority: u8,
        pub obj_size: u8,
        pub time_over: u8,
        pub range_over: u8,
        pub obj_interlace: u8,
        pub scroll_prev: u8,
        pub scroll_prev2: u8,
        pub mosaic_size: u8,
        pub mosaic_start_line: u8,
        pub m7prev: u8,
        pub m7large_field: u8,
        pub m7char_fill: u8,
        pub m7x_flip: u8,
        pub m7y_flip: u8,
        pub m7ext_bg: u8,
        pub window1left: u8,
        pub window1right: u8,
        pub window2left: u8,
        pub window2right: u8,
        pub clip_mode: u8,
        pub prevent_math_mode: u8,
        pub add_subscreen: u8,
        pub subtract_color: u8,
        pub half_color: u8,
        pub fixed_r: u8,
        pub fixed_g: u8,
        pub fixed_b: u8,
        pub forced_blank: u8,
        pub brightness: u8,
        pub mode: u8,
        pub bg3priority: u8,
        pub even_frame: u8,
        pub pseudo_hires: u8,
        pub overscan: u8,
        pub frame_overscan: u8,
        pub interlace: u8,
        pub frame_interlace: u8,
        pub direct_color: u8,
        pub h_count_second: u8,
        pub v_count_second: u8,
        pub counters_latched: u8,
        pub ppu1open_bus: u8,
        pub ppu2open_bus: u8,
    }

    #[repr(C)]
    #[derive(Default)]
    pub struct Io {
        pub h_timer: u16,
        pub v_timer: u16,
        pub nmi_enabled: u8,
        pub h_irq_enabled: u8,
        pub v_irq_enabled: u8,
        pub auto_joy_read: u8,
        pub fast_mem: u8,
        pub hdma_enabled: u8,
        pub h_pos: u16,
        pub v_pos: u16,
    }

    // tools/oracle/build_lakesnes.sh builds the archive; it resolves at the final link.
    #[link(name = "oracle_lakesnes", kind = "static", modifiers = "-bundle")]
    unsafe extern "C" {
        pub fn oh_open(rom: *const u8, length: c_int) -> *mut Machine;
        pub fn oh_close(machine: *mut Machine);
        pub fn oh_set_recording(machine: *mut Machine, picture: bool, apu: bool);
        pub fn oh_step(machine: *mut Machine, pad: u16) -> c_int;
        pub fn oh_reset(machine: *mut Machine, hard: bool);
        pub fn oh_wram(machine: *mut Machine) -> *mut u8;
        pub fn oh_vram(machine: *mut Machine) -> *mut u16;
        pub fn oh_cgram(machine: *mut Machine) -> *mut u16;
        pub fn oh_oam(machine: *mut Machine) -> *mut u16;
        pub fn oh_high_oam(machine: *mut Machine) -> *mut u8;
        pub fn oh_sram(machine: *mut Machine, size: *mut c_int) -> *mut u8;
        pub fn oh_apu_ram(machine: *mut Machine) -> *mut u8;
        pub fn oh_dsp_registers(machine: *mut Machine) -> *mut u8;
        pub fn oh_facts(machine: *mut Machine) -> *const Facts;
        pub fn oh_apu_log(machine: *mut Machine, length: *mut u32) -> *const ApuAccess;
        pub fn oh_picture(machine: *mut Machine) -> *const u16;
        pub fn oh_line_brightness(machine: *mut Machine) -> *const u8;
        pub fn oh_line_forced_blank(machine: *mut Machine) -> *const u8;
        pub fn oh_ppu_written(machine: *mut Machine) -> *const u8;
        pub fn oh_io_written(machine: *mut Machine) -> *const u8;
        pub fn oh_set_wait_sites(machine: *mut Machine, pcs: *const u32, count: u32);
        pub fn oh_set_cpu(machine: *mut Machine, cpu: *const Cpu);
        pub fn oh_poke(machine: *mut Machine, adr: u32, value: u8);
        pub fn oh_set_beam(machine: *mut Machine, h: u16, v: u16);
        pub fn oh_set_spc(machine: *mut Machine, spc: *const Spc);
        pub fn oh_apu_write(machine: *mut Machine, adr: u16, value: u8);
        pub fn oh_dsp_write(machine: *mut Machine, adr: u8, value: u8);
        pub fn oh_set_profile(machine: *mut Machine, profile: bool);
        pub fn oh_bank_instructions(machine: *mut Machine) -> *const u32;
        pub fn oh_set_write_log(machine: *mut Machine, on: bool);
        pub fn oh_write_log(machine: *mut Machine, length: *mut u32) -> *const Write;
        pub fn oh_set_watch(machine: *mut Machine, pcs: *const u32, count: u32, peek: u32);
        pub fn oh_hits(machine: *mut Machine, length: *mut u32) -> *const Hit;
        pub fn oh_running_nmi(machine: *mut Machine) -> u32;
        pub fn oh_cpu(machine: *mut Machine, out: *mut Cpu);
        pub fn oh_spc(machine: *mut Machine, out: *mut Spc);
        pub fn oh_ppu(machine: *mut Machine, out: *mut Ppu);
        pub fn oh_io(machine: *mut Machine, out: *mut Io);
        pub fn oh_dma_registers(machine: *mut Machine, out: *mut [u8; 12]);
        pub fn oh_save_state(machine: *mut Machine, data: *mut u8) -> c_int;
        pub fn oh_load_state(machine: *mut Machine, data: *mut u8, size: c_int) -> bool;
    }
}

/// Upstream LakeSnes running the cartridge.
pub struct LakeSnes {
    machine: *mut ffi::Machine,
    tiers: Tiers,
    reset_pending: bool,
    record: FrameRecord,
}

impl Drop for LakeSnes {
    fn drop(&mut self) {
        // SAFETY: `machine` came from `oh_open` and is closed once, here.
        unsafe { ffi::oh_close(self.machine) };
    }
}

/// The bytes of `length` items at `pointer`, owned by the live machine.
///
/// # Safety
/// `pointer` must address `length` initialized items that live until the
/// returned slice is dropped, with no writer meanwhile.
unsafe fn view<'a, T>(pointer: *const T, length: usize) -> &'a [T] {
    // SAFETY: the caller's contract.
    unsafe { core::slice::from_raw_parts(pointer, length) }
}

impl LakeSnes {
    fn new(rom: &[u8]) -> Result<Self, OracleError> {
        let length = c_int::try_from(rom.len()).map_err(|_| OracleError::Rom)?;
        // SAFETY: `rom` is a live slice of `length` bytes; the bridge copies it.
        let machine = unsafe { ffi::oh_open(rom.as_ptr(), length) };
        if machine.is_null() {
            return Err(OracleError::Rom);
        }
        let mut oracle = Self {
            machine,
            tiers: Tiers::default(),
            reset_pending: false,
            record: FrameRecord {
                index: 0,
                video_frame: 0,
                pad: Pad::default(),
                cut: Cut::VblankWithoutNmi,
                joypad_reads: 0,
                overscan: false,
                wait_entry: None,
                cpu: CpuRegisters::default(),
                wram: vec![0; 0x2_0000].into_boxed_slice(),
                vram: vec![0; 0x8000].into_boxed_slice(),
                cgram: vec![0; 0x100].into_boxed_slice(),
                oam: vec![0; 0x220].into_boxed_slice(),
                sram: Box::default(),
                ppu: PpuRegisters::default(),
                apu_log: Vec::new(),
                picture: None,
                spc: None,
                bank_instructions: None,
                writes: Vec::new(),
                hits: Vec::new(),
                legacy_ppu_digest: 0,
            },
        };
        oracle.set_tiers(Tiers::default());
        let entries = crate::game::WAIT_SITES.map(|site| site.entry);
        // SAFETY: the machine is live; `entries` outlives the call, which copies it.
        unsafe { ffi::oh_set_wait_sites(machine, entries.as_ptr(), entries.len() as u32) };
        Ok(oracle)
    }

    fn load_state(&mut self, mut state: Vec<u8>) -> Result<(), OracleError> {
        let size = c_int::try_from(state.len()).map_err(|_| OracleError::State)?;
        // SAFETY: `state` is a live, exclusively borrowed buffer of `size` bytes.
        let loaded = unsafe { ffi::oh_load_state(self.machine, state.as_mut_ptr(), size) };
        loaded.then_some(()).ok_or(OracleError::State)
    }

    fn capture(&mut self) {
        let machine = self.machine;
        let record = &mut self.record;
        // SAFETY: the machine is between steps; every pointer addresses the
        // fixed-size buffer the bridge documents, and nothing writes it while
        // these views live.
        unsafe {
            let facts = &*ffi::oh_facts(machine);
            record.video_frame = facts.video_frame;
            record.pad = Pad(facts.pad_latched);
            record.joypad_reads = facts.polls;
            record.overscan = facts.overscan != 0;
            record.wait_entry = (facts.wait_v != 0xffff).then_some(Beam {
                v: facts.wait_v,
                h: facts.wait_h,
            });
            record.cut = if facts.ended_at_rti != 0 {
                let count = usize::try_from(facts.nmi_count).unwrap_or(usize::MAX);
                let last = facts.nmis[count.min(facts.nmis.len()) - 1];
                Cut::NmiReturn(Nmi {
                    kind: if crate::game::is_wait_site(last.interrupted_pc) {
                        NmiKind::Requested
                    } else {
                        NmiKind::Overrun
                    },
                    interrupted_pc: last.interrupted_pc,
                    polled: last.polled != 0,
                })
            } else {
                let running = ffi::oh_running_nmi(machine);
                if running == u32::MAX {
                    Cut::VblankWithoutNmi
                } else {
                    Cut::NmiStillRunning {
                        interrupted_pc: running,
                    }
                }
            };
            let mut cpu = ffi::Cpu::default();
            ffi::oh_cpu(machine, &mut cpu);
            record.cpu = CpuRegisters {
                a: cpu.a,
                x: cpu.x,
                y: cpu.y,
                s: cpu.sp,
                pc: cpu.pc,
                d: cpu.dp,
                k: cpu.k,
                db: cpu.db,
                p: cpu.p,
                emulation: cpu.e != 0,
            };
            record
                .wram
                .copy_from_slice(view(ffi::oh_wram(machine), 0x2_0000));
            record
                .vram
                .copy_from_slice(view(ffi::oh_vram(machine), 0x8000));
            record
                .cgram
                .copy_from_slice(view(ffi::oh_cgram(machine), 0x100));
            let (low, high) = record.oam.split_at_mut(0x200);
            for (bytes, word) in low
                .as_chunks_mut::<2>()
                .0
                .iter_mut()
                .zip(view(ffi::oh_oam(machine), 0x100))
            {
                bytes.copy_from_slice(&word.to_le_bytes());
            }
            high.copy_from_slice(view(ffi::oh_high_oam(machine), 0x20));
            let mut sram_size = 0;
            let sram = ffi::oh_sram(machine, &mut sram_size);
            let sram_size = usize::try_from(sram_size).unwrap_or(0);
            if record.sram.len() != sram_size {
                record.sram = vec![0; sram_size].into_boxed_slice();
            }
            if sram_size > 0 {
                record.sram.copy_from_slice(view(sram, sram_size));
            }
            let mut ppu = ffi::Ppu::default();
            ffi::oh_ppu(machine, &mut ppu);
            let mut io = ffi::Io::default();
            ffi::oh_io(machine, &mut io);
            let mut dma = [[0u8; 12]; 8];
            ffi::oh_dma_registers(machine, dma.as_mut_ptr());
            let written = view(ffi::oh_ppu_written(machine), 0x40);
            let io_written = view(ffi::oh_io_written(machine), 0x20);
            record.ppu = registers(&ppu, &io, &dma, written, io_written);
            record.legacy_ppu_digest = legacy_ppu_digest(&ppu, written);
            record.apu_log.clear();
            if self.tiers.apu_log {
                let mut length = 0;
                let log = ffi::oh_apu_log(machine, &mut length);
                if length > 0 {
                    let log = view(log, usize::try_from(length).unwrap_or(0));
                    record.apu_log.extend(log.iter().map(|access| ApuAccess {
                        kind: match access.kind {
                            0 => ApuAccessKind::CpuWrite,
                            1 => ApuAccessKind::CpuRead,
                            2 => ApuAccessKind::DmaWrite,
                            3 => ApuAccessKind::DmaRead,
                            _ => ApuAccessKind::SpcWrite,
                        },
                        port: access.port,
                        value: access.value,
                        apu_cycle: access.apu_cycle,
                    }));
                }
            }
            record.picture = self.tiers.picture.then(|| {
                let mut picture = record.picture.take().unwrap_or_default();
                picture.lines = if facts.overscan != 0 { MAX_LINES } else { 224 };
                picture
                    .dots
                    .copy_from_slice(view(ffi::oh_picture(machine), MAX_LINES * WIDTH));
                picture
                    .brightness
                    .copy_from_slice(view(ffi::oh_line_brightness(machine), MAX_LINES));
                for (blank, &raw) in picture
                    .forced_blank
                    .iter_mut()
                    .zip(view(ffi::oh_line_forced_blank(machine), MAX_LINES))
                {
                    *blank = raw != 0;
                }
                picture
            });
            record.writes.clear();
            if self.tiers.write_log {
                let mut length = 0;
                let log = ffi::oh_write_log(machine, &mut length);
                if length > 0 {
                    let log = view(log, usize::try_from(length).unwrap_or(0));
                    record.writes.extend(log.iter().map(|write| RegisterWrite {
                        beam: Beam {
                            v: write.v,
                            h: write.h,
                        },
                        register: write.adr,
                        value: write.value,
                        source: match write.source {
                            0 => WriteSource::Main,
                            1 => WriteSource::Nmi,
                            2 => WriteSource::Irq,
                            3 => WriteSource::Dma,
                            _ => WriteSource::Hdma,
                        },
                    }));
                }
            }
            record.hits.clear();
            let mut length = 0;
            let hits = ffi::oh_hits(machine, &mut length);
            if length > 0 {
                let hits = view(hits, usize::try_from(length).unwrap_or(0));
                record.hits.extend(hits.iter().map(|hit| Hit {
                    pc: hit.pc,
                    beam: Beam { v: hit.v, h: hit.h },
                    peek: hit.peek,
                }));
            }
            record.bank_instructions = self
                .tiers
                .profile
                .then(|| view(ffi::oh_bank_instructions(machine), 256).into());
            record.spc = self.tiers.apu_state.then(|| {
                let mut spc = ffi::Spc::default();
                ffi::oh_spc(machine, &mut spc);
                let mut dsp = [0; 0x80];
                dsp.copy_from_slice(view(ffi::oh_dsp_registers(machine), 0x80));
                SpcState {
                    a: spc.a,
                    x: spc.x,
                    y: spc.y,
                    sp: spc.sp,
                    pc: spc.pc,
                    psw: spc.psw,
                    stopped: spc.stopped != 0,
                    cpu_to_apu: spc.in_ports,
                    apu_to_cpu: spc.out_ports,
                    ram: view(ffi::oh_apu_ram(machine), 0x1_0000).into(),
                    dsp,
                }
            });
        }
    }
}

/// The architectural registers, rebuilt from LakeSnes's decoded PPU state
/// (which survives a state load) and the bridge's record of the last write.
fn registers(
    ppu: &ffi::Ppu,
    io: &ffi::Io,
    dma: &[[u8; 12]; 8],
    written: &[u8],
    io_written: &[u8],
) -> PpuRegisters {
    let bit = |value: u8, shift: u32| value << shift;
    let window_select = core::array::from_fn(|pair| {
        let nibble = |layer: usize| {
            bit(ppu.window_inversed1[layer], 0)
                | bit(ppu.window_enabled1[layer], 1)
                | bit(ppu.window_inversed2[layer], 2)
                | bit(ppu.window_enabled2[layer], 3)
        };
        nibble(pair * 2) | (nibble(pair * 2 + 1) << 4)
    });
    let screen = |flags: &[u8; 5]| {
        flags
            .iter()
            .enumerate()
            .fold(0u8, |mask, (layer, &on)| mask | (on << layer))
    };
    let math = ppu
        .math_enabled
        .iter()
        .enumerate()
        .fold(0u8, |mask, (layer, &on)| mask | (on << layer));
    let name_select =
        ((ppu.obj_tile_adr2.wrapping_sub(ppu.obj_tile_adr1) >> 9).wrapping_sub(8)) as u8 & 0x18;
    PpuRegisters {
        inidisp: bit(ppu.forced_blank, 7) | ppu.brightness,
        obsel: (ppu.obj_size << 5) | name_select | ((ppu.obj_tile_adr1 >> 13) as u8 & 7),
        bgmode: ppu.mode
            | bit(ppu.bg3priority, 3)
            | ppu
                .bg
                .iter()
                .enumerate()
                .fold(0, |mask, (layer, bg)| mask | (bg.big_tiles << (4 + layer))),
        mosaic: (ppu.mosaic_size.wrapping_sub(1) << 4)
            | ppu
                .bg
                .iter()
                .enumerate()
                .fold(0, |mask, (layer, bg)| mask | (bg.mosaic_enabled << layer)),
        bg_sc: core::array::from_fn(|layer| {
            let bg = &ppu.bg[layer];
            ((bg.tilemap_adr >> 8) as u8 & 0xfc) | bg.tilemap_wider | bit(bg.tilemap_higher, 1)
        }),
        bg12nba: ((ppu.bg[0].tile_adr >> 12) as u8) | (((ppu.bg[1].tile_adr >> 12) as u8) << 4),
        bg34nba: ((ppu.bg[2].tile_adr >> 12) as u8) | (((ppu.bg[3].tile_adr >> 12) as u8) << 4),
        bg_scroll: core::array::from_fn(|layer| [ppu.bg[layer].h_scroll, ppu.bg[layer].v_scroll]),
        m7sel: bit(ppu.m7large_field, 7)
            | bit(ppu.m7char_fill, 6)
            | bit(ppu.m7y_flip, 1)
            | ppu.m7x_flip,
        m7: core::array::from_fn(|index| ppu.m7matrix[index]),
        window_select,
        window_positions: [
            ppu.window1left,
            ppu.window1right,
            ppu.window2left,
            ppu.window2right,
        ],
        wbglog: (0..4).fold(0, |mask, layer| {
            mask | (ppu.mask_logic[layer] << (layer * 2))
        }),
        wobjlog: ppu.mask_logic[4] | (ppu.mask_logic[5] << 2),
        screens: [
            screen(&ppu.layer_main),
            screen(&ppu.layer_sub),
            screen(&ppu.layer_main_windowed),
            screen(&ppu.layer_sub_windowed),
        ],
        cgwsel: (ppu.clip_mode << 6)
            | (ppu.prevent_math_mode << 4)
            | bit(ppu.add_subscreen, 1)
            | ppu.direct_color,
        cgadsub: bit(ppu.subtract_color, 7) | bit(ppu.half_color, 6) | math,
        fixed_color: [ppu.fixed_r, ppu.fixed_g, ppu.fixed_b],
        setini: written[0x33],
        nmitimen: bit(io.nmi_enabled, 7)
            | bit(io.v_irq_enabled, 5)
            | bit(io.h_irq_enabled, 4)
            | io.auto_joy_read,
        htime: io.h_timer,
        vtime: io.v_timer,
        hdmaen: io_written[0x0c],
        memsel: io.fast_mem,
        dma: core::array::from_fn(|channel| core::array::from_fn(|index| dma[channel][index])),
    }
}

/// The existing lock's `ppu` stream: the old PPU struct's fields, hashed in
/// the order and with the types the old engine's `result_digests` used (git tag `archive/old-engine`). LakeSnes's
/// struct is the same upstream PPU; the snesrev renderer's register caches
/// (`windowsel`, `screenEnabled`, `screenWindowed`) are rebuilt from the
/// registers, and its renderer scratch (`objPixelBufferXX`) is zero.
fn legacy_ppu_digest(ppu: &ffi::Ppu, written: &[u8]) -> u64 {
    let b = |value: u8| value != 0;
    let d = &mut Digest::default();
    (b(ppu.range_over), b(ppu.time_over), b(ppu.even_frame)).hash(d);
    (ppu.brightness, b(ppu.forced_blank)).hash(d);
    let geometry = (
        b(ppu.interlace),
        b(ppu.obj_interlace),
        b(ppu.overscan),
        b(ppu.pseudo_hires),
    );
    (geometry, b(ppu.frame_interlace), b(ppu.frame_overscan)).hash(d);
    (
        ppu.clip_mode,
        ppu.prevent_math_mode,
        b(ppu.add_subscreen),
        b(ppu.direct_color),
    )
        .hash(d);
    let math: [bool; 6] = core::array::from_fn(|layer| b(ppu.math_enabled[layer]));
    (b(ppu.subtract_color), b(ppu.half_color), math).hash(d);
    (ppu.fixed_r, ppu.fixed_g, ppu.fixed_b).hash(d);
    (
        ppu.window1left,
        ppu.window1right,
        ppu.window2left,
        ppu.window2right,
    )
        .hash(d);
    let windowsel = u32::from(written[0x23])
        | (u32::from(written[0x24]) << 8)
        | (u32::from(written[0x25]) << 16);
    windowsel.hash(d);
    for layer in 0..6 {
        let windows = (
            b(ppu.window_enabled1[layer]),
            b(ppu.window_enabled2[layer]),
            b(ppu.window_inversed1[layer]),
        );
        (
            windows,
            b(ppu.window_inversed2[layer]),
            ppu.mask_logic[layer],
        )
            .hash(d);
    }
    (
        [written[0x2c], written[0x2d]],
        [written[0x2e], written[0x2f]],
    )
        .hash(d);
    for layer in 0..5 {
        let main = (b(ppu.layer_main[layer]), b(ppu.layer_main_windowed[layer]));
        (
            main,
            b(ppu.layer_sub[layer]),
            b(ppu.layer_sub_windowed[layer]),
        )
            .hash(d);
    }
    for bg in &ppu.bg {
        (
            b(bg.tilemap_wider),
            b(bg.tilemap_higher),
            bg.tilemap_adr,
            bg.tile_adr,
        )
            .hash(d);
        (
            bg.h_scroll,
            bg.v_scroll,
            b(bg.big_tiles),
            b(bg.mosaic_enabled),
        )
            .hash(d);
    }
    (
        ppu.mode,
        b(ppu.bg3priority),
        ppu.mosaic_size,
        ppu.mosaic_start_line,
    )
        .hash(d);
    (
        b(ppu.m7large_field),
        b(ppu.m7char_fill),
        b(ppu.m7x_flip),
        b(ppu.m7y_flip),
    )
        .hash(d);
    (b(ppu.m7ext_bg), ppu.m7matrix).hash(d);
    (
        ppu.h_count,
        ppu.v_count,
        b(ppu.h_count_second),
        b(ppu.v_count_second),
    )
        .hash(d);
    (b(ppu.counters_latched), ppu.ppu1open_bus, ppu.ppu2open_bus).hash(d);
    (
        ppu.cgram_pointer,
        b(ppu.cgram_second_write),
        ppu.cgram_buffer,
    )
        .hash(d);
    (
        ppu.vram_pointer,
        ppu.vram_increment,
        b(ppu.vram_increment_on_high),
    )
        .hash(d);
    (ppu.vram_remap_mode, ppu.vram_read_buffer).hash(d);
    (ppu.scroll_prev, ppu.scroll_prev2, ppu.m7prev).hash(d);
    (
        ppu.oam_adr,
        ppu.oam_adr_written,
        b(ppu.oam_in_high),
        b(ppu.oam_in_high_written),
    )
        .hash(d);
    (b(ppu.oam_second_write), ppu.oam_buffer, b(ppu.obj_priority)).hash(d);
    (ppu.obj_tile_adr1, ppu.obj_tile_adr2, ppu.obj_size).hash(d);
    [0u8; 19][..].hash(d);
    d.finish()
}

impl Oracle for LakeSnes {
    fn identity(&self) -> OracleId {
        IDENTITY
    }

    fn open(rom: &[u8], start: Start) -> Result<Self, OracleError> {
        let mut oracle = Self::new(rom)?;
        match start {
            Start::PowerOn { wram_fill } => {
                // SAFETY: the machine is live and between steps; WRAM is 128K.
                unsafe { core::slice::from_raw_parts_mut(ffi::oh_wram(oracle.machine), 0x2_0000) }
                    .fill(wram_fill);
            }
            Start::State(state) => oracle.load_state(state)?,
            Start::Capture(capture) => crate::seed::apply(&mut oracle, &capture)?,
        }
        Ok(oracle)
    }

    fn step(&mut self, pad: Pad) -> Result<&FrameRecord, OracleError> {
        if core::mem::take(&mut self.reset_pending) {
            // SAFETY: the machine is live and between steps.
            unsafe { ffi::oh_reset(self.machine, false) };
        }
        // SAFETY: the machine is live and between steps.
        unsafe { ffi::oh_step(self.machine, pad.0) };
        self.capture();
        self.record.index += 1;
        Ok(&self.record)
    }

    fn save_state(&self) -> Vec<u8> {
        // SAFETY: a null buffer asks only for the size; the second call fills
        // a buffer of that size.
        unsafe {
            let size = ffi::oh_save_state(self.machine, core::ptr::null_mut());
            let mut state = vec![0; usize::try_from(size).unwrap_or(0)];
            ffi::oh_save_state(self.machine, state.as_mut_ptr());
            state
        }
    }

    fn set_tiers(&mut self, tiers: Tiers) {
        self.tiers = tiers;
        // SAFETY: the machine is live and between steps.
        unsafe { ffi::oh_set_recording(self.machine, tiers.picture, tiers.apu_log) };
        // SAFETY: the machine is live and between steps.
        unsafe { ffi::oh_set_profile(self.machine, tiers.profile) };
        // SAFETY: the machine is live and between steps.
        unsafe { ffi::oh_set_write_log(self.machine, tiers.write_log) };
    }

    fn reset(&mut self) {
        self.reset_pending = true;
    }

    fn set_watch(&mut self, addresses: &[u32], peek: u32) {
        let count = u32::try_from(addresses.len().min(32)).unwrap_or(32);
        // SAFETY: the machine is live; the bridge copies at most 32 addresses.
        unsafe { ffi::oh_set_watch(self.machine, addresses.as_ptr(), count, peek) };
    }
}

/// Write access to a LakeSnes machine between steps, for loading a migration seed.
pub(crate) struct Raw<'a> {
    machine: *mut ffi::Machine,
    _borrow: core::marker::PhantomData<&'a mut LakeSnes>,
}

impl LakeSnes {
    pub(crate) fn raw(&mut self) -> Raw<'_> {
        Raw {
            machine: self.machine,
            _borrow: core::marker::PhantomData,
        }
    }
}

impl Raw<'_> {
    /// # Safety
    /// `pointer` must address `length` items of the live machine, which this
    /// borrow holds exclusively between steps.
    unsafe fn slice<T>(&mut self, pointer: *mut T, length: usize) -> &mut [T] {
        // SAFETY: the caller's contract.
        unsafe { core::slice::from_raw_parts_mut(pointer, length) }
    }

    pub fn wram(&mut self) -> &mut [u8] {
        // SAFETY: WRAM is 128K bytes of the live machine.
        unsafe { self.slice(ffi::oh_wram(self.machine), 0x2_0000) }
    }

    pub fn vram(&mut self) -> &mut [u16] {
        // SAFETY: VRAM is 32K words.
        unsafe { self.slice(ffi::oh_vram(self.machine), 0x8000) }
    }

    pub fn cgram(&mut self) -> &mut [u16] {
        // SAFETY: CGRAM is 256 words.
        unsafe { self.slice(ffi::oh_cgram(self.machine), 0x100) }
    }

    pub fn oam(&mut self) -> &mut [u16] {
        // SAFETY: low OAM is 256 words.
        unsafe { self.slice(ffi::oh_oam(self.machine), 0x100) }
    }

    pub fn high_oam(&mut self) -> &mut [u8] {
        // SAFETY: high OAM is 32 bytes.
        unsafe { self.slice(ffi::oh_high_oam(self.machine), 0x20) }
    }

    pub fn sram(&mut self) -> &mut [u8] {
        let mut size = 0;
        // SAFETY: the bridge reports the cartridge RAM and its size.
        unsafe {
            let sram = ffi::oh_sram(self.machine, &mut size);
            if sram.is_null() {
                return &mut [];
            }
            self.slice(sram, usize::try_from(size).unwrap_or(0))
        }
    }

    pub fn apu_ram(&mut self) -> &mut [u8] {
        // SAFETY: SPC RAM is 64K bytes.
        unsafe { self.slice(ffi::oh_apu_ram(self.machine), 0x1_0000) }
    }

    /// Write a CPU-bus register as the CPU would.
    pub fn poke(&mut self, address: u32, value: u8) {
        // SAFETY: the machine is live and between steps.
        unsafe { ffi::oh_poke(self.machine, address, value) }
    }

    pub fn set_cpu(&mut self, cpu: CpuRegisters) {
        let raw = ffi::Cpu {
            a: cpu.a,
            x: cpu.x,
            y: cpu.y,
            sp: cpu.s,
            pc: cpu.pc,
            dp: cpu.d,
            k: cpu.k,
            db: cpu.db,
            p: cpu.p,
            e: u8::from(cpu.emulation),
        };
        // SAFETY: the machine is live and between steps.
        unsafe { ffi::oh_set_cpu(self.machine, &raw) }
    }

    pub fn set_beam(&mut self, beam: Beam) {
        // SAFETY: the machine is live and between steps.
        unsafe { ffi::oh_set_beam(self.machine, beam.h, beam.v) }
    }

    pub fn set_spc(&mut self, spc: &SpcRegisters) {
        let raw = ffi::Spc {
            a: spc.a,
            x: spc.x,
            y: spc.y,
            sp: spc.sp,
            pc: spc.pc,
            psw: spc.psw,
            stopped: 0,
            in_ports: spc.cpu_to_apu,
            out_ports: spc.apu_to_cpu,
        };
        // SAFETY: the machine is live and between steps.
        unsafe { ffi::oh_set_spc(self.machine, &raw) }
    }

    /// Write an SPC700 I/O register $F0-$FF.
    pub fn apu_write(&mut self, address: u16, value: u8) {
        // SAFETY: the machine is live and between steps.
        unsafe { ffi::oh_apu_write(self.machine, address, value) }
    }

    pub fn dsp_write(&mut self, address: u8, value: u8) {
        // SAFETY: the machine is live and between steps.
        unsafe { ffi::oh_dsp_write(self.machine, address, value) }
    }
}

/// The SPC700 registers and ports a seed sets.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SpcRegisters {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub pc: u16,
    pub psw: u8,
    pub cpu_to_apu: [u8; 4],
    pub apu_to_cpu: [u8; 4],
}
