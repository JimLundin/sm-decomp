//! The record's binary encoding: one [`FrameRecord`] as little-endian fields in declaration
//! order, behind a magic and a version, so a reader can refuse bytes it doesn't know.
//!
//! The lockstep cache (`oracle corpus --cache`, `verify::cache`) stores records in it. A
//! sequence is a length-prefixed list of its items; an option is a tag byte, 0 for `None`.

use crate::{
    ApuAccess, ApuAccessKind, Beam, CpuRegisters, Cut, FrameRecord, Hit, MAX_LINES, Nmi, NmiKind,
    Pad, Picture, PpuRegisters, RegisterWrite, SpcState, WriteSource,
};
use core::fmt;

/// The bytes every encoded record starts with.
pub const MAGIC: [u8; 4] = *b"SMFR";
/// The encoding's version, after the magic.
pub const VERSION: u8 = 1;

/// Whether an encoding keeps the record's picture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PictureMode {
    /// Keep it, when the record has one.
    Keep,
    /// Encode the record as if it had none: a third of the size.
    Drop,
}

/// Why bytes are not an encoded record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodecError {
    /// The bytes don't start with [`MAGIC`].
    BadMagic,
    /// The encoding is a version this reader doesn't know.
    BadVersion(u8),
    /// The bytes end inside the named field.
    Truncated(&'static str),
    /// The named field's tag byte names no variant.
    BadTag {
        /// The field.
        field: &'static str,
        /// Its tag.
        tag: u8,
    },
    /// A length doesn't fit this machine's memory.
    TooLong(&'static str),
    /// Bytes follow the record.
    Trailing(usize),
}

impl fmt::Display for CodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadMagic => f.write_str("not an encoded frame record"),
            Self::BadVersion(version) => write!(f, "record encoding version {version}"),
            Self::Truncated(field) => write!(f, "the bytes end inside {field}"),
            Self::BadTag { field, tag } => write!(f, "{field}: tag {tag}"),
            Self::TooLong(field) => write!(f, "{field}: too long"),
            Self::Trailing(count) => write!(f, "{count} bytes after the record"),
        }
    }
}

impl std::error::Error for CodecError {}

/// `record` as bytes, appended to `out`.
pub fn encode(record: &FrameRecord, pictures: PictureMode, out: &mut Vec<u8>) {
    let mut w = Writer(out);
    w.bytes(&MAGIC);
    w.u8(VERSION);
    w.u64(record.index);
    w.u32(record.video_frame);
    w.u16(record.pad.0);
    match record.cut {
        Cut::NmiReturn(nmi) => {
            w.u8(0);
            w.u8(match nmi.kind {
                NmiKind::Requested => 0,
                NmiKind::Overrun => 1,
            });
            w.u32(nmi.interrupted_pc);
            w.bool(nmi.polled);
        }
        Cut::VblankWithoutNmi => w.u8(1),
        Cut::NmiStillRunning { interrupted_pc } => {
            w.u8(2);
            w.u32(interrupted_pc);
        }
    }
    w.u32(record.joypad_reads);
    w.option(record.wait_entry.as_ref(), Writer::beam);
    w.bool(record.overscan);
    w.cpu(&record.cpu);
    w.seq8(&record.wram);
    w.seq16(&record.vram);
    w.seq16(&record.cgram);
    w.seq8(&record.oam);
    w.seq8(&record.sram);
    w.ppu(&record.ppu);
    w.len(record.apu_log.len());
    for access in &record.apu_log {
        w.u8(match access.kind {
            ApuAccessKind::CpuWrite => 0,
            ApuAccessKind::CpuRead => 1,
            ApuAccessKind::DmaWrite => 2,
            ApuAccessKind::DmaRead => 3,
            ApuAccessKind::SpcWrite => 4,
        });
        w.u8(access.port);
        w.u8(access.value);
        w.u32(access.apu_cycle);
    }
    let picture = match pictures {
        PictureMode::Keep => record.picture.as_ref(),
        PictureMode::Drop => None,
    };
    w.option(picture, Writer::picture);
    w.option(record.spc.as_ref(), Writer::spc);
    w.option(record.bank_instructions.as_deref(), |w, counts| {
        w.len(counts.len());
        counts.iter().for_each(|&count| w.u32(count));
    });
    w.len(record.writes.len());
    for write in &record.writes {
        w.beam(&write.beam);
        w.u8(write.register);
        w.u8(write.value);
        w.u8(match write.source {
            WriteSource::Main => 0,
            WriteSource::Nmi => 1,
            WriteSource::Irq => 2,
            WriteSource::Dma => 3,
            WriteSource::Hdma => 4,
        });
    }
    w.len(record.hits.len());
    for hit in &record.hits {
        w.u32(hit.pc);
        w.beam(&hit.beam);
        w.bytes(&hit.peek);
    }
    w.u64(record.legacy_ppu_digest);
}

