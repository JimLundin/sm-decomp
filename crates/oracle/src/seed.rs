//! Migration seeds: the lsnes (bsnes v085) room captures today's corpus
//! starts from (`target/seeds/<capture>/<NNNN>-<ROOM>-*`). They hold the
//! memories and a register CSV, but no SPC700, DSP or timer state, so no
//! emulator resumes them exactly. Harness-native seeds replace them.

use crate::OracleError;
use crate::lakesnes::{LakeSnes, SpcRegisters};
use crate::record::{Beam, CpuRegisters};
use std::collections::BTreeMap;
use std::path::Path;

/// One lsnes capture.
#[derive(Clone, Debug)]
pub struct Capture {
    pub wram: Vec<u8>,
    pub vram: Vec<u8>,
    pub cgram: Vec<u8>,
    pub oam: Vec<u8>,
    pub sram: Vec<u8>,
    pub apu_ram: Vec<u8>,
    /// The register CSV: lsnes's names and values.
    pub registers: BTreeMap<String, i64>,
}

impl Capture {
    /// Read capture `index` (1-based) from `directory`, whatever its room.
    pub fn read(directory: &Path, index: u32) -> Result<Self, String> {
        let prefix = format!("{index:04}-");
        let entries = std::fs::read_dir(directory)
            .map_err(|error| format!("{}: {error}", directory.display()))?;
        let mut stem = None;
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if let Some(base) = name
                .strip_prefix(&prefix)
                .and_then(|rest| rest.strip_suffix("-wram.bin"))
            {
                stem = Some(format!("{prefix}{base}"));
            }
        }
        let stem = stem.ok_or_else(|| format!("no capture {index} in {}", directory.display()))?;
        let file = |part: &str| {
            let path = directory.join(format!("{stem}-{part}"));
            std::fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))
        };
        let csv = String::from_utf8(file("registers.csv")?).map_err(|error| error.to_string())?;
        let registers = csv
            .lines()
            .skip(1)
            .filter_map(|line| {
                let (name, value) = line.split_once(',')?;
                Some((name.to_owned(), value.trim().parse().ok()?))
            })
            .collect();
        Ok(Self {
            wram: file("wram.bin")?,
            vram: file("vram.bin")?,
            cgram: file("cgram.bin")?,
            oam: file("oam.bin")?,
            sram: file("sram.bin")?,
            apu_ram: file("apuram.bin")?,
            registers,
        })
    }

    /// A register's value from the CSV.
    pub fn register(&self, name: &str) -> Option<i64> {
        self.registers.get(name).copied()
    }
}

/// LEGACY(seed): where a migration seed resumes the SPC700, which the lsnes
/// captures do not record: the sound driver's main loop, waiting on timer 0
/// (`$15BF MOV Y,$FD; BEQ`), with the stack and flags LakeSnes shows there
/// when it runs the driver from power-on. Harness-native seeds retire this.
const DRIVER_IDLE: SpcRegisters = SpcRegisters {
    a: 0,
    x: 0,
    y: 0,
    sp: 0xcf,
    pc: 0x15bf,
    psw: 0x02,
    cpu_to_apu: [0; 4],
    apu_to_cpu: [0; 4],
};

/// LEGACY(seed): the DSP's flags at a migration seed: echo writes off, so the
/// captured SPC RAM is kept, and not muted. The other DSP registers are not
/// captured and start from reset; the driver rewrites voices as it plays.
const DSP_FLAGS: u8 = 0x20;

/// The registers whose two writes form one value, low byte first.
const WORD_REGISTERS: [(u32, &str); 6] = [
    (0x211b, "ppu_m7a"),
    (0x211c, "ppu_m7b"),
    (0x211d, "ppu_m7c"),
    (0x211e, "ppu_m7d"),
    (0x211f, "ppu_m7x"),
    (0x2120, "ppu_m7y"),
];

