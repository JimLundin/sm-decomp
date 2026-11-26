//! The facts a run holds from its seed, and the content it imports from the player's ROM.

use super::word;
use crate::replay::{connected, data_failure, movement, progress, result::Outcome};
use crate::source;
use data::{
    assets::world,
    rom::{Rom, RomAddress},
    room::RoomKey,
};
use game::production::connected::inputs::Inputs;
use game::production::{played, room_loading};
use game::{exact::save_files::Format, production::save_files};
use std::path::Path;
use types::save_files::{BatteryImage, SaveFilesState};

/// Cartridge state seeded from the complete physical image in an ordinary room.
pub(super) struct Battery {
    format: Format,
    state: SaveFilesState,
}

impl Battery {
    pub(super) fn load(record: &oracle_record::FrameRecord, path: &Path) -> Result<Self, Outcome> {
        let bad = || data_failure(None, "played cartridge seed");
        let rom = Rom::new(std::fs::read(path).map_err(|_| bad())?).map_err(|_| bad())?;
        let format = Format::load(rom.lorom()).map_err(|_| bad())?;
        let image = BatteryImage::new(record.sram.to_vec()).ok_or_else(bad)?;
        let state = save_files::power_on(&format, &image);
        Ok(Self { format, state })
    }

    pub(super) fn held(
        &mut self,
        record: &oracle_record::FrameRecord,
        cut: u64,
    ) -> Result<(), Outcome> {
        let output = save_files::advance(&mut self.state, &self.format, save_files::Input::Hold)
            .map_err(|_| data_failure(Some(cut), "played cartridge hold"))?;
        if output.image.as_slice() != record.sram.as_ref() {
            return Err(Outcome::Difference {
                cut,
                field: "save-files.battery".into(),
            });
        }
        Ok(())
    }
}

pub(super) fn bg1_offset(wram: &[u8]) -> Result<game::state::video::Scroll, Outcome> {
    let offset = |offset| {
        word(
            wram,
            source::WramWord {
                name: "BG1 scroll offset",
                offset,
            },
        )
    };
    Ok(game::state::video::Scroll {
        x: offset(0x091d)?,
        y: offset(0x091f)?,
    })
}

pub(super) fn language(wram: &[u8]) -> Result<types::options::Language, Outcome> {
    let japanese = word(
        wram,
        source::WramWord {
            name: "warning language",
            offset: 0x09e2,
        },
    )?;
    Ok(if japanese == 0 {
        types::options::Language::English
    } else {
        types::options::Language::Japanese
    })
}

pub(super) fn station_selector(wram: &[u8]) -> Result<types::StationId, Outcome> {
    word(
        wram,
        source::WramWord {
            name: "station selector",
            offset: 0x078b,
        },
    )
    .map(types::StationId)
}

/// Seed-held room facts.
#[derive(Clone)]
pub(super) struct Room {
    pub(super) inputs: Inputs,
    pub(super) ceres: boss_ceres_ridley::Stage,
    /// The HUD's neighbours outside the loop.
    pub(super) hud: super::hud::Held,
    /// Where her room sits on her map, and the area it is in.
    pub(super) map: (interface_hud::Room, data::room::Area),
}

/// The facts held from the seed, and what play reads of the seed's room: its geometry and
/// camera room as the seed holds them, and its doors by the seed's door cells.
pub(super) fn room(
    record: &oracle_record::FrameRecord,
    content: &connected::Content,
    world: &world::World,
) -> Result<(Room, played::passage::Room, run_progress::Progress), Outcome> {
    let seed = &record.wram;
    let unsupported = word(seed, source::CINEMATIC_ACTIVE)? != 0
        || word(seed, source::ELEVATOR_STATUS)? != 0
        || crate::replay::hud::time_stopped(seed)?;
    if unsupported {
        return Err(Outcome::Unsupported {
            cut: None,
            dependency: "played seed outside ordinary play".into(),
        });
    }
    let items = movement::items(word(seed, source::EQUIPPED_ITEMS)?);
    // The interactive list still describes the old population during
    // retirement, when passage does not query Samus's collision world.
    let geometry = if return_fetch(record, world)?.is_some_and(|seed| seed.awaiting_actors) {
        movement::level_world(seed, &content.movement, Vec::new())?
    } else {
        movement::borrowed(seed, seed, &content.movement)?.world
    };
    let inputs = connected::seed_inputs(seed, items, geometry)?;
    let room = imported_room(seed, world)?;
    let (doorways, map) = doorways(seed, world, room)?;
    let progress = progress::decode(seed)?;
    let entrance = word(
        seed,
        source::WramWord {
            name: "played incoming door",
            offset: 0x078d,
        },
    )?;
    let entrance = import::world::doorway(RomAddress::new(0x83, entrance));
    let tileset = room_loading::tileset_of(room, entrance, &progress)
        .map_err(|_| data_failure(None, "played room graphics"))?;
    let played = played::passage::Room {
        id: room.id,
        area: room.place.area,
        entered_through: entrance,
        tileset,
        entry: crate::replay::shaft_entry::observed(record)?,
        geometry: inputs.geometry.clone(),
        doorways,
        camera: inputs.camera.clone(),
    };
    let held = Room {
        inputs,
        map,
        hud: super::hud::held(seed)?,
        ceres: match word(seed, source::CERES_STATUS)? {
            0 => boss_ceres_ridley::Stage::Calm,
            1 => boss_ceres_ridley::Stage::Getaway,
            // The escape, its countdown the HUD's State from the seed. The shaft rotating
            // (8000h) is still refused.
            2 => boss_ceres_ridley::Stage::Escape,
            _ => {
                return Err(Outcome::Unsupported {
                    cut: None,
                    dependency: "played seed in the Ceres escape".into(),
                });
            }
        },
    };
    Ok((held, played, progress))
}

