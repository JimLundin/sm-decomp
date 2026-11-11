use std::collections::BTreeSet;

use super::super::Measure;
use super::{ModuleFile, model_section, module_of, names_from, resolve};
use crate::size::outline;

/// A model crate of three modules: the root exports a type from `a` and steps with `b`'s input.
const FIXTURE: [(&str, &str); 3] = [
    (
        "lib.rs",
        "//! The fixture model. More.\nmod a;\nmod b;\n\npub use a::{Thing, helper};\n\
         use clean_types::Pair;\n\npub struct Root(Pair);\n\nimpl Root {\n    \
         pub fn step(self, input: &b::Input) -> Self {\n        self\n    }\n}\n",
    ),
    (
        "a.rs",
        "//! Things.\nuse super::b::Input;\nuse clean_types::{Fixed, World};\n\n\
         pub enum Thing {\n    One,\n    Two(u8),\n}\n\nimpl Thing {\n    \
         pub const fn make(input: Input) -> Self {\n        Self::One\n    }\n    \
         fn private(&self) {}\n}\n\npub fn helper() -> Fixed {\n    Fixed::ZERO\n}\n",
    ),
    (
        "b.rs",
        "//! Inputs: what a frame brings.\npub(crate) struct Input;\nfn unused() {}\n",
    ),
];

fn section() -> String {
    let paths: Vec<String> = FIXTURE
        .iter()
        .map(|(file, _)| format!("crates/clean_model/fixture/src/{file}"))
        .collect();
    let files: Vec<ModuleFile<'_>> = FIXTURE
        .iter()
        .zip(&paths)
        .map(|((file, text), path)| ModuleFile {
            path,
            file,
            module: module_of(file),
            area: if *file == "lib.rs" { "-" } else { "steps" }.to_owned(),
            measure: Measure::default(),
            outline: outline::of(text),
        })
        .collect();
    let shared: BTreeSet<&str> = files
        .iter()
        .flat_map(|file| names_from(&file.outline, "clean_types"))
        .collect();
    let mut out = String::new();
    model_section(&mut out, "fixture", &files, &shared);
    out
}

#[test]
fn a_model_section_shows_its_modules_interface_and_graph() {
    let out = section();
    let expected = [
        "### fixture",
        "`crates/clean_model/fixture`, 3 modules.",
        "| `a.rs` | steps | Things. | `enum Thing (2)`, `fn helper`, `Thing::make`, 1 private | 0 | 0 |",
        "| `b.rs` | steps | Inputs: what a frame brings. | `struct Input`, 1 private | 0 | 0 |",
        "| `lib.rs` | - | The fixture model. | `struct Root`, `Root::step` | 0 | 0 |",
        "- from `a`: `Thing`, `helper`",
        "Thing::make(input: Input) -> Self",
        "helper() -> Fixed",
        "Root::step(self, input: &b::Input) -> Self",
        "Shared types it takes from `clean_types`: `Fixed`, `Pair`, `World`. Other crates it uses: none.",
        "  subgraph a_steps[\"steps\"]",
        "  m_a --> m_b",
        "  m_lib --> m_a",
        "  m_lib --> m_b",
    ];
    for line in expected {
        assert!(out.lines().any(|got| got == line), "{line}\n{out}");
    }
    // A private function is no interface, and `b` uses no other module.
    assert!(!out.contains("private(&self)"), "{out}");
    assert!(!out.contains("m_b --> "), "{out}");
}

#[test]
fn the_section_is_the_same_every_time() {
    assert_eq!(section(), section());
}

#[test]
fn paths_resolve_from_the_module_that_names_them() {
    let modules: BTreeSet<Vec<&str>> =
        BTreeSet::from([vec![], vec!["a"], vec!["a", "b"], vec!["c"]]);
    assert_eq!(
        resolve(&["a", "b"], &["super", "super", "c", "X"], &modules),
        Some(vec!["c", "X"])
    );
    assert_eq!(
        resolve(&["a"], &["b", "Y"], &modules),
        Some(vec!["a", "b", "Y"])
    );
    assert_eq!(resolve(&["a"], &["crate", "c"], &modules), Some(vec!["c"]));
    // A name that is no module of the crate is another crate's, or a type's.
    assert_eq!(resolve(&["a"], &["clean_types", "Pair"], &modules), None);
    assert_eq!(module_of("chart/table.rs"), vec!["chart", "table"]);
    assert_eq!(module_of("lib.rs"), Vec::<&str>::new());
}
