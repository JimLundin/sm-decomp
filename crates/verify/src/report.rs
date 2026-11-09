//! The rebuild's size and progress report, `docs/REPORT.md`, for `tools report` (Jim,
//! 2026-11-08: the numbers observable at any commit, sizes also in lines).
//!
//! Per rebuilt domain ([`DOMAINS`]) and in total: the model's score (`size`) over
//! `crates/clean_model/<domain>`, with `crates/clean_types` shared out; the spec's units and
//! budget by area (`spec_units` on `docs/behaviours/<domain>.md` and
//! `scenarios/vectors/<domain>.toml`, k and the areas from the Budget section of
//! `docs/behaviours/<domain>.decode.md`) and the fit, score ÷ budget; the old code's baseline as
//! that section states it; the bridge's and the harness's scores; code lines ([`lines`]); the
//! harness floors and the vector counts; the harnesses' oracle coverage, unverified frames
//! and per catalogue entry; and the engine code not yet rebuilt, `crates/game`
//! without the bridges.
//!
//! The report is generated, never edited: [`check`] fails when the committed file differs from
//! what the tree generates. Everything in it is sorted or in registry order, with paths relative
//! to the repository root and no timestamp, so one tree gives one report.

pub mod lines;

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use crate::size::{self, Size};
use crate::spec_units::{Count, Vectors, budget_section};

/// The command line `run` takes.
pub const USAGE: &str = "tools report [--write | --check]";
/// Where the report is committed, from the repository root.
pub const PATH: &str = "docs/REPORT.md";
/// The one command that regenerates the committed report, from the repository root.
pub const COMMAND: &str = "cargo run -p tools --bin tools -- report --write";
/// The old engine's density in weighted tokens per code line, which turns a score into lines:
/// about 5.5 to 7, measured on samus-motion's old code at `c7f927ab6` and the camera's at
/// `cfa5b8144`.
const TOKENS_PER_LINE: usize = 6;
/// The harness floors, per domain and row.
const FLOORS: &str = "crates/verify/src/harness/floors.txt";
/// The harnesses' oracle coverage over every row, per domain and catalogue entry.
const COVERAGE: &str = "crates/verify/src/harness/coverage.txt";
/// The crate of types the clean models share.
const SHARED_TYPES: &str = "crates/clean_types";
/// The engine crate, the bridges' home and the code not yet rebuilt.
const ENGINE: &str = "crates/game";

/// A rebuilt domain. Its spec is `docs/behaviours/<name>.md`, its budget the `## Budget`
/// section of `docs/behaviours/<name>.decode.md`, its vectors
/// `scenarios/vectors/<name>.toml`, its model `crates/clean_model/<name>`, its harness
/// `crates/verify/src/harness/<name, snake case>*` and its floors the rows of `floors.txt` that
/// name it.
struct Domain {
    /// The domain's name.
    name: &'static str,
    /// The bridge's files under `crates/game/src`: those whose names start with it.
    bridge: &'static str,
    /// The Budget section's areas, each with the model files (under the model's `src`) that
    /// hold it. A
    /// model file in no area is the domain's shared code: it counts in the domain's total only.
    areas: &'static [(&'static str, &'static [&'static str])],
}

/// The rebuilt domains, in the report's order. Adding one is one entry.
const DOMAINS: &[Domain] = &[
    Domain {
        name: "samus-motion",
        bridge: "samus_world",
        areas: &[
            ("motion", &["motion.rs", "samus.rs", "step.rs", "tick.rs"]),
            ("pose fit", &["fit.rs"]),
            (
                "pose chart",
                &["chart.rs", "chart/table.rs", "rows.rs", "situation.rs"],
            ),
            ("collision", &["collide.rs"]),
        ],
    },
    Domain {
        name: "camera",
        bridge: "camera_world",
        areas: &[
            ("follow", &["follow.rs"]),
            ("limits", &["axis.rs", "room.rs"]),
        ],
    },
    // The model keeps happened and places together in `world.rs`, and the percentage in
    // `inventory.rs`: those two areas have no file of their own.
    Domain {
        name: "progress",
        bridge: "progress_world",
        areas: &[
            ("happened", &[]),
            ("places", &["world.rs", "per_area.rs"]),
            ("equipment", &["inventory.rs", "supplies.rs"]),
            ("rooms", &["room_states.rs"]),
            ("percentage", &[]),
        ],
    },
    // The pictures and the places, which both the status bar and the mini-map read, are the
    // domain's shared code, with the grid, the inputs and the frame.
    Domain {
        name: "hud",
        bridge: "hud_world",
        areas: &[
            ("status", &["status_bar.rs"]),
            ("selection", &["selection.rs", "item.rs"]),
            ("minimap", &["mini_map.rs", "map.rs"]),
        ],
    },
];

