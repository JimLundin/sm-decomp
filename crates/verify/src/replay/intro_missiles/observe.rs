//! Projections for the scoped weapon State and deliveries.

use super::data_failure;
use crate::replay::weapons::word;
use crate::replay::{compare::Observation, result::Outcome};
use game::{
    exact::oam::HIDDEN_Y,
    production::{
        intro_missile_scope::{Output, State},
        played::intro_missiles::slot_id,
    },
    state::video::Oam,
};
use weapons_projectiles as w1;

pub(super) const STOCK: &str = "weapons.intro-stock";
pub(super) const CLOCK: &str = "weapons.intro-spacing";
pub(super) const SLOTS: &str = "weapons.intro-slots";
pub(super) const EXHAUST: &str = "weapons.intro-exhaust";
pub(super) const DEBIT: &str = "weapons.intro-release";
pub(super) const IMPACT: &str = "weapons.intro-impact";
pub(super) const PUFFS: &str = "weapons.intro-puffs";
pub(super) const CANDIDATES: &str = "weapons.intro-candidates";
pub(super) const WORLD: &str = "weapons.intro-point-reads";
pub(super) const OAM: &str = "weapons.intro-oam";
pub(super) const SOUND: &str = "weapons.intro-sound";

pub(super) fn field(name: &str, values: impl IntoIterator<Item = u16>) -> Observation {
    Observation {
        field: name.to_owned(),
        bytes: values.into_iter().flat_map(u16::to_le_bytes).collect(),
    }
}

fn id(picture: Option<&w1::PictureId>) -> u16 {
    picture
        .and_then(|picture| picture.0.parse::<u16>().ok())
        .unwrap_or(u16::MAX)
}

pub(super) fn actual_state(state: &State) -> Vec<Observation> {
    weapon_state(&state.weapons.0, state.supplies)
}

pub(super) fn weapon_state(
    weapons: &w1::State,
    supplies: game::production::played::intro_missiles::Supplies,
) -> Vec<Observation> {
    use w1::ShotState::{Flying, Free, Impact};
    let w1::State {
        count,
        cooldown,
        slots,
        exhaust,
    } = weapons;
    let slots = slots.values().flat_map(|shot| match shot {
        Free {
            trail,
            retained_fraction,
        } => {
            vec![0, trail.0, retained_fraction.x.0, retained_fraction.y.0]
        }
        Flying { body, consumed } => parts(body, 1, *consumed),
        Impact { body } => parts(body, 2, false),
    });
    let exhaust = exhaust.iter().flat_map(|exhaust| match exhaust {
        w1::ExhaustSlot::Free => vec![0],
        w1::ExhaustSlot::Puff(puff) => vec![
            1,
            puff.position.x,
            puff.position.y,
            index(puff.next_frame.index()),
            puff.remaining_hold.0,
            id(puff.picture.as_ref()),
        ],
    });
    vec![
        field(
            STOCK,
            [
                supplies.ammo,
                u16::from(supplies.selected),
                u16::from(supplies.cancel_after_shot),
                supplies.protection,
            ],
        ),
        field(CLOCK, [cooldown.0, count.0]),
        field(SLOTS, slots),
        field(EXHAUST, exhaust),
    ]
}

fn index(value: usize) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
}

fn parts(body: &w1::ShotBody, phase: u16, consumed: bool) -> Vec<u16> {
    let axes = body.heading.axes();
    let kind = match body.kind {
        w1::ShotKind::Beam => 0,
        w1::ShotKind::Missile(w1::MissileLaunch::Pending) => 1,
        w1::ShotKind::Missile(w1::MissileLaunch::Launched) => 2,
    };
    vec![
        phase,
        body.trail.0,
        body.position.x.fraction().0,
        body.position.y.fraction().0,
        body.position.x.integer(),
        body.position.y.integer(),
        body.speed.x.0.cast_unsigned(),
        body.speed.y.0.cast_unsigned(),
        axes.x.cast_unsigned(),
        axes.y.cast_unsigned(),
        u16::from(body.radius.x.0),
        u16::from(body.radius.y.0),
        body.damage.value(),
        body.remaining_hold.0,
        index(body.next_frame.index()),
        id(body.picture.as_ref()),
        kind,
        u16::from(consumed),
        u16::from(body.trail_appearance.is_some()),
    ]
}

