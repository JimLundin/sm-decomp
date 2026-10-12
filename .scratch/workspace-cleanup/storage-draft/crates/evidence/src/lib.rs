//! Spike of the content-addressed evidence store (`.scratch/artifact-storage/spec.md`).
//!
//! Every object is `kind byte ‖ canonical payload`; its id is the sha256 of
//! those bytes. Objects are immutable and written once, so identical pages,
//! DMA payloads, trace segments, images and whole runs are shared by
//! construction. A run's seal is the id of its root tree.

use anyhow::{Context, Result, bail, ensure};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

pub type Id = [u8; 32];

pub const BLOB: u8 = b'B';
pub const NODE: u8 = b'N';
pub const IMAGE: u8 = b'I';
pub const SEGMENT: u8 = b'G';
pub const STREAM: u8 = b'S';
pub const TREE: u8 = b'T';

/// Memory images are split into pages of this size.
pub const PAGE: usize = 256;
/// Fanout of image Merkle nodes.
pub const FANOUT: usize = 16;
/// Trace segments cover this many emulated frames.
pub const SEGMENT_FRAMES: u64 = 64;
/// Minimum consecutive data-port writes that become one `Block` record.
pub const MIN_BURST: usize = 16;
/// PPU data ports written once per byte by DMA: OAMDATA, VMDATAL, VMDATAH, CGDATA.
pub const DATA_PORTS: [u64; 4] = [0x04, 0x18, 0x19, 0x22];
/// Trace record tags; index is the encoded tag.
pub const TAGS: [&str; 2] = ["ppu", "apu"];

pub fn hex(id: &Id) -> String {
    id.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn parse_id(text: &str) -> Result<Id> {
    ensure!(text.len() == 64, "object id must be 64 hex digits");
    let mut id = [0; 32];
    for (i, byte) in id.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[2 * i..2 * i + 2], 16)?;
    }
    Ok(id)
}

#[derive(Default, Debug, Clone)]
pub struct KindStats {
    pub new_objects: u64,
    pub new_bytes: u64,
    pub shared_objects: u64,
    pub shared_bytes: u64,
}

pub struct Store {
    root: PathBuf,
    pub stats: BTreeMap<u8, KindStats>,
}

impl Store {
    pub fn open(root: &Path) -> Result<Self> {
        fs::create_dir_all(root.join("objects"))?;
        Ok(Self { root: root.to_owned(), stats: BTreeMap::new() })
    }

    fn path(&self, id: &Id) -> PathBuf {
        let name = hex(id);
        self.root.join("objects").join(&name[..2]).join(&name[2..])
    }

    pub fn put(&mut self, kind: u8, payload: &[u8]) -> Result<Id> {
        let mut hasher = Sha256::new();
        hasher.update([kind]);
        hasher.update(payload);
        let id: Id = hasher.finalize().into();
        let path = self.path(&id);
        let stats = self.stats.entry(kind).or_default();
        let size = payload.len() as u64 + 1;
        if path.exists() {
            stats.shared_objects += 1;
            stats.shared_bytes += size;
            return Ok(id);
        }
        stats.new_objects += 1;
        stats.new_bytes += size;
        fs::create_dir_all(path.parent().unwrap())?;
        let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
        let mut file = fs::File::create(&temporary)?;
        file.write_all(&[kind])?;
        file.write_all(payload)?;
        drop(file);
        fs::rename(&temporary, &path)?;
        Ok(id)
    }

    pub fn get(&self, id: &Id) -> Result<(u8, Vec<u8>)> {
        let bytes = fs::read(self.path(id)).with_context(|| format!("missing object {}", hex(id)))?;
        let digest: Id = Sha256::digest(&bytes).into();
        ensure!(&digest == id, "object {} is corrupt", hex(id));
        ensure!(!bytes.is_empty(), "empty object {}", hex(id));
        Ok((bytes[0], bytes[1..].to_vec()))
    }

    fn get_kind(&self, id: &Id, kind: u8) -> Result<Vec<u8>> {
        let (actual, payload) = self.get(id)?;
        ensure!(actual == kind, "object {} has kind {}, expected {}", hex(id), actual as char, kind as char);
        Ok(payload)
    }
}

// ---- canonical encoding primitives -------------------------------------------------------