/// A body of code's size and lines, the score's parts in eighths of a token as `size` keeps them.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
struct Measure {
    /// The weighted score, in eighths.
    score: usize,
    /// The score's depth part (Q1), in eighths.
    depth: usize,
    /// The score's clump part (Q2), in eighths.
    clumps: usize,
    /// Logic tokens.
    logic: usize,
    /// Declaration tokens.
    types: usize,
    /// Data tokens.
    data: usize,
    /// Code lines.
    lines: usize,
}

impl Measure {
    const fn of(size: Size, lines: usize) -> Self {
        Self {
            score: size.score.eighths(),
            depth: size.depth.eighths(),
            clumps: size.clumps.eighths(),
            logic: size.logic,
            types: size.types,
            data: size.data,
            lines,
        }
    }

    const fn plus(self, other: Self) -> Self {
        Self {
            score: self.score.saturating_add(other.score),
            depth: self.depth.saturating_add(other.depth),
            clumps: self.clumps.saturating_add(other.clumps),
            logic: self.logic.saturating_add(other.logic),
            types: self.types.saturating_add(other.types),
            data: self.data.saturating_add(other.data),
            lines: self.lines.saturating_add(other.lines),
        }
    }

    /// What its files hold most of.
    const fn holds(self) -> Holds {
        if self.types.saturating_add(self.data) > self.logic {
            Holds::TablesAndTypes
        } else {
            Holds::Logic
        }
    }

    /// The score's whole tokens.
    fn score(self) -> String {
        tokens(self.score)
    }

    /// The lines the score would take at the old engine's density.
    fn logic_lines(self) -> String {
        grouped(rounded(self.score, TOKENS_PER_LINE.saturating_mul(8)))
    }

    /// Score, logic, types, data, depth, clumps and lines, as table cells.
    fn cells(self) -> String {
        format!(
            "{} | {} | {} | {} | {} | {} | {}",
            self.score(),
            grouped(self.logic),
            grouped(self.types),
            grouped(self.data),
            tokens(self.depth),
            tokens(self.clumps),
            grouped(self.lines)
        )
    }
}

impl std::iter::Sum for Measure {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::default(), Self::plus)
    }
}

/// Whether a file holds mostly tables and types or mostly logic, by its tokens.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Holds {
    /// More declaration and data tokens than logic tokens.
    TablesAndTypes,
    /// At least as many logic tokens as the others.
    Logic,
}

impl Holds {
    const fn label(self) -> &'static str {
        match self {
            Self::TablesAndTypes => "tables and types",
            Self::Logic => "logic",
        }
    }
}

/// One measured file, by its path from the root.
struct File {
    path: String,
    measure: Measure,
}

/// A domain's measured parts.
struct Measured {
    domain: &'static Domain,
    /// The model's files, each with the area that holds it.
    model: Vec<(File, Option<&'static str>)>,
    /// Whether its model depends on the shared types.
    uses_shared: bool,
    count: Count,
    vectors: usize,
    pose_steps: usize,
    /// The commit the Budget section's baseline or budget paragraph names last: the old
    /// code's.
    baseline: Option<String>,
    bridge: Vec<File>,
    harness: Vec<File>,
}

impl Measured {
    /// The model: the files the domain owns.
    fn model_total(&self) -> Measure {
        self.model.iter().map(|(file, _)| file.measure).sum()
    }

    fn area(&self, name: &str) -> Measure {
        self.model
            .iter()
            .filter(|(_, area)| *area == Some(name))
            .map(|(file, _)| file.measure)
            .sum()
    }

    /// The model's files in no area.
    fn unassigned(&self) -> Measure {
        self.model
            .iter()
            .filter(|(_, area)| area.is_none())
            .map(|(file, _)| file.measure)
            .sum()
    }

