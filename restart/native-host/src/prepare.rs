#[cfg(feature = "native-preparation")]
use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};
use sm_binary_evidence::Map;
use sm_binary_evidence::Value;
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Read;
#[cfg(feature = "native-preparation")]
use std::io::Write;
#[cfg(any(test, feature = "native-preparation", feature = "private-checks"))]
use std::path::Component;
use std::path::{Path, PathBuf};
#[cfg(feature = "native-preparation")]
use std::process::{Command, Stdio};
#[cfg(feature = "native-preparation")]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(feature = "native-preparation")]
use std::time::{Duration, Instant};

pub const ARCHIVE_SHA256: &str = "69c78831351112887575f852dadf0281dd5c23cef890b07a268ae6cf60aa9193";
pub const SOURCE_REVISION: &str = "578f90b3cc49557bb70060ad033bb90b8cf8ac50";
#[path = "compiler_process.rs"]
#[cfg(feature = "native-preparation")]
mod compiler_process;
#[path = "patches.rs"]
#[cfg(feature = "native-preparation")]
mod patches;
#[path = "source_prepare.rs"]
#[cfg(feature = "native-preparation")]
mod source_prepare;
#[cfg(feature = "native-preparation")]
use patches::replace_once;
#[cfg(all(unix, feature = "native-preparation"))]
const FLAGS: &[&str] = &["-std=gnu11", "-O2", "-fno-strict-aliasing", "-fPIC", "-shared", "-Wl,-z,defs"];
#[cfg(all(windows, feature = "native-preparation"))]
const FLAGS: &[&str] = &["-std=gnu11", "-O2", "-fno-strict-aliasing", "-shared", "-DCOMPILER_TCC=1"];
#[cfg(all(unix, feature = "native-preparation"))]
const LIBRARY_NAME: &str = "libsm_native.so";
#[cfg(all(windows, feature = "native-preparation"))]
const LIBRARY_NAME: &str = "sm_native.dll";
#[cfg(feature = "native-preparation")]
static TEMP_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub struct Prepared {
    pub library: PathBuf,
    pub manifest: PathBuf,
    pub reused: bool,
}

pub struct Manifest {
    format: String,
    source_revision: String,
    archive_sha256: String,
    recipe_sha256: String,
    compiler: String,
    compiler_sha256: String,
    compiler_version: String,
    compiler_arguments: Vec<String>,
    original_sources: BTreeMap<String, String>,
    retained_sources: BTreeMap<String, String>,
    patches: Vec<PatchRecord>,
    library_sha256: String,
}

struct PatchRecord {
    path: String,
    before: String,
    after: String,
}

impl Manifest {
    fn encode_for_validation(&self) -> Result<Vec<u8>, String> {
        let strings = |values: &BTreeMap<String, String>| {
            Value::Table(values.iter().map(|(k, v)| (k.clone(), v.clone().into())).collect())
        };
        sm_binary_evidence::encode(&Value::Table(Map::from([
            ("format".into(), self.format.clone().into()),
            ("source_revision".into(), self.source_revision.clone().into()),
            ("archive_sha256".into(), self.archive_sha256.clone().into()),
            ("recipe_sha256".into(), self.recipe_sha256.clone().into()),
            ("compiler".into(), self.compiler.clone().into()),
            ("compiler_sha256".into(), self.compiler_sha256.clone().into()),
            ("compiler_version".into(), self.compiler_version.clone().into()),
            (
                "compiler_arguments".into(),
                Value::Array(self.compiler_arguments.iter().cloned().map(Value::String).collect()),
            ),
            ("original_sources".into(), strings(&self.original_sources)),
            ("retained_sources".into(), strings(&self.retained_sources)),
            (
                "patches".into(),
                Value::Array(
                    self.patches
                        .iter()
                        .map(|patch| {
                            Value::Table(Map::from([
                                ("path".into(), patch.path.clone().into()),
                                ("before".into(), patch.before.clone().into()),
                                ("after".into(), patch.after.clone().into()),
                            ]))
                        })
                        .collect(),
                ),
            ),
            ("library_sha256".into(), self.library_sha256.clone().into()),
        ])))
    }

