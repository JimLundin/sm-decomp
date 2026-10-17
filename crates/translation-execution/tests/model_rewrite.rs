#![cfg(all(target_os = "linux", target_arch = "x86_64"))]
use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::Command,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use translation_execution::{
    CapsuleBinding, ExecutionPhase, OperationContext, Replacement, RewriteCapsule, RewriteLimits, SourceReplacements,
    model::sha256,
    model_rewrite::{self, ModelRewriteAdapter, Provider},
    operation, retention, rewrite,
    rewrite_adapter::RewriteAdapter,
};

static NEXT: AtomicU64 = AtomicU64::new(0);
const TOKEN: &str = "loopback-test-credential";

struct Fixture {
    root: PathBuf,
    capsule: RewriteCapsule,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "sm-model-transport-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/domain.rs"), "pub fn doubled(n: u32) -> u32 { n * 2 }\n").unwrap();
        fs::copy(env!("CARGO_BIN_EXE_sm-translation-sandbox"), root.join("helper")).unwrap();
        let capsule = rewrite::capture(
            &root,
            CapsuleBinding {
                run_id: sha256(root.to_str().unwrap()),
                attempt: 1,
                policy_sha256: sha256("frozen policy"),
                source_sha256: sha256("frozen source"),
            },
            &["src/domain.rs".into()],
            "Make the multiply explicit while preserving results.".into(),
            limits(),
        )
        .unwrap();
        Self { root, capsule }
    }
    fn context(&self) -> OperationContext {
        OperationContext {
            run_id: self.capsule.run_id.clone(),
            attempt: 1,
            policy_sha256: self.capsule.policy_sha256.clone(),
            operation_key: sha256("rewrite"),
            phase: ExecutionPhase::Candidate,
        }
    }
    fn adapter(&self, provider: Provider) -> ModelRewriteAdapter {
        let helper = self.root.join("helper");
        ModelRewriteAdapter::new(
            self.root.join("operations"),
            provider,
            helper.clone(),
            retention::seal_file(&helper, 512 * 1024 * 1024).unwrap().sha256,
        )
        .unwrap()
    }
}
fn limits() -> RewriteLimits {
    RewriteLimits { max_context_bytes: 8192, max_response_bytes: 8192, max_file_bytes: 4096, max_total_bytes: 4096 }
}
fn provider(listener: &TcpListener) -> Provider {
    Provider {
        endpoint: format!("http://{}/responses", listener.local_addr().unwrap()),
        token_env: "SM_MODEL_TRANSPORT_TEST_TOKEN".into(),
        model: "frozen-model".into(),
        reasoning_effort: "low".into(),
        max_output_tokens: 2048,
        timeout_seconds: 4,
        max_response_bytes: 16384,
    }
}
fn set_token() {
    unsafe {
        std::env::set_var("SM_MODEL_TRANSPORT_TEST_TOKEN", TOKEN);
    }
}
fn read_request(stream: &mut TcpStream) -> serde_json::Value {
    stream.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    let mut bytes = Vec::new();
    let mut block = [0u8; 1024];
    let header_end = loop {
        let n = stream.read(&mut block).unwrap();
        assert!(n > 0);
        bytes.extend_from_slice(&block[..n]);
        if let Some(pos) = bytes.windows(4).position(|b| b == b"\r\n\r\n") {
            break pos + 4;
        }
    };
    let header = std::str::from_utf8(&bytes[..header_end]).unwrap().to_ascii_lowercase();
    assert!(header.contains(&format!("authorization: bearer {TOKEN}")));
    let size: usize = header.lines().find_map(|s| s.strip_prefix("content-length: ")).unwrap().parse().unwrap();
    while bytes.len() < header_end + size {
        let n = stream.read(&mut block).unwrap();
        assert!(n > 0);
        bytes.extend_from_slice(&block[..n]);
    }
    let value: serde_json::Value = serde_json::from_slice(&bytes[header_end..header_end + size]).unwrap();
    assert_eq!(value["tools"], serde_json::json!([]));
    assert_eq!(value["tool_choice"], "none");
    assert_eq!(value["store"], false);
    assert_eq!(value["text"]["format"]["strict"], true);
    value
}
fn reply(stream: &mut TcpStream, status: &str, body: &[u8], extra: &str) {
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n{extra}\r\n",
        body.len()
    )
    .unwrap();
    stream.write_all(body).unwrap();
}
fn source_response(capsule: &RewriteCapsule) -> Vec<u8> {
    let source = &capsule.files[0];
    let replacements = SourceReplacements {
        capsule_id: capsule.id.clone(),
        files: vec![Replacement {
            path: source.path.clone(),
            base_sha256: source.sha256.clone(),
            content: "pub fn doubled(n: u32) -> u32 { n + n }\n".into(),
        }],
    };
    serde_json::to_vec(&serde_json::json!({"status":"completed","error":null,"output":[{"type":"message","role":"assistant","status":"completed",
        "content":[{"type":"output_text","text":serde_json::to_string(&replacements).unwrap()}]}]})).unwrap()
}

