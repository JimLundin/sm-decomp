//! The elevator's admitted state and independent receiving observations.

use crate::replay::explosion::word;
use crate::replay::{data_failure, result::Outcome};
use data::map::{MAP_BIT_BYTES, MapCells};
use game::production::played::landing_site::{
    self, Area, AreaMaps, Entrance, ExploredMap, ShadowTilemaps,
};
use game::production::played::{
    departure::{Carry, Room, Run, Scene},
    explosion::{Cut, Flow},
};
use game::state::video::{Video, VideoMemory};
use oracle_record::FrameRecord;

pub(in crate::replay) fn room(
    record: &FrameRecord,
) -> Result<(Room, Area, AreaMaps, ShadowTilemaps), Outcome> {
    let (room, area, maps) = placement_and_maps(record)?;
    let shadow = |at| {
        Ok(ram(record, at, 0x800)?
            .as_chunks::<2>()
            .0
            .iter()
            .copied()
            .map(u16::from_le_bytes)
            .collect())
    };
    Ok((
        room,
        area,
        maps,
        ShadowTilemaps {
            bg1: shadow(0x4000)?,
            bg2: shadow(0x3000)?,
        },
    ))
}

pub(in crate::replay) fn placement_and_maps(
    record: &FrameRecord,
) -> Result<(Room, Area, AreaMaps), Outcome> {
    let w = |at| word(record, at);
    let cells = |at| {
        Ok(MapCells::from_bitmap(
            ram(record, at, MAP_BIT_BYTES)?
                .try_into()
                .map_err(|_| data_failure(Some(record.index), "departure map extent"))?,
        ))
    };
    let area = u8::try_from(w(0x079f)?)
        .ok()
        .and_then(|id| Area::try_from(id).ok())
        .ok_or_else(|| data_failure(Some(record.index), "departure area"))?;
    Ok((
        Room {
            placement: crate::replay::landing_site::placement(record)?,
            station: w(0x078b)?,
        },
        area,
        AreaMaps {
            active: ExploredMap {
                cells: cells(0x07f7)?,
                available: w(0x0789)? != 0,
            },
            crateria: ExploredMap {
                cells: cells(0xcd52)?,
                available: record
                    .wram
                    .get(0xd908)
                    .copied()
                    .ok_or_else(|| data_failure(Some(record.index), "Crateria map flag"))?
                    != 0,
            },
        },
    ))
}

fn ram(record: &FrameRecord, at: usize, count: usize) -> Result<&[u8], Outcome> {
    record
        .wram
        .get(at..at.saturating_add(count))
        .ok_or_else(|| data_failure(Some(record.index), "departure RAM extent"))
}

pub(in crate::replay) fn check_placement_and_maps(
    state: &game::production::played::State,
    selected: &types::save_files::RunSnapshot,
    record: &FrameRecord,
    cut: u64,
) -> Result<(), Outcome> {
    let (room, area, maps) =
        state
            .departure_room(selected)
            .ok_or_else(|| Outcome::Unsupported {
                cut: Some(cut),
                dependency: "live departure room and exploration".into(),
            })?;
    let (native_room, native_area, native_maps) = placement_and_maps(record)?;
    for (equal, field) in [
        (room.placement.room == native_room.placement.room, "room"),
        (room.placement.door == native_room.placement.door, "door"),
        (
            room.placement.camera == native_room.placement.camera,
            "camera",
        ),
        (room.placement.samus == native_room.placement.samus, "samus"),
        (room.station == native_room.station, "station"),
        (area == native_area, "area"),
        (maps.active == native_maps.active, "active-map"),
        (maps.crateria == native_maps.crateria, "crateria-map"),
    ] {
        if !equal {
            return Err(Outcome::Difference {
                cut,
                field: format!("departure.{field}"),
            });
        }
    }
    Ok(())
}

