use crate::digest;
use crate::readiness_metadata::toml_from_slice;
use crate::readiness_run::Run;
use crate::readiness_run::identity;
use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    time::Duration,
};
use toml::Value;
fn document(path: &Path) -> Result<Value> {
    toml_from_slice(&fs::read(path)?)
}
pub(crate) fn movie_inputs(run: &mut Run, entry: &Value) -> Result<Value> {
    let rom = run.root.join("rom/sm.sfc");
    let movie = run.root.join("movies").join(crate::documents::field(entry, "file")?);
    let rom_hash = run.guard(&rom).context("missing ROM: place verified unheadered Japan/USA ROM at rom/sm.sfc")?;
    ensure!(
        entry["rom_sha256"] == crate::readiness_metadata::value(&rom_hash).expect("validated native metadata"),
        "ROM checksum mismatch"
    );
    let movie_hash = run.guard(&movie).context("original movie missing: run just fetch-movies")?;
    ensure!(
        entry["sha256"] == crate::readiness_metadata::value(&movie_hash).expect("validated native metadata"),
        "movie checksum mismatch: run just fetch-movies"
    );
    let mut archive = zip::ZipArchive::new(File::open(&movie)?)?;
    let mut names = BTreeSet::new();
    for name in archive.file_names() {
        ensure!(names.insert(name.to_owned()), "duplicate movie members");
        ensure!(
            !matches!(name, "savestate" | "savestate.anchor" | "movie.bin")
                && !name.starts_with("moviesram.")
                && !name.starts_with("ramcontent.")
                && !name.starts_with("setting.")
                && !(name.starts_with("port") && name[4..].chars().all(|c| c.is_ascii_digit())),
            "unsupported movie initialization: {name}"
        );
    }
    for (name, expected) in [
        ("gametype", "snes_ntsc"),
        ("controlsversion", "0"),
        ("coreversion", "bsnes v085 (Compatibility core)"),
        ("rom.sha256", &rom_hash),
    ] {
        let mut text = String::new();
        archive.by_name(name)?.read_to_string(&mut text)?;
        ensure!(text.trim() == expected, "movie container identity differs: {name}");
    }
    let mut raw = Vec::new();
    archive.by_name("input")?.read_to_end(&mut raw)?;
    let mut pads = Vec::new();
    let buttons = b"BYsSudlrAXLR";
    for line in std::str::from_utf8(&raw)?.lines() {
        let (header, pad) = line.split_once('|').context("movie controller record missing")?;
        ensure!(
            ["F.", "F. 0 0"].contains(&header) && pad.len() == 12,
            "unsupported movie reset/subframe/controller record"
        );
        let mut mask = 0u16;
        for (i, b) in pad.bytes().enumerate() {
            ensure!(b == b'.' || b == b' ' || b == buttons[i], "invalid movie input button");
            if b == buttons[i] {
                mask |= 0x8000 >> i;
            }
        }
        pads.push(mask);
    }
    ensure!(
        !pads.is_empty() && entry["frames"].as_integer().and_then(|n| u64::try_from(n).ok()) == Some(pads.len() as u64),
        "movie input frame count differs from maintained manifest"
    );
    let converted = movie.with_extension("inputs");
    if converted.exists() {
        run.guard(&converted)?;
        let actual = fs::read_to_string(&converted)?
            .lines()
            .filter(|s| !s.trim().is_empty() && !s.starts_with('#'))
            .map(|s| Ok(u16::from_str_radix(s.trim(), 16)?))
            .collect::<Result<Vec<_>>>()?;
        ensure!(actual == pads, "converted movie input differs from original container: run just import-movies");
    }
    let table = run.root.join("out/generated/steps.rs");
    let mut table_status = "unavailable; regenerated table must be verified before compiled replay";
    let mut prior = None;
    if table.exists() {
        let hash = digest(&table)?;
        let text = fs::read_to_string(&table)?;
        let suffix = text.split_once("ROM_SHA256:").context("compiled table lacks ROM identity")?.1;
        let suffix = suffix.split_once("Some(\"").context("compiled table ROM identity missing")?.1;
        ensure!(
            suffix.split('"').next().unwrap().eq_ignore_ascii_case(&rom_hash) && digest(&table)? == hash,
            "compiled table ROM identity differs: regenerate with just recompile-steps"
        );
        prior = Some(hash);
        table_status = "verified";
    }
    let output = run.path.join("generated/movie.inputs");
    ensure!(!output.exists(), "verified input evidence already exists");
    fs::write(&output, pads.iter().map(|pad| format!("{pad:04X}\n")).collect::<String>())?;
    run.record_artifact("verified-movie-inputs", &output, None)?;
    let mut record = crate :: report ! ({ "rom_sha256" : rom_hash , "movie_sha256" : movie_hash , "container_input_sha256" : format ! ("{:x}" , Sha256 :: digest (raw)) , "input_sha256" : digest (& output) ? , "input_format" : "P1 hexadecimal masks, uppercase, LF" , "frames" : pads . len () , "seed" : 0 , "reset" : "clean default power-on; no reset/subframe records" , "profile" : "lsnes-profile" , "controller" : "P1 SNES gamepad" , "region" : "NTSC" , "compiled_table_rom" : table_status , "input_evidence" : "generated/movie.inputs" , });
    if let Some(hash) = prior {
        {
            let value: toml::Value = hash.into();
            record
                .as_table_mut()
                .context("native metadata table missing")?
                .insert(("prior_compiled_table_sha256").into(), value);
        };
    }
    run.manifest.identities.insert("inputs".into(), record.clone());
    run.save()?;
    Ok(record)
}
fn build_input(relative: &str) -> bool {
    let p = Path::new(relative);
    let parts = p.iter().map(|v| v.to_string_lossy()).collect::<Vec<_>>();
    ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "rustfmt.toml"].contains(&relative)
        || (parts.len() >= 3
            && parts[0] == "crates"
            && [
                "rustysnes-core",
                "rustysnes-cpu",
                "rustysnes-ppu",
                "rustysnes-apu",
                "rustysnes-cart",
                "rustysnes-savestate",
            ]
            .contains(&parts[1].as_ref())
            && (parts[2] == "src" || ["Cargo.toml", "build.rs"].contains(&parts[2].as_ref())))
}
fn files(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let p = entry?.path();
        let relative = p.strip_prefix(root)?.to_str().context("hardware path UTF-8")?;
        if p.is_dir()
            && ![
                "crates",
                "crates/rustysnes-core",
                "crates/rustysnes-cpu",
                "crates/rustysnes-ppu",
                "crates/rustysnes-apu",
                "crates/rustysnes-cart",
                "crates/rustysnes-savestate",
            ]
            .contains(&relative)
            && !build_input(relative)
        {
            continue;
        }
        if !p.is_dir() && !build_input(relative) {
            continue;
        }
        ensure!(!p.is_symlink(), "hardware source symlink");
        if p.is_dir() {
            files(root, &p, out)?;
        } else if build_input(p.strip_prefix(root)?.to_str().context("hardware path UTF-8")?) {
            out.push(p);
        }
    }
    Ok(())
}
struct Temporary(PathBuf);
impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
pub(crate) fn hardware(run: &mut Run) -> Result<Value> {
    let dependencies = document(&run.root.join("tools/dependencies.toml"))?;
    let pin = dependencies
        .get("dependencies")
        .and_then(Value::as_array)
        .context("dependency pins missing")?
        .iter()
        .find(|v| v["name"] == crate::readiness_metadata::value(&"rustysnes").expect("validated native metadata"))
        .context("hardware dependency pin missing")?;
    let archive = run.root.join("deps/downloads").join(crate::documents::field(pin, "archive")?);
    ensure!(
        run.guard(&archive)? == crate::documents::field(pin, "sha256")?,
        "pinned hardware download missing or invalid: run just setup"
    );
    let temporary = run.path.join("generated/hardware-origin");
    fs::create_dir(&temporary)?;
    let _temporary = Temporary(temporary.clone());
    let report = run.stage(
        "hardware-archive",
        &[
            "tar".into(),
            "-xzf".into(),
            archive.to_string_lossy().into_owned(),
            "--strip-components=1".into(),
            "-C".into(),
            temporary.to_string_lossy().into_owned(),
        ],
        &run.root.clone(),
        &BTreeMap::new(),
        Duration::from_secs(60),
        &BTreeMap::new(),
    )?;
    ensure!(
        report["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata"),
        "hardware archive extraction failed"
    );
    let mut patches = Vec::new();
    for name in [
        "rustysnes-hdma",
        "rustysnes-clocked-apu",
        "rustysnes-clocked-dma",
        "rustysnes-clocked-interrupts",
        "rustysnes-lsnes-reset",
        "rustysnes-readiness-observation",
        "rustysnes-readiness-checkpoints",
    ] {
        let relative = format!("tools/patches/{name}.patch");
        let patch = run.root.join(&relative);
        patches.push(crate :: report ! ({ "path" : relative , "sha256" : run . guard (& patch) ? , }));
        let process = run.stage(
            &format!("hardware-patch-{name}"),
            &[
                "patch".into(),
                "--batch".into(),
                "--forward".into(),
                "-p1".into(),
                "-i".into(),
                patch.to_string_lossy().into_owned(),
            ],
            &temporary,
            &BTreeMap::new(),
            Duration::from_secs(60),
            &BTreeMap::new(),
        )?;
        ensure!(
            process["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata"),
            "hardware patch reconstruction failed: {name}"
        );
        let record = patch.with_extension("toml");
        if record.exists() {
            run.guard(&record)?;
            for entry in document(&record)?.get("files").and_then(Value::as_array).context("patch records missing")? {
                ensure!(
                    digest(&temporary.join(crate::documents::field(entry, "path")?))?
                        == crate::documents::field(entry, "patched_sha256")?,
                    "reconstructed patch differs from declared output"
                );
            }
        } else if name == "rustysnes-hdma" {
            ensure!(
                digest(&temporary.join("crates/rustysnes-core/src/dma.rs"))?
                    == "c854a6c72ed20c5863819befa551f4b2f74d67656602c2a1a979785f9250b718",
                "HDMA reconstruction differs"
            );
        }
    }
    let mut reconstructed = Vec::new();
    files(&temporary, &temporary, &mut reconstructed)?;
    let mut expected = BTreeMap::new();
    for path in reconstructed {
        expected.insert(path.strip_prefix(&temporary)?.to_string_lossy().into_owned(), digest(&path)?);
    }
    let upstream = run.root.join("deps/rustysnes");
    let mut actual_files = Vec::new();
    files(&upstream, &upstream, &mut actual_files)?;
    let actual_names = actual_files
        .iter()
        .map(|p| p.strip_prefix(&upstream).unwrap().to_string_lossy().into_owned())
        .collect::<BTreeSet<_>>();
    ensure!(actual_names == expected.keys().cloned().collect::<BTreeSet<_>>(), "extra or missing hardware build input");
    let mut actual = Vec::new();
    for (relative, hash) in expected {
        ensure!(
            run.guard(&upstream.join(&relative))? == hash,
            "hardware source differs from pinned patched profile: {relative}"
        );
        actual.push(crate :: report ! ({ "path" : relative , "sha256" : hash , }));
    }
    let cargo = fs::read_to_string(run.root.join("Cargo.toml"))?;
    ensure!(
        cargo.contains(crate::documents::field(pin, "revision")?) && cargo.contains("features = [\"lsnes-profile\"]"),
        "workspace hardware profile selection differs"
    );
    let record = crate :: report ! ({ "revision" : pin ["revision"] , "archive_sha256" : pin ["sha256"] , "source_sha256" : identity (& actual) ? , "files" : actual , "patches" : patches , "features" : crate :: report ! (vec ! ["std" , "lsnes-profile" , "clocked-smp" , "clocked-dma" , "clocked-interrupts"]) , });
    run.manifest.identities.insert("hardware".into(), record.clone());
    run.save()?;
    fs::remove_dir_all(temporary)?;
    Ok(record)
}
pub(crate) fn verify(run: &mut Run) -> Result<Vec<String>> {
    run.capture_source()?;
    verify_existing(run)
}
pub(crate) fn verify_existing(run: &mut Run) -> Result<Vec<String>> {
    run.check_sources()?;
    let mut failures = Vec::new();
    let root = run.root.clone();
    let doctor = run.stage(
        "doctor",
        &["just".into(), "doctor".into()],
        &root,
        &BTreeMap::new(),
        Duration::from_secs(180),
        &BTreeMap::new(),
    )?;
    if doctor["status"] != crate::readiness_metadata::value(&"pass").expect("validated native metadata") {
        failures.push("Inspect logs/doctor.stderr.log and rerun just doctor".into());
    }
    let input = (|| -> Result<Value> {
        let manifest = document(&root.join("movies/manifest.toml"))?;
        let entry = manifest
            .get("movies")
            .and_then(Value::as_array)
            .context("movie manifest missing")?
            .iter()
            .find(|v| {
                v["file"]
                    == crate::readiness_metadata::value(&"super-metroid-any-3653.lsmv")
                        .expect("validated native metadata")
            })
            .context("canonical movie missing")?;
        movie_inputs(run, entry)
    })();
    match input {
        Ok(record) => run.record_check(
            "input-identity",
            crate :: report ! ({ "status" : "pass" , "complete" : true , "frames" : record ["frames"] , }),
        )?,
        Err(error) => {
            failures.push(format!("{error:#}"));
            run.record_check(
                "input-identity",
                crate :: report ! ({ "status" : "error" , "complete" : false , "reason" : format ! ("{error:#}") , }),
            )?;
        }
    }
    match hardware(run) {
        Ok(_) => {
            run.record_check("hardware-identity", crate :: report ! ({ "status" : "pass" , "complete" : true , }))?
        }
        Err(error) => {
            failures.push(format!("{error:#}"));
            run.record_check(
                "hardware-identity",
                crate :: report ! ({ "status" : "error" , "complete" : false , "reason" : format ! ("{error:#}") , }),
            )?;
        }
    }
    let compiler = run.stage(
        "compiler",
        &["rustc".into(), "--version".into(), "--verbose".into()],
        &root,
        &BTreeMap::new(),
        Duration::from_secs(30),
        &BTreeMap::new(),
    )?;
    if compiler["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata") {
        run . manifest . identities . insert ("compiler" . into () , crate :: report ! ({ "version" : fs :: read_to_string (run . path . join (crate::documents::field (& compiler , "stdout") ?)) ? . trim () , })) ;
    } else {
        failures.push("Install the pinned Rust toolchain; inspect compiler logs".into());
    }
    for name in ["Cargo.lock", "rust-toolchain.toml"] {
        if root.join(name).is_file() {
            let hash = run.guard(&root.join(name))?;
            run.manifest.identities.insert(name.into(), crate :: report ! ({ "sha256" : hash , }));
        }
    }
    run . manifest . build_settings = crate :: report ! ({ "cargo" : crate :: report ! (vec ! ["--locked" , "--offline"]) , "profile" : "release" , "seed" : 0 , "features" : crate :: report ! (vec ! ["lsnes-profile"]) , }) . as_table () . unwrap () . clone () ;
    run.save()?;
    run.check_sources()?;
    run . record_check ("preflight" , crate :: report ! ({ "status" : if failures . is_empty () { "pass" } else { "error" } , "complete" : failures . is_empty () , "failures" : failures , })) ? ;
    Ok(failures)
}
pub(crate) fn authenticate(owner: &Path, manifest: &Value) -> Result<()> {
    let source = owner.join("source/files");
    let movies = document(&source.join("movies/manifest.toml"))?;
    let entry = movies
        .get("movies")
        .and_then(Value::as_array)
        .context("movie manifest missing")?
        .iter()
        .find(|r| {
            r["file"]
                == crate::readiness_metadata::value(&"super-metroid-any-3653.lsmv").expect("validated native metadata")
        })
        .context("canonical movie pin missing")?;
    let inputs = &manifest["identities"]["inputs"];
    for (key, expected) in [
        ("rom_sha256", entry["rom_sha256"].clone()),
        ("movie_sha256", entry["sha256"].clone()),
        ("frames", entry["frames"].clone()),
        ("seed", Value::Integer(0)),
        ("profile", Value::String("lsnes-profile".into())),
        ("controller", Value::String("P1 SNES gamepad".into())),
        ("region", Value::String("NTSC".into())),
        ("reset", Value::String("clean default power-on; no reset/subframe records".into())),
    ] {
        ensure!(
            inputs[key] == crate::readiness_metadata::value(&expected).expect("validated native metadata"),
            "preflight pinned domain differs: {key}"
        );
    }
    ensure!(
        digest(&crate::readiness_run::safe_artifact(owner, crate::documents::field(inputs, "input_evidence")?)?)?
            == crate::documents::field(inputs, "input_sha256")?,
        "canonical inputs changed"
    );
    let dependencies = document(&source.join("tools/dependencies.toml"))?;
    let pin = dependencies
        .get("dependencies")
        .and_then(Value::as_array)
        .context("dependency pins missing")?
        .iter()
        .find(|r| r["name"] == crate::readiness_metadata::value(&"rustysnes").expect("validated native metadata"))
        .context("hardware pin missing")?;
    let hardware = &manifest["identities"]["hardware"];
    ensure!(
        hardware["revision"] == pin["revision"]
            && hardware["archive_sha256"] == pin["sha256"]
            && identity(&hardware["files"])? == crate::documents::field(hardware, "source_sha256")?,
        "hardware pin/source inventory differs"
    );
    let expected = [
        "rustysnes-hdma",
        "rustysnes-clocked-apu",
        "rustysnes-clocked-dma",
        "rustysnes-clocked-interrupts",
        "rustysnes-lsnes-reset",
        "rustysnes-readiness-observation",
        "rustysnes-readiness-checkpoints",
    ]
    .map(|name| format!("tools/patches/{name}.patch"));
    let patches = crate::documents::array(hardware, "patches")?;
    ensure!(patches.len() == expected.len(), "hardware patch recipe incomplete");
    for (patch, path) in patches.iter().zip(expected) {
        ensure!(
            patch["path"] == crate::readiness_metadata::value(&path).expect("validated native metadata")
                && digest(&crate::readiness_run::safe_artifact(&source, &path)?)?
                    == crate::documents::field(patch, "sha256")?,
            "hardware patch recipe differs"
        );
    }
    let root = owner.parent().and_then(|p| p.parent()).and_then(|p| p.parent()).context("checkout owner missing")?;
    for entry in crate::documents::array(hardware, "files")? {
        ensure!(
            digest(&crate::readiness_run::safe_artifact(
                &root.join("deps/rustysnes"),
                crate::documents::field(entry, "path")?
            )?)? == crate::documents::field(entry, "sha256")?,
            "current patched hardware differs"
        );
    }
    Ok(())
}
pub fn operate(request: Value) -> Result<Value> {
    let root = request.get("checkout").and_then(Value::as_str).map(PathBuf::from).unwrap_or(std::env::current_dir()?);
    let mut run = Run::create(
        &root,
        "preflight",
        crate :: report ! ({ "domain" : "Japan/USA any%" , "frames" : 18000 , }),
        None,
    )?;
    eprintln!("Run: {}\nBrief: {}", run.path.display(), run.path.join("brief.md").display());
    match verify(&mut run) {
        Ok(failures) => run.finalize(
            if failures.is_empty() { "pass" } else { "error" },
            Some("preflight"),
            Some(&if failures.is_empty() {
                "Setup verified; preflight scope only, readiness gates remain pending".into()
            } else {
                failures.join("; ")
            }),
            &["preflight".into()],
        ),
        Err(error) => run.finalize(
            if run.was_interrupted() { "incomplete" } else { "error" },
            Some("preflight"),
            Some(&format!("{error:#}")),
            &[],
        ),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::Write,
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };
    struct Temp(PathBuf);
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn root() -> Temp {
        let path = std::env::temp_dir().join(format!(
            "sm-preflight-controls-{}",
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        assert!(Command::new("git").args(["init", "-q"]).arg(&path).status().unwrap().success());
        for dir in ["rom", "movies", "crates"] {
            fs::create_dir(path.join(dir)).unwrap();
        }
        fs::write(path.join("Cargo.toml"), "[workspace]\n").unwrap();
        fs::write(path.join("rom/sm.sfc"), b"independent ROM").unwrap();
        Temp(path)
    }
    fn movie(root: &Path, extra: Option<(&str, &str)>) -> Value {
        let path = root.join("movies/test.lsmv");
        let rom_hash = digest(&root.join("rom/sm.sfc")).unwrap();
        let mut zip = zip::ZipWriter::new(File::create(&path).unwrap());
        for (name, value) in [
            ("gametype", "snes_ntsc"),
            ("controlsversion", "0"),
            ("coreversion", "bsnes v085 (Compatibility core)"),
            ("rom.sha256", rom_hash.as_str()),
            ("input", "F.|B...........\nF. 0 0|.Y..........\n"),
        ]
        .into_iter()
        .chain(extra)
        {
            zip.start_file(name, zip::write::SimpleFileOptions::default()).unwrap();
            zip.write_all(value.as_bytes()).unwrap();
        }
        zip.finish().unwrap();
        crate :: report ! ({ "file" : "test.lsmv" , "rom_sha256" : rom_hash , "sha256" : digest (& path) . unwrap () , "frames" : 2 , })
    }
    #[test]
    fn original_movie_inputs_are_independently_bound_and_stateful_or_modified_inputs_fail() {
        let root = root();
        let entry = movie(&root.0, None);
        let mut run = Run::create(&root.0, "preflight", crate::report!({}), None).unwrap();
        run.capture_source().unwrap();
        let record = movie_inputs(&mut run, &entry).unwrap();
        assert_eq!(record["frames"], toml::Value::try_from(2).unwrap());
        assert_eq!(fs::read_to_string(run.path.join("generated/movie.inputs")).unwrap(), "8000\n4000\n");
        fs::write(root.0.join("movies/test.inputs"), "0000\n4000\n").unwrap();
        let mut run = Run::create(&root.0, "preflight", crate::report!({}), None).unwrap();
        run.capture_source().unwrap();
        assert!(movie_inputs(&mut run, &entry).unwrap_err().to_string().contains("converted movie input"));
        fs::remove_file(root.0.join("movies/test.inputs")).unwrap();
        let entry = movie(&root.0, Some(("savestate", "hidden state")));
        let mut run = Run::create(&root.0, "preflight", crate::report!({}), None).unwrap();
        run.capture_source().unwrap();
        assert!(movie_inputs(&mut run, &entry).unwrap_err().to_string().contains("unsupported movie initialization"));
    }
    #[test]
    fn hardware_inventory_ignores_documentation_and_rejects_symlinked_build_inputs() {
        let root = root();
        fs::create_dir_all(root.0.join("crates/rustysnes-core/src")).unwrap();
        fs::write(root.0.join("crates/rustysnes-core/src/lib.rs"), "// hardware").unwrap();
        std::os::unix::fs::symlink("README.md", root.0.join("CLAUDE.md")).unwrap();
        let mut inventory = Vec::new();
        files(&root.0, &root.0, &mut inventory).unwrap();
        assert_eq!(inventory.len(), 2);
        std::os::unix::fs::symlink("lib.rs", root.0.join("crates/rustysnes-core/src/extra.rs")).unwrap();
        assert!(files(&root.0, &root.0, &mut Vec::new()).is_err());
    }
}
