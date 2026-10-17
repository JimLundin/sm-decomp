#![cfg(all(target_os = "linux", target_arch = "x86_64"))]
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};
use translation_execution::confinement::{Program, ReadOnlyBind, RunLimits, compiler_runtime, helper_runtime, run};

fn helper() -> PathBuf {
    std::env::var_os("SM_CONFINEMENT_TEST_HELPER")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_BIN_EXE_sm-translation-sandbox")))
}

struct Fixture {
    root: PathBuf,
    program: Program,
}
impl Fixture {
    fn new() -> Self {
        static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "sm-confinement-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let source = root.join("probe.c");
        fs::write(&source, r#"
#define _GNU_SOURCE
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/socket.h>
#include <sys/mman.h>
#include <sys/mount.h>
#include <sys/prctl.h>
#include <sys/wait.h>
#include <sched.h>
#include <fcntl.h>
#include <errno.h>
#include <unistd.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
int main(int argc, char **argv) {
  if (argc < 2) return 2;
  if (!strcmp(argv[1], "echo")) { char buf[64]; ssize_t n=read(0,buf,sizeof(buf));if(n<0)return 3;return write(1,buf,n)==n?0:4; }
  if (!strcmp(argv[1], "denials")) {
    if(open(argv[2],O_RDONLY)>=0||open("/inputs/secret",O_WRONLY)>=0||open("/usr/bin/sm-pipeline",O_RDONLY)>=0||open("/work/other-fixture/verdict.toml",O_WRONLY|O_CREAT,0600)>=0)return 10;
    if(socket(AF_INET,SOCK_STREAM,0)>=0||socket(AF_UNIX,SOCK_STREAM,0)>=0)return 11;
    if(unshare(CLONE_NEWUSER)==0||mount("tmpfs","/work","tmpfs",0,"size=1000000")==0)return 12;
    if(getenv("SM_SECRET")||getenv("SM_PIPELINE_ROOT"))return 13;
    int f=open("/inputs/secret",O_RDONLY);char b[8]={0};if(f<0||read(f,b,7)!=7||strcmp(b,"allowed"))return 14;close(f);
    puts("denied");return 0;
  }
  if (!strcmp(argv[1], "files")) {int f=open("result.bin",O_WRONLY|O_CREAT,0600);if(f<0)return 5;return write(f,"result",6)==6?0:6;}
  if (!strcmp(argv[1], "undeclared")) {int f=open("surprise",O_WRONLY|O_CREAT,0600);return f<0?7:0;}
  if (!strcmp(argv[1], "stdout")) {char b[4096];memset(b,'x',sizeof(b));for(int i=0;i<4;i++)if(write(1,b,sizeof(b))<0)return 22;return 0;}
  if (!strcmp(argv[1], "ignore-stdout")) {char b[4096];memset(b,'x',sizeof(b));for(int i=0;i<4;i++){ssize_t ignored=write(1,b,sizeof(b));(void)ignored;}int ignored=ftruncate(1,0);(void)ignored;return 0;}
  if (!strcmp(argv[1], "quota")) {char b[4096];memset(b,'q',sizeof(b));for(int i=0;i<256;i++){char p[64];snprintf(p,sizeof(p),"tmp/%d",i);int f=open(p,O_WRONLY|O_CREAT,0600);if(f<0)return 20;if(write(f,b,sizeof(b))!=sizeof(b))return 21;close(f);}return 0;}
  if (!strcmp(argv[1], "ignore-quota")) {char b[4096];memset(b,'q',sizeof(b));int f=open("tmp/transient",O_WRONLY|O_CREAT,0600);while(write(f,b,sizeof(b))>0){}close(f);unlink("tmp/transient");return 0;}
  if (!strcmp(argv[1], "sparse-quota")) {for(int i=0;i<3;i++){char p[64];snprintf(p,sizeof(p),"tmp/sparse%d",i);int f=open(p,O_WRONLY|O_CREAT,0600);if(f<0||ftruncate(f,65536)<0)return 23;close(f);}for(int i=0;i<3;i++){char p[64];snprintf(p,sizeof(p),"tmp/sparse%d",i);unlink(p);}return 0;}
  if (!strcmp(argv[1], "unlinked-sparse-quota")) {int f[3];for(int i=0;i<3;i++){char p[64];snprintf(p,sizeof(p),"tmp/sparse%d",i);f[i]=open(p,O_WRONLY|O_CREAT,0600);if(f[i]<0)return 24;unlink(p);if(ftruncate(f[i],65536)<0)return 25;}for(int i=0;i<3;i++)close(f[i]);return 0;}
  if (!strcmp(argv[1], "anonymous-file-denied")) {int f=memfd_create("unbounded-output",0);return f<0&&errno==EPERM?0:26;}
  if (!strcmp(argv[1], "private-descriptors")) {if(prctl(PR_SET_DUMPABLE,0,0,0,0)<0)return 27;int f=open("tmp/private",O_WRONLY|O_CREAT,0600);if(f<0)return 28;unlink("tmp/private");if(ftruncate(f,argc>2?65536:4096)<0)return 29;if(argc>2){int second=open("tmp/private-two",O_WRONLY|O_CREAT,0600);if(second<0)return 30;unlink("tmp/private-two");if(ftruncate(second,70000)<0)return 31;}return 0;}
  if (!strcmp(argv[1], "descendant")) {pid_t p=fork();if(p<0)return 8;if(!p){sleep(1);puts("survived");fflush(stdout);return 0;}puts("parent");return 0;}
  if (!strcmp(argv[1], "sleep")) {if(!fork()){setsid();sleep(30);return 0;}sleep(30);return 0;}
  return 1;
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
                arguments: vec![],
                inputs: vec![],
                runtime: helper_runtime().unwrap(),
                helper: helper(),
                outputs: vec![],
            },
        }
    }
    fn call(&self, mode: &str, limits: &RunLimits) -> anyhow::Result<translation_execution::confinement::RunOutput> {
        let mut program = self.program.clone();
        program.arguments = vec![mode.into()];
        run(&program, limits, b"capsule")
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
fn limits() -> RunLimits {
    RunLimits {
        timeout: Duration::from_secs(5),
        writable_bytes: 128 * 1024,
        stdout_bytes: 8192,
        stderr_bytes: 8192,
        stdin_bytes: 8192,
    }
}

#[test]
fn declared_input_readonly_host_paths_network_and_namespace_escape_denied() {
    let mut fixture = Fixture::new();
    let secret = fixture.root.join("secret");
    fs::write(&secret, b"allowed").unwrap();
    fixture.program.inputs.push(ReadOnlyBind { source: secret.clone(), destination: "/inputs/secret".into() });
    fixture.program.arguments = vec!["denials".into(), secret.display().to_string()];
    let output = run(&fixture.program, &limits(), b"").unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout, b"denied\n");
    assert_eq!(fs::read(secret).unwrap(), b"allowed");
}

#[test]
fn bounded_stdin_stdout_and_declared_file_transport() {
    let mut fixture = Fixture::new();
    let output = fixture.call("echo", &limits()).unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout, b"capsule");
    fixture.program.arguments = vec!["files".into()];
    fixture.program.outputs = vec!["result.bin".into()];
    let output = run(&fixture.program, &limits(), b"").unwrap();
    assert_eq!(output.outputs[&PathBuf::from("result.bin")], b"result");
    assert!(fixture.call("stdout", &limits()).is_err());
    assert!(fixture.call("ignore-stdout", &limits()).is_err());
    let mut low = limits();
    low.stdin_bytes = 2;
    assert!(fixture.call("echo", &low).is_err());
}

