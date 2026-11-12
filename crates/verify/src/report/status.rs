//! The README's status: where the rebuild stands against the whole-game plan, from the numbers
//! the tree already holds (Jim, 2026-11-10: the README shows where the project is, and can't go
//! stale).
//!
//! The headline milestone (MODEL §1 decision 14) comes first: its stretch of a movie from
//! power-on, how much of it the engine plays alone today and the plan items it still needs, from
//! the plan's `[milestone]`, the movie's seeds and the free-run gate's green pairs.
//!
//! The plan's domains, their waves and the rounds landed are [`PLAN`]; the models' vectors, harness
//! frames, scores and budgets are the report's own measures; the verification state is the
//! free-run gate's green pairs ([`GREEN`]), the lockstep ratchet ([`LOCKSTEP`]) and the lock
//! ([`LOCK`]), with the rows the harness floors name as the rows cached for the harnesses.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

use serde::Deserialize;

mod graph;

pub(super) use graph::{section as graph_section, undeclared};

use super::timeline::{percent, seed_frame};
use super::{COMMAND, Tree, fit, grouped, or_dash, read, table};
use crate::gate::Ratchet;
use crate::lock::{Lock, RowId};
use crate::ratchet;

/// The whole-game plan as data, from the repository root.
pub(super) const PLAN: &str = "docs/architecture/domains.toml";
/// The free-run gate's green (row, stream) pairs.
pub(super) const GREEN: &str = "scenarios/locks/lsnes.green.txt";
/// The lockstep ratchet.
pub(super) const LOCKSTEP: &str = "scenarios/locks/lockstep.ratchet.txt";
/// The lock both are measured against.
pub(super) const LOCK: &str = "scenarios/locks/lsnes.toml";

/// The plan file.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Plan {
    model: Vec<Model>,
    domain: Vec<PlanDomain>,
    platform: Vec<Platform>,
    milestone: Milestone,
    /// The ladder of rungs toward the milestone, in order.
    #[serde(default)]
    rung: Vec<Rung>,
}

/// A rung of the ladder toward the headline milestone: a shorter stretch of its movie, between
/// two seeds, that the engine is to play alone first, and the plan items it needs.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Rung {
    /// Its id, `M0` onwards.
    id: String,
    /// What the README calls it.
    name: String,
    /// The seed whose movie frame starts the stretch, `CAPTURE:INDEX`.
    starts: String,
    /// The seed whose movie frame ends it, `CAPTURE:INDEX`.
    ends: String,
    /// The plan ids of the domains and platform work the stretch needs.
    needs: Vec<String>,
}

/// A seed named `CAPTURE:INDEX`: its capture and index.
fn seed(name: &str) -> Result<(&str, u32), String> {
    name.split_once(':')
        .and_then(|(capture, index)| Some((capture, index.parse().ok()?)))
        .ok_or_else(|| format!("{PLAN}: `{name}` is not a seed, CAPTURE:INDEX"))
}

/// The headline milestone: a stretch of a movie, from power-on, that the engine is to replay
/// alone, and the plan items it needs.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Milestone {
    /// What the README calls it.
    name: String,
    /// The movie: the key of a movie of `movies/manifest.toml`, its file name's word after
    /// `super-metroid-`.
    movie: String,
    /// The seed whose movie frame ends the stretch, `CAPTURE:INDEX`: the first room past it.
    ends: String,
    /// The plan ids of the domains and platform work the stretch needs.
    needs: Vec<String>,
}

impl Milestone {
    /// The seed that ends the stretch: its capture and index.
    fn end(&self) -> Result<(&str, u32), String> {
        seed(&self.ends)
    }
}

/// A clean-room model crate and the rounds landed on it.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Model {
    name: String,
    rounds: usize,
}