    fn old(&self) -> Option<u64> {
        self.count.areas.iter().try_fold(0_u64, |sum, area| {
            area.old.map(|old| sum.saturating_add(old))
        })
    }
}

/// `n` with thousands separated by commas.
fn grouped(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (at, digit) in digits.chars().enumerate() {
        if at > 0 && digits.len().saturating_sub(at) % 3 == 0 {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

/// `n` ÷ `by`, rounded half up.
fn rounded(n: usize, by: usize) -> usize {
    n.saturating_add(by / 2).checked_div(by).unwrap_or_default()
}

/// Eighths of a token as whole tokens.
fn tokens(eighths: usize) -> String {
    grouped(rounded(eighths, 8))
}

/// A score in eighths over a budget in tokens, to two places, or "-" without a budget.
fn fit(eighths: usize, budget: Option<u64>) -> String {
    let budget = budget.and_then(|budget| usize::try_from(budget).ok());
    budget
        .and_then(|budget| {
            let hundredths = rounded(eighths.saturating_mul(100), budget.checked_mul(8)?);
            Some(format!("{}.{:02}", hundredths / 100, hundredths % 100))
        })
        .unwrap_or_else(|| String::from("-"))
}

/// An optional figure, or "-".
fn or_dash(figure: Option<u64>) -> String {
    figure
        .and_then(|figure| usize::try_from(figure).ok())
        .map_or_else(|| String::from("-"), grouped)
}

/// `path` from `root`, with `/` between its parts.
fn relative(root: &Path, path: &Path) -> String {
    let path = path.strip_prefix(root).unwrap_or(path);
    path.components()
        .map(|part| part.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

/// The `.rs` files under each of `paths`, sorted.
fn sources(paths: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for path in paths {
        size::sources(path, &mut files)?;
    }
    files.sort();
    files.dedup();
    Ok(files)
}

/// The entries of `dir` whose names start with `prefix`, sorted.
fn prefixed(dir: &Path, prefix: &str) -> Result<Vec<PathBuf>, String> {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| {
            path.file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with(prefix))
        })
        .collect();
    entries.sort();
    Ok(entries)
}

/// Each file's measure, the files measured together (a clump recurs across them).
fn measure(root: &Path, files: &[PathBuf]) -> Result<Vec<File>, String> {
    let texts = files
        .iter()
        .map(|file| read(file))
        .collect::<Result<Vec<_>, _>>()?;
    let srcs: Vec<&str> = texts.iter().map(String::as_str).collect();
    Ok(files
        .iter()
        .zip(&srcs)
        .zip(size::sizes(&srcs))
        .map(|((file, src), size)| File {
            path: relative(root, file),
            measure: Measure::of(size, lines::code_lines(src)),
        })
        .collect())
}

fn total(files: &[File]) -> Measure {
    files.iter().map(|file| file.measure).sum()
}

/// The commit the Budget section's `**Baseline` paragraph names last, in backticks, else its
/// `**Budget` paragraph's: the old code's.
fn baseline(section: &str) -> Option<String> {
    ["**Baseline", "**Budget"]
        .iter()
        .find_map(|label| last_commit(section, label))
}

/// The commit the paragraph starting at `label` names last, in backticks.
fn last_commit(section: &str, label: &str) -> Option<String> {
    let start = section.find(label)?;
    let paragraph = section.get(start..)?;
    let paragraph = paragraph
        .find("\n\n")
        .and_then(|end| paragraph.get(..end))
        .unwrap_or(paragraph);
    paragraph
        .split('`')
        .skip(1)
        .step_by(2)
        .filter(|span| {
            (7..=40).contains(&span.len()) && span.chars().all(|c| c.is_ascii_hexdigit())
        })
        .last()
        .map(str::to_owned)
}

/// How many `[[name]]` tables the vector file holds.
fn tables(toml: &str, name: &str) -> usize {
    let header = format!("[[{name}]]");
    toml.lines().filter(|line| line.trim() == header).count()
}

/// The floors, by (domain, row): `domain row frames` lines, `#` comments skipped.
fn floors(text: &str) -> Result<BTreeMap<(String, String), usize>, String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(
            |line| match line.split_whitespace().collect::<Vec<_>>().as_slice() {
                [domain, row, frames] => frames
                    .parse()
                    .map(|frames| (((*domain).to_owned(), (*row).to_owned()), frames))
                    .map_err(|e| format!("{FLOORS}: {line}: {e}")),
                _ => Err(format!("{FLOORS}: {line}: not `domain row frames`")),
            },
        )
        .collect()
}

/// Whether a model's manifest depends on the shared types.
fn uses_shared_types(manifest: &str) -> bool {
    manifest
        .lines()
        .any(|line| line.trim_start().starts_with("clean_types"))
}

fn measure_domain(root: &Path, domain: &'static Domain) -> Result<Measured, String> {
    let model_dir = root.join("crates/clean_model").join(domain.name);
    let model_src = model_dir.join("src");
    let model = measure(root, &sources(std::slice::from_ref(&model_src))?)?;
    let in_model = |file: &str| relative(root, &model_src.join(file));
    for (area, files) in domain.areas {
        for file in *files {
            if !model.iter().any(|measured| measured.path == in_model(file)) {
                return Err(format!(
                    "{}: area {area}: no model file {file}",
                    domain.name
                ));
            }
        }
    }
    let model = model
        .into_iter()
        .map(|file| {
            let area = domain
                .areas
                .iter()
                .find(|(_, files)| files.iter().any(|name| in_model(name) == file.path))
                .map(|(area, _)| *area);
            (file, area)
        })
        .collect();
    let behaviours = root.join("docs/behaviours");
    let spec = read(&behaviours.join(format!("{}.md", domain.name)))?;
    let notes = read(&behaviours.join(format!("{}.decode.md", domain.name)))?;
    let budget = budget_section(&notes);
    let toml = read(
        &root
            .join("scenarios/vectors")
            .join(format!("{}.toml", domain.name)),
    )?;
    let count = Count::of(&spec, &Vectors::parse(&toml), Some(budget), None, &[]);
    for (area, _) in domain.areas {
        if !count.areas.iter().any(|stated| stated.name == *area) {
            return Err(format!(
                "{}: the decode notes' Budget section states no area {area}",
                domain.name
            ));
        }
    }
    let harness = prefixed(
        &root.join("crates/verify/src/harness"),
        &domain.name.replace('-', "_"),
    )?;
    let uses_shared = uses_shared_types(&read(&model_dir.join("Cargo.toml"))?);
    Ok(Measured {
        domain,
        model,
        uses_shared,
        count,
        vectors: tables(&toml, "vector"),
        pose_steps: tables(&toml, "pose_step"),
        baseline: baseline(budget),
        bridge: measure(
            root,
            &sources(&prefixed(&root.join(ENGINE).join("src"), domain.bridge)?)?,
        )?,
        harness: measure(root, &sources(&harness)?)?,
    })
}

/// The report for the tree at `root`.
///
/// # Errors
///
/// The path and the error when a file can't be read, or a registry entry that names an area the
/// decode notes' Budget section doesn't state or a file the model doesn't hold.
pub fn generate(root: &Path) -> Result<String, String> {
    let shared_files = measure(root, &sources(&[root.join(SHARED_TYPES)])?)?;
    let shared = total(&shared_files);
    let domains = DOMAINS
        .iter()
        .map(|domain| measure_domain(root, domain))
        .collect::<Result<Vec<_>, _>>()?;
    let bridges: Vec<&str> = domains
        .iter()
        .flat_map(|domain| &domain.bridge)
        .map(|file| file.path.as_str())
        .collect();
    let engine: Vec<PathBuf> = sources(&[root.join(ENGINE)])?
        .into_iter()
        .filter(|file| !bridges.contains(&relative(root, file).as_str()))
        .collect();
    let remaining = total(&measure(root, &engine)?);
    let floors = floors(&read(&root.join(FLOORS))?)?;
    let coverage = coverage(&read(&root.join(COVERAGE))?)?;

    let mut out = String::new();
    header(&mut out);
    summary(&mut out, &domains, shared, &floors);
    models(&mut out, &domains, shared);
    lines_section(&mut out, &domains, &shared_files);
    areas(&mut out, &domains);
    bridge_and_harness(&mut out, &domains);
    floors_section(&mut out, &domains, &floors);
    coverage_section(&mut out, &domains, &coverage);
    engine_section(&mut out, &domains, &shared_files, remaining, engine.len());
    Ok(out)
}

fn header(out: &mut String) {
    let _ = write!(
        out,
        "# Rebuild report

Generated by `{COMMAND}` (`verify::report`) from this commit's tree; never edit it by hand.
`tools/check_report.sh` fails when it differs from what the tree generates.

How it is counted:

- **Score** is `tools size` (`verify::size`): weighted logic tokens, with the depth (Q1) and
  clump (Q2) parts it includes; **logic**, **types** and **data** are token counts, the last two
  not held against the budget. Scores are rounded to whole tokens.
- **Model** is the files a domain owns, `crates/clean_model/<domain>`; its score, fit and lines
  count those alone. `{SHARED_TYPES}` is a row of its own, **shared types**, with the domains
  whose model depends on it, and is counted once in the total.
- **Budget** and **units** are `tools spec-units` (`verify::spec_units`) on the domain's spec and
  vectors, k and the areas from the Budget section of its decode notes, which the implementer
  never sees (MODEL §1 decision 10). **Fit** is score ÷ budget; the total's score includes the
  shared types. An area's model score is the model files the registry
  (`verify::report::DOMAINS`) assigns it; files in no area are the domain's shared code and count
  in its total only.
- **Old code** is the sum of the Budget section's area `old` figures, at the commit its
  baseline or budget paragraph names last.
- **Lines** are code lines: non-blank, non-comment, tests excluded (`tests` directories,
  `tests.rs` files and `#[cfg(test)]` items). A file holds **tables and types** when its type
  and data tokens outnumber its logic tokens, else **logic**. **Logic-equivalent lines** are
  score ÷ {TOKENS_PER_LINE}: the old engine's density, about 5.5 to 7 weighted tokens per line,
  measured on samus-motion's old code at `c7f927ab6` and the camera's at `cfa5b8144`.
"
    );
}

/// A table's header and its rule, its first `labels` columns aligned left, the rest right.
fn table(out: &mut String, labels: usize, columns: &[&str]) {
    let _ = writeln!(out, "| {} |", columns.join(" | "));
    let rule: Vec<&str> = columns
        .iter()
        .enumerate()
        .map(|(at, _)| if at < labels { "---" } else { "---:" })
        .collect();
    let _ = writeln!(out, "| {} |", rule.join(" | "));
}

fn harness_frames(floors: &BTreeMap<(String, String), usize>, domain: &str) -> (usize, usize) {
    floors
        .iter()
        .filter(|((name, _), _)| name == domain)
        .fold((0, 0), |(rows, frames), (_, &floor)| {
            (rows.saturating_add(1), frames.saturating_add(floor))
        })
}

/// The shared types' row label: the crate and the domains whose model uses it.
fn shared_label(domains: &[Measured]) -> String {
    let used_by: Vec<&str> = domains
        .iter()
        .filter(|domain| domain.uses_shared)
        .map(|domain| domain.domain.name)
        .collect();
    format!(
        "shared types (`{SHARED_TYPES}`, used by {})",
        used_by.join(", ")
    )
}

fn summary(
    out: &mut String,
    domains: &[Measured],
    shared: Measure,
    floors: &BTreeMap<(String, String), usize>,
) {
    out.push_str("\n## Summary\n\n");
    table(
        out,
        1,
        &[
            "Domain",
            "Model score",
            "Budget",
            "Fit",
            "Old code",
            "Model lines",
            "Logic-equivalent lines",
            "Harness frames",
            "Vectors",
        ],
    );
    let mut sum = (shared, Some(0_u64), Some(0_u64), 0usize, 0usize);
    for domain in domains {
        let model = domain.model_total();
        let (_, frames) = harness_frames(floors, domain.domain.name);
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            domain.domain.name,
            model.score(),
            or_dash(domain.count.budget()),
            fit(model.score, domain.count.budget()),
            or_dash(domain.old()),
            grouped(model.lines),
            model.logic_lines(),
            grouped(frames),
            grouped(domain.vectors),
        );
        sum = (
            sum.0.plus(model),
            sum.1
                .zip(domain.count.budget())
                .map(|(a, b)| a.saturating_add(b)),
            sum.2.zip(domain.old()).map(|(a, b)| a.saturating_add(b)),
            sum.3.saturating_add(frames),
            sum.4.saturating_add(domain.vectors),
        );
    }
    let _ = writeln!(
        out,
        "| {} | {} | - | - | - | {} | {} | - | - |",
        shared_label(domains),
        shared.score(),
        grouped(shared.lines),
        shared.logic_lines(),
    );
    let (model, budget, old, frames, vectors) = sum;
    let _ = writeln!(
        out,
        "| total | {} | {} | {} | {} | {} | {} | {} | {} |",
        model.score(),
        or_dash(budget),
        fit(model.score, budget),
        or_dash(old),
        grouped(model.lines),
        model.logic_lines(),
        grouped(frames),
        grouped(vectors),
    );
}

const MEASURE_COLUMNS: [&str; 7] = [
    "Score", "Logic", "Types", "Data", "Depth", "Clumps", "Lines",
];

fn measure_table(out: &mut String, labels: usize, first: &[&str]) {
    let columns: Vec<&str> = first.iter().chain(&MEASURE_COLUMNS).copied().collect();
    table(out, labels, &columns);
}

fn models(out: &mut String, domains: &[Measured], shared: Measure) {
    out.push_str("\n## Models\n");
    for domain in domains {
        let _ = write!(out, "\n### {}\n\n", domain.domain.name);
        measure_table(out, 3, &["File", "Area", "Holds"]);
        for (file, area) in &domain.model {
            let _ = writeln!(
                out,
                "| `{}` | {} | {} | {} |",
                file.path,
                area.unwrap_or("-"),
                file.measure.holds().label(),
                file.measure.cells()
            );
        }
        let _ = writeln!(out, "| total | | | {} |", domain.model_total().cells());
    }
    out.push_str("\n### Shared types\n\n");
    measure_table(out, 1, &["Crate"]);
    let _ = writeln!(out, "| {} | {} |", shared_label(domains), shared.cells());
}

/// Lines in files that hold mostly `holds`.
fn lines_holding<'a>(files: impl Iterator<Item = &'a File>, holds: Holds) -> usize {
    files
        .filter(|file| file.measure.holds() == holds)
        .map(|file| file.measure.lines)
        .sum()
}

fn lines_section(out: &mut String, domains: &[Measured], shared: &[File]) {
    out.push_str("\n## Lines\n\n");
    table(
        out,
        1,
        &[
            "Domain",
            "Model lines",
            "Tables-and-types files",
            "Logic files",
            "Logic-equivalent lines (score ÷ 6)",
        ],
    );
    let mut rows: Vec<(String, Measure, usize, usize)> = domains
        .iter()
        .map(|domain| {
            let files = || domain.model.iter().map(|(file, _)| file);
            (
                domain.domain.name.to_owned(),
                domain.model_total(),
                lines_holding(files(), Holds::TablesAndTypes),
                lines_holding(files(), Holds::Logic),
            )
        })
        .collect();
    rows.push((
        shared_label(domains),
        total(shared),
        lines_holding(shared.iter(), Holds::TablesAndTypes),
        lines_holding(shared.iter(), Holds::Logic),
    ));
    let sum = rows.iter().fold(
        (Measure::default(), 0usize, 0usize),
        |(model, tables, logic), (_, measure, t, l)| {
            (
                model.plus(*measure),
                tables.saturating_add(*t),
                logic.saturating_add(*l),
            )
        },
    );
    rows.push((String::from("total"), sum.0, sum.1, sum.2));
    for (label, measure, tables, logic) in rows {
        let _ = writeln!(
            out,
            "| {label} | {} | {} | {} | {} |",
            grouped(measure.lines),
            grouped(tables),
            grouped(logic),
            measure.logic_lines()
        );
    }
}

fn areas(out: &mut String, domains: &[Measured]) {
    out.push_str("\n## Spec, budget and fit by area\n");
    for domain in domains {
        let name = domain.domain.name;
        let _ = write!(
            out,
            "\n### {name}\n\nSpec [`docs/behaviours/{name}.md`](behaviours/{name}.md), k = {}. \
             Old code {}, at `{}` as its decode notes' Budget section states.\n\n",
            or_dash(domain.count.k),
            or_dash(domain.old()),
            domain.baseline.as_deref().unwrap_or("-")
        );
        table(
            out,
            1,
            &[
                "Area",
                "Entries",
                "Units",
                "Budget",
                "Old code",
                "Model score",
                "Fit",
            ],
        );
        for area in &domain.count.areas {
            let score = domain.area(&area.name);
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} | {} | {} | {} |",
                area.name,
                area.entries,
                area.tally.units(),
                or_dash(area.budget()),
                or_dash(area.old),
                score.score(),
                fit(score.score, area.budget())
            );
        }
        let _ = writeln!(
            out,
            "| shared (in no area) | | | | | {} | |",
            domain.unassigned().score()
        );
        let model = domain.model_total();
        let entries: usize = domain.count.areas.iter().map(|area| area.entries).sum();
        let _ = writeln!(
            out,
            "| total | {entries} | {} | {} | {} | {} | {} |",
            domain.count.total().units(),
            or_dash(domain.count.budget()),
            or_dash(domain.old()),
            model.score(),
            fit(model.score, domain.count.budget())
        );
    }
}

