//! Selection must reduce work while retaining unknown family inputs.
#[path = "support/binary_fixture.rs"]
mod binary_fixture;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};
use toml::Value;

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Project(PathBuf);
impl Drop for Project {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
impl Project {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "sm-affected-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        assert!(Command::new("git").args(["init", "-q"]).arg(&root).status().unwrap().success());
        let project = Self(root);
        project.write("tools/port/batch.toml", "[families]\na='family'\nb='family'\n[profiles]\ngameplay=['a','b']\n");
        project.write("crates/port/src/memory.rs", "pub struct Wram;");
        project.write("crates/port/src/a.rs", "pub fn a() {} // future_name");
        project.write("crates/port/src/b.rs", "pub fn b() {} // helper");
        project.write("crates/port/src/helper.rs", "pub fn helper() {} ");
        project.manifest("a", "a", "");
        project.manifest("b", "b", "");
        project.commit();
        project
    }
    fn write(&self, name: &str, text: &str) {
        let path = self.0.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        if path.extension().is_some_and(|e| e == "smf") {
            binary_fixture::write(&path, text).unwrap();
        } else {
            fs::write(path, text).unwrap();
        }
    }
    fn manifest(&self, name: &str, module: &str, extra: &str) {
        self.write(
            &format!("tools/port/{name}.smf"),
            &format!(
                "family='{name}'\nbank='sm_80.c'\nrust_module='crates/port/src/{module}.rs'\nroutines=[]\n{extra}"
            ),
        );
    }
    fn commit(&self) {
        assert!(Command::new("git").args(["add", "."]).current_dir(&self.0).status().unwrap().success());
        assert!(
            Command::new("git")
                .args(["-c", "user.name=Fixture", "-c", "user.email=fixture@example.test", "commit", "-qm", "fixture"])
                .current_dir(&self.0)
                .status()
                .unwrap()
                .success()
        );
    }
    fn plan(&self) -> Value {
        let output = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
            .args(["batch", "plan", "--changed-since", "HEAD"])
            .current_dir(&self.0)
            .output()
            .unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert!(!self.0.join("out").exists(), "planning created output or batch state");
        toml::from_str(std::str::from_utf8(&output.stdout).unwrap()).unwrap()
    }
}
fn families<'a>(report: &'a Value, group: &str) -> Vec<&'a str> {
    report[group].as_array().unwrap().iter().map(|decision| decision["family"].as_str().unwrap()).collect()
}

#[test]
fn metadata_commands_are_read_only_even_with_metrics_enabled() {
    for operation in ["plan", "catalog"] {
        let project = Project::new();
        let output = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
            .args(["batch", operation])
            .env("SM_PIPELINE_METRICS", "enabled")
            .current_dir(&project.0)
            .output()
            .unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let report: Value = toml::from_str(std::str::from_utf8(&output.stdout).unwrap()).unwrap();
        assert!(report.get("metrics").is_none(), "{operation} published a default metrics receipt");
        assert!(!project.0.join("out").exists(), "{operation} created output state");
    }
}

#[test]
fn cold_plan_selects_only_the_leaf_and_transitive_dependencies() {
    let project = Project::new();
    project.write("crates/port/src/helper.rs", "pub fn helper() { let changed=1; }");
    let report = project.plan();
    assert_eq!(families(&report, "selected"), ["b"]);
    assert_eq!(families(&report, "excluded"), ["a"]);
    assert_eq!(report["selected"][0]["reasons"][0]["paths"][0].as_str(), Some("crates/port/src/helper.rs"));
}

