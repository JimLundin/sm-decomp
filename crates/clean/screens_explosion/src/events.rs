//! What the scene asks of the other owners, each on the frame it happens.

use types::explosion::{ArtSet, MapDrawing, ScreenPoint, SpacePalette, StationFlashStyle};

/// A song request to the audio service.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicRequest {
    /// Stop the music.
    Stop,
    /// Load the Ceres music data.
    CeresMusicData,
    /// Play the flight to Zebes song.
    ZebesFlightSong,
}

/// How many frames the audio service waits before it takes a request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MusicDelay(pub u8);

/// A region of the affine plane's map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapRegion {
    /// The upper region.
    Upper,
    /// The lower region.
    Lower,
}

/// Something the scene asks of another owner, in the order asked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SceneEvent {
    /// E2: install the stage and sprite art.
    InstallArt(ArtSet),
    /// R4: replace the space palette and its active effects with this palette and its
    /// navigation light.
    ReplaceSpacePalette(SpacePalette),
    /// U1: take this song request after this delay.
    RequestMusic {
        /// The request.
        request: MusicRequest,
        /// Its delay.
        delay: MusicDelay,
    },
    /// E3: start the station flash in this style, centred here.
    StartStationFlash {
        /// The flash's style.
        style: StationFlashStyle,
        /// Its position in whole pixels.
        center: ScreenPoint,
    },
    /// E2: install this drawing into this region of the plane's map.
    InstallMapDrawing {
        /// The drawing.
        drawing: MapDrawing,
        /// Where it goes.
        region: MapRegion,
    },
    /// T1: the scene is over; begin the flight to Zebes.
    BeginZebesFlight,
}