    #[cfg(any(test, feature = "private-checks"))]
    fn decode(bytes: &[u8]) -> Result<Self, String> {
        Self::from_value(sm_binary_evidence::decode(bytes)?)
    }

    fn from_value(value: Value) -> Result<Self, String> {
        fn text(value: &Value, key: &str) -> Result<String, String> {
            value
                .get(key)
                .and_then(Value::as_str)
                .map(str::to_owned)
                .ok_or_else(|| format!("missing manifest string {key}"))
        }
        fn strings(value: &Value, key: &str) -> Result<BTreeMap<String, String>, String> {
            value
                .get(key)
                .and_then(Value::as_table)
                .ok_or_else(|| format!("missing manifest table {key}"))?
                .iter()
                .map(|(k, v)| {
                    v.as_str().map(|s| (k.clone(), s.to_owned())).ok_or_else(|| format!("invalid source hash {k}"))
                })
                .collect()
        }

        if value.as_table().is_none_or(|map| map.len() != 12) {
            return Err("invalid manifest field count".into());
        }
        let arguments = value
            .get("compiler_arguments")
            .and_then(Value::as_array)
            .ok_or("missing compiler arguments")?
            .iter()
            .map(|v| v.as_str().map(str::to_owned).ok_or("invalid compiler argument".into()))
            .collect::<Result<_, String>>()?;
        let patches = value
            .get("patches")
            .and_then(Value::as_array)
            .ok_or("missing manifest patches")?
            .iter()
            .map(|patch| {
                if patch.as_table().is_none_or(|map| map.len() != 3) {
                    return Err("invalid patch fields".into());
                }
                Ok(PatchRecord {
                    path: text(patch, "path")?,
                    before: text(patch, "before")?,
                    after: text(patch, "after")?,
                })
            })
            .collect::<Result<_, String>>()?;
        Ok(Self {
            format: text(&value, "format")?,
            source_revision: text(&value, "source_revision")?,
            archive_sha256: text(&value, "archive_sha256")?,
            recipe_sha256: text(&value, "recipe_sha256")?,
            compiler: text(&value, "compiler")?,
            compiler_sha256: text(&value, "compiler_sha256")?,
            compiler_version: text(&value, "compiler_version")?,
            compiler_arguments: arguments,
            original_sources: strings(&value, "original_sources")?,
            retained_sources: strings(&value, "retained_sources")?,
            patches,
            library_sha256: text(&value, "library_sha256")?,
        })
    }
}

pub fn hash_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn hash_file(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| format!("open {}: {e}", path.display()))?;
    let mut hash = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

fn recipe_hash() -> String {
    hash_bytes(
        [
            include_str!("shim.c"),
            include_str!("observe.h"),
            include_str!("pilot.c"),
            include_str!("scroll.c"),
            include_str!("ceres_door.c"),
            include_str!("room_transition.c"),
            include_str!("door_scrolling.c"),
            include_str!("camera.c"),
            include_str!("item_instruction.c"),
            include_str!("persistence.c"),
            include_str!("persistence_hooks.rs"),
            include_str!("persistence_types.rs"),
            include_str!("persistence_prepare.rs"),
            include_str!("player_motion.c"),
            include_str!("player_motion_hooks.rs"),
            include_str!("player_motion_types.rs"),
            include_str!("player_motion_prepare.rs"),
            include_str!("waver.c"),
            include_str!("waver_hooks.rs"),
            include_str!("waver_types.rs"),
            include_str!("waver_prepare.rs"),
            include_str!("ripper.c"),
            include_str!("ripper_hooks.rs"),
            include_str!("ripper_types.rs"),
            include_str!("ripper_prepare.rs"),
            include_str!("rio.c"),
            include_str!("rio_prepare.rs"),
            include_str!("rio_hooks.rs"),
            include_str!("rio_native.rs"),
            include_str!("item_instruction_hooks.rs"),
            include_str!("item_instruction_types.rs"),
            include_str!("item_instruction_prepare.rs"),
            include_str!("prepare.rs"),
            include_str!("patches.rs"),
            include_str!("source_prepare.rs"),
            include_str!("compiler_process.rs"),
            include_str!("lib.rs"),
            include_str!("hooks.rs"),
            include_str!("scroll_hooks.rs"),
            include_str!("ceres_door_hooks.rs"),
            include_str!("room_transition_hooks.rs"),
            include_str!("room_transition_types.rs"),
            include_str!("room_transition_prepare.rs"),
            include_str!("door_scrolling_hooks.rs"),
            include_str!("door_scrolling_types.rs"),
            include_str!("door_scrolling_prepare.rs"),
            include_str!("camera_hooks.rs"),
            include_str!("camera_types.rs"),
            include_str!("camera_prepare.rs"),
            include_str!("transaction.rs"),
        ]
        .join("\0")
        .as_bytes(),
    )
}

