//! Contact receipt as emitted by an enemy and as reflected in Samus's reaction.

use crate::{
    replay::{compare::Observation, result::Outcome},
    source,
};
use types::{Hit, Side};

pub(super) fn expected(
    field: &str,
    previous: &[u8],
    current: &[u8],
) -> Result<Observation, Outcome> {
    let protection = source::SAMUS_HIT_PROTECTION;
    let before = super::word(previous, protection)?;
    let after = super::word(current, protection)?;
    let receipt = if before == 0 && after == 95 {
        let side = super::word(current, source::SAMUS_KNOCKBACK_X_DIRECTION)?;
        let push_left = super::word(current, source::SAMUS_KNOCKBACK_TIMER)?;
        [1, i64::from(side), i64::from(push_left).saturating_add(1)]
    } else {
        [0; 3]
    };
    Ok(impact(field, receipt))
}

/// The emitted side and push. The health observations separately compare the energy and
/// protection the impact actually leaves.
pub(super) fn actual(field: &str, hit: Option<Hit>) -> Observation {
    impact(
        field,
        hit.map_or([0; 3], |hit| {
            [
                1,
                i64::from(hit.side == Side::Right),
                i64::from(hit.push.frames()),
            ]
        }),
    )
}

fn impact(field: &str, receipt: [i64; 3]) -> Observation {
    Observation {
        field: field.into(),
        bytes: receipt.into_iter().flat_map(i64::to_le_bytes).collect(),
    }
}