/// A clean-room domain of the plan.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlanDomain {
    id: String,
    name: String,
    area: Area,
    wave: u8,
    /// The plan's estimate of its rounds.
    rounds: usize,
    /// The model crate that holds it, once one does.
    model: Option<String>,
    #[serde(default)]
    status: Stage,
    /// The ids of the domains whose exports it reads.
    #[serde(default)]
    reads: Vec<String>,
}

/// The part of the game a domain belongs to, and the first word of its model's name.
#[derive(Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
enum Area {
    Samus,
    Weapons,
    World,
    Enemy,
    Boss,
    Interface,
    Run,
    Screens,
    Audio,
}

impl Area {
    /// Every area, in the plan's order.
    const ALL: [Self; 9] = [
        Self::Samus,
        Self::Weapons,
        Self::World,
        Self::Enemy,
        Self::Boss,
        Self::Interface,
        Self::Run,
        Self::Screens,
        Self::Audio,
    ];

    /// The word a model of the area starts with.
    const fn word(self) -> &'static str {
        match self {
            Self::Samus => "samus",
            Self::Weapons => "weapons",
            Self::World => "world",
            Self::Enemy => "enemy",
            Self::Boss => "boss",
            Self::Interface => "interface",
            Self::Run => "run",
            Self::Screens => "screens",
            Self::Audio => "audio",
        }
    }

    /// Whether `model` is named in this area: its first word is the area's.
    fn names(self, model: &str) -> bool {
        model.split('-').next() == Some(self.word())
    }
}

/// Platform work of the plan, outside the clean room.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Platform {
    id: String,
    name: String,
    wave: u8,
    lanes: usize,
    #[serde(default)]
    status: Stage,
}

/// How far a domain's work has come.
#[derive(Deserialize, Default, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
enum Stage {
    /// No round has started.
    #[default]
    NotStarted,
    /// Its model exists, and its queue still holds work worth a round.
    InProgress,
    /// Its queue holds nothing worth a round (PROCESS, step 12).
    Done,
}

impl Stage {
    const fn label(self) -> &'static str {
        match self {
            Self::NotStarted => "not started",
            Self::InProgress => "in progress",
            Self::Done => "done",
        }
    }
}

impl Plan {
    /// The plan in `text`, checked against the rebuilt domains: the models are the report's
    /// domains, each domain's model is one of them and named in its area, a started domain
    /// names its model, and each domain reads only other domains of the plan.
    pub(super) fn parse(text: &str, rebuilt: &BTreeSet<&str>) -> Result<Self, String> {
        let plan: Self = toml::from_str(text).map_err(|e| format!("{PLAN}: {e}"))?;
        let listed: BTreeSet<&str> = plan.model.iter().map(|model| model.name.as_str()).collect();
        if listed != *rebuilt || listed.len() != plan.model.len() {
            return Err(format!(
                "{PLAN}: the models {listed:?} are not once each the report's domains {rebuilt:?}"
            ));
        }
        for domain in &plan.domain {
            match (&domain.model, domain.status) {
                (Some(model), _) if !listed.contains(model.as_str()) => {
                    return Err(format!("{PLAN}: {}: no model {model}", domain.id));
                }
                (Some(model), _) if !domain.area.names(model) => {
                    return Err(format!(
                        "{PLAN}: {}: the model {model} is not in its area, {}",
                        domain.id,
                        domain.area.word()
                    ));
                }
                (None, Stage::InProgress | Stage::Done) => {
                    return Err(format!(
                        "{PLAN}: {}: {} without a model",
                        domain.id,
                        domain.status.label()
                    ));
                }
                (Some(_) | None, _) => {}
            }
            if let Some(read) = domain.reads.iter().find(|read| {
                **read == domain.id || !plan.domain.iter().any(|other| other.id == **read)
            }) {
                return Err(format!(
                    "{PLAN}: {} reads {read}, which is not another domain of the plan",
                    domain.id
                ));
            }
        }
        plan.milestone.end()?;
        let needs = plan
            .milestone
            .needs
            .iter()
            .map(|need| ("the milestone", need));
        let rungs = plan
            .rung
            .iter()
            .flat_map(|rung| rung.needs.iter().map(move |need| (rung.id.as_str(), need)));
        if let Some((who, need)) = needs
            .chain(rungs)
            .find(|(_, need)| plan.stage_of(need).is_none())
        {
            return Err(format!("{PLAN}: {who} needs {need}, which the plan lacks"));
        }
        for rung in &plan.rung {
            seed(&rung.starts)?;
            seed(&rung.ends)?;
        }
        Ok(plan)
    }

