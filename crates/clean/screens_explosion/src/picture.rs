//! The sprites the scene's picture shows, for the sprite renderer (E1).

use types::Pair;
use types::explosion::{
    AsteroidField, BackdropFrame, BurstId, ExplosionScene, ExplosionState, ScreenPoint, Subpixels,
};

use crate::bursts::{self, BurstFrame};
use crate::sky;

/// A live burst as it shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BurstPicture {
    /// Which burst it is.
    pub id: BurstId,
    /// Where it is.
    pub position: ScreenPoint,
    /// What it shows.
    pub frame: BurstFrame,
}

/// The scene's sprites as they show, in the importer's drawing order: the large asteroids, the
/// bursts, the small asteroids, then the purple background at its imported place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpritePicture {
    /// Where the large asteroids are.
    pub large_asteroids: ScreenPoint,
    /// The live bursts, front to back.
    pub bursts: Vec<BurstPicture>,
    /// Where the small asteroids are.
    pub small_asteroids: ScreenPoint,
    /// Which drawing the purple background shows.
    pub backdrop: BackdropFrame,
}

/// The sprites of the picture `state` shows.
#[must_use]
pub fn sprites(state: &ExplosionState, scene: &ExplosionScene) -> SpritePicture {
    let field_place = |field: AsteroidField| Pair {
        x: state.asteroids[field],
        y: Subpixels::whole(i32::from(scene.sky.asteroids[field].start.y)),
    };
    SpritePicture {
        large_asteroids: field_place(AsteroidField::Large),
        bursts: state
            .bursts
            .iter()
            .map(|burst| BurstPicture {
                id: burst.id,
                position: burst.position,
                frame: bursts::frame(&scene.tracks[burst.id.group], burst.age),
            })
            .collect(),
        small_asteroids: field_place(AsteroidField::Small),
        backdrop: sky::backdrop_frame(state.age),
    }
}
