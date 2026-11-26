//! Independent record projections of the continuing game and its physical drawing state.
use crate::replay::compare::Observation;
use crate::replay::{
    data_failure, explosion::word, landing_site, played::explosion as joined, readers::ExpectedCut,
    result::Outcome,
};
use data::assets::landing_site::Graphics;
use game::exact::direct_dma::{PendingWrite, Transfer};
use game::production::{
    Production,
    health_tail::HealthPin,
    played::{
        departure::{Input, Remembered, Room, Scene},
        explosion,
        landing_site::{Area, AreaMaps, Entrance, Phase, ShadowTilemaps, Work},
        zebes,
    },
};
use game::state::video::{DmaChannel, Registers, VramWord};
use oracle_record::FrameRecord;

fn bytes(field: &str, bytes: Vec<u8>) -> Observation {
    Observation {
        field: format!("departure.{field}"),
        bytes,
    }
}
fn words(field: &str, values: impl IntoIterator<Item = u16>) -> Observation {
    bytes(
        field,
        values.into_iter().flat_map(u16::to_le_bytes).collect(),
    )
}
fn room(
    mut room: Room,
    area: Area,
    maps: &AreaMaps,
    shadow: &ShadowTilemaps,
    entrance: Entrance,
) -> Observation {
    if entrance == Entrance::LandingSite {
        room.station = 0x12;
    }
    let mut data = canonical::to_bytes(&room);
    data.push(u8::from(area));
    data.extend(canonical::to_bytes(maps));
    data.extend(canonical::to_bytes(shadow));
    bytes("room", data)
}
fn plane(r: &Registers) -> Observation {
    words(
        "plane",
        r.mode7
            .matrix
            .into_iter()
            .chain(r.mode7.centre)
            .map(i16::cast_unsigned)
            .chain(r.scroll.into_iter().flat_map(|s| [s.x, s.y])),
    )
}
fn channels(channels: [DmaChannel; 8]) -> Observation {
    bytes(
        "dma",
        channels
            .into_iter()
            .enumerate()
            .filter(|(i, _)| !matches!(i, 2 | 3))
            .flat_map(|(_, channel)| landing_site::io_bytes(channel))
            .collect(),
    )
}
fn palette(values: impl IntoIterator<Item = u16>) -> Observation {
    words("palette", values)
}
fn payload(fields: Vec<Observation>, outputs: Vec<Observation>) -> ExpectedCut {
    ExpectedCut {
        semantics: fields.clone(),
        semantic_count: fields.len(),
        output_count: outputs.len(),
        fields: fields.into_iter().chain(outputs).collect(),
        replies: 0,
    }
}

