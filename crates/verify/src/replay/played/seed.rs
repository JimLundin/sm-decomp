//! The facts a run holds from its seed, and the content it imports from the player's ROM.

use super::word;
use crate::replay::{connected, data_failure, movement, progress, result::Outcome};
use crate::source;
use data::{assets::world, rom::Rom, room::RoomKey};
use game::production::connected::inputs::Inputs;
use game::production::{played, room_loading};

/// Seed-held room facts.
pub(super) struct Room {
    pub(super) inputs: Inputs,
    /// Audio is outside the loop: the seed's queue holds no sound, and none is taken to wait.
    pub(super) sounds_sent: bool,
    pub(super) progress: run_progress::Progress,
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
) -> Result<(Room, played::passage::Room), Outcome> {
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
        sounds_sent: inputs.sounds_waiting.count() == 0,
        inputs,
        map,
        hud: super::hud::held(seed)?,
        progress: progress::seed(seed)?,
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
    Ok((held, played))
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
