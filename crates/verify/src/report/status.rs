//! Current replay evidence and the remaining whole-game plan.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::Path;

use serde::Deserialize;

mod graph;

pub(super) use graph::{section as graph_section, undeclared};

use super::timeline::seed_frame;
use super::{COMMAND, Tree, fit, grouped, or_dash, table};
use crate::lock::Stream;

/// The whole-game plan as data, from the repository root.
pub(super) const PLAN: &str = "docs/architecture/domains.toml";
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
    /// The streams a rung counts only when it needs the work that gives the engine them.
    #[serde(default)]
    stream: Vec<StreamClaim>,
    /// The streams a rung's rows hold green whether or not the engine plays them, whose green
    /// pairs are counted apart.
    #[serde(default)]
    steady: Vec<Steady>,
}

/// A compared stream a rung's rows can hold green without the engine playing them (#402): its
/// green pairs are shown apart from the others', so they don't stand for play.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Steady {
    /// The stream's name in the lock; the plan says above it why its pairs are green without
    /// play.
    name: String,
}

/// A compared stream the engine gives only once a plan item is done: a rung counts it when its
/// needs hold that item and is measured on the other streams when they don't (the root,
/// 2026-11-12). The milestone needs every stream.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StreamClaim {
    /// The stream's name in the lock.
    name: String,
    /// The plan id of the work that gives the engine the stream.
    given_by: String,
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
    /// The plan ids of the work that waits for it: the domains that consume its content. A
    /// stretch that needs one of them needs this too.
    #[serde(default)]
    before: Vec<String>,
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
    /// names its model, each domain reads only other domains of the plan, platform work comes
    /// before only other work of the plan, and the milestone and each rung need plan items, with
    /// the platform work that comes before them.
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
        for platform in &plan.platform {
            if let Some(after) = platform
                .before
                .iter()
                .find(|after| **after == platform.id || plan.stage_of(after).is_none())
            {
                return Err(format!(
                    "{PLAN}: {} comes before {after}, which is not other work of the plan",
                    platform.id
                ));
            }
        }
        plan.milestone.end()?;
        let stretches = std::iter::once(("the milestone", &plan.milestone.needs))
            .chain(plan.rung.iter().map(|rung| (rung.id.as_str(), &rung.needs)));
        for (who, needs) in stretches {
            if let Some(need) = needs.iter().find(|need| plan.stage_of(need).is_none()) {
                return Err(format!("{PLAN}: {who} needs {need}, which the plan lacks"));
            }
            if let Some((platform, need)) = plan.platform.iter().find_map(|platform| {
                let need = platform.before.iter().find(|after| needs.contains(after))?;
                (!needs.contains(&platform.id)).then_some((&platform.id, need))
            }) {
                return Err(format!(
                    "{PLAN}: {who} needs {need} but not {platform}, which comes before it"
                ));
            }
        }
        for rung in &plan.rung {
            seed(&rung.starts)?;
            seed(&rung.ends)?;
        }
        for steady in &plan.steady {
            if !Stream::try_from(steady.name.as_str()).is_ok_and(Stream::compared) {
                return Err(format!("{PLAN}: {} is not a compared stream", steady.name));
            }
        }
        for claim in &plan.stream {
            if !Stream::try_from(claim.name.as_str()).is_ok_and(Stream::compared) {
                return Err(format!("{PLAN}: {} is not a compared stream", claim.name));
            }
            if plan.stage_of(&claim.given_by).is_none() {
                return Err(format!(
                    "{PLAN}: {} is given by {}, which the plan lacks",
                    claim.name, claim.given_by
                ));
            }
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

    /// The items of `needs` that must come before `item`, as the plan records it: platform
    /// work whose `before` names it, and, for a domain, the domains it reads from, since a
    /// reader's round waits until the domain it reads has landed (PROCESS, "What can run at
    /// once"). Work outside `needs` doesn't count: while a stretch is the slice nothing outside
    /// its needs starts (MODEL §1 decision 24), so the item takes that neighbour from the
    /// records.
    fn prerequisites<'a>(&'a self, item: &str, needs: &'a [String]) -> BTreeSet<&'a str> {
        let platforms = self
            .platform
            .iter()
            .filter(|platform| platform.before.iter().any(|after| after == item))
            .map(|platform| platform.id.as_str());
        let reads = self
            .domain
            .iter()
            .filter(|domain| domain.id == item)
            .flat_map(|domain| domain.reads.iter().map(String::as_str));
        platforms
            .chain(reads)
            .filter(|id| needs.iter().any(|need| need == id))
            .collect()
    }

    /// The order of the work `needs` still holds: each item not done and the items of `needs`
    /// not started that must come before it.
    pub(super) fn order<'a>(&'a self, needs: &'a [String]) -> Order<'a> {
        let mut order = Order::default();
        for need in needs {
            let Some(stage) = self.stage_of(need).filter(|stage| *stage != Stage::Done) else {
                continue;
            };
            let waits: Vec<&str> = self
                .prerequisites(need, needs)
                .into_iter()
                .filter(|id| self.stage_of(id) == Some(Stage::NotStarted))
                .collect();
            match (stage, waits.is_empty()) {
                (Stage::NotStarted, true) => order.ready.push(need),
                (Stage::NotStarted, false) => order.waiting.push((need, waits)),
                (Stage::InProgress, false) => order.ahead.push((need, waits)),
                (Stage::InProgress | Stage::Done, true) | (Stage::Done, false) => {}
            }
        }
        order
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

/// The order of a stretch's remaining work, from the plan's edges ([`Plan::order`]).
#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct Order<'a> {
    /// Not started, and nothing the stretch needs before it is unstarted: work that can start.
    ready: Vec<&'a str>,
    /// Not started, with the unstarted work that must come first.
    waiting: Vec<(&'a str, Vec<&'a str>)>,
    /// In progress though work that comes before it hasn't started: it runs on the records or
    /// the original's layout until that work lands.
    ahead: Vec<(&'a str, Vec<&'a str>)>,
}

impl Order<'_> {
    /// The order as the README's sentence: what can start, what waits on what, and what runs
    /// ahead of work it needs.
    pub(super) fn sentence(&self) -> String {
        let list = |items: &[(&str, Vec<&str>)]| {
            items
                .iter()
                .map(|(item, on)| format!("{item} on {}", on.join(", ")))
                .collect::<Vec<_>>()
                .join("; ")
        };
        let mut parts = Vec::new();
        if !self.ready.is_empty() {
            parts.push(format!("**can start:** {}", self.ready.join(", ")));
        }
        if !self.waiting.is_empty() {
            parts.push(format!("**waiting:** {}", list(&self.waiting)));
        }
        if !self.ahead.is_empty() {
            parts.push(format!(
                "**in progress ahead of work it needs:** {}",
                list(&self.ahead)
            ));
        }
        if parts.is_empty() {
            String::from("nothing left to start")
        } else {
            parts.join(". ")
        }
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

pub(super) fn status(root: &Path, tree: &Tree) -> Result<String, String> {
    let plan = &tree.plan;
    let mut out = format!(
        "\n<!-- Generated by `{COMMAND}` from the indexed baseline and plan. -->\n\n## Where it stands\n"
    );
    let (capture, index) = plan.milestone.end()?;
    let frames = seed_frame(root, capture, index)?;
    let _ = writeln!(
        out,
        "\n**{}**: the {} movie from power-on through frame {}. Still needed: {}.\n",
        plan.milestone.name,
        plan.milestone.movie,
        grouped(frames),
        plan.still_needed(&plan.milestone.needs)
    );
    out.push_str("Whole-movie acceptance requires a complete movie case. Scoped cases and partial integrated cases earn no whole-movie credit.\n");
    tree.replay.section(&mut out);
    out.push_str(
        "\n### The ladder\n\n| Target | Start | End | Still needed |\n| --- | --- | --- | --- |\n",
    );
    for rung in &plan.rung {
        let _ = writeln!(
            out,
            "| {} {} | {} | {} | {} |",
            rung.id,
            rung.name,
            rung.starts,
            rung.ends,
            plan.still_needed(&rung.needs)
        );
    }
    if let Some(rung) = plan.rung.first() {
        let _ = writeln!(
            out,
            "\n{}: {}.\n",
            rung.id,
            plan.order(&rung.needs).sentence()
        );
    }
    headline(&mut out, plan);
    out.push_str("\n### The models\n\nModel implementation and vectors are separate from the replay evidence above.\n\n");
    table(
        &mut out,
        3,
        &[
            "Model", "Plan", "Status", "Rounds", "Vectors", "Score", "Budget", "Fit",
        ],
    );
    for domain in &tree.domains {
        let name = domain.domain.name;
        let model = domain.model_total();
        let _ = writeln!(
            out,
            "| {name} | {} | {} | {} | {} | {} | {} | {} |",
            ids_by_stage(
                plan.holding(name)
                    .map(|domain| (domain.id.as_str(), domain.status))
            ),
            plan.stage(name).label(),
            grouped(plan.rounds(name)),
            grouped(domain.vectors),
            model.score(),
            or_dash(domain.count.budget()),
            fit(model.score, domain.count.budget())
        );
    }
    waves(&mut out, plan);
    out.push_str("\nEvery case's declared scope and limitations are in `scenarios/replay/index.toml`. Model structure and measurements are in [the report](docs/REPORT.md).\n");
    Ok(out)
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
