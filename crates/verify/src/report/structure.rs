//! The report's structure section: how each clean model is built, for a reviewer who judges the
//! shape without opening every file (Jim, 2026-11-11: tools for reviewing the implementers'
//! work).
//!
//! From the tree alone, with the size lexer's outline (`size::outline`): the workspace's crates
//! and their dependencies, the clean-room boundary marked; per model, each module's area, what
//! its doc says it is, its measure and its items, the interface its crate root exports, the
//! shared types it takes and which module uses which; the shared types' items by the models
//! that use them; and the engine files that name each model, its bridge among them.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use serde::Deserialize;

use super::{
    ENGINE, File, Measure, Measured, SHARED_TYPES, Tree, grouped, read, relative, sources,
};
use crate::size::outline::{self, Item, Kind, Outline, Use, Visibility};

/// The root manifest, whose workspace lists the crates.
const WORKSPACE: &str = "Cargo.toml";
/// The engine's crates, as `crates/README.md` names them.
const ENGINE_CRATES: [&str; 4] = ["data", "game", "runtime", "app"];
/// The State codec's crates.
const CODEC_CRATES: [&str; 2] = ["canonical", "canonical_derive"];
/// What a clean crate's name starts with.
const CLEAN_PREFIX: &str = "clean_";
/// The shared types' crate name, as code names it.
const SHARED_CRATE: &str = "types";

/// A source read for the structure: its path from the root and its text.
pub(super) struct Source {
    path: String,
    text: String,
}

/// A workspace crate and the workspace crates it depends on.
pub(super) struct Crate {
    name: String,
    dependencies: BTreeSet<String>,
}

/// What the structure section reads, with the tree.
pub(super) struct Sources {
    /// Each rebuilt domain's model sources, in registry order.
    models: Vec<Vec<Source>>,
    /// The shared types' sources.
    shared: Vec<Source>,
    /// The engine crate's sources.
    engine: Vec<Source>,
    /// The workspace's crates, sorted by name.
    crates: Vec<Crate>,
}

#[derive(Deserialize)]
struct RootManifest {
    workspace: WorkspaceTable,
}

#[derive(Deserialize)]
struct WorkspaceTable {
    members: Vec<String>,
}

#[derive(Deserialize)]
struct Manifest {
    package: Package,
    #[serde(default)]
    dependencies: BTreeMap<String, toml::Value>,
}

#[derive(Deserialize)]
struct Package {
    name: String,
}

/// The sources of each `.rs` file under `dir`, sorted.
fn read_sources(root: &Path, dir: &Path) -> Result<Vec<Source>, String> {
    sources(&[dir.to_path_buf()])?
        .iter()
        .map(|file| {
            Ok(Source {
                path: relative(root, file),
                text: read(file)?,
            })
        })
        .collect()
}