/// The CPU I/O registers restored from their last written value; NMITIMEN last.
const IO_REGISTERS: [(u32, &str); 7] = [
    (0x4201, "io_4201"),
    (0x4207, "io_4207"),
    (0x4208, "io_4208"),
    (0x4209, "io_4209"),
    (0x420a, "io_420A"),
    (0x420d, "io_420D"),
    (0x4200, "io_4200"),
];

impl Capture {
    fn value(&self, name: &str) -> Result<i64, OracleError> {
        self.register(name)
            .ok_or_else(|| OracleError::Seed(format!("the capture has no {name}")))
    }

    fn flag(&self, name: &str) -> Result<u8, OracleError> {
        Ok(u8::from(self.value(name)? != 0))
    }

    fn bytes(&self, name: &str) -> Result<[u8; 2], OracleError> {
        let [low, high, ..] = self.value(name)?.to_le_bytes();
        Ok([low, high])
    }

    fn byte(&self, name: &str) -> Result<u8, OracleError> {
        Ok(self.bytes(name)?[0])
    }

    fn word(&self, name: &str) -> Result<u16, OracleError> {
        Ok(u16::from_le_bytes(self.bytes(name)?))
    }

    fn indexed(&self, name: &str, index: usize) -> Result<i64, OracleError> {
        self.value(&format!("{name}[{index}]"))
    }

    fn indexed_flag(&self, name: &str, index: usize) -> Result<u8, OracleError> {
        Ok(u8::from(self.indexed(name, index)? != 0))
    }

    /// `name[index]` shifted right by `shift`, as a register field.
    fn indexed_field(&self, name: &str, index: usize, shift: u32) -> Result<u8, OracleError> {
        Ok((self.indexed(name, index)? >> shift).to_le_bytes()[0])
    }

    /// The flags `name[0..count]` as a mask, bit `i` from entry `i`.
    fn mask(&self, name: &str, count: usize) -> Result<u8, OracleError> {
        (0..count).try_fold(0u8, |mask, index| {
            Ok(mask | (self.indexed_flag(name, index)? << index))
        })
    }

    /// One window select nibble: window 1 invert and enable, window 2 invert and enable.
    fn window(&self, layer: usize) -> Result<u8, OracleError> {
        Ok(self.indexed_flag("ppu_window1_invert", layer)?
            | (self.indexed_flag("ppu_window1_enabled", layer)? << 1)
            | (self.indexed_flag("ppu_window2_invert", layer)? << 2)
            | (self.indexed_flag("ppu_window2_enabled", layer)? << 3))
    }

