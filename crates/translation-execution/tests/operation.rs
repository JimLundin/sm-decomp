#![cfg(all(target_os = "linux", target_arch = "x86_64"))]
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};
use translation_execution::{
    ExecutionPending,
    confinement::{Program, RunLimits, helper_runtime},
    model::*,
    operation,
};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture {
    root: PathBuf,
    program: Program,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "translation-operation-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let source = root.join("probe.c");
        fs::write(&source, r#"
#define _GNU_SOURCE
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
int main(int argc,char **argv) {
 if(argc!=2)return 2;
 if(!strcmp(argv[1],"success")){puts("only-once");int f=open("result.bin",O_WRONLY|O_CREAT,0600);return f<0||write(f,"sealed",6)!=6?3:0;}
 if(!strcmp(argv[1],"failure")){for(int i=0;i<12000;i++)putchar('f');puts("rejected");return 7;}
 if(!strcmp(argv[1],"quota")){char b[4096];memset(b,'q',4096);int f=open("tmp/full",O_WRONLY|O_CREAT,0600);while(write(f,b,4096)>0){}close(f);unlink("tmp/full");return 0;}
 if(!strcmp(argv[1],"sleep")){usleep(600000);puts("recovered-original");return 0;}
 if(!strcmp(argv[1],"long")){if(!fork()){setsid();sleep(30);}sleep(30);return 0;}
 return 8;
}
"#).unwrap();
        let executable = root.join("probe");
        assert!(
            Command::new("/usr/bin/cc")
                .args(["-static", "-O2"])
                .arg(&source)
                .arg("-o")
                .arg(&executable)
                .status()
                .unwrap()
                .success()
        );
        Self {
            root,
            program: Program {
                executable,
                arguments: vec!["success".into()],
                inputs: vec![],
                runtime: helper_runtime().unwrap(),
                helper: env!("CARGO_BIN_EXE_sm-translation-sandbox").into(),
                outputs: vec!["result.bin".into()],
            },
        }
    }
    fn operation_root(&self) -> PathBuf {
        self.root.join("operations")
    }
    fn context(&self, operation: &str) -> OperationContext {
        context(operation)
    }
    fn run(&self, context: &OperationContext) -> anyhow::Result<operation::OperationResult> {
        operation::run(&self.operation_root(), context, &self.program, &limits(), b"bounded-input")
    }
    fn mode(&mut self, mode: &str) {
        self.program.arguments = vec![mode.into()];
        self.program.outputs.clear();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if std::env::var_os("SM_OPERATION_TEST_KEEP").is_some_and(|value| value == "1") {
            eprintln!("Retained operation control: {}", self.root.display());
            return;
        }
        let _ = fs::remove_dir_all(&self.root);
    }
}
fn context(name: &str) -> OperationContext {
    OperationContext {
        run_id: sha256("run"),
        attempt: 1,
        policy_sha256: sha256("policy"),
        operation_key: sha256(name),
        phase: ExecutionPhase::Candidate,
    }
}
fn limits() -> RunLimits {
    RunLimits {
        timeout: Duration::from_secs(5),
        writable_bytes: 128 * 1024,
        stdout_bytes: 16 * 1024,
        stderr_bytes: 8192,
        stdin_bytes: 8192,
    }
}

#[test]
fn sealed_process_output_lost_response_retry_and_checkout_retirement() {
    let fixture = Fixture::new();
    let ctx = fixture.context("success");
    let first = fixture.run(&ctx).unwrap();
    assert!(first.passed, "{}", first.diagnostic);
    assert_eq!(first.status_raw, Some(0));
    assert_eq!(first.stdout, b"only-once\n");
    assert_eq!(first.outputs[Path::new("result.bin")], b"sealed");
    let paths = operation::references(&fixture.operation_root(), &ctx).unwrap();
    let original_bytes = paths.iter().map(|r| (r.path.clone(), fs::read(&r.path).unwrap())).collect::<Vec<_>>();
    assert_eq!(fixture.run(&ctx).unwrap(), first);
    for (path, bytes) in &original_bytes {
        assert_eq!(fs::read(path).unwrap(), *bytes);
    }
    // Independent retained authentication survives the producer's retirement;
    // no new gate or recapture is allocated by resume.
    fs::remove_file(&fixture.program.executable).unwrap();
    fs::remove_file(fixture.root.join("probe.c")).unwrap();
    assert_eq!(fixture.run(&ctx).unwrap(), first);
    assert_eq!(operation::authenticate(&fixture.operation_root(), &ctx).unwrap(), first);
}

