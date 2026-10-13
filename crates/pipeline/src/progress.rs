//! Shared, evidence-linked observations over the complete pinned C inventory.
//! An observation is never an admission or a gameplay acceptance verdict.
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const INVENTORY: &str = "tools/port/source-inventory.toml";
const REGISTRY: &str = ".scratch/decomp-progress/registry.json";
const HTML: &str = ".scratch/decomp-progress/index.html";
const MAX_METADATA: u64 = 64 * 1024 * 1024;
const MAX_UPDATE: u64 = 1024 * 1024;
static NEXT_FILE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
enum Stage {
    Translating,
    Translated,
    Verified,
    Integrated,
    LiveExercised,
}
impl Stage {
    fn name(&self) -> &'static str {
        match self {
            Self::Translating => "translating",
            Self::Translated => "translated",
            Self::Verified => "verified",
            Self::Integrated => "integrated",
            Self::LiveExercised => "live-exercised",
        }
    }
    fn all() -> [Self; 5] {
        [Self::Translating, Self::Translated, Self::Verified, Self::Integrated, Self::LiveExercised]
    }
}

/// Evidence scope is reported provenance, not automatic source authentication.
/// Legacy observations retain their safe, unspecified context and serialization.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum EvidenceContext {
    #[default]
    Unspecified,
    HistoricalFrozen,
    CurrentCheckout,
}
impl EvidenceContext {
    fn name(&self) -> &'static str {
        match self {
            Self::Unspecified => "unspecified",
            Self::HistoricalFrozen => "historical-frozen",
            Self::CurrentCheckout => "current-checkout",
        }
    }
    fn is_unspecified(&self) -> bool {
        *self == Self::Unspecified
    }
    fn all() -> [Self; 3] {
        [Self::Unspecified, Self::HistoricalFrozen, Self::CurrentCheckout]
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Evidence {
    path: String,
    sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Update {
    id: String,
    scope: String,
    agent: String,
    stage: Stage,
    #[serde(default, skip_serializing_if = "EvidenceContext::is_unspecified")]
    evidence_context: EvidenceContext,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    supersedes: Vec<String>,
    routines: Vec<String>,
    source_revision: String,
    inventory_sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    commit: Option<String>,
    evidence: Vec<Evidence>,
    limitations: Vec<String>,
    updated_at: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EvidenceInput {
    path: String,
    sha256: Option<String>,
}

/// The source IDs are mandatory: a worker cannot silently report against a
/// newer inventory after another worker refreshes the shared report.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UpdateInput {
    id: String,
    scope: String,
    agent: String,
    stage: Stage,
    #[serde(default)]
    evidence_context: EvidenceContext,
    #[serde(default)]
    supersedes: Vec<String>,
    routines: Vec<String>,
    source_revision: String,
    inventory_sha256: String,
    commit: Option<String>,
    evidence: Vec<EvidenceInput>,
    limitations: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Routine {
    symbol: String,
    source: String,
    line: usize,
    original_address: Option<String>,
    body_lines: usize,
    definition_sha256: String,
    calls: Vec<String>,
    declared_in: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Source {
    revision: String,
    inventory_sha256: String,
    identity_sha256: String,
    definition_count: usize,
    body_lines: usize,
    scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_root: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_verified_at: Option<u64>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Count {
    definitions: usize,
    body_lines: usize,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Summary {
    definitions: usize,
    body_lines: usize,
    declared_definitions: usize,
    declared_body_lines: usize,
    updates: usize,
    stale_updates: usize,
    stages: BTreeMap<String, Count>,
    #[serde(default)]
    context_stages: BTreeMap<String, BTreeMap<String, Count>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Registry {
    schema_version: u32,
    root: String,
    updated_at: u64,
    source: Source,
    routines: Vec<Routine>,
    updates: Vec<Update>,
    notices: Vec<String>,
    #[serde(default)]
    stale_update_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    superseded_update_ids: Vec<String>,
    summary: Summary,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    coverage_history: Option<history::History>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InventoryRoutine {
    symbol: String,
    source: String,
    line: usize,
    original_address: Option<String>,
    signature: String,
    body_lines: usize,
    definition_sha256: String,
    calls: Vec<String>,
    indirect_indexed_calls: Vec<String>,
    defined_dependencies: Vec<String>,
    external_or_macro_calls: Vec<String>,
    translation_status: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InventoryBank {
    definitions: usize,
    body_lines: usize,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PinnedSource {
    revision: String,
    archive_sha256: String,
    files: BTreeMap<String, String>,
    freshness: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Inventory {
    schema_version: Option<u32>,
    source_identity: PinnedSource,
    source_identity_sha256: Option<String>,
    definition_count: usize,
    scope: String,
    banks: BTreeMap<String, InventoryBank>,
    routines: Vec<InventoryRoutine>,
}
#[derive(Deserialize)]
struct Pins {
    dependencies: Vec<Pin>,
}
#[derive(Deserialize)]
struct Pin {
    name: String,
    revision: String,
    sha256: String,
}
#[derive(Deserialize)]
struct Declaration {
    family: Option<String>,
    #[serde(default)]
    routines: Vec<DeclaredRoutine>,
}
#[derive(Deserialize)]
struct DeclaredRoutine {
    symbol: String,
}

mod declarations;
mod history;
mod source;

#[derive(Clone)]
struct Options {
    root: PathBuf,
    registry: PathBuf,
    html: PathBuf,
    source: Option<PathBuf>,
    update: Option<PathBuf>,
    bind: SocketAddr,
}
fn options(command: &str, args: &[String]) -> Result<Options> {
    let mut values = BTreeMap::new();
    for pair in args.chunks(2) {
        ensure!(pair.len() == 2, "progress option {} needs a value", pair[0]);
        let key = pair[0].as_str();
        ensure!(
            matches!(key, "--root" | "--registry" | "--html")
                || (command == "refresh" && key == "--source")
                || (command == "update" && key == "--update")
                || (command == "serve" && key == "--bind"),
            "unknown progress {command} option {key}"
        );
        ensure!(!pair[1].is_empty() && !pair[1].starts_with("--"), "invalid value for {key}");
        ensure!(values.insert(key, pair[1].as_str()).is_none(), "duplicate progress option {key}");
    }
    let root = values
        .get("--root")
        .map(PathBuf::from)
        .unwrap_or(std::env::current_dir()?)
        .canonicalize()
        .context("progress root does not exist")?;
    ensure!(root.is_dir(), "progress root must be a directory");
    let resolve = |value: &str| {
        let path = Path::new(value);
        if path.is_absolute() { path.to_owned() } else { root.join(path) }
    };
    let registry = resolve(values.get("--registry").copied().unwrap_or(REGISTRY));
    let html = resolve(values.get("--html").copied().unwrap_or(HTML));
    ensure!(registry != html, "registry and HTML paths must differ");
    ensure!(registry.extension().is_some_and(|v| v == "json"), "progress registry must use .json");
    ensure!(html.extension().is_some_and(|v| v == "html"), "progress dashboard must use .html");
    let source = values.get("--source").map(|v| resolve(v));
    let update = values.get("--update").map(|v| resolve(v));
    let bind: SocketAddr = values.get("--bind").copied().unwrap_or("127.0.0.1:4185").parse()?;
    ensure!(bind.ip().is_loopback(), "progress server must bind to a loopback address");
    ensure!(command != "update" || update.is_some(), "progress update requires --update UPDATE.toml");
    Ok(Options { root, registry, html, source, update, bind })
}

pub(crate) fn cli(args: &[String]) -> Result<()> {
    let command = args.first().context(
        "usage: sm-pipeline progress refresh|history|update|status|serve [--root DIR] [--registry FILE] [--html FILE]",
    )?;
    ensure!(
        ["refresh", "history", "update", "status", "serve"].contains(&command.as_str()),
        "unknown progress command {command}"
    );
    let options = options(command, &args[1..])?;
    match command.as_str() {
        "refresh" => {
            let registry = refresh(&options)?;
            print_status(&registry, &options);
        }
        "update" => {
            let registry = update(&options)?;
            print_status(&registry, &options);
        }
        "history" => {
            let _lock = registry_lock(&options.registry, Duration::from_secs(10))?;
            let mut registry = read_registry(&options)?;
            check_current(&options.root, &registry.source)?;
            observe_evidence(&options.root, &mut registry)?;
            registry.coverage_history = Some(history::reconstruct(&options.root, &registry)?);
            registry.updated_at = now()?;
            persist(&options, &registry)?;
            let history = registry.coverage_history.as_ref().unwrap();
            println!(
                "Source coverage history: {} checkpoints / {} current definitions / {} undated uncommitted definitions",
                history.checkpoints.len(),
                history.current.definitions,
                history.uncommitted.len()
            );
            println!("Registry: {}\nDashboard: {}", options.registry.display(), options.html.display());
        }
        "status" => {
            let mut registry = read_registry(&options)?;
            check_current(&options.root, &registry.source)?;
            observe_evidence(&options.root, &mut registry)?;
            print_status(&registry, &options);
        }
        "serve" => serve(&options)?,
        _ => unreachable!(),
    }
    Ok(())
}

fn now() -> Result<u64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}
fn bounded_text(path: &Path, max: u64) -> Result<String> {
    let mut text = String::new();
    File::open(path)?.take(max + 1).read_to_string(&mut text)?;
    ensure!(text.len() as u64 <= max, "metadata exceeds {max} bytes: {}", path.display());
    Ok(text)
}
fn nonempty(value: &str, kind: &str) -> Result<()> {
    ensure!(!value.trim().is_empty() && value.len() <= 4096 && !value.contains('\0'), "invalid {kind}");
    Ok(())
}
fn hex(value: &str, length: usize) -> bool {
    value.len() == length && value.bytes().all(|v| v.is_ascii_hexdigit())
}
fn relative(value: &str) -> bool {
    !value.is_empty() && Path::new(value).components().all(|c| matches!(c, Component::Normal(_)))
}
fn sm_source(value: &str) -> bool {
    relative(value)
        && Path::new(value).parent() == Some(Path::new("src"))
        && Path::new(value).extension().is_some_and(|v| v == "c")
        && Path::new(value).file_stem().is_some_and(|v| v.to_string_lossy().starts_with("sm_"))
}

/// The lock inode persists. Removing it would let two processes lock different
/// inodes; process exit (including crashes) releases this operating-system lock.
fn registry_lock(path: &Path, timeout: Duration) -> Result<File> {
    let parent = path.parent().context("registry path has no parent")?;
    fs::create_dir_all(parent)?;
    let lock = path.with_extension("json.lock");
    let file = OpenOptions::new().create(true).append(true).open(&lock)?;
    let started = Instant::now();
    loop {
        match file.try_lock() {
            Ok(()) => return Ok(file),
            Err(std::fs::TryLockError::WouldBlock) => {
                ensure!(started.elapsed() < timeout, "progress registry lock timed out: {}", lock.display());
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(std::fs::TryLockError::Error(error)) => return Err(error.into()),
        }
    }
}

/// Each file is replaced atomically. The JSON is authoritative; the embedded
/// dashboard snapshot can always be regenerated and live views fetch the JSON.
fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().context("output path has no parent")?;
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(
        ".{}-{}-{}.tmp",
        path.file_name().context("output path has no filename")?.to_string_lossy(),
        std::process::id(),
        NEXT_FILE.fetch_add(1, Ordering::Relaxed)
    ));
    let result = (|| {
        let mut file = OpenOptions::new().write(true).create_new(true).open(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        fs::rename(&temporary, path)?;
        File::open(parent)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}
fn render(registry: &Registry) -> Result<String> {
    let json = serde_json::to_string(registry)?.replace('<', "\\u003c");
    Ok(include_str!("progress.html.in").replace("__PROGRESS_JSON__", &json))
}
fn persist(options: &Options, registry: &Registry) -> Result<()> {
    // Render before committing the authoritative registry so template errors
    // cannot result in accepting an update without a usable dashboard.
    let html = render(registry)?;
    let mut json = serde_json::to_vec_pretty(registry)?;
    json.push(b'\n');
    atomic_write(&options.registry, &json)?;
    atomic_write(&options.html, html.as_bytes())
}

fn pin(root: &Path) -> Result<(String, String, String)> {
    let path = root.join("tools/dependencies.toml");
    let pins: Pins = toml::from_str(&bounded_text(&path, MAX_UPDATE)?)?;
    let matches = pins.dependencies.iter().filter(|v| v.name == "snesrev-sm").collect::<Vec<_>>();
    ensure!(matches.len() == 1, "exactly one snesrev-sm source pin is required");
    let pin = matches[0];
    ensure!(hex(&pin.revision, 40) && hex(&pin.sha256, 64), "invalid snesrev-sm source pin");
    Ok((pin.revision.clone(), pin.sha256.clone(), crate::digest(&path)?))
}
fn check_current(root: &Path, source: &Source) -> Result<()> {
    let (revision, _, identity) = pin(root)?;
    ensure!(
        revision == source.revision && identity == source.identity_sha256,
        "progress source pin is stale; run progress refresh"
    );
    ensure!(
        crate::digest(&root.join(INVENTORY))? == source.inventory_sha256,
        "progress inventory is stale; run progress refresh"
    );
    Ok(())
}
fn load_inventory(root: &Path, checked_source: Option<&Path>) -> Result<(Source, Vec<Routine>, Vec<String>)> {
    let inventory_path = root.join(INVENTORY);
    let inventory: Inventory =
        toml::from_str(&bounded_text(&inventory_path, MAX_METADATA)?).context("invalid C source inventory")?;
    ensure!(inventory.schema_version.is_none_or(|v| v == 1), "unsupported C inventory schema");
    let (revision, archive, identity_sha256) = pin(root)?;
    ensure!(
        inventory.source_identity.revision == revision && inventory.source_identity.archive_sha256 == archive,
        "C inventory does not match the root's pinned C source"
    );
    if let Some(identity) = &inventory.source_identity_sha256 {
        ensure!(*identity == identity_sha256, "C inventory source pin digest changed");
    }
    nonempty(&inventory.scope, "inventory scope")?;
    nonempty(&inventory.source_identity.freshness, "inventory source freshness")?;
    ensure!(
        inventory.definition_count > 0 && inventory.definition_count == inventory.routines.len(),
        "C inventory definition count differs"
    );
    ensure!(!inventory.source_identity.files.is_empty(), "C inventory has no pinned source files");
    for (path, sha) in &inventory.source_identity.files {
        ensure!(relative(path) && hex(sha, 64), "invalid C inventory source identity file {path}");
    }
    let source_files =
        inventory.source_identity.files.keys().filter(|p| sm_source(p)).cloned().collect::<BTreeSet<_>>();
    let mut counts: BTreeMap<String, Count> = BTreeMap::new();
    let mut keys = BTreeSet::new();
    let mut routines = Vec::new();
    for routine in inventory.routines {
        ensure!(crate::family::identifier(&routine.symbol), "invalid C inventory routine {}", routine.symbol);
        ensure!(
            sm_source(&routine.source) && source_files.contains(&routine.source),
            "routine source is outside pinned sm_*.c files"
        );
        ensure!(
            routine.line > 0
                && routine.line as u64 <= MAX_METADATA
                && routine.body_lines > 0
                && routine.body_lines as u64 <= MAX_METADATA
                && hex(&routine.definition_sha256, 64),
            "invalid C definition identity"
        );
        ensure!(
            keys.insert((routine.source.clone(), routine.line, routine.symbol.clone())),
            "duplicate C inventory definition"
        );
        nonempty(&routine.signature, "C signature")?;
        ensure!(routine.translation_status == "unverified", "C lexical inventory must not grant translation status");
        for name in routine
            .calls
            .iter()
            .chain(&routine.indirect_indexed_calls)
            .chain(&routine.defined_dependencies)
            .chain(&routine.external_or_macro_calls)
        {
            ensure!(crate::family::identifier(name), "invalid lexical call {name}");
        }
        if let Some(address) = &routine.original_address {
            ensure!(address.starts_with("0x") && hex(&address[2..], 6), "invalid C routine address");
        }
        let count = counts.entry(routine.source.clone()).or_default();
        count.definitions += 1;
        count.body_lines += routine.body_lines;
        routines.push(Routine {
            symbol: routine.symbol,
            source: routine.source,
            line: routine.line,
            original_address: routine.original_address,
            body_lines: routine.body_lines,
            definition_sha256: routine.definition_sha256,
            calls: routine.calls,
            declared_in: Vec::new(),
        });
    }
    ensure!(source_files == counts.keys().cloned().collect(), "C inventory does not cover every pinned sm_*.c file");
    ensure!(counts.len() == inventory.banks.len(), "C inventory bank count differs");
    for (source, count) in &counts {
        let bank = inventory.banks.get(source).context("C inventory bank missing")?;
        ensure!(
            bank.definitions == count.definitions && bank.body_lines == count.body_lines,
            "C inventory totals differ for {source}"
        );
    }
    let mut notices = vec![
        "Fixture manifests are declarations only. Stage observations do not grant routine admission or gameplay acceptance.".into(),
        "Each stage counts its own explicit observations; a translated claim does not count as verified, integrated or live-exercised.".into(),
        "Coverage is the pinned sm_*.c lexical definition inventory, including support files; external, macro and indirect dependencies require review.".into(),
        "Evidence context is reported provenance, not automatic authentication: historical-frozen refers to retained source, current-checkout requires a supporting source check, and legacy observations remain unspecified.".into(),
        "Reporting completeness is limited to substantiated retained observations. Missing reports do not establish missing Rust implementation; declarations, unavailable artifacts and unproved internal call closure do not grant verified coverage.".into(),
    ];
    if inventory.schema_version.is_none() || inventory.source_identity_sha256.is_none() {
        notices.push("Legacy source inventory lacks a schema version or pin digest. Its declared revision and archive hash match the root pin; refresh with --source to regenerate it.".into());
    }
    if checked_source.is_none() {
        notices.push("This refresh consumed the retained source inventory. Source bytes were not rechecked; use refresh --source DIR for an exact pinned-archive check.".into());
    }
    add_declarations(root, &mut routines, &mut notices)?;
    let source = Source {
        revision,
        inventory_sha256: crate::digest(&inventory_path)?,
        identity_sha256,
        definition_count: routines.len(),
        body_lines: routines.iter().map(|r| r.body_lines).sum(),
        scope: inventory.scope,
        source_root: checked_source.map(|p| p.to_string_lossy().into_owned()),
        source_verified_at: checked_source.map(|_| now()).transpose()?,
    };
    Ok((source, routines, notices))
}

fn add_declarations(root: &Path, routines: &mut [Routine], notices: &mut Vec<String>) -> Result<()> {
    let mut paths =
        fs::read_dir(root.join("tools/port"))?.map(|entry| Ok(entry?.path())).collect::<Result<Vec<_>>>()?;
    paths.sort();
    let known = routines.iter().map(|r| r.symbol.clone()).collect::<BTreeSet<_>>();
    let mut declarations: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for path in paths {
        if path.extension().is_none_or(|v| v != "toml") || path == root.join(INVENTORY) {
            continue;
        }
        let declaration = declarations::parse(&bounded_text(&path, MAX_METADATA)?)
            .with_context(|| format!("invalid declaration manifest {}", path.display()))?;
        if declaration.family.is_none() {
            continue;
        }
        let name = path.strip_prefix(root)?.to_string_lossy().into_owned();
        for routine in declaration.routines {
            ensure!(crate::family::identifier(&routine.symbol), "invalid declared symbol in {name}");
            if known.contains(&routine.symbol) {
                let manifests = declarations.entry(routine.symbol).or_default();
                if !manifests.contains(&name) {
                    manifests.push(name.clone());
                }
            } else {
                notices.push(format!(
                    "Declaration {name} names {} outside the C inventory; it contributes no C coverage.",
                    routine.symbol
                ));
            }
        }
    }
    for routine in routines {
        routine.declared_in = declarations.get(&routine.symbol).cloned().unwrap_or_default();
    }
    Ok(())
}

fn read_registry(options: &Options) -> Result<Registry> {
    let registry: Registry = serde_json::from_str(&bounded_text(&options.registry, MAX_METADATA)?)
        .context("invalid progress registry; run refresh to create it")?;
    ensure!(registry.schema_version == 1, "unsupported progress registry schema");
    ensure!(registry.root == options.root.to_string_lossy(), "progress registry belongs to another root");
    ensure!(
        hex(&registry.source.revision, 40)
            && hex(&registry.source.inventory_sha256, 64)
            && hex(&registry.source.identity_sha256, 64),
        "invalid progress source identity"
    );
    nonempty(&registry.source.scope, "progress source scope")?;
    let mut definitions = BTreeSet::new();
    for routine in &registry.routines {
        ensure!(
            crate::family::identifier(&routine.symbol)
                && sm_source(&routine.source)
                && routine.line > 0
                && routine.line as u64 <= MAX_METADATA
                && routine.body_lines > 0
                && routine.body_lines as u64 <= MAX_METADATA
                && hex(&routine.definition_sha256, 64),
            "invalid progress C definition"
        );
        ensure!(
            definitions.insert((&routine.source, routine.line, &routine.symbol)),
            "duplicate progress C definition"
        );
        for declaration in &routine.declared_in {
            ensure!(relative(declaration), "invalid declaration path in progress registry");
        }
    }
    ensure!(registry.source.definition_count == registry.routines.len(), "progress registry definition count differs");
    ensure!(
        registry.source.body_lines == registry.routines.iter().map(|r| r.body_lines).sum::<usize>(),
        "progress registry line count differs"
    );
    let mut ids = BTreeSet::new();
    for update in &registry.updates {
        ensure!(ids.insert(&update.id), "duplicate registry update ID");
        validate_update_shape(update)?;
    }
    superseded_updates(&registry.updates)?;
    Ok(registry)
}
fn refresh(options: &Options) -> Result<Registry> {
    let _lock = registry_lock(&options.registry, Duration::from_secs(10))?;
    // Never discard an unreadable registry: a malformed file needs inspection.
    let previous = if options.registry.exists() { Some(read_registry(options)?) } else { None };
    let checked_source = options.source.as_ref().map(|p| p.canonicalize()).transpose()?;
    if let Some(source) = &checked_source {
        crate::source_inventory::run(&options.root, source, &options.root.join(INVENTORY))?;
    }
    let (source, routines, notices) = load_inventory(&options.root, checked_source.as_deref())?;
    let mut registry = Registry {
        schema_version: 1,
        root: options.root.to_string_lossy().into_owned(),
        updated_at: now()?,
        source,
        routines,
        updates: previous.as_ref().map(|r| r.updates.clone()).unwrap_or_default(),
        notices,
        stale_update_ids: Vec::new(),
        superseded_update_ids: Vec::new(),
        summary: Summary::default(),
        coverage_history: previous.and_then(|r| r.coverage_history),
    };
    observe_evidence(&options.root, &mut registry)?;
    persist(options, &registry)?;
    Ok(registry)
}

fn validate_update_shape(update: &Update) -> Result<()> {
    nonempty(&update.id, "update ID")?;
    nonempty(&update.scope, "translation scope")?;
    nonempty(&update.agent, "agent")?;
    let mut supersedes = BTreeSet::new();
    for id in &update.supersedes {
        nonempty(id, "superseded update ID")?;
        ensure!(id != &update.id, "a progress update cannot supersede itself");
        ensure!(supersedes.insert(id), "duplicate superseded update ID {id}");
    }
    ensure!(hex(&update.source_revision, 40) && hex(&update.inventory_sha256, 64), "invalid update source identity");
    ensure!(!update.routines.is_empty(), "progress update must name at least one C routine");
    let mut symbols = BTreeSet::new();
    for symbol in &update.routines {
        ensure!(
            crate::family::identifier(symbol) && symbols.insert(symbol),
            "invalid or duplicate progress routine {symbol}"
        );
    }
    ensure!(!update.evidence.is_empty(), "progress update must retain evidence");
    let mut paths = BTreeSet::new();
    for evidence in &update.evidence {
        nonempty(&evidence.path, "evidence path")?;
        ensure!(Path::new(&evidence.path).is_absolute() || relative(&evidence.path), "invalid evidence path");
        ensure!(hex(&evidence.sha256, 64) && paths.insert(&evidence.path), "invalid or duplicate evidence pointer");
    }
    for limitation in &update.limitations {
        nonempty(limitation, "limitation")?;
    }
    if let Some(commit) = &update.commit {
        ensure!(
            (7..=40).contains(&commit.len()) && commit.bytes().all(|v| v.is_ascii_hexdigit()),
            "commit must be a Git object ID"
        );
    }
    ensure!(update.stage != Stage::Integrated || update.commit.is_some(), "integrated observations require a commit");
    Ok(())
}

fn validate_supersession(update: &Update, target: &Update) -> Result<()> {
    ensure!(update.stage == target.stage, "superseded update {} has a different stage", target.id);
    ensure!(
        update.source_revision == target.source_revision && update.inventory_sha256 == target.inventory_sha256,
        "superseded update {} has a different source inventory",
        target.id
    );
    Ok(())
}

/// Corrections are permanent history, independent of replacement freshness.
/// A later stale source must never revive the inaccurate observation it replaced.
fn superseded_updates(updates: &[Update]) -> Result<BTreeMap<String, BTreeSet<String>>> {
    let by_id = updates.iter().map(|update| (update.id.as_str(), update)).collect::<BTreeMap<_, _>>();
    let mut incoming = by_id.keys().map(|id| (*id, 0usize)).collect::<BTreeMap<_, _>>();
    let mut superseded = BTreeMap::<String, BTreeSet<String>>::new();
    for update in updates {
        for id in &update.supersedes {
            let target = by_id.get(id.as_str()).with_context(|| format!("unknown superseded update ID {id}"))?;
            validate_supersession(update, target)?;
            *incoming.get_mut(id.as_str()).unwrap() += 1;
            superseded.entry(id.clone()).or_default().insert(update.id.clone());
        }
    }
    // Iterative topological validation avoids stack growth for long retained histories.
    let mut ready = incoming.iter().filter_map(|(id, count)| (*count == 0).then_some(*id)).collect::<Vec<_>>();
    let mut visited = 0;
    while let Some(id) = ready.pop() {
        visited += 1;
        for target in &by_id[id].supersedes {
            let count = incoming.get_mut(target.as_str()).unwrap();
            *count -= 1;
            if *count == 0 {
                ready.push(target.as_str());
            }
        }
    }
    ensure!(visited == updates.len(), "progress supersession cycle");
    Ok(superseded)
}

fn evidence_path(root: &Path, pointer: &str) -> Result<PathBuf> {
    let path = Path::new(pointer);
    ensure!(path.is_absolute() || relative(pointer), "evidence path must be absolute or a safe root-relative path");
    let path = if path.is_absolute() { path.to_owned() } else { root.join(path) };
    ensure!(path.is_file(), "evidence file is missing: {}", path.display());
    Ok(path)
}
fn input_update(root: &Path, input: UpdateInput) -> Result<Update> {
    let mut evidence = Vec::new();
    for pointer in input.evidence {
        let path = evidence_path(root, &pointer.path)?;
        let sha256 = crate::digest(&path)?;
        if let Some(expected) = pointer.sha256 {
            ensure!(hex(&expected, 64) && expected == sha256, "evidence digest differs: {}", path.display());
        }
        evidence.push(Evidence { path: pointer.path, sha256 });
    }
    let update = Update {
        id: input.id,
        scope: input.scope,
        agent: input.agent,
        stage: input.stage,
        evidence_context: input.evidence_context,
        supersedes: input.supersedes,
        routines: input.routines,
        source_revision: input.source_revision,
        inventory_sha256: input.inventory_sha256,
        commit: input.commit,
        evidence,
        limitations: input.limitations,
        updated_at: now()?,
    };
    validate_update_shape(&update)?;
    Ok(update)
}
fn insert_update(registry: &mut Registry, mut update: Update) -> Result<bool> {
    validate_update_shape(&update)?;
    ensure!(
        update.source_revision == registry.source.revision
            && update.inventory_sha256 == registry.source.inventory_sha256,
        "update source inventory is stale"
    );
    let mut symbols: BTreeMap<&str, usize> = BTreeMap::new();
    for routine in &registry.routines {
        *symbols.entry(&routine.symbol).or_default() += 1;
    }
    for symbol in &update.routines {
        ensure!(symbols.contains_key(symbol.as_str()), "unknown C routine {symbol}");
        ensure!(
            symbols[symbol.as_str()] == 1,
            "ambiguous C routine {symbol}; select a distinct source inventory scope"
        );
    }
    if let Some(existing) = registry.updates.iter().find(|v| v.id == update.id) {
        update.updated_at = existing.updated_at;
        ensure!(
            *existing == update,
            "conflicting progress update ID {}; use a new ID for a new observation",
            update.id
        );
        return Ok(false);
    }
    for id in &update.supersedes {
        let target = registry
            .updates
            .iter()
            .find(|target| &target.id == id)
            .with_context(|| format!("unknown superseded update ID {id}"))?;
        validate_supersession(&update, target)?;
    }
    registry.updated_at = update.updated_at;
    registry.updates.push(update);
    Ok(true)
}
fn update(options: &Options) -> Result<Registry> {
    let path = options.update.as_ref().context("update file missing")?;
    ensure!(path.extension().is_some_and(|v| v == "toml"), "progress update must use .toml");
    let input: UpdateInput =
        toml::from_str(&bounded_text(path, MAX_UPDATE)?).context("invalid progress update TOML")?;
    // Read/hashing can happen before locking, but the source identity and
    // retained bytes are rechecked under the lock before publication.
    let update = input_update(&options.root, input)?;
    let _lock = registry_lock(&options.registry, Duration::from_secs(10))?;
    let mut registry = read_registry(options)?;
    check_current(&options.root, &registry.source)?;
    for pointer in &update.evidence {
        ensure!(
            crate::digest(&evidence_path(&options.root, &pointer.path)?)? == pointer.sha256,
            "evidence changed while publishing update"
        );
    }
    insert_update(&mut registry, update)?;
    observe_evidence(&options.root, &mut registry)?;
    // An idempotent retry also repairs a dashboard left unwritten by a crash.
    persist(options, &registry)?;
    Ok(registry)
}

fn observe_evidence(root: &Path, registry: &mut Registry) -> Result<()> {
    let superseded = superseded_updates(&registry.updates)?;
    registry.notices.retain(|v| !v.starts_with("Stale update ") && !v.starts_with("Superseded update "));
    let mut stale = BTreeSet::new();
    let mut hashes: BTreeMap<String, Option<String>> = BTreeMap::new();
    let known = registry.routines.iter().map(|r| r.symbol.as_str()).collect::<BTreeSet<_>>();
    for update in &registry.updates {
        let reason = if update.source_revision != registry.source.revision
            || update.inventory_sha256 != registry.source.inventory_sha256
        {
            Some("source inventory changed".to_owned())
        } else if update.routines.iter().any(|r| !known.contains(r.as_str())) {
            Some("a routine is absent from the inventory".to_owned())
        } else {
            update.evidence.iter().find_map(|pointer| {
                let digest = hashes
                    .entry(pointer.path.clone())
                    .or_insert_with(|| evidence_path(root, &pointer.path).and_then(|p| crate::digest(&p)).ok());
                match digest {
                    Some(digest) if *digest == pointer.sha256 => None,
                    Some(_) => Some(format!("evidence changed: {}", pointer.path)),
                    None => Some(format!("evidence unavailable: {}", pointer.path)),
                }
            })
        };
        if let Some(reason) = reason {
            stale.insert(update.id.clone());
            registry.notices.push(format!("Stale update {}: {reason}; excluded from stage totals.", update.id));
        }
    }
    registry.stale_update_ids = stale.iter().cloned().collect();
    registry.superseded_update_ids = superseded.keys().cloned().collect();
    for (id, replacements) in &superseded {
        registry.notices.push(format!(
            "Superseded update {id}: replaced by {}; excluded from stage totals even if replacement evidence becomes stale.",
            replacements.iter().cloned().collect::<Vec<_>>().join(", ")
        ));
    }
    registry.summary = summarize(registry, &stale, &registry.superseded_update_ids.iter().cloned().collect());
    history::observe(root, registry);
    Ok(())
}
fn summarize(registry: &Registry, stale: &BTreeSet<String>, superseded: &BTreeSet<String>) -> Summary {
    let mut summary = Summary {
        definitions: registry.routines.len(),
        body_lines: registry.source.body_lines,
        updates: registry.updates.len(),
        stale_updates: stale.len(),
        ..Summary::default()
    };
    // A definition appears once per explicitly observed stage, even when several
    // agents or scopes report that same stage. Stages do not imply each other.
    for stage in Stage::all() {
        let symbols = registry
            .updates
            .iter()
            .filter(|u| u.stage == stage && !stale.contains(&u.id) && !superseded.contains(&u.id))
            .flat_map(|u| u.routines.iter().map(String::as_str))
            .collect::<BTreeSet<_>>();
        let mut count = Count::default();
        for routine in &registry.routines {
            if symbols.contains(routine.symbol.as_str()) {
                count.definitions += 1;
                count.body_lines += routine.body_lines;
            }
        }
        summary.stages.insert(stage.name().into(), count);
    }
    for context in EvidenceContext::all() {
        let mut stages = BTreeMap::new();
        for stage in Stage::all() {
            let symbols = registry
                .updates
                .iter()
                .filter(|u| {
                    u.stage == stage
                        && u.evidence_context == context
                        && !stale.contains(&u.id)
                        && !superseded.contains(&u.id)
                })
                .flat_map(|u| u.routines.iter().map(String::as_str))
                .collect::<BTreeSet<_>>();
            let mut count = Count::default();
            for routine in &registry.routines {
                if symbols.contains(routine.symbol.as_str()) {
                    count.definitions += 1;
                    count.body_lines += routine.body_lines;
                }
            }
            stages.insert(stage.name().into(), count);
        }
        summary.context_stages.insert(context.name().into(), stages);
    }
    for routine in &registry.routines {
        if !routine.declared_in.is_empty() {
            summary.declared_definitions += 1;
            summary.declared_body_lines += routine.body_lines;
        }
    }
    summary
}
fn print_status(registry: &Registry, options: &Options) {
    println!(
        "C inventory: {} definitions, {} body lines at {}",
        registry.source.definition_count, registry.source.body_lines, registry.source.revision
    );
    println!("Fixture declarations only: {} definitions", registry.summary.declared_definitions);
    for (stage, count) in &registry.summary.stages {
        println!("{stage}: {} explicitly observed definitions, {} body lines", count.definitions, count.body_lines);
    }
    for (context, stages) in &registry.summary.context_stages {
        let counts = stages.iter().map(|(stage, count)| format!("{stage}={}", count.definitions)).collect::<Vec<_>>();
        println!("Evidence context {context}: {}", counts.join(", "));
    }
    println!(
        "Observations: {}; stale: {}; superseded: {}",
        registry.updates.len(),
        registry.summary.stale_updates,
        registry.superseded_update_ids.len()
    );
    for notice in &registry.notices {
        println!("Notice: {notice}");
    }
    println!("Registry: {}\nDashboard: {}", options.registry.display(), options.html.display());
}

fn serve(options: &Options) -> Result<()> {
    let registry = read_registry(options)?;
    check_current(&options.root, &registry.source)?;
    let listener = TcpListener::bind(options.bind)?;
    println!("Progress dashboard: http://{}/index.html", listener.local_addr()?);
    for connection in listener.incoming() {
        match connection {
            Ok(mut stream) => {
                if let Err(error) = respond(&mut stream, options) {
                    eprintln!("progress request: {error:#}");
                }
            }
            Err(error) => eprintln!("progress connection: {error}"),
        }
    }
    Ok(())
}
fn response(status: &str, mime: &str, body: &[u8]) -> Vec<u8> {
    let mut bytes = format!("HTTP/1.1 {status}\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\nX-Content-Type-Options: nosniff\r\n\r\n", body.len()).into_bytes();
    bytes.extend_from_slice(body);
    bytes
}
fn request_target(bytes: &[u8]) -> Result<&str> {
    let text = std::str::from_utf8(bytes).context("HTTP request is not UTF-8")?;
    let line = text.split("\r\n").next().context("HTTP request line missing")?;
    let fields = line.split(' ').collect::<Vec<_>>();
    ensure!(
        fields.len() == 3 && fields[0] == "GET" && matches!(fields[2], "HTTP/1.0" | "HTTP/1.1"),
        "only bounded HTTP GET requests are supported"
    );
    Ok(fields[1].split('?').next().unwrap_or(fields[1]))
}
fn respond(stream: &mut TcpStream, options: &Options) -> Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    let mut bytes = Vec::new();
    let mut buffer = [0u8; 1024];
    let started = Instant::now();
    loop {
        ensure!(started.elapsed() < Duration::from_secs(2), "HTTP request timed out");
        let count = stream.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        bytes.extend_from_slice(&buffer[..count]);
        if bytes.len() > 8192 {
            stream.write_all(&response("431 Request Header Fields Too Large", "text/plain", b"request too large\n"))?;
            return Ok(());
        }
        if bytes.windows(4).any(|v| v == b"\r\n\r\n") {
            break;
        }
    }
    if !bytes.windows(4).any(|v| v == b"\r\n\r\n") {
        stream.write_all(&response("400 Bad Request", "text/plain", b"complete HTTP headers required\n"))?;
        return Ok(());
    }
    let target = match request_target(&bytes) {
        Ok(target) => target,
        Err(_) => {
            stream.write_all(&response("400 Bad Request", "text/plain", b"only HTTP GET is supported\n"))?;
            return Ok(());
        }
    };
    let mime = match target {
        "/" | "/index.html" => "text/html; charset=utf-8",
        "/registry.json" => "application/json; charset=utf-8",
        _ => {
            stream.write_all(&response("404 Not Found", "text/plain", b"not found\n"))?;
            return Ok(());
        }
    };
    // Load the latest atomic JSON on every request and recheck retained evidence
    // so a removed or altered file is never shown as current observed coverage.
    let snapshot = (|| -> Result<String> {
        let mut registry = read_registry(options)?;
        check_current(&options.root, &registry.source)?;
        observe_evidence(&options.root, &mut registry)?;
        if target == "/registry.json" { Ok(serde_json::to_string(&registry)?) } else { render(&registry) }
    })();
    let body = match snapshot {
        Ok(body) => body,
        Err(error) => {
            stream.write_all(&response(
                "409 Conflict",
                "text/plain; charset=utf-8",
                format!("Progress snapshot unavailable: {error:#}\nRun progress refresh.\n").as_bytes(),
            ))?;
            return Ok(());
        }
    };
    stream.write_all(&response("200 OK", mime, body.as_bytes()))?;
    Ok(())
}

#[cfg(test)]
mod tests;
