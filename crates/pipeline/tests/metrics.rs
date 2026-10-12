//! Complete measurements through the native process/capture interfaces.
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
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
    exec_with_options(root, binary, label, 240, "enabled")
}
fn exec_with_options(root: &Path, binary: &Path, label: &str, timeout: u64, telemetry: &str) -> std::process::Output {
    spawn_exec(root, binary, label, timeout, telemetry).wait_with_output().unwrap()
}
fn spawn_exec(root: &Path, binary: &Path, label: &str, timeout: u64, telemetry: &str) -> Child {
    #[derive(serde::Serialize)]
    struct Request<'a> {
        command: Vec<&'a Path>,
        run: &'a Path,
        label: &'a str,
        root: PathBuf,
        workers: usize,
        timeout: u64,
    }
    let request = Request { command: vec![binary], run: root, label, root: root.join("workers"), workers: 1, timeout };
    let mut child = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
        .arg("exec")
        .env_remove("SM_PIPELINE_METRICS_CONTEXT")
        .env("SM_PIPELINE_METRICS", telemetry)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(toml::to_string(&request).unwrap().as_bytes()).unwrap();
    child
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
    assert!(!exec_with_options(&run.0, &binary, "timeout", 1, "enabled").status.success());
    let metrics = receipt(&run.0.join("timeout.metrics.toml"));
    assert_eq!(metrics["outcome"].as_str(), Some("timed-out"));
    assert_eq!(metrics["processes"][0]["reap_count"].as_integer(), Some(1));
    assert_eq!(metrics["processes"][0]["signal"].as_integer(), Some(9));
    let quick = compile(&run.0, "int main(void){return 0;}\n");
    assert!(exec(&run.0, &quick, "after-timeout").status.success());
}

// A failing teardown control must not leave its deliberately long-lived forks
// running. Keep their /proc identities so cleanup cannot kill a reused PID.
struct Descendant {
    pid: i32,
    start: Option<String>,
}
impl Descendant {
    fn read(path: &Path) -> Self {
        let pid = fs::read_to_string(path).unwrap().trim().parse().unwrap();
        let start = Self::stat(pid).map(|(_, start)| start);
        Self { pid, start }
    }
    fn stat(pid: i32) -> Option<(char, String)> {
        let stat = match fs::read_to_string(format!("/proc/{pid}/stat")) {
            Ok(stat) => stat,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return None,
            Err(error) => panic!("cannot inspect descendant {pid}: {error}"),
        };
        let fields: Vec<_> = stat.rsplit_once(')').unwrap().1.split_whitespace().collect();
        Some((fields[0].chars().next().unwrap(), fields[19].into()))
    }
    fn active(&self) -> bool {
        Self::stat(self.pid)
            .is_some_and(|(state, start)| self.start.as_ref() == Some(&start) && !matches!(state, 'Z' | 'X' | 'x'))
    }
}
impl Drop for Descendant {
    fn drop(&mut self) {
        if self.active() {
            // SAFETY: the test recorded this fork's PID and checked its start
            // identity immediately before sending its cleanup signal.
            unsafe { libc::kill(self.pid, libc::SIGKILL) };
        }
    }
}

fn retained_tree(root: &Path) -> PathBuf {
    compile(
        root,
        r#"
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
static void retain(const char *name) {
  volatile unsigned char *p=malloc(32*1024*1024);
  if(!p) _exit(2);
  for(unsigned i=0;i<32*1024*1024;i+=4096) p[i]=1;
  FILE *pid=fopen(name,"w");
  if(!pid) _exit(3);
  fprintf(pid,"%ld\n",(long)getpid());
  if(fclose(pid)) _exit(4);
  printf("retained %s\n",name);
  fflush(stdout);
  sleep(30);
  free((void*)p);
  _exit(0);
}
int main(void) {
  FILE *pid=fopen("producer.pid","w");
  if(!pid) return 7;
  fprintf(pid,"%ld\n",(long)getpid());
  if(fclose(pid)) return 8;
  pid_t child=fork();
  if(child<0) return 5;
  if(child==0) {
    pid_t grandchild=fork();
    if(grandchild<0) _exit(6);
    if(grandchild==0) retain("grandchild.pid");
    retain("child.pid");
  }
  sleep(30);
  return 0;
}
"#,
    )
}
fn assert_descendants_stopped(descendants: &[Descendant]) {
    // SIGKILL can still be completing on another core when the owned direct
    // child is reaped. A short bounded wait permits that kernel transition.
    let stopped = Instant::now();
    while descendants.iter().any(Descendant::active) && stopped.elapsed() < Duration::from_secs(1) {
        std::thread::sleep(Duration::from_millis(10));
    }
    for descendant in descendants {
        assert!(!descendant.active(), "descendant {} survived public exec teardown", descendant.pid);
    }
}

