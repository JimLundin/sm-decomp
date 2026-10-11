//! Bootstrap pinned sources before the main Cargo workspace can be resolved.
mod lsnes;
use anyhow::{Context, Result, bail, ensure};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

const PATCHES: [&str; 7] = [
    "rustysnes-clocked-apu",
    "rustysnes-clocked-dma",
    "rustysnes-clocked-interrupts",
    "rustysnes-lsnes-reset",
    "rustysnes-readiness-observation",
    "rustysnes-readiness-checkpoints",
    "rustysnes-hdma",
];
static NEXT: AtomicUsize = AtomicUsize::new(0);

fn digest(path: &Path) -> Result<String> {
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
    Ok(format!("{:x}", hash.finalize()))
}
fn document<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    Ok(toml::from_str(&fs::read_to_string(path)?)?)
}
#[derive(Deserialize)]
struct Dependencies {
    schema_version: u32,
    dependencies: Vec<Dependency>,
}
#[derive(Deserialize)]
struct Dependency {
    name: String,
    kind: String,
    archive: String,
    sha256: String,
    url: String,
}
#[derive(Deserialize)]
struct PatchInventory {
    schema_version: u32,
    files: Vec<PatchRecord>,
}
#[derive(Deserialize)]
struct PatchRecord {
    path: String,
    original_sha256: Option<String>,
    patched_sha256: String,
}
#[derive(Deserialize)]
struct Movies {
    schema_version: u32,
    movies: Vec<MoviePin>,
}
#[derive(Deserialize)]
struct MoviePin {
    file: String,
    sha256: String,
}
fn run(root: &Path, command: &mut Command) -> Result<()> {
    let status = command.current_dir(root).status()?;
    ensure!(status.success(), "command failed ({status}): {command:?}");
    Ok(())
}
fn prepare(root: &Path, dep: &Dependency, offline: bool) -> Result<()> {
    let name = dep.name.as_str();
    let kind = dep.kind.as_str();
    let expected = dep.sha256.as_str();
    let archive = root.join("deps/downloads").join(&dep.archive);
    fs::create_dir_all(archive.parent().unwrap())?;
    if !archive.exists() {
        ensure!(!offline, "missing cached download: {}", archive.display());
        let partial = archive.with_extension("part");
        run(
            root,
            Command::new("curl")
                .args([
                    "-L",
                    "--fail",
                    "--silent",
                    "--show-error",
                    "--retry",
                    "3",
                    "--connect-timeout",
                    "30",
                    &dep.url,
                    "-o",
                ])
                .arg(&partial),
        )?;
        ensure!(digest(&partial)? == expected, "download checksum mismatch: {}", partial.display());
        fs::rename(&partial, &archive)?;
    }
    ensure!(digest(&archive)? == expected, "archive checksum mismatch: {}", archive.display());
    let destination = root.join("deps").join(if kind == "runtime-deb" { "mesen-runtime" } else { name });
    let stamp = destination.join(format!(".setup-{name}"));
    if stamp.exists() && fs::read_to_string(&stamp)?.trim() == expected {
        println!("Ready: {name}");
        return Ok(());
    }
    fs::create_dir_all(&destination)?;
    match kind {
        "source" | "native-tests" => {
            let mut cmd = Command::new("tar");
            cmd.arg("-xzf").arg(&archive).arg("--strip-components=1").arg("-C").arg(&destination);
            if kind == "native-tests" {
                cmd.args(["--wildcards", "*/README.md", "*/v1/*.n.json"]);
            }
            run(root, &mut cmd)?;
        }
        "zip" => run(root, Command::new("unzip").arg("-qo").arg(&archive).arg("-d").arg(&destination))?,
        "runtime-deb" | "tool-deb" => run(root, Command::new("dpkg-deb").arg("-x").arg(&archive).arg(&destination))?,
        _ => bail!("unsupported dependency kind: {kind}"),
    }
    fs::write(stamp, format!("{expected}\n"))?;
    Ok(())
}

