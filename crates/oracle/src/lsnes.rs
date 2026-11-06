//! The lsnes backend: lsnes rr2-beta23 (the bsnes v085 core that recorded the
//! movies and seeds) as a separate process, driven by
//! `tools/oracle/lsnes_server.lua` over its standard input and a record file.
//! No emulator code is linked or copied (docs/oracle/HARNESS.md).
//!
//! The cut is the instruction the NMI interrupted, reached again with the
//! stack back where the NMI found it: the RTI has just completed. A frame in
//! which no NMI ran is cut at lsnes's frame boundary (line 241).
//!
//! An NMI that outlasts that boundary returns early in lsnes's next frame,
//! whose input lsnes commits at its start. So that frame takes the pad given
//! to [`Lsnes::preview`] before the step, and the next step must bring that
//! pad. Without a preview such a step is cut at the boundary with its NMI
//! still running ([`Cut::NmiStillRunning`]), before the NMI's counter writes
//! and joypad read.
//!
//! With the picture or APU log tiers on ([`Oracle::set_tiers`] before the
//! first step), each record also carries the picture (the screenshot of the
//! video frame the step ends in, sent as lsnes's PNG and mapped back from its
//! 8-bit palette to BGR555 at the one brightness that produces it), the CPU's
//! accesses to
//! $2140-$2143 (no DMA or SPC700 side, no APU cycle) and the PPU registers
//! (rebuilt from bsnes's decoded fields, with the DMA channels; `nmitimen`,
//! `htime`, `vtime`, `hdmaen` and `memsel` stay at their defaults).
//!
//! Not recorded by this backend: the SPC state (lsnes exposes SPC700 and DSP
//! only as raw core structs), the instruction profile, the PPU register write
//! log, watched-PC hits, the per-line forced blank, `overscan` and the old
//! lock's `legacy_ppu_digest`.
//!
//! A saved state is lsnes's own, taken at the frame boundary that ends the
//! step it was requested before: after that step's cut, or inside its NMI
//! when the NMI outlasts the boundary. Either way it lies before the next
//! step's NMI, so a resumed oracle's first step reproduces the next cut; its
//! counts restart where the straight run's do (at the RTI of an NMI the
//! state was saved inside).

use crate::record::{
    ApuAccess, ApuAccessKind, Beam, CpuRegisters, Cut, FrameRecord, MAX_LINES, Nmi, NmiKind, Pad,
    Picture, PpuRegisters, WIDTH,
};
use crate::{Oracle, OracleError, OracleId, Start, Tiers};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

/// The server script, written next to the run's files at start.
const SERVER: &str = include_str!("../../../tools/oracle/lsnes_server.lua");
/// The record header: magic through the memory flag.
const HEADER: usize = 43;
const WRAM: usize = 0x2_0000;
const VRAM: usize = 0x1_0000;
const CGRAM: usize = 0x200;
const OAM: usize = 0x220;
const SRAM: usize = 0x2000;
/// lsnes's standard error and the server's Lua error log, in the run's
/// directory. lsnes runs there, so its own `lsnes.log` (where a Lua syntax
/// error goes; lsnes then keeps running without a script) lands there too.
const STDERR: &str = "stderr.txt";
const LUA_LOG: &str = "lua.log";
/// lsnes's configuration directory, inside the run's directory. lsnes takes
/// it from `APPDATA` (`%APPDATA%/lsnes`); a shared one races when several
/// processes start in the same second, and the loser stops at a fatal
/// "not writable" message without exiting.
const CONFIG: &str = "config";
/// How long the first record may take: lsnes starts, loads the ROM and the
/// server, and runs a frame.
const START_TIMEOUT: Duration = Duration::from_secs(60);
/// How long any later record may take.
const STEP_TIMEOUT: Duration = Duration::from_secs(30);
/// How long lsnes may take to quit once asked.
const QUIT_TIMEOUT: Duration = Duration::from_secs(5);

/// The executable: `SM_LSNES`, or the copy this machine keeps.
fn executable() -> PathBuf {
    std::env::var_os("SM_LSNES").map_or_else(
        || {
            PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap_or_default()).join(
                "sm-decomp-experiments/20261025/tas-alignment/lsnes-rr2-beta23/lsnes-bsnes.exe",
            )
        },
        PathBuf::from,
    )
}

