//! Public wrapper discovery and source facts agree on real Rust inputs.
use anyhow::Result;
use offline_verification::{
    HostInputs, OfflineFamilies,
    authoring::WrapperRequest,
    source_facts::{self, RustInput},
};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

#[test]
fn wrapper_discovery_ignores_prose_and_literal_module_names() -> Result<()> {
    let directory = tool_execution::testing::Directory::new()?;
    let sources = directory.0.join("crates/port/src");
    let run = directory.0.join("run");
    fs::create_dir_all(&sources)?;
    fs::create_dir(&run)?;
    fs::write(sources.join("memory.rs"), "pub struct Wram;")?;
    fs::write(sources.join("dependency.rs"), "pub fn step() {}")?;
    fs::write(sources.join("unrelated.rs"), "pub fn step() {}")?;
    fs::write(
        sources.join("candidate.rs"),
        r###"
/// ROM-aware preinstructions include native room projectile spawning.
/* unrelated /* unrelated */ unrelated */
pub fn step() {
    crate::dependency::step();
    let _ = "unrelated";
    let _ = r##"unrelated"##;
    let _ = b"unrelated";
    let _ = br#"unrelated"#;
}
"###,
    )?;
    let verifier = OfflineFamilies::new(HostInputs::new(
        &[("fixture-host.rs", "// unused by source generation")],
        "0".repeat(64),
    )?);
    let wrapper = verifier.generate_wrapper(WrapperRequest {
        checkout: directory.0.clone(),
        run,
        modules: vec![sources.join("candidate.rs")],
        imports: vec![],
    })?;
    let text = fs::read_to_string(wrapper.path)?;
    assert!(text.contains("pub mod dependency;"), "{text}");
    assert!(!text.contains("pub mod unrelated;"), "prose expanded the compiler closure: {text}");
    Ok(())
}

#[test]
fn facts_ignore_prose_preserve_exact_bytes_and_are_shareable() -> Result<()> {
    let source = br###"
/// ROM-aware preinstructions include native room projectile spawning.
/// Trail coordinates include native Ceres Mode7 arithmetic and ROM trail frames.
/* doc include /* dependency */ include_str */
pub fn step() {
    let include = 1;
    let env = 2;
    let File = 3;
    let _ = (include, env, File);
    let _ = "literal_only dependency";
    let _ = r##"raw_only include!("source.rs")"##;
    let _ = b"byte_only include_str";
    let _ = br#"raw_byte_only include_bytes"#;
    let _ = '\'';
}
"###;
    let facts = source_facts::scan(Path::new("candidate.rs"), source)?;
    facts.require_complete()?;
    assert!(facts.inputs().is_empty());
    for excluded in [
        "doc",
        "ROM",
        "dependency",
        "include_str",
        "include_bytes",
        "literal_only",
        "raw_only",
        "byte_only",
        "raw_byte_only",
    ] {
        assert!(!facts.identifiers().contains(excluded), "prose became code: {excluded}");
    }
    for actual in ["step", "include", "env", "File"] {
        assert!(facts.identifiers().contains(actual));
    }
    assert_eq!(facts.sha256(), format!("{:x}", Sha256::digest(source)));
    assert_eq!(facts.source_bytes(), source.len());
    let changed = [source.as_slice(), b"\n// different prose\n"].concat();
    let changed = source_facts::scan(Path::new("candidate.rs"), &changed)?;
    assert_eq!(facts.identifiers(), changed.identifiers());
    assert_ne!(facts.sha256(), changed.sha256());
    let aliased = source_facts::scan(Path::new("alias.rs"), source)?;
    assert_eq!(facts.sha256(), aliased.sha256());
    assert_ne!(facts.logical_path(), aliased.logical_path());
    std::thread::scope(|scope| {
        assert_eq!(scope.spawn(|| facts.identifiers().clone()).join().unwrap(), *facts.identifiers());
    });
    Ok(())
}

#[test]
fn facts_keep_cfg_union_aliases_inline_modules_and_macro_arguments() -> Result<()> {
    let source = br#"
#[cfg(any())] use crate::{r#left as active, branch::{child, leaf as renamed}};
#[cfg(not(any()))] use crate::right;
mod inline { pub fn step() { crate::nested::step(); } }
pub fn step() {
    assert_eq!(crate::macro_argument::read(), 1);
    let _ = matches!(crate::another::read(), Some(_));
    let _ = vec![crate::vector_input::read()];
}
"#;
    let facts = source_facts::scan(Path::new("candidate.rs"), source)?;
    facts.require_complete()?;
    for dependency in [
        "left",
        "active",
        "branch",
        "child",
        "leaf",
        "renamed",
        "right",
        "inline",
        "nested",
        "macro_argument",
        "another",
        "vector_input",
    ] {
        assert!(facts.identifiers().contains(dependency), "missing real code identifier {dependency}");
    }
    Ok(())
}