fn bridge_and_harness(out: &mut String, domains: &[Measured]) {
    out.push_str(
        "\n## Bridges and harnesses\n\nThe bridge (`crates/game/src/<bridge>*`) answers the \
         model's world from the engine; the harness (`crates/verify/src/harness/<domain>*`) \
         checks the model against the oracle. Neither is the model's.\n\n",
    );
    measure_table(out, 2, &["Domain", "Part", "Files"]);
    let mut sum = Measure::default();
    for domain in domains {
        for (part, files) in [("bridge", &domain.bridge), ("harness", &domain.harness)] {
            let measure = total(files);
            let _ = writeln!(
                out,
                "| {} | {part} | {} | {} |",
                domain.domain.name,
                files.len(),
                measure.cells()
            );
            sum = sum.plus(measure);
        }
    }
    let _ = writeln!(out, "| total | | | {} |", sum.cells());
}

fn floors_section(
    out: &mut String,
    domains: &[Measured],
    floors: &BTreeMap<(String, String), usize>,
) {
    out.push_str(
        "\n## Harness floors and vectors\n\nFrames matched and verified on every unmasked field, \
         per row (`crates/verify/src/harness/floors.txt`).\n\n",
    );
    let names: Vec<&str> = domains.iter().map(|domain| domain.domain.name).collect();
    let columns: Vec<&str> = std::iter::once("Row")
        .chain(names.iter().copied())
        .collect();
    table(out, 1, &columns);
    let mut rows: Vec<&str> = floors
        .keys()
        .filter(|(domain, _)| names.contains(&domain.as_str()))
        .map(|(_, row)| row.as_str())
        .collect();
    rows.sort_unstable();
    rows.dedup();
    for row in &rows {
        let cells: Vec<String> = names
            .iter()
            .map(|&name| {
                floors
                    .get(&(name.to_owned(), (*row).to_owned()))
                    .map_or_else(|| String::from("-"), |&frames| grouped(frames))
            })
            .collect();
        let _ = writeln!(out, "| {row} | {} |", cells.join(" | "));
    }
    let totals: Vec<String> = names
        .iter()
        .map(|name| {
            let (count, frames) = harness_frames(floors, name);
            format!("{} in {count} rows", grouped(frames))
        })
        .collect();
    let _ = writeln!(out, "| total | {} |", totals.join(" | "));
    out.push('\n');
    table(out, 1, &["Domain", "Vectors", "Pose steps"]);
    for domain in domains {
        let _ = writeln!(
            out,
            "| {} | {} | {} |",
            domain.domain.name,
            grouped(domain.vectors),
            grouped(domain.pose_steps)
        );
    }
}