#[test]
fn timeout_stops_child_and_grandchild_holding_output_and_allocated_memory() {
    let run = Run::new();
    let binary = retained_tree(&run.0);
    let output = exec_with_options(&run.0, &binary, "tree-timeout", 1, "enabled");
    let descendants = [Descendant::read(&run.0.join("child.pid")), Descendant::read(&run.0.join("grandchild.pid"))];
    let _producer = Descendant::read(&run.0.join("producer.pid"));
    assert!(!output.status.success(), "timeout unexpectedly succeeded");
    assert_descendants_stopped(&descendants);
    let stdout = fs::read_to_string(run.0.join("tree-timeout.stdout")).unwrap();
    assert!(stdout.contains("retained child.pid") && stdout.contains("retained grandchild.pid"));
    let metrics = receipt(&run.0.join("tree-timeout.metrics.toml"));
    assert_eq!(metrics["outcome"].as_str(), Some("timed-out"));
    let processes = metrics["processes"].as_array().unwrap();
    assert_eq!(processes.len(), 1, "descendants must not receive duplicate wait ownership");
    assert_eq!(processes[0]["reap_count"].as_integer(), Some(1));
    assert_eq!(processes[0]["signal"].as_integer(), Some(libc::SIGKILL.into()));
    assert!(metrics["resources"]["aggregate_sampled_rss_bytes"]["value"].as_integer().unwrap() >= 60 * 1024 * 1024);
    let quick = compile(&run.0, "int main(void){return 0;}\n");
    assert!(exec(&run.0, &quick, "after-tree-timeout").status.success());
}