fn put_varint(out: &mut Vec<u8>, mut value: u64) {
    while value >= 0x80 {
        out.push(value as u8 | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, at: 0 }
    }
    fn done(&self) -> bool {
        self.at == self.bytes.len()
    }
    fn byte(&mut self) -> Result<u8> {
        let byte = *self.bytes.get(self.at).context("truncated object")?;
        self.at += 1;
        Ok(byte)
    }
    fn varint(&mut self) -> Result<u64> {
        let (mut value, mut shift) = (0u64, 0);
        loop {
            let byte = self.byte()?;
            ensure!(shift < 64, "varint overflow");
            value |= u64::from(byte & 0x7f) << shift;
            if byte < 0x80 {
                // Canonical: no redundant trailing zero groups.
                ensure!(byte != 0 || shift == 0, "non-canonical varint");
                return Ok(value);
            }
            shift += 7;
        }
    }
    fn take(&mut self, len: usize) -> Result<&'a [u8]> {
        ensure!(self.at + len <= self.bytes.len(), "truncated object");
        let slice = &self.bytes[self.at..self.at + len];
        self.at += len;
        Ok(slice)
    }
    fn id(&mut self) -> Result<Id> {
        Ok(self.take(32)?.try_into().unwrap())
    }
}

// ---- memory images: Merkle tree of pages -------------------------------------------------

pub fn put_image(store: &mut Store, bytes: &[u8]) -> Result<Id> {
    let mut level: Vec<Id> = bytes.chunks(PAGE).map(|page| store.put(BLOB, page)).collect::<Result<_>>()?;
    let mut height = 0u64;
    while level.len() > 1 {
        height += 1;
        level = level
            .chunks(FANOUT)
            .map(|children| store.put(NODE, &children.concat()))
            .collect::<Result<_>>()?;
    }
    let mut payload = Vec::new();
    put_varint(&mut payload, bytes.len() as u64);
    put_varint(&mut payload, height);
    if let Some(root) = level.first() {
        payload.extend_from_slice(root);
    }
    store.put(IMAGE, &payload)
}

/// Page ids of an image in address order.
fn image_pages(store: &Store, id: &Id) -> Result<(u64, Vec<Id>)> {
    let payload = store.get_kind(id, IMAGE)?;
    let mut reader = Reader::new(&payload);
    let len = reader.varint()?;
    let height = reader.varint()?;
    if len == 0 {
        return Ok((0, Vec::new()));
    }
    let mut level = vec![reader.id()?];
    for _ in 0..height {
        let mut next = Vec::new();
        for node in &level {
            next.extend(store.get_kind(node, NODE)?.chunks(32).map(|c| <Id>::try_from(c).unwrap()));
        }
        level = next;
    }
    Ok((len, level))
}

pub fn write_image(store: &Store, id: &Id, out: &mut dyn Write) -> Result<()> {
    let (len, pages) = image_pages(store, id)?;
    let mut written = 0u64;
    for page in pages {
        let bytes = store.get_kind(&page, BLOB)?;
        out.write_all(&bytes)?;
        written += bytes.len() as u64;
    }
    ensure!(written == len, "image length mismatch");
    Ok(())
}

/// Differing page offsets between two images, descending only into unequal subtrees.
/// Returns (offsets, node objects read).
pub fn diff_images(store: &Store, left: &Id, right: &Id) -> Result<(Vec<u64>, u64)> {
    let header = |id: &Id| -> Result<(u64, u64, Option<Id>)> {
        let payload = store.get_kind(id, IMAGE)?;
        let mut reader = Reader::new(&payload);
        let len = reader.varint()?;
        let height = reader.varint()?;
        Ok((len, height, if len == 0 { None } else { Some(reader.id()?) }))
    };
    let (llen, lheight, lroot) = header(left)?;
    let (rlen, rheight, rroot) = header(right)?;
    ensure!(llen == rlen && lheight == rheight, "images have different shapes");
    let mut offsets = Vec::new();
    let mut reads = 0;
    // (left, right, height, first page index)
    let mut work = match (lroot, rroot) {
        (Some(l), Some(r)) => vec![(l, r, lheight, 0u64)],
        _ => Vec::new(),
    };
    while let Some((l, r, height, first)) = work.pop() {
        if l == r {
            continue;
        }
        if height == 0 {
            offsets.push(first * PAGE as u64);
            continue;
        }
        reads += 2;
        let lchildren = store.get_kind(&l, NODE)?;
        let rchildren = store.get_kind(&r, NODE)?;
        let span = (FANOUT as u64).pow(height as u32 - 1);
        for (i, (lc, rc)) in lchildren.chunks(32).zip(rchildren.chunks(32)).enumerate().rev() {
            work.push((lc.try_into().unwrap(), rc.try_into().unwrap(), height - 1, first + i as u64 * span));
        }
    }
    offsets.sort();
    Ok((offsets, reads))
}