    /// The PPU and CPU I/O register writes that rebuild the captured state,
    /// in an order that respects their latches.
    fn register_writes(&self) -> Result<Vec<(u32, u8)>, OracleError> {
        let mut writes = Vec::new();
        let brightness = self.byte("ppu_display_brightness")? & 0xf;
        writes.push((
            0x2100,
            (self.flag("ppu_display_disabled")? << 7) | brightness,
        ));
        let base = (self.value("ppu_oam_tdaddr")? >> 14).to_le_bytes()[0] & 7;
        let names = (self.byte("ppu_oam_nameselect")? & 3) << 3;
        writes.push((0x2101, (self.byte("ppu_oam_basesize")? << 5) | names | base));
        let [oam_low, oam_high] = self.bytes("ppu_oam_baseaddr")?;
        writes.push((0x2102, oam_low));
        writes.push((
            0x2103,
            (self.flag("ppu_oam_priority")? << 7) | (oam_high & 1),
        ));
        let tiles = (0..4).try_fold(0u8, |mask, layer| {
            Ok::<_, OracleError>(
                mask | (self.indexed_flag("ppu_bg_tilesize", layer)? << (4 + layer)),
            )
        })?;
        let mode = self.byte("ppu_bg_mode")? & 7;
        writes.push((0x2105, mode | (self.flag("ppu_bg3_priority")? << 3) | tiles));
        let mosaic = (self.byte("ppu_mosaic_size")? & 0xf) << 4;
        writes.push((0x2106, mosaic | self.mask("ppu_mosaic_enabled", 4)?));
        for (layer, address) in (0x2107..=0x210a).enumerate() {
            let tilemap = self.indexed_field("ppu_bg_scaddr", layer, 9)? & 0xfc;
            writes.push((
                address,
                tilemap | (self.indexed_field("ppu_bg_scsize", layer, 0)? & 3),
            ));
        }
        let nibble =
            |layer| Ok::<_, OracleError>(self.indexed_field("ppu_bg_tdaddr", layer, 13)? & 0xf);
        writes.push((0x210b, nibble(0)? | (nibble(1)? << 4)));
        writes.push((0x210c, nibble(2)? | (nibble(3)? << 4)));
        for (layer, address) in (0x210d..=0x2113).step_by(2).enumerate() {
            for (offset, name) in [(0, "ppu_bg_hofs"), (1, "ppu_bg_vofs")] {
                let [low, high, ..] = self.indexed(name, layer)?.to_le_bytes();
                writes.push((address + offset, low));
                writes.push((address + offset, high));
            }
        }
        let flips = (self.flag("ppu_mode7_vflip")? << 1) | self.flag("ppu_mode7_hflip")?;
        writes.push((0x211a, ((self.byte("ppu_mode7_repeat")? & 3) << 6) | flips));
        for (address, name) in WORD_REGISTERS {
            let [low, high] = self.bytes(name)?;
            writes.push((address, low));
            writes.push((address, high));
        }
        for (pair, address) in (0x2123..=0x2125).enumerate() {
            writes.push((
                address,
                self.window(pair * 2)? | (self.window(pair * 2 + 1)? << 4),
            ));
        }
        let positions = [
            "ppu_window1_left",
            "ppu_window1_right",
            "ppu_window2_left",
            "ppu_window2_right",
        ];
        for (address, name) in (0x2126..=0x2129).zip(positions) {
            writes.push((address, self.byte(name)?));
        }
        let logic = |layer| {
            self.indexed_field("ppu_window_mask", layer, 0)
                .map(|value| value & 3)
        };
        writes.push((
            0x212a,
            logic(0)? | (logic(1)? << 2) | (logic(2)? << 4) | (logic(3)? << 6),
        ));
        writes.push((0x212b, logic(4)? | (logic(5)? << 2)));
        let screens = [
            "ppu_bg_enabled",
            "ppu_bgsub_enabled",
            "ppu_window_enabled",
            "ppu_sub_window_enabled",
        ];
        for (address, name) in (0x212c..=0x212f).zip(screens) {
            writes.push((address, self.mask(name, 5)?));
        }
        let clip = ((self.byte("ppu_color_mask")? & 3) << 6)
            | ((self.byte("ppu_colorsub_mask")? & 3) << 4);
        let math = (self.flag("ppu_addsub_mode")? << 1) | self.flag("ppu_direct_color")?;
        writes.push((0x2130, clip | math));
        let color = (self.flag("ppu_color_mode")? << 7) | (self.flag("ppu_color_halve")? << 6);
        writes.push((0x2131, color | self.mask("ppu_color_enabled", 6)?));
        for (plane, name) in [
            (0x20, "ppu_color_r"),
            (0x40, "ppu_color_g"),
            (0x80, "ppu_color_b"),
        ] {
            writes.push((0x2132, plane | (self.byte(name)? & 0x1f)));
        }
        let geometry = (self.flag("ppu_pseudo_hires")? << 3)
            | (self.flag("ppu_overscan")? << 2)
            | (self.flag("ppu_oam_interlace")? << 1)
            | self.flag("ppu_interlace")?;
        writes.push((0x2133, (self.flag("ppu_mode7_extbg")? << 6) | geometry));
        let step = match self.value("ppu_vram_incsize")? {
            1 => 0,
            32 => 1,
            _ => 2,
        };
        let mapping = (self.byte("ppu_vram_mapping")? & 3) << 2;
        writes.push((
            0x2115,
            (self.flag("ppu_vram_incmode")? << 7) | mapping | step,
        ));
        let [vram_low, vram_high] = self.bytes("ppu_vram_addr")?;
        writes.push((0x2116, vram_low));
        writes.push((0x2117, vram_high));
        writes.push((0x2121, (self.word("ppu_cgram_addr")? >> 1).to_le_bytes()[0]));
        for channel in 0..8u32 {
            for index in 0..11u32 {
                let name = format!("dma_{channel}_{index}");
                writes.push((0x4300 + channel * 16 + index, self.byte(&name)?));
            }
        }
        for (address, name) in IO_REGISTERS {
            writes.push((address, self.byte(name)?));
        }
        Ok(writes)
    }
}