fn records(root: &Path, name: &str) -> Result<Vec<PatchRecord>> {
    if name == "rustysnes-hdma" {
        return Ok(vec![PatchRecord {
            path: "crates/rustysnes-core/src/dma.rs".into(),
            original_sha256: Some("caf0b70ac78a39a6a2bfa1e5389ee904d18535cdf1ebf17eb1cde7b38214fe75".into()),
            patched_sha256: "c854a6c72ed20c5863819befa551f4b2f74d67656602c2a1a979785f9250b718".into(),
        }]);
    }
    let record: PatchInventory = document(&root.join(format!("tools/patches/{name}.toml")))?;
    ensure!(record.schema_version == 1, "unknown patch inventory version");
    Ok(record.files)
}
fn apply_patches(root: &Path) -> Result<()> {
    // This order is part of the profile; later patches intentionally edit earlier targets.
    let order = [PATCHES[6], PATCHES[0], PATCHES[1], PATCHES[2], PATCHES[3], PATCHES[4], PATCHES[5]];
    let all = order.iter().map(|name| records(root, name)).collect::<Result<Vec<_>>>()?;
    let source = root.join("deps/rustysnes");
    for (index, name) in order.iter().enumerate() {
        let identities = all[index]
            .iter()
            .map(|r| {
                let target = source.join(&r.path);
                Ok(if target.exists() { Some(digest(&target)?) } else { None })
            })
            .collect::<Result<Vec<_>>>()?;
        let already = all[index].iter().zip(&identities).all(|(record, identity)| {
            all[index..]
                .iter()
                .flatten()
                .any(|later| later.path == record.path && Some(later.patched_sha256.as_str()) == identity.as_deref())
        });
        if already {
            println!("Ready: {name}");
            continue;
        }
        ensure!(
            all[index]
                .iter()
                .zip(&identities)
                .all(|(record, identity)| record.original_sha256.as_deref() == identity.as_deref()),
            "{name} targets differ from pinned source; preserve and review local edits"
        );
        run(
            &source,
            Command::new("patch").args(["--batch", "-p1", "-i"]).arg(root.join(format!("tools/patches/{name}.patch"))),
        )?;
        for record in &all[index] {
            ensure!(
                digest(&source.join(&record.path))? == record.patched_sha256.as_str(),
                "{name} patch checksum mismatch"
            );
        }
    }
    // Also authenticate final state, rather than treating any intermediate as ready.
    let mut final_hashes = BTreeMap::new();
    for record in all.iter().flatten() {
        final_hashes.insert(&record.path, record.patched_sha256.as_str());
    }
    for (path, expected) in final_hashes {
        ensure!(digest(&source.join(path))? == expected, "hardware profile differs: {path}");
    }
    Ok(())
}

fn symbols(root: &Path) -> Result<()> {
    let mut symbols = BTreeMap::new();
    let names = fs::read_to_string(root.join("deps/snesrev-sm/assets/names.txt"))?;
    for line in names.lines() {
        let (address, name) = line.split_once(char::is_whitespace).context("invalid label")?;
        let value = u32::from_str_radix(address.strip_prefix("0x").unwrap_or(address), 16)?;
        if value >= 0x808000 && value & 0xffff >= 0x8000 {
            symbols.insert(value, name.trim_start());
        }
    }
    let text = toml::to_string(
        &symbols.iter().map(|(address, name)| (format!("{address:06X}"), *name)).collect::<BTreeMap<_, _>>(),
    )?;
    fs::create_dir_all(root.join("out"))?;
    fs::write(root.join("out/symbols.toml"), text)?;
    println!("symbols: {} ROM labels -> out/symbols.toml", symbols.len());
    Ok(())
}

fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            ensure!(!entry.file_type()?.is_symlink(), "unexpected hardware source symlink");
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