/// The record `bytes` encode, all of them.
///
/// # Errors
///
/// A [`CodecError`] when the bytes are not exactly one encoded record.
pub fn decode(bytes: &[u8]) -> Result<FrameRecord, CodecError> {
    let mut r = Reader { rest: bytes };
    if r.array::<4>("magic")? != MAGIC {
        return Err(CodecError::BadMagic);
    }
    let version = r.u8("version")?;
    if version != VERSION {
        return Err(CodecError::BadVersion(version));
    }
    let index = r.u64("index")?;
    let video_frame = r.u32("video_frame")?;
    let pad = Pad(r.u16("pad")?);
    let cut = match r.u8("cut")? {
        0 => {
            let kind = match r.u8("nmi kind")? {
                0 => NmiKind::Requested,
                1 => NmiKind::Overrun,
                tag => {
                    return Err(CodecError::BadTag {
                        field: "nmi kind",
                        tag,
                    });
                }
            };
            Cut::NmiReturn(Nmi {
                kind,
                interrupted_pc: r.u32("interrupted_pc")?,
                polled: r.bool("polled")?,
            })
        }
        1 => Cut::VblankWithoutNmi,
        2 => Cut::NmiStillRunning {
            interrupted_pc: r.u32("interrupted_pc")?,
        },
        tag => return Err(CodecError::BadTag { field: "cut", tag }),
    };
    let joypad_reads = r.u32("joypad_reads")?;
    let wait_entry = r.option("wait_entry", Reader::beam)?;
    let overscan = r.bool("overscan")?;
    let cpu = r.cpu()?;
    let wram = r.seq8("wram")?;
    let vram = r.seq16("vram")?;
    let cgram = r.seq16("cgram")?;
    let oam = r.seq8("oam")?;
    let sram = r.seq8("sram")?;
    let ppu = r.ppu()?;
    let apu_log = r.list("apu_log", |r| {
        let kind = match r.u8("apu kind")? {
            0 => ApuAccessKind::CpuWrite,
            1 => ApuAccessKind::CpuRead,
            2 => ApuAccessKind::DmaWrite,
            3 => ApuAccessKind::DmaRead,
            4 => ApuAccessKind::SpcWrite,
            tag => {
                return Err(CodecError::BadTag {
                    field: "apu kind",
                    tag,
                });
            }
        };
        Ok(ApuAccess {
            kind,
            port: r.u8("apu port")?,
            value: r.u8("apu value")?,
            apu_cycle: r.u32("apu cycle")?,
        })
    })?;
    let picture = r.option("picture", Reader::picture)?;
    let spc = r.option("spc", Reader::spc)?;
    let bank_instructions = r.option("bank_instructions", |r| {
        r.list("bank_instructions", |r| r.u32("bank_instructions"))
            .map(Vec::into_boxed_slice)
    })?;
    let writes = r.list("writes", |r| {
        let beam = r.beam()?;
        let register = r.u8("write register")?;
        let value = r.u8("write value")?;
        let source = match r.u8("write source")? {
            0 => WriteSource::Main,
            1 => WriteSource::Nmi,
            2 => WriteSource::Irq,
            3 => WriteSource::Dma,
            4 => WriteSource::Hdma,
            tag => {
                return Err(CodecError::BadTag {
                    field: "write source",
                    tag,
                });
            }
        };
        Ok(RegisterWrite {
            beam,
            register,
            value,
            source,
        })
    })?;
    let hits = r.list("hits", |r| {
        Ok(Hit {
            pc: r.u32("hit pc")?,
            beam: r.beam()?,
            peek: r.array("hit peek")?,
        })
    })?;
    let legacy_ppu_digest = r.u64("legacy_ppu_digest")?;
    if !r.rest.is_empty() {
        return Err(CodecError::Trailing(r.rest.len()));
    }
    Ok(FrameRecord {
        index,
        video_frame,
        pad,
        cut,
        joypad_reads,
        wait_entry,
        overscan,
        cpu,
        wram,
        vram,
        cgram,
        oam,
        sram,
        ppu,
        apu_log,
        picture,
        spc,
        bank_instructions,
        writes,
        hits,
        legacy_ppu_digest,
    })
}