/// A domain's oracle coverage over every row, as `coverage.txt` commits it.
#[derive(Default, Debug, PartialEq, Eq)]
struct Coverage {
    /// Frames, matched and verified, unverified and the null model's matched.
    frames: Option<[usize; 4]>,
    /// Each catalogue entry: its name, the frames matched and verified, the frames exercised.
    entries: Vec<(String, usize, usize)>,
}

/// The coverage by domain: `domain frames <4 counts>` and `domain entry <name> <2 counts>` lines,
/// `#` comments skipped.
fn coverage(text: &str) -> Result<BTreeMap<String, Coverage>, String> {
    let mut domains: BTreeMap<String, Coverage> = BTreeMap::new();
    let number = |line: &str, word: &str| {
        word.parse::<usize>()
            .map_err(|e| format!("{COVERAGE}: {line}: {e}"))
    };
    for line in text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
        match line.split_whitespace().collect::<Vec<_>>().as_slice() {
            [domain, "frames", counts @ ..] if counts.len() == 4 => {
                let mut frames = [0; 4];
                for (slot, word) in frames.iter_mut().zip(counts) {
                    *slot = number(line, word)?;
                }
                domains.entry((*domain).to_owned()).or_default().frames = Some(frames);
            }
            [domain, "entry", name, matched, exercised] => {
                let entry = (
                    (*name).to_owned(),
                    number(line, matched)?,
                    number(line, exercised)?,
                );
                domains
                    .entry((*domain).to_owned())
                    .or_default()
                    .entries
                    .push(entry);
            }
            _ => return Err(format!("{COVERAGE}: {line}: not a coverage line")),
        }
    }
    Ok(domains)
}

