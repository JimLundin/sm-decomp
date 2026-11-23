//! Independent observations and actual owner publications at the same named uses.

use game::production::{Production, health_tail::Output, hud::tilemap};
use samus_health::{State, Warning, WarningSound};
use serde::Deserialize;

use super::{CUTS, owner};
use crate::replay::{compare::Observation, data_failure, readers::ExpectedCut, result::Outcome};

const GAUGE_WORDS: usize = 16;
const SNAPSHOT_FIELDS: usize = 6;
// Actor and Maintenance snapshots, protection at both Drawing reads and the
// close, four Drawing energy/history quantities, and four gauge quantities.
const CARRIED_FIELDS_PER_CUT: usize = 23;
// The Warning request and all ordered CPU gauge words.
pub(in crate::replay) const OUTPUTS_PER_CUT: usize = 1 + GAUGE_WORDS;

pub(in crate::replay) fn field_count(cuts: usize) -> Option<usize> {
    let carried = cuts.checked_mul(CARRIED_FIELDS_PER_CUT)?;
    if cuts == 0 {
        Some(carried)
    } else {
        carried.checked_add(SNAPSHOT_FIELDS)
    }
}

#[derive(Deserialize)]
struct Health {
    energy_whole: u16,
    energy_fraction: u16,
    pending_whole: u16,
    pending_fraction: u16,
    warning: u16,
}

impl From<State> for Health {
    fn from(state: State) -> Self {
        Self {
            energy_whole: state.energy.whole.get(),
            energy_fraction: state.energy.fraction,
            pending_whole: state.pending_loss.whole.get(),
            pending_fraction: state.pending_loss.fraction,
            warning: u16::from(state.warning == Warning::On),
        }
    }
}

#[derive(Deserialize)]
struct Maintenance {
    #[serde(flatten)]
    health: Health,
    // The fixed native opportunities qualify a Start and a quiet return. This
    // source evidence is projected to the request boundary only.
    warning_enqueued: bool,
}

#[derive(Deserialize)]
struct Drawing {
    first_energy: u16,
    second_energy: u16,
    previous_energy_before: u16,
    previous_energy_after: u16,
}

#[derive(Deserialize)]
struct Status {
    energy: u16,
    shown_before: u16,
    shown_after: u16,
    gauge_redrawn: bool,
    gauge_words: [u16; GAUGE_WORDS],
}

#[derive(Deserialize)]
pub(in crate::replay) struct Expected {
    pub(super) end_cut: u64,
    protection_at_tail_gate: u8,
    protection_after_impact: u8,
    protection_after_late_tick: u8,
    before_impact: Health,
    after_impact: Health,
    accepted_hit: Option<toml::Table>,
    after_maintenance: Maintenance,
    drawing: Drawing,
    status: Status,
}

impl Expected {
    pub(super) const fn observed_impact(&self) -> bool {
        self.accepted_hit.is_some()
    }

    pub(in crate::replay) fn read(tape: &[Self; 2], cut: u64) -> Result<ExpectedCut, Outcome> {
        let entry = tape
            .iter()
            .find(|entry| entry.end_cut == cut)
            .ok_or_else(|| data_failure(Some(cut), "Health tail comparison cut"))?;
        Ok(project(
            &entry.before_impact,
            entry
                .observed_impact()
                .then_some((&entry.after_impact, entry.protection_after_impact)),
            &entry.after_maintenance.health,
            [
                entry.protection_at_tail_gate,
                entry.protection_after_impact,
                entry.protection_after_impact,
                entry.protection_after_impact,
                entry.protection_after_late_tick,
            ],
            if entry.after_maintenance.warning_enqueued {
                WarningSound::Start
            } else {
                WarningSound::None
            },
            &entry.drawing,
            &entry.status,
        ))
    }
}

fn snapshot(phase: &str, health: &Health, protection: u8) -> [Observation; SNAPSHOT_FIELDS] {
    [
        Observation::quantity(format!("health.{phase}.protection"), i64::from(protection)),
        Observation::quantity(
            format!("health.{phase}.energy.whole"),
            i64::from(health.energy_whole),
        ),
        Observation::quantity(
            format!("health.{phase}.energy.fraction"),
            i64::from(health.energy_fraction),
        ),
        Observation::quantity(
            format!("health.{phase}.pending.whole"),
            i64::from(health.pending_whole),
        ),
        Observation::quantity(
            format!("health.{phase}.pending.fraction"),
            i64::from(health.pending_fraction),
        ),
        Observation::quantity(format!("health.{phase}.warning"), i64::from(health.warning)),
    ]
}