/// Why an lsnes process stopped serving.
#[derive(Debug)]
pub enum Cause {
    /// It closed its standard output: it exited or crashed.
    Exited,
    /// It sent no record within this long, and was killed.
    Silent(Duration),
    /// A request could not be written to its standard input.
    Request(std::io::Error),
    /// Its standard output could not be read.
    Reply(std::io::Error),
    /// Its announcement or record file was malformed.
    Record(String),
}

impl core::fmt::Display for Cause {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Exited => write!(f, "exited"),
            Self::Silent(waited) => write!(f, "sent no record in {}s; killed", waited.as_secs()),
            Self::Request(error) => write!(f, "took no request: {error}"),
            Self::Reply(error) => write!(f, "reply unreadable: {error}"),
            Self::Record(what) => write!(f, "bad record: {what}"),
        }
    }
}

/// An lsnes process that stopped serving, with the evidence it left.
#[derive(Debug)]
pub struct Failure {
    pub cause: Cause,
    /// The step it was asked for.
    pub step: u64,
    /// Its process id.
    pub pid: u32,
    /// Its exit status and the tails of its logs; its directory is kept.
    pub report: String,
}

impl core::fmt::Display for Failure {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "lsnes (pid {}) {} at step {}: {}",
            self.pid, self.cause, self.step, self.report
        )
    }
}

/// One lsnes process running the original game.
pub struct Lsnes {
    child: Child,
    requests: ChildStdin,
    /// Its standard output, a line at a time, from a reader thread, so that a
    /// silent lsnes can be timed out. Disconnected once lsnes closes it.
    replies: Receiver<std::io::Result<String>>,
    /// A failure met outside a step (by `reset`), returned by the next step.
    failed: Option<Failure>,
    directory: PathBuf,
    record: FrameRecord,
    steps: u64,
    saves: u32,
    raw: Vec<u8>,
    /// Keep the run's directory at the end: lsnes exited early, or
    /// `SM_LSNES_KEEP` is set.
    keep: bool,
    /// Whether each record carries the memories (on by default; the streams
    /// need them, and leaving them out roughly doubles the speed).
    pub memories: bool,
    /// The pad of the step after the coming one, from [`Lsnes::preview`].
    next_pad: Option<Pad>,
}

/// A little-endian word to write at a work-RAM offset (`$7E:0000` + offset).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WramWord {
    pub offset: u32,
    pub value: u16,
}

impl core::str::FromStr for WramWord {
    type Err = String;

    /// `OFFSET=VALUE`, both hexadecimal: `09C2=0001`.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let bad = || format!("{text}: expected OFFSET=VALUE in hexadecimal");
        let (offset, value) = text.split_once('=').ok_or_else(bad)?;
        let offset = u32::from_str_radix(offset, 16).map_err(|_| bad())?;
        let value = u16::from_str_radix(value, 16).map_err(|_| bad())?;
        if usize::try_from(offset).map_or(true, |offset| offset + 1 >= WRAM) {
            return Err(format!("{text}: the word lies outside work RAM"));
        }
        Ok(Self { offset, value })
    }
}

impl WramWord {
    /// The word at this offset in a work-RAM image.
    pub fn read(self, wram: &[u8]) -> u16 {
        let at = usize::try_from(self.offset).unwrap_or(usize::MAX);
        wram.get(at..)
            .and_then(<[u8]>::first_chunk::<2>)
            .map_or(0, |pair| u16::from_le_bytes(*pair))
    }
}

/// A harness-native seed: an lsnes state and the work-RAM words written right
/// after it loads. A scenario seed is its base's state with the scenario's
/// edits: lsnes saves before it loads at a frame boundary, and a save asked for
/// after the load waits for the next boundary, so an edited state cannot be
/// saved at the base's own boundary.
#[derive(Clone, Debug, Default)]
pub struct NativeSeed {
    pub state: Vec<u8>,
    pub words: Vec<WramWord>,
}

impl NativeSeed {
    /// The edits beside a seed's state: `NNNN.words`, one `OFFSET=VALUE` a line.
    fn words_path(path: &Path) -> PathBuf {
        path.with_extension("words")
    }

