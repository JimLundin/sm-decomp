//! Public binary Fixture authoring commands use relative paths and preserve identity.
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

struct Workspace(PathBuf);
impl Workspace {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "sm-fixture-cli-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_sm-pipeline")).arg("fixture").args(args).current_dir(&self.0).output().unwrap()
    }

    fn success(&self, args: &[&str]) -> Vec<u8> {
        let output = self.run(args);
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        output.stdout
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn inspect_import_and_diff_preserve_bytes_and_report_implicit_case_order() {
    let workspace = Workspace::new();
    let declaration = "family='cli'\n[[routines]]\nsymbol='Step'\n\
        cases=[{name='first',writes=[{offset=3,data=[1,2,3]}]},{name='second'}]\n";
    fs::write(workspace.0.join("edit.toml"), declaration).unwrap();
    workspace.success(&["import", "edit.toml", "before.smf"]);
    let before = fs::read(workspace.0.join("before.smf")).unwrap();
    assert!(before.starts_with(b"SMFIXT01"));
    let inspected = workspace.success(&["inspect", "before.smf"]);
    fs::write(workspace.0.join("edit.toml"), inspected).unwrap();
    workspace.success(&["import", "edit.toml", "after.smf"]);
    assert_eq!(before, fs::read(workspace.0.join("after.smf")).unwrap());
    assert!(workspace.success(&["diff", "before.smf", "after.smf"]).is_empty());
    let case = workspace.success(&["inspect", "before.smf", "first"]);
    let case: toml::Value = toml::from_str(std::str::from_utf8(&case).unwrap()).unwrap();
    assert_eq!(case["name"].as_str(), Some("first"));
    assert_eq!(case["writes"][0]["data"][1].as_integer(), Some(2));

    let reordered = "family='cli'\n[[routines]]\nsymbol='Step'\n\
        cases=[{name='second'},{name='first',writes=[{offset=3,data=[1,2,3]}]}]\n";
    fs::write(workspace.0.join("edit.toml"), reordered).unwrap();
    workspace.success(&["import", "edit.toml", "after.smf"]);
    let diff = workspace.success(&["diff", "before.smf", "after.smf"]);
    assert!(std::str::from_utf8(&diff).unwrap().contains("case order"));

    fs::write(workspace.0.join("edit.toml"), "not a fixture").unwrap();
    assert!(!workspace.run(&["import", "edit.toml", "before.smf"]).status.success());
    assert_eq!(before, fs::read(workspace.0.join("before.smf")).unwrap());
    assert!(!workspace.run(&["inspect", "edit.toml"]).status.success());
}
