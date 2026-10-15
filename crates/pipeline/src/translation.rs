//! Bounded translation research discovery; planning facts never grant gameplay admission.
use crate::assignment::{Job, Kind};
use anyhow::{Context, Result, ensure};
use decomp_source::Source;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
};

const MAX_JOBS: usize = 8;
const MAX_DOCUMENT_BYTES: u64 = 32 * 1024 * 1024;
const MAX_INTEGRATION_GATE_SECONDS: u64 = 3600;
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
    #[serde(default)]
    pub origin_ticket: Option<PathBuf>,
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
    pub ready: bool,
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
#[derive(Deserialize, Serialize)]
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

fn identifier(symbol: &str) -> bool {
    let mut chars = symbol.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
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
    Ok(toml::from_str(&tool_execution::bounded::text(path)?)?)
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
    let efforts = tool_execution::bounded::entries(&scratch)?;
    for effort in efforts {
        if effort.is_symlink() || !effort.is_dir() {
            continue;
        }
        let issues = effort.join("issues");
        if !issues.is_dir() || issues.is_symlink() {
            continue;
        }
        for path in tool_execution::bounded::entries(&issues)? {
            if path.extension().is_some_and(|e| e == "md") && !path.is_symlink() {
                paths.push(path);
            }
            ensure!(paths.len() <= 10_000, "local ticket discovery exceeds 10000 files");
        }
    }
    paths.sort();
    let mut result = Vec::new();
    for path in &paths {
        tool_execution::bounded::check()?;
        ensure!(fs::metadata(path)?.len() <= MAX_DOCUMENT_BYTES, "ticket is too large");
        let text = tool_execution::bounded::text(path)?;
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
fn inventory(checkout: &Path, path: &Path) -> Result<Inventory> {
    let value: Inventory = document(path)?;
    ensure!(value.schema_version == 1 && !value.routines.is_empty(), "unsupported or empty source inventory");
    ensure!(
        tool_execution::bounded::digest(&checkout.join("tools/dependencies.toml"))? == value.source_identity_sha256,
        "source inventory dependency pin changed"
    );
    let identity = decomp_source::verify_source(checkout, &checkout.join("deps/snesrev-sm"))?;
    ensure!(value.source_identity == identity, "source inventory pinned source differs");
    let mut symbols = BTreeSet::new();
    ensure!(value.routines.len() <= 65536, "scope source routine inventory exceeded");
    let mut source_text = BTreeMap::new();
    let all_symbols: BTreeSet<_> = value.routines.iter().map(|r| r.symbol.as_str()).collect();
    for routine in &value.routines {
        tool_execution::bounded::check()?;
        ensure!(
            identifier(&routine.symbol) && relative(Path::new(&routine.source)) && symbols.insert(&routine.symbol),
            "invalid or duplicate source inventory routine"
        );
        ensure!(identity.files.contains_key(&routine.source), "routine source missing from pinned inventory");
        if !source_text.contains_key(&routine.source) {
            let text = tool_execution::bounded::text(checkout.join("deps/snesrev-sm").join(&routine.source))?;
            source_text.insert(routine.source.clone(), decomp_source::DefinitionSource::new(text)?);
        }
        let definition = source_text[&routine.source].at_line(routine.line)?;
        ensure!(definition.symbol == routine.symbol, "routine symbol differs from pinned definition");
        ensure!(definition.sha256 == routine.definition_sha256, "routine definition identity differs");
        let calls = definition.calls;
        let indexed = definition.indirect_indexed_calls;
        let defined: Vec<_> = calls.iter().filter(|s| all_symbols.contains(s.as_str())).cloned().collect();
        let external: Vec<_> = calls.iter().filter(|s| !all_symbols.contains(s.as_str())).cloned().collect();
        ensure!(
            defined == routine.defined_dependencies
                && external == routine.external_or_macro_calls
                && indexed == routine.indirect_indexed_calls,
            "routine lexical dependency inventory differs from pinned source"
        );
        if let Some(address) = address(routine) {
            let opening_line = definition.opening_line;
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
        value.rom_sha256 == ROM_SHA256
            && tool_execution::bounded::digest(&checkout.join("rom/sm.sfc"))? == value.rom_sha256,
        "exercise ROM identity differs"
    );
    ensure!(
        tool_execution::bounded::digest(&checkout.join("movies/super-metroid-any-3653.inputs"))? == value.inputs_sha256,
        "exercise movie identity differs"
    );
    ensure!(
        source.files.get("src/sm_86.c") == Some(&value.original_sm86_sha256),
        "exercise original bank86 source differs"
    );
    ensure!(
        tool_execution::bounded::digest(&checkout.join("tools/decomp/headless.c"))? == value.host_sha256,
        "exercise host source differs"
    );
    let run = fs::canonicalize(path.parent().context("exercise parent missing")?)?;
    for required in
        ["projectiles.tsv", "sm_86.c", "headless.c", "headless_frame.h", "inventory", "original-source/sm_86.c"]
    {
        let expected =
            value.artifacts.get(required).with_context(|| format!("exercise artifact seal missing: {required}"))?;
        let artifact = run.join(required);
        ensure!(
            !artifact.is_symlink() && tool_execution::bounded::digest(&artifact)? == *expected,
            "exercise artifact changed: {required}"
        );
    }
    ensure!(
        value.artifacts["original-source/sm_86.c"] == value.original_sm86_sha256,
        "exercise original snapshot differs"
    );
    crate::projectile_inventory::authenticate_inputs(checkout, &run, &value.build_command, source)?;
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
        tool_execution::bounded::digest(Path::new(&value.replay_command[1]))? == value.rom_sha256
            && tool_execution::bounded::digest(Path::new(&value.replay_command[2]))? == value.inputs_sha256,
        "exercise replay inputs changed"
    );
    let table = tool_execution::bounded::text(run.join("projectiles.tsv"))?;
    ensure!(
        table.lines().next() == Some("kind\taddress\thits\tfirst_frame\tlast_frame"),
        "exercise table header differs"
    );
    let mut result = Vec::new();
    let mut seen = BTreeSet::new();
    for line in table.lines().skip(1) {
        tool_execution::bounded::check()?;
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
        tool_execution::bounded::check()?;
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
        let origin_type = ticket.kind.as_deref().unwrap_or("task");
        let inputs = BTreeMap::from([
            ("ticket-semantic-sha256".into(), hash(&semantic)),
            ("origin-ticket-type".into(), origin_type.into()),
            ("origin-ticket".into(), ticket.path.display().to_string()),
        ]);
        jobs.push(job(format!("ticket-{}",hash(&ticket.path.display().to_string())), subject, inputs,
            format!("Honor existing local ticket {}. It is open, unclaimed and dependency-ready. Read its requirements and establish a complete Translation scope before implementation: observed exercise provenance, required dependencies, exact writable paths and concrete verification argv. Include a bounded pre-integration test and an integration gate. Use native translation-scope-v1 TOML; limit each initial experiment to approximately 60 seconds. Research facts do not grant acceptance. Preserve task-origin ownership until its implementation is integrated, verified and cleaned.", ticket.path.display()), Some(ticket.path.clone()))?);
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
        for (rank, (routine, hit)) in selected.into_iter().enumerate() {
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
                (file_key(&inventory_path), tool_execution::bounded::digest(&inventory_path)?),
                (
                    file_key(observed_path.as_ref().unwrap()),
                    tool_execution::bounded::digest(observed_path.as_ref().unwrap())?,
                ),
            ]);
            inputs.insert(
                file_key(Path::new(&format!("deps/snesrev-sm/{}", routine.source))),
                loaded.source_identity.files[&routine.source].clone(),
            );
            inputs.insert("candidate-rank".into(), rank.to_string());
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
        jobs.push(job("translation-research-establish-inputs".into(),"Establish translation exercise and scope inputs".into(), BTreeMap::from([(file_key(Path::new("tools/dependencies.toml")),tool_execution::bounded::digest(&checkout.join("tools/dependencies.toml"))?)]), format!("{reason}. Establish pinned source inventory and naturally exercised coherent gameplay candidates. Existing claimed/resolved tickets remain authoritative; preserve historical evidence. Use existing Rust source-inventory and projectile-inventory commands as appropriate; start with bounded approximately 60-second experiments. Missing observations remain research, never implementation readiness."),None)?);
    }
    Ok(jobs)
}

fn mentions(text: &str, routine: &Routine) -> bool {
    text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).any(|word| word == routine.symbol)
        || address(routine).is_some_and(|a| text.to_ascii_lowercase().contains(&format!("{a:06x}")))
}

