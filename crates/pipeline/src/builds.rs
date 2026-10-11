//! Translation-unit reuse; generated candidate units are always rebuilt.
use crate::{acquire_lease, atomic_json, cached_artifact};
use anyhow::{Context, Result, bail, ensure};
use serde_json::{Value, json};
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Instant,
};

fn headers(directory: &Path, result: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let p = entry?.path();
        if p.is_dir() {
            headers(&p, result)?;
        } else if p.extension().and_then(|v| v.to_str()) == Some("h") {
            result.push(p);
        }
    }
    Ok(())
}

fn execute(command: &[String], run: &Path, label: &str, root: &Path, workers: usize) -> Result<Value> {
    let started = Instant::now();
    let (_held, wait) = acquire_lease(root, workers)?;
    let active = Instant::now();
    let mut child = Command::new(&command[0])
        .args(&command[1..])
        .current_dir(run)
        .stdout(Stdio::from(File::create(run.join(format!("{label}.stdout")))?))
        .stderr(Stdio::from(File::create(run.join(format!("{label}.stderr")))?))
        .spawn()?;
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if active.elapsed() > std::time::Duration::from_secs(240) {
            child.kill()?;
            child.wait()?;
            atomic_json(
                &run.join(format!("{label}.timing.json")),
                &json!({"argv":command,"seconds":started.elapsed().as_secs_f64(),"worker_wait_seconds":wait,"error":"compiler timeout"}),
            )?;
            bail!("{label} compiler timeout");
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    };
    let record = json!({"argv":command,"seconds":started.elapsed().as_secs_f64(),"worker_wait_seconds":wait,"exit_code":status.code()});
    atomic_json(&run.join(format!("{label}.timing.json")), &record)?;
    ensure!(status.success(), "{label} failed: {status}");
    Ok(record)
}

pub fn c_build(request: Value) -> Result<Value> {
    let run = PathBuf::from(request["run"].as_str().context("run missing")?);
    let checkout = PathBuf::from(request["checkout"].as_str().context("checkout missing")?);
    let root = PathBuf::from(request["root"].as_str().context("root missing")?);
    let workers = request["workers"].as_u64().context("worker count missing")? as usize;
    let command: Vec<String> = request["command"]
        .as_array()
        .context("command missing")?
        .iter()
        .map(|v| Ok(v.as_str().context("invalid argument")?.to_owned()))
        .collect::<Result<_>>()?;
    ensure!(!command.is_empty(), "empty C command");
    let mut flags = Vec::new();
    let mut sources = Vec::new();
    let mut inputs = Vec::new();
    let mut i = 1;
    while i < command.len() {
        let arg = &command[i];
        if arg.ends_with(".c") {
            sources.push(PathBuf::from(arg));
        } else if ["-I", "-iquote", "-isystem"].contains(&arg.as_str()) {
            let include = command.get(i + 1).context("include directory missing")?;
            headers(Path::new(include), &mut inputs)?;
            flags.extend([arg.clone(), include.clone()]);
            i += 1;
        } else if arg == "-o" {
            ensure!(i + 1 < command.len(), "output missing");
            i += 1;
        } else if arg.starts_with("-Wl,") || arg.starts_with("-l") || arg.ends_with(".a") || arg.ends_with(".o") {
        } else if arg.starts_with("-O")
            || arg.starts_with("-f")
            || arg.starts_with("-D")
            || arg.starts_with("-U")
            || arg.starts_with("-std=")
            || arg.starts_with("-m")
            || arg.starts_with("-W")
            || arg == "-pthread"
            || arg == "-g"
        {
            flags.push(arg.clone());
        } else {
            bail!("unsupported C build option: {arg}");
        }
        i += 1;
    }
    ensure!(!sources.is_empty(), "C build has no translation units");
    let pinned = fs::canonicalize(checkout.join("deps"))?;
    let mut records = Vec::new();
    let mut objects = Vec::new();
    for (index, source) in sources.iter().enumerate() {
        let source = fs::canonicalize(if source.is_absolute() { source.clone() } else { run.join(source) })?;
        let destination = run.join(format!("c-object-{index:03}.o"));
        let mut compile = vec![command[0].clone()];
        compile.extend(flags.clone());
        compile.extend(["-c".into(), source.to_string_lossy().into_owned(), "-o".into(), "{artifact}".into()]);
        let build = |artifact: &Path| {
            let mut actual = compile.clone();
            *actual.last_mut().unwrap() = artifact.to_string_lossy().into_owned();
            execute(&actual, &run, &format!("c-object-{index:03}"), &root, workers).map(|_| ())
        };
        if source.starts_with(&pinned) {
            // Ask the compiler for its complete preprocessor closure, including
            // system headers and textually included sources. Header edits and
            // changed include search paths must invalidate even a warm object.
            let (_held, _) = acquire_lease(&root, workers)?;
            let dependencies =
                Command::new(&command[0]).args(&flags).arg("-M").arg(&source).current_dir(&run).output()?;
            drop(_held);
            ensure!(
                dependencies.status.success(),
                "C dependency scan failed: {}",
                String::from_utf8_lossy(&dependencies.stderr)
            );
            let mut closure = inputs.clone();
            for name in crate::inputs::dep_paths(&String::from_utf8(dependencies.stdout)?)? {
                let p = PathBuf::from(name);
                closure.push(if p.is_absolute() { p } else { run.join(p) });
            }
            closure.push(source);
            records.push(cached_artifact(
                &json!({"root":root,"checkout":checkout,"command":compile,"inputs":closure,"destination":destination}),
                build,
            )?);
        } else {
            build(&destination)?;
        }
        objects.push(destination);
    }
    let source_names: Vec<_> = sources.iter().map(|p| p.to_string_lossy().into_owned()).collect();
    let mut linked: Vec<String> = command.into_iter().filter(|arg| !source_names.contains(arg)).collect();
    linked.splice(1..1, objects.iter().map(|p| p.to_string_lossy().into_owned()));
    let result = execute(&linked, &run, "c-link", &root, workers);
    if result.is_ok() {
        for p in objects {
            fs::remove_file(p)?;
        }
    }
    atomic_json(&run.join("c-cache.json"), &json!(records))?;
    result
}
