//! The facts a run holds from its seed, and the content it imports from the player's ROM.

use super::word;
use crate::replay::{connected, data_failure, movement, progress, result::Outcome};
use crate::source;
use data::{assets::world, rom::Rom, room::RoomKey};
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
    seed: &[u8],
    content: &connected::Content,
    world: &world::World,
) -> Result<(Room, played::passage::Room, run_progress::Progress), Outcome> {
    let borrowed = movement::borrowed(seed, seed, &content.movement)?;
    if borrowed.cinematic || borrowed.elevator || crate::replay::hud::time_stopped(seed)? {
        return Err(Outcome::Unsupported {
            cut: None,
            dependency: "played seed outside ordinary play".into(),
        });
    }
    let inputs = connected::seed_inputs(seed, borrowed.items, borrowed.world)?;
    let (doorways, map) = doorways(seed, world)?;
    let played = played::passage::Room {
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
    Ok((held, played, progress::decode(seed)?))
}

type Doorways = (
    Vec<played::passage::Doorway>,
    (interface_hud::Room, data::room::Area),
);

/// The seed room's doors to other rooms, by the seed's door cells, and where the room sits on
/// her map, from the imported world.
fn doorways(seed: &[u8], world: &world::World) -> Result<Doorways, Outcome> {
    let bad = || data_failure(None, "played room doors");
    let id = import::world::id(RoomKey::from_room_pointer(word(seed, source::ROOM_PTR)?))
        .ok_or_else(bad)?;
    let room = world.room(id).ok_or_else(bad)?;
    let doorways =
        room_loading::doorways(world, room, movement::door_cells(seed)?).map_err(|_| bad())?;
    Ok((doorways, (room_loading::map_room(room), room.place.area)))
}

/// The rooms the engine loads, their tilesets, the sprites' colours and the slopes, imported
/// from the player's ROM.
pub(super) fn rooms(image: &Rom) -> Result<room_loading::Rooms, Outcome> {
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
