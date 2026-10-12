//! Bounded translation research discovery; planning facts never grant gameplay admission.
use crate::assignment::{Job, Kind};
use crate::family_model::Source;
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
};

const MAX_JOBS: usize = 8;
const MAX_DOCUMENT_BYTES: u64 = 32 * 1024 * 1024;
const ROM_SHA256: &str = "12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Verification {
    pub argv: Vec<String>,
    pub timeout_seconds: u64,
    #[serde(default)]
    pub integration: bool,
}

/// Research agents publish this native document before requesting implementation.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Scope {
    pub format: String,
    pub subject: String,
    pub source_inventory: PathBuf,
    pub source_inventory_sha256: String,
    pub exercise: PathBuf,
    pub exercise_sha256: String,
    pub routines: Vec<String>,
    pub dependencies: Vec<PathBuf>,
    pub dependency_review: BTreeMap<String, DependencyReview>,
    pub writable_paths: Vec<PathBuf>,
    pub inputs: BTreeMap<String, String>,
    pub verification: Vec<Verification>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DependencyReview {
    pub rationale: String,
    /// Pinned source, passing evidence or completed ticket supporting this decision.
    pub evidence: Vec<PathBuf>,
}

#[derive(Deserialize)]
struct Inventory {
    schema_version: u32,
    source_identity: Source,
    source_identity_sha256: String,
    routines: Vec<Routine>,
}
#[derive(Clone, Debug, Deserialize)]
struct Routine {
    symbol: String,
    source: String,
    line: usize,
    definition_sha256: String,
    #[serde(default)]
    original_address: Option<String>,
    #[serde(default)]
    defined_dependencies: Vec<String>,
    #[serde(default)]
    indirect_indexed_calls: Vec<String>,
    #[serde(default)]
    external_or_macro_calls: Vec<String>,
}
#[derive(Deserialize)]
struct Exercise {
    schema_version: u32,
    status: String,
    accepted: bool,
    frames: u64,
    source: Source,
    artifacts: BTreeMap<String, String>,
    rom_sha256: String,
    inputs_sha256: String,
    original_sm86_sha256: String,
    host_sha256: String,
    build_command: Vec<String>,
    replay_command: Vec<String>,
}
#[derive(Clone, Debug)]
struct Hit {
    address: u32,
    count: u64,
    first: u64,
    kind: String,
}
#[derive(Clone, Debug)]
struct Ticket {
    path: PathBuf,
    text: String,
    number: u64,
    status: String,
    triage: Option<String>,
    kind: Option<String>,
    blockers: Vec<PathBuf>,
    missing_blockers: bool,
}

fn relative(path: &Path) -> bool {
    !path.as_os_str().is_empty() && path.components().all(|c| matches!(c, Component::Normal(_)))
}
fn resolve(checkout: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() { path.into() } else { checkout.join(path) }
}
fn document<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let metadata = fs::symlink_metadata(path)?;
    ensure!(
        metadata.is_file() && !metadata.file_type().is_symlink(),
        "document must be a regular file: {}",
        path.display()
    );
    ensure!(metadata.len() <= MAX_DOCUMENT_BYTES, "planning document is too large: {}", path.display());
    Ok(toml::from_str(&fs::read_to_string(path)?)?)
}
fn metadata<'a>(text: &'a str, name: &str) -> Option<&'a str> {
    text.lines().take_while(|line| !line.starts_with("## ")).find_map(|line| line.strip_prefix(name).map(str::trim))
}
fn finished(status: &str) -> bool {
    matches!(status, "resolved" | "done")
}