    /// The stage of the domain or platform work whose plan id is `id`.
    fn stage_of(&self, id: &str) -> Option<Stage> {
        self.domain
            .iter()
            .map(|domain| (&domain.id, domain.status))
            .chain(
                self.platform
                    .iter()
                    .map(|platform| (&platform.id, platform.status)),
            )
            .find(|(listed, _)| *listed == id)
            .map(|(_, stage)| stage)
    }

    /// The items of `needs` not done, by stage, with their count: "2 of 5: S1 (in progress);
    /// R1 (not started)", or "nothing".
    fn still_needed(&self, needs: &[String]) -> String {
        let needed: Vec<(&str, Stage)> = needs
            .iter()
            .filter_map(|need| Some((need.as_str(), self.stage_of(need)?)))
            .filter(|(_, stage)| *stage != Stage::Done)
            .collect();
        if needed.is_empty() {
            String::from("nothing")
        } else {
            format!(
                "{} of {}: {}",
                needed.len(),
                needs.len(),
                ids_by_stage(needed.iter().copied())
            )
        }
    }

    /// The plan's domains a model holds.
    fn holding<'a>(&'a self, model: &'a str) -> impl Iterator<Item = &'a PlanDomain> {
        self.domain
            .iter()
            .filter(move |domain| domain.model.as_deref() == Some(model))
    }

    /// A model's stage: done once every plan domain it holds is done.
    fn stage(&self, model: &str) -> Stage {
        if self
            .holding(model)
            .all(|domain| domain.status == Stage::Done)
        {
            Stage::Done
        } else {
            Stage::InProgress
        }
    }

    fn rounds(&self, model: &str) -> usize {
        self.model
            .iter()
            .find(|listed| listed.name == model)
            .map_or(0, |listed| listed.rounds)
    }

    fn waves(&self) -> BTreeSet<u8> {
        self.domain
            .iter()
            .map(|domain| domain.wave)
            .chain(self.platform.iter().map(|platform| platform.wave))
            .collect()
    }

    fn count(&self, stage: Stage) -> usize {
        self.domain
            .iter()
            .filter(|domain| domain.status == stage)
            .count()
    }
}

/// The plan ids of `items`, grouped by stage: "S1, S2 (in progress); S3 (not started)".
fn ids_by_stage<'a>(items: impl Iterator<Item = (&'a str, Stage)>) -> String {
    let mut groups: Vec<(Stage, Vec<&str>)> = Vec::new();
    for (id, status) in items {
        match groups.iter_mut().find(|(stage, _)| *stage == status) {
            Some((_, ids)) => ids.push(id),
            None => groups.push((status, vec![id])),
        }
    }
    groups
        .iter()
        .map(|(stage, ids)| format!("{} ({})", ids.join(", "), stage.label()))
        .collect::<Vec<_>>()
        .join("; ")
}