fn coverage_section(out: &mut String, domains: &[Measured], coverage: &BTreeMap<String, Coverage>) {
    out.push_str(
        "\n## Oracle coverage\n\nOver every row the floors name \
         (`crates/verify/src/harness/coverage.txt`, which the floor tests hold equal to a run over \
         every row). A frame is matched and verified when every unmasked field matched the \
         oracle, and unverified when it matched only through a stand-in's answer copied from the \
         next record; the null model is a step that does nothing.\n\n",
    );
    table(
        out,
        1,
        &[
            "Domain",
            "Frames",
            "Matched and verified",
            "Unverified",
            "Null model",
        ],
    );
    for domain in domains {
        let name = domain.domain.name;
        let cells = coverage
            .get(name)
            .and_then(|coverage| coverage.frames)
            .map_or_else(
                || String::from("- | - | - | -"),
                |frames| frames.map(grouped).join(" | "),
            );
        let _ = writeln!(out, "| {name} | {cells} |");
    }
    out.push_str(
        "\nPer catalogue entry, the frames that exercised it and those of them matched and \
         verified. An entry no frame exercised is checked by its vectors only.\n\n",
    );
    table(
        out,
        2,
        &["Domain", "Entry", "Matched and verified", "Exercised"],
    );
    for domain in domains {
        let name = domain.domain.name;
        for (entry, matched, exercised) in coverage
            .get(name)
            .map(|coverage| coverage.entries.as_slice())
            .unwrap_or_default()
        {
            let matched = if *exercised == 0 {
                String::from("vectors only")
            } else {
                grouped(*matched)
            };
            let _ = writeln!(
                out,
                "| {name} | {entry} | {matched} | {} |",
                grouped(*exercised)
            );
        }
    }
}