// ---- hardware event traces ---------------------------------------------------------------

/// One parsed `hardware.log` line: `<tag> <frame> <fields...>`.
struct Event {
    tag: u8,
    frame: u64,
    fields: Vec<u64>,
}

fn parse_event(line: &[u8]) -> Result<Event> {
    let text = std::str::from_utf8(line)?;
    let mut words = text.split(' ');
    let tag = words.next().context("empty event")?;
    let tag = TAGS.iter().position(|t| *t == tag).with_context(|| format!("unknown event tag {tag:?}"))? as u8;
    let frame = words.next().context("event without frame")?.parse()?;
    let fields = words.map(str::parse).collect::<Result<Vec<u64>, _>>()?;
    ensure!(fields.len() < 16, "too many event fields");
    Ok(Event { tag, frame, fields })
}

fn render_event(out: &mut Vec<u8>, tag: u8, frame: u64, fields: &[u64]) {
    out.extend_from_slice(TAGS[tag as usize].as_bytes());
    write!(out, " {frame}").unwrap();
    for field in fields {
        write!(out, " {field}").unwrap();
    }
    out.push(b'\n');
}

fn burst_member(event: &Event, head: &Event) -> bool {
    event.tag == 0
        && event.frame == head.frame
        && event.fields.len() == 3
        && event.fields[0] == head.fields[0]
        && DATA_PORTS.contains(&event.fields[1])
        && event.fields[2] < 256
}

/// Segment record layout (all varints unless noted):
///   Write: header = 0x00 | tag << 4 | nfields, frame delta, fields...
///   Block: header = 0x80 | tag << 4,           frame delta, field0, payload id (32 bytes)
/// A block payload is the `(reg, value)` byte pairs of a data-port burst, stored as a blob.
fn encode_segment(store: &mut Store, base: u64, events: &[Event]) -> Result<Id> {
    let mut out = Vec::new();
    let mut previous = base;
    let mut at = 0;
    while at < events.len() {
        let head = &events[at];
        let delta = head.frame - previous;
        previous = head.frame;
        let mut end = at;
        if head.tag == 0 && head.fields.len() == 3 && DATA_PORTS.contains(&head.fields[1]) && head.fields[2] < 256 {
            while end < events.len() && burst_member(&events[end], head) {
                end += 1;
            }
        }
        if end - at >= MIN_BURST {
            let payload: Vec<u8> = events[at..end].iter().flat_map(|e| [e.fields[1] as u8, e.fields[2] as u8]).collect();
            let blob = store.put(BLOB, &payload)?;
            out.push(0x80 | head.tag << 4);
            put_varint(&mut out, delta);
            put_varint(&mut out, head.fields[0]);
            out.extend_from_slice(&blob);
            at = end;
        } else {
            out.push(head.tag << 4 | head.fields.len() as u8);
            put_varint(&mut out, delta);
            for field in &head.fields {
                put_varint(&mut out, *field);
            }
            at += 1;
        }
    }
    store.put(SEGMENT, &out)
}

fn render_segment(store: &Store, id: &Id, base: u64, out: &mut Vec<u8>) -> Result<()> {
    let payload = store.get_kind(id, SEGMENT)?;
    let mut reader = Reader::new(&payload);
    let mut frame = base;
    while !reader.done() {
        let header = reader.byte()?;
        let tag = header >> 4 & 0x7;
        ensure!((tag as usize) < TAGS.len(), "bad tag");
        frame += reader.varint()?;
        if header & 0x80 != 0 {
            let field0 = reader.varint()?;
            let blob = store.get_kind(&reader.id()?, BLOB)?;
            ensure!(blob.len() % 2 == 0, "odd block payload");
            for pair in blob.chunks(2) {
                render_event(out, tag, frame, &[field0, u64::from(pair[0]), u64::from(pair[1])]);
            }
        } else {
            let fields = (0..header & 0xf).map(|_| reader.varint()).collect::<Result<Vec<_>>>()?;
            render_event(out, tag, frame, &fields);
        }
    }
    Ok(())
}

/// Parse one line as a canonical event that continues the stream, or `None` if it does not.
fn canonical_event(line: &[u8], last_frame: u64, rendered: &mut Vec<u8>) -> Option<Event> {
    if line.last() != Some(&b'\n') {
        return None;
    }
    let event = parse_event(&line[..line.len() - 1]).ok()?;
    rendered.clear();
    render_event(rendered, event.tag, event.frame, &event.fields);
    (rendered == line && event.frame >= last_frame).then_some(event)
}