/// The README's status for the tree at `root`, the report's measures `tree`: the text between
/// its markers.
///
/// # Errors
///
/// The path and the error when a file can't be read.
pub(super) fn status(root: &Path, tree: &Tree) -> Result<String, String> {
    let plan = &tree.plan;
    let mut out = format!(
        "\n<!-- Generated with docs/REPORT.md by `{COMMAND}` (verify::report) from the tree; \
         `tools/check_report.sh` fails when it is stale. Never edit it by hand. -->\n"
    );
    out.push_str("\n## Where it stands\n");
    milestone(&mut out, root, plan, tree)?;
    ladder(&mut out, root, plan, tree)?;
    headline(&mut out, plan);
    let _ = write!(
        out,
        "\n![How much of each movie the checks cover]({svg})\n\nHow much of each whole movie the \
         lock's rows, the harnesses, the models, the free-run gate and lockstep cover, frame by \
         frame; the figures are in [the report's timeline](docs/REPORT.md#timeline).\n",
        svg = super::timeline::SVG
    );
    models(&mut out, plan, tree);
    waves(&mut out, plan);
    engine(&mut out, tree);
    verification(&mut out, root, tree)?;
    out.push_str(
        "\nEvery figure, file by file and area by area, is in [the rebuild report](docs/REPORT.md); \
         how each model is built, module by module, is in [its structure \
         section](docs/REPORT.md#structure).\n",
    );
    Ok(out)
}

/// The milestone's row: its stretch's frames, those the engine plays alone today and the plan
/// items still needed, by stage.
fn milestone(out: &mut String, root: &Path, plan: &Plan, tree: &Tree) -> Result<(), String> {
    let milestone = &plan.milestone;
    let (capture, index) = milestone.end()?;
    let frames = seed_frame(root, capture, index)?;
    let (category, alone) = tree
        .timeline
        .green_from_start(&milestone.movie, frames)
        .ok_or_else(|| {
            format!(
                "{PLAN}: the milestone's movie {} has no row of the lock",
                milestone.movie
            )
        })?;
    let still = plan.still_needed(&milestone.needs);
    // The last column is text, so it is aligned left as the first is.
    out.push_str(
        "\n| Milestone | Frames | The engine plays alone | Still needed |\n\
         | --- | ---: | ---: | --- |\n",
    );
    let _ = writeln!(
        out,
        "| **{}**, the {category} movie from power-on | {} | {} ({}) | {still} |",
        milestone.name,
        grouped(frames),
        grouped(alone),
        percent(alone, frames),
    );
    out.push_str(
        "\nThe headline milestone ([MODEL §1](docs/architecture/MODEL.md#1-the-goal-and-what-is-exact), \
         decision 14): the engine replays the stretch from power-on with nothing fed back, every \
         compared cut and byte matching. Until it does, the frames it plays alone are those \
         inside the movie's rows whose every stream the free-run gate holds green, each row run \
         from its own seed. Still needed are the plan's domains and platform work the stretch \
         touches ([domains.md §4](docs/architecture/domains.md#4-order)) that are not done.\n",
    );
    Ok(())
}

/// The ladder toward the milestone: each rung's stretch, the frames of it the engine plays
/// alone today and the plan items it still needs.
fn ladder(out: &mut String, root: &Path, plan: &Plan, tree: &Tree) -> Result<(), String> {
    if plan.rung.is_empty() {
        return Ok(());
    }
    out.push_str(
        "\nThe ladder toward it ([domains.md §4](docs/architecture/domains.md#4-order)): shorter \
         stretches of the same movie, each played alone from its first seed, in the order the \
         work reaches them. A rung is reached when the engine plays every frame of it alone.\n\n\
         | Rung | Stretch | Frames | The engine plays alone | Still needed |\n\
         | --- | --- | ---: | ---: | --- |\n",
    );
    for rung in &plan.rung {
        let (start_capture, start_index) = seed(&rung.starts)?;
        let (end_capture, end_index) = seed(&rung.ends)?;
        let start = seed_frame(root, start_capture, start_index)?;
        let end = seed_frame(root, end_capture, end_index)?;
        if end <= start {
            return Err(format!(
                "{PLAN}: {} ends at frame {end}, not after its start {start}",
                rung.id
            ));
        }
        let (_, alone) = tree
            .timeline
            .green_in(&plan.milestone.movie, start..end)
            .ok_or_else(|| {
                format!(
                    "{PLAN}: the milestone's movie {} has no row of the lock",
                    plan.milestone.movie
                )
            })?;
        let frames = end.saturating_sub(start);
        let _ = writeln!(
            out,
            "| {} | {}, frames {}-{} | {} | {} ({}) | {} |",
            rung.id,
            rung.name,
            grouped(start),
            grouped(end.saturating_sub(1)),
            grouped(frames),
            grouped(alone),
            percent(alone, frames),
            plan.still_needed(&rung.needs),
        );
    }
    Ok(())
}