pub(super) fn actual_output(output: Option<&Output>) -> Vec<Observation> {
    let turn = output.and_then(|out| out.turn.as_ref());
    let events = turn.into_iter().flat_map(|turn| &turn.events);
    let debit = u16::try_from(
        events
            .clone()
            .filter(|event| **event == w1::Event::MissileDebit)
            .count(),
    )
    .unwrap_or(u16::MAX);
    let sound = u16::try_from(
        events
            .clone()
            .filter(|event| matches!(event, w1::Event::FireSound(_) | w1::Event::ImpactSound(_)))
            .count(),
    )
    .unwrap_or(u16::MAX);
    let puffs = events
        .filter_map(|event| match event {
            w1::Event::PuffRequest { slot, centre, .. } => {
                Some([u16::from(slot_id(*slot)), centre.x, centre.y])
            }
            w1::Event::MissileDebit
            | w1::Event::ClearSelection
            | w1::Event::Protection(_)
            | w1::Event::PendingAimConsumed
            | w1::Event::Trail { .. }
            | w1::Event::FireSound(_)
            | w1::Event::ImpactSound(_) => None,
        })
        .flatten();
    vec![
        field(DEBIT, [debit]),
        field(
            IMPACT,
            output
                .and_then(|out| out.impact)
                .map(|slot| u16::from(slot_id(slot))),
        ),
        field(PUFFS, puffs),
        field(
            CANDIDATES,
            turn.into_iter()
                .flat_map(|turn| turn.candidates.iter())
                .flat_map(|shot| [u16::from(slot_id(shot.slot)), shot.whole_x]),
        ),
        field(WORLD, [index(output.map_or(0, |out| out.point_contacts))]),
        field(SOUND, [sound]),
        field(
            OAM,
            output
                .and_then(|out| out.sprites.as_ref())
                .into_iter()
                .flat_map(actual_oam)
                .flatten(),
        ),
    ]
}

pub(super) fn pieces(low: impl IntoIterator<Item = [u8; 4]>, high: &[u8]) -> Vec<[u16; 4]> {
    low.into_iter()
        .enumerate()
        .filter_map(|(i, [x, y, tile, attrs])| {
            if y == HIDDEN_Y {
                return None;
            }
            let bits = high.get(i / 4)? >> ((i % 4).saturating_mul(2));
            Some([
                (u16::from(bits & 1) << 8) | u16::from(x),
                u16::from(y),
                u16::from_le_bytes([tile, attrs]),
                u16::from((bits >> 1) & 1),
            ])
        })
        .collect()
}

pub(super) fn actual_oam(oam: &Oam) -> Vec<[u16; 4]> {
    pieces(
        oam.low
            .iter()
            .map(|piece| [piece.x, piece.y, piece.tile, piece.attributes]),
        &oam.high,
    )
}

pub(super) fn expected_events(
    prior: Option<&[u8]>,
    current: &[u8],
    cut: u64,
) -> Result<Vec<Observation>, Outcome> {
    let mut debit = 0;
    let mut impact = Vec::new();
    let mut puffs = Vec::new();
    let mut candidates = Vec::new();
    if let Some(prior) = prior {
        debit = word(prior, 0x09c6)?
            .checked_sub(word(current, 0x09c6)?)
            .ok_or_else(|| data_failure(Some(cut), "missile stock transition"))?;
        for id in (0..5_u16).rev() {
            let offset = usize::from(id).saturating_mul(2);
            let before = |base: usize| word(prior, base.saturating_add(offset));
            let after = |base: usize| word(current, base.saturating_add(offset));
            let hit = before(0x0c18)? != 0x8800 && after(0x0c18)? == 0x8800;
            if hit {
                impact.push(id);
            }
            // Each flight update advances the trail countdown, including a shot that
            // becomes an impact after flight. A held slot produces no candidate or puff.
            let flew = hit
                || (after(0x0c18)? == 0x8100
                    && (before(0x0c18)? != 0x8100 || before(0x0c90)? != after(0x0c90)?));
            if flew && before(0x0c18)? == 0x8100 && before(0x0c90)? == 1 {
                puffs.extend([id, before(0x0b64)?, before(0x0b78)?]);
            }
            if flew {
                candidates.extend([id, after(0x0b64)?]);
            }
        }
    }
    Ok(vec![
        field(DEBIT, [debit]),
        field(IMPACT, impact),
        field(PUFFS, puffs),
        field(CANDIDATES, candidates),
        field(WORLD, [0]),
        field(SOUND, [0]),
    ])
}

#[test]
fn recorded_flight_and_stock_changes_report_events_even_with_stationary_scene_clocks() {
    let mut before = vec![0; 0x2000];
    let slot = 2;
    let set = |ram: &mut [u8], address: usize, value: u16| {
        ram[address..address + 2].copy_from_slice(&value.to_le_bytes());
    };
    set(&mut before, 0x09c6, 4);
    set(&mut before, 0x0c18 + slot, 0x8100);
    set(&mut before, 0x0c90 + slot, 1);
    set(&mut before, 0x0b64 + slot, 50);
    set(&mut before, 0x0b78 + slot, 30);
    let mut after = before.clone();
    set(&mut after, 0x09c6, 3);
    set(&mut after, 0x0c90 + slot, 4);
    set(&mut after, 0x0b64 + slot, 48);
    let moving = expected_events(Some(&before), &after, 1).unwrap();
    assert!(moving.contains(&field(PUFFS, [1, 50, 30])));
    assert!(moving.contains(&field(CANDIDATES, [1, 48])));
    assert!(moving.contains(&field(DEBIT, [1])));
    let held = expected_events(Some(&after), &after, 2).unwrap();
    assert!(held.contains(&field(PUFFS, [])));
    assert!(held.contains(&field(CANDIDATES, [])));
    assert!(held.contains(&field(DEBIT, [0])));
}
