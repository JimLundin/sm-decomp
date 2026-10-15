//! Original compact causal diagnostic at the trusted supervisor boundary.
use crate::acquire_lease_until;
use anyhow::{Context, Result, bail, ensure};
use offline_verification::fixtures::Asset;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Read, Write},
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tool_execution::bounded;
use translation_execution::{
    OperationContext,
    budget::ByteBudget,
    confinement::{self, Program, ReadOnlyBind, RunLimits},
    model::{digest_valid, sealed, sha256},
    operation,
    retention::{self, Reference},
};

const FRAMES: u64 = 2240;
const SECONDS: u64 = 60;
const MAX_RECORD: u64 = 16 * 1024 * 1024;
const MAX_FILE: u64 = 256 * 1024 * 1024;
const MAX_RAW: u64 = 512 * 1024 * 1024;
const MAX_HISTORY: u64 = 8 * 1024 * 1024 * 1024;
const MAX_MEMBERS: usize = 20_000;
const ROM_SHA256: &str = "12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72";
const MOVIE_SHA256: &str = "e53b1ce15b3f01516eb64c68a6db2911f91bd97197eb05960779561be5c7cbe9";
const C_REVISION: &str = "578f90b3cc49557bb70060ad033bb90b8cf8ac50";
const C_ARCHIVE_SHA256: &str = "69c78831351112887575f852dadf0281dd5c23cef890b07a268ae6cf60aa9193";
const LSNES_SHA256: &str = "4c0468053a79eaa557d5ee536d9cda9c4c8cd9555848b6c95d243c08a568ec32";
const COMPACT_READER_SHA256: &str = "2898c6a3f553e74051abbb00f47a429fd653522d0caa4a06be33161eb581e57a";
const HISTORICAL_SEALS: [(&str, &str, &str); 5] = [
    (
        "full",
        "764d9e92fbd0fe572a2ba595a016ec6d6fa4c273ab6bfe56540cfc0c3e708983",
        "c1036b826bbe6d611e7f330d4c465bf6e4318267bcf05104ef9bfbd8897d91a8",
    ),
    (
        "full",
        "cfda18ec67d62ee947b1d94363096a9482284c10a906637cd7497356296d021e",
        "b31a8ed5de08e2715c86d5344187d9abaac7473e3e318a5a1b9b0aa92a4ae6f7",
    ),
    ("compact", "79c00cb7bb32a120f981b1c14f99b639e773d88f5f544bdb41e964409ce7a916", COMPACT_READER_SHA256),
    ("compact", "7c8c04d2f53d3f7a247a715a807d2dcf0e2d0004be13facd780c3d74a717fe88", COMPACT_READER_SHA256),
    ("compact", "051a7195bf0ed3461eae81a9789f052a01f5c616d0d7ed716a37848c2a59d07e", COMPACT_READER_SHA256),
];
const SOURCE_PATHS: [&str; 9] = [
    "crates/pipeline/src/decomp_prefix.rs",
    "crates/pipeline/src/prefix_observer.lua.in",
    "crates/pipeline/src/prefix_compact_observer.lua.in",
    "crates/pipeline/tests/prefix.rs",
    "docs/DECOMP_PREFIX.md",
    "tools/decomp/prefix_host.c",
    "tools/decomp/headless.c",
    "tools/decomp/headless_frame.h",
    "tools/dependencies.toml",
];
const FIELDS: [(&str, usize); 19] = [
    ("game_state", 0x998),
    ("room", 0x79b),
    ("samus_x", 0xaf6),
    ("samus_y", 0xafa),
    ("health", 0x9c2),
    ("items", 0x9a4),
    ("equipped_items", 0x9a2),
    ("clock_frames", 0x9da),
    ("clock_seconds", 0x9dc),
    ("clock_minutes", 0x9de),
    ("clock_hours", 0x9e0),
    ("rng", 0x5e5),
    ("held", 0x8b),
    ("pressed", 0x8f),
    ("previous", 0x97),
    ("accepted_nmi", 0x5b6),
    ("all_nmi", 0x5b8),
    ("upload", 0x617),
    ("waiting", 0x5b4),
];
const HOOKS: [(&str, u32); 14] = [
    ("reset", 0x80841c),
    ("boot-clear", 0x808482),
    ("controller-entry", 0x809459),
    ("controller-return", 0x809496),
    ("dispatch-entry", 0x828948),
    ("dispatch-complete", 0x82897a),
    ("explicit-wait", 0x808338),
    ("nmi-entry", 0x809583),
    ("message-entry", 0x858080),
    ("message-return", 0x8580ba),
    ("upload-entry", 0x808028),
    ("upload-return", 0x8080f1),
    ("upload-return-short", 0x80802e),
    ("upload-return-abort", 0x8080ff),
];

fn now_ms() -> Result<u64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis().try_into()?)
}

fn relative(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 4096
        && !value.contains(['\\', '\0'])
        && value.split('/').all(|part| !part.is_empty() && part != "." && part != "..")
        && Path::new(value).components().all(|part| matches!(part, Component::Normal(_)))
}