fn headline(out: &mut String, plan: &Plan) {
    let landed: usize = plan.model.iter().map(|model| model.rounds).sum();
    let planned: usize = plan.domain.iter().map(|domain| domain.rounds).sum();
    let _ = write!(
        out,
        "\nOf the {} clean-room domains in the \
         [whole-game plan](docs/architecture/domains.md), **{} are in progress** and **{} done**; \
         {} have not started. **{} rounds** have landed on {} models; the plan estimates {} rounds \
         for the whole game.\n",
        plan.domain.len(),
        plan.count(Stage::InProgress),
        plan.count(Stage::Done),
        plan.count(Stage::NotStarted),
        grouped(landed),
        plan.model.len(),
        grouped(planned),
    );
}

fn models(out: &mut String, plan: &Plan, tree: &Tree) {
    out.push_str(
        "\n### The models\n\nEach model is built in the clean room from its spec alone, then \
         checked frame by frame against the original game: its harness frames are the recorded \
         frames on which it matched the original on every compared field, of the frames it ran \
         over. Score and budget are weighted tokens; fit is score over budget.\n\n",
    );
    table(
        out,
        3,
        &[
            "Model",
            "Plan",
            "Status",
            "Rounds",
            "Vectors",
            "Harness frames",
            "Score",
            "Budget",
            "Fit",
        ],
    );
    for domain in &tree.domains {
        let name = domain.domain.name;
        let frames = tree
            .coverage
            .get(name)
            .and_then(|coverage| coverage.frames)
            .map_or_else(
                || String::from("-"),
                |[compared, matched, ..]| format!("{} of {}", grouped(matched), grouped(compared)),
            );
        let model = domain.model_total();
        let _ = writeln!(
            out,
            "| {name} | {} | {} | {} | {} | {frames} | {} | {} | {} |",
            ids_by_stage(
                plan.holding(name)
                    .map(|domain| (domain.id.as_str(), domain.status))
            ),
            plan.stage(name).label(),
            grouped(plan.rounds(name)),
            grouped(domain.vectors),
            model.score(),
            or_dash(domain.count.budget()),
            fit(model.score, domain.count.budget()),
        );
    }
}

fn waves(out: &mut String, plan: &Plan) {
    out.push_str(
        "\n### The whole-game plan\n\nThe plan's waves ([domains.md §4](docs/architecture/domains.md#4-order)), \
         each domain with its plan id; rounds are the plan's estimates, platform work is in \
         lanes.\n\n",
    );
    table(
        out,
        3,
        &[
            "Wave",
            "Clean-room domains",
            "Platform work",
            "Started",
            "Rounds planned",
        ],
    );
    for wave in plan.waves() {
        let domains: Vec<&PlanDomain> = plan
            .domain
            .iter()
            .filter(|domain| domain.wave == wave)
            .collect();
        let named = domains
            .iter()
            .map(|domain| match domain.status {
                Stage::NotStarted => format!("{} {}", domain.id, domain.name),
                Stage::InProgress | Stage::Done => format!(
                    "**{} {}** ({})",
                    domain.id,
                    domain.name,
                    domain.status.label()
                ),
            })
            .collect::<Vec<_>>()
            .join("; ");
        let started = domains
            .iter()
            .filter(|domain| domain.status != Stage::NotStarted)
            .count();
        let platform: Vec<String> = plan
            .platform
            .iter()
            .filter(|platform| platform.wave == wave)
            .map(|platform| format!("{} {} ({})", platform.id, platform.name, platform.lanes))
            .collect();
        let platform = if platform.is_empty() {
            String::from("-")
        } else {
            platform.join("; ")
        };
        let _ = writeln!(
            out,
            "| {wave} | {named} | {platform} | {started} of {} | {} |",
            domains.len(),
            grouped(domains.iter().map(|domain| domain.rounds).sum()),
        );
    }
}