pub(in crate::replay) fn origin(
    station: &game::production::played::explosion::State,
    record: &FrameRecord,
) -> Result<Carry, Outcome> {
    let (room, area, maps, shadow) = room(record)?;
    Ok(Carry {
        room,
        landing: landing_site::State::new(area, maps, shadow),
        progress: (&crate::replay::progress::decode(&record.wram)?).into(),
        publication: crate::ppu::registers(&record.ppu),
        random: word(record, 0x05e5)?,
        work_ready: matches!(
            crate::replay::explosion::timing(record)?,
            Cut::Finished { .. }
        ),
        video: Video {
            memory: VideoMemory {
                vram: record.vram.to_vec(),
                oam: station.oam,
                cgram: station.palette,
            },
            committed: crate::ppu::registers(&record.ppu),
            dma: record.ppu.dma.map(crate::replay::landing_site::channel),
            batch: game::state::video::NmiBatch {
                fixed_colour: crate::replay::landing_site::queued_colour(record)?
                    .map(game::state::video::ColourWrite),
                cgram: station.palette,
                oam: station.oam,
                registers: crate::ppu::registers(&record.ppu),
                ..game::state::video::NmiBatch::default()
            },
            ..Video::default()
        },
    })
}

pub(super) fn check(run: &Run, record: &FrameRecord, timing: Cut, cut: u64) -> Result<(), Outcome> {
    let carry = &run.carry;
    let differs = |field: &str| Outcome::Difference {
        cut,
        field: format!("departure.{field}"),
    };
    if carry.random != word(record, 0x05e5)? {
        return Err(differs("random"));
    }
    let (observed_room, area, maps, shadow) = room(record)?;
    let mut actual_room = carry.room.clone();
    if carry.landing.entrance == Entrance::LandingSite {
        actual_room.station = 0x12;
    }
    if actual_room != observed_room
        || area != carry.landing.area
        || maps != carry.landing.maps
        || shadow != carry.landing.shadow
    {
        return Err(differs("room-map"));
    }
    if carry.progress != (&crate::replay::progress::decode(&record.wram)?).into() {
        return Err(differs("progress"));
    }
    let (health, settled) = match &run.scene {
        Scene::Station(state) => (
            &state.health.0,
            matches!(state.flow, Flow::Station | Flow::ZebesFlight),
        ),
        Scene::Flight(state) => (
            &state.health.0,
            state.scene.0.stage != types::zebes::ZebesStage::Pending,
        ),
    };
    if *health != crate::replay::health_tail::decode(&record.wram, Some(cut))? {
        return Err(differs("health"));
    }
    if (settled && matches!(timing, Cut::Finished { .. }))
        || carry.landing.entrance == Entrance::LandingSite
    {
        if let Some(i) = carry
            .video
            .memory
            .vram
            .iter()
            .zip(&record.vram)
            .position(|(a, b)| a != b)
        {
            return Err(differs(&format!("vram[{i}]")));
        }
        let actual = &carry.video.committed;
        let native = crate::ppu::registers(&record.ppu);
        if actual.mode7 != native.mode7 || actual.scroll != native.scroll {
            return Err(differs("affine"));
        }
        // E3's line-window owner retains its hardware settings after breakup. The
        // flight clears them on setup; its settled cuts compare every display register.
        if matches!(&run.scene, Scene::Flight(_)) && *actual != native {
            return Err(differs("display"));
        }
        let oam = &carry.video.memory.oam;
        if let Some(i) = oam
            .low
            .iter()
            .flat_map(|e| [e.x, e.y, e.tile, e.attributes])
            .chain(oam.high)
            .zip(record.oam.iter().copied())
            .position(|(a, b)| a != b)
        {
            return Err(differs(&format!("oam[{i}]")));
        }
        if carry
            .video
            .memory
            .cgram
            .iter()
            .map(|c| c.0)
            .ne(record.cgram.iter().copied())
        {
            return Err(differs("palette"));
        }
    }
    Ok(())
}
