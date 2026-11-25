//! One frame of the flight, in the order that matters, and what it hands the others.

use types::Colour;
use types::flight::{
    Brightness, FlightStage, MusicRequest, Scene, State, TypedLetter, WholePlace, whole_pixels,
};

use crate::plane::{PlaneTransform, transform};
use crate::rules::Rules;
use crate::sprites::{SpriteView, drift, rush, views};
use crate::stages::{MusicQueue, StageEvents, advance};

/// What the flight hands the others this frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameReport {
    /// Spec: `PlaneTransforms`. The plane's transformation, to the renderer's plane.
    pub transform: PlaneTransform,
    /// Spec: `PlaneTransforms`. The scroll: the whole pixels of the plane's position.
    pub scroll: WholePlace,
    /// The flash as it stands, to the renderer's colour blend.
    pub flash: Colour,
    /// Each sprite's kind and whole-pixel place, to the sprite drawing.
    pub sprites: Vec<SpriteView>,
    /// Spec: `LabelIsTyped`. The letter typed as its step began this frame, to the tile writing.
    pub typed: Option<TypedLetter>,
    /// The screen's brightness, to the renderer.
    pub brightness: Brightness,
    /// Spec: `FlightIsMadeReady`. The music requests this frame, in order, to audio: load and
    /// play the Ceres flight tune on the frame the flight is made ready, none on any other.
    pub music: Vec<MusicRequest>,
    /// Spec: `FlightEnds`. Whether the flight is done: the next frame starts the new game.
    pub done: bool,
}

/// Runs one frame of the flight, its steps in the order that matters.
///
/// First the stage's step; then the stars' rush if the stage is then `RushingIn` (so they rush
/// on the frame the picture appears, and are gone before they would rush on the turn's); then
/// every placed sprite's drift (so the turn's sprites drift on the frame they are placed); then
/// the transformation and the scroll from what the frame left.
pub fn step(state: &mut State, scene: &Scene, music: MusicQueue, rules: Rules) -> FrameReport {
    let StageEvents {
        typed,
        music: requests,
    } = advance(state, scene, music, rules);
    if state.stage == FlightStage::RushingIn {
        rush(&mut state.sprites, &mut state.plane);
    }
    drift(&mut state.sprites);
    FrameReport {
        transform: transform(&state.plane),
        scroll: whole_pixels(state.plane.position),
        flash: state.flash,
        sprites: views(&state.sprites),
        typed,
        brightness: state.brightness,
        music: requests,
        done: state.stage == FlightStage::Done,
    }
}