    /// The seed whose state is `path` (`NNNN.lss`), with its edits if it has any.
    pub fn read(path: &Path) -> Result<Self, String> {
        let state = std::fs::read(path)
            .map_err(|error| format!("native seed {}: {error}", path.display()))?;
        let words_path = Self::words_path(path);
        let words = match std::fs::read_to_string(&words_path) {
            Ok(text) => text
                .lines()
                .filter(|line| !line.is_empty())
                .map(str::parse)
                .collect::<Result<_, _>>()
                .map_err(|error| format!("{}: {error}", words_path.display()))?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(error) => return Err(format!("{}: {error}", words_path.display())),
        };
        Ok(Self { state, words })
    }

    /// Write the state to `path` and the edits beside it.
    pub fn write(&self, path: &Path) -> Result<(), String> {
        let words: String = self
            .words
            .iter()
            .map(|word| {
                format!(
                    "{:04X}={:04X}
",
                    word.offset, word.value
                )
            })
            .collect();
        let words_path = Self::words_path(path);
        std::fs::write(path, &self.state)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        std::fs::write(&words_path, words)
            .map_err(|error| format!("{}: {error}", words_path.display()))
    }

    /// Ask a just-opened oracle resumed from this seed's state for its edits.
    pub fn apply(&self, lsnes: &mut Lsnes) -> Result<(), OracleError> {
        self.words
            .iter()
            .try_for_each(|&word| lsnes.write_wram(word))
    }
}

/// lsnes rr2-beta23 with the bsnes v085 core.
pub const IDENTITY: OracleId = OracleId {
    emulator: "lsnes-bsnes",
    version: "rr2-beta23 (bsnes v085)",
    patches: &[],
    fixes: &[],
};

/// The memories after the header, when the record carries them.
const MEMORIES: usize = WRAM + VRAM + CGRAM + OAM + SRAM;

/// The record's optional streams, appended after the memories when a `T`
/// request asked for any (`tools/oracle/lsnes_server.lua`).
const STREAMS_MAGIC: &[u8; 4] = b"OHS1";
/// The PPU block: `PpuRegisters` from `inidisp` to `setini` as the bytes the
/// game wrote (scroll and mode 7 words little-endian), then `$43x0-$43xA`.
const PPU_BYTES: usize = 58 + 88;

/// One master-brightness level's 8-bit channel values (lsnes's lrgb
/// palette: `l * c * 255 / 496` rounded, `l` brightness plus one) mapped
/// back to the 5-bit component; ambiguous values keep the smallest.
fn channel_inverse(level: u32) -> [Option<u8>; 256] {
    let mut inverse = [None; 256];
    for component in (0..32u8).rev() {
        let scaled = level * u32::from(component);
        let value = ((scaled << 8) - scaled + 248) / 496;
        if let Some(slot) = usize::try_from(value)
            .ok()
            .and_then(|at| inverse.get_mut(at))
        {
            *slot = Some(component);
        }
    }
    inverse
}

/// The picture from lsnes's decoded screenshot: the brightest level whose
/// palette produces every channel value, its inverse as BGR555 dots. A
/// hires or interlaced screenshot keeps its even dots and lines.
fn decode_picture(width: usize, height: usize, rgb: &[u8]) -> Picture {
    let mut picture = Picture::default();
    let (step_x, step_y) = (width / WIDTH.max(1), if height > MAX_LINES { 2 } else { 1 });
    picture.lines = (height / step_y).min(MAX_LINES);
    let level = (1..=16u32)
        .rev()
        .map(|level| (level, channel_inverse(level)))
        .find(|(_, inverse)| {
            rgb.iter()
                .all(|&value| inverse[usize::from(value)].is_some())
        });
    let Some((level, inverse)) = level else {
        return picture;
    };
    let brightness = u8::try_from(level - 1).unwrap_or(0);
    picture.brightness.fill(brightness);
    for (line, row) in picture
        .dots
        .chunks_mut(WIDTH)
        .take(picture.lines)
        .enumerate()
    {
        for (x, dot) in row.iter_mut().enumerate() {
            let at = 3 * ((line * step_y * width) + (x * step_x.max(1)));
            let Some([r, g, b]) = rgb
                .get(at..at + 3)
                .and_then(|part| <[u8; 3]>::try_from(part).ok())
            else {
                continue;
            };
            let component = |value: u8| u16::from(inverse[usize::from(value)].unwrap_or(0));
            *dot = component(r) | (component(g) << 5) | (component(b) << 10);
        }
    }
    picture
}