impl Sources {
    /// The sources under `root` for the domains `names`, in that order.
    ///
    /// # Errors
    ///
    /// The path and the error when a file can't be read or a manifest doesn't parse.
    pub(super) fn read(root: &Path, names: &[&str]) -> Result<Self, String> {
        let models = names
            .iter()
            .map(|name| {
                read_sources(
                    root,
                    &root
                        .join("crates/clean")
                        .join(name.replace('-', "_"))
                        .join("src"),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let workspace: RootManifest = toml::from_str(&read(&root.join(WORKSPACE))?)
            .map_err(|e| format!("{WORKSPACE}: {e}"))?;
        let mut crates = member_folders(root, &workspace.workspace.members)?
            .iter()
            .map(|member| {
                let path = format!("{member}/Cargo.toml");
                let manifest: Manifest = toml::from_str(&read(&root.join(&path))?)
                    .map_err(|e| format!("{path}: {e}"))?;
                let dependencies = manifest
                    .dependencies
                    .iter()
                    .filter(|(_, value)| value.get("path").is_some())
                    .map(|(name, _)| name.clone())
                    .collect();
                Ok(Crate {
                    name: manifest.package.name,
                    dependencies,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        crates.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(Self {
            models,
            shared: read_sources(root, &root.join(SHARED_TYPES).join("src"))?,
            engine: read_sources(root, &root.join(ENGINE).join("src"))?,
            crates,
        })
    }
}

/// The workspace's member folders, as cargo reads `members`: a `folder/*` member is each folder
/// under `folder` that holds a manifest, in name order.
fn member_folders(root: &Path, members: &[String]) -> Result<Vec<String>, String> {
    let mut folders = Vec::new();
    for member in members {
        let Some(parent) = member.strip_suffix("/*") else {
            folders.push(member.clone());
            continue;
        };
        let entries = fs::read_dir(root.join(parent)).map_err(|e| format!("{parent}: {e}"))?;
        let mut found: Vec<String> = entries
            .filter_map(Result::ok)
            .filter(|entry| entry.path().join("Cargo.toml").is_file())
            .map(|entry| format!("{parent}/{}", entry.file_name().to_string_lossy()))
            .collect();
        found.sort();
        folders.extend(found);
    }
    Ok(folders)
}

/// Whether a crate, by its name, is in a group of the crate diagram.
type InGroup = fn(&str) -> bool;

/// A module of a crate, by its path from the crate root: `[]` is `lib.rs`.
type ModulePath<'a> = Vec<&'a str>;

/// One model file, outlined.
struct ModuleFile<'a> {
    /// Its path from the repository root.
    path: &'a str,
    /// Its path under the crate's `src`.
    file: &'a str,
    module: ModulePath<'a>,
    /// Its area, and the functions in it counted in other areas.
    area: String,
    measure: Measure,
    outline: Outline<'a>,
}

/// The module a file under a crate's `src` holds: `lib.rs` the root, `a/b.rs` and `a/b/mod.rs`
/// the module `a::b`.
fn module_of(file: &str) -> ModulePath<'_> {
    let stem = file.strip_suffix(".rs").unwrap_or(file);
    let mut module: ModulePath<'_> = stem.split('/').collect();
    if matches!(module.last(), Some(&("lib" | "main" | "mod"))) {
        module.pop();
    }
    module
}

/// The path under the crate's `src` of a path from the root.
fn under_src(path: &str) -> &str {
    path.split_once("/src/").map_or(path, |(_, file)| file)
}

/// The absolute module path `path` names from `current`, if it is the crate's own: from
/// `crate`, `super`, `self` or a module inside `current`.
fn resolve<'a>(
    current: &[&'a str],
    path: &[&'a str],
    modules: &BTreeSet<ModulePath<'a>>,
) -> Option<ModulePath<'a>> {
    let (&head, rest) = path.split_first()?;
    let mut base: ModulePath<'a> = match head {
        "crate" => Vec::new(),
        "self" => current.to_vec(),
        "super" => current
            .split_last()
            .map(|(_, parent)| parent.to_vec())
            .unwrap_or_default(),
        _ => {
            let mut child = current.to_vec();
            child.push(head);
            if !modules.contains(&child) {
                return None;
            }
            child.extend(rest);
            return Some(child);
        }
    };
    let mut rest = rest;
    while let Some((&"super", tail)) = rest.split_first() {
        base.pop();
        rest = tail;
    }
    base.extend(rest);
    Some(base)
}

/// The longest module of `modules` that `path` starts with.
fn module_holding<'a>(path: &[&'a str], modules: &BTreeSet<ModulePath<'a>>) -> ModulePath<'a> {
    (0..=path.len())
        .rev()
        .filter_map(|len| path.get(..len))
        .find(|prefix| modules.contains(*prefix))
        .map(<[&str]>::to_vec)
        .unwrap_or_default()
}

/// Every path a file names, its `use` leaves among them.
fn named_paths<'b, 'a>(outline: &'b Outline<'a>) -> impl Iterator<Item = &'b [&'a str]> {
    outline
        .uses
        .iter()
        .map(|leaf| leaf.path.as_slice())
        .chain(outline.paths.iter().map(Vec::as_slice))
}

/// The names a file takes from the crate `name`: its `use` leaves' and its paths' second names.
fn names_from<'a>(outline: &Outline<'a>, name: &str) -> BTreeSet<&'a str> {
    let leaves = outline
        .uses
        .iter()
        .filter(|leaf| leaf.path.first() == Some(&name))
        .filter_map(|leaf| {
            leaf.path
                .get(1)
                .copied()
                .or_else(|| leaf.glob.then_some("*"))
        });
    let paths = outline
        .paths
        .iter()
        .filter(|path| path.first() == Some(&name))
        .filter_map(|path| path.get(1).copied());
    leaves.chain(paths).collect()
}