fn safe(path: &Path) -> Result<()> {
    ensure!(path.is_absolute(), "compact path must be absolute");
    let mut current = PathBuf::new();
    for part in path.components() {
        ensure!(matches!(part, Component::RootDir | Component::Normal(_)), "compact path alias");
        current.push(part);
        match current.symlink_metadata() {
            Ok(metadata) => ensure!(!metadata.file_type().is_symlink(), "compact path crosses symlink"),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn read(path: &Path, maximum: u64) -> Result<Vec<u8>> {
    safe(path)?;
    let mut file = OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW).open(path)?;
    let before = file.metadata()?;
    ensure!(before.is_file() && before.len() <= maximum, "compact input extent exceeded");
    let mut bytes = Vec::new();
    (&mut file).take(maximum + 1).read_to_end(&mut bytes)?;
    ensure!(bytes.len() as u64 == before.len(), "compact input changed while reading");
    Ok(bytes)
}

fn publish(path: &Path, bytes: &[u8], executable: bool) -> Result<()> {
    safe(path)?;
    ensure!(bytes.len() as u64 <= MAX_RAW, "compact publication extent exceeded");
    if path.try_exists()? {
        ensure!(read(path, bytes.len() as u64)? == bytes, "compact immutable output changed");
        return Ok(());
    }
    let parent = path.parent().context("compact output parent missing")?;
    fs::create_dir_all(parent)?;
    let pending = path.with_extension("pending");
    safe(&pending)?;
    if pending.try_exists()? {
        ensure!(pending.symlink_metadata()?.is_file(), "compact pending path is not regular");
        fs::remove_file(&pending)?;
    }
    let mut file =
        OpenOptions::new().write(true).create_new(true).mode(0o600).custom_flags(libc::O_NOFOLLOW).open(&pending)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    file.set_permissions(fs::Permissions::from_mode(if executable { 0o500 } else { 0o400 }))?;
    fs::rename(pending, path)?;
    File::open(parent)?.sync_all()?;
    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct CompactContract {
    pub schema: String,
    pub frames: u64,
    pub timeout_seconds: u64,
    pub source_files: BTreeMap<String, String>,
    pub private_inputs: BTreeMap<String, Reference>,
    pub c_units: Vec<String>,
    pub runtime: Vec<RuntimeBinding>,
    pub historical: Vec<HistoricalCapture>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct RuntimeBinding {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub sha256: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct HistoricalCapture {
    pub profile: String,
    pub verdict: Reference,
    pub reader: Reference,
    pub private_inputs: Vec<Reference>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct PreparedCompact {
    pub contract: CompactContract,
    pub dependencies_sha256: String,
    pub verification_sha256: String,
}

/// Freeze the original inputs, including the unchanged full observer. No
/// candidate-controlled configuration supplies a command or reduced target.
pub(crate) fn contract_for_original(
    checkout: &Path,
    assets: &Path,
    rustc: &Path,
    historical: Vec<HistoricalCapture>,
) -> Result<CompactContract> {
    let source = decomp_source::verify_source(checkout, &assets.join("deps/snesrev-sm"))?;
    let mut private_inputs = BTreeMap::new();
    for (name, path) in [
        ("rom.sfc", assets.join("rom/sm.sfc")),
        ("movie.lsmv", assets.join("movies/super-metroid-any-3653.lsmv")),
        ("movies.toml", assets.join("movies/manifest.toml")),
        ("lsnes", assets.join("deps/lsnes/source/src/util/lsnes-dumpavi.util")),
        ("c/LICENSE.txt", assets.join("deps/snesrev-sm/LICENSE.txt")),
    ] {
        private_inputs.insert(name.into(), retention::seal_file(&fs::canonicalize(path)?, MAX_FILE)?);
    }
    let pins: toml::Value =
        toml::from_str(&String::from_utf8(read(&checkout.join("tools/dependencies.toml"), MAX_RECORD)?)?)?;
    let pin = pins["dependencies"]
        .as_array()
        .context("dependency pins missing")?
        .iter()
        .find(|pin| pin["name"].as_str() == Some("snesrev-sm"))
        .context("pinned C archive missing")?;
    private_inputs.insert(
        "source.tar.gz".into(),
        retention::seal_file(
            &fs::canonicalize(
                assets.join("deps/downloads").join(pin["archive"].as_str().context("C archive name missing")?),
            )?,
            MAX_FILE,
        )?,
    );
    for (name, expected) in &source.files {
        if name.starts_with("src/") {
            let reference = retention::seal_file(&assets.join("deps/snesrev-sm").join(name), MAX_FILE)?;
            ensure!(&reference.sha256 == expected, "pinned compact C dependency changed");
            private_inputs.insert(format!("c/{name}"), reference);
        }
    }
    let mut runtime = ::translation_execution::compiler_runtime::discover(rustc)?;
    runtime.retain(|bind| !bind.destination.starts_with("/toolchain"));
    // Exact dynamic closure of the retained lsnes binary on the supported host.
    // The producer receives individual libraries, never host /usr or HOME.
    for name in [
        "libcurl.so.4",
        "libbz2.so.1.0",
        "libnghttp2.so.14",
        "libidn2.so.0",
        "librtmp.so.1",
        "libssh.so.4",
        "libpsl.so.5",
        "libssl.so.1.1",
        "libcrypto.so.1.1",
        "libgssapi_krb5.so.2",
        "libldap_r-2.4.so.2",
        "liblber-2.4.so.2",
        "libbrotlidec.so.1",
        "libunistring.so.2",
        "libgnutls.so.30",
        "libhogweed.so.5",
        "libnettle.so.7",
        "libkrb5.so.3",
        "libk5crypto.so.3",
        "libcom_err.so.2",
        "libkrb5support.so.0",
        "libresolv.so.2",
        "libsasl2.so.2",
        "libgssapi.so.3",
        "libbrotlicommon.so.1",
        "libp11-kit.so.0",
        "libtasn1.so.6",
        "libkeyutils.so.1",
        "libheimntlm.so.0",
        "libkrb5.so.26",
        "libasn1.so.8",
        "libhcrypto.so.4",
        "libroken.so.18",
        "libffi.so.7",
        "libwind.so.0",
        "libheimbase.so.1",
        "libhx509.so.5",
        "libsqlite3.so.0",
        "libcrypt.so.1",
    ] {
        let destination = Path::new("/lib/x86_64-linux-gnu").join(name);
        runtime.push(ReadOnlyBind { source: fs::canonicalize(&destination)?, destination });
    }
    for name in ["libboost_iostreams.so.1.71.0", "libboost_filesystem.so.1.71.0", "liblua5.2.so.0"] {
        runtime.push(ReadOnlyBind {
            source: fs::canonicalize(assets.join("deps/lsnes-build-deps/usr/lib/x86_64-linux-gnu").join(name))?,
            destination: Path::new("/lib/x86_64-linux-gnu").join(name),
        });
    }
    runtime.sort_by(|a, b| a.destination.cmp(&b.destination));
    runtime.dedup_by(|a, b| a == b);
    let runtime = runtime
        .into_iter()
        .map(|bind| {
            Ok(RuntimeBinding {
                sha256: runtime_hash(&bind.source)?,
                source: bind.source,
                destination: bind.destination,
            })
        })
        .collect::<Result<_>>()?;
    let source_files = SOURCE_PATHS
        .into_iter()
        .map(|name| Ok((name.into(), retention::seal_file(&checkout.join(name), MAX_RECORD)?.sha256)))
        .collect::<Result<_>>()?;
    let c_root = assets.join("deps/snesrev-sm");
    let c_units = decomp_source::c_units(&c_root)?
        .into_iter()
        .map(|path| Ok(format!("c/{}", path.strip_prefix(&c_root)?.display())))
        .collect::<Result<_>>()?;
    Ok(CompactContract {
        schema: "translation-compact-contract-v1".into(),
        frames: FRAMES,
        timeout_seconds: SECONDS,
        source_files,
        private_inputs,
        c_units,
        runtime,
        historical,
    })
}

fn runtime_hash(path: &Path) -> Result<String> {
    fn visit(root: &Path, path: &Path, entries: &mut BTreeMap<String, String>, total: &mut u64) -> Result<()> {
        bounded::check()?;
        ensure!(entries.len() < 65536, "compact runtime closure count exceeded");
        let metadata = path.symlink_metadata()?;
        let name = path.strip_prefix(root)?.to_string_lossy().into_owned();
        if metadata.file_type().is_symlink() {
            entries.insert(name, format!("link:{}", fs::read_link(path)?.display()));
        } else if metadata.is_dir() {
            entries.insert(name, "directory".into());
            let mut children = fs::read_dir(path)?
                .map(|entry| entry.map(|entry| entry.path()))
                .collect::<std::io::Result<Vec<_>>>()?;
            children.sort();
            for child in children {
                visit(root, &child, entries, total)?;
            }
        } else {
            ensure!(metadata.is_file(), "compact runtime contains a special file");
            *total = total.checked_add(metadata.len()).context("compact runtime extent overflow")?;
            ensure!(*total <= 1024 * 1024 * 1024, "compact runtime byte closure exceeded");
            account_input(metadata.len())?;
            entries.insert(name, retention::seal_file(path, MAX_FILE)?.sha256);
        }
        Ok(())
    }
    safe(path)?;
    let mut entries = BTreeMap::new();
    visit(path, path, &mut entries, &mut 0)?;
    sealed(&entries)
}

fn account_input(mut bytes: u64) -> Result<()> {
    while bytes > 0 {
        let chunk = bytes.min(MAX_RECORD);
        bounded::account(chunk)?;
        bytes -= chunk;
    }
    bounded::check()
}

fn prepare_reference(reference: &Reference, authenticated: &mut BTreeMap<PathBuf, Reference>) -> Result<()> {
    if let Some(previous) = authenticated.get(&reference.path) {
        ensure!(previous == reference, "compact prepared input has conflicting identities");
    } else {
        account_input(reference.bytes)?;
        retention::authenticate_reference(reference, MAX_FILE)?;
        authenticated.insert(reference.path.clone(), reference.clone());
    }
    bounded::check()
}

pub(crate) fn prepare(checkout: &Path, contract: &CompactContract) -> Result<PreparedCompact> {
    decomp_source::context::limited(|| prepare_inner(checkout, contract))
}

pub(crate) fn prepare_until(
    checkout: &Path,
    contract: &CompactContract,
    deadline: u64,
    bytes: u64,
) -> Result<PreparedCompact> {
    decomp_source::context::until(deadline, bytes, || prepare_inner(checkout, contract))
}

fn prepare_inner(checkout: &Path, contract: &CompactContract) -> Result<PreparedCompact> {
    ensure!(
        contract.frames == 2240 && contract.timeout_seconds == 60,
        "original compact diagnostic requires frame 2240 and one 60-second experiment"
    );
    ensure!(
        contract.historical.iter().any(|capture| capture.profile == "full")
            && contract.historical.iter().any(|capture| capture.profile == "compact"),
        "original compact acceptance requires full and compact historical captures"
    );
    ensure!(
        contract.historical.len() == HISTORICAL_SEALS.len()
            && contract.historical.iter().map(|capture| capture.verdict.sha256.as_str()).collect::<BTreeSet<_>>()
                == HISTORICAL_SEALS.iter().map(|(_, hash, _)| *hash).collect(),
        "compact acceptance requires the complete original full/compact capture lineage"
    );
    ensure!(contract.schema == "translation-compact-contract-v1", "unsupported compact contract");
    ensure!(
        contract.source_files.keys().map(String::as_str).collect::<BTreeSet<_>>() == SOURCE_PATHS.into_iter().collect(),
        "compact source closure omits original controller/full observer or C host dependencies"
    );
    safe(checkout)?;
    for (name, expected) in &contract.source_files {
        account_input(checkout.join(name).metadata()?.len())?;
        ensure!(
            retention::seal_file(&checkout.join(name), MAX_RECORD)?.sha256 == *expected,
            "compact reviewed source changed: {name}"
        );
    }
    ensure!(
        contract.private_inputs.len() <= 1024 && contract.c_units.len() <= 256,
        "compact input closure count exceeded"
    );
    let mut private_bytes = 0u64;
    let mut authenticated = BTreeMap::new();
    for (name, reference) in &contract.private_inputs {
        ensure!(relative(name), "compact private input path alias");
        private_bytes = private_bytes.checked_add(reference.bytes).context("compact private input extent overflow")?;
        ensure!(private_bytes <= MAX_RAW, "compact private input aggregate byte bound exceeded");
        prepare_reference(reference, &mut authenticated)?;
    }
    ensure!(
        contract.private_inputs.get("rom.sfc").is_some_and(|reference| reference.sha256 == ROM_SHA256),
        "original compact ROM seal differs"
    );
    for name in ["movie.lsmv", "movies.toml", "source.tar.gz", "lsnes", "c/LICENSE.txt"] {
        ensure!(contract.private_inputs.contains_key(name), "compact private input missing: {name}");
    }
    ensure!(
        contract.private_inputs["source.tar.gz"].sha256 == C_ARCHIVE_SHA256
            && contract.private_inputs["lsnes"].sha256 == LSNES_SHA256
            && contract.private_inputs["movie.lsmv"].sha256 == MOVIE_SHA256,
        "original compact private archive, movie or native executable differs"
    );
    let c_root = contract.private_inputs["c/LICENSE.txt"].path.parent().context("compact C root missing")?;
    let source = decomp_source::verify_source(checkout, c_root)?;
    ensure!(
        source.revision == C_REVISION && source.archive_sha256 == C_ARCHIVE_SHA256,
        "compact original pinned C revision differs"
    );
    let expected_c = source
        .files
        .iter()
        .filter(|(name, _)| name.starts_with("src/") || name.as_str() == "LICENSE.txt")
        .map(|(name, hash)| (format!("c/{name}"), hash))
        .collect::<BTreeMap<_, _>>();
    ensure!(
        contract.private_inputs.len() == expected_c.len() + 5
            && contract.private_inputs.keys().filter(|name| name.starts_with("c/")).eq(expected_c.keys()),
        "compact original private C closure differs"
    );
    for (name, expected) in expected_c {
        let reference = &contract.private_inputs[&name];
        ensure!(
            &reference.sha256 == expected && reference.path == c_root.join(name.strip_prefix("c/").unwrap()),
            "compact original private C dependency differs: {name}"
        );
    }
    let units = decomp_source::c_units(c_root)?
        .iter()
        .map(|path| Ok(format!("c/{}", path.strip_prefix(c_root)?.display())))
        .collect::<Result<Vec<_>>>()?;
    ensure!(contract.c_units == units, "compact original C unit inventory differs");
    #[derive(Deserialize)]
    struct Corpus {
        movies: Vec<MovieIdentity>,
    }
    #[derive(Deserialize)]
    struct MovieIdentity {
        sha256: String,
        category: String,
        rom_sha256: String,
    }
    let corpus: Corpus =
        toml::from_str(&String::from_utf8(read(&contract.private_inputs["movies.toml"].path, MAX_RECORD)?)?)?;
    ensure!(
        corpus.movies.len() <= 1024
            && corpus.movies.iter().any(|movie| movie.sha256 == MOVIE_SHA256
                && movie.category == "any%"
                && movie.rom_sha256 == ROM_SHA256),
        "original any% movie manifest identity differs"
    );
    let movie = movie::load(&contract.private_inputs["movie.lsmv"].path)?;
    ensure!(
        movie.rom_sha256.as_deref() == Some(ROM_SHA256) && movie.frames.len() == 129712,
        "compact original movie initialization or frame count differs"
    );
    let mut destinations = BTreeSet::new();
    ensure!(
        !contract.runtime.is_empty() && contract.runtime.len() < 240,
        "compact runtime closure missing or excessive"
    );
    for binding in &contract.runtime {
        let destination = binding.destination.to_string_lossy();
        ensure!(
            destinations.insert(&binding.destination)
                && (destination.starts_with("/lib/")
                    || destination.starts_with("/lib64/")
                    || destination.starts_with("/usr/lib/")
                    || destination == "/usr/include"
                    || ["/usr/bin/cc", "/usr/bin/as", "/usr/bin/ld"].contains(&destination.as_ref())),
            "compact runtime exposes undeclared host directories"
        );
        safe(&binding.source)?;
        let metadata = binding.source.symlink_metadata()?;
        if metadata.is_dir() {
            ensure!(
                ["/usr/include", "/usr/lib/gcc/x86_64-linux-gnu/9/include"].contains(&destination.as_ref())
                    && binding.source == fs::canonicalize(&binding.destination)?,
                "compact runtime directory is not an exact compiler header directory"
            );
        } else {
            ensure!(metadata.is_file(), "compact runtime binding is not an approved regular file");
        }
        ensure!(runtime_hash(&binding.source)? == binding.sha256, "compact installed runtime changed");
    }
    ensure!(contract.historical.len() <= 8, "compact historical closure count exceeded");
    for capture in &contract.historical {
        historical_identity(capture)?;
        ensure!(capture.verdict.bytes <= MAX_RECORD, "historical verdict metadata extent exceeded");
        prepare_reference(&capture.verdict, &mut authenticated)?;
        prepare_reference(&capture.reader, &mut authenticated)?;
        ensure!(
            !capture.private_inputs.is_empty() && capture.private_inputs.len() <= 1024,
            "historical private inputs missing"
        );
        ensure!(
            capture.private_inputs.iter().try_fold(0u64, |total, input| total
                .checked_add(input.bytes)
                .context("historical private input extent overflow"))?
                <= MAX_RAW,
            "historical private input aggregate extent exceeded"
        );
        for reference in &capture.private_inputs {
            prepare_reference(reference, &mut authenticated)?;
        }
    }
    Ok(PreparedCompact {
        contract: contract.clone(),
        dependencies_sha256: sealed(&(
            "original-compact-dependencies-v1",
            &contract.source_files,
            &contract.private_inputs,
            &contract.runtime,
        ))?,
        verification_sha256: sealed(&("original-compact-verification-v1", contract))?,
    })
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ProducerStatus {
    schema: String,
    phase: String,
    status: Option<i32>,
    diagnostic: String,
}

/// Internal entry point. The installed supervisor invokes this only through
/// operation::run_until; every child inherits its namespace, seccomp and tmpfs.
pub(crate) fn producer(args: &[String]) -> Result<()> {
    ensure!(
        Path::new("/trusted-helper").is_file() && Path::new("/program").is_file(),
        "compact producer requires the installed confined operation"
    );
    let mut filesystem = std::mem::MaybeUninit::<libc::statfs>::uninit();
    ensure!(
        unsafe { libc::statfs(c"/work".as_ptr(), filesystem.as_mut_ptr()) } == 0
            && unsafe { filesystem.assume_init() }.f_type == libc::TMPFS_MAGIC,
        "compact producer requires the quota-backed writable mount"
    );
    let phase = args.first().map(String::as_str).context("compact producer phase missing")?;
    ensure!(
        (phase == "native" && args.len() == 1) || (["candidate", "control"].contains(&phase) && args.len() == 3),
        "invalid compact producer arguments"
    );
    let raw = Path::new("/work/tmp/raw");
    fs::create_dir_all(raw)?;
    let mut command = if phase == "native" {
        let mut command = Command::new("/inputs/lsnes");
        command.args([
            "--dumper=INTERNAL-NULL",
            "--rom-a=/inputs/rom.sfc",
            "--length=2240",
            "--lua=/inputs/observe.lua",
            "/inputs/movie.lsmv",
        ]);
        command
    } else {
        let polls: u64 = args[1].parse()?;
        let completions: u64 = args[2].parse()?;
        ensure!(
            (1..=50_000).contains(&polls) && (1..=20_000).contains(&completions),
            "compact subject count outside original bound"
        );
        fs::create_dir(raw.join("fixtures"))?;
        fs::create_dir(raw.join("saves"))?;
        let mut command = Command::new("/inputs/prefix-host");
        command.args([
            "/inputs/rom.sfc",
            "/inputs/causal.inputs",
            &args[1],
            if phase == "candidate" { "causal-polling" } else { "changed-causal-rng" },
            &args[2],
        ]);
        command
    };
    command
        .current_dir(raw)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", "/work/tmp")
        .env("LD_LIBRARY_PATH", "/lib/x86_64-linux-gnu")
        .env("XDG_CONFIG_HOME", raw.join("config"))
        .stdin(Stdio::null())
        .stdout(File::create(raw.join("replay.stdout"))?)
        .stderr(File::create(raw.join("replay.stderr"))?);
    let result = command.status();
    let status = ProducerStatus {
        schema: "translation-compact-producer-v1".into(),
        phase: phase.into(),
        status: result.as_ref().ok().and_then(|status| status.code()),
        diagnostic: result.as_ref().err().map(ToString::to_string).unwrap_or_default(),
    };
    fs::write(raw.join("producer.json"), serde_json::to_vec(&status)?)?;
    pack(raw, Path::new("/work/capture.zst"))?;
    // Failed subjects still return their raw bounded diagnosis. The parent
    // checks status and observations; this exit never grants diagnostic success.
    Ok(())
}

fn files(root: &Path, maximum: u64) -> Result<Vec<(String, PathBuf, u64)>> {
    fn visit(
        root: &Path,
        directory: &Path,
        found: &mut Vec<(String, PathBuf, u64)>,
        total: &mut u64,
        maximum: u64,
    ) -> Result<()> {
        let mut entries = fs::read_dir(directory)?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<std::io::Result<Vec<_>>>()?;
        entries.sort();
        for path in entries {
            ensure!(found.len() < MAX_MEMBERS, "compact output member bound exceeded");
            let metadata = path.symlink_metadata()?;
            ensure!(!metadata.file_type().is_symlink(), "compact output contains a symlink");
            if metadata.is_dir() {
                visit(root, &path, found, total, maximum)?;
            } else {
                ensure!(metadata.is_file() && metadata.len() <= MAX_FILE, "compact output member extent exceeded");
                *total = total.checked_add(metadata.len()).context("compact output extent overflow")?;
                ensure!(*total <= maximum, "compact aggregate raw output quota exceeded");
                found.push((path.strip_prefix(root)?.to_string_lossy().into_owned(), path, metadata.len()));
            }
        }
        Ok(())
    }
    let mut found = Vec::new();
    visit(root, root, &mut found, &mut 0, maximum)?;
    Ok(found)
}

fn pack(root: &Path, output: &Path) -> Result<()> {
    let members = files(root, MAX_RAW)?;
    let mut encoded = zstd::stream::write::Encoder::new(File::create(output)?, 1)?;
    encoded.write_all(b"SMTCAP01")?;
    encoded.write_all(&u32::try_from(members.len())?.to_le_bytes())?;
    for (name, path, length) in members {
        ensure!(relative(&name), "compact output path alias");
        encoded.write_all(&u16::try_from(name.len())?.to_le_bytes())?;
        encoded.write_all(name.as_bytes())?;
        encoded.write_all(&length.to_le_bytes())?;
        let copied = std::io::copy(&mut File::open(path)?, &mut encoded)?;
        ensure!(copied == length, "compact output changed during packing");
    }
    encoded.finish()?.sync_all()?;
    Ok(())
}

#[derive(Default)]
struct RawCapture {
    members: BTreeMap<String, (u64, String)>,
    text: BTreeMap<String, Vec<u8>>,
    states: BTreeMap<String, [u16; 19]>,
    bytes: u64,
}

fn state(bytes: &[u8]) -> Result<[u16; 19]> {
    ensure!(bytes.len() == 0x20000, "compact WRAM snapshot extent differs");
    Ok(FIELDS.map(|(_, offset)| {
        if offset == 0x5b4 { u16::from(bytes[offset]) } else { u16::from_le_bytes([bytes[offset], bytes[offset + 1]]) }
    }))
}

fn unpack(bytes: &[u8]) -> Result<RawCapture> {
    ensure!(bytes.len() as u64 <= MAX_RAW, "compact packed extent exceeded");
    let mut decoder = zstd::stream::read::Decoder::new(bytes)?;
    let mut magic = [0; 8];
    decoder.read_exact(&mut magic)?;
    ensure!(&magic == b"SMTCAP01", "unknown compact capture envelope");
    let mut count = [0; 4];
    decoder.read_exact(&mut count)?;
    let count = u32::from_le_bytes(count) as usize;
    ensure!(count > 0 && count <= MAX_MEMBERS, "compact capture member count exceeded");
    let mut raw = RawCapture::default();
    for _ in 0..count {
        let mut name_length = [0; 2];
        decoder.read_exact(&mut name_length)?;
        let length = usize::from(u16::from_le_bytes(name_length));
        ensure!((1..=4096).contains(&length), "compact capture path extent exceeded");
        let mut name = vec![0; length];
        decoder.read_exact(&mut name)?;
        let name = String::from_utf8(name)?;
        ensure!(relative(&name) && !raw.members.contains_key(&name), "compact capture path alias or duplicate");
        let mut length = [0; 8];
        decoder.read_exact(&mut length)?;
        let length = u64::from_le_bytes(length);
        raw.bytes = raw.bytes.checked_add(length).context("compact decoded extent overflow")?;
        ensure!(length <= MAX_FILE && raw.bytes <= MAX_RAW, "compact decoded aggregate quota exceeded");
        let mut bytes = vec![0; usize::try_from(length)?];
        decoder.read_exact(&mut bytes)?;
        raw.members.insert(name.clone(), (length, sha256(&bytes)));
        if name.ends_with(".wram") {
            raw.states.insert(name.clone(), state(&bytes)?);
        }
        if ["events.csv", "events.txt", "replay.stdout", "replay.stderr", "producer.json"].contains(&name.as_str()) {
            ensure!(length <= MAX_RECORD, "compact text observation extent exceeded");
            raw.text.insert(name, bytes);
        }
    }
    ensure!(decoder.read(&mut [0; 1])? == 0, "compact envelope has unaccounted trailing bytes");
    Ok(raw)
}

fn complete(raw: &RawCapture, phase: &str) -> Result<()> {
    let status: ProducerStatus =
        serde_json::from_slice(raw.text.get("producer.json").context("compact producer status missing")?)?;
    ensure!(
        status.schema == "translation-compact-producer-v1" && status.phase == phase,
        "compact producer identity differs"
    );
    ensure!(
        status.status == Some(0) && status.diagnostic.is_empty(),
        "compact {phase} process failed (status {:?}): {} {}",
        status.status,
        status.diagnostic,
        String::from_utf8_lossy(raw.text.get("replay.stderr").map(Vec::as_slice).unwrap_or_default())
            .chars()
            .take(2048)
            .collect::<String>()
    );
    Ok(())
}

#[derive(Clone, Debug)]
struct Event {
    frame: u64,
    event: u64,
    state: [u16; 19],
    pad: Option<u16>,
}

struct NativeTrace {
    polls: Vec<Event>,
    completed: Vec<Event>,
    snapshots: BTreeMap<String, [u16; 19]>,
    frames: u64,
}

fn native_trace(reader: impl BufRead, frames: u64, compact: bool) -> Result<NativeTrace> {
    let mut lines = reader.lines();
    ensure!(
        lines.next().transpose()?.as_deref()
            == Some(&format!("frame,event,kind,address,value,{}", FIELDS.map(|(name, _)| name).join(","))),
        "original native event columns differ"
    );
    let mut trace = NativeTrace { polls: vec![], completed: vec![], snapshots: BTreeMap::new(), frames: 0 };
    let mut entry = false;
    let mut registers = BTreeMap::<u32, Vec<u16>>::new();
    let mut material = None;
    let mut prior_frame = 0;
    for (index, line) in lines.enumerate() {
        ensure!(index < 10_000_000, "native event count exceeded");
        let line = line?;
        ensure!(line.len() <= 4096, "native event extent exceeded");
        let columns = line.split(',').collect::<Vec<_>>();
        ensure!(columns.len() == 24, "native event extent differs");
        let frame: u64 = columns[0].parse()?;
        let ordinal: u64 = columns[1].parse()?;
        ensure!(
            ordinal == index as u64 + 1 && frame >= prior_frame && frame <= frames,
            "native event order or frame bound differs"
        );
        prior_frame = frame;
        let kind = columns[2];
        let address: u32 = columns[3].parse()?;
        let value: u16 = columns[4].parse()?;
        let mut observed = [0u16; 19];
        for (field, column) in observed.iter_mut().zip(&columns[5..]) {
            *field = column.parse()?;
        }
        let event = Event { frame, event: ordinal, state: observed, pad: None };
        match kind {
            "controller-entry" => {
                ensure!(!entry, "nested native controller poll");
                entry = true;
                registers.clear();
            }
            "controller-register" if entry => {
                registers.entry(address & 0xffff).or_default().push(value);
            }
            "controller-return" => {
                ensure!(entry, "native controller return has no entry");
                let low = registers.get(&0x4218).context("native low controller read missing")?;
                let high = registers.get(&0x4219).context("native high controller read missing")?;
                ensure!(
                    low.len() == 1 && high.len() == 1 && low[0] < 256 && high[0] < 256,
                    "native controller read pair incomplete"
                );
                ensure!(
                    (!registers.contains_key(&0x421a) && !registers.contains_key(&0x421b))
                        || (registers.get(&0x421a).map(Vec::as_slice) == Some(&[0])
                            && registers.get(&0x421b).map(Vec::as_slice) == Some(&[0])),
                    "native second controller read pair differs"
                );
                trace.polls.push(Event { pad: Some(low[0] | (high[0] << 8)), ..event.clone() });
                entry = false;
            }
            "dispatch-complete" => trace.completed.push(event.clone()),
            "frame-complete" => {
                trace.frames += 1;
                ensure!(frame == trace.frames, "native completed frame order differs");
            }
            _ => {}
        }
        let changed = if kind == "dispatch-complete" {
            let identity = [observed[0], observed[1], observed[4], observed[5], observed[6]];
            let changed = material != Some(identity);
            material = Some(identity);
            changed
        } else {
            false
        };
        let expected = if compact {
            kind == "boot-clear"
                || (frame <= 320 && ["dispatch-complete", "controller-entry", "controller-return"].contains(&kind))
                || changed
                || (kind == "frame-complete" && frame == frames)
        } else {
            ["boot-clear", "dispatch-complete", "controller-entry", "controller-return"].contains(&kind)
        };
        if expected {
            trace.snapshots.insert(format!("state-{ordinal}.wram"), observed);
        }
    }
    ensure!(
        trace.frames == frames && !trace.polls.is_empty() && !trace.completed.is_empty(),
        "native capture incomplete: {} of {frames} frames",
        trace.frames
    );
    Ok(trace)
}

struct SubjectTrace {
    polls: Vec<Event>,
    completed: Vec<Event>,
}

fn subject_trace(reader: impl BufRead, json_lines: bool) -> Result<SubjectTrace> {
    let mut trace = SubjectTrace { polls: vec![], completed: vec![] };
    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        ensure!(index < 200_000 && line.len() < 8192, "compact subject event bound exceeded");
        let (ordinal, kind, state) = if json_lines {
            let row: serde_json::Value = serde_json::from_str(&line)?;
            let mut values = [0; 19];
            for ((name, _), value) in FIELDS.iter().zip(&mut values) {
                *value = u16::try_from(row["state"][name].as_u64().context("historical subject state missing")?)?;
            }
            (
                row["event"].as_u64().context("subject event missing")?,
                row["kind"].as_str().context("subject kind missing")?.to_owned(),
                values,
            )
        } else {
            let columns = line.split_whitespace().collect::<Vec<_>>();
            ensure!(columns.len() == 23, "compact subject event extent differs");
            let mut values = [0; 19];
            for (value, column) in values.iter_mut().zip(&columns[4..]) {
                *value = column.parse()?;
            }
            (columns[0].parse()?, columns[2].into(), values)
        };
        ensure!(ordinal == index as u64 + 1, "subject event order differs");
        let event = Event { frame: 0, event: ordinal, state, pad: None };
        match kind.as_str() {
            "controller-return" => trace.polls.push(event),
            "dispatch-complete" => trace.completed.push(event),
            _ => {}
        }
    }
    Ok(trace)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct CompactComparison {
    pub ordinal: u64,
    pub source_frame: u64,
    pub source_event: u64,
    pub candidate_event: u64,
    pub native_polls: u64,
    pub candidate_polls: u64,
    pub differences: BTreeMap<String, [u16; 2]>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct CompactSummary {
    pub frames: u64,
    pub native_completed_updates: u64,
    pub candidate_completed_updates: u64,
    pub polls: u64,
    pub prelude_polls: u64,
    pub matching_prefix: u64,
    pub changed_rng_detected: bool,
    pub matching_frontier_frame: Option<u64>,
    pub first_divergence_frame: Option<u64>,
    pub comparisons: Vec<CompactComparison>,
}

fn comparisons(native: &NativeTrace, candidate: &SubjectTrace, control: &SubjectTrace) -> Result<CompactSummary> {
    ensure!(
        candidate.polls.iter().map(|event| event.state[12]).eq(native.polls.iter().map(|event| event.state[12])),
        "causal controller samples changed, missing or reordered"
    );
    ensure!(
        control.polls.iter().map(|event| event.state[12]).eq(native.polls.iter().map(|event| event.state[12])),
        "independent changed-gameplay control poll sequence differs"
    );
    let rows = native
        .completed
        .iter()
        .zip(&candidate.completed)
        .enumerate()
        .map(|(index, (a, b))| {
            let differences = FIELDS
                .iter()
                .enumerate()
                .filter(|(index, _)| ![15, 16, 18].contains(index))
                .filter_map(|(index, (name, _))| {
                    (a.state[index] != b.state[index]).then(|| ((*name).into(), [a.state[index], b.state[index]]))
                })
                .collect();
            CompactComparison {
                ordinal: index as u64 + 1,
                source_frame: a.frame,
                source_event: a.event,
                candidate_event: b.event,
                native_polls: native.polls.partition_point(|poll| poll.event < a.event) as u64,
                candidate_polls: candidate.polls.partition_point(|poll| poll.event < b.event) as u64,
                differences,
            }
        })
        .collect::<Vec<_>>();
    let matching =
        rows.iter().take_while(|row| row.native_polls == row.candidate_polls && row.differences.is_empty()).count();
    let changed_rng_detected =
        candidate.completed.iter().zip(&control.completed).any(|(a, b)| a.state[11] != b.state[11]);
    Ok(CompactSummary {
        frames: native.frames,
        native_completed_updates: native.completed.len() as u64,
        candidate_completed_updates: candidate.completed.len() as u64,
        polls: native.polls.len() as u64,
        prelude_polls: native.polls.partition_point(|poll| poll.event < native.completed[0].event) as u64,
        matching_prefix: matching as u64,
        changed_rng_detected,
        matching_frontier_frame: matching.checked_sub(1).map(|index| native.completed[index].frame),
        first_divergence_frame: native.completed.get(matching).map(|event| event.frame),
        comparisons: rows,
    })
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct StrictRuntime {
    pub helper: PathBuf,
    pub producer: PathBuf,
    pub operation_root: PathBuf,
    pub pipeline_root: PathBuf,
    pub workers: usize,
    pub gate_timeout_seconds: u64,
    pub deadline_unix_ms: Option<u64>,
    pub max_output_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct CompactReceipt {
    pub prepared_sha256: String,
    pub candidate_sha256: String,
    pub context: OperationContext,
    pub verdict: Asset,
    pub passed: bool,
    pub diagnostic: String,
    pub evidence_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct CompactInvocation {
    pub label: String,
    pub context: OperationContext,
    pub program: Program,
    pub limits: RunLimits,
    pub reserved_bytes: u64,
    pub result_sha256: String,
    pub references: Vec<Reference>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct HistoricalAuthentication {
    pub capture: HistoricalCapture,
    pub artifact_count: usize,
    pub artifact_bytes: u64,
    pub inventory_sha256: String,
    pub summary: Option<CompactSummary>,
    pub original_status: String,
    pub fresh_capture: bool,
    pub reader_execution: Option<CompactInvocation>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct CompactVerdict {
    pub schema: String,
    pub evidence_kind: String,
    pub prepared_sha256: String,
    pub candidate_sha256: String,
    pub context: OperationContext,
    pub runtime: StrictRuntime,
    pub requested_frames: u64,
    pub experiment_budget_seconds: u64,
    pub experiment_started_unix_ms: Option<u64>,
    pub experiment_deadline_unix_ms: Option<u64>,
    pub experiment_finished_unix_ms: Option<u64>,
    pub source_snapshots: BTreeMap<String, Reference>,
    pub input_snapshots: BTreeMap<String, Reference>,
    pub projections: BTreeMap<String, Reference>,
    pub historical_dependencies: Vec<HistoricalCapture>,
    pub historical: Vec<HistoricalAuthentication>,
    pub operations: Vec<CompactInvocation>,
    pub summary: Option<CompactSummary>,
    pub passed: bool,
    pub diagnostic: String,
    pub original_rom_parity: bool,
    pub live_replay: bool,
    pub limitations: Vec<String>,
    pub reserved_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Intent {
    prepared_sha256: String,
    candidate_sha256: String,
    context: OperationContext,
    runtime: StrictRuntime,
    deadline_unix_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct InvocationIntent {
    context: OperationContext,
    request_sha256: String,
    limits: RunLimits,
    reserved_bytes: u64,
    started_unix_ms: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Experiment {
    started_unix_ms: u64,
    deadline_unix_ms: u64,
}

fn authenticate_experiment(experiment: &Experiment, gate_deadline: u64) -> Result<()> {
    ensure!(
        experiment.started_unix_ms < experiment.deadline_unix_ms
            && experiment.deadline_unix_ms <= experiment.started_unix_ms.saturating_add(SECONDS * 1000)
            && experiment.deadline_unix_ms <= gate_deadline,
        "original compact whole-experiment 60-second interval differs"
    );
    Ok(())
}

fn limitations() -> Vec<String> {
    [
        "Infrastructure causal diagnostic only; Live replay and original-ROM parity remain unaccepted.",
        "Completed state comparison preserves the original 16 semantic words; raw NMI counts and waiting byte remain excluded.",
        "The full observer remains the default. Compact capture omits APU reads, standalone primitive and omitted-wait controls.",
        "Original compact snapshot policy retains all early, boot-clear, material-transition and final-frame WRAM snapshots.",
        "Historical captures preserve their original readers and private inputs; no historical pass counts as a fresh experiment.",
    ].map(str::to_owned).to_vec()
}

fn candidate(prepared: &PreparedCompact, checkout: &Path) -> Result<String> {
    ensure!(
        prepared.verification_sha256 == sealed(&("original-compact-verification-v1", &prepared.contract))?
            && prepared.dependencies_sha256
                == sealed(&(
                    "original-compact-dependencies-v1",
                    &prepared.contract.source_files,
                    &prepared.contract.private_inputs,
                    &prepared.contract.runtime
                ))?,
        "compact prepared identity changed"
    );
    for (name, expected) in &prepared.contract.source_files {
        ensure!(
            retention::seal_file(&checkout.join(name), MAX_RECORD)?.sha256 == *expected,
            "original compact candidate/source dependency changed: {name}"
        );
    }
    sealed(&prepared.contract.source_files)
}

fn observer(template: &str) -> String {
    let mut hooks = String::new();
    for (kind, address) in HOOKS {
        hooks.push_str(&format!(
            "memory.registerexec('BUS',{address},function(a,v) observe('{kind}',{address},v) end)\n"
        ));
    }
    for address in [0x4218, 0x4219, 0x421a, 0x421b, 0x804218, 0x804219, 0x80421a, 0x80421b] {
        hooks.push_str(&format!(
            "memory.registerread('BUS',{address},function(a,v) observe('controller-register',{address},v) end)\n"
        ));
    }
    template
        .replace("__OUTPUT__", "/work/tmp/raw")
        .replace("__FRAMES__", "2240")
        .replace("__FIELDS__", &FIELDS.map(|(name, _)| name).join(","))
        .replace("__OFFSETS__", &FIELDS.map(|(_, address)| address.to_string()).join(","))
        .replace("__HOOKS__", &hooks)
}

fn limits(label: &str) -> RunLimits {
    let mib = match label {
        "native" => 256,
        "compile" => 64,
        "candidate" | "control" => 384,
        _ => 8,
    };
    RunLimits {
        timeout: Duration::from_secs(if label.starts_with("history-") { 3600 } else { SECONDS }),
        writable_bytes: mib * 1024 * 1024,
        stdout_bytes: 64 * 1024,
        stderr_bytes: 64 * 1024,
        stdin_bytes: 0,
    }
}

fn recipe(
    prepared: &PreparedCompact,
    output: &Path,
    runtime: &StrictRuntime,
    label: &str,
    counts: Option<(u64, u64)>,
) -> Result<Program> {
    let mut program = Program {
        executable: runtime.producer.clone(),
        helper: runtime.helper.clone(),
        runtime: prepared
            .contract
            .runtime
            .iter()
            .filter(|binding| {
                label == "compile"
                    || binding.destination.starts_with("/lib")
                    || binding.destination.starts_with("/lib64")
            })
            .map(|binding| ReadOnlyBind { source: binding.source.clone(), destination: binding.destination.clone() })
            .collect(),
        inputs: vec![],
        arguments: vec!["translation-compact-producer".into(), label.into()],
        outputs: vec!["capture.zst".into()],
    };
    let mut bind = |source: PathBuf, destination: &str| {
        program.inputs.push(ReadOnlyBind { source, destination: destination.into() })
    };
    match label {
        "native" => {
            bind(output.join("private/rom.sfc"), "/inputs/rom.sfc");
            bind(output.join("private/movie.lsmv"), "/inputs/movie.lsmv");
            bind(prepared.contract.private_inputs["lsnes"].path.clone(), "/inputs/lsnes");
            bind(output.join("observe.lua"), "/inputs/observe.lua");
        }
        "compile" => {
            bind(output.join("private/c"), "/inputs/c");
            bind(output.join("source/tools/decomp"), "/inputs/host");
            program.executable = prepared
                .contract
                .runtime
                .iter()
                .find(|binding| binding.destination == Path::new("/usr/bin/cc"))
                .context("compact C compiler missing")?
                .source
                .clone();
            program.arguments = [
                "-std=gnu11",
                "-O2",
                "-fno-strict-aliasing",
                "-ffunction-sections",
                "-fdata-sections",
                "-iquote",
                "/inputs/c/src",
                "-B/usr/lib/gcc/x86_64-linux-gnu/9/",
                "-B/usr/bin/",
            ]
            .map(str::to_owned)
            .to_vec();
            program.arguments.extend(prepared.contract.c_units.iter().map(|name| format!("/inputs/{name}")));
            program.arguments.extend(["/inputs/host/prefix_host.c".into(), "-Wl,--gc-sections".into()]);
            for symbol in [
                "Call",
                "cpu_runOpcode",
                "DecompressToMem",
                "ScreenOn",
                "WaitUntilEndOfVblankAndClearHdma",
                "RtlApuUpload",
                "ReadJoypadInputs",
                "Vector_NMI",
                "NextRandom",
            ] {
                program.arguments.push(format!("-Wl,--wrap={symbol}"));
            }
            program.arguments.extend(["-lm".into(), "-o".into(), "/work/prefix-host".into()]);
            program.outputs = vec!["prefix-host".into()];
        }
        "candidate" | "control" => {
            bind(output.join("private/rom.sfc"), "/inputs/rom.sfc");
            bind(output.join("prefix-host"), "/inputs/prefix-host");
            bind(output.join("causal.inputs"), "/inputs/causal.inputs");
            let (polls, completions) = counts.context("compact subject counts missing")?;
            program.arguments.extend([polls.to_string(), completions.to_string()]);
        }
        _ => bail!("unknown compact fixed phase"),
    }
    Ok(program)
}

fn historical_program(capture: &HistoricalCapture, helper: &Path) -> Result<Program> {
    historical_identity(capture)?;
    ensure!(capture.profile == "compact", "historical source reader cannot execute as a runtime");
    let mut runtime = confinement::helper_runtime()?;
    // The exact old ELF initializes its identity through current_exe(). This
    // opt-in makes only a synthetic symlink to its existing /program bind;
    // the confinement launcher must never mount procfs or expose host processes.
    runtime.push(ReadOnlyBind { source: capture.reader.path.clone(), destination: "/proc/self/exe".into() });
    Ok(Program {
        executable: capture.reader.path.clone(),
        helper: helper.into(),
        runtime,
        inputs: vec![ReadOnlyBind {
            source: capture.verdict.path.parent().context("historical capture root missing")?.into(),
            destination: "/inputs/capture".into(),
        }],
        arguments: vec!["decomp-prefix-authenticate".into(), "/inputs/capture".into()],
        outputs: vec![],
    })
}

struct Gate<'a> {
    output: &'a Path,
    context: &'a OperationContext,
    runtime: &'a StrictRuntime,
    budget: ByteBudget,
    invocations: Vec<CompactInvocation>,
}

impl Gate<'_> {
    fn invoke(&mut self, label: &str, program: Program, deadline: u64) -> Result<operation::OperationResult> {
        let mut context = self.context.clone();
        context.operation_key = sealed(&(&self.context.operation_key, self.output, label))?;
        let declared = limits(label);
        let request_sha256 = sealed(&(&program, &declared))?;
        let path = self.output.join(format!("{label}-invocation.toml"));
        let existed = path.is_file();
        let mut lease = None;
        let intent: InvocationIntent = if existed {
            let saved: InvocationIntent = toml::from_str(&String::from_utf8(read(&path, MAX_RECORD)?)?)?;
            ensure!(
                saved.context == context
                    && saved.request_sha256 == request_sha256
                    && saved.limits.timeout > Duration::ZERO
                    && saved.limits.timeout <= declared.timeout
                    && saved.limits.writable_bytes == declared.writable_bytes
                    && saved.started_unix_ms < deadline
                    && saved.limits.timeout <= Duration::from_millis(deadline - saved.started_unix_ms),
                "compact invocation retry identity changed"
            );
            self.budget.reserve(saved.reserved_bytes)?;
            saved
        } else {
            ensure!(
                now_ms()? < deadline,
                "original compact whole-experiment 60-second deadline exhausted before {label}"
            );
            lease = Some(acquire_lease_until(&self.runtime.pipeline_root, self.runtime.workers, deadline)?.0);
            let reservation = operation::reserve_bytes_until(&program, &declared, 0, deadline)?
                .checked_add(declared.writable_bytes)
                .and_then(|bytes| bytes.checked_add(1024 * 1024))
                .context("compact aggregate reservation overflow")?;
            self.budget.reserve(reservation)?;
            let started_unix_ms = now_ms()?;
            let remaining = deadline.saturating_sub(started_unix_ms);
            ensure!(remaining > 0, "original compact whole-experiment deadline exhausted before {label} launch");
            let mut actual = declared;
            actual.timeout = actual.timeout.min(Duration::from_millis(remaining));
            let intent = InvocationIntent {
                context: context.clone(),
                request_sha256,
                limits: actual,
                reserved_bytes: reservation,
                started_unix_ms,
            };
            publish(&path, toml::to_string_pretty(&intent)?.as_bytes(), false)?;
            intent
        };
        let directory = operation::directory(&self.runtime.operation_root, &context)?;
        let result = if existed {
            ensure!(
                directory.join("intent.json").is_file(),
                "compact invocation stopped before durable producer launch; unchanged resume cannot allocate a capture"
            );
            if directory.join("result.json").is_file() {
                operation::authenticate_request(&self.runtime.operation_root, &context, &program, &intent.limits, &[])?
            } else if now_ms()? >= deadline {
                operation::stop(&self.runtime.operation_root, &context)?;
                operation::authenticate_request(&self.runtime.operation_root, &context, &program, &intent.limits, &[])?
            } else {
                operation::run_until(&self.runtime.operation_root, &context, &program, &intent.limits, &[], deadline)?
            }
        } else {
            operation::run_until(&self.runtime.operation_root, &context, &program, &intent.limits, &[], deadline)?
        };
        drop(lease);
        self.invocations.push(CompactInvocation {
            label: label.into(),
            context: context.clone(),
            program,
            limits: intent.limits,
            reserved_bytes: intent.reserved_bytes,
            result_sha256: result.result_sha256.clone(),
            references: operation::references(&self.runtime.operation_root, &context)?,
        });
        Ok(result)
    }

    fn project(&self, name: &str, bytes: &[u8], executable: bool) -> Result<Reference> {
        let path = self.output.join(name);
        publish(&path, bytes, executable)?;
        retention::seal_file(&path, MAX_RAW)
    }
}

fn phase_capture(result: &operation::OperationResult, phase: &str) -> Result<RawCapture> {
    ensure!(result.passed && result.status_raw == Some(0), "confined compact {phase} failed: {}", result.diagnostic);
    let raw = unpack(result.outputs.get(Path::new("capture.zst")).context("compact producer omitted raw capture")?)?;
    complete(&raw, phase)?;
    Ok(raw)
}

fn validate_native(raw: &RawCapture) -> Result<NativeTrace> {
    let logs = ["replay.stdout", "replay.stderr"]
        .map(|name| String::from_utf8_lossy(raw.text.get(name).map(Vec::as_slice).unwrap_or_default()).into_owned())
        .join("\n");
    ensure!(
        ["Finished!", "bsnes v085 (Compatibility core)", "snes_ntsc"].iter().all(|text| logs.contains(text)),
        "native completion/core/region missing"
    );
    let trace = native_trace(raw.text.get("events.csv").context("native events missing")?.as_slice(), FRAMES, true)?;
    for (name, expected) in &trace.snapshots {
        ensure!(raw.states.get(name) == Some(expected), "native completed/retained WRAM state differs: {name}");
    }
    Ok(trace)
}

fn validate_subject(raw: &RawCapture) -> Result<SubjectTrace> {
    let trace = subject_trace(raw.text.get("events.txt").context("compact subject events missing")?.as_slice(), false)?;
    ensure!(!trace.completed.is_empty(), "compact subject completed no dispatches");
    for (index, event) in trace.completed.iter().enumerate() {
        ensure!(
            raw.states.get(&format!("complete-{:04}.wram", index + 1)) == Some(&event.state),
            "compact completed-state snapshot differs at ordinal {}",
            index + 1
        );
    }
    Ok(trace)
}

pub(crate) fn run(
    prepared: &PreparedCompact,
    checkout: &Path,
    output: &Path,
    runtime: &StrictRuntime,
    context: &OperationContext,
) -> Result<CompactReceipt> {
    safe(output)?;
    let receipt_path = output.join("compact-receipt.toml");
    let verdict_path = output.join("compact-verdict.toml");
    if receipt_path.is_file() || verdict_path.is_file() {
        // Finalization authenticates the stored production interval even after
        // it has expired. This branch cannot set up or launch another producer.
        return decomp_source::context::reconcile(MAX_HISTORY, || {
            if receipt_path.is_file() {
                let receipt: CompactReceipt = toml::from_str(&String::from_utf8(read(&receipt_path, MAX_RECORD)?)?)?;
                let verdict = authenticate(prepared, checkout, &receipt)?;
                ensure!(
                    verdict.context == *context && verdict.runtime == *runtime,
                    "compact completed retry belongs to another runtime or operation"
                );
                return Ok(receipt);
            }
            let verdict: CompactVerdict = toml::from_str(&String::from_utf8(read(&verdict_path, MAX_RECORD)?)?)?;
            ensure!(
                verdict.context == *context && verdict.runtime == *runtime,
                "compact finalization recovery changed identity"
            );
            finish(prepared, checkout, &verdict_path, &verdict)
        });
    }
    let candidate_sha256 = candidate(prepared, checkout)?;
    ensure!(
        (1..=3600).contains(&runtime.gate_timeout_seconds)
            && (1..=64).contains(&runtime.workers)
            && (MAX_RECORD..=MAX_HISTORY).contains(&runtime.max_output_bytes),
        "compact installed aggregate gate budgets invalid"
    );
    safe(&runtime.operation_root)?;
    safe(&runtime.pipeline_root)?;
    fs::create_dir_all(output)?;
    let intent_path = output.join("compact-intent.toml");
    let prepared_sha256 = sealed(prepared)?;
    let intent = if intent_path.is_file() {
        let intent: Intent = toml::from_str(&String::from_utf8(read(&intent_path, MAX_RECORD)?)?)?;
        ensure!(
            intent.prepared_sha256 == prepared_sha256
                && intent.candidate_sha256 == candidate_sha256
                && intent.runtime == *runtime
                && intent.context == *context,
            "compact interrupted retry identity changed"
        );
        intent
    } else {
        let deadline =
            now_ms()?.checked_add(runtime.gate_timeout_seconds * 1000).context("compact deadline overflow")?;
        let intent = Intent {
            prepared_sha256: prepared_sha256.clone(),
            candidate_sha256: candidate_sha256.clone(),
            context: context.clone(),
            runtime: runtime.clone(),
            deadline_unix_ms: runtime.deadline_unix_ms.map_or(deadline, |outer| outer.min(deadline)),
        };
        publish(&intent_path, toml::to_string_pretty(&intent)?.as_bytes(), false)?;
        intent
    };
    let mut gate =
        Gate { output, context, runtime, budget: ByteBudget::new(runtime.max_output_bytes), invocations: vec![] };
    gate.budget.reserve(4 * MAX_RECORD)?;
    let mut retained_inputs = BTreeMap::new();
    for capture in &prepared.contract.historical {
        for reference in
            std::iter::once(&capture.verdict).chain(std::iter::once(&capture.reader)).chain(&capture.private_inputs)
        {
            retained_inputs.insert(reference.path.clone(), reference.bytes);
        }
    }
    gate.budget.reserve(retained_inputs.values().try_fold(0u64, |total, bytes| {
        total.checked_add(*bytes).context("historical dependency reservation overflow")
    })?)?;
    let mut verdict = CompactVerdict {
        schema: "translation-compact-verdict-v1".into(),
        evidence_kind: "infrastructure-diagnostic".into(),
        prepared_sha256,
        candidate_sha256,
        context: context.clone(),
        runtime: runtime.clone(),
        requested_frames: FRAMES,
        experiment_budget_seconds: SECONDS,
        experiment_started_unix_ms: None,
        experiment_deadline_unix_ms: None,
        experiment_finished_unix_ms: None,
        source_snapshots: BTreeMap::new(),
        input_snapshots: BTreeMap::new(),
        projections: BTreeMap::new(),
        historical_dependencies: prepared.contract.historical.clone(),
        historical: vec![],
        operations: vec![],
        summary: None,
        passed: false,
        diagnostic: String::new(),
        original_rom_parity: false,
        live_replay: false,
        limitations: limitations(),
        reserved_bytes: 0,
    };
    let result = (|| -> Result<()> {
        for (name, expected) in &prepared.contract.source_files {
            let bytes = read(&checkout.join(name), MAX_RECORD)?;
            ensure!(sha256(&bytes) == *expected, "compact source changed before snapshot");
            gate.budget.reserve(bytes.len() as u64 * 2)?;
            let reference = gate.project(&format!("source/{name}"), &bytes, false)?;
            verdict.source_snapshots.insert(name.clone(), reference);
        }
        for (name, source) in &prepared.contract.private_inputs {
            retention::authenticate_reference(source, MAX_FILE)?;
            if name == "lsnes" || name == "source.tar.gz" {
                continue;
            }
            gate.budget.reserve(source.bytes * 2)?;
            let reference = gate.project(&format!("private/{name}"), &read(&source.path, MAX_FILE)?, false)?;
            verdict.input_snapshots.insert(name.clone(), reference);
        }
        for capture in &prepared.contract.historical {
            let mut checked = authenticate_historical(capture)?;
            if capture.profile == "compact" && checked.original_status == "nonaccepting" {
                let label = format!("history-{}", verdict.historical.len());
                let program = historical_program(capture, &runtime.helper)?;
                let result = gate.invoke(&label, program, intent.deadline_unix_ms)?;
                ensure!(
                    result.passed
                        && result.status_raw == Some(0)
                        && String::from_utf8_lossy(&result.stdout).contains("acceptance remains false"),
                    "original compact reader rejected retained capture: {}",
                    result.diagnostic
                );
                checked.reader_execution = gate.invocations.last().cloned();
            }
            verdict.historical.push(checked);
        }
        for binding in &prepared.contract.runtime {
            ensure!(runtime_hash(&binding.source)? == binding.sha256, "compact runtime changed before experiment");
        }
        let experiment_path = output.join("experiment-deadline.toml");
        let experiment: Experiment = if experiment_path.is_file() {
            toml::from_str(&String::from_utf8(read(&experiment_path, MAX_RECORD)?)?)?
        } else {
            let started_unix_ms = now_ms()?;
            let experiment = Experiment {
                started_unix_ms,
                deadline_unix_ms: started_unix_ms.saturating_add(SECONDS * 1000).min(intent.deadline_unix_ms),
            };
            publish(&experiment_path, toml::to_string(&experiment)?.as_bytes(), false)?;
            experiment
        };
        authenticate_experiment(&experiment, intent.deadline_unix_ms)?;
        verdict.experiment_started_unix_ms = Some(experiment.started_unix_ms);
        verdict.experiment_deadline_unix_ms = Some(experiment.deadline_unix_ms);
        let template = String::from_utf8(read(
            &output.join("source/crates/pipeline/src/prefix_compact_observer.lua.in"),
            MAX_RECORD,
        )?)?;
        verdict
            .projections
            .insert("observe.lua".into(), gate.project("observe.lua", observer(&template).as_bytes(), false)?);
        let native_result =
            gate.invoke("native", recipe(prepared, output, runtime, "native", None)?, experiment.deadline_unix_ms)?;
        if let Some(bytes) = native_result.outputs.get(Path::new("capture.zst")) {
            verdict.projections.insert("native.zst".into(), gate.project("native.zst", bytes, false)?);
        }
        let native = validate_native(&phase_capture(&native_result, "native")?)?;
        let inputs = native
            .polls
            .iter()
            .map(|event| format!("{:04x}\n", event.pad.expect("validated native read pair")))
            .collect::<String>();
        verdict.projections.insert("causal.inputs".into(), gate.project("causal.inputs", inputs.as_bytes(), false)?);
        let build =
            gate.invoke("compile", recipe(prepared, output, runtime, "compile", None)?, experiment.deadline_unix_ms)?;
        ensure!(build.passed && build.status_raw == Some(0), "original compact C build failed: {}", build.diagnostic);
        let binary = build.outputs.get(Path::new("prefix-host")).context("compact C build omitted host executable")?;
        verdict.projections.insert("prefix-host".into(), gate.project("prefix-host", binary, true)?);
        let counts = Some((native.polls.len() as u64, native.completed.len() as u64));
        let candidate_result = gate.invoke(
            "candidate",
            recipe(prepared, output, runtime, "candidate", counts)?,
            experiment.deadline_unix_ms,
        )?;
        if let Some(bytes) = candidate_result.outputs.get(Path::new("capture.zst")) {
            verdict.projections.insert("candidate.zst".into(), gate.project("candidate.zst", bytes, false)?);
        }
        let candidate = validate_subject(&phase_capture(&candidate_result, "candidate")?)?;
        let control_result =
            gate.invoke("control", recipe(prepared, output, runtime, "control", counts)?, experiment.deadline_unix_ms)?;
        if let Some(bytes) = control_result.outputs.get(Path::new("capture.zst")) {
            verdict.projections.insert("control.zst".into(), gate.project("control.zst", bytes, false)?);
        }
        let control = validate_subject(&phase_capture(&control_result, "control")?)?;
        verdict.summary = Some(comparisons(&native, &candidate, &control)?);
        let summary = verdict.summary.as_ref().unwrap();
        ensure!(
            summary.matching_prefix == summary.native_completed_updates
                && summary.native_completed_updates == summary.candidate_completed_updates
                && control.completed.len() == native.completed.len()
                && summary.changed_rng_detected,
            "original compact completed-state or ordered-poll comparison/control differs"
        );
        verdict.experiment_finished_unix_ms = Some(now_ms()?);
        ensure!(
            verdict.experiment_finished_unix_ms <= verdict.experiment_deadline_unix_ms,
            "original compact whole-experiment 60-second deadline exhausted"
        );
        candidate_identity_unchanged(prepared, checkout, &verdict.candidate_sha256)?;
        verdict.passed = true;
        Ok(())
    })();
    if let Err(error) = result {
        if error.downcast_ref::<translation_execution::ExecutionPending>().is_some() {
            return Err(error);
        }
        verdict.passed = false;
        verdict.diagnostic = format!("{error:#}").chars().take(4096).collect();
    }
    if verdict.experiment_started_unix_ms.is_some() && verdict.experiment_finished_unix_ms.is_none() {
        verdict.experiment_finished_unix_ms = Some(now_ms()?);
    }
    verdict.operations = gate.invocations;
    verdict.reserved_bytes = gate.budget.used();
    let encoded = toml::to_string_pretty(&verdict)?;
    ensure!(encoded.len() as u64 <= MAX_RECORD, "compact verdict metadata extent exceeded");
    publish(&verdict_path, encoded.as_bytes(), false)?;
    finish(prepared, checkout, &verdict_path, &verdict)
}

fn finish(
    prepared: &PreparedCompact,
    checkout: &Path,
    verdict_path: &Path,
    verdict: &CompactVerdict,
) -> Result<CompactReceipt> {
    let mut receipt = CompactReceipt {
        prepared_sha256: verdict.prepared_sha256.clone(),
        candidate_sha256: verdict.candidate_sha256.clone(),
        context: verdict.context.clone(),
        verdict: Asset { file: verdict_path.into(), sha256: retention::seal_file(verdict_path, MAX_RECORD)?.sha256 },
        passed: verdict.passed,
        diagnostic: verdict.diagnostic.clone(),
        evidence_bytes: 0,
    };
    receipt.evidence_bytes = references(&receipt)?.iter().try_fold(0u64, |total, reference| {
        total.checked_add(reference.bytes).context("compact evidence extent overflow")
    })?;
    ensure!(receipt.evidence_bytes <= verdict.reserved_bytes, "compact evidence exceeded aggregate pre-reservation");
    authenticate(prepared, checkout, &receipt)?;
    let receipt_path = verdict_path.parent().context("compact verdict parent missing")?.join("compact-receipt.toml");
    publish(&receipt_path, toml::to_string_pretty(&receipt)?.as_bytes(), false)?;
    Ok(receipt)
}

fn candidate_identity_unchanged(prepared: &PreparedCompact, checkout: &Path, expected: &str) -> Result<()> {
    ensure!(candidate(prepared, checkout)? == expected, "compact candidate changed during original diagnostic");
    Ok(())
}

/// The exact full/compact lineage named by the original owner bridge. Legacy
/// driver.py is retained evidence only; this adapter never starts Python.
pub(crate) fn historical_for_original(root: &Path) -> Result<Vec<HistoricalCapture>> {
    let source = decomp_source::verify_source(root, &root.join("deps/snesrev-sm"))?;
    let pins: toml::Value =
        toml::from_str(&String::from_utf8(read(&root.join("tools/dependencies.toml"), MAX_RECORD)?)?)?;
    let pin = pins["dependencies"]
        .as_array()
        .context("private historical pins missing")?
        .iter()
        .find(|pin| pin["name"].as_str() == Some("snesrev-sm"))
        .context("historical C pin missing")?;
    let mut private_inputs = vec![];
    for path in [
        root.join("rom/sm.sfc"),
        root.join("movies/super-metroid-any-3653.lsmv"),
        root.join("deps/lsnes/source/src/util/lsnes-dumpavi.util"),
        root.join("deps/downloads").join(pin["archive"].as_str().context("historical archive name missing")?),
    ] {
        private_inputs.push(retention::seal_file(&fs::canonicalize(path)?, MAX_FILE)?);
    }
    for (name, expected) in &source.files {
        if name.starts_with("src/") {
            let reference = retention::seal_file(&root.join("deps/snesrev-sm").join(name), MAX_FILE)?;
            ensure!(&reference.sha256 == expected, "historical private C changed");
            private_inputs.push(reference);
        }
    }
    let full = [
        (
            "out/port/controller-terminal-seam/20261011T072946.717098Z",
            "764d9e92fbd0fe572a2ba595a016ec6d6fa4c273ab6bfe56540cfc0c3e708983",
        ),
        (
            "out/port/controller-1600-frontier/20261011T072145.761041Z",
            "cfda18ec67d62ee947b1d94363096a9482284c10a906637cd7497356296d021e",
        ),
    ];
    let mut captures = vec![];
    for (path, expected) in full {
        let verdict = retention::seal_file(&root.join(path).join("verdict.json"), MAX_RECORD)?;
        ensure!(verdict.sha256 == expected, "original full-capture bridge seal changed");
        captures.push(HistoricalCapture {
            profile: "full".into(),
            verdict,
            reader: retention::seal_file(&root.join(path).join("driver.py"), MAX_FILE)?,
            private_inputs: private_inputs.clone(),
        });
    }
    let reader = retention::seal_file(
        Path::new("/tmp/sm-ceres-continuation-compact-controller-1/target/debug/sm-pipeline"),
        MAX_FILE,
    )?;
    for (path, expected) in [
        ("1791873256094852042-256891", "79c00cb7bb32a120f981b1c14f99b639e773d88f5f544bdb41e964409ce7a916"),
        ("1791873514143170077-260799", "7c8c04d2f53d3f7a247a715a807d2dcf0e2d0004be13facd780c3d74a717fe88"),
        ("1791874177761738742-271877", "051a7195bf0ed3461eae81a9789f052a01f5c616d0d7ed716a37848c2a59d07e"),
    ] {
        let verdict = retention::seal_file(
            &root.join("out/port/ceres-continuation/controller").join(path).join("verdict.toml"),
            MAX_RECORD,
        )?;
        ensure!(verdict.sha256 == expected, "original compact-capture submission seal changed");
        captures.push(HistoricalCapture {
            profile: "compact".into(),
            verdict,
            reader: reader.clone(),
            private_inputs: private_inputs.clone(),
        });
    }
    Ok(captures)
}

fn string_at<'a>(value: &'a serde_json::Value, key: &str) -> Result<&'a str> {
    value.get(key).and_then(serde_json::Value::as_str).with_context(|| format!("historical {key} missing"))
}

fn historical_hashes(value: &serde_json::Value, profile: &str) -> Result<BTreeMap<String, String>> {
    fn add(found: &mut BTreeMap<String, String>, name: String, hash: &str) -> Result<()> {
        ensure!(
            relative(&name) && digest_valid(hash) && found.len() < MAX_MEMBERS,
            "historical artifact path/hash invalid"
        );
        if let Some(old) = found.insert(name, hash.into()) {
            ensure!(old == hash, "historical artifact has conflicting hashes");
        }
        Ok(())
    }
    fn map(found: &mut BTreeMap<String, String>, value: &serde_json::Value, prefix: &str) -> Result<()> {
        for (name, expected) in value.as_object().context("historical artifact inventory missing")? {
            add(found, format!("{prefix}{name}"), expected.as_str().context("historical hash is not text")?)?;
        }
        Ok(())
    }
    fn fields(
        found: &mut BTreeMap<String, String>,
        value: &serde_json::Value,
        prefix: &str,
        names: &[(&str, &str)],
    ) -> Result<()> {
        for (key, name) in names {
            if let Some(hash) = value.get(*key) {
                add(found, format!("{prefix}{name}"), hash.as_str().context("historical hash type differs")?)?;
            }
        }
        Ok(())
    }
    let mut found = BTreeMap::new();
    if profile == "compact" {
        map(&mut found, &value["artifacts"], "")?;
        return Ok(found);
    }
    fields(
        &mut found,
        value,
        "",
        &[
            ("timeline_sha256", "timeline.json"),
            ("controller_causality_sha256", "controller-causality.json"),
            ("derived_inputs_sha256", "derived.inputs"),
            ("poll_cases_sha256", "poll-cases.txt"),
        ],
    )?;
    fields(&mut found, &value["causal_first_completion"], "", &[("inputs_sha256", "causal.inputs")])?;
    fields(
        &mut found,
        &value["build"],
        "",
        &[
            ("binary_sha256", "prefix-host"),
            ("host_sha256", "prefix_host.c"),
            ("base_host_sha256", "headless.c"),
            ("frame_header_sha256", "headless_frame.h"),
            ("driver_sha256", "driver.py"),
        ],
    )?;
    fields(
        &mut found,
        &value["native"],
        "native/",
        &[
            ("events_sha256", "events.csv"),
            ("observer_sha256", "observe.lua"),
            ("stdout_sha256", "stdout.log"),
            ("stderr_sha256", "stderr.log"),
        ],
    )?;
    map(&mut found, &value["native"]["raw_states"], "native/")?;
    for name in [
        "candidate",
        "changed-gameplay",
        "causal-candidate",
        "causal-control",
        "omitted-wait-control",
        "omitted-screen-wait-control",
    ] {
        if let Some(subject) = value.get(name) {
            let prefix = format!("{name}/");
            fields(
                &mut found,
                subject,
                &prefix,
                &[
                    ("events_sha256", "events.jsonl"),
                    ("terminal_sha256", "terminal.bin"),
                    ("rom_view_sha256", "rom-view.bin"),
                    ("stdout_sha256", "stdout.log"),
                    ("stderr_sha256", "stderr.log"),
                ],
            )?;
            map(&mut found, &subject["complete_states"], &prefix)?;
        }
    }
    for (row, name) in [
        (&value["poll_primitive"], "poll-primitive"),
        (&value["poll_primitive"]["changed_sample_control"], "changed-poll-primitive"),
        (&value["poll_primitive"]["unrelated_state_control"], "changed-state-poll-primitive"),
    ] {
        fields(
            &mut found,
            row,
            &format!("{name}/"),
            &[("stdout_sha256", "stdout.log"), ("stderr_sha256", "stderr.log")],
        )?;
    }
    Ok(found)
}

fn historical_identity(capture: &HistoricalCapture) -> Result<()> {
    ensure!(
        HISTORICAL_SEALS.iter().any(|(profile, verdict, reader)| capture.profile == *profile
            && capture.verdict.sha256 == *verdict
            && capture.reader.sha256 == *reader),
        "historical capture or exact original reader seal differs"
    );
    Ok(())
}

pub(crate) fn authenticate_historical(capture: &HistoricalCapture) -> Result<HistoricalAuthentication> {
    historical_identity(capture)?;
    retention::authenticate_reference(&capture.verdict, MAX_RECORD)?;
    retention::authenticate_reference(&capture.reader, MAX_FILE)?;
    for reference in &capture.private_inputs {
        retention::authenticate_reference(reference, MAX_FILE)?;
    }
    let bytes = read(&capture.verdict.path, MAX_RECORD)?;
    let compact = capture.profile == "compact";
    let value: serde_json::Value = if compact {
        let value: toml::Value = toml::from_str(std::str::from_utf8(&bytes)?)?;
        serde_json::to_value(value)?
    } else {
        serde_json::from_slice(&bytes)?
    };
    let original_status = string_at(&value, "status")?;
    ensure!(
        value["accepted"] == false && ["error", "nonaccepting"].contains(&original_status),
        "historical diagnostic acceptance changed"
    );
    if compact {
        ensure!(
            value["schema_version"] == 1 && value["profile"] == "compact",
            "original compact profile/schema differs"
        );
    } else {
        ensure!(
            value["contract"] == "decomp-original-prefix-v1" && original_status == "nonaccepting",
            "original full capture contract differs"
        );
    }
    let root = capture.verdict.path.parent().context("historical capture root missing")?;
    let hashes = historical_hashes(&value, &capture.profile)?;
    ensure!(!hashes.is_empty(), "historical artifact closure empty");
    let mut artifact_bytes = 0u64;
    for (name, expected) in &hashes {
        let observed = retention::seal_file(&root.join(name), MAX_HISTORY)?;
        ensure!(&observed.sha256 == expected, "original historical artifact changed: {name}");
        artifact_bytes = artifact_bytes.checked_add(observed.bytes).context("historical aggregate extent overflow")?;
        ensure!(artifact_bytes <= MAX_HISTORY, "historical capture aggregate bound exceeded");
    }
    let private = capture.private_inputs.iter().map(|reference| reference.sha256.as_str()).collect::<BTreeSet<_>>();
    let native = &value["native"];
    for expected in if compact {
        vec![
            string_at(native, "rom_sha256")?,
            string_at(native, "movie_sha256")?,
            string_at(native, "executable_sha256")?,
        ]
    } else {
        vec![
            string_at(&value, "rom_sha256")?,
            string_at(&value, "movie_sha256")?,
            string_at(native, "executable_sha256")?,
            string_at(&value, "source_archive_sha256")?,
        ]
    } {
        ensure!(private.contains(expected), "historical exact private input dependency missing");
    }
    if !compact {
        ensure!(
            capture.reader.path == root.join("driver.py")
                && capture.reader.sha256 == string_at(&value["build"], "driver_sha256")?,
            "historical full reader differs from original sealed driver"
        );
        for expected in value["build"]["sources"].as_object().context("historical C closure missing")?.values() {
            ensure!(
                private.contains(expected.as_str().context("historical C hash invalid")?),
                "historical private C source missing or changed"
            );
        }
    }
    let frames = value["requested_frames"].as_u64().context("historical frame count missing")?;
    ensure!((320..=20_000).contains(&frames), "historical frame bound differs");
    let original = native_trace(BufReader::new(File::open(root.join("native/events.csv"))?), frames, compact)?;
    for (name, expected) in &original.snapshots {
        ensure!(
            state(&read(&root.join("native").join(name), 0x20000)?)? == *expected,
            "historical native state snapshot differs"
        );
    }
    let summary = if original_status == "nonaccepting" {
        let events_name = if compact { "events.txt" } else { "events.jsonl" };
        let candidate =
            subject_trace(BufReader::new(File::open(root.join("causal-candidate").join(events_name))?), !compact)?;
        let control =
            subject_trace(BufReader::new(File::open(root.join("causal-control").join(events_name))?), !compact)?;
        let summary = comparisons(&original, &candidate, &control)?;
        let reported = if compact { &value } else { &value["causal_update_prefix"] };
        ensure!(
            reported["matching_causal_prefix"].as_u64() == Some(summary.matching_prefix)
                && reported["native_completed_updates"].as_u64() == Some(summary.native_completed_updates)
                && reported[if compact { "causal_completed_updates" } else { "candidate_completed_updates" }].as_u64()
                    == Some(summary.candidate_completed_updates)
                && summary.changed_rng_detected,
            "historical causal comparison or changed-gameplay control differs"
        );
        Some(summary)
    } else {
        None
    };
    Ok(HistoricalAuthentication {
        capture: capture.clone(),
        artifact_count: hashes.len(),
        artifact_bytes,
        inventory_sha256: sealed(&hashes)?,
        summary,
        original_status: original_status.into(),
        fresh_capture: false,
        reader_execution: None,
    })
}

pub(crate) fn references(receipt: &CompactReceipt) -> Result<Vec<Reference>> {
    let reference = retention::seal_file(&receipt.verdict.file, MAX_RECORD)?;
    ensure!(reference.sha256 == receipt.verdict.sha256, "compact verdict seal changed");
    let verdict: CompactVerdict = toml::from_str(&String::from_utf8(read(&receipt.verdict.file, MAX_RECORD)?)?)?;
    let output = receipt.verdict.file.parent().context("compact verdict parent missing")?;
    let mut references = vec![reference, retention::seal_file(&output.join("compact-intent.toml"), MAX_RECORD)?];
    references.extend(verdict.source_snapshots.into_values());
    references.extend(verdict.input_snapshots.into_values());
    references.extend(verdict.projections.into_values());
    if output.join("experiment-deadline.toml").is_file() {
        references.push(retention::seal_file(&output.join("experiment-deadline.toml"), MAX_RECORD)?);
    }
    for invocation in verdict.operations {
        references
            .push(retention::seal_file(&output.join(format!("{}-invocation.toml", invocation.label)), MAX_RECORD)?);
        references.extend(invocation.references);
    }
    for historical in verdict.historical_dependencies {
        references.push(historical.verdict);
        references.push(historical.reader);
        references.extend(historical.private_inputs);
    }
    let mut unique = BTreeMap::new();
    for reference in references {
        if let Some(previous) = unique.insert(reference.path.clone(), reference.clone()) {
            ensure!(previous == reference, "compact retained dependency has conflicting identities");
        }
    }
    Ok(unique.into_values().collect())
}

pub(crate) fn authenticate(
    prepared: &PreparedCompact,
    checkout: &Path,
    receipt: &CompactReceipt,
) -> Result<CompactVerdict> {
    ensure!(
        receipt.prepared_sha256 == sealed(prepared)? && receipt.candidate_sha256 == candidate(prepared, checkout)?,
        "compact receipt belongs to different prepared/original source"
    );
    ensure!(
        retention::seal_file(&receipt.verdict.file, MAX_RECORD)?.sha256 == receipt.verdict.sha256,
        "compact receipt verdict changed"
    );
    let output = receipt.verdict.file.parent().context("compact verdict parent missing")?;
    let verdict: CompactVerdict = toml::from_str(&String::from_utf8(read(&receipt.verdict.file, MAX_RECORD)?)?)?;
    ensure!(
        verdict.schema == "translation-compact-verdict-v1"
            && verdict.evidence_kind == "infrastructure-diagnostic"
            && verdict.prepared_sha256 == receipt.prepared_sha256
            && verdict.candidate_sha256 == receipt.candidate_sha256
            && verdict.context == receipt.context
            && verdict.passed == receipt.passed
            && verdict.diagnostic == receipt.diagnostic
            && verdict.requested_frames == FRAMES
            && verdict.experiment_budget_seconds == SECONDS
            && !verdict.original_rom_parity
            && !verdict.live_replay
            && verdict.limitations == limitations()
            && verdict.historical_dependencies == prepared.contract.historical
            && verdict.reserved_bytes <= verdict.runtime.max_output_bytes
            && receipt.evidence_bytes <= verdict.reserved_bytes,
        "compact acceptance, target, limitations or aggregate budget changed"
    );
    let intent: Intent = toml::from_str(&String::from_utf8(read(&output.join("compact-intent.toml"), MAX_RECORD)?)?)?;
    ensure!(
        intent.prepared_sha256 == verdict.prepared_sha256
            && intent.candidate_sha256 == verdict.candidate_sha256
            && intent.context == verdict.context
            && intent.runtime == verdict.runtime,
        "compact durable intent differs"
    );
    if let Some(started) = verdict.experiment_started_unix_ms {
        let experiment: Experiment =
            toml::from_str(&String::from_utf8(read(&output.join("experiment-deadline.toml"), MAX_RECORD)?)?)?;
        authenticate_experiment(&experiment, intent.deadline_unix_ms)?;
        ensure!(
            experiment.started_unix_ms == started
                && Some(experiment.deadline_unix_ms) == verdict.experiment_deadline_unix_ms
                && verdict.experiment_finished_unix_ms.is_some_and(|finished| finished >= started),
            "compact retained experiment interval changed"
        );
    } else {
        ensure!(
            verdict.experiment_deadline_unix_ms.is_none() && verdict.experiment_finished_unix_ms.is_none(),
            "compact absent experiment has timing evidence"
        );
    }
    for (name, reference) in &verdict.source_snapshots {
        ensure!(
            prepared.contract.source_files.get(name) == Some(&reference.sha256)
                && reference.path == output.join("source").join(name),
            "compact retained original source differs"
        );
        retention::authenticate_reference(reference, MAX_RECORD)?;
    }
    for (name, reference) in &verdict.input_snapshots {
        ensure!(
            prepared
                .contract
                .private_inputs
                .get(name)
                .is_some_and(|input| input.sha256 == reference.sha256 && input.bytes == reference.bytes)
                && reference.path == output.join("private").join(name),
            "compact retained private input differs"
        );
        retention::authenticate_reference(reference, MAX_FILE)?;
    }
    for (name, reference) in &verdict.projections {
        ensure!(
            ["observe.lua", "causal.inputs", "prefix-host", "native.zst", "candidate.zst", "control.zst"]
                .contains(&name.as_str())
                && reference.path == output.join(name),
            "compact projection path differs"
        );
        retention::authenticate_reference(reference, MAX_RAW)?;
    }
    if let Some(script) = verdict.projections.get("observe.lua") {
        let template = String::from_utf8(read(
            &output.join("source/crates/pipeline/src/prefix_compact_observer.lua.in"),
            MAX_RECORD,
        )?)?;
        ensure!(
            script.sha256 == sha256(observer(&template).as_bytes()),
            "compact observer differs from original submitted template"
        );
    }
    let mut labels = BTreeSet::new();
    let mut native = None;
    let mut subjects = BTreeMap::new();
    let mut fresh_order = Vec::new();
    for invocation in &verdict.operations {
        ensure!(labels.insert(invocation.label.clone()), "compact producer duplicated");
        let mut context = verdict.context.clone();
        context.operation_key = sealed(&(&verdict.context.operation_key, output, &invocation.label))?;
        ensure!(invocation.context == context, "compact producer operation lineage changed");
        let expected = if let Some(index) = invocation.label.strip_prefix("history-") {
            let index: usize = index.parse()?;
            let capture =
                prepared.contract.historical.get(index).context("compact historical operation index changed")?;
            historical_program(capture, &verdict.runtime.helper)?
        } else {
            let counts =
                native.as_ref().map(|trace: &NativeTrace| (trace.polls.len() as u64, trace.completed.len() as u64));
            fresh_order.push(invocation.label.as_str());
            recipe(prepared, output, &verdict.runtime, &invocation.label, counts)?
        };
        let declared = limits(&invocation.label);
        ensure!(
            invocation.program == expected
                && invocation.limits.timeout > Duration::ZERO
                && invocation.limits.timeout <= declared.timeout
                && invocation.limits.writable_bytes == declared.writable_bytes
                && invocation.limits.stdout_bytes == declared.stdout_bytes
                && invocation.limits.stderr_bytes == declared.stderr_bytes
                && invocation.limits.stdin_bytes == 0,
            "compact fixed producer recipe or bounds changed"
        );
        let invocation_intent: InvocationIntent = toml::from_str(&String::from_utf8(read(
            &output.join(format!("{}-invocation.toml", invocation.label)),
            MAX_RECORD,
        )?)?)?;
        let deadline = if invocation.label.starts_with("history-") {
            intent.deadline_unix_ms
        } else {
            verdict.experiment_deadline_unix_ms.context("compact producer has no original experiment interval")?
        };
        ensure!(
            invocation_intent.context == invocation.context
                && invocation_intent.limits == invocation.limits
                && invocation_intent.reserved_bytes == invocation.reserved_bytes
                && invocation_intent.started_unix_ms < deadline
                && invocation_intent.limits.timeout
                    <= Duration::from_millis(deadline - invocation_intent.started_unix_ms)
                && (invocation.label.starts_with("history-")
                    || Some(invocation_intent.started_unix_ms) >= verdict.experiment_started_unix_ms)
                && invocation_intent.request_sha256 == sealed(&(&invocation.program, &declared))?,
            "compact invocation intent differs"
        );
        let result = operation::authenticate_request(
            &verdict.runtime.operation_root,
            &invocation.context,
            &invocation.program,
            &invocation.limits,
            &[],
        )?;
        ensure!(
            result.result_sha256 == invocation.result_sha256
                && operation::references(&verdict.runtime.operation_root, &invocation.context)?
                    == invocation.references,
            "compact retained producer evidence changed"
        );
        ensure!(
            !verdict.passed || result.passed && result.status_raw == Some(0),
            "compact accepted a failed original operation"
        );
        if invocation.label == "compile" {
            if let Some(reference) = verdict.projections.get("prefix-host") {
                ensure!(
                    result.outputs.get(Path::new("prefix-host")).is_some_and(|bytes| sha256(bytes) == reference.sha256),
                    "compact host is not the confined compiler output"
                );
            }
        } else if ["native", "candidate", "control"].contains(&invocation.label.as_str()) {
            if let Some(reference) = verdict.projections.get(&format!("{}.zst", invocation.label)) {
                ensure!(
                    result.outputs.get(Path::new("capture.zst")).is_some_and(|bytes| sha256(bytes) == reference.sha256),
                    "compact raw capture is not the confined output"
                );
                if verdict.passed || verdict.summary.is_some() || invocation.label == "native" && fresh_order.len() > 1
                {
                    let raw = phase_capture(&result, &invocation.label)?;
                    if invocation.label == "native" {
                        native = Some(validate_native(&raw)?);
                    } else {
                        subjects.insert(invocation.label.clone(), validate_subject(&raw)?);
                    }
                } else if invocation.label == "native" {
                    native = phase_capture(&result, "native").and_then(|raw| validate_native(&raw)).ok();
                }
            }
        }
    }
    ensure!(
        fresh_order.len() <= 4 && fresh_order == ["native", "compile", "candidate", "control"][..fresh_order.len()],
        "compact causal experiment phase order differs"
    );
    for (index, historical) in verdict.historical.iter().enumerate() {
        ensure!(
            prepared.contract.historical.get(index) == Some(&historical.capture) && !historical.fresh_capture,
            "compact historical closure identity changed"
        );
        let mut current = authenticate_historical(&historical.capture)?;
        current.reader_execution = historical.reader_execution.clone();
        ensure!(current == *historical, "historical full/compact authentication changed");
        ensure!(
            historical.reader_execution.is_some()
                == (historical.capture.profile == "compact" && historical.original_status == "nonaccepting"),
            "historical successful compact capture omitted its exact original reader"
        );
        if let Some(reader) = &historical.reader_execution {
            ensure!(verdict.operations.contains(reader), "historical reader execution is not a retained operation");
            let result = operation::authenticate_request(
                &verdict.runtime.operation_root,
                &reader.context,
                &reader.program,
                &reader.limits,
                &[],
            )?;
            ensure!(
                result.passed
                    && result.status_raw == Some(0)
                    && String::from_utf8_lossy(&result.stdout).contains("acceptance remains false"),
                "retained original compact reader failed authentication"
            );
        }
    }
    if verdict.passed || verdict.summary.is_some() {
        let native = native.context("compact accepted native capture missing")?;
        let candidate = subjects.get("candidate").context("compact accepted candidate missing")?;
        let control = subjects.get("control").context("compact independent changed-gameplay control missing")?;
        let summary = comparisons(&native, candidate, control)?;
        ensure!(verdict.summary.as_ref() == Some(&summary), "compact retained completed-state comparison differs");
        ensure!(
            !verdict.passed
                || (summary.matching_prefix == summary.native_completed_updates
                    && summary.native_completed_updates == summary.candidate_completed_updates
                    && control.completed.len() == native.completed.len()
                    && summary.changed_rng_detected),
            "compact accepted a divergent completed-state comparison or absent control"
        );
        let expected_inputs = native
            .polls
            .iter()
            .map(|event| format!("{:04x}\n", event.pad.expect("validated native poll")))
            .collect::<String>();
        ensure!(
            read(&output.join("causal.inputs"), MAX_RECORD)? == expected_inputs.as_bytes(),
            "compact retained ordered input derivation differs"
        );
    }
    if verdict.passed {
        ensure!(
            verdict.historical.len() == prepared.contract.historical.len()
                && fresh_order.len() == 4
                && verdict.source_snapshots.len() == prepared.contract.source_files.len()
                && verdict.input_snapshots.len() + 2 == prepared.contract.private_inputs.len()
                && verdict.diagnostic.is_empty(),
            "compact acceptance omits original coverage or contains a failure"
        );
        ensure!(
            verdict.experiment_deadline_unix_ms.is_some()
                && verdict.experiment_finished_unix_ms.is_some()
                && verdict.experiment_finished_unix_ms <= verdict.experiment_deadline_unix_ms
                && verdict.experiment_deadline_unix_ms <= Some(intent.deadline_unix_ms),
            "compact whole-experiment deadline changed"
        );
    } else {
        ensure!(!verdict.diagnostic.is_empty(), "compact failed run omitted its bounded diagnosis");
    }
    let bytes = references(receipt)?.iter().try_fold(0u64, |total, reference| {
        total.checked_add(reference.bytes).context("compact retained byte overflow")
    })?;
    ensure!(bytes == receipt.evidence_bytes, "compact retained evidence extent differs");
    Ok(verdict)
}

#[cfg(test)]
#[path = "translation_compact_recovery_tests.rs"]
mod recovery_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supervisor_preparation_refuses_a_shorter_passing_substitute() {
        let contract = CompactContract {
            schema: "translation-compact-contract-v1".into(),
            frames: 320,
            timeout_seconds: 60,
            source_files: BTreeMap::new(),
            private_inputs: BTreeMap::new(),
            c_units: vec![],
            runtime: vec![],
            historical: vec![],
        };
        let error = prepare(Path::new("/unused"), &contract).unwrap_err();
        assert!(format!("{error:#}").contains("frame 2240"));
    }

    #[test]
    fn supervisor_preparation_requires_both_original_capture_profiles() {
        let contract = CompactContract {
            schema: "translation-compact-contract-v1".into(),
            frames: 2240,
            timeout_seconds: 60,
            source_files: BTreeMap::new(),
            private_inputs: BTreeMap::new(),
            c_units: vec![],
            runtime: vec![],
            historical: vec![],
        };
        let error = prepare(Path::new("/unused"), &contract).unwrap_err();
        assert!(format!("{error:#}").contains("full and compact historical captures"));
    }

    #[test]
    fn supervisor_preparation_preserves_the_complete_original_capture_lineage() {
        let reference = Reference { path: "/unavailable".into(), sha256: "0".repeat(64), bytes: 0 };
        let contract = CompactContract {
            schema: "translation-compact-contract-v1".into(),
            frames: 2240,
            timeout_seconds: 60,
            source_files: BTreeMap::new(),
            private_inputs: BTreeMap::new(),
            c_units: vec![],
            runtime: vec![],
            historical: ["full", "compact"]
                .map(|profile| HistoricalCapture {
                    profile: profile.into(),
                    verdict: reference.clone(),
                    reader: reference.clone(),
                    private_inputs: vec![],
                })
                .into(),
        };
        let error = prepare(Path::new("/unused"), &contract).unwrap_err();
        assert!(format!("{error:#}").contains("complete original full/compact capture lineage"));
    }

    #[test]
    #[ignore = "requires the sealed original ROM/movie/reader closure and installed confined producer"]
    fn original_experiment_retains_exact_target_and_unchanged_resume() {
        let checkout = PathBuf::from(std::env::var("SM_COMPACT_SOURCE").unwrap());
        let assets = PathBuf::from(std::env::var("SM_COMPACT_ASSETS").unwrap());
        let output = PathBuf::from(std::env::var("SM_COMPACT_PROOF_ROOT").unwrap());
        let rustc = PathBuf::from(std::env::var("SM_COMPACT_RUSTC").unwrap());
        let historical = historical_for_original(&assets).unwrap();
        let contract = contract_for_original(&checkout, &assets, &rustc, historical).unwrap();
        let prepared = prepare(&checkout, &contract).unwrap();
        fs::create_dir_all(&output).unwrap();
        let prepared_path = output.join("prepared.toml");
        let encoded = toml::to_string_pretty(&prepared).unwrap();
        if prepared_path.is_file() {
            assert_eq!(fs::read_to_string(&prepared_path).unwrap(), encoded);
        } else {
            fs::write(&prepared_path, encoded).unwrap();
        }
        let context = OperationContext {
            run_id: sha256(b"compact-original-confined-proof"),
            attempt: 1,
            policy_sha256: sealed(&prepared).unwrap(),
            operation_key: "c".repeat(64),
            phase: translation_execution::model::ExecutionPhase::Candidate,
        };
        let runtime = StrictRuntime {
            producer: PathBuf::from(std::env::var("SM_COMPACT_PRODUCER").unwrap()),
            helper: PathBuf::from(std::env::var("SM_COMPACT_HELPER").unwrap()),
            operation_root: output.join("operations"),
            pipeline_root: PathBuf::from("/home/jim/sm-decomp/out/pipeline"),
            workers: 4,
            gate_timeout_seconds: 3600,
            deadline_unix_ms: None,
            max_output_bytes: 4 * 1024 * 1024 * 1024,
        };
        let first = run(&prepared, &checkout, &output.join("gate"), &runtime, &context).unwrap();
        // run authenticates before publishing or returning a retained receipt.
        // Keep the unchanged-resume authentication below without hashing every
        // multi-gigabyte historical capture a redundant third time here.
        let retained: CompactVerdict =
            toml::from_str(&String::from_utf8(read(&first.verdict.file, MAX_RECORD).unwrap()).unwrap()).unwrap();
        assert_eq!(retained.historical.len(), 5, "{}", first.diagnostic);
        assert_eq!(retained.historical[0].summary.as_ref().unwrap().matching_prefix, 1097);
        assert_eq!(retained.historical[2].summary.as_ref().unwrap().matching_prefix, 1736);
        assert!(retained.historical[2].reader_execution.is_some());
        assert!(retained.operations.iter().any(|invocation| invocation.label == "native"), "{}", first.diagnostic);
        assert_eq!(retained.requested_frames, 2240);
        assert_eq!(retained.experiment_budget_seconds, 60);
        assert!(!retained.original_rom_parity && !retained.live_replay);
        if first.passed {
            let summary = retained.summary.as_ref().unwrap();
            assert_eq!(summary.native_completed_updates, 1736);
            assert_eq!(summary.polls, 1887);
            assert_eq!(summary.matching_prefix, 1736);
            assert!(summary.changed_rng_detected);
        } else {
            assert!(!first.diagnostic.is_empty());
        }
        let repeat = run(&prepared, &checkout, &output.join("gate"), &runtime, &context).unwrap();
        assert_eq!(first, repeat);
    }

    #[test]
    #[ignore = "requires a retained original experiment and its exact private reader/input closure"]
    fn retained_original_experiment_authenticates_without_execution() {
        let checkout = PathBuf::from(std::env::var("SM_COMPACT_SOURCE").unwrap());
        let output = PathBuf::from(std::env::var("SM_COMPACT_PROOF_ROOT").unwrap());
        let prepared: PreparedCompact =
            toml::from_str(&fs::read_to_string(output.join("prepared.toml")).unwrap()).unwrap();
        let receipt: CompactReceipt =
            toml::from_str(&fs::read_to_string(output.join("gate/compact-receipt.toml")).unwrap()).unwrap();
        let before = files(&output.join("operations"), MAX_HISTORY).unwrap();
        let references_before = references(&receipt).unwrap();
        let verdict = authenticate(&prepared, &checkout, &receipt).unwrap();
        assert_eq!(verdict.requested_frames, 2240);
        assert_eq!(verdict.experiment_budget_seconds, 60);
        assert_eq!(verdict.historical.len(), 5);
        assert!(!verdict.live_replay && !verdict.original_rom_parity);
        assert_eq!(references_before, references(&receipt).unwrap());
        assert_eq!(before, files(&output.join("operations"), MAX_HISTORY).unwrap());
        println!(
            "Retained compact experiment authenticated; passed={}; diagnosis={}",
            receipt.passed, receipt.diagnostic
        );
    }
}
