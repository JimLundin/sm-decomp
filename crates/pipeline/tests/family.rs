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
struct Project(PathBuf);
impl Drop for Project {
    fn drop(&mut self) {
        if self.0.join("out").is_dir() {
            for deduplicate in [false, true] {
                let mut command = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"));
                command.args(["cleanup", "--root"]).arg(self.0.join("out")).arg("--apply");
                if deduplicate {
                    command.arg("--deduplicate");
                }
                let output = command.output().unwrap();
                assert!(output.status.success(), "cleanup failed: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
        let _ = fs::remove_dir_all(&self.0);
    }
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
        Self(root)
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
        if let Ok(entries) = fs::read_dir(project.0.join("out")) {
            if let Some(dir) = entries
                .filter_map(Result::ok)
                .map(|e| e.path().join("sleep"))
                .find(|p| p.join("descendant.pid").is_file())
            {
                break dir;
            }
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
            request["changed_since"] = "HEAD".into();
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
