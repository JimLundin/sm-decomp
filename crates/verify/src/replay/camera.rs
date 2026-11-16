//! Independent camera starts and a bounded unchanged upstream movement tape.
//! Inputs come from the prior cut; expected quantities come only from the current cut.
use super::{compare::Observation, data_failure, result::Outcome};
use core::num::Wrapping;
use types::{Facing, Pair, Parts, ScreenKind, VerticalFraction, fixed};
use world_camera::{
    Camera, Frame, Heights, LeadSetting, RememberedPosition, Room, Samus, ViewPosition,
};

pub struct Content;

fn word(wram: &[u8], at: usize) -> Result<u16, Outcome> {
    let bytes = wram
        .get(at..at.saturating_add(2))
        .ok_or_else(|| data_failure(None, "camera record word"))?;
    Ok(u16::from_le_bytes(
        bytes
            .try_into()
            .map_err(|_| data_failure(None, "camera record word"))?,
    ))
}
fn unsupported(dependency: &str) -> Outcome {
    Outcome::Unsupported {
        cut: None,
        dependency: dependency.to_owned(),
    }
}
fn position(wram: &[u8], x: usize, y: usize) -> Result<types::Position, Outcome> {
    Ok(Pair {
        x: fixed(Wrapping(word(wram, x)?), word(wram, x.saturating_add(2))?),
        y: fixed(Wrapping(word(wram, y)?), word(wram, y.saturating_add(2))?),
    })
}

pub fn seed(wram: &[u8], _content: &Content) -> Result<Camera, Outcome> {
    Ok(Camera::restore(
        ViewPosition(Pair {
            x: fixed(Wrapping(word(wram, 0x0911)?), word(wram, 0x090f)?),
            y: fixed(Wrapping(word(wram, 0x0915)?), word(wram, 0x0913)?),
        }),
        RememberedPosition(position(wram, 0x0b10, 0x0b14)?),
        VerticalFraction(word(wram, 0x0da8)?),
    ))
}

pub struct OwnedInput {
    room: Room,
    samus: Samus,
    lead: LeadSetting,
    time_stopped: bool,
}
impl OwnedInput {
    pub const fn frame(&self) -> Frame<'_> {
        Frame {
            room: &self.room,
            samus: self.samus,
            lead: self.lead,
            time_stopped: self.time_stopped,
        }
    }
}

/// Borrowed upstream facts certify only the indexed unchanged producer interval.
/// They do not establish movement, placement, scroll production or downstream pixels.
/// The case's call metadata must separately establish one complete ordinary follow.
pub fn validate_step(prior: &[u8], current: &[u8]) -> Result<(), Outcome> {
    for record in [prior, current] {
        if word(record, 0x0998)? != 8
            || word(record, 0x0cf8)? != 0
            || word(record, 0x07e9)? != 0
            || matches!(word(record, 0x0a1e)?.to_le_bytes()[1], 0x16 | 0x1a | 0x1b)
        {
            return Err(unsupported("camera complete ordinary follow producer"));
        }
    }
    if word(current, 0x05b6)?.wrapping_sub(word(prior, 0x05b6)?) != 1 {
        return Err(unsupported("camera exactly one gameplay tick"));
    }
    // Camera-owned view, remembered centre and receipt are deliberately absent.
    for at in [
        0x079b, 0x0af6, 0x0af8, 0x0afa, 0x0afc, 0x0a1c, 0x0a1e, 0x0a52, 0x0b4a, 0x0b36, 0x0941,
        0x0a78, 0x07a9, 0x07ab, 0x07ad, 0x07af,
    ] {
        if word(prior, at)? != word(current, at)? {
            return Err(unsupported(
                "camera unchanged borrowed movement and room producers",
            ));
        }
    }
    if screens(prior)? != screens(current)? {
        return Err(unsupported("camera unchanged borrowed scroll producer"));
    }
    Ok(())
}
fn screens(wram: &[u8]) -> Result<&[u8], Outcome> {
    wram.get(0xcd20..0xcd52)
        .ok_or_else(|| data_failure(None, "camera scroll run"))
}