fn tickets(checkout: &Path) -> Result<Vec<Ticket>> {
    let scratch = checkout.join(".scratch");
    if !scratch.is_dir() {
        return Ok(vec![]);
    }
    let mut paths = Vec::new();
    let mut efforts = fs::read_dir(scratch)?.map(|e| Ok(e?.path())).collect::<Result<Vec<_>>>()?;
    efforts.sort();
    for effort in efforts {
        if effort.is_symlink() || !effort.is_dir() {
            continue;
        }
        let issues = effort.join("issues");
        if !issues.is_dir() || issues.is_symlink() {
            continue;
        }
        for entry in fs::read_dir(issues)? {
            let path = entry?.path();
            if path.extension().is_some_and(|e| e == "md") && !path.is_symlink() {
                paths.push(path);
            }
            ensure!(paths.len() <= 10_000, "local ticket discovery exceeds 10000 files");
        }
    }
    paths.sort();
    let mut result = Vec::new();
    for path in &paths {
        ensure!(fs::metadata(path)?.len() <= MAX_DOCUMENT_BYTES, "ticket is too large");
        let text = fs::read_to_string(path)?;
        let Some(number) =
            path.file_name().and_then(|p| p.to_str()).and_then(|s| s.split('-').next()).and_then(|s| s.parse().ok())
        else {
            continue;
        };
        let status = metadata(&text, "Status:").unwrap_or("unknown").to_owned();
        let triage = metadata(&text, "Triage:").map(str::to_owned);
        let kind = metadata(&text, "Type:").map(str::to_owned);
        let mut blockers = Vec::new();
        let mut missing_blockers = false;
        if let Some(value) = metadata(&text, "Blocked by:") {
            for number in value.split(',').map(str::trim).filter(|s| !s.is_empty() && *s != "None") {
                let matches: Vec<_> = paths
                    .iter()
                    .filter(|p| {
                        p.parent() == path.parent()
                            && p.file_name().and_then(|p| p.to_str()).is_some_and(|s| {
                                s.split('-').next().and_then(|n| n.parse::<u64>().ok()) == number.parse().ok()
                            })
                    })
                    .collect();
                if matches.len() == 1 {
                    blockers.push(matches[0].strip_prefix(checkout)?.into());
                } else {
                    missing_blockers = true;
                }
            }
        } else {
            missing_blockers = true;
        }
        result.push(Ticket {
            path: path.strip_prefix(checkout)?.into(),
            text,
            number,
            status,
            triage,
            kind,
            blockers,
            missing_blockers,
        });
    }
    result.sort_by(|a, b| a.number.cmp(&b.number).then(a.path.cmp(&b.path)));
    Ok(result)
}
fn ready(ticket: &Ticket, all: &[Ticket]) -> bool {
    ticket.status == "open"
        && ticket.triage.as_deref().is_none_or(|s| s == "ready-for-agent")
        && !ticket.missing_blockers
        && ticket.blockers.iter().all(|p| all.iter().any(|t| &t.path == p && finished(&t.status)))
        && ticket.kind.as_deref().is_none_or(|s| matches!(s, "task" | "research"))
}
fn job(
    id: String,
    subject: String,
    inputs: BTreeMap<String, String>,
    instructions: String,
    ticket: Option<PathBuf>,
) -> Result<Job> {
    let revision = hash(&toml::to_string(&inputs)?);
    Ok(Job {
        id,
        revision,
        kind: Kind::Research,
        subject,
        writable_paths: vec![],
        blockers: vec![],
        inputs,
        instructions,
        ticket,
    })
}
fn file_key(path: &Path) -> String {
    format!("file:{}", path.display())
}
fn hash(text: &str) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(text.as_bytes()))
}
fn c_code(text: &str) -> Vec<u8> {
    let bytes = text.as_bytes();
    let mut result = bytes.to_vec();
    let mut index = 0;
    while index < bytes.len() {
        let end = if bytes[index..].starts_with(b"/*") {
            bytes[index + 2..].windows(2).position(|v| v == b"*/").map(|p| index + p + 4).unwrap_or(bytes.len())
        } else if bytes[index..].starts_with(b"//") {
            bytes[index..].iter().position(|v| *v == b'\n').map(|p| index + p).unwrap_or(bytes.len())
        } else if b"\"'".contains(&bytes[index]) {
            let quote = bytes[index];
            let mut cursor = index + 1;
            while cursor < bytes.len() {
                if bytes[cursor] == b'\\' {
                    cursor += 2;
                } else if bytes[cursor] == quote {
                    cursor += 1;
                    break;
                } else {
                    cursor += 1;
                }
            }
            cursor.min(bytes.len())
        } else {
            index += 1;
            continue;
        };
        for byte in &mut result[index..end] {
            if *byte != b'\n' {
                *byte = b' ';
            }
        }
        index = end;
    }
    result
}
fn inventory(checkout: &Path, path: &Path) -> Result<Inventory> {
    let value: Inventory = document(path)?;
    ensure!(value.schema_version == 1 && !value.routines.is_empty(), "unsupported or empty source inventory");
    ensure!(
        crate::digest(&checkout.join("tools/dependencies.toml"))? == value.source_identity_sha256,
        "source inventory dependency pin changed"
    );
    let identity = crate::family::verify_source(checkout, &checkout.join("deps/snesrev-sm"))?;
    ensure!(value.source_identity == identity, "source inventory pinned source differs");
    let mut symbols = BTreeSet::new();
    let mut source_text = BTreeMap::new();
    for routine in &value.routines {
        ensure!(
            crate::family::identifier(&routine.symbol)
                && relative(Path::new(&routine.source))
                && symbols.insert(&routine.symbol),
            "invalid or duplicate source inventory routine"
        );
        ensure!(identity.files.contains_key(&routine.source), "routine source missing from pinned inventory");
        if !source_text.contains_key(&routine.source) {
            let text = fs::read_to_string(checkout.join("deps/snesrev-sm").join(&routine.source))?;
            let code = c_code(&text);
            source_text.insert(routine.source.clone(), (text, code));
        }
        let (text, code) = &source_text[&routine.source];
        ensure!(routine.line > 0, "routine source line missing");
        let offset = text.split_inclusive('\n').take(routine.line - 1).map(str::len).sum::<usize>();
        let start = text.get(offset..).context("routine source line outside pinned file")?;
        let clean = &code[offset..];
        let open = clean.iter().position(|c| *c == b'{').context("routine definition body missing")?;
        ensure!(
            start[..open].contains(&format!("{}(", routine.symbol))
                || start[..open].contains(&format!("{} (", routine.symbol)),
            "routine symbol differs from pinned definition"
        );
        let mut depth = 1;
        let mut end = open + 1;
        // The inventory producer hashes the complete C definition. A supplied
        // inventory cannot relabel another definition's observed address.
        while end < start.len() && depth > 0 {
            match clean[end] {
                b'{' => depth += 1,
                b'}' => depth -= 1,
                _ => {}
            }
            end += 1;
        }
        ensure!(depth == 0 && hash(&start[..end]) == routine.definition_sha256, "routine definition identity differs");
        if let Some(address) = address(routine) {
            let opening_line = start[open..].lines().next().unwrap_or_default();
            ensure!(
                opening_line.to_ascii_lowercase().contains(&format!("0x{address:06x}")),
                "routine observed address differs from pinned definition"
            );
        }
    }
    Ok(value)
}
fn address(routine: &Routine) -> Option<u32> {
    let value = routine.original_address.as_deref()?;
    u32::from_str_radix(value.strip_prefix("0x").unwrap_or(value), 16).ok()
}
fn exercise(checkout: &Path, path: &Path, source: &Source) -> Result<Vec<Hit>> {
    let value: Exercise = document(path)?;
    ensure!(
        value.schema_version == 2
            && value.status == "diagnostic-complete"
            && !value.accepted
            && value.frames > 0
            && value.frames <= 129712,
        "native exercise diagnostic is incomplete or unsupported"
    );
    ensure!(&value.source == source, "exercise source differs from pinned inventory");
    ensure!(
        value.rom_sha256 == ROM_SHA256 && crate::digest(&checkout.join("rom/sm.sfc"))? == value.rom_sha256,
        "exercise ROM identity differs"
    );
    ensure!(
        crate::digest(&checkout.join("movies/super-metroid-any-3653.inputs"))? == value.inputs_sha256,
        "exercise movie identity differs"
    );
    ensure!(
        source.files.get("src/sm_86.c") == Some(&value.original_sm86_sha256),
        "exercise original bank86 source differs"
    );
    ensure!(
        crate::digest(&checkout.join("tools/decomp/headless.c"))? == value.host_sha256,
        "exercise host source differs"
    );
    let run = fs::canonicalize(path.parent().context("exercise parent missing")?)?;
    for required in ["projectiles.tsv", "sm_86.c", "headless.c", "inventory", "original-source/sm_86.c"] {
        let expected =
            value.artifacts.get(required).with_context(|| format!("exercise artifact seal missing: {required}"))?;
        let artifact = run.join(required);
        ensure!(
            !artifact.is_symlink() && crate::digest(&artifact)? == *expected,
            "exercise artifact changed: {required}"
        );
    }
    ensure!(
        value.artifacts["original-source/sm_86.c"] == value.original_sm86_sha256,
        "exercise original snapshot differs"
    );
    ensure!(
        value.build_command.iter().any(|s| Path::new(s) == run.join("sm_86.c"))
            && value.build_command.iter().any(|s| Path::new(s) == run.join("headless.c")),
        "exercise build does not bind instrumented sources"
    );
    ensure!(
        value.replay_command.len() == 5
            && Path::new(&value.replay_command[0]) == run.join("inventory")
            && value.replay_command[3].parse::<u64>()? == value.frames,
        "exercise replay identity differs"
    );
    ensure!(
        crate::digest(Path::new(&value.replay_command[1]))? == value.rom_sha256
            && crate::digest(Path::new(&value.replay_command[2]))? == value.inputs_sha256,
        "exercise replay inputs changed"
    );
    let table = fs::read_to_string(run.join("projectiles.tsv"))?;
    ensure!(
        table.lines().next() == Some("kind\taddress\thits\tfirst_frame\tlast_frame"),
        "exercise table header differs"
    );
    let mut result = Vec::new();
    let mut seen = BTreeSet::new();
    for line in table.lines().skip(1) {
        let values: Vec<_> = line.split('\t').collect();
        ensure!(
            values.len() == 5 && ["id", "init", "pre", "instruction"].contains(&values[0]),
            "exercise counter is malformed"
        );
        let low = u16::from_str_radix(values[1], 16)?;
        let count = values[2].parse::<u64>()?;
        let first = values[3].parse::<u64>()?;
        let last = values[4].parse::<u64>()?;
        ensure!(
            count > 0 && first <= last && last < value.frames && seen.insert((values[0], low)),
            "exercise counter bounds or uniqueness differ"
        );
        if values[0] != "id" {
            result.push(Hit { address: 0x860000 | u32::from(low), count, first, kind: values[0].into() });
        }
    }
    ensure!(!result.is_empty(), "exercise contains no routine execution");
    Ok(result)
}
fn default_exercise(checkout: &Path) -> Result<Option<PathBuf>> {
    let root = checkout.join("out/port/projectile-inventory");
    if !root.is_dir() {
        return Ok(None);
    }
    let mut paths = Vec::new();
    for (index, entry) in fs::read_dir(root)?.take(129).enumerate() {
        ensure!(index < 128, "exercise discovery exceeds 128 runs; specify an explicit native verdict");
        let path = entry?.path();
        if path.is_dir() && !path.is_symlink() && path.join("verdict.toml").is_file() {
            paths.push(path.join("verdict.toml"));
        }
    }
    paths.sort();
    Ok(paths.pop())
}

