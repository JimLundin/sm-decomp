//! Complete measurements through the native process/capture interfaces.
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

struct Run(PathBuf);
impl Run {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "sm-metrics-control-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Run {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn compile(root: &Path, source: &str) -> PathBuf {
    fs::write(root.join("control.c"), source).unwrap();
    let binary = root.join("control");
    assert!(
        Command::new("cc").args(["-O2"]).arg(root.join("control.c")).arg("-o").arg(&binary).status().unwrap().success()
    );
    binary
}
fn exec(root: &Path, binary: &Path, label: &str) -> std::process::Output {
    #[derive(serde::Serialize)]
    struct Request<'a> {
        command: Vec<&'a Path>,
        run: &'a Path,
        label: &'a str,
        root: PathBuf,
        workers: usize,
    }
    let request = Request { command: vec![binary], run: root, label, root: root.join("workers"), workers: 1 };
    let mut child = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
        .arg("exec")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(toml::to_string(&request).unwrap().as_bytes()).unwrap();
    child.wait_with_output().unwrap()
}
fn receipt(path: &Path) -> toml::Value {
    toml::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn executed_process_reports_its_cpu_memory_and_complete_monotonic_interval() {
    let run = Run::new();
    let binary = compile(
        &run.0,
        "#include <stdlib.h>\n#include <time.h>\n#include <unistd.h>\nint main(void){volatile unsigned char *p=malloc(32*1024*1024);for(unsigned i=0;i<32*1024*1024;i+=4096)p[i]=1;clock_t begin=clock();volatile unsigned long n=0;while(clock()-begin<CLOCKS_PER_SEC/8)n++;usleep(150000);free((void*)p);return n==0;}\n",
    );
    let output = exec(&run.0, &binary, "cpu-memory");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let metrics = receipt(&run.0.join("cpu-memory.metrics.toml"));
    assert_eq!(metrics["format"].as_str(), Some("pipeline-metrics-v1"));
    assert_eq!(metrics["outcome"].as_str(), Some("success"));
    assert_eq!(metrics["clock"]["name"].as_str(), Some("CLOCK_MONOTONIC"));
    assert!(metrics["end_ns"].as_integer().unwrap() > metrics["start_ns"].as_integer().unwrap());
    let process = &metrics["processes"].as_array().unwrap()[0];
    assert!(
        process["user_cpu_ns"]["value"].as_integer().unwrap() + process["system_cpu_ns"]["value"].as_integer().unwrap()
            >= 90_000_000
    );
    assert!(process["peak_rss_bytes"]["value"].as_integer().unwrap() >= 30 * 1024 * 1024);
    assert_eq!(process["reap_count"].as_integer(), Some(1));
    assert!(metrics["resources"]["aggregate_sampled_rss_bytes"]["value"].as_integer().unwrap() >= 30 * 1024 * 1024);
}

#[test]
fn timed_out_process_is_accounted_and_releases_worker_capacity() {
    let run = Run::new();
    let binary = compile(&run.0, "#include <unistd.h>\nint main(void){sleep(60);return 0;}\n");
    let request = format!(
        "command = [\"{}\"]\nrun = \"{}\"\nlabel = \"timeout\"\nroot = \"{}\"\nworkers = 1\ntimeout = 1\n",
        binary.display(),
        run.0.display(),
        run.0.join("workers").display()
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
        .arg("exec")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(request.as_bytes()).unwrap();
    assert!(!child.wait_with_output().unwrap().status.success());
    let metrics = receipt(&run.0.join("timeout.metrics.toml"));
    assert_eq!(metrics["outcome"].as_str(), Some("timed-out"));
    assert_eq!(metrics["processes"][0]["reap_count"].as_integer(), Some(1));
    assert_eq!(metrics["processes"][0]["signal"].as_integer(), Some(9));
    let quick = compile(&run.0, "int main(void){return 0;}\n");
    assert!(exec(&run.0, &quick, "after-timeout").status.success());
}

#[test]
fn unrelated_monotonic_domain_is_rejected_before_capture_readiness() {
    let run = Run::new();
    let reader = std::env::var_os("SM_EVIDENCE_TEST_BINARY")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp/sm-pipeline-upgrade-metrics-target/debug/sm-evidence"));
    let clock = pipeline_model::metrics::Clock::new().unwrap();
    let context = pipeline_model::metrics::ContextReceipt {
        run_id: "foreign".into(),
        scope_id: "foreign-child".into(),
        parent_scope: None,
        clock: pipeline_model::metrics::Clock { domain: "unrelated-host-clock".into(), ..clock },
        enabled: true,
        path: run.0.join("child.metrics.toml"),
    };
    fs::write(run.0.join("context.toml"), toml::to_string(&context).unwrap()).unwrap();
    let output = Command::new(reader)
        .arg("inventory")
        .arg(run.0.join("store"))
        .arg("0".repeat(64))
        .env("SM_PIPELINE_METRICS_CONTEXT", run.0.join("context.toml"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("clock domain"));
}