struct Temporary(PathBuf);
impl Temporary {
    fn new(root: &Path) -> Result<Self> {
        let path = root.join(format!(
            "out/bootstrap/hardware-{}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_nanos()
        ));
        fs::create_dir_all(path.parent().unwrap())?;
        fs::create_dir(&path)?;
        Ok(Self(path))
    }
}
impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn hardware_check(root: &Path) -> Result<()> {
    let workspace = Temporary::new(root)?;
    for name in ["rustysnes-apu", "rustysnes-savestate"] {
        copy_tree(&root.join("deps/rustysnes/crates").join(name), &workspace.0.join(name))?;
    }
    fs::write(
        workspace.0.join("Cargo.toml"),
        "[workspace]\nresolver = \"3\"\nmembers = [\"rustysnes-apu\", \"rustysnes-savestate\"]\n[workspace.package]\nedition = \"2024\"\nrust-version = \"1.96\"\nlicense = \"MIT OR Apache-2.0\"\nauthors = []\n[workspace.lints.rust]\nunsafe_code = \"forbid\"\n[workspace.lints.clippy]\nall = \"warn\"\n",
    )?;
    let lock = fs::read_to_string(root.join("Cargo.lock"))?;
    let mut packages = lock.split("[[package]]");
    let mut retained = packages.next().unwrap().to_owned();
    for package in packages {
        if package.contains("source = \"registry+")
            || ["rustysnes-apu", "rustysnes-savestate"]
                .iter()
                .any(|name| package.contains(&format!("name = \"{name}\"")))
        {
            retained.push_str("[[package]]");
            retained.push_str(package);
        }
    }
    fs::write(workspace.0.join("Cargo.lock"), retained)?;
    run(
        root,
        Command::new("cargo")
            .args(["test", "--manifest-path"])
            .arg(workspace.0.join("Cargo.toml"))
            .args(["-p", "rustysnes-apu", "--lib", "--features", "clocked-smp", "--offline"])
            .env("CARGO_TARGET_DIR", root.join("target/hardware-tests")),
    )
}