pub(crate) fn discover(checkout: &Path, source_inventory: Option<&Path>, observed: Option<&Path>) -> Result<Vec<Job>> {
    let checkout = fs::canonicalize(checkout)?;
    let all = tickets(&checkout)?;
    let mut jobs = Vec::new();
    for ticket in all.iter().filter(|t| ready(t, &all)).take(MAX_JOBS) {
        let subject =
            ticket.text.lines().find_map(|s| s.strip_prefix("# ")).unwrap_or("Existing local work").to_owned();
        let semantic = ticket
            .text
            .lines()
            .filter(|line| !line.starts_with("Status:") && !line.starts_with("<!-- assignment:"))
            .collect::<Vec<_>>()
            .join("\n");
        let inputs = BTreeMap::from([("ticket-semantic-sha256".into(), hash(&semantic))]);
        jobs.push(job(format!("ticket-{}",hash(&ticket.path.display().to_string())), subject, inputs,
            format!("Honor existing local ticket {}. It is open, unclaimed and dependency-ready. Read its requirements and establish a complete Translation scope before implementation: observed exercise provenance, required dependencies, exact writable paths and concrete verification argv. Use native translation-scope-v1 TOML; limit each initial experiment to approximately 60 seconds. Research facts do not grant acceptance.", ticket.path.display()), Some(ticket.path.clone()))?);
    }
    if jobs.len() == MAX_JOBS {
        return Ok(jobs);
    }
    let inventory_path = resolve(&checkout, source_inventory.unwrap_or(Path::new("tools/port/source-inventory.toml")));
    let observed_path = match observed {
        Some(path) => Some(resolve(&checkout, path)),
        None => default_exercise(&checkout)?,
    };
    let loaded = inventory(&checkout, &inventory_path);
    let hits =
        loaded.as_ref().ok().and_then(|i| observed_path.as_ref().map(|p| exercise(&checkout, p, &i.source_identity)));
    if let (Ok(loaded), Some(Ok(hits))) = (&loaded, &hits) {
        let mut selected: Vec<_> = loaded
            .routines
            .iter()
            .filter_map(|r| hits.iter().find(|h| Some(h.address) == address(r)).map(|h| (r, h)))
            .collect();
        selected.sort_by(|(a, ah), (b, bh)| {
            a.defined_dependencies
                .len()
                .cmp(&b.defined_dependencies.len())
                .then(bh.count.cmp(&ah.count))
                .then(ah.first.cmp(&bh.first))
                .then(a.symbol.cmp(&b.symbol))
        });
        let mut covered = BTreeSet::new();
        for (routine, hit) in selected {
            if jobs.len() >= MAX_JOBS {
                break;
            }
            if !covered.insert(&routine.symbol) || all.iter().any(|t| mentions(&t.text, routine)) {
                continue;
            }
            let related: Vec<_> = routine.defined_dependencies.iter().take(8).cloned().collect();
            for dependency in &related {
                if let Some(dependency) = loaded.routines.iter().find(|r| &r.symbol == dependency) {
                    covered.insert(&dependency.symbol);
                }
            }
            let mut inputs = BTreeMap::from([
                (file_key(&inventory_path), crate::digest(&inventory_path)?),
                (file_key(observed_path.as_ref().unwrap()), crate::digest(observed_path.as_ref().unwrap())?),
            ]);
            inputs.insert(
                file_key(Path::new(&format!("deps/snesrev-sm/{}", routine.source))),
                loaded.source_identity.files[&routine.source].clone(),
            );
            jobs.push(job(format!("translation-research-{}",routine.symbol), format!("{} caller and dependency scope",routine.symbol), inputs,
                format!("Research the observed {} at {:#08x} in {} ({} {} calls; first frame {}). Ranked by fewer lexical dependencies, then observed calls and first exercise. Bundle coherent caller/dependency behavior with {}. Indirect calls {:?} and external/macro calls {:?} need review. Check maintained Rust and existing evidence before declaring a gap: inventory translation_status is not translation proof. Establish native translation-scope-v1 findings with execution provenance, complete dependencies, safe exact writable ownership and concrete verification argv; initial experiments approximately 60 seconds. Do not infer caller hit counts from a callee counter or grant acceptance.",routine.symbol,hit.address,routine.source,hit.count,hit.kind,hit.first,related.join(", "),routine.indirect_indexed_calls,routine.external_or_macro_calls),None)?);
        }
    } else {
        let reason = match (&loaded, &hits) {
            (Err(e), _) => format!("Source inventory unavailable or stale: {e:#}"),
            (_, Some(Err(e))) => format!("Observed exercise provenance unavailable: {e:#}"),
            _ => "No native projectile exercise verdict is available; historical JSON is not a native scope receipt"
                .into(),
        };
        jobs.push(job("translation-research-establish-inputs".into(),"Establish translation exercise and scope inputs".into(), BTreeMap::from([(file_key(Path::new("tools/dependencies.toml")),crate::digest(&checkout.join("tools/dependencies.toml"))?)]), format!("{reason}. Establish pinned source inventory and naturally exercised coherent gameplay candidates. Existing claimed/resolved tickets remain authoritative; preserve historical evidence. Use existing Rust source-inventory and projectile-inventory commands as appropriate; start with bounded approximately 60-second experiments. Missing observations remain research, never implementation readiness."),None)?);
    }
    Ok(jobs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        process::Command,
        sync::atomic::{AtomicUsize, Ordering},
    };
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Project(PathBuf);
    impl Drop for Project {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn write(root: &Path, path: &str, data: impl AsRef<[u8]>) {
        let path = root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, data).unwrap();
    }
    fn fresh() -> Project {
        let root = std::env::temp_dir().join(format!(
            "sm-translation-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        write(&root, "tools/dependencies.toml", "schema_version = 1\n");
        Project(root)
    }
    fn ticket(root: &Path, number: u64, status: &str, blockers: &str, triage: &str) {
        write(
            root,
            &format!(".scratch/work/issues/{number:02}-work.md"),
            format!(
                "# Work {number}\nStatus: {status}\nBlocked by: {blockers}\nTriage: {triage}\nType: task\n\nResearch first.\n"
            ),
        );
    }
    #[test]
    fn local_work_honors_claims_blockers_triage_and_order_before_new_research() {
        let project = fresh();
        ticket(&project.0, 1, "done", "None", "ready-for-agent");
        ticket(&project.0, 2, "claimed", "None", "ready-for-agent");
        ticket(&project.0, 3, "open", "2", "ready-for-agent");
        ticket(&project.0, 4, "open", "1", "ready-for-agent");
        ticket(&project.0, 5, "open", "None", "ready-for-human");
        ticket(&project.0, 6, "open", "99", "ready-for-agent");
        let jobs = discover(&project.0, None, None).unwrap();
        assert_eq!(jobs.len(), 2);
        assert_eq!(jobs[0].ticket, Some(".scratch/work/issues/04-work.md".into()));
        assert_eq!(jobs[0].kind, Kind::Research);
        assert!(jobs[0].writable_paths.is_empty());
        assert!(jobs[1].instructions.contains("Source inventory unavailable"));
        assert!(jobs.iter().all(|j| !j.id.contains(':')));
        assert_eq!(jobs, discover(&project.0, None, None).unwrap());
    }
    #[test]
    fn discovery_and_scope_reading_are_bounded_and_do_not_follow_ticket_symlinks() {
        let project = fresh();
        for number in 1..=12 {
            ticket(&project.0, number, "open", "None", "ready-for-agent");
        }
        let jobs = discover(&project.0, None, None).unwrap();
        assert_eq!(jobs.len(), MAX_JOBS);
        assert!(jobs.iter().all(|j| j.ticket.is_some()));
        let root = project.0.join("out/port/projectile-inventory");
        for number in 0..129 {
            fs::create_dir_all(root.join(number.to_string())).unwrap();
        }
        assert!(default_exercise(&project.0).unwrap_err().to_string().contains("128 runs"));
    }
    fn pinned_inventory(root: &Path) -> Source {
        write(
            root,
            "deps/snesrev-sm/src/sm_86.c",
            "void Observed(void) { /* 0x8693ca */ Helper(); }\nvoid Helper(void) {}\n",
        );
        write(root, "deps/snesrev-sm/LICENSE.txt", "test pinned source\n");
        fs::create_dir_all(root.join("deps/downloads")).unwrap();
        let archive = root.join("deps/downloads/oracle.tar.gz");
        assert!(
            Command::new("tar")
                .arg("-czf")
                .arg(&archive)
                .arg("-C")
                .arg(root.join("deps"))
                .arg("snesrev-sm")
                .status()
                .unwrap()
                .success()
        );
        write(
            root,
            "tools/dependencies.toml",
            format!(
                "schema_version = 1\n[[dependencies]]\nname = \"snesrev-sm\"\narchive = \"oracle.tar.gz\"\nrevision = \"test-source\"\nsha256 = {:?}\n",
                crate::digest(&archive).unwrap()
            ),
        );
        let source = crate::family::verify_source(root, &root.join("deps/snesrev-sm")).unwrap();
        let mut value = toml::Value::Table(toml::Table::new());
        value["schema_version"] = 1.into();
        value["source_identity"] = toml::Value::try_from(&source).unwrap();
        value["source_identity_sha256"] = crate::digest(&root.join("tools/dependencies.toml")).unwrap().into();
        value["routines"] = toml::Value::Array(vec![
            toml::Value::Table(toml::Table::from_iter([
                ("symbol".into(), "Observed".into()),
                ("source".into(), "src/sm_86.c".into()),
                ("line".into(), 1.into()),
                ("definition_sha256".into(), hash("void Observed(void) { /* 0x8693ca */ Helper(); }").into()),
                ("original_address".into(), "0x8693ca".into()),
                ("defined_dependencies".into(), toml::Value::try_from(vec!["Helper"]).unwrap()),
            ])),
            toml::Value::Table(toml::Table::from_iter([
                ("symbol".into(), "Helper".into()),
                ("source".into(), "src/sm_86.c".into()),
                ("line".into(), 2.into()),
                ("definition_sha256".into(), hash("void Helper(void) {}").into()),
            ])),
        ]);
        write(root, "inventory.toml", toml::to_string(&value).unwrap());
        source
    }
    #[test]
    fn stale_inventory_and_missing_exercise_remain_research_without_invented_admission() {
        let project = fresh();
        pinned_inventory(&project.0);
        let jobs = discover(&project.0, Some(Path::new("inventory.toml")), None).unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].kind, Kind::Research);
        assert!(jobs[0].instructions.contains("No native projectile exercise verdict"));
        write(&project.0, "deps/snesrev-sm/src/sm_86.c", "changed\n");
        let jobs = discover(&project.0, Some(Path::new("inventory.toml")), None).unwrap();
        assert_eq!(jobs[0].kind, Kind::Research);
        assert!(jobs[0].instructions.contains("stale"));
    }
    #[test]
    fn native_scope_requires_every_mandatory_fact_before_promotion() {
        let project = fresh();
        write(&project.0, "scope.toml", "format = \"translation-scope-v1\"\nsubject = \"Incomplete\"\n");
        assert!(validate_scope(&project.0, Path::new("scope.toml")).is_err());
        let scope = Scope {
            format: "translation-scope-v1".into(),
            subject: "Empty".into(),
            source_inventory: "inventory.toml".into(),
            source_inventory_sha256: "missing".into(),
            exercise: "exercise.toml".into(),
            exercise_sha256: "missing".into(),
            routines: vec![],
            dependencies: vec![],
            dependency_review: BTreeMap::new(),
            writable_paths: vec![],
            inputs: BTreeMap::new(),
            verification: vec![],
        };
        write(&project.0, "scope.toml", toml::to_string(&scope).unwrap());
        assert!(validate_scope(&project.0, Path::new("scope.toml")).unwrap_err().to_string().contains("1..32"));
    }
    #[test]
    fn routine_and_address_mentions_preserve_existing_work_ownership() {
        let routine = Routine {
            symbol: "Observed".into(),
            source: "src/sm_86.c".into(),
            line: 1,
            definition_sha256: "unused".into(),
            original_address: Some("0x8693ca".into()),
            defined_dependencies: vec![],
            indirect_indexed_calls: vec![],
            external_or_macro_calls: vec![],
        };
        assert!(mentions("Implemented `Observed`", &routine));
        assert!(mentions("Status: claimed\nTranslate 8693CA", &routine));
        assert!(!mentions("ObservedExtra", &routine));
    }
}
fn mentions(text: &str, routine: &Routine) -> bool {
    text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).any(|word| word == routine.symbol)
        || address(routine).is_some_and(|a| text.to_ascii_lowercase().contains(&format!("{a:06x}")))
}

