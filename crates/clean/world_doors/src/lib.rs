//! The world-doors model, built from its behaviour spec alone: the passage through a door to a
//! room, from the frame after she touches it until play goes on in the next room.
//!
//! The room goes quiet and dark around the HUD, the view slides into the next room with her
//! carried along, she lands at the door's mouth, the new room's colours come up, and play goes
//! on with her moving as she was.
//!
//! A caller holds a [`Doors`] between frames and steps it once a frame with [`Doors::step`],
//! giving it the [`Scene`] the passage moves (her position, the view, the colours), what the
//! frame brings ([`Inputs`]) and the [`Rules`]. It answers [`Exports`]: the doors' next state,
//! the scene as the passage leaves it, and the frame's [`Report`] (what runs, the room's call,
//! the sound requests, the cost and whether the frame is finished).

mod catalogue;
mod colours;
mod io;
mod passage;
mod rules;
mod step;
mod view;

pub use catalogue::{Behaviour, BehaviourId, CATALOGUE, catalogue};
pub use colours::{COLOURS, ColourGroup, FadeTime, Palette};
pub use io::{
    Activity, Cost, CostEvent, Exports, Finishing, HerSound, Inputs, Report, RoomCall, RoomSound,
    Scene, SlideScreen,
};
pub use passage::{
    Arrival, Doors, Fetching, LAST_BEFORE_THE_SLIDE, LAST_LANDING, Passage, Phase, WorkFrames,
};
pub use rules::{Axis, Rules};
