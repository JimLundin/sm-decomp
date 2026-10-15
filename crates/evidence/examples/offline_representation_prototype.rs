//! Throwaway, bounded Offline representation study; not an admission adapter.
//!
//! Every logical member of one already authenticated, completed Offline run is
//! retained, including its untouched verdict, producer records and source copies.
//! This measures conversion of finished staging, not direct producer transport.
//! See .scratch/decomp-verification-refactor-plan/evidence-study.md for commands.
use anyhow::{Context, Result, bail, ensure};
use evidence::{Entry, EntryKind, Id, Provenance, Store, hex, parse_id};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

const FORMAT: &str = "offline-representation-prototype-v1";
const MAX_FILE: u64 = 64 << 20;
const MAX_INPUT: u64 = 256 << 20;
const MAX_MEMBERS: usize = 4096;

#[path = "offline_representation_prototype/controls.rs"]
mod failure_controls;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Member {
    path: String,
    bytes: u64,
    sha256: String,
    mode: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Seal {
    path: PathBuf,
    sha256: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    format: String,
    source: PathBuf,
    verdict_sha256: String,
    authenticated_receipt: Seal,
    cleanup_reader: Seal,
    family: String,
    fixture_pairs: u64,
    members: Vec<Member>,
    source_disk: Disk,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum Mode {
    Directory,
    Merkle256,
    Raw1m,
    Raw1mImport4,
}
impl Mode {
    fn name(self) -> &'static str {
        match self {
            Self::Directory => "directory",
            Self::Merkle256 => "merkle256",
            Self::Raw1m => "raw1m",
            Self::Raw1mImport4 => "raw1m-import4",
        }
    }
    fn parse(value: &str) -> Result<Self> {
        [Self::Directory, Self::Merkle256, Self::Raw1m, Self::Raw1mImport4]
            .into_iter()
            .find(|m| m.name() == value)
            .context("unknown study mode")
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Publication {
    format: String,
    input_sha256: String,
    mode: Mode,
    run: Option<String>,
    // A private store has exactly this physical membership. This is not a
    // proposed shared-store GC policy; imported unreferenced objects are legal.
    packs: Vec<String>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Disk {
    files: u64,
    logical_bytes: u64,
    unique_files: u64,
    unique_allocated_bytes: u64,
    directories: u64,
    directory_allocated_bytes: u64,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
struct Resources {
    user_ticks: u64,
    system_ticks: u64,
    waited_child_user_ticks: u64,
    waited_child_system_ticks: u64,
    io_counters_available: bool,
    rchar: Option<u64>,
    wchar: Option<u64>,
    syscr: Option<u64>,
    syscw: Option<u64>,
    read_bytes: Option<u64>,
    write_bytes: Option<u64>,
    peak_rss_kib: u64,
}
impl Resources {
    fn read() -> Result<Self> {
        let stat = fs::read_to_string("/proc/self/stat")?;
        let fields: Vec<_> = stat.rsplit_once(')').context("invalid proc stat")?.1.split_whitespace().collect();
        let io = if Path::new("/proc/self/io").exists() { proc_fields("/proc/self/io")? } else { BTreeMap::new() };
        let status = proc_fields("/proc/self/status")?;
        Ok(Self {
            user_ticks: fields[11].parse()?,
            system_ticks: fields[12].parse()?,
            waited_child_user_ticks: fields[13].parse()?,
            waited_child_system_ticks: fields[14].parse()?,
            io_counters_available: !io.is_empty(),
            rchar: io.get("rchar").copied(),
            wchar: io.get("wchar").copied(),
            syscr: io.get("syscr").copied(),
            syscw: io.get("syscw").copied(),
            read_bytes: io.get("read_bytes").copied(),
            write_bytes: io.get("write_bytes").copied(),
            peak_rss_kib: status["VmHWM"],
        })
    }
    fn since(&self, earlier: &Self) -> Self {
        Self {
            user_ticks: self.user_ticks.saturating_sub(earlier.user_ticks),
            system_ticks: self.system_ticks.saturating_sub(earlier.system_ticks),
            waited_child_user_ticks: self.waited_child_user_ticks.saturating_sub(earlier.waited_child_user_ticks),
            waited_child_system_ticks: self.waited_child_system_ticks.saturating_sub(earlier.waited_child_system_ticks),
            io_counters_available: self.io_counters_available && earlier.io_counters_available,
            rchar: self.rchar.zip(earlier.rchar).map(|(a, b)| a.saturating_sub(b)),
            wchar: self.wchar.zip(earlier.wchar).map(|(a, b)| a.saturating_sub(b)),
            syscr: self.syscr.zip(earlier.syscr).map(|(a, b)| a.saturating_sub(b)),
            syscw: self.syscw.zip(earlier.syscw).map(|(a, b)| a.saturating_sub(b)),
            read_bytes: self.read_bytes.zip(earlier.read_bytes).map(|(a, b)| a.saturating_sub(b)),
            write_bytes: self.write_bytes.zip(earlier.write_bytes).map(|(a, b)| a.saturating_sub(b)),
            peak_rss_kib: self.peak_rss_kib,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Stage {
    name: String,
    seconds: f64,
    resources: Resources,
}

#[derive(Debug, Serialize, Deserialize)]
struct Sample {
    format: String,
    scope: String,
    path: PathBuf,
    mode: Mode,
    input_sha256: String,
    publication_sha256: String,
    logical_members: usize,
    logical_bytes: u64,
    simulated_capture_seconds: f64,
    from_completed_staging_seconds: f64,
    complete_lifecycle_seconds: f64,
    clock_ticks_per_second: u64,
    host_start: Host,
    host_end: Host,
    resources: Resources,
    staged: Disk,
    before_staging_release: Disk,
    retained_representation: Disk,
    stages: Vec<Stage>,
    writers: Vec<WriterTiming>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Host {
    unix_milliseconds: u64,
    loadavg: String,
    cpu_count: usize,
    mem_available_kib: u64,
    swap_free_kib: u64,
}
impl Host {
    fn read() -> Result<Self> {
        let memory = proc_fields("/proc/meminfo")?;
        Ok(Self {
            unix_milliseconds: SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis().try_into()?,
            loadavg: fs::read_to_string("/proc/loadavg")?.trim().into(),
            cpu_count: std::thread::available_parallelism()?.get(),
            mem_available_kib: memory["MemAvailable"],
            swap_free_kib: memory["SwapFree"],
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct WriterTiming {
    writer: String,
    encode_hash_compress_and_closure_seconds: f64,
    commit_seconds: f64,
    new_objects: u64,
    shared_objects: u64,
}

#[derive(Clone)]
struct Leaf {
    path: String,
    kind: EntryKind,
    id: Id,
}

fn proc_fields(path: &str) -> Result<BTreeMap<String, u64>> {
    Ok(fs::read_to_string(path)?
        .lines()
        .filter_map(|line| {
            let (key, value) = line.split_once(':')?;
            Some((key.into(), value.split_whitespace().next()?.parse().ok()?))
        })
        .collect())
}

fn hash(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes).into())
}

fn file_hash(path: &Path) -> Result<String> {
    let mut input = File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let n = input.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    Ok(hex(&hash.finalize().into()))
}

fn read_document<T: for<'a> Deserialize<'a>>(path: &Path, sha256: &str) -> Result<T> {
    let bytes = bounded_bytes(path, 2 << 20)?;
    ensure!(hash(&bytes) == sha256, "study document seal differs: {}", path.display());
    Ok(toml::from_str(std::str::from_utf8(&bytes)?)?)
}

fn bounded_bytes(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let meta = path.symlink_metadata()?;
    ensure!(meta.is_file() && meta.len() <= limit, "file type or byte bound differs");
    let mut bytes = Vec::new();
    File::open(path)?.take(limit + 1).read_to_end(&mut bytes)?;
    ensure!(bytes.len() as u64 <= limit, "file grew beyond byte bound");
    Ok(bytes)
}

fn publish<T: Serialize>(path: &Path, value: &T) -> Result<String> {
    ensure!(!path.exists(), "publication already exists");
    let bytes = toml::to_string_pretty(value)?.into_bytes();
    let pending = path.with_extension("pending");
    let mut file = OpenOptions::new().write(true).create_new(true).open(&pending)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    drop(file);
    fs::rename(&pending, path)?;
    File::open(path.parent().context("missing publication parent")?)?.sync_all()?;
    Ok(hash(&bytes))
}

fn safe_relative(path: &str) -> Result<()> {
    ensure!(!path.is_empty() && !path.contains(['\\', '\0']), "invalid logical member name");
    ensure!(path.split('/').all(|s| !s.is_empty() && s != "." && s != ".."), "invalid logical member component");
    ensure!(Path::new(path).components().all(|c| matches!(c, Component::Normal(_))), "unsafe member path");
    Ok(())
}

fn paths(root: &Path) -> Result<Vec<String>> {
    fn walk(root: &Path, prefix: &str, depth: usize, out: &mut Vec<String>) -> Result<()> {
        ensure!(depth <= 64 && fs::symlink_metadata(root)?.is_dir(), "invalid capture directory");
        for entry in fs::read_dir(root)? {
            let entry = entry?;
            let name = entry.file_name().into_string().map_err(|_| anyhow::anyhow!("non UTF-8 member"))?;
            let relative = format!("{prefix}{name}");
            safe_relative(&relative)?;
            let meta = entry.path().symlink_metadata()?;
            if meta.is_dir() {
                walk(&entry.path(), &format!("{relative}/"), depth + 1, out)?;
            } else {
                ensure!(meta.is_file(), "symlink or special member rejected");
                ensure!(out.len() < MAX_MEMBERS && meta.len() <= MAX_FILE, "capture bounds exceeded");
                out.push(relative);
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    walk(root, "", 0, &mut out)?;
    out.sort();
    Ok(out)
}

fn disk(root: &Path) -> Result<Disk> {
    fn walk(path: &Path, seen: &mut BTreeSet<(u64, u64)>, out: &mut Disk) -> Result<()> {
        let meta = fs::symlink_metadata(path)?;
        ensure!(meta.is_dir() || meta.is_file(), "special file in study outputs");
        if meta.is_dir() {
            out.directories += 1;
            out.directory_allocated_bytes += meta.blocks() * 512;
            for entry in fs::read_dir(path)? {
                walk(&entry?.path(), seen, out)?;
            }
        } else {
            out.files += 1;
            out.logical_bytes += meta.len();
            if seen.insert((meta.dev(), meta.ino())) {
                out.unique_files += 1;
                out.unique_allocated_bytes += meta.blocks() * 512;
            }
        }
        Ok(())
    }
    let mut out = Disk::default();
    walk(root, &mut BTreeSet::new(), &mut out)?;
    Ok(out)
}

fn inventory(root: &Path) -> Result<Vec<Member>> {
    let mut total = 0;
    paths(root)?
        .into_iter()
        .map(|path| {
            let full = root.join(&path);
            let meta = full.symlink_metadata()?;
            total += meta.len();
            ensure!(total <= MAX_INPUT, "input exceeds total byte bound");
            Ok(Member { path, bytes: meta.len(), sha256: file_hash(&full)?, mode: meta.permissions().mode() & 0o777 })
        })
        .collect()
}

fn validate_members(members: &[Member]) -> Result<()> {
    ensure!(!members.is_empty() && members.len() <= MAX_MEMBERS, "invalid member count");
    let mut previous: Option<&str> = None;
    let mut total = 0;
    for member in members {
        safe_relative(&member.path)?;
        ensure!(previous.is_none_or(|p| p < member.path.as_str()), "duplicate or unordered member");
        previous = Some(&member.path);
        parse_id(&member.sha256)?;
        total += member.bytes;
        ensure!(member.bytes <= MAX_FILE && total <= MAX_INPUT, "input bounds exceeded");
    }
    Ok(())
}

fn input(root: &Path, seal: &str) -> Result<Input> {
    let input: Input = read_document(&root.join("input.toml"), seal)?;
    ensure!(input.format == FORMAT, "unknown study input format");
    validate_members(&input.members)?;
    Ok(input)
}

fn prepare(source: &Path, authenticated: &Path, cleanup: &Path, root: &Path) -> Result<()> {
    ensure!(!root.exists(), "study needs a new output root");
    let source = fs::canonicalize(source)?;
    let verdict_bytes = bounded_bytes(&source.join("verdict.toml"), 2 << 20)?;
    let verdict: toml::Value = toml::from_str(std::str::from_utf8(&verdict_bytes)?)?;
    let receipt_bytes = bounded_bytes(authenticated, 2 << 20)?;
    let receipt: toml::Value = toml::from_str(std::str::from_utf8(&receipt_bytes)?)?;
    ensure!(verdict["status"].as_str() == Some("offline-verified"), "only completed ordinary Offline input supported");
    ensure!(
        receipt["status"] == verdict["status"] && receipt["run"] == verdict["run"],
        "authenticated input receipt differs"
    );
    ensure!(Path::new(verdict["run"].as_str().context("missing run")?) == source, "verdict run path differs");
    let members = inventory(&source)?;
    validate_members(&members)?;
    let found: BTreeMap<_, _> = members.iter().map(|m| (m.path.as_str(), m.sha256.as_str())).collect();
    for (path, seal) in verdict["artifacts"].as_table().context("missing original artifact seals")? {
        ensure!(found.get(path.as_str()).copied() == seal.as_str(), "original artifact seal differs: {path}");
    }
    let input = Input {
        format: FORMAT.into(),
        source: source.clone(),
        verdict_sha256: hash(&verdict_bytes),
        authenticated_receipt: Seal { path: fs::canonicalize(authenticated)?, sha256: hash(&receipt_bytes) },
        cleanup_reader: Seal { path: fs::canonicalize(cleanup)?, sha256: file_hash(cleanup)? },
        family: verdict["family"].as_str().context("missing family")?.into(),
        fixture_pairs: verdict["verified_cases"].as_integer().context("missing case count")?.try_into()?,
        members,
        source_disk: disk(&source)?,
    };
    fs::create_dir_all(root)?;
    let seal = publish(&root.join("input.toml"), &input)?;
    println!("input_sha256 = {seal:?}");
    println!("logical_members = {}\nfixture_pairs = {}", input.members.len(), input.fixture_pairs);
    Ok(())
}

// Copy into independent staging, preserving only hard links *within* the new
// directory. Never hard-link a study output to the read-only original proof.
fn capture(source: &Path, target: &Path, members: &[Member]) -> Result<()> {
    ensure!(!target.exists(), "capture target already exists");
    fs::create_dir(target)?;
    let mut links = BTreeMap::new();
    for member in members {
        let source_path = source.join(&member.path);
        let target_path = target.join(&member.path);
        let meta = source_path.symlink_metadata()?;
        ensure!(meta.is_file() && meta.len() == member.bytes, "capture source changed");
        fs::create_dir_all(target_path.parent().unwrap())?;
        if let Some(previous) = links.get(&(meta.dev(), meta.ino())) {
            fs::hard_link(previous, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path)?;
            links.insert((meta.dev(), meta.ino()), target_path);
        }
    }
    Ok(())
}

fn stage<T>(stages: &mut Vec<Stage>, name: &str, op: impl FnOnce() -> Result<T>) -> Result<T> {
    let resources = Resources::read()?;
    let start = Instant::now();
    let result = op()?;
    let seconds = start.elapsed().as_secs_f64();
    stages.push(Stage { name: name.into(), seconds, resources: Resources::read()?.since(&resources) });
    Ok(result)
}

fn provenance(input_sha256: &str) -> Provenance {
    Provenance {
        subject: Some("storage-only copy of completed Offline evidence; not new admission".into()),
        inputs_sha256: Some(input_sha256.into()),
        ..Default::default()
    }
}

fn image_member(name: &str) -> bool {
    [".wram", ".rom", ".bin", ".srm", ".sram"].iter().any(|suffix| name.ends_with(suffix))
}

fn tree(store: &mut Store, leaves: &[Leaf]) -> Result<Id> {
    fn build(store: &mut Store, leaves: &[(&str, &Leaf)]) -> Result<Id> {
        let mut directories: BTreeMap<&str, Vec<(&str, &Leaf)>> = BTreeMap::new();
        let mut entries = Vec::new();
        for (path, leaf) in leaves {
            if let Some((first, rest)) = path.split_once('/') {
                directories.entry(first).or_default().push((rest, leaf));
            } else {
                entries.push(Entry { name: (*path).into(), kind: leaf.kind, id: leaf.id });
            }
        }
        for (name, children) in directories {
            entries.push(Entry { name: name.into(), kind: EntryKind::Tree, id: build(store, &children)? });
        }
        evidence::put_tree(store, entries)
    }
    build(store, &leaves.iter().map(|l| (l.path.as_str(), l)).collect::<Vec<_>>())
}

fn write_store(
    staging: &Path,
    target: &Path,
    members: &[Member],
    mode: Mode,
    input_sha256: &str,
) -> Result<(Id, Vec<Leaf>, WriterTiming)> {
    let start = Instant::now();
    let mut store = Store::create(target)?;
    let mut leaves = Vec::new();
    for member in members {
        // At most one bounded file per writer is owned; no whole-family cache.
        let bytes = bounded_bytes(&staging.join(&member.path), MAX_FILE)?;
        ensure!(bytes.len() as u64 == member.bytes && hash(&bytes) == member.sha256, "staged member differs");
        let (kind, id) = if mode == Mode::Merkle256 && image_member(&member.path) {
            (EntryKind::Image, evidence::put_image(&mut store, &bytes)?)
        } else {
            (EntryKind::File, evidence::put_file(&mut store, &mut bytes.as_slice())?)
        };
        leaves.push(Leaf { path: member.path.clone(), kind, id });
    }
    let observations = tree(&mut store, &leaves)?;
    let run = evidence::put_run(&mut store, observations, &provenance(input_sha256))?;
    let encode_seconds = start.elapsed().as_secs_f64();
    let stats = store.stats();
    let start = Instant::now();
    store.commit()?;
    let timing = WriterTiming {
        writer: target.file_name().unwrap().to_string_lossy().into(),
        encode_hash_compress_and_closure_seconds: encode_seconds,
        commit_seconds: start.elapsed().as_secs_f64(),
        new_objects: stats.new_objects,
        shared_objects: stats.shared_objects,
    };
    Ok((run, leaves, timing))
}

fn check_store(root: &Path, run: &Id, expected: &[Member], input_sha256: &str) -> Result<()> {
    let store = Store::open(root)?;
    store.verify_packs()?;
    let (run, inventory) = evidence::verified_inventory(&store, run)?;
    ensure!(run.provenance == provenance(input_sha256), "stored provenance differs");
    ensure!(inventory.len() == expected.len(), "logical membership differs");
    for member in expected {
        let found = inventory.get(&member.path).context("missing logical member")?;
        ensure!(
            found.bytes == member.bytes && hex(&found.sha256) == member.sha256,
            "logical bytes differ: {}",
            member.path
        );
    }
    Ok(())
}

fn packs(root: &Path) -> Result<Vec<String>> {
    let names = paths(root)?;
    ensure!(names.iter().all(|n| n.ends_with(".pack") && !n.contains('/')), "unexpected private store member");
    Ok(names)
}

fn verify_publication(root: &Path, publication: &Publication, input: &Input, seal: &str) -> Result<()> {
    ensure!(publication.format == FORMAT && publication.input_sha256 == seal, "publication inputs differ");
    if publication.mode == Mode::Directory {
        ensure!(publication.run.is_none() && publication.packs.is_empty(), "directory has unexpected store fields");
        ensure!(inventory(&root.join("staging"))? == input.members, "directory members, modes or bytes differ");
    } else {
        ensure!(packs(&root.join("store"))? == publication.packs, "private pack membership differs");
        check_store(
            &root.join("store"),
            &parse_id(publication.run.as_deref().context("missing run id")?)?,
            &input.members,
            seal,
        )?;
    }
    Ok(())
}

fn verify(root: &Path, input_sha: &str, sample: &Path, publication_sha: &str) -> Result<()> {
    let input = input(root, input_sha)?;
    let publication: Publication = read_document(&sample.join("publication.toml"), publication_sha)?;
    verify_publication(sample, &publication, &input, input_sha)?;
    println!("status = \"exact-original-members-authenticated\"\nlogical_members = {}", input.members.len());
    Ok(())
}

fn cleanup(reader: &Path, root: &Path, deduplicate: bool) -> Result<()> {
    let suffix = if deduplicate { "deduplicate" } else { "ordinary" };
    let out = File::create(root.join(format!("cleanup-{suffix}.toml")))?;
    let err = File::create(root.join(format!("cleanup-{suffix}.stderr")))?;
    let mut command = Command::new(reader);
    command.args(["cleanup", "--root"]).arg(root).arg("--apply");
    if deduplicate {
        command.arg("--deduplicate");
    }
    ensure!(command.stdout(out).stderr(err).status()?.success(), "scoped cleanup failed");
    Ok(())
}

fn collect_leaves(store: &Store, root: &Id, prefix: &str, out: &mut BTreeMap<String, Leaf>) -> Result<()> {
    ensure!(prefix.split('/').count() <= 65 && out.len() <= MAX_MEMBERS, "extraction bounds exceeded");
    for entry in evidence::read_tree(store, root)? {
        let name = format!("{prefix}{}", entry.name);
        if entry.kind == EntryKind::Tree {
            collect_leaves(store, &entry.id, &format!("{name}/"), out)?;
        } else {
            ensure!(
                out.insert(name.clone(), Leaf { path: name, kind: entry.kind, id: entry.id }).is_none(),
                "duplicate extracted member"
            );
        }
    }
    Ok(())
}

// Compare actual extracted bytes, not only matching digest declarations.
fn exact_files(left: &Path, right: &Path) -> Result<()> {
    let mut left = File::open(left)?;
    let mut right = File::open(right)?;
    let mut a = [0; 65536];
    let mut b = [0; 65536];
    loop {
        let n = left.read(&mut a)?;
        if n == 0 {
            ensure!(right.read(&mut b[..1])? == 0, "extraction has trailing bytes");
            return Ok(());
        }
        right.read_exact(&mut b[..n])?;
        ensure!(a[..n] == b[..n], "extracted bytes differ");
    }
}

fn extract(root: &Path, publication: &Publication, input: &Input) -> Result<()> {
    let target = root.join("extraction");
    ensure!(!target.exists(), "extraction target already exists");
    if publication.mode == Mode::Directory {
        capture(&root.join("staging"), &target, &input.members)?;
    } else {
        fs::create_dir(&target)?;
        let store = Store::open(&root.join("store"))?;
        let run = evidence::read_run(&store, &parse_id(publication.run.as_deref().unwrap())?)?;
        let mut leaves = BTreeMap::new();
        collect_leaves(&store, &run.observations, "", &mut leaves)?;
        ensure!(leaves.len() == input.members.len(), "extraction membership differs");
        for member in &input.members {
            let leaf = leaves.get(&member.path).context("missing extraction member")?;
            let path = target.join(&member.path);
            fs::create_dir_all(path.parent().unwrap())?;
            let mut out = OpenOptions::new().create_new(true).write(true).open(&path)?;
            evidence::write_entry(&store, leaf.kind, &leaf.id, &mut out)?;
            fs::set_permissions(path, fs::Permissions::from_mode(member.mode))?;
        }
    }
    ensure!(inventory(&target)? == input.members, "extraction inventory differs");
    for member in &input.members {
        exact_files(&input.source.join(&member.path), &target.join(&member.path))?;
    }
    Ok(())
}

fn sample(root: &Path, seal: &str, mode: Mode, output: &Path) -> Result<()> {
    let input = input(root, seal)?;
    ensure!(file_hash(&input.cleanup_reader.path)? == input.cleanup_reader.sha256, "cleanup reader changed");
    ensure!(!output.exists(), "sample output already exists");
    fs::create_dir_all(output)?;
    let output = fs::canonicalize(output)?;
    let host_start = Host::read()?;
    let resources = Resources::read()?;
    let start = Instant::now();
    let mut stages = Vec::new();
    stage(&mut stages, "simulated_staging_capture_preserving_existing_hardlinks", || {
        capture(&input.source, &output.join("staging"), &input.members)
    })?;
    let simulated_capture_seconds = stages.last().unwrap().seconds;
    let staged = disk(&output.join("staging"))?;
    let from_staging = Instant::now();
    stage(&mut stages, "completed_staging_inventory_authentication", || {
        ensure!(inventory(&output.join("staging"))? == input.members, "captured input changed");
        Ok(())
    })?;
    let mut writers = Vec::new();
    let mut run = None;
    if mode == Mode::Raw1mImport4 {
        let mut shards = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        for (i, member) in input.members.iter().enumerate() {
            shards[i % 4].push(member.clone());
        }
        let results = stage(&mut stages, "four_private_writers_encode_hash_compress_commit", || {
            std::thread::scope(|scope| {
                let handles: Vec<_> = shards
                    .iter()
                    .enumerate()
                    .map(|(i, members)| {
                        let staging = output.join("staging");
                        let target = output.join(format!("private-{i}"));
                        scope.spawn(move || write_store(&staging, &target, members, Mode::Raw1m, seal))
                    })
                    .collect();
                handles
                    .into_iter()
                    .map(|h| h.join().map_err(|_| anyhow::anyhow!("private writer panicked"))?)
                    .collect::<Result<Vec<_>>>()
            })
        })?;
        stage(&mut stages, "private_stores_reopen_physical_and_logical_authentication", || {
            for (i, (id, _, _)) in results.iter().enumerate() {
                check_store(&output.join(format!("private-{i}")), id, &shards[i], seal)?;
            }
            Ok(())
        })?;
        run = Some(stage(&mut stages, "ordinal_remapping_import_combined_closure_and_commit", || {
            let mut destination = Store::create(&output.join("store"))?;
            let mut leaves = Vec::new();
            for (i, (_, shard_leaves, _)) in results.iter().enumerate() {
                let source = Store::open(&output.join(format!("private-{i}")))?;
                destination.import_store(&source)?;
                leaves.extend(shard_leaves.iter().cloned());
            }
            let tree = tree(&mut destination, &leaves)?;
            let run = evidence::put_run(&mut destination, tree, &provenance(seal))?;
            destination.commit()?;
            Ok(run)
        })?);
        writers.extend(results.into_iter().map(|(_, _, timing)| timing));
    } else if mode != Mode::Directory {
        let (id, _, timing) = stage(&mut stages, "encode_hash_compress_closure_and_commit", || {
            write_store(&output.join("staging"), &output.join("store"), &input.members, mode, seal)
        })?;
        writers.push(timing);
        run = Some(id);
    }
    let publication = Publication {
        format: FORMAT.into(),
        input_sha256: seal.into(),
        mode,
        run: run.as_ref().map(hex),
        packs: if run.is_some() { packs(&output.join("store"))? } else { Vec::new() },
    };
    stage(&mut stages, "reopen_complete_representation_and_authenticate", || {
        verify_publication(&output, &publication, &input, seal)
    })?;
    let publication_sha256 = stage(&mut stages, "publish_after_authenticated_commit", || {
        publish(&output.join("publication.toml"), &publication)
    })?;
    let before_staging_release = disk(&output)?;
    stage(&mut stages, "release_only_identified_unsealed_staging_and_import_sources", || {
        if mode != Mode::Directory {
            fs::remove_dir_all(output.join("staging"))?;
        }
        if mode == Mode::Raw1mImport4 {
            for i in 0..4 {
                fs::remove_dir_all(output.join(format!("private-{i}")))?;
            }
        }
        Ok(())
    })?;
    stage(&mut stages, "supported_ordinary_cleanup", || cleanup(&input.cleanup_reader.path, &output, false))?;
    stage(&mut stages, "supported_deduplicating_cleanup", || cleanup(&input.cleanup_reader.path, &output, true))?;
    stage(&mut stages, "post_cleanup_fresh_reader_authentication", || {
        let publication = read_document(&output.join("publication.toml"), &publication_sha256)?;
        verify_publication(&output, &publication, &input, seal)
    })?;
    stage(&mut stages, "fresh_reader_exact_extraction_and_original_byte_comparison", || {
        extract(&output, &publication, &input)
    })?;
    stage(&mut stages, "remove_identified_unsealed_extraction", || {
        fs::remove_dir_all(output.join("extraction"))?;
        Ok(())
    })?;
    let from_completed_staging_seconds = from_staging.elapsed().as_secs_f64();
    let complete_lifecycle_seconds = start.elapsed().as_secs_f64();
    let measured_resources = Resources::read()?.since(&resources);
    let retained_representation = disk(&output.join(if mode == Mode::Directory { "staging" } else { "store" }))?;
    let clk_tck = Command::new("getconf").arg("CLK_TCK").output()?;
    ensure!(clk_tck.status.success(), "cannot read kernel clock frequency");
    let report = Sample {
        format: FORMAT.into(),
        scope: "storage-only; finished authenticated ordinary run; initial producer execution excluded".into(),
        path: output.clone(),
        mode,
        input_sha256: seal.into(),
        publication_sha256,
        logical_members: input.members.len(),
        logical_bytes: input.members.iter().map(|m| m.bytes).sum(),
        simulated_capture_seconds,
        from_completed_staging_seconds,
        complete_lifecycle_seconds,
        clock_ticks_per_second: String::from_utf8(clk_tck.stdout)?.trim().parse()?,
        host_start,
        host_end: Host::read()?,
        resources: measured_resources,
        staged,
        before_staging_release,
        retained_representation,
        stages,
        writers,
    };
    publish(&output.join("sample.toml"), &report)?;
    print!("{}", toml::to_string_pretty(&report)?);
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Matrix {
    format: String,
    input_sha256: String,
    per_sample_timeout_seconds: u64,
    samples: Vec<Sample>,
}

fn measure(root: &Path, seal: &str) -> Result<()> {
    input(root, seal)?;
    let parent = root.join("samples");
    ensure!(!parent.exists(), "matrix already exists");
    fs::create_dir(&parent)?;
    let mut samples = Vec::new();
    for repetition in 0..3 {
        for candidate in [Mode::Merkle256, Mode::Raw1m, Mode::Raw1mImport4] {
            let order = if repetition % 2 == 0 { [Mode::Directory, candidate] } else { [candidate, Mode::Directory] };
            for (position, mode) in order.into_iter().enumerate() {
                let name = format!("r{repetition}-{}-{position}-{}", candidate.name(), mode.name());
                let path = parent.join(&name);
                let stdout = File::create(parent.join(format!("{name}.stdout.toml")))?;
                let stderr = File::create(parent.join(format!("{name}.stderr")))?;
                let status = Command::new("timeout")
                    .args(["--signal=TERM", "--kill-after=5", "60"])
                    .arg(std::env::current_exe()?)
                    .arg("sample")
                    .arg(root)
                    .arg(seal)
                    .arg(mode.name())
                    .arg(&path)
                    .stdout(stdout)
                    .stderr(stderr)
                    .status()?;
                ensure!(status.success(), "sample {name} failed or timed out: {status}");
                let sample: Sample = toml::from_str(&fs::read_to_string(path.join("sample.toml"))?)?;
                eprintln!(
                    "{name}: {:.4}s from staged input, {} retained bytes, {} files",
                    sample.from_completed_staging_seconds,
                    sample.retained_representation.unique_allocated_bytes,
                    sample.retained_representation.files
                );
                samples.push(sample);
            }
        }
    }
    let matrix = Matrix { format: FORMAT.into(), input_sha256: seal.into(), per_sample_timeout_seconds: 60, samples };
    println!("matrix_sha256 = {:?}", publish(&root.join("matrix.toml"), &matrix)?);
    Ok(())
}

fn check_matrix(root: &Path, seal: &str, matrix_sha: &str) -> Result<()> {
    let matrix: Matrix = read_document(&root.join("matrix.toml"), matrix_sha)?;
    ensure!(matrix.format == FORMAT && matrix.input_sha256 == seal, "matrix input differs");
    for sample in matrix.samples {
        verify(root, seal, &sample.path, &sample.publication_sha256)?;
    }
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [command, source, authenticated, cleanup, root] if command == "prepare" => {
            prepare(Path::new(source), Path::new(authenticated), Path::new(cleanup), Path::new(root))
        }
        [command, root, seal, mode, output] if command == "sample" => {
            sample(Path::new(root), seal, Mode::parse(mode)?, Path::new(output))
        }
        [command, root, seal] if command == "measure" => measure(Path::new(root), seal),
        [command, root, seal, output, publication] if command == "verify" => {
            verify(Path::new(root), seal, Path::new(output), publication)
        }
        [command, root, seal, matrix] if command == "check-matrix" => check_matrix(Path::new(root), seal, matrix),
        [command, root, seal, matrix] if command == "controls" => {
            failure_controls::controls(Path::new(root), seal, matrix)
        }
        [command, root, seal] if command == "crash-writer" => failure_controls::crash_writer(Path::new(root), seal),
        _ => bail!(
            "usage: offline_representation_prototype prepare SOURCE_RUN AUTHENTICATED_RECEIPT CLEANUP_READER NEW_ROOT | sample ROOT INPUT_SHA MODE NEW_OUTPUT | measure ROOT INPUT_SHA | verify ROOT INPUT_SHA SAMPLE PUBLICATION_SHA | check-matrix ROOT INPUT_SHA MATRIX_SHA"
        ),
    }
}