#[test]
fn aggregate_tmpfs_quota_and_undeclared_output_refuse_completion() {
    let fixture = Fixture::new();
    let error = fixture.call("quota", &limits()).unwrap_err().to_string();
    assert!(error.contains("quota"), "{error}");
    let error = fixture.call("ignore-quota", &limits()).unwrap_err().to_string();
    assert!(error.contains("quota"), "{error}");
    let error = fixture.call("sparse-quota", &limits()).unwrap_err().to_string();
    assert!(error.contains("budget"), "{error}");
    let error = fixture.call("unlinked-sparse-quota", &limits()).unwrap_err().to_string();
    assert!(error.contains("budget"), "{error}");
    assert!(fixture.call("anonymous-file-denied", &limits()).unwrap().status.success());
    let error = fixture.call("undeclared", &limits()).unwrap_err().to_string();
    assert!(error.contains("undeclared"), "{error}");
}

#[test]
fn private_descriptor_metadata_refuses_acceptance_and_unlinked_quota_bypass() {
    let fixture = Fixture::new();
    let error = fixture.call("private-descriptors", &limits()).unwrap_err().to_string();
    assert!(
        error.contains("descriptor") && error.contains("Permission denied"),
        "private descriptor observer was not refused: {error}"
    );
    let mut program = fixture.program.clone();
    program.arguments = vec!["private-descriptors".into(), "overflow".into()];
    let error = run(&program, &limits(), b"").unwrap_err().to_string();
    assert!(
        error.contains("descriptor") && error.contains("Permission denied"),
        "unlinked private extents escaped quota: {error}"
    );
}