#[cfg(feature = "test-support")]
#[test]
fn complete_operation_admission_reads_each_retained_byte_once() {
    let fixture = Fixture::new();
    let context = fixture.context("single-admission");
    let produced = fixture.run(&context).unwrap();
    assert!(produced.passed, "{}", produced.diagnostic);
    let root = fixture.operation_root();
    fs::write(
        fixture.root.join("admission-request.json"),
        serde_json::to_vec(&serde_json::json!({
            "root": root,
            "program": fixture.program,
            "context": context,
            "limits": limits(),
        }))
        .unwrap(),
    )
    .unwrap();
    let ((result, result_reference, references), read_bytes) =
        translation_execution::retention::testing::read_bytes(|| {
            let result =
                operation::authenticate_request(&root, &context, &fixture.program, &limits(), b"bounded-input")
                    .unwrap();
            let result_reference = operation::reference(&root, &context).unwrap();
            let references = operation::references(&root, &context).unwrap();
            (result, result_reference, references)
        });
    assert_eq!(result, produced);
    assert!(references.contains(&result_reference));
    let expected_bytes = references.iter().map(|reference| reference.bytes).sum::<u64>();
    assert_eq!(read_bytes, expected_bytes, "complete operation admission reread retained evidence");
}

#[test]
fn failed_gate_has_one_bounded_stop_record_and_no_retained_large_output() {
    let mut fixture = Fixture::new();
    fixture.mode("failure");
    let ctx = fixture.context("failure");
    let first = fixture.run(&ctx).unwrap();
    assert!(!first.passed);
    assert!(first.diagnostic.len() <= operation::MAX_DIAGNOSTIC_BYTES);
    assert!(first.stdout.is_empty() && first.stderr.is_empty() && first.outputs.is_empty());
    assert!(!operation::directory(&fixture.operation_root(), &ctx).unwrap().join("output").exists());
    assert_eq!(fixture.run(&ctx).unwrap(), first);
    let mut changed = fixture.program.clone();
    changed.arguments = vec!["sleep".into()];
    assert!(
        operation::run(&fixture.operation_root(), &ctx, &changed, &limits(), b"bounded-input")
            .unwrap_err()
            .to_string()
            .contains("changed request")
    );
}

#[test]
fn expired_outer_deadline_seals_no_launch_and_retry_cannot_extend_it() {
    let fixture = Fixture::new();
    let context = fixture.context("outer-deadline");
    let result =
        operation::run_until(&fixture.operation_root(), &context, &fixture.program, &limits(), b"bounded-input", 0)
            .unwrap();
    assert!(!result.passed);
    assert_eq!(result.status_raw, None);
    assert!(result.diagnostic.contains("deadline"));
    assert!(!operation::directory(&fixture.operation_root(), &context).unwrap().join("runner.json").exists());
    assert!(!fixture.operation_root().join("installed-runtime").exists());
    assert_eq!(
        operation::run_until(
            &fixture.operation_root(),
            &context,
            &fixture.program,
            &limits(),
            b"bounded-input",
            u64::MAX,
        )
        .unwrap(),
        result
    );
    assert_eq!(
        operation::authenticate_request(
            &fixture.operation_root(),
            &context,
            &fixture.program,
            &limits(),
            b"bounded-input",
        )
        .unwrap(),
        result
    );
}

#[test]
fn input_reservation_and_hashing_obey_the_original_stage_deadline() {
    let mut fixture = Fixture::new();
    fixture.program.executable = fixture.root.join("slow-input");
    fs::File::create(&fixture.program.executable).unwrap().set_len(64 * 1024 * 1024).unwrap();
    fixture.program.runtime.clear();
    let started = Instant::now();
    let refused = operation::reserve_bytes_until(&fixture.program, &limits(), 0, 0).unwrap_err();
    assert!(refused.to_string().contains("deadline"));
    assert!(started.elapsed() < Duration::from_secs(1) && !fixture.operation_root().exists());
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64;
    let ctx = fixture.context("input-hash-deadline");
    let started = Instant::now();
    let failed =
        operation::run_until(&fixture.operation_root(), &ctx, &fixture.program, &limits(), &[], now + 15).unwrap();
    assert!(!failed.passed && failed.status_raw.is_none() && failed.diagnostic.contains("deadline"));
    assert!(started.elapsed() < Duration::from_secs(1), "input hashing ignored the stage deadline");
    assert!(!operation::directory(&fixture.operation_root(), &ctx).unwrap().join("runner.json").exists());
    assert_eq!(
        operation::run_until(&fixture.operation_root(), &ctx, &fixture.program, &limits(), &[], u64::MAX).unwrap(),
        failed
    );
    assert_eq!(operation::authenticate(&fixture.operation_root(), &ctx).unwrap(), failed);
}