fn decode_ppu(bytes: &[u8]) -> PpuRegisters {
    let mut at = 0;
    let mut byte = || {
        let value = bytes[at];
        at += 1;
        value
    };
    let mut ppu = PpuRegisters {
        inidisp: byte(),
        obsel: byte(),
        bgmode: byte(),
        mosaic: byte(),
        ..PpuRegisters::default()
    };
    ppu.bg_sc = [byte(), byte(), byte(), byte()];
    ppu.bg12nba = byte();
    ppu.bg34nba = byte();
    for layer in &mut ppu.bg_scroll {
        for value in layer.iter_mut() {
            *value = u16::from_le_bytes([byte(), byte()]);
        }
    }
    ppu.m7sel = byte();
    for value in &mut ppu.m7 {
        *value = i16::from_le_bytes([byte(), byte()]);
    }
    ppu.window_select = [byte(), byte(), byte()];
    ppu.window_positions = [byte(), byte(), byte(), byte()];
    ppu.wbglog = byte();
    ppu.wobjlog = byte();
    ppu.screens = [byte(), byte(), byte(), byte()];
    ppu.cgwsel = byte();
    ppu.cgadsub = byte();
    ppu.fixed_color = [byte(), byte(), byte()];
    ppu.setini = byte();
    for channel in &mut ppu.dma {
        for value in channel.iter_mut() {
            *value = byte();
        }
    }
    ppu
}

fn le16(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}

