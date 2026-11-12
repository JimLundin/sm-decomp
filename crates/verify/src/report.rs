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
//! and per catalogue entry; the engine code not yet rebuilt, `crates/game`
//! without the bridges; the timeline of each movie's cover by the checks ([`timeline`], also
//! drawn as `docs/timeline.svg`); and how each clean model is built ([`structure`]).
//!
//! The same measures, with the whole-game plan and the verification state, give the README's
//! status ([`status`]), written between its markers.
//!
//! The report, the picture and the status are generated, never edited: [`check`] fails when any
//! committed one differs from what the tree generates. Everything in them is sorted or in registry order,
//! with paths relative to the repository root and no timestamp, so one tree gives one report.

pub mod lines;
mod status;
mod structure;
mod timeline;

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
/// The README, from the repository root, whose status the report's command writes too.
pub const README: &str = "README.md";
/// The line before the README's status.
const STATUS_BEGIN: &str = "<!-- status:begin -->";
/// The line after the README's status.
const STATUS_END: &str = "<!-- status:end -->";
/// The old engine's density in weighted tokens per code line, which turns a score into lines:
/// about 5.5 to 7, measured on samus-motion's old code at `c7f927ab6` and the camera's at
/// `cfa5b8144`.
const TOKENS_PER_LINE: usize = 6;
/// The share of a model's score, in percent, above which its shared code (in no area) is a
/// finding (Jim, 2026-11-11): no spec unit pays for it, so it bends the fit.
const SHARED_FLAG: usize = 25;
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
    /// hold it. A model file in no area is the domain's shared code: it counts in the domain's
    /// total only.
    areas: &'static [(&'static str, &'static [&'static str])],
    /// The functions that count in another area than their file's.
    apart: &'static [Apart],
}

/// A function of a model file that counts in an area of its own; the file's other code counts
/// where the file's area says. It is measured with the rest of the model, as its file is.
struct Apart {
    /// The file, under the model's `src`.
    file: &'static str,
    /// The function's name, which the file holds once.
    function: &'static str,
    /// The area that holds it.
    area: &'static str,
}