pub fn input(prior: &[u8], _content: &Content) -> Result<OwnedInput, Outcome> {
    let facing = match word(prior, 0x0a1e)?.to_le_bytes()[0] {
        0 => Facing::Screen,
        4 => Facing::Left,
        8 => Facing::Right,
        _ => return Err(data_failure(None, "camera facing observation")),
    };
    let lead = match word(prior, 0x0941)? {
        0 => LeadSetting::Ahead,
        2 => LeadSetting::Near,
        4 => LeadSetting::KeepLeft,
        6 => LeadSetting::KeepRight,
        _ => return Err(data_failure(None, "camera lead observation")),
    };
    let room = Room::new(
        screens(prior)?
            .iter()
            .map(|value| match value {
                0 => ScreenKind::Closed,
                1 => ScreenKind::Short,
                _ => ScreenKind::Open,
            })
            .collect(),
        Pair {
            x: usize::from(word(prior, 0x07a9)?),
            y: usize::from(word(prior, 0x07ab)?),
        },
        Heights {
            while_not_rising: Wrapping(word(prior, 0x07ad)?),
            while_rising: Wrapping(word(prior, 0x07af)?),
        },
    )
    .map_err(|_| data_failure(None, "camera room relation"))?;
    Ok(OwnedInput {
        room,
        samus: Samus {
            position: position(prior, 0x0af6, 0x0afa)?,
            placed: None,
            facing,
            backwards: word(prior, 0x0a52)? != 0
                || word(prior, 0x0a1e)?.to_le_bytes()[1] == 0x10
                || word(prior, 0x0b4a)? == 1,
            rising: word(prior, 0x0b36)? == 1,
        },
        lead,
        time_stopped: word(prior, 0x0a78)? != 0,
    })
}

pub fn expected(wram: &[u8]) -> Result<Vec<Observation>, Outcome> {
    let fields = [
        ("camera.view.x.pixel", 0x0911),
        ("camera.view.x.fraction", 0x090f),
        ("camera.view.y.pixel", 0x0915),
        ("camera.view.y.fraction", 0x0913),
        ("camera.remembered.x.pixel", 0x0b10),
        ("camera.remembered.x.fraction", 0x0b12),
        ("camera.remembered.y.pixel", 0x0b14),
        ("camera.remembered.y.fraction", 0x0b16),
        ("camera.vertical-fraction", 0x0da8),
    ];
    fields
        .into_iter()
        .map(|(field, at)| Ok(Observation::quantity(field, i64::from(word(wram, at)?))))
        .collect()
}
pub fn actual(state: &Camera, _outputs: &Camera) -> Vec<Observation> {
    initial(state)
}
pub fn initial(state: &Camera) -> Vec<Observation> {
    let view = state.position();
    let remembered = state.seen();
    [
        ("camera.view.x.pixel", view.x.pixel().0),
        ("camera.view.x.fraction", view.x.sub()),
        ("camera.view.y.pixel", view.y.pixel().0),
        ("camera.view.y.fraction", view.y.sub()),
        ("camera.remembered.x.pixel", remembered.x.pixel().0),
        ("camera.remembered.x.fraction", remembered.x.sub()),
        ("camera.remembered.y.pixel", remembered.y.pixel().0),
        ("camera.remembered.y.fraction", remembered.y.sub()),
        ("camera.vertical-fraction", state.vertical_fraction().0),
    ]
    .into_iter()
    .map(|(field, value)| Observation::quantity(field, i64::from(value)))
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn write(wram: &mut [u8], at: usize, value: u16) {
        wram[at..at.saturating_add(2)].copy_from_slice(&value.to_le_bytes());
    }
    #[test]
    fn seed_keeps_distinct_view_and_remembered_fractions() {
        let mut wram = vec![0; 0x20000];
        for (at, value) in [
            (0x090f, 11),
            (0x0913, 13),
            (0x0b12, 17),
            (0x0b16, 19),
            (0x0da8, 23),
        ] {
            write(&mut wram, at, value);
        }
        let camera = seed(&wram, &Content).unwrap();
        assert_eq!(initial(&camera), expected(&wram).unwrap());
        write(&mut wram, 0x0913, 29);
        let observed = expected(&wram).unwrap();
        assert!(observed.contains(&Observation::quantity("camera.view.y.fraction", 29)));
        assert!(!initial(&camera).contains(&Observation::quantity("camera.view.y.fraction", 29)));
    }
    #[test]
    fn changing_borrowed_producer_rejects_but_camera_outputs_do_not_admit_inputs() {
        let mut prior = vec![0; 0x20000];
        write(&mut prior, 0x0998, 8);
        let mut current = prior.clone();
        write(&mut current, 0x05b6, 1);
        write(&mut current, 0x0911, 37);
        write(&mut current, 0x0b14, 41);
        assert!(validate_step(&prior, &current).is_ok());
        write(&mut current, 0x0af6, 43);
        assert!(matches!(
            validate_step(&prior, &current),
            Err(Outcome::Unsupported { .. })
        ));
        assert!(matches!(expected(&[]), Err(Outcome::DataFailure { .. })));
    }
}
