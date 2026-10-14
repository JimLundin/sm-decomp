//! Native evidence must reject wrong results, incomplete coverage and unchecked C.
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
    fs::write(p, bytes).unwrap();
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
        for entry in fs::read_dir(crate_root.join("src")).unwrap() {
            let entry = entry.unwrap();
            if entry.path().is_file() {
                write(
                    &root,
                    &format!("crates/pipeline/src/{}", entry.file_name().to_string_lossy()),
                    fs::read(entry.path()).unwrap(),
                );
            }
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
        write(&self.0, "tools/port/fixture.toml", toml::to_string(&manifest).unwrap());
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
                ("manifest".to_owned(), Value::String(("tools/port/fixture.toml").to_owned())),
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
}

#[test]
fn dispatcher_pointer_banks_normalize_raw_and_host_pointers_and_reject_other_banks() {
    let project = Project::new();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let function = fs::read_to_string(root.join("src/dispatcher_pointer.c")).unwrap();
    write(
        &project.0,
        "pointer.c",
        format!(
            "#include <stdint.h>\n#include <stdlib.h>\n#include <assert.h>\nstatic uint8_t rom[0x400000];static uint8_t *g_rom=rom;\n{function}\nint main(int argc,char **argv){{if(argc>1){{PortPointerResult((void*)(rom+0x190000),0x86);return 1;}}for(int bank=0x84;bank<=0xb2;bank++){{if(bank!=0x84&&bank!=0x86&&bank!=0xb2)continue;assert(PortPointerResult((void*)(rom+((bank&0x7f)*0x8000)+0x1234),bank)==((bank<<16)|0x9234));assert(PortPointerResult((void*)0x9234,bank)==((bank<<16)|0x9234));assert(PortPointerResult(0,bank)==0);}}return 0;}}"
        ),
    );
    assert!(
        Command::new("cc")
            .arg(project.0.join("pointer.c"))
            .arg("-o")
            .arg(project.0.join("pointer"))
            .status()
            .unwrap()
            .success()
    );
    assert!(Command::new(project.0.join("pointer")).status().unwrap().success());
    assert_eq!(Command::new(project.0.join("pointer")).arg("wrong-bank").status().unwrap().code(), Some(3));
}

