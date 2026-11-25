//! Her arrival at Ceres as a new game begins: the dark wait while the start is made ready, the
//! frame she arrives, and the fade in up to play.
//!
//! While the start is made ready nothing of play runs: the game's one frame of work runs the
//! whole preparation, drawing the random number as it begins, and the platform's lag says which
//! of its frames finish. On the frame it is ready she arrives at the top of the shaft facing the
//! screen, the elevator's pad is placed under her, the screen takes its colours and the mini-map
//! shows; no frame of play runs on it. Each frame of the fade in is a frame of play, and the
//! fade takes its step after it.

use super::passage::Passing;
use super::{Content, Fault, Outputs, Rules, frame};
use boss_ceres_ridley::PadAndCover;
use screens_arrival::{Preparation, Start, SuitColours};
use types::arrival::{ArrivalStage, ArrivalState, FadeFrames};
use types::brightness::{Blanking, Brightness, Level};
use types::flight::Visibility;
use world_doors::Finishing;

/// The arrival, carried frame to frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    /// The arrival's own State: its stage, the screen's brightness and the fade's count.
    pub arrival: ArrivalState,
    /// Whether the preparation's frame of work has begun.
    pub preparation: Begun,
}

/// Whether a new game's start is made ready this frame (the start's loading's, with the
/// platform's lag).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Readiness {
    /// Not on this frame.
    NotYet,
    /// It is ready this frame.
    Ready,
}

/// Whether the one frame of work that makes the start ready has begun.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Begun {
    /// It begins as the next frame of work does.
    Not,
    /// It is under way: no other frame of work begins until the start is ready.
    UnderWay,
}

impl State {
    /// Play: the arrival is over, the picture fully up.
    pub const PLAYING: Self = Self {
        arrival: ArrivalState {
            stage: ArrivalStage::Playing,
            brightness: types::brightness::Brightness::FULLY_UP,
            count: types::arrival::FadeFrames::NONE,
        },
        preparation: Begun::UnderWay,
    };

    /// Enter the new game's preparation with the brightness the flight left behind.
    pub(super) fn after_flight(brightness: types::flight::Brightness) -> Self {
        Self {
            arrival: ArrivalState {
                stage: ArrivalStage::Preparing,
                brightness: Brightness {
                    level: Level::new(brightness.level.get()).unwrap_or(Level::BLACK),
                    blanking: match brightness.visibility {
                        Visibility::Blanked => Blanking::Blanked,
                        Visibility::Shown => Blanking::Shown,
                    },
                },
                count: FadeFrames::NONE,
            },
            preparation: Begun::Not,
        }
    }

    /// Whether the start is still being made ready, so that no frame of play runs.
    #[must_use]
    pub fn preparing(&self) -> bool {
        self.arrival.stage == ArrivalStage::Preparing
    }

    /// The preparation's single frame of work draws once. Its waits can finish additional
    /// frames without drawing again.
    pub(super) fn prepare_frame(
        &mut self,
        frame: frame::State,
        begun: bool,
        finishing: Finishing,
        borrowed_random: Option<types::RandomSequence>,
    ) -> frame::State {
        let random = if begun && self.preparation == Begun::Not {
            self.preparation = Begun::UnderWay;
            frame::draw(borrowed_random.unwrap_or(frame.random))
        } else {
            frame.random
        };
        frame.after(finishing, random)
    }

    /// The arrival model's own State and outputs for one frame.
    pub(super) fn advance(
        &mut self,
        preparation: Preparation<'_>,
        rules: screens_arrival::Rules,
    ) -> screens_arrival::Report {
        screens_arrival::step(&mut self.arrival, preparation, rules)
    }

    /// The fade's step after a frame of play (`PictureBrightens`, then `PlayBegins`); once she
    /// plays it changes nothing.
    pub(super) fn after_play(mut self, rules: &Rules) -> Self {
        self.advance(Preparation::Underway, rules.arrival);
        self
    }
}

impl super::State {
    /// A frame while the start is made ready: the random number is drawn as the preparation's
    /// frame of work begins, and the counts of finished frames follow the platform's lag. On
    /// the frame the start is ready she arrives (`ArrivesAtTheTop`).
    pub(super) fn prepare(
        &mut self,
        passing: &Passing<'_>,
        content: &Content,
        rules: &Rules,
    ) -> Result<Outputs, Fault> {
        let start = Start {
            x: content.start_x,
            suit: suit(content, passing)?,
            goal: types::Palette(self.palette.goal),
        };
        let report = self.arrival.advance(
            if passing.start == Readiness::Ready {
                Preparation::Ready(&start)
            } else {
                Preparation::Underway
            },
            rules.arrival,
        );
        if let Some(arrived) = report.arrival {
            let samus = &mut self.samus.movement.0;
            samus.position.x = types::fixed(arrived.place.x, 0);
            samus.position.y = types::fixed(arrived.place.y, 0);
            samus.pose.facing = arrived.facing;
            self.station.pad_and_cover =
                PadAndCover::arrival(arrived.place, &rules.ceres).pad_and_cover;
            self.palette.screen = arrived.colours.0;
            self.hud.0.mini_map = interface_hud::MiniMapPower::On;
        }
        // The preparation's own waits for the frame's end are the frames it finishes.
        let doors = world_doors::Report {
            finishing: frame::finishing(passing.work_done),
            ..world_doors::Report::play()
        };
        self.frame = self.arrival.prepare_frame(
            self.frame,
            passing.work_begun,
            doors.finishing,
            passing.random,
        );
        Ok(Outputs {
            movement: None,
            sounds: Vec::new(),
            doors,
            health: None,
            hud: None,
            ceres_doors: Vec::new(),
            lights: None,
            ridley: None,
            ridley_placed: None,
            quake: None,
        })
    }
}

