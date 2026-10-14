//! Bounded offline scope validation over the maintained queue and evidence store.
//! This command grants no integrated or live gameplay stage.
#![allow(dead_code)]
use anyhow::{Context, Result, ensure};
use evidence::{Entry, EntryKind, Store};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::{BTreeMap, BTreeSet}, fs, io::Read, path::{Path, PathBuf}, process::Command};

#[path = "/home/jim/sm-decomp/crates/pipeline/src/assignment.rs"]
mod assignment;

thread_local! {
    static CAPTURE: RefCell<Option<(Store, Vec<Entry>, Vec<String>)>> = const { RefCell::new(None) };
}

// Original-byte hashes needed by the queue are produced and stored by evidence.
// The engine retains its own established source/claim fingerprint contract.
fn digest(path: &Path) -> Result<String> {
    CAPTURE.with(|capture| {
        let mut capture = capture.borrow_mut();
        if capture.is_none() {
            *capture = Some((Store::create(Path::new("/home/jim/sm-decomp/out/evidence"))?, vec![], vec![]));
        }
        let (store, entries, paths) = capture.as_mut().unwrap();
        let id = evidence::put_file(store, &mut fs::File::open(path)?)?;
        let entry = Entry { name: format!("input-{}", paths.len()), kind: EntryKind::File, id };
        let tree = evidence::put_tree(store, vec![entry.clone()])?;
        let inventory = evidence::tree_inventory(store, &tree)?;
        let value = evidence::hex(&inventory[&entry.name].sha256);
        entries.push(entry);
        paths.push(path.display().to_string());
        Ok(value)
    })
}

