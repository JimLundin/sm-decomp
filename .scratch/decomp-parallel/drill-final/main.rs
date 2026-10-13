//! Disposable actual-session drill. No environment session impersonation.
use anyhow::{Context, Result, bail, ensure};
use serde_json::{Value as J, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{Read, Write},
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const ROOT: &str = "/tmp/sm-decomp-concurrency-drill-final-20261013";
const EFFORT: &str = "concurrency-drill-final";
const STORE: &str = "/home/jim/sm-decomp/.scratch/decomp-parallel/drill-final";
fn now() -> String {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().to_string()
}
fn write(path: &Path, value: &J) -> Result<()> {
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(path, toml::to_string_pretty(&toml::Value::try_from(value)?)?)?;
    Ok(())
}
fn read(path: &Path) -> Result<J> {
    Ok(serde_json::to_value(toml::from_str::<toml::Value>(&fs::read_to_string(path)?)?)?)
}
fn hash(path: &Path) -> Result<String> {
    Ok(format!("{:x}", Sha256::digest(fs::read(path)?)))
}
fn fingerprint(root: &Path) -> Result<String> {
    fn collect(root: &Path, path: &Path, files: &mut BTreeMap<String, String>) -> Result<()> {
        ensure!(!path.is_symlink(), "source symlink: {}", path.display());
        if path.is_dir() {
            for e in fs::read_dir(path)? {
                collect(root, &e?.path(), files)?;
            }
        } else if path.is_file() {
            files.insert(path.strip_prefix(root)?.display().to_string(), hash(path)?);
        }
        Ok(())
    }
    let mut files = BTreeMap::new();
    for path in [
        "crates/pipeline/src",
        "crates/pipeline-model/src",
        "crates/port/src",
        "crates/port/tests",
        "tools",
        ".cargo",
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        "crates/port/Cargo.toml",
    ] {
        let path = root.join(path);
        if path.exists() {
            collect(root, &path, &mut files)?;
        }
    }
    Ok(format!("{:x}", Sha256::digest(toml::to_string(&files)?.as_bytes())))
}
fn seal(path: &Path) -> Result<J> {
    Ok(json!({"path": path, "sha256":hash(path)?}))
}
fn state() -> PathBuf {
    Path::new(ROOT).join(".scratch/drill-control")
}
fn git(dir: &Path, args: &[&str]) -> Result<String> {
    let r = Command::new("git").args(args).current_dir(dir).output()?;
    ensure!(r.status.success(), "git {:?}: {}", args, String::from_utf8_lossy(&r.stderr));
    Ok(String::from_utf8(r.stdout)?.trim().into())
}
fn identity() -> Result<String> {
    let id = std::env::var("CODEX_THREAD_ID").context("run this role command from its actual Codex session")?;
    ensure!(id.len() == 36, "real thread id required");
    Ok(id)
}
fn slot(name: &str) -> Result<J> {
    read(&state().join(format!("{name}.toml")))
}
fn put_slot(name: &str, v: &J) -> Result<()> {
    write(&state().join(format!("{name}.toml")), v)
}
fn queue(token: &str, generation: u64) -> Result<J> {
    Ok(
        json!({"operation":"status","checkout":ROOT,"effort":EFFORT,"coordinator":token,"capacity":3,"available_slots":1,"authority":{"session_id":identity()?,"generation":generation}}),
    )
}
fn engine() -> Result<PathBuf> {
    Ok(PathBuf::from(fs::read_to_string(state().join("engine-path"))?.trim()))
}
fn invoke(label: &str, route: &str, request: &J, reject: bool) -> Result<J> {
    let n = format!("{}-{}-{}", label, std::process::id(), SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos());
    let dir = state().join("receipts");
    fs::create_dir_all(&dir)?;
    let req = dir.join(format!("{n}.request.toml"));
    write(&req, request)?;
    let mut c = Command::new(engine()?);
    c.arg(route)
        .current_dir(ROOT)
        .env("SM_PIPELINE_ROOT", Path::new(ROOT).join("out/pipeline"))
        .env("SM_PIPELINE_WORKERS", "4")
        .stdin(File::open(&req)?);
    let child = c.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    let process = observe_process(child.id())?;
    let o = child.wait_with_output()?;
    let out = dir.join(format!("{n}.stdout"));
    let err = dir.join(format!("{n}.stderr"));
    fs::write(&out, &o.stdout)?;
    fs::write(&err, &o.stderr)?;
    write(
        &dir.join(format!("{n}.execution.toml")),
        &json!({"schema":"decomp-drill-command-v1","session_id":std::env::var("CODEX_THREAD_ID").unwrap_or_default(),"pid":process["pid"],"process":process,"route":route,"request":seal(&req)?,"stdout":seal(&out)?,"stderr":seal(&err)?,"exit_code":o.status.code().unwrap_or(-1),"observed_at":now()}),
    )?;
    if reject {
        ensure!(!o.status.success(), "expected rejection {label} succeeded");
        return Ok(json!({"rejected":true,"stderr":String::from_utf8_lossy(&o.stderr)}));
    }
    ensure!(o.status.success(), "{label}: {}", String::from_utf8_lossy(&o.stderr));
    if o.stdout.is_empty() {
        return Ok(json!({}));
    }
    Ok(serde_json::to_value(toml::from_str::<toml::Value>(&String::from_utf8(o.stdout)?)?)?)
}
fn scopes() -> Result<J> {
    let list = read(&state().join("scopes.toml"))?;
    Ok(list["scopes"].clone())
}
fn admission() -> Result<J> {
    Ok(json!({"domain":"infrastructure","scopes":scopes()?}))
}
fn route(label: &str, operation: &str, q: J, reject: bool) -> Result<J> {
    invoke(label, "decomp-session", &json!({"operation":operation,"queue":q,"admission":admission()?}), reject)
}
fn observe_process(pid: u32) -> Result<J> {
    let dir = PathBuf::from(format!("/proc/{pid}"));
    let stat = fs::read_to_string(dir.join("stat"));
    let record = if let Ok(stat) = stat {
        let fields: Vec<&str> = stat.rsplit_once(')').unwrap().1.split_whitespace().collect();
        let argv: Vec<String> = fs::read(dir.join("cmdline"))?
            .split(|x| *x == 0)
            .filter(|x| !x.is_empty())
            .map(|x| String::from_utf8_lossy(x).into())
            .collect();
        json!({"pid":pid,"boot_id":fs::read_to_string("/proc/sys/kernel/random/boot_id")?.trim(),"start_ticks":fields[19].parse::<u64>()?,"kernel_state":fields[0],"argv":argv,"cwd":fs::read_link(dir.join("cwd")).unwrap_or_default(),"observed_at":now(),"status":if fields[0]=="Z" {"completed"}else{"running"}})
    } else {
        json!({"pid":pid,"observed_at":now(),"status":"not-running"})
    };
    Ok(record)
}
fn surviving(parent_name: &str, worker_name: &str, claim: &str, token: &str, generation: u64) -> Result<J> {
    let parent = slot(parent_name)?;
    let worker = slot(worker_name)?;
    let assignment = slot(&format!("assignment-{claim}"))?;
    let parent_observed = observe_process(parent["pid"].as_u64().unwrap() as u32)?;
    let worker_observed = observe_process(worker["pid"].as_u64().unwrap() as u32)?;
    ensure!(parent_observed["status"] != "running", "parent process still live");
    ensure!(
        worker_observed["status"] == "running"
            && worker_observed["start_ticks"] == worker["process"]["start_ticks"]
            && worker_observed["boot_id"] == worker["process"]["boot_id"],
        "child no longer same live process"
    );
    let ledger = read(&Path::new(ROOT).join(format!(".scratch/{EFFORT}/ledger.toml")))?;
    let held = &ledger["jobs"][claim];
    ensure!(
        held["state"] == "running" && held["claim"] == assignment["claim"] && held["agent"] == worker["session_id"],
        "surviving child claim changed"
    );
    let parent_launch = state().join(format!("{parent_name}-survival-launch.toml"));
    let worker_launch = state().join(format!("{worker_name}-survival-launch.toml"));
    write(&parent_launch, &parent)?;
    write(&worker_launch, &worker)?;
    let parent_obs = state().join(format!("{parent_name}-survival-observation.toml"));
    let worker_obs = state().join(format!("{worker_name}-survival-observation.toml"));
    write(&parent_obs, &parent_observed)?;
    write(&worker_obs, &worker_observed)?;
    let queue_result = invoke(
        "survival-inspect",
        "decomp-session",
        &json!({"operation":"inspect","queue":queue(token,generation)?}),
        false,
    )?;
    ensure!(
        queue_result["jobs"].as_array().is_some_and(|jobs| jobs.iter().any(|job| job["claim"] == assignment["claim"]
            && job["agent"] == worker["session_id"]
            && job["state"] == "running")),
        "executed inspect lost live child ownership"
    );
    let mut commands: Vec<_> = fs::read_dir(state().join("receipts"))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name().unwrap().to_string_lossy().starts_with("survival-inspect")
                && p.extension().is_some_and(|e| e == "toml")
                && p.file_name().unwrap().to_string_lossy().contains("execution")
        })
        .collect();
    commands.sort();
    let latest = read(commands.last().context("survival inspect command receipt missing")?)?;
    let result = json!({"schema":"decomp-drill-surviving-child-v1","parent":seal(&parent_launch)?,"worker":seal(&worker_launch)?,"parent_observation":seal(&parent_obs)?,"worker_observation":seal(&worker_obs)?,"claim":assignment["claim"],"queue":latest["stdout"],"observed_at":now()});
    write(&state().join("surviving-child.toml"), &result)?;
    Ok(result)
}
fn launch(name: &str, prompt: &Path, resume: Option<&str>) -> Result<J> {
    let logs = Path::new(STORE).join("sessions");
    fs::create_dir_all(&logs)?;
    let log = logs.join(format!("{name}-{}.jsonl", now()));
    let err = log.with_extension("stderr");
    let mut c = Command::new("codex");
    c.args(["exec", "-s", "danger-full-access", "-C", ROOT]);
    if let Some(id) = resume {
        c.arg("resume").arg(id);
    }
    c.args(["--json", "-"])
        .current_dir(ROOT)
        .env_remove("CODEX_THREAD_ID")
        .env("SM_PIPELINE_ROOT", Path::new(ROOT).join("out/pipeline"))
        .env("SM_PIPELINE_WORKERS", "4")
        .stdin(File::open(prompt)?)
        .stdout(File::create(&log)?)
        .stderr(File::create(&err)?)
        .process_group(0);
    let child = c.spawn()?;
    let pid = child.id();
    let start = std::time::Instant::now();
    let id = 'outer: loop {
        if let Ok(text) = fs::read_to_string(&log) {
            for line in text.lines() {
                if let Ok(v) = serde_json::from_str::<J>(line) {
                    if v["type"] == "thread.started" {
                        if let Some(id) = v["thread_id"].as_str() {
                            break 'outer id.to_owned();
                        }
                    }
                }
            }
        }
        ensure!(start.elapsed() < Duration::from_secs(120), "thread.started timeout; see {}", log.display());
        std::thread::sleep(Duration::from_millis(100));
    };
    let record = json!({"schema":"decomp-drill-session-launch-v1","name":name,"session_id":id,"pid":pid,"process":observe_process(pid)?,"log":log,"stderr":err,"prompt":seal(prompt)?,"observed_at":now()});
    put_slot(&format!("{name}-launch-{}", now()), &record)?;
    put_slot(name, &record)?;
    Ok(record)
}
fn sync_wait(marker: &str, seconds: u64) -> Result<J> {
    let began = std::time::Instant::now();
    while !state().join(marker).exists() {
        ensure!(began.elapsed() < Duration::from_secs(seconds), "marker wait bound: {marker}");
        std::thread::sleep(Duration::from_millis(100));
    }
    Ok(json!({"marker":marker,"observed_at":now()}))
}
fn observation(name: &str) -> Result<J> {
    let s = slot(name)?;
    let unique = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let path = state().join(format!("{name}-{unique}-session-observation.toml"));
    let record = state().join(format!("{name}-{unique}-actual-launch.toml"));
    write(&record, &s)?;
    write(
        &path,
        &json!({"schema":"decomp-session-observation-v1","session_id":s["session_id"],"status":"running","observed_at":now(),"evidence":[seal(&record)?]}),
    )?;
    seal(&path)
}
fn prepare() -> Result<J> {
    let a = slot("a")?;
    ensure!(identity()? == a["session_id"], "wrong integrator caller");
    let bridge = state().join("bridge.toml");
    invoke(
        "prepare",
        "decomp-authority",
        &json!({"operation":"prepare","checkout":ROOT,"coordinator":"drill-integrator","authority":{"session_id":identity()?,"generation":1},"preparation":{"canonical_effort":"decomp-parallel","capacity":3,"admitted_efforts":[],"drill_efforts":[EFFORT],"bridge":seal(&bridge)?,"integrator":{"token":"drill-integrator","kind":"integrator","session_id":identity()?,"observation":observation("a")?}}}),
        false,
    )
}
fn register(name: &str) -> Result<J> {
    let s = slot(name)?;
    invoke(
        "register",
        "decomp-authority",
        &json!({"operation":"register","checkout":ROOT,"coordinator":"drill-integrator","authority":{"session_id":identity()?,"generation":1},"registration":{"token":format!("drill-{name}"),"kind":"dispatcher","session_id":s["session_id"],"observation":observation(name)?}}),
        false,
    )
}
fn next(name: &str, generation: u64) -> Result<J> {
    next_token(&format!("drill-{name}"), generation)
}
fn next_token(token: &str, generation: u64) -> Result<J> {
    let mut q = queue(token, generation)?;
    q["operation"] = json!("next");
    let r = route(&format!("next-{token}"), "infrastructure-next", q, false)?;
    if r.get("assignment").is_some() {
        let claim = r["assignment"]["claim"]["id"].as_str().context("claim missing")?;
        put_slot(&format!("assignment-{claim}"), &r["assignment"])?;
        put_slot(&format!("assignment-{claim}-attempt-{}", r["assignment"]["claim"]["attempt"]), &r["assignment"])?;
    }
    Ok(r)
}
fn init_roles() -> Result<J> {
    let a = prepare()?;
    let ra = register("a")?;
    let rb = register("b")?;
    fs::write(state().join("registered-ready"), now())?;
    Ok(json!({"prepare":a,"dispatchers":[ra,rb]}))
}
fn session_initial(name: &str) -> Result<J> {
    let raced = next(name, 1)?;
    let claim = raced["assignment"]["claim"]["id"].as_str().context("session got no raced assignment")?.to_owned();
    fs::write(state().join(format!("{name}-claim-id")), &claim)?;
    fs::write(state().join(format!("{name}-raced-ready")), now())?;
    let worker = format!("worker-{name}");
    let token = format!("drill-{name}");
    let dispatched = dispatch(&claim, &worker, &token, 1)?;
    sync_wait(&format!("{worker}-edited"), 90)?;
    let saved = checkpoint(&claim, &token, 1)?;
    fs::write(state().join(format!("{name}-checkpointed-ready")), now())?;
    let capacity = if name == "b" {
        let third = next(name, 1)?;
        let fourth = next(name, 1)?;
        ensure!(fourth.get("assignment").is_none(), "capacity3 exceeded");
        fs::write(state().join("capacity-ready"), now())?;
        json!({"third":third,"fourth":fourth})
    } else {
        json!({})
    };
    Ok(json!({"reservation":raced,"dispatch":dispatched,"checkpoint":saved,"capacity":capacity}))
}
fn base_handoff(a: &J, worker: &J, token: &str, generation: u64) -> Result<J> {
    let w = &a["workspace"];
    let integration = a["job"]["kind"] == "integrate";
    let worktree = if integration { json!(ROOT) } else { w["worktree"].clone() };
    let process = &worker["process"];
    let observed = json!({"pid":process["pid"],"boot_id":process["boot_id"],"start_ticks":process["start_ticks"],"argv":process["argv"],"cwd":process["cwd"],"observed_at":process["observed_at"],"status":process["status"]});
    let mut h = json!({"schema":"decomp-worker-handoff-v1","sequence":1,"session_id":identity()?,"worker_session_id":worker["session_id"],"agent":worker["name"],"authority_generation":generation,"effort":EFFORT,"claim":a["claim"],"job":a["job"],"baseline":a["job"]["inputs"]["base-head"],"worktree":worktree,"source_commit":git(Path::new(worktree.as_str().unwrap()),&["rev-parse","HEAD"])? ,"runs":[],"processes":[observed],"evidence":[],"dependencies":[],"stages":[],"limitations":[format!("isolated infrastructure drill owned by {token}; no gameplay")],"cleanup":{"status":"pending","reclaimed_bytes":0,"receipts":[],"retained_dependencies":[],"remaining_action":"scoped cleanup after writers stop"},"next":{"cwd":worktree,"argv":["cargo","test","--locked","--offline"],"env":{"SM_PIPELINE_ROOT":format!("{ROOT}/out/pipeline"),"SM_PIPELINE_WORKERS":"4"},"timeout_seconds":60}});
    if !integration {
        h["branch"] = w["branch"].clone();
    }
    Ok(h)
}
fn start(claim: &str, worker: &str, token: &str, generation: u64) -> Result<J> {
    let a = slot(&format!("assignment-{claim}"))?;
    let w = slot(worker)?;
    let wt = PathBuf::from(a["workspace"]["worktree"].as_str().unwrap_or(ROOT));
    if a["job"]["kind"] == "implement" && !wt.join(".git").exists() {
        git(
            Path::new(ROOT),
            &[
                "worktree",
                "add",
                "-b",
                a["workspace"]["branch"].as_str().unwrap(),
                wt.to_str().unwrap(),
                a["job"]["inputs"]["base-head"].as_str().unwrap(),
            ],
        )?;
    }
    let h = base_handoff(&a, &w, token, generation)?;
    let r = invoke(
        "start",
        "decomp-session",
        &json!({"operation":"start","queue":queue(token,generation)?,"handoff":h}),
        false,
    )?;
    put_slot(&format!("handoff-{claim}"), &r)?;
    Ok(r)
}
fn dispatch(claim: &str, worker: &str, token: &str, generation: u64) -> Result<J> {
    let assignment = slot(&format!("assignment-{claim}"))?;
    let wt = PathBuf::from(assignment["workspace"]["worktree"].as_str().context("implementation worktree missing")?);
    if !wt.join(".git").exists() {
        git(
            Path::new(ROOT),
            &[
                "worktree",
                "add",
                "-b",
                assignment["workspace"]["branch"].as_str().unwrap(),
                wt.to_str().unwrap(),
                assignment["job"]["inputs"]["base-head"].as_str().unwrap(),
            ],
        )?;
    }
    let attempt = assignment["claim"]["attempt"].as_u64().unwrap();
    if attempt > 1 {
        let old = slot(&format!("checkpoint-{claim}-attempt-{}", attempt - 1))?;
        let commit = old["source_commit"].as_str().context("retained previous scoped WIP missing")?;
        if git(&wt, &["merge-base", "--is-ancestor", commit, "HEAD"]).is_err() {
            git(&wt, &["cherry-pick", commit])?;
        }
    }
    let instructions = state().join(format!("{worker}-instructions.toml"));
    write(
        &instructions,
        &json!({"worker":worker,"dispatcher_session_id":identity()?,"claim":assignment["claim"],"worktree":wt,"writable_paths":assignment["job"]["writable_paths"],"baseline":assignment["job"]["inputs"]["base-head"]}),
    )?;
    let prompt = state().join(format!("{worker}-prompt.md"));
    fs::write(
        &prompt,
        format!(
            "You are an actual delegated Rust infrastructure drill worker named {worker}. Read {}. Use only its exact assigned worktree and one owned document. No production/ROM/gameplay work. The caller explicitly authorized this isolated crash drill. Do not set CODEX_THREAD_ID. Always use /tmp/sm-decomp-drill-final-harness-build/debug/decomp-drill; shorthand below means that absolute executable. First run decomp-drill identity {worker}. Wait for marker {worker}-start-ready using decomp-drill wait MARKER 50, retry if necessary. Spawn an actual collaboration subagent inheriting your model, task limited to appending a sentence 'lane scoped WIP from {worker}' to the exact owned document and reviewing no other paths. Record returned real subagent identity and final status in isolated .scratch/drill-control/{worker}-subagent.txt. Wait for its result, verify only scoped document changed, then decomp-drill mark {worker}-edited. Stay actively alive independently while parent is interrupted: repeatedly run decomp-drill wait {worker}-release 50; if timeout, retry until release, at most 30 attempts. On release report your actual identity and scope. Do not commit or run expensive tests; dispatcher uses maintained checkpoint/gates. Keep user messages brief.",
            instructions.display()
        ),
    )?;
    let launched = launch(worker, &prompt, None)?;
    let handoff = start(claim, worker, token, generation)?;
    fs::write(state().join(format!("{worker}-start-ready")), now())?;
    Ok(json!({"worker_launch":launched,"handoff":handoff}))
}
fn checkpoint(claim: &str, token: &str, generation: u64) -> Result<J> {
    let p = slot(&format!("handoff-{claim}"))?;
    let mut h = read(Path::new(p["handoff"].as_str().unwrap()))?;
    h["sequence"] = json!(h["sequence"].as_u64().unwrap() + 1);
    let r = invoke(
        "checkpoint",
        "decomp-session",
        &json!({"operation":"checkpoint","queue":queue(token,generation)?,"handoff":h,"message":format!("drill: retain scoped {claim} WIP")}),
        false,
    )?;
    put_slot(&format!("handoff-{claim}"), &r)?;
    put_slot(&format!("checkpoint-{claim}-attempt-{}", r["claim"]["attempt"]), &r)?;
    let handoff = read(Path::new(r["handoff"].as_str().unwrap()))?;
    let retained = Path::new(STORE).join("source-snapshots").join(format!("{claim}-attempt-{}", r["claim"]["attempt"]));
    for path in handoff["job"]["writable_paths"].as_array().unwrap() {
        let path = Path::new(path.as_str().unwrap());
        let target = retained.join(path);
        fs::create_dir_all(target.parent().unwrap())?;
        fs::copy(Path::new(handoff["worktree"].as_str().unwrap()).join(path), target)?;
    }
    write(
        &retained.join("checkpoint.toml"),
        &json!({"schema":"decomp-drill-retained-checkpoint-v1","claim":handoff["claim"],"source_commit":handoff["source_commit"],"baseline":handoff["baseline"],"handoff":seal(Path::new(r["handoff"].as_str().unwrap()))?}),
    )?;
    Ok(r)
}
fn handoff_op(op: &str, claim: &str, token: &str, generation: u64, transfer: Option<&Path>, reject: bool) -> Result<J> {
    let p = slot(&format!("handoff-{claim}"))?;
    let mut req = json!({"operation":op,"queue":queue(token,generation)?,"handoff":p["handoff"]});
    if let Some(t) = transfer {
        req["transfer"] = seal(t)?;
    }
    let r = invoke(op, "decomp-session", &req, reject)?;
    if !reject && op == "adopt" {
        put_slot(&format!("handoff-{claim}"), &r)?;
    }
    Ok(r)
}
fn finish(claim: &str, token: &str, generation: u64) -> Result<J> {
    let a = slot(&format!("assignment-{claim}"))?;
    let wt = Path::new(a["workspace"]["worktree"].as_str().unwrap());
    let commit = git(wt, &["rev-parse", "HEAD"])?;
    let evidence = state().join(format!("{claim}-submitted-source.toml"));
    if !evidence.exists() {
        write(
            &evidence,
            &json!({"schema":"decomp-drill-source-v1","commit":commit,"claim":a["claim"],"source_path":a["job"]["writable_paths"][0],"source_sha256":hash(&wt.join(a["job"]["writable_paths"][0].as_str().unwrap()))?}),
        )?;
    }
    let mut q = queue(token, generation)?;
    q["operation"] = json!("record");
    q["update"] = json!({"event":"finished","claim":a["claim"],"result":{"result":"submitted","commit":commit,"evidence":[seal(&evidence)?]}});
    let request = state().join(format!("{claim}-exact-finish.toml"));
    write(&request, &q)?;
    route("finish", "infrastructure-record", q, false)
}
fn transfer(token: &str, old_slot: &str, generation: u64, replacement_slot: &str) -> Result<J> {
    let old = slot(old_slot)?;
    let replacement = slot(replacement_slot)?;
    ensure!(identity()? == replacement["session_id"], "replacement must call from its own actual session");
    let observed = observe_process(old["pid"].as_u64().unwrap() as u32)?;
    ensure!(observed["status"] != "running", "original session process is still running");
    let evidence_path = state().join(format!("{token}-{generation}-{old_slot}-stopped-process.toml"));
    write(
        &evidence_path,
        &json!({"schema":"decomp-drill-stopped-session-v1","original_launch":old,"observed_process":observed,"observed_at":now()}),
    )?;
    let ledger = read(&Path::new(ROOT).join(format!(".scratch/{EFFORT}/ledger.toml")))?;
    let mut children = Vec::new();
    for (_, entry) in ledger["jobs"].as_object().context("ledger jobs missing")? {
        if !["reserved", "running", "submitted"].contains(&entry["state"].as_str().unwrap_or_default())
            || entry["claim"]["coordinator"] != token
        {
            continue;
        }
        let agent = entry.get("agent").and_then(J::as_str);
        let mut child_status = "not-launched".to_string();
        let mut inspection = json!({"schema":"decomp-drill-unlaunched-v1","claim":entry["claim"],"state":entry["state"],"observed_at":now()});
        let mut owner = old["session_id"].clone();
        if let Some(agent) = agent {
            let candidates = fs::read_dir(state())?
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|e| e == "toml"));
            for path in candidates {
                if let Ok(s) = read(&path) {
                    if s["schema"] == "decomp-drill-session-launch-v1" && s["session_id"] == agent {
                        let p = observe_process(s["pid"].as_u64().unwrap() as u32)?;
                        child_status = if p["status"] == "running" { "running" } else { "completed" }.into();
                        owner = json!(agent);
                        inspection = json!({"schema":"decomp-drill-child-inspection-v1","launch":s,"process":p,"observed_at":now()});
                        break;
                    }
                }
            }
            ensure!(child_status != "not-launched", "cannot inspect real child {agent}");
        }
        let id = entry["claim"]["id"].as_str().unwrap();
        let inspection_path = state().join(format!("{token}-{id}-{generation}-child-inspection.toml"));
        write(&inspection_path, &inspection)?;
        let proof_path = state().join(format!("{token}-{id}-{generation}-child-proof.toml"));
        let mut proof = json!({"schema":"decomp-child-observation-v1","effort":EFFORT,"claim":entry["claim"],"session_id":owner,"status":child_status,"observed_at":now(),"evidence":[seal(&inspection_path)?]});
        if let Some(agent) = agent {
            proof["agent"] = json!(agent);
        }
        write(&proof_path, &proof)?;
        let mut child =
            json!({"effort":EFFORT,"claim":entry["claim"],"status":child_status,"observation":seal(&proof_path)?});
        if let Some(agent) = agent {
            child["agent"] = json!(agent);
        }
        children.push(child);
    }
    let proof = state().join(format!("{token}-generation-{generation}-transfer.toml"));
    write(
        &proof,
        &json!({"schema":"decomp-session-transfer-v1","token":token,"session_id":old["session_id"],"generation":generation,"status":"interrupted","observed_at":now(),"evidence":[seal(&evidence_path)?],"replacement":observation(replacement_slot)?,"children":children,"processes":[]}),
    )?;
    let req = json!({"operation":"transfer","checkout":ROOT,"coordinator":token,"authority":{"session_id":identity()?,"generation":generation+1},"transfer":{"token":token,"session_id":identity()?,"observation":seal(&proof)?}});
    let r = invoke("transfer", "decomp-authority", &req, false)?;
    put_slot(&format!("transfer-{token}"), &json!({"proof":proof,"request":req,"receipt":r}))?;
    Ok(r)
}
fn stop_owner(claim: &str, worker: &str, token: &str, generation: u64) -> Result<J> {
    let a = slot(&format!("assignment-{claim}"))?;
    let w = slot(worker)?;
    let observed = observe_process(w["pid"].as_u64().unwrap() as u32)?;
    ensure!(observed["status"] != "running", "worker still running");
    let inspection = state().join(format!("{claim}-owner-stop-inspection.toml"));
    write(
        &inspection,
        &json!({"schema":"decomp-drill-stopped-worker-v1","launch":w,"observed_process":observed,"claim":a["claim"],"observed_at":now()}),
    )?;
    let proof = state().join(format!("{claim}-owner-stop.toml"));
    write(
        &proof,
        &json!({"agent":w["session_id"],"session_id":w["session_id"],"coordinator":token,"status":"interrupted","claim":a["claim"],"observed_at":now(),"evidence":[seal(&inspection)?],"processes":[]}),
    )?;
    let mut q = queue(token, generation)?;
    q["operation"] = json!("record");
    q["update"] = json!({"event":"owner-stopped","claim":a["claim"],"observation":seal(&proof)?});
    route("owner-stopped", "infrastructure-record", q, false)
}
fn lease_probe(name: &str) -> Result<J> {
    let receipts = state().join("leases");
    fs::create_dir_all(&receipts)?;
    let mut c = Command::new(engine()?);
    c.args(["lease", &format!("{ROOT}/out/pipeline"), "4"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = c.spawn()?;
    let started = now();
    let process = observe_process(child.id())?;
    write(
        &receipts.join(format!("{name}-started.toml")),
        &json!({"schema":"decomp-drill-lease-start-v1","session_id":identity()?,"process":process,"observed_at":started}),
    )?;
    let mut output = child.stdout.take().unwrap();
    let mut len = [0u8; 4];
    output.read_exact(&mut len)?;
    let mut bytes = vec![0u8; u32::from_le_bytes(len) as usize];
    output.read_exact(&mut bytes)?;
    fs::write(receipts.join(format!("{name}-acquired.toml")), &bytes)?;
    let began = std::time::Instant::now();
    while !state().join("release-leases").exists() {
        ensure!(began.elapsed() < Duration::from_secs(90), "lease probe bound reached");
        std::thread::sleep(Duration::from_millis(100));
    }
    child.stdin.take().unwrap().write_all(b"release\n")?;
    let exit = child.wait()?;
    ensure!(exit.success(), "lease process failed");
    let record = json!({"schema":"decomp-drill-lease-finished-v1","session_id":identity()?,"name":name,"process":process,"lease_receipt":seal(&receipts.join(format!("{name}-acquired.toml")))?,"exit_code":exit.code().unwrap_or(-1),"observed_at":now()});
    write(&receipts.join(format!("{name}-finished.toml")), &record)?;
    Ok(record)
}
fn integrate(claim: &str, generation: u64) -> Result<J> {
    let a = slot(&format!("assignment-{claim}"))?;
    let submission = a["job"]["inputs"]["submission-id"].as_str().unwrap();
    let s = slot(&format!("assignment-{submission}"))?;
    let wt = PathBuf::from(s["workspace"]["worktree"].as_str().unwrap());
    let commit = git(&wt, &["rev-parse", "HEAD"])?;
    if git(Path::new(ROOT), &["merge-base", "--is-ancestor", &commit, "HEAD"]).is_err() {
        git(Path::new(ROOT), &["merge", "--no-edit", &commit])?;
    }
    let snapshots = state().join("source-snapshots").join(submission);
    fs::create_dir_all(&snapshots)?;
    let mut retained = Vec::new();
    for p in s["job"]["writable_paths"].as_array().unwrap() {
        let p = Path::new(p.as_str().unwrap());
        let dest = snapshots.join(p);
        fs::create_dir_all(dest.parent().unwrap())?;
        fs::copy(wt.join(p), &dest)?;
        retained.push(seal(&dest)?);
    }
    let receipts_root = Path::new(ROOT).join(format!(".scratch/{EFFORT}/receipts"));
    fs::create_dir_all(&receipts_root)?;
    let mut cleans = Vec::new();
    for dedup in [false, true] {
        let q = json!({"root":receipts_root,"apply":true,"deduplicate":dedup});
        let r = invoke(if dedup { "cleanup-deduplicate" } else { "cleanup" }, "cleanup", &q, false)?;
        let path = state().join(format!("{claim}-cleanup-{dedup}.toml"));
        write(&path, &r)?;
        cleans.push(seal(&path)?);
    }
    git(Path::new(ROOT), &["worktree", "remove", wt.to_str().unwrap()])?;
    let cleanup = state().join(format!("{claim}-cleanup.toml"));
    let reclaimed = cleans
        .iter()
        .filter_map(|v| read(Path::new(v["path"].as_str().unwrap())).ok())
        .filter_map(|r| r["bytes"].as_u64())
        .sum::<u64>();
    write(
        &cleanup,
        &json!({"status":"complete","reclaimed_bytes":reclaimed,"receipts":cleans,"retained_dependencies":retained}),
    )?;
    let placeholder = state().join(format!("{claim}-authentication-input.toml"));
    write(
        &placeholder,
        &json!({"schema":"decomp-drill-authentication-request-v1","claim":a["claim"],"source_commit":git(Path::new(ROOT),&["rev-parse","HEAD"])?}),
    )?;
    let mut q = queue("drill-integrator", generation)?;
    q["operation"] = json!("record");
    q["update"] = json!({"event":"finished","claim":a["claim"],"result":{"result":"integrated","submissions":[s["claim"].clone()],"evidence":retained,"authentication":seal(&placeholder)?,"cleanup":seal(&cleanup)?}});
    write(&state().join(format!("{claim}-exact-integrate.toml")), &q)?;
    let r = route("integrated", "infrastructure-record", q, false)?;
    put_slot(&format!("integrated-{submission}"), &r)?;
    Ok(r)
}
fn integration_retry(claim: &str, generation: u64) -> Result<J> {
    let mut q = read(&state().join(format!("{claim}-exact-integrate.toml")))?;
    q["authority"] = json!({"session_id":identity()?,"generation":generation});
    route("integrated-exact-retry", "infrastructure-record", q, false)
}
fn retain_integrator(token: &str, generation: u64) -> Result<J> {
    let path = state().join("integrator-checkpoint.toml");
    let info = json!({"schema":"decomp-drill-integrator-checkpoint-v1","session_id":identity()?,"token":token,"generation":generation,"source_commit":git(Path::new(ROOT),&["rev-parse","HEAD"])? ,"source_fingerprint":fingerprint(Path::new(ROOT))?,"queue":seal(&Path::new(ROOT).join(format!(".scratch/{EFFORT}/ledger.toml")))?,"next":{"cwd":ROOT,"argv":["cargo","test","--locked","--offline"],"env":{"SM_PIPELINE_ROOT":format!("{ROOT}/out/pipeline"),"SM_PIPELINE_WORKERS":"4"},"timeout_seconds":60}});
    write(&path, &info)?;
    Ok(info)
}
fn retired_implementer(claim: &str) -> Result<J> {
    let mut removed = 0u64;
    let glob = state();
    for item in fs::read_dir(&glob)? {
        let p = item?.path();
        let name = p.file_name().unwrap().to_string_lossy();
        if !name.starts_with(&format!("assignment-{claim}-attempt-")) {
            continue;
        }
        let a = read(&p)?;
        let Some(wt) = a["workspace"]["worktree"].as_str() else {
            continue;
        };
        let path = Path::new(wt);
        if path.join(".git").exists() {
            ensure!(git(path, &["status", "--porcelain"])?.is_empty(), "retained stopped worktree dirty");
            git(Path::new(ROOT), &["worktree", "remove", wt])?;
            removed += 1;
        }
    }
    Ok(json!({"retired_worktrees":removed,"claim":claim,"retained_source_root":format!("{STORE}/source-snapshots")}))
}
fn controls() -> Result<J> {
    let mut list = read(&state().join("scopes.toml"))?;
    let ledger = read(&Path::new(ROOT).join(format!(".scratch/{EFFORT}/ledger.toml")))?;
    let active = ledger["jobs"]
        .as_object()
        .context("jobs missing")?
        .values()
        .filter(|e| ["reserved", "running", "submitted"].contains(&e["state"].as_str().unwrap_or_default()))
        .collect::<Vec<_>>();
    ensure!(active.len() >= 2, "negative controls require observed live held lanes");
    for (id, target, ticket) in [("overlap", "a", "overlap.md"), ("duplicate-ticket", "b", "b.md")] {
        let mut scope = read(&state().join(format!("scope-{target}.toml")))?;
        scope["job"]["id"] = json!(format!("control-{id}"));
        scope["job"]["subject"] = json!(format!("Negative control {id}"));
        scope["job"]["ticket"] = json!(format!(".scratch/{EFFORT}/issues/{ticket}"));
        if id == "duplicate-ticket" {
            scope["job"]["writable_paths"] = json!(["docs/drill-duplicate.md"]);
            scope["job"]["ticket"] = active[1]["job"]["ticket"].clone();
        }
        if id == "overlap" {
            scope["job"]["writable_paths"] = active[0]["job"]["writable_paths"].clone();
        }
        let path = state().join(format!("scope-{id}.toml"));
        write(&path, &scope)?;
        list["scopes"].as_array_mut().unwrap().push(seal(&path)?);
    }
    write(&state().join("scopes.toml"), &list)?;
    Ok(list)
}
fn budget_snapshot() -> Result<J> {
    let dir = state().join("leases");
    let mut held = Vec::new();
    let mut waiter = None;
    for entry in fs::read_dir(&dir)? {
        let path = entry?.path();
        let name = path.file_name().unwrap().to_string_lossy();
        if !name.ends_with("-started.toml") {
            continue;
        }
        let start = read(&path)?;
        let observed = observe_process(start["process"]["pid"].as_u64().unwrap() as u32)?;
        ensure!(
            observed["status"] == "running" && observed["start_ticks"] == start["process"]["start_ticks"],
            "lease identity no longer live"
        );
        let name = name.trim_end_matches("-started.toml");
        let acquired = dir.join(format!("{name}-acquired.toml"));
        if acquired.exists() {
            held.push(json!({"process":observed,"lease_receipt":seal(&acquired)?}));
        } else {
            ensure!(waiter.is_none(), "multiple unexpected lease waiters");
            waiter = Some(json!({"process":observed,"acquired":false}));
        }
    }
    ensure!(held.len() == 4 && waiter.is_some(), "need exactly four owners and one observed waiter");
    let snapshot = json!({"schema":"decomp-drill-worker-budget-snapshot-v1","observed_at":now(),"held":held,"waiter":waiter.unwrap(),"root":format!("{ROOT}/out/pipeline"),"limit":4});
    write(&state().join("worker-budget-snapshot.toml"), &snapshot)?;
    Ok(snapshot)
}
fn budget_finish() -> Result<J> {
    let dir = state().join("leases");
    let mut artifacts = vec![seal(&state().join("worker-budget-snapshot.toml"))?];
    let mut finished = 0;
    let mut waited = false;
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.file_name().unwrap().to_string_lossy().ends_with("-finished.toml") {
            let r = read(&path)?;
            ensure!(r["exit_code"] == 0, "lease failed");
            let receipt = read(Path::new(r["lease_receipt"]["path"].as_str().unwrap()))?;
            waited |= receipt["worker_wait_seconds"].as_f64().unwrap_or(0.) > 0.1;
            artifacts.push(seal(&path)?);
            finished += 1;
        }
    }
    ensure!(finished == 5 && waited, "five released leases with observed contention needed");
    let result =
        json!({"schema":"decomp-drill-worker-budget-v1","limit":4,"held":4,"contended":true,"evidence":artifacts});
    write(&state().join("worker-budget.toml"), &result)?;
    Ok(result)
}
fn release_reserved(claim: &str, token: &str, generation: u64) -> Result<J> {
    let a = slot(&format!("assignment-{claim}"))?;
    let mut q = queue(token, generation)?;
    q["operation"] = json!("record");
    q["update"] = json!({"event":"launch-failed","claim":a["claim"],"reason":"observed no worker launch: disposable negative-control capacity slot release"});
    route("launch-failed", "infrastructure-record", q, false)
}
fn retry(claim: &str, token: &str, generation: u64, reject: bool) -> Result<J> {
    let mut q = read(&state().join(format!("{claim}-exact-finish.toml")))?;
    q["coordinator"] = json!(token);
    q["authority"] = json!({"session_id":identity()?,"generation":generation});
    route("exact-retry", "infrastructure-record", q, reject)
}
fn stale_attempt(claim: &str, worker: &str, token: &str, generation: u64) -> Result<J> {
    let a = slot(&format!("assignment-{claim}-attempt-1"))?;
    let proof = state().join(format!("{claim}-owner-stop.toml"));
    ensure!(proof.exists(), "stopped original observation missing");
    let w = slot(worker)?;
    ensure!(a["claim"] != slot(&format!("assignment-{claim}"))?["claim"], "new attempt not yet exists");
    ensure!(w["session_id"] == read(&proof)?["agent"], "stale observation child differs");
    let mut q = queue(token, generation)?;
    q["operation"] = json!("record");
    q["update"] = json!({"event":"owner-stopped","claim":a["claim"],"observation":seal(&proof)?});
    route("stale-attempt", "infrastructure-record", q, true)
}
fn stale_revision(claim: &str, token: &str, generation: u64) -> Result<J> {
    let exact = read(&state().join(format!("{claim}-exact-finish.toml")))?;
    let assignment = slot(&format!("assignment-{claim}"))?;
    let mut scopes_list = read(&state().join("scopes.toml"))?;
    let original = assignment["job"]["inputs"]["infrastructure-scope-file"].as_str().unwrap();
    let mut scope = read(Path::new(original))?;
    scope["job"]["instructions"] = json!("Reviewed pending revision must reject the original submission result");
    let revision = state().join(format!("{claim}-pending-scope.toml"));
    write(&revision, &scope)?;
    let entries = scopes_list["scopes"].as_array_mut().unwrap();
    for item in entries.iter_mut() {
        if item["path"] == original {
            *item = seal(&revision)?;
        }
    }
    write(&state().join("scopes.toml"), &scopes_list)?;
    let mut plan = queue(token, generation)?;
    plan["operation"] = json!("next");
    plan["available_slots"] = json!(0);
    route("pending-revision", "infrastructure-next", plan, false)?;
    route("stale-revision", "infrastructure-record", exact, true)
}
fn setup(engine: &Path, source: &Path) -> Result<J> {
    ensure!(!Path::new(ROOT).exists(), "disposable path already exists");
    fs::create_dir_all(state())?;
    fs::write(state().join("engine-path"), engine.display().to_string())?;
    for dir in ["docs", "crates/pipeline/tests"] {
        fs::create_dir_all(Path::new(ROOT).join(dir))?;
    }
    fs::write(
        Path::new(ROOT).join("Cargo.toml"),
        "[package]\nname = \"drill-fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[[test]]\nname = \"scoped-lanes\"\npath = \"crates/pipeline/tests/drill.rs\"\n",
    )?;
    fs::write(
        Path::new(ROOT).join("crates/pipeline/tests/drill.rs"),
        "#[test]\nfn restored_lane_documents_keep_scoped_source() { for name in [\"a\",\"b\",\"c\"] { let text = std::fs::read_to_string(format!(\"docs/drill-{name}.md\")).unwrap(); assert!(text.contains(\"lane\")); } }\n",
    )?;
    for name in ["a", "b", "c"] {
        fs::write(Path::new(ROOT).join(format!("docs/drill-{name}.md")), format!("lane {name}: baseline\n"))?;
    }
    fs::write(Path::new(ROOT).join(".gitignore"), ".scratch/\nout/\ntarget/\n")?;
    git(Path::new(ROOT), &["init"])?;
    git(Path::new(ROOT), &["checkout", "-b", "drill-main"])?;
    git(Path::new(ROOT), &["config", "user.name", "Decomp drill"])?;
    git(Path::new(ROOT), &["config", "user.email", "drill@localhost"])?;
    let c = Command::new("cargo").args(["generate-lockfile", "--offline"]).current_dir(ROOT).output()?;
    ensure!(c.status.success(), "fixture lock failed");
    git(Path::new(ROOT), &["add", "Cargo.toml", "Cargo.lock", ".gitignore", "docs", "crates/pipeline/tests"])?;
    git(Path::new(ROOT), &["commit", "-m", "drill: pin isolated fixture baseline"])?;
    let baseline = git(Path::new(ROOT), &["rev-parse", "HEAD"])?;
    write(
        &state().join("bridge.toml"),
        &json!({"schema":"decomp-legacy-owner-bridge-v1","source_baseline":baseline,"holdings":[]}),
    )?;
    let mut seals = Vec::new();
    for name in ["a", "b", "c"] {
        let scope = state().join(format!("scope-{name}.toml"));
        write(
            &scope,
            &json!({"schema":"decomp-infrastructure-scope-v1","baseline":baseline,"job":{"id":format!("lane-{name}"),"revision":"reviewed","kind":"implement","subject":format!("Disposable lane {name}"),"writable_paths":[format!("docs/drill-{name}.md")],"blockers":[],"inputs":{"base-head":baseline},"instructions":"Append one scoped WIP lane statement; retain source and handoff; infrastructure-only drill.","ticket":format!(".scratch/{EFFORT}/issues/{name}.md")},"gates":[{"argv":["cargo","test","--locked","--offline"],"timeout_seconds":60,"integration":false},{"argv":["cargo","test","--locked","--offline"],"timeout_seconds":60,"integration":true}],"limitations":["isolated infrastructure concurrency evidence; no gameplay evidence"]}),
        )?;
        seals.push(seal(&scope)?);
    }
    write(&state().join("scopes.toml"), &json!({"scopes":seals}))?;
    let build = state().join("engine-build.toml");
    write(
        &build,
        &json!({"schema":"decomp-drill-engine-build-v1","source_checkout":source,"source_commit":git(source,&["rev-parse","HEAD"])? ,"source_fingerprint":fingerprint(source)?,"binary_sha256":hash(engine)?}),
    )?;
    let record = json!({"schema":"decomp-drill-frozen-input-v1","engine_binary":seal(engine)?,"engine_source_checkout":source,"engine_source_commit":git(source,&["rev-parse","HEAD"])? ,"engine_source_fingerprint":fingerprint(source)?,"engine_build":seal(&build)?,"isolated_fixture_checkout":ROOT,"isolated_fixture_baseline":baseline,"harness_source":seal(&Path::new(STORE).join("main.rs"))?,"created_at":now()});
    write(&state().join("frozen-input.toml"), &record)?;
    Ok(record)
}
fn print(v: J) -> Result<()> {
    println!("{}", toml::to_string_pretty(&toml::Value::try_from(v)?)?);
    Ok(())
}
fn main() -> Result<()> {
    let a: Vec<String> = std::env::args().collect();
    match a.get(1).map(String::as_str) {
        Some("setup") => print(setup(Path::new(&a[2]), Path::new(&a[3]))?),
        Some("launch") => print(launch(&a[2], Path::new(&a[3]), a.get(4).map(String::as_str))?),
        Some("wait") => print(sync_wait(&a[2], a[3].parse()?)?),
        Some("mark") => {
            fs::write(state().join(&a[2]), now())?;
            print(json!({"marker":a[2]}))
        }
        Some("prepare") => print(prepare()?),
        Some("init-roles") => print(init_roles()?),
        Some("session-initial") => print(session_initial(&a[2])?),
        Some("register") => print(register(&a[2])?),
        Some("identity") => {
            let id = identity()?;
            let p = state().join(format!("{}-self.toml", a[2]));
            write(&p, &json!({"session_id":id,"pid":std::process::id(),"observed_at":now()}))?;
            print(json!({"session_id":id}))
        }
        Some("next") => print(next(&a[2], a.get(3).map_or(Ok(1), |x| x.parse::<u64>())?)?),
        Some("next-token") => print(next_token(&a[2], a[3].parse()?)?),
        Some("start") => print(start(&a[2], &a[3], &a[4], a[5].parse()?)?),
        Some("dispatch") => print(dispatch(&a[2], &a[3], &a[4], a[5].parse()?)?),
        Some("checkpoint") => print(checkpoint(&a[2], &a[3], a[4].parse()?)?),
        Some("resume") => print(handoff_op("resume", &a[2], &a[3], a[4].parse()?, None, a.get(5).is_some())?),
        Some("adopt") => print(handoff_op("adopt", &a[2], &a[3], a[4].parse()?, Some(Path::new(&a[5])), false)?),
        Some("finish") => print(finish(&a[2], &a[3], a[4].parse()?)?),
        Some("transfer") => print(transfer(&a[2], &a[3], a[4].parse()?, &a[5])?),
        Some("stop-owner") => print(stop_owner(&a[2], &a[3], &a[4], a[5].parse()?)?),
        Some("lease") => print(lease_probe(&a[2])?),
        Some("integrate") => print(integrate(&a[2], a[3].parse()?)?),
        Some("integration-retry") => print(integration_retry(&a[2], a[3].parse()?)?),
        Some("retain-integrator") => print(retain_integrator(&a[2], a[3].parse()?)?),
        Some("retire-attempts") => print(retired_implementer(&a[2])?),
        Some("control-scopes") => print(controls()?),
        Some("budget-snapshot") => print(budget_snapshot()?),
        Some("budget-finish") => print(budget_finish()?),
        Some("release-reserved") => print(release_reserved(&a[2], &a[3], a[4].parse()?)?),
        Some("inspect") => print(invoke(
            "inspect",
            "decomp-session",
            &json!({"operation":"inspect","queue":queue(&a[2],a[3].parse()?)?}),
            false,
        )?),
        Some("retry") => print(retry(&a[2], &a[3], a[4].parse()?, a.get(5).is_some())?),
        Some("stale-attempt") => print(stale_attempt(&a[2], &a[3], &a[4], a[5].parse()?)?),
        Some("stale-revision") => print(stale_revision(&a[2], &a[3], a[4].parse()?)?),
        Some("observe") => {
            let s = slot(&a[2])?;
            let p = observe_process(s["pid"].as_u64().unwrap() as u32)?;
            write(&state().join(format!("{}-process-{}.toml", a[2], now())), &p)?;
            print(p)
        }
        Some("surviving") => print(surviving(&a[2], &a[3], &a[4], &a[5], a[6].parse()?)?),
        Some("signal") => {
            let s = slot(&a[2])?;
            let pid = s["pid"].as_u64().unwrap() as i32;
            let current = observe_process(pid as u32)?;
            ensure!(current["start_ticks"] == s["process"]["start_ticks"], "PID identity changed");
            unsafe { ensure!(libc::kill(pid, libc::SIGINT) == 0, "interrupt failed") };
            print(current)
        }
        Some("request") => {
            let req = read(Path::new(&a[3]))?;
            print(invoke("custom", &a[2], &req, a.get(4).is_some())?)
        }
        _ => bail!(
            "usage: decomp-drill setup ENGINE SOURCE | launch NAME PROMPT [RESUME_ID] | prepare | register a|b | next a|b [GEN] | start CLAIM WORKER TOKEN GEN | checkpoint CLAIM TOKEN GEN | finish CLAIM TOKEN GEN | retry CLAIM TOKEN GEN [reject] | resume CLAIM TOKEN GEN [reject] | adopt CLAIM TOKEN GEN TRANSFER | observe NAME | signal NAME | request ROUTE FILE [reject]"
        ),
    }
}
