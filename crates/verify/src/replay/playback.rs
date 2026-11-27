//! Scoped intro playback. Cinematic work and motion are borrowed from the pinned records.

use super::compare::Observation;
use super::index::Case;
use super::intro_sources::{self, Join};
use super::readers::{
    ActualReader, AdmittedCase, CutTape, ExpectedCut, ExpectedReader, InputReader,
};
use super::result::Outcome;
use super::{Sources, data_failure, production_failure};
use data::rom::{Lorom, Rom, RomAddress};
use game::production::playback::{
    Authority, CrossfadeCount, Cut, CutReport, Handoff, ScenePhase, SceneStart, WholeX, buttons,
};
use game::production::{
    AdvanceInput, Production, ProductionContent, ProductionOutputs, ProductionRules, ProductionSeed,
};
use game::state::owned::OwnedState;
use types::playback::{PadSample, TimelinePlace};

pub(super) fn advance(
    production: &mut Production,
    inputs: &CutTape<Cut>,
    cut: u64,
) -> Result<ProductionOutputs, Outcome> {
    production
        .advance(AdvanceInput::Playback(*inputs.at(cut)?))
        .map_err(|fault| production_failure(cut, fault))
}

pub(super) fn admit(case: &Case, sources: &Sources) -> Result<AdmittedCase, Outcome> {
    let Some(Join::Playback { segments }) = &case.intro_join else {
        return Err(data_failure(None, "intro playback sources"));
    };
    let lock = super::read_lock(sources)?;
    let image = std::fs::read(&sources.rom).map_err(|_| data_failure(None, "player ROM file"))?;
    let rom = Rom::new(image).map_err(|_| data_failure(None, "player ROM pin"))?;
    let content = ProductionContent::Playback(
        import::playback::import(rom.lorom())
            .map_err(|_| data_failure(None, "recorded pad timeline import"))?
            .into(),
    );
    let rules = ProductionRules::Playback;
    let (rows, prior) = intro_sources::origin(case, sources, &lock)?;
    let window = case.cache_window()?;
    let first = *window.cuts.start();
    let origin = window.origin;
    if !super::played::intro::active(&prior.wram)?
        || Phase::native(&prior.wram)? != Phase::Off
        || Pad::retained(&prior.wram)? != Pad::NEUTRAL
        || word(&prior.wram, 0x1a57)? != 0
    {
        return Err(data_failure(
            Some(origin),
            "prior page with no demo or actors",
        ));
    }
    let production = Production::admit(
        ProductionSeed::Playback(ScenePhase::Off),
        content.clone(),
        rules.clone(),
    )
    .map_err(|_| data_failure(Some(origin), "playback origin admission"))?;
    let native = NativeTimelines::read(rom.lorom())?;
    let mut inputs = Vec::new();
    let mut expected = Vec::new();
    intro_sources::visit(
        rows,
        prior,
        segments,
        window.cuts.clone(),
        |_, prior, current| {
            inputs.push(stimulus(&prior.wram, &current.wram)?);
            expected.push(native.observe(&prior.wram, &current.wram)?);
            Ok(())
        },
    )?;
    Ok(AdmittedCase {
        production,
        content,
        rules,
        inputs: InputReader::Playback(CutTape {
            first,
            values: inputs,
        }),
        expected: ExpectedReader::RecordedCuts(CutTape {
            first,
            values: expected,
        }),
        actual: ActualReader::Playback,
        schedule: window.cuts.collect(),
    })
}

fn word(ram: &[u8], at: usize) -> Result<u16, Outcome> {
    let bytes = ram
        .get(at..)
        .and_then(<[u8]>::first_chunk::<2>)
        .ok_or_else(|| data_failure(None, "intro replay WRAM word"))?;
    Ok(u16::from_le_bytes(*bytes))
}

// On these rows the cinematic counter increment and the pad step lie on the same side of
// every cut. The old gate includes each room's last actor step before its subsequent hide.
fn actor_reached(before: &[u8], after: &[u8]) -> Result<bool, Outcome> {
    Ok(word(before, 0x1a57)? != 0 && word(before, 0x1a51)? != word(after, 0x1a51)?)
}

