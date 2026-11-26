//! Expectations decoded from the two native Ceres flyby streams.

use crate::replay::{compare::Observation, data_failure, result::Outcome};
use oracle_record::FrameRecord;

const PHASE: usize = 0x0fa8;
const READY: u16 = 0xaa11;
const FLYING: u16 = 0xaa4f;
const WARNING: u16 = 0xc04e;

const TILES: [(usize, usize); 8] = [
    (0x504, 2),
    (0x584, 2),
    (0x00b, 4),
    (0x080, 14),
    (0x100, 14),
    (0x181, 12),
    (0x201, 15),
    (0x280, 16),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum State {
    Ready {
        baby: u16,
        wings: u16,
    },
    Flying {
        shown: u16,
        scroll: [i16; 2],
        turn: u16,
        baby: u16,
        wings: u16,
    },
    Warning,
}

pub(super) fn state(record: &FrameRecord) -> Result<Option<State>, Outcome> {
    let read = |at| word(&record.wram, at);
    Ok(match read(PHASE)? {
        READY => Some(State::Ready {
            baby: read(0x802c)?,
            wings: read(0x802e)?,
        }),
        FLYING => Some(State::Flying {
            shown: read(0x8026)? / 2,
            scroll: [signed(read(0x8028)?), signed(read(0x802a)?)],
            turn: read(0x7814)?,
            baby: read(0x802c)?,
            wings: read(0x802e)?,
        }),
        WARNING => Some(State::Warning),
        _ => None,
    })
}

pub(in crate::replay::played) fn expected(
    previous: &FrameRecord,
    current: &FrameRecord,
) -> Result<Vec<Observation>, Outcome> {
    let own = state(current)?;
    if !matches!(own, Some(State::Flying { .. } | State::Warning)) {
        return Ok(Vec::new());
    }
    let read = |at| word(&current.wram, at);
    let active = matches!(own, Some(State::Flying { .. }));
    let mut fields = Vec::new();
    if let Some(own) = own {
        fields.extend(owned(own));
    }
    fields.extend(geometry(
        u16::from(byte(&current.wram, 0x56)?),
        u16::from(byte(&current.wram, 0x5f)?),
        [signed(read(0xb1)?), signed(read(0xb3)?)],
        [
            signed(read(0x78)?),
            signed(read(0x7a)?),
            signed(read(0x7c)?),
            signed(read(0x7e)?),
            signed(read(0x80)?),
            signed(read(0x82)?),
        ],
    ));
    fields.extend(physical(
        current.ppu.bgmode,
        current.ppu.m7sel,
        current.ppu.bg_scroll.first().copied().unwrap_or_default(),
        current.ppu.m7,
    ));
    if active {
        fields.push(Observation::quantity(
            "ridley.flyby.zoom",
            i64::from(read(0x8020)?),
        ));
        let colours = (0..15_u16)
            .map(|index| read(0xc0a2_usize.saturating_add(usize::from(index.saturating_mul(2)))))
            .collect::<Result<Vec<_>, _>>()?;
        let cgram = current
            .cgram
            .get(81..96)
            .ok_or_else(|| data_failure(None, "Ridley flyby CGRAM"))?;
        fields.push(words("ridley.flyby.body.current", colours));
        fields.push(words("ridley.flyby.body.cgram", cgram.iter().copied()));
    }
    let armed = read(0x0a5a)? == 0xe12e && word(&previous.wram, 0x0a5a)? != 0xe12e;
    fields.push(Observation::quantity("ridley.flyby.push", i64::from(armed)));
    fields.extend(tile_fields(&current.vram)?);
    let before = byte(&previous.wram, 0x0647)?;
    let after = byte(&current.wram, 0x0647)?;
    let appended = before != after
        && byte(
            &current.wram,
            0x0666_usize.saturating_add(usize::from(before)),
        )? == 0x4e;
    fields.push(Observation::quantity(
        "ridley.flyby.sound",
        i64::from(appended),
    ));
    Ok(fields)
}

pub(super) fn owned(state: State) -> Vec<Observation> {
    let (phase, shown, scroll, turn, baby, wings) = match state {
        State::Ready { baby, wings } => (0, 0, [0, 0], 0, baby, wings),
        State::Flying {
            shown,
            scroll,
            turn,
            baby,
            wings,
        } => (1, shown, scroll, turn, baby, wings),
        State::Warning => (2, 0, [0, 0], 0, 0, 0),
    };
    vec![
        Observation::quantity("ridley.flyby.phase", phase),
        Observation::quantity("ridley.flyby.shown", i64::from(shown)),
        words("ridley.flyby.position", scroll.map(bits)),
        Observation::quantity("ridley.flyby.turn", i64::from(turn)),
        Observation::quantity("ridley.flyby.baby", i64::from(baby)),
        Observation::quantity("ridley.flyby.wings", i64::from(wings)),
    ]
}

pub(super) fn geometry(
    plane: u16,
    outside: u16,
    scroll: [i16; 2],
    matrix_and_pivot: [i16; 6],
) -> [Observation; 4] {
    [
        Observation::quantity("ridley.flyby.plane", i64::from(plane)),
        Observation::quantity("ridley.flyby.overflow", i64::from(outside)),
        words("ridley.flyby.scroll", scroll.map(bits)),
        words("ridley.flyby.matrix-and-pivot", matrix_and_pivot.map(bits)),
    ]
}

pub(super) fn physical(
    mode: u8,
    outside: u8,
    scroll: [u16; 2],
    matrix_and_pivot: [i16; 6],
) -> [Observation; 4] {
    [
        Observation::quantity("ridley.flyby.ppu.mode", i64::from(mode)),
        Observation::quantity("ridley.flyby.ppu.overflow", i64::from(outside)),
        words("ridley.flyby.ppu.scroll", scroll),
        words(
            "ridley.flyby.ppu.matrix-and-pivot",
            matrix_and_pivot.map(bits),
        ),
    ]
}

pub(super) fn tile_fields(vram: &[u16]) -> Result<Vec<Observation>, Outcome> {
    TILES
        .iter()
        .enumerate()
        .map(|(index, &(start, count))| {
            let span = vram
                .get(start..start.saturating_add(count))
                .ok_or_else(|| data_failure(None, "Ridley flyby VRAM"))?;
            Ok(Observation {
                field: format!("ridley.flyby.tiles.{index}"),
                bytes: span.iter().map(|word| word.to_le_bytes()[0]).collect(),
            })
        })
        .collect()
}

pub(super) fn words(field: &str, values: impl IntoIterator<Item = u16>) -> Observation {
    Observation {
        field: field.into(),
        bytes: values.into_iter().flat_map(u16::to_le_bytes).collect(),
    }
}

const fn signed(word: u16) -> i16 {
    i16::from_le_bytes(word.to_le_bytes())
}

const fn bits(word: i16) -> u16 {
    u16::from_le_bytes(word.to_le_bytes())
}

fn word(wram: &[u8], at: usize) -> Result<u16, Outcome> {
    let [low, high] = wram
        .get(at..at.saturating_add(2))
        .ok_or_else(|| data_failure(None, "Ridley flyby observation"))?
    else {
        return Err(data_failure(None, "Ridley flyby observation"));
    };
    Ok(u16::from_le_bytes([*low, *high]))
}

fn byte(wram: &[u8], at: usize) -> Result<u8, Outcome> {
    wram.get(at)
        .copied()
        .ok_or_else(|| data_failure(None, "Ridley flyby observation"))
}
