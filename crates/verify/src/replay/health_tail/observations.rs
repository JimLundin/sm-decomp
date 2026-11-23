//! Independent observations and the actual carried owners and outputs at each frame cut.

use game::production::{Production, health_tail::Output, hud::tilemap};
use samus_health::{State, Warning, WarningSound};
use serde::Deserialize;

use super::owner;
use crate::replay::{compare::Observation, data_failure, readers::ExpectedCut, result::Outcome};

const GAUGE_WORDS: usize = 16;
// Health's complete State, Drawing's energy history, and the gauge's shown energy and redraw.
const FIELDS_PER_CUT: usize = 9;
// The Warning request, the protection Drawing reads and all ordered CPU gauge words.
pub(in crate::replay) const OUTPUTS_PER_CUT: usize = 2 + GAUGE_WORDS;

pub(in crate::replay) const fn field_count(cuts: usize) -> Option<usize> {
    cuts.checked_mul(FIELDS_PER_CUT)
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
    // The fixed native frames qualify a Start and a quiet return. This source evidence is
    // projected to the request only.
    warning_enqueued: bool,
}

#[derive(Deserialize)]
struct Drawing {
    previous_energy_after: u16,
}

#[derive(Deserialize)]
struct Status {
    shown_after: u16,
    gauge_redrawn: bool,
    gauge_words: [u16; GAUGE_WORDS],
}

/// One frame's observations. Health does not change after its Maintenance, so Health at the
/// cut is Health after Maintenance with the protection the frame leaves.
#[derive(Deserialize)]
pub(in crate::replay) struct Expected {
    pub(super) end_cut: u64,
    protection_after_impact: u8,
    protection_after_late_tick: u8,
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
        Ok(project(&Cut {
            health: &entry.after_maintenance.health,
            protection: entry.protection_after_late_tick,
            protection_drawn: entry.protection_after_impact,
            drawn_energy: entry.drawing.previous_energy_after,
            shown_energy: entry.status.shown_after,
            gauge_redrawn: entry.status.gauge_redrawn,
            warning: if entry.after_maintenance.warning_enqueued {
                WarningSound::Start
            } else {
                WarningSound::None
            },
            gauge_words: entry.status.gauge_words,
        }))
    }
}

struct Cut<'a> {
    health: &'a Health,
    protection: u8,
    protection_drawn: u8,
    drawn_energy: u16,
    shown_energy: u16,
    gauge_redrawn: bool,
    warning: WarningSound,
    gauge_words: [u16; GAUGE_WORDS],
}

fn project(cut: &Cut<'_>) -> ExpectedCut {
    let health = cut.health;
    let semantics = vec![
        Observation::quantity("health.protection", i64::from(cut.protection)),
        Observation::quantity("health.energy.whole", i64::from(health.energy_whole)),
        Observation::quantity("health.energy.fraction", i64::from(health.energy_fraction)),
        Observation::quantity("health.pending.whole", i64::from(health.pending_whole)),
        Observation::quantity(
            "health.pending.fraction",
            i64::from(health.pending_fraction),
        ),
        Observation::quantity("health.warning", i64::from(health.warning)),
        Observation::quantity("appearance.drawing.energy", i64::from(cut.drawn_energy)),
        Observation::quantity("hud.gauge.shown", i64::from(cut.shown_energy)),
        Observation::quantity("hud.gauge.redrawn", i64::from(cut.gauge_redrawn)),
    ];
    let semantic_count = semantics.len();
    let mut fields = semantics.clone();
    fields.push(Observation::quantity(
        "health.warning.request",
        match cut.warning {
            WarningSound::None => 0,
            WarningSound::Start => 1,
            WarningSound::Stop => 2,
        },
    ));
    fields.push(Observation::quantity(
        "health.protection_drawn",
        i64::from(cut.protection_drawn),
    ));
    fields.extend(cut.gauge_words.iter().enumerate().map(|(index, word)| {
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
    output: Output,
) -> Result<Vec<Observation>, Outcome> {
    let owner = owner(production)?;
    let health = owner.health();
    let hud = owner.hud();
    Ok(project(&Cut {
        health: &health.into(),
        protection: health.protection.get(),
        protection_drawn: output.protection_drawn.get(),
        drawn_energy: owner.drawn_energy(),
        shown_energy: hud.shown.energy,
        gauge_redrawn: output.gauge_redrawn,
        warning: output.warning_sound,
        gauge_words: gauge_words(hud)?,
    })
    .fields)
}

pub(in crate::replay) fn initial(production: &Production) -> Result<Vec<Observation>, Outcome> {
    let protection_drawn = owner(production)?.health().protection;
    actual(
        production,
        Output {
            warning_sound: WarningSound::None,
            protection_drawn,
            gauge_redrawn: false,
        },
    )
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
