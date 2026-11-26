//! B8 requests compared at the actor turn, including turns that interrupt before publication.

use super::{Frame, Overlay, SHAFT, overlay, word};
use crate::replay::{compare::Observation, data_failure, result::Outcome};
use boss_ceres_ridley::{
    AlternatingLook, BeaconStep, Lights, OverlayTurn, RumbleBurst, RumbleParticle,
};
use core::num::Wrapping;
use oracle_record::FrameRecord;
use types::{Pair, Px};

const PROJECTILE_HEADER: usize = 0x1997;
const PROJECTILE_X: usize = 0x1a4b;
const PROJECTILE_Y: usize = 0x1a93;
const PROJECTILE_CLIP: usize = 0x1b47;
const DUST_OR_EXPLOSION: u16 = 0xe509;
const SOUND_LIMIT: u8 = 6;

fn fresh_particle(frame: &Frame<'_>) -> Result<Option<(RumbleParticle, Pair<Px>)>, Outcome> {
    let mut particle = None;
    for slot in (0..36).step_by(2) {
        let header = PROJECTILE_HEADER.saturating_add(slot);
        if word(frame.current, header)? != DUST_OR_EXPLOSION
            || word(frame.previous, header)? == DUST_OR_EXPLOSION
        {
            continue;
        }
        let kind = match word(frame.current, PROJECTILE_CLIP.saturating_add(slot))? {
            0xe1ea | 0xe1ee => RumbleParticle::Smoke,
            0xe138 | 0xe13c => RumbleParticle::SmallExplosion,
            _ => return Err(data_failure(Some(frame.cut), "shaft burst clip")),
        };
        let centre = Pair {
            x: Wrapping(word(frame.current, PROJECTILE_X.saturating_add(slot))?),
            y: Wrapping(word(frame.current, PROJECTILE_Y.saturating_add(slot))?),
        };
        if particle.replace((kind, centre)).is_some() {
            return Err(data_failure(Some(frame.cut), "shaft burst multiplicity"));
        }
    }
    Ok(particle)
}

fn culled_particle(frame: &Frame<'_>) -> Result<(RumbleParticle, Pair<Px>), Outcome> {
    // The tenth burst visits the second point. The native effect player retires it
    // immediately because it is beyond the carried view.
    let x = word(frame.current, 0x0f7a)?.wrapping_sub(2);
    let y = word(frame.current, 0x0f7e)?.wrapping_add(22);
    let left = word(frame.previous, 0x0911)?;
    let top = word(frame.previous, 0x0915)?;
    if x.wrapping_sub(left) < 256 && y.wrapping_sub(top) < 256 {
        return Err(data_failure(
            Some(frame.cut),
            "missing shaft burst inside view",
        ));
    }
    let particle = if word(frame.current, 0x05e5)? < 16_384 {
        RumbleParticle::Smoke
    } else {
        RumbleParticle::SmallExplosion
    };
    Ok((
        particle,
        Pair {
            x: Wrapping(x),
            y: Wrapping(y),
        },
    ))
}

fn burst(
    frame: &Frame<'_>,
    before: Option<Overlay>,
    after: Overlay,
) -> Result<Option<RumbleBurst>, Outcome> {
    let due = matches!(
        (before, after),
        (
            Some(Overlay::Rumble { wait: 0, .. }),
            Overlay::Rumble { wait: 4, .. }
        )
    );
    let (particle, centre) = match (due, fresh_particle(frame)?) {
        (true, Some(spawn)) => spawn,
        (true, None) if frame.place == (SHAFT, 31) => culled_particle(frame)?,
        (false, None) => return Ok(None),
        _ => return Err(data_failure(Some(frame.cut), "shaft burst spawn cut")),
    };
    // The queue-call contract is independent of the particle and its eventual retirement.
    Ok(Some(RumbleBurst {
        particle,
        centre,
        sound_limit: SOUND_LIMIT,
    }))
}

fn lights(before: &FrameRecord) -> Result<Lights, Outcome> {
    let count = word(before, 0x05b6)?;
    let beacon = if word(before, 0xc400)? == 0 {
        BeaconStep::new((count / 8) % 8)
    } else {
        None
    };
    let platform = if (count / 2).is_multiple_of(2) {
        AlternatingLook::First
    } else {
        AlternatingLook::Second
    };
    Ok(Lights {
        beacon,
        platform: Some(platform),
    })
}