/// A Mermaid node id: the name's letters and digits, the rest as `_`.
fn node(prefix: &str, name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    format!("{prefix}_{cleaned}")
}

/// A module's name in the diagram: `lib` for the root.
fn module_label(module: &[&str]) -> String {
    if module.is_empty() {
        String::from("lib")
    } else {
        module.join("::")
    }
}

/// Text for a table cell: its pipes escaped.
fn cell(text: &str) -> String {
    text.replace('|', "\\|")
}

/// The items a module declares for others, in short form, then how many it keeps private.
fn items_cell(outline: &Outline<'_>) -> String {
    let mut parts: Vec<String> = Vec::new();
    let mut methods: Vec<(&str, Vec<&str>)> = Vec::new();
    let mut private = 0usize;
    for item in &outline.items {
        if item.kind == (Kind::Module { inline: false }) || !item.module.is_empty() {
            continue;
        }
        if item.visibility == Visibility::Private {
            private = private.saturating_add(1);
            continue;
        }
        if let Some(owner) = item.owner {
            match methods.iter_mut().find(|(name, _)| *name == owner) {
                Some((_, names)) => names.push(item.name),
                None => methods.push((owner, vec![item.name])),
            }
            continue;
        }
        parts.push(format!("`{}`", short(item)));
    }
    parts.extend(methods.iter().map(|(owner, names)| match names.as_slice() {
        [one] => format!("`{owner}::{one}`"),
        _ => format!("`{owner}::{{{}}}`", names.join(", ")),
    }));
    parts.extend(
        outline
            .impls
            .iter()
            .map(|imp| format!("`impl {} for {}`", imp.trait_name, imp.self_type)),
    );
    if private > 0 {
        parts.push(format!("{private} private"));
    }
    if parts.is_empty() {
        String::from("-")
    } else {
        cell(&parts.join(", "))
    }
}

/// An item's short form: its kind's keyword and its name, an enum with its variant count.
fn short(item: &Item<'_>) -> String {
    let keyword = match item.kind {
        Kind::Struct => "struct",
        Kind::Enum { variants } => return format!("enum {} ({variants})", item.name),
        Kind::Union => "union",
        Kind::Trait => "trait",
        Kind::Alias => "type",
        Kind::Function => "fn",
        Kind::Constant => "const",
        Kind::Static => "static",
        Kind::Module { .. } => "mod",
    };
    format!("{keyword} {}", item.name)
}

/// What a model's crate root exports: its `pub use` leaves by the path they come from, and its
/// `pub mod`s.
struct Exports<'a> {
    /// Each leaf's source path (its path less its name) and its name, `*` for a glob.
    leaves: BTreeMap<String, Vec<String>>,
    /// The items exported, by (module, name).
    items: BTreeSet<(ModulePath<'a>, &'a str)>,
    /// The modules everything public of is exported: `pub mod`s and globs.
    modules: BTreeSet<ModulePath<'a>>,
}

fn exports<'a>(root: &Outline<'a>, modules: &BTreeSet<ModulePath<'a>>) -> Exports<'a> {
    let mut out = Exports {
        leaves: BTreeMap::new(),
        items: BTreeSet::new(),
        modules: BTreeSet::new(),
    };
    for leaf in root
        .uses
        .iter()
        .filter(|leaf| leaf.visibility == Visibility::Public)
    {
        let source: Vec<&str> = if leaf.glob {
            leaf.path.clone()
        } else {
            leaf.path
                .split_last()
                .map(|(_, source)| source.to_vec())
                .unwrap_or_default()
        };
        let named = leaf.name().map_or_else(|| String::from("*"), str::to_owned);
        out.leaves.entry(source.join("::")).or_default().push(named);
        let Some(absolute) = resolve(&[], &leaf.path, modules) else {
            continue;
        };
        if leaf.glob {
            out.modules.insert(absolute);
        } else if let Some((name, module)) = absolute.split_last() {
            out.items.insert((module.to_vec(), name));
        }
    }
    for item in &root.items {
        if matches!(item.kind, Kind::Module { .. }) && item.visibility == Visibility::Public {
            out.modules.insert(vec![item.name]);
        }
    }
    out
}