#[cfg(any(test, feature = "native-preparation", feature = "private-checks"))]
fn relative(path: &Path) -> Result<(), String> {
    if path.as_os_str().is_empty() || path.components().any(|c| !matches!(c, Component::Normal(_))) {
        return Err(format!("unsafe relative source path: {}", path.display()));
    }
    Ok(())
}

fn source_files(root: &Path, at: &Path, result: &mut BTreeMap<String, String>) -> Result<(), String> {
    for entry in fs::read_dir(at).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let kind = entry.file_type().map_err(|e| e.to_string())?;
        if kind.is_dir() {
            source_files(root, &entry.path(), result)?;
        } else if kind.is_file() {
            let path = entry.path();
            let name = path.strip_prefix(root).unwrap().to_string_lossy().into_owned();
            result.insert(name, hash_file(&path)?);
        } else {
            return Err("non-regular retained source".into());
        }
    }
    Ok(())
}

#[cfg(feature = "native-preparation")]
fn compiler_path() -> Result<PathBuf, String> {
    let name = std::env::var_os("CC").unwrap_or_else(|| if cfg!(windows) { "gcc.exe".into() } else { "cc".into() });
    let candidate = PathBuf::from(&name);
    if candidate.components().count() > 1 {
        return candidate.canonicalize().map_err(|e| format!("compiler: {e}"));
    }
    for directory in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
        let candidate = directory.join(&name);
        if candidate.is_file() {
            return candidate.canonicalize().map_err(|e| e.to_string());
        }
    }
    Err(format!("compiler {name:?} not found"))
}