fn project(
    before: &Health,
    impact: Option<(&Health, u8)>,
    maintenance: &Health,
    protection: [u8; 5],
    warning: WarningSound,
    drawing: &Drawing,
    status: &Status,
) -> ExpectedCut {
    let [
        at_actor,
        at_maintenance,
        first_drawing,
        second_drawing,
        at_close,
    ] = protection;
    let mut fields = Vec::from(snapshot("actor", before, at_actor));
    if let Some((health, at_impact)) = impact {
        fields.extend(snapshot("impact", health, at_impact));
    }
    fields.extend(snapshot("maintenance", maintenance, at_maintenance));
    fields.extend([
        Observation::quantity("health.active_close.protection", i64::from(at_close)),
        Observation::quantity("health.drawing.first_protection", i64::from(first_drawing)),
        Observation::quantity(
            "health.drawing.second_protection",
            i64::from(second_drawing),
        ),
        Observation::quantity(
            "appearance.drawing.first_energy",
            i64::from(drawing.first_energy),
        ),
        Observation::quantity(
            "appearance.drawing.previous_energy_before",
            i64::from(drawing.previous_energy_before),
        ),
        Observation::quantity(
            "appearance.drawing.second_energy",
            i64::from(drawing.second_energy),
        ),
        Observation::quantity(
            "appearance.drawing.previous_energy_after",
            i64::from(drawing.previous_energy_after),
        ),
        Observation::quantity("hud.gauge.current_energy", i64::from(status.energy)),
        Observation::quantity("hud.gauge.shown_before", i64::from(status.shown_before)),
        Observation::quantity("hud.gauge.shown_after", i64::from(status.shown_after)),
        Observation::quantity("hud.gauge.redrawn", i64::from(status.gauge_redrawn)),
    ]);
    let semantics = fields.clone();
    let semantic_count = semantics.len();
    fields.push(Observation::quantity(
        "health.warning.request",
        match warning {
            WarningSound::None => 0,
            WarningSound::Start => 1,
            WarningSound::Stop => 2,
        },
    ));
    fields.extend(status.gauge_words.iter().enumerate().map(|(index, word)| {
        Observation::quantity(format!("hud.gauge.word.{index}"), i64::from(*word))
    }));
    ExpectedCut {
        output_count: fields.len().saturating_sub(semantic_count),
        semantic_count,
        semantics,
        fields,
        replies: 0,
    }
}

pub(in crate::replay) fn actual(
    production: &Production,
    outputs: &[Output],
    cut: u64,
) -> Result<Vec<Observation>, Outcome> {
    let (before, impact, downstream) = match outputs {
        [
            Output::HudControls,
            Output::Protection(before),
            Output::Impact(impact),
            downstream @ ..,
        ] if cut == CUTS[0] => (before, Some(impact), downstream),
        [
            Output::HudControls,
            Output::Protection(before),
            downstream @ ..,
        ] if cut == CUTS[1] => (before, None, downstream),
        _ => {
            return Err(data_failure(
                Some(cut),
                "Health tail actor publication order",
            ));
        }
    };
    let [
        Output::HudMap,
        Output::Maintenance(after, warning),
        Output::FirstDrawingRead(first_read, first_protection),
        Output::SecondDrawingRead(second_energy, previous_energy_after, second_protection),
        Output::Status(energy, shown_before, gauge_redrawn),
        Output::ActiveClose(closed),
    ] = downstream
    else {
        return Err(data_failure(
            Some(cut),
            "Health tail downstream publication order",
        ));
    };
    let hud = owner(production)?.hud();
    let hit = impact.map(|health| (Health::from(*health), health.protection.get()));
    Ok(project(
        &(*before).into(),
        hit.as_ref()
            .map(|(health, protection)| (health, *protection)),
        &(*after).into(),
        [
            before.protection.get(),
            after.protection.get(),
            first_protection.get(),
            second_protection.get(),
            closed.get(),
        ],
        *warning,
        &Drawing {
            first_energy: first_read.current,
            second_energy: *second_energy,
            previous_energy_before: first_read.previous,
            previous_energy_after: *previous_energy_after,
        },
        &Status {
            energy: *energy,
            shown_before: *shown_before,
            shown_after: hud.shown.energy,
            gauge_redrawn: *gauge_redrawn,
            gauge_words: gauge_words(hud)?,
        },
    )
    .fields)
}

pub(in crate::replay) fn initial(production: &Production) -> Result<Vec<Observation>, Outcome> {
    let owner = owner(production)?;
    let health = owner.health();
    let hud = owner.hud();
    let unchanged = health.into();
    // The per-cut comparison selects the impact fields only on the delivered cut.
    Ok(project(
        &unchanged,
        Some((&unchanged, health.protection.get())),
        &unchanged,
        [health.protection.get(); 5],
        WarningSound::None,
        &Drawing {
            first_energy: health.energy.whole.get(),
            second_energy: health.energy.whole.get(),
            previous_energy_before: owner.previous_draw_energy(),
            previous_energy_after: owner.previous_draw_energy(),
        },
        &Status {
            energy: health.energy.whole.get(),
            shown_before: hud.shown.energy,
            shown_after: hud.shown.energy,
            gauge_redrawn: false,
            gauge_words: gauge_words(hud)?,
        },
    )
    .fields)
}

pub(super) fn gauge_words(hud: &interface_hud::HudState) -> Result<[u16; GAUGE_WORDS], Outcome> {
    let encoded = tilemap(hud).map_err(|_| data_failure(None, "Health tail HUD tile encoding"))?;
    let mut gauge = [0; GAUGE_WORDS];
    // Coordinate order of the published gauge: bottom tank row, top tank row,
    // then the two energy digits on the bottom HUD status row.
    for (slot, cell) in gauge
        .iter_mut()
        .zip((33_usize..40).chain(1..8).chain(70..72))
    {
        *slot = encoded
            .get(cell.saturating_mul(2)..)
            .and_then(<[u8]>::first_chunk::<2>)
            .copied()
            .map(u16::from_le_bytes)
            .ok_or_else(|| data_failure(None, "Health tail gauge cell"))?;
    }
    Ok(gauge)
}
