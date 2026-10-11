//! A real C callback must match amounts, ordering and full memory independently.
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};
use toml::Value;
struct Run(PathBuf);
impl Drop for Run {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn invoke(command: &str, request: Value, success: bool) -> Value {
    let mut process = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
        .arg(command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    writeln!(process.stdin.take().unwrap(), "{}", toml::to_string(&request).unwrap()).unwrap();
    let output = process.wait_with_output().unwrap();
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
#[test]
fn quicksand_amount_and_ordered_actual_callback_snapshots_authenticate_and_wrong_state_fails() {
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    if !checkout.join("rom/sm.sfc").exists() {
        return;
    }
    let root = Run(std::env::temp_dir()
        .join(format!("sm-callback-test-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos())));
    fs::create_dir(&root.0).unwrap();
    let module = root.0.join("candidate.rs");
    let original = "use crate::Wram;pub trait Host{fn spawn_plm(&mut self,ram:&mut Wram<'_>,header:u16,amount:&mut i32)->Result<u8,&'static str>;}pub fn dispatch_host(ram:&mut Wram,_:&[u8],host:&mut impl Host,_:&str,a:i32,b:i32)->Result<u32,&'static str>{let mut amount=a;host.spawn_plm(ram,b as u16,&mut amount)?;Ok(amount as u32)}";
    fs::write(&module, original).unwrap();
    let manifest = root.0.join("manifest.toml");
    let rom = checkout.join("rom/sm.sfc");
    let rom_hash = format!("{:x}", Sha256::digest(fs::read(&rom).unwrap()));
    fs::write(
        &manifest,
        toml::to_string(&Value::Table(toml::Table::from_iter([
            ("family".to_owned(), Value::String(("callback-probe").to_owned())),
            ("infrastructure_probe".to_owned(), Value::Boolean(true)),
            ("rust_module".to_owned(), Value::try_from(&(module)).unwrap()),
            (
                "rom".to_owned(),
                Value::Table(toml::Table::from_iter([
                    ("file".to_owned(), Value::try_from(&(rom)).unwrap()),
                    ("sha256".to_owned(), Value::try_from(&(rom_hash)).unwrap()),
                ])),
            ),
            (
                "routines".to_owned(),
                Value::Array(vec![Value::Table(toml::Table::from_iter([
                    ("symbol".to_owned(), Value::String(("CollisionCallbackProbe").to_owned())),
                    (
                        "arg_types".to_owned(),
                        Value::Array(vec![Value::String(("int32").to_owned()), Value::String(("uint16").to_owned())]),
                    ),
                    ("return".to_owned(), Value::String(("uint32").to_owned())),
                    (
                        "cases".to_owned(),
                        Value::Array(vec![Value::Table(toml::Table::from_iter([
                            ("name".to_owned(), Value::String(("quicksand").to_owned())),
                            ("a".to_owned(), Value::Integer(65536_i64)),
                            ("b".to_owned(), Value::Integer(0xb72b_i64)),
                            (
                                "writes".to_owned(),
                                Value::Array(vec![Value::Table(toml::Table::from_iter([
                                    ("offset".into(), 0xb02.into()),
                                    ("data".into(), Value::Array(vec![3.into(), 0.into()])),
                                ]))]),
                            ),
                            ("expected_return".to_owned(), Value::Integer(0x3000_i64)),
                            ("minimum_callbacks".to_owned(), Value::Integer(1_i64)),
                        ]))]),
                    ),
                ]))]),
            ),
        ])))
        .unwrap(),
    )
    .unwrap();
    let request = Value::Table(toml::Table::from_iter([
        ("checkout".to_owned(), Value::try_from(checkout).unwrap()),
        ("source".to_owned(), Value::try_from(checkout.join("deps/snesrev-sm")).unwrap()),
        ("manifest".to_owned(), Value::try_from(&(manifest)).unwrap()),
        ("output".to_owned(), Value::try_from(root.0.join("out")).unwrap()),
        ("adapter".to_owned(), Value::String(("callback").to_owned())),
        ("root".to_owned(), Value::try_from(root.0.join("workers")).unwrap()),
        ("workers".to_owned(), Value::Integer(2_i64)),
        ("case_jobs".to_owned(), Value::Integer(2_i64)),
    ]));
    let verdict = invoke("family-verify", request.clone(), true);
    assert_eq!(verdict["cases"][0]["callbacks"].as_integer(), Some(1));
    assert_eq!(verdict["gameplay_translation"].as_bool(), Some(false));
    let path = Path::new(verdict["run"].as_str().unwrap()).join("verdict.toml");
    invoke(
        "family-authenticate",
        Value::Table(toml::Table::from_iter([
            ("checkout".to_owned(), Value::try_from(checkout).unwrap()),
            ("verdict".to_owned(), Value::try_from(&(path)).unwrap()),
        ])),
        true,
    );
    let callback = path.parent().unwrap().join("quicksand/rust/callbacks.txt");
    fs::write(&callback, "").unwrap();
    invoke(
        "family-authenticate",
        Value::Table(toml::Table::from_iter([
            ("checkout".to_owned(), Value::try_from(checkout).unwrap()),
            ("verdict".to_owned(), Value::try_from(&(path)).unwrap()),
        ])),
        false,
    );
    fs::write(&module, original.replace("Ok(amount as u32)", "ram.set_byte(0x10000,1);Ok(amount as u32)")).unwrap();
    let rejected = invoke("family-verify", request, false);
    assert!(rejected["error"].as_str().unwrap().contains("WRAM mismatch"));
}
