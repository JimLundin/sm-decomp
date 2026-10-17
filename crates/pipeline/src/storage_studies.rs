//! Bounded, read-only storage experiments with typed TOML summaries.
//!
//! These measurements are not acceptance verdicts. Native spike runs use the
//! production binary evidence tree and a separately sealed TOML receipt; the
//! historical SQLite/JSON spike is intentionally a different format.
use anyhow::{Context, Result, bail, ensure};
use evidence::{Entry, EntryKind, Id, Store};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::Instant;

const MIB: usize = 1 << 20;
const MEMORY_BUDGET: usize = 512 * MIB;
const RECEIPT_SCHEMA: &str = "storage-study-run-v2";

#[derive(Default)]
struct Options {
    values: BTreeMap<String, Vec<String>>,
    positionals: Vec<String>,
}
impl Options {
    fn parse(args: &[String], flags: &[&str], keys: &[&str]) -> Result<Self> {
        let mut result = Self::default();
        let mut index = 0;
        while index < args.len() {
            let word = &args[index];
            if word.starts_with("--") {
                ensure!(flags.contains(&word.as_str()) || keys.contains(&word.as_str()), "unknown option {word}");
                let value = if flags.contains(&word.as_str()) {
                    String::new()
                } else {
                    index += 1;
                    args.get(index).filter(|v| !v.starts_with("--")).context("option requires a value")?.clone()
                };
                result.values.entry(word.clone()).or_default().push(value);
            } else {
                result.positionals.push(word.clone());
            }
            index += 1;
        }
        Ok(result)
    }
    fn one(&self, key: &str) -> Result<&str> {
        let values = self.values.get(key).with_context(|| format!("required option {key}"))?;
        ensure!(values.len() == 1, "option {key} may occur only once");
        Ok(&values[0])
    }
    fn optional(&self, key: &str) -> Result<Option<&str>> {
        self.values.get(key).map(|_| self.one(key)).transpose()
    }
    fn number(&self, key: &str, default: usize, min: usize, max: usize) -> Result<usize> {
        let n = self.optional(key)?.map(str::parse).transpose()?.unwrap_or(default);
        ensure!((min..=max).contains(&n), "{key} must be {min}..={max}");
        Ok(n)
    }
    fn flag(&self, key: &str) -> Result<bool> {
        Ok(self.optional(key)?.is_some())
    }
    fn roots(&self) -> Result<Vec<PathBuf>> {
        let roots = self.values.get("--capture-root").context("required option --capture-root")?;
        ensure!(!roots.is_empty() && roots.len() <= 64, "capture count must be 1..=64");
        roots.iter().map(|r| capture_root(Path::new(r))).collect()
    }
    fn no_positionals(&self) -> Result<()> {
        ensure!(self.positionals.is_empty(), "unexpected positional arguments");
        Ok(())
    }
}

pub fn run(args: &[String]) -> Result<()> {
    match args.first().map(String::as_str) {
        Some("format") => format_study(&args[1..]),
        Some("merkle") => merkle_study(&args[1..]),
        Some("benchmark") => benchmark(&args[1..], false),
        Some("rust-benchmark") => benchmark(&args[1..], true),
        Some("verify") => {
            let options = Options::parse(&args[1..], &[], &["--store", "--run"])?;
            options.no_positionals()?;
            let store = Store::open(Path::new(options.one("--store")?))?;
            let receipt = verify_receipt(&store, Path::new(options.one("--store")?), options.one("--run")?)?;
            store.verify_packs()?;
            println!("Verified {}: {} files, {} bytes", options.one("--run")?, receipt.members.len(), receipt.bytes);
            Ok(())
        }
        Some("extract") => {
            let options = Options::parse(&args[1..], &[], &["--store", "--run", "--member", "--output"])?;
            options.no_positionals()?;
            let root = Path::new(options.one("--store")?);
            let store = Store::open(root)?;
            let receipt = verify_receipt(&store, root, options.one("--run")?)?;
            let entry = find_entry(&store, evidence::parse_id(&receipt.observations)?, options.one("--member")?)?;
            ensure!(entry.kind != EntryKind::Tree, "member is a directory");
            let output = safe_output(Path::new(options.one("--output")?), &[root.to_path_buf()])?;
            let (mut owned, mut file) = TemporaryFile::create(output)?;
            evidence::write_entry(&store, entry.kind, &entry.id, &mut file)?;
            file.sync_all()?;
            owned.owned = false;
            Ok(())
        }
        Some("cold-read") => cold_read(&args[1..]),
        _ => bail!("usage: sm-pipeline storage-study format|merkle|benchmark|rust-benchmark|verify|extract [OPTIONS]"),
    }
}

