//! Declared rule-to-code navigation, independent of replay execution and selection for landing.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use verify::replay::{
    self,
    baseline::{Baseline, BaselineFault, Floor},
    index::{Case, CaseId, Index, IndexFault, Start},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Role {
    Model,
    Adapter,
    Navigation,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Target {
    role: Role,
    path: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Rule {
    id: String,
    targets: Vec<Target>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Mapping {
    format: u16,
    rule: Vec<Rule>,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Fault {
    Arguments,
    Missing,
    Syntax,
    Format,
    Index(IndexFault),
    Baseline(BaselineFault),
    DuplicateRule(String),
    UnknownRule(String),
    MissingRule(String),
    InvalidTarget(String),
    DuplicateTarget(String),
    MissingRole(String),
    UnknownPath(String),
    Encoding,
}

fn safe_path(root: &Path, path: &str) -> Result<(), Fault> {
    if path.is_empty()
        || path.contains(['\\', ':'])
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(Fault::InvalidTarget(path.to_owned()));
    }
    let target = root.join(path);
    let resolved = target
        .canonicalize()
        .map_err(|_| Fault::InvalidTarget(path.to_owned()))?;
    if !target.is_file() || !resolved.starts_with(root) {
        return Err(Fault::InvalidTarget(path.to_owned()));
    }
    Ok(())
}

impl Mapping {
    fn parse(text: &str) -> Result<Self, Fault> {
        toml::from_str(text).map_err(|_| Fault::Syntax)
    }

    fn validate(&self, index: &Index, root: &Path) -> Result<(), Fault> {
        index.validate().map_err(Fault::Index)?;
        if self.format != 1 {
            return Err(Fault::Format);
        }
        let inventory: BTreeSet<_> = index
            .cases
            .iter()
            .flat_map(|case| case.rules.iter())
            .collect();
        let mut mapped = BTreeSet::new();
        for rule in &self.rule {
            if !mapped.insert(&rule.id) {
                return Err(Fault::DuplicateRule(rule.id.clone()));
            }
            if !inventory.contains(&rule.id) {
                return Err(Fault::UnknownRule(rule.id.clone()));
            }
            let mut paths = BTreeSet::new();
            for target in &rule.targets {
                safe_path(root, &target.path)?;
                if !paths.insert(&target.path) {
                    return Err(Fault::DuplicateTarget(target.path.clone()));
                }
            }
            if ![Role::Model, Role::Adapter]
                .iter()
                .all(|role| rule.targets.iter().any(|target| target.role == *role))
            {
                return Err(Fault::MissingRole(rule.id.clone()));
            }
        }
        for id in inventory {
            if !mapped.contains(id) {
                return Err(Fault::MissingRule(id.clone()));
            }
        }
        Ok(())
    }

    fn select<'a>(&self, index: &'a Index, selection: &Selection) -> Result<Vec<&'a Case>, Fault> {
        if let Some(rule) = &selection.rule
            && !self.rule.iter().any(|mapped| mapped.id == *rule)
        {
            return Err(Fault::UnknownRule(rule.clone()));
        }
        if let Some(path) = &selection.path
            && !self
                .rule
                .iter()
                .any(|rule| rule.targets.iter().any(|target| target.path == *path))
        {
            return Err(Fault::UnknownPath(path.clone()));
        }
        let cases = index
            .select(&selection.ids, selection.rule.as_deref(), None)
            .map_err(Fault::Index)?;
        let cases: Vec<_> = cases
            .into_iter()
            .filter(|case| {
                selection.path.as_ref().is_none_or(|path| {
                    self.rule.iter().any(|rule| {
                        case.rules.contains(&rule.id)
                            && rule.targets.iter().any(|target| target.path == *path)
                    })
                })
            })
            .collect();
        if cases.is_empty() {
            return Err(Fault::Index(IndexFault::Empty));
        }
        Ok(cases)
    }
}

#[derive(Default)]
struct Selection {
    ids: Vec<CaseId>,
    rule: Option<String>,
    path: Option<String>,
}

struct Options {
    mapping: PathBuf,
    baseline: PathBuf,
    root: PathBuf,
    selection: Selection,
}

fn options(index: &Path, args: &[String]) -> Result<Options, Fault> {
    let parent = index.parent().ok_or(Fault::Arguments)?;
    let mut options = Options {
        mapping: parent.join("code-map.toml"),
        baseline: parent.join("baseline.toml"),
        root: PathBuf::from("."),
        selection: Selection::default(),
    };
    let mut flags = BTreeSet::new();
    let mut remaining = args;
    while let Some((flag, tail)) = remaining.split_first() {
        let Some((value, tail)) = tail.split_first() else {
            return Err(Fault::Arguments);
        };
        if flag != "--case" && !flags.insert(flag) {
            return Err(Fault::Arguments);
        }
        match flag.as_str() {
            "--mapping" => options.mapping = PathBuf::from(value),
            "--baseline" => options.baseline = PathBuf::from(value),
            "--repo" => options.root = PathBuf::from(value),
            "--case" => options.selection.ids.push(CaseId(value.clone())),
            "--rule" => options.selection.rule = Some(value.clone()),
            "--path" => options.selection.path = Some(value.clone()),
            _ => return Err(Fault::Arguments),
        }
        remaining = tail;
    }
    options.root = options.root.canonicalize().map_err(|_| Fault::Missing)?;
    Ok(options)
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Coordinates {
    RowLocalCache,
    NativeFixture,
    WithinVideoNativeOpportunity,
    NativeSenderOfferCut,
    ConstructedSenderOfferCut,
    PowerOn,
}

const fn coordinates(case: &Case) -> Coordinates {
    match case.start {
        Start::CacheCut { .. } => Coordinates::RowLocalCache,
        Start::NativeOpportunity { .. } => Coordinates::WithinVideoNativeOpportunity,
        Start::NativeSeed { .. } | Start::NativeRecord { .. } | Start::NativeCheckpoint { .. } => {
            Coordinates::NativeFixture
        }
        Start::NativeSenderOffer { .. } => Coordinates::NativeSenderOfferCut,
        Start::ConstructedSenderOffer { .. } => Coordinates::ConstructedSenderOfferCut,
        Start::PowerOn => Coordinates::PowerOn,
    }
}

#[derive(Serialize)]
struct CaseView<'a> {
    coordinates: Coordinates,
    first_compared_cut: Option<u64>,
    declaration: &'a Case,
    reviewed_requirement: &'a Floor,
    declared_relevance: Vec<&'a Rule>,
}

fn views<'a>(
    mapping: &'a Mapping,
    baseline: &'a Baseline,
    cases: &[&'a Case],
) -> Result<Vec<CaseView<'a>>, Fault> {
    cases
        .iter()
        .map(|case| {
            let floor = baseline
                .cases
                .iter()
                .find(|floor| floor.id == case.id)
                .ok_or_else(|| Fault::Baseline(BaselineFault::MissingCase(case.id.clone())))?;
            Ok(CaseView {
                coordinates: coordinates(case),
                first_compared_cut: case.first_cut(),
                declaration: case,
                reviewed_requirement: floor,
                declared_relevance: mapping
                    .rule
                    .iter()
                    .filter(|rule| case.rules.contains(&rule.id))
                    .collect(),
            })
        })
        .collect()
}