fn le32(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

fn empty_record() -> FrameRecord {
    FrameRecord {
        index: 0,
        video_frame: 0,
        pad: Pad(0),
        cut: Cut::VblankWithoutNmi,
        joypad_reads: 0,
        wait_entry: None,
        overscan: false,
        cpu: CpuRegisters::default(),
        wram: vec![0; WRAM].into_boxed_slice(),
        vram: vec![0; VRAM / 2].into_boxed_slice(),
        cgram: vec![0; CGRAM / 2].into_boxed_slice(),
        oam: vec![0; OAM].into_boxed_slice(),
        sram: vec![0; SRAM].into_boxed_slice(),
        ppu: PpuRegisters::default(),
        apu_log: Vec::new(),
        picture: None,
        spc: None,
        bank_instructions: None,
        writes: Vec::new(),
        hits: Vec::new(),
        legacy_ppu_digest: 0,
    }
}

impl Lsnes {
    fn directory() -> PathBuf {
        let unique = format!(
            "oracle-lsnes-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |elapsed| elapsed.as_nanos())
        );
        std::env::temp_dir().join(unique)
    }

    fn send(&mut self, line: &str) -> Result<(), OracleError> {
        writeln!(self.requests, "{line}")
            .and_then(|()| self.requests.flush())
            .map_err(|error| self.fail(Cause::Request(error)))
    }

    /// Announce the pad of the step after the coming one, so that the coming
    /// step is cut at its NMI's RTI even when the NMI outlasts lsnes's frame.
    /// The step after must then bring this pad, and no reset before it.
    pub fn preview(&mut self, next: Pad) {
        self.next_pad = Some(next);
    }

    /// Record the picture and the APU port log as `tiers` asks, and the PPU
    /// registers with them, from the next step on. Ask before the first
    /// step: the server registers the port callbacks only then.
    pub fn record_streams(&mut self, tiers: Tiers) -> Result<(), OracleError> {
        let request = format!(
            "T {} {} 1",
            u8::from(tiers.picture),
            u8::from(tiers.apu_log)
        );
        self.send(&request)
    }

    /// Ask for the state at the frame boundary that ends the coming step,
    /// written to the returned file once the step after it has run;
    /// [`Oracle::save_state`] then reads it.
    pub fn request_save(&mut self) -> Result<PathBuf, OracleError> {
        self.saves += 1;
        let path = self.directory.join(format!("state-{}.lss", self.saves));
        self.send(&format!("V {}", path.display()))?;
        Ok(path)
    }

    /// Write `edit` to work RAM. Asked for before a resumed run's first step,
    /// it lands right after the start state loads, at that state's boundary.
    pub fn write_wram(&mut self, edit: WramWord) -> Result<(), OracleError> {
        self.send(&format!("W {:x} {:x}", edit.offset, edit.value))
    }

    /// End a process that stopped serving, and say why. One still alive
    /// (silent, or refusing requests) is killed first, so none outlives its run.
    fn fail(&mut self, cause: Cause) -> OracleError {
        // Killing a process that already exited fails harmlessly.
        let _ = self.child.kill();
        OracleError::Lsnes(Failure {
            cause,
            step: self.steps,
            pid: self.child.id(),
            report: self.exit_report(),
        })
    }

    /// Why lsnes ended: its exit status, the Lua errors the server logged and
    /// the end of its standard error. The run's directory is kept for a look.
    fn exit_report(&mut self) -> String {
        let status = self
            .child
            .wait()
            .map_or_else(|error| error.to_string(), |status| status.to_string());
        let tail = |name: &str| {
            // lsnes.log holds device names that are not UTF-8.
            let bytes = std::fs::read(self.directory.join(name)).unwrap_or_default();
            let text = String::from_utf8_lossy(&bytes);
            let lines: Vec<&str> = text.lines().collect();
            lines[lines.len().saturating_sub(20)..].join("\n")
        };
        let report = format!(
            "{status}; directory {}\nLua log:\n{}\nstderr:\n{}\nlsnes.log:\n{}",
            self.directory.display(),
            tail(LUA_LOG),
            tail(STDERR),
            tail("lsnes.log")
        );
        self.keep = true;
        report
    }

    /// The last record as the server wrote it.
    pub fn raw(&self) -> &[u8] {
        &self.raw
    }

    fn decode(&mut self, bytes: &[u8]) -> Result<(), Cause> {
        if bytes.len() < HEADER || &bytes[..4] != b"OHR1" {
            return Err(Cause::Record(format!("no header in {} bytes", bytes.len())));
        }
        let dumped = bytes[42] != 0;
        if dumped && bytes.len() < HEADER + MEMORIES {
            return Err(Cause::Record(format!(
                "{} bytes, short of the memories",
                bytes.len()
            )));
        }
        let record = &mut self.record;
        let interrupted_pc = le32(bytes, 5);
        let polled = bytes[9] != 0;
        record.cut = match bytes[4] {
            0 => Cut::NmiReturn(Nmi {
                kind: if crate::game::is_wait_site(interrupted_pc) {
                    NmiKind::Requested
                } else {
                    NmiKind::Overrun
                },
                interrupted_pc,
                polled,
            }),
            1 => Cut::VblankWithoutNmi,
            2 => Cut::NmiStillRunning { interrupted_pc },
            other => return Err(Cause::Record(format!("kind {other}"))),
        };
        record.joypad_reads = le32(bytes, 10);
        record.wait_entry = (bytes[14] != 0).then(|| Beam {
            v: le16(bytes, 15),
            h: le16(bytes, 17),
        });
        record.overscan = bytes[19] != 0;
        record.video_frame = le32(bytes, 20);
        record.pad = Pad(le16(bytes, 24));
        record.cpu = CpuRegisters {
            a: le16(bytes, 26),
            x: le16(bytes, 28),
            y: le16(bytes, 30),
            s: le16(bytes, 32),
            pc: le16(bytes, 34),
            d: le16(bytes, 36),
            k: bytes[38],
            db: bytes[39],
            p: bytes[40],
            emulation: bytes[41] != 0,
        };
        let mut at = HEADER;
        if dumped {
            let mut take = |size: usize| {
                let part = &bytes[at..at + size];
                at += size;
                part
            };
            record.wram.copy_from_slice(take(WRAM));
            for (word, pair) in record.vram.iter_mut().zip(take(VRAM).as_chunks::<2>().0) {
                *word = u16::from_le_bytes(*pair);
            }
            for (word, pair) in record.cgram.iter_mut().zip(take(CGRAM).as_chunks::<2>().0) {
                *word = u16::from_le_bytes(*pair);
            }
            record.oam.copy_from_slice(take(OAM));
            record.sram.copy_from_slice(take(SRAM));
        }
        Self::decode_streams(record, &bytes[at..])
    }

    /// The optional streams after the memories (`STREAMS_MAGIC`).
    fn decode_streams(record: &mut FrameRecord, bytes: &[u8]) -> Result<(), Cause> {
        record.apu_log.clear();
        record.picture = None;
        if bytes.is_empty() {
            return Ok(());
        }
        let short = || Cause::Record(format!("streams cut short at {} bytes", bytes.len()));
        let Some((STREAMS_MAGIC, rest)) = bytes.split_first_chunk::<4>() else {
            return Err(Cause::Record("no streams header".into()));
        };
        let (&flags, mut rest) = rest.split_first().ok_or_else(short)?;
        if flags & 4 != 0 {
            let (ppu, after) = rest.split_at_checked(PPU_BYTES).ok_or_else(short)?;
            record.ppu = decode_ppu(ppu);
            rest = after;
        }
        if flags & 2 != 0 {
            let (count, after) = rest.split_first_chunk::<4>().ok_or_else(short)?;
            let count = usize::try_from(u32::from_le_bytes(*count)).map_err(|_| short())?;
            let (log, after) = after
                .split_at_checked(count.saturating_mul(3))
                .ok_or_else(short)?;
            record
                .apu_log
                .extend(
                    log.as_chunks::<3>()
                        .0
                        .iter()
                        .map(|&[kind, port, value]| ApuAccess {
                            kind: if kind == 0 {
                                ApuAccessKind::CpuWrite
                            } else {
                                ApuAccessKind::CpuRead
                            },
                            port,
                            value,
                            apu_cycle: 0,
                        }),
                );
            rest = after;
        }
        if flags & 8 != 0 {
            let (length, after) = rest.split_first_chunk::<4>().ok_or_else(short)?;
            let length = usize::try_from(u32::from_le_bytes(*length)).map_err(|_| short())?;
            let text = after.get(..length).ok_or_else(short)?;
            let (width, height, rgb) = crate::png::base64(text)
                .as_deref()
                .and_then(crate::png::rgb)
                .ok_or_else(|| Cause::Record("picture: not a PNG this decoder reads".into()))?;
            record.picture = Some(decode_picture(width, height, &rgb));
        } else if flags & 1 != 0 {
            let (size, rgb) = rest.split_first_chunk::<4>().ok_or_else(short)?;
            let width = usize::from(u16::from_le_bytes([size[0], size[1]]));
            let height = usize::from(u16::from_le_bytes([size[2], size[3]]));
            if rgb.len() != width * height * 3 {
                return Err(short());
            }
            record.picture = Some(decode_picture(width, height, rgb));
        }
        Ok(())
    }
}

impl Oracle for Lsnes {
    fn identity(&self) -> OracleId {
        IDENTITY
    }

    fn open(rom: &[u8], start: Start) -> Result<Self, OracleError> {
        let directory = Self::directory();
        std::fs::create_dir_all(&directory).map_err(|_| OracleError::Rom)?;
        let rom_path = directory.join("game.sfc");
        let script = directory.join("server.lua");
        std::fs::write(&rom_path, rom).map_err(|_| OracleError::Rom)?;
        std::fs::write(&script, SERVER).map_err(|_| OracleError::Rom)?;
        let mut command = Command::new(executable());
        match start {
            Start::PowerOn { wram_fill: 0x55 } => {}
            // bsnes v085 always fills work RAM with 0x55.
            Start::PowerOn { .. } => return Err(OracleError::Rom),
            Start::State(state) => {
                let path = directory.join("start.lss");
                std::fs::write(&path, state).map_err(|_| OracleError::State)?;
                // Given on the command line, a state's core type (snes_ntsc)
                // mismatches the ROM's (SNES) and lsnes hangs; the server
                // loads it before the first frame instead.
                command.env("OH_START", &path);
            }
            Start::Capture(_) => {
                return Err(OracleError::Seed(
                    "lsnes resumes only its own states".into(),
                ));
            }
        }
        let stderr = std::fs::File::create(directory.join(STDERR)).map_err(|_| OracleError::Rom)?;
        let config = directory.join(CONFIG);
        std::fs::create_dir_all(&config).map_err(|_| OracleError::Rom)?;
        let mut child = command
            .arg(format!("--rom={}", rom_path.display()))
            .arg(format!("--lua={}", script.display()))
            .arg("--unpause")
            .current_dir(&directory)
            .env("OH_RECORD", directory.join("record.bin"))
            .env("OH_LOG", directory.join(LUA_LOG))
            .env("APPDATA", &config)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(stderr)
            .spawn()
            .map_err(|_| OracleError::Rom)?;
        let (Some(requests), Some(stdout)) = (child.stdin.take(), child.stdout.take()) else {
            let _ = child.kill();
            let _ = child.wait();
            return Err(OracleError::Rom);
        };
        let (sender, replies) = mpsc::channel();
        // Ends when lsnes closes its standard output, or the oracle is gone.
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                if sender.send(line).is_err() {
                    break;
                }
            }
        });
        Ok(Self {
            child,
            requests,
            replies,
            failed: None,
            directory,
            record: empty_record(),
            steps: 0,
            saves: 0,
            raw: Vec::new(),
            keep: std::env::var_os("SM_LSNES_KEEP").is_some(),
            memories: true,
            next_pad: None,
        })
    }

    fn step(&mut self, pad: Pad) -> Result<&FrameRecord, OracleError> {
        if let Some(failure) = self.failed.take() {
            return Err(OracleError::Lsnes(failure));
        }
        let memories = u8::from(self.memories);
        let request = match self.next_pad.take() {
            Some(next) => format!("S {:04x} {memories} {:04x}", pad.0, next.0),
            None => format!("S {:04x} {memories}", pad.0),
        };
        self.send(&request)?;
        let waited = if self.steps == 0 {
            START_TIMEOUT
        } else {
            STEP_TIMEOUT
        };
        let deadline = Instant::now() + waited;
        let size = loop {
            let line = match self
                .replies
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            {
                Ok(Ok(line)) => line,
                Ok(Err(error)) => return Err(self.fail(Cause::Reply(error))),
                Err(RecvTimeoutError::Disconnected) => return Err(self.fail(Cause::Exited)),
                Err(RecvTimeoutError::Timeout) => return Err(self.fail(Cause::Silent(waited))),
            };
            // Any other line is lsnes's own output.
            if let Some(size) = line
                .strip_prefix("R ")
                .and_then(|rest| rest.split(' ').next())
            {
                match size.trim().parse::<usize>() {
                    Ok(size) => break size,
                    Err(_) => return Err(self.fail(Cause::Record(format!("announced {line}")))),
                }
            }
        };
        let bytes = match std::fs::read(self.directory.join("record.bin")) {
            Ok(bytes) if bytes.len() == size => bytes,
            Ok(bytes) => {
                let cause = Cause::Record(format!("{} bytes, announced {size}", bytes.len()));
                return Err(self.fail(cause));
            }
            Err(error) => return Err(self.fail(Cause::Record(error.to_string()))),
        };
        if let Err(cause) = self.decode(&bytes) {
            return Err(self.fail(cause));
        }
        self.raw = bytes;
        self.record.index = self.steps;
        self.steps += 1;
        Ok(&self.record)
    }

    fn save_state(&self) -> Vec<u8> {
        let path = self.directory.join(format!("state-{}.lss", self.saves));
        std::fs::read(path).unwrap_or_default()
    }

    /// Records stay lean whatever the tiers: the streams cost speed, and
    /// ladders compare raw records. [`Lsnes::record_streams`] asks for them.
    fn set_tiers(&mut self, _tiers: Tiers) {}

    fn reset(&mut self) {
        if let Err(OracleError::Lsnes(failure)) = self.send("R") {
            self.failed = Some(failure);
        }
    }

    fn set_watch(&mut self, _addresses: &[u32], _peek: u32) {}
}

impl Drop for Lsnes {
    fn drop(&mut self) {
        let _ = writeln!(self.requests, "Q").and_then(|()| self.requests.flush());
        let deadline = Instant::now() + QUIT_TIMEOUT;
        while matches!(self.child.try_wait(), Ok(None)) && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(20));
        }
        // One that has not quit by now never will.
        let _ = self.child.kill();
        let _ = self.child.wait();
        if !self.keep {
            let _ = std::fs::remove_dir_all(&self.directory);
        }
    }
}