#[test]
fn successful_exchange_reconciles_without_resending_and_authenticates_tamper() {
    set_token();
    let fixture = Fixture::new();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let config = provider(&listener);
    let capsule = fixture.capsule.clone();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let value = read_request(&mut stream);
        let sent: RewriteCapsule = serde_json::from_str(value["input"][1]["content"].as_str().unwrap()).unwrap();
        assert_eq!(sent, capsule);
        reply(&mut stream, "200 OK", &source_response(&capsule), "");
    });
    let mut adapter = fixture.adapter(config.clone());
    let context = fixture.context();
    let first = adapter.rewrite(&context, &fixture.capsule, limits()).unwrap();
    server.join().unwrap();
    assert!(first.files[0].content.contains("n + n"));
    assert_eq!(adapter.rewrite(&context, &fixture.capsule, limits()).unwrap(), first);
    let references = model_rewrite::references(&fixture.root.join("operations"), &context).unwrap();
    for reference in references {
        assert!(!fs::read(&reference.path).unwrap().windows(TOKEN.len()).any(|b| b == TOKEN.as_bytes()));
    }
    let mut changed = config;
    changed.model = "another-model".into();
    assert!(fixture.adapter(changed).rewrite(&context, &fixture.capsule, limits()).is_err());
    let response = fixture.root.join("operations").join(&context.operation_key).join("response.json");
    fs::set_permissions(&response, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(response, b"tampered").unwrap();
    assert!(model_rewrite::authenticate(&fixture.root.join("operations"), &context).is_err());
}

#[test]
fn redirect_error_refusal_and_overflow_are_retained_without_retry_or_secret() {
    set_token();
    for mode in ["redirect", "error", "refusal", "overflow", "secret", "disconnect"] {
        let fixture = Fixture::new();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let config = provider(&listener);
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            read_request(&mut stream);
            match mode {
                "redirect" => {
                    reply(&mut stream, "307 Temporary Redirect", b"", "Location: http://127.0.0.1:1/responses\r\n")
                }
                "error" => reply(&mut stream, "429 Too Many Requests", TOKEN.as_bytes(), ""),
                "refusal" => reply(&mut stream, "200 OK", b"{\"status\":\"incomplete\",\"output\":[]}", ""),
                "overflow" => {
                    write!(stream, "HTTP/1.1 200 OK\r\nContent-Length: 20000\r\n\r\n").unwrap();
                }
                "secret" => reply(&mut stream, "200 OK", TOKEN.as_bytes(), ""),
                _ => {}
            }
        });
        let mut adapter = fixture.adapter(config);
        let context = fixture.context();
        assert!(adapter.rewrite(&context, &fixture.capsule, limits()).is_err());
        server.join().unwrap();
        let before = model_rewrite::authenticate(&fixture.root.join("operations"), &context).unwrap();
        assert!(!before.passed);
        assert!(adapter.rewrite(&context, &fixture.capsule, limits()).is_err());
        let after = model_rewrite::authenticate(&fixture.root.join("operations"), &context).unwrap();
        assert_eq!(before, after);
        for reference in model_rewrite::references(&fixture.root.join("operations"), &context).unwrap() {
            assert!(!fs::read(reference.path).unwrap().windows(TOKEN.len()).any(|b| b == TOKEN.as_bytes()));
        }
    }
}

#[test]
fn killed_controller_recovers_the_surviving_original_exchange() {
    set_token();
    let fixture = Fixture::new();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let config = provider(&listener);
    fs::write(fixture.root.join("provider.json"), serde_json::to_vec(&config).unwrap()).unwrap();
    fs::write(fixture.root.join("capsule.json"), serde_json::to_vec(&fixture.capsule).unwrap()).unwrap();
    fs::write(fixture.root.join("context.json"), serde_json::to_vec(&fixture.context()).unwrap()).unwrap();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (continue_tx, continue_rx) = mpsc::channel();
    let capsule = fixture.capsule.clone();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        read_request(&mut stream);
        entered_tx.send(()).unwrap();
        continue_rx.recv().unwrap();
        reply(&mut stream, "200 OK", &source_response(&capsule), "");
    });
    let mut controller = Command::new(std::env::current_exe().unwrap())
        .args(["model_controller_child", "--exact", "--ignored"])
        .env("SM_MODEL_CONTROLLER_CHILD", &fixture.root)
        .spawn()
        .unwrap();
    entered_rx.recv_timeout(Duration::from_secs(10)).unwrap();
    controller.kill().unwrap();
    controller.wait().unwrap();
    let mut adapter = fixture.adapter(config);
    let context = fixture.context();
    assert!(
        adapter
            .rewrite(&context, &fixture.capsule, limits())
            .unwrap_err()
            .downcast_ref::<translation_execution::ExecutionPending>()
            .is_some()
    );
    continue_tx.send(()).unwrap();
    server.join().unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match adapter.rewrite(&context, &fixture.capsule, limits()) {
            Ok(value) => {
                assert!(value.files[0].content.contains("n + n"));
                break;
            }
            Err(error)
                if error.downcast_ref::<translation_execution::ExecutionPending>().is_some()
                    && Instant::now() < deadline =>
            {
                std::thread::sleep(Duration::from_millis(20))
            }
            Err(error) => panic!("{error:#}"),
        }
    }
}

