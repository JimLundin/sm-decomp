//! Throwaway format study. No production reader or acceptance is changed.
#![allow(dead_code)]
#[path = "original_family_model.rs"]
mod family_model;

use anyhow::{Context, Result, ensure};
use family_model::{Case, Manifest, Write};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs,
    hint::black_box,
    os::fd::AsRawFd,
    path::{Path, PathBuf},
    time::Instant,
};

const WRAM: usize = 0x20000;
const HEADER: usize = 64;
const MAGIC: &[u8; 8] = b"SMFXST01";

#[derive(Clone, Copy, Debug, Serialize)]
enum Layout {
    IndexedWrites,
    CompiledSpans,
    RootImages,
    MergedSpans,
}
impl Layout {
    fn name(self) -> &'static str {
        match self {
            Self::IndexedWrites => "indexed-writes",
            Self::CompiledSpans => "compiled-spans",
            Self::RootImages => "root-images",
            Self::MergedSpans => "merged-spans",
        }
    }
}
fn cases(m: &Manifest) -> Vec<&Case> {
    m.cases.iter().chain(m.routines.iter().flat_map(|r| &r.cases)).collect()
}
fn without_writes(m: &Manifest) -> Manifest {
    let mut m = m.clone();
    for c in m.cases.iter_mut().chain(m.routines.iter_mut().flat_map(|r| &mut r.cases)) {
        c.writes.clear();
    }
    m
}
fn u64_at(bytes: &[u8], at: usize) -> Result<usize> {
    let end = at.checked_add(8).context("integer overflow")?;
    Ok(usize::try_from(u64::from_le_bytes(bytes.get(at..end).context("truncated u64")?.try_into()?))?)
}
fn u32_at(bytes: &[u8], at: usize) -> Result<usize> {
    let end = at.checked_add(4).context("integer overflow")?;
    Ok(u32::from_le_bytes(bytes.get(at..end).context("truncated u32")?.try_into()?) as usize)
}
fn put64(bytes: &mut [u8], at: usize, v: usize) {
    bytes[at..at + 8].copy_from_slice(&(v as u64).to_le_bytes());
}
fn put32(bytes: &mut [u8], at: usize, v: usize) {
    bytes[at..at + 4].copy_from_slice(&u32::try_from(v).unwrap().to_le_bytes());
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn inherited(bytes: &mut [u8], seed: u8) {
    for (i, b) in bytes.iter_mut().enumerate() {
        *b = (i.wrapping_mul(37) as u8).wrapping_add(seed);
    }
}
fn initialize(bytes: &mut [u8], c: &Case, seed: u8) {
    if c.continue_from.is_some() {
        inherited(bytes, seed);
    } else {
        bytes.fill(c.fill.unwrap_or(0));
    }
}
fn ordinary(bytes: &mut [u8], c: &Case, seed: u8) {
    initialize(bytes, c, seed);
    for w in &c.writes {
        bytes[w.offset..w.offset + w.data.len()].copy_from_slice(&w.data);
    }
}
fn compile_writes(c: &Case, layout: Layout) -> Result<Vec<Write>> {
    ensure!(c.before_file.is_none(), "study only selects inline-state fixtures");
    ensure!(c.continue_from.is_none() || c.fill.is_none(), "invalid continuation fill");
    for w in &c.writes {
        ensure!(w.offset.checked_add(w.data.len()).is_some_and(|n| n <= WRAM), "write outside WRAM");
    }
    if matches!(layout, Layout::IndexedWrites) {
        return Ok(c.writes.clone());
    }
    let fill = c.fill.unwrap_or(0);
    let mut bytes = vec![fill; WRAM];
    let mut touched = vec![false; WRAM];
    for w in &c.writes {
        bytes[w.offset..w.offset + w.data.len()].copy_from_slice(&w.data);
        touched[w.offset..w.offset + w.data.len()].fill(true);
    }
    if matches!(layout, Layout::RootImages) && c.continue_from.is_none() {
        return Ok(vec![Write { offset: 0, data: bytes }]);
    }
    let keep = |i: usize| if c.continue_from.is_some() { touched[i] } else { bytes[i] != fill };
    // Root fill bytes are known and may bridge short holes; continuation holes belong to the predecessor.
    let maximum_gap = if matches!(layout, Layout::MergedSpans) && c.continue_from.is_none() { 16 } else { 0 };
    let mut out = Vec::new();
    let mut at = 0;
    while at < WRAM {
        if !keep(at) {
            at += 1;
            continue;
        }
        let start = at;
        let mut end = at + 1;
        at += 1;
        while at < WRAM {
            if keep(at) {
                end = at + 1;
            } else if at - end >= maximum_gap {
                break;
            }
            at += 1;
        }
        out.push(Write { offset: start, data: bytes[start..end].to_vec() });
    }
    Ok(out)
}

#[derive(Serialize)]
struct Encoding {
    layout: String,
    bytes: usize,
    metadata_bytes: usize,
    cases: usize,
    patch_records: usize,
    unique_payload_bytes: usize,
    compilation_ms: f64,
    encoded_sha256: String,
}
fn encode(m: &Manifest, layout: Layout) -> Result<(Vec<u8>, Encoding)> {
    let clock = Instant::now();
    let metadata = serde_json::to_vec(&without_writes(m))?;
    let input_cases = cases(m);
    let count = input_cases.len();
    let index = (HEADER + metadata.len() + 7) & !7;
    let records_start = index + count * 16;
    let mut records = Vec::<(usize, usize, usize)>::new();
    let mut indices = Vec::new();
    let mut blobs = Vec::new();
    let mut shared = HashMap::<Vec<u8>, usize>::new();
    for c in input_cases {
        let start = records.len();
        for w in compile_writes(c, layout)? {
            let at = if let Some(at) = shared.get(&w.data) {
                *at
            } else {
                let at = blobs.len();
                blobs.extend_from_slice(&w.data);
                shared.insert(w.data.clone(), at);
                at
            };
            records.push((w.offset, w.data.len(), at));
        }
        indices.push((start, records.len() - start));
    }
    let payload_start = records_start + records.len() * 16;
    let mut out = vec![0; payload_start];
    out[..8].copy_from_slice(MAGIC);
    put32(&mut out, 8, 1);
    put32(&mut out, 12, layout as usize);
    put64(&mut out, 16, HEADER);
    put64(&mut out, 24, metadata.len());
    put64(&mut out, 32, index);
    put64(&mut out, 40, count);
    put64(&mut out, 48, records_start);
    put64(&mut out, 56, payload_start);
    out[HEADER..HEADER + metadata.len()].copy_from_slice(&metadata);
    for (i, (start, count)) in indices.iter().enumerate() {
        put64(&mut out, index + i * 16, *start);
        put64(&mut out, index + i * 16 + 8, *count);
    }
    for (i, (offset, len, at)) in records.iter().enumerate() {
        put32(&mut out, records_start + i * 16, *offset);
        put32(&mut out, records_start + i * 16 + 4, *len);
        put64(&mut out, records_start + i * 16 + 8, payload_start + at);
    }
    out.extend_from_slice(&blobs);
    let evidence = Encoding {
        layout: layout.name().into(),
        bytes: out.len(),
        metadata_bytes: metadata.len(),
        cases: count,
        patch_records: records.len(),
        unique_payload_bytes: blobs.len(),
        compilation_ms: clock.elapsed().as_secs_f64() * 1000.0,
        encoded_sha256: hash(&out),
    };
    Ok((out, evidence))
}

struct Opened {
    bytes: Vec<u8>,
    metadata: Manifest,
    index: usize,
    records: usize,
    payload: usize,
    count: usize,
    layout: usize,
}
impl Opened {
    fn new(bytes: Vec<u8>) -> Result<Self> {
        ensure!(bytes.get(..8) == Some(MAGIC), "invalid magic");
        ensure!(u32_at(&bytes, 8)? == 1, "invalid format version");
        let layout = u32_at(&bytes, 12)?;
        ensure!(layout <= 3, "invalid layout");
        let meta = u64_at(&bytes, 16)?;
        let meta_len = u64_at(&bytes, 24)?;
        let index = u64_at(&bytes, 32)?;
        let count = u64_at(&bytes, 40)?;
        let records = u64_at(&bytes, 48)?;
        let payload = u64_at(&bytes, 56)?;
        ensure!(meta == HEADER && meta_len <= bytes.len().saturating_sub(meta), "metadata extent");
        ensure!(index == (meta + meta_len + 7) & !7, "index location");
        ensure!(count <= 1_000_000 && index.checked_add(count * 16) == Some(records), "case index extent");
        ensure!(records <= payload && payload <= bytes.len() && (payload - records) % 16 == 0, "record extent");
        let metadata: Manifest = serde_json::from_slice(&bytes[meta..meta + meta_len])?;
        ensure!(cases(&metadata).len() == count, "case coverage");
        let nrecords = (payload - records) / 16;
        for i in 0..count {
            let start = u64_at(&bytes, index + i * 16)?;
            let n = u64_at(&bytes, index + i * 16 + 8)?;
            ensure!(start.checked_add(n).is_some_and(|v| v <= nrecords), "case records extent");
        }
        for i in 0..nrecords {
            let at = records + i * 16;
            let offset = u32_at(&bytes, at)?;
            let len = u32_at(&bytes, at + 4)?;
            let blob = u64_at(&bytes, at + 8)?;
            ensure!(offset.checked_add(len).is_some_and(|v| v <= WRAM), "WRAM extent");
            ensure!(blob >= payload && blob.checked_add(len).is_some_and(|v| v <= bytes.len()), "payload extent");
        }
        Ok(Self { bytes, metadata, index, records, payload, count, layout })
    }
    fn apply(&self, number: usize, bytes: &mut [u8], c: &Case, seed: u8) {
        // Precompiled root images replace the complete input, so no prior fill is needed.
        if self.layout != Layout::RootImages as usize || c.continue_from.is_some() {
            initialize(bytes, c, seed);
        }
        self.copy_writes(number, bytes);
    }
    fn root_image(&self, number: usize) -> &[u8] {
        let first = u64_at(&self.bytes, self.index + number * 16).unwrap();
        assert_eq!(u64_at(&self.bytes, self.index + number * 16 + 8).unwrap(), 1);
        let at = self.records + first * 16;
        assert_eq!(u32_at(&self.bytes, at).unwrap(), 0);
        assert_eq!(u32_at(&self.bytes, at + 4).unwrap(), WRAM);
        let blob = u64_at(&self.bytes, at + 8).unwrap();
        &self.bytes[blob..blob + WRAM]
    }
    fn copy_writes(&self, number: usize, bytes: &mut [u8]) {
        let first = u64_at(&self.bytes, self.index + number * 16).unwrap();
        let count = u64_at(&self.bytes, self.index + number * 16 + 8).unwrap();
        for i in first..first + count {
            let at = self.records + i * 16;
            let offset = u32_at(&self.bytes, at).unwrap();
            let len = u32_at(&self.bytes, at + 4).unwrap();
            let blob = u64_at(&self.bytes, at + 8).unwrap();
            bytes[offset..offset + len].copy_from_slice(&self.bytes[blob..blob + len]);
        }
    }
}

#[derive(Clone)]
enum Kind {
    Toml,
    Binary,
    Compressed,
}
struct Variant {
    name: String,
    path: PathBuf,
    kind: Kind,
    file_hash: String,
    decoded_hash: String,
    decoded_len: usize,
}
fn save_variant(directory: &Path, name: &str, content: &[u8], kind: Kind, decoded: &[u8]) -> Result<Variant> {
    let path = directory.join(name);
    fs::write(&path, content)?;
    fs::File::open(&path)?.sync_all()?;
    Ok(Variant {
        name: name.into(),
        path,
        kind,
        file_hash: hash(content),
        decoded_hash: hash(decoded),
        decoded_len: decoded.len(),
    })
}
enum Loaded {
    Toml(Manifest),
    Binary(Opened),
}
fn load(v: &Variant) -> Result<Loaded> {
    let bytes = fs::read(&v.path)?;
    ensure!(hash(&bytes) == v.file_hash, "physical input changed");
    match v.kind {
        Kind::Toml => Ok(Loaded::Toml(toml::from_str(std::str::from_utf8(&bytes)?)?)),
        Kind::Binary => Ok(Loaded::Binary(Opened::new(bytes)?)),
        Kind::Compressed => {
            let decoded = zstd::bulk::decompress(&bytes, v.decoded_len)?;
            ensure!(decoded.len() == v.decoded_len && hash(&decoded) == v.decoded_hash, "decoded input changed");
            Ok(Loaded::Binary(Opened::new(decoded)?))
        }
    }
}
fn cpu_seconds() -> f64 {
    let mut usage = std::mem::MaybeUninit::<libc::rusage>::uninit();
    assert_eq!(unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) }, 0);
    let usage = unsafe { usage.assume_init() };
    usage.ru_utime.tv_sec as f64
        + usage.ru_utime.tv_usec as f64 / 1e6
        + usage.ru_stime.tv_sec as f64
        + usage.ru_stime.tv_usec as f64 / 1e6
}
fn materialize(input: &Loaded) -> u64 {
    let metadata = match input {
        Loaded::Toml(m) => m,
        Loaded::Binary(b) => &b.metadata,
    };
    let input_cases = cases(metadata);
    let mut check = 0u64;
    for (i, c) in input_cases.iter().enumerate() {
        let ram = match input {
            Loaded::Binary(b) if b.layout == Layout::RootImages as usize && c.continue_from.is_none() => {
                b.root_image(i).to_vec()
            }
            _ => {
                let mut ram = vec![c.fill.unwrap_or(0); WRAM];
                if c.continue_from.is_some() {
                    inherited(&mut ram, i as u8);
                }
                match input {
                    Loaded::Toml(_) => {
                        for w in &c.writes {
                            ram[w.offset..w.offset + w.data.len()].copy_from_slice(&w.data);
                        }
                    }
                    Loaded::Binary(b) => b.copy_writes(i, &mut ram),
                }
                ram
            }
        };
        check = check.wrapping_add(ram[i % WRAM] as u64);
        black_box(&ram);
    }
    black_box(check)
}
fn verify(original: &Manifest, opened: &Opened) -> Result<String> {
    ensure!(without_writes(original) == opened.metadata, "metadata differs");
    let a = cases(original);
    let b = cases(&opened.metadata);
    let mut before = vec![0; WRAM];
    let mut after = vec![0; WRAM];
    let mut digest = Sha256::new();
    for (i, (a, b)) in a.into_iter().zip(b).enumerate() {
        let seeds: &[u8] = if a.continue_from.is_some() { &[0, 91, 255] } else { &[0] };
        for seed in seeds {
            ordinary(&mut before, a, *seed);
            after.fill(0x6d);
            opened.apply(i, &mut after, b, *seed);
            ensure!(before == after, "complete WRAM differs at case {i}, seed {seed}");
            digest.update(&after);
        }
    }
    Ok(format!("{:x}", digest.finalize()))
}
#[derive(Serialize)]
struct Sample {
    variant: String,
    cache: String,
    repetition: usize,
    read_hash_decode_ms: f64,
    read_hash_decode_cpu_ms: f64,
    materialize_ms: f64,
    materialize_cpu_ms: f64,
}
fn measure(v: &Variant, cache: &str, repetition: usize) -> Result<Sample> {
    if cache == "warm" {
        black_box(fs::read(&v.path)?);
    } else {
        let f = fs::File::open(&v.path)?;
        ensure!(
            unsafe { libc::posix_fadvise(f.as_raw_fd(), 0, 0, libc::POSIX_FADV_DONTNEED) } == 0,
            "cache eviction hint failed"
        );
    }
    let cpu = cpu_seconds();
    let start = Instant::now();
    let loaded = load(v)?;
    let read_hash_decode_ms = start.elapsed().as_secs_f64() * 1000.0;
    let read_hash_decode_cpu_ms = (cpu_seconds() - cpu) * 1000.0;
    let cpu = cpu_seconds();
    let start = Instant::now();
    black_box(materialize(&loaded));
    let materialize_ms = start.elapsed().as_secs_f64() * 1000.0;
    let materialize_cpu_ms = (cpu_seconds() - cpu) * 1000.0;
    Ok(Sample {
        variant: v.name.clone(),
        cache: cache.into(),
        repetition,
        read_hash_decode_ms,
        read_hash_decode_cpu_ms,
        materialize_ms,
        materialize_cpu_ms,
    })
}
#[derive(Serialize)]
struct FamilyResult {
    family: String,
    original_bytes: usize,
    original_sha256: String,
    cases: usize,
    continuations: usize,
    original_writes: usize,
    encodings: Vec<Encoding>,
    state_identity: String,
    samples: Vec<Sample>,
}
#[derive(Serialize)]
struct Study {
    contract: &'static str,
    baseline: String,
    rustc: String,
    repetitions: usize,
    environment: HashMap<String, String>,
    scope: Vec<&'static str>,
    families: Vec<FamilyResult>,
}
fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(args.len() >= 3, "usage: fixture-format-study CHECKOUT OUTPUT REPETITIONS [FAMILY...]");
    let checkout = Path::new(&args[0]);
    let output = Path::new(&args[1]);
    fs::create_dir_all(output)?;
    let repetitions: usize = args[2].parse()?;
    ensure!((1..=8).contains(&repetitions), "bounded repetitions");
    let names = if args.len() > 3 {
        args[3..].to_vec()
    } else {
        vec!["floater-spike-motion".into(), "mother-brain-bomb-update".into(), "ridley-subsystem".into()]
    };
    let mut environment = HashMap::new();
    for path in ["/proc/loadavg", "/proc/meminfo", "/proc/version"] {
        environment.insert(path.into(), fs::read_to_string(path)?.lines().take(5).collect::<Vec<_>>().join("\n"));
    }
    let mut study = Study {
        contract: "fixture-format-study-v2",
        baseline: String::from_utf8(
            std::process::Command::new("git").arg("-C").arg(checkout).args(["rev-parse", "HEAD"]).output()?.stdout,
        )?
        .trim()
        .into(),
        rustc: String::from_utf8(std::process::Command::new("rustc").arg("--version").output()?.stdout)?.trim().into(),
        repetitions,
        environment,
        scope: vec![
            "Same typed Manifest; every metadata field retained in a smaller JSON section.",
            "Contiguous indexed binary patch records; shared immutable payloads; no Vec allocation per patch on read.",
            "Every layout authenticated by SHA-256 before timed decode; compressed layout authenticates both physical and decoded bytes.",
            "All full WRAM results compared before timing; continuations use three varied synthetic inherited states, not actual C executions.",
            "Cache-eviction mode is a POSIX_FADV_DONTNEED hint on private copied files, not a cold-machine guarantee.",
            "Materialization includes allocating a 128 KiB buffer per case, but excludes per-case file output, compilation, execution, and complete evidence authentication.",
            "Other sessions may use this machine. CPU time and wall time are both retained.",
            "No claim of fastest possible format; metadata JSON, mmap, output formats and end-to-end integration remain separate work.",
        ],
        families: Vec::new(),
    };
    for family in names {
        eprintln!("PREPARE {family}");
        let original = fs::read(checkout.join("tools/port").join(format!("{family}.toml")))?;
        let manifest: Manifest = toml::from_str(std::str::from_utf8(&original)?)?;
        let case_refs = cases(&manifest);
        let count = case_refs.len();
        let continuations = case_refs.iter().filter(|c| c.continue_from.is_some()).count();
        let original_writes = case_refs.iter().map(|c| c.writes.len()).sum();
        let directory = output.join(&family);
        fs::create_dir(&directory)?;
        let mut variants = vec![save_variant(&directory, "current.toml", &original, Kind::Toml, &original)?];
        let mut encodings = Vec::new();
        let mut state_identity = None;
        let selected = std::env::var("SM_FORMAT_STUDY_LAYOUTS")
            .unwrap_or_else(|_| "indexed-writes,compiled-spans,root-images,merged-spans".into());
        for layout in [Layout::IndexedWrites, Layout::CompiledSpans, Layout::RootImages, Layout::MergedSpans] {
            if !selected.split(',').any(|name| name == layout.name()) {
                continue;
            }
            eprintln!("COMPILE {family} {}", layout.name());
            let (bytes, evidence) = encode(&manifest, layout)?;
            let opened = Opened::new(bytes)?;
            let identity = verify(&manifest, &opened)?;
            if let Some(previous) = &state_identity {
                ensure!(previous == &identity, "layout state identity changed");
            }
            state_identity = Some(identity);
            variants.push(save_variant(&directory, layout.name(), &opened.bytes, Kind::Binary, &opened.bytes)?);
            if matches!(layout, Layout::CompiledSpans | Layout::MergedSpans) {
                let compression_clock = Instant::now();
                let compressed = zstd::bulk::compress(&opened.bytes, 1)?;
                let name = format!("{}-zstd-1", layout.name());
                encodings.push(Encoding {
                    layout: name.clone(),
                    bytes: compressed.len(),
                    metadata_bytes: evidence.metadata_bytes,
                    cases: count,
                    patch_records: evidence.patch_records,
                    unique_payload_bytes: evidence.unique_payload_bytes,
                    compilation_ms: evidence.compilation_ms + compression_clock.elapsed().as_secs_f64() * 1000.0,
                    encoded_sha256: hash(&compressed),
                });
                variants.push(save_variant(&directory, &name, &compressed, Kind::Compressed, &opened.bytes)?);
            }
            encodings.push(evidence);
        }
        drop(manifest);
        let mut samples = Vec::new();
        for cache in ["warm", "eviction-hint"] {
            for repetition in 0..repetitions {
                for position in 0..variants.len() {
                    let v = &variants[(position + repetition) % variants.len()];
                    let sample = measure(v, cache, repetition)?;
                    eprintln!(
                        "SAMPLE {family} {cache} {} load={:.3}ms materialize={:.3}ms",
                        v.name, sample.read_hash_decode_ms, sample.materialize_ms
                    );
                    samples.push(sample);
                }
            }
        }
        study.families.push(FamilyResult {
            family,
            original_bytes: original.len(),
            original_sha256: hash(&original),
            cases: count,
            continuations,
            original_writes,
            encodings,
            state_identity: state_identity.unwrap(),
            samples,
        });
        fs::write(output.join("results.json"), serde_json::to_vec_pretty(&study)?)?;
    }
    println!("{}", serde_json::to_string_pretty(&study)?);
    Ok(())
}