impl Exports<'_> {
    /// Whether a public function of `module` is the crate's interface: at the root, in a module
    /// exported whole, free and exported, or on an exported type, wherever its `impl` is.
    fn exports(&self, module: &[&str], function: &Item<'_>) -> bool {
        module.is_empty()
            || self.modules.iter().any(|whole| module.starts_with(whole))
            || self.items.iter().any(|(holder, item)| {
                function.owner.map_or_else(
                    || holder.as_slice() == module && *item == function.name,
                    |owner| *item == owner,
                )
            })
    }
}

/// The structure section for the tree.
pub(super) fn section(out: &mut String, tree: &Tree) {
    out.push_str(
        "\n## Structure\n\nHow the code is built, read from the tree (`verify::report::structure`, \
         over the size lexer's outline): the crates, then per clean model its modules, the \
         interface its crate root exports, the shared types it takes and which of its modules \
         uses which, then the shared types and the engine files that adapt each model. A \
         module's items are those it declares for others (`pub` or `pub(crate)`), an enum with \
         its variant count and a type's functions as `Type::{...}`, and how many it keeps \
         private; what its doc says it is is its module doc's first sentence.\n",
    );
    crates_section(out, &tree.sources.crates);
    let mut uses_shared: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for (domain, model) in tree.domains.iter().zip(&tree.sources.models) {
        let files = model_files(domain, model);
        let shared: BTreeSet<&str> = files
            .iter()
            .flat_map(|file| names_from(&file.outline, SHARED_CRATE))
            .collect();
        model_section(out, domain.domain.name, &files, &shared);
        for name in shared {
            uses_shared
                .entry(name)
                .or_default()
                .insert(domain.domain.name);
        }
    }
    shared_section(out, &tree.sources.shared, &uses_shared);
    bridges_section(out, tree);
}

fn crates_section(out: &mut String, crates: &[Crate]) {
    out.push_str(
        "\n### Crates\n\nThe workspace's crates and the workspace crates each depends on. The \
         clean room's crates are built from their specs alone; the engine runs them through its \
         bridges, and the verification crates hold them to the oracle.\n\n```mermaid\ngraph LR\n",
    );
    let groups: [(&str, &str, InGroup); 4] = [
        ("engine", "engine", |name| ENGINE_CRATES.contains(&name)),
        ("clean", "clean room", |name| name.starts_with(CLEAN_PREFIX)),
        ("codec", "State codec", |name| CODEC_CRATES.contains(&name)),
        ("verification", "verification", |name| {
            !ENGINE_CRATES.contains(&name)
                && !CODEC_CRATES.contains(&name)
                && !name.starts_with(CLEAN_PREFIX)
        }),
    ];
    for (id, label, holds) in groups {
        let _ = writeln!(out, "  subgraph {id}[\"{label}\"]");
        for krate in crates.iter().filter(|krate| holds(&krate.name)) {
            let _ = writeln!(out, "    {}[\"{}\"]", node("c", &krate.name), krate.name);
        }
        out.push_str("  end\n");
    }
    for krate in crates {
        for dependency in &krate.dependencies {
            let _ = writeln!(
                out,
                "  {} --> {}",
                node("c", &krate.name),
                node("c", dependency)
            );
        }
    }
    out.push_str("```\n");
}

/// A model's files, outlined, with their areas and whole-file measures.
fn model_files<'a>(domain: &Measured, sources: &'a [Source]) -> Vec<ModuleFile<'a>> {
    sources
        .iter()
        .map(|source| {
            let file = under_src(&source.path);
            let parts: Vec<_> = domain
                .model
                .iter()
                .filter(|part| part.file.path == source.path)
                .collect();
            let area = parts
                .iter()
                .find(|part| part.function.is_none())
                .and_then(|part| part.area)
                .unwrap_or("-");
            let apart: Vec<String> = parts
                .iter()
                .filter_map(|part| Some(format!("`fn {}` in {}", part.function?, part.area?)))
                .collect();
            let area = if apart.is_empty() {
                area.to_owned()
            } else {
                format!("{area}; {}", apart.join(", "))
            };
            ModuleFile {
                path: &source.path,
                file,
                module: module_of(file),
                area,
                measure: parts.iter().map(|part| part.file.measure).sum(),
                outline: outline::of(&source.text),
            }
        })
        .collect()
}

