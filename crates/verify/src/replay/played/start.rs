//! First-room admission from imported new-game content. Timings and controls are held
//! from the original run seed, and the arrival and frame clock come from the active scene.

use super::seed;
use crate::replay::{connected, data_failure, result::Outcome};
use data::assets::start::NewGame;
use game::production::played;
use game::production::room_loading::{self, Rooms};

/// The actor, controls, room effects and held appearance inputs that precede room loading.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(super) struct Carried {
    pub(super) language: types::options::Language,
    pub(super) health: samus_health::State,
    pub(super) bindings: crate::replay::hud::Bindings,
    pub(super) appearance: connected::AppearanceInputs,
    pub(super) quake: world_quake::State,
}

impl Carried {
    pub(super) fn seed(wram: &[u8]) -> Result<Self, Outcome> {
        Ok(Self {
            language: super::seed::language(wram)?,
            health: crate::replay::health_tail::decode(wram, None)?,
            bindings: crate::replay::hud::bindings(wram)?,
            appearance: connected::AppearanceInputs::seed(wram)?,
            quake: super::quake::seed(wram)?,
        })
    }

    /// The intro still has its borrowed controls. Its end restores the saved bindings.
    pub(super) fn last_pages(wram: &[u8]) -> Result<Self, Outcome> {
        let saved = |offset: usize| {
            wram.get(offset..offset.wrapping_add(2))
                .and_then(|bytes| bytes.try_into().ok())
                .map(u16::from_le_bytes)
                .ok_or_else(|| data_failure(None, "intro saved bindings"))
        };
        Ok(Self {
            bindings: crate::replay::hud::Bindings::new(saved(0x0d8a)?, saved(0x0d88)?),
            ..Self::seed(wram)?
        })
    }
}

pub(super) fn admit(
    carried: Carried,
    start: &NewGame,
    rooms: &Rooms,
    arrival: played::arrival::State,
    mut frame: played::frame::State,
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
        &mut frame.random,
    )
    .map_err(|_| bad())?;
    let inputs = carried.appearance.inputs(
        carried.health.energy.whole.get(),
        progress.inventory.items_switched_on,
        types::Set::default(),
        started.room.geometry.clone(),
        started.room.camera.clone(),
    );
    let map = started.map;
    let state = played::State::new_game(
        start,
        started,
        carried.health,
        arrival,
        frame,
        carried.quake,
    )
    .map_err(|_| bad())?;
    Ok((
        seed::Room {
            inputs,
            hud: super::hud::fresh(carried.bindings, &progress)?,
            ceres,
            map,
        },
        state,
    ))
}