/// Copy `source` over `target`, which must be as long.
fn copy(target: &mut [u8], source: &[u8], what: &str) -> Result<(), OracleError> {
    if target.len() != source.len() {
        return Err(OracleError::Seed(format!(
            "{what}: {} bytes, expected {}",
            source.len(),
            target.len()
        )));
    }
    target.copy_from_slice(source);
    Ok(())
}

fn words(bytes: &[u8]) -> Vec<u16> {
    bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|&pair| u16::from_le_bytes(pair))
        .collect()
}

/// Load `capture` into a powered-on LakeSnes.
pub(crate) fn apply(oracle: &mut LakeSnes, capture: &Capture) -> Result<(), OracleError> {
    let writes = capture.register_writes()?;
    let mut raw = oracle.raw();
    raw.set_beam(Beam {
        h: capture.word("hcounter")?,
        v: capture.word("vcounter")?,
    });
    // Registers before memory: the VRAM address write prefetches a word.
    for (address, value) in writes {
        raw.poke(address, value);
    }
    copy(raw.wram(), &capture.wram, "WRAM")?;
    copy(raw.sram(), &capture.sram, "SRAM")?;
    copy(raw.apu_ram(), &capture.apu_ram, "SPC RAM")?;
    let (vram, cgram) = (words(&capture.vram), words(&capture.cgram));
    if vram.len() != 0x8000 || cgram.len() != 0x100 || capture.oam.len() != 0x220 {
        return Err(OracleError::Seed(
            "VRAM, CGRAM or OAM has the wrong size".into(),
        ));
    }
    raw.vram().copy_from_slice(&vram);
    raw.cgram().copy_from_slice(&cgram);
    let (low, high) = capture.oam.split_at(0x200);
    raw.oam().copy_from_slice(&words(low));
    raw.high_oam().copy_from_slice(high);
    // The SPC700's I/O registers come back from their RAM shadows.
    let shadow = |address: u16| capture.apu_ram[usize::from(address)];
    raw.apu_write(0xf1, shadow(0xf1) & 0x87);
    for address in [0xf2, 0xfa, 0xfb, 0xfc] {
        raw.apu_write(address, shadow(address));
    }
    raw.dsp_write(0x6c, DSP_FLAGS);
    let ports = [shadow(0xf4), shadow(0xf5), shadow(0xf6), shadow(0xf7)];
    raw.set_spc(&SpcRegisters {
        cpu_to_apu: ports,
        apu_to_cpu: ports,
        ..DRIVER_IDLE
    });
    // lsnes's exec hook ran during the opcode fetch at the boundary, after the
    // PC moved on but before PLB ran: resume at the hooked instruction.
    let resume = match capture.register("seed_boundary_pc") {
        Some(boundary) => boundary,
        None => (capture.value("pb")? << 16) | capture.value("pc")?,
    };
    raw.set_cpu(CpuRegisters {
        a: capture.word("a")?,
        x: capture.word("x")?,
        y: capture.word("y")?,
        s: capture.word("s")?,
        pc: u16::try_from(resume & 0xffff).unwrap_or_default(),
        d: capture.word("d")?,
        k: (resume >> 16).to_le_bytes()[0],
        db: capture.byte("db")?,
        p: capture.byte("p")?,
        emulation: capture.value("e")? != 0,
    });
    Ok(())
}
