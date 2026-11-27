use std::fs;
use std::path::{Path, PathBuf};

use super::{Policy, check};

fn repo() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap()
}

fn tags(root: &Path) -> Vec<String> {
    let policy = Policy::load(repo()).unwrap();
    check(root, &policy)
        .unwrap()
        .into_iter()
        .map(|f| f.to_string())
        .collect()
}

#[test]
fn the_clean_fixture_passes() {
    let found = tags(&repo().join("tools/cleanroom/rule_fixtures/clean"));
    assert!(found.is_empty(), "{found:#?}");
}

#[test]
fn every_other_fixture_is_refused_by_its_tag() {
    let mut count = 0;
    for entry in fs::read_dir(repo().join("tools/cleanroom/rule_fixtures")).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_str().unwrap().to_owned();
        let Some((tag, _)) = name.split_once("__") else {
            continue;
        };
        count += 1;
        let found = tags(&path);
        assert!(
            found.iter().any(|f| f.contains(&format!(": {tag}: "))),
            "{name}: {found:#?}"
        );
    }
    assert!(count > 70, "{count} fixtures");
}

struct Workspace(PathBuf);

impl Workspace {
    fn new(name: &str) -> Self {
        let dir =
            std::env::temp_dir().join(format!("cleanroom-rules-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        copy(&repo().join("tools/cleanroom/rule_fixtures/clean"), &dir);
        Self(dir)
    }

    fn workspace(&self, deps: &str) -> &Self {
        fs::write(
            self.0.join("Cargo.toml"),
            format!(
                "[workspace.package]\nedition = \"2024\"\n\n[workspace.dependencies]\n{deps}\n"
            ),
        )
        .unwrap();
        self
    }

    fn member(&self, section: &str, deps: &str, package: &str) -> &Self {
        fs::write(
            self.0.join("crates/clean/demo/Cargo.toml"),
            format!(
                "[package]\nname = \"demo\"\nedition.workspace = true\n{package}\n\n[lints]\nworkspace = true\n\n[{section}]\n{deps}\n"
            ),
        )
        .unwrap();
        self
    }

    fn refuses(&self, tag: &str) {
        let found = tags(&self.0);
        assert!(
            found.iter().any(|f| f.contains(&format!(": {tag}: "))),
            "{found:#?}"
        );
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn copy(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            copy(&entry.path(), &to.join(entry.file_name()));
        } else {
            fs::copy(entry.path(), to.join(entry.file_name())).unwrap();
        }
    }
}

fn pinned(name: &str) -> String {
    fs::read_to_string(repo().join("Cargo.toml"))
        .unwrap()
        .lines()
        .find(|l| l.starts_with(&format!("{name} = ")))
        .unwrap()
        .to_owned()
}

#[test]
fn an_inherited_dependency_takes_its_version_and_features_from_the_workspace() {
    let ws = Workspace::new("inherit");
    ws.workspace(&pinned("serde")).member(
        "dependencies",
        "serde = { workspace = true, features = [\"derive\"] }",
        "",
    );
    assert_eq!(tags(&ws.0), Vec::<String>::new());
    ws.workspace("serde = \"=0.0.1\"").refuses("dep-registry");
    let version = pinned("serde");
    let version = version.split_once("= ").unwrap().1;
    ws.workspace(&format!(
        "serde = {{ version = {version}, features = ['rc'] }}"
    ))
    .refuses("dep-registry");
    ws.workspace(&pinned("serde"))
        .member(
            "dependencies",
            "serde = { workspace = true, features = [\"rc\"] }",
            "",
        )
        .refuses("dep-registry");
    ws.member(
        "dependencies",
        "serde.workspace = true",
        "workspace = \"../..\"",
    )
    .refuses("dep-workspace");
    ws.member("dependencies", "toml.workspace = true", "")
        .workspace(&pinned("toml"))
        .refuses("dep-registry");
    ws.workspace("toml = { path = \"crates/tools\" }")
        .refuses("dep-path");
    ws.workspace("toml = { version = \"=1\", registry = \"other\" }")
        .refuses("dep-git");
    ws.workspace("toml = {").refuses("manifest");
}

#[test]
fn a_test_that_includes_the_spec_reads_docs() {
    assert!(super::reads_docs(
        "const SPEC: &str = include_str!(\"../../../../docs/behaviours/demo.md\");"
    ));
    assert!(!super::reads_docs(
        "const V: &str = include_str!(\"../../../../scenarios/vectors/demo.toml\"); // docs/"
    ));
}

/// What clippy reports on SOURCE, a crate of its own built under the clean crates'
/// `.clippy.toml`, with the capability lints the workspace forbids.
fn capability(name: &str, source: &str) -> String {
    let dir = std::env::temp_dir().join(format!(
        "cleanroom-capability-{}-{name}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"probe\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[workspace]\n",
    )
    .unwrap();
    fs::write(dir.join("src/text.rs"), "\"text\"\n").unwrap();
    fs::write(
        dir.join("src/lib.rs"),
        format!(
            "#![forbid(clippy::disallowed_macros, clippy::disallowed_methods, clippy::disallowed_types)]\n{source}"
        ),
    )
    .unwrap();
    let out = std::process::Command::new("cargo")
        .current_dir(&dir)
        .env("CLIPPY_CONF_DIR", repo().join("crates/clean"))
        .env("CARGO_TARGET_DIR", dir.join("target"))
        .args(["clippy", "--offline", "-q"])
        .output()
        .unwrap();
    let _ = fs::remove_dir_all(&dir);
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn the_capability_rule_refuses_unix_sockets_symlinks_and_raw_descriptors() {
    let found = capability(
        "sockets",
        "pub fn reach() {\n\
         let _ = std::os::unix::net::UnixStream::connect(\"/var/run/docker.sock\");\n\
         let _ = std::os::unix::net::UnixListener::bind(\"/tmp/a\");\n\
         let _ = std::os::unix::net::UnixDatagram::unbound();\n\
         let _ = std::os::unix::fs::symlink(\"/\", \"/tmp/b\");\n\
         let _ = std::fs::soft_link(\"/\", \"/tmp/c\");\n\
         }\n\
         pub fn adopt<T: std::os::fd::FromRawFd>() -> T {\n\
         unsafe { T::from_raw_fd(3) }\n\
         }\n",
    );
    for path in [
        "type `std::os::unix::net::UnixStream`",
        "type `std::os::unix::net::UnixListener`",
        "type `std::os::unix::net::UnixDatagram`",
        "method `std::os::unix::fs::symlink`",
        "method `std::fs::soft_link`",
        "type `std::os::fd::FromRawFd`",
    ] {
        assert!(found.contains(&format!("disallowed {path}")), "{path}: {found}");
    }
}
