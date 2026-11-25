//! First-room admission from imported new-game content. Timings and controls are held
//! from the original run seed, and the arrival and frame clock come from the active scene.

use super::seed;
use crate::replay::{connected, data_failure, result::Outcome};
use data::assets::start::NewGame;
use game::production::played;
use game::production::room_loading::{self, Rooms};

pub(super) fn admit(
    seed: &[u8],
    start: &NewGame,
    rooms: &Rooms,
    arrival: played::arrival::State,
    frame: played::frame::State,
    quake: world_quake::State,
) -> Result<(seed::Room, played::State), Outcome> {
    let bad = || data_failure(None, "played new game's start");
    let progress = run_progress::Progress::default();
    let ceres = boss_ceres_ridley::Stage::Calm;
    let id = import::world::id(start.room).ok_or_else(bad)?;
    let started = room_loading::start(
        id,
        room_loading::Run {
            progress: &progress,
            ceres,
        },
        rooms,
        &start.her_row,
    )
    .map_err(|_| bad())?;
    let inputs = connected::held_inputs(
        seed,
        progress.inventory.items_switched_on,
        types::Set::default(),
        started.room.geometry.clone(),
        started.room.camera.clone(),
    )?;
    let map = started.map;
    let state = played::State::new_game(start, started, &progress, arrival, frame, quake)
        .map_err(|_| bad())?;
    Ok((
        seed::Room {
            inputs,
            hud: super::hud::fresh(seed, &progress)?,
            progress,
            ceres,
            map,
        },
        state,
    ))
}
