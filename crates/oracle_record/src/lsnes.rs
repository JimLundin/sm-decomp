//! The lsnes server's raw record (`OHR1`, `tools/oracle/lsnes_server.lua`): the bytes
//! `oracle::lsnes` reads after each step and the seed cutter keeps beside a native seed
//! (`seeds/<capture>/NNNN.record`).
//!
//! Its own module, outside the emulator crate, so `verify` and `tools` read a bundle's seed
//! records without linking an emulator. The picture stays undecoded here ([`RawPicture`]):
//! mapping lsnes's screenshot back to BGR555 is the backend's (`oracle::lsnes`).

use crate::{
    ApuAccess, ApuAccessKind, Beam, CpuRegisters, Cut, FrameRecord, Nmi, NmiKind, Pad, PpuRegisters,
};
use core::fmt;

/// The record header: magic through the memory flag.
const HEADER: usize = 43;
const WRAM: usize = 0x2_0000;
const VRAM: usize = 0x1_0000;
const CGRAM: usize = 0x200;
const OAM: usize = 0x220;
const SRAM: usize = 0x2000;
/// The memories after the header, when the record carries them.
const MEMORIES: usize = WRAM + VRAM + CGRAM + OAM + SRAM;
/// The record's optional streams, appended after the memories when a `T` request asked for
/// any.
const STREAMS_MAGIC: &[u8; 4] = b"OHS1";
/// The PPU block: `PpuRegisters` from `inidisp` to `setini` as the bytes the game wrote
/// (scroll and mode 7 words little-endian), then `$43x0-$43xA`.
const PPU_BYTES: usize = 58 + 88;

/// The stream flags after [`STREAMS_MAGIC`].
const PICTURE_RGB: u8 = 1;
const APU_LOG: u8 = 2;
const PPU: u8 = 4;
const PICTURE_PNG: u8 = 8;

/// Why raw record bytes don't decode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RawError {
    /// No `OHR1` header in this many bytes.
    NoHeader(usize),
    /// The memory flag is set and this many bytes fall short of the memories.
    ShortOfMemories(usize),
    /// The cut kind byte names no cut.
    Kind(u8),
    /// Bytes follow the memories without the streams header.
    NoStreamsHeader,
    /// The streams end early; this many stream bytes were there.
    StreamsCutShort(usize),
}

impl fmt::Display for RawError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoHeader(length) => write!(f, "no header in {length} bytes"),
            Self::ShortOfMemories(length) => write!(f, "{length} bytes, short of the memories"),
            Self::Kind(kind) => write!(f, "kind {kind}"),
            Self::NoStreamsHeader => write!(f, "no streams header"),
            Self::StreamsCutShort(length) => write!(f, "streams cut short at {length} bytes"),
        }
    }
}

/// The picture a record carries, as lsnes sent it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RawPicture<'a> {
    /// lsnes's screenshot as a base64 PNG.
    Png(&'a [u8]),
    /// The screenshot's 8-bit RGB triples, line by line.
    Rgb {
        /// Its width in dots.
        width: usize,
        /// Its height in lines.
        height: usize,
        /// The triples.
        rgb: &'a [u8],
    },
}

/// A record with no state yet, sized for the memories [`decode`] fills.
#[must_use]
pub fn blank() -> FrameRecord {
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

fn le16(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}

fn le32(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
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

/// Decodes `bytes` into `record`, whose memories keep their last values when the record
/// carries none, and returns the picture undecoded. `record.picture` is left `None` and
/// `record.index` untouched: the caller sets both.
///
/// # Errors
///
/// [`RawError`] when the bytes aren't a whole record.
pub fn decode<'a>(
    bytes: &'a [u8],
    record: &mut FrameRecord,
) -> Result<Option<RawPicture<'a>>, RawError> {
    if bytes.len() < HEADER || &bytes[..4] != b"OHR1" {
        return Err(RawError::NoHeader(bytes.len()));
    }
    let dumped = bytes[42] != 0;
    if dumped && bytes.len() < HEADER + MEMORIES {
        return Err(RawError::ShortOfMemories(bytes.len()));
    }
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
        other => return Err(RawError::Kind(other)),
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
    decode_streams(record, &bytes[at..])
}

/// The optional streams after the memories ([`STREAMS_MAGIC`]).
fn decode_streams<'a>(
    record: &mut FrameRecord,
    bytes: &'a [u8],
) -> Result<Option<RawPicture<'a>>, RawError> {
    record.apu_log.clear();
    record.picture = None;
    if bytes.is_empty() {
        return Ok(None);
    }
    let short = || RawError::StreamsCutShort(bytes.len());
    let Some((STREAMS_MAGIC, rest)) = bytes.split_first_chunk::<4>() else {
        return Err(RawError::NoStreamsHeader);
    };
    let (&flags, mut rest) = rest.split_first().ok_or_else(short)?;
    if flags & PPU != 0 {
        let (ppu, after) = rest.split_at_checked(PPU_BYTES).ok_or_else(short)?;
        record.ppu = decode_ppu(ppu);
        rest = after;
    }
    if flags & APU_LOG != 0 {
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
    if flags & PICTURE_PNG != 0 {
        let (length, after) = rest.split_first_chunk::<4>().ok_or_else(short)?;
        let length = usize::try_from(u32::from_le_bytes(*length)).map_err(|_| short())?;
        let text = after.get(..length).ok_or_else(short)?;
        Ok(Some(RawPicture::Png(text)))
    } else if flags & PICTURE_RGB != 0 {
        let (size, rgb) = rest.split_first_chunk::<4>().ok_or_else(short)?;
        let width = usize::from(u16::from_le_bytes([size[0], size[1]]));
        let height = usize::from(u16::from_le_bytes([size[2], size[3]]));
        if rgb.len() != width * height * 3 {
            return Err(short());
        }
        Ok(Some(RawPicture::Rgb { width, height, rgb }))
    } else {
        Ok(None)
    }
}

/// A seed's raw record ([`decode`] into a [`blank`] record), refused when it lacks the
/// memories a State is decoded from.
///
/// # Errors
///
/// [`RawError`] when the bytes don't decode or carry no memories.
pub fn read(bytes: &[u8]) -> Result<FrameRecord, RawError> {
    if bytes.get(42) == Some(&0) {
        return Err(RawError::ShortOfMemories(bytes.len()));
    }
    // LEGACY(seed-ppu): a seed without the optional PPU tier keeps the default
    // registers, even where its native state retains other values.
    let mut record = blank();
    decode(bytes, &mut record)?;
    Ok(record)
}
