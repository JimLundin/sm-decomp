//! Measurements for the architecture lane (docs/oracle/MEASUREMENTS.md): busy
//! spans, door transitions, decompression calls, line timing of the IRQ and
//! HDMA-object handlers, and the sound driver's port latency. Each reads a
//! run's frame records; the few game addresses come from the bank logs
//! (https://patrickjohnston.org/bank/).

use crate::record::{ApuAccessKind, FrameRecord, NmiKind, WriteSource};
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// `$88:84B9` HDMA object handler: its two RTLs.
pub const HDMA_HANDLER_RETURNS: [u32; 2] = [0x88_8516, 0x88_851b];
/// `$80:B119` decompression (variable destination) and `$80:B271` to VRAM.
pub const DECOMPRESSION_ENTRIES: [u32; 2] = [0x80_b119, 0x80_b271];
/// Their RTLs, `$80:B13F` and `$80:B296`.
pub const DECOMPRESSION_RETURNS: [u32; 2] = [0x80_b13f, 0x80_b296];
/// The WRAM the watches peek: the decompression source pointer $47-$49.
pub const PEEK: u32 = 0x47;
/// Every address the measurements watch.
pub const WATCHED: [u32; 6] = [
    HDMA_HANDLER_RETURNS[0],
    HDMA_HANDLER_RETURNS[1],
    DECOMPRESSION_ENTRIES[0],
    DECOMPRESSION_ENTRIES[1],
    DECOMPRESSION_RETURNS[0],
    DECOMPRESSION_RETURNS[1],
];

fn word(wram: &[u8], address: usize) -> u16 {
    u16::from_le_bytes([wram[address], wram[address + 1]])
}

/// The game state ($0998) values of a door transition: 9 (hit a door), $A
/// and $B (loading the next room).
const DOOR_STATES: [u16; 3] = [0x09, 0x0a, 0x0b];

/// One decompression call.
#[derive(Clone, Copy, Debug)]
struct Decompression {
    entry: u32,
    /// The compressed stream's 24-bit address.
    source: u32,
    /// The frame and line it started at.
    start: (usize, u16),
    /// The frame and line it returned at.
    end: Option<(usize, u16)>,
}

/// What a frame contributes, kept so spans can be summarised afterwards.
#[derive(Clone, Debug)]
struct Frame {
    overrun: Option<u32>,
    room: u16,
    door: u16,
    state: u16,
    hdma_end: Option<u16>,
    irq_writes: Vec<(u16, u16, u8)>,
    apu_writes: usize,
    dma_bytes_blank: usize,
}

/// A run's measurements, accumulated one frame at a time.
#[derive(Clone, Debug, Default)]
pub struct Measurements {
    frames: Vec<Frame>,
    decompressions: Vec<Decompression>,
    open_decompression: Option<usize>,
    /// CPU port writes awaiting the SPC700's echo: (port, value, apu cycle).
    pending_echo: Vec<(u8, u8, u32)>,
    /// Echo latencies in SPC700 cycles, per port.
    echo_cycles: BTreeMap<u8, Vec<u32>>,
    cpu_port_writes: [usize; 4],
    cpu_port_reads: [usize; 4],
    spc_port_writes: [usize; 4],
}