/// Her suit's colours as the room's loading chooses them.
///
/// # Errors
/// [`Fault::SuitColours`] for a colour of more than 15 bits or a palette short of sixteen.
fn suit(content: &Content, passing: &Passing<'_>) -> Result<SuitColours, Fault> {
    let colours: Option<Vec<_>> =
        crate::production::room_loading::suit(&content.rooms, &passing.progress.inventory)
            .0
            .iter()
            .map(|colour| types::Colour::new(colour.0))
            .collect();
    colours
        .and_then(|colours| colours.try_into().ok())
        .map(SuitColours)
        .ok_or(Fault::SuitColours)
}

#[cfg(test)]
mod tests {
    use super::{Begun, Readiness, State};
    use crate::production::played::passage::{PassageAudio, Passing};
    use crate::production::played::tests::{content, hud_frame, inputs, rules, state};
    use core::num::Wrapping;
    use types::Parts;
    use types::arrival::{ArrivalStage, ArrivalState, FadeFrames};
    use types::brightness::{Blanking, Brightness, Level};

    const DARK: Brightness = Brightness {
        level: Level::BLACK,
        blanking: Blanking::Blanked,
    };

    /// The State as a new game's preparation begins: dark, the preparation's work not begun.
    fn preparing() -> crate::production::played::State {
        let mut start = state(Vec::new());
        start.arrival = State {
            arrival: ArrivalState {
                stage: ArrivalStage::Preparing,
                brightness: DARK,
                count: FadeFrames::NONE,
            },
            preparation: Begun::Not,
        };
        start.frame.random.number = types::RandomNumber(0x1234);
        start
    }

    fn frame(
        state: &mut crate::production::played::State,
        work_begun: bool,
        work_done: bool,
        start: Readiness,
    ) {
        let progress = run_progress::Progress::default();
        let passing = Passing {
            work_begun,
            work_done,
            audio: PassageAudio {
                sounds_sent: true,
                music_ready: true,
            },
            progress: &progress,
            random: None,
            start,
        };
        state
            .step(
                &inputs(types::Buttons::default()),
                &passing,
                &hud_frame(),
                &content(),
                &rules(),
            )
            .unwrap();
    }

    #[test]
    fn the_preparation_draws_the_random_number_once_and_its_own_waits_count_as_finished_frames() {
        let mut dark = preparing();
        let before = dark.clone();
        // The intro's last frame of work, overrunning: nothing begins.
        frame(&mut dark, false, true, Readiness::NotYet);
        assert_eq!(dark.frame.random, before.frame.random);
        frame(&mut dark, true, false, Readiness::NotYet);
        let drawn = dark.frame.random;
        assert_ne!(drawn, before.frame.random);
        // Its own waits for the frame's end begin no frame of work.
        frame(&mut dark, true, true, Readiness::NotYet);
        assert_eq!(dark.frame.random, drawn);
        assert_eq!(dark.frame.finished_frames, 2);
        assert_eq!(dark.arrival.arrival, before.arrival.arrival);
        assert_eq!(dark.samus, before.samus);
        assert_eq!(dark.palette, before.palette);
    }

    #[test]
    fn as_the_start_is_ready_she_arrives_on_the_pad_with_her_suits_colours_and_the_fade_follows() {
        let mut content = content();
        content.rooms.samus.suits.power = data::assets::art::Palette(
            [data::assets::art::Colour(0x0123); data::assets::art::ROW_COLOURS],
        );
        let mut arriving = preparing();
        arriving.palette.goal = [types::Colour::new(0x7fff).unwrap(); world_doors::COLOURS];
        arriving.hud.0.mini_map = interface_hud::MiniMapPower::Off;
        let progress = run_progress::Progress::default();
        let passing = Passing {
            work_begun: true,
            work_done: true,
            audio: PassageAudio {
                sounds_sent: true,
                music_ready: true,
            },
            progress: &progress,
            random: None,
            start: Readiness::Ready,
        };
        arriving
            .step(
                &inputs(types::Buttons::default()),
                &passing,
                &hud_frame(),
                &content,
                &rules(),
            )
            .unwrap();
        let her = arriving.samus.movement.0.position;
        assert_eq!(
            (her.x.pixel(), her.y.pixel()),
            (Wrapping(0x80), Wrapping(0))
        );
        assert_eq!(arriving.samus.movement.0.pose.facing, types::Facing::Screen);
        assert!(arriving.station.pad_and_cover.holds_her());
        assert_eq!(arriving.hud.0.mini_map, interface_hud::MiniMapPower::On);
        let screen = arriving.palette.screen;
        assert_eq!(screen[191].bits(), 0x7fff);
        assert!(
            screen[192..208]
                .iter()
                .all(|colour| colour.bits() == 0x0123)
        );
        assert_eq!(screen[223], types::Colour::BLACK);
        assert_eq!(
            arriving.palette.goal[200].bits(),
            0x7fff,
            "the goal is kept"
        );
        assert_eq!(arriving.arrival.arrival.stage, ArrivalStage::FadingIn);
        assert_eq!(arriving.arrival.arrival.brightness, DARK);
        // Each frame of the fade in is a frame of play, then the fade's step: a level every two.
        for _ in 0..29 {
            frame(&mut arriving, true, true, Readiness::NotYet);
        }
        assert_eq!(arriving.arrival.arrival.stage, ArrivalStage::FadingIn);
        assert_eq!(
            arriving.arrival.arrival.brightness.level,
            Level::new(14).unwrap()
        );
        frame(&mut arriving, true, true, Readiness::NotYet);
        assert_eq!(arriving.arrival.arrival.stage, ArrivalStage::Playing);
        assert_eq!(arriving.arrival.arrival.brightness, Brightness::FULLY_UP);
    }
}
