//! Native evidence must reject wrong results, incomplete coverage and unchecked C.
#[path = "support/binary_fixture.rs"]
mod binary_fixture;
#[path = "../src/source_stamps.rs"]
mod source_stamps;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
use toml::Value;
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Project(PathBuf, bool);
impl Drop for Project {
    fn drop(&mut self) {
        let mut cleanup = Vec::new();
        for root in ["out", "batch"] {
            if !self.0.join(root).is_dir() {
                continue;
            }
            for deduplicate in [false, true] {
                let mut command = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"));
                command.args(["cleanup", "--root"]).arg(self.0.join(root)).arg("--apply");
                if deduplicate {
                    command.arg("--deduplicate");
                }
                let output = command.output().unwrap();
                assert!(output.status.success(), "cleanup failed: {}", String::from_utf8_lossy(&output.stderr));
                if self.1 {
                    cleanup.push(toml::from_str::<Value>(std::str::from_utf8(&output.stdout).unwrap()).unwrap());
                }
            }
        }
        if self.1 {
            let mut disposable_bytes = 0;
            for disposable in ["cache", ".git"] {
                if self.0.join(disposable).is_dir() {
                    disposable_bytes += tree_bytes(&self.0.join(disposable));
                    fs::remove_dir_all(self.0.join(disposable)).unwrap();
                }
            }
            write(
                &self.0,
                "retention.toml",
                toml::to_string(&Value::Table(toml::Table::from_iter([
                    ("cleanup".into(), Value::Array(cleanup)),
                    ("disposable_bytes_removed".into(), Value::try_from(disposable_bytes).unwrap()),
                    ("retained".into(), "sealed family runs and exact source snapshot; .git/cache removed".into()),
                ])))
                .unwrap(),
            );
        } else {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}
fn tree_bytes(path: &Path) -> u64 {
    let metadata = fs::symlink_metadata(path).unwrap();
    if metadata.is_file() {
        metadata.len()
    } else if metadata.is_dir() {
        fs::read_dir(path).unwrap().map(|entry| tree_bytes(&entry.unwrap().path())).sum()
    } else {
        0
    }
}
impl Project {
    fn git(&self, args: &[&str]) -> String {
        let output = Command::new("git").args(args).current_dir(&self.0).output().unwrap();
        assert!(output.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&output.stderr));
        String::from_utf8(output.stdout).unwrap().trim().into()
    }
    fn initialize_git(&self) {
        write(&self.0, ".gitignore", "/out/\n/batch/\n/cache/\n/target/\n/rom/\n/movies/\n/.scratch/coordinator/\n");
        self.git(&["init", "-q"]);
        self.git(&["add", "."]);
        self.git(&["-c", "user.name=Fixture", "-c", "user.email=fixture@example.test", "commit", "-qm", "source"]);
    }
    fn assign(&self, operation: &str, update: Option<Value>, success: bool) -> Value {
        let mut request = Value::Table(toml::Table::from_iter([
            ("checkout".into(), Value::try_from(&self.0).unwrap()),
            ("operation".into(), operation.into()),
            ("effort".into(), "coordinator".into()),
            ("coordinator".into(), "fixture-owner".into()),
            ("available_slots".into(), 2.into()),
            ("pipeline_root".into(), Value::try_from(self.0.join("cache")).unwrap()),
            ("workers".into(), 2.into()),
            ("translations".into(), true.into()),
        ]));
        if let Some(update) = update {
            request.as_table_mut().unwrap().insert("update".into(), update);
        } else if matches!(operation, "plan" | "next") {
            request.as_table_mut().unwrap().insert("changed_since".into(), "HEAD".into());
        }
        self.invoke("assign", request, success)
    }
    fn start_assignment(&self, assignment: &Value) {
        self.assign(
            "record",
            Some(Value::Table(toml::Table::from_iter([
                ("event".into(), "started".into()),
                ("claim".into(), assignment["claim"].clone()),
                ("agent".into(), "fixture-agent".into()),
            ]))),
            true,
        );
    }
}

fn sealed(path: &Path) -> Value {
    Value::Table(toml::Table::from_iter([
        ("path".into(), Value::try_from(path).unwrap()),
        ("sha256".into(), format!("{:x}", Sha256::digest(fs::read(path).unwrap())).into()),
    ]))
}
fn finish(claim: &Value, result: Value) -> Value {
    Value::Table(toml::Table::from_iter([
        ("event".into(), "finished".into()),
        ("claim".into(), claim.clone()),
        ("result".into(), result),
    ]))
}
fn write(root: &Path, name: &str, bytes: impl AsRef<[u8]>) {
    let p = root.join(name);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    if p.extension().is_some_and(|e| e == "smf") {
        binary_fixture::write(&p, bytes).unwrap();
    } else {
        fs::write(p, bytes).unwrap();
    }
}
impl Project {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "sm-family-test-{}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        Self::at(root, false)
    }
    fn at(root: PathBuf, retain: bool) -> Self {
        fs::create_dir(&root).unwrap();
        let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        for (name, source) in source_stamps::all() {
            write(&root, name, source);
        }
        write(
            &root,
            "crates/port/src/memory.rs",
            fs::read(crate_root.parent().unwrap().join("port/src/memory.rs")).unwrap(),
        );
        write(&root,"crates/port/src/fixture.rs", b"use crate::memory::Wram; pub fn dispatch(ram:&mut Wram<'_>,name:&str,a:u16,_b:u16)->Result<u32,&'static str>{if name!=\"Touch\" || a==42{return Err(\"bounded rejection\");}ram.set_byte(17,a as u8);Ok(a as u32+1)}");
        write(&root,"deps/snesrev-sm/src/sm_80.c", b"#include <stdint.h>\nextern uint8_t g_ram[0x20000];uint16_t Touch(uint16_t a){g_ram[17]=(uint8_t)a;return a+1;}");
        write(&root, "deps/snesrev-sm/src/sm_rtl.h", b"#include <stdbool.h>\n");
        write(&root, "deps/snesrev-sm/src/funcs.h", b"");
        write(&root, "deps/snesrev-sm/LICENSE.txt", b"test oracle");
        fs::create_dir(root.join("deps/downloads")).unwrap();
        let archive = root.join("deps/downloads/oracle.tar.gz");
        assert!(
            Command::new("tar")
                .args(["-czf"])
                .arg(&archive)
                .arg("-C")
                .arg(root.join("deps"))
                .arg("snesrev-sm")
                .status()
                .unwrap()
                .success()
        );
        let hash = format!("{:x}", Sha256::digest(fs::read(archive).unwrap()));
        write(
            &root,
            "tools/dependencies.toml",
            format!(
                "schema_version = 1\n[[dependencies]]\nname = \"snesrev-sm\"\narchive = \"oracle.tar.gz\"\nsha256 = {hash:?}\nrevision = \"test-pinned-source\"\n"
            ),
        );
        Self(root, retain)
    }
    fn manifest(&self, cases: Value) {
        let manifest = Value::Table(toml::Table::from_iter([
            ("family".into(), "fixture".into()),
            ("bank".into(), "sm_80.c".into()),
            ("rust_module".into(), "crates/port/src/fixture.rs".into()),
            (
                "routines".into(),
                Value::Array(vec![Value::Table(toml::Table::from_iter([
                    ("symbol".into(), "Touch".into()),
                    ("args".into(), 1.into()),
                    ("return".into(), "uint16".into()),
                    ("cases".into(), cases),
                ]))]),
            ),
        ]));
        write(&self.0, "tools/port/fixture.smf", toml::to_string(&manifest).unwrap());
    }

