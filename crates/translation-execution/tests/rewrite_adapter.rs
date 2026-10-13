#![cfg(all(target_os = "linux", target_arch = "x86_64"))]
use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};
use translation_execution::{
    CapsuleBinding, ExecutionPhase, OperationContext, Replacement, RewriteCapsule, RewriteLimits, SourceReplacements,
    confinement::{Program, ReadOnlyBind, RunLimits, helper_runtime},
    model::sha256,
    rewrite,
    rewrite_adapter::{InstalledRewriteAdapter, RecordedRewriteAdapter, RewriteAdapter, installed_identity},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    root: PathBuf,
    capsule: RewriteCapsule,
    response: SourceReplacements,
}

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "sm-rewrite-adapter-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/native.rs"), "pub fn translated() -> u8 { 0 }\n").unwrap();
        fs::write(root.join("src/dependency.rs"), "pub const DEPENDENCY: u8 = 1;\n").unwrap();
        fs::create_dir_all(root.join("tests")).unwrap();
        fs::write(root.join("tests/frozen.rs"), "frozen verification rules\n").unwrap();
        fs::write(root.join("Cargo.toml"), "frozen manifest\n").unwrap();
        // Cargo artifacts have two links. Install independent immutable copies,
        // just as the supervisor installation does, before sealing executables.
        fs::copy(env!("CARGO_BIN_EXE_sm-translation-sandbox"), root.join("sandbox-helper")).unwrap();
        fs::copy(env!("CARGO_BIN_EXE_sm-translation-recorded-rewrite"), root.join("recorded-rewrite")).unwrap();
        let capsule = rewrite::capture(
            &root,
            CapsuleBinding {
                run_id: sha256("restricted-rewrite-proof"),
                attempt: 1,
                policy_sha256: sha256("fixed-policy"),
                source_sha256: sha256("fixed-source"),
            },
            &["src/native.rs".into(), "src/dependency.rs".into()],
            "recovered C and declared dependency context".into(),
            limits(),
        )
        .unwrap();
        let original = capsule.files.iter().find(|file| file.path == "src/native.rs").unwrap();
        let response = SourceReplacements {
            capsule_id: capsule.id.clone(),
            files: vec![Replacement {
                path: original.path.clone(),
                base_sha256: original.sha256.clone(),
                content: "pub fn translated() -> u8 { 1 }\n".into(),
            }],
        };
        Self { root, capsule, response }
    }

    fn context(&self, name: &str) -> OperationContext {
        OperationContext {
            run_id: self.capsule.run_id.clone(),
            attempt: self.capsule.attempt,
            policy_sha256: self.capsule.policy_sha256.clone(),
            operation_key: sha256(name),
            phase: ExecutionPhase::Candidate,
        }
    }

    fn recorded_program(&self) -> Program {
        RecordedRewriteAdapter::program(
            self.root.join("recorded-rewrite"),
            self.root.join("sandbox-helper"),
            &self.response,
        )
        .unwrap()
    }

    fn recorded(&self) -> RecordedRewriteAdapter {
        let program = self.recorded_program();
        let seal = installed_identity(&program, &run_limits()).unwrap();
        RecordedRewriteAdapter::new(
            self.root.join("operations"),
            program,
            run_limits(),
            seal,
            &self.capsule,
            &self.response,
            limits(),
        )
        .unwrap()
    }

    fn probe(&self, name: &str, behavior: &str, output: &[u8]) -> Program {
        let source = self.root.join(format!("{name}.rs"));
        let executable = self.root.join(name);
        let text = format!(
            r#"
use std::io::{{Read, Write}};
fn main() {{
    if std::env::args().nth(1).as_deref() == Some("--host-verifier") {{
        std::fs::write({marker:?}, b"unrestricted verifier ran").unwrap();
        return;
    }}
    let mut capsule = Vec::new();
    std::io::stdin().read_to_end(&mut capsule).unwrap();
    assert!(capsule.starts_with(b"{{"));
    {behavior}
    std::io::stdout().write_all(&{output:?}).unwrap();
}}
"#,
            marker = self.root.join("host-verifier-marker").display().to_string()
        );
        fs::write(&source, text).unwrap();
        let compile = Command::new("rustc")
            .args(["--edition=2024", "-C", "opt-level=0"])
            .arg(&source)
            .arg("-o")
            .arg(&executable)
            .output()
            .unwrap();
        assert!(compile.status.success(), "{}", String::from_utf8_lossy(&compile.stderr));
        Program {
            executable,
            arguments: vec![],
            inputs: vec![],
            outputs: vec![],
            runtime: helper_runtime().unwrap(),
            helper: self.root.join("sandbox-helper"),
        }
    }

    fn installed(&self, program: Program) -> InstalledRewriteAdapter {
        let seal = installed_identity(&program, &run_limits()).unwrap();
        InstalledRewriteAdapter::new(self.root.join("operations"), program, run_limits(), seal).unwrap()
    }

    fn assert_original(&self) {
        assert_eq!(fs::read_to_string(self.root.join("src/native.rs")).unwrap(), "pub fn translated() -> u8 { 0 }\n");
        assert_eq!(fs::read_to_string(self.root.join("src/dependency.rs")).unwrap(), "pub const DEPENDENCY: u8 = 1;\n");
        assert_eq!(fs::read_to_string(self.root.join("tests/frozen.rs")).unwrap(), "frozen verification rules\n");
        assert_eq!(fs::read_to_string(self.root.join("Cargo.toml")).unwrap(), "frozen manifest\n");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn limits() -> RewriteLimits {
    RewriteLimits { max_context_bytes: 4096, max_response_bytes: 8192, max_file_bytes: 4096, max_total_bytes: 8192 }
}

fn run_limits() -> RunLimits {
    RunLimits {
        timeout: Duration::from_secs(5),
        writable_bytes: 128 * 1024,
        stdout_bytes: 8192,
        stderr_bytes: 8192,
        stdin_bytes: 16384,
    }
}

fn retained(path: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            entries.extend(retained(&path));
        } else {
            entries.push((path.clone(), fs::read(path).unwrap()));
        }
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries
}