#[test]
fn shared_cyclic_source_closures_fit_one_planning_budget() {
    let project = Project::new();
    let names = (0..70).map(|index| format!("shared{index}")).collect::<Vec<_>>();
    let mut catalog = String::from("[families]\n");
    for name in &names {
        catalog.push_str(&format!("{name}='family'\n"));
        project.manifest(name, "node0", "");
    }
    catalog.push_str(&format!("[profiles]\ngameplay={names:?}\n"));
    project.write("tools/port/batch.toml", &catalog);
    let mut source = String::from("pub fn step() {\n");
    for index in 0..32 {
        source.push_str(&format!("crate::node{index}::step();\n"));
    }
    source.push_str("}\n");
    for index in 0..32 {
        project.write(&format!("crates/port/src/node{index}.rs"), &source);
    }
    project.commit();
    project.write("docs/unrelated.md", "Only documentation changed.\n");
    let report = project.plan();
    assert!(families(&report, "selected").is_empty());
    assert_eq!(families(&report, "excluded").len(), names.len());
    assert_eq!(report["planning_statistics"]["rust_source_parses"].as_integer(), Some(33));

    project.write("crates/port/src/node17.rs", &(source + "pub fn added() {}\n"));
    let changed = project.plan();
    assert_eq!(families(&changed, "selected").len(), names.len());
    assert!(families(&changed, "excluded").is_empty());
    assert!(changed["selected"].as_array().unwrap().iter().all(|decision| {
        decision["reasons"][0]["code"].as_str() == Some("changed-dependency")
            && decision["dependencies"].as_array().unwrap().contains(&Value::from("crates/port/src/node17.rs"))
    }));
}

#[test]
fn comment_and_literal_names_do_not_select_unrelated_families() {
    let project = Project::new();
    project.write(
        "crates/port/src/a.rs",
        r###"
/// ROM-aware preinstructions include native room projectile spawning.
/// Trail coordinates include native Ceres Mode7 arithmetic and ROM trail frames.
/* helper /* include_str!("helper") */ future_name */
pub fn a() {
    let _ = "helper File fs mod include include_bytes env option_env macro_rules";
    let _ = r##"helper include!("anything")"##;
    let _ = b"helper include_str";
    let _ = br#"helper include_bytes"#;
    let _ = '\'';
}
"###,
    );
    project.commit();
    project.write("crates/port/src/helper.rs", "pub fn helper() { let changed = 1; }");
    let report = project.selection();
    assert_eq!(families(&report, "selected"), ["b"]);
    assert_eq!(families(&report, "excluded"), ["a"]);
    assert_eq!(
        report["excluded"][0]["dependencies"],
        Value::Array(["crates/port/src/a.rs", "crates/port/src/memory.rs", "tools/port/a.smf"].map(Value::from).to_vec())
    );
}

#[test]
fn untracked_module_namespace_can_affect_an_unchanged_family() {
    let project = Project::new();
    project.write("crates/port/src/future_name.rs", "pub fn new_behavior() {}");
    let report = project.plan();
    assert_eq!(families(&report, "selected"), ["a"]);
    assert_eq!(families(&report, "excluded"), ["b"]);
    assert_eq!(report["changed_paths"][0].as_str(), Some("crates/port/src/future_name.rs"));
}

#[test]
fn manifests_and_captured_assets_are_family_owned_inputs() {
    let project = Project::new();
    project.write("fixtures/capture before.wram", "original");
    project.manifest(
        "a",
        "a",
        &format!(
            "cases=[{{name='one',before_file='fixtures/capture before.wram',before_sha256='{:x}'}}]\n",
            Sha256::digest(b"original")
        ),
    );
    project.commit();
    project.write("fixtures/capture before.wram", "changed");
    let report = project.plan();
    assert_eq!(families(&report, "selected"), ["a"]);
    assert_eq!(families(&report, "excluded"), ["b"]);
    assert_eq!(report["selected"][0]["reasons"][0]["code"].as_str(), Some("uncertain-inputs"));
    project.manifest("b", "b", "domain='edited fixture declaration'\n");
    assert_eq!(families(&project.plan(), "selected"), ["a", "b"]);
}