#[test]
#[ignore = "invoked only as the crash-recovery test's child controller"]
fn model_controller_child() {
    let root = PathBuf::from(std::env::var_os("SM_MODEL_CONTROLLER_CHILD").unwrap());
    let provider = serde_json::from_slice(&fs::read(root.join("provider.json")).unwrap()).unwrap();
    let capsule = serde_json::from_slice(&fs::read(root.join("capsule.json")).unwrap()).unwrap();
    let context = serde_json::from_slice(&fs::read(root.join("context.json")).unwrap()).unwrap();
    let helper = root.join("helper");
    let adapter = ModelRewriteAdapter::new(
        root.join("operations"),
        provider,
        helper.clone(),
        retention::seal_file(&helper, 512 * 1024 * 1024).unwrap().sha256,
    )
    .unwrap();
    let deadline = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64 + 10000;
    adapter.run_until(&context, &capsule, limits(), deadline).unwrap();
}

#[test]
fn killing_exact_runner_seals_ambiguous_delivery_and_never_resends() {
    set_token();
    let fixture = Fixture::new();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let config = provider(&listener);
    let (tx, rx) = mpsc::channel();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        read_request(&mut stream);
        tx.send(()).unwrap();
        let mut block = [0];
        let _ = stream.read(&mut block);
    });
    let adapter = fixture.adapter(config.clone());
    let context = fixture.context();
    let capsule = fixture.capsule.clone();
    let copied = context.clone();
    let worker = std::thread::spawn(move || {
        let deadline = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64 + 10000;
        adapter.run_until(&copied, &capsule, limits(), deadline)
    });
    rx.recv_timeout(Duration::from_secs(10)).unwrap();
    let operation_path = fixture.root.join("operations").join(&context.operation_key);
    let value: serde_json::Value =
        serde_json::from_slice(&fs::read(operation_path.join("runner.json")).unwrap()).unwrap();
    let identity: operation::ProcessIdentity = serde_json::from_value(value["process"].clone()).unwrap();
    assert_eq!(operation::process_identity(identity.pid).unwrap().as_ref(), Some(&identity));
    unsafe {
        libc::kill(identity.pid as i32, libc::SIGKILL);
    }
    let result = worker.join().unwrap().unwrap();
    assert!(!result.passed);
    server.join().unwrap();
    assert_eq!(result, model_rewrite::stop(&fixture.root.join("operations"), &context).unwrap());
    let mut replay = fixture.adapter(config);
    assert!(replay.rewrite(&context, &fixture.capsule, limits()).is_err());
}

#[test]
#[ignore = "one real bounded Foundry request; opt-in with FOUNDRY_TOKEN and SM_MODEL_FOUNDRY_PROOF"]
fn foundry_rewrite_uses_only_capsule_and_structured_source_response() {
    let root = PathBuf::from(std::env::var_os("SM_MODEL_FOUNDRY_PROOF").unwrap());
    assert!(!root.exists());
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/domain.rs"), "pub fn add_two(n: u32) -> u32 { n.wrapping_add(1).wrapping_add(1) }\n")
        .unwrap();
    fs::copy(env!("CARGO_BIN_EXE_sm-translation-sandbox"), root.join("helper")).unwrap();
    let limits = RewriteLimits {
        max_context_bytes: 8192,
        max_response_bytes: 16384,
        max_file_bytes: 8192,
        max_total_bytes: 8192,
    };
    let capsule = rewrite::capture(&root, CapsuleBinding { run_id: sha256(root.to_str().unwrap()), attempt: 1,
        policy_sha256: sha256("Foundry structured output readiness"), source_sha256: sha256("Foundry readiness source") },
        &["src/domain.rs".into()], "Simplify two wrapping additions to one wrapping_add(2), preserving all u32 results. This is a transport readiness diagnostic.".into(), limits).unwrap();
    let fixture = Fixture { root: root.clone(), capsule };
    let config = Provider {
        endpoint: "https://fcg.palantirfoundry.com/language-model-service/api/proxy/openai/v1/responses".into(),
        token_env: "FOUNDRY_TOKEN".into(),
        model: "gpt-6.1-sol".into(),
        reasoning_effort: "low".into(),
        max_output_tokens: 4096,
        timeout_seconds: 90,
        max_response_bytes: 65536,
    };
    let mut adapter = fixture.adapter(config);
    let context = fixture.context();
    let value = adapter.rewrite(&context, &fixture.capsule, limits).unwrap();
    assert!(value.files[0].content.contains("wrapping_add(2)"));
    assert_eq!(adapter.rewrite(&context, &fixture.capsule, limits).unwrap(), value);
    fs::write(
        root.join("references.json"),
        serde_json::to_vec(&model_rewrite::references(&root.join("operations"), &context).unwrap()).unwrap(),
    )
    .unwrap();
}
