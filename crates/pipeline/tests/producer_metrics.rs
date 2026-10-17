//! Real text producers retain exact observations while reporting bounded flush work.
use evidence::{Pointer, Provenance};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

struct Run(PathBuf);
impl Run {
    fn new() -> Self {
        let root = std::env::var_os("SM_PRODUCER_TEST_ROOT").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
        let path = root.join(format!(
            "producer-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn producer(&self, extra: &str, main: &str) -> PathBuf {
        let source = format!(
            "#define _GNU_SOURCE\n#include <stdio.h>\n#include <stdlib.h>\n#include <stdint.h>\n#include <unistd.h>\n#include <errno.h>\n{extra}\n{}\n{main}\n",
            include_str!("../src/effect_writer.c.in")
        );
        fs::write(self.0.join("producer.c"), source).unwrap();
        let binary = self.0.join("producer");
        let mut compile = Command::new("cc");
        compile.args(["-std=c11", "-O2", "-Wall", "-Wextra", "-Werror", "-pthread"]);
        if main.contains("__wrap_write") {
            compile.arg("-Wl,--wrap=write");
        }
        if main.contains("__wrap_clock_gettime") {
            compile.arg("-Wl,--wrap=clock_gettime");
        }
        let output = compile.arg(self.0.join("producer.c")).arg("-o").arg(&binary).output().unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        binary
    }
}

#[test]
fn failed_effect_destination_records_failure_even_when_open_never_succeeds() {
    let run = Run::new();
    let producer = run.producer(
        "",
        r#"
int main(void) {
 port_effects=PortEffectsOpen();if(!port_effects)return 2;
 PortEffect(port_effects,"apu %u %u %u\n",0,3,255);
 return PortEffectsClose()?2:0;
}
"#,
    );
    let path = run.0.join("metrics.toml");
    let status = Command::new(producer)
        .env("SM_EVIDENCE_PIPE", run.0.join("missing/effects.pipe"))
        .env("SM_PIPELINE_METRICS", "enabled")
        .env("SM_PIPELINE_METRICS_PRODUCER_PATH", &path)
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(2));
    let receipt: Receipt = toml::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(receipt.outcome, "failed");
    assert_eq!(receipt.transport_bytes, 0);
}

#[test]
fn nonzero_text_producer_cannot_publish_an_accepted_capture() {
    let run = Run::new();
    let producer = run.producer(
        "",
        r#"
int main(void) {
 port_effects=PortEffectsOpen();if(!port_effects)return 2;
 PortEffect(port_effects,"apu %u %u %u\n",0,3,255);
 if(PortEffectsClose())return 2;
 return 7;
}
"#,
    );
    let directory = run.0.join("subject");
    fs::create_dir(&directory).unwrap();
    fs::copy(reader(), run.0.join("sm-evidence")).unwrap();
    let hash = |path: &Path| format!("{:x}", Sha256::digest(fs::read(path).unwrap()));
    #[derive(Serialize)]
    struct Capture {
        directory: PathBuf,
        reader: PathBuf,
        command: Vec<String>,
        provenance: Provenance,
        root: PathBuf,
        workers: usize,
        timeout: f64,
    }
    let request = Capture {
        directory: directory.clone(),
        reader: run.0.join("sm-evidence"),
        command: vec![producer.to_string_lossy().into_owned()],
        provenance: Provenance {
            binary_sha256: Some(hash(&producer)),
            reader_sha256: Some(hash(&run.0.join("sm-evidence"))),
            ..Default::default()
        },
        root: run.0.join("workers"),
        workers: 2,
        timeout: 5.0,
    };
    let mut child = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
        .arg("packed-capture")
        .env("SM_PIPELINE_METRICS", "enabled")
        .env("SM_PIPELINE_METRICS_PRODUCER_PATH", directory.join("producer-metrics.toml"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    use std::io::Write;
    child.stdin.take().unwrap().write_all(toml::to_string(&request).unwrap().as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(!output.status.success(), "{}", String::from_utf8_lossy(&output.stdout));
    assert!(!directory.join("evidence.toml").exists());
    assert!(directory.join(".evidence-stage").exists());
    assert!(directory.join("replay.stderr").exists());
}

#[test]
fn interrupted_and_short_system_writes_restore_every_text_byte() {
    let run = Run::new();
    let producer = run.producer(
        "",
        r#"
ssize_t __real_write(int,const void *,size_t);
ssize_t __wrap_write(int fd,const void *data,size_t size) {
 static unsigned calls;
 if(size>65536){errno=EFBIG;return -1;}
 if(calls++==0){errno=EINTR;return -1;}
 return __real_write(fd,data,size>17?17:size);
}
int main(void) {
 port_effects=PortEffectsOpen();if(!port_effects)return 2;
 char giant[140001];for(size_t i=0;i<140000;i++)giant[i]='g';giant[140000]=0;
 PortEffect(port_effects,"literal 0 %s\n",giant);
 for(unsigned i=0;i<2000;i++)PortEffect(port_effects,"apu %u %u %u\n",0,3,255);
 return PortEffectsClose()?2:0;
}
"#,
    );
    let path = run.0.join("effects.txt");
    let receipt_path = run.0.join("metrics.toml");
    let output = Command::new(producer)
        .env("SM_EVIDENCE_PIPE", &path)
        .env("SM_PIPELINE_METRICS", "enabled")
        .env("SM_PIPELINE_METRICS_PRODUCER_PATH", &receipt_path)
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let mut expected = b"literal 0 ".to_vec();
    expected.extend(std::iter::repeat_n(b'g', 140000));
    expected.push(b'\n');
    expected.extend_from_slice(&b"apu 0 3 255\n".repeat(2000));
    assert_eq!(fs::read(path).unwrap(), expected);
    let receipt: Receipt = toml::from_str(&fs::read_to_string(receipt_path).unwrap()).unwrap();
    assert_eq!(receipt.interruptions, 1);
    assert!(receipt.short_writes > 1 && receipt.write_calls > receipt.flush_count);
    assert_eq!(receipt.outcome, "success");
}

#[test]
fn broken_pipe_and_abandoned_stream_leave_failure_receipts() {
    let run = Run::new();
    let producer = run.producer(
        "",
        r#"
int main(int argc,char **argv) {
 (void)argv;
 port_effects=PortEffectsOpen();if(!port_effects)return 2;
 if(argc>1){PortEffect(port_effects,"apu %u %u %u\n",0,3,255);exit(7);}
 for(unsigned i=0;i<100000;i++)PortEffect(port_effects,"ppu %u %d %u %u\n",0,0,24,255);
 return PortEffectsClose()?2:0;
}

"#,
    );
    let failed_path = run.0.join("broken.toml");
    let mut child = Command::new(&producer)
        .env("SM_EVIDENCE_PIPE", "/proc/self/fd/1")
        .env("SM_PIPELINE_METRICS", "enabled")
        .env("SM_PIPELINE_METRICS_PRODUCER_PATH", &failed_path)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    assert_eq!(child.wait().unwrap().code(), Some(2));
    let failed: Receipt = toml::from_str(&fs::read_to_string(failed_path).unwrap()).unwrap();
    assert_eq!(failed.outcome, "failed");
    assert!(failed.flush_count > 0 && failed.write_calls > 0);
    let abandoned_path = run.0.join("abandoned.toml");
    let status = Command::new(producer)
        .arg("abandon")
        .env("SM_EVIDENCE_PIPE", run.0.join("effects.txt"))
        .env("SM_PIPELINE_METRICS", "disabled")
        .env("SM_PIPELINE_METRICS_PRODUCER_PATH", &abandoned_path)
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(7));
    let abandoned: Receipt = toml::from_str(&fs::read_to_string(abandoned_path).unwrap()).unwrap();
    assert_eq!(abandoned.outcome, "failed");
    assert!(!abandoned.enabled);
}
#[test]
fn undurable_metrics_sidecar_refuses_success_after_exact_effect_flush() {
    let run = Run::new();
    let producer = run.producer(
        "",
        r#"
int main(void) {
 port_effects=PortEffectsOpen();if(!port_effects)return 2;
 PortEffect(port_effects,"apu %u %u %u\n",0,3,255);
 return PortEffectsClose()?2:0;
}
"#,
    );
    let effects = run.0.join("effects.txt");
    let missing = run.0.join("missing/receipt.toml");
    let status = Command::new(producer)
        .env("SM_EVIDENCE_PIPE", &effects)
        .env("SM_PIPELINE_METRICS", "enabled")
        .env("SM_PIPELINE_METRICS_PRODUCER_PATH", &missing)
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(2));
    assert_eq!(fs::read(effects).unwrap(), b"apu 0 3 255\n");
    assert!(!missing.exists());
}
impl Drop for Run {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn failed_or_backwards_flush_clocks_cannot_report_metric_success() {
    let run = Run::new();
    let producer = run.producer(
        "",
        r#"
int __wrap_clock_gettime(clockid_t clock_id,struct timespec *now) {
 (void)clock_id;static unsigned calls;
 const char *mode=getenv("CLOCK_CONTROL");
 if(!strcmp(mode,"failed")){errno=EIO;return -1;}
 if(!strcmp(mode,"overflow")){now->tv_sec=INT64_MAX;now->tv_nsec=0;return 0;}
 now->tv_sec=calls++?1:2;now->tv_nsec=0;return 0;
}
int main(void) {
 port_effects=PortEffectsOpen();if(!port_effects)return 2;
 PortEffect(port_effects,"apu %u %u %u\n",0,3,255);
 return PortEffectsClose()?2:0;
}
"#,
    );
    for mode in ["failed", "backwards", "overflow"] {
        let path = run.0.join(format!("{mode}.toml"));
        let status = Command::new(&producer)
            .env("SM_EVIDENCE_PIPE", run.0.join(format!("{mode}.txt")))
            .env("SM_PIPELINE_METRICS", "enabled")
            .env("SM_PIPELINE_METRICS_PRODUCER_PATH", &path)
            .env("CLOCK_CONTROL", mode)
            .status()
            .unwrap();
        assert_eq!(status.code(), Some(2));
        let receipt: Receipt = toml::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(receipt.outcome, "failed");
    }
}
#[test]
fn effect_pipe_failures_preserve_the_hosts_existing_sigpipe_handler() {
    let run = Run::new();
    let producer = run.producer(
        "",
        r#"
#include <signal.h>
static volatile sig_atomic_t hits;
static void handler(int signal_number){(void)signal_number;hits++;}
int main(void) {
 if(signal(SIGPIPE,handler)==SIG_ERR)return 9;
 port_effects=PortEffectsOpen();if(!port_effects)return 2;
 /* Parent closes its pipe reader before allowing the failing write. */
 if(getchar()!='x')return 9;
 raise(SIGPIPE);if(hits!=1)return 9;
 PortEffect(port_effects,"apu %u %u %u\n",0,3,255);
 if(!PortEffectsClose())return 9;
 if(hits!=1)return 9;
 raise(SIGPIPE);if(hits!=2)return 9;
 fprintf(stderr,"preserved 2\n");return 2;
}
"#,
    );
    let path = run.0.join("metrics.toml");
    let mut child = Command::new(producer)
        .env("SM_EVIDENCE_PIPE", "/proc/self/fd/1")
        .env("SM_PIPELINE_METRICS", "enabled")
        .env("SM_PIPELINE_METRICS_PRODUCER_PATH", &path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    use std::io::Write;
    child.stdin.take().unwrap().write_all(b"x").unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(2), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.stderr, b"preserved 2\n");
    let receipt: Receipt = toml::from_str(&fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(receipt.outcome, "failed");
}
#[derive(Debug, Deserialize)]
struct Receipt {
    format: String,
    enabled: bool,
    buffer_bytes: u64,
    transport_bytes: u64,
    flush_count: u64,
    write_calls: u64,
    write_elapsed_ns: u64,
    max_flush_ns: u64,
    short_writes: u64,
    interruptions: u64,
    outcome: String,
}
fn reader() -> PathBuf {
    std::env::var_os("SM_EVIDENCE_TEST_BINARY").map(PathBuf::from).unwrap_or_else(|| {
        Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().join("target/debug/sm-evidence")
    })
}
fn capture(root: &Path, input: &Path) -> (String, Pointer) {
    fs::create_dir(root).unwrap();
    fs::create_dir(root.join("stage")).unwrap();
    fs::write(root.join("stage/terminal.bin"), b"independent terminal observation").unwrap();
    fs::write(root.join("provenance.toml"), "schema_version = 2\n").unwrap();
    let output = Command::new(reader())
        .arg("capture")
        .arg(root.join("store"))
        .arg(root.join("trace.toml"))
        .arg(root.join("stage"))
        .arg(root.join("provenance.toml"))
        .arg(root.join("pointer.toml"))
        .stdin(fs::File::open(input).unwrap())
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let trace: toml::Value = toml::from_str(&fs::read_to_string(root.join("trace.toml")).unwrap()).unwrap();
    let pointer = toml::from_str(&fs::read_to_string(root.join("pointer.toml")).unwrap()).unwrap();
    (trace["stream"].as_str().unwrap().to_owned(), pointer)
}

#[test]
fn enabled_and_disabled_text_producers_keep_exact_bytes_and_stream_identity() {
    let run = Run::new();
    let producer = run.producer(
        "",
        r#"
int main(void) {
 port_effects=PortEffectsOpen();if(!port_effects)return 2;
 PortEffect(port_effects,"plm %u %d %d %u\n",65535,-2147483647,2147483647,255);
 PortEffect(port_effects,"ppu %u %d %u %u\n",4294967295u,1,255,0);
 PortEffect(port_effects,"apu %u %u %u\n",0,4294967295u,255);
 char giant[70001];for(size_t i=0;i<70000;i++)giant[i]='x';giant[70000]=0;
 PortEffect(port_effects,"literal 0 %s\n",giant);
 for(unsigned i=0;i<10000;i++)PortEffect(port_effects,"ppu %u %d %u %u\n",i%2,0,24,i%256);
 return PortEffectsClose()?2:0;
}
"#,
    );
    let mut expected =
        b"plm 65535 -2147483647 2147483647 255\nppu 4294967295 1 255 0\napu 0 4294967295 255\nliteral 0 ".to_vec();
    expected.extend(std::iter::repeat_n(b'x', 70000));
    expected.push(b'\n');
    for i in 0..10000 {
        expected.extend_from_slice(format!("ppu {} 0 24 {}\n", i % 2, i % 256).as_bytes());
    }
    fs::write(run.0.join("reference.txt"), &expected).unwrap();
    let reference = capture(&run.0.join("reference"), &run.0.join("reference.txt"));
    let mut receipts = Vec::new();
    for mode in ["enabled", "disabled"] {
        let input = run.0.join(format!("{mode}.txt"));
        let receipt_path = run.0.join(format!("{mode}.toml"));
        let output = Command::new(&producer)
            .env("SM_EVIDENCE_PIPE", &input)
            .env("SM_PIPELINE_METRICS", mode)
            .env("SM_PIPELINE_METRICS_PRODUCER_PATH", &receipt_path)
            .output()
            .unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(fs::read(&input).unwrap(), expected);
        let receipt: Receipt = toml::from_str(&fs::read_to_string(receipt_path).unwrap()).unwrap();
        assert_eq!(receipt.format, "pipeline-producer-metrics-v1");
        assert_eq!(receipt.enabled, mode == "enabled");
        assert_eq!(receipt.buffer_bytes, 65536);
        assert_eq!(receipt.transport_bytes, expected.len() as u64);
        assert!(receipt.flush_count > 1 && receipt.write_calls >= receipt.flush_count);
        assert_eq!(receipt.short_writes, 0);
        assert_eq!(receipt.interruptions, 0);
        assert_eq!(receipt.outcome, "success");
        if receipt.enabled {
            assert!(receipt.write_elapsed_ns > 0 && receipt.max_flush_ns > 0);
        } else {
            assert_eq!(receipt.write_elapsed_ns, 0);
            assert_eq!(receipt.max_flush_ns, 0);
        }
        let actual = capture(&run.0.join(mode), &input);
        assert_eq!(actual.0, reference.0);
        assert_eq!(actual.1.observations, reference.1.observations);
        assert_eq!(actual.1.artifacts, reference.1.artifacts);
        receipts.push(receipt);
    }
    assert_eq!(receipts[0].flush_count, receipts[1].flush_count);
    assert_eq!(receipts[0].write_calls, receipts[1].write_calls);
}