#[test]
fn deleting_a_discovered_module_keeps_its_dependents_affected() {
    let project = Project::new();
    fs::remove_file(project.0.join("crates/port/src/helper.rs")).unwrap();
    let report = project.plan();
    assert_eq!(families(&report, "selected"), ["b"]);
    assert_eq!(families(&report, "excluded"), ["a"]);
    assert_eq!(report["selected"][0]["reasons"][0]["code"].as_str(), Some("uncertain-inputs"));
}

#[test]
fn missing_captured_asset_cannot_exclude_its_family() {
    let project = Project::new();
    project.manifest(
        "a",
        "a",
        "cases=[{name='one',expected_after_file='missing.wram',expected_after_sha256='missing'}]\n",
    );
    project.commit();
    let report = project.plan();
    assert_eq!(families(&report, "selected"), ["a"]);
    assert_eq!(families(&report, "excluded"), ["b"]);
}

#[test]
fn ignored_new_rust_input_remains_explicit_uncertainty() {
    let project = Project::new();
    project.write(".gitignore", "crates/port/src/future_name.rs\n");
    project.commit();
    project.write("crates/port/src/future_name.rs", "pub fn omitted_from_diff() {}");
    let report = project.plan();
    assert_eq!(families(&report, "selected"), ["a"]);
    assert_eq!(families(&report, "excluded"), ["b"]);
    assert_eq!(report["selected"][0]["reasons"][0]["code"].as_str(), Some("uncertain-inputs"));
}

#[test]
fn deleted_dependencies_and_newline_paths_do_not_disappear() {
    let project = Project::new();
    project.write("fixtures/old\nname", "old");
    project.manifest("a", "a", "rust_dependencies=[\"fixtures/old\\nname\"]\n");
    project.commit();
    fs::rename(project.0.join("fixtures/old\nname"), project.0.join("fixtures/new\nname")).unwrap();
    let report = project.plan();
    assert_eq!(families(&report, "selected"), ["a"]);
    assert_eq!(families(&report, "excluded"), ["b"]);
    let changed = report["changed_paths"].as_array().unwrap();
    assert!(changed.iter().any(|path| path.as_str() == Some("fixtures/old\nname")));
    assert!(changed.iter().any(|path| path.as_str() == Some("fixtures/new\nname")));
    assert_eq!(report["selected"][0]["reasons"][0]["code"].as_str(), Some("uncertain-inputs"));
}

#[test]
fn dynamic_rust_inputs_remain_uncertain_and_shared_memory_selects_every_family() {
    let project = Project::new();
    project.write("crates/port/src/a.rs", "include!(\"nested.rs\");");
    project.commit();
    let report = project.plan();
    assert_eq!(families(&report, "selected"), ["a"]);
    assert_eq!(report["selected"][0]["reasons"][0]["code"].as_str(), Some("uncertain-inputs"));
    project.write("crates/port/src/memory.rs", "pub struct ChangedWram;");
    assert_eq!(families(&project.plan(), "selected"), ["a", "b"]);
}

#[test]
fn new_untracked_manifest_is_discovered_without_selecting_existing_families() {
    let project = Project::new();
    project.manifest("new", "a", "");
    let output = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
        .args(["batch", "plan", "--profile", "offline-all", "--changed-since", "HEAD"])
        .current_dir(&project.0)
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let report: Value = toml::from_str(std::str::from_utf8(&output.stdout).unwrap()).unwrap();
    assert_eq!(families(&report, "selected"), ["new"]);
    assert_eq!(families(&report, "excluded"), ["a", "b"]);
    assert!(!project.0.join("out").exists());
}

#[test]
fn verification_owner_changes_keep_every_family_affected() {
    for path in [
        "crates/pipeline/src/main.rs",
        "crates/offline-verification/src/lib.rs",
        "crates/tool-execution/src/lib.rs",
        "crates/decomp-source/src/lib.rs",
    ] {
        let project = Project::new();
        project.write(path, "// changed verification input\n");
        let report = project.plan();
        assert_eq!(families(&report, "selected"), ["a", "b"], "{path}");
        assert!(families(&report, "excluded").is_empty(), "{path}");
    }
}
