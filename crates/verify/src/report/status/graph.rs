//! The domain graph: the plan's domains and the reads between them ([`section`]), and the check
//! that holds the clean crates and their bridges to it ([`undeclared`]).
//!
//! A clean crate is `crates/clean/<crate>`, a model's name with underscores for dashes, and its
//! bridge the engine's files `crates/game/src/<crate>_bridge*`. Either may use the shared
//! `types`, its own crate and the crates of the domains its domains read, and no other clean
//! crate: a use the plan doesn't declare is a dependency the domain graph hides.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use super::{Area, PLAN, Plan, Stage};
use crate::report::{ENGINE, prefixed, read, relative};

/// The clean crates' folder.
const CLEAN: &str = "crates/clean";
/// The clean crate every other may use.
const SHARED: &str = "types";

/// Mermaid's name for a plan id: ids such as `C2-C11` hold a dash.
fn node(id: &str) -> String {
    id.replace('-', "_")
}

/// The report's domain graph section: each area's domains as a subgraph, an arrow from each
/// domain to each domain that reads it, and the domains whose work has started marked.
pub(in crate::report) fn section(out: &mut String, plan: &Plan) {
    let _ = write!(
        out,
        "\n## Domain graph\n\nThe plan's domains ([domains.toml](architecture/domains.toml)) by \
         area, with an arrow from a domain to each domain that reads its exports; the domains \
         whose work has started are filled. Audio's sound requests are a convention, not \
         arrows. The report's tests fail when a clean crate or its bridge uses a clean crate \
         its domains don't read.\n\n```mermaid\nflowchart LR\n"
    );
    for area in Area::ALL {
        let mut domains = plan
            .domain
            .iter()
            .filter(|domain| domain.area == area)
            .peekable();
        if domains.peek().is_none() {
            continue;
        }
        let _ = writeln!(out, "  subgraph area_{} [{}]", area.word(), area.word());
        for domain in domains {
            let _ = writeln!(
                out,
                "    {}[\"{} {}\"]",
                node(&domain.id),
                domain.id,
                domain.name
            );
        }
        out.push_str("  end\n");
    }
    for domain in &plan.domain {
        for read in &domain.reads {
            let _ = writeln!(out, "  {} --> {}", node(read), node(&domain.id));
        }
    }
    let started: Vec<String> = plan
        .domain
        .iter()
        .filter(|domain| domain.status != Stage::NotStarted)
        .map(|domain| node(&domain.id))
        .collect();
    if !started.is_empty() {
        let _ = writeln!(
            out,
            "  classDef started fill:#cde8c4,stroke:#3c7a2e\n  class {} started",
            started.join(",")
        );
    }
    out.push_str("```\n");
}

/// The `.rs` files at or under `path`, tests included: a test's use is a use.
fn rust_files(path: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    if path.is_file() {
        if path.extension().is_some_and(|extension| extension == "rs") {
            out.push(path.to_path_buf());
        }
        return Ok(());
    }
    for entry in fs::read_dir(path).map_err(|e| format!("{}: {e}", path.display()))? {
        let entry = entry.map_err(|e| format!("{}: {e}", path.display()))?;
        rust_files(&entry.path(), out)?;
    }
    Ok(())
}

/// Whether `text` names the crate `name` as a path's first segment, `name::`.
fn names_crate(text: &str, name: &str) -> bool {
    let pattern = format!("{name}::");
    text.match_indices(&pattern).any(|(at, _)| {
        !text[..at]
            .chars()
            .next_back()
            .is_some_and(|before| before.is_alphanumeric() || before == '_' || before == ':')
    })
}

/// The clean crates, by folder name.
fn clean_crates(root: &Path) -> Result<BTreeSet<String>, String> {
    let dir = root.join(CLEAN);
    let mut crates = BTreeSet::new();
    for entry in fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let path = entry.map_err(|e| format!("{}: {e}", dir.display()))?.path();
        if path.join("Cargo.toml").is_file()
            && let Some(name) = path.file_name()
        {
            crates.insert(name.to_string_lossy().into_owned());
        }
    }
    Ok(crates)
}

/// The clean crates a crate's manifest depends on, in any dependency table.
fn manifest_uses(manifest: &str, crates: &BTreeSet<String>) -> Result<BTreeSet<String>, String> {
    let table: toml::Table = toml::from_str(manifest).map_err(|e| e.to_string())?;
    Ok(["dependencies", "dev-dependencies", "build-dependencies"]
        .iter()
        .filter_map(|key| table.get(*key).and_then(toml::Value::as_table))
        .flat_map(toml::Table::keys)
        .filter(|name| crates.contains(*name))
        .cloned()
        .collect())
}

impl Plan {
    /// The clean crates a crate's domains may use: [`SHARED`], its own, and those of the
    /// domains they read. `None` when no plan domain is held by the crate's model.
    fn allowed(&self, crate_name: &str) -> Option<BTreeSet<String>> {
        let model = crate_name.replace('_', "-");
        let own: Vec<_> = self.holding(&model).collect();
        if own.is_empty() {
            return None;
        }
        let read: BTreeSet<&str> = own
            .iter()
            .flat_map(|domain| &domain.reads)
            .map(String::as_str)
            .collect();
        let mut allowed: BTreeSet<String> = self
            .domain
            .iter()
            .filter(|domain| read.contains(domain.id.as_str()))
            .filter_map(|domain| domain.model.as_deref())
            .map(|model| model.replace('-', "_"))
            .collect();
        allowed.insert(SHARED.to_owned());
        allowed.insert(crate_name.to_owned());
        Some(allowed)
    }
}

/// The uses of clean crates that the plan doesn't declare: each clean crate's manifest and its
/// bridge's files may use only the crates [`Plan::allowed`] gives it.
///
/// # Errors
///
/// Every undeclared use, one a line, naming the user, the crate it uses and the plan to amend;
/// or why a crate or a bridge can't be read.
pub(in crate::report) fn undeclared(root: &Path, plan: &Plan) -> Result<(), String> {
    let crates = clean_crates(root)?;
    let mut found = Vec::new();
    for name in crates.iter().filter(|name| *name != SHARED) {
        let Some(allowed) = plan.allowed(name) else {
            found.push(format!(
                "{CLEAN}/{name}: no domain of {PLAN} is held by its model"
            ));
            continue;
        };
        let manifest = root.join(CLEAN).join(name).join("Cargo.toml");
        let depends = manifest_uses(&read(&manifest)?, &crates)
            .map_err(|e| format!("{}: {e}", relative(root, &manifest)))?;
        let mut checked = vec![(relative(root, &manifest), depends)];
        let mut bridge = Vec::new();
        for path in prefixed(&root.join(ENGINE).join("src"), &format!("{name}_bridge"))? {
            rust_files(&path, &mut bridge)?;
        }
        bridge.sort();
        for file in bridge {
            let text = read(&file)?;
            let uses = crates
                .iter()
                .filter(|other| names_crate(&text, other))
                .cloned()
                .collect();
            checked.push((relative(root, &file), uses));
        }
        for (user, uses) in checked {
            for used in uses.difference(&allowed) {
                found.push(format!(
                    "{user} uses {used}, which no domain of {name} reads in {PLAN}"
                ));
            }
        }
    }
    if found.is_empty() {
        Ok(())
    } else {
        Err(found.join("\n"))
    }
}
