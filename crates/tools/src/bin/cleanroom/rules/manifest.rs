use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Dep {
    Version(String),
    Table(DepTable),
}

#[derive(Deserialize, Default, Clone)]
pub struct DepTable {
    version: Option<String>,
    features: Option<Vec<String>>,
    package: Option<String>,
    path: Option<String>,
    git: Option<toml::Value>,
    registry: Option<toml::Value>,
    #[serde(rename = "registry-index")]
    registry_index: Option<toml::Value>,
    workspace: Option<bool>,
    #[serde(flatten)]
    other: BTreeMap<String, toml::Value>,
}

impl DepTable {
    fn extra_keys(&self) -> impl Iterator<Item = &String> {
        self.other.keys().filter(|key| *key != "default-features")
    }
}

impl Dep {
    pub fn version(&self) -> Option<String> {
        match self {
            Self::Version(version) => Some(version.clone()),
            Self::Table(table) => table.version.clone(),
        }
    }

    fn table(&self) -> DepTable {
        match self {
            Self::Version(version) => DepTable {
                version: Some(version.clone()),
                ..DepTable::default()
            },
            Self::Table(table) => table.clone(),
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Edition {
    Named(String),
    Inherited { workspace: bool },
}

#[derive(Deserialize)]
struct Package {
    workspace: Option<String>,
    build: Option<toml::Value>,
    links: Option<toml::Value>,
    edition: Option<Edition>,
}

#[derive(Deserialize)]
struct Target {
    path: Option<String>,
    #[serde(rename = "proc-macro")]
    proc_macro: Option<toml::Value>,
    #[serde(rename = "proc_macro")]
    proc_macro_underscore: Option<toml::Value>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Targets {
    One(Target),
    Many(Vec<Target>),
}

#[derive(Deserialize, Default)]
struct Scope {
    #[serde(default)]
    dependencies: BTreeMap<String, Dep>,
    #[serde(default, rename = "dev-dependencies")]
    dev_dependencies: BTreeMap<String, Dep>,
    #[serde(default, rename = "dev_dependencies")]
    dev_dependencies_underscore: BTreeMap<String, Dep>,
    #[serde(rename = "build-dependencies")]
    build_dependencies: Option<toml::Value>,
    #[serde(rename = "build_dependencies")]
    build_dependencies_underscore: Option<toml::Value>,
}

#[derive(Deserialize)]
struct Lints {
    workspace: Option<bool>,
}

#[derive(Deserialize)]
struct Manifest {
    package: Option<Package>,
    lints: Option<Lints>,
    workspace: Option<toml::Value>,
    patch: Option<toml::Value>,
    replace: Option<toml::Value>,
    lib: Option<Targets>,
    bin: Option<Targets>,
    test: Option<Targets>,
    example: Option<Targets>,
    bench: Option<Targets>,
    #[serde(default)]
    target: BTreeMap<String, Scope>,
    #[serde(flatten)]
    scope: Scope,
}

#[derive(Deserialize, Default)]
struct WorkspacePackage {
    edition: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct Workspace {
    #[serde(default)]
    package: WorkspacePackage,
    #[serde(default)]
    pub dependencies: BTreeMap<String, Dep>,
}

#[derive(Deserialize, Default)]
pub struct WorkspaceFile {
    #[serde(default)]
    pub workspace: Workspace,
}

pub struct Approval {
    pub version: String,
    pub kinds: BTreeSet<String>,
    pub features: BTreeSet<String>,
}

pub struct Checked<'a> {
    pub crate_dir: &'a Path,
    pub root: &'a Path,
    pub allowed_paths: &'a BTreeSet<PathBuf>,
    pub approved: &'a BTreeMap<String, Approval>,
}

pub fn normal(path: &Path) -> Option<PathBuf> {
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            Component::ParentDir => {
                if !out.pop() {
                    return None;
                }
            }
            Component::CurDir => {}
            Component::Normal(name) => out.push(name),
            Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    Some(out)
}

type Findings = Vec<(&'static str, String)>;

/// The keys a clean crate's manifest may hold, by table. Cargo reads keys the structs above skip,
/// such as a target's `crate-type`, so every other key is refused.
const TOP_KEYS: &[&str] = &[
    "package",
    "lints",
    "dependencies",
    "dev-dependencies",
    "dev_dependencies",
    "build-dependencies",
    "build_dependencies",
    "target",
    "lib",
    "bin",
    "test",
    "example",
    "bench",
    "workspace",
    "patch",
    "replace",
];
const PACKAGE_KEYS: &[&str] = &[
    "name",
    "version",
    "edition",
    "license",
    "publish",
    "description",
    "workspace",
    "build",
    "links",
];
const TARGET_KEYS: &[&str] = &[
    "name",
    "path",
    "test",
    "doctest",
    "bench",
    "harness",
    "proc-macro",
    "proc_macro",
];
const SCOPE_KEYS: &[&str] = &[
    "dependencies",
    "dev-dependencies",
    "dev_dependencies",
    "build-dependencies",
    "build_dependencies",
];
const LINTS_KEYS: &[&str] = &["workspace"];

/// Every key outside `TOP_KEYS` and the allow-list of its table, and every `crate-type`.
fn keys(manifest: &toml::Table, out: &mut Findings) {
    let mut unknown = |table: &toml::Table, allowed: &[&str], at: &str| {
        for key in table.keys() {
            if matches!(key.as_str(), "crate-type" | "crate_type") {
                out.push(("crate-type", format!("{at}{key}")));
            } else if !allowed.contains(&key.as_str()) {
                out.push(("manifest-key", format!("{at}{key}")));
            }
        }
    };
    unknown(manifest, TOP_KEYS, "");
    for (section, allowed) in [("package", PACKAGE_KEYS), ("lints", LINTS_KEYS)] {
        if let Some(table) = manifest.get(section).and_then(toml::Value::as_table) {
            unknown(table, allowed, &format!("{section}."));
        }
    }
    for section in ["lib", "bin", "test", "example", "bench"] {
        let targets = match manifest.get(section) {
            Some(toml::Value::Array(list)) => list.iter().collect(),
            Some(one) => vec![one],
            None => Vec::new(),
        };
        for table in targets.into_iter().filter_map(toml::Value::as_table) {
            unknown(table, TARGET_KEYS, &format!("{section}."));
        }
    }
    if let Some(cfgs) = manifest.get("target").and_then(toml::Value::as_table) {
        for (cfg, scope) in cfgs {
            if let Some(table) = scope.as_table() {
                unknown(table, SCOPE_KEYS, &format!("target.{cfg}."));
            }
        }
    }
}

impl Checked<'_> {
    pub fn check(&self, manifest: &str, workspace: &str) -> Findings {
        let mut out = Vec::new();
        let parsed = toml::from_str::<Manifest>(manifest)
            .map_err(|e| e.to_string())
            .and_then(|m| {
                toml::from_str::<WorkspaceFile>(workspace)
                    .map(|w| (m, w.workspace))
                    .map_err(|e| e.to_string())
            });
        if let Ok(table) = toml::from_str::<toml::Table>(manifest) {
            keys(&table, &mut out);
        }
        match parsed {
            Ok((manifest, workspace)) => self.manifest(&manifest, &workspace, &mut out),
            Err(e) => out.push((
                "manifest",
                format!("invalid manifest or dependency policy: {e}"),
            )),
        }
        out
    }

    fn manifest(&self, manifest: &Manifest, workspace: &Workspace, out: &mut Findings) {
        for (name, section) in [
            ("workspace", &manifest.workspace),
            ("patch", &manifest.patch),
            ("replace", &manifest.replace),
        ] {
            if section.is_some() {
                out.push(("manifest-section", format!("[{name}]")));
            }
        }
        if let Some(package) = &manifest.package {
            if package
                .workspace
                .as_ref()
                .is_some_and(|w| normal(&self.crate_dir.join(w)).as_deref() != Some(self.root))
            {
                out.push((
                    "dep-workspace",
                    "package selects another workspace".to_owned(),
                ));
            }
            if package.build.is_some() {
                out.push(("build-key", "package.build".to_owned()));
            }
            if package.links.is_some() {
                out.push(("links-key", "package.links".to_owned()));
            }
        }
        let edition = match manifest.package.as_ref().and_then(|p| p.edition.as_ref()) {
            Some(Edition::Named(name)) => Some(name.as_str()),
            Some(Edition::Inherited { workspace: true }) => workspace.package.edition.as_deref(),
            Some(Edition::Inherited { workspace: false }) | None => None,
        };
        if edition != Some("2024") {
            out.push((
                "edition",
                format!("expected edition 2024, found {edition:?}"),
            ));
        }
        if manifest.lints.as_ref().and_then(|l| l.workspace) != Some(true) {
            out.push(("lints-key", "no `[lints] workspace = true`".to_owned()));
        }
        for (key, targets) in [
            ("lib", &manifest.lib),
            ("bin", &manifest.bin),
            ("test", &manifest.test),
            ("example", &manifest.example),
            ("bench", &manifest.bench),
        ] {
            let list = match targets {
                Some(Targets::One(t)) => std::slice::from_ref(t),
                Some(Targets::Many(ts)) => ts.as_slice(),
                None => &[],
            };
            for target in list {
                if target.proc_macro.is_some() || target.proc_macro_underscore.is_some() {
                    out.push(("proc-macro", key.to_owned()));
                }
                if let Some(path) = &target.path {
                    let outside = Path::new(path).is_absolute()
                        || Path::new(path)
                            .components()
                            .any(|c| c == Component::ParentDir)
                        || path.get(1..2) == Some(":");
                    if outside {
                        out.push(("target-path", path.clone()));
                    }
                }
            }
        }
        for scope in std::iter::once(&manifest.scope).chain(manifest.target.values()) {
            if scope.build_dependencies.is_some() || scope.build_dependencies_underscore.is_some() {
                out.push(("build-deps", "build dependencies".to_owned()));
            }
            for (deps, kind) in [
                (&scope.dependencies, "normal"),
                (&scope.dev_dependencies, "dev"),
                (&scope.dev_dependencies_underscore, "dev"),
            ] {
                for (name, dep) in deps {
                    self.dependency(name, dep, kind, workspace, out);
                }
            }
        }
    }

    fn resolve<'b>(
        &'b self,
        name: &str,
        dep: &Dep,
        workspace: &Workspace,
        out: &mut Findings,
    ) -> Option<(DepTable, &'b Path)> {
        let mut table = dep.table();
        let Some(inherit) = table.workspace else {
            return Some((table, self.crate_dir));
        };
        let Some(inherited) = workspace.dependencies.get(name).filter(|_| inherit) else {
            out.push((
                "dep-workspace",
                format!("{name}: missing workspace dependency"),
            ));
            return None;
        };
        let overridden = table.version.is_some()
            || table.package.is_some()
            || table.path.is_some()
            || table.git.is_some()
            || table.registry.is_some()
            || table.registry_index.is_some()
            || table.extra_keys().next().is_some();
        if overridden {
            out.push((
                "dep-registry",
                format!("{name}: invalid override on an inherited dependency"),
            ));
            return None;
        }
        let extra = table.features.take().unwrap_or_default();
        let mut definition = inherited.table();
        definition
            .features
            .get_or_insert_with(Vec::new)
            .extend(extra);
        Some((definition, self.root))
    }

    fn dependency(
        &self,
        name: &str,
        dep: &Dep,
        kind: &str,
        workspace: &Workspace,
        out: &mut Findings,
    ) {
        let Some((table, base)) = self.resolve(name, dep, workspace, out) else {
            return;
        };
        if table.git.is_some() || table.registry.is_some() || table.registry_index.is_some() {
            out.push((
                "dep-git",
                format!("{name}: git or alternate-registry dependency"),
            ));
            return;
        }
        if let Some(path) = &table.path {
            if !normal(&base.join(path)).is_some_and(|p| self.allowed_paths.contains(&p)) {
                out.push((
                    "dep-path",
                    format!("{name}: path {path} resolves outside the clean crates"),
                ));
            }
            return;
        }
        let package = table.package.as_deref().unwrap_or(name);
        let Some(approval) = self.approved.get(package) else {
            out.push((
                "dep-registry",
                format!("{name}: {package} is not in allowed_deps.txt"),
            ));
            return;
        };
        if table.version.as_deref() != Some(approval.version.as_str())
            || !approval.kinds.contains(kind)
        {
            out.push((
                "dep-registry",
                format!("{name}: {kind} dependency requires approval at its exact version"),
            ));
        }
        for key in table.extra_keys() {
            out.push((
                "dep-registry",
                format!("{name}: the key `{key}` is not allowed on a registry dependency"),
            ));
        }
        for feature in table.features.iter().flatten() {
            if !approval.features.contains(feature) {
                out.push((
                    "dep-registry",
                    format!("{name}: feature `{feature}` is not in allowed_deps.txt"),
                ));
            }
        }
    }
}