#[derive(Serialize)]
struct Report<'a> {
    format: u16,
    purpose: &'static str,
    index_lock: &'a Path,
    case: Vec<CaseView<'a>>,
}
pub(super) fn run(args: &[String]) -> Result<String, Fault> {
    let [index_path, rest @ ..] = args else {
        return Err(Fault::Arguments);
    };
    let index_path = Path::new(index_path);
    let options = options(index_path, rest)?;
    let snapshot = replay::read_index(index_path).map_err(Fault::Index)?;
    let baseline = Baseline::read(&options.baseline).map_err(Fault::Baseline)?;
    baseline
        .validate_index(&snapshot)
        .map_err(Fault::Baseline)?;
    let index = snapshot.index();
    let mapping =
        Mapping::parse(&std::fs::read_to_string(options.mapping).map_err(|_| Fault::Missing)?)?;
    mapping.validate(index, &options.root)?;
    let selected = mapping.select(index, &options.selection)?;
    toml::to_string(&Report { format: 1, purpose: "Declared navigation and reviewed requirements; no replay executed; no exhaustive impact or dynamic coverage claim; complete index remains required for landing", index_lock: &index.lock, case: views(&mapping, &baseline, &selected)? }).map_err(|_| Fault::Encoding)
}

#[cfg(test)]
mod tests;