struct Writer<'a>(&'a mut Vec<u8>);

impl Writer<'_> {
    fn bytes(&mut self, bytes: &[u8]) {
        self.0.extend_from_slice(bytes);
    }

    fn u8(&mut self, value: u8) {
        self.0.push(value);
    }

    fn bool(&mut self, value: bool) {
        self.u8(u8::from(value));
    }

    fn u16(&mut self, value: u16) {
        self.bytes(&value.to_le_bytes());
    }

    fn u32(&mut self, value: u32) {
        self.bytes(&value.to_le_bytes());
    }

    fn u64(&mut self, value: u64) {
        self.bytes(&value.to_le_bytes());
    }

    /// A length as a u32; the machine's memories are far below 4 GiB.
    fn len(&mut self, len: usize) {
        self.u32(u32::try_from(len).unwrap_or(u32::MAX));
    }

    fn seq8(&mut self, bytes: &[u8]) {
        self.len(bytes.len());
        self.bytes(bytes);
    }

    fn seq16(&mut self, words: &[u16]) {
        self.len(words.len());
        words.iter().for_each(|&word| self.u16(word));
    }

    fn option<T: ?Sized>(&mut self, value: Option<&T>, write: impl FnOnce(&mut Self, &T)) {
        match value {
            Some(value) => {
                self.u8(1);
                write(self, value);
            }
            None => self.u8(0),
        }
    }

    fn beam(&mut self, beam: &Beam) {
        self.u16(beam.v);
        self.u16(beam.h);
    }

    fn cpu(&mut self, cpu: &CpuRegisters) {
        let CpuRegisters {
            a,
            x,
            y,
            s,
            pc,
            d,
            k,
            db,
            p,
            emulation,
        } = *cpu;
        [a, x, y, s, pc, d]
            .into_iter()
            .for_each(|word| self.u16(word));
        self.bytes(&[k, db, p]);
        self.bool(emulation);
    }

    fn ppu(&mut self, ppu: &PpuRegisters) {
        let PpuRegisters {
            inidisp,
            obsel,
            bgmode,
            mosaic,
            bg_sc,
            bg12nba,
            bg34nba,
            bg_scroll,
            m7sel,
            m7,
            window_select,
            window_positions,
            wbglog,
            wobjlog,
            screens,
            cgwsel,
            cgadsub,
            fixed_color,
            setini,
            nmitimen,
            htime,
            vtime,
            hdmaen,
            memsel,
            dma,
        } = *ppu;
        self.bytes(&[inidisp, obsel, bgmode, mosaic]);
        self.bytes(&bg_sc);
        self.bytes(&[bg12nba, bg34nba]);
        bg_scroll
            .as_flattened()
            .iter()
            .for_each(|&scroll| self.u16(scroll));
        self.u8(m7sel);
        m7.iter()
            .for_each(|&value| self.bytes(&value.to_le_bytes()));
        self.bytes(&window_select);
        self.bytes(&window_positions);
        self.bytes(&[wbglog, wobjlog]);
        self.bytes(&screens);
        self.bytes(&[cgwsel, cgadsub]);
        self.bytes(&fixed_color);
        self.bytes(&[setini, nmitimen]);
        self.u16(htime);
        self.u16(vtime);
        self.bytes(&[hdmaen, memsel]);
        self.bytes(dma.as_flattened());
    }

    fn picture(&mut self, picture: &Picture) {
        self.len(picture.lines);
        self.seq16(&picture.dots);
        self.bytes(&picture.brightness);
        picture
            .forced_blank
            .iter()
            .for_each(|&blank| self.bool(blank));
    }

    fn spc(&mut self, spc: &SpcState) {
        self.bytes(&[spc.a, spc.x, spc.y, spc.sp]);
        self.u16(spc.pc);
        self.u8(spc.psw);
        self.bool(spc.stopped);
        self.bytes(&spc.cpu_to_apu);
        self.bytes(&spc.apu_to_cpu);
        self.seq8(&spc.ram);
        self.bytes(&spc.dsp);
    }
}