    fn pinned_c(&self, source: &str) {
        write(&self.0, "deps/snesrev-sm/src/sm_80.c", source);
        // The extracted actual-C body includes this pinned common header.
        write(&self.0, "deps/snesrev-sm/src/sm_rtl.h", "#include <stdbool.h>\n#include <unistd.h>\n");
        let archive = self.0.join("deps/downloads/oracle.tar.gz");
        assert!(
            Command::new("tar")
                .arg("-czf")
                .arg(&archive)
                .arg("-C")
                .arg(self.0.join("deps"))
                .arg("snesrev-sm")
                .status()
                .unwrap()
                .success()
        );
        let pins = self.0.join("tools/dependencies.toml");
        let mut value: Value = toml::from_str(&fs::read_to_string(&pins).unwrap()).unwrap();
        value["dependencies"][0]["sha256"] = format!("{:x}", Sha256::digest(fs::read(archive).unwrap())).into();
        fs::write(pins, toml::to_string(&value).unwrap()).unwrap();
    }
    fn invoke(&self, operation: &str, request: Value, success: bool) -> Value {
        let mut child = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
            .arg(operation)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        writeln!(child.stdin.take().unwrap(), "{}", toml::to_string(&request).unwrap()).unwrap();
        let output = child.wait_with_output().unwrap();
        assert_eq!(
            output.status.success(),
            success,
            "{}\n{}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        );
        if output.stdout.is_empty() {
            Value::Table(toml::Table::new())
        } else {
            toml::from_str(&String::from_utf8(output.stdout).unwrap()).unwrap()
        }
    }
    fn verify(&self, success: bool) -> Value {
        self.verify_mode(success, "auto")
    }
    fn verify_mode(&self, success: bool, mode: &str) -> Value {
        self.invoke(
            "family-verify",
            Value::Table(toml::Table::from_iter([
                ("checkout".to_owned(), Value::try_from(&(self.0)).unwrap()),
                ("source".to_owned(), Value::try_from(self.0.join("deps/snesrev-sm")).unwrap()),
                ("manifest".to_owned(), Value::String(("tools/port/fixture.smf").to_owned())),
                ("output".to_owned(), Value::try_from(self.0.join("out")).unwrap()),
                ("root".to_owned(), Value::try_from(self.0.join("cache")).unwrap()),
                ("workers".to_owned(), Value::Integer(2_i64)),
                ("case_jobs".to_owned(), Value::Integer(4_i64)),
                ("execution".into(), mode.into()),
            ])),
            success,
        )
    }
    fn authenticate(&self, verdict: &Value, success: bool) {
        self.invoke(
            "family-authenticate",
            Value::Table(toml::Table::from_iter([
                ("checkout".to_owned(), Value::try_from(&(self.0)).unwrap()),
                (
                    "verdict".to_owned(),
                    Value::try_from(Path::new(verdict["run"].as_str().unwrap()).join("verdict.toml")).unwrap(),
                ),
            ])),
            success,
        );
    }

    fn batch(&self, success: bool) -> Value {
        self.invoke(
            "batch",
            Value::Table(toml::Table::from_iter([
                ("checkout".to_owned(), Value::try_from(&(self.0)).unwrap()),
                ("operation".to_owned(), Value::String(("verify").to_owned())),
                ("batch".to_owned(), Value::try_from(self.0.join("batch/batch.toml")).unwrap()),
                ("families".to_owned(), Value::Array(vec![Value::String(("fixture").to_owned())])),
                ("root".to_owned(), Value::try_from(self.0.join("cache")).unwrap()),
                ("workers".to_owned(), Value::Integer(2_i64)),
                ("jobs".to_owned(), Value::Integer(2_i64)),
                ("case_jobs".to_owned(), Value::Integer(2_i64)),
            ])),
            success,
        )
    }

    // Performance controls isolate artifact-cache contents while sharing the
    // session's process capacity. Ordinary isolated capacity tests are unchanged.
    fn measured_batch(&self, fresh: bool, sampling: &str, success: bool) -> (Value, f64) {
        let (child, started) = self.spawn_measured_batch(fresh, sampling);
        let output = child.wait_with_output().unwrap();
        let seconds = started.elapsed().as_secs_f64();
        assert_eq!(output.status.success(), success, "{}", String::from_utf8_lossy(&output.stderr));
        (toml::from_str(&String::from_utf8(output.stdout).unwrap()).unwrap(), seconds)
    }

    fn spawn_measured_batch(&self, fresh: bool, sampling: &str) -> (std::process::Child, std::time::Instant) {
        let cache = self.0.join("cache");
        fs::create_dir_all(&cache).unwrap();
        let workers = if let Some(shared) = std::env::var_os("SM_PIPELINE_ROOT") {
            let worker_path = cache.join("workers");
            if !worker_path.exists() {
                std::os::unix::fs::symlink(Path::new(&shared).join("workers"), &worker_path).unwrap();
            }
            std::env::var("SM_PIPELINE_WORKERS").unwrap().parse::<i64>().unwrap()
        } else {
            2
        };
        let request = Value::Table(toml::Table::from_iter([
            ("checkout".into(), Value::try_from(&self.0).unwrap()),
            ("operation".into(), "verify".into()),
            ("batch".into(), Value::try_from(self.0.join("batch/batch.toml")).unwrap()),
            ("families".into(), Value::Array(vec!["fixture".into()])),
            ("root".into(), Value::try_from(cache).unwrap()),
            ("workers".into(), workers.into()),
            ("jobs".into(), 1.into()),
            ("case_jobs".into(), 1.into()),
            ("fresh".into(), fresh.into()),
        ]));
        let started = std::time::Instant::now();
        let mut child = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
            .arg("batch")
            .env("SM_PIPELINE_METRICS", sampling)
            .env_remove("SM_PIPELINE_METRICS_CONTEXT")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(toml::to_string(&request).unwrap().as_bytes()).unwrap();
        (child, started)
    }
}

#[test]
fn invalid_dispatcher_pointer_bank_is_rejected_before_compilation() {
    let project = Project::new();
    project.manifest(Value::Array(vec![Value::Table(toml::Table::from_iter([
        ("name".to_owned(), Value::String(("one").to_owned())),
        ("a".to_owned(), Value::Integer(1_i64)),
    ]))]));
    let path = project.0.join("tools/port/fixture.smf");
    let mut manifest: Value = toml::from_str(&binary_fixture::read(&path).unwrap()).unwrap();
    manifest["routines"][0]["return"] = "rom_ptr".into();
    manifest["routines"][0].as_table_mut().unwrap().insert("pointer_bank".into(), 0x100.into());
    write(&project.0, "tools/port/fixture.smf", toml::to_string(&manifest).unwrap());
    let verdict = project.verify(false);
    assert!(verdict["error"].as_str().unwrap().contains("invalid pointer bank"));
    assert!(!Path::new(verdict["run"].as_str().unwrap()).join("c").exists());
}

#[test]
fn packed_word_calls_preserve_argument_order_high_bits_and_manifest_acceptance() {
    let project = Project::new();
    write(
        &project.0,
        "deps/snesrev-sm/src/sm_80.c",
        r#"#include <stdint.h>
extern uint8_t g_ram[0x20000];
static void word(unsigned at,uint16_t value){g_ram[at]=value;g_ram[at+1]=value>>8;}
uint32_t Touch3(uint16_t a,uint16_t b,uint16_t c){word(16,a);word(18,b);word(20,c);return (uint32_t)a+b+c;}
uint32_t Touch4(uint16_t a,uint16_t b,uint16_t c,uint16_t d){word(16,a);word(18,b);word(20,c);word(22,d);return (uint32_t)a+b+c+d;}
"#,
    );
    let archive = project.0.join("deps/downloads/oracle.tar.gz");
    assert!(
        Command::new("tar")
            .arg("-czf")
            .arg(&archive)
            .arg("-C")
            .arg(project.0.join("deps"))
            .arg("snesrev-sm")
            .status()
            .unwrap()
            .success()
    );
    let pins = project.0.join("tools/dependencies.toml");
    let mut pin: Value = toml::from_str(&fs::read_to_string(&pins).unwrap()).unwrap();
    pin["dependencies"][0]["sha256"] = format!("{:x}", Sha256::digest(fs::read(archive).unwrap())).into();
    fs::write(pins, toml::to_string(&pin).unwrap()).unwrap();
    write(
        &project.0,
        "crates/port/src/fixture.rs",
        r#"use crate::memory::Wram;
pub fn dispatch_i32(ram:&mut Wram<'_>,name:&str,a:i32,b:i32)->Result<u32,&'static str>{
let words=match name {"Touch3"=>vec![a as u16,b as u16,((b as u32)>>16) as u16],"Touch4"=>vec![a as u16,((a as u32)>>16) as u16,b as u16,((b as u32)>>16) as u16],_=>return Err("routine")};
for (i,value) in words.iter().enumerate(){ram.set_word(16+2*i,*value);}
Ok(words.iter().map(|word|*word as u32).sum())}
"#,
    );
    let manifest = r#"family='fixture'
bank='sm_80.c'
rust_module='crates/port/src/fixture.rs'
[[routines]]
symbol='Touch3'
arg_types=['uint16','uint32']
packed_u16_args=3
return='uint32'
cases=[{name='three',a=43981,b=4294840884,expected_return=114175}]
[[routines]]
symbol='Touch4'
arg_types=['uint32','uint32']
packed_u16_args=4
return='uint32'
cases=[{name='four',a=2147527629,b=4294840884,expected_return=146943}]
"#;
    write(&project.0, "tools/port/fixture.smf", manifest);
    let binary = fs::read(project.0.join("tools/port/fixture.smf")).unwrap();
    let verdict = project.verify_mode(true, "fresh");
    assert_eq!(verdict["verified_cases"].as_integer(), Some(2));
    assert_eq!(fs::read(project.0.join("tools/port/fixture.smf")).unwrap(), binary);
    project.authenticate(&verdict, true);
    let run = PathBuf::from(verdict["run"].as_str().unwrap());
    let three = fs::read(run.join("three/c-after.wram")).unwrap();
    assert_eq!(&three[16..22], &[0xcd, 0xab, 0x34, 0x12, 0xfe, 0xff]);
    let four = fs::read(run.join("four/c-after.wram")).unwrap();
    assert_eq!(&four[16..24], &[0xcd, 0xab, 0x00, 0x80, 0x34, 0x12, 0xfe, 0xff]);
    write(&project.0, "tools/port/fixture.smf", manifest.replacen("['uint16','uint32']", "['uint32','uint32']", 1));
    let rejected = project.verify_mode(false, "fresh");
    assert!(rejected["error"].as_str().unwrap().contains("invalid packed"));
    assert!(!Path::new(rejected["run"].as_str().unwrap()).join("c").exists());
}

#[test]
fn binary_roots_and_continuations_preserve_full_state_and_compact_evidence() {
    let project = match std::env::var_os("SM_BINARY_FIXTURE_OUTPUT") {
        Some(path) => Project::at(PathBuf::from(path), true),
        None => Project::new(),
    };
    let declarations: Value = toml::from_str(
        r#"
cases = [
    { name = "root", a = 1, fill = 165, expected_return = 2, writes = [
        { offset = 10, data = [4, 5, 6] },
        { offset = 11, data = [0] },
        { offset = 30, data = [9] },
        { offset = 131071, data = [77] },
    ] },
    { name = "child", a = 2, continue_from = "root", expected_return = 3, writes = [
        { offset = 12, data = [0] },
        { offset = 24, data = [0, 3] },
        { offset = 25, data = [0] },
    ] },
]
"#,
    )
    .unwrap();
    project.manifest(declarations["cases"].clone());
    let mut root_before = vec![165; 131072];
    root_before[10..13].copy_from_slice(&[4, 0, 6]);
    root_before[30] = 9;
    root_before[131071] = 77;
    let mut root_after = root_before.clone();
    root_after[17] = 1;
    let mut child_before = root_after.clone();
    child_before[12] = 0;
    child_before[24..26].fill(0);
    let mut child_after = child_before.clone();
    child_after[17] = 2;
    let mut verdicts = Vec::new();
    for mode in ["fresh", "fork"] {
        let verdict = project.verify_mode(true, mode);
        assert_eq!(verdict["verified_cases"].as_integer(), Some(2));
        let run = Path::new(verdict["run"].as_str().unwrap());
        assert!(fs::read(run.join("manifest.smf")).unwrap().starts_with(b"SMFIXT01"));
        for (name, before, after) in [("root", &root_before, &root_after), ("child", &child_before, &child_after)] {
            let case = run.join(name);
            assert_eq!(fs::read(case.join("before.wram")).unwrap(), *before);
            for producer in ["c", "rust"] {
                assert_eq!(fs::read(case.join(format!("{producer}-after.wram"))).unwrap(), *after);
            }
            assert_eq!(fs::read(case.join("case.sha256")).unwrap().len(), 64);
            assert!(!case.join("case.toml").exists());
        }
        project.authenticate(&verdict, true);
        verdicts.push(verdict);
    }
    if project.1 {
        let checkout = project.0.clone();
        drop(project);
        let retained = std::mem::ManuallyDrop::new(Project(checkout, false));
        for verdict in &verdicts {
            retained.authenticate(verdict, true);
        }
    }
}

#[test]
fn valid_cases_authenticate_and_tampering_is_rejected_even_with_rebound_inventory() {
    let project = Project::new();
    project.manifest(Value::Array(vec![
        Value::Table(toml::Table::from_iter([
            ("name".to_owned(), Value::String(("one").to_owned())),
            ("a".to_owned(), Value::Integer(1_i64)),
            ("expected_return".to_owned(), Value::Integer(2_i64)),
        ])),
        Value::Table(toml::Table::from_iter([
            ("name".to_owned(), Value::String(("two").to_owned())),
            ("a".to_owned(), Value::Integer(2_i64)),
            ("fill".to_owned(), Value::Integer(165_i64)),
            ("expected_return".to_owned(), Value::Integer(3_i64)),
        ])),
    ]));
    let mut verdict = project.verify(true);
    project.authenticate(&verdict, true);
    let run = PathBuf::from(verdict["run"].as_str().unwrap());
    let mut state = fs::read(run.join("one/c-after.wram")).unwrap();
    state[17] = 99;
    fs::write(run.join("one/c-after.wram"), &state).unwrap();
    project.authenticate(&verdict, false);
    verdict["artifacts"]["one/c-after.wram"] = Value::try_from(format!("{:x}", Sha256::digest(&state))).unwrap();
    write(&run, "verdict.toml", toml::to_string(&verdict).unwrap());
    project.authenticate(&verdict, false);

    // Equal full-state outputs still have to match their retained case hashes,
    // even when the artifact inventory has been rebound to both altered files.
    fs::write(run.join("one/rust-after.wram"), &state).unwrap();
    verdict["artifacts"]["one/rust-after.wram"] = verdict["artifacts"]["one/c-after.wram"].clone();
    write(&run, "verdict.toml", toml::to_string(&verdict).unwrap());
    project.authenticate(&verdict, false);
}

#[test]
fn live_generation_rejects_missing_tampered_and_stale_admission_before_writing_sources() {
    let project = Project::new();
    project.manifest(Value::Array(vec![Value::Table(toml::Table::from_iter([
        ("name".to_owned(), Value::String(("one").to_owned())),
        ("a".to_owned(), Value::Integer(1_i64)),
        ("expected_return".to_owned(), Value::Integer(2_i64)),
    ]))]));
    let verdict = project.verify(true);
    let owner = PathBuf::from(verdict["run"].as_str().unwrap());
    let admission = owner.join("verdict.toml");
    let run = project.0.join("live");
    fs::create_dir(&run).unwrap();
    let invoke = |path: &Path| {
        project.invoke(
            "live-generate",
            Value::Table(toml::Table::from_iter([
                ("checkout".to_owned(), Value::try_from(&(project.0)).unwrap()),
                ("run".to_owned(), Value::try_from(&(run)).unwrap()),
                ("family_verdicts".to_owned(), Value::Array(vec![Value::try_from(path).unwrap()])),
            ])),
            false,
        );
        assert!(fs::read_dir(&run).unwrap().next().is_none());
    };
    invoke(&project.0.join("missing.toml"));
    let output = owner.join("one/rust-after.wram");
    let original = fs::read(&output).unwrap();
    fs::write(&output, vec![0u8; 0x20000]).unwrap();
    invoke(&admission);
    fs::write(&output, original).unwrap();
    write(&project.0, "crates/port/src/fixture.rs", "// stale candidate\n");
    invoke(&admission);
}

#[test]
fn rust_admission_rejection_precedes_c_and_other_cases_keep_diagnostics() {
    let project = Project::new();
    project.manifest(Value::Array(vec![
        Value::Table(toml::Table::from_iter([
            ("name".to_owned(), Value::String(("reject").to_owned())),
            ("a".to_owned(), Value::Integer(42_i64)),
        ])),
        Value::Table(toml::Table::from_iter([
            ("name".to_owned(), Value::String(("pass").to_owned())),
            ("a".to_owned(), Value::Integer(7_i64)),
        ])),
    ]));
    let verdict = project.verify(false);
    let run = PathBuf::from(verdict["run"].as_str().unwrap());
    assert!(!run.join("reject/c.stdout").exists());
    assert!(run.join("reject/rust.stderr").exists());
    assert!(run.join("pass/c-after.wram").exists());
    assert_eq!(verdict["cases"][0]["case"].as_str(), Some("pass"));
}

#[test]
fn independent_chains_overlap_and_failed_returns_block_descendants_in_both_modes() {
    let project = Project::new();
    let cases:Value=toml::from_str::<Value>("cases=[{name='fail',a=1,expected_return=99},{name='blocked',a=2,continue_from='fail'},{name='grandchild',a=3,continue_from='blocked'},{name='independent',a=7}]").unwrap()["cases"].clone();
    project.manifest(cases);
    for mode in ["fresh", "fork"] {
        let verdict = project.verify_mode(false, mode);
        let run = Path::new(verdict["run"].as_str().unwrap());
        assert!(run.join("fail/c-after.wram").is_file());
        assert!(!run.join("blocked/rust.stdout").exists());
        assert!(!run.join("grandchild/c.stdout").exists());
        assert_eq!(fs::read_to_string(run.join("blocked/blocked.toml")).unwrap(), "blocked_by = \"fail\"\n");
        assert!(run.join("independent/c-after.wram").is_file());
    }
    // The pinned C rendezvous fails if a continuation serializes unrelated
    // roots. Rust retains its complete, input-free captured source contract.
    project.pinned_c(
        r#"#include <stdint.h>
extern uint8_t g_ram[0x20000];
uint16_t Touch(uint16_t a) {
  if (a == 1 || a == 7) {
    char ready[64], other[64];
    snprintf(ready, sizeof(ready), "../ready-%u", a);
    snprintf(other, sizeof(other), "../ready-%u", a == 1 ? 7 : 1);
    FILE *out = fopen(ready, "w"); if (!out) return 65535; fputs("ready", out); fclose(out);
    for (int remaining = 3000; access(other, 0) != 0; --remaining) {
      if (remaining == 0) return 65535;
      usleep(1000);
    }
  }
  g_ram[17] = (uint8_t)a; return a + 1;
}"#,
    );
    project.manifest(
        toml::from_str::<Value>(
            "cases=[{name='root-a',a=1},{name='next-a',a=2,continue_from='root-a'},{name='root-b',a=7}]",
        )
        .unwrap()["cases"]
            .clone(),
    );
    for mode in ["fresh", "fork"] {
        let verdict = project.verify_mode(true, mode);
        project.authenticate(&verdict, true);
        assert_eq!(verdict["cases"][0]["case"].as_str(), Some("root-a"));
        assert_eq!(verdict["cases"][1]["case"].as_str(), Some("next-a"));
    }
}

fn scheduling_project(project: &Project, bounded_delay: bool) {
    write(
        &project.0,
        "peer.rs",
        r#"fn main(){std::fs::write("admitted",b"ready").unwrap();
let deadline=std::time::Instant::now()+std::time::Duration::from_secs(10);
while !std::path::Path::new("release").exists(){assert!(std::time::Instant::now()<deadline,"peer release timeout");std::thread::sleep(std::time::Duration::from_millis(1));}}"#,
    );
    assert!(
        Command::new("rustc")
            .arg(project.0.join("peer.rs"))
            .arg("-o")
            .arg(project.0.join("peer"))
            .status()
            .unwrap()
            .success()
    );
    project.pinned_c(
        r#"#include <stdint.h>
extern uint8_t g_ram[0x20000];
uint16_t Touch(uint16_t a) {
  if (a == 1 || a == 4) {
    for (int remaining = 3000; access("../short/c-after.wram", 0) != 0; --remaining) {
      if (remaining == 0) return 65535;
      usleep(1000);
    }
    FILE *out = fopen("../waiting", "w"); if (!out) return 65535; fputs("ready", out); fclose(out);
    if (a == 4) { usleep(600000); }
    else {
      char directory[4096], peer[8192];
      if (!getcwd(directory, sizeof(directory))) return 65535;
      char *last = strrchr(directory, '/'); if (!last) return 65535; *last = 0;
      snprintf(peer, sizeof(peer), "%s-peer/admitted", directory);
      for (int remaining = 3000; access(peer, 0) != 0; --remaining) {
        if (remaining == 0) return 65535;
        usleep(1000);
      }
    }
  }
  g_ram[17] = (uint8_t)a; return a + 1;
}"#,
    );
    let a = if bounded_delay { 4 } else { 1 };
    project.manifest(
        toml::from_str::<Value>(&format!(
            "cases=[{{name='long',a={a}}},{{name='short',a=2}},{{name='continuation',a=3,continue_from='long'}}]"
        ))
        .unwrap()["cases"]
            .clone(),
    );
}

// Cross the family and process-execution interfaces: both worker threads first run
// real Fixture pairs, then an unrelated caller needs the idle thread's slot.
fn family_with_lease_peer(project: &Project, binary: &Path, mode: &str) -> (Value, Value, f64) {
    let request = toml::to_string(&toml::Table::from_iter([
        ("checkout".into(), Value::try_from(&project.0).unwrap()),
        ("source".into(), Value::try_from(project.0.join("deps/snesrev-sm")).unwrap()),
        ("manifest".into(), "tools/port/fixture.smf".into()),
        ("output".into(), Value::try_from(project.0.join("out")).unwrap()),
        ("root".into(), Value::try_from(project.0.join("cache")).unwrap()),
        ("workers".into(), 2.into()),
        ("case_jobs".into(), 2.into()),
        ("execution".into(), mode.into()),
    ]))
    .unwrap();
    let previous: Vec<_> =
        fs::read_dir(project.0.join("out")).into_iter().flatten().map(|entry| entry.unwrap().path()).collect();
    // Initialize executable identity before submitting the peer's execution
    // request. Hashing a debug controller at the rendezvous would measure its
    // startup cost instead of shared-capacity contention.
    let mut peer = Command::new(binary)
        .arg("exec")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let started = std::time::Instant::now();
    let mut family = Command::new(binary)
        .arg("family-verify")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    family.stdin.take().unwrap().write_all(request.as_bytes()).unwrap();
    let run = loop {
        if let Ok(entries) = fs::read_dir(project.0.join("out"))
            && let Some(run) = entries
                .map(|entry| entry.unwrap().path())
                .find(|path| !previous.contains(path) && path.join("waiting").is_file())
        {
            break run;
        }
        if family.try_wait().unwrap().is_some() || started.elapsed().as_secs() >= 30 {
            let _ = family.kill();
            let _ = peer.kill();
            let _ = peer.wait();
            let output = family.wait_with_output().unwrap();
            panic!("Fixture rendezvous failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    };
    let peer_run = run.with_file_name(format!("{}-peer", run.file_name().unwrap().to_str().unwrap()));
    fs::create_dir(&peer_run).unwrap();
    let peer_request = toml::to_string(&toml::Table::from_iter([
        ("command".into(), Value::try_from(vec![project.0.join("peer")]).unwrap()),
        ("run".into(), Value::try_from(&peer_run).unwrap()),
        ("label".into(), "peer".into()),
        ("root".into(), Value::try_from(project.0.join("cache")).unwrap()),
        ("workers".into(), 2.into()),
        ("timeout".into(), 10.into()),
    ]))
    .unwrap();
    peer.stdin.take().unwrap().write_all(peer_request.as_bytes()).unwrap();
    // Retain the peer's lease through the continuation. Actual execution must
    // still fit in the single remaining slot while helpers may remain alive.
    let output = family.wait_with_output().unwrap();
    write(&peer_run, "release", b"ready");
    let peer_output = peer.wait_with_output().unwrap();
    assert!(peer_output.status.success(), "{}", String::from_utf8_lossy(&peer_output.stderr));
    let receipt: Value = toml::from_str(std::str::from_utf8(&peer_output.stdout).unwrap()).unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let verdict: Value = toml::from_str(std::str::from_utf8(&output.stdout).unwrap()).unwrap();
    assert_eq!(verdict["cases"][0]["case"].as_str(), Some("long"));
    assert_eq!(verdict["cases"][1]["case"].as_str(), Some("short"));
    assert_eq!(verdict["cases"][2]["case"].as_str(), Some("continuation"));
    project.authenticate(&verdict, true);
    (verdict, receipt, started.elapsed().as_secs_f64())
}

#[test]
fn idle_fixture_chain_workers_release_shared_execution_capacity_in_both_modes() {
    let project = match std::env::var_os("SM_SCHEDULING_REGRESSION_OUTPUT") {
        Some(path) => Project::at(PathBuf::from(path), true),
        None => Project::new(),
    };
    scheduling_project(&project, false);
    if let Some(source) = std::env::var_os("SM_SCHEDULING_BASELINE_SOURCE") {
        write(&project.0, "crates/offline-verification/src/fixture_execution.rs", fs::read(source).unwrap());
    }
    let binary = std::env::var_os("SM_SCHEDULING_REGRESSION_BINARY")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_BIN_EXE_sm-pipeline")));
    for mode in ["fresh", "fork"] {
        family_with_lease_peer(&project, &binary, mode);
    }
}

#[test]
#[ignore = "bounded comparison retains actual-C evidence and requires an explicit baseline binary"]
fn measure_fixture_chain_shared_capacity() {
    let output = PathBuf::from(std::env::var_os("SM_SCHEDULING_MEASUREMENT_OUTPUT").expect("output required"));
    let baseline = PathBuf::from(std::env::var_os("SM_SCHEDULING_BASELINE_BINARY").expect("baseline required"));
    let baseline_source =
        PathBuf::from(std::env::var_os("SM_SCHEDULING_BASELINE_SOURCE").expect("baseline source required"));
    assert!(output.is_absolute() && !output.exists(), "use a new absolute measurement directory");
    assert!(baseline.is_absolute() && baseline.is_file());
    fs::create_dir(&output).unwrap();
    let baseline_project = Project::at(output.join("baseline"), true);
    let candidate_project = Project::at(output.join("candidate"), true);
    scheduling_project(&baseline_project, true);
    scheduling_project(&candidate_project, true);
    write(
        &baseline_project.0,
        "crates/offline-verification/src/fixture_execution.rs",
        fs::read(&baseline_source).unwrap(),
    );
    for input in ["crates/port/src/fixture.rs", "tools/port/fixture.smf", "deps/snesrev-sm/src/sm_80.c", "peer.rs"] {
        assert_eq!(
            fs::read(baseline_project.0.join(input)).unwrap(),
            fs::read(candidate_project.0.join(input)).unwrap()
        );
    }
    let candidate = std::env::var_os("SM_SCHEDULING_CANDIDATE_BINARY")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_BIN_EXE_sm-pipeline")));
    // Preserve exact source snapshots for both binaries. Warm each private
    // build cache once with identical Fixture input and generated peer source.
    let variants =
        [("baseline", &baseline_project, baseline.as_path()), ("candidate", &candidate_project, candidate.as_path())];
    let mut rows = Vec::new();
    let mut verdicts = Vec::new();
    for (_, project, binary) in variants {
        let (verdict, _, _) = family_with_lease_peer(project, binary, "fork");
        verdicts.push((project.0.clone(), verdict));
    }
    for mode in ["fresh", "fork"] {
        for (label, project, binary) in variants {
            let (verdict, receipt, seconds) = family_with_lease_peer(project, binary, mode);
            let run = Path::new(verdict["run"].as_str().unwrap());
            rows.push(Value::Table(toml::Table::from_iter([
                ("scheduler".into(), label.into()),
                ("execution".into(), mode.into()),
                ("harness_seconds".into(), seconds.into()),
                ("peer_wait_seconds".into(), receipt["worker_wait_seconds"].clone()),
                (
                    "peer_receipt".into(),
                    sealed(
                        &run.with_file_name(format!("{}-peer", run.file_name().unwrap().to_str().unwrap()))
                            .join("peer.timing.toml"),
                    ),
                ),
                (
                    "verdict".into(),
                    Value::try_from(Path::new(verdict["run"].as_str().unwrap()).join("verdict.toml")).unwrap(),
                ),
            ])));
            verdicts.push((project.0.clone(), verdict));
        }
    }
    let report = toml::Table::from_iter([
        ("schema".into(), "fixture-scheduling-measurement-v1".into()),
        ("baseline_binary".into(), sealed(&baseline)),
        ("baseline_source".into(), sealed(&baseline_source)),
        ("candidate_binary".into(), sealed(&candidate)),
        ("fixture_source".into(), sealed(&candidate_project.0.join("crates/port/src/fixture.rs"))),
        ("peer_source".into(), sealed(&candidate_project.0.join("peer.rs"))),
        ("rows".into(), Value::Array(rows)),
        ("limitations".into(), "bounded synthetic contention, three actual-C Fixture pairs per family, two shared slots, one external Rust process caller, 600 ms root delay, warm build cache, one sample per binary/mode; harness_seconds includes family execution, peer shutdown and authentication; measures stranded capacity rather than full-route throughput".into()),
    ]);
    write(&output, "measurement.toml", toml::to_string_pretty(&report).unwrap());
    eprintln!("{}", toml::to_string_pretty(&report).unwrap());
    drop(baseline_project);
    drop(candidate_project);
    // Cleanup retains exact source paths. Authenticate through the public
    // interface again after both cleanup passes and cache removal.
    for (checkout, verdict) in &verdicts {
        let retained = std::mem::ManuallyDrop::new(Project(checkout.clone(), false));
        retained.authenticate(verdict, true);
    }
}

#[test]
fn fork_parent_is_pristine_and_execution_seals_cannot_be_stripped() {
    let project = Project::new();
    write(&project.0,"deps/snesrev-sm/src/sm_80.c",b"#include <stdint.h>\nextern uint8_t g_ram[0x20000];static int count;uint16_t Touch(uint16_t a){g_ram[17]=(uint8_t)a;return a+(++count);}");
    // Re-pin the intentionally changed actual-C test source.
    let archive = project.0.join("deps/downloads/oracle.tar.gz");
    assert!(
        Command::new("tar")
            .arg("-czf")
            .arg(&archive)
            .arg("-C")
            .arg(project.0.join("deps"))
            .arg("snesrev-sm")
            .status()
            .unwrap()
            .success()
    );
    let pins = project.0.join("tools/dependencies.toml");
    let mut value: Value = toml::from_str(&fs::read_to_string(&pins).unwrap()).unwrap();
    value["dependencies"][0]["sha256"] = format!("{:x}", Sha256::digest(fs::read(archive).unwrap())).into();
    fs::write(pins, toml::to_string(&value).unwrap()).unwrap();
    project.manifest(
        toml::from_str::<Value>("cases=[{name='one',a=1},{name='two',a=2},{name='three',a=3}]").unwrap()["cases"]
            .clone(),
    );
    let mut verdict = project.verify(true);
    project.authenticate(&verdict, true);
    let metrics: Value = toml::from_str(
        &fs::read_to_string(Path::new(verdict["run"].as_str().unwrap()).join("family.metrics.toml")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        metrics["processes"].as_array().unwrap().iter().filter(|p| p["role"].as_str() == Some("rust")).count(),
        3
    );
    assert!(!Path::new(verdict["run"].as_str().unwrap()).join("one/rust.metrics.toml").exists());
    verdict["implementation"].as_table_mut().unwrap().remove("fixture_execution");
    write(Path::new(verdict["run"].as_str().unwrap()), "verdict.toml", toml::to_string(&verdict).unwrap());
    project.authenticate(&verdict, false);
}

struct WriterControl(Option<std::process::Child>);
impl Drop for WriterControl {
    fn drop(&mut self) {
        if let Some(child) = &mut self.0
            && child.try_wait().unwrap().is_none()
        {
            // SAFETY: this test owns the still-unreaped controller.
            unsafe { libc::kill(child.id() as i32, libc::SIGTERM) };
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
            while child.try_wait().unwrap().is_none() {
                if std::time::Instant::now() >= deadline {
                    child.kill().unwrap();
                    child.wait().unwrap();
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
    }
}

// Process observation belongs to the trusted test host, so the synthetic Rust
// candidate does not need unmodeled runtime or foreign-linkage capabilities.
fn processes_with_cwd(directory: &Path) -> Vec<(u32, u32)> {
    fs::read_dir("/proc")
        .unwrap()
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let pid = entry.file_name().to_str()?.parse().ok()?;
            if fs::read_link(entry.path().join("cwd")).ok()? != directory {
                return None;
            }
            let stat = fs::read_to_string(entry.path().join("stat")).ok()?;
            let parent = stat.rsplit_once(')')?.1.split_whitespace().nth(1)?.parse().ok()?;
            Some((pid, parent))
        })
        .collect()
}

fn process_identity(pid: u32) -> (PathBuf, String) {
    let path = PathBuf::from(format!("/proc/{pid}/stat"));
    let stat = fs::read_to_string(&path).unwrap();
    let started = stat.rsplit_once(')').unwrap().1.split_whitespace().nth(19).unwrap().to_owned();
    (path, started)
}

#[test]
fn cancelling_family_reaps_helper_fixture_and_descendant() {
    let project = Project::new();
    write(
        &project.0,
        "crates/port/src/fixture.rs",
        r#"use crate::memory::Wram;
pub fn dispatch(ram:&mut Wram<'_>,name:&str,a:u16,_b:u16)->Result<u32,&'static str>{
if name!="Touch"{return Err("routine");}
if a==1{loop{std::thread::yield_now();}}
ram.set_byte(17,a as u8);Ok(a as u32+1)}"#,
    );
    project.pinned_c(
        r#"#include <stdint.h>
extern uint8_t g_ram[0x20000];
uint16_t Touch(uint16_t a) {
  if (a == 9) {
    FILE *out = fopen("child.pid", "w"); fprintf(out, "%d", getpid()); fclose(out);
    int child = fork(); if (child < 0) return 65535;
    if (child == 0) {
      out = fopen("descendant.pid", "w"); fprintf(out, "%d", getpid()); fclose(out);
    }
    sleep(30);
  }
  g_ram[17] = (uint8_t)a; return a + 1;
}"#,
    );
    project.manifest(
        toml::from_str::<Value>(
            "cases=[{name='sleep',a=1},{name='short',a=2},{name='blocked',a=3,continue_from='sleep'},{name='forked',a=9}]",
        )
        .unwrap()["cases"]
            .clone(),
    );
    let request = toml::to_string(&toml::Table::from_iter([
        ("checkout".into(), Value::try_from(&project.0).unwrap()),
        ("source".into(), Value::try_from(project.0.join("deps/snesrev-sm")).unwrap()),
        ("manifest".into(), "tools/port/fixture.smf".into()),
        ("output".into(), Value::try_from(project.0.join("out")).unwrap()),
        ("root".into(), Value::try_from(project.0.join("cache")).unwrap()),
        ("workers".into(), 4.into()),
        ("case_jobs".into(), 4.into()),
        ("execution".into(), "fork".into()),
    ]))
    .unwrap();
    let mut process = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
        .arg("family-verify")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    process.stdin.take().unwrap().write_all(request.as_bytes()).unwrap();
    let mut writer = WriterControl(Some(process));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    let dir = loop {
        if let Ok(entries) = fs::read_dir(project.0.join("out"))
            && let Some(dir) = entries.filter_map(Result::ok).map(|e| e.path().join("sleep")).find(|p| {
                fs::read_to_string(p.parent().unwrap().join("forked/descendant.pid"))
                    .is_ok_and(|text| text.parse::<u32>().is_ok())
                    && p.parent().unwrap().join("short/c-after.wram").is_file()
            })
        {
            break dir;
        }
        assert!(std::time::Instant::now() < deadline, "cancellation fixture did not start");
        assert!(writer.0.as_mut().unwrap().try_wait().unwrap().is_none(), "cancellation controller stopped early");
        std::thread::sleep(std::time::Duration::from_millis(5));
    };
    let controller = writer.0.as_ref().unwrap().id();
    let helpers = processes_with_cwd(dir.parent().unwrap())
        .into_iter()
        .filter(|&(_, parent)| parent == controller)
        .map(|(pid, _)| pid)
        .collect::<Vec<_>>();
    assert!(helpers.len() >= 5, "active and idle helper pairs were not observed");
    let rust = processes_with_cwd(&dir);
    assert_eq!(rust.len(), 1, "blocked Rust Fixture missing");
    let mut identities = helpers.into_iter().chain([rust[0].0]).map(process_identity).collect::<Vec<_>>();
    for name in ["child.pid", "descendant.pid"] {
        identities.push(process_identity(
            fs::read_to_string(dir.parent().unwrap().join("forked").join(name)).unwrap().parse().unwrap(),
        ));
    }
    unsafe {
        libc::kill(controller as i32, libc::SIGTERM);
    }
    let output = writer.0.take().unwrap().wait_with_output().unwrap();
    assert!(!output.status.success());
    for (path, started) in identities {
        assert!(
            !fs::read_to_string(path).is_ok_and(|stat| {
                stat.rsplit_once(')').unwrap().1.split_whitespace().nth(19) == Some(started.as_str())
            }),
            "helper, Fixture or descendant survived cancellation"
        );
    }
    assert!(!dir.join("c.stdout").exists());
    assert!(dir.parent().unwrap().join("blocked/blocked.toml").is_file());
    let verdict: Value =
        toml::from_str(&fs::read_to_string(dir.parent().unwrap().join("verdict.toml")).unwrap()).unwrap();
    assert!(verdict["error"].as_str().unwrap().contains("interrupted by signal"));
}

#[test]
fn empty_duplicate_and_out_of_bounds_cases_cannot_pass() {
    let project = Project::new();
    for cases in [
        Value::Array(vec![]),
        Value::Array(vec![
            Value::Table(toml::Table::from_iter([("name".to_owned(), Value::String(("same").to_owned()))])),
            Value::Table(toml::Table::from_iter([("name".to_owned(), Value::String(("same").to_owned()))])),
        ]),
    ] {
        project.manifest(cases);
        assert_eq!(project.verify(false)["status"].as_str(), Some("rejected"));
    }
    let overflow = project.0.join("tools/port/overflow.smf");
    let error = binary_fixture::write(
        &overflow,
        "family='overflow'\ncases=[{name='overflow',writes=[{offset=131071,data=[1,2]}]}]",
    )
    .unwrap_err();
    assert!(error.to_string().contains("outside WRAM"));
    assert!(!overflow.exists(), "invalid state must fail before publication or execution");
}

#[test]
fn native_projectile_candidate_is_available_to_the_compiled_host_wrapper() {
    let project = Project::new();
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    if !checkout.join("rom/sm.sfc").exists() {
        return;
    }
    let mut manifest: Value = toml::from_str(
        &binary_fixture::read(&checkout.join("tools/port/observed-projectile-initializers.smf")).unwrap(),
    )
    .unwrap();
    manifest["routines"].as_array_mut().unwrap().truncate(1);
    manifest["routines"][0]["cases"].as_array_mut().unwrap().truncate(1);
    write(&project.0, "one-native.smf", toml::to_string(&manifest).unwrap());
    let verdict = project.invoke(
        "family-verify",
        Value::Table(toml::Table::from_iter([
            ("checkout".into(), Value::try_from(checkout).unwrap()),
            ("source".into(), Value::try_from(checkout.join("deps/snesrev-sm")).unwrap()),
            ("manifest".into(), Value::try_from(project.0.join("one-native.smf")).unwrap()),
            ("output".into(), Value::try_from(project.0.join("out")).unwrap()),
            ("root".into(), Value::try_from(project.0.join("workers")).unwrap()),
            ("workers".into(), 2.into()),
        ])),
        true,
    );
    assert_eq!(verdict["verified_cases"].as_integer(), Some(1));
}

#[test]
fn changed_current_manifest_cannot_authenticate_prior_family_admission() {
    let project = Project::new();
    project.manifest(Value::Array(vec![Value::Table(toml::Table::from_iter([
        ("name".to_owned(), Value::String("one".to_owned())),
        ("a".to_owned(), Value::Integer(1)),
        ("expected_return".to_owned(), Value::Integer(2)),
    ]))]));
    let verdict = project.verify(true);
    project.authenticate(&verdict, true);
    let path = project.0.join("tools/port/fixture.smf");
    let original = fs::read(&path).unwrap();
    let mut manifest: Value = toml::from_str(&binary_fixture::read(&path).unwrap()).unwrap();
    manifest["routines"][0]["cases"][0]["a"] = 2.into();
    write(&project.0, "tools/port/fixture.smf", toml::to_string(&manifest).unwrap());
    project.authenticate(&verdict, false);
    let mut stripped = verdict.clone();
    stripped["implementation"].as_table_mut().unwrap().remove("manifest");
    let verdict_path = Path::new(verdict["run"].as_str().unwrap()).join("verdict.toml");
    fs::write(&verdict_path, toml::to_string(&stripped).unwrap()).unwrap();
    project.authenticate(&stripped, false);
    let mut rebound = verdict.clone();
    rebound["implementation"]["manifest"]["file"] =
        Value::try_from(Path::new(verdict["run"].as_str().unwrap()).join("manifest.smf")).unwrap();
    fs::write(&verdict_path, toml::to_string(&rebound).unwrap()).unwrap();
    project.authenticate(&rebound, false);
    fs::write(verdict_path, toml::to_string(&verdict).unwrap()).unwrap();
    fs::write(path, original).unwrap();
    project.authenticate(&verdict, true);
}

#[test]
fn changed_since_verifies_one_actual_c_family_without_visiting_unrelated_history() {
    let project = Project::new();
    project.manifest(Value::Array(vec![Value::Table(toml::Table::from_iter([
        ("name".into(), "one".into()),
        ("a".into(), 1.into()),
        ("expected_return".into(), 2.into()),
    ]))]));
    let first_manifest = project.0.join("tools/port/fixture.smf");
    let mut other: Value = toml::from_str(&binary_fixture::read(&first_manifest).unwrap()).unwrap();
    other["family"] = "other".into();
    other["rust_module"] = "crates/port/src/other.rs".into();
    write(&project.0, "tools/port/other.smf", toml::to_string(&other).unwrap());
    write(&project.0, "crates/port/src/other.rs", fs::read(project.0.join("crates/port/src/fixture.rs")).unwrap());
    write(
        &project.0,
        "tools/port/batch.toml",
        "[families]\nfixture='family'\nother='family'\n[profiles]\ngameplay=['fixture','other']\n",
    );
    write(&project.0, ".gitignore", "/batch/\n/cache/\n/out/\n");
    assert!(Command::new("git").args(["init", "-q"]).arg(&project.0).status().unwrap().success());
    assert!(Command::new("git").args(["add", "."]).current_dir(&project.0).status().unwrap().success());
    assert!(
        Command::new("git")
            .args(["-c", "user.name=Fixture", "-c", "user.email=fixture@example.test", "commit", "-qm", "source"])
            .current_dir(&project.0)
            .status()
            .unwrap()
            .success()
    );
    let request = |operation: &str, changed: bool| {
        let mut request = Value::Table(toml::Table::from_iter([
            ("checkout".into(), Value::try_from(&project.0).unwrap()),
            ("operation".into(), operation.into()),
            ("batch".into(), Value::try_from(project.0.join("batch/batch.toml")).unwrap()),
            ("root".into(), Value::try_from(project.0.join("cache")).unwrap()),
            ("workers".into(), 2.into()),
        ]));
        if changed {
            request.as_table_mut().unwrap().insert("changed_since".into(), "HEAD".into());
        }
        request
    };
    let initial = project.invoke("batch", request("verify", false), true);
    for name in ["fixture", "other"] {
        assert_eq!(initial["families"][name]["status"].as_str(), Some("offline-verified"));
    }
    let count_runs =
        |name: &str| fs::read_dir(project.0.join(format!("batch/families/{name}/evidence"))).unwrap().count();
    assert_eq!(count_runs("fixture"), 1);
    assert_eq!(count_runs("other"), 1);

    // A semantic-preserving leaf edit invalidates exactly one current source seal.
    let candidate = project.0.join("crates/port/src/fixture.rs");
    let source = fs::read_to_string(&candidate).unwrap();
    fs::write(&candidate, format!("{source}\n// focused leaf revision\n")).unwrap();
    let plan = project.invoke("batch", request("plan", true), true);
    assert_eq!(plan["selected"].as_array().unwrap().len(), 1);
    assert_eq!(plan["selected"][0]["family"].as_str(), Some("fixture"));
    assert_eq!(plan["excluded"][0]["family"].as_str(), Some("other"));

    let other_verdict = PathBuf::from(initial["families"]["other"]["verdict"].as_str().unwrap());
    let mut retained: Value = toml::from_str(&fs::read_to_string(&other_verdict).unwrap()).unwrap();
    let unrelated_artifact = other_verdict.parent().unwrap().join("candidate.rs");
    let unrelated_original = fs::read(&unrelated_artifact).unwrap();
    // Deliberately corrupt unrelated history. Focused verification must leave it
    // untouched; explicit current authentication still rejects this evidence.
    fs::write(&unrelated_artifact, b"tampered unrelated retained artifact").unwrap();
    project.authenticate(&retained, false);
    let focused = project.invoke("batch", request("verify", true), true);
    assert_eq!(focused["selected"].as_array().unwrap(), &[Value::String("fixture".into())]);
    assert_eq!(focused["families"]["fixture"]["status"].as_str(), Some("offline-verified"));
    assert_eq!(focused["families"]["other"], initial["families"]["other"]);
    assert_eq!(count_runs("fixture"), 2);
    assert_eq!(count_runs("other"), 1);
    assert_ne!(focused["families"]["fixture"]["verdict"], initial["families"]["fixture"]["verdict"]);
    fs::write(&unrelated_artifact, unrelated_original).unwrap();
    project.authenticate(&retained, true);

    let reused = project.invoke("batch", request("verify", true), true);
    assert_eq!(reused["families"]["fixture"]["status"].as_str(), Some("reused"));
    assert_eq!(reused["families"]["fixture"]["verdict"], focused["families"]["fixture"]["verdict"]);
    assert_eq!(count_runs("fixture"), 2);
    assert_eq!(count_runs("other"), 1);

    fs::write(&candidate, source.replace("a as u32+1", "a as u32+2")).unwrap();
    let rejected = project.invoke("batch", request("verify", true), false);
    assert_eq!(rejected["families"]["fixture"]["status"].as_str(), Some("rejected"));
    assert_eq!(rejected["families"]["other"], initial["families"]["other"]);
    assert_eq!(count_runs("fixture"), 3);
    assert_eq!(count_runs("other"), 1);
    // Keep the restored unrelated verdict live so evidence remains valid at exit.
    retained = toml::from_str(&fs::read_to_string(&other_verdict).unwrap()).unwrap();
    project.authenticate(&retained, true);
}

#[test]
fn batch_reuse_reports_current_elapsed_time_and_retains_original_run_time() {
    let project = Project::new();
    project.manifest(Value::Array(vec![Value::Table(toml::Table::from_iter([
        ("name".into(), "one".into()),
        ("a".into(), 1.into()),
        ("expected_return".into(), 2.into()),
    ]))]));
    write(&project.0, "tools/port/batch.toml", "[families]\nfixture='family'\n[profiles]\ngameplay=['fixture']\n");
    let (first, _) = project.measured_batch(false, "disabled", true);
    assert_eq!(first["families"]["fixture"]["status"].as_str(), Some("offline-verified"));
    let read_metrics = |result: &Value| -> pipeline_model::metrics::Receipt {
        let receipt: pipeline_model::metrics::Receipt =
            toml::from_str(&fs::read_to_string(result["metrics"].as_str().unwrap()).unwrap()).unwrap();
        receipt.validate().unwrap();
        receipt
    };
    let first_metrics = read_metrics(&first);
    assert!(!first_metrics.enabled);
    assert_eq!(first_metrics.resources.rss_samples, 0);
    for name in [
        "batch-planning",
        "registered-input-authentication",
        "family-input-authentication",
        "family-builds",
        "compiler-preparation",
        "compiler-cache",
        "shared-capacity-wait",
        "fixture-execution",
        "fixture-comparison",
        "family-publication",
        "family-evidence-admission",
        "batch-publication",
        "batch-scoped-cleanup",
    ] {
        assert!(
            first_metrics.stages.iter().any(|stage| stage.name == name && stage.outcome == "success"),
            "missing successful stage: {name}"
        );
    }
    // Batch timing is advisory, outside authenticated Fixture evidence. A large
    // retained duration makes copying it distinguishable from any current run.
    let batch_path = project.0.join("batch/batch.toml");
    let mut retained: Value = toml::from_str(&fs::read_to_string(&batch_path).unwrap()).unwrap();
    retained["families"]["fixture"]["seconds"] = 86_400.0.into();
    fs::write(&batch_path, toml::to_string(&retained).unwrap()).unwrap();
    for _ in 0..2 {
        let (reused, invocation_seconds) = project.measured_batch(false, "disabled", true);
        let record = &reused["families"]["fixture"];
        assert_eq!(record["status"].as_str(), Some("reused"));
        assert_eq!(record["verdict"], first["families"]["fixture"]["verdict"]);
        let current = record["seconds"].as_float().unwrap();
        assert!(
            current > 0.0 && current <= invocation_seconds,
            "reported {current}s for {invocation_seconds}s invocation"
        );
        assert_eq!(record["original_run_seconds"].as_float(), Some(86_400.0));
        let authentication = record["reuse_authentication_seconds"].as_float().unwrap();
        assert!(authentication > 0.0 && authentication <= current);
        let receipt = read_metrics(&reused);
        assert!(receipt.processes.is_empty(), "unchanged reuse launched a producer");
        assert!(receipt.stages.iter().any(|stage| stage.name == "reuse-authentication" && stage.outcome == "success"));
        assert!(receipt.scopes.iter().any(|scope| {
            scope
                .configuration
                .get("unavailable-stage:family-verification")
                .is_some_and(|reason| reason.contains("retained evidence"))
        }));
        println!(
            "reuse current={current:.6}s authentication={authentication:.6}s historical=86400s invocation={invocation_seconds:.6}s"
        );
    }

    let candidate = project.0.join("crates/port/src/fixture.rs");
    let source = fs::read_to_string(&candidate).unwrap();
    fs::write(&candidate, source.replace("a as u32+1", "a as u32+2")).unwrap();
    let (rejected, _) = project.measured_batch(false, "disabled", false);
    let record = &rejected["families"]["fixture"];
    assert_eq!(record["status"].as_str(), Some("rejected"));
    assert!(record["seconds"].as_float().unwrap() >= record["reuse_authentication_seconds"].as_float().unwrap());
    assert!(record["reuse_authentication_error"].as_str().unwrap().contains("changed"));
    let rejected_metrics = read_metrics(&rejected);
    assert_eq!(rejected_metrics.outcome, "failed");
    for name in ["reuse-authentication", "fixture-comparison", "family-verification"] {
        assert!(
            rejected_metrics.stages.iter().any(|stage| stage.name == name && stage.outcome == "failed"),
            "failed stage missing: {name}"
        );
    }

    fs::write(&candidate, source).unwrap();
    let (fresh, _) = project.measured_batch(true, "disabled", true);
    assert_eq!(fresh["families"]["fixture"]["status"].as_str(), Some("offline-verified"));
    assert_ne!(fresh["families"]["fixture"]["verdict"], first["families"]["fixture"]["verdict"]);
    assert!(fresh["families"]["fixture"].get("original_run_seconds").is_none());
    assert!(fresh["families"]["fixture"].get("reuse_authentication_seconds").is_none());
    for deduplicate in [false, true] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"));
        command.args(["cleanup", "--root"]).arg(project.0.join("batch")).arg("--apply");
        if deduplicate {
            command.arg("--deduplicate");
        }
        let output = command.output().unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let cleanup: Value = toml::from_str(std::str::from_utf8(&output.stdout).unwrap()).unwrap();
        println!("post-control cleanup deduplicate={deduplicate} bytes={}", cleanup["bytes"]);
    }
    for result in [&first, &fresh] {
        let verdict: Value =
            toml::from_str(&fs::read_to_string(result["families"]["fixture"]["verdict"].as_str().unwrap()).unwrap())
                .unwrap();
        project.authenticate(&verdict, true);
    }
}

#[test]
fn interrupted_batch_reports_current_time_and_recovers_shared_capacity() {
    use std::time::{Duration, Instant};
    let project = Project::new();
    project.manifest(toml::from_str::<Value>("cases=[{name='one',a=1,expected_return=2}]").unwrap()["cases"].clone());
    write(&project.0, "tools/port/batch.toml", "[families]\nfixture='family'\n[profiles]\ngameplay=['fixture']\n");
    let candidate = project.0.join("crates/port/src/fixture.rs");
    let original = fs::read(&candidate).unwrap();
    write(
        &project.0,
        "crates/port/src/fixture.rs",
        r#"use crate::memory::Wram;
pub fn dispatch(_ram:&mut Wram<'_>,_name:&str,_a:u16,_b:u16)->Result<u32,&'static str>{
loop{std::thread::yield_now();}}"#,
    );
    let (child, started) = project.spawn_measured_batch(false, "disabled");
    let mut writer = WriterControl(Some(child));
    let evidence = project.0.join("batch/families/fixture/evidence");
    let deadline = Instant::now() + Duration::from_secs(30);
    let directory = loop {
        if let Ok(entries) = fs::read_dir(&evidence)
            && let Some(directory) = entries
                .filter_map(Result::ok)
                .map(|entry| entry.path().join("one"))
                .find(|path| path.join("rust.stdout").is_file() && !processes_with_cwd(path).is_empty())
        {
            break directory;
        }
        assert!(Instant::now() < deadline, "batch interruption Fixture did not start");
        assert!(writer.0.as_mut().unwrap().try_wait().unwrap().is_none(), "batch returned before interruption control");
        std::thread::sleep(Duration::from_millis(10));
    };
    let rust = processes_with_cwd(&directory);
    assert_eq!(rust.len(), 1, "admitted Rust Fixture process missing");
    let identities = [rust[0].0, rust[0].1].into_iter().map(process_identity).collect::<Vec<_>>();
    // SAFETY: the controller is still owned and unreaped.
    assert_eq!(unsafe { libc::kill(writer.0.as_ref().unwrap().id() as i32, libc::SIGTERM) }, 0);
    let deadline = Instant::now() + Duration::from_secs(5);
    while writer.0.as_mut().unwrap().try_wait().unwrap().is_none() {
        assert!(Instant::now() < deadline, "batch did not finish after interruption");
        std::thread::sleep(Duration::from_millis(10));
    }
    let output = writer.0.take().unwrap().wait_with_output().unwrap();
    let invocation_seconds = started.elapsed().as_secs_f64();
    assert!(!output.status.success());
    let result: Value = toml::from_str(std::str::from_utf8(&output.stdout).unwrap()).unwrap();
    let record = &result["families"]["fixture"];
    assert_eq!(record["status"].as_str(), Some("interrupted"));
    let seconds = record["seconds"].as_float().unwrap();
    assert!(seconds > 0.0 && seconds <= invocation_seconds);
    let metrics: pipeline_model::metrics::Receipt =
        toml::from_str(&fs::read_to_string(result["metrics"].as_str().unwrap()).unwrap()).unwrap();
    metrics.validate().unwrap();
    assert_eq!(metrics.outcome, "interrupted");
    assert!(
        metrics.stages.iter().any(|stage| stage.name == "batch-family-verification" && stage.outcome == "interrupted")
    );
    assert!(
        metrics.scopes.iter().any(|scope| scope.configuration.contains_key("unavailable-stage:fixture-comparison"))
    );
    assert!(metrics.processes.iter().all(|process| process.reap_count == 1));
    for (path, start) in identities {
        assert!(
            !fs::read_to_string(path).is_ok_and(|stat| {
                let fields = stat.rsplit_once(')').unwrap().1.split_whitespace().collect::<Vec<_>>();
                fields[19] == start && !matches!(fields[0], "Z" | "X" | "x")
            }),
            "Fixture writer survived cancellation"
        );
    }
    fs::write(candidate, original).unwrap();
    let (recovered, _) = project.measured_batch(true, "disabled", true);
    assert_eq!(recovered["families"]["fixture"]["status"].as_str(), Some("offline-verified"));
    let verdict: Value =
        toml::from_str(&fs::read_to_string(recovered["families"]["fixture"]["verdict"].as_str().unwrap()).unwrap())
            .unwrap();
    for deduplicate in [false, true] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"));
        command.args(["cleanup", "--root"]).arg(project.0.join("batch")).arg("--apply");
        if deduplicate {
            command.arg("--deduplicate");
        }
        let output = command.output().unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let cleanup: Value = toml::from_str(std::str::from_utf8(&output.stdout).unwrap()).unwrap();
        println!("interruption cleanup deduplicate={deduplicate} bytes={}", cleanup["bytes"]);
    }
    project.authenticate(&verdict, true);
    println!("interrupted current={seconds:.6}s invocation={invocation_seconds:.6}s; fresh recovery authenticated");
}

#[test]
fn registered_admission_binds_computation_instead_of_caller_kind_hints() {
    use offline_verification::{BatchAdmission, CatalogRequest, FamilyEvidence, HostInputs, OfflineFamilies};
    let project = match std::env::var_os("SM_DEPENDENCY_IDENTITY_ROOT") {
        Some(root) => Project::at(PathBuf::from(root), true),
        None => Project::new(),
    };
    let candidate = project.0.join("crates/port/src/fixture.rs");
    let original = fs::read_to_string(&candidate).unwrap();
    fs::write(
        &candidate,
        original.replace("a as u32+1", "crate::helper::add_one(a)")
            + "\n// unrelated include!(\"unexpected.rs\")\nconst PROSE: &str = \"unrelated include_str!(data)\";\n",
    )
    .unwrap();
    write(
        &project.0,
        "crates/port/src/helper.rs",
        "#[path=\"extra_impl.rs\"] mod owned; pub fn add_one(a:u16)->u32 { owned::add(a) }",
    );
    write(&project.0, "crates/port/src/extra_impl.rs", "pub fn add(a:u16)->u32 { crate::adjust::add(a) }");
    write(&project.0, "crates/port/src/adjust.rs", "pub fn add(a:u16)->u32 { a as u32 + 1 }");
    write(&project.0, "crates/port/src/unrelated.rs", "compile_error!(\"prose entered the compiler closure\");");
    project.manifest(Value::Array(vec![Value::Table(toml::Table::from_iter([
        ("name".into(), "one".into()),
        ("a".into(), 1.into()),
        ("expected_return".into(), 2.into()),
    ]))]));
    let configuration = "[families]\nfixture='family'\n[profiles]\ngameplay=['fixture']\n";
    write(&project.0, "tools/port/batch.toml", configuration);
    let (first, _) = project.measured_batch(false, "disabled", true);
    assert_eq!(first["families"]["fixture"]["status"].as_str(), Some("offline-verified"));
    let verdict = PathBuf::from(first["families"]["fixture"]["verdict"].as_str().unwrap());
    let dependencies =
        tool_execution::builds::dep_paths(&fs::read_to_string(verdict.parent().unwrap().join("compiler.d")).unwrap())
            .unwrap();
    for name in ["fixture.rs", "helper.rs", "extra_impl.rs", "adjust.rs"] {
        assert!(
            dependencies.iter().any(|path| Path::new(path).file_name().is_some_and(|part| part == name)),
            "actual compiler omitted {name}"
        );
    }
    assert!(
        !dependencies.iter().any(|path| Path::new(path).ends_with("unrelated.rs")),
        "prose became an actual compiler input"
    );
    let offline = OfflineFamilies::new(
        HostInputs::new(
            source_stamps::HOST_SOURCES,
            format!("{:x}", Sha256::digest(fs::read(env!("CARGO_BIN_EXE_sm-pipeline")).unwrap())),
        )
        .unwrap(),
    );
    let required = vec!["fixture".into()];
    let admit = |kind: &str, claimed_manifest: Option<&str>| -> anyhow::Result<_> {
        let catalog = offline.capture_catalog(CatalogRequest::new(&project.0)?)?;
        let facts = catalog.family("fixture")?;
        offline.authenticate_batch(BatchAdmission {
            catalog: &catalog,
            required: &required,
            source: project.0.join("deps/snesrev-sm"),
            evidence: vec![FamilyEvidence {
                family: "fixture".into(),
                verdict: verdict.clone(),
                kind: kind.into(),
                manifest_sha256: claimed_manifest.unwrap_or(facts.manifest_sha256()).into(),
            }],
        })
    };
    assert!(admit("family", None).is_ok());
    assert!(admit("family", Some(&"0".repeat(64))).is_err(), "claimed content hash admitted another requirement");
    // Alias catalog labels with the same complete recipe may reuse evidence.
    write(&project.0, "tools/port/batch.toml", configuration.replace("='family'", "='offline-family'"));
    assert!(admit("offline-family", None).is_ok());
    write(&project.0, "tools/port/batch.toml", configuration.replace("='family'", "='callback'"));
    let different_recipe = admit("callback", None);
    // Restore before asserting so even the red run retains authenticatable
    // original evidence and no changed registration masquerades as its proof.
    write(&project.0, "tools/port/batch.toml", configuration);
    assert!(different_recipe.is_err(), "caller kind hint replaced the admitted recipe contract");

    for name in [
        "crates/port/src/fixture.rs",
        "crates/port/src/adjust.rs",
        "crates/offline-verification/src/family.rs",
        "Cargo.lock",
    ] {
        let path = project.0.join(name);
        let original = fs::read(&path).unwrap();
        fs::write(&path, [original.as_slice(), b"\n// changed exact source input\n"].concat()).unwrap();
        let changed = admit("family", None);
        fs::write(&path, &original).unwrap();
        assert!(changed.is_err(), "changed candidate/comparison source was accepted: {name}");
    }
    let manifest = project.0.join("tools/port/fixture.smf");
    let original = fs::read(&manifest).unwrap();
    project.manifest(Value::Array(vec![Value::Table(toml::Table::from_iter([
        ("name".into(), "another-case".into()),
        ("a".into(), 1.into()),
        ("expected_return".into(), 2.into()),
    ]))]));
    let changed = admit("family", None);
    fs::write(&manifest, &original).unwrap();
    assert!(changed.is_err(), "current complete requirements were replaced by a retained declaration");
    let dependency = project.0.join("crates/port/src/extra_impl.rs");
    let original = fs::read(&dependency).unwrap();
    fs::remove_file(&dependency).unwrap();
    let missing = admit("family", None);
    fs::write(&dependency, &original).unwrap();
    assert!(missing.is_err(), "missing current compiler closure was accepted");
    assert!(admit("family", None).is_ok());
}

#[test]
fn native_batch_reuses_authenticated_runs_and_rejects_stale_candidate() {
    let project = Project::new();
    project.manifest(Value::Array(vec![Value::Table(toml::Table::from_iter([
        ("name".to_owned(), Value::String(("one").to_owned())),
        ("a".to_owned(), Value::Integer(1_i64)),
        ("expected_return".to_owned(), Value::Integer(2_i64)),
    ]))]));
    write(
        &project.0,
        "tools/port/batch.toml",
        "[families]\nfixture = \"family\"\n[profiles]\ngameplay = [\"fixture\"]\n",
    );
    let first = project.batch(true);
    assert_eq!(first["families"]["fixture"]["status"].as_str(), Some("offline-verified"));
    let reused = project.batch(true);
    assert_eq!(reused["families"]["fixture"]["status"].as_str(), Some("reused"));
    assert_eq!(first["families"]["fixture"]["verdict"], reused["families"]["fixture"]["verdict"]);
    let candidate = project.0.join("crates/port/src/fixture.rs");
    let text = fs::read_to_string(&candidate).unwrap();
    fs::write(candidate, text.replace("a as u32+1", "a as u32+2")).unwrap();
    let stale = project.batch(false);
    assert_eq!(stale["families"]["fixture"]["status"].as_str(), Some("rejected"));
    assert_ne!(first["families"]["fixture"]["verdict"], stale["families"]["fixture"]["verdict"]);
}

#[test]
fn native_batch_lock_conflicts_do_not_block_independent_batches() {
    let project = Project::new();
    project.manifest(Value::Array(vec![Value::Table(toml::Table::from_iter([
        ("name".to_owned(), Value::String(("one").to_owned())),
        ("a".to_owned(), Value::Integer(1_i64)),
    ]))]));
    write(
        &project.0,
        "tools/port/batch.toml",
        "[families]\nfixture = \"family\"\n[profiles]\ngameplay = [\"fixture\"]\n",
    );
    fs::create_dir(project.0.join("batch")).unwrap();
    let held = fs::OpenOptions::new().create(true).append(true).open(project.0.join("batch/batch.lock")).unwrap();
    held.lock().unwrap();
    project.batch(false);
    let other = project.invoke(
        "batch",
        Value::Table(toml::Table::from_iter([
            ("checkout".to_owned(), Value::try_from(&(project.0)).unwrap()),
            ("operation".to_owned(), Value::String(("verify").to_owned())),
            ("batch".to_owned(), Value::try_from(project.0.join("other/batch.toml")).unwrap()),
            ("families".to_owned(), Value::Array(vec![Value::String(("fixture").to_owned())])),
            ("root".to_owned(), Value::try_from(project.0.join("cache")).unwrap()),
            ("workers".to_owned(), Value::Integer(2_i64)),
        ])),
        true,
    );
    assert_eq!(other["status"].as_str(), Some("complete"));
    drop(held);
    assert_eq!(project.batch(true)["status"].as_str(), Some("complete"));
}

#[test]
fn coordinator_authenticates_verification_and_derives_failure_diagnosis() {
    let project = Project::new();
    project.manifest(Value::Array(vec![Value::Table(toml::Table::from_iter([
        ("name".into(), "one".into()),
        ("a".into(), 1.into()),
        ("expected_return".into(), 2.into()),
    ]))]));
    write(&project.0, "tools/port/batch.toml", "[families]\nfixture='family'\n[profiles]\ngameplay=['fixture']\n");
    project.initialize_git();
    let candidate = project.0.join("crates/port/src/fixture.rs");
    let source = fs::read_to_string(&candidate).unwrap();
    fs::write(&candidate, format!("{source}\n// changed leaf\n")).unwrap();
    let mut next = Value::Table(toml::Table::from_iter([
        ("operation".into(), "next".into()),
        ("checkout".into(), Value::try_from(&project.0).unwrap()),
        ("effort".into(), "coordinator".into()),
        ("coordinator".into(), "fixture-owner".into()),
        ("translations".into(), false.into()),
        ("available_slots".into(), 1.into()),
        ("changed_since".into(), "HEAD".into()),
    ]));
    let ready = project.invoke("assign", next.clone(), true)["queue"]["assignment"].clone();
    assert_eq!(ready["job"]["kind"].as_str(), Some("verify"));
    project.start_assignment(&ready);
    project.batch(true);
    let batch = project.0.join("batch/batch.toml");
    let placeholder = sealed(&batch);
    let result = finish(
        &ready["claim"],
        Value::Table(toml::Table::from_iter([
            ("result".into(), "verified".into()),
            ("evidence".into(), Value::Array(vec![sealed(&batch)])),
            ("authentication".into(), placeholder.clone()),
        ])),
    );
    let completed = project.assign("record", Some(result.clone()), true);
    assert_eq!(completed["jobs"][0]["state"].as_str(), Some("complete"));
    assert_ne!(completed["jobs"][0]["result"]["authentication"], placeholder);
    let repeated = project.assign("record", Some(result), true);
    assert_eq!(repeated["jobs"][0]["result"], completed["jobs"][0]["result"]);

    fs::write(&candidate, source.replace("a as u32+1", "a as u32+2")).unwrap();
    let ready = project.invoke("assign", next.clone(), true)["queue"]["assignment"].clone();
    project.start_assignment(&ready);
    project.batch(false);
    let failed = finish(
        &ready["claim"],
        Value::Table(toml::Table::from_iter([
            ("result".into(), "failed".into()),
            ("error".into(), "Rust differs from actual C".into()),
            ("evidence".into(), Value::Array(vec![sealed(&batch)])),
            ("followups".into(), Value::Array(vec![])),
        ])),
    );
    project.assign("record", Some(failed), true);
    next["changed_since"] = "HEAD".into();
    let diagnosis = project.invoke("assign", next, true)["queue"]["assignment"].clone();
    assert_eq!(diagnosis["job"]["kind"].as_str(), Some("diagnose"));
    assert!(diagnosis["job"]["instructions"].as_str().unwrap().contains("60 seconds"));
    assert!(batch.is_file());
}

struct IsolatedWorktree<'a> {
    project: &'a Project,
    path: PathBuf,
}
impl Drop for IsolatedWorktree<'_> {
    fn drop(&mut self) {
        if self.path.join(".git").exists() {
            self.project.git(&["worktree", "remove", "--force", self.path.to_str().unwrap()]);
        }
    }
}

#[test]
fn coordinator_promotes_research_runs_required_gates_and_integrates_with_cleanup() {
    let rom = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rom/sm.sfc");
    if !rom.is_file() {
        eprintln!("coordinator native exercise control requires the repository's local ROM fixture");
        return;
    }
    let project = Project::new();
    let root = &project.0;
    project.manifest(Value::Array(vec![Value::Table(toml::Table::from_iter([
        ("name".into(), "one".into()),
        ("a".into(), 1.into()),
        ("expected_return".into(), 2.into()),
    ]))]));
    write(root, "tools/port/batch.toml", "[families]\nfixture='family'\n[profiles]\ngameplay=['fixture']\n");
    write(
        root,
        "deps/snesrev-sm/src/sm_86.c",
        concat!(
            "#include <stdlib.h>\n",
            "typedef unsigned short uint16; typedef unsigned int uint32; typedef unsigned char uint8;\n",
            "extern int snes_frame_counter; uint16 eproj_id[2]={0x93ca};\n",
            "static void InventoryHit(unsigned kind,uint16 address);\n",
            "int Observed(void) { // 0x8693ca\n  return 1;\n}\n",
            "static void CallEprojPreInstr(uint32 ea, uint16 k) { (void)ea; (void)k; }\n",
            "static const uint8 *CallEprojInstr(uint32 ea, uint16 k, const uint8 *j) { (void)ea; (void)k; return j; }\n",
            "static void CallEprojInit(uint32 ea, uint16 j) { if(ea==0x8693ca&&j==0&&Observed()!=1)exit(6); }\n",
            "static void EprojRunOne(uint16 k) { CallEprojInit(0x8693ca,k); }\n",
            "void RunOne(void) { EprojRunOne(0); }\n",
        ),
    );
    fs::create_dir_all(root.join("deps/snesrev-sm/src/snes")).unwrap();
    let archive = root.join("deps/downloads/oracle.tar.gz");
    assert!(
        Command::new("tar")
            .arg("-czf")
            .arg(&archive)
            .arg("-C")
            .arg(root.join("deps"))
            .arg("snesrev-sm")
            .status()
            .unwrap()
            .success()
    );
    write(
        root,
        "tools/dependencies.toml",
        format!(
            "schema_version=1\n[[dependencies]]\nname='snesrev-sm'\narchive='oracle.tar.gz'\nrevision='test-pinned-source'\nsha256={:?}\n",
            sealed(&archive)["sha256"].as_str().unwrap()
        ),
    );
    let host = concat!(
        "#include <stdio.h>\n#include <stdlib.h>\n",
        "typedef unsigned short uint16; typedef unsigned int uint32; typedef unsigned char uint8;\n",
        "int snes_frame_counter,decompression_calls; extern void RunOne(void);\n",
        "void DecompressToMem(uint32 src,uint8 *dst){(void)src;(void)dst;}\n",
        "extern void __real_DecompressToMem(uint32 src,uint8 *dst);\n",
        "static void SaveRoutineState(void) {}\n// Linker wrapping\n",
        "void __wrap_DecompressToMem(uint32 src,uint8 *dst) {}\n// The host has no audio thread\n",
        "int main(int argc,char **argv){\n",
        "  if(argc!=5||atoi(argv[3])!=1)return 2;\n",
        "  FILE *rom=fopen(argv[1],\"rb\");if(!rom||fgetc(rom)==EOF)return 3;fclose(rom);\n",
        "  FILE *inputs=fopen(argv[2],\"rb\");if(!inputs||fgetc(inputs)==EOF)return 4;\n",
        "  RunOne();FILE *terminal=fopen(argv[4],\"wb\");if(!terminal)return 5;fclose(terminal);\n",
        "  fclose(inputs);\n  return 0;\n}\n",
    );
    write(root, "tools/decomp/headless.c", host);
    write(root, "tools/decomp/headless_frame.h", "// tiny native diagnostic frame contract\n");
    fs::create_dir_all(root.join("rom")).unwrap();
    fs::copy(&rom, root.join("rom/sm.sfc")).unwrap();
    write(root, "movies/super-metroid-any-3653.inputs", "native coordinator diagnostic input\n");
    write(
        root,
        "Cargo.toml",
        "[workspace]\n[package]\nname='fixture-port'\nversion='0.1.0'\nedition='2024'\n[lib]\npath='crates/port/src/lib.rs'\n",
    );
    write(root, "crates/port/src/lib.rs", "pub mod memory;\npub mod fixture;\n");
    let candidate = root.join("crates/port/src/fixture.rs");
    let mut source = fs::read_to_string(&candidate).unwrap();
    source.push_str("\n#[cfg(test)] mod tests { #[test] fn candidate_returns() { let mut bytes=vec![0;0x20000];let mut ram=crate::memory::Wram::new(&mut bytes).unwrap();assert_eq!(super::dispatch(&mut ram,\"Touch\",1,0),Ok(2));assert_eq!(ram.byte(17),1); } }\n");
    fs::write(&candidate, &source).unwrap();
    assert!(
        Command::new("cargo").args(["generate-lockfile", "--offline"]).current_dir(root).status().unwrap().success()
    );
    write(
        root,
        ".scratch/work/issues/01-ready.md",
        "# Dependencies ready\nStatus: resolved\nBlocked by: None\nType: task\n",
    );
    let origin = ".scratch/work/issues/02-translate.md";
    write(root, origin, "# Translate Observed\nStatus: open\nBlocked by: 01\nTriage: ready-for-agent\nType: task\n");
    project.initialize_git();
    let inventory = root.join("out/inventory.toml");
    let output = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
        .arg("source-inventory")
        .arg("--output")
        .arg(&inventory)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let output = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
        .args(["projectile-inventory", "--frames", "1", "--output"])
        .arg(root.join("out/exercise"))
        .env("SM_PIPELINE_ROOT", root.join("cache"))
        .env("SM_PIPELINE_WORKERS", "2")
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    let exercise: Value = toml::from_str(std::str::from_utf8(&output.stdout).unwrap()).unwrap();
    let exercise_path = Path::new(exercise["evidence"].as_str().unwrap()).join("verdict.toml");
    let gate = |argv: Vec<String>, integration: bool| {
        Value::Table(toml::Table::from_iter([
            ("argv".into(), Value::try_from(argv).unwrap()),
            ("timeout_seconds".into(), 60.into()),
            ("integration".into(), integration.into()),
        ]))
    };
    let cargo_gate = vec![
        "cargo".into(),
        "test".into(),
        "--locked".into(),
        "--offline".into(),
        "--target-dir".into(),
        root.join("cache/port-gates").display().to_string(),
        "candidate_returns".into(),
    ];
    let gate_batch = root.join("out/gate-batch/batch.toml");
    let scope = Value::Table(toml::Table::from_iter([
        ("format".into(), "translation-scope-v1".into()),
        ("subject".into(), "Translate observed coherent behavior".into()),
        ("origin_ticket".into(), origin.into()),
        ("source_inventory".into(), Value::try_from(&inventory).unwrap()),
        ("source_inventory_sha256".into(), sealed(&inventory)["sha256"].clone()),
        ("exercise".into(), Value::try_from(&exercise_path).unwrap()),
        ("exercise_sha256".into(), sealed(&exercise_path)["sha256"].clone()),
        ("routines".into(), Value::try_from(vec!["Observed"]).unwrap()),
        ("dependencies".into(), Value::try_from(vec![".scratch/work/issues/01-ready.md"]).unwrap()),
        ("dependency_review".into(), Value::Table(toml::Table::new())),
        ("writable_paths".into(), Value::try_from(vec!["crates/port/src/fixture.rs"]).unwrap()),
        (
            "inputs".into(),
            Value::Table(toml::Table::from_iter(
                ["deps/snesrev-sm/src/sm_86.c", "crates/port/src/fixture.rs", ".scratch/work/issues/01-ready.md"]
                    .into_iter()
                    .map(|name| (name.into(), sealed(&root.join(name))["sha256"].clone())),
            )),
        ),
        (
            "verification".into(),
            Value::Array(vec![
                gate(cargo_gate.clone(), false),
                gate(cargo_gate, true),
                gate(
                    vec![
                        "sm-pipeline".into(),
                        "batch".into(),
                        "verify".into(),
                        "--families".into(),
                        "fixture".into(),
                        "--batch".into(),
                        gate_batch.display().to_string(),
                    ],
                    true,
                ),
            ]),
        ),
    ]));
    let scope_path = root.join("out/scope.toml");
    write(root, "out/scope.toml", toml::to_string(&scope).unwrap());
    let research = project.assign("next", None, true)["queue"]["assignment"].clone();
    assert_eq!(research["job"]["kind"].as_str(), Some("research"));
    assert_eq!(research["job"]["ticket"].as_str(), Some(origin));
    project.start_assignment(&research);
    project.assign(
        "record",
        Some(finish(
            &research["claim"],
            Value::Table(toml::Table::from_iter([
                ("result".into(), "researched".into()),
                ("evidence".into(), Value::Array(vec![sealed(&scope_path)])),
                ("followups".into(), Value::Array(vec![])),
            ])),
        )),
        true,
    );
    assert!(fs::read_to_string(root.join(origin)).unwrap().contains("Status: claimed"));
    let implement = project.assign("next", None, true)["queue"]["assignment"].clone();
    assert_eq!(implement["job"]["kind"].as_str(), Some("implement"));
    let worktree = PathBuf::from(implement["workspace"]["worktree"].as_str().unwrap());
    project.git(&[
        "worktree",
        "add",
        "-q",
        "-b",
        implement["workspace"]["branch"].as_str().unwrap(),
        worktree.to_str().unwrap(),
        "HEAD",
    ]);
    let guard = IsolatedWorktree { project: &project, path: worktree.clone() };
    project.start_assignment(&implement);
    let commit = |text: &str| {
        fs::write(worktree.join("crates/port/src/fixture.rs"), text).unwrap();
        assert!(
            Command::new("git")
                .args(["add", "crates/port/src/fixture.rs"])
                .current_dir(&worktree)
                .status()
                .unwrap()
                .success()
        );
        assert!(
            Command::new("git")
                .args([
                    "-c",
                    "user.name=Fixture",
                    "-c",
                    "user.email=fixture@example.test",
                    "commit",
                    "-qm",
                    "implementation"
                ])
                .current_dir(&worktree)
                .status()
                .unwrap()
                .success()
        );
        String::from_utf8(
            Command::new("git").args(["rev-parse", "HEAD"]).current_dir(&worktree).output().unwrap().stdout,
        )
        .unwrap()
        .trim()
        .to_owned()
    };
    let submitted = |commit: String| {
        finish(
            &implement["claim"],
            Value::Table(toml::Table::from_iter([
                ("result".into(), "submitted".into()),
                ("commit".into(), commit.into()),
                ("evidence".into(), Value::Array(vec![sealed(&scope_path)])),
            ])),
        )
    };
    let no_tests = commit(&source.replace("candidate_returns", "renamed_test"));
    project.assign("record", Some(submitted(no_tests)), false);
    let good_commit = commit(&format!("{source}\n// reviewed scoped implementation\n"));
    let submission = submitted(good_commit.clone());
    let completed = project.assign("record", Some(submission.clone()), true);
    let repeated = project.assign("record", Some(submission.clone()), true);
    assert_eq!(completed["jobs"], repeated["jobs"]);
    let integration = project.assign("next", None, true)["queue"]["assignment"].clone();
    assert_eq!(integration["job"]["kind"].as_str(), Some("integrate"));
    project.start_assignment(&integration);
    project.git(&["merge", "--ff-only", &good_commit]);
    project.git(&["worktree", "remove", worktree.to_str().unwrap()]);
    drop(guard);
    // A lost submission response remains retryable after integration removes its worktree.
    project.assign("record", Some(submission), true);
    let integrated = finish(
        &integration["claim"],
        Value::Table(toml::Table::from_iter([
            ("result".into(), "integrated".into()),
            ("submissions".into(), Value::Array(vec![implement["claim"].clone()])),
            ("evidence".into(), Value::Array(vec![sealed(&scope_path)])),
            ("authentication".into(), sealed(&scope_path)),
            ("cleanup".into(), sealed(&scope_path)),
        ])),
    );
    let finished = project.assign("record", Some(integrated.clone()), true);
    assert!(fs::read_to_string(root.join(origin)).unwrap().contains("Status: resolved"));
    let repeated = project.assign("record", Some(integrated), true);
    assert_eq!(finished["jobs"], repeated["jobs"]);
    let batch: Value = toml::from_str(&fs::read_to_string(&gate_batch).unwrap()).unwrap();
    assert_eq!(batch["families"]["fixture"]["status"].as_str(), Some("offline-verified"));
    let verdict: Value =
        toml::from_str(&fs::read_to_string(batch["families"]["fixture"]["verdict"].as_str().unwrap()).unwrap())
            .unwrap();
    project.authenticate(&verdict, true);
}

#[test]
#[ignore = "bounded measurement retains actual-C evidence under the explicit output directory"]
fn measure_affected_family_work_with_retained_evidence() {
    let output =
        PathBuf::from(std::env::var_os("SM_ASSIGNMENT_MEASUREMENT_OUTPUT").expect("measurement output required"));
    assert!(output.is_absolute() && !output.exists(), "use a new absolute measurement directory");
    fs::create_dir_all(output.parent().unwrap()).unwrap();
    let project = Project::at(output.clone(), true);
    project.manifest(Value::Array(vec![Value::Table(toml::Table::from_iter([
        ("name".into(), "one".into()),
        ("a".into(), 1.into()),
        ("expected_return".into(), 2.into()),
    ]))]));
    let manifest: Value =
        toml::from_str(&binary_fixture::read(&output.join("tools/port/fixture.smf")).unwrap()).unwrap();
    let mut other = manifest.clone();
    other["family"] = "other".into();
    other["rust_module"] = "crates/port/src/other.rs".into();
    write(&output, "tools/port/other.smf", toml::to_string(&other).unwrap());
    write(&output, "crates/port/src/other.rs", fs::read(output.join("crates/port/src/fixture.rs")).unwrap());
    write(
        &output,
        "tools/port/batch.toml",
        "[families]\nfixture='family'\nother='family'\n[profiles]\ngameplay=['fixture','other']\n",
    );
    project.initialize_git();
    let invoke = |operation: &str, batch: &str, focused: bool| {
        let mut request = Value::Table(toml::Table::from_iter([
            ("checkout".into(), Value::try_from(&output).unwrap()),
            ("operation".into(), operation.into()),
            ("batch".into(), Value::try_from(output.join(format!("batch/{batch}/batch.toml"))).unwrap()),
            ("root".into(), Value::try_from(output.join("cache")).unwrap()),
            ("workers".into(), 2.into()),
            ("jobs".into(), 2.into()),
            ("case_jobs".into(), 2.into()),
        ]));
        if focused {
            request.as_table_mut().unwrap().insert("changed_since".into(), "HEAD".into());
        }
        let started = std::time::Instant::now();
        let result = project.invoke("batch", request, true);
        let seconds = started.elapsed().as_secs_f64();
        assert!(seconds < 60.0, "initial measurement exceeded 60 seconds: {operation} {batch}");
        (result, seconds)
    };
    let (initial, initial_seconds) = invoke("verify", "broad", false);
    let candidate = output.join("crates/port/src/fixture.rs");
    let revised = format!("{}\n// bounded representative leaf edit\n", fs::read_to_string(&candidate).unwrap());
    fs::write(&candidate, revised).unwrap();
    // Clone only mutable batch metadata. Both invocations begin with identical
    // authenticated predecessors and the same warm compiler cache.
    write(&output, "batch/focused/batch.toml", fs::read(output.join("batch/broad/batch.toml")).unwrap());
    let (plan, plan_seconds) = invoke("plan", "focused", true);
    assert_eq!(plan["selected"].as_array().unwrap().len(), 1);
    let (broad, broad_seconds) = invoke("verify", "broad", false);
    let (focused, focused_seconds) = invoke("verify", "focused", true);
    assert_eq!(broad["selected"].as_array().unwrap().len(), 2);
    assert_eq!(broad["families"]["other"]["status"].as_str(), Some("reused"));
    assert_eq!(focused["selected"].as_array().unwrap(), &[Value::String("fixture".into())]);
    assert_eq!(focused["families"]["other"], initial["families"]["other"]);
    let (reuse, reuse_seconds) = invoke("verify", "focused", true);
    assert_eq!(reuse["families"]["fixture"]["status"].as_str(), Some("reused"));
    let other_verdict = PathBuf::from(initial["families"]["other"]["verdict"].as_str().unwrap());
    let retained: Value = toml::from_str(&fs::read_to_string(&other_verdict).unwrap()).unwrap();
    let artifacts_avoided = retained["artifacts"].as_table().unwrap().len();
    let bytes_avoided: u64 = retained["artifacts"]
        .as_table()
        .unwrap()
        .keys()
        .map(|name| fs::metadata(other_verdict.parent().unwrap().join(name)).unwrap().len())
        .sum();
    for result in [&broad, &focused] {
        for name in ["fixture", "other"] {
            let verdict: Value =
                toml::from_str(&fs::read_to_string(result["families"][name]["verdict"].as_str().unwrap()).unwrap())
                    .unwrap();
            project.authenticate(&verdict, true);
        }
    }
    let report = Value::Table(toml::Table::from_iter([
        ("schema".into(), "affected-family-measurement-v1".into()),
        ("initial_seconds".into(), initial_seconds.into()), ("selection_seconds".into(), plan_seconds.into()),
        ("broad_seconds".into(), broad_seconds.into()), ("focused_seconds".into(), focused_seconds.into()), ("focused_reuse_seconds".into(), reuse_seconds.into()),
        ("broad_selected".into(), broad["selected"].clone()), ("focused_selected".into(), focused["selected"].clone()),
        ("unrelated_artifact_hashes_avoided".into(), Value::try_from(artifacts_avoided).unwrap()),
        ("unrelated_artifact_bytes_avoided".into(), Value::try_from(bytes_avoided).unwrap()),
        ("limitations".into(), "one case per family; warm cache; broad verification ran first; selector rebuilds current conservative discovery rather than authenticating unrelated full-WRAM evidence; no full-route timing claim".into()),
    ]));
    write(&output, "measurement.toml", toml::to_string_pretty(&report).unwrap());
    eprintln!("{}", toml::to_string_pretty(&report).unwrap());
    drop(project);
    // Source paths remain exactly where the sealed manifests record them after
    // cleanup removes the temporary repository metadata and compiler cache.
    for result in [&broad, &focused] {
        for name in ["fixture", "other"] {
            let request = Value::Table(toml::Table::from_iter([
                ("checkout".into(), Value::try_from(&output).unwrap()),
                ("verdict".into(), result["families"][name]["verdict"].clone()),
            ]));
            let mut child = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
                .arg("family-authenticate")
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            writeln!(child.stdin.take().unwrap(), "{}", toml::to_string(&request).unwrap()).unwrap();
            let authenticated = child.wait_with_output().unwrap();
            assert!(
                authenticated.status.success(),
                "post-cleanup authentication: {}",
                String::from_utf8_lossy(&authenticated.stderr)
            );
        }
    }
}

#[test]
#[ignore = "bounded batch metrics comparison retains exact sources, tools and actual-C evidence"]
fn measure_batch_metrics_with_retained_evidence() {
    use std::{collections::BTreeSet, os::unix::fs::MetadataExt, time::Instant};
    fn storage(root: &Path) -> Value {
        fn walk(path: &Path, seen: &mut BTreeSet<(u64, u64)>, totals: &mut [u64; 3]) {
            let metadata = fs::symlink_metadata(path).unwrap();
            if metadata.is_dir() {
                for entry in fs::read_dir(path).unwrap() {
                    walk(&entry.unwrap().path(), seen, totals);
                }
            } else if metadata.is_file() {
                totals[0] += 1;
                totals[1] += metadata.len();
                if seen.insert((metadata.dev(), metadata.ino())) {
                    totals[2] += metadata.blocks() * 512;
                }
            }
        }
        let mut totals = [0; 3];
        walk(root, &mut BTreeSet::new(), &mut totals);
        Value::Table(toml::Table::from_iter([
            ("files".into(), Value::try_from(totals[0]).unwrap()),
            ("logical_file_bytes".into(), Value::try_from(totals[1]).unwrap()),
            ("unique_allocated_file_bytes".into(), Value::try_from(totals[2]).unwrap()),
        ]))
    }
    fn output(command: &mut Command) -> String {
        let result = command.output().unwrap();
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
        String::from_utf8(result.stdout).unwrap().trim().into()
    }
    fn host() -> Value {
        Value::Table(toml::Table::from_iter([
            ("loadavg".into(), fs::read_to_string("/proc/loadavg").unwrap().into()),
            ("meminfo".into(), fs::read_to_string("/proc/meminfo").unwrap().into()),
            ("process_activity".into(), output(Command::new("ps").args(["-eo", "comm=,pcpu=,rss="])).into()),
        ]))
    }
    let root =
        PathBuf::from(std::env::var_os("SM_BATCH_METRICS_MEASUREMENT_OUTPUT").expect("measurement output required"));
    assert!(root.is_absolute() && !root.exists(), "use a new absolute measurement directory");
    assert_eq!(std::env::var("SM_PIPELINE_WORKERS").as_deref(), Ok("4"));
    assert!(std::env::var_os("SM_PIPELINE_ROOT").is_some(), "shared capacity directory required");
    fs::create_dir_all(root.parent().unwrap()).unwrap();
    let project = Project::at(root.clone(), true);
    project.manifest(toml::from_str::<Value>("cases=[{name='one',a=1,expected_return=2}]").unwrap()["cases"].clone());
    write(&root, "tools/port/batch.toml", "[families]\nfixture='family'\n[profiles]\ngameplay=['fixture']\n");
    project.initialize_git();
    let fixture_source_revision = project.git(&["rev-parse", "HEAD"]);
    let worktree = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    let revision = output(Command::new("git").args(["rev-parse", "HEAD"]).current_dir(worktree));
    let tools = root.join("retained-tools");
    fs::create_dir(&tools).unwrap();
    let reader = tools.join("sm-pipeline");
    fs::copy(env!("CARGO_BIN_EXE_sm-pipeline"), &reader).unwrap();
    fs::copy(std::env::current_exe().unwrap(), tools.join("family-control")).unwrap();
    write(&root, "measurement-source.rs", include_str!("family.rs"));
    let mut report = toml::Table::from_iter([
        ("format".into(), "batch-metrics-measurement-v1".into()),
        ("implementation_revision".into(), revision.into()),
        ("fixture_source_revision".into(), fixture_source_revision.into()),
        ("reader".into(), sealed(&reader)),
        ("control".into(), sealed(&tools.join("family-control"))),
        ("control_source".into(), sealed(&root.join("measurement-source.rs"))),
        ("rustc_version".into(), output(Command::new("rustc").arg("-vV")).into()),
        ("cc_version".into(), output(Command::new("cc").arg("--version")).into()),
        ("cargo_profile".into(), "dev/test; debug=0; no extra features; locked/offline; jobs1".into()),
        ("workers".into(), 4.into()),
        ("family_jobs".into(), 1.into()),
        ("case_jobs".into(), 1.into()),
        ("required_families".into(), 1.into()),
        ("required_routines".into(), 1.into()),
        ("required_fixture_pairs".into(), 1.into()),
        ("initial_host".into(), host()),
        ("minimum_useful_change".into(), "greater of 5% or 20ms, and greater than observed variation; three pairs do not establish significance".into()),
        ("physical_disk_io".into(), "unavailable; logical bytes and allocated blocks are not OS disk I/O".into()),
        ("limitations".into(), "one complete tiny actual-C family; alternating sampling comparison is advisory; concurrent compilation/host activity must be assessed from each sample; no full-catalog speedup claim".into()),
    ]);
    assert!(!root.join("cache").exists());
    let started = Instant::now();
    let mut samples = Vec::new();
    let mut verdicts = BTreeSet::new();
    let mut sample = |scenario: &str, fresh: bool, sampling: &str| {
        let host_before = host();
        let (result, invocation_seconds) = project.measured_batch(fresh, sampling, true);
        assert!(invocation_seconds < 60.0, "{scenario} exceeded its declared executable budget");
        let record = &result["families"]["fixture"];
        let path = record["verdict"].as_str().unwrap();
        verdicts.insert(path.to_owned());
        let verdict: Value = toml::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(verdict["verified_cases"].as_integer(), Some(1));
        assert_eq!(verdict["verified_routines"].as_integer(), Some(1));
        let metrics: pipeline_model::metrics::Receipt =
            toml::from_str(&fs::read_to_string(result["metrics"].as_str().unwrap()).unwrap()).unwrap();
        metrics.validate().unwrap();
        assert_eq!(metrics.enabled, sampling == "enabled");
        if scenario == "retained-authentication" {
            assert_eq!(record["status"].as_str(), Some("reused"));
            assert!(metrics.processes.is_empty(), "retained authentication launched a producer");
        } else {
            assert_eq!(record["status"].as_str(), Some("offline-verified"));
            assert!(!metrics.processes.is_empty(), "fresh verification did not execute producers");
        }
        let item = Value::Table(toml::Table::from_iter([
            ("scenario".into(), scenario.into()),
            ("sampling".into(), sampling.into()),
            ("fresh_required".into(), fresh.into()),
            ("invocation_seconds".into(), invocation_seconds.into()),
            ("batch_elapsed_ns".into(), Value::try_from(metrics.elapsed_ns).unwrap()),
            ("owned_processes".into(), Value::try_from(metrics.processes.len()).unwrap()),
            ("host_before".into(), host_before),
            ("host_after".into(), host()),
            ("batch_storage".into(), storage(&root.join("batch"))),
            ("receipt".into(), result),
        ]));
        write(&root, &format!("samples/{}.toml", samples.len()), toml::to_string_pretty(&item).unwrap());
        samples.push(item);
    };
    sample("fresh-empty-private-cache", true, "disabled");
    for _ in 0..3 {
        sample("retained-authentication", false, "disabled");
        sample("retained-authentication", false, "enabled");
    }
    sample("fresh-existing-private-cache", true, "disabled");
    drop(sample);
    report.insert("samples".into(), samples.into());
    report.insert("storage_before_cleanup".into(), storage(&root.join("batch")));
    report.insert("private_cache_before_removal".into(), storage(&root.join("cache")));
    report.insert("cache_interpretation".into(), "empty/existing artifact namespaces are distinguished; retained compiler/cache receipts determine actual hits; run-specific build identities may still miss".into());
    let cleanup_started = Instant::now();
    drop(project);
    report.insert("cleanup_seconds".into(), cleanup_started.elapsed().as_secs_f64().into());
    assert!(!root.join("cache").exists() && !root.join(".git").exists());
    report.insert(
        "cleanup".into(),
        toml::from_str::<Value>(&fs::read_to_string(root.join("retention.toml")).unwrap()).unwrap(),
    );
    let admission_started = Instant::now();
    for (index, verdict) in verdicts.iter().enumerate() {
        let request = toml::to_string(&toml::Table::from_iter([
            ("checkout".into(), Value::try_from(&root).unwrap()),
            ("verdict".into(), verdict.clone().into()),
        ]))
        .unwrap();
        let mut child = Command::new(&reader)
            .arg("family-authenticate")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(request.as_bytes()).unwrap();
        let result = child.wait_with_output().unwrap();
        write(&root, &format!("post-cleanup/{index}.stdout"), &result.stdout);
        write(&root, &format!("post-cleanup/{index}.stderr"), &result.stderr);
        assert!(result.status.success(), "post-cleanup admission: {}", String::from_utf8_lossy(&result.stderr));
    }
    report.insert("post_cleanup_admission_seconds".into(), admission_started.elapsed().as_secs_f64().into());
    report.insert("complete_elapsed_seconds".into(), started.elapsed().as_secs_f64().into());
    report.insert("storage_after_cleanup".into(), storage(&root.join("batch")));
    report.insert("free_space_after".into(), output(Command::new("df").args(["-B1"]).arg(&root)).into());
    report.insert("final_host".into(), host());
    write(&root, "measurement.toml", toml::to_string_pretty(&report).unwrap());
    println!("retained batch metrics control: {}", root.join("measurement.toml").display());
}
