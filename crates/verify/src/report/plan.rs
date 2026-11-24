//! The whole-game plan: its domains, platform work, milestone and ladder, checked against the
//! rebuilt domains.

use std::collections::BTreeSet;

use serde::Deserialize;

mod graph;

pub(super) use graph::{section as graph_section, undeclared};

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
}

/// A rung of the ladder toward the headline milestone: a shorter stretch of its movie, between
/// two seeds, that the engine is to play alone first, and the plan items it needs.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Rung {
    /// Its id, `M0` onwards.
    pub(super) id: String,
    /// What the README calls it.
    pub(super) name: String,
    /// The seed whose movie frame starts the stretch, `CAPTURE:INDEX`.
    pub(super) starts: String,
    /// The seed whose movie frame ends it, `CAPTURE:INDEX`.
    pub(super) ends: String,
    /// The plan ids of the domains and platform work the stretch needs.
    needs: Vec<String>,
}

/// A seed named `CAPTURE:INDEX`: its capture and index.
///
/// # Errors
///
/// The plan and the name when it isn't `CAPTURE:INDEX`.
pub(super) fn seed(name: &str) -> Result<(&str, u32), String> {
    name.split_once(':')
        .and_then(|(capture, index)| Some((capture, index.parse().ok()?)))
        .ok_or_else(|| format!("{PLAN}: `{name}` is not a seed, CAPTURE:INDEX"))
}

/// The headline milestone: a stretch of a movie, from power-on, that the engine is to replay
/// alone, and the plan items it needs.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Milestone {
    /// What the README calls it.
    pub(super) name: String,
    /// The movie: the key of a movie of `movies/manifest.toml`, its file name's word after
    /// `super-metroid-`.
    pub(super) movie: String,
    /// The seed whose movie frame ends the stretch, `CAPTURE:INDEX`: the first room past it.
    ends: String,
    /// The plan ids of the domains and platform work the stretch needs.
    needs: Vec<String>,
}

impl Milestone {
    /// The seed that ends the stretch: its capture and index.
    pub(super) fn end(&self) -> Result<(&str, u32), String> {
        seed(&self.ends)
    }
}

/// A clean-room model crate and the rounds landed on it.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Model {
    name: String,
    /// The rounds landed on it, which the plan records for its readers; no page shows it.
    #[serde(rename = "rounds")]
    _rounds: usize,
}

/// A clean-room domain of the plan.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlanDomain {
    id: String,
    name: String,
    area: Area,
    /// Its wave, which the plan records for its readers; no page shows it.
    #[serde(rename = "wave")]
    _wave: u8,
    /// The plan's estimate of its rounds, for its readers; no page shows it.
    #[serde(rename = "rounds")]
    _rounds: usize,
    /// The model crates that hold it, once work starts.
    #[serde(default, deserialize_with = "model_owners")]
    model: Vec<String>,
    #[serde(default)]
    status: Stage,
    /// The ids of the domains whose exports it reads.
    #[serde(default)]
    reads: Vec<String>,
}

fn model_owners<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Owners {
        Single(String),
        Multiple(Vec<String>),
    }

    match Owners::deserialize(deserializer)? {
        Owners::Single(model) => Ok(vec![model]),
        Owners::Multiple(models) if models.is_empty() => {
            Err(serde::de::Error::custom("empty domain model list"))
        }
        Owners::Multiple(models) => Ok(models),
    }
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
    Game,
    Screens,
    Audio,
}

impl Area {
    /// Every area, in the plan's order.
    const ALL: [Self; 10] = [
        Self::Samus,
        Self::Weapons,
        Self::World,
        Self::Enemy,
        Self::Boss,
        Self::Interface,
        Self::Run,
        Self::Game,
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
            Self::Game => "game",
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
    /// Its name, wave and lanes, which the plan records for its readers; no page shows them.
    #[serde(rename = "name")]
    _name: String,
    #[serde(rename = "wave")]
    _wave: u8,
    #[serde(rename = "lanes")]
    _lanes: usize,
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
    /// domains, each domain's models occur once and are named in its area, a started domain
    /// names at least one model, each domain reads only other domains of the plan, platform work comes
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
            if domain.model.is_empty() && domain.status != Stage::NotStarted {
                return Err(format!(
                    "{PLAN}: {}: {} without a model",
                    domain.id,
                    domain.status.label()
                ));
            }
            let mut owners = BTreeSet::new();
            for model in &domain.model {
                if !listed.contains(model.as_str()) {
                    return Err(format!("{PLAN}: {}: no model {model}", domain.id));
                }
                if !domain.area.names(model) {
                    return Err(format!(
                        "{PLAN}: {}: the model {model} is not in its area, {}",
                        domain.id,
                        domain.area.word()
                    ));
                }
                if !owners.insert(model) {
                    return Err(format!("{PLAN}: {}: repeated model {model}", domain.id));
                }
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
        Ok(plan)
    }

    /// The headline milestone.
    pub(super) const fn milestone(&self) -> &Milestone {
        &self.milestone
    }

    /// The ladder toward the milestone, in order.
    pub(super) fn rungs(&self) -> &[Rung] {
        &self.rung
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

    /// The plan's domains a model holds.
    fn holding<'a>(&'a self, model: &'a str) -> impl Iterator<Item = &'a PlanDomain> {
        self.domain
            .iter()
            .filter(move |domain| domain.model.iter().any(|owner| owner == model))
    }
}