impl Measurements {
    /// Fold in one frame's record (with the write log and watch tiers on).
    pub fn frame(&mut self, record: &FrameRecord) {
        let index = self.frames.len();
        for hit in &record.hits {
            let (pc, beam) = (hit.pc, hit.beam);
            if DECOMPRESSION_ENTRIES.contains(&pc) {
                // The source pointer at entry: $47-$48 and the bank in $49.
                let [low, high, bank, _] = hit.peek;
                let source = u32::from_le_bytes([low, high, bank, 0]);
                self.open_decompression = Some(self.decompressions.len());
                self.decompressions.push(Decompression {
                    entry: pc,
                    source,
                    start: (index, beam.v),
                    end: None,
                });
            } else if DECOMPRESSION_RETURNS.contains(&pc)
                && let Some(open) = self.open_decompression.take()
            {
                self.decompressions[open].end = Some((index, beam.v));
            }
        }
        let hdma_end = record
            .hits
            .iter()
            .filter(|hit| HDMA_HANDLER_RETURNS.contains(&hit.pc))
            .map(|hit| hit.beam.v)
            .next_back();
        let irq_writes = record
            .writes
            .iter()
            .filter(|write| write.source == WriteSource::Irq)
            .map(|write| (write.beam.v, write.beam.h, write.register))
            .collect();
        let blank = record.ppu.inidisp & 0x80 != 0;
        let dma_bytes_blank = if blank {
            record
                .writes
                .iter()
                .filter(|write| write.source == WriteSource::Dma)
                .count()
        } else {
            0
        };
        for access in &record.apu_log {
            let port = usize::from(access.port & 3);
            match access.kind {
                ApuAccessKind::CpuWrite | ApuAccessKind::DmaWrite => {
                    self.cpu_port_writes[port] += 1;
                    self.pending_echo
                        .retain(|&(pending, _, _)| pending != access.port);
                    self.pending_echo
                        .push((access.port, access.value, access.apu_cycle));
                }
                ApuAccessKind::CpuRead | ApuAccessKind::DmaRead => self.cpu_port_reads[port] += 1,
                ApuAccessKind::SpcWrite => {
                    self.spc_port_writes[port] += 1;
                    if let Some(position) =
                        self.pending_echo.iter().position(|&(pending, value, _)| {
                            pending == access.port && value == access.value
                        })
                    {
                        let (_, _, written) = self.pending_echo.remove(position);
                        self.echo_cycles
                            .entry(access.port)
                            .or_default()
                            .push(access.apu_cycle.wrapping_sub(written));
                    }
                }
            }
        }
        let apu_writes = record
            .apu_log
            .iter()
            .filter(|access| access.kind == ApuAccessKind::CpuWrite)
            .count();
        self.frames.push(Frame {
            overrun: record
                .nmi()
                .filter(|nmi| nmi.kind == NmiKind::Overrun)
                .map(|nmi| nmi.interrupted_pc),
            room: word(&record.wram, 0x079b),
            door: word(&record.wram, 0x078d),
            state: word(&record.wram, 0x0998),
            hdma_end,
            irq_writes,
            apu_writes,
            dma_bytes_blank,
        });
    }