fn engine_section(
    out: &mut String,
    domains: &[Measured],
    shared: &[File],
    remaining: Measure,
    files: usize,
) {
    out.push_str(
        "\n## Engine not yet rebuilt\n\nThe engine's code, every non-test source of `crates/game` without the bridges: \
         what the rebuilt domains have yet to replace, beside what they hold.\n\n",
    );
    measure_table(out, 1, &["Part", "Files", "Logic-equivalent lines"]);
    let models = domains
        .iter()
        .map(Measured::model_total)
        .sum::<Measure>()
        .plus(total(shared));
    let model_files = domains
        .iter()
        .map(|domain| domain.model.len())
        .sum::<usize>()
        .saturating_add(shared.len());
    let bridges: Vec<&File> = domains.iter().flat_map(|domain| &domain.bridge).collect();
    let bridge: Measure = bridges.iter().map(|file| file.measure).sum();
    for (part, count, measure) in [
        ("engine, not yet rebuilt", files, remaining),
        ("bridges", bridges.len(), bridge),
        ("rebuilt models, shared types included", model_files, models),
    ] {
        let _ = writeln!(
            out,
            "| {part} | {count} | {} | {} |",
            measure.logic_lines(),
            measure.cells()
        );
    }
}

/// The committed report's staleness: an error naming the command that regenerates it when the
/// committed file differs from what the tree at `root` generates.
///
/// # Errors
///
/// The regeneration command when the report is stale or missing, or why it can't be generated.
pub fn check(root: &Path) -> Result<(), String> {
    let committed = fs::read_to_string(root.join(PATH)).unwrap_or_default();
    current(&committed, &generate(root)?)
}

/// Whether the committed report is the fresh one; a checkout's CRLF line endings don't count.
fn current(committed: &str, fresh: &str) -> Result<(), String> {
    if committed.replace("\r\n", "\n") == fresh {
        Ok(())
    } else {
        Err(format!(
            "{PATH} is stale: regenerate it with `{COMMAND}` from the repository root"
        ))
    }
}

/// Prints the report for the tree in the current directory; with `--write`, writes it to
/// [`PATH`]; with `--check`, fails when the committed one is stale.
///
/// # Errors
///
/// The usage on a wrong command line, why the report can't be generated or written, or
/// [`check`]'s error.
pub fn run(args: &[String]) -> Result<String, String> {
    let root = Path::new(".");
    match args {
        [] => generate(root),
        [flag] if flag == "--write" => {
            let report = generate(root)?;
            fs::write(root.join(PATH), report).map_err(|e| format!("{PATH}: {e}"))?;
            Ok(format!("wrote {PATH}\n"))
        }
        [flag] if flag == "--check" => check(root).map(|()| format!("{PATH} is current\n")),
        _ => Err(USAGE.to_owned()),
    }
}