#[test]
fn sigterm_records_interruption_stops_the_process_tree_and_releases_worker_capacity() {
    let run = Run::new();
    let binary = retained_tree(&run.0);
    let mut pipeline = spawn_exec(&run.0, &binary, "interrupted", 30, "enabled");
    let ready = Instant::now();
    let published_pid =
        |name| fs::read_to_string(run.0.join(name)).ok().is_some_and(|text| text.trim().parse::<i32>().is_ok());
    while !(published_pid("child.pid") && published_pid("grandchild.pid")) && ready.elapsed() < Duration::from_secs(5) {
        std::thread::sleep(Duration::from_millis(10));
    }
    if !(published_pid("child.pid") && published_pid("grandchild.pid")) {
        let producer = run.0.join("producer.pid");
        if published_pid("producer.pid") {
            let pid = fs::read_to_string(producer).unwrap().trim().parse::<i32>().unwrap();
            // SAFETY: this test's still-owned producer group has not been reaped.
            unsafe { libc::kill(-pid, libc::SIGKILL) };
        }
        pipeline.kill().unwrap();
        pipeline.wait().unwrap();
        panic!("process-tree control failed to publish both descendant PIDs");
    }
    let descendants = [Descendant::read(&run.0.join("child.pid")), Descendant::read(&run.0.join("grandchild.pid"))];
    let _producer = Descendant::read(&run.0.join("producer.pid"));
    // SAFETY: pipeline remains owned and unreaped; its live PID cannot be reused.
    assert_eq!(unsafe { libc::kill(pipeline.id() as i32, libc::SIGTERM) }, 0);
    let interrupted = Instant::now();
    let status = loop {
        if let Some(status) = pipeline.try_wait().unwrap() {
            break status;
        }
        if interrupted.elapsed() > Duration::from_secs(5) {
            pipeline.kill().unwrap();
            pipeline.wait().unwrap();
            panic!("public exec did not finish after SIGTERM");
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    assert!(!status.success());
    assert_descendants_stopped(&descendants);
    let metrics = receipt(&run.0.join("interrupted.metrics.toml"));
    assert_eq!(metrics["outcome"].as_str(), Some("interrupted"));
    assert!(metrics["error"].as_str().unwrap().contains("interrupted by signal"));
    let processes = metrics["processes"].as_array().unwrap();
    assert_eq!(processes.len(), 1);
    assert_eq!(processes[0]["reap_count"].as_integer(), Some(1));
    assert_eq!(processes[0]["signal"].as_integer(), Some(libc::SIGKILL.into()));
    let quick = compile(&run.0, "int main(void){return 0;}\n");
    assert!(exec(&run.0, &quick, "after-interruption").status.success());
}

#[test]
fn inherited_clock_sigterm_still_stops_descendants_and_releases_capacity() {
    use pipeline_model::metrics::{Clock, ContextReceipt};
    let run = Run::new();
    let binary = retained_tree(&run.0);
    let inherited = run.0.join("parent.context.toml");
    fs::write(
        &inherited,
        toml::to_string(&ContextReceipt {
            run_id: "benchmark-parent".into(),
            scope_id: "live-child".into(),
            parent_scope: Some("benchmark-parent".into()),
            clock: Clock::new().unwrap(),
            enabled: true,
            path: run.0.join("inherited.metrics.toml"),
        })
        .unwrap(),
    )
    .unwrap();
    let request = toml::toml! {
        command=[(binary.to_str().unwrap())]
        run=(run.0.to_str().unwrap())
        label="inherited"
        root=(run.0.join("workers").to_str().unwrap().to_owned())
        workers=1
        timeout=30
    };
    let mut pipeline = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
        .arg("exec")
        .env("SM_PIPELINE_METRICS_CONTEXT", &inherited)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    pipeline.stdin.take().unwrap().write_all(toml::to_string(&request).unwrap().as_bytes()).unwrap();
    let started = Instant::now();
    while !run.0.join("grandchild.pid").is_file() && started.elapsed() < Duration::from_secs(5) {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(run.0.join("grandchild.pid").is_file());
    let descendants = [Descendant::read(&run.0.join("child.pid")), Descendant::read(&run.0.join("grandchild.pid"))];
    let _producer = Descendant::read(&run.0.join("producer.pid"));
    // SAFETY: the test owns this still-running native orchestration process.
    assert_eq!(unsafe { libc::kill(pipeline.id() as i32, libc::SIGTERM) }, 0);
    let output = pipeline.wait_with_output().unwrap();
    assert!(!output.status.success());
    assert_descendants_stopped(&descendants);
    let metrics = receipt(&run.0.join("inherited.metrics.toml"));
    assert_eq!(metrics["outcome"].as_str(), Some("interrupted"));
    assert_eq!(metrics["parent_scope"].as_str(), Some("benchmark-parent"));
    let quick = compile(&run.0, "int main(void){return 0;}\n");
    assert!(exec(&run.0, &quick, "after-inherited-interruption").status.success());
}

#[test]
fn disabled_telemetry_preserves_execution_and_one_owned_wait4_record() {
    let run = Run::new();
    let binary = compile(
        &run.0,
        "#include <stdio.h>\n#include <unistd.h>\nint main(void){puts(\"same output\");fputs(\"same diagnostics\\n\",stderr);usleep(150000);return 0;}\n",
    );
    let enabled = exec_with_options(&run.0, &binary, "enabled", 240, "enabled");
    let disabled = exec_with_options(&run.0, &binary, "disabled", 240, "disabled");
    assert!(enabled.status.success() && disabled.status.success());
    for suffix in ["stdout", "stderr"] {
        assert_eq!(
            fs::read(run.0.join(format!("enabled.{suffix}"))).unwrap(),
            fs::read(run.0.join(format!("disabled.{suffix}"))).unwrap()
        );
    }
    let metrics = receipt(&run.0.join("disabled.metrics.toml"));
    assert_eq!(metrics["enabled"].as_bool(), Some(false));
    assert_eq!(metrics["outcome"].as_str(), Some("success"));
    assert!(metrics["stages"].as_array().unwrap().is_empty());
    assert_eq!(metrics["resources"]["rss_samples"].as_integer(), Some(0));
    assert_eq!(metrics["resources"]["disk_samples"].as_integer(), Some(0));
    for field in [
        "aggregate_sampled_rss_bytes",
        "staging_allocated_bytes",
        "private_store_allocated_bytes",
        "combined_allocated_bytes",
        "minimum_free_bytes",
    ] {
        assert_eq!(metrics["resources"][field]["state"].as_str(), Some("unavailable"));
        assert!(metrics["resources"][field].get("value").is_none());
    }
    let processes = metrics["processes"].as_array().unwrap();
    assert_eq!(processes.len(), 1);
    assert_eq!(processes[0]["reap_count"].as_integer(), Some(1));
    assert_eq!(processes[0]["exit_code"].as_integer(), Some(0));
    assert!(processes[0]["accounting"].as_str().unwrap().contains("wait4"));
    for field in ["user_cpu_ns", "system_cpu_ns", "peak_rss_bytes"] {
        assert_eq!(processes[0][field]["state"].as_str(), Some("measured"));
    }
    let enabled_metrics = receipt(&run.0.join("enabled.metrics.toml"));
    assert_eq!(enabled_metrics["enabled"].as_bool(), Some(true));
    assert!(!enabled_metrics["stages"].as_array().unwrap().is_empty());
    assert_eq!(enabled_metrics["resources"]["aggregate_sampled_rss_bytes"]["state"].as_str(), Some("measured"));
}
