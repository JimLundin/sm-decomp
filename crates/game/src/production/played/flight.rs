//! The gunship's flight to Ceres: dark while its music loads, the rush in, the turn, the flight
//! away, the station's label and the fade out (screens-flight).
//!
//! The flight runs ahead of the new game, with no room: a run starts here at the flight's first
//! frame, from the start below, and reads nothing of room play. It steps once per frame of work
//! that finished; a lag frame is the platform's and changes nothing of it. On the first finished
//! frame after the flight is done the new game begins, which is a later piece.

use screens_flight::{FrameReport, MusicQueue, Rules, step};
use types::flight::{
    Angle, Brightness, FadeFrames, FlightSprites, FlightStage, Plane, Scene, State, Zoom,
};
use types::{Colour, Pair, ZERO};

/// How a frame ended: its work finished, with audio's reply as it began, or the frame lagged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameEnd {
    /// The frame's work finished; whether a track waited in audio's queue as it began.
    Finished(MusicQueue),
    /// The frame lagged: its work goes on into the next.
    Lagged,
}

/// Why a frame was not played. State is unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// The flight is done and this frame begins the new game, a later piece.
    NewGameBegins,
}

/// The flight as it starts: yet to be made ready, with everything else empty and the screen
/// dark.
#[must_use]
pub fn start() -> State {
    State {
        stage: FlightStage::Starting,
        plane: Plane {
            angle: Angle::SQUARE,
            zoom: Zoom::NONE,
            position: Pair { x: ZERO, y: ZERO },
        },
        sprites: FlightSprites::default(),
        flash: Colour::BLACK,
        brightness: Brightness::DARK,
        fade_count: FadeFrames::ZERO,
    }
}

/// Play one frame of the flight. A lagged frame changes nothing and reports nothing.
///
/// # Errors
/// [`Fault::NewGameBegins`] on a finished frame once the flight is done.
pub fn played(
    state: &mut State,
    frame: FrameEnd,
    scene: &Scene,
    rules: Rules,
) -> Result<Option<FrameReport>, Fault> {
    let FrameEnd::Finished(music) = frame else {
        return Ok(None);
    };
    if state.stage == FlightStage::Done {
        return Err(Fault::NewGameBegins);
    }
    Ok(Some(step(state, scene, music, rules)))
}

#[cfg(test)]
mod tests {
    use super::{Fault, FrameEnd, played, start};
    use screens_flight::{MusicQueue, Rules};
    use std::num::NonZeroU16;
    use types::flight::{
        Angle, Brightness, FlightStage, Keystroke, LabelStep, RushSpeed, Scene, SceneSetup,
        SceneTurn, Zoom,
    };
    use types::{Pair, flight::WholePlace};

    const AT: WholePlace = Pair { x: 0, y: 0 };

    fn scene() -> Scene {
        Scene {
            setup: SceneSetup {
                angle: Angle(0x20),
                zoom: Zoom::new(48).unwrap(),
                plane: AT,
                stars: AT,
                speed: RushSpeed(0),
            },
            turn: SceneTurn {
                angle: Angle(0),
                plane: AT,
                sprites: enum_map::EnumMap::default(),
            },
            label: vec![LabelStep {
                keystroke: Keystroke::Pause,
                hold: NonZeroU16::MIN,
            }],
        }
    }

    const CLEAR: FrameEnd = FrameEnd::Finished(MusicQueue::Clear);

    #[test]
    fn a_lagged_frame_changes_nothing() {
        let mut state = start();
        assert_eq!(
            played(&mut state, FrameEnd::Lagged, &scene(), Rules::REFERENCE),
            Ok(None)
        );
        assert_eq!(state, start());
    }

    #[test]
    fn the_flight_plays_to_its_end_and_then_hands_on() {
        let mut state = start();
        let mut frames = 0;
        while state.stage != FlightStage::Done {
            assert!(
                played(&mut state, CLEAR, &scene(), Rules::REFERENCE)
                    .unwrap()
                    .is_some()
            );
            frames += 1;
            assert!(frames < 1000, "the flight ends");
        }
        assert_eq!(state.brightness, Brightness::DARK);
        let done = state;
        assert_eq!(
            played(&mut state, CLEAR, &scene(), Rules::REFERENCE),
            Err(Fault::NewGameBegins)
        );
        assert_eq!(state, done, "a refused frame changes nothing");
    }
}
