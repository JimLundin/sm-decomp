//! What a frame brings the doors and what the doors answer it.

use enum_map::Enum;
use types::{Fixed, Pair, Position, Px, Set};

use crate::colours::{COLOURS, ColourGroup, Palette};
use crate::passage::{Arrival, Doors};

/// What the passage moves that belongs to other domains.
///
/// Her position, where the camera last saw her, the camera and the colours: the passage reads
/// each as the frame before left it and sets it. Nothing else of hers or theirs is here, so her
/// pose, facing, speeds and momentum are as the touch left them across the passage
/// (`SheKeepsHerMotion`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scene {
    /// Her position (samus-movement's).
    pub her: Position,
    /// The pixel parts of where the camera last saw her (the camera's `Remembering`); its
    /// subpixel parts are the camera's, and stay.
    pub seen: Pair<Px>,
    /// The camera's position; the passage sets its pixel parts and keeps its subpixel parts.
    pub camera: Position,
    /// The screen's colours and the colours they head for (the palettes').
    pub palette: Palette,
}

/// What a frame brings the doors from the other domains, each as this frame finds it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "each is one answer of its own domain, as the spec lists the inputs"
)]
pub struct Inputs {
    /// She touched a door to a room this frame (samus-movement), read in play only (`TheTouch`).
    pub touched: bool,
    /// The work of the frame of work in progress, of the cost event the passage names, is done
    /// by this frame's end (the platform, `PassageTime`).
    pub work_done: bool,
    /// Every sound request so far has been sent (audio, `Quieting`).
    pub sounds_sent: bool,
    /// No track waits in the queue of tracks, as this frame of work finds it (audio, `Landing`).
    pub music_ready: bool,
    /// The escape's timer is running (`TheFades`, `WhatRuns`).
    pub timer_running: bool,
    /// The room she leaves or the next one keeps the items dark (each room's content).
    pub items_dark: bool,
    /// The doorway's arrival (the world's records), which `Squaring` takes when it holds none.
    pub doorway: Option<Arrival>,
    /// Which group each of the screen's colours is in (the HUD's content and hers).
    pub groups: [ColourGroup; COLOURS],
}

/// What runs or is drawn on a frame of work (`WhatRuns`). Listed alphabetically.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum Activity {
    /// The blocks' run runs once (world-blocks' `TheScriptsRun`).
    Blocks,
    /// She, her shots, the enemies and their projectiles are drawn, still, where they are.
    Drawn,
    /// The enemies run.
    Enemies,
    /// The enemies' projectiles run.
    Projectiles,
    /// The animated tiles run.
    Tiles,
    /// The escape's timer is drawn, as it stands.
    Timer,
}

/// What the passage asks the room's loading to do, on the frame of work its phase names; what
/// each does is the room's loading's. Listed in the course's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoomCall {
    /// `Squaring`, taking the arrival: the room's objects that hold the line effects are let go.
    LeaveRoom,
    /// `BeforeTheSlide(0)`, for a way `Up`: the next room's top row is drawn ahead of the slide.
    DrawTopRow,
    /// `BeforeTheSlide(1)`: the next room is built, from the doorway touched.
    BuildRoom,
    /// `Arriving`, fetching `Tiles`: the tileset's tiles and colours.
    FetchTiles,
    /// `Arriving`, fetching `Contents`: the room's contents.
    FetchContents,
    /// `Landing(1)`: one run of the animated tiles.
    AnimateTiles,
    /// `Music { started: false }`, the music ready: the next room's track.
    PlayMusic,
    /// `Music { started: true }`: her surroundings' liquid, for her motion.
    ReadLiquid,
}

/// The room's sound requests (audio's). Listed alphabetically.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
pub enum RoomSound {
    /// Sounds are held off until play goes on.
    Held,
    /// Sounds are let through again.
    LetThrough,
    /// The silence.
    Silence,
}

/// The requests to her sounds (samus-appearance's, which decides whether she makes one).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HerSound {
    /// Her sound at the door, on the first frame of work.
    AtTheDoor,
    /// Her spin's sound again, as play goes on.
    SpinAgain,
}

/// The cost event of a frame of work, the passage's report to the platform (`PassageTime`).
/// Listed alphabetically.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CostEvent {
    /// `BeforeTheSlide(1)`: the next room built.
    BuildRoom,
    /// A step of a fade.
    FadeStep,
    /// `Arriving`, fetching `Contents`.
    FetchContents,
    /// `Arriving`, fetching `Tiles`.
    FetchTiles,
    /// Any other frame of work.
    Other,
}

/// What the platform is told of a frame's cost.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cost {
    /// Play runs: no passage, so the frame's cost is play's own.
    Play,
    /// The passage waits on its own, for the slide's end: the platform is asked nothing.
    OwnWait,
    /// The cost event of the frame of work in progress.
    Event(CostEvent),
}

/// What is shown below the HUD on a frame that restarts the HUD's frame count (`FinishedFrames`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlideScreen {
    /// The front layer and the sprites.
    FrontAndSprites,
    /// Only the sprites: either room keeps the items dark.
    Sprites,
}

/// Whether a frame is a finished frame, its picture, sprites and colours sent to the screen, its
/// pad read and its frame counts stepped (`FinishedFrames`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Finishing {
    /// Not a finished frame: the work overran it.
    Overrun,
    /// A finished frame.
    Finished,
    /// A finished frame on which the HUD's frame count is 0 before it counts, so 1 after, with
    /// what is shown below the HUD.
    Restarts(SlideScreen),
}

/// What the doors report on a frame, besides the scene they leave.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Report {
    /// A passage is under way this frame: her motion, the camera, the blocks' run, the HUD and
    /// play time don't run.
    pub passing: bool,
    /// What runs or is drawn this frame (`WhatRuns`).
    pub runs: Set<Activity>,
    /// The room's call this frame, if its phase makes one.
    pub call: Option<RoomCall>,
    /// The room's sound requests this frame.
    pub room_sounds: Set<RoomSound>,
    /// The request to her sounds this frame, if any.
    pub her_sound: Option<HerSound>,
    /// What the platform is told of this frame's cost.
    pub cost: Cost,
    /// Whether this is a finished frame.
    pub finishing: Finishing,
    /// The mini-map is turned on this frame (the HUD's input `the mini-map turned on`).
    pub minimap_on: bool,
    /// Layer 2's move with the view this frame, if the slide moves it (`Sliding`); where layer 2
    /// stands is the backgrounds'.
    pub layer_2: Option<Pair<Fixed>>,
}

impl Report {
    /// A frame of play: nothing of the passage's runs, and it is a finished frame.
    #[must_use]
    pub fn play() -> Self {
        Self::blank(false, Cost::Play, Finishing::Finished)
    }

    /// A frame of a passage at this cost, before its frame of work and its finishing are known.
    pub(crate) fn of_passage(cost: Cost) -> Self {
        Self::blank(true, cost, Finishing::Overrun)
    }

    /// A frame on which nothing is run, called, asked for or moved.
    fn blank(passing: bool, cost: Cost, finishing: Finishing) -> Self {
        Self {
            passing,
            runs: Set::default(),
            call: None,
            room_sounds: Set::default(),
            her_sound: None,
            cost,
            finishing,
            minimap_on: false,
            layer_2: None,
        }
    }
}

/// What a frame answers: the doors' state for the next frame, the scene as the passage leaves it,
/// and the frame's report.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Exports {
    /// The doors' state at the frame's end.
    pub doors: Doors,
    /// Her position, the view and the colours at the frame's end.
    pub scene: Scene,
    /// What the frame reports.
    pub report: Report,
}
