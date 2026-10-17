use super::*;
use crate::translation_execution::{
    installation_identity,
    tests::{Project, native_installation},
};
use ::translation_execution::{
    model::*,
    model_rewrite::{self, Provider},
};
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
fn proof_tools() -> bool {
    std::env::var_os("SM_TRANSLATION_TEST_HELPER").is_some()
}
fn provider(listener: &TcpListener) -> Provider {
    Provider {
        endpoint: format!("http://{}/responses", listener.local_addr().unwrap()),
        token_env: "SM_TRANSLATION_LAUNCH_TEST_TOKEN".into(),
        model: "fixed-local-model".into(),
        reasoning_effort: "low".into(),
        max_output_tokens: 4096,
        timeout_seconds: 20,
        max_response_bytes: 65536,
    }
}
fn serve(listener: TcpListener, count: usize, fail: bool) -> std::thread::JoinHandle<Vec<String>> {
    std::thread::spawn(move || {
        let mut paths = Vec::new();
        for _ in 0..count {
            let (mut stream, _) = listener.accept().unwrap();
            stream.set_read_timeout(Some(Duration::from_secs(60))).unwrap();
            let mut bytes = Vec::new();
            let mut buffer = [0u8; 4096];
            let end = loop {
                let n = stream.read(&mut buffer).unwrap();
                assert!(n > 0);
                bytes.extend_from_slice(&buffer[..n]);
                if let Some(end) = bytes.windows(4).position(|s| s == b"\r\n\r\n") {
                    break end + 4;
                }
            };
            let headers = String::from_utf8_lossy(&bytes[..end]).to_ascii_lowercase();
            let size: usize =
                headers.lines().find_map(|s| s.strip_prefix("content-length: ")).unwrap().parse().unwrap();
            while bytes.len() < end + size {
                let n = stream.read(&mut buffer).unwrap();
                assert!(n > 0);
                bytes.extend_from_slice(&buffer[..n]);
            }
            let request: serde_json::Value = serde_json::from_slice(&bytes[end..end + size]).unwrap();
            assert_eq!(request["tools"], serde_json::json!([]));
            assert_eq!(request["store"], false);
            let capsule: RewriteCapsule =
                serde_json::from_str(request["input"][1]["content"].as_str().unwrap()).unwrap();
            paths.push(capsule.run_id.clone());
            let original = &capsule.files[0];
            let value = SourceReplacements {
                capsule_id: capsule.id,
                files: vec![Replacement {
                    path: original.path.clone(),
                    base_sha256: original.sha256.clone(),
                    content: if fail {
                        "pub fn doubled(value: u32) -> u32 { value * 3 }\n".into()
                    } else {
                        "pub fn doubled(value: u32) -> u32 { value + value }\n".into()
                    },
                }],
            };
            let body = serde_json::to_vec(&serde_json::json!({"status":"completed","output":[{"type":"message","role":"assistant","status":"completed",
            "content":[{"type":"output_text","text":serde_json::to_string(&value).unwrap()}]}]})).unwrap();
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n",
                body.len()
            )
            .unwrap();
            stream.write_all(&body).unwrap();
        }
        paths
    })
}
fn plan(projects: &mut [Project], provider: Provider) -> Result<PathBuf> {
    let root = std::env::temp_dir().join(format!(
        "sm-ordered-launch-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&root)?;
    let mut scopes = Vec::new();
    for (i, project) in projects.iter_mut().enumerate() {
        project.installation.policy.scope_id = format!("arithmetic-scope-{i}");
        project.installation.rewrite.recorded = false;
        project.installation.rewrite.provider = Some(provider.clone());
        project.installation.policy.adapter_sha256 = installation_identity(&project.installation)?;
        let path = root.join(format!("scope-{i}.json"));
        immutable(&path, &project.installation)?;
        scopes.push(Scope {
            id: format!("scope-{i}"),
            after: if i == 0 { vec![] } else { vec![format!("scope-{}", i - 1)] },
            installation: retention::seal_file(&path, MAX_RECORD)?,
        });
    }
    let plan = Plan {
        schema: "translation-launch-plan-v1".into(),
        id: "ordered-proof".into(),
        state_root: root.join("state"),
        scopes,
    };
    let path = root.join("plan.json");
    immutable(&path, &plan)?;
    Ok(path)
}
fn files(root: &Path) -> Result<BTreeMap<PathBuf, String>> {
    let mut selected = BTreeMap::new();
    if root.is_dir() {
        for entry in fs::read_dir(root)? {
            let path = entry?.path();
            if path.is_dir() {
                selected.extend(files(&path)?);
            } else {
                selected.insert(path.clone(), crate::digest(&path)?);
            }
        }
    }
    Ok(selected)
}
use std::collections::BTreeMap;

#[test]
fn ordered_launch_prepares_runs_once_and_authenticates_read_only_recovery() -> Result<()> {
    if !proof_tools() {
        return Ok(());
    }
    unsafe {
        std::env::set_var("SM_TRANSLATION_LAUNCH_TEST_TOKEN", "fixed-loopback-secret");
    }
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let config = provider(&listener);
    let server = serve(listener, 2, false);
    let mut projects = [Project::new()?, Project::new()?];
    let path = plan(&mut projects, config)?;
    let prepared = run(&path, "prepare", None, None)?;
    ensure!(prepared.state == "prepared", "prepare advanced a scope");
    let inspected = run(&path, "inspect", None, None)?;
    ensure!(inspected.state == "ready", "initial read-only state differs");
    let complete = run(&path, "run", None, None)?;
    ensure!(
        complete.state == "complete" && complete.completed_scopes == ["scope-0", "scope-1"],
        "ordered scopes did not complete: {complete:?}"
    );
    ensure!(server.join().unwrap().len() == 2, "model call count differs");
    let plan: Plan = read(&path)?;
    let before = files(&plan.state_root)?;
    ensure!(
        run(&path, "inspect", None, None)?.state == "complete" && files(&plan.state_root)? == before,
        "inspect changed launch state"
    );
    ensure!(
        run(&path, "run", None, None)?.state == "complete" && files(&plan.state_root)? == before,
        "resume changed launch state"
    );
    fs::remove_file(completed_path(&plan, "scope-1"))?;
    let before = files(&projects[1].installation.state_root)?;
    ensure!(
        run(&path, "run", None, None)?.state == "complete" && files(&projects[1].installation.state_root)? == before,
        "missing launch projection executed a producer"
    );
    let mut changed = plan.clone();
    changed.scopes[0].after = vec!["scope-1".into()];
    ensure!(changed.validate(&path).is_err(), "out-of-order plan accepted");
    ensure!(run(&path, "repair", Some("scope-0"), Some("explicit reason")).is_err(), "completed scope reevaluated");
    Ok(())
}

#[test]
fn failing_scope_stops_ordered_plan_without_resending_or_starting_successor() -> Result<()> {
    if !proof_tools() {
        return Ok(());
    }
    unsafe {
        std::env::set_var("SM_TRANSLATION_LAUNCH_TEST_TOKEN", "fixed-loopback-secret");
    }
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let config = provider(&listener);
    let server = serve(listener, 1, true);
    let mut projects = [Project::new()?, Project::new()?];
    let path = plan(&mut projects, config)?;
    let stopped = run(&path, "run", None, None)?;
    ensure!(
        stopped.state == "stopped" && stopped.current_scope.as_deref() == Some("scope-0"),
        "failed scope did not stop: {stopped:?}"
    );
    server.join().unwrap();
    ensure!(!projects[1].installation.state_root.exists(), "failed predecessor started successor");
    let before = files(&projects[0].installation.state_root)?;
    ensure!(
        run(&path, "run", None, None)?.state == "stopped" && files(&projects[0].installation.state_root)? == before,
        "failed scope silently reevaluated"
    );
    ensure!(run(&path, "repair", Some("scope-1"), Some("reason")).is_err(), "operator bypassed unfinished predecessor");
    let plan: Plan = read(&path)?;
    let _held = lease(&plan.state_root)?;
    ensure!(run(&path, "run", None, None).is_err(), "concurrent launch admitted");
    Ok(())
}

#[test]
fn preparing_model_installation_is_repeatable_and_refuses_changed_request() -> Result<()> {
    if !proof_tools() {
        return Ok(());
    }
    let project = Project::new()?;
    let root = project.root.join("launch-inputs");
    fs::create_dir_all(&root)?;
    let template = root.join("template.json");
    immutable(&template, &project.installation)?;
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let request = crate::translation_launch_prepare::Request {
        schema: "translation-launch-request-v1".into(),
        id: "preparation-proof".into(),
        output_root: project.root.join("model-installed"),
        helper: retention::seal_file(&project.installation.helper.path, 512 * 1024 * 1024)?,
        provider: provider(&listener),
        scopes: vec![crate::translation_launch_prepare::ScopeRequest {
            id: "arithmetic".into(),
            after: vec![],
            template: retention::seal_file(&template, MAX_RECORD)?,
        }],
    };
    let path = root.join("request.json");
    immutable(&path, &request)?;
    let first = crate::translation_launch_prepare::prepare(&path, false)?;
    let before = files(&request.output_root)?;
    ensure!(
        crate::translation_launch_prepare::prepare(&path, false)? == first && files(&request.output_root)? == before,
        "identical preparation changed outputs"
    );
    ensure!(
        crate::translation_launch_prepare::prepare(&path, true)? == first && files(&request.output_root)? == before,
        "read-only preparation changed outputs"
    );
    let mut changed = request.clone();
    changed.provider.model = "different-model".into();
    let other = root.join("different-request.json");
    immutable(&other, &changed)?;
    ensure!(
        crate::translation_launch_prepare::prepare(&other, false).is_err(),
        "changed model reused installed execution"
    );
    Ok(())
}

#[test]
#[ignore = "fresh closed actual-C source and concrete Foundry launch request; does not call a model"]
fn prepare_concrete_foundry_native_launch_request() -> Result<()> {
    let root = PathBuf::from(std::env::var("SM_MODEL_NATIVE_LAUNCH_ROOT")?);
    let template_root = root.join("approved-native");
    let mut template = native_installation(&template_root)?;
    let controller = PathBuf::from(std::env::var("SM_MODEL_LAUNCH_CLI")?);
    template.controller =
        crate::translation_execution::InstalledFile { path: controller.clone(), sha256: crate::digest(&controller)? };
    template.policy.budgets.max_attempts = 3;
    template.policy.scope_id = "enemy-jump-instructions-model-launch".into();
    template.policy.adapter_sha256 = installation_identity(&template)?;
    let template_path = root.join("approved-template.json");
    immutable(&template_path, &template)?;
    let helper = PathBuf::from(std::env::var("SM_TRANSLATION_TEST_HELPER")?);
    let provider = Provider {
        endpoint: "https://fcg.palantirfoundry.com/language-model-service/api/proxy/openai/v1/responses".into(),
        token_env: "FOUNDRY_TOKEN".into(),
        model: "gpt-6.1-sol".into(),
        reasoning_effort: "high".into(),
        max_output_tokens: 32768,
        timeout_seconds: 300,
        max_response_bytes: 1024 * 1024,
    };
    let request = crate::translation_launch_prepare::Request {
        schema: "translation-launch-request-v1".into(),
        id: "foundry-native-launch".into(),
        output_root: root.join("execution"),
        helper: retention::seal_file(&helper, 512 * 1024 * 1024)?,
        provider,
        scopes: vec![crate::translation_launch_prepare::ScopeRequest {
            id: "enemy-jump-instructions".into(),
            after: vec![],
            template: retention::seal_file(&template_path, MAX_RECORD)?,
        }],
    };
    let request_path = root.join("request.json");
    immutable(&request_path, &request)?;
    println!("Concrete launch request: {}", request_path.display());
    Ok(())
}

#[test]
#[ignore = "authenticate the separately launched real Foundry/actual-C lifecycle without producers"]
fn authenticate_concrete_foundry_native_launch() -> Result<()> {
    let root = PathBuf::from(std::env::var("SM_MODEL_NATIVE_LAUNCH_ROOT")?);
    let plan: Plan = read(&root.join("execution/plan.json"))?;
    let installed: Installation = read(&plan.scopes[0].installation.path)?;
    let run_id = sha256(format!("translation-execution-run-v1:{}", sealed(&installed.policy)?));
    let run_root = installed.state_root.join(&run_id);
    let before = files(&run_root)?;
    let started = Instant::now();
    let policy = crate::translation_execution::authenticate_retained_proof(
        &plan.scopes[0].installation.path,
        &run_root.join("attempt-0001/publication/completion.json"),
    )?;
    ensure!(policy == installed.policy, "real model lifecycle policy differs");
    ensure!(files(&run_root)? == before, "retained authentication allocated evidence");
    let mut model_calls = 0;
    for entry in fs::read_dir(run_root.join("model-operations"))? {
        let path = entry?.path();
        let intent: serde_json::Value = read(&path.join("intent.json"))?;
        let context: OperationContext = serde_json::from_value(intent["context"].clone())?;
        let result = model_rewrite::authenticate(&run_root.join("model-operations"), &context)?;
        ensure!(result.passed, "real model operation failed");
        model_calls += 1;
    }
    ensure!(model_calls == 1, "real lifecycle evaluated model more than once");
    immutable(
        &root.join("authenticated.json"),
        &serde_json::json!({"schema":"translation-model-launch-proof-v1", "model_calls":model_calls,
        "elapsed_seconds":started.elapsed().as_secs_f64(), "retained_files":before.len(), "authenticated":true,
        "fixture_pairs_per_gate":16,"gates":4,"original_rom_parity":"not-tested","live_replay":"not-tested"}),
    )?;
    Ok(())
}
