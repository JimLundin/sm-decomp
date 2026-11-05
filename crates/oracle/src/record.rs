//! The frame record: one closed interval (cut k-1, cut k] of the machine.

/// The picture's width in dots.
pub const WIDTH: usize = 256;
/// The most lines a picture holds (an overscan frame).
pub const MAX_LINES: usize = 239;

/// A controller word in JOY1 order: B is bit 15, R is bit 4.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Pad(pub u16);

/// The NMI that ends a record's interval.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Nmi {
    pub kind: NmiKind,
    /// The 24-bit address the NMI interrupted.
    pub interrupted_pc: u32,
    /// Whether the interrupted address is not one of the game's wait sites:
    /// the main thread had not finished its frame.
    pub overrun: bool,
}

/// What the NMI handler did with the frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NmiKind {
    /// The handler read the controllers: the commit path of a requested frame.
    Requested,
    /// The handler did not read the controllers: a lag frame.
    Lag,
}

/// How a record's interval ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cut {
    /// At the RTI of the NMI serviced in this frame's vblank.
    NmiReturn(Nmi),
    /// At vblank start: NMI was disabled.
    VblankWithoutNmi,
    /// At the end of vblank, with the NMI handler still running.
    NmiStillRunning { interrupted_pc: u32 },
}

/// A beam position: the line (V) and the master-clock dot (H, 0-1363).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Beam {
    pub v: u16,
    pub h: u16,
}

/// The 65816 registers at the cut.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CpuRegisters {
    pub a: u16,
    pub x: u16,
    pub y: u16,
    pub s: u16,
    pub pc: u16,
    pub d: u16,
    pub k: u8,
    pub db: u8,
    pub p: u8,
    pub emulation: bool,
}

/// The architectural PPU and CPU I/O registers in effect at the cut, as the
/// values the game wrote: no latches, open bus or renderer scratch.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PpuRegisters {
    /// $2100
    pub inidisp: u8,
    /// $2101
    pub obsel: u8,
    /// $2105
    pub bgmode: u8,
    /// $2106
    pub mosaic: u8,
    /// $2107-$210A
    pub bg_sc: [u8; 4],
    /// $210B
    pub bg12nba: u8,
    /// $210C
    pub bg34nba: u8,
    /// $210D-$2114, horizontal then vertical per layer, 10 bits each.
    pub bg_scroll: [[u16; 2]; 4],
    /// $211A
    pub m7sel: u8,
    /// $211B-$2120: A, B, C, D, X, Y.
    pub m7: [i16; 6],
    /// $2123-$2125
    pub window_select: [u8; 3],
    /// $2126-$2129
    pub window_positions: [u8; 4],
    /// $212A
    pub wbglog: u8,
    /// $212B
    pub wobjlog: u8,
    /// $212C-$212F: TM, TS, TMW, TSW.
    pub screens: [u8; 4],
    /// $2130
    pub cgwsel: u8,
    /// $2131
    pub cgadsub: u8,
    /// $2132 as three 5-bit components: red, green, blue.
    pub fixed_color: [u8; 3],
    /// $2133
    pub setini: u8,
    /// $4200
    pub nmitimen: u8,
    /// $4207-$4208
    pub htime: u16,
    /// $4209-$420A
    pub vtime: u16,
    /// $420C, as last written.
    pub hdmaen: u8,
    /// $420D
    pub memsel: u8,
    /// $43x0-$43xA for each channel.
    pub dma: [[u8; 11]; 8],
}

/// One CPU or DMA access to the APU ports $2140-$2143.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ApuAccess {
    pub kind: ApuAccessKind,
    pub port: u8,
    pub value: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ApuAccessKind {
    CpuWrite,
    CpuRead,
    DmaWrite,
    DmaRead,
}

/// The picture scanned out in a record's interval, in the PPU's domain:
/// 15-bit BGR after colour math, before the master brightness.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Picture {
    /// 224, or 239 on an overscan frame.
    pub lines: usize,
    /// `MAX_LINES` rows of `WIDTH` BGR555 dots; rows past `lines` are unused.
    pub dots: Box<[u16]>,
    /// Each line's master brightness (INIDISP bits 0-3).
    pub brightness: [u8; MAX_LINES],
    /// Each line's forced blank (INIDISP bit 7).
    pub forced_blank: [bool; MAX_LINES],
}

impl Default for Picture {
    fn default() -> Self {
        Self {
            lines: 224,
            dots: vec![0; MAX_LINES * WIDTH].into_boxed_slice(),
            brightness: [0; MAX_LINES],
            forced_blank: [false; MAX_LINES],
        }
    }
}

impl Picture {
    /// The scanned-out rows.
    pub fn rows(&self) -> impl Iterator<Item = &[u16]> {
        self.dots
            .as_chunks::<WIDTH>()
            .0
            .iter()
            .map(|row| &row[..])
            .take(self.lines)
    }
}

/// The APU side at the cut (tier b).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpcState {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub pc: u16,
    pub psw: u8,
    pub stopped: bool,
    /// $2140-$2143 as the SPC700 reads them.
    pub cpu_to_apu: [u8; 4],
    /// $2140-$2143 as the CPU reads them.
    pub apu_to_cpu: [u8; 4],
    pub ram: Box<[u8]>,
    pub dsp: [u8; 0x80],
}

/// One video frame of the machine: the closed interval (cut k-1, cut k].
#[derive(Clone, Debug)]
pub struct FrameRecord {
    /// The frame's index from the start of the run, from 0.
    pub index: u64,
    /// The emulator's video frame counter at vblank k.
    pub video_frame: u32,
    /// The pad word auto-joypad read at vblank k.
    pub pad: Pad,
    /// How the interval ended.
    pub cut: Cut,
    /// Joypad register reads ($4218-$421F) in the interval.
    pub joypad_reads: u32,
    /// The beam position at which the main thread entered a wait-for-NMI
    /// routine in the interval: where it finished its frame.
    pub wait_entry: Option<Beam>,
    /// Whether vblank k began at line 240.
    pub overscan: bool,
    pub cpu: CpuRegisters,
    pub wram: Box<[u8]>,
    /// 32K words.
    pub vram: Box<[u16]>,
    pub cgram: Box<[u16]>,
    /// 256 words of low OAM followed by the 32 bytes of high OAM.
    pub oam: Box<[u8]>,
    pub sram: Box<[u8]>,
    pub ppu: PpuRegisters,
    /// The APU port accesses of the interval, in order (tier).
    pub apu_log: Vec<ApuAccess>,
    /// The picture scanned out in the interval: the one NMI k-1 committed (tier).
    pub picture: Option<Picture>,
    /// The APU at the cut (tier).
    pub spc: Option<SpcState>,
    /// Backend detail kept only to compare with the existing lock's `ppu`
    /// stream, which hashed the old PPU's struct fields.
    pub legacy_ppu_digest: u64,
}

impl FrameRecord {
    /// The NMI that ended the interval, if one returned.
    pub fn nmi(&self) -> Option<Nmi> {
        match self.cut {
            Cut::NmiReturn(nmi) => Some(nmi),
            Cut::VblankWithoutNmi | Cut::NmiStillRunning { .. } => None,
        }
    }
}