fn observations(burst: Option<RumbleBurst>, lights: Option<Lights>) -> [Observation; 3] {
    let field = |name: &str, value: String| Observation {
        field: format!("shaft-actors.{name}"),
        bytes: value.into_bytes(),
    };
    [
        field(
            "burst-particle",
            format!("{:?}", burst.map(|b| (b.particle, b.centre))),
        ),
        field(
            "burst-sound-limit",
            format!("{:?}", burst.map(|b| b.sound_limit)),
        ),
        field("lights", format!("{lights:?}")),
    ]
}

pub(super) fn expected(
    frame: &Frame<'_>,
    placed: bool,
    after: Overlay,
) -> Result<[Observation; 3], Outcome> {
    let before = if placed {
        None
    } else {
        Some(overlay(frame.previous)?)
    };
    let lights_due = matches!(
        (before, after),
        (Some(Overlay::Opening), Overlay::Rumble { .. })
    ) || before == Some(Overlay::Hidden)
        && super::super::finished(frame.previous)
        && word(frame.previous, 0x099c)? == 0xe737;
    let lights = lights_due.then(|| lights(frame.previous)).transpose()?;
    Ok(observations(burst(frame, before, after)?, lights))
}

pub(super) fn actual(turn: Option<OverlayTurn>) -> [Observation; 3] {
    observations(turn.and_then(|t| t.burst), turn.and_then(|t| t.lights))
}

#[test]
#[ignore = "pinned record cache and player ROM"]
fn native_shaft_lights_reach_the_palette_and_platform_on_their_own_cuts() {
    use super::super::{open_rows, read_lock, tests::sources};
    use super::HALL;
    use data::{assets::actors::BeaconColours, rom::Rom};

    // Four low bytes of the Mode 7 map are replaced together; the tile graphics in
    // the same words' upper bytes carry through.
    let platform =
        |record: &FrameRecord| -> [u16; 4] { record.vram[0x060e..=0x0611].try_into().unwrap() };
    let sources = sources();
    let lock = read_lock(&sources).unwrap();
    let rom = Rom::new(std::fs::read(&sources.rom).unwrap()).unwrap();
    let art = import::shaft::import(rom.lorom()).unwrap();
    let beacon = import::actors::import(rom.lorom()).unwrap().beacon;
    let show_beacon = |cut: &FrameRecord, request: Lights| {
        let step = request.beacon.unwrap();
        let observed = 0xc000 + 2 * BeaconColours::FIRST;
        let matches = beacon.sets[usize::from(step.place())]
            .iter()
            .enumerate()
            .all(|(index, colour)| word(cut, observed + 2 * index).ok() == Some(colour.0));
        assert!(matches, "beacon palette at {}", cut.index);
    };
    let show_platform = |before: &FrameRecord, after: &FrameRecord, request: Lights| {
        let asset_index = match request.platform.unwrap() {
            AlternatingLook::First => 0,
            AlternatingLook::Second => 1,
        };
        let words = platform(after);
        let matches = words.map(|word| word.to_le_bytes()[0]) == art.platform[asset_index];
        assert!(matches, "platform tiles at {}", after.index);
        let preserved = words.map(|word| word.to_le_bytes()[1])
            == platform(before).map(|word| word.to_le_bytes()[1]);
        assert!(preserved, "platform high bytes at {}", after.index);
    };

    let mut hall = open_rows(&sources, &lock, HALL).unwrap();
    let previous = hall.record(238).unwrap();
    let actor = hall.record(239).unwrap();
    let published = hall.record(240).unwrap();
    let request = lights(&previous).unwrap();
    show_beacon(&actor, request);
    let retained = platform(&previous) == platform(&actor);
    assert!(retained, "interrupted platform tiles");
    show_platform(&actor, &published, request);

    let mut shaft = open_rows(&sources, &lock, SHAFT).unwrap();
    let hidden = shaft.record(34).unwrap();
    let first = shaft.record(35).unwrap();
    let request = lights(&hidden).unwrap();
    show_beacon(&first, request);
    show_platform(&hidden, &first, request);
}
