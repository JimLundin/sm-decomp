//! Static shaft setup, read independently from the passage, the room grid and the display.

use super::{compare::Observation, data_failure, result::Outcome};
use crate::source;
use data::level::Block;
use game::{
    production::shaft_entry::{Haze, Rotation, State, View},
    state::video::{BgMode, Mode7, Mode7Select},
};
use oracle_record::FrameRecord;

fn word(wram: &[u8], offset: u16) -> Result<u16, Outcome> {
    source::WramWord {
        name: "shaft entry",
        offset,
    }
    .read(wram)
    .map_err(|_| data_failure(None, "shaft entry observation"))
}

fn program(wram: &[u8], positions: &[u16]) -> Result<bool, Outcome> {
    for slot in (0u16..12).step_by(2) {
        if word(wram, 0x18b4_u16.saturating_add(slot))? != 0
            && positions.contains(&word(wram, 0x18cc_u16.saturating_add(slot))?)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The independently recorded room, mirror and pending setup at one cut.
pub(super) fn observed(record: &FrameRecord) -> Result<State, Outcome> {
    let wram = &record.wram;
    let get = |at| word(wram, at);
    let width = get(source::ROOM_WIDTH_IN_BLOCKS.offset)?;
    let height = get(source::ROOM_HEIGHT_IN_BLOCKS.offset)?;
    let length = usize::from(width).saturating_mul(usize::from(height));
    let words = source::LEVEL_DATA
        .read(wram)
        .ok()
        .and_then(|bytes| bytes.get(..length.saturating_mul(2)))
        .ok_or_else(|| data_failure(Some(record.index), "shaft level words"))?;
    let marks = source::BTS
        .read(wram)
        .ok()
        .and_then(|bytes| bytes.get(..length))
        .ok_or_else(|| data_failure(Some(record.index), "shaft level marks"))?;
    let cells = words
        .as_chunks::<2>()
        .0
        .iter()
        .zip(marks)
        .map(|(word, mark)| Block::new(u16::from_le_bytes(*word), *mark))
        .collect();
    let sample = get(0x7e1)?;
    let placement = if get(0x1c85)? == 0xba48 {
        let index = get(0x1cd5)? / 2;
        Some([
            index
                .checked_rem(width)
                .ok_or_else(|| data_failure(None, "shaft placement width"))?,
            index
                .checked_div(width)
                .ok_or_else(|| data_failure(None, "shaft placement width"))?,
        ])
    } else {
        None
    };
    let [scale_x, shear_x, shear_y, scale_y, centre_x, centre_y] = record.ppu.m7;
    Ok(State {
        width,
        cells,
        placement,
        displayed: Mode7 {
            select: Mode7Select(record.ppu.m7sel),
            matrix: [scale_x, shear_x, shear_y, scale_y],
            centre: [centre_x, centre_y],
        },
        haze: if program(wram, &[0xdeeb, 0xdefa, 0xdefe])? {
            Haze::Red
        } else if program(wram, &[0xded3, 0xdee2, 0xdee6])? {
            Haze::Ordinary
        } else {
            Haze::Absent
        },
        view: View {
            hud: BgMode(get(0x55)?.to_le_bytes()[0]),
            room: BgMode(get(0x56)?.to_le_bytes()[0]),
            hud_lines: program(wram, &[0xd939, 0xd945])?.then_some(31),
            affine: get(0x783)? != 0,
            transform: Mode7 {
                select: Mode7Select(record.ppu.m7sel),
                matrix: [get(0x78)?, get(0x7a)?, get(0x7c)?, get(0x7e)?].map(u16::cast_signed),
                centre: [get(0x80)?, get(0x82)?].map(u16::cast_signed),
            },
            rotation: Rotation {
                sample: (sample & 0x7fff).cast_signed().saturating_sub(34),
                reverse: sample & 0x8000 != 0,
                hold: get(0x7e3)?,
            },
        },
    })
}

fn part(name: &str, bytes: Vec<u8>) -> Observation {
    Observation {
        field: format!("shaft-entry.{name}"),
        bytes,
    }
}

pub(super) fn fields(state: &State) -> Vec<Observation> {
    let view = state.view;
    let mut software = vec![
        view.hud.0,
        view.room.0,
        view.hud_lines.unwrap_or(0),
        u8::from(view.affine),
    ];
    software.extend(canonical::to_bytes(&view.transform));
    let mut grid = state.width.to_le_bytes().to_vec();
    for block in &state.cells {
        grid.extend(block.word().to_le_bytes());
        grid.push(block.kind.bts());
    }
    vec![
        part("view", software),
        part("affine-registers", canonical::to_bytes(&state.displayed)),
        part("rotation", canonical::to_bytes(&view.rotation)),
        part("cells", grid),
        part("placement", canonical::to_bytes(&state.placement)),
        part("haze", canonical::to_bytes(&state.haze)),
    ]
}

pub(super) fn expected(record: &FrameRecord) -> Result<Vec<Observation>, Outcome> {
    Ok(fields(&observed(record)?))
}