pub(crate) fn validate_scope(checkout: &Path, path: &Path) -> Result<Job> {
    decomp_source::context::limited(|| validate_scope_bounded(checkout, path))
}

fn validate_scope_bounded(checkout: &Path, path: &Path) -> Result<Job> {
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
        tool_execution::bounded::digest(&inventory_path)? == scope.source_inventory_sha256
            && tool_execution::bounded::digest(&exercise_path)? == scope.exercise_sha256,
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
                review.ready && !review.rationale.trim().is_empty() && !review.evidence.is_empty(),
                "scope dependency is not ready or lacks rationale/evidence: {dependency}"
            );
            for evidence in &review.evidence {
                ensure!(
                    relative(evidence)
                        && scope.inputs.get(&evidence.to_string_lossy().into_owned())
                            == Some(&tool_execution::bounded::digest(&checkout.join(evidence))?),
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
    if let Some(origin) = &scope.origin_ticket {
        ensure!(relative(origin) && origin.starts_with(".scratch"), "scope origin must name a local task");
        let ticket = all.iter().find(|ticket| &ticket.path == origin).context("scope origin ticket missing")?;
        ensure!(
            matches!(ticket.status.as_str(), "open" | "claimed")
                && ticket.triage.as_deref().is_none_or(|s| s == "ready-for-agent")
                && ticket.kind.as_deref().is_none_or(|kind| kind == "task"),
            "scope origin task cannot be assigned"
        );
        ensure!(
            !ticket.missing_blockers
                && ticket.blockers.iter().all(|p| all.iter().any(|t| &t.path == p && finished(&t.status))),
            "scope origin task has unresolved blockers"
        );
    }
    for dependency in &scope.dependencies {
        ensure!(
            relative(dependency) && dependency.starts_with(".scratch"),
            "scope dependency must name a local ticket"
        );
        let ticket = all.iter().find(|t| &t.path == dependency).context("scope dependency ticket missing")?;
        ensure!(finished(&ticket.status), "scope dependency is not resolved: {}", dependency.display());
        ensure!(
            scope.inputs.get(&dependency.to_string_lossy().into_owned())
                == Some(&tool_execution::bounded::digest(&checkout.join(dependency))?),
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
            target.extension().is_some_and(|e| e == "rs" || e == "smf"),
            "writable ownership requires exact source/test/manifest files"
        );
        for ancestor in checkout.join(target).ancestors().take_while(|p| *p != checkout) {
            ensure!(!ancestor.is_symlink(), "writable path crosses a symlink");
        }
        if checkout.join(target).exists() {
            ensure!(
                scope.inputs.get(&target.to_string_lossy().into_owned())
                    == Some(&tool_execution::bounded::digest(&checkout.join(target))?),
                "current writable source input seal missing: {}",
                target.display()
            );
        }
    }
    ensure!(!scope.inputs.is_empty(), "scope input seals missing");
    for (name, expected) in &scope.inputs {
        ensure!(
            relative(Path::new(name)) && tool_execution::bounded::digest(&checkout.join(name))? == *expected,
            "scope input changed: {name}"
        );
    }
    ensure!(!scope.verification.is_empty(), "scope verification commands missing");
    ensure!(scope.verification.iter().any(|v| !v.integration), "scope needs bounded pre-integration verification");
    ensure!(scope.verification.iter().any(|v| v.integration), "scope needs merged-source integration verification");
    for verification in &scope.verification {
        ensure!(
            !verification.argv.is_empty()
                && verification.argv.iter().all(|v| !v.is_empty() && !v.contains(['\n', '\r', '\0'])),
            "scope verification argv malformed"
        );
        ensure!(
            verification.timeout_seconds > 0
                && verification.timeout_seconds
                    <= if verification.integration { MAX_INTEGRATION_GATE_SECONDS } else { 60 },
            "verification timeout exceeds the bounded experiment or integration limit"
        );
        ensure!(
            verification_command(&verification.argv),
            "scope command must execute cargo test or existing Rust verification operations"
        );
    }
    let mut inputs: BTreeMap<_, _> =
        scope.inputs.iter().map(|(key, value)| (file_key(Path::new(key)), value.clone())).collect();
    inputs.insert(file_key(&path), tool_execution::bounded::digest(&path)?);
    inputs.insert("scope-file".into(), path.display().to_string());
    inputs.insert("scope-sha256".into(), tool_execution::bounded::digest(&path)?);
    if let Some(origin) = &scope.origin_ticket {
        inputs.insert("origin-ticket".into(), origin.display().to_string());
        inputs.insert("origin-ticket-type".into(), "task".into());
    }
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
        ticket: scope.origin_ticket,
    })
}