#[test]
fn recorded_rewrite_uses_confined_process_and_replays_without_new_output() {
    let fixture = Fixture::new();
    let mut adapter = fixture.recorded();
    let context = fixture.context("recorded-rewrite");
    let response = adapter.rewrite(&context, &fixture.capsule, limits()).unwrap();
    assert_eq!(response, fixture.response);
    fixture.assert_original();
    let before = retained(&fixture.root.join("operations"));
    let mut resumed = fixture.recorded();
    assert_eq!(resumed.rewrite(&context, &fixture.capsule, limits()).unwrap(), response);
    assert_eq!(before, retained(&fixture.root.join("operations")));
    rewrite::apply(&fixture.root, &fixture.capsule, &response, limits()).unwrap();
    assert_eq!(fs::read_to_string(fixture.root.join("src/native.rs")).unwrap(), "pub fn translated() -> u8 { 1 }\n");
    assert_eq!(fs::read_to_string(fixture.root.join("tests/frozen.rs")).unwrap(), "frozen verification rules\n");
    assert_eq!(fs::read_to_string(fixture.root.join("Cargo.toml")).unwrap(), "frozen manifest\n");
}

#[test]
fn actual_worker_cannot_read_or_write_protected_state_or_invoke_host_verifier() {
    let fixture = Fixture::new();
    let protected = fixture.root.join("protected");
    fs::create_dir_all(&protected).unwrap();
    let names = ["ledger.toml", "authority.toml", "policy.json", "verdict.toml", "credentials.json"];
    for name in names {
        fs::write(protected.join(name), b"protected state").unwrap();
    }
    let checks = names
        .iter()
        .map(|name| {
            let path = protected.join(name).display().to_string();
            format!(
                "assert!(std::fs::read({path:?}).is_err()); assert!(std::fs::write({path:?}, b\"attack\").is_err());"
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let verifier = fixture.root.join("denials").display().to_string();
    let behavior = format!(
        r#"
{checks}
for path in ["/usr/bin/cargo", "/usr/bin/sm-pipeline", {verifier:?}] {{
    assert!(std::process::Command::new(path).arg("--host-verifier").status().is_err());
}}
for name in ["SM_REWRITE_TEST_SECRET", "SM_PIPELINE_ROOT", "SM_AUTHORITY_TOKEN", "OPENAI_API_KEY"] {{
    assert!(std::env::var_os(name).is_none());
}}
assert_eq!(std::env::var("PATH").unwrap(), "/nonexistent");
assert_eq!(std::env::var("HOME").unwrap(), "/work/tmp/home");
assert_eq!(std::env::var("CARGO_HOME").unwrap(), "/work/tmp/cargo-home");
assert!(std::fs::read("/proc/self/environ").is_err());
assert!(std::fs::read("/trusted-helper").map(|bytes| bytes.is_empty()).unwrap_or(true));
assert!(std::process::Command::new("/trusted-helper").status().is_err());
"#
    );
    let program = fixture.probe("denials", &behavior, &serde_json::to_vec(&fixture.response).unwrap());
    let mut adapter = fixture.installed(program);
    assert_eq!(
        adapter.rewrite(&fixture.context("protected-denial"), &fixture.capsule, limits()).unwrap(),
        fixture.response
    );
    for name in names {
        assert_eq!(fs::read(protected.join(name)).unwrap(), b"protected state");
    }
    assert!(!fixture.root.join("host-verifier-marker").exists());
    fixture.assert_original();
}

#[test]
fn stdout_overflow_and_ignored_temporary_quota_exhaustion_stop_before_mutation() {
    let fixture = Fixture::new();
    let overflow = fixture.probe(
        "stdout_overflow",
        r#"
        let bytes = [b'x'; 4096];
        for _ in 0..16 { let _ = std::io::stdout().write_all(&bytes); }
    "#,
        &serde_json::to_vec(&fixture.response).unwrap(),
    );
    let mut adapter = fixture.installed(overflow);
    let context = fixture.context("stdout-overflow");
    let error = adapter.rewrite(&context, &fixture.capsule, limits()).unwrap_err().to_string();
    assert!(error.contains("budget") || error.contains("quota"), "{error}");
    let before = retained(&fixture.root.join("operations"));
    assert_eq!(adapter.rewrite(&context, &fixture.capsule, limits()).unwrap_err().to_string(), error);
    assert_eq!(before, retained(&fixture.root.join("operations")));
    let quota = fixture.probe(
        "tmp_overflow",
        r#"
        let mut file = std::fs::File::create("/tmp/transient").unwrap();
        let bytes = [b'q'; 4096];
        while file.write_all(&bytes).is_ok() {}
        drop(file);
        let _ = std::fs::remove_file("/tmp/transient");
    "#,
        &serde_json::to_vec(&fixture.response).unwrap(),
    );
    let mut adapter = fixture.installed(quota);
    let started = Instant::now();
    let error = adapter.rewrite(&fixture.context("tmp-overflow"), &fixture.capsule, limits()).unwrap_err().to_string();
    assert!(error.contains("quota"), "{error}");
    assert!(started.elapsed() < Duration::from_secs(5));
    fixture.assert_original();
}

#[test]
fn installed_runtime_host_tools_aliases_and_missing_confinement_refuse_launch() {
    let fixture = Fixture::new();
    let program = fixture.probe("valid", "", &serde_json::to_vec(&fixture.response).unwrap());
    let expected = installed_identity(&program, &run_limits()).unwrap();
    let mut invalid = program.clone();
    invalid.inputs.push(ReadOnlyBind { source: fixture.root.clone(), destination: "/inputs".into() });
    assert!(installed_identity(&invalid, &run_limits()).is_err());
    let mut invalid = program.clone();
    invalid.outputs.push("response.json".into());
    assert!(installed_identity(&invalid, &run_limits()).is_err());
    let mut invalid = program.clone();
    invalid.runtime.push(ReadOnlyBind { source: "/usr".into(), destination: "/usr".into() });
    assert!(installed_identity(&invalid, &run_limits()).is_err());
    let mut invalid = program.clone();
    invalid.arguments = vec!["sm-pipeline".into(), "family-verify".into()];
    assert!(installed_identity(&invalid, &run_limits()).is_err());
    let mut invalid = program.clone();
    invalid.helper = fixture.root.join("missing-helper");
    assert!(
        InstalledRewriteAdapter::new(fixture.root.join("operations"), invalid, run_limits(), expected.clone()).is_err()
    );
    let alias = fixture.root.join("program-alias");
    symlink(&program.executable, &alias).unwrap();
    let mut invalid = program.clone();
    invalid.executable = alias;
    assert!(installed_identity(&invalid, &run_limits()).is_err());
    let hardlink = fixture.root.join("program-hardlink");
    fs::hard_link(&program.executable, &hardlink).unwrap();
    assert!(installed_identity(&program, &run_limits()).is_err());
    fs::remove_file(hardlink).unwrap();
    let mut adapter =
        InstalledRewriteAdapter::new(fixture.root.join("operations"), program.clone(), run_limits(), expected).unwrap();
    fs::set_permissions(&program.executable, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(adapter.rewrite(&fixture.context("changed-program"), &fixture.capsule, limits()).is_err());
    assert!(!fixture.root.join("operations").exists());
    fixture.assert_original();
}

#[test]
fn stale_capsule_and_unknown_or_foreign_responses_never_reach_source_mutation() {
    let fixture = Fixture::new();
    let mut adapter = fixture.recorded();
    let mut wrong = fixture.context("wrong-context");
    wrong.attempt += 1;
    assert!(adapter.rewrite(&wrong, &fixture.capsule, limits()).is_err());
    let mut wrong = fixture.context("wrong-stage");
    wrong.phase = ExecutionPhase::Publication;
    assert!(adapter.rewrite(&wrong, &fixture.capsule, limits()).is_err());
    let mut tampered = fixture.capsule.clone();
    tampered.context.push_str(" tampered");
    assert!(adapter.rewrite(&fixture.context("tampered"), &tampered, limits()).is_err());
    assert!(!fixture.root.join("operations").exists());
    let mut messages = vec![b"not source JSON".to_vec()];
    let mut foreign = fixture.response.clone();
    foreign.files[0].path = "tests/frozen.rs".into();
    messages.push(serde_json::to_vec(&foreign).unwrap());
    let mut stale = fixture.response.clone();
    stale.capsule_id = sha256("another capsule");
    messages.push(serde_json::to_vec(&stale).unwrap());
    for field in ["command", "stage", "authority", "evidence"] {
        let mut unknown = serde_json::to_value(&fixture.response).unwrap();
        unknown[field] = serde_json::json!({"value":"attack"});
        messages.push(serde_json::to_vec(&unknown).unwrap());
    }
    let mut oversize = fixture.response.clone();
    oversize.files[0].content = "x".repeat(4097);
    messages.push(serde_json::to_vec(&oversize).unwrap());
    for (index, message) in messages.into_iter().enumerate() {
        let name = format!("invalid_{index}");
        let program = fixture.probe(&name, "", &message);
        let mut adapter = fixture.installed(program);
        assert!(
            adapter.rewrite(&fixture.context(&name), &fixture.capsule, limits()).is_err(),
            "accepted invalid response {index}"
        );
        fixture.assert_original();
    }
}

#[test]
fn valid_response_cannot_apply_after_an_original_dependency_changes() {
    let fixture = Fixture::new();
    let mut adapter = fixture.recorded();
    let response = adapter.rewrite(&fixture.context("changed-source"), &fixture.capsule, limits()).unwrap();
    fs::write(fixture.root.join("src/dependency.rs"), "changed by another owner\n").unwrap();
    assert!(rewrite::apply(&fixture.root, &fixture.capsule, &response, limits()).is_err());
    assert_eq!(fs::read_to_string(fixture.root.join("src/native.rs")).unwrap(), "pub fn translated() -> u8 { 0 }\n");
    assert_eq!(fs::read_to_string(fixture.root.join("tests/frozen.rs")).unwrap(), "frozen verification rules\n");
}