#[test]
fn invalid_dispatcher_pointer_bank_is_rejected_before_compilation() {
    let project = Project::new();
    project.manifest(Value::Array(vec![Value::Table(toml::Table::from_iter([
        ("name".to_owned(), Value::String(("one").to_owned())),
        ("a".to_owned(), Value::Integer(1_i64)),
    ]))]));
    let path = project.0.join("tools/port/fixture.toml");
    let mut manifest: Value = toml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    manifest["routines"][0]["return"] = "rom_ptr".into();
    manifest["routines"][0].as_table_mut().unwrap().insert("pointer_bank".into(), 0x100.into());
    write(&project.0, "tools/port/fixture.toml", toml::to_string(&manifest).unwrap());
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
    write(&project.0, "tools/port/fixture.toml", manifest);
    let verdict = project.verify_mode(true, "fresh");
    assert_eq!(verdict["verified_cases"].as_integer(), Some(2));
    assert_eq!(fs::read_to_string(project.0.join("tools/port/fixture.toml")).unwrap(), manifest);
    project.authenticate(&verdict, true);
    let run = PathBuf::from(verdict["run"].as_str().unwrap());
    let three = fs::read(run.join("three/c-after.wram")).unwrap();
    assert_eq!(&three[16..22], &[0xcd, 0xab, 0x34, 0x12, 0xfe, 0xff]);
    let four = fs::read(run.join("four/c-after.wram")).unwrap();
    assert_eq!(&four[16..24], &[0xcd, 0xab, 0x00, 0x80, 0x34, 0x12, 0xfe, 0xff]);
    write(&project.0, "tools/port/fixture.toml", manifest.replacen("['uint16','uint32']", "['uint32','uint32']", 1));
    let rejected = project.verify_mode(false, "fresh");
    assert!(rejected["error"].as_str().unwrap().contains("invalid packed"));
    assert!(!Path::new(rejected["run"].as_str().unwrap()).join("c").exists());
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
    // Filesystem rendezvous fails if a continuation serializes unrelated roots.
    write(
        &project.0,
        "crates/port/src/fixture.rs",
        r#"use crate::memory::Wram;
pub fn dispatch(ram:&mut Wram<'_>,name:&str,a:u16,_b:u16)->Result<u32,&'static str>{
if name!="Touch"{return Err("routine");}
if a==1||a==7{std::fs::write(format!("../ready-{a}"),b"ready").unwrap();let other=if a==1{7}else{1};let deadline=std::time::Instant::now()+std::time::Duration::from_secs(3);while !std::path::Path::new(&format!("../ready-{other}")).exists(){if std::time::Instant::now()>deadline{return Err("independent chains serialized");}std::thread::sleep(std::time::Duration::from_millis(1));}}
ram.set_byte(17,a as u8);Ok(a as u32+1)}"#,
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

#[test]
fn cancelling_family_reaps_helper_fixture_and_descendant() {
    let project = Project::new();
    write(
        &project.0,
        "crates/port/src/fixture.rs",
        r#"use crate::memory::Wram;
unsafe extern "C"{fn fork()->i32;fn getppid()->i32;}
pub fn dispatch(_ram:&mut Wram<'_>,_name:&str,_a:u16,_b:u16)->Result<u32,&'static str>{
std::fs::write("helper.pid",unsafe{getppid()}.to_string()).unwrap();
std::fs::write("child.pid",std::process::id().to_string()).unwrap();
let pid=unsafe{fork()};assert!(pid>=0);if pid==0{std::fs::write("descendant.pid",std::process::id().to_string()).unwrap();}
std::thread::sleep(std::time::Duration::from_secs(30));Ok(0)}"#,
    );
    project.manifest(toml::from_str::<Value>("cases=[{name='sleep',a=1}]").unwrap()["cases"].clone());
    let request = toml::to_string(&toml::Table::from_iter([
        ("checkout".into(), Value::try_from(&project.0).unwrap()),
        ("source".into(), Value::try_from(project.0.join("deps/snesrev-sm")).unwrap()),
        ("manifest".into(), "tools/port/fixture.toml".into()),
        ("output".into(), Value::try_from(project.0.join("out")).unwrap()),
        ("root".into(), Value::try_from(project.0.join("cache")).unwrap()),
        ("workers".into(), 1.into()),
        ("case_jobs".into(), 1.into()),
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
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    let dir = loop {
        if let Ok(entries) = fs::read_dir(project.0.join("out"))
            && let Some(dir) = entries
                .filter_map(Result::ok)
                .map(|e| e.path().join("sleep"))
                .find(|p| p.join("descendant.pid").is_file())
        {
            break dir;
        }
        assert!(std::time::Instant::now() < deadline, "cancellation fixture did not start");
        std::thread::sleep(std::time::Duration::from_millis(5));
    };
    unsafe {
        libc::kill(process.id() as i32, libc::SIGTERM);
    }
    let output = process.wait_with_output().unwrap();
    assert!(!output.status.success());
    for name in ["helper.pid", "child.pid", "descendant.pid"] {
        let pid: i32 = fs::read_to_string(dir.join(name)).unwrap().parse().unwrap();
        assert_eq!(unsafe { libc::kill(pid, 0) }, -1, "{name} survived cancellation");
    }
    assert!(!dir.join("c.stdout").exists());
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
        Value::Array(vec![Value::Table(toml::Table::from_iter([
            ("name".to_owned(), Value::String(("overflow").to_owned())),
            (
                "writes".to_owned(),
                Value::Array(vec![Value::Table(toml::Table::from_iter([
                    ("offset".into(), 131071.into()),
                    ("data".into(), Value::Array(vec![1.into(), 2.into()])),
                ]))]),
            ),
        ]))]),
    ] {
        project.manifest(cases);
        assert_eq!(project.verify(false)["status"].as_str(), Some("rejected"));
    }
}

#[test]
fn native_projectile_candidate_is_available_to_the_compiled_host_wrapper() {
    let project = Project::new();
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    if !checkout.join("rom/sm.sfc").exists() {
        return;
    }
    let mut manifest: Value =
        toml::from_str(&fs::read_to_string(checkout.join("tools/port/observed-projectile-initializers.toml")).unwrap())
            .unwrap();
    manifest["routines"].as_array_mut().unwrap().truncate(1);
    manifest["routines"][0]["cases"].as_array_mut().unwrap().truncate(1);
    write(&project.0, "one-native.toml", toml::to_string(&manifest).unwrap());
    let verdict = project.invoke(
        "family-verify",
        Value::Table(toml::Table::from_iter([
            ("checkout".into(), Value::try_from(checkout).unwrap()),
            ("source".into(), Value::try_from(checkout.join("deps/snesrev-sm")).unwrap()),
            ("manifest".into(), Value::try_from(project.0.join("one-native.toml")).unwrap()),
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
    let path = project.0.join("tools/port/fixture.toml");
    let original = fs::read(&path).unwrap();
    let mut manifest: Value = toml::from_str(&String::from_utf8(original.clone()).unwrap()).unwrap();
    manifest["routines"][0]["cases"][0]["a"] = 2.into();
    write(&project.0, "tools/port/fixture.toml", toml::to_string(&manifest).unwrap());
    project.authenticate(&verdict, false);
    let mut stripped = verdict.clone();
    stripped["implementation"].as_table_mut().unwrap().remove("manifest");
    let verdict_path = Path::new(verdict["run"].as_str().unwrap()).join("verdict.toml");
    fs::write(&verdict_path, toml::to_string(&stripped).unwrap()).unwrap();
    project.authenticate(&stripped, false);
    let mut rebound = verdict.clone();
    rebound["implementation"]["manifest"]["file"] =
        Value::try_from(Path::new(verdict["run"].as_str().unwrap()).join("manifest.toml")).unwrap();
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
    let first_manifest = project.0.join("tools/port/fixture.toml");
    let mut other: Value = toml::from_str(&fs::read_to_string(&first_manifest).unwrap()).unwrap();
    other["family"] = "other".into();
    other["rust_module"] = "crates/port/src/other.rs".into();
    write(&project.0, "tools/port/other.toml", toml::to_string(&other).unwrap());
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
    let manifest: Value = toml::from_str(&fs::read_to_string(output.join("tools/port/fixture.toml")).unwrap()).unwrap();
    let mut other = manifest.clone();
    other["family"] = "other".into();
    other["rust_module"] = "crates/port/src/other.rs".into();
    write(&output, "tools/port/other.toml", toml::to_string(&other).unwrap());
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
