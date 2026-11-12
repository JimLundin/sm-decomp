//! The samus-movement model on recorded frames, through the engine's adapter: each frame decoded
//! from record N, stepped by the model over the engine's world, compared with record N+1 on
//! her position and speeds. `SM_CACHE` and `SM_ROM`, as the harness tests read them.

use std::path::{Path, PathBuf};

use game::collision::Room;
use game::collision::solid_enemy::SolidEnemies;
use game::samus_movement_bridge::SamusWorld;
use game::samus_movement_bridge::frame::{FrameSource, body};
use game::samus_movement_bridge::state::{clean_of, write_back};
use game::state::State;
use samus_movement::Rules;

use crate::bundle::{content, rooms};
use crate::cache::Cache;
use crate::decode::{Quiescence, decode};
use crate::lock::{Lock, RowId};

/// The fields compared: her position and speeds, as the game keeps them.
fn motion(state: &State) -> impl PartialEq + std::fmt::Debug {
    let motion = &state.samus.motion;
    (
        motion.position,
        motion.base_speed,
        motion.extra_run,
        motion.vertical_speed,
        motion.vertical,
    )
}

/// The model's step from `from`, with the pad `to` read.
fn stepped(from: &State, to: &State, content: &game::Content) -> Option<State> {
    let mut samus = from.samus;
    let mut model = clean_of(&samus)?;
    let room = Room {
        blocks: &from.room.blocks,
        width: from.room.width,
        slopes: Some(&content.slopes),
        doors: content.room_doors.of(from.room.room),
    };
    let world = SamusWorld::new(room, SolidEnemies::default());
    let sized = |pose| body(content, &samus, pose);
    let frame = FrameSource {
        samus: &samus,
        controller: &to.controller,
        ride: from.elevator,
        completed: from.clock.counters.completed.0,
        content,
    }
    .frame(&sized);
    let outcome = model.step(&frame, &world, &Rules::default()).ok()?;
    write_back(&model, outcome.rising, &mut samus);
    Some(State {
        samus,
        ..from.clone()
    })
}

/// The first recorded frame of item-morph-ball on which she runs, stepped by the model, lands
/// where record N+1 has her, at its speeds.
#[test]
fn the_model_steps_a_recorded_frame_as_the_game_did() {
    let (Some(cache), Some(rom)) = (std::env::var_os("SM_CACHE"), std::env::var_os("SM_ROM"))
    else {
        return;
    };
    let lock_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenarios/locks/lsnes.toml");
    let text = std::fs::read_to_string(lock_path).unwrap_or_default();
    let Ok(lock) = Lock::parse(&text) else {
        panic!("the lock parses");
    };
    let Some(row) = lock.row(&RowId("item-morph-ball".to_owned())) else {
        panic!("item-morph-ball is in the lock");
    };
    let cache = match Cache::open(&PathBuf::from(cache), row) {
        Ok(cache) => cache,
        Err(error) => panic!("cache: {error}"),
    };
    let image = std::fs::read(rom).unwrap_or_default();
    let Ok(content) = content(image.clone()) else {
        panic!("content");
    };
    let Ok(rooms) = rooms(&content, &image) else {
        panic!("rooms");
    };
    let decoded: Vec<_> = cache
        .records()
        .filter_map(Result::ok)
        .map(|(_, record)| decode(&record, &rooms).ok())
        .collect();
    let pair = decoded
        .windows(2)
        .filter_map(|pair| match pair {
            [Some(from), Some(to)] if from.quiescence == Quiescence::Quiet => Some((from, to)),
            _ => None,
        })
        .find(|(from, to)| {
            from.state.samus.motion.base_speed.0 != 0
                && from.state.samus.motion.position != to.state.samus.motion.position
        });
    let Some((from, to)) = pair else {
        panic!("a frame she moves on");
    };
    let Some(model) = stepped(&from.state, &to.state, &content) else {
        panic!("her pose is in the domain");
    };
    assert_eq!(motion(&model), motion(&to.state));
}