fn finish_capture() -> Result<Option<String>> {
    CAPTURE.with(|capture| {
        let Some((mut store, mut entries, paths)) = capture.borrow_mut().take() else { return Ok(None) };
        let bytes = toml::to_string(&BTreeMap::from([("paths", paths)]))?;
        let id = evidence::put_file(&mut store, &mut bytes.as_bytes())?;
        entries.push(Entry { name: "paths.toml".into(), kind: EntryKind::File, id });
        let tree = evidence::put_tree(&mut store, entries)?;
        let run = evidence::put_run(&mut store, tree, &evidence::Provenance::default())?;
        store.commit()?;
        Ok(Some(evidence::hex(&run)))
    })
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Scope {
    schema: String,
    baseline: String,
    subject: String,
    routines: Vec<String>,
    roots: Vec<String>,
    writable_paths: Vec<PathBuf>,
    inputs: BTreeMap<String, String>,
    limitations: Vec<String>,
}

fn git(cwd: &Path, args: &[&str]) -> Result<String> {
    let result = Command::new("git").args(args).current_dir(cwd).output()?;
    ensure!(result.status.success(), "git failed: {}", String::from_utf8_lossy(&result.stderr));
    Ok(String::from_utf8(result.stdout)?.trim().to_owned())
}

fn document(path: &Path) -> Result<toml::Value> { Ok(toml::from_str(&fs::read_to_string(path)?)?) }

fn scope_job(path: &Path) -> Result<assignment::Job> {
    let root = Path::new("/home/jim/sm-decomp");
    let scope: Scope = toml::from_str(&fs::read_to_string(path)?)?;
    ensure!(scope.schema == "decomp-offline-coverage-scope-v1" && !scope.subject.is_empty(), "invalid offline scope");
    ensure!(scope.baseline.len() == 40 && git(root, &["rev-parse", &scope.baseline])? == scope.baseline, "inexact baseline");
    ensure!(!scope.routines.is_empty() && scope.routines.len() <= 32 && !scope.roots.is_empty(), "bounded routines and roots required");
    ensure!(!scope.limitations.is_empty(), "offline limitations required");
    let routines: BTreeSet<_> = scope.routines.iter().collect();
    ensure!(routines.len() == scope.routines.len() && scope.roots.iter().all(|r| routines.contains(r)), "invalid or duplicate roots/routines");
    let inventory_path = "tools/port/source-inventory.toml";
    ensure!(scope.inputs.contains_key(inventory_path), "sealed pinned inventory required");
    for (name, hash) in &scope.inputs {
        let path = Path::new(name);
        ensure!(!path.is_absolute() && path.components().all(|p| matches!(p, std::path::Component::Normal(_))), "unsafe input");
        ensure!(digest(&root.join(path))? == *hash, "input changed: {name}");
    }
    let inventory = document(&root.join(inventory_path))?;
    let declared = inventory["routines"].as_array().context("missing inventory")?;
    let mut reachable: BTreeSet<String> = scope.roots.iter().cloned().collect();
    loop {
        let before = reachable.len();
        for row in declared {
            let symbol = row["symbol"].as_str().unwrap();
            if reachable.contains(symbol) {
                for call in row["defined_dependencies"].as_array().unwrap() {
                    let call = call.as_str().unwrap();
                    ensure!(scope.routines.iter().any(|r| r == call), "unowned gameplay dependency: {symbol} -> {call}");
                    reachable.insert(call.to_owned());
                }
            }
        }
        if before == reachable.len() { break; }
    }
    ensure!(scope.routines.iter().all(|r| reachable.contains(r)), "unrelated routines outside coherent root closure");
    for symbol in &scope.routines {
        let row = declared.iter().find(|r| r["symbol"].as_str() == Some(symbol)).context("routine missing")?;
        let source = row["source"].as_str().unwrap();
        let name = format!("deps/snesrev-sm/{source}");
        ensure!(scope.inputs.get(&name) == inventory["source_identity"]["files"].get(source).and_then(toml::Value::as_str).map(str::to_owned).as_ref(), "pinned C input seal missing");
        ensure!(row["indirect_indexed_calls"].as_array().unwrap().is_empty(), "offline scope requires explicit indirect dispatch review");
    }
    let owned: BTreeSet<_> = scope.writable_paths.iter().collect();
    ensure!(owned.len() == 3 && owned.len() == scope.writable_paths.len(), "exact source, test and manifest ownership required");
    for name in &scope.writable_paths {
        ensure!(name.components().all(|p| matches!(p, std::path::Component::Normal(_))) && (name.starts_with("crates/port/src") || name.starts_with("crates/port/tests") || name.starts_with("tools/port")), "invalid offline ownership");
    }
    let seal = digest(path)?;
    let mut inputs: BTreeMap<_,_> = scope.inputs.into_iter().map(|(k,v)| (format!("file:{k}"),v)).collect();
    inputs.insert("base-head".into(), scope.baseline);
    inputs.insert("offline-scope-file".into(), fs::canonicalize(path)?.display().to_string());
    inputs.insert("offline-scope-sha256".into(), seal.clone());
    inputs.insert("admission-domain".into(), "offline-coverage-v1".into());
    Ok(assignment::Job { id: "pause-equipment-render-offline".into(), revision: seal, kind: assignment::Kind::Implement,
        subject: scope.subject, writable_paths: scope.writable_paths, blockers: vec![], inputs,
        instructions: "Translate the eight-routine complete renderer and verify all roots through actual compiled C. Use only returned files and isolated worktree. Publish source commit and actual-C receipts; integrated/live stages remain pending. No speculative research or shared-source refactors.".into(),
        ticket: Some(".scratch/decomp-coverage/issues/02-pause-equipment-render.md".into()) })
}

fn prepare(path: &Path, output: &Path) -> Result<()> {
    let root = Path::new("/home/jim/sm-decomp");
    let inventory = document(&root.join("tools/port/source-inventory.toml"))?;
    let mut inputs = BTreeMap::new();
    for name in ["tools/port/source-inventory.toml", "tools/dependencies.toml", "deps/snesrev-sm/src/sm_81.c", "deps/snesrev-sm/src/sm_82.c", "crates/port/src/memory.rs"] {
        inputs.insert(name.to_owned(), digest(&root.join(name))?);
    }
    let scope = Scope { schema: "decomp-offline-coverage-scope-v1".into(), baseline: git(root, &["rev-parse", "HEAD"])?,
        subject: "Complete pause equipment rendering, selector, reserve tanks and wireframe queue".into(),
        routines: ["DrawMenuSpritemap", "DrawPauseScreenSpriteAnim", "EquipmentScreenDrawItemSelector", "EquipmentScreenDisplayReserveTankAmount", "EquipmentScreenDisplayReserveTankAmount_", "EquipmentScreenDisplayReserves_PaletteSetup", "WriteSamusWireframeTilemap", "WriteSamusWireframeTilemapAndQueue"].map(str::to_owned).to_vec(),
        roots: ["EquipmentScreenDrawItemSelector", "EquipmentScreenDisplayReserveTankAmount", "WriteSamusWireframeTilemapAndQueue"].map(str::to_owned).to_vec(),
        writable_paths: ["crates/port/src/pause_equipment_render.rs", "crates/port/tests/pause_equipment_render.rs", "tools/port/pause-equipment-render.toml"].map(PathBuf::from).to_vec(), inputs,
        limitations: vec!["Actual-C offline fixtures only. Live pause input, scheduling and hardware acceptance remain pending.".into(), "Lib/catalog wiring stays owned by the original integrator.".into()] };
    let _ = inventory;
    fs::write(path, toml::to_string_pretty(&scope)?)?;
    let job = scope_job(path)?;
    let request = assignment::Request { operation: "next".into(), checkout: root.into(), effort: "decomp-parallel".into(), coordinator: "decomp-run-20261013".into(),
        authority: Some(assignment::SessionAuthority { session_id: std::env::var("CODEX_THREAD_ID")?, generation: 1 }), capacity: 3, available_slots: 1, jobs: vec![job], update: None };
    let receipt = assignment::operate(request)?;
    fs::write(output, toml::to_string_pretty(&receipt)?)?;
    Ok(())
}

// Offline admission is deliberately local to this declared source batch. It
// uses the maintained ownership engine without claiming live/native admission.
fn submit(scope_path: &Path, commit: &str, output: &Path) -> Result<()> {
    let root = Path::new("/home/jim/sm-decomp");
    let worktree = Path::new("/tmp/sm-decomp-parallel-pause-equipment-render-offline-1");
    let job = scope_job(scope_path)?;
    let scope: Scope = toml::from_str(&fs::read_to_string(scope_path)?)?;
    ensure!(commit.len() == 40 && git(worktree, &["rev-parse", "HEAD"])? == commit, "submission is not the assigned source head");
    ensure!(git(worktree, &["status", "--porcelain"] )?.is_empty(), "assigned source worktree must be clean");
    let ancestry = Command::new("git").args(["merge-base", "--is-ancestor", &scope.baseline, commit]).current_dir(worktree).status()?;
    ensure!(ancestry.success(), "submission must descend from approved baseline");
    let changed = git(worktree, &["diff", "--name-only", &scope.baseline, commit])?;
    let actual: BTreeSet<_> = changed.lines().map(PathBuf::from).collect();
    let expected: BTreeSet<_> = scope.writable_paths.iter().cloned().collect();
    ensure!(actual == expected, "submission must contain exactly the three approved files");
    let ledger: assignment::Receipt = assignment::operate(assignment::Request {
        checkout: root.into(), effort: "decomp-parallel".into(), capacity: 3, ..Default::default()
    })?;
    let held = ledger.jobs.iter().find(|entry| entry.job.id == job.id).context("offline claim missing")?;
    ensure!(held.job == job && held.state == assignment::State::Running && !held.stale && held.pending_job.is_none(), "offline claim is not current and running");
    let claim = held.claim.clone().context("offline claim absent")?;
    ensure!(claim.coordinator == "decomp-run-20261013" && claim.attempt == 1, "wrong offline owner or attempt");

    let package = root.join(".scratch/decomp-coverage/renderer");
    let family_auth = package.join("family-authenticate.stdout.toml");
    let family = document(&family_auth)?;
    ensure!(family["status"].as_str() == Some("offline-verified") && family["verified_cases"].as_integer() == Some(247)
        && family["verified_routines"].as_integer() == Some(7), "complete passing generic actual-C gate required");
    let family_path = PathBuf::from(family["run"].as_str().context("family run missing")?).join("verdict.toml");
    ensure!(document(&family_path)? == family, "authenticated family differs from retained verdict");
    let roots: BTreeSet<_> = family["cases"].as_array().context("family cases missing")?.iter()
        .map(|row| row["routine"].as_str().unwrap().to_owned()).collect();
    let generic: BTreeSet<_> = scope.routines.iter().filter(|symbol| symbol.as_str() != "DrawPauseScreenSpriteAnim").cloned().collect();
    ensure!(roots == generic, "generic actual-C gate must cover all seven roots");
    let implementation = &family["implementation"];
    ensure!(digest(&worktree.join("crates/port/src/pause_equipment_render.rs"))? == implementation["module_sha256"].as_str().unwrap()
        && digest(&worktree.join("tools/port/pause-equipment-render.toml"))? == implementation["manifest"]["sha256"].as_str().unwrap(), "actual-C inputs differ from submission");
    for table in ["rust_dependencies", "harness_sources"] {
        for (path, hash) in implementation[table].as_table().context("family source seals missing")? {
            let path = Path::new(path);
            let input = if path.is_absolute() { path.into() } else { worktree.join(path) };
            ensure!(digest(&input)? == hash.as_str().unwrap(), "family source changed: {}", input.display());
        }
    }
    let animation_path = root.join("out/port/decomp-coverage-20261013/direct-animation/verdict.toml");
    let animation = document(&animation_path)?;
    let direct_auth = root.join("out/port/decomp-coverage-20261013/direct-animation-authenticate-retry.stdout");
    let authenticated = document(&direct_auth)?;
    ensure!(animation["status"].as_str() == Some("offline-verified") && animation["verified_cases"].as_integer() == Some(360)
        && animation["routine"].as_str() == Some("DrawPauseScreenSpriteAnim")
        && authenticated["status"].as_str() == Some("authenticated")
        && authenticated["verified_cases"] == animation["verified_cases"]
        && authenticated["evidence_run"] == animation["evidence_run"], "complete authenticated direct actual-C gate required");
    let direct_exec = package.join("direct-animation-authenticate-retry.stdout.toml");
    ensure!(document(&direct_exec)?["exit_code"].as_integer() == Some(0), "direct authentication execution did not pass");
    let source_commit = animation["source_commit"].as_str().context("direct source commit missing")?;
    let direct_ancestry = Command::new("git").args(["merge-base", "--is-ancestor", source_commit, commit]).current_dir(worktree).status()?;
    ensure!(direct_ancestry.success() && git(worktree, &["diff", "--name-only", source_commit, commit, "--", "crates/port/src/pause_equipment_render.rs", "crates/port/src/memory.rs"] )?.is_empty(), "direct verified source changed");
    for (path, hash) in animation["inputs"].as_table().context("direct input seals missing")? {
        ensure!(digest(Path::new(path))? == hash.as_str().unwrap(), "direct actual-C input changed: {path}");
    }
    let tests = package.join("submission-tests-retry.stdout.toml");
    let test_output = root.join("out/port/decomp-coverage-20261013/submission-tests/renderer-submission-tests.stdout");
    let test_receipt = document(&tests)?;
    let test_request = document(&package.join("submission-tests.toml"))?;
    ensure!(test_receipt["exit_code"].as_integer() == Some(0) && test_receipt["argv"] == test_request["command"]
        && test_receipt["seconds"].as_float().is_some_and(|seconds| seconds <= 60.0)
        && fs::read_to_string(&test_output)?.contains("test result: ok. 10 passed; 0 failed"), "bounded nonzero Rust test gate required");
    let cleanup = package.join("cleanup.stdout.toml");
    let dedup = package.join("cleanup-deduplicate.stdout.toml");
    ensure!(document(&cleanup)?["apply"].as_bool() == Some(true) && document(&dedup)?["apply"].as_bool() == Some(true)
        && document(&dedup)?["snapshots_checked"].as_integer() == Some(741), "both scoped cleanup passes required");
    let artifacts = [scope_path.to_owned(), family_path, family_auth, animation_path, direct_auth, direct_exec, tests, test_output, cleanup, dedup]
        .into_iter().map(|path| Ok(assignment::Artifact { sha256: digest(&path)?, path })).collect::<Result<Vec<_>>>()?;
    let receipt = assignment::operate(assignment::Request { operation: "record".into(), checkout: root.into(),
        effort: "decomp-parallel".into(), coordinator: claim.coordinator.clone(), capacity: 3,
        authority: Some(assignment::SessionAuthority { session_id: std::env::var("CODEX_THREAD_ID")?, generation: 1 }),
        update: Some(assignment::Update::Finished { claim, result: assignment::WorkResult::Submitted { commit: commit.into(), evidence: artifacts } }),
        ..Default::default() })?;
    fs::write(output, toml::to_string_pretty(&receipt)?)?;
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let result = match args.get(1).map(String::as_str) {
        Some("prepare") => prepare(Path::new(&args[2]), Path::new(&args[3])),
        Some("submit") => submit(Path::new(&args[2]), &args[3], Path::new(&args[4])),
        Some("queue") => {
            let mut input = String::new(); std::io::stdin().read_to_string(&mut input)?;
            let request: assignment::Request = toml::from_str(&input)?;
            ensure!(request.jobs.is_empty(), "raw jobs forbidden; use sealed offline prepare");
            if let Some(assignment::Update::Finished { result, .. }) = &request.update {
                ensure!(!matches!(result, assignment::WorkResult::Submitted { .. } | assignment::WorkResult::Integrated { .. }), "offline source results require validated submit; live/native integration is not admitted here");
            }
            let receipt = assignment::operate(request)?;
            println!("{}", toml::to_string_pretty(&receipt)?); Ok(())
        }
        _ => anyhow::bail!("use prepare SCOPE RECEIPT, submit SCOPE COMMIT RECEIPT, or queue < typed-request.toml"),
    };
    if let Some(run) = finish_capture()? { eprintln!("evidence_run = {run}"); }
    result
}