#[cfg(feature = "native-preparation")]
fn compiler_version(compiler: &Path, output: &Path) -> Result<String, String> {
    let log_path =
        output.join(format!("compiler-version-{}-{}.log", std::process::id(), TEMP_ID.fetch_add(1, Ordering::Relaxed)));
    let log = File::options().write(true).create_new(true).open(&log_path).map_err(|e| e.to_string())?;
    let mut command = Command::new(compiler);
    command.arg("--version").stdout(Stdio::from(log.try_clone().map_err(|e| e.to_string())?)).stderr(Stdio::from(log));
    let mut process = compiler_process::Compiler::spawn(&mut command)?;
    let start = Instant::now();
    loop {
        if fs::metadata(&log_path).map_err(|e| e.to_string())?.len() > 512 * 1024
            || start.elapsed() > Duration::from_secs(10)
        {
            return Err(format!("compiler --version exceeded time/output bound; retained {}", log_path.display()));
        }
        if let Some(status) = process.try_wait()? {
            if !status.success() {
                return Err(format!("compiler --version failed; retained {}", log_path.display()));
            }
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    // Drop also stops unexpected descendants before reading their retained log.
    drop(process);
    let mut bytes = Vec::new();
    File::open(&log_path)
        .map_err(|e| e.to_string())?
        .take(512 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 512 * 1024 {
        return Err("compiler --version output exceeded bound".into());
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// Authenticate the pinned archive, privately extract only C source/headers and
/// notices, apply reviewed observational patches and build the native library.
/// A cache hit authenticates every retained source and the resulting library.
#[cfg(not(feature = "native-preparation"))]
pub fn prepare(_archive: &Path, _output: &Path) -> Result<Prepared, String> {
    Err("this host requires an authenticated prepared native library; use the native-preparation CLI to build one"
        .into())
}

#[cfg(feature = "native-preparation")]
pub fn prepare(archive: &Path, output: &Path) -> Result<Prepared, String> {
    if hash_file(archive)? != ARCHIVE_SHA256 {
        return Err("recovered-C archive SHA-256 mismatch".into());
    }
    fs::create_dir_all(output).map_err(|e| e.to_string())?;
    let output = output.canonicalize().map_err(|e| e.to_string())?;
    let compiler = compiler_path()?;
    let compiler_sha256 = hash_file(&compiler)?;
    let compiler_version = compiler_version(&compiler, &output)?;
    let recipe_sha256 = recipe_hash();
    let key = hash_bytes(format!("{recipe_sha256}\0{compiler_sha256}\0{compiler_version}").as_bytes());
    let destination = output.join(&key);
    let library = destination.join(LIBRARY_NAME);
    if destination.exists() {
        verify_library(&library)?;
        return Ok(Prepared { library, manifest: destination.join("library.sme"), reused: true });
    }
    let work = output.join(format!("building-{}-{}", std::process::id(), TEMP_ID.fetch_add(1, Ordering::Relaxed)));
    fs::create_dir(&work).map_err(|e| e.to_string())?;
    let root = work.join("source");
    fs::create_dir(&root).map_err(|e| e.to_string())?;
    let mut archive = tar::Archive::new(GzDecoder::new(File::open(archive).map_err(|e| e.to_string())?));
    let prefix = format!("sm-{SOURCE_REVISION}");
    for entry in archive.entries().map_err(|e| e.to_string())? {
        let mut entry = entry.map_err(|e| e.to_string())?;
        if entry.header().entry_type().is_pax_global_extensions() {
            continue;
        }
        let path = entry.path().map_err(|e| e.to_string())?.into_owned();
        let path = path.strip_prefix(&prefix).map_err(|_| format!("archive root mismatch: {}", path.display()))?;
        if path.as_os_str().is_empty() {
            continue;
        }
        relative(path)?;
        let selected = path == Path::new("LICENSE.txt")
            || (path.starts_with("src")
                && matches!(path.extension().and_then(|v| v.to_str()), Some("c" | "h" | "inc")));
        if !selected {
            continue;
        }
        if !entry.header().entry_type().is_file() {
            return Err("selected archive source is not a regular file".into());
        }
        let target = root.join(path);
        fs::create_dir_all(target.parent().unwrap()).map_err(|e| e.to_string())?;
        let mut file = File::options().write(true).create_new(true).open(target).map_err(|e| e.to_string())?;
        std::io::copy(&mut entry, &mut file).map_err(|e| e.to_string())?;
    }
    let mut original_sources = BTreeMap::new();
    source_files(&root, &root, &mut original_sources)?;
    if !original_sources.contains_key("LICENSE.txt") {
        return Err("missing source license".into());
    }
    let patches = source_prepare::prepare_sources(&root)?;
    let mut retained_sources = BTreeMap::new();
    source_files(&root, &root, &mut retained_sources)?;
    let mut arguments: Vec<String> = FLAGS.iter().map(|s| (*s).into()).collect();
    // Quote-only lookup is required: upstream src/features.h would otherwise
    // shadow glibc's <features.h> and break every system header.
    arguments.extend([
        "-iquote".into(),
        "src".into(),
        "-include".into(),
        "restart_observe.h".into(),
        "restart_shim.c".into(),
    ]);
    arguments.push("restart_pilot.c".into());
    arguments.push("restart_scroll.c".into());
    arguments.push("restart_ceres_door.c".into());
    arguments.push("restart_room_transition.c".into());
    arguments.push("restart_door_scrolling.c".into());
    arguments.push("restart_camera.c".into());
    arguments.push("restart_item_instruction.c".into());
    arguments.push("restart_persistence.c".into());
    arguments.push("restart_player_motion.c".into());
    arguments.push("restart_waver.c".into());
    arguments.push("restart_ripper.c".into());
    arguments.push("restart_rio.c".into());
    for path in retained_sources.keys() {
        let source = Path::new(path);
        let name = source.file_name().unwrap().to_string_lossy();
        if source.extension().is_some_and(|s| s == "c")
            && ((source.parent() == Some(Path::new("src"))
                && (name.starts_with("sm_") || ["spc_player.c", "tracing.c", "util.c"].contains(&name.as_ref())))
                || source.parent() == Some(Path::new("src/snes")))
        {
            arguments.push(path.clone());
        }
    }
    arguments.extend(["-lm".into(), "-o".into(), format!("../{LIBRARY_NAME}")]);
    let log = File::create(work.join("compiler.log")).map_err(|e| e.to_string())?;
    let mut command = Command::new(&compiler);
    command
        .args(&arguments)
        .current_dir(&root)
        .stdout(Stdio::from(log.try_clone().map_err(|e| e.to_string())?))
        .stderr(Stdio::from(log));
    let mut compiler_process = compiler_process::Compiler::spawn(&mut command)?;
    // Command retains its configured log handles after spawn. Release the parent
    // copies so Windows can rename the prepared directory after the child exits.
    drop(command);
    let start = Instant::now();
    let status = loop {
        if fs::metadata(work.join("compiler.log")).map_err(|e| e.to_string())?.len() > 1024 * 1024 {
            return Err(format!("native compiler output exceeded 1 MiB; retained {}", work.display()));
        }
        if let Some(status) = compiler_process.try_wait()? {
            break status;
        }
        if start.elapsed() > Duration::from_secs(60) {
            compiler_process.terminate();
            return Err(format!("native compiler exceeded 60 seconds; retained {}", work.display()));
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    drop(compiler_process);
    if !status.success() {
        return Err(format!("native compiler failed ({status}); see {}", work.join("compiler.log").display()));
    }
    let manifest = Manifest {
        format: "sm-native-library-v2".into(),
        source_revision: SOURCE_REVISION.into(),
        archive_sha256: ARCHIVE_SHA256.into(),
        recipe_sha256,
        compiler: compiler.to_string_lossy().into_owned(),
        compiler_sha256,
        compiler_version,
        compiler_arguments: arguments,
        original_sources,
        retained_sources,
        patches,
        library_sha256: hash_file(&work.join(LIBRARY_NAME))?,
    };
    let mut file = File::create(work.join("library.sme")).map_err(|e| e.to_string())?;
    file.write_all(&manifest.encode_for_validation()?).map_err(|e| e.to_string())?;
    // Windows refuses directory publication while the manifest handle is open.
    drop(file);
    fs::rename(&work, &destination)
        .map_err(|e| format!("publish prepared native library: {e}; retained {}", work.display()))?;
    Ok(Prepared { library, manifest: destination.join("library.sme"), reused: false })
}

pub(crate) fn verify_library(library: &Path) -> Result<Manifest, String> {
    let parent = library.parent().ok_or("library has no parent directory")?;
    let manifest = Manifest::from_value(sm_binary_evidence::read(parent.join("library.sme"))?)?;
    if manifest.format != "sm-native-library-v2"
        || manifest.source_revision != SOURCE_REVISION
        || manifest.archive_sha256 != ARCHIVE_SHA256
        || manifest.recipe_sha256 != recipe_hash()
    {
        return Err("native library manifest does not match this host recipe".into());
    }
    manifest.encode_for_validation()?;
    if hash_file(library)? != manifest.library_sha256 {
        return Err("native library SHA-256 mismatch".into());
    }
    let mut actual = BTreeMap::new();
    let root = parent.join("source");
    source_files(&root, &root, &mut actual)?;
    if actual != manifest.retained_sources {
        return Err("retained native source identity mismatch".into());
    }
    Ok(manifest)
}

#[cfg(any(test, feature = "private-checks"))]
mod tests {
    use super::*;
    #[cfg_attr(test, test)]
    pub(super) fn rejects_escape_paths() {
        for value in ["", "../x", "/x", "src/../../x"] {
            assert!(relative(Path::new(value)).is_err());
        }
        assert!(relative(Path::new("src/snes/cpu.c")).is_ok());
    }
    #[cfg_attr(test, test)]
    pub(super) fn sha256_known_vector() {
        assert_eq!(hash_bytes(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }

    fn independent_manifest_table() -> Map {
        Map::from([
            ("format".into(), "sm-native-library-v2".into()),
            ("source_revision".into(), "source-distinct".into()),
            ("archive_sha256".into(), "archive-distinct".into()),
            ("recipe_sha256".into(), "recipe-distinct".into()),
            ("compiler".into(), "compiler-path-distinct".into()),
            ("compiler_sha256".into(), "compiler-hash-distinct".into()),
            ("compiler_version".into(), "compiler-version-distinct".into()),
            ("compiler_arguments".into(), Value::Array(vec!["arg-first".into(), "arg-second".into()])),
            ("original_sources".into(), Value::Table(Map::from([("original.c".into(), "original-hash".into())]))),
            ("retained_sources".into(), Value::Table(Map::from([("retained.c".into(), "retained-hash".into())]))),
            (
                "patches".into(),
                Value::Array(vec![
                    Value::Table(Map::from([
                        ("path".into(), "patch.c".into()),
                        ("before".into(), "before-first".into()),
                        ("after".into(), "after-first".into()),
                    ])),
                    Value::Table(Map::from([
                        ("path".into(), "patch.c".into()),
                        ("before".into(), "after-first".into()),
                        ("after".into(), "after-second".into()),
                    ])),
                ]),
            ),
            ("library_sha256".into(), "library-distinct".into()),
        ])
    }

    fn manifest_bytes(table: Map) -> Vec<u8> {
        sm_binary_evidence::encode(&Value::Table(table)).unwrap()
    }

    #[cfg_attr(test, test)]
    pub(super) fn manifest_mapping_preserves_independently_distinct_fields() {
        let expected = independent_manifest_table();
        let manifest = Manifest::decode(&manifest_bytes(expected.clone())).unwrap();
        assert_eq!(manifest.format, "sm-native-library-v2");
        assert_eq!(manifest.source_revision, "source-distinct");
        assert_eq!(manifest.archive_sha256, "archive-distinct");
        assert_eq!(manifest.recipe_sha256, "recipe-distinct");
        assert_eq!(manifest.compiler, "compiler-path-distinct");
        assert_eq!(manifest.compiler_sha256, "compiler-hash-distinct");
        assert_eq!(manifest.compiler_version, "compiler-version-distinct");
        assert_eq!(manifest.compiler_arguments, ["arg-first", "arg-second"]);
        assert_eq!(manifest.original_sources, BTreeMap::from([("original.c".into(), "original-hash".into())]));
        assert_eq!(manifest.retained_sources, BTreeMap::from([("retained.c".into(), "retained-hash".into())]));
        assert_eq!(manifest.patches.len(), 2);
        assert_eq!(manifest.patches[0].path, "patch.c");
        assert_eq!(manifest.patches[0].before, "before-first");
        assert_eq!(manifest.patches[0].after, "after-first");
        assert_eq!(manifest.patches[1].path, "patch.c");
        assert_eq!(manifest.patches[1].before, "after-first");
        assert_eq!(manifest.patches[1].after, "after-second");
        assert_eq!(manifest.library_sha256, "library-distinct");
        assert_eq!(
            sm_binary_evidence::decode(&manifest.encode_for_validation().unwrap()).unwrap(),
            Value::Table(expected)
        );
    }

    #[cfg_attr(test, test)]
    pub(super) fn each_manifest_key_and_type_is_required_independently() {
        let baseline = independent_manifest_table();
        assert!(Manifest::decode(&manifest_bytes(baseline.clone())).is_ok());
        for key in baseline.keys() {
            let mut missing = baseline.clone();
            missing.remove(key);
            assert!(Manifest::decode(&manifest_bytes(missing)).is_err(), "missing {key}");
            let mut replaced = baseline.clone();
            let value = replaced.remove(key).unwrap();
            replaced.insert("unknown-key".into(), value);
            assert!(Manifest::decode(&manifest_bytes(replaced)).is_err(), "replaced {key}");
            let mut mistyped = baseline.clone();
            mistyped.insert(key.clone(), Value::Boolean(true));
            assert!(Manifest::decode(&manifest_bytes(mistyped)).is_err(), "type {key}");
        }
        let mut unknown = baseline;
        unknown.insert("unknown-key".into(), Value::Boolean(false));
        assert!(Manifest::decode(&manifest_bytes(unknown)).is_err());
    }

    #[cfg_attr(test, test)]
    pub(super) fn nested_manifest_types_and_patch_keys_are_required() {
        for key in ["original_sources", "retained_sources"] {
            let mut invalid = independent_manifest_table();
            invalid.insert(key.into(), Value::Table(Map::from([("file.c".into(), Value::Integer(1))])));
            assert!(Manifest::decode(&manifest_bytes(invalid)).is_err());
        }
        let mut arguments = independent_manifest_table();
        arguments.insert("compiler_arguments".into(), Value::Array(vec![Value::Boolean(true)]));
        assert!(Manifest::decode(&manifest_bytes(arguments)).is_err());
        let mut patch_scalar = independent_manifest_table();
        patch_scalar.insert("patches".into(), Value::Array(vec![Value::Boolean(false)]));
        assert!(Manifest::decode(&manifest_bytes(patch_scalar)).is_err());
        for key in ["path", "before", "after"] {
            for operation in [0, 1, 2] {
                let mut patch = Map::from([
                    ("path".into(), "p".into()),
                    ("before".into(), "b".into()),
                    ("after".into(), "a".into()),
                ]);
                match operation {
                    0 => {
                        patch.remove(key);
                    }
                    1 => {
                        let v = patch.remove(key).unwrap();
                        patch.insert("unknown".into(), v);
                    }
                    _ => {
                        patch.insert(key.into(), Value::Integer(1));
                    }
                }
                let mut invalid = independent_manifest_table();
                invalid.insert("patches".into(), Value::Array(vec![Value::Table(patch)]));
                assert!(Manifest::decode(&manifest_bytes(invalid)).is_err(), "patch {key}/{operation}");
            }
        }
    }

    #[cfg_attr(test, test)]
    pub(super) fn manifest_boundary_rejects_truncation_trailing_and_bad_version() {
        let bytes = manifest_bytes(independent_manifest_table());
        for end in 0..bytes.len() {
            assert!(Manifest::decode(&bytes[..end]).is_err());
        }
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(Manifest::decode(&trailing).is_err());
        let mut version = bytes;
        version[7] = b'2';
        assert!(Manifest::decode(&version).is_err());
    }
}

#[cfg(feature = "private-checks")]
pub fn private_assertions() -> Vec<(&'static str, fn())> {
    vec![
        ("rejects_escape_paths", tests::rejects_escape_paths as fn()),
        ("sha256_known_vector", tests::sha256_known_vector as fn()),
        (
            "manifest_mapping_preserves_independently_distinct_fields",
            tests::manifest_mapping_preserves_independently_distinct_fields as fn(),
        ),
        (
            "each_manifest_key_and_type_is_required_independently",
            tests::each_manifest_key_and_type_is_required_independently as fn(),
        ),
        (
            "nested_manifest_types_and_patch_keys_are_required",
            tests::nested_manifest_types_and_patch_keys_are_required as fn(),
        ),
        (
            "manifest_boundary_rejects_truncation_trailing_and_bad_version",
            tests::manifest_boundary_rejects_truncation_trailing_and_bad_version as fn(),
        ),
    ]
}