#[test]
fn reservation_covers_measured_retained_closure_before_any_output_is_produced() {
    let fixture = Fixture::new();
    let reservation = operation::reserve_bytes(&fixture.program, &limits(), 13).unwrap();
    let ctx = fixture.context("reserved");
    assert!(fixture.run(&ctx).unwrap().passed);
    let references = operation::references(&fixture.operation_root(), &ctx).unwrap();
    let bytes = references.iter().map(|r| r.bytes).sum::<u64>();
    assert!(bytes <= reservation, "{bytes} > {reservation}");
}

#[test]
fn actual_ignored_quota_overflow_seals_failure_and_never_accepts() {
    let mut fixture = Fixture::new();
    fixture.mode("quota");
    let ctx = fixture.context("quota");
    let first = fixture.run(&ctx).unwrap();
    assert!(!first.passed && first.diagnostic.contains("quota"), "{}", first.diagnostic);
    assert_eq!(fixture.run(&ctx).unwrap(), first);
    assert!(first.diagnostic.len() <= 4096);
}

#[test]
fn output_and_input_manifest_tampering_rejects_at_independent_authentication() {
    let fixture = Fixture::new();
    let ctx = fixture.context("tamper");
    let output_result = fixture.run(&ctx).unwrap();
    assert!(output_result.passed, "{}", output_result.diagnostic);
    let references = operation::references(&fixture.operation_root(), &ctx).unwrap();
    let output = references.iter().find(|r| r.path.file_name().unwrap() == "stdout.bin").unwrap();
    fs::set_permissions(&output.path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&output.path, b"forged\n").unwrap();
    assert!(operation::authenticate(&fixture.operation_root(), &ctx).is_err());
    fs::write(&output.path, b"only-once\n").unwrap();
    assert!(operation::authenticate(&fixture.operation_root(), &ctx).is_ok());
    let identity = references.iter().find(|r| r.path.parent().unwrap().ends_with("installed-runtime")).unwrap();
    let original = fs::read(&identity.path).unwrap();
    fs::set_permissions(&identity.path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&identity.path, b"forged identity").unwrap();
    assert!(operation::authenticate(&fixture.operation_root(), &ctx).is_err());
    fs::write(&identity.path, original).unwrap();
    assert!(operation::authenticate(&fixture.operation_root(), &ctx).is_ok());
    let intent = references.iter().find(|r| r.path.file_name().unwrap() == "intent.json").unwrap();
    fs::set_permissions(&intent.path, fs::Permissions::from_mode(0o600)).unwrap();
    let mut altered: serde_json::Value = serde_json::from_slice(&fs::read(&intent.path).unwrap()).unwrap();
    altered["identities"][0]["sha256"] = serde_json::Value::String("f".repeat(64));
    fs::write(&intent.path, serde_json::to_vec(&altered).unwrap()).unwrap();
    assert!(
        operation::authenticate(&fixture.operation_root(), &ctx).is_err(),
        "input identity mutation must reject original result"
    );
}

#[test]
fn repeated_fixtures_reuse_inputs_but_same_size_mtime_edits_and_links_invalidate() {
    let mut fixture = Fixture::new();
    let shared = fixture.root.join("shared-input");
    fs::create_dir(&shared).unwrap();
    let payload = shared.join("payload");
    fs::write(&payload, b"before").unwrap();
    let modified = payload.metadata().unwrap().modified().unwrap();
    fixture.program.inputs.push(translation_execution::confinement::ReadOnlyBind {
        source: shared.clone(),
        destination: "/inputs/shared".into(),
    });
    let first = fixture.context("cached-first");
    assert!(fixture.run(&first).unwrap().passed);
    let identity = |context: &OperationContext| {
        operation::references(&fixture.operation_root(), context)
            .unwrap()
            .into_iter()
            .find(|reference| {
                if !reference.path.parent().unwrap().ends_with("installed-runtime") {
                    return false;
                }
                let value: serde_json::Value = serde_json::from_slice(&fs::read(&reference.path).unwrap()).unwrap();
                value["source"].as_str() == Some(shared.to_str().unwrap())
            })
            .unwrap()
    };
    let original = identity(&first);
    let original_modified = original.path.metadata().unwrap().modified().unwrap();
    let second = fixture.context("cached-second");
    assert!(fixture.run(&second).unwrap().passed);
    assert_eq!(identity(&second), original);
    assert_eq!(original.path.metadata().unwrap().modified().unwrap(), original_modified);

    fs::write(&payload, b"after!").unwrap();
    fs::File::options()
        .write(true)
        .open(&payload)
        .unwrap()
        .set_times(fs::FileTimes::new().set_modified(modified))
        .unwrap();
    let changed = fixture.context("cached-changed");
    assert!(fixture.run(&changed).unwrap().passed);
    assert_ne!(identity(&changed), original, "content edit with restored mtime reused a stale identity");
    assert!(operation::authenticate(&fixture.operation_root(), &first).unwrap().passed);

    std::os::unix::fs::symlink("/etc/passwd", shared.join("alias")).unwrap();
    let refused = fixture.context("cached-link");
    let result = fixture.run(&refused).unwrap();
    assert!(!result.passed && result.diagnostic.contains("symlink"), "{}", result.diagnostic);
    assert!(!operation::directory(&fixture.operation_root(), &refused).unwrap().join("runner.json").exists());
}