pub(super) fn native(
    previous: &FrameRecord,
    record: &FrameRecord,
    graphics: &Graphics,
    input: Input,
) -> Result<ExpectedCut, Outcome> {
    let w = |at| word(record, at);
    let (placement, area, maps, shadow) = joined::carry::room(record)?;
    let loading = w(0x0998)? == 6;
    let receiving = matches!(input, Input::Loader(_));
    let phase = if receiving {
        4
    } else if loading {
        3
    } else {
        match w(0x1f51)? {
            0xc699 => 1,
            0xc79c | 0xc7ca | 0xa38f | 0xc9f9 | 0xca36 | 0xca85 | 0xcad0 | 0xcade | 0xcadf => 2,
            _ => 0,
        }
    };
    let remembered = Remembered::from(&crate::replay::progress::seed(&record.wram)?);
    let health = canonical::model::Model::<HealthPin>(crate::replay::health_tail::decode(
        &record.wram,
        Some(record.index),
    )?);
    let fields = vec![
        words(
            "schedule",
            [phase, w(0x05b6)?, w(0xd914)?, w(0x09c4)?, w(0x09e2)?],
        ),
        bytes("health", canonical::to_bytes(&health)),
        words("random", [w(0x05e5)?]),
        bytes("progress", canonical::to_bytes(&remembered)),
        room(placement, area, &maps, &shadow, Entrance::Carried),
    ];
    let mut outputs = Vec::new();
    let transfer = if let Input::Loader(Work::Open { accepted, selected }) = input {
        let offset = accepted
            .checked_sub(1)
            .ok_or_else(|| data_failure(Some(record.index), "pending sprite byte"))?;
        let value = graphics
            .sprites
            .get(usize::from(offset))
            .copied()
            .ok_or_else(|| data_failure(Some(record.index), "sprite source byte"))?;
        Some(Transfer {
            source: graphics.sprite_source,
            destination: VramWord(0x6000),
            length: graphics.sprite_length,
            accepted,
            committed: offset,
            pending: Some(PendingWrite { offset, value }),
            selected: selected.then(|| graphics.sprite_source.advanced(accepted)),
        })
    } else {
        None
    };
    outputs.push(bytes("pipeline", canonical::to_bytes(&transfer)));
    if receiving
        || matches!(input, Input::Scene(explosion::Cut::Finished { .. }))
            && !matches!(w(0x1f51)?, 0xc11b)
    {
        outputs.extend([
            words("vram", record.vram.iter().copied()),
            bytes("oam", record.oam.to_vec()),
            palette(record.cgram.iter().copied()),
            plane(&landing_site::registers(&record.ppu)),
        ]);
    }
    if receiving {
        outputs.extend([
            bytes(
                "display",
                canonical::to_bytes(&landing_site::registers(&record.ppu)),
            ),
            bytes(
                "published",
                canonical::to_bytes(&landing_site::registers(&previous.ppu)),
            ),
            bytes("mirrors", landing_site::observed_mirrors(record)?),
            channels(record.ppu.dma.map(landing_site::channel)),
        ]);
    }
    Ok(payload(fields, outputs))
}

pub(in crate::replay) fn actual(
    production: &Production,
    cut: u64,
) -> Result<Vec<Observation>, Outcome> {
    let run = joined::state(production, cut)?;
    let carry = &run.carry;
    let (phase, finished, destination, maximum, japanese, health) = match &run.scene {
        Scene::Station(s) => (
            u16::from(s.flow == explosion::Flow::ZebesFlight),
            s.finished,
            match s.destination {
                explosion::Destination::LandingSite => 0x22,
                explosion::Destination::CeresStart => 0x1f,
            },
            s.maximum,
            s.japanese_subtitles,
            &s.health,
        ),
        Scene::Flight(s) => {
            let phase = if !matches!(carry.landing.phase, Phase::Pending) {
                4
            } else if matches!(s.flow, zebes::Flow::Loading(_)) {
                3
            } else if s.scene.0.stage == types::zebes::ZebesStage::Pending {
                1
            } else {
                2
            };
            (
                phase,
                s.finished,
                crate::replay::zebes::destination_word(s.flow.destination()),
                s.maximum,
                s.subtitles_enabled(),
                &s.health,
            )
        }
    };
    let video = &carry.video;
    let result = vec![
        words(
            "schedule",
            [phase, finished, destination, maximum, u16::from(japanese)],
        ),
        bytes("health", canonical::to_bytes(health)),
        words("random", [carry.random]),
        bytes("progress", canonical::to_bytes(&carry.progress)),
        room(
            carry.room.clone(),
            carry.landing.area,
            &carry.landing.maps,
            &carry.landing.shadow,
            carry.landing.entrance,
        ),
        bytes(
            "pipeline",
            canonical::to_bytes(&match carry.landing.phase {
                Phase::Sprites(t) => Some(t),
                Phase::Pending | Phase::Ready => None,
            }),
        ),
        words("vram", video.memory.vram.iter().copied()),
        bytes(
            "oam",
            video
                .memory
                .oam
                .low
                .iter()
                .flat_map(|s| [s.x, s.y, s.tile, s.attributes])
                .chain(video.memory.oam.high)
                .collect(),
        ),
        palette(video.memory.cgram.iter().map(|c| c.0)),
        plane(&video.committed),
        bytes("display", canonical::to_bytes(&video.committed)),
        bytes("published", canonical::to_bytes(&carry.publication)),
        bytes(
            "mirrors",
            landing_site::display_mirrors(&video.batch, carry.landing.control),
        ),
        channels(video.dma),
    ];
    Ok(result)
}
