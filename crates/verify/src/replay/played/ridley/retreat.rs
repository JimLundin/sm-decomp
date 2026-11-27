//! Native observations for the retreat's move and the backdrop's publication.

use crate::replay::compare::Observation;
use crate::replay::result::Outcome;
use crate::replay::{ceres, data_failure};
use boss_ceres_ridley::{Bound, CeresDoor, DoorKind, Phase, RidleyPhase, RidleyRun, Stage};
use game::production::played;
use oracle_record::FrameRecord;

/// Current palette spans, in colour words. Colour zero of each row stays transparent.
const PALETTES: [(&str, &str, std::ops::Range<usize>); 3] = [
    ("ridley.backdrop.body", "ridley.backdrop.cgram.body", 81..96),
    (
        "ridley.backdrop.bg-walls",
        "ridley.backdrop.cgram.bg-walls",
        33..41,
    ),
    (
        "ridley.backdrop.sprite-walls",
        "ridley.backdrop.cgram.sprite-walls",
        241..249,
    ),
];

fn word(wram: &[u8], at: usize) -> Result<u16, Outcome> {
    wram.as_chunks::<2>()
        .0
        .get(at / 2)
        .copied()
        .map(u16::from_le_bytes)
        .ok_or_else(|| data_failure(None, "Ridley retreat observation"))
}

const fn retreat_phase(phase: RidleyPhase) -> bool {
    matches!(
        phase,
        RidleyPhase::Retreating
            | RidleyPhase::CoastingAbove(_)
            | RidleyPhase::Getaway(types::ceres_getaway::GetawayState::GetawayReady(_))
    )
}

pub(in crate::replay::played) fn expected(
    record: &FrameRecord,
) -> Result<Vec<Observation>, Outcome> {
    let wram = &record.wram;
    let phase = word(wram, 0x0fa8)?;
    if !matches!(phase, 0xa971 | 0xa9a0 | 0xaa11) || word(wram, 0x0f78)? != 0xe13f {
        return Ok(Vec::new());
    }
    let walls = ceres::doors(wram)?
        .into_iter()
        .map(|slot| ceres::seed(wram, slot))
        .collect::<Result<Vec<_>, _>>()?;
    let mut fields = state(word(wram, 0x093f)?, word(wram, 0x2000)?);
    fields.extend(wall_fields(&walls));
    fields.push(Observation::quantity(
        "ridley.retreat.bound",
        i64::from(word(wram, 0x783e)?),
    ));
    for (current, physical, range) in PALETTES {
        let (mut colours, mut cgram) = (Vec::new(), Vec::new());
        if phase == 0xaa11 {
            for index in range {
                let colour = word(wram, 0xc000_usize.saturating_add(index.saturating_mul(2)))?;
                colours.extend(colour.to_le_bytes());
                cgram.extend(
                    record
                        .cgram
                        .get(index)
                        .ok_or_else(|| data_failure(None, "Ridley backdrop CGRAM"))?
                        .to_le_bytes(),
                );
            }
        }
        fields.push(Observation {
            field: current.into(),
            bytes: colours,
        });
        fields.push(Observation {
            field: physical.into(),
            bytes: cgram,
        });
    }
    Ok(fields)
}

/// With no new turn the last backdrop stays visible, and no bound is hit.
pub(in crate::replay::played) fn actual(
    own: &played::State,
    run: Option<&RidleyRun>,
) -> Vec<Observation> {
    if run.is_some_and(|run| !retreat_phase(run.ridley.phase)) {
        return Vec::new();
    }
    let stage = match own.station.stage {
        Stage::Calm => 0,
        Stage::Getaway => 1,
        Stage::Escape => 2,
    };
    let backdrop = run.map_or_else(
        || {
            own.station
                .ridley
                .is_some_and(|ridley| matches!(ridley.phase, RidleyPhase::Getaway(_)))
        },
        |run| run.backdrop.is_some(),
    );
    let mut fields = state(stage, u16::from(!backdrop));
    fields.extend(wall_fields(&own.station.doors));
    fields.push(Observation::quantity(
        "ridley.retreat.bound",
        match run.and_then(|run| run.bound) {
            None => 0,
            Some(Bound::Left) => 1,
            Some(Bound::Right) => 2,
            Some(Bound::Top) => 4,
            Some(Bound::Bottom) => 8,
        },
    ));
    for (current, physical, range) in PALETTES {
        let bytes = if backdrop {
            own.palette
                .screen
                .get(range)
                .unwrap_or_default()
                .iter()
                .flat_map(|colour| colour.bits().to_le_bytes())
                .collect()
        } else {
            Vec::new()
        };
        // Compare drawing's palette source separately with the physical span at the native cut.
        fields.push(Observation {
            field: physical.into(),
            bytes: bytes.clone(),
        });
        fields.push(Observation {
            field: current.into(),
            bytes,
        });
    }
    fields
}

fn state(stage: u16, swings: u16) -> Vec<Observation> {
    vec![
        Observation::quantity("ridley.retreat.station-stage", i64::from(stage)),
        Observation::quantity("ridley.retreat.tail-active", i64::from(swings)),
    ]
}

fn wall_fields(doors: &[CeresDoor]) -> [Observation; 2] {
    [
        ("ridley.backdrop.left-wall", types::Side::Left),
        ("ridley.backdrop.right-wall", types::Side::Right),
    ]
    .map(|(field, side)| {
        let position = doors
            .iter()
            .find(|door| {
                door.kind == DoorKind::GetawayWall
                    && door.facing == side
                    && door.phase == Phase::Placed
            })
            .map_or(-1, |door| {
                (i64::from(door.position.x.0) << 16) | i64::from(door.position.y.0)
            });
        Observation::quantity(field, position)
    })
}