pub(crate) fn validate_scope(checkout: &Path, path: &Path) -> Result<Job> {
    let checkout = fs::canonicalize(checkout)?;
    let path = resolve(&checkout, path);
    let scope: Scope = document(&path)?;
    ensure!(
        scope.format == "translation-scope-v1" && !scope.subject.trim().is_empty(),
        "unsupported or unnamed Translation scope"
    );
    ensure!(
        !scope.routines.is_empty() && scope.routines.len() <= 32,
        "Translation scope must identify 1..32 coherent routines"
    );
    let inventory_path = resolve(&checkout, &scope.source_inventory);
    let exercise_path = resolve(&checkout, &scope.exercise);
    ensure!(
        crate::digest(&inventory_path)? == scope.source_inventory_sha256
            && crate::digest(&exercise_path)? == scope.exercise_sha256,
        "research input receipt changed"
    );
    let loaded = inventory(&checkout, &inventory_path)?;
    let observed = exercise(&checkout, &exercise_path, &loaded.source_identity)?;
    let mut routines = BTreeSet::new();
    let mut has_observed = false;
    for symbol in &scope.routines {
        ensure!(routines.insert(symbol), "duplicate Translation scope routine");
        let routine = loaded
            .routines
            .iter()
            .find(|r| &r.symbol == symbol)
            .with_context(|| format!("scope routine missing from pinned inventory: {symbol}"))?;
        has_observed |= observed.iter().any(|h| Some(h.address) == address(routine));
        let source = format!("deps/snesrev-sm/{}", routine.source);
        ensure!(
            scope.inputs.get(&source) == loaded.source_identity.files.get(&routine.source),
            "scope source input seal missing: {source}"
        );
        for dependency in routine
            .defined_dependencies
            .iter()
            .chain(&routine.indirect_indexed_calls)
            .chain(&routine.external_or_macro_calls)
        {
            if scope.routines.contains(dependency) {
                continue;
            }
            let review = scope
                .dependency_review
                .get(dependency)
                .with_context(|| format!("scope dependency review missing: {symbol} -> {dependency}"))?;
            ensure!(
                !review.rationale.trim().is_empty() && !review.evidence.is_empty(),
                "scope dependency needs rationale and evidence: {dependency}"
            );
            for evidence in &review.evidence {
                ensure!(
                    relative(evidence)
                        && scope.inputs.get(&evidence.to_string_lossy().into_owned())
                            == Some(&crate::digest(&checkout.join(evidence))?),
                    "scope dependency evidence input missing or changed: {}",
                    evidence.display()
                );
            }
        }
    }
    ensure!(has_observed, "Translation scope has no directly observed routine; lexical callers do not prove exercise");
    // Unobserved helpers may form part of an exercised root's coherent closure.
    let mut reachable: BTreeSet<String> = scope
        .routines
        .iter()
        .filter(|symbol| {
            loaded
                .routines
                .iter()
                .any(|r| &r.symbol == *symbol && observed.iter().any(|h| Some(h.address) == address(r)))
        })
        .cloned()
        .collect();
    loop {
        let before = reachable.len();
        for routine in &loaded.routines {
            if reachable.contains(&routine.symbol) {
                reachable.extend(routine.defined_dependencies.iter().filter(|d| scope.routines.contains(d)).cloned());
            }
        }
        if before == reachable.len() {
            break;
        }
    }
    ensure!(
        scope.routines.iter().all(|symbol| reachable.contains(symbol)),
        "scope includes unrelated or unobserved routines outside the exercised dependency closure"
    );
    let all = tickets(&checkout)?;
    for dependency in &scope.dependencies {
        ensure!(
            relative(dependency) && dependency.starts_with(".scratch"),
            "scope dependency must name a local ticket"
        );
        let ticket = all.iter().find(|t| &t.path == dependency).context("scope dependency ticket missing")?;
        ensure!(finished(&ticket.status), "scope dependency is not resolved: {}", dependency.display());
        ensure!(
            scope.inputs.get(&dependency.to_string_lossy().into_owned())
                == Some(&crate::digest(&checkout.join(dependency))?),
            "scope dependency input seal missing"
        );
    }
    ensure!(!scope.writable_paths.is_empty(), "scope writable ownership missing");
    let mut writable = BTreeSet::new();
    for target in &scope.writable_paths {
        ensure!(relative(target) && writable.insert(target), "unsafe or duplicate writable path");
        ensure!(
            target.starts_with("crates/port/src")
                || target.starts_with("crates/port/tests")
                || target.starts_with("tools/port"),
            "translation writes must name gameplay source, tests or family matrices"
        );
        ensure!(
            target.extension().is_some_and(|e| e == "rs" || e == "toml"),
            "writable ownership requires exact source/test/manifest files"
        );
        for ancestor in checkout.join(target).ancestors().take_while(|p| *p != checkout) {
            ensure!(!ancestor.is_symlink(), "writable path crosses a symlink");
        }
        if checkout.join(target).exists() {
            ensure!(
                scope.inputs.get(&target.to_string_lossy().into_owned())
                    == Some(&crate::digest(&checkout.join(target))?),
                "current writable source input seal missing: {}",
                target.display()
            );
        }
    }
    ensure!(!scope.inputs.is_empty(), "scope input seals missing");
    for (name, expected) in &scope.inputs {
        ensure!(
            relative(Path::new(name)) && crate::digest(&checkout.join(name))? == *expected,
            "scope input changed: {name}"
        );
    }
    ensure!(!scope.verification.is_empty(), "scope verification commands missing");
    ensure!(scope.verification.iter().any(|v| !v.integration), "scope needs bounded pre-integration verification");
    for verification in &scope.verification {
        ensure!(
            !verification.argv.is_empty()
                && verification.argv.iter().all(|v| !v.is_empty() && !v.contains(['\n', '\r', '\0'])),
            "scope verification argv malformed"
        );
        ensure!(
            ["cargo", "sm-pipeline"].contains(&verification.argv[0].as_str()),
            "scope verification must use existing Rust tooling"
        );
        ensure!(
            verification.timeout_seconds > 0 && (verification.integration || verification.timeout_seconds <= 60),
            "initial verification must be bounded at 60 seconds"
        );
        ensure!(
            verification
                .argv
                .iter()
                .any(|a| ["test", "family-verify", "batch", "live-verify", "routine-live"].contains(&a.as_str())),
            "scope command does not declare verification"
        );
    }
    let mut inputs: BTreeMap<_, _> =
        scope.inputs.iter().map(|(key, value)| (file_key(Path::new(key)), value.clone())).collect();
    inputs.insert(file_key(&path), crate::digest(&path)?);
    inputs.insert("scope-file".into(), path.display().to_string());
    inputs.insert("scope-sha256".into(), crate::digest(&path)?);
    inputs.insert(file_key(&inventory_path), scope.source_inventory_sha256.clone());
    inputs.insert(file_key(&exercise_path), scope.exercise_sha256.clone());
    let mut identity_routines = scope.routines.clone();
    identity_routines.sort();
    let id = format!("translation-implementation-{}", hash(&identity_routines.join("\n")));
    let revision = hash(&toml::to_string(&scope)?);
    let verification = scope
        .verification
        .iter()
        .map(|v| format!("{:?} ({}s, integration={})", v.argv, v.timeout_seconds, v.integration))
        .collect::<Vec<_>>()
        .join("; ");
    Ok(Job {
        id,
        revision,
        kind: Kind::Implement,
        subject: scope.subject,
        writable_paths: scope.writable_paths,
        blockers: vec![],
        inputs,
        instructions: format!(
            "Implement the researched Translation scope in {} using an isolated worktree and independent batch record. Honor exact writable paths. Required verification argv: {}. Existing current-source authentication grants admission; research does not. Submission requires serialized integration, merged-source verification after freeze and scoped cleanup before resolution.",
            path.display(),
            verification
        ),
        ticket: None,
    })
}