struct Reader<'a> {
    rest: &'a [u8],
}

impl Reader<'_> {
    fn array<const N: usize>(&mut self, field: &'static str) -> Result<[u8; N], CodecError> {
        let (head, rest) = self
            .rest
            .split_first_chunk::<N>()
            .ok_or(CodecError::Truncated(field))?;
        self.rest = rest;
        Ok(*head)
    }

    fn u8(&mut self, field: &'static str) -> Result<u8, CodecError> {
        self.array::<1>(field).map(u8::from_le_bytes)
    }

    fn bool(&mut self, field: &'static str) -> Result<bool, CodecError> {
        match self.u8(field)? {
            0 => Ok(false),
            1 => Ok(true),
            tag => Err(CodecError::BadTag { field, tag }),
        }
    }

    fn u16(&mut self, field: &'static str) -> Result<u16, CodecError> {
        self.array(field).map(u16::from_le_bytes)
    }

    fn i16(&mut self, field: &'static str) -> Result<i16, CodecError> {
        self.array(field).map(i16::from_le_bytes)
    }

    fn u32(&mut self, field: &'static str) -> Result<u32, CodecError> {
        self.array(field).map(u32::from_le_bytes)
    }

    fn u64(&mut self, field: &'static str) -> Result<u64, CodecError> {
        self.array(field).map(u64::from_le_bytes)
    }

    fn len(&mut self, field: &'static str) -> Result<usize, CodecError> {
        usize::try_from(self.u32(field)?).map_err(|_| CodecError::TooLong(field))
    }

    fn take(&mut self, field: &'static str, count: usize) -> Result<&[u8], CodecError> {
        let (head, rest) = self
            .rest
            .split_at_checked(count)
            .ok_or(CodecError::Truncated(field))?;
        self.rest = rest;
        Ok(head)
    }

    fn seq8(&mut self, field: &'static str) -> Result<Box<[u8]>, CodecError> {
        let len = self.len(field)?;
        self.take(field, len).map(Box::from)
    }

    fn seq16(&mut self, field: &'static str) -> Result<Box<[u16]>, CodecError> {
        let len = self.len(field)?;
        let bytes = len.checked_mul(2).ok_or(CodecError::TooLong(field))?;
        let words = self.take(field, bytes)?;
        Ok(words
            .as_chunks::<2>()
            .0
            .iter()
            .map(|&pair| u16::from_le_bytes(pair))
            .collect())
    }

    fn list<T>(
        &mut self,
        field: &'static str,
        mut item: impl FnMut(&mut Self) -> Result<T, CodecError>,
    ) -> Result<Vec<T>, CodecError> {
        let len = self.len(field)?;
        // Each item is at least a byte: a corrupt length can't reserve more than the bytes.
        let mut items = Vec::with_capacity(len.min(self.rest.len()));
        for _ in 0..len {
            items.push(item(self)?);
        }
        Ok(items)
    }

    fn option<T>(
        &mut self,
        field: &'static str,
        read: impl FnOnce(&mut Self) -> Result<T, CodecError>,
    ) -> Result<Option<T>, CodecError> {
        match self.u8(field)? {
            0 => Ok(None),
            1 => read(self).map(Some),
            tag => Err(CodecError::BadTag { field, tag }),
        }
    }

    fn beam(&mut self) -> Result<Beam, CodecError> {
        Ok(Beam {
            v: self.u16("beam v")?,
            h: self.u16("beam h")?,
        })
    }

    fn cpu(&mut self) -> Result<CpuRegisters, CodecError> {
        Ok(CpuRegisters {
            a: self.u16("cpu a")?,
            x: self.u16("cpu x")?,
            y: self.u16("cpu y")?,
            s: self.u16("cpu s")?,
            pc: self.u16("cpu pc")?,
            d: self.u16("cpu d")?,
            k: self.u8("cpu k")?,
            db: self.u8("cpu db")?,
            p: self.u8("cpu p")?,
            emulation: self.bool("cpu emulation")?,
        })
    }

    fn ppu(&mut self) -> Result<PpuRegisters, CodecError> {
        let field = "ppu";
        let [inidisp, obsel, bgmode, mosaic] = self.array(field)?;
        let bg_sc = self.array(field)?;
        let [bg12nba, bg34nba] = self.array(field)?;
        let mut bg_scroll = [[0; 2]; 4];
        for scroll in bg_scroll.as_flattened_mut() {
            *scroll = self.u16(field)?;
        }
        let m7sel = self.u8(field)?;
        let mut m7 = [0; 6];
        for value in &mut m7 {
            *value = self.i16(field)?;
        }
        let window_select = self.array(field)?;
        let window_positions = self.array(field)?;
        let [wbglog, wobjlog] = self.array(field)?;
        let screens = self.array(field)?;
        let [cgwsel, cgadsub] = self.array(field)?;
        let fixed_color = self.array(field)?;
        let [setini, nmitimen] = self.array(field)?;
        let htime = self.u16(field)?;
        let vtime = self.u16(field)?;
        let [hdmaen, memsel] = self.array(field)?;
        let mut dma = [[0; 11]; 8];
        for channel in &mut dma {
            *channel = self.array(field)?;
        }
        Ok(PpuRegisters {
            inidisp,
            obsel,
            bgmode,
            mosaic,
            bg_sc,
            bg12nba,
            bg34nba,
            bg_scroll,
            m7sel,
            m7,
            window_select,
            window_positions,
            wbglog,
            wobjlog,
            screens,
            cgwsel,
            cgadsub,
            fixed_color,
            setini,
            nmitimen,
            htime,
            vtime,
            hdmaen,
            memsel,
            dma,
        })
    }

    fn picture(&mut self) -> Result<Picture, CodecError> {
        let lines = self.len("picture lines")?;
        let dots = self.seq16("picture dots")?;
        let brightness = self.array::<MAX_LINES>("picture brightness")?;
        let mut forced_blank = [false; MAX_LINES];
        for blank in &mut forced_blank {
            *blank = self.bool("picture forced blank")?;
        }
        Ok(Picture {
            lines,
            dots,
            brightness,
            forced_blank,
        })
    }

    fn spc(&mut self) -> Result<SpcState, CodecError> {
        let [a, x, y, sp] = self.array("spc")?;
        Ok(SpcState {
            a,
            x,
            y,
            sp,
            pc: self.u16("spc pc")?,
            psw: self.u8("spc psw")?,
            stopped: self.bool("spc stopped")?,
            cpu_to_apu: self.array("spc ports")?,
            apu_to_cpu: self.array("spc ports")?,
            ram: self.seq8("spc ram")?,
            dsp: self.array("spc dsp")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> FrameRecord {
        FrameRecord {
            index: 7,
            video_frame: 8539,
            pad: Pad(0x8010),
            cut: Cut::NmiReturn(Nmi {
                kind: NmiKind::Overrun,
                interrupted_pc: 0x85_846D,
                polled: true,
            }),
            joypad_reads: 3,
            wait_entry: Some(Beam { v: 200, h: 1000 }),
            overscan: false,
            cpu: CpuRegisters {
                a: 1,
                s: 0x1FF0,
                k: 0x85,
                emulation: false,
                ..CpuRegisters::default()
            },
            wram: vec![0x55; 0x2_0000].into_boxed_slice(),
            vram: (0..0x8000u16).collect(),
            cgram: vec![0x7FFF; 0x100].into_boxed_slice(),
            oam: vec![1; 0x220].into_boxed_slice(),
            sram: vec![2; 0x2000].into_boxed_slice(),
            ppu: PpuRegisters {
                inidisp: 0x0F,
                m7: [-1, 2, -3, 4, -5, 6],
                bg_scroll: [[1, 2], [3, 4], [5, 6], [7, 0x3FF]],
                vtime: 31,
                dma: [[9; 11]; 8],
                ..PpuRegisters::default()
            },
            apu_log: vec![ApuAccess {
                kind: ApuAccessKind::CpuWrite,
                port: 1,
                value: 0x2A,
                apu_cycle: 8500,
            }],
            picture: Some(Picture::default()),
            spc: None,
            bank_instructions: Some(vec![4; 256].into_boxed_slice()),
            writes: vec![RegisterWrite {
                beam: Beam { v: 31, h: 900 },
                register: 0x09,
                value: 0x13,
                source: WriteSource::Irq,
            }],
            hits: vec![Hit {
                pc: 0x84_88F3,
                beam: Beam { v: 1, h: 2 },
                peek: [1, 2, 3, 4],
            }],
            legacy_ppu_digest: 0xDEAD_BEEF,
        }
    }

    fn same(a: &FrameRecord, b: &FrameRecord) -> bool {
        (
            a.index,
            a.video_frame,
            a.pad,
            a.cut,
            a.joypad_reads,
            a.wait_entry,
        ) == (
            b.index,
            b.video_frame,
            b.pad,
            b.cut,
            b.joypad_reads,
            b.wait_entry,
        ) && (
            a.overscan, a.cpu, &a.wram, &a.vram, &a.cgram, &a.oam, &a.sram,
        ) == (
            b.overscan, b.cpu, &b.wram, &b.vram, &b.cgram, &b.oam, &b.sram,
        ) && (a.ppu, &a.apu_log, &a.picture, &a.spc, &a.bank_instructions)
            == (b.ppu, &b.apu_log, &b.picture, &b.spc, &b.bank_instructions)
            && (&a.writes, &a.hits, a.legacy_ppu_digest)
                == (&b.writes, &b.hits, b.legacy_ppu_digest)
    }

    #[test]
    fn a_record_decodes_to_itself() {
        let record = sample();
        let mut bytes = Vec::new();
        encode(&record, PictureMode::Keep, &mut bytes);
        assert_eq!(
            decode(&bytes).map(|decoded| same(&decoded, &record)),
            Ok(true)
        );
    }

    #[test]
    fn dropping_the_picture_drops_only_the_picture() {
        let record = sample();
        let mut bytes = Vec::new();
        encode(&record, PictureMode::Drop, &mut bytes);
        let expected = FrameRecord {
            picture: None,
            ..record
        };
        assert_eq!(
            decode(&bytes).map(|decoded| same(&decoded, &expected)),
            Ok(true)
        );
    }

    #[test]
    fn short_and_long_bytes_are_refused() {
        let mut bytes = Vec::new();
        encode(&sample(), PictureMode::Keep, &mut bytes);
        let short = bytes
            .get(..bytes.len().saturating_sub(1))
            .unwrap_or_default();
        assert!(matches!(decode(short), Err(CodecError::Truncated(_))));
        bytes.push(0);
        assert_eq!(decode(&bytes).map(|_| ()), Err(CodecError::Trailing(1)));
        assert_eq!(decode(b"NOPE").map(|_| ()), Err(CodecError::BadMagic));
    }
}