fn model_section(out: &mut String, name: &str, files: &[ModuleFile<'_>], shared: &BTreeSet<&str>) {
    let crate_dir = files
        .first()
        .and_then(|file| file.path.split_once("/src/"))
        .map_or("", |(dir, _)| dir);
    let _ = write!(
        out,
        "\n### {name}\n\n`{crate_dir}`, {} modules.\n\n",
        files.len()
    );
    super::table(
        out,
        4,
        &["Module", "Area", "Says it is", "Items", "Score", "Lines"],
    );
    for file in files {
        let _ = writeln!(
            out,
            "| `{}` | {} | {} | {} | {} | {} |",
            file.file,
            file.area,
            cell(file.outline.doc.as_deref().unwrap_or("-")),
            items_cell(&file.outline),
            file.measure.score(),
            grouped(file.measure.lines),
        );
    }
    let modules: BTreeSet<ModulePath<'_>> = files.iter().map(|file| file.module.clone()).collect();
    interface(out, files, &modules);
    let others: BTreeSet<&str> = files
        .iter()
        .flat_map(|file| &file.outline.uses)
        .filter_map(|leaf: &Use<'_>| leaf.path.first().copied())
        .filter(|head| head.starts_with(|c: char| c.is_ascii_lowercase()))
        .filter(|head| !matches!(*head, "crate" | "self" | "super" | SHARED_CRATE))
        .filter(|head| !modules.contains(&vec![*head]))
        .collect();
    let names = |set: &BTreeSet<&str>| {
        if set.is_empty() {
            String::from("none")
        } else {
            set.iter()
                .map(|name| format!("`{name}`"))
                .collect::<Vec<_>>()
                .join(", ")
        }
    };
    let _ = writeln!(
        out,
        "\nShared types it takes from `{SHARED_CRATE}`: {}. Other crates it uses: {}.",
        names(shared),
        names(&others)
    );
    module_graph(out, files, &modules);
}

/// The crate root's exports and the public functions a caller reaches through them.
fn interface(out: &mut String, files: &[ModuleFile<'_>], modules: &BTreeSet<ModulePath<'_>>) {
    let Some(root) = files.iter().find(|file| file.module.is_empty()) else {
        return;
    };
    let exports = exports(&root.outline, modules);
    out.push_str("\nIts crate root exports:\n\n");
    for (source, names) in &exports.leaves {
        let names: Vec<String> = names.iter().map(|name| format!("`{name}`")).collect();
        let _ = writeln!(out, "- from `{source}`: {}", names.join(", "));
    }
    for module in &exports.modules {
        if root.outline.items.iter().any(|item| {
            matches!(item.kind, Kind::Module { .. }) && module.as_slice() == [item.name]
        }) {
            let _ = writeln!(out, "- the module `{}`", module.join("::"));
        }
    }
    let functions: Vec<String> = files
        .iter()
        .flat_map(|file| {
            file.outline
                .items
                .iter()
                .filter(|item| {
                    item.kind == Kind::Function
                        && item.visibility == Visibility::Public
                        && exports.exports(&file.module, item)
                })
                .filter_map(|item| {
                    let signature = item.signature.as_deref()?;
                    Some(item.owner.map_or_else(
                        || signature.to_owned(),
                        |owner| format!("{owner}::{signature}"),
                    ))
                })
        })
        .collect();
    if !functions.is_empty() {
        let _ = write!(
            out,
            "\nThe public functions a caller reaches through them:\n\n```rust\n{}\n```\n",
            functions.join("\n")
        );
    }
}

/// The modules as a Mermaid graph, grouped by area: an arrow from a module to each it uses.
fn module_graph(out: &mut String, files: &[ModuleFile<'_>], modules: &BTreeSet<ModulePath<'_>>) {
    out.push_str("\nWhich module uses which, grouped by area (a module's own area, its functions counted elsewhere aside):\n\n```mermaid\ngraph LR\n");
    let mut areas: BTreeMap<&str, Vec<&ModuleFile<'_>>> = BTreeMap::new();
    for file in files {
        let area = file.area.split(';').next().unwrap_or("-");
        areas.entry(area).or_default().push(file);
    }
    for (area, members) in &areas {
        let label = if *area == "-" { "in no area" } else { area };
        let _ = writeln!(out, "  subgraph {}[\"{label}\"]", node("a", label));
        for file in members {
            let label = module_label(&file.module);
            let _ = writeln!(out, "    {}[\"{label}\"]", node("m", &label));
        }
        out.push_str("  end\n");
    }
    let mut edges: BTreeSet<(String, String)> = BTreeSet::new();
    for file in files {
        for path in named_paths(&file.outline) {
            let Some(absolute) = resolve(&file.module, path, modules) else {
                continue;
            };
            let target = module_holding(&absolute, modules);
            if target != file.module {
                edges.insert((module_label(&file.module), module_label(&target)));
            }
        }
    }
    for (from, to) in &edges {
        let _ = writeln!(out, "  {} --> {}", node("m", from), node("m", to));
    }
    out.push_str("```\n");
}

fn shared_section(out: &mut String, shared: &[Source], uses: &BTreeMap<&str, BTreeSet<&str>>) {
    out.push_str(
        "\n### Shared types by model\n\nThe public items of `crates/clean/types` the clean models share, \
         and the models that name each.\n\n",
    );
    super::table(out, 2, &["Item", "Used by"]);
    for source in shared {
        let outline = outline::of(&source.text);
        for item in outline.items.iter().filter(|item| {
            item.visibility == Visibility::Public && item.owner.is_none() && item.module.is_empty()
        }) {
            let models_using = uses.get(item.name).map_or_else(
                || String::from("-"),
                |models| models.iter().copied().collect::<Vec<_>>().join(", "),
            );
            // An inline module's row names its items: the models name the module.
            let held: Vec<&str> = outline
                .items
                .iter()
                .filter(|inner| inner.module.first() == Some(&item.name) && inner.owner.is_none())
                .map(|inner| inner.name)
                .collect();
            let holds = if held.is_empty() {
                String::new()
            } else {
                format!(" ({})", held.join(", "))
            };
            let _ = writeln!(out, "| `{}`{holds} | {models_using} |", short(item));
        }
    }
}

fn bridges_section(out: &mut String, tree: &Tree) {
    out.push_str(
        "\n### Bridges\n\nThe engine files (`crates/game/src`) that name each model's crate: its \
         bridge, which the registry names, and the others. Names are what each takes from the \
         model.\n\n",
    );
    super::table(
        out,
        4,
        &["Model", "Engine file", "Part", "Names", "Score", "Lines"],
    );
    let outlines: Vec<(&Source, Outline<'_>)> = tree
        .sources
        .engine
        .iter()
        .map(|source| (source, outline::of(&source.text)))
        .collect();
    for domain in &tree.domains {
        let krate = domain.domain.name.replace('-', "_");
        for (source, outline) in &outlines {
            let bridge = domain.bridge.iter().find(|file| file.path == source.path);
            let names = names_from(outline, &krate);
            if bridge.is_none() && names.is_empty() {
                continue;
            }
            let measure = tree
                .domains
                .iter()
                .flat_map(|domain| &domain.bridge)
                .chain(&tree.engine)
                .find(|file| file.path == source.path)
                .map(|file: &File| file.measure)
                .unwrap_or_default();
            let names: Vec<String> = names.iter().map(|name| format!("`{name}`")).collect();
            let _ = writeln!(
                out,
                "| {} | `{}` | {} | {} | {} | {} |",
                domain.domain.name,
                source.path,
                if bridge.is_some() {
                    "bridge"
                } else {
                    "names it"
                },
                if names.is_empty() {
                    String::from("-")
                } else {
                    names.join(", ")
                },
                measure.score(),
                grouped(measure.lines),
            );
        }
    }
}

#[cfg(test)]
mod tests;
