//! The lsnes backend: lsnes rr2-beta23 (the bsnes v085 core that recorded the
//! movies and seeds) as a separate process, driven by
//! `tools/oracle/lsnes_server.lua` over its standard input and a record file.
//! No emulator code is linked or copied (docs/oracle/HARNESS.md).
//!
//! The cut is the instruction the NMI interrupted, reached again with the
//! stack back where the NMI found it: the RTI has just completed. A frame in
//! which no NMI returned is cut at lsnes's frame boundary.
//!
//! Not recorded by this backend: the picture, the APU port log, the SPC state
//! (lsnes exposes SPC700 and DSP only as raw core structs), the instruction
//! profile, the PPU register write log, watched-PC hits, `PpuRegisters` (left
//! at their defaults), `overscan` and the old lock's `legacy_ppu_digest`.
//!
//! A saved state is lsnes's own, taken at the frame boundary that follows the
//! last cut: it lies before the next step's NMI, so a resumed oracle's first
//! step reproduces the next cut, with only that interval's counts partial.

use crate::record::{Beam, CpuRegisters, Cut, FrameRecord, Nmi, NmiKind, Pad, PpuRegisters};
use crate::{Oracle, OracleError, OracleId, Start, Tiers};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

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

/// One lsnes process running the original game.
pub struct Lsnes {
    child: Child,
    requests: ChildStdin,
    replies: BufReader<ChildStdout>,
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
}

/// lsnes rr2-beta23 with the bsnes v085 core.
pub const IDENTITY: OracleId = OracleId {
    emulator: "lsnes-bsnes",
    version: "rr2-beta23 (bsnes v085)",
    patches: &[],
    fixes: &[],
};

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

    fn send(&mut self, line: &str) {
        writeln!(self.requests, "{line}")
            .and_then(|()| self.requests.flush())
            .unwrap_or_else(|error| panic!("lsnes request {line}: {error}"));
    }

    /// Ask for the state at the next frame boundary, written to the returned
    /// file once the next step has run; [`Oracle::save_state`] then reads it.
    pub fn request_save(&mut self) -> PathBuf {
        self.saves += 1;
        let path = self.directory.join(format!("state-{}.lss", self.saves));
        self.send(&format!("V {}", path.display()));
        path
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

    fn decode(&mut self, bytes: &[u8]) {
        assert!(
            bytes.len() >= HEADER && &bytes[..4] == b"OHR1",
            "bad lsnes record"
        );
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
            other => panic!("lsnes record kind {other}"),
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
        if bytes[42] == 0 {
            return;
        }
        let mut at = HEADER;
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
        let mut child = command
            .arg(format!("--rom={}", rom_path.display()))
            .arg(format!("--lua={}", script.display()))
            .arg("--unpause")
            .current_dir(&directory)
            .env("OH_RECORD", directory.join("record.bin"))
            .env("OH_LOG", directory.join(LUA_LOG))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(stderr)
            .spawn()
            .map_err(|_| OracleError::Rom)?;
        let requests = child.stdin.take().ok_or(OracleError::Rom)?;
        let replies = BufReader::new(child.stdout.take().ok_or(OracleError::Rom)?);
        Ok(Self {
            child,
            requests,
            replies,
            directory,
            record: empty_record(),
            steps: 0,
            saves: 0,
            raw: Vec::new(),
            keep: std::env::var_os("SM_LSNES_KEEP").is_some(),
            memories: true,
        })
    }

    fn step(&mut self, pad: Pad) -> &FrameRecord {
        self.send(&format!("S {:04x} {}", pad.0, u8::from(self.memories)));
        let mut line = String::new();
        let size = loop {
            line.clear();
            let read = self
                .replies
                .read_line(&mut line)
                .unwrap_or_else(|error| panic!("lsnes reply: {error}"));
            if read == 0 {
                let report = self.exit_report();
                panic!("lsnes exited at step {}: {report}", self.steps);
            }
            if let Some(size) = line
                .strip_prefix("R ")
                .and_then(|rest| rest.split(' ').next())
            {
                break size.trim().parse::<usize>().expect("lsnes record size");
            }
        };
        let bytes = std::fs::read(self.directory.join("record.bin"))
            .unwrap_or_else(|error| panic!("lsnes record: {error}"));
        assert_eq!(bytes.len(), size, "lsnes record size");
        self.decode(&bytes);
        self.raw = bytes;
        self.record.index = self.steps;
        self.steps += 1;
        &self.record
    }

    fn save_state(&self) -> Vec<u8> {
        let path = self.directory.join(format!("state-{}.lss", self.saves));
        std::fs::read(path).unwrap_or_default()
    }

    fn set_tiers(&mut self, _tiers: Tiers) {}

    fn reset(&mut self) {
        self.send("R");
    }

    fn set_watch(&mut self, _addresses: &[u32], _peek: u32) {}
}

impl Drop for Lsnes {
    fn drop(&mut self) {
        let _ = writeln!(self.requests, "Q").and_then(|()| self.requests.flush());
        let _ = self.child.wait();
        if !self.keep {
            let _ = std::fs::remove_dir_all(&self.directory);
        }
    }
}
