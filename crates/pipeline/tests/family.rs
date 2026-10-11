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
        assert_eq!(output.status.success(), success, "{}", String::from_utf8_lossy(&output.stderr));
        if output.stdout.is_empty() {
            Value::Table(toml::Table::new())
        } else {
            toml::from_str(&String::from_utf8(output.stdout).unwrap()).unwrap()
        }
    }
    fn verify(&self, success: bool) -> Value {
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