fn engine(out: &mut String, tree: &Tree) {
    out.push_str(
        "\n### The engine\n\nThe engine's code still to be rebuilt (`crates/game` without the \
         bridges), beside the rebuilt models. Logic-equivalent lines are score ÷ 6, the old \
         engine's density.\n\n",
    );
    table(
        out,
        1,
        &["Code", "Files", "Logic-equivalent lines", "Score", "Lines"],
    );
    let (remaining, files) = tree.remaining();
    let (bridge, bridges) = tree.bridges();
    let (models, model_files) = tree.models();
    for (part, count, measure) in [
        ("not yet rebuilt", files, remaining),
        ("bridges from the engine to the models", bridges, bridge),
        ("rebuilt models, shared types included", model_files, models),
    ] {
        let _ = writeln!(
            out,
            "| {part} | {count} | {} | {} | {} |",
            measure.logic_lines(),
            measure.score(),
            grouped(measure.lines)
        );
    }
}

fn verification(out: &mut String, root: &Path, tree: &Tree) -> Result<(), String> {
    let green = Ratchet::parse(&read(&root.join(GREEN))?).map_err(|e| format!("{GREEN}: {e}"))?;
    let green_rows: BTreeSet<&RowId> = green.pairs.iter().map(|(row, _)| row).collect();
    let lockstep =
        ratchet::parse(&read(&root.join(LOCKSTEP))?).map_err(|e| format!("{LOCKSTEP}: {e}"))?;
    let (equal, ticks) = lockstep.iter().fold((0_u64, 0_u64), |(equal, ticks), row| {
        (
            equal.saturating_add(row.equal),
            ticks.saturating_add(row.ticks),
        )
    });
    let lock = Lock::parse(&read(&root.join(LOCK))?).map_err(|e| format!("{LOCK}: {e}"))?;
    let frames: BTreeMap<&str, u64> = lock
        .rows
        .iter()
        .map(|row| (row.id.0.as_str(), row.frames.0))
        .collect();
    let cached: BTreeSet<&str> = tree.floors.keys().map(|(_, row)| row.as_str()).collect();
    let cached_frames = cached.iter().try_fold(0_u64, |sum, row| {
        frames
            .get(row)
            .map(|frames| sum.saturating_add(*frames))
            .ok_or_else(|| format!("the harness floors name {row}, which {LOCK} lacks"))
    })?;
    let all_frames = frames
        .values()
        .fold(0_u64, |sum, frames| sum.saturating_add(*frames));
    let figure = |n: u64| usize::try_from(n).map_or_else(|_| n.to_string(), grouped);
    out.push_str(
        "\n### Verification\n\nThe original game on an emulator (the oracle) recorded the lock's \
         rows; the engine is held to them. See [STATUS](docs/STATUS.md) for how each is run.\n\n",
    );
    table(out, 2, &["Check", "State"]);
    let _ = writeln!(
        out,
        "| the lock | {} rows of play, {} frames |",
        grouped(lock.rows.len()),
        figure(all_frames)
    );
    let _ = writeln!(
        out,
        "| the free-run gate: the engine runs a row alone | {} green (row, stream) pairs over {} rows |",
        grouped(green.pairs.len()),
        grouped(green_rows.len())
    );
    let _ = writeln!(
        out,
        "| the lockstep ratchet: the engine re-seeded each Tick | {} of {} Ticks fully equal over {} rows |",
        figure(equal),
        figure(ticks),
        grouped(lockstep.len())
    );
    let _ = writeln!(
        out,
        "| the harnesses: each model on recorded frames | {} rows cached, {} frames |",
        grouped(cached.len()),
        figure(cached_frames)
    );
    Ok(())
}