/// Ingest a text event stream. Every line is checked to re-render byte-identically, so the
/// stored stream is a lossless, canonical representation of the input. Parsing stops at the
/// first line that is not a canonical continuation (for example a line cut off by an
/// interrupted run); everything from there on is kept verbatim as a tail blob.
/// Fails only when not even the first line is an event, so callers can store a blob instead.
pub fn put_stream(store: &mut Store, path: &Path) -> Result<Id> {
    let mut input = BufReader::with_capacity(1 << 20, fs::File::open(path)?);
    let mut segments: Vec<(u64, Id)> = Vec::new();
    let mut pending: Vec<Event> = Vec::new();
    let mut pending_index = 0u64;
    let mut line = Vec::new();
    let mut rendered = Vec::new();
    let mut last_frame = 0u64;
    let mut lines = 0u64;
    let mut tail = None;
    loop {
        line.clear();
        if input.read_until(b'\n', &mut line)? == 0 {
            break;
        }
        let Some(event) = canonical_event(&line, last_frame, &mut rendered) else {
            ensure!(lines > 0, "not an event stream");
            let mut rest = std::mem::take(&mut line);
            std::io::Read::read_to_end(&mut input, &mut rest)?;
            tail = Some(store.put(BLOB, &rest)?);
            break;
        };
        last_frame = event.frame;
        lines += 1;
        let index = event.frame / SEGMENT_FRAMES;
        if !pending.is_empty() && index != pending_index {
            segments.push((pending_index, encode_segment(store, pending_index * SEGMENT_FRAMES, &pending)?));
            pending.clear();
        }
        pending_index = index;
        pending.push(event);
    }
    if !pending.is_empty() {
        segments.push((pending_index, encode_segment(store, pending_index * SEGMENT_FRAMES, &pending)?));
    }
    // Stream: line count, segment count, (segment index delta, segment id) pairs, then a
    // tail flag byte and, if set, the tail blob id.
    let mut payload = Vec::new();
    put_varint(&mut payload, lines);
    put_varint(&mut payload, segments.len() as u64);
    let mut previous = 0;
    for (index, id) in &segments {
        put_varint(&mut payload, index - previous);
        previous = *index;
        payload.extend_from_slice(id);
    }
    match tail {
        Some(id) => {
            payload.push(1);
            payload.extend_from_slice(&id);
        }
        None => payload.push(0),
    }
    store.put(STREAM, &payload)
}

pub fn write_stream(store: &Store, id: &Id, out: &mut dyn Write) -> Result<()> {
    let payload = store.get_kind(id, STREAM)?;
    let mut reader = Reader::new(&payload);
    let lines = reader.varint()?;
    let count = reader.varint()?;
    let mut index = 0;
    let mut buffer = Vec::new();
    let mut rendered = 0u64;
    for _ in 0..count {
        index += reader.varint()?;
        buffer.clear();
        render_segment(store, &reader.id()?, index * SEGMENT_FRAMES, &mut buffer)?;
        rendered += buffer.iter().filter(|b| **b == b'\n').count() as u64;
        out.write_all(&buffer)?;
    }
    ensure!(rendered == lines, "stream line count mismatch");
    match reader.byte()? {
        0 => {}
        1 => out.write_all(&store.get_kind(&reader.id()?, BLOB)?)?,
        _ => bail!("bad stream tail flag"),
    }
    ensure!(reader.done(), "trailing stream bytes");
    Ok(())
}

// ---- run trees ---------------------------------------------------------------------------

/// Tree entry kinds. The entry kind decides how a file was modelled.
pub const ENTRY_BLOB: u8 = b'b';
pub const ENTRY_IMAGE: u8 = b'i';
pub const ENTRY_STREAM: u8 = b's';
pub const ENTRY_TREE: u8 = b't';
pub const ENTRY_LINK: u8 = b'l';

fn classify(name: &str) -> u8 {
    if name == "hardware.log" || name == "routine-calls.log" {
        ENTRY_STREAM
    } else if [".wram", ".rom", ".bin", ".srm", ".sram"].iter().any(|s| name.ends_with(s)) {
        ENTRY_IMAGE
    } else {
        ENTRY_BLOB
    }
}

