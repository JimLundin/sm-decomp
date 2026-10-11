use crate::readiness_metadata::toml_from_slice;
use crate::{
    atomic_toml, checkpoints, digest, family, lsnes_configure, preflight,
    readiness_run::{Run, identity, safe_artifact},
    replay,
};
use anyhow::{Context, Result, ensure};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};
use toml::Value;
const INPUT_KEYS: [&str; 8] =
    ["rom_sha256", "movie_sha256", "input_sha256", "frames", "seed", "reset", "controller", "region"];
const OFFICIAL: &str = "aa56d499d0393e24c8d7c9eaf0ed337916f07c02fdb2d281613bcb1cd052a1d7";
const SOURCE: &str = "e9c0336218a47586e85b7c38341114995b3b7cdd727baf7c09c307d82072ef25";
fn document(path: &Path) -> Result<Value> {
    toml_from_slice(&fs::read(path)?)
}
fn linked(owner: &Path, value: &str) -> Result<PathBuf> {
    let path = Path::new(value);
    if path.is_absolute() { Ok(path.to_owned()) } else { safe_artifact(owner, value) }
}
fn inventory(root: &Path, dir: &Path, files: &mut BTreeMap<String, String>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        ensure!(!path.is_symlink(), "original source symlink");
        if path.is_dir() {
            inventory(root, &path, files)?;
        } else {
            files.insert(path.strip_prefix(root)?.to_string_lossy().into_owned(), digest(&path)?);
        }
    }
    Ok(())
}
fn source_inventory(root: &Path) -> Result<Value> {
    let mut files = BTreeMap::new();
    inventory(root, root, &mut files)?;
    Ok(Value::Array(
        files
            .into_iter()
            .map(|(path, sha256)| crate :: report ! ({ "path" : path , "sha256" : sha256 , }))
            .collect::<Vec<_>>(),
    ))
}
fn extract(source: &Path, destination: &Path) -> Result<()> {
    let listing = Command::new("tar").arg("-tf").arg(source).output()?;
    ensure!(listing.status.success(), "original archive listing failed");
    for name in String::from_utf8(listing.stdout)?.lines() {
        let name = name.trim_end_matches('/').strip_prefix("./").unwrap_or(name.trim_end_matches('/'));
        if !name.is_empty() {
            safe_artifact(destination, name)?;
        }
    }
    let status = Command::new("tar").arg("-xf").arg(source).arg("-C").arg(destination).status()?;
    ensure!(status.success(), "original source extraction failed");
    Ok(())
}
fn bind(run: &Run, path: &Path) -> Result<String> {
    Ok(path.strip_prefix(&run.path)?.to_string_lossy().into_owned())
}
fn stage(
    run: &mut Run,
    name: &str,
    command: &[String],
    cwd: &Path,
    env: &BTreeMap<String, String>,
    timeout: Duration,
) -> Result<Value> {
    let report = run.stage(name, command, cwd, env, timeout, &BTreeMap::new())?;
    ensure!(
        report["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata"),
        "{name} failed or incomplete"
    );
    Ok(report)
}
fn build_original(run: &mut Run) -> Result<(PathBuf, Value)> {
    let root = run.root.clone();
    let pins = document(&root.join("docs/lsnes-dependencies.toml"))?;
    let official = root.join("deps/downloads").join(family::field(&pins["archive"], "filename")?);
    let tar = root.join("deps/lsnes/sources.tar");
    ensure!(
        run.guard(&official)? == OFFICIAL && run.guard(&tar)? == SOURCE,
        "missing/invalid independently pinned original source archive; run just setup-lsnes"
    );
    let build = run.path.join("generated/original-build");
    let source = build.join("source");
    let libraries = build.join("libraries");
    fs::create_dir_all(&source)?;
    fs::create_dir(&libraries)?;
    extract(&tar, &source)?;
    let mut packages = Vec::new();
    for (index, record) in family::array(&pins, "packages")?.iter().enumerate() {
        let path = root.join("deps/downloads").join(family::field(record, "filename")?);
        ensure!(run.guard(&path)? == family::field(record, "sha256")?, "invalid original build package");
        stage(
            run,
            &format!("original-package-{index}"),
            &[
                "dpkg-deb".into(),
                "-x".into(),
                path.to_string_lossy().into_owned(),
                libraries.to_string_lossy().into_owned(),
            ],
            &root,
            &BTreeMap::new(),
            Duration::from_secs(60),
        )?;
        packages.push(
            crate :: report ! ({ "filename" : record ["filename"] , "sha256" : record ["sha256"] , "path" : path , }),
        );
    }
    let curl = fs::canonicalize("/usr/lib/x86_64-linux-gnu/libcurl.so.4")?;
    for (alias, target) in [
        (libraries.join("usr/bin/lua"), PathBuf::from("lua5.2")),
        (libraries.join("usr/bin/luac"), PathBuf::from("luac5.2")),
        (libraries.join("usr/lib/x86_64-linux-gnu/libcurl.so"), curl.clone()),
    ] {
        if alias.exists() || alias.is_symlink() {
            fs::remove_file(&alias)?;
        }
        std::os::unix::fs::symlink(target, alias)?;
    }
    lsnes_configure::configure(&source, &tar, &libraries)?;
    let files = source_inventory(&source)?;
    let source_hash = identity(&files)?;
    let inventory_path = build.join("source-inventory.toml");
    atomic_toml(&inventory_path, &crate :: report ! ({ "sha256" : source_hash , "files" : files , }))?;
    let compiler = stage(
        run,
        "original-compiler",
        &["g++".into(), "--version".into()],
        &root,
        &BTreeMap::new(),
        Duration::from_secs(30),
    )?;
    let compiler = fs::read_to_string(run.path.join(family::field(&compiler, "stdout")?))?.trim().to_owned();
    let compiler_path = Command::new("sh").args(["-c", "command -v g++"]).output()?;
    ensure!(compiler_path.status.success(), "original compiler path unavailable");
    let compiler_path = fs::canonicalize(String::from_utf8(compiler_path.stdout)?.trim())?;
    let mut dependencies = vec![
        crate :: report ! ({ "path" : compiler_path , "sha256" : run . guard (& compiler_path) ? }),
        crate :: report ! ({ "path" : curl , "sha256" : run . guard (& curl) ? }),
    ];
    fn libraries_inventory(path: &Path, out: &mut Vec<Value>) -> Result<()> {
        for entry in fs::read_dir(path)? {
            let p = entry?.path();
            if p.is_symlink() {
                continue;
            }
            if p.is_dir() {
                libraries_inventory(&p, out)?;
            } else {
                out.push(crate :: report ! ({ "path" : p , "sha256" : digest (& p) ? , }));
            }
        }
        Ok(())
    }
    libraries_inventory(&libraries, &mut dependencies)?;
    let env = BTreeMap::from([
        ("PATH".into(), format!("{}:{}", libraries.join("usr/bin").display(), std::env::var("PATH")?)),
        ("PKG_CONFIG_PATH".into(), libraries.join("usr/lib/x86_64-linux-gnu/pkgconfig").to_string_lossy().into_owned()),
        ("MAKEFLAGS".into(), String::new()),
        ("MFLAGS".into(), String::new()),
        ("LD_PRELOAD".into(), String::new()),
        ("CFLAGS".into(), String::new()),
        ("CPPFLAGS".into(), String::new()),
        ("CXXFLAGS".into(), String::new()),
        ("LDFLAGS".into(), String::new()),
    ]);
    let process = stage(
        run,
        "original-build",
        &["make".into(), "-j1".into(), "headless".into()],
        &source,
        &env,
        Duration::from_secs(1800),
    )?;
    for row in files.as_array().unwrap() {
        ensure!(
            digest(&source.join(family::field(row, "path")?))? == family::field(row, "sha256")?,
            "original source changed during build"
        );
    }
    let binary = source.join("src/util/lsnes-dumpavi.util");
    run.record_artifact("original-binary", &binary, None)?;
    run.manifest.identities.get_mut("original-binary").context("original binary identity missing")?["evidence"] =
        bind(run, &binary)?.into();
    let recipe = run.path.join("source/files/crates/pipeline/src/lsnes_configure.rs");
    let provenance = crate :: report ! ({ "method" : "verified-clean-build" , "archive_sha256" : OFFICIAL , "source_tar_sha256" : SOURCE , "source_sha256" : source_hash , "binary_sha256" : digest (& binary) ? , "build_log_sha256" : digest (& run . path . join (family :: field (& process , "stdout") ?)) ? , "build_report_sha256" : digest (& run . path . join ("logs/original-build.toml")) ? , "build_command" : process ["command"] , "compiler" : compiler , "features" : crate :: report ! (vec ! ["bsnes v085 Compatibility" , "NTSC" , "DUMMY graphics/sound/joystick"]) , "recipe_sha256" : digest (& recipe) ? , "configured_libraries" : libraries , "packages" : packages , "build_dependencies" : dependencies , "artifacts" : crate :: report ! ({ "official_archive" : official , "source_tar" : tar , "source_root" : bind (run , & source) ? , "source_inventory" : bind (run , & inventory_path) ? , "binary" : bind (run , & binary) ? , "build_log" : process ["stdout"] , "build_report" : "logs/original-build.toml" , "recipe" : bind (run , & recipe) ? , }) , });
    atomic_toml(&build.join("attestation.toml"), &provenance)?;
    run.record_artifact("original-build", &build.join("attestation.toml"), None)?;
    run.save()?;
    Ok((binary, provenance))
}
fn metadata(
    run: &Run,
    role: &str,
    out: &Path,
    observer: &Path,
    provenance: Value,
    process: &Value,
    frames: u64,
) -> Result<Value> {
    let inputs = INPUT_KEYS
        .iter()
        .map(|key| ((*key).to_owned(), run.manifest.identities["inputs"][key].clone()))
        .collect::<toml::Table>();
    let rows = checkpoints::read(out)?;
    let mut files = Vec::new();
    for path in [
        out.to_owned(),
        observer.to_owned(),
        run.root.join("rom/sm.sfc"),
        run.root.join("movies/super-metroid-any-3653.lsmv"),
        run.path.join("generated/movie.inputs"),
        linked(&run.path, family::field(&provenance["artifacts"], "binary")?)?,
    ] {
        files . push (crate :: report ! ({ "command_path" : path , "artifact" : path . strip_prefix (& run . path) . map (| p | p . to_string_lossy () . into_owned ()) . unwrap_or_else (| _ | path . to_string_lossy () . into_owned ()) , "sha256" : digest (& path) ? , })) ;
    }
    let name = if role == "original" { "original-replay" } else { "sync-reference-replay" };
    Ok(
        crate :: report ! ({ "schema_version" : 1 , "role" : role , "run_id" : run . manifest . run_id , "completed_frames" : frames , "checkpoint_sha256" : digest (out) ? , "event_count" : rows . len () , "inputs" : inputs , "observer" : crate :: report ! ({ "checkpoint_pc" : "828948" , "fields" : checkpoints :: FIELDS , "semantics" : "actual opcode fetch; discarded interrupt fetch excluded" , "frame_labels" : "lsnes currentframe; NTSC callback at V241" , "sha256" : digest (observer) ? , "artifact" : bind (run , observer) ? , }) , "provenance" : provenance , "checkpoint_artifact" : bind (run , out) ? , "capture_report" : format ! ("logs/{name}.toml") , "capture_report_sha256" : digest (& run . path . join (format ! ("logs/{name}.toml"))) ? , "command" : process ["command"] , "command_files" : files , "owner_origin" : run . path , }),
    )
}
fn capture_original(run: &mut Run, binary: &Path, provenance: Value, frames: u64) -> Result<PathBuf> {
    let out = run.path.join("generated/original.csv");
    let observer = out.with_extension("lua");
    fs::write(&observer, replay::lsnes_observer(&out)?)?;
    run.guard(&observer)?;
    let root = run.root.clone();
    let libraries = Path::new(family::field(&provenance, "configured_libraries")?).join("usr/lib/x86_64-linux-gnu");
    let env = BTreeMap::from([
        ("XDG_CONFIG_HOME".into(), run.path.join("generated/original-config").to_string_lossy().into_owned()),
        ("LD_LIBRARY_PATH".into(), libraries.to_string_lossy().into_owned()),
    ]);
    let command = vec![
        binary.to_string_lossy().into_owned(),
        "--dumper=INTERNAL-NULL".into(),
        format!("--rom-a={}", root.join("rom/sm.sfc").display()),
        format!("--length={frames}"),
        format!("--lua={}", observer.display()),
        root.join("movies/super-metroid-any-3653.lsmv").to_string_lossy().into_owned(),
    ];
    let process = stage(run, "original-replay", &command, &root, &env, Duration::from_secs(1800))?;
    let logs = fs::read_to_string(run.path.join(family::field(&process, "stdout")?))?
        + &fs::read_to_string(run.path.join(family::field(&process, "stderr")?))?;
    ensure!(
        ["Finished!", "bsnes v085 (Compatibility core)", "snes_ntsc"].iter().all(|s| logs.contains(s)),
        "original replay lacks completion/core/region diagnostics"
    );
    atomic_toml(
        &out.with_extension("capture.toml"),
        &metadata(run, "original", &out, &observer, provenance, &process, frames)?,
    )?;
    Ok(out)
}
fn capture_candidate(run: &mut Run, frames: u64) -> Result<PathBuf> {
    let target = run.path.join("generated/reference-target");
    let root = run.root.clone();
    let env = BTreeMap::from([("CARGO_TARGET_DIR".into(), target.to_string_lossy().into_owned())]);
    let process = stage(
        run,
        "sync-reference-build",
        &[
            "cargo",
            "build",
            "--release",
            "-p",
            "snes",
            "--bin",
            "reference-replay",
            "--locked",
            "--offline",
            "--jobs",
            "1",
        ]
        .map(str::to_owned),
        &root,
        &env,
        Duration::from_secs(600),
    )?;
    let binary = run.path.join("generated/reference-replay");
    run.record_artifact(
        "sync-reference-binary",
        &target.join("release/reference-replay"),
        Some("generated/reference-replay"),
    )?;
    run.guard(&binary)?;
    let observer = run.path.join("source/files/crates/snes/src/checkpoint.rs");
    let provenance = crate :: report ! ({ "method" : "verified-clean-build" , "source_sha256" : run . manifest . identities ["source"] ["sha256"] , "hardware_source_sha256" : run . manifest . identities ["hardware"] ["source_sha256"] , "lockfile_sha256" : run . manifest . identities ["Cargo.lock"] ["sha256"] , "hardware" : run . manifest . identities ["hardware"] , "hardware_root" : root . join ("deps/rustysnes") , "binary_sha256" : digest (& binary) ? , "build_log_sha256" : digest (& run . path . join (family :: field (& process , "stdout") ?)) ? , "build_report_sha256" : digest (& run . path . join ("logs/sync-reference-build.toml")) ? , "build_command" : process ["command"] , "compiler" : run . manifest . identities ["compiler"] ["version"] , "features" : run . manifest . identities ["hardware"] ["features"] , "recipe_sha256" : digest (& run . path . join ("source/files/Cargo.toml")) ? , "artifacts" : crate :: report ! ({ "binary" : "generated/reference-replay" , "build_log" : process ["stdout"] , "build_report" : "logs/sync-reference-build.toml" , "source_inventory" : "source/inventory.toml" , "source_root" : "source/files" , "recipe" : "source/files/Cargo.toml" , }) , });
    let out = run.path.join("generated/candidate.csv");
    let command = vec![
        binary.to_string_lossy().into_owned(),
        run.path.join("generated/movie.inputs").to_string_lossy().into_owned(),
        "--rom".into(),
        root.join("rom/sm.sfc").to_string_lossy().into_owned(),
        "--frames".into(),
        frames.to_string(),
        "--out".into(),
        run.path.join("generated/reference-fingerprints.csv").to_string_lossy().into_owned(),
        "--checkpoints".into(),
        out.to_string_lossy().into_owned(),
        "--lsnes-frame-labels".into(),
    ];
    let process = stage(run, "sync-reference-replay", &command, &root, &env, Duration::from_secs(2400))?;
    let raw = document(&out.with_extension("capture.toml"))?;
    ensure!(
        raw["completed_frames"] == crate::readiness_metadata::value(&frames).expect("validated native metadata")
            && raw["checkpoint_sha256"]
                == crate::readiness_metadata::value(&digest(&out)?).expect("validated native metadata"),
        "reference capture completion count/hash differs"
    );
    atomic_toml(&out.with_extension("runner.toml"), &raw)?;
    atomic_toml(
        &out.with_extension("capture.toml"),
        &metadata(run, "candidate", &out, &observer, provenance, &process, frames)?,
    )?;
    Ok(out)
}
fn verify_process(owner: &Path, path: &Path, hash: &str, command: &Value) -> Result<Value> {
    ensure!(digest(path)? == hash, "process report checksum differs");
    let process = document(path)?;
    ensure!(
        process["command"] == crate::readiness_metadata::value(command).expect("validated native metadata")
            && process["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata")
            && process["complete"] == crate::readiness_metadata::value(&true).expect("validated native metadata")
            && process["returncode"] == crate::readiness_metadata::value(&0).expect("validated native metadata")
            && process.get("signal").is_none(),
        "capture/build process lacks successful recorded invocation"
    );
    let manifest = document(&owner.join("manifest.toml"))?;
    ensure!(
        family::array(&manifest, "commands")?.contains(&process),
        "capture/build process lacks owner invocation origin"
    );
    for name in ["stdout", "stderr"] {
        ensure!(linked(owner, family::field(&process, name)?)?.is_file(), "capture/build log missing");
    }
    Ok(process)
}
pub(crate) fn verify_capture(
    path: &Path,
    role: &str,
    inputs: &Value,
    frames: u64,
) -> Result<(Vec<BTreeMap<String, i64>>, Value)> {
    let metadata = document(&path.with_extension("capture.toml"))?;
    let owner = if path.parent().is_some_and(|p| p.file_name().is_some_and(|n| n == "generated")) {
        path.parent().unwrap().parent().unwrap()
    } else {
        path.parent().context("capture parent missing")?
    };
    let manifest = document(&owner.join("manifest.toml"))?;
    ensure!(
        metadata["schema_version"] == crate::readiness_metadata::value(&1).expect("validated native metadata")
            && metadata["role"] == crate::readiness_metadata::value(&role).expect("validated native metadata")
            && metadata["run_id"] == manifest["run_id"]
            && metadata["checkpoint_sha256"]
                == crate::readiness_metadata::value(&digest(path)?).expect("validated native metadata"),
        "capture schema/role/run/checksum differs"
    );
    let completed = metadata["completed_frames"]
        .as_integer()
        .and_then(|n| u64::try_from(n).ok())
        .context("capture completion missing")?;
    ensure!(
        completed >= frames && (role != "candidate" || completed == frames),
        "capture truncated or fresh candidate scope differs"
    );
    for key in INPUT_KEYS {
        ensure!(
            !inputs.get(key).is_none() && metadata["inputs"][key] == inputs[key],
            "capture input/settings identity differs: {key}"
        );
    }
    let observer = &metadata["observer"];
    ensure!(
        observer["checkpoint_pc"] == crate::readiness_metadata::value(&"828948").expect("validated native metadata")
            && observer["fields"]
                == crate::readiness_metadata::value(&checkpoints::FIELDS).expect("validated native metadata")
            && observer["semantics"]
                == crate::readiness_metadata::value(&"actual opcode fetch; discarded interrupt fetch excluded")
                    .expect("validated native metadata")
            && observer["frame_labels"]
                == crate::readiness_metadata::value(&"lsnes currentframe; NTSC callback at V241")
                    .expect("validated native metadata"),
        "incompatible actual-fetch observer identity"
    );
    let observer_path = linked(owner, family::field(observer, "artifact")?)?;
    ensure!(digest(&observer_path)? == family::field(observer, "sha256")?, "observer artifact checksum differs");
    let provenance = &metadata["provenance"];
    let artifacts = &provenance["artifacts"];
    ensure!(
        provenance["method"]
            == crate::readiness_metadata::value(&"verified-clean-build").expect("validated native metadata")
            && !family::field(provenance, "compiler")?.is_empty()
            && !family::array(provenance, "features")?.is_empty(),
        "missing verified source/build provenance"
    );
    for (name, hash) in [("binary", "binary_sha256"), ("build_log", "build_log_sha256"), ("recipe", "recipe_sha256")] {
        ensure!(
            digest(&linked(owner, family::field(artifacts, name)?)?)? == family::field(provenance, hash)?,
            "provenance artifact checksum differs: {name}"
        );
    }
    let build = verify_process(
        owner,
        &linked(owner, family::field(artifacts, "build_report")?)?,
        family::field(provenance, "build_report_sha256")?,
        &provenance["build_command"],
    )?;
    ensure!(
        linked(owner, family::field(&build, "stdout")?)? == linked(owner, family::field(artifacts, "build_log")?)?,
        "build output log origin differs"
    );
    let inventory = document(&linked(owner, family::field(artifacts, "source_inventory")?)?)?;
    ensure!(
        identity(&inventory["files"])? == family::field(provenance, "source_sha256")?
            && inventory["sha256"] == provenance["source_sha256"],
        "source inventory checksum differs"
    );
    let source_root = linked(owner, family::field(artifacts, "source_root")?)?;
    for entry in family::array(&inventory, "files")? {
        ensure!(
            digest(&safe_artifact(&source_root, family::field(entry, "path")?)?)? == family::field(entry, "sha256")?,
            "source snapshot artifact checksum differs"
        );
    }
    verify_process(
        owner,
        &linked(owner, family::field(&metadata, "capture_report")?)?,
        family::field(&metadata, "capture_report_sha256")?,
        &metadata["command"],
    )?;
    let files = family::array(&metadata, "command_files")?;
    let mut digests = BTreeSet::new();
    let mut command_files = BTreeMap::new();
    for record in files {
        let hash = family::field(record, "sha256")?;
        let artifact = linked(owner, family::field(record, "artifact")?)?;
        ensure!(digest(&artifact)? == hash, "capture command artifact/input changed");
        digests.insert(hash.to_owned());
        command_files.insert(family::field(record, "command_path")?.to_owned(), (hash.to_owned(), artifact));
    }
    let required = [
        family::field(&metadata, "checkpoint_sha256")?,
        family::field(observer, "sha256")?,
        family::field(provenance, "binary_sha256")?,
        family::field(inputs, "rom_sha256")?,
        family::field(inputs, "movie_sha256")?,
        family::field(inputs, "input_sha256")?,
    ];
    ensure!(
        required.iter().all(|hash| digests.contains(*hash)),
        "capture command lacks required input/artifact bindings"
    );
    let command = crate::pattern_contract::strings(&metadata["command"])?;
    ensure!(
        command_files
            .get(&command[0])
            .is_some_and(|(hash, _)| hash == family::field(provenance, "binary_sha256").unwrap()),
        "capture executable differs"
    );
    let find = |hash: &str| -> Result<String> {
        let matches = command_files.iter().filter(|(_, (actual, _))| actual == hash).collect::<Vec<_>>();
        ensure!(matches.len() == 1, "ambiguous capture command artifact identity");
        Ok(matches[0].0.clone())
    };
    let checkpoint = find(family::field(&metadata, "checkpoint_sha256")?)?;
    let rom = find(family::field(inputs, "rom_sha256")?)?;
    let movie = find(family::field(inputs, "movie_sha256")?)?;
    if role == "original" {
        ensure!(
            provenance["archive_sha256"]
                == crate::readiness_metadata::value(&OFFICIAL).expect("validated native metadata")
                && provenance["source_tar_sha256"]
                    == crate::readiness_metadata::value(&SOURCE).expect("validated native metadata"),
            "original archive differs from independently pinned official origin"
        );
        let official = linked(owner, family::field(artifacts, "official_archive")?)?;
        let tar = linked(owner, family::field(artifacts, "source_tar")?)?;
        ensure!(digest(&official)? == OFFICIAL && digest(&tar)? == SOURCE, "official original source artifact differs");
        for record in
            family::array(provenance, "packages")?.iter().chain(family::array(provenance, "build_dependencies")?)
        {
            ensure!(
                digest(&linked(owner, family::field(record, "path")?)?)? == family::field(record, "sha256")?,
                "original build dependency changed"
            );
        }
        ensure!(
            provenance["build_command"] == crate::report!(vec!["make", "-j1", "headless"]),
            "original build command differs from headless target"
        );
        let observer_command = find(family::field(observer, "sha256")?)?;
        ensure!(
            command
                == vec![
                    command[0].clone(),
                    "--dumper=INTERNAL-NULL".into(),
                    format!("--rom-a={rom}"),
                    format!("--length={completed}"),
                    format!("--lua={observer_command}"),
                    movie
                ]
                && fs::read_to_string(&observer_path)? == replay::lsnes_observer(Path::new(&checkpoint))?,
            "original capture invocation/observer scope differs"
        );
        let temporary = std::env::temp_dir().join(format!(
            "sm-original-verify-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_nanos()
        ));
        fs::create_dir(&temporary)?;
        let outcome = (|| -> Result<()> {
            extract(&tar, &temporary)?;
            lsnes_configure::configure(
                &temporary,
                &tar,
                &linked(owner, family::field(provenance, "configured_libraries")?)?,
            )?;
            ensure!(
                identity(&source_inventory(&temporary)?)? == family::field(provenance, "source_sha256")?,
                "original source inventory differs from official archive plus declared build edits"
            );
            Ok(())
        })();
        fs::remove_dir_all(temporary)?;
        outcome?;
    } else {
        let identities = &manifest["identities"];
        ensure!(
            provenance["source_sha256"] == identities["source"]["sha256"]
                && provenance["hardware_source_sha256"] == identities["hardware"]["source_sha256"]
                && provenance["hardware"] == identities["hardware"]
                && provenance["lockfile_sha256"] == identities["Cargo.lock"]["sha256"]
                && provenance["compiler"] == identities["compiler"]["version"],
            "candidate provenance differs from owner identities"
        );
        ensure!(
            digest(&source_root.join("Cargo.lock"))? == family::field(provenance, "lockfile_sha256")?
                && provenance["build_command"]
                    == crate::report!(vec![
                        "cargo",
                        "build",
                        "--release",
                        "-p",
                        "snes",
                        "--bin",
                        "reference-replay",
                        "--locked",
                        "--offline",
                        "--jobs",
                        "1"
                    ]),
            "candidate lockfile/build invocation differs"
        );
        let hardware = &provenance["hardware"];
        ensure!(
            identity(&hardware["files"])? == family::field(hardware, "source_sha256")?,
            "candidate hardware inventory identity differs"
        );
        for entry in family::array(hardware, "files")? {
            ensure!(
                digest(&safe_artifact(
                    &linked(owner, family::field(provenance, "hardware_root")?)?,
                    family::field(entry, "path")?
                )?)? == family::field(entry, "sha256")?,
                "candidate hardware artifact changed"
            );
        }
        let input = find(family::field(inputs, "input_sha256")?)?;
        ensure!(
            command.len() == 11
                && command[1] == input
                && command[2] == "--rom"
                && command[3] == rom
                && command[4] == "--frames"
                && command[5] == completed.to_string()
                && command[6] == "--out"
                && command[8] == "--checkpoints"
                && command[9] == checkpoint
                && command[10] == "--lsnes-frame-labels",
            "candidate capture invocation scope/labels differs"
        );
        ensure!(
            observer_path == source_root.join("crates/snes/src/checkpoint.rs"),
            "candidate observer source differs"
        );
    }
    let rows = checkpoints::read(path)?;
    ensure!(
        metadata["event_count"] == crate::readiness_metadata::value(&rows.len()).expect("validated native metadata")
            && rows.last().unwrap()["frame"] <= completed as i64 + 1,
        "capture event/completion count mismatch"
    );
    Ok((rows, metadata))
}
pub(crate) fn compare(original: &Path, candidate: &Path, inputs: &Value, frames: u64) -> Result<Value> {
    let start = Instant::now();
    let (expected, original_metadata) = verify_capture(original, "original", inputs, frames)?;
    let (actual, candidate_metadata) = verify_capture(candidate, "candidate", inputs, frames)?;
    let compared = checkpoints::compare(&expected, &actual, 272, frames as i64, 0)?;
    let mut report = Value::try_from(&compared)?;
    let table = report.as_table_mut().unwrap();
    table.insert("status".into(), Value::String(if compared.status == "PASS" { "pass" } else { "divergence" }.into()));
    table.insert("complete".into(), Value::Boolean(true));
    for (name, value) in [
        ("expected_frames", Value::try_from(frames)?),
        ("completed_frames", Value::try_from(frames)?),
        ("original_completed_frames", original_metadata["completed_frames"].clone()),
        ("candidate_completed_frames", candidate_metadata["completed_frames"].clone()),
        ("original", Value::try_from(original)?),
        ("candidate", Value::try_from(candidate)?),
        ("checkpoint_pc", Value::String("828948".into())),
        ("original_sha256", Value::String(digest(original)?)),
        ("candidate_sha256", Value::String(digest(candidate)?)),
        ("elapsed_seconds", Value::Float(start.elapsed().as_secs_f64())),
    ] {
        table.insert(name.into(), value);
    }
    Ok(report)
}
pub(crate) fn synchronize(run: &mut Run, original_run: Option<&Path>, frames: u64) -> Result<Value> {
    ensure!(frames >= 272, "synchronization requires checkpoint-bearing prefix");
    let began = Instant::now();
    let original = if let Some(owner) = original_run {
        let owner = fs::canonicalize(owner)?;
        let verdict = document(&owner.join("verdict.toml"))?;
        let manifest = document(&owner.join("manifest.toml"))?;
        ensure!(
            verdict["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata")
                && manifest["state"]
                    == crate::readiness_metadata::value(&"complete").expect("validated native metadata")
                && ["synchronization", "original-attestation"].iter().any(|name| verdict["checks"][name]["status"]
                    == crate::readiness_metadata::value(&"pass").expect("validated native metadata")),
            "original reuse requires complete passing immutable run"
        );
        let original = if manifest["type"]
            == crate::readiness_metadata::value(&"original-attestation").expect("validated native metadata")
        {
            let (sealed, _) = crate::readiness_origins::snapshot(&owner)?;
            crate::readiness_origins::owned(&owner, &sealed)?;
            let report = &verdict["checks"]["original-attestation"];
            ensure!(
                report["complete"] == crate::readiness_metadata::value(&true).expect("validated native metadata")
                    && report["status"]
                        == crate::readiness_metadata::value(&"pass").expect("validated native metadata"),
                "attestation incomplete"
            );
            let capture = PathBuf::from(family::field(report, "capture")?);
            ensure!(digest(&capture)? == family::field(report, "checkpoint_sha256")?, "attested capture changed");
            capture
        } else {
            owner.join("generated/original.csv")
        };
        let (_, metadata) = verify_capture(&original, "original", &run.manifest.identities["inputs"], frames)?;
        run.guard(&original)?;
        run.guard(&original.with_extension("capture.toml"))?;
        run . manifest . reused_evidence . push (crate :: report ! ({ "kind" : "original-oracle" , "run_id" : metadata ["run_id"] , "path" : owner , "checkpoint_sha256" : digest (& original) ? }) . as_table () . unwrap () . clone ()) ;
        run.save()?;
        original
    } else {
        let (binary, provenance) = build_original(run)?;
        capture_original(run, &binary, provenance, frames)?
    };
    let candidate = capture_candidate(run, frames)?;
    let mut report = compare(&original, &candidate, &run.manifest.identities["inputs"], frames)?;
    {
        let value: toml::Value = Value::Float(began.elapsed().as_secs_f64());
        report.as_table_mut().context("native metadata table missing")?.insert(("elapsed_seconds").into(), value);
    };
    run.check_sources()?;
    run.record_check("synchronization", report.clone())?;
    Ok(report)
}
pub fn operate(request: Value) -> Result<Value> {
    let root = request.get("checkout").and_then(Value::as_str).map(PathBuf::from).unwrap_or(std::env::current_dir()?);
    if let Some(previous) = request.get("attest_original").and_then(Value::as_str) {
        return attest(&root, Path::new(previous));
    }
    let frames = request.get("frames").and_then(Value::as_integer).and_then(|n| u64::try_from(n).ok()).unwrap_or(18000);
    let mut run = Run::create(
        &root,
        "synchronization",
        crate :: report ! ({ "domain" : "Japan/USA any%" , "frames" : frames , }),
        None,
    )?;
    eprintln!("Run: {}", run.path.display());
    let outcome = (|| -> Result<Value> {
        let failures = preflight::verify(&mut run)?;
        ensure!(failures.is_empty(), "preflight failed: {}", failures.join("; "));
        synchronize(&mut run, request.get("original_run").and_then(Value::as_str).map(Path::new), frames)
    })();
    match outcome {
        Ok(report) => run.finalize(
            family::field(&report, "status")?,
            Some("synchronization"),
            report
                .get("reason")
                .and_then(Value::as_str)
                .or(Some("strict original-core scope complete; readiness gates remain pending")),
            &["preflight".into(), "synchronization".into()],
        ),
        Err(error) => run.finalize(
            if run.was_interrupted() { "incomplete" } else { "error" },
            Some("synchronization"),
            Some(&format!("{error:#}")),
            &[],
        ),
    }
}
fn attest(root: &Path, previous: &Path) -> Result<Value> {
    let previous = fs::canonicalize(previous)?;
    let mut run =
        Run::create(root, "original-attestation", crate :: report ! ({ "checkpoint_pc" : "828948" , }), None)?;
    eprintln!("Run: {}", run.path.display());
    let result = (|| -> Result<()> {
        run.capture_source()?;
        let (manifest, _) = crate::readiness_origins::snapshot(&previous)?;
        crate::readiness_origins::owned(&previous, &manifest)?;
        ensure!(
            manifest["state"] == crate::readiness_metadata::value(&"complete").expect("validated native metadata"),
            "original capture owner incomplete"
        );
        let capture = previous.join("generated/original.csv");
        let metadata = document(&capture.with_extension("capture.toml"))?;
        let frames = metadata["completed_frames"]
            .as_integer()
            .and_then(|n| u64::try_from(n).ok())
            .filter(|n| *n >= 272)
            .context("original scope too short")?;
        let (rows, metadata) = verify_capture(&capture, "original", &metadata["inputs"], frames)?;
        run.guard(&capture)?;
        run.guard(&capture.with_extension("capture.toml"))?;
        run . record_check ("original-attestation" , crate :: report ! ({ "status" : "pass" , "complete" : true , "capture" : capture , "capture_owner" : previous , "capture_run_id" : metadata ["run_id"] , "completed_frames" : frames , "events" : rows . len () , "checkpoint_sha256" : digest (& capture) ? , "capture_metadata_sha256" : digest (& capture . with_extension ("capture.toml")) ? , }) ,) ? ;
        run.check_sources()
    })();
    match result {
        Ok(()) => run.finalize(
            "pass",
            Some("original-attestation"),
            Some("original-only origin verified; retained capture owner remains required"),
            &["original-attestation".into()],
        ),
        Err(error) => run.finalize("error", Some("original-attestation"), Some(&format!("{error:#}")), &[]),
    }
}