type Doorways = (Vec<played::passage::Doorway>, (interface_hud::Room, data::room::Area));

fn imported_room<'a>(seed: &[u8], world: &'a world::World) -> Result<&'a world::Room, Outcome> {
    let bad = || data_failure(None, "played room doors");
    let id = import::world::id(RoomKey::from_room_pointer(word(seed, source::ROOM_PTR)?))
        .ok_or_else(bad)?;
    world.room(id).ok_or_else(bad)
}

/// The seed room's doors to other rooms, by the seed's door cells, and where the room sits on
/// her map, from the imported world.
fn doorways(seed: &[u8], world: &world::World, room: &world::Room) -> Result<Doorways, Outcome> {
    let bad = || data_failure(None, "played room doors");
    let doorways =
        room_loading::doorways(world, room, movement::door_cells(seed)?).map_err(|_| bad())?;
    Ok((doorways, room))
}

/// The rooms the engine loads, their tilesets, the sprites' colours and the slopes, imported
/// from the player's ROM.
pub(in crate::replay) fn rooms(image: &Rom) -> Result<room_loading::Rooms, Outcome> {
    let bad = || data_failure(None, "played rooms import");
    let world = import::world::import(image.lorom(), &import::world::LOADED).map_err(|_| bad())?;
    let tilesets = import::tileset::import(image.lorom(), &world).map_err(|_| bad())?;
    Ok(room_loading::Rooms {
        world,
        tilesets,
        actors: import::actors::import(image.lorom()).map_err(|_| bad())?,
        samus: import::samus::palettes::import(image.lorom()).map_err(|_| bad())?,
        slopes: data::level::slope::Slopes::load(image.lorom()).map_err(|_| bad())?,
    })
}

/// The pending return and whether its destination population still needs placing.
#[derive(Clone, Copy)]
pub(super) struct ReturnFetch {
    pub(super) passage: world_doors::Passage,
    pub(super) doorway: played::passage::Doorway,
    pub(super) awaiting_actors: bool,
}

/// A Ceres return after the outgoing population has cleared or the fetch has completed.
pub(super) fn return_fetch(
    record: &oracle_record::FrameRecord,
    world: &world::World,
) -> Result<Option<ReturnFetch>, Outcome> {
    use data::assets::world::{doorways::RIDLEY_TO_ESCAPE_58, rooms::CERES_RIDLEY_ROOM};
    use world_doors::{Fetching, Passage, Phase, WorkFrames};

    let wram = &record.wram;
    let read = |offset| {
        word(
            wram,
            source::WramWord {
                name: "return passage",
                offset,
            },
        )
    };
    if read(0x0998)? != 11 || read(0x099c)? != 0xe4a9 {
        return Ok(None);
    }
    let pointer = import::world::doorway(RomAddress::new(0x83, read(0x078d)?));
    if pointer != Some(RIDLEY_TO_ESCAPE_58) {
        return Ok(None);
    }
    let slide_frames = read(0x0925)?;
    let ended = read(0x0931)? & 0x8000 != 0;
    let awaiting_actors = !ended
        && matches!(
            super::platform::actor_fetch(record),
            played::passage::ActorFetch::Clearing(_) | played::passage::ActorFetch::Graphics
        )
        && read(0x0f78)? == 0
        && read(0x0fb8)? == 0
        && read(0x0ff8)? == 0
        && read(0x1038)? == 0;
    let phase = if ended && super::finished(record) {
        Phase::Landing(WorkFrames::FIRST)
    } else if awaiting_actors
        || (!ended && matches!(record.nmi(), Some(nmi) if nmi.interrupted_pc == 0x0082_e529))
    {
        Phase::Arriving {
            placed: true,
            fetching: Fetching::Done,
        }
    } else {
        return Ok(None);
    };
    let bad = || data_failure(Some(record.index), "played return passage seed");
    let from = world.room(CERES_RIDLEY_ROOM).ok_or_else(bad)?;
    let through = world.doorway(RIDLEY_TO_ESCAPE_58).ok_or_else(bad)?;
    let here = import::world::id(RoomKey::from_room_pointer(read(0x079b)?));
    if here != Some(through.room) || read(0x093f)? != 2 {
        return Err(bad());
    }
    // The collision cell is used only to detect a touch in play. The pending passage
    // instead reads the same imported arrival and lighting as a real touch.
    let door = from
        .doors
        .iter()
        .position(|door| *door == world::Door::To(RIDLEY_TO_ESCAPE_58))
        .ok_or_else(bad)?;
    let neutral = types::Cell { x: 0, y: 0 };
    let [doorway] = room_loading::doorways(world, from, [(neutral, door)])
        .map_err(|_| bad())?
        .try_into()
        .map_err(|_| bad())?;
    let passage = Passage {
        phase,
        arrival: Some(doorway.arrival),
        slide_frames,
    };
    Ok(Some(ReturnFetch {
        passage,
        doorway,
        awaiting_actors,
    }))
}