fn capture_root(path: &Path) -> Result<PathBuf> {
    ensure!(fs::symlink_metadata(path)?.is_dir(), "capture root must be a directory");
    Ok(path.canonicalize()?)
}
fn absolute(path: &Path) -> Result<PathBuf> {
    let path = if path.is_absolute() { path.to_path_buf() } else { std::env::current_dir()?.join(path) };
    let mut resolved = PathBuf::new();
    for part in path.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                resolved.pop();
            }
            other => resolved.push(other.as_os_str()),
        }
    }
    // Resolve the longest existing ancestor, including symlinks, before checking containment.
    let mut ancestor = resolved.clone();
    let mut suffix = Vec::new();
    while !ancestor.exists() {
        suffix.push(ancestor.file_name().context("output has no existing ancestor")?.to_owned());
        ensure!(ancestor.pop(), "output has no existing ancestor");
    }
    let mut result = ancestor.canonicalize()?;
    for part in suffix.into_iter().rev() {
        result.push(part);
    }
    Ok(result)
}
fn safe_output(path: &Path, roots: &[PathBuf]) -> Result<PathBuf> {
    let output = absolute(path)?;
    for root in roots {
        let root = root.canonicalize()?;
        ensure!(!output.starts_with(&root), "output must be outside source capture/store");
    }
    ensure!(!output.exists() && fs::symlink_metadata(&output).is_err(), "output already exists");
    Ok(output)
}
fn save<T: Serialize>(path: &Path, report: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(toml::to_string_pretty(report)?.as_bytes())?;
    file.sync_all()?;
    Ok(())
}
fn sha(bytes: &[u8]) -> String {
    evidence::hex(&Sha256::digest(bytes).into())
}
fn file_sha(path: &Path) -> Result<String> {
    ensure!(fs::symlink_metadata(path)?.is_file(), "source is not a regular file: {}", path.display());
    let mut digest = Sha256::new();
    let mut buffer = vec![0; MIB];
    let mut file = File::open(path)?;
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        digest.update(&buffer[..n]);
    }
    Ok(evidence::hex(&digest.finalize().into()))
}
fn matching(directory: &Path, prefix: &str, suffix: &str) -> Result<Vec<PathBuf>> {
    if !directory.exists() {
        return Ok(Vec::new());
    }
    ensure!(fs::symlink_metadata(directory)?.is_dir(), "snapshot directory is not regular");
    let mut paths = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_str().context("non-UTF-8 snapshot name")?;
        if name.starts_with(prefix) && name.ends_with(suffix) {
            ensure!(entry.file_type()?.is_file(), "snapshot symlink/special file rejected");
            paths.push(entry.path());
        }
    }
    paths.sort();
    Ok(paths)
}
fn snapshot_sets(root: &Path, rom: bool) -> Result<Vec<(&'static str, Vec<PathBuf>)>> {
    let mut result =
        vec![("wram", matching(&root.join("fixtures"), "", ".wram")?), ("frames", matching(root, "frame-", ".bin")?)];
    if rom {
        result.push(("rom-views", matching(&root.join("fixtures"), "", ".rom")?));
    }
    Ok(result)
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Member {
    name: String,
    bytes: u64,
    sha256: String,
}
fn inventory(paths: &[PathBuf], root: &Path) -> Result<Vec<Member>> {
    paths
        .iter()
        .map(|path| {
            Ok(Member {
                name: path.strip_prefix(root)?.to_str().context("non-UTF-8 path")?.to_owned(),
                bytes: fs::symlink_metadata(path)?.len(),
                sha256: file_sha(path)?,
            })
        })
        .collect()
}
fn inventory_hash(members: &[Member]) -> Result<String> {
    #[derive(Serialize)]
    struct Inventory<'a> {
        schema: &'static str,
        members: &'a [Member],
    }
    Ok(sha(toml::to_string(&Inventory { schema: "storage-study-inventory-v2", members })?.as_bytes()))
}
fn read_members(paths: &[PathBuf]) -> Result<Vec<Vec<u8>>> {
    let mut used = 0usize;
    let mut result = Vec::new();
    for path in paths {
        let len = usize::try_from(fs::symlink_metadata(path)?.len())?;
        used = used.checked_add(len).context("snapshot size overflow")?;
        ensure!(used <= MEMORY_BUDGET, "snapshot sample exceeds 512 MiB; reduce --snapshot-limit");
        result.push(fs::read(path)?);
    }
    Ok(result)
}
fn compressed(data: &[u8], level: i32) -> Result<Vec<u8>> {
    Ok(zstd::bulk::compress(data, level)?)
}
fn restored(data: &[u8], size: usize) -> Result<Vec<u8>> {
    let decoded = zstd::bulk::decompress(data, size)?;
    ensure!(decoded.len() == size, "decoded length differs");
    Ok(decoded)
}
#[derive(Serialize)]
struct Codec {
    codec: String,
    bytes: usize,
    ratio: f64,
    compress_seconds: f64,
    decompress_seconds: f64,
}
fn codecs(data: &[u8]) -> Result<Vec<Codec>> {
    let mut rows = Vec::new();
    for (name, level) in [("gzip-1", 0), ("zstd-1", 1), ("zstd-3", 3), ("zstd-9", 9)] {
        let start = Instant::now();
        let packed = if level == 0 {
            let mut encoder = flate2::GzBuilder::new().mtime(0).write(Vec::new(), flate2::Compression::fast());
            encoder.write_all(data)?;
            encoder.finish()?
        } else {
            compressed(data, level)?
        };
        let compressed_at = start.elapsed();
        let decoded = if level == 0 {
            let mut result = Vec::new();
            flate2::read::GzDecoder::new(packed.as_slice()).take(data.len() as u64 + 1).read_to_end(&mut result)?;
            result
        } else {
            restored(&packed, data.len())?
        };
        ensure!(decoded == data, "codec round trip differs");
        rows.push(Codec {
            codec: name.into(),
            bytes: packed.len(),
            ratio: ratio(data.len(), packed.len()),
            compress_seconds: compressed_at.as_secs_f64(),
            decompress_seconds: (start.elapsed() - compressed_at).as_secs_f64(),
        });
    }
    Ok(rows)
}
fn ratio(raw: usize, stored: usize) -> f64 {
    raw as f64 / stored.max(1) as f64
}
fn varint(mut value: u64, out: &mut Vec<u8>) {
    while value >= 128 {
        out.push((value as u8 & 127) | 128);
        value >>= 7;
    }
    out.push(value as u8);
}
fn read_varint(data: &[u8], offset: &mut usize) -> Result<u64> {
    let mut result = 0;
    for shift in (0..70).step_by(7) {
        let byte = *data.get(*offset).context("truncated varint")?;
        *offset += 1;
        ensure!(shift != 63 || byte <= 1, "overflow varint");
        result |= (byte as u64 & 127) << shift;
        if byte & 128 == 0 {
            return Ok(result);
        }
    }
    bail!("overflow varint")
}
#[derive(Clone, Debug)]
struct DecimalEvent {
    frame: i64,
    payload: EventPayload,
}
#[derive(Clone, Debug)]
enum EventPayload {
    Ppu(u8, u8, u8),
    Apu(u32, u8),
}
impl DecimalEvent {
    fn parse(line: &[u8]) -> Result<Self> {
        let line = std::str::from_utf8(line)?.strip_suffix('\n').context("trace record lacks newline")?;
        let words: Vec<_> = line.split(' ').collect();
        ensure!(words.iter().all(|w| !w.is_empty()), "trace uses noncanonical spacing");
        let frame = words.get(1).context("trace record lacks frame")?.parse()?;
        let payload = match words.as_slice() {
            ["ppu", _, device, address, value] => EventPayload::Ppu(device.parse()?, address.parse()?, value.parse()?),
            ["apu", _, address, value] => EventPayload::Apu(address.parse()?, value.parse()?),
            _ => bail!("unsupported event grammar"),
        };
        Ok(Self { frame, payload })
    }
    fn render(&self, out: &mut dyn Write) -> Result<()> {
        match self.payload {
            EventPayload::Ppu(device, address, value) => {
                writeln!(out, "ppu {} {device} {address} {value}", self.frame)?
            }
            EventPayload::Apu(address, value) => writeln!(out, "apu {} {address} {value}", self.frame)?,
        }
        Ok(())
    }
    fn native(&self) -> Result<evidence::Event> {
        let frame = u64::try_from(self.frame).context("native writer benchmark requires unsigned frame labels")?;
        Ok(match self.payload {
            EventPayload::Ppu(device, register, value) => {
                evidence::Event::Ppu { frame, device: device.into(), register: register.into(), value: value.into() }
            }
            EventPayload::Apu(port, value) => evidence::Event::Apu { frame, port: port.into(), value: value.into() },
        })
    }
}
fn encode_events(text: &[u8]) -> Result<(Vec<u8>, usize)> {
    let mut out = Vec::new();
    let mut previous = 0i64;
    let mut count = 0;
    for line in text.split_inclusive(|b| *b == b'\n') {
        let event = DecimalEvent::parse(line)?;
        let delta = i64::try_from(event.frame as i128 - previous as i128).context("frame delta overflow")?;
        varint(((delta as u64) << 1) ^ ((delta >> 63) as u64), &mut out);
        match event.payload {
            EventPayload::Ppu(device, address, value) => out.extend_from_slice(&[0, device, address, value]),
            EventPayload::Apu(address, value) => {
                out.push(1);
                out.extend_from_slice(&address.to_le_bytes());
                out.push(value);
            }
        }
        previous = event.frame;
        count += 1;
    }
    Ok((out, count))
}
fn decode_events(data: &[u8]) -> Result<Vec<u8>> {
    let mut offset = 0;
    let mut frame = 0i64;
    let mut out = Vec::new();
    while offset < data.len() {
        let delta = read_varint(data, &mut offset)?;
        let delta = (delta >> 1) as i64 ^ -((delta & 1) as i64);
        frame = frame.checked_add(delta).context("frame label overflow")?;
        let tag = *data.get(offset).context("truncated event tag")?;
        offset += 1;
        let bytes = match tag {
            0 => 3,
            1 => 5,
            _ => bail!("unknown event tag"),
        };
        let end = offset.checked_add(bytes).context("record offset overflow")?;
        let payload = data.get(offset..end).context("truncated event record")?;
        let payload = if tag == 0 {
            EventPayload::Ppu(payload[0], payload[1], payload[2])
        } else {
            EventPayload::Apu(u32::from_le_bytes(payload[..4].try_into()?), payload[4])
        };
        DecimalEvent { frame, payload }.render(&mut out)?;
        offset = end;
    }
    Ok(out)
}
#[derive(Serialize)]
struct Controls {
    exact_events: bool,
    duplicates_order_resets: bool,
    truncation_overflow: bool,
    compression_integrity: bool,
    same_process_determinism: bool,
}
fn controls() -> Result<Controls> {
    let text = b"ppu 0 1 4 7\nppu 0 1 4 8\nppu 0 1 4 8\napu 4 16777215 255\nppu 0 0 0 0\n";
    let (binary, count) = encode_events(text)?;
    ensure!(count == 5 && decode_events(&binary)? == text, "event round-trip control failed");
    let changed = text.iter().copied().enumerate().map(|(i, b)| if i == 10 { b'9' } else { b }).collect::<Vec<_>>();
    ensure!(encode_events(&changed)?.0 != binary, "changed event control failed");
    let lines: Vec<_> = text.split_inclusive(|b| *b == b'\n').collect();
    let reordered = [lines[1], lines[0], lines[2], lines[3], lines[4]].concat();
    ensure!(encode_events(&reordered)?.0 != binary, "reordered event control failed");
    ensure!(read_varint(&[128], &mut 0).is_err() && read_varint(&[255; 10], &mut 0).is_err(), "bad varint accepted");
    ensure!(decode_events(&binary[..binary.len() - 1]).is_err(), "truncated record accepted");
    let packed = compressed(&binary, 3)?;
    ensure!(restored(&packed[..packed.len() - 1], binary.len()).is_err(), "truncated compression accepted");
    ensure!(compressed(&binary, 3)? == packed, "same-process codec determinism failed");
    Ok(Controls {
        exact_events: true,
        duplicates_order_resets: true,
        truncation_overflow: true,
        compression_integrity: true,
        same_process_determinism: true,
    })
}
#[derive(Serialize)]
struct TraceMeasurement {
    path: PathBuf,
    offset: u64,
    source_file_bytes: u64,
    text_bytes: usize,
    text_sha256: String,
    records: usize,
    binary_bytes: usize,
    encode_seconds: f64,
    decode_seconds: f64,
    text_codecs: Vec<Codec>,
    binary_codecs: Vec<Codec>,
    chunked: ChunkMeasurement,
    round_trip: bool,
}
#[derive(Serialize)]
struct ChunkMeasurement {
    chunks: usize,
    frames_per_group: usize,
    text_input_cap: usize,
    binary_bytes: usize,
    text_bytes: usize,
    index_bytes_estimate: usize,
    round_trip_seconds: f64,
}
fn trace_sample(path: &Path, numerator: u64, denominator: u64, size: usize) -> Result<(u64, Vec<u8>)> {
    ensure!(fs::symlink_metadata(path)?.is_file(), "trace is not regular file");
    let mut reader = BufReader::new(File::open(path)?);
    reader.seek(SeekFrom::Start(reader.get_ref().metadata()?.len().saturating_mul(numerator) / denominator))?;
    if numerator != 0 {
        reader.read_until(b'\n', &mut Vec::new())?;
    }
    let offset = reader.stream_position()?;
    let mut text = Vec::with_capacity(size);
    (&mut reader).take(size as u64).read_to_end(&mut text)?;
    text.truncate(text.iter().rposition(|b| *b == b'\n').map_or(0, |p| p + 1));
    Ok((offset, text))
}
fn trace_measure(path: &Path, numerator: u64, denominator: u64, size: usize) -> Result<TraceMeasurement> {
    let (offset, text) = trace_sample(path, numerator, denominator, size)?;
    let start = Instant::now();
    let (binary, records) = encode_events(&text)?;
    let encoded_at = start.elapsed();
    ensure!(decode_events(&binary)? == text, "event grammar is not byte-exact");
    let decoded_at = start.elapsed();
    let mut chunks = Vec::new();
    let mut current = Vec::new();
    let mut previous = None;
    for line in text.split_inclusive(|b| *b == b'\n') {
        let group = DecimalEvent::parse(line)?.frame.div_euclid(64);
        if !current.is_empty() && (Some(group) != previous || current.len() >= MIB) {
            chunks.push(std::mem::take(&mut current));
        }
        current.extend_from_slice(line);
        previous = Some(group);
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    let chunk_start = Instant::now();
    let mut binary_bytes = 0;
    let mut text_bytes = 0;
    for chunk in &chunks {
        let encoded = encode_events(chunk)?.0;
        let packed = compressed(&encoded, 3)?;
        ensure!(decode_events(&restored(&packed, encoded.len())?)? == *chunk, "chunk event round trip differs");
        binary_bytes += packed.len();
        let packed = compressed(chunk, 3)?;
        ensure!(restored(&packed, chunk.len())? == *chunk, "chunk text round trip differs");
        text_bytes += packed.len();
    }
    let chunk_seconds = chunk_start.elapsed().as_secs_f64();
    let mut original = File::open(path)?;
    original.seek(SeekFrom::Start(offset))?;
    let mut reread = vec![0; text.len()];
    original.read_exact(&mut reread)?;
    ensure!(reread == text, "source trace changed during experiment");
    Ok(TraceMeasurement {
        path: path.to_path_buf(),
        offset,
        source_file_bytes: original.metadata()?.len(),
        text_bytes: text.len(),
        text_sha256: sha(&text),
        records,
        binary_bytes: binary.len(),
        encode_seconds: encoded_at.as_secs_f64(),
        decode_seconds: (decoded_at - encoded_at).as_secs_f64(),
        text_codecs: codecs(&text)?,
        binary_codecs: codecs(&binary)?,
        chunked: ChunkMeasurement {
            chunks: chunks.len(),
            frames_per_group: 64,
            text_input_cap: MIB,
            binary_bytes,
            text_bytes,
            index_bytes_estimate: chunks.len() * 64,
            round_trip_seconds: chunk_seconds,
        },
        round_trip: true,
    })
}
#[derive(Serialize)]
struct Candidate {
    name: String,
    bytes: usize,
    ratio: f64,
    round_trip_seconds: f64,
    unique_pages: usize,
    raw_page_table_bytes: usize,
}
#[derive(Serialize)]
struct SnapshotMeasurement {
    label: String,
    files: usize,
    available_files: usize,
    raw_bytes: usize,
    unique_whole_files: usize,
    whole_file_dedup_bytes: usize,
    manifest_sha256: String,
    manifest_zstd_bytes: usize,
    chunk_members: usize,
    median_changed_byte_fraction: Option<f64>,
    candidates: Vec<Candidate>,
    round_trip: bool,
}
fn snapshots_measure(root: &Path, paths: &[PathBuf], label: &str, available: usize) -> Result<SnapshotMeasurement> {
    let before = inventory(paths, root)?;
    let members = read_members(paths)?;
    let raw: usize = members.iter().map(Vec::len).sum();
    let unique: BTreeSet<_> = members.iter().collect();
    let mut changed = members
        .windows(2)
        .map(|pair| {
            let shared_changes = pair[0].iter().zip(&pair[1]).filter(|(a, b)| a != b).count();
            (shared_changes + pair[0].len().abs_diff(pair[1].len())) as f64
                / pair[0].len().max(pair[1].len()).max(1) as f64
        })
        .collect::<Vec<_>>();
    changed.sort_by(f64::total_cmp);
    let median = if changed.is_empty() {
        None
    } else {
        Some((changed[(changed.len() - 1) / 2] + changed[changed.len() / 2]) / 2.0)
    };
    let mut candidates = Vec::new();
    for level in [1, 3, 9] {
        let start = Instant::now();
        let mut bytes = 0;
        for member in &members {
            let packed = compressed(member, level)?;
            ensure!(restored(&packed, member.len())? == *member, "individual snapshot differs");
            bytes += packed.len();
        }
        candidates.push(Candidate {
            name: format!("individual-zstd-{level}"),
            bytes,
            ratio: ratio(raw, bytes),
            round_trip_seconds: start.elapsed().as_secs_f64(),
            unique_pages: 0,
            raw_page_table_bytes: 0,
        });
    }
    for page_size in [256, 1024, 4096] {
        let start = Instant::now();
        let mut total = 0;
        let mut unique_pages = 0;
        let mut table_bytes = 0;
        for block in members.chunks(32) {
            let mut dictionary = BTreeMap::new();
            let mut pages = Vec::new();
            let mut tables = Vec::new();
            for member in block {
                let mut ids = Vec::new();
                for chunk in member.chunks(page_size) {
                    let mut page = vec![0; page_size];
                    page[..chunk.len()].copy_from_slice(chunk);
                    let identity = if let Some(id) = dictionary.get(&page) {
                        *id
                    } else {
                        let id = u32::try_from(pages.len() / page_size)?;
                        dictionary.insert(page.clone(), id);
                        pages.extend_from_slice(&page);
                        id
                    };
                    ids.push(identity);
                }
                tables.push(ids);
            }
            let pool_size = pages.len();
            for table in &tables {
                for id in table {
                    pages.extend_from_slice(&id.to_le_bytes());
                }
            }
            unique_pages += dictionary.len();
            table_bytes += pages.len() - pool_size;
            let packed = compressed(&pages, 3)?;
            let decoded = restored(&packed, pages.len())?;
            ensure!(decoded == pages, "page payload differs");
            let mut cursor = pool_size;
            for (member, table) in block.iter().zip(tables) {
                let mut restored_member = Vec::new();
                for _ in table {
                    let id = u32::from_le_bytes(decoded[cursor..cursor + 4].try_into()?) as usize;
                    cursor += 4;
                    restored_member.extend_from_slice(&decoded[id * page_size..(id + 1) * page_size]);
                }
                restored_member.truncate(member.len());
                ensure!(restored_member == *member, "page snapshot differs");
            }
            total += packed.len();
        }
        candidates.push(Candidate {
            name: format!("pages-{page_size}-zstd-3"),
            bytes: total,
            ratio: ratio(raw, total),
            round_trip_seconds: start.elapsed().as_secs_f64(),
            unique_pages,
            raw_page_table_bytes: table_bytes,
        });
    }
    for xor in [false, true] {
        let start = Instant::now();
        let mut bytes = 0;
        for block in members.chunks(32) {
            let mut payload = Vec::new();
            let mut previous: Option<&Vec<u8>> = None;
            for member in block {
                if xor && let Some(prev) = previous {
                    ensure!(prev.len() == member.len(), "XOR experiment requires a fixed state layout");
                    payload.extend(member.iter().zip(prev).map(|(a, b)| a ^ b));
                } else {
                    payload.extend_from_slice(member);
                }
                previous = Some(member);
            }
            let packed = compressed(&payload, 3)?;
            let decoded = restored(&packed, payload.len())?;
            let mut cursor = 0;
            let mut previous = Vec::<u8>::new();
            for member in block {
                let mut restored_member = decoded[cursor..cursor + member.len()].to_vec();
                if xor && !previous.is_empty() {
                    for (byte, old) in restored_member.iter_mut().zip(&previous) {
                        *byte ^= old;
                    }
                }
                ensure!(restored_member == *member, "chunk snapshot differs");
                previous = restored_member;
                cursor += member.len();
            }
            bytes += packed.len();
        }
        candidates.push(Candidate {
            name: format!("{}-zstd-3", if xor { "xor" } else { "concatenated" }),
            bytes,
            ratio: ratio(raw, bytes),
            round_trip_seconds: start.elapsed().as_secs_f64(),
            unique_pages: 0,
            raw_page_table_bytes: 0,
        });
    }
    ensure!(inventory(paths, root)? == before, "source snapshot changed during experiment");
    let manifest = inventory_hash(&before)?;
    Ok(SnapshotMeasurement {
        label: label.into(),
        files: paths.len(),
        available_files: available,
        raw_bytes: raw,
        unique_whole_files: unique.len(),
        whole_file_dedup_bytes: unique.iter().map(|x| x.len()).sum(),
        manifest_sha256: manifest,
        manifest_zstd_bytes: compressed(toml::to_string(&InventoryDocument { members: &before })?.as_bytes(), 3)?.len(),
        chunk_members: 32,
        median_changed_byte_fraction: median,
        candidates,
        round_trip: true,
    })
}
#[derive(Serialize)]
struct InventoryDocument<'a> {
    members: &'a [Member],
}
#[derive(Serialize)]
struct FormatReport {
    schema: &'static str,
    source_root: PathBuf,
    zstd_version: String,
    controls: Controls,
    traces: Vec<TraceMeasurement>,
    snapshots: Vec<SnapshotMeasurement>,
}
fn format_study(args: &[String]) -> Result<()> {
    let options =
        Options::parse(args, &["--only-traces"], &["--capture-root", "--output", "--trace-mib", "--snapshot-limit"])?;
    options.no_positionals()?;
    let root = capture_root(Path::new(options.one("--capture-root")?))?;
    let output = safe_output(Path::new(options.one("--output")?), std::slice::from_ref(&root))?;
    let trace_size = options.number("--trace-mib", 4, 1, 64)? * MIB;
    let limit = options.number("--snapshot-limit", 1000, 1, 2000)?;
    let mut report = FormatReport {
        schema: "storage-format-study-v2",
        source_root: root.clone(),
        zstd_version: zstd::zstd_safe::version_string().into(),
        controls: controls()?,
        traces: Vec::new(),
        snapshots: Vec::new(),
    };
    if root.join("hardware.log").exists() {
        for (numerator, denominator) in [(0, 1), (1, 2), (9, 10)] {
            report.traces.push(trace_measure(&root.join("hardware.log"), numerator, denominator, trace_size)?);
        }
    }
    if !options.flag("--only-traces")? {
        for (label, paths) in snapshot_sets(&root, true)? {
            let selected = &paths[..paths.len().min(if label == "rom-views" { limit.min(32) } else { limit })];
            if !selected.is_empty() {
                report.snapshots.push(snapshots_measure(&root, selected, label, paths.len())?);
            }
        }
    }
    ensure!(!report.traces.is_empty() || !report.snapshots.is_empty(), "capture contains no supported study artifacts");
    save(&output, &report)?;
    println!("Storage format measurements: {}", output.display());
    Ok(())
}

