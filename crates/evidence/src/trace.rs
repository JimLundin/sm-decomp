use crate::canonical::{MAX_OBJECT, Reader, varint};
use crate::tree::HashWriter;
use crate::{BLOB, Id, SEGMENT, STREAM, Store};
use anyhow::{Context, Result, bail, ensure};
use sha2::{Digest, Sha256};
use std::io::{BufRead, BufWriter, Write};

const MAX_EVENTS: usize = 32768;
const MAX_LITERAL: usize = 65536;
const MAX_PARTS: usize = 2_000_000;
const INDEX_PARTS: usize = 1024;
const DATA_PORTS: [u64; 4] = [4, 24, 25, 34];

/// Native producers can submit events without formatting or reparsing text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Ppu { frame: u64, device: u64, register: u64, value: u64 },
    Apu { frame: u64, port: u64, value: u64 },
}
impl Event {
    pub fn frame(&self) -> u64 {
        match *self {
            Self::Ppu { frame, .. } | Self::Apu { frame, .. } => frame,
        }
    }
    fn burst(&self) -> Option<(u64, u8, u8)> {
        match *self {
            Self::Ppu { device, register, value, .. } if DATA_PORTS.contains(&register) && value < 256 => {
                Some((device, register as u8, value as u8))
            }
            _ => None,
        }
    }
    fn render(&self, out: &mut dyn Write) -> Result<()> {
        match *self {
            Self::Ppu { frame, device, register, value } => writeln!(out, "ppu {frame} {device} {register} {value}")?,
            Self::Apu { frame, port, value } => writeln!(out, "apu {frame} {port} {value}")?,
        }
        Ok(())
    }
}

fn parse(line: &[u8]) -> Option<Event> {
    let text = std::str::from_utf8(line.strip_suffix(b"\n")?).ok()?;
    let mut words = text.split(' ');
    let tag = words.next()?;
    let mut num = || -> Option<u64> {
        let word = words.next()?;
        if word.is_empty() || (word.len() > 1 && word.starts_with('0')) || !word.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        word.parse().ok()
    };
    let frame = num()?;
    let event = match tag {
        "ppu" => Event::Ppu { frame, device: num()?, register: num()?, value: num()? },
        "apu" => Event::Apu { frame, port: num()?, value: num()? },
        _ => return None,
    };
    if words.next().is_some() { None } else { Some(event) }
}

#[derive(Clone, Debug)]
struct Part {
    frame: Option<u64>,
    typed: bool,
    id: Id,
}
#[derive(Clone, Debug)]
pub struct StreamInfo {
    pub bytes: u64,
    pub lines: u64,
    pub events: u64,
    pub legacy_sha256: Id,
    pub segments: usize,
    parts: Vec<Part>,
}