#[test]
fn namespace_descendants_reaped_on_success_and_timeout() {
    let fixture = Fixture::new();
    let started = Instant::now();
    let output = fixture.call("descendant", &limits()).unwrap();
    assert!(started.elapsed() < Duration::from_secs(1));
    assert_eq!(output.stdout, b"parent\n");
    let mut short = limits();
    short.timeout = Duration::from_millis(100);
    let started = Instant::now();
    let error = fixture.call("sleep", &short).unwrap_err();
    assert!(error.to_string().contains("timeout"));
    assert!(started.elapsed() < Duration::from_secs(2));
}

#[test]
fn unavailable_helper_and_aliased_sources_fail_before_launch() {
    let mut fixture = Fixture::new();
    fixture.program.helper = fixture.root.join("missing");
    assert!(run(&fixture.program, &limits(), b"").is_err());
    fixture.program.helper = helper();
    let alias = fixture.root.join("alias");
    std::os::unix::fs::symlink(&fixture.program.executable, &alias).unwrap();
    fixture.program.executable = alias;
    assert!(run(&fixture.program, &limits(), b"").is_err());
}

#[test]
fn actual_rust_compiler_cannot_include_undeclared_host_file() {
    let fixture = Fixture::new();
    let secret = fixture.root.join("controller-secret");
    fs::write(&secret, b"protected").unwrap();
    let source = fixture.root.join("candidate.rs");
    fs::write(
        &source,
        format!(
            "const SECRET: &[u8] = include_bytes!({:?}); fn main() {{println!(\"{{}}\",SECRET.len());}}",
            secret.display().to_string()
        ),
    )
    .unwrap();
    let compiler = Command::new("rustup").args(["which", "rustc"]).output().unwrap();
    assert!(compiler.status.success());
    let compiler = PathBuf::from(String::from_utf8(compiler.stdout).unwrap().trim());
    let program = Program {
        executable: compiler.clone(),
        arguments: vec![
            "--edition=2024".into(),
            "--emit=metadata".into(),
            "/inputs/candidate.rs".into(),
            "-o".into(),
            "/work/tmp/candidate.rmeta".into(),
        ],
        inputs: vec![ReadOnlyBind { source, destination: "/inputs/candidate.rs".into() }],
        runtime: compiler_runtime(&compiler).unwrap(),
        helper: fixture.program.helper.clone(),
        outputs: vec![],
    };
    let mut bounds = limits();
    bounds.writable_bytes = 8 * 1024 * 1024;
    bounds.stderr_bytes = 65536;
    let output = run(&program, &bounds, b"").unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("couldn't read"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fs::read(secret).unwrap(), b"protected");
}

#[test]
fn readonly_rust_test_inputs_compile_through_linker_descendants() {
    let fixture = Fixture::new();
    let source = fixture.root.join("frozen.rs");
    fs::write(&source, "#[test] fn doubles(){assert_eq!(7u32.wrapping_mul(2),14);}\n").unwrap();
    let compiler = Command::new("rustup").args(["which", "rustc"]).output().unwrap();
    assert!(compiler.status.success());
    let compiler = PathBuf::from(String::from_utf8(compiler.stdout).unwrap().trim());
    let runtime = compiler_runtime(&compiler).unwrap();
    let program = Program {
        executable: compiler.clone(),
        arguments: vec![
            "--edition=2024".into(),
            "--test".into(),
            "--crate-name=confined_linker_probe".into(),
            "/inputs/frozen.rs".into(),
            "--sysroot".into(),
            "/toolchain".into(),
            "-C".into(),
            "linker=/usr/bin/x86_64-linux-gnu-gcc-9".into(),
            "-C".into(),
            "linker-features=-lld".into(),
            "-C".into(),
            "link-arg=-B/usr/lib/gcc/x86_64-linux-gnu/9/".into(),
            "-C".into(),
            "link-arg=-B/usr/bin/".into(),
            "--emit=link,dep-info=/work/compiler.d".into(),
            "-o".into(),
            "/work/tests".into(),
        ],
        inputs: vec![ReadOnlyBind { source, destination: "/inputs/frozen.rs".into() }],
        runtime,
        helper: fixture.program.helper.clone(),
        outputs: vec!["tests".into(), "compiler.d".into()],
    };
    let mut bounds = limits();
    bounds.timeout = Duration::from_secs(30);
    bounds.writable_bytes = 32 * 1024 * 1024;
    bounds.stderr_bytes = 65536;
    for attempt in 0..6 {
        let output = run(&program, &bounds, b"")
            .unwrap_or_else(|error| panic!("standalone test compilation {attempt} lost confinement: {error:#}"));
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert!(!output.outputs[&PathBuf::from("tests")].is_empty());
    }
}