fn doctor(root: &Path) -> Result<()> {
    let mut missing = Vec::new();
    let mut check = |label: &str, ok: bool| {
        println!("{}: {label}", if ok { "OK" } else { "MISSING" });
        if !ok {
            missing.push(label.to_owned());
        }
    };
    let search = std::env::var_os("PATH").context("PATH missing")?;
    for tool in
        ["rustup", "cargo", "cc", "g++", "make", "pkg-config", "curl", "tar", "patch", "unzip", "dpkg-deb", "just"]
    {
        check(tool, std::env::split_paths(&search).any(|dir| dir.join(tool).is_file()));
    }
    let toolchain = fs::read_to_string(root.join("rust-toolchain.toml"))?;
    let channel = toolchain
        .lines()
        .find_map(|line| line.strip_prefix("channel = \"").and_then(|v| v.strip_suffix('"')))
        .context("toolchain channel missing")?;
    let installed = Command::new("rustup").args(["run", channel, "rustc", "--version"]).output()?;
    check(
        &format!("Rust {channel}"),
        installed.status.success() && String::from_utf8_lossy(&installed.stdout).contains("nightly"),
    );
    check(
        "unheadered Super Metroid ROM (expected SHA-256)",
        digest(&root.join("rom/sm.sfc"))
            .is_ok_and(|v| v == "12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72"),
    );
    check("RustySNES sources", root.join("deps/rustysnes/crates/rustysnes-core/Cargo.toml").exists());
    let mut final_hashes = BTreeMap::new();
    for name in [
        "rustysnes-hdma",
        "rustysnes-clocked-apu",
        "rustysnes-clocked-dma",
        "rustysnes-clocked-interrupts",
        "rustysnes-lsnes-reset",
        "rustysnes-readiness-observation",
        "rustysnes-readiness-checkpoints",
    ] {
        for record in records(root, name)? {
            final_hashes.insert(record.path, record.patched_sha256);
        }
    }
    for (path, expected) in final_hashes {
        check(
            &format!("hardware profile patch {path}"),
            digest(&root.join("deps/rustysnes").join(&path)).is_ok_and(|v| v == expected),
        );
    }
    check("snesrev symbols", root.join("out/symbols.toml").exists());
    let count = fs::read_dir(root.join("deps/65816/v1"))
        .map(|entries| {
            entries.filter_map(|e| e.ok()).filter(|e| e.file_name().to_string_lossy().ends_with(".n.json")).count()
        })
        .unwrap_or(0);
    check("65816 corpus (256 native-mode opcode files)", count == 256);
    check("MesenCE binary", root.join("deps/mesen/Mesen").exists());
    check("original lsnes recording core", root.join("deps/lsnes/source/src/util/lsnes-dumpavi.util").exists());
    let movies: Movies = document(&root.join("movies/manifest.toml"))?;
    ensure!(movies.schema_version == 1, "unknown movie manifest version");
    for movie in movies.movies {
        let file = &movie.file;
        check(&format!("TAS movie {file}"), digest(&root.join("movies").join(file)).is_ok_and(|v| v == movie.sha256));
    }
    ensure!(missing.is_empty(), "missing or invalid prerequisites: {}", missing.join(", "));
    Ok(())
}

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let operation = args.first().map(String::as_str).unwrap_or("help");
    let root = std::env::var_os("SM_PROJECT_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().to_path_buf());
    let root = fs::canonicalize(root)?;
    let offline = args.iter().any(|v| v == "--offline");
    let skip_mesen = args.iter().any(|v| v == "--skip-mesen");
    if ["setup-lsnes", "configure-lsnes"].contains(&operation) {
        let mut jobs = std::thread::available_parallelism().map_or(1, usize::from).min(4);
        let mut index = 1;
        while index < args.len() {
            match args[index].as_str() {
                "--offline" => {}
                "--jobs" => {
                    index += 1;
                    jobs = args.get(index).context("jobs missing")?.parse()?;
                }
                _ => bail!("unknown lsnes option: {}", args[index]),
            }
            index += 1;
        }
        return lsnes::setup(&root, offline, jobs, operation == "configure-lsnes");
    }
    ensure!(
        args.iter().skip(1).all(|v| ["--offline", "--skip-mesen"].contains(&v.as_str())),
        "unknown bootstrap option"
    );
    match operation {
        "prepare-rustysnes" => {
            let deps: Dependencies = document(&root.join("tools/dependencies.toml"))?;
            ensure!(deps.schema_version == 1, "unknown dependencies version");
            let dep = deps.dependencies.iter().find(|r| r.name == "rustysnes").context("RustySNES pin missing")?;
            prepare(&root, dep, offline)?;
            apply_patches(&root)
        }
        "symbols" => symbols(&root),
        "hardware-check" => hardware_check(&root),
        "doctor" => doctor(&root),
        "setup" => {
            ensure!(
                skip_mesen || cfg!(all(target_os = "linux", target_arch = "x86_64")),
                "Mesen runtime requires Linux x64; use --skip-mesen"
            );
            let deps: Dependencies = document(&root.join("tools/dependencies.toml"))?;
            ensure!(deps.schema_version == 1, "unknown dependencies version");
            for dep in &deps.dependencies {
                if skip_mesen && !["source", "native-tests"].contains(&dep.kind.as_str()) {
                    continue;
                }
                prepare(&root, dep, offline)?;
                if dep.name == "rustysnes" {
                    apply_patches(&root)?;
                }
            }
            if !skip_mesen {
                let libraries = root.join("deps/mesen-runtime/lib/x86_64-linux-gnu");
                run(
                    &root,
                    Command::new(root.join("deps/patchelf/usr/bin/patchelf"))
                        .arg("--set-interpreter")
                        .arg(libraries.join("ld-linux-x86-64.so.2"))
                        .arg("--set-rpath")
                        .arg(libraries)
                        .arg(root.join("deps/mesen/Mesen")),
                )?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    fs::set_permissions(root.join("deps/mesen/Mesen"), fs::Permissions::from_mode(0o755))?;
                }
            }
            symbols(&root)?;
            let mut fetch = Command::new("cargo");
            fetch.args(["fetch", "--locked"]);
            if offline {
                fetch.arg("--offline");
            }
            run(&root, &mut fetch)?;
            let mut build = Command::new("cargo");
            build.args(["build", "--workspace", "--locked"]);
            if offline {
                build.arg("--offline");
            }
            run(&root, &mut build)
        }
        _ => {
            bail!("usage: sm-bootstrap prepare-rustysnes | setup [--offline] [--skip-mesen] | symbols | hardware-check")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unexpected_hardware_edits_are_rejected_before_patch_execution() {
        let root = std::env::temp_dir().join(format!("sm-bootstrap-test-{}", std::process::id()));
        fs::create_dir_all(root.join("deps/rustysnes/crates/rustysnes-core/src")).unwrap();
        let project = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
        copy_tree(&project.join("tools/patches"), &root.join("tools/patches")).unwrap();
        fs::write(root.join("deps/rustysnes/crates/rustysnes-core/src/dma.rs"), b"local edits").unwrap();
        assert!(apply_patches(&root).unwrap_err().to_string().contains("preserve and review local edits"));
        fs::remove_dir_all(root).unwrap();
    }
}
