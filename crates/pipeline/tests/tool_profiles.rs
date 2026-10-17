//! Opt-in, retained-tool performance comparison through the actual family CLI.
//! The probe builds no controller or sandbox and keeps every source and receipt.
use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Instant,
};
use toml::Value;

fn write(path: &Path, bytes: impl AsRef<[u8]>) -> Result<()> {
    fs::create_dir_all(path.parent().context("missing parent")?)?;
    fs::write(path, bytes)?;
    Ok(())
}

fn digest(path: &Path) -> Result<String> {
    let mut input = fs::File::open(path)?;
    let mut hash = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        let count = input.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

fn table(entries: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    Value::Table(entries.into_iter().map(|(key, value)| (key.to_owned(), value)).collect())
}

fn path(path: &Path) -> Value {
    Value::try_from(path).unwrap()
}

fn command(tool: &Path, arguments: &[&str], input: Option<&Value>, log: &Path) -> Result<(Value, f64)> {
    fs::create_dir_all(log)?;
    if let Some(input) = input {
        write(&log.join("request.toml"), toml::to_string(input)?)?;
    }
    let started = Instant::now();
    let mut child = Command::new("timeout")
        .args(["--signal=TERM", "--kill-after=2s", "30s"])
        .arg(tool)
        .args(arguments)
        .env("SM_PIPELINE_METRICS", "enabled")
        .env_remove("SM_PIPELINE_METRICS_CONTEXT")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    if let Some(input) = input {
        child.stdin.take().unwrap().write_all(toml::to_string(input)?.as_bytes())?;
    }
    drop(child.stdin.take());
    let output = child.wait_with_output()?;
    let seconds = started.elapsed().as_secs_f64();
    write(&log.join("stdout.toml"), &output.stdout)?;
    write(&log.join("stderr.txt"), &output.stderr)?;
    write(
        &log.join("timing.toml"),
        format!("seconds = {seconds:.9}\nexit_code = {}\n", output.status.code().unwrap_or(-1)),
    )?;
    ensure!(output.status.success(), "{arguments:?}: {}", String::from_utf8_lossy(&output.stderr));
    let output =
        if output.stdout.is_empty() { table([]) } else { toml::from_str(std::str::from_utf8(&output.stdout)?)? };
    Ok((output, seconds))
}

fn checkout(source: &Path, root: &Path, importer: &Path) -> Result<()> {
    fs::create_dir(root)?;
    let files = Command::new("git")
        .args(["ls-files", "-z", "--", "crates", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml"])
        .current_dir(source)
        .output()?;
    ensure!(files.status.success(), "cannot enumerate exact source snapshot");
    for name in files.stdout.split(|byte| *byte == 0).filter(|name| !name.is_empty()) {
        let name = std::str::from_utf8(name)?;
        write(&root.join(name), fs::read(source.join(name))?)?;
    }
    write(
        &root.join("crates/port/src/fixture.rs"),
        "use crate::memory::Wram; pub fn dispatch(ram:&mut Wram<'_>,name:&str,a:u16,_b:u16)->Result<u32,&'static str>{if name!=\"Touch\"{return Err(\"routine\");}ram.set_byte(17,a as u8);Ok(a as u32+1)}",
    )?;
    write(
        &root.join("deps/snesrev-sm/src/sm_80.c"),
        "#include <stdint.h>\nextern uint8_t g_ram[0x20000];uint16_t Touch(uint16_t a){g_ram[17]=(uint8_t)a;return a+1;}\n",
    )?;
    write(&root.join("deps/snesrev-sm/src/sm_rtl.h"), "#include <stdbool.h>\n")?;
    write(&root.join("deps/snesrev-sm/src/funcs.h"), "")?;
    write(&root.join("deps/snesrev-sm/LICENSE.txt"), "bounded profile-comparison oracle\n")?;
    fs::create_dir_all(root.join("deps/downloads"))?;
    let archive = root.join("deps/downloads/oracle.tar.gz");
    let status =
        Command::new("tar").args(["-czf"]).arg(&archive).arg("-C").arg(root.join("deps")).arg("snesrev-sm").status()?;
    ensure!(status.success(), "cannot archive actual-C source");
    write(
        &root.join("tools/dependencies.toml"),
        format!(
            "schema_version = 1\n[[dependencies]]\nname = 'snesrev-sm'\narchive = 'oracle.tar.gz'\nsha256 = {:?}\nrevision = 'profile-comparison-source'\n",
            digest(&archive)?
        ),
    )?;
    let mut declaration = String::from(
        "family = 'fixture'\nbank = 'sm_80.c'\nrust_module = 'crates/port/src/fixture.rs'\n[[routines]]\nsymbol = 'Touch'\nargs = 1\nreturn = 'uint16'\n",
    );
    for index in 0..8 {
        declaration.push_str(&format!("[[routines.cases]]\nname = 'root-{index}'\na = {}\nfill = {}\nexpected_return = {}\nwrites = [{{offset = 131071, data = [77]}}]\n", index + 1, index + 32, index + 2));
    }
    for index in 0..4 {
        let predecessor = if index == 0 { "root-0".to_owned() } else { format!("child-{}", index - 1) };
        declaration.push_str(&format!("[[routines.cases]]\nname = 'child-{index}'\na = {}\ncontinue_from = '{predecessor}'\nexpected_return = {}\nwrites = [{{offset = {}, data = [99]}}]\n", index + 16, index + 17, index + 24));
    }
    let input = root.join("tools/port/fixture.toml");
    let output = root.join("tools/port/fixture.smf");
    write(&input, declaration)?;
    command(
        importer,
        &["fixture", "import", input.to_str().unwrap(), output.to_str().unwrap()],
        None,
        &root.join("import"),
    )?;
    Ok(())
}

struct Profile {
    name: &'static str,
    tool: PathBuf,
    sha256: String,
    cache: PathBuf,
}
struct Sample {
    cases: Value,
    run: PathBuf,
    seconds: f64,
}

fn sample(profile: &Profile, checkout: &Path, directory: &Path, cache_state: &str) -> Result<Sample> {
    fs::create_dir_all(directory)?;
    let host_load = fs::read_to_string("/proc/loadavg").unwrap_or_default();
    let request = table([
        ("checkout", path(checkout)),
        ("source", path(&checkout.join("deps/snesrev-sm"))),
        ("manifest", "tools/port/fixture.smf".into()),
        ("output", path(&directory.join("out"))),
        ("root", path(&profile.cache)),
        ("workers", 4.into()),
        ("case_jobs", 4.into()),
        ("execution", "fresh".into()),
    ]);
    let started = Instant::now();
    let (verdict, verification) =
        command(&profile.tool, &["family-verify"], Some(&request), &directory.join("verify"))?;
    ensure!(verdict["status"].as_str() == Some("offline-verified"), "family was not verified");
    ensure!(verdict["verified_cases"].as_integer() == Some(12), "complete twelve-case domain required");
    let run = PathBuf::from(verdict["run"].as_str().context("missing run")?);
    let producer: Value = toml::from_str(&fs::read_to_string(run.join("pipeline-inputs.toml"))?)?;
    ensure!(producer["binary_sha256"].as_str() == Some(&profile.sha256), "evidence omitted exact controller identity");
    let cases = verdict["cases"].clone();
    ensure!(cases.as_array().context("missing cases")?.len() == 12, "missing declared case");
    for case in cases.as_array().unwrap() {
        let state = run.join(case["case"].as_str().context("missing case name")?);
        let rust = fs::read(state.join("rust-after.wram"))?;
        let c = fs::read(state.join("c-after.wram"))?;
        ensure!(rust.len() == 0x20000 && rust == c, "full actual-C/Rust output differs");
    }
    let authentication_request = table([("checkout", path(checkout)), ("verdict", path(&run.join("verdict.toml")))]);
    let (_, authentication) = command(
        &profile.tool,
        &["family-authenticate"],
        Some(&authentication_request),
        &directory.join("authenticate"),
    )?;
    let (_, cleanup) = command(
        &profile.tool,
        &["cleanup", "--root", run.to_str().unwrap(), "--apply"],
        None,
        &directory.join("cleanup"),
    )?;
    let (_, deduplication) = command(
        &profile.tool,
        &["cleanup", "--root", run.to_str().unwrap(), "--deduplicate", "--apply"],
        None,
        &directory.join("deduplicate"),
    )?;
    let (_, final_authentication) = command(
        &profile.tool,
        &["family-authenticate"],
        Some(&authentication_request),
        &directory.join("authenticate-after-cleanup"),
    )?;
    let seconds = started.elapsed().as_secs_f64();
    let summary = table([
        ("profile", profile.name.into()),
        ("tool", path(&profile.tool)),
        ("tool_sha256", profile.sha256.clone().into()),
        ("cache_state", cache_state.into()),
        ("cache", path(&profile.cache)),
        ("workers", 4.into()),
        ("case_jobs", 4.into()),
        ("execution", "fresh".into()),
        ("verified_cases", 12.into()),
        ("host_load_before", host_load.into()),
        ("verification_seconds", verification.into()),
        ("authentication_seconds", authentication.into()),
        ("cleanup_seconds", cleanup.into()),
        ("deduplication_seconds", deduplication.into()),
        ("post_cleanup_authentication_seconds", final_authentication.into()),
        ("complete_seconds", seconds.into()),
        ("run", path(&run)),
    ]);
    write(&directory.join("measurement.toml"), toml::to_string_pretty(&summary)?)?;
    println!("{} {}: complete={seconds:.6}s verify={verification:.6}s cases=12", profile.name, directory.display());
    Ok(Sample { cases, run, seconds })
}

#[test]
#[ignore = "explicit retained-tool profile measurement; needs shared four-worker capacity"]
fn complete_family_profile_comparison() -> Result<()> {
    ensure!(std::env::var("SM_PIPELINE_WORKERS").as_deref() == Ok("4"), "four-worker capacity required");
    let shared = PathBuf::from(std::env::var_os("SM_PIPELINE_ROOT").context("shared capacity required")?);
    let source =
        fs::canonicalize(std::env::var_os("SM_TOOL_PROFILE_SOURCE").context("exact source checkout required")?)?;
    let root = PathBuf::from(std::env::var_os("SM_TOOL_PROFILE_PROOF").context("fresh proof root required")?);
    fs::create_dir(&root)?;
    let mut profiles = Vec::new();
    for (name, variable) in [("baseline", "SM_TOOL_PROFILE_BASELINE"), ("candidate", "SM_TOOL_PROFILE_CANDIDATE")] {
        let Some(tool) = std::env::var_os(variable) else { continue };
        let tool = fs::canonicalize(tool)?;
        let cache = root.join("caches").join(name);
        fs::create_dir_all(&cache)?;
        std::os::unix::fs::symlink(shared.join("workers"), cache.join("workers"))?;
        profiles.push(Profile { name, sha256: digest(&tool)?, tool, cache });
    }
    ensure!(!profiles.is_empty() && profiles[0].name == "baseline", "baseline tool required");
    let checkout_path = root.join("source");
    checkout(&source, &checkout_path, &profiles[0].tool)?;
    let pilot = std::env::var_os("SM_TOOL_PROFILE_PILOT").is_some();
    let mut canonical = None;
    let mut last = Vec::new();
    for profile in &profiles {
        let value = sample(
            profile,
            &checkout_path,
            &root.join(format!("{}-cold", profile.name)),
            "new private namespace; build artifacts absent",
        )?;
        if let Some(canonical) = &canonical {
            ensure!(&value.cases == canonical, "profile changed actual case outcomes");
        }
        canonical = Some(value.cases);
    }
    if !pilot {
        ensure!(profiles.len() == 2, "two profiles required for comparison");
        let mut measured = String::new();
        for round in 0..3 {
            let order = if round % 2 == 0 { [0, 1] } else { [1, 0] };
            for index in order {
                let profile = &profiles[index];
                let value = sample(
                    profile,
                    &checkout_path,
                    &root.join(format!("{}-warm-{round}", profile.name)),
                    "same private namespace warmed by complete cold sample; fresh Fixture execution",
                )?;
                ensure!(Some(&value.cases) == canonical.as_ref(), "profile changed actual case outcomes");
                measured.push_str(&format!(
                    "[[sample]]\nround = {round}\nprofile = {:?}\ncomplete_seconds = {:.9}\n",
                    profile.name, value.seconds
                ));
                write(&root.join("alternating.toml"), &measured)?;
                last.push((index, value.run));
            }
        }
        for (index, run) in last {
            let request = table([("checkout", path(&checkout_path)), ("verdict", path(&run.join("verdict.toml")))]);
            command(
                &profiles[1 - index].tool,
                &["family-authenticate"],
                Some(&request),
                &run.parent().unwrap().parent().unwrap().join("alternate-reader"),
            )?;
        }
    }
    Ok(())
}