    /// The summary, as Markdown lines (ids, frames, counts and beam positions).
    pub fn report(&self, rom: &[u8]) -> String {
        let mut text = String::new();
        let frames = self.frames.len();
        let overrun = self
            .frames
            .iter()
            .filter(|frame| frame.overrun.is_some())
            .count();
        let _ = writeln!(
            text,
            "- frames {frames}, busy (overrun NMI) frames {overrun}"
        );
        // Busy spans: runs of overrun frames.
        let mut spans: Vec<(usize, usize)> = Vec::new();
        for (index, frame) in self.frames.iter().enumerate() {
            if frame.overrun.is_none() {
                continue;
            }
            match spans.last_mut() {
                Some((_, end)) if *end + 1 == index => *end = index,
                _ => spans.push((index, index)),
            }
        }
        for &(first, last) in &spans {
            let before = &self.frames[first.saturating_sub(1)];
            let after = &self.frames[(last + 1).min(frames - 1)];
            let mut pcs: BTreeMap<u32, usize> = BTreeMap::new();
            for frame in &self.frames[first..=last] {
                *pcs.entry(frame.overrun.unwrap_or_default()).or_default() += 1;
            }
            let mut pcs: Vec<(usize, u32)> =
                pcs.into_iter().map(|(pc, count)| (count, pc)).collect();
            pcs.sort_unstable_by(|a, b| b.cmp(a));
            let histogram: Vec<String> = pcs
                .iter()
                .take(4)
                .map(|(count, pc)| format!("${pc:06X}:{count}"))
                .collect();
            let _ = writeln!(
                text,
                "  - busy span frames {first}-{last} ({} frames): room {:04X} -> {:04X}, door {:04X}; interrupted {}",
                last - first + 1,
                before.room,
                after.room,
                after.door,
                histogram.join(" ")
            );
        }
        // Door transitions: runs of the door game states.
        let mut doors: Vec<(usize, usize)> = Vec::new();
        for (index, frame) in self.frames.iter().enumerate() {
            if !DOOR_STATES.contains(&frame.state) {
                continue;
            }
            match doors.last_mut() {
                Some((_, end)) if *end + 1 == index => *end = index,
                _ => doors.push((index, index)),
            }
        }
        for &(first, last) in &doors {
            let busy = self.frames[first..=last]
                .iter()
                .filter(|frame| frame.overrun.is_some())
                .count();
            let uploads: usize = self.frames[first..=last]
                .iter()
                .map(|frame| frame.apu_writes)
                .sum();
            let dma: usize = self.frames[first..=last]
                .iter()
                .map(|frame| frame.dma_bytes_blank)
                .sum();
            let _ = writeln!(
                text,
                "  - door transition frames {first}-{last} ({} frames, {busy} busy): room {:04X} -> {:04X}; APU port writes {uploads}; DMA bytes under forced blank {dma}",
                last - first + 1,
                self.frames[first.saturating_sub(1)].room,
                self.frames[(last + 1).min(frames - 1)].room
            );
        }
        for call in &self.decompressions {
            let (output, commands) = decompressed_size(rom, call.source);
            let mix: Vec<String> = commands
                .iter()
                .enumerate()
                .filter(|&(_, &count)| count > 0)
                .map(|(command, count)| format!("{command}:{count}"))
                .collect();
            let duration = call.end.map_or("unfinished".to_owned(), |(frame, line)| {
                let lines = ((frame - call.start.0) * 262 + usize::from(line))
                    .saturating_sub(usize::from(call.start.1));
                format!("{lines} lines")
            });
            let _ = writeln!(
                text,
                "  - decompression ${:06X} at frame {} line {}: source ${:06X}, {output} bytes out, commands {}; {duration}",
                call.entry,
                call.start.0,
                call.start.1,
                call.source,
                mix.join(" ")
            );
        }
        let ends: Vec<u16> = self
            .frames
            .iter()
            .filter_map(|frame| frame.hdma_end)
            .collect();
        if !ends.is_empty() {
            let mut sorted = ends.clone();
            sorted.sort_unstable();
            let _ = writeln!(
                text,
                "- HDMA object handler finished on {} frames: line min {}, median {}, max {}",
                ends.len(),
                sorted[0],
                sorted[sorted.len() / 2],
                sorted[sorted.len() - 1]
            );
        }
        let mut irq: BTreeMap<(u16, u8), (usize, u16, u16)> = BTreeMap::new();
        for frame in &self.frames {
            for &(v, h, register) in &frame.irq_writes {
                let entry = irq.entry((v, register)).or_insert((0, u16::MAX, 0));
                entry.0 += 1;
                entry.1 = entry.1.min(h);
                entry.2 = entry.2.max(h);
            }
        }
        if !irq.is_empty() {
            let _ = writeln!(
                text,
                "- IRQ handler register writes (line, register: writes, H range):"
            );
            for ((v, register), (count, low, high)) in &irq {
                let _ = writeln!(
                    text,
                    "  - line {v}, $21{register:02X}: {count} writes, H {low}-{high}"
                );
            }
        }
        let _ = writeln!(
            text,
            "- APU ports: CPU writes {:?}, CPU reads {:?}, SPC700 writes {:?} (ports 0-3)",
            self.cpu_port_writes, self.cpu_port_reads, self.spc_port_writes
        );
        for (port, cycles) in &self.echo_cycles {
            let mut sorted = cycles.clone();
            sorted.sort_unstable();
            let _ = writeln!(
                text,
                "  - port {port} echo latency over {} echoes: SPC700 cycles min {}, median {}, max {}",
                sorted.len(),
                sorted[0],
                sorted[sorted.len() / 2],
                sorted[sorted.len() - 1]
            );
        }
        text
    }
}

/// The output size and command mix (counts of commands 0-7) of the
/// compressed stream at `source` in the LoROM cartridge `rom`.
pub fn decompressed_size(rom: &[u8], source: u32) -> (usize, [usize; 8]) {
    let mut address = source;
    let mut next = || {
        let bank = (address >> 16) & 0x7f;
        let offset = usize::try_from((bank << 15) | (address & 0x7fff)).unwrap_or(usize::MAX);
        address = if address & 0xffff == 0xffff {
            ((address & 0xff_0000) + 0x1_0000) | 0x8000
        } else {
            address + 1
        };
        rom.get(offset).copied()
    };
    let mut output = 0;
    let mut commands = [0; 8];
    while let Some(header) = next() {
        if header == 0xff {
            break;
        }
        let (command, length) = if header >> 5 == 7 {
            let Some(low) = next() else { break };
            (
                (header >> 2) & 7,
                ((usize::from(header & 3) << 8) | usize::from(low)) + 1,
            )
        } else {
            (header >> 5, usize::from(header & 0x1f) + 1)
        };
        commands[usize::from(command)] += 1;
        output += length;
        let parameters = match command {
            0 => length,
            2 | 4 | 5 => 2,
            _ => 1,
        };
        for _ in 0..parameters {
            next();
        }
    }
    (output, commands)
}