/// Keep command identity explicit: mentioning a verifier in arbitrary argv does
/// not make a build, status report or agent-chosen executable a required gate.
fn verification_command(argv: &[String]) -> bool {
    let args: Vec<_> = argv.iter().map(String::as_str).collect();
    // Gate execution owns the checkout and the shared worker root. Research
    // cannot redirect them or select a different source tree through argv.
    if args.iter().any(|arg| {
        ["--checkout", "--source", "--root", "--changed-since", "--manifest-path", "--config", "-C"]
            .iter()
            .any(|name| *arg == *name || arg.starts_with(&format!("{name}=")))
    }) {
        return false;
    }
    if args.starts_with(&["cargo", "test"]) {
        return !args.iter().any(|a| matches!(*a, "--no-run" | "--help" | "-h" | "--list"));
    }
    let operation = if args.first() == Some(&"sm-pipeline") {
        &args[1..]
    } else if args.starts_with(&["cargo", "run"]) {
        let Some(separator) = args.iter().position(|a| *a == "--") else { return false };
        let mut package = false;
        let mut locked = false;
        let mut offline = false;
        let mut index = 2;
        while index < separator {
            match args[index] {
                "-p" | "--package" if !package && args.get(index + 1) == Some(&"pipeline") => {
                    package = true;
                    index += 2;
                }
                "--bin" if args.get(index + 1) == Some(&"sm-pipeline") => index += 2,
                "--locked" if !locked => {
                    locked = true;
                    index += 1;
                }
                "--offline" if !offline => {
                    offline = true;
                    index += 1;
                }
                _ => return false,
            }
        }
        if !package || !locked || !offline {
            return false;
        }
        &args[separator + 1..]
    } else {
        return false;
    };
    match operation.first() {
        Some(&"family-verify") => operation
            .iter()
            .position(|arg| *arg == "--manifest")
            .and_then(|index| operation.get(index + 1))
            .is_some_and(|manifest| relative(Path::new(manifest)) && Path::new(manifest).starts_with("tools/port")),
        Some(&"live-verify" | &"routine-live") => operation.len() > 1,
        Some(&"batch") => matches!(operation.get(1), Some(&"verify" | &"run")),
        _ => false,
    }
}