#[derive(Default)]
struct Merkle {
    objects: BTreeMap<Id, Vec<u8>>,
    retained_bytes: usize,
}
impl Merkle {
    fn put(&mut self, payload: Vec<u8>) -> Result<Id> {
        let id: Id = Sha256::digest(&payload).into();
        if let Some(previous) = self.objects.get(&id) {
            ensure!(*previous == payload, "Merkle hash collision");
        } else {
            self.retained_bytes =
                self.retained_bytes.checked_add(payload.len() + 32).context("Merkle size overflow")?;
            ensure!(self.retained_bytes <= MEMORY_BUDGET, "Merkle objects exceed 512 MiB study budget");
            self.objects.insert(id, payload);
        }
        Ok(id)
    }
    fn store(&mut self, data: &[u8]) -> Result<Id> {
        ensure!(!data.is_empty(), "empty snapshot unsupported in Merkle experiment");
        let mut layer = Vec::new();
        for chunk in data.chunks(256) {
            let mut page = vec![b'P'];
            page.extend_from_slice(&(chunk.len() as u16).to_le_bytes());
            page.extend_from_slice(chunk);
            layer.push(self.put(page)?);
        }
        while layer.len() > 1 {
            let mut next = Vec::new();
            for chunk in layer.chunks(16) {
                let mut node = vec![b'N'];
                node.extend_from_slice(&(chunk.len() as u16).to_le_bytes());
                for id in chunk {
                    node.extend_from_slice(id);
                }
                next.push(self.put(node)?);
            }
            layer = next;
        }
        let mut root = vec![b'M'];
        root.extend_from_slice(&(data.len() as u64).to_le_bytes());
        root.extend_from_slice(&layer[0]);
        self.put(root)
    }
    fn restore(&self, id: &Id) -> Result<Vec<u8>> {
        self.restore_at(id, 0)
    }
    fn restore_at(&self, id: &Id, depth: usize) -> Result<Vec<u8>> {
        ensure!(depth <= 64, "Merkle depth exceeds bound");
        let payload = self.objects.get(id).context("missing Merkle object")?;
        ensure!(<[u8; 32]>::from(Sha256::digest(payload)) == *id, "Merkle object hash differs");
        match payload.first() {
            Some(b'P') => {
                let length = u16::from_le_bytes(payload.get(1..3).context("truncated page")?.try_into()?) as usize;
                ensure!(length <= 256 && payload.len() == length + 3, "Merkle page length differs");
                Ok(payload[3..].to_vec())
            }
            Some(b'M') => {
                ensure!(payload.len() == 41, "Merkle root length differs");
                let length = usize::try_from(u64::from_le_bytes(payload[1..9].try_into()?))?;
                ensure!(length <= MEMORY_BUDGET, "Merkle snapshot exceeds bound");
                let data = self.restore_at(&payload[9..41].try_into()?, depth + 1)?;
                ensure!(data.len() == length, "Merkle snapshot length differs");
                Ok(data)
            }
            Some(b'N') => {
                let count = u16::from_le_bytes(payload.get(1..3).context("truncated node")?.try_into()?) as usize;
                ensure!((1..=16).contains(&count) && payload.len() == 3 + count * 32, "Merkle node length differs");
                let mut result = Vec::new();
                for id in payload[3..].as_chunks::<32>().0 {
                    let child = self.restore_at(id, depth + 1)?;
                    ensure!(result.len() + child.len() <= MEMORY_BUDGET, "Merkle restore exceeds bound");
                    result.extend_from_slice(&child);
                }
                Ok(result)
            }
            _ => bail!("unknown Merkle object tag"),
        }
    }
    fn stats(&self) -> Result<MerkleStats> {
        let mut pack = Vec::new();
        for (id, payload) in &self.objects {
            pack.extend_from_slice(id);
            pack.extend_from_slice(&u32::try_from(payload.len())?.to_le_bytes());
            pack.extend_from_slice(payload);
        }
        let packed = compressed(&pack, 3)?;
        ensure!(restored(&packed, pack.len())? == pack, "Merkle object pack round trip differs");
        Ok(MerkleStats {
            objects: self.objects.len(),
            pages: self.objects.values().filter(|p| p.first() == Some(&b'P')).count(),
            nodes: self.objects.values().filter(|p| p.first() == Some(&b'N')).count(),
            snapshot_roots: self.objects.values().filter(|p| p.first() == Some(&b'M')).count(),
            object_payload_bytes: self.objects.values().map(Vec::len).sum(),
            pack_bytes_including_object_ids_lengths: packed.len(),
            single_pack_uncompressed_bytes: pack.len(),
        })
    }
}
#[derive(Serialize)]
struct MerkleStats {
    objects: usize,
    pages: usize,
    nodes: usize,
    snapshot_roots: usize,
    object_payload_bytes: usize,
    pack_bytes_including_object_ids_lengths: usize,
    single_pack_uncompressed_bytes: usize,
}
#[derive(Serialize)]
struct MerkleDataset {
    run: PathBuf,
    kind: String,
    files: usize,
    logical_bytes: u64,
    new_objects: usize,
    new_pack_bytes: i64,
    cumulative: MerkleStats,
}
#[derive(Serialize)]
struct FlatStats {
    unique_pages: usize,
    raw_bytes: usize,
    zstd_bytes: usize,
}
#[derive(Serialize)]
struct MerkleReport {
    schema: &'static str,
    page_bytes: usize,
    fanout: usize,
    zstd_version: String,
    compression_level: i32,
    scope: &'static str,
    datasets: Vec<MerkleDataset>,
    roots_sha256: String,
    totals: MerkleStats,
    global_flat_pages: FlatStats,
    round_trip: bool,
}
fn merkle_study(args: &[String]) -> Result<()> {
    let options = Options::parse(args, &[], &["--capture-root", "--output"])?;
    options.no_positionals()?;
    let roots = options.roots()?;
    let output = safe_output(Path::new(options.one("--output")?), &roots)?;
    let mut merkle = Merkle::default();
    let mut page_ids: BTreeMap<Vec<u8>, u32> = BTreeMap::new();
    let mut pages = Vec::new();
    let mut flat_tables = Vec::new();
    let mut identities = Sha256::new();
    identities.update(b"storage-merkle-roots-v2\0");
    let mut datasets = Vec::new();
    for root in &roots {
        for (label, paths) in snapshot_sets(root, false)? {
            let before = merkle.stats()?;
            let mut raw = 0u64;
            for path in &paths {
                let size = usize::try_from(fs::symlink_metadata(path)?.len())?;
                ensure!(size <= MEMORY_BUDGET / 4, "snapshot exceeds 128 MiB study bound");
                let data = fs::read(path)?;
                let id = merkle.store(&data)?;
                ensure!(merkle.restore(&id)? == data, "Merkle snapshot round trip differs");
                let mut ids = Vec::new();
                for chunk in data.chunks(256) {
                    let mut page = vec![0; 256];
                    page[..chunk.len()].copy_from_slice(chunk);
                    let id = if let Some(id) = page_ids.get(&page) {
                        *id
                    } else {
                        ensure!(
                            pages.len() + flat_tables.len() + 256 <= MEMORY_BUDGET,
                            "flat page study exceeds 512 MiB"
                        );
                        let id = u32::try_from(page_ids.len())?;
                        page_ids.insert(page.clone(), id);
                        pages.extend_from_slice(&page);
                        id
                    };
                    ids.push(id);
                }
                let mut restored_member = Vec::new();
                for id in &ids {
                    restored_member.extend_from_slice(&pages[*id as usize * 256..(*id as usize + 1) * 256]);
                }
                restored_member.truncate(data.len());
                ensure!(restored_member == data, "flat page snapshot round trip differs");
                ensure!(
                    pages.len() + flat_tables.len() + 12 + ids.len() * 4 <= MEMORY_BUDGET,
                    "flat page tables exceed 512 MiB"
                );
                flat_tables.extend_from_slice(&(data.len() as u64).to_le_bytes());
                flat_tables.extend_from_slice(&u32::try_from(ids.len())?.to_le_bytes());
                for id in ids {
                    flat_tables.extend_from_slice(&id.to_le_bytes());
                }
                let name = path.to_str().context("non-UTF-8 source path")?.as_bytes();
                identities.update((name.len() as u64).to_le_bytes());
                identities.update(name);
                identities.update(id);
                identities.update(Sha256::digest(&data));
                ensure!(file_sha(path)? == sha(&data), "source snapshot changed during experiment");
                raw = raw.checked_add(data.len() as u64).context("logical size overflow")?;
            }
            let after = merkle.stats()?;
            let new_pack_bytes = i64::try_from(after.pack_bytes_including_object_ids_lengths)?
                - i64::try_from(before.pack_bytes_including_object_ids_lengths)?;
            datasets.push(MerkleDataset {
                run: root.clone(),
                kind: label.into(),
                files: paths.len(),
                logical_bytes: raw,
                new_objects: after.objects - before.objects,
                new_pack_bytes,
                cumulative: after,
            });
        }
    }
    ensure!(datasets.iter().any(|d| d.files != 0), "captures contain no supported snapshots");
    pages.extend_from_slice(&flat_tables);
    let flat = compressed(&pages, 3)?;
    ensure!(restored(&flat, pages.len())? == pages, "flat page pack round trip differs");
    let report = MerkleReport {
        schema: "storage-merkle-estimate-v3",
        page_bytes: 256,
        fanout: 16,
        zstd_version: zstd::zstd_safe::version_string().into(),
        compression_level: 3,
        scope: "all WRAM/full frames, no traces; optimistic single object pack; excludes physical seek index/run catalog/allocated filesystem bytes",
        datasets,
        roots_sha256: evidence::hex(&identities.finalize().into()),
        totals: merkle.stats()?,
        global_flat_pages: FlatStats { unique_pages: page_ids.len(), raw_bytes: pages.len(), zstd_bytes: flat.len() },
        round_trip: true,
    };
    save(&output, &report)?;
    println!("Merkle measurements: {}", output.display());
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    schema: String,
    complete: bool,
    observations: String,
    source: PathBuf,
    producer_sha256: String,
    source_inventory_sha256: String,
    bytes: u64,
    members: Vec<Member>,
}
fn receipt_id(receipt: &Receipt) -> Result<String> {
    let mut hash = Sha256::new();
    hash.update(b"storage-study-run-v2\0");
    hash.update(toml::to_string(receipt)?.as_bytes());
    Ok(evidence::hex(&hash.finalize().into()))
}
fn load_receipt(root: &Path, run: &str) -> Result<Receipt> {
    evidence::parse_id(run)?;
    let path = root.join("study-runs").join(format!("{run}.toml"));
    ensure!(
        fs::symlink_metadata(&path)?.is_file() && fs::metadata(&path)?.len() <= 16 * MIB as u64,
        "invalid study receipt"
    );
    let receipt: Receipt = toml::from_str(&fs::read_to_string(path)?)?;
    ensure!(
        receipt.schema == RECEIPT_SCHEMA && receipt.complete && receipt_id(&receipt)? == run,
        "study run seal/status differs"
    );
    ensure!(inventory_hash(&receipt.members)? == receipt.source_inventory_sha256, "study inventory seal differs");
    let mut names = BTreeSet::new();
    let mut bytes = 0u64;
    for member in &receipt.members {
        valid_member(&member.name)?;
        ensure!(names.insert(&member.name), "duplicate study member");
        evidence::parse_id(&member.sha256)?;
        bytes = bytes.checked_add(member.bytes).context("study byte count overflow")?;
    }
    ensure!(bytes == receipt.bytes && !receipt.members.is_empty(), "study receipt counts differ");
    evidence::parse_id(&receipt.observations)?;
    evidence::parse_id(&receipt.producer_sha256)?;
    Ok(receipt)
}
fn verify_receipt(store: &Store, root: &Path, run: &str) -> Result<Receipt> {
    let receipt = load_receipt(root, run)?;
    let inventory = evidence::tree_inventory(store, &evidence::parse_id(&receipt.observations)?)?;
    ensure!(
        inventory.len() == receipt.members.len()
            && receipt.members.iter().all(|m| inventory
                .get(&m.name)
                .is_some_and(|actual| actual.bytes == m.bytes && evidence::hex(&actual.sha256) == m.sha256)),
        "stored study inventory differs"
    );
    Ok(receipt)
}
fn valid_member(name: &str) -> Result<()> {
    ensure!(
        !name.is_empty()
            && name.len() <= 65535
            && !name.contains(['\\', '\0'])
            && name.split('/').all(|p| !p.is_empty() && p != "." && p != ".."),
        "unsafe study member path"
    );
    Ok(())
}
fn find_entry(store: &Store, mut tree: Id, name: &str) -> Result<Entry> {
    valid_member(name)?;
    let names: Vec<_> = name.split('/').collect();
    for (index, name) in names.iter().enumerate() {
        let entry =
            evidence::read_tree(store, &tree)?.into_iter().find(|e| e.name == *name).context("study member missing")?;
        if index + 1 == names.len() {
            return Ok(entry);
        }
        ensure!(entry.kind == EntryKind::Tree, "study path component is not directory");
        tree = entry.id;
    }
    bail!("study member missing")
}
#[derive(Default)]
struct HashSink {
    hash: Sha256,
    bytes: u64,
}
impl Write for HashSink {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.hash.update(bytes);
        self.bytes =
            self.bytes.checked_add(bytes.len() as u64).ok_or_else(|| std::io::Error::other("byte count overflow"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn cold_read(args: &[String]) -> Result<()> {
    let options = Options::parse(args, &[], &["--store", "--run", "--member", "--frame"])?;
    options.no_positionals()?;
    let root = Path::new(options.one("--store")?);
    let receipt = load_receipt(root, options.one("--run")?)?;
    let store = Store::open(root)?;
    let name = options.one("--member")?;
    let member = receipt.members.iter().find(|m| m.name == name).context("member absent from receipt")?;
    let entry = find_entry(&store, evidence::parse_id(&receipt.observations)?, name)?;
    let mut sink = HashSink::default();
    if let Some(frame) = options.optional("--frame")? {
        ensure!(entry.kind == EntryKind::Stream, "frame member is not a stream");
        evidence::write_frame(&store, &entry.id, frame.parse()?, &mut sink)?;
    } else {
        evidence::write_entry(&store, entry.kind, &entry.id, &mut sink)?;
        ensure!(
            sink.bytes == member.bytes && evidence::hex(&sink.hash.finalize().into()) == member.sha256,
            "cold member differs"
        );
    }
    Ok(())
}
#[derive(Serialize)]
struct DiskStats {
    bytes: u64,
    allocated_bytes: u64,
    files: usize,
    by_file: BTreeMap<String, u64>,
}
fn disk(root: &Path) -> Result<DiskStats> {
    fn walk(root: &Path, path: &Path, result: &mut DiskStats, inodes: &mut BTreeSet<(u64, u64)>) -> Result<()> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            ensure!(!entry.file_type()?.is_symlink(), "study output contains symlink");
            if metadata.is_dir() {
                walk(root, &entry.path(), result, inodes)?;
            } else {
                ensure!(metadata.is_file(), "study output contains special file");
                result.files += 1;
                result.bytes = result.bytes.checked_add(metadata.len()).context("disk byte count overflow")?;
                if inodes.insert((metadata.dev(), metadata.ino())) {
                    result.allocated_bytes = result
                        .allocated_bytes
                        .checked_add(metadata.blocks().checked_mul(512).context("allocated size overflow")?)
                        .context("allocated size overflow")?;
                }
                result.by_file.insert(
                    entry.path().strip_prefix(root)?.to_str().context("non-UTF-8 path")?.to_owned(),
                    metadata.len(),
                );
            }
        }
        Ok(())
    }
    let mut stats = DiskStats { bytes: 0, allocated_bytes: 0, files: 0, by_file: BTreeMap::new() };
    walk(root, root, &mut stats, &mut BTreeSet::new())?;
    Ok(stats)
}
struct Scope {
    root: PathBuf,
    members: Vec<PathBuf>,
    directories: Vec<PathBuf>,
}
impl Scope {
    fn create(root: &Path) -> Result<Self> {
        fs::create_dir(root)?;
        Ok(Self { root: root.to_path_buf(), members: Vec::new(), directories: Vec::new() })
    }
    #[cfg(test)]
    fn link(&mut self, source: &Path, dest: &Path) -> Result<()> {
        ensure!(fs::symlink_metadata(source)?.is_file(), "scope source is not regular file");
        fs::hard_link(source, dest)?;
        self.members.push(dest.to_path_buf());
        Ok(())
    }
    fn clean(&mut self) -> Result<()> {
        for path in self.members.drain(..).rev() {
            fs::remove_file(path)?;
        }
        for path in self.directories.drain(..).rev() {
            fs::remove_dir(path)?;
        }
        fs::remove_dir(&self.root)?;
        Ok(())
    }
}
impl Drop for Scope {
    fn drop(&mut self) {
        for path in self.members.drain(..).rev() {
            let _ = fs::remove_file(path);
        }
        for path in self.directories.drain(..).rev() {
            let _ = fs::remove_dir(path);
        }
        let _ = fs::remove_dir(&self.root);
    }
}
fn source_members(root: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = vec![root.join("hardware.log")];
    ensure!(fs::symlink_metadata(&paths[0])?.is_file(), "benchmark requires plain hardware.log");
    for (_, mut snapshots) in snapshot_sets(root, false)? {
        paths.append(&mut snapshots);
    }
    paths.sort();
    ensure!(paths.len() <= 100_000, "study scope exceeds 100,000 members");
    Ok(paths)
}
fn ingest_scope(store: &mut Store, root: &Path, paths: &[PathBuf]) -> Result<Id> {
    let mut entries = Vec::new();
    let mut fixtures = Vec::new();
    for path in paths {
        ensure!(fs::symlink_metadata(path)?.is_file(), "scope source is not regular file");
        let name = path
            .file_name()
            .context("scope source lacks filename")?
            .to_str()
            .context("non-UTF-8 scope name")?
            .to_owned();
        let (kind, id) = if name == "hardware.log" {
            (EntryKind::Stream, evidence::ingest_trace(store, &mut BufReader::with_capacity(MIB, File::open(path)?))?)
        } else {
            ensure!(fs::metadata(path)?.len() <= 128 * MIB as u64, "scope snapshot exceeds 128 MiB study bound");
            (EntryKind::Image, evidence::put_image(store, &fs::read(path)?)?)
        };
        let entry = Entry { name, kind, id };
        if path.parent() == Some(root.join("fixtures").as_path()) {
            fixtures.push(entry);
        } else {
            entries.push(entry);
        }
    }
    if !fixtures.is_empty() {
        entries.push(Entry {
            name: "fixtures".into(),
            kind: EntryKind::Tree,
            id: evidence::put_tree(store, fixtures)?,
        });
    }
    evidence::put_tree(store, entries)
}
fn source_allocated(paths: &[PathBuf]) -> Result<u64> {
    let mut seen = BTreeSet::new();
    let mut allocated = 0u64;
    for path in paths {
        let meta = fs::symlink_metadata(path)?;
        ensure!(meta.is_file(), "source is not regular file");
        if seen.insert((meta.dev(), meta.ino())) {
            allocated = allocated
                .checked_add(meta.blocks().checked_mul(512).context("allocated size overflow")?)
                .context("allocated size overflow")?;
        }
    }
    Ok(allocated)
}
#[derive(Serialize)]
struct ReadSample {
    member: String,
    frame: Option<u64>,
    cold_process_seconds: f64,
    plain_read_seconds: Option<f64>,
    logical_bytes: u64,
}
#[derive(Serialize)]
struct CaptureBenchmark {
    source: PathBuf,
    files: usize,
    logical_bytes: u64,
    source_allocated_bytes_unique_inodes: u64,
    run: String,
    observations: String,
    ingest_seconds: f64,
    verify_seconds: f64,
    pack_sha256: Option<String>,
    new_objects: u64,
    shared_objects: u64,
    store_added_bytes: u64,
    store_after: DiskStats,
    reads: Vec<ReadSample>,
    source_inventory_sha256: String,
}
#[derive(Serialize)]
struct WriterBenchmark {
    mode: String,
    repeat: usize,
    events: usize,
    loaded_events: usize,
    seconds: f64,
    bytes: u64,
    sink: u64,
    exact_readback: bool,
    cleanup_intermediates_bytes: u64,
}
#[derive(Serialize)]
struct StorageBenchmark {
    schema: &'static str,
    format: &'static str,
    comparison_scope: &'static str,
    replaced_prototype: &'static str,
    binary: PathBuf,
    binary_sha256: String,
    store: PathBuf,
    zstd_version: String,
    controls: Controls,
    native_writer: Vec<WriterBenchmark>,
    captures: Vec<CaptureBenchmark>,
    store_after: DiskStats,
    pack_sha256: BTreeMap<String, String>,
    cleanup_intermediates_bytes: u64,
    cleanup_scope_files: usize,
}
fn cold_sample(binary: &Path, root: &Path, run: &str, member: &str, frame: Option<u64>) -> Result<f64> {
    let mut command = Command::new(binary);
    command.args(["storage-study", "cold-read", "--store"]).arg(root).args(["--run", run, "--member", member]);
    if let Some(frame) = frame {
        command.arg("--frame").arg(frame.to_string());
    }
    command.stdout(std::process::Stdio::null());
    let start = Instant::now();
    let output = command.output()?;
    ensure!(output.status.success(), "cold read failed: {}", String::from_utf8_lossy(&output.stderr));
    Ok(start.elapsed().as_secs_f64())
}
fn benchmark(args: &[String], matched: bool) -> Result<()> {
    let options = Options::parse(
        args,
        &["--skip-writer"],
        &[
            "--output",
            "--capture-root",
            "--events",
            "--writer-repeats",
            "--binary",
            "--scope-root",
            "--store",
            "--report",
        ],
    )?;
    let roots = if matched {
        ensure!(
            !options.positionals.is_empty() && options.positionals.len() <= 64,
            "rust-benchmark requires 1..=64 source captures"
        );
        options.positionals.iter().map(|root| capture_root(Path::new(root))).collect::<Result<Vec<_>>>()?
    } else {
        options.no_positionals()?;
        options.roots()?
    };
    let (output, scope_root, store_root, report_path) = if matched {
        let scope = safe_output(Path::new(options.one("--scope-root")?), &roots)?;
        let store = safe_output(Path::new(options.one("--store")?), &roots)?;
        let report = safe_output(Path::new(options.one("--report")?), &roots)?;
        ensure!(
            !scope.starts_with(&store)
                && !store.starts_with(&scope)
                && !report.starts_with(&scope)
                && !report.starts_with(&store),
            "scope/store/report overlap"
        );
        (None, scope, store, report)
    } else {
        let output = safe_output(Path::new(options.one("--output")?), &roots)?;
        (Some(output.clone()), output.join("scopes"), output.join("store"), output.join("benchmark.toml"))
    };
    let binary = if let Some(path) = options.optional("--binary")? {
        Path::new(path).canonicalize()?
    } else {
        std::env::current_exe()?.canonicalize()?
    };
    let binary_sha256 = file_sha(&binary)?;
    let executable_image =
        File::open(if cfg!(target_os = "linux") { PathBuf::from("/proc/self/exe") } else { std::env::current_exe()? })?;
    // Cold process probes must run the same typed receipt reader as ingestion.
    ensure!(
        file_sha(&std::env::current_exe()?)? == binary_sha256,
        "--binary must identify this sm-pipeline executable; sm-evidence uses a different run contract"
    );
    let events = options.number("--events", 5_000_000, 1, 50_000_000)?;
    let repeats = options.number("--writer-repeats", 3, 1, 10)?;
    let controls = controls()?;
    if let Some(output) = &output {
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::create_dir(output)?;
    }
    if let Some(parent) = scope_root.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut scope = Scope::create(&scope_root)?;
    if let Some(parent) = store_root.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir(&store_root)?;
    // Cargo may replace its output while parallel workers are running. Retain
    // the already opened executable image used to produce and read this study.
    let retained_binary = report_path.with_extension("reader");
    if let Some(parent) = retained_binary.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut retained = OpenOptions::new().write(true).create_new(true).open(&retained_binary)?;
    std::io::copy(&mut &executable_image, &mut retained)?;
    use std::os::unix::fs::PermissionsExt;
    retained.set_permissions(fs::Permissions::from_mode(0o755))?;
    retained.sync_all()?;
    drop(retained);
    ensure!(file_sha(&retained_binary)? == binary_sha256, "retained study reader differs");
    let binary = retained_binary;
    let native_writer = if !matched && !options.flag("--skip-writer")? {
        writer_benchmarks(output.as_ref().context("writer benchmark output missing")?, &roots[0], events, repeats)?
    } else {
        Vec::new()
    };
    let mut captures = Vec::new();
    let cleanup_scope_files = 0;
    for root in &roots {
        let paths = source_members(root)?;
        let before = inventory(&paths, root)?;
        let allocated = source_allocated(&paths)?;
        let prior = disk(&store_root)?;
        let start = Instant::now();
        let mut store = Store::create(&store_root)?;
        let tree = ingest_scope(&mut store, root, &paths)?;
        let stats = store.stats();
        let pack = store.commit()?;
        let ingest_seconds = start.elapsed().as_secs_f64();
        ensure!(inventory(&paths, root)? == before, "source changed during ingestion");
        let logical_bytes =
            before.iter().try_fold(0u64, |sum, m| sum.checked_add(m.bytes).context("logical byte overflow"))?;
        let receipt = Receipt {
            schema: RECEIPT_SCHEMA.into(),
            complete: true,
            observations: evidence::hex(&tree),
            source: root.clone(),
            producer_sha256: binary_sha256.clone(),
            source_inventory_sha256: inventory_hash(&before)?,
            bytes: logical_bytes,
            members: before,
        };
        let run = receipt_id(&receipt)?;
        let start = Instant::now();
        let store = Store::open(&store_root)?;
        store.verify_packs()?;
        ensure!(
            evidence::verify_tree(&store, &tree, None)? == (paths.len() as u64, logical_bytes),
            "reopened tree counts differ"
        );
        let receipt_path = store_root.join("study-runs").join(format!("{run}.toml"));
        save(&receipt_path, &receipt)?;
        verify_receipt(&store, &store_root, &run)?;
        ensure!(inventory(&paths, root)? == receipt.members, "source changed during reopened verification");
        let verify_seconds = start.elapsed().as_secs_f64();
        drop(store);
        let after = disk(&store_root)?;
        let mut reads = Vec::new();
        let mut snapshot_samples: Vec<_> = receipt.members.iter().filter(|m| m.name.starts_with("frame-")).collect();
        snapshot_samples.sort_by_key(|m| Sha256::digest(format!("storage-study-sample-7:{}", m.name)).to_vec());
        for member in snapshot_samples.into_iter().take(10) {
            let seconds = cold_sample(&binary, &store_root, &run, &member.name, None)?;
            let start = Instant::now();
            ensure!(file_sha(&root.join(&member.name))? == member.sha256, "plain source changed during read benchmark");
            reads.push(ReadSample {
                member: member.name.clone(),
                frame: None,
                cold_process_seconds: seconds,
                plain_read_seconds: Some(start.elapsed().as_secs_f64()),
                logical_bytes: member.bytes,
            });
        }
        let mut frames = BTreeSet::from([60u64]);
        // Bound discovery independently of total trace size. Frame read samples
        // come from beginning/middle/end windows, preserving reset labels.
        for (numerator, denominator) in [(0, 1), (1, 2), (9, 10)] {
            let (_, sample) = trace_sample(&root.join("hardware.log"), numerator, denominator, MIB)?;
            for line in sample.split_inclusive(|b| *b == b'\n') {
                if let Ok(event) = DecimalEvent::parse(line)
                    && let Ok(frame) = u64::try_from(event.frame)
                {
                    frames.insert(frame);
                }
            }
        }
        let mut frames: Vec<_> = frames.into_iter().collect();
        frames.sort_by_key(|frame| Sha256::digest(format!("storage-study-frame-7:{frame}")).to_vec());
        let read_store = Store::open(&store_root)?;
        let stream = find_entry(&read_store, tree, "hardware.log")?;
        for frame in frames.into_iter().take(10) {
            let frame_seconds = cold_sample(&binary, &store_root, &run, "hardware.log", Some(frame))?;
            let mut output = HashSink::default();
            evidence::write_frame(&read_store, &stream.id, frame, &mut output)?;
            reads.push(ReadSample {
                member: "hardware.log".into(),
                frame: Some(frame),
                cold_process_seconds: frame_seconds,
                plain_read_seconds: None,
                logical_bytes: output.bytes,
            });
        }
        captures.push(CaptureBenchmark {
            source: root.clone(),
            files: paths.len(),
            logical_bytes,
            source_allocated_bytes_unique_inodes: allocated,
            run,
            observations: receipt.observations,
            ingest_seconds,
            verify_seconds,
            pack_sha256: pack.map(|id| evidence::hex(&id)),
            new_objects: stats.new_objects,
            shared_objects: stats.shared_objects,
            store_added_bytes: after.bytes.checked_sub(prior.bytes).context("store shrank during measurement")?,
            store_after: after,
            reads,
            source_inventory_sha256: receipt.source_inventory_sha256,
        });
    }
    scope.clean()?;
    let mut packs = BTreeMap::new();
    for path in matching(&store_root, "", ".pack")? {
        packs.insert(
            path.file_name().context("pack lacks filename")?.to_str().context("non-UTF-8 pack name")?.into(),
            file_sha(&path)?,
        );
    }
    let cleanup_intermediates_bytes = native_writer.iter().try_fold(0u64, |total, row| {
        total.checked_add(row.cleanup_intermediates_bytes).context("cleanup byte count overflow")
    })?;
    let report = StorageBenchmark {
        schema: "storage-benchmark-v2",
        format: "sm-evidence-v1 observations + sealed storage-study-run-v2 TOML receipts",
        comparison_scope: "hardware.log + fixtures/*.wram + frame-*.bin; exact matched scope; cold process timing includes pack-index loading and receipt authentication; snapshot samples use seed 7; trace frame samples use bounded beginning/middle/end windows and seed 7; OS page cache is not cleared; retained reader bytes are excluded from store size",
        replaced_prototype: "historical SQLite storage-spike-v1 is not read or rewritten; binary production evidence implements native benchmark/verify/extract",
        binary,
        binary_sha256,
        store: store_root.clone(),
        zstd_version: zstd::zstd_safe::version_string().into(),
        controls,
        native_writer,
        captures,
        store_after: disk(&store_root)?,
        pack_sha256: packs,
        cleanup_intermediates_bytes,
        cleanup_scope_files,
    };
    save(&report_path, &report)?;
    println!("Storage benchmark: {}", report_path.display());
    Ok(())
}

struct TemporaryFile {
    path: PathBuf,
    owned: bool,
}
impl TemporaryFile {
    fn create(path: PathBuf) -> Result<(Self, File)> {
        let file = OpenOptions::new().write(true).create_new(true).open(&path)?;
        Ok((Self { path, owned: true }, file))
    }
    fn remove(&mut self) -> Result<u64> {
        let bytes = fs::metadata(&self.path)?.len();
        fs::remove_file(&self.path)?;
        self.owned = false;
        Ok(bytes)
    }
}
impl Drop for TemporaryFile {
    fn drop(&mut self) {
        if self.owned {
            let _ = fs::remove_file(&self.path);
        }
    }
}
struct TemporaryDirectory {
    path: PathBuf,
    owned: bool,
}
impl TemporaryDirectory {
    fn create(path: PathBuf) -> Result<Self> {
        fs::create_dir(&path)?;
        Ok(Self { path, owned: true })
    }
    fn remove(&mut self) -> Result<u64> {
        let bytes = disk(&self.path)?.bytes;
        fs::remove_dir_all(&self.path)?;
        self.owned = false;
        Ok(bytes)
    }
}
impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        if self.owned {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
fn writer_benchmarks(output: &Path, source: &Path, count: usize, repeats: usize) -> Result<Vec<WriterBenchmark>> {
    let (_, text) = trace_sample(&source.join("hardware.log"), 1, 2, 16 * MIB)?;
    let events =
        text.split_inclusive(|b| *b == b'\n').take(1_000_000).map(DecimalEvent::parse).collect::<Result<Vec<_>>>()?;
    ensure!(!events.is_empty(), "writer benchmark sample has no events");
    let native = events.iter().map(DecimalEvent::native).collect::<Result<Vec<_>>>()?;
    let mut expected = HashSink::default();
    for event in events.iter().cycle().take(count) {
        event.render(&mut expected)?;
    }
    let expected_sha = evidence::hex(&expected.hash.finalize().into());
    let expected_bytes = expected.bytes;
    let mut report = Vec::new();
    for repeat in 0..repeats {
        for mode in ["null", "plain", "gzip", "zstd-text", "binary"] {
            let path = output.join(format!("writer-{mode}-{repeat}"));
            let mut checksum = 0u64;
            let (seconds, bytes) = if mode == "binary" {
                let mut temporary = TemporaryDirectory::create(path.clone())?;
                let mut store = Store::create(&path)?;
                let start = Instant::now();
                let mut writer = evidence::TraceWriter::new(&mut store);
                for event in native.iter().cycle().take(count) {
                    writer.push(event.clone())?;
                }
                let stream = writer.finish()?;
                store.commit()?;
                let seconds = start.elapsed().as_secs_f64();
                let bytes = disk(&path)?.bytes;
                let store = Store::open(&path)?;
                store.verify_packs()?;
                let mut actual = HashSink::default();
                evidence::write_stream(&store, &stream, &mut actual)?;
                ensure!(
                    actual.bytes == expected_bytes && evidence::hex(&actual.hash.finalize().into()) == expected_sha,
                    "binary writer readback differs"
                );
                drop(store);
                temporary.remove()?;
                (seconds, bytes)
            } else {
                let (mut temporary, file) = TemporaryFile::create(path.clone())?;
                let start = Instant::now();
                match mode {
                    "null" => {
                        for event in events.iter().cycle().take(count) {
                            let value = match event.payload {
                                EventPayload::Ppu(_, _, v) | EventPayload::Apu(_, v) => v,
                            };
                            checksum = checksum.wrapping_add(std::hint::black_box(value as u64));
                        }
                        file.sync_all()?;
                    }
                    "plain" => {
                        let mut writer = BufWriter::new(file);
                        for event in events.iter().cycle().take(count) {
                            event.render(&mut writer)?;
                        }
                        writer.flush()?;
                        writer.get_ref().sync_all()?;
                    }
                    "gzip" => {
                        let mut writer =
                            flate2::GzBuilder::new().mtime(0).write(BufWriter::new(file), flate2::Compression::fast());
                        for event in events.iter().cycle().take(count) {
                            event.render(&mut writer)?;
                        }
                        let mut writer = writer.finish()?;
                        writer.flush()?;
                        writer.get_ref().sync_all()?;
                    }
                    "zstd-text" => {
                        let mut writer = zstd::stream::write::Encoder::new(BufWriter::new(file), 3)?;
                        for event in events.iter().cycle().take(count) {
                            event.render(&mut writer)?;
                        }
                        let mut writer = writer.finish()?;
                        writer.flush()?;
                        writer.get_ref().sync_all()?;
                    }
                    _ => unreachable!(),
                }
                let seconds = start.elapsed().as_secs_f64();
                let bytes = fs::metadata(&path)?.len();
                if mode != "null" {
                    let file = File::open(&path)?;
                    let mut reader: Box<dyn Read> = match mode {
                        "gzip" => Box::new(flate2::read::GzDecoder::new(file)),
                        "zstd-text" => Box::new(zstd::stream::read::Decoder::new(file)?),
                        _ => Box::new(file),
                    };
                    let mut actual = HashSink::default();
                    std::io::copy(&mut reader, &mut actual)?;
                    ensure!(
                        actual.bytes == expected_bytes && evidence::hex(&actual.hash.finalize().into()) == expected_sha,
                        "writer readback differs: {mode}"
                    );
                }
                temporary.remove()?;
                (seconds, bytes)
            };
            report.push(WriterBenchmark {
                mode: mode.into(),
                repeat,
                events: count,
                loaded_events: events.len(),
                seconds,
                bytes,
                sink: checksum,
                exact_readback: mode != "null",
                cleanup_intermediates_bytes: bytes,
            });
        }
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    fn temporary() -> TemporaryDirectory {
        TemporaryDirectory::create(std::env::temp_dir().join(format!(
            "storage-study-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )))
        .unwrap()
    }
    fn fixture(root: &Path) -> PathBuf {
        let capture = root.join("capture");
        fs::create_dir(&capture).unwrap();
        fs::create_dir(capture.join("fixtures")).unwrap();
        fs::write(
            capture.join("hardware.log"),
            b"ppu 0 1 4 7\nppu 0 1 4 8\nppu 0 1 4 8\napu 60 16777215 255\nppu 0 0 0 0\n",
        )
        .unwrap();
        let mut state: Vec<_> = (0..600).map(|i| (i % 256) as u8).collect();
        fs::write(capture.join("fixtures/001.wram"), &state).unwrap();
        state[20] ^= 1;
        fs::write(capture.join("fixtures/002.wram"), &state).unwrap();
        fs::write(capture.join("frame-000001.bin"), &state).unwrap();
        capture
    }
    #[test]
    fn event_and_codec_adversarial_controls() -> Result<()> {
        controls()?;
        for bad in [
            &b"ppu 0 1 4 7"[..],
            &b"ppu 0 256 4 7\n"[..],
            &b"apu 0 4294967296 0\n"[..],
            &b"ppu 0  1 4 7\n"[..],
            &b"ppu 0 01 4 7\n"[..],
        ] {
            if let Ok((encoded, _)) = encode_events(bad) {
                ensure!(decode_events(&encoded)? != bad, "noncanonical record was accepted as exact")
            }
        }
        for bad in [vec![0], vec![0, 0], vec![0, 1, 0, 0, 0], vec![0, 5], vec![255; 10]] {
            ensure!(decode_events(&bad).is_err(), "malformed event was accepted");
        }
        let edges = b"ppu -9223372036854775808 1 4 7\nppu -9223372036854775808 1 4 8\n";
        ensure!(decode_events(&encode_events(edges)?.0)? == edges, "signed label boundary round trip differs");
        ensure!(
            encode_events(b"ppu -9223372036854775808 0 0 0\nppu 9223372036854775807 0 0 0\n").is_err(),
            "delta overflow accepted"
        );
        Ok(())
    }
    #[test]
    fn native_format_and_merkle_preserve_sources_and_sampling() -> Result<()> {
        let mut tmp = temporary();
        let capture = fixture(&tmp.path);
        let paths = source_members(&capture)?;
        let before = inventory(&paths, &capture)?;
        let format = tmp.path.join("format.toml");
        format_study(&[
            "--capture-root".into(),
            capture.display().to_string(),
            "--output".into(),
            format.display().to_string(),
            "--trace-mib".into(),
            "1".into(),
            "--snapshot-limit".into(),
            "2".into(),
        ])?;
        let report: toml::Value = toml::from_str(&fs::read_to_string(&format)?)?;
        ensure!(report["traces"].as_array().context("traces")?.len() == 3, "trace sampling changed");
        ensure!(
            report["snapshots"][0]["candidates"].as_array().context("candidates")?.len() == 8,
            "snapshot candidate disappeared"
        );
        let merkle = tmp.path.join("merkle.toml");
        merkle_study(&[
            "--capture-root".into(),
            capture.display().to_string(),
            "--output".into(),
            merkle.display().to_string(),
        ])?;
        let report: toml::Value = toml::from_str(&fs::read_to_string(merkle)?)?;
        ensure!(report["totals"]["pages"].as_integer().context("page count")? > 0, "Merkle pages absent");
        ensure!(inventory(&paths, &capture)? == before, "read-only study mutated sources");
        let actual = writer_benchmarks(&tmp.path, &capture, 200, 1)?;
        ensure!(
            actual.len() == 5 && actual.iter().filter(|r| r.exact_readback).count() == 4,
            "writer candidate disappeared"
        );
        ensure!(
            fs::read_dir(&tmp.path)?.all(|e| !e.unwrap().file_name().to_string_lossy().starts_with("writer-")),
            "writer intermediates remain"
        );
        tmp.remove()?;
        Ok(())
    }
    #[test]
    fn merkle_missing_corrupt_partial_pages_and_shared_roots() -> Result<()> {
        let mut merkle = Merkle::default();
        let data = (0..10_013).map(|i| (i % 251) as u8).collect::<Vec<_>>();
        let root = merkle.store(&data)?;
        let objects = merkle.objects.len();
        ensure!(merkle.store(&data)? == root && merkle.objects.len() == objects, "global sharing differs");
        ensure!(merkle.restore(&root)? == data, "partial-page restore differs");
        let page = *merkle.objects.iter().find(|(_, p)| p[0] == b'P').context("page")?.0;
        let old = merkle.objects.remove(&page).context("page")?;
        ensure!(merkle.restore(&root).is_err(), "missing dependency accepted");
        let mut changed = old;
        changed[3] ^= 1;
        merkle.objects.insert(page, changed);
        ensure!(merkle.restore(&root).is_err(), "corrupt object accepted");
        ensure!(merkle.store(&[]).is_err(), "unsupported empty snapshot accepted");
        Ok(())
    }
    #[test]
    fn receipts_authenticate_inventory_and_producer_independently() -> Result<()> {
        let mut tmp = temporary();
        let capture = fixture(&tmp.path);
        let store_root = tmp.path.join("store");
        let members = inventory(&source_members(&capture)?, &capture)?;
        let mut store = Store::create(&store_root)?;
        let tree = ingest_scope(&mut store, &capture, &source_members(&capture)?)?;
        store.commit()?;
        let receipt = Receipt {
            schema: RECEIPT_SCHEMA.into(),
            complete: true,
            observations: evidence::hex(&tree),
            source: capture,
            producer_sha256: "ab".repeat(32),
            source_inventory_sha256: inventory_hash(&members)?,
            bytes: members.iter().map(|m| m.bytes).sum(),
            members,
        };
        let run = receipt_id(&receipt)?;
        let receipt_path = store_root.join("study-runs").join(format!("{run}.toml"));
        save(&receipt_path, &receipt)?;
        let store = Store::open(&store_root)?;
        verify_receipt(&store, &store_root, &run)?;
        let mut changed = receipt.clone();
        changed.producer_sha256 = "cd".repeat(32);
        ensure!(
            changed.observations == receipt.observations && receipt_id(&changed)? != run,
            "producer provenance affected observation identity"
        );
        fs::write(&receipt_path, toml::to_string(&changed)?)?;
        ensure!(load_receipt(&store_root, &run).is_err(), "changed receipt accepted under old run ID");
        changed.members[0].sha256 = "ef".repeat(32);
        changed.source_inventory_sha256 = inventory_hash(&changed.members)?;
        let altered_run = receipt_id(&changed)?;
        save(&store_root.join("study-runs").join(format!("{altered_run}.toml")), &changed)?;
        ensure!(
            verify_receipt(&store, &store_root, &altered_run).is_err(),
            "sealed receipt with wrong underlying bytes accepted"
        );
        ensure!(find_entry(&store, tree, "../hardware.log").is_err(), "unsafe extraction path accepted");
        drop(store);
        tmp.remove()?;
        Ok(())
    }
    #[test]
    fn output_containment_and_scope_failure_cleanup() -> Result<()> {
        let mut tmp = temporary();
        let capture = fixture(&tmp.path);
        ensure!(
            safe_output(&capture.join("new/report.toml"), std::slice::from_ref(&capture)).is_err(),
            "source-contained output allowed"
        );
        std::os::unix::fs::symlink(&capture, tmp.path.join("alias"))?;
        ensure!(
            safe_output(&tmp.path.join("alias/new/report.toml"), std::slice::from_ref(&capture)).is_err(),
            "symlink source containment escaped"
        );
        let scope_path = tmp.path.join("scope");
        {
            let mut scope = Scope::create(&scope_path)?;
            scope.link(&capture.join("hardware.log"), &scope_path.join("hardware.log"))?;
            ensure!(
                scope.link(&capture.join("missing"), &scope_path.join("missing")).is_err(),
                "missing scope source accepted"
            );
        }
        ensure!(!scope_path.exists() && capture.join("hardware.log").exists(), "failed scope cleanup damaged source");
        fs::remove_file(tmp.path.join("alias"))?;
        tmp.remove()?;
        Ok(())
    }
}