/// The rebuilt domains, in the report's order. Adding one is one entry.
const DOMAINS: &[Domain] = &[
    Domain {
        name: "samus-motion",
        bridge: "samus_world",
        areas: &[
            (
                "motion",
                &[
                    "jump_held.rs",
                    "moves.rs",
                    "run.rs",
                    "samus.rs",
                    "step.rs",
                    "tick.rs",
                    "vertical.rs",
                ],
            ),
            // `change.rs` is the pose change. Its shrink is pose fit's (`ShrinkingLowersHer`); its
            // other two functions count apart, with their entries.
            ("pose fit", &["change.rs", "fit.rs"]),
            (
                "pose chart",
                &["chart.rs", "chart/table.rs", "rows.rs", "situation.rs"],
            ),
            ("collision", &["collide.rs"]),
        ],
        apart: &[
            // `PoseFollows`: whether she takes the pose chosen, what her moves met, then the
            // row's effects in order.
            Apart {
                file: "change.rs",
                function: "change_pose",
                area: "pose chart",
            },
            // One effect on her speeds, dash and position: `Jumping`, `TurningRound`,
            // `Moonfall`, `WallJump`.
            Apart {
                file: "change.rs",
                function: "apply",
                area: "motion",
            },
        ],
    },
    // `rules.rs` is the lead's: its functions give a lead setting's lead for the way she travels,
    // and its other values (the push, the margin) are constants.
    Domain {
        name: "camera",
        bridge: "camera_world",
        areas: &[
            ("follow", &["follow.rs", "rules.rs"]),
            ("limits", &["axis.rs", "room.rs"]),
        ],
        apart: &[],
    },
    // The rules (equipment's beams and percentage's divisors) and the catalogue are the domain's
    // shared code; the keyed tables and sets are `clean_types'`. `progress.rs` holds what has
    // happened with the whole progress. The report handler is one function whose arms are
    // mostly the places' (emptied places, doors, maps, stations, items placed), so it counts
    // there.
    Domain {
        name: "progress",
        bridge: "progress_world",
        areas: &[
            ("happened", &["progress.rs"]),
            ("places", &["places.rs", "report.rs"]),
            ("equipment", &["inventory.rs"]),
            ("rooms", &["room_states.rs"]),
            ("percentage", &["percentage.rs"]),
        ],
        apart: &[],
    },
    // The status bar's pictures and layout are its own (§1.2, §1.4), the mini-map's two places
    // in the layout being constants. The frame's order, the grid, the inputs, the tiles, the
    // State, the flash's and blink's timing and the catalogue are the domain's shared code.
    Domain {
        name: "hud",
        bridge: "hud_world",
        areas: &[
            ("status", &["status_bar.rs", "pictures.rs", "layout.rs"]),
            ("selection", &["selection.rs", "item.rs"]),
            ("minimap", &["mini_map.rs", "map.rs"]),
        ],
        apart: &[
            // `CyclingItems`: whether Select can land on an item.
            Apart {
                file: "inputs.rs",
                function: "has",
                area: "selection",
            },
            // The reference rules: the bosses' screens (`BossBlanksTheMap`) and her blink are
            // the mini-map's; the flash's one value rides with them.
            Apart {
                file: "rules.rs",
                function: "reference",
                area: "minimap",
            },
        ],
    },
    // The cells' grid and kinds, the inputs, the report and the catalogue are the domain's
    // shared code. The places and their run, with the reactions' scripts they run, are the
    // places' area, the scroll cells' touch and run among them; the requests' path and the rules
    // by family, with the room's start beside them in the State, the breakables'. The scroll's
    // area has no file of its own.
    Domain {
        name: "blocks",
        bridge: "blocks_world",
        areas: &[
            ("places", &["places.rs", "reactions.rs"]),
            ("scroll", &[]),
            ("breakables", &["rules.rs", "state.rs"]),
        ],
        apart: &[
            // `PatternCells`: a show's pattern cell writes a cell.
            Apart {
                file: "cells.rs",
                function: "show",
                area: "places",
            },
            // `Regrowing`: a cell grows back.
            Apart {
                file: "cells.rs",
                function: "regrow",
                area: "places",
            },
            // `PatternCells`: where a pattern's cell lands from its origin.
            Apart {
                file: "cells.rs",
                function: "offset",
                area: "places",
            },
            // `AnswersToHerMotion`: what a touch of a cell's kind answers.
            Apart {
                file: "cells.rs",
                function: "answer",
                area: "breakables",
            },
            // The plain cell a reaction that stops reacting leaves (the rules by family).
            Apart {
                file: "cells.rs",
                function: "plain",
                area: "breakables",
            },
            // `BlocksMarkedNothing`: the family a cell marked `Nothing` or `Other` is met as.
            Apart {
                file: "cells.rs",
                function: "met_as",
                area: "breakables",
            },
        ],
    },
    // The inputs, the frame and its update, the State's shapes, the one countdown rule, the
    // rules and the catalogue are the domain's shared code. The clip's format, what a frame's
    // start tells, the playhead, the clip's play and its start in a new pose are the clip's; her
    // moves' writes and the run's pace and boost stages are the other two areas.
    Domain {
        name: "samus-looks",
        bridge: "samus_looks_world",
        areas: &[
            ("clip", &["clip.rs", "ahead.rs", "play.rs", "starting.rs"]),
            ("moves", &["moves.rs"]),
            ("run", &["run.rs"]),
        ],
        apart: &[
            // `ClipControls`: the playhead on a step, with its hold slowed.
            Apart {
                file: "state.rs",
                function: "on",
                area: "clip",
            },
            // `ClipPlays`: the time left runs down, and the clip moves on.
            Apart {
                file: "state.rs",
                function: "run_down",
                area: "clip",
            },
            // `BoostStages`: a stage's laps count down.
            Apart {
                file: "state.rs",
                function: "after_lap",
                area: "run",
            },
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
    /// The score's duplication part, in eighths.
    duplication: usize,
    /// What clumps (Q2) would add, in eighths: a diagnostic outside the score.
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
            duplication: size.duplication.eighths(),
            clumps: size.clumps.eighths(),
            logic: size.logic,
            types: size.types,
            data: size.data,
            lines,
        }
    }

    /// This less a part of it.
    const fn minus(self, part: Self) -> Self {
        Self {
            score: self.score.saturating_sub(part.score),
            depth: self.depth.saturating_sub(part.depth),
            duplication: self.duplication.saturating_sub(part.duplication),
            clumps: self.clumps.saturating_sub(part.clumps),
            logic: self.logic.saturating_sub(part.logic),
            types: self.types.saturating_sub(part.types),
            data: self.data.saturating_sub(part.data),
            lines: self.lines.saturating_sub(part.lines),
        }
    }

    const fn plus(self, other: Self) -> Self {
        Self {
            score: self.score.saturating_add(other.score),
            depth: self.depth.saturating_add(other.depth),
            duplication: self.duplication.saturating_add(other.duplication),
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

    /// Score, logic, types, data, depth, duplication, clumps and lines, as table cells.
    fn cells(self) -> String {
        format!(
            "{} | {} | {} | {} | {} | {} | {} | {}",
            self.score(),
            grouped(self.logic),
            grouped(self.types),
            grouped(self.data),
            tokens(self.depth),
            tokens(self.duplication),
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

/// A part of a model: a file, less the functions of it counted apart, or one of those.
struct Part {
    /// The file, or the function's: its measure is the part's.
    file: File,
    /// The function, when the part is one counted apart.
    function: Option<&'static str>,
    /// The area that holds it.
    area: Option<&'static str>,
}

/// A domain's measured parts.
struct Measured {
    domain: &'static Domain,
    /// The model's parts: its files, each less the functions counted apart, then those.
    model: Vec<Part>,
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
        self.model.iter().map(|part| part.file.measure).sum()
    }

    fn area(&self, name: &str) -> Measure {
        self.model
            .iter()
            .filter(|part| part.area == Some(name))
            .map(|part| part.file.measure)
            .sum()
    }

    /// The model's files in no area.
    fn unassigned(&self) -> Measure {
        self.model
            .iter()
            .filter(|part| part.area.is_none())
            .map(|part| part.file.measure)
            .sum()
    }

    /// The old code: the notes' figure for the whole domain, else the sum of the areas'.
    fn old(&self) -> Option<u64> {
        self.count.old.or_else(|| {
            self.count.areas.iter().try_fold(0_u64, |sum, area| {
                area.old.map(|old| sum.saturating_add(old))
            })
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

/// The shared code's share of a model's score, `shared` of `model` in eighths, in whole percent,
/// flagged over [`SHARED_FLAG`].
fn shared_share(shared: usize, model: usize) -> String {
    let percent = rounded(shared.saturating_mul(100), model);
    if shared.saturating_mul(100) > SHARED_FLAG.saturating_mul(model) {
        format!("{percent} % (over {SHARED_FLAG} %)")
    } else {
        format!("{percent} %")
    }
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

/// The model's parts with the functions counted apart: each taken out of its file's part and
/// placed after it, in its own area, measured with all the model's files as they are.
fn with_apart(
    domain: &Domain,
    files: &[PathBuf],
    parts: Vec<Part>,
    in_model: impl Fn(&str) -> String,
) -> Result<Vec<Part>, String> {
    let texts = files
        .iter()
        .map(|file| read(file))
        .collect::<Result<Vec<_>, _>>()?;
    let srcs: Vec<&str> = texts.iter().map(String::as_str).collect();
    let functions = size::functions(&srcs);
    let mut parts: Vec<(Part, Vec<Part>)> =
        parts.into_iter().map(|part| (part, Vec::new())).collect();
    for apart in domain.apart {
        let path = in_model(apart.file);
        let at = parts
            .iter()
            .position(|(part, _)| part.file.path == path)
            .ok_or_else(|| format!("{}: apart: no model file {}", domain.name, apart.file))?;
        let named: Vec<&size::Function<'_>> = functions
            .get(at)
            .into_iter()
            .flatten()
            .filter(|function| function.name == apart.function)
            .collect();
        let (&[function], Some((part, taken))) = (named.as_slice(), parts.get_mut(at)) else {
            return Err(format!(
                "{}: apart: {} holds {} functions {}, not one",
                domain.name,
                apart.file,
                named.len(),
                apart.function
            ));
        };
        let measure = Measure::of(function.size, lines::code_lines(function.text));
        part.file.measure = part.file.measure.minus(measure);
        taken.push(Part {
            file: File { path, measure },
            function: Some(apart.function),
            area: Some(apart.area),
        });
    }
    Ok(parts
        .into_iter()
        .flat_map(|(part, taken)| std::iter::once(part).chain(taken))
        .collect())
}

fn measure_domain(root: &Path, domain: &'static Domain) -> Result<Measured, String> {
    let model_dir = root.join("crates/clean_model").join(domain.name);
    let model_src = model_dir.join("src");
    let files = sources(std::slice::from_ref(&model_src))?;
    let model = measure(root, &files)?;
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
        .map(|file| Part {
            area: domain
                .areas
                .iter()
                .find(|(_, files)| files.iter().any(|name| in_model(name) == file.path))
                .map(|(area, _)| *area),
            function: None,
            file,
        })
        .collect();
    let model = with_apart(domain, &files, model, in_model)?;
    let behaviours = root.join("docs/behaviours");
    let spec = read(&behaviours.join(format!("{}.md", domain.name)))?;
    let notes = read(&behaviours.join(format!("{}.decode.md", domain.name)))?;
    let budget = budget_section(&notes);
    let toml = read(
        &root
            .join("scenarios/vectors")
            .join(format!("{}.toml", domain.name)),
    )?;
    let count = Count::of(&spec, &Vectors::parse(&toml), Some(budget), None);
    let named = domain.areas.iter().map(|(area, _)| area);
    for area in named.chain(domain.apart.iter().map(|apart| &apart.area)) {
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

/// What the tree holds, measured once for the report and the README's status.
struct Tree {
    /// The shared types' files.
    shared_files: Vec<File>,
    /// The rebuilt domains, in registry order.
    domains: Vec<Measured>,
    /// The engine code not yet rebuilt, file by file.
    engine: Vec<File>,
    /// The sources the structure section outlines.
    sources: structure::Sources,
    /// The movies' cover by the checks.
    timeline: timeline::Timeline,
    floors: BTreeMap<(String, String), usize>,
    coverage: BTreeMap<String, Coverage>,
}

impl Tree {
    /// The tree at `root`, measured.
    ///
    /// # Errors
    ///
    /// The path and the error when a file can't be read, or a registry entry that names an area
    /// the decode notes' Budget section doesn't state or a file the model doesn't hold.
    fn measure(root: &Path) -> Result<Self, String> {
        let shared_files = measure(root, &sources(&[root.join(SHARED_TYPES)])?)?;
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
        let engine = measure(root, &engine)?;
        let names: Vec<&str> = DOMAINS.iter().map(|domain| domain.name).collect();
        let floors = floors(&read(&root.join(FLOORS))?)?;
        Ok(Self {
            shared_files,
            domains,
            engine,
            sources: structure::Sources::read(root, &names)?,
            timeline: timeline::Timeline::measure(root, &floors)?,
            floors,
            coverage: coverage(&read(&root.join(COVERAGE))?)?,
        })
    }

    /// The rebuilt models with the shared types: their measure and their file count.
    fn models(&self) -> (Measure, usize) {
        let measure = self
            .domains
            .iter()
            .map(Measured::model_total)
            .sum::<Measure>()
            .plus(total(&self.shared_files));
        let files = self
            .domains
            .iter()
            .map(|domain| {
                domain
                    .model
                    .iter()
                    .filter(|part| part.function.is_none())
                    .count()
            })
            .sum::<usize>()
            .saturating_add(self.shared_files.len());
        (measure, files)
    }

    /// The engine code not yet rebuilt: its measure and its file count.
    fn remaining(&self) -> (Measure, usize) {
        (total(&self.engine), self.engine.len())
    }

    /// The bridges: their measure and their file count.
    fn bridges(&self) -> (Measure, usize) {
        let files: Vec<&File> = self
            .domains
            .iter()
            .flat_map(|domain| &domain.bridge)
            .collect();
        (files.iter().map(|file| file.measure).sum(), files.len())
    }
}

/// The report for the tree at `root`.
///
/// # Errors
///
/// The path and the error when a file can't be read, or a registry entry that names an area the
/// decode notes' Budget section doesn't state or a file the model doesn't hold.
pub fn generate(root: &Path) -> Result<String, String> {
    Ok(report(&Tree::measure(root)?))
}

fn report(tree: &Tree) -> String {
    let domains = &tree.domains;
    let shared = total(&tree.shared_files);
    let mut out = String::new();
    header(&mut out);
    summary(&mut out, domains, shared, &tree.floors);
    models(&mut out, domains, shared);
    lines_section(&mut out, domains, &tree.shared_files);
    areas(&mut out, domains);
    bridge_and_harness(&mut out, domains);
    floors_section(&mut out, domains, &tree.floors);
    coverage_section(&mut out, domains, &tree.coverage);
    engine_section(&mut out, tree);
    tree.timeline.section(&mut out);
    structure::section(&mut out, tree);
    out
}

fn header(out: &mut String) {
    let _ = write!(
        out,
        "# Rebuild report

Generated by `{COMMAND}` (`verify::report`) from this commit's tree; never edit it by hand.
`tools/check_report.sh` fails when it differs from what the tree generates.

How it is counted:

- **Score** is `tools size` (`verify::size`): weighted logic tokens, with the depth (Q1) and
  duplication parts it includes; **logic**, **types** and **data** are token counts, the last two
  not held against the budget. **Clumps** (Q2) is a diagnostic, not part of the score. Scores
  are rounded to whole tokens.
- **Model** is the files a domain owns, `crates/clean_model/<domain>`; its score, fit and lines
  count those alone. `{SHARED_TYPES}` is a row of its own, **shared types**, with the domains
  whose model depends on it, and is counted once in the total.
- **Budget** and **units** are `tools spec-units` (`verify::spec_units`) on the domain's spec and
  vectors, k and the areas from the Budget section of its decode notes, which the implementer
  never sees (MODEL §1 decision 10). **Fit** is score ÷ budget; the total's score includes the
  shared types. An area's model score is the model files the registry
  (`verify::report::DOMAINS`) assigns it, less the functions it counts apart, plus the functions
  it assigns the area: a file that holds more than one area's code has a row per function counted
  apart (`fn` and its name), measured with the rest of the model. Files in no area are the
  domain's shared code and count in its total only. **Area fit** is the areas' score ÷ the
  budget, the shared code left out; **shared** is the shared code's share of the model's score,
  flagged when it is over {SHARED_FLAG} %, since no spec unit pays for it. The total's shared code
  includes the shared types.
- **Old code** is the Budget section's figure for the whole domain where it gives one, else the
  sum of its area `old` figures, at the commit its
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
            "Area fit",
            "Shared",
            "Old code",
            "Model lines",
            "Logic-equivalent lines",
            "Harness frames",
            "Vectors",
        ],
    );
    // The models, their shared code (the shared types' included), budgets, old code, harness
    // frames and vectors.
    let mut sum = (shared, shared, Some(0_u64), Some(0_u64), 0usize, 0usize);
    for domain in domains {
        let model = domain.model_total();
        let unassigned = domain.unassigned();
        let (_, frames) = harness_frames(floors, domain.domain.name);
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            domain.domain.name,
            model.score(),
            or_dash(domain.count.budget()),
            fit(model.score, domain.count.budget()),
            fit(model.minus(unassigned).score, domain.count.budget()),
            shared_share(unassigned.score, model.score),
            or_dash(domain.old()),
            grouped(model.lines),
            model.logic_lines(),
            grouped(frames),
            grouped(domain.vectors),
        );
        sum = (
            sum.0.plus(model),
            sum.1.plus(unassigned),
            sum.2
                .zip(domain.count.budget())
                .map(|(a, b)| a.saturating_add(b)),
            sum.3.zip(domain.old()).map(|(a, b)| a.saturating_add(b)),
            sum.4.saturating_add(frames),
            sum.5.saturating_add(domain.vectors),
        );
    }
    let _ = writeln!(
        out,
        "| {} | {} | - | - | - | - | - | {} | {} | - | - |",
        shared_label(domains),
        shared.score(),
        grouped(shared.lines),
        shared.logic_lines(),
    );
    let (model, unassigned, budget, old, frames, vectors) = sum;
    let _ = writeln!(
        out,
        "| total | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
        model.score(),
        or_dash(budget),
        fit(model.score, budget),
        fit(model.minus(unassigned).score, budget),
        shared_share(unassigned.score, model.score),
        or_dash(old),
        grouped(model.lines),
        model.logic_lines(),
        grouped(frames),
        grouped(vectors),
    );
}

const MEASURE_COLUMNS: [&str; 8] = [
    "Score",
    "Logic",
    "Types",
    "Data",
    "Depth",
    "Duplication",
    "Clumps",
    "Lines",
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
        for part in &domain.model {
            let function = part
                .function
                .map(|name| format!(" `fn {name}`"))
                .unwrap_or_default();
            let _ = writeln!(
                out,
                "| `{}`{function} | {} | {} | {} |",
                part.file.path,
                part.area.unwrap_or("-"),
                part.file.measure.holds().label(),
                part.file.measure.cells()
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
            let files = || domain.model.iter().map(|part| &part.file);
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
            domain
                .count
                .k
                .map_or_else(|| String::from("-"), |k| k.to_string()),
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
        let unassigned = domain.unassigned();
        let _ = writeln!(
            out,
            "\nArea fit {}: the areas' score, {}, ÷ the budget. Shared code {}, its share of the model {}.",
            fit(model.minus(unassigned).score, domain.count.budget()),
            model.minus(unassigned).score(),
            unassigned.score(),
            shared_share(unassigned.score, model.score)
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
    /// Each declared field: its name, the frames it was compared on, the frames it was masked on.
    fields: Vec<(String, usize, usize)>,
    /// Each catalogue entry: its name and its counts.
    entries: Vec<EntryCoverage>,
}

/// A catalogue entry's oracle coverage over every row.
#[derive(Default, Debug, PartialEq, Eq)]
struct EntryCoverage {
    /// The entry's name.
    name: String,
    /// The frames that exercised it, matched and verified.
    matched: usize,
    /// Of the exercised frames, those on which the null model matched every field it reaches.
    null_matched: usize,
    /// The frames that exercised it with its fields compared.
    exercised: usize,
    /// The frames that exercised it unchecked.
    unchecked: usize,
}

/// The coverage by domain: `domain frames <4 counts>`, `domain field <2 counts> <name>` and
/// `domain entry <name> <4 counts>` lines, `#` comments skipped.
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
            [domain, "field", compared, masked, name @ ..] if !name.is_empty() => {
                let field = (
                    name.join(" "),
                    number(line, compared)?,
                    number(line, masked)?,
                );
                domains
                    .entry((*domain).to_owned())
                    .or_default()
                    .fields
                    .push(field);
            }
            [
                domain,
                "entry",
                name,
                matched,
                null_matched,
                exercised,
                unchecked,
            ] => {
                let entry = EntryCoverage {
                    name: (*name).to_owned(),
                    matched: number(line, matched)?,
                    null_matched: number(line, null_matched)?,
                    exercised: number(line, exercised)?,
                    unchecked: number(line, unchecked)?,
                };
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
        "\nPer declared field, the compared frames on which it was compared and those on which \
         a mask, by its named writer, put it out of scope. The summary's harness frames count a \
         frame matched on its unmasked fields alone.\n\n",
    );
    table(out, 2, &["Domain", "Field", "Compared", "Masked"]);
    for domain in domains {
        let name = domain.domain.name;
        for (field, compared, masked) in coverage
            .get(name)
            .map(|coverage| coverage.fields.as_slice())
            .unwrap_or_default()
        {
            let _ = writeln!(
                out,
                "| {name} | {field} | {} | {} |",
                grouped(*compared),
                grouped(*masked)
            );
        }
    }
    out.push_str(
        "\nPer catalogue entry, the frames that exercised it with every field its output reaches \
         compared, those of them matched and verified, those of them on which the null model \
         matched every such field too, and the frames that exercised it unchecked: one of its \
         fields masked, or an output (an event) the harness doesn't compare. An entry the null \
         model matches on as many frames as the model is only weakly checked by the rows, since \
         doing nothing would pass them too; an entry no frame verifies is checked by its vectors \
         only.\n\n",
    );
    table(
        out,
        2,
        &[
            "Domain",
            "Entry",
            "Matched and verified",
            "Null model",
            "Exercised",
            "Unchecked",
        ],
    );
    for domain in domains {
        let name = domain.domain.name;
        for entry in coverage
            .get(name)
            .map(|coverage| coverage.entries.as_slice())
            .unwrap_or_default()
        {
            let (matched, null) = if entry.exercised == 0 {
                (String::from("vectors only"), String::from("-"))
            } else {
                (grouped(entry.matched), grouped(entry.null_matched))
            };
            let _ = writeln!(
                out,
                "| {name} | {} | {matched} | {null} | {} | {} |",
                entry.name,
                grouped(entry.exercised),
                grouped(entry.unchecked)
            );
        }
    }
}

fn engine_section(out: &mut String, tree: &Tree) {
    out.push_str(
        "\n## Engine not yet rebuilt\n\nThe engine's code, every non-test source of `crates/game` without the bridges: \
         what the rebuilt domains have yet to replace, beside what they hold.\n\n",
    );
    measure_table(out, 1, &["Part", "Files", "Logic-equivalent lines"]);
    let (remaining, files) = tree.remaining();
    let (bridge, bridges) = tree.bridges();
    let (models, model_files) = tree.models();
    for (part, count, measure) in [
        ("engine, not yet rebuilt", files, remaining),
        ("bridges", bridges, bridge),
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

/// The pages the tree at `root` generates, measured once.
struct Pages {
    report: String,
    /// The README's status.
    status: String,
    /// The timeline's picture.
    svg: String,
}

fn pages(root: &Path) -> Result<Pages, String> {
    let tree = Tree::measure(root)?;
    Ok(Pages {
        report: report(&tree),
        status: status::status(root, &tree)?,
        svg: tree.timeline.svg(),
    })
}

/// The committed report's and the README status's staleness: an error naming the command that
/// regenerates them when either differs from what the tree at `root` generates.
///
/// # Errors
///
/// The regeneration command when the report or the README's status is stale or missing, or why
/// they can't be generated.
pub fn check(root: &Path) -> Result<(), String> {
    let pages = pages(root)?;
    let committed = fs::read_to_string(root.join(PATH)).unwrap_or_default();
    current(&committed, &pages.report)?;
    let svg = fs::read_to_string(root.join(timeline::SVG)).unwrap_or_default();
    current_at(timeline::SVG, &svg, &pages.svg)?;
    let readme = fs::read_to_string(root.join(README)).unwrap_or_default();
    status_current(&readme, &pages.status)
}

/// Whether the committed report is the fresh one; a checkout's CRLF line endings don't count.
fn current(committed: &str, fresh: &str) -> Result<(), String> {
    current_at(PATH, committed, fresh)
}

/// Whether the committed page at `path` is the fresh one; CRLF line endings don't count.
fn current_at(path: &str, committed: &str, fresh: &str) -> Result<(), String> {
    if committed.replace("\r\n", "\n") == fresh {
        Ok(())
    } else {
        Err(format!(
            "{path} is stale: regenerate it with `{COMMAND}` from the repository root"
        ))
    }
}

/// Whether the README's status, between its markers, is the fresh one; CRLF doesn't count.
fn status_current(readme: &str, fresh: &str) -> Result<(), String> {
    let readme = readme.replace("\r\n", "\n");
    if status_section(&readme) == Some(fresh) {
        Ok(())
    } else {
        Err(format!(
            "{README}'s status is stale: regenerate it with `{COMMAND}` from the repository root"
        ))
    }
}

/// The text between the README's status markers.
fn status_section(readme: &str) -> Option<&str> {
    let (_, after) = readme.split_once(STATUS_BEGIN)?;
    let (section, _) = after.split_once(STATUS_END)?;
    Some(section)
}

/// `readme` with `status` between its markers, and the text around them kept.
fn with_status(readme: &str, status: &str) -> Result<String, String> {
    let readme = readme.replace("\r\n", "\n");
    let missing = || format!("{README}: no `{STATUS_BEGIN}` line before a `{STATUS_END}` line");
    let (before, after) = readme.split_once(STATUS_BEGIN).ok_or_else(missing)?;
    let (_, rest) = after.split_once(STATUS_END).ok_or_else(missing)?;
    Ok(format!("{before}{STATUS_BEGIN}{status}{STATUS_END}{rest}"))
}

/// Prints the report for the tree in the current directory; with `--write`, writes it to
/// [`PATH`] and the status into [`README`]; with `--check`, fails when either is stale.
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
            let pages = pages(root)?;
            fs::write(root.join(PATH), pages.report).map_err(|e| format!("{PATH}: {e}"))?;
            fs::write(root.join(timeline::SVG), pages.svg)
                .map_err(|e| format!("{}: {e}", timeline::SVG))?;
            let readme = with_status(&read(&root.join(README))?, &pages.status)?;
            fs::write(root.join(README), readme).map_err(|e| format!("{README}: {e}"))?;
            Ok(format!(
                "wrote {PATH}, {} and {README}'s status\n",
                timeline::SVG
            ))
        }
        [flag] if flag == "--check" => check(root).map(|()| {
            format!(
                "{PATH}, {} and {README}'s status are current\n",
                timeline::SVG
            )
        }),
        _ => Err(USAGE.to_owned()),
    }
}
