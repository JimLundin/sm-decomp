//! The gunship's flight to Ceres: dark while its music loads, the rush in, the turn, the flight
//! away, the station's label and the fade out (screens-flight).
//!
//! The flight runs ahead of the new game, with no room: a run starts here at the flight's first
//! frame, from the start below, and reads nothing of room play. It steps once per frame of work
//! that finished; a lag frame is the platform's and changes nothing of it. The frame of work
//! after the flight is done begins the new game, even if that frame has not yet finished.

use super::{arrival, frame};
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

/// A flight or the new game's dark preparation, with the frame owner's carried State.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Journey {
    /// The active scene's State.
    pub stage: Stage,
    /// The game's counters and random sequence.
    pub frame: frame::State,
    setup_already_drew: bool,
}

/// The scene that owns the picture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    /// The flight through its last frame.
    Flight(State),
    /// The new game is preparing its start.
    Arrival(arrival::State),
}

/// Why a frame was not played. State is unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// The flight handed on. The room owner steps all later frames.
    RoomBegun,
}

impl Journey {
    /// Begin the flight with the frame owner's State as the scene starts.
    #[must_use]
    pub fn start(frame: frame::State) -> Self {
        Self {
            stage: Stage::Flight(start()),
            frame,
            setup_already_drew: false,
        }
    }

    /// Resume the unfinished setup whose opening draw is already in the frame State.
    #[must_use]
    pub fn resume_setup(frame: frame::State) -> Self {
        Self {
            setup_already_drew: true,
            ..Self::start(frame)
        }
    }

    /// Step the flight or its handoff into the dark preparation. The frame of work that
    /// changes scenes runs before the preparation's own frame of work. A finished flight
    /// frame returns its report, including the ordered music requests and their delays for audio.
    ///
    /// # Errors
    /// [`Fault::RoomBegun`] after the handoff to the new-game room owner.
    pub fn step(
        &mut self,
        begun: bool,
        end: FrameEnd,
        scene: &Scene,
        rules: Rules,
    ) -> Result<Option<FrameReport>, Fault> {
        let finishing = frame::finishing(matches!(end, FrameEnd::Finished(_)));
        let (report, drew) = match &mut self.stage {
            Stage::Flight(flight) if flight.stage == FlightStage::Done => {
                self.stage = Stage::Arrival(arrival::State::after_flight(flight.brightness));
                (None, begun)
            }
            Stage::Flight(flight) => {
                let report = played(flight, end, scene, rules);
                // A flight frame draws once. Its audio service can overrun before that draw,
                // so the frame owner accounts for it where the platform finishes the work.
                let drew = report.is_some() && !std::mem::take(&mut self.setup_already_drew);
                (report, drew)
            }
            Stage::Arrival(_) => return Err(Fault::RoomBegun),
        };
        let random = if drew {
            frame::draw(self.frame.random)
        } else {
            self.frame.random
        };
        self.frame = self.frame.after(finishing, random);
        Ok(report)
    }
}

/// The flight as it starts: yet to be made ready, with everything else empty and the screen
/// dark.
#[must_use]
fn start() -> State {
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
fn played(state: &mut State, frame: FrameEnd, scene: &Scene, rules: Rules) -> Option<FrameReport> {
    let FrameEnd::Finished(music) = frame else {
        return None;
    };
    Some(step(state, scene, music, rules))
}

#[cfg(test)]
mod tests {
    use super::{Fault, FrameEnd, Journey, Stage, arrival, played, start};
    use screens_flight::{MusicQueue, Rules};
    use std::num::NonZeroU16;
    use types::flight::{
        Angle, Brightness, FlightStage, Keystroke, LabelStep, MusicRequest, RushSpeed, Scene,
        SceneSetup, SceneTurn, TuneDelay, Zoom,
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
    fn the_setup_reports_music_requests_in_order() {
        let frame = crate::production::played::tests::state(Vec::new()).frame;
        let mut journey = Journey::start(frame);
        let scene = scene();
        let mut step = |end| journey.step(true, end, &scene, Rules::REFERENCE).unwrap();
        assert_eq!(step(FrameEnd::Lagged), None);
        let report = step(CLEAR).unwrap();
        assert_eq!(
            report.music,
            [
                MusicRequest::LoadCeresFlightTune(TuneDelay(8)),
                MusicRequest::PlayCeresFlightTune(TuneDelay(14)),
            ]
        );
        assert_eq!(step(FrameEnd::Lagged), None);
        let report = step(CLEAR).unwrap();
        assert_eq!(report.music, []);
    }

    #[test]
    fn a_lagged_frame_changes_nothing() {
        let mut state = start();
        assert_eq!(
            played(&mut state, FrameEnd::Lagged, &scene(), Rules::REFERENCE),
            None
        );
        assert_eq!(state, start());
    }

    #[test]
    fn the_flight_plays_to_its_end_and_then_hands_on() {
        let frame = crate::production::played::tests::state(Vec::new()).frame;
        let mut journey = Journey::start(frame);
        let scene = scene();
        let Stage::Flight(flight) = &mut journey.stage else {
            panic!("the journey starts in flight");
        };
        let mut frames = 0;
        while flight.stage != FlightStage::Done {
            assert!(played(flight, CLEAR, &scene, Rules::REFERENCE).is_some());
            frames += 1;
            assert!(frames < 1000, "the flight ends");
        }
        assert_eq!(flight.brightness, Brightness::DARK);
        let step = |journey: &mut Journey, end| journey.step(true, end, &scene, Rules::REFERENCE);
        assert_eq!(step(&mut journey, FrameEnd::Lagged), Ok(None));
        let Stage::Arrival(arrival) = journey.stage else {
            panic!("the lagged handoff starts the arrival");
        };
        assert!(arrival.preparing());
        assert_eq!(arrival.preparation, arrival::Begun::Not);

        let waiting = journey;
        assert_eq!(step(&mut journey, CLEAR), Err(Fault::RoomBegun));
        assert_eq!(journey, waiting);
    }
}