// A real second controller process enters the public interface. Killing this
// process after sandbox start simulates a lost parent while its runner lives.
#[test]
fn child_controller_entry() {
    let Some(request) = std::env::var_os("SM_OPERATION_TEST_REQUEST") else {
        return;
    };
    let value: serde_json::Value = serde_json::from_slice(&fs::read(request).unwrap()).unwrap();
    let program: Program = serde_json::from_value(value["program"].clone()).unwrap();
    let context: OperationContext = serde_json::from_value(value["context"].clone()).unwrap();
    let limits: RunLimits = serde_json::from_value(value["limits"].clone()).unwrap();
    let root = PathBuf::from(value["root"].as_str().unwrap());
    let result = operation::run(&root, &context, &program, &limits, b"bounded-input").unwrap();
    assert!(result.passed, "{}", result.diagnostic);
}

fn launch_controller(fixture: &Fixture, context: &OperationContext) -> std::process::Child {
    let request = fixture.root.join("child-request.json");
    fs::write(&request, serde_json::to_vec(&serde_json::json!({"root":fixture.operation_root(),"program":fixture.program,"context":context,"limits":limits()})).unwrap()).unwrap();
    Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "child_controller_entry", "--nocapture"])
        .env("SM_OPERATION_TEST_REQUEST", &request)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap()
}
fn wait_for(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !path.exists() {
        assert!(Instant::now() < deadline, "{}", path.display());
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn parent_crash_retains_live_writer_and_recovers_original_output() {
    let mut fixture = Fixture::new();
    fixture.mode("sleep");
    let ctx = fixture.context("crash");
    let mut controller = launch_controller(&fixture, &ctx);
    let directory = operation::directory(&fixture.operation_root(), &ctx).unwrap();
    wait_for(&directory.join("sandbox.json"));
    let original_start = fs::read(directory.join("runner.json")).unwrap();
    controller.kill().unwrap();
    controller.wait().unwrap();
    let error = fixture.run(&ctx).unwrap_err();
    assert!(error.downcast_ref::<ExecutionPending>().is_some(), "{error:#}");
    wait_for(&directory.join("result.json"));
    let deadline = Instant::now() + Duration::from_secs(2);
    let output = loop {
        match fixture.run(&ctx) {
            Ok(result) => break result,
            Err(error) if error.downcast_ref::<ExecutionPending>().is_some() && Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(5))
            }
            Err(error) => panic!("{error:#}"),
        }
    };
    assert!(output.passed, "{}", output.diagnostic);
    assert_eq!(output.stdout, b"recovered-original\n");
    assert_eq!(fs::read(directory.join("runner.json")).unwrap(), original_start);
}

#[test]
fn stopping_a_crashed_controller_runner_tears_down_descendants_and_never_relaunches() {
    let mut fixture = Fixture::new();
    fixture.mode("long");
    let ctx = fixture.context("stop");
    let mut controller = launch_controller(&fixture, &ctx);
    let directory = operation::directory(&fixture.operation_root(), &ctx).unwrap();
    wait_for(&directory.join("sandbox.json"));
    controller.kill().unwrap();
    controller.wait().unwrap();
    let started = Instant::now();
    let result = operation::stop(&fixture.operation_root(), &ctx).unwrap();
    assert!(!result.passed, "stop accepted writer");
    assert!(started.elapsed() < Duration::from_secs(3));
    assert_eq!(fixture.run(&ctx).unwrap(), result);
}

#[test]
fn dead_runner_partial_result_is_retired_without_duplicate_gate() {
    let fixture = Fixture::new();
    let ctx = fixture.context("partial");
    let output = fixture.run(&ctx).unwrap();
    assert!(output.passed);
    let directory = operation::directory(&fixture.operation_root(), &ctx).unwrap();
    // Keep the real intent and start identities, then expose an interrupted
    // result publication with original controller-owned partial members.
    fs::rename(directory.join("result.json"), directory.join("result.pending")).unwrap();
    let recovered = fixture.run(&ctx).unwrap();
    assert!(!recovered.passed && recovered.diagnostic.contains("no gate was rerun"));
    assert!(recovered.stdout.is_empty());
    assert!(!directory.join("output/stdout.bin").exists());
    assert!(directory.join("interrupted-output.json").is_file());
    assert_eq!(fixture.run(&ctx).unwrap(), recovered);
}