fn stimulus(before: &[u8], after: &[u8]) -> Result<Cut, Outcome> {
    // The intro independently opens each room's actor gate and places Samus.
    let start = if word(before, 0x1a57)? == 0 && word(after, 0x1a57)? != 0 {
        match word(after, 0x0af6)? {
            155 => Some(SceneStart::Battle),
            376 => Some(SceneStart::Discovery),
            _ => return Err(data_failure(None, "intro scene entry placement")),
        }
    } else {
        None
    };
    let actor = if actor_reached(before, after)? {
        Some((
            WholeX(word(before, 0x0af6)?),
            CrossfadeCount(word(after, 0x1a4b)?),
        ))
    } else {
        None
    };
    Ok(Cut { start, actor })
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Pad {
    held: i64,
    fresh: i64,
}

impl Pad {
    const NEUTRAL: Self = Self { held: 0, fresh: 0 };
    const ABSENT: Self = Self {
        held: -1,
        fresh: -1,
    };

    fn actual(sample: PadSample) -> Self {
        Self {
            held: i64::from(buttons(sample.held).0),
            fresh: i64::from(buttons(sample.pressed).0),
        }
    }

    fn retained(ram: &[u8]) -> Result<Self, Outcome> {
        let held = word(ram, 0x0a84)?;
        let fresh = word(ram, 0x0a86)?;
        if held != word(ram, 0x0a8c)? || fresh != word(ram, 0x0a8e)? || fresh & !held != 0 {
            return Err(data_failure(None, "native retained pad pairs"));
        }
        Ok(Self {
            held: native_buttons(held)?,
            fresh: native_buttons(fresh)?,
        })
    }
}

fn native_buttons(word: u16) -> Result<i64, Outcome> {
    use types::button;
    if word & !0x02d0 != 0 {
        return Err(data_failure(None, "native recorded pad button"));
    }
    let controls = [
        (0x0200, button::LEFT),
        (0x0080, button::JUMP),
        (0x0040, button::SHOOT),
        (0x0010, button::AIM_UP),
    ];
    Ok(controls
        .into_iter()
        .filter(|(mask, _)| word & mask != 0)
        .map(|(_, control)| i64::from(control.0))
        .sum())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Off,
    Battle,
    Approach,
    Watch,
    Locked,
    Released,
}

impl Phase {
    const fn active(self) -> bool {
        matches!(self, Self::Battle | Self::Approach | Self::Watch)
    }

    fn native(ram: &[u8]) -> Result<Self, Outcome> {
        let updates = [word(ram, 0x0a42)?, word(ram, 0x0a44)?, word(ram, 0x0a60)?];
        match (word(ram, 0x0a88)?, updates) {
            (0x8000, [0xe6c9, 0xe833, 0xe91d]) => match word(ram, 0x0a7a)? {
                0x83bf => Ok(Self::Battle),
                0x864f => Ok(Self::Approach),
                0x866a => Ok(Self::Watch),
                _ => Err(data_failure(None, "native playback phase")),
            },
            (0, [0xe713, 0xe833, 0xe90e]) => Ok(Self::Locked),
            (0, [0xe8cd, 0xe8cd, 0xe913]) => Ok(Self::Released),
            (0, _) => Ok(Self::Off),
            _ => Err(data_failure(
                None,
                "native playback admission and authority",
            )),
        }
    }

    const fn code(self) -> i64 {
        match self {
            Self::Off => 0,
            Self::Battle => 1,
            Self::Approach => 2,
            Self::Watch => 3,
            Self::Locked => 4,
            Self::Released => 5,
        }
    }
}

// The native pointer refers to the token after the current span. These maps use original
// addresses and durations independently of the typed pad import and its production sampling.
struct NativeTimeline {
    start: u16,
    holds: Vec<(u16, u32, u16)>,
    tail: Option<(u16, u32)>,
}

impl NativeTimeline {
    fn read(rom: Lorom<'_>, start: u16, count: u16, tail: Option<u16>) -> Result<Self, Outcome> {
        let mut holds = Vec::new();
        let mut total = 0_u32;
        let mut pointer = start;
        for _ in 0..count {
            let duration = word(
                rom.from(RomAddress::new(0x91, pointer))
                    .map_err(|_| data_failure(None, "native recorded pad span"))?,
                0,
            )?;
            pointer = pointer.wrapping_add(6);
            holds.push((pointer, total, duration));
            total = total.saturating_add(u32::from(duration));
        }
        Ok(Self {
            start,
            holds,
            tail: tail.map(|pointer| (pointer, total)),
        })
    }

    fn progress(&self, ram: &[u8]) -> Result<i64, Outcome> {
        let pointer = word(ram, 0x0a7e)?;
        let remaining = word(ram, 0x0a7c)?;
        if pointer == self.start && remaining == 1 {
            return Ok(0);
        }
        if let Some((tail, total)) = self.tail
            && pointer == tail
            && remaining == 1
        {
            return Ok(i64::from(total));
        }
        self.holds
            .iter()
            .find_map(|&(after, first, duration)| {
                (after == pointer && (1..=duration).contains(&remaining)).then(|| {
                    i64::from(first)
                        .saturating_add(i64::from(duration.saturating_sub(remaining)))
                        .saturating_add(1)
                })
            })
            .ok_or_else(|| data_failure(None, "native playback progress"))
    }
}

struct NativeTimelines {
    battle: NativeTimeline,
    approach: NativeTimeline,
    watch: NativeTimeline,
}

impl NativeTimelines {
    fn read(rom: Lorom<'_>) -> Result<Self, Outcome> {
        Ok(Self {
            battle: NativeTimeline::read(rom, 0x8694, 17, None)?,
            approach: NativeTimeline::read(rom, 0x860d, 2, Some(0x861f))?,
            watch: NativeTimeline::read(rom, 0x8623, 5, Some(0x8647))?,
        })
    }

    fn observe(&self, before: &[u8], after: &[u8]) -> Result<ExpectedCut, Outcome> {
        let phase = Phase::native(after)?;
        let prior = Phase::native(before)?;
        let progress = match phase {
            Phase::Battle => self.battle.progress(after)?,
            Phase::Approach => self.approach.progress(after)?,
            Phase::Watch => self.watch.progress(after)?,
            Phase::Off | Phase::Locked | Phase::Released => -1,
        };
        let last = Pad::retained(after)?;
        let mut publication = Publication::EMPTY;
        match (prior, phase, progress) {
            (Phase::Off, Phase::Battle, 0) => publication.start = Some(Handoff::StartBattle),
            (Phase::Locked, Phase::Approach, 0) => {
                publication.start = Some(Handoff::StartDiscovery);
            }
            (Phase::Battle, Phase::Locked, _) => {
                if word(after, 0x0a1c)? != 2 {
                    return Err(data_failure(None, "native terminal standing-left pose"));
                }
                publication.finish = Some(Handoff::StandLeft);
                publication.authority = Some(Authority::Controlled);
            }
            (Phase::Watch, Phase::Released, _) => {
                if word(after, 0x0a1c)? != word(before, 0x0a1c)? {
                    return Err(data_failure(None, "native release pose continuity"));
                }
                publication.finish = Some(Handoff::Release);
                publication.authority = Some(Authority::Releasing);
            }
            _ if actor_reached(before, after)? => {
                publication.authority = Some(match phase {
                    Phase::Battle | Phase::Approach | Phase::Watch => Authority::Controlled,
                    Phase::Locked => Authority::BattleLocked,
                    Phase::Off | Phase::Released => Authority::Outside,
                });
            }
            _ => {}
        }
        if publication.finish.is_some()
            || (publication.authority == Some(Authority::Controlled) && phase.active())
        {
            publication.current = last;
            publication.previous = Pad::retained(before)?;
        }
        let semantics = state_fields(phase, progress, last);
        let outputs = publication.fields();
        let semantic_count = semantics.len();
        let output_count = outputs.len();
        let mut fields = semantics.clone();
        fields.extend(outputs);
        Ok(ExpectedCut {
            fields,
            semantics,
            semantic_count,
            output_count,
            replies: 0,
        })
    }
}

fn state_fields(phase: Phase, progress: i64, last: Pad) -> Vec<Observation> {
    fields([
        ("playback.phase", phase.code()),
        ("playback.progress", progress),
        ("playback.last.held", last.held),
        ("playback.last.fresh", last.fresh),
    ])
}

struct Publication {
    start: Option<Handoff>,
    current: Pad,
    previous: Pad,
    finish: Option<Handoff>,
    authority: Option<Authority>,
}

impl Publication {
    const EMPTY: Self = Self {
        start: None,
        current: Pad::ABSENT,
        previous: Pad::ABSENT,
        finish: None,
        authority: None,
    };

    fn actual(report: CutReport) -> Self {
        let mut publication = Self {
            start: report.start,
            ..Self::EMPTY
        };
        if let Some(turn) = report.actor {
            publication.finish = turn.handoff;
            publication.authority = Some(turn.authority);
            if let Some(pad) = turn.pad {
                publication.current = Pad::actual(pad.current);
                publication.previous = Pad::actual(pad.previous);
            }
        }
        publication
    }

    fn fields(&self) -> Vec<Observation> {
        fields([
            ("playback.start", handoff_code(self.start)),
            ("playback.current.held", self.current.held),
            ("playback.current.fresh", self.current.fresh),
            ("playback.previous.held", self.previous.held),
            ("playback.previous.fresh", self.previous.fresh),
            ("playback.finish", handoff_code(self.finish)),
            ("playback.authority", authority_code(self.authority)),
        ])
    }
}

fn fields<const N: usize>(values: [(&str, i64); N]) -> Vec<Observation> {
    values
        .into_iter()
        .map(|(name, value)| Observation::quantity(name, value))
        .collect()
}

const fn handoff_code(handoff: Option<Handoff>) -> i64 {
    match handoff {
        Some(Handoff::StartBattle) => 1,
        Some(Handoff::StartDiscovery) => 2,
        Some(Handoff::StandLeft) => 3,
        Some(Handoff::Release) => 4,
        None => -1,
    }
}

const fn authority_code(authority: Option<Authority>) -> i64 {
    match authority {
        Some(Authority::Outside) => 0,
        Some(Authority::Controlled) => 1,
        Some(Authority::BattleLocked) => 2,
        Some(Authority::Releasing) => 3,
        None => -1,
    }
}

pub(super) fn actual(
    production: &Production,
    report: CutReport,
) -> Result<Vec<Observation>, Outcome> {
    let OwnedState::Playback(phase) = production.state() else {
        return Err(data_failure(None, "playback owned scope"));
    };
    let result = match phase.0 {
        ScenePhase::Off => (Phase::Off, None),
        ScenePhase::Battle(place) => (Phase::Battle, Some(place)),
        ScenePhase::Approach(place) => (Phase::Approach, Some(place)),
        ScenePhase::Watch(place) => (Phase::Watch, Some(place)),
        ScenePhase::BattleLocked => (Phase::Locked, None),
        ScenePhase::Released => (Phase::Released, None),
    };
    let (progress, last) = result
        .1
        .map_or((-1, Pad::NEUTRAL), |TimelinePlace { progress, last }| {
            (i64::from(progress.0), Pad::actual(last))
        });
    let mut fields = state_fields(result.0, progress, last);
    fields.extend(Publication::actual(report).fields());
    Ok(fields)
}

pub(super) fn initial(production: &Production) -> Result<Vec<Observation>, Outcome> {
    actual(
        production,
        CutReport {
            start: None,
            actor: None,
        },
    )
}

#[test]
fn holding_a_started_scene_keeps_its_phase_without_reissuing_the_handoff() {
    let mut production = Production::admit(
        ProductionSeed::Playback(ScenePhase::Off),
        ProductionContent::Playback(
            data::assets::playback::ScenePad {
                battle: Vec::new(),
                approach: Vec::new(),
                watch: Vec::new(),
            }
            .into(),
        ),
        ProductionRules::Playback,
    )
    .unwrap();
    let cut = Cut {
        start: Some(SceneStart::Battle),
        actor: None,
    };
    let output = production.advance(AdvanceInput::Playback(cut)).unwrap();
    let mut reader = ActualReader::Playback;
    let started = reader
        .accept(
            &InputReader::Playback(CutTape {
                first: 1,
                values: vec![cut],
            }),
            &production,
            output,
            1,
        )
        .unwrap();
    let held = reader.held(&production, 2).unwrap();
    assert!(started.contains(&Observation::quantity("playback.start", 1)));
    assert!(held.contains(&Observation::quantity("playback.phase", 1)));
    assert!(held.contains(&Observation::quantity("playback.start", -1)));
}