#[test]
fn real_input_macros_keep_literal_or_dynamic_input_facts() -> Result<()> {
    let cases = [
        ("include!(\"nested.rs\");", RustInput::FileMacro { name: "include".into(), path: Some("nested.rs".into()) }),
        (
            "const TEXT: &str = include_str!(r#\"data.txt\"#);",
            RustInput::FileMacro { name: "include_str".into(), path: Some("data.txt".into()) },
        ),
        (
            "const DATA: &[u8] = include_bytes!(concat!(\"data\", \".bin\"));",
            RustInput::FileMacro { name: "include_bytes".into(), path: None },
        ),
        (
            "const CONFIG: &str = env!(\"CONFIG\");",
            RustInput::EnvironmentMacro { name: "env".into(), variable: Some("CONFIG".into()) },
        ),
        (
            "const CONFIG: Option<&str> = option_env!(\"CONFIG\");",
            RustInput::EnvironmentMacro { name: "option_env".into(), variable: Some("CONFIG".into()) },
        ),
        (
            "#[doc = include_str!(\"README.md\")] pub fn step() {}",
            RustInput::FileMacro { name: "include_str".into(), path: Some("README.md".into()) },
        ),
        ("#[path = \"external.rs\"] mod selected;", RustInput::ModulePath { path: Some("external.rs".into()) }),
        ("mod external;", RustInput::ExternalModule { name: "external".into() }),
        (
            "pub fn step() { assert_eq!(include_str!(\"value.txt\"), \"value\"); }",
            RustInput::FileMacro { name: "include_str".into(), path: Some("value.txt".into()) },
        ),
    ];
    for (source, input) in cases {
        let facts = source_facts::scan(Path::new("candidate.rs"), source.as_bytes())?;
        assert!(facts.require_complete().is_err(), "input became complete: {source}");
        assert!(facts.inputs().contains(&input), "missing input {input:?}: {:?}", facts.inputs());
    }
    Ok(())
}

#[test]
fn unmodeled_expansion_aliases_runtime_and_invalid_source_never_exclude() -> Result<()> {
    for source in [
        "macro_rules! generated { () => { mod generated_input; } } generated!();",
        "#[custom_attribute] pub fn step() {}",
        "#[derive(CustomDerive)] pub struct State;",
        "#[cfg_attr(unix, path = \"other.rs\")] mod selected;",
        "use crate::macros::vec; pub fn step() { let _ = vec![1]; }",
        "use crate::macros::*; pub fn step() { let _ = matches!(1, 1); }",
        "pub fn step() { assert_eq!(external::matches!(1), true); }",
        "pub fn step() { let _ = vec![{ use external::vec; vec![1] }]; }",
        "pub fn step() { let _ = std::env::var(\"RUNTIME\"); }",
        "pub fn step() { assert_eq!(std::env::var(\"RUNTIME\").ok(), None); }",
        "use std::fs::read as load; pub fn step() { let _ = load(\"input\"); }",
        "use std::{env as settings, process::Command as Spawn}; pub fn step() {}",
        "unsafe extern \"C\" { fn read_external(); }",
        "extern crate generated_dependency;",
        "pub fn step( { missing",
        "/* unterminated Rust comment",
        "dependency dependency dependency",
    ] {
        let facts = source_facts::scan(Path::new("candidate.rs"), source.as_bytes())?;
        assert!(facts.require_complete().is_err(), "unmodeled source became a complete closure: {source}");
        assert!(!facts.incompleteness().is_empty());
    }
    let facts = source_facts::scan(Path::new("candidate.rs"), b"\xff")?;
    assert!(!facts.is_complete());
    Ok(())
}

#[test]
fn scanner_limits_and_inherited_deadlines_are_hard_errors() -> Result<()> {
    let source = b"pub fn step() {}";
    assert!(tool_execution::bounded::until(0, 1024, || source_facts::scan(Path::new("candidate.rs"), source)).is_err());
    assert!(source_facts::scan(Path::new("candidate.rs"), &vec![b' '; 8 * 1024 * 1024 + 1]).is_err());
    let nested = format!("pub fn step() {{ let _ = {}1{}; }}", "(".repeat(200), ")".repeat(200));
    assert!(source_facts::scan(Path::new("candidate.rs"), nested.as_bytes()).is_err());
    let unary = format!("pub fn step() {{ let _ = {}true; }}", "!".repeat(2000));
    assert!(source_facts::scan(Path::new("candidate.rs"), unary.as_bytes()).is_err());
    Ok(())
}