#[cfg(test)]
pub(crate) fn managed_native_scope_fixture(root: &Path, rom: &Path) -> Result<Scope> {
    tests::native_scope_with_rom(root, rom).context("managed native proof requires the local ROM fixture")
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
        assert_eq!(jobs[0].ticket, Some(PathBuf::from(".scratch/work/issues/04-work.md")));
        assert_eq!(jobs[0].inputs.get("origin-ticket").map(String::as_str), Some(".scratch/work/issues/04-work.md"));
        assert_eq!(jobs[0].inputs.get("origin-ticket-type").map(String::as_str), Some("task"));
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
        assert!(jobs.iter().all(|j| j.inputs.contains_key("origin-ticket")));
        let root = project.0.join("out/port/projectile-inventory");
        for number in 0..129 {
            fs::create_dir_all(root.join(number.to_string())).unwrap();
        }
        assert!(default_exercise(&project.0).unwrap_err().to_string().contains("128 runs"));
    }
    fn pinned_inventory(root: &Path) -> Source {
        pinned_inventory_with(root, "void Observed(void) { /* 0x8693ca */ Helper(); }\nvoid Helper(void) {}\n")
    }
    fn pinned_inventory_with(root: &Path, bank: &str) -> Source {
        write(root, "deps/snesrev-sm/src/sm_86.c", bank);
        write(root, "deps/snesrev-sm/LICENSE.txt", "test pinned source\n");
        write(root, "deps/snesrev-sm/src/sm_rtl.h", "#include <stdbool.h>\n");
        write(root, "deps/snesrev-sm/src/funcs.h", "// pinned native test declarations\n");
        fs::create_dir_all(root.join("deps/downloads")).unwrap();
        let archive = root.join("deps/downloads/oracle.tar.gz");
        let mut tar = Vec::new();
        decomp_source::testing::append_tar(&mut tar, "snesrev-sm/src/sm_86.c", b'0', bank.as_bytes());
        decomp_source::testing::append_tar(&mut tar, "snesrev-sm/LICENSE.txt", b'0', b"test pinned source\n");
        decomp_source::testing::append_tar(&mut tar, "snesrev-sm/src/sm_rtl.h", b'0', b"#include <stdbool.h>\n");
        decomp_source::testing::append_tar(
            &mut tar,
            "snesrev-sm/src/funcs.h",
            b'0',
            b"// pinned native test declarations\n",
        );
        tar.extend_from_slice(&[0u8; 1024]);
        decomp_source::testing::gzip_archive(&archive, &tar).unwrap();
        write(
            root,
            "tools/dependencies.toml",
            format!(
                "schema_version = 1\n[[dependencies]]\nname = \"snesrev-sm\"\narchive = \"oracle.tar.gz\"\nrevision = \"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"\nsha256 = {:?}\n",
                tool_execution::bounded::digest(&archive).unwrap()
            ),
        );
        let source = decomp_source::verify_source(root, &root.join("deps/snesrev-sm")).unwrap();
        let routines = toml::Value::Array(vec![
            toml::Value::Table(toml::Table::from_iter([
                ("symbol".into(), "Observed".into()),
                ("source".into(), "src/sm_86.c".into()),
                (
                    "line".into(),
                    toml::Value::Integer(
                        (bank.lines().position(|line| line.starts_with("void Observed(void) {")).unwrap() + 1) as i64,
                    ),
                ),
                ("definition_sha256".into(), hash("void Observed(void) { /* 0x8693ca */ Helper(); }").into()),
                ("original_address".into(), "0x8693ca".into()),
                ("defined_dependencies".into(), toml::Value::try_from(vec!["Helper"]).unwrap()),
            ])),
            toml::Value::Table(toml::Table::from_iter([
                ("symbol".into(), "Helper".into()),
                ("source".into(), "src/sm_86.c".into()),
                (
                    "line".into(),
                    toml::Value::Integer(
                        (bank.lines().position(|line| line.starts_with("void Helper(void) {")).unwrap() + 1) as i64,
                    ),
                ),
                ("definition_sha256".into(), hash("void Helper(void) {}").into()),
            ])),
        ]);
        let value = toml::Value::Table(toml::Table::from_iter([
            ("schema_version".into(), 1.into()),
            ("source_identity".into(), toml::Value::try_from(&source).unwrap()),
            (
                "source_identity_sha256".into(),
                tool_execution::bounded::digest(&root.join("tools/dependencies.toml")).unwrap().into(),
            ),
            ("routines".into(), routines),
        ]));
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
            origin_ticket: None,
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

    #[test]
    fn scope_verification_executes_real_gates_and_rejects_misleading_commands() {
        let command = |parts: &[&str]| verification_command(&parts.iter().map(|s| (*s).into()).collect::<Vec<_>>());
        for parts in [
            vec!["cargo", "test", "-p", "port", "--locked", "--offline"],
            vec!["sm-pipeline", "family-verify", "--manifest", "tools/port/example.toml"],
            vec!["sm-pipeline", "batch", "verify", "--families", "example"],
            vec!["sm-pipeline", "batch", "run", "--families", "example"],
            vec!["cargo", "run", "-p", "pipeline", "--locked", "--offline", "--", "live-verify", "--frames", "9000"],
            vec![
                "cargo",
                "run",
                "--offline",
                "--package",
                "pipeline",
                "--bin",
                "sm-pipeline",
                "--locked",
                "--",
                "routine-live",
                "--frames",
                "9000",
            ],
        ] {
            assert!(command(&parts), "valid verification rejected: {parts:?}");
        }
        for parts in [
            vec!["cargo", "build", "--bin", "test"],
            vec!["cargo", "clean", "test"],
            vec!["cargo", "test", "--no-run"],
            vec!["cargo", "test", "--", "--list"],
            vec!["cargo", "run", "-p", "pipeline", "--", "live-verify"],
            vec!["cargo", "run", "-p", "other", "--locked", "--offline", "--", "live-verify"],
            vec!["cargo", "run", "-p", "pipeline", "--locked", "--offline", "--", "batch", "status"],
            vec!["sm-pipeline", "batch", "catalog", "--profile", "test"],
            vec!["sm-pipeline", "unknown", "family-verify"],
            vec!["bash", "cargo", "test"],
            vec!["cargo", "test", "--manifest-path", "/tmp/other/Cargo.toml"],
            vec!["cargo", "test", "--config=build.target-dir='/tmp/other'"],
            vec!["sm-pipeline", "batch", "verify", "--changed-since", "HEAD"],
            vec!["sm-pipeline", "family-verify", "--checkout", "/tmp/other"],
            vec!["sm-pipeline", "family-verify", "--source", "/tmp/other"],
            vec!["sm-pipeline", "batch", "verify", "--root", "/tmp/private"],
            vec!["sm-pipeline", "family-verify"],
            vec!["sm-pipeline", "family-verify", "--manifest", "/tmp/other.toml"],
            vec!["sm-pipeline", "live-verify"],
            vec!["sm-pipeline", "routine-live"],
        ] {
            assert!(!command(&parts), "non-verification accepted: {parts:?}");
        }
    }

    /// Execute a small C diagnostic with the production ROM identity and seal
    /// its native outputs. This tests the actual scope validator without any
    /// production bypass or long gameplay replay.
    fn native_scope(project: &Project) -> Option<Scope> {
        native_scope_at(&project.0)
    }

    pub(super) fn native_scope_at(root: &Path) -> Option<Scope> {
        let rom = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rom/sm.sfc");
        native_scope_with_rom(root, &rom)
    }

    pub(super) fn native_scope_with_rom(root: &Path, rom: &Path) -> Option<Scope> {
        if !rom.is_file() {
            eprintln!("native Translation scope control requires the repository's local ROM fixture");
            return None;
        }
        let bank = concat!(
            "void Helper(void);\n",
            "void Observed(void) { /* 0x8693ca */ Helper(); }\n",
            "void Helper(void) {}\n",
            "#include <stdlib.h>\n",
            "typedef unsigned short uint16; typedef unsigned int uint32; typedef unsigned char uint8;\n",
            "extern int snes_frame_counter; uint16 eproj_id[2] = {0x93ca};\n",
            "static void InventoryHit(unsigned kind, uint16 address);\n",
            "static void CallEprojPreInstr(uint32 ea, uint16 k) { (void)ea; (void)k; }\n",
            "static const uint8 *CallEprojInstr(uint32 ea, uint16 k, const uint8 *j) { (void)ea; (void)k; return j; }\n",
            "static void CallEprojInit(uint32 ea, uint16 j) { if (ea == 0x8693ca && j == 0) Observed(); }\n",
            "static void EprojRunOne(uint16 k) { CallEprojInit(0x8693ca, k); }\n",
            "void RunOne(void) { EprojRunOne(0); }\n",
        );
        let source = pinned_inventory_with(root, bank);
        fs::create_dir_all(root.join("deps/snesrev-sm/src/snes")).unwrap();
        fs::create_dir_all(root.join("rom")).unwrap();
        fs::copy(&rom, root.join("rom/sm.sfc")).unwrap();
        assert_eq!(tool_execution::bounded::digest(&root.join("rom/sm.sfc")).unwrap(), ROM_SHA256);
        write(root, "movies/super-metroid-any-3653.inputs", "native test movie identity\n");
        let host = concat!(
            "#include <stdio.h>\n#include <stdlib.h>\n",
            "typedef unsigned short uint16; typedef unsigned int uint32; typedef unsigned char uint8;\n",
            "int snes_frame_counter, decompression_calls; extern void RunOne(void);\n",
            "void DecompressToMem(uint32 src, uint8 *dst) { (void)src; (void)dst; }\n",
            "static void SaveRoutineState(void) {}\n",
            "// Linker wrapping\n",
            "void __wrap_DecompressToMem(uint32 src, uint8 *dst) {}\n",
            "// The host has no audio thread\n",
            "int main(int argc, char **argv) {\n",
            "  if (argc != 5 || atoi(argv[3]) != 1) return 2;\n",
            "  FILE *rom = fopen(argv[1], \"rb\"); if (!rom || fgetc(rom) == EOF) return 3; fclose(rom);\n",
            "  FILE *inputs = fopen(argv[2], \"rb\"); if (!inputs || fgetc(inputs) == EOF) return 4;\n",
            "  RunOne();\n",
            "  FILE *terminal = fopen(argv[4], \"wb\"); if (!terminal) return 5; fclose(terminal);\n",
            "  fclose(inputs);\n  return 0;\n}\n",
        );
        write(root, "tools/decomp/headless.c", host);
        write(root, "tools/decomp/headless_frame.h", "// tiny native diagnostic frame contract\n");
        write(root, "run/headless_frame.h", fs::read(root.join("tools/decomp/headless_frame.h")).unwrap());
        let (instrumented_bank, instrumented_host) = crate::projectile_inventory::instrument(bank, host).unwrap();
        write(root, "run/headless.c", instrumented_host);
        write(root, "run/sm_86.c", instrumented_bank);
        write(root, "run/original-source/sm_86.c", fs::read(root.join("deps/snesrev-sm/src/sm_86.c")).unwrap());
        let run = fs::canonicalize(root.join("run")).unwrap();
        let build = crate::projectile_inventory::build_command(&root.join("deps/snesrev-sm"), &run).unwrap();
        assert!(Command::new(&build[0]).args(&build[1..]).status().unwrap().success());
        let replay = vec![
            run.join("inventory").display().to_string(),
            root.join("rom/sm.sfc").display().to_string(),
            root.join("movies/super-metroid-any-3653.inputs").display().to_string(),
            "1".to_owned(),
            run.join("terminal.wram").display().to_string(),
        ];
        assert!(Command::new(&replay[0]).args(&replay[1..]).current_dir(&run).status().unwrap().success());
        let exercise = Exercise {
            schema_version: 2,
            status: "diagnostic-complete".into(),
            accepted: false,
            frames: 1,
            source: source.clone(),
            artifacts: [
                "projectiles.tsv",
                "sm_86.c",
                "headless.c",
                "headless_frame.h",
                "inventory",
                "original-source/sm_86.c",
            ]
            .into_iter()
            .map(|name| (name.into(), tool_execution::bounded::digest(&run.join(name)).unwrap()))
            .collect(),
            rom_sha256: ROM_SHA256.into(),
            inputs_sha256: tool_execution::bounded::digest(&root.join("movies/super-metroid-any-3653.inputs")).unwrap(),
            original_sm86_sha256: source.files["src/sm_86.c"].clone(),
            host_sha256: tool_execution::bounded::digest(&root.join("tools/decomp/headless.c")).unwrap(),
            build_command: build,
            replay_command: replay,
        };
        write(root, "run/verdict.toml", toml::to_string(&exercise).unwrap());
        write(root, "crates/port/src/example.rs", "// maintained source\n");
        ticket(root, 1, "resolved", "None", "ready-for-agent");
        ticket(root, 2, "claimed", "1", "ready-for-agent");
        Some(Scope {
            format: "translation-scope-v1".into(),
            subject: "Observed and coherent helper behavior".into(),
            origin_ticket: Some(".scratch/work/issues/02-work.md".into()),
            source_inventory: "inventory.toml".into(),
            source_inventory_sha256: tool_execution::bounded::digest(&root.join("inventory.toml")).unwrap(),
            exercise: "run/verdict.toml".into(),
            exercise_sha256: tool_execution::bounded::digest(&root.join("run/verdict.toml")).unwrap(),
            routines: vec!["Observed".into(), "Helper".into()],
            dependencies: vec![".scratch/work/issues/01-work.md".into()],
            dependency_review: BTreeMap::new(),
            writable_paths: vec!["crates/port/src/example.rs".into()],
            inputs: ["deps/snesrev-sm/src/sm_86.c", "crates/port/src/example.rs", ".scratch/work/issues/01-work.md"]
                .into_iter()
                .map(|name| (name.into(), tool_execution::bounded::digest(&root.join(name)).unwrap()))
                .collect(),
            verification: vec![
                Verification {
                    argv: vec![
                        "cargo".into(),
                        "test".into(),
                        "-p".into(),
                        "port".into(),
                        "--locked".into(),
                        "--offline".into(),
                    ],
                    timeout_seconds: 60,
                    integration: false,
                },
                Verification {
                    argv: vec![
                        "sm-pipeline".into(),
                        "batch".into(),
                        "verify".into(),
                        "--families".into(),
                        "example".into(),
                    ],
                    timeout_seconds: 60,
                    integration: true,
                },
            ],
        })
    }

    fn promote(root: &Path, scope: &Scope) -> Result<Job> {
        write(root, "scope.toml", toml::to_string(scope).unwrap());
        validate_scope(root, Path::new("scope.toml"))
    }

    #[test]
    fn native_scope_promotes_only_sealed_observed_dependency_complete_owned_work() {
        let project = fresh();
        let Some(scope) = native_scope(&project) else { return };
        let root = &project.0;
        let job = promote(root, &scope).unwrap();
        assert_eq!(job.kind, Kind::Implement);
        assert_eq!(job.writable_paths, scope.writable_paths);
        assert_eq!(job.ticket, scope.origin_ticket);
        assert_eq!(job.inputs["scope-sha256"], tool_execution::bounded::digest(&root.join("scope.toml")).unwrap());
        assert_eq!(job.inputs["origin-ticket-type"], "task");
        let candidates =
            discover(root, Some(Path::new("inventory.toml")), Some(Path::new("run/verdict.toml"))).unwrap();
        assert!(candidates.iter().any(|job| job.id == "translation-research-Observed"));

        let original_bank = fs::read(root.join("run/sm_86.c")).unwrap();
        write(root, "run/sm_86.c", "int arbitrary(void) { return 1; }\n");
        let mut substituted: Exercise = document(&root.join("run/verdict.toml")).unwrap();
        substituted
            .artifacts
            .insert("sm_86.c".into(), tool_execution::bounded::digest(&root.join("run/sm_86.c")).unwrap());
        write(root, "run/verdict.toml", toml::to_string(&substituted).unwrap());
        let mut substituted_scope = scope.clone();
        substituted_scope.exercise_sha256 = tool_execution::bounded::digest(&root.join("run/verdict.toml")).unwrap();
        assert!(promote(root, &substituted_scope).unwrap_err().to_string().contains("transformed sources differ"));
        write(root, "run/sm_86.c", original_bank);
        substituted
            .artifacts
            .insert("sm_86.c".into(), tool_execution::bounded::digest(&root.join("run/sm_86.c")).unwrap());
        write(root, "run/verdict.toml", toml::to_string(&substituted).unwrap());
        assert_eq!(tool_execution::bounded::digest(&root.join("run/verdict.toml")).unwrap(), scope.exercise_sha256);

        let mut missing_helper = scope.clone();
        missing_helper.routines.pop();
        assert!(promote(root, &missing_helper).unwrap_err().to_string().contains("dependency review missing"));
        let mut unready_helper = missing_helper.clone();
        unready_helper.dependency_review.insert(
            "Helper".into(),
            DependencyReview {
                ready: false,
                rationale: "pinned helper reviewed".into(),
                evidence: vec!["deps/snesrev-sm/src/sm_86.c".into()],
            },
        );
        assert!(promote(root, &unready_helper).unwrap_err().to_string().contains("not ready"));
        unready_helper.dependency_review.get_mut("Helper").unwrap().ready = true;
        assert_eq!(promote(root, &unready_helper).unwrap().kind, Kind::Implement);

        let mut no_integration = scope.clone();
        no_integration.verification.retain(|v| !v.integration);
        assert!(promote(root, &no_integration).unwrap_err().to_string().contains("integration verification"));
        let mut unsafe_command = scope.clone();
        unsafe_command.verification[1].argv = vec!["sm-pipeline".into(), "batch".into(), "status".into()];
        assert!(promote(root, &unsafe_command).unwrap_err().to_string().contains("must execute"));
        let mut oversized_gate = scope.clone();
        oversized_gate.verification[1].timeout_seconds = MAX_INTEGRATION_GATE_SECONDS + 1;
        assert!(promote(root, &oversized_gate).unwrap_err().to_string().contains("timeout exceeds"));

        ticket(root, 1, "open", "None", "ready-for-agent");
        assert!(promote(root, &scope).unwrap_err().to_string().contains("unresolved blockers"));
        ticket(root, 1, "resolved", "None", "ready-for-agent");
        write(root, "crates/port/src/example.rs", "// concurrent change\n");
        assert!(promote(root, &scope).unwrap_err().to_string().contains("current writable source"));
        write(root, "crates/port/src/example.rs", "// maintained source\n");

        write(root, "run/projectiles.tsv", "kind\taddress\thits\tfirst_frame\tlast_frame\ninit\t93ca\t1\t0\t1\n");
        assert!(promote(root, &scope).unwrap_err().to_string().contains("exercise artifact changed"));
        let mut exercise: Exercise = document(&root.join("run/verdict.toml")).unwrap();
        exercise.artifacts.insert(
            "projectiles.tsv".into(),
            tool_execution::bounded::digest(&root.join("run/projectiles.tsv")).unwrap(),
        );
        write(root, "run/verdict.toml", toml::to_string(&exercise).unwrap());
        let mut resealed = scope.clone();
        resealed.exercise_sha256 = tool_execution::bounded::digest(&root.join("run/verdict.toml")).unwrap();
        assert!(promote(root, &resealed).unwrap_err().to_string().contains("counter bounds"));

        let mut inventory: toml::Value = document(&root.join("inventory.toml")).unwrap();
        inventory["routines"].as_array_mut().unwrap()[0]["defined_dependencies"] = toml::Value::Array(vec![]);
        write(root, "inventory.toml", toml::to_string(&inventory).unwrap());
        resealed.source_inventory_sha256 = tool_execution::bounded::digest(&root.join("inventory.toml")).unwrap();
        assert!(promote(root, &resealed).unwrap_err().to_string().contains("lexical dependency inventory differs"));
    }
}