/// Buffers at most 32768 events or 64 KiB of literal bytes. Segment boundaries
/// depend on observations, so identical inputs receive identical stream ids.
pub struct TraceWriter<'a> {
    store: &'a mut Store,
    events: Vec<Event>,
    literal: Vec<u8>,
    literal_frame: Option<u64>,
    parts: Vec<Part>,
    hash: Sha256,
    bytes: u64,
    lines: u64,
    count: u64,
}
impl<'a> TraceWriter<'a> {
    pub fn new(store: &'a mut Store) -> Self {
        Self {
            store,
            events: Vec::new(),
            literal: Vec::new(),
            literal_frame: None,
            parts: Vec::new(),
            hash: Sha256::new(),
            bytes: 0,
            lines: 0,
            count: 0,
        }
    }
    fn observe(&mut self, bytes: &[u8]) -> Result<()> {
        self.hash.update(bytes);
        self.bytes = self.bytes.checked_add(bytes.len() as u64).context("stream byte count overflow")?;
        self.lines += bytes.iter().filter(|b| **b == b'\n').count() as u64;
        Ok(())
    }
    pub fn push(&mut self, event: Event) -> Result<()> {
        let mut text = Vec::with_capacity(64);
        event.render(&mut text)?;
        self.observe(&text)?;
        self.add_event(event)
    }
    fn add_event(&mut self, event: Event) -> Result<()> {
        self.flush_literal()?;
        if self.events.len() == MAX_EVENTS || self.events.first().is_some_and(|e| e.frame() != event.frame()) {
            self.flush_events()?;
        }
        self.events.push(event);
        self.count += 1;
        Ok(())
    }
    /// Preserve unrecognised bytes. A frame association is optional and explicit.
    pub fn literal(&mut self, bytes: &[u8], frame: Option<u64>) -> Result<()> {
        self.observe(bytes)?;
        self.add_literal(bytes, frame)
    }
    fn add_literal(&mut self, mut bytes: &[u8], frame: Option<u64>) -> Result<()> {
        self.flush_events()?;
        if self.literal_frame != frame {
            self.flush_literal()?;
        }
        self.literal_frame = frame;
        while !bytes.is_empty() {
            let n = bytes.len().min(MAX_LITERAL - self.literal.len());
            self.literal.extend_from_slice(&bytes[..n]);
            bytes = &bytes[n..];
            if self.literal.len() == MAX_LITERAL {
                self.flush_literal()?;
            }
        }
        Ok(())
    }
    fn part(&mut self, part: Part) -> Result<()> {
        ensure!(self.parts.len() < MAX_PARTS, "stream segment limit exceeded");
        self.parts.push(part);
        Ok(())
    }
    fn flush_literal(&mut self) -> Result<()> {
        if !self.literal.is_empty() {
            let id = self.store.put(BLOB, &self.literal)?;
            self.part(Part { frame: self.literal_frame, typed: false, id })?;
            self.literal.clear();
        }
        Ok(())
    }
    fn flush_events(&mut self) -> Result<()> {
        if self.events.is_empty() {
            return Ok(());
        }
        let frame = self.events[0].frame();
        let mut out = Vec::new();
        let mut at = 0;
        while at < self.events.len() {
            if let Some((device, _, _)) = self.events[at].burst() {
                let mut end = at;
                while end < self.events.len() && self.events[end].burst().is_some_and(|b| b.0 == device) {
                    end += 1;
                }
                if end - at >= 16 {
                    let mut pairs = Vec::with_capacity((end - at) * 2);
                    for event in &self.events[at..end] {
                        let (_, reg, value) = event.burst().unwrap();
                        pairs.extend_from_slice(&[reg, value]);
                    }
                    let id = self.store.put(BLOB, &pairs)?;
                    out.push(2);
                    varint(&mut out, device);
                    out.extend_from_slice(&id);
                    at = end;
                    continue;
                }
            }
            match self.events[at] {
                Event::Ppu { device, register, value, .. } => {
                    out.push(0);
                    varint(&mut out, device);
                    varint(&mut out, register);
                    varint(&mut out, value);
                }
                Event::Apu { port, value, .. } => {
                    out.push(1);
                    varint(&mut out, port);
                    varint(&mut out, value);
                }
            }
            at += 1;
        }
        let id = self.store.put(SEGMENT, &out)?;
        self.part(Part { frame: Some(frame), typed: true, id })?;
        self.events.clear();
        Ok(())
    }
    pub fn finish(mut self) -> Result<Id> {
        self.flush_events()?;
        self.flush_literal()?;
        let mut out = Vec::new();
        varint(&mut out, self.bytes);
        varint(&mut out, self.lines);
        varint(&mut out, self.count);
        out.extend_from_slice(&self.hash.finalize());
        varint(&mut out, self.parts.len() as u64);
        for parts in self.parts.chunks(INDEX_PARTS) {
            let mut index = Vec::new();
            varint(&mut index, parts.len() as u64);
            for p in parts {
                index.push(u8::from(p.typed));
                match p.frame {
                    Some(frame) => {
                        index.push(1);
                        varint(&mut index, frame);
                    }
                    None => index.push(0),
                }
                index.extend_from_slice(&p.id);
            }
            let id = self.store.put(b'D', &index)?;
            out.extend_from_slice(&id);
        }
        ensure!(out.len() <= MAX_OBJECT, "stream index exceeds limit");
        self.store.put(STREAM, &out)
    }
}

/// Lossless legacy ingestion. Noncanonical lines, resets and incomplete final
/// lines do not disable typed encoding for subsequent canonical events.
pub fn ingest_trace(store: &mut Store, input: &mut dyn BufRead) -> Result<Id> {
    let mut writer = TraceWriter::new(store);
    let mut line = Vec::new();
    let mut long_line = false;
    loop {
        let available = input.fill_buf()?;
        if available.is_empty() {
            break;
        }
        let newline = available.iter().position(|b| *b == b'\n');
        let n = newline.map(|i| i + 1).unwrap_or(available.len()).min(MAX_LITERAL - line.len());
        line.extend_from_slice(&available[..n]);
        input.consume(n);
        let complete = line.last() == Some(&b'\n');
        if complete || line.len() == MAX_LITERAL {
            writer.observe(&line)?;
            if !long_line && complete {
                if let Some(event) = parse(&line) {
                    writer.add_event(event)?;
                } else {
                    writer.add_literal(&line, inferred_frame(&line))?;
                }
            } else {
                writer.add_literal(&line, None)?;
            }
            long_line = !complete;
            line.clear();
        }
    }
    if !line.is_empty() {
        writer.literal(&line, if long_line { None } else { inferred_frame(&line) })?;
    }
    writer.finish()
}
fn inferred_frame(line: &[u8]) -> Option<u64> {
    std::str::from_utf8(line).ok()?.split_ascii_whitespace().nth(1)?.parse().ok()
}