/// Tree payload: entries sorted by name bytes; each is
/// `name length varint ‖ name ‖ entry kind ‖ id`.
pub fn put_tree(store: &mut Store, directory: &Path, progress: &mut dyn FnMut(&Path)) -> Result<Id> {
    let mut names: Vec<_> = fs::read_dir(directory)?.map(|e| e.map(|e| e.file_name())).collect::<Result<_, _>>()?;
    names.sort();
    let mut payload = Vec::new();
    for name in names {
        let path = directory.join(&name);
        let text = name.to_str().with_context(|| format!("non-UTF-8 name {path:?}"))?;
        let metadata = fs::symlink_metadata(&path)?;
        let (kind, id) = if metadata.file_type().is_symlink() {
            (ENTRY_LINK, store.put(BLOB, fs::read_link(&path)?.as_os_str().as_encoded_bytes())?)
        } else if metadata.is_dir() {
            (ENTRY_TREE, put_tree(store, &path, progress)?)
        } else {
            progress(&path);
            match classify(text) {
                ENTRY_STREAM => match put_stream(store, &path) {
                    Ok(id) => (ENTRY_STREAM, id),
                    // Not a parseable event log: keep it losslessly as a blob.
                    Err(_) => (ENTRY_BLOB, store.put(BLOB, &fs::read(&path)?)?),
                },
                ENTRY_IMAGE => (ENTRY_IMAGE, put_image(store, &fs::read(&path)?)?),
                _ => (ENTRY_BLOB, store.put(BLOB, &fs::read(&path)?)?),
            }
        };
        put_varint(&mut payload, text.len() as u64);
        payload.extend_from_slice(text.as_bytes());
        payload.push(kind);
        payload.extend_from_slice(&id);
    }
    store.put(TREE, &payload)
}

pub struct Entry {
    pub name: String,
    pub kind: u8,
    pub id: Id,
}

pub fn read_tree(store: &Store, id: &Id) -> Result<Vec<Entry>> {
    let payload = store.get_kind(id, TREE)?;
    let mut reader = Reader::new(&payload);
    let mut entries: Vec<Entry> = Vec::new();
    while !reader.done() {
        let len = reader.varint()? as usize;
        let name = String::from_utf8(reader.take(len)?.to_vec())?;
        if let Some(previous) = entries.last() {
            ensure!(previous.name.as_bytes() < name.as_bytes(), "tree entries not sorted");
        }
        let kind = reader.byte()?;
        entries.push(Entry { name, kind, id: reader.id()? });
    }
    Ok(entries)
}

/// Re-materialize a stored file exactly as it was ingested.
pub fn write_entry(store: &Store, kind: u8, id: &Id, out: &mut dyn Write) -> Result<()> {
    match kind {
        ENTRY_BLOB | ENTRY_LINK => out.write_all(&store.get_kind(id, BLOB)?)?,
        ENTRY_IMAGE => write_image(store, id, out)?,
        ENTRY_STREAM => write_stream(store, id, out)?,
        _ => bail!("cannot materialize entry kind {}", kind as char),
    }
    Ok(())
}

struct HashWriter(Sha256, u64);

impl Write for HashWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.update(bytes);
        self.1 += bytes.len() as u64;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn file_digest(path: &Path) -> Result<(Id, u64)> {
    let mut writer = HashWriter(Sha256::new(), 0);
    std::io::copy(&mut fs::File::open(path)?, &mut writer)?;
    Ok((writer.0.finalize().into(), writer.1))
}

/// Check that every file under `directory` re-materializes from the tree with the same
/// sha256 and length. Returns (files verified, bytes verified).
pub fn verify_tree(store: &Store, id: &Id, directory: &Path) -> Result<(u64, u64)> {
    let mut totals = (0, 0);
    for entry in read_tree(store, id)? {
        let path = directory.join(&entry.name);
        if entry.kind == ENTRY_TREE {
            let (files, bytes) = verify_tree(store, &entry.id, &path)?;
            totals = (totals.0 + files, totals.1 + bytes);
            continue;
        }
        if entry.kind == ENTRY_LINK {
            let target = fs::read_link(&path)?;
            ensure!(store.get_kind(&entry.id, BLOB)? == target.as_os_str().as_encoded_bytes(), "{path:?} link differs");
            continue;
        }
        let mut writer = HashWriter(Sha256::new(), 0);
        write_entry(store, entry.kind, &entry.id, &mut writer)?;
        let rendered: Id = writer.0.finalize().into();
        let (original, len) = file_digest(&path)?;
        ensure!(rendered == original && writer.1 == len, "{path:?} does not round-trip");
        totals = (totals.0 + 1, totals.1 + len);
    }
    Ok(totals)
}