pub fn read_stream(store: &Store, id: &Id) -> Result<StreamInfo> {
    let bytes = store.get_kind(id, STREAM)?;
    let mut r = Reader::new(&bytes);
    let size = r.varint()?;
    let lines = r.varint()?;
    let events = r.varint()?;
    let hash = r.id()?;
    let count = r.len(MAX_PARTS)?;
    let mut parts = Vec::with_capacity(count);
    for chunk in 0..count.div_ceil(INDEX_PARTS) {
        let index = store.get_kind(&r.id()?, b'D')?;
        let mut ir = Reader::new(&index);
        let n = ir.len(INDEX_PARTS)?;
        ensure!(n == (count - chunk * INDEX_PARTS).min(INDEX_PARTS), "stream index shape differs");
        for _ in 0..n {
            let typed = match ir.byte()? {
                0 => false,
                1 => true,
                _ => bail!("invalid segment type"),
            };
            let frame = match ir.byte()? {
                0 => None,
                1 => Some(ir.varint()?),
                _ => bail!("invalid frame flag"),
            };
            ensure!(!typed || frame.is_some(), "typed segment without frame");
            parts.push(Part { typed, frame, id: ir.id()? });
        }
        ir.done()?;
    }
    r.done()?;
    Ok(StreamInfo { bytes: size, lines, events, legacy_sha256: hash, segments: count, parts })
}

fn render(store: &Store, part: &Part, out: &mut dyn Write) -> Result<u64> {
    if !part.typed {
        out.write_all(&store.get_kind(&part.id, BLOB)?)?;
        return Ok(0);
    }
    let bytes = store.get_kind(&part.id, SEGMENT)?;
    let mut r = Reader::new(&bytes);
    let frame = part.frame.context("missing segment frame")?;
    let mut count = 0;
    while r.at < bytes.len() {
        let tag = r.byte()?;
        match tag {
            0 => {
                Event::Ppu { frame, device: r.varint()?, register: r.varint()?, value: r.varint()? }.render(out)?;
                count += 1;
            }
            1 => {
                Event::Apu { frame, port: r.varint()?, value: r.varint()? }.render(out)?;
                count += 1;
            }
            2 => {
                let device = r.varint()?;
                let pairs = store.get_kind(&r.id()?, BLOB)?;
                ensure!(
                    pairs.len() >= 32 && pairs.len() <= MAX_EVENTS * 2 && pairs.len() % 2 == 0,
                    "invalid burst size"
                );
                for p in pairs.as_chunks::<2>().0 {
                    ensure!(DATA_PORTS.contains(&u64::from(p[0])), "invalid burst register");
                    Event::Ppu { frame, device, register: u64::from(p[0]), value: u64::from(p[1]) }.render(out)?;
                    count += 1;
                }
            }
            _ => bail!("invalid event tag"),
        }
        ensure!(count <= MAX_EVENTS as u64, "segment event limit exceeded");
    }
    Ok(count)
}

struct CheckedWriter<'a> {
    out: &'a mut dyn Write,
    hash: HashWriter,
    lines: u64,
}
impl Write for CheckedWriter<'_> {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.out.write_all(b)?;
        self.hash.write_all(b)?;
        self.lines += b.iter().filter(|b| **b == b'\n').count() as u64;
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.out.flush()
    }
}
/// Reconstruct and authenticate the full legacy stream (including literal bytes).
pub fn write_stream(store: &Store, id: &Id, out: &mut dyn Write) -> Result<()> {
    let info = read_stream(store, id)?;
    let mut checked = CheckedWriter { out, hash: HashWriter::default(), lines: 0 };
    let mut events = 0;
    {
        let mut buffered = BufWriter::with_capacity(65536, &mut checked);
        for part in &info.parts {
            events += render(store, part, &mut buffered)?;
        }
        buffered.flush()?;
    }
    let hash: Id = checked.hash.0.finalize().into();
    ensure!(
        checked.hash.1 == info.bytes
            && checked.lines == info.lines
            && events == info.events
            && hash == info.legacy_sha256,
        "stream restoration seal differs"
    );
    Ok(())
}
/// Read all segments associated with a frame, in observed order. Unassigned
/// literal bytes are available through the full stream only.
pub fn write_frame(store: &Store, id: &Id, frame: u64, out: &mut dyn Write) -> Result<u64> {
    let info = read_stream(store, id)?;
    let mut events = 0;
    let mut buffered = BufWriter::with_capacity(65536, out);
    for part in &info.parts {
        if part.frame == Some(frame) {
            events += render(store, part, &mut buffered)?;
        }
    }
    buffered.flush()?;
    Ok(events)
}
