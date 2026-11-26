//! The escape countdown (§5.1): the time left in an escape, the word TIME and `m'ss"cc` shown
//! over the room rather than in the status bar's cells.
//!
//! The escape's scene starts it at a time it names. It holds still in the middle of the screen
//! for its first 16 frames, then runs down there, one or two hundredths a frame by the rhythm,
//! and after its 96th frame slides up and right into the screen's top right corner, where it
//! stays. It runs on the frames whose scene runs it and stands still on every other, shown as it
//! stands. When its time runs out it says so; what the escape does then is the scene's.

use enum_map::{Enum, EnumMap};
use serde::{Deserialize, Serialize};
use types::Pair;

/// Spec: CountdownWaits. The frames a countdown just begun stands still before it runs down.
const STILL_FRAMES: u8 = 16;
/// Spec: CountdownSlides. The age at which it leaves the middle of the screen for its corner;
/// its age counts up to this and no further.
const SLIDE_AGE: u8 = 96;
/// The 1/256ths of a pixel in a whole pixel: a place is held in them.
const SUBPIXELS_PER_PIXEL: u16 = 256;
/// Spec: CountdownBegins. Where a countdown appears: the middle of the screen, (128, 128) whole
/// pixels. No place lies further left or further down.
const MIDDLE: CountdownPlace = CountdownPlace::at_pixels(128, 128);
/// Spec: CountdownSlides. Where the slide stops: the screen's top right corner, (220, 48) whole
/// pixels. No place lies further right or further up.
const CORNER: CountdownPlace = CountdownPlace::at_pixels(220, 48);
/// Spec: CountdownSlides. How far it slides each frame, in 1/256ths of a pixel: a little under a
/// pixel right (`x`) and up (`y`).
const SLIDE_STEP: Pair<u16> = Pair { x: 224, y: 193 };

/// Spec: CountdownTicks. The rhythm's beats: 128, four rows of 32.
const BEATS: usize = 128;

/// Spec: CountdownTicks. A beat of the rhythm: the count of finished frames mod 128, so 0 to
/// 127.
///
/// The rhythm has a cell for each of the 128 beats (`RHYTHM` is keyed by `Beat`), and every count
/// of finished frames falls on one of them, so every count has its hundredths: the lookup can't
/// miss. Only the count's remainder is read, so its wrapping at 16 bits changes nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Beat(usize);

impl Beat {
    /// The beat a running frame falls on: the count of finished frames mod 128.
    fn of(finished: FinishedFrames) -> Self {
        Self::wrapped(usize::from(finished.0))
    }

    /// The beat `count` falls on, mod 128.
    const fn wrapped(count: usize) -> Self {
        Self(count % BEATS)
    }
}

/// The beats key the rhythm, a table with one cell for each.
impl Enum for Beat {
    type Array<V> = [V; BEATS];

    fn from_usize(value: usize) -> Self {
        Self::wrapped(value)
    }

    fn into_usize(self) -> usize {
        self.0
    }
}

/// Spec: CountdownTicks. The rhythm: the hundredths a running frame takes, by its beat, in the
/// spec's four rows of 32 (beats 0 to 31, 32 to 63, 64 to 95, 96 to 127). Each row takes 53
/// hundredths but the last, which takes 54: 213 over 128 frames, as near as whole hundredths come
/// to 100 every 60 frames.
const RHYTHM: EnumMap<Beat, u8> = EnumMap::from_array([
    1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 1, 2, 2, 1, 2, 2, 1, 2,
    1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 1, 2, 2, 1, 2, 2, 1, 2,
    1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 1, 2, 2, 1, 2, 2, 1, 2,
    1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 1, 2, 2, 2, 1, 2, 2,
]);

/// The hundredths in a second.
const HUNDREDTHS_PER_SECOND: u32 = 100;
/// The hundredths in a minute.
const HUNDREDTHS_PER_MINUTE: u32 = 6000;
/// The most minutes a time left shows.
const MOST_MINUTES: u32 = 99;
/// The most seconds a time left shows, past its whole minutes.
const MOST_SECONDS: u8 = 59;
/// The most hundredths a time left shows, past its whole seconds.
const MOST_HUNDREDTHS: u8 = 99;

/// The time left in an escape: minutes 0 to 99, seconds 0 to 59 and hundredths 0 to 99.
///
/// It is held as one count of hundredths, `t = m × 6000 + s × 100 + c`, so the minutes and
/// seconds turn over as the hundredths do.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u32", into = "u32")]
pub struct TimeLeft(u32);

impl TimeLeft {
    /// No time left: 0'00"00, where a countdown that runs out stops.
    pub const RUN_OUT: Self = Self(0);

    /// The value left at `minutes'seconds"hundredths`, if it is one: minutes at most 99, seconds
    /// at most 59, hundredths at most 99.
    #[must_use]
    pub fn new(minutes: u8, seconds: u8, hundredths: u8) -> Option<Self> {
        if seconds > MOST_SECONDS || hundredths > MOST_HUNDREDTHS {
            return None;
        }
        let total = u32::from(minutes)
            .saturating_mul(HUNDREDTHS_PER_MINUTE)
            .saturating_add(u32::from(seconds).saturating_mul(HUNDREDTHS_PER_SECOND))
            .saturating_add(u32::from(hundredths));
        Self::in_hundredths(total)
    }

    /// The time of this many hundredths, if it is one: at most 99'59"99.
    #[must_use]
    pub const fn in_hundredths(total: u32) -> Option<Self> {
        if total / HUNDREDTHS_PER_MINUTE <= MOST_MINUTES {
            Some(Self(total))
        } else {
            None
        }
    }

    /// The count of hundredths it holds, `t`.
    #[must_use]
    pub const fn hundredths(self) -> u32 {
        self.0
    }

    /// The time as it is shown, `[minutes, seconds, hundredths]`.
    #[must_use]
    pub const fn shown(self) -> [u32; 3] {
        [
            self.0 / HUNDREDTHS_PER_MINUTE,
            self.0 % HUNDREDTHS_PER_MINUTE / HUNDREDTHS_PER_SECOND,
            self.0 % HUNDREDTHS_PER_SECOND,
        ]
    }

    /// Spec: CountdownTicks, CountdownRunsOut. A running frame takes the rhythm's hundredths for
    /// the count of finished frames. While the time stays above 0 it goes on; taking its last
    /// hundredth, or more than are left, stops it at 0'00"00, not below, and it has run out. A
    /// time already at 0'00"00 runs out again.
    fn tick(&mut self, finished: FinishedFrames) -> CountdownExpiry {
        self.0 = self.0.saturating_sub(hundredths_taken(finished));
        if *self == Self::RUN_OUT {
            CountdownExpiry::Expired
        } else {
            CountdownExpiry::NoExpiry
        }
    }
}

impl TryFrom<u32> for TimeLeft {
    type Error = &'static str;

    fn try_from(total: u32) -> Result<Self, Self::Error> {
        Self::in_hundredths(total).ok_or("a time left is at most 99'59\"99")
    }
}

impl From<TimeLeft> for u32 {
    fn from(time: TimeLeft) -> Self {
        time.hundredths()
    }
}

/// Spec: CountdownTicks. The hundredths the rhythm takes on a running frame: its beat's cell.
fn hundredths_taken(finished: FinishedFrames) -> u32 {
    u32::from(RHYTHM[Beat::of(finished)])
}

/// A countdown's age: the frames it has run since it began, 0 to 96, counted up to 96 and no
/// further.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct CountdownAge(u8);

/// Where a countdown's age puts it: it waits, runs down in the middle, or slides to its corner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    /// Spec: CountdownWaits. Its first 16 frames: it stands still.
    Waiting,
    /// Spec: CountdownTicks. Until its 96th frame: it runs down in the middle of the screen.
    InTheMiddle,
    /// Spec: CountdownSlides. From then: it runs down sliding to its corner, or resting there.
    Sliding,
}

impl CountdownAge {
    /// A countdown just begun: age 0.
    pub const BEGUN: Self = Self(0);

    /// The age of this many frames, if it is one: at most 96.
    #[must_use]
    pub const fn new(frames: u8) -> Option<Self> {
        if frames <= SLIDE_AGE {
            Some(Self(frames))
        } else {
            None
        }
    }

    /// Its frames.
    #[must_use]
    pub const fn frames(self) -> u8 {
        self.0
    }

    /// Where this age puts a countdown, as a frame begins.
    const fn stage(self) -> Stage {
        if self.0 < STILL_FRAMES {
            Stage::Waiting
        } else if self.0 < SLIDE_AGE {
            Stage::InTheMiddle
        } else {
            Stage::Sliding
        }
    }

    /// One running frame older: `+ 1` while below 96, and 96 after.
    const fn older(self) -> Self {
        if self.0 < SLIDE_AGE {
            Self(self.0.saturating_add(1))
        } else {
            self
        }
    }
}

impl TryFrom<u8> for CountdownAge {
    type Error = &'static str;

    fn try_from(frames: u8) -> Result<Self, Self::Error> {
        Self::new(frames).ok_or("a countdown's age is at most 96 frames")
    }
}

impl From<CountdownAge> for u8 {
    fn from(age: CountdownAge) -> Self {
        age.frames()
    }
}

/// A countdown's place on the screen, from its top left, in 1/256ths of a pixel: x from the
/// middle's 128 to the corner's 220 whole pixels, y from the corner's 48 to the middle's 128.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Pair<u16>", into = "Pair<u16>")]
pub struct CountdownPlace(Pair<u16>);

impl CountdownPlace {
    /// The place at these whole pixels.
    const fn at_pixels(x: u16, y: u16) -> Self {
        Self(Pair {
            x: x.saturating_mul(SUBPIXELS_PER_PIXEL),
            y: y.saturating_mul(SUBPIXELS_PER_PIXEL),
        })
    }

    /// The place at these 1/256ths of a pixel, if it is one: between the middle of the screen
    /// and its top right corner, those included.
    #[must_use]
    pub fn new(subpixels: Pair<u16>) -> Option<Self> {
        let across = (MIDDLE.0.x..=CORNER.0.x).contains(&subpixels.x);
        let down = (CORNER.0.y..=MIDDLE.0.y).contains(&subpixels.y);
        (across && down).then_some(Self(subpixels))
    }

    /// Its x and y in 1/256ths of a pixel.
    #[must_use]
    pub const fn subpixels(self) -> Pair<u16> {
        self.0
    }

    /// Spec: CountdownShown. Its x and y in whole screen pixels, each 1/256ths count `div 256`.
    #[must_use]
    pub const fn whole_pixels(self) -> Pair<u16> {
        Pair {
            x: self.0.x / SUBPIXELS_PER_PIXEL,
            y: self.0.y / SUBPIXELS_PER_PIXEL,
        }
    }

    /// Spec: CountdownSlides. One frame's slide: right and up by its step, each axis stopping at
    /// the corner on its own.
    fn slid(self) -> Self {
        Self(Pair {
            x: self.0.x.saturating_add(SLIDE_STEP.x).min(CORNER.0.x),
            y: self.0.y.saturating_sub(SLIDE_STEP.y).max(CORNER.0.y),
        })
    }
}

impl TryFrom<Pair<u16>> for CountdownPlace {
    type Error = &'static str;

    fn try_from(subpixels: Pair<u16>) -> Result<Self, Self::Error> {
        Self::new(subpixels).ok_or("a countdown's place lies between the middle and the corner")
    }
}

impl From<CountdownPlace> for Pair<u16> {
    fn from(place: CountdownPlace) -> Self {
        place.subpixels()
    }
}

/// A countdown that has begun: its time left, its age and its place, each set and read on its
/// own.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BegunCountdown {
    /// The time left.
    pub time_left: TimeLeft,
    /// The frames it has run since it began, up to 96.
    pub age: CountdownAge,
    /// Its place on the screen.
    pub place: CountdownPlace,
}

impl BegunCountdown {
    /// Spec: CountdownBegins. A countdown begun at `start`: its time left `m'ss"00`, age 0, in
    /// the middle of the screen.
    const fn begun_at(start: StartTime) -> Self {
        Self {
            time_left: start.0,
            age: CountdownAge::BEGUN,
            place: MIDDLE,
        }
    }

    /// A frame whose scene runs the countdown. The age, the time and the place each change from
    /// their values as the frame begins, so the frame the age reaches 96 runs the time but
    /// doesn't move the place.
    ///
    /// - Spec: CountdownWaits. In its first 16 frames only its age goes on; it never runs out.
    /// - Spec: CountdownTicks. After them each frame takes the rhythm's hundredths, and its age
    ///   goes on up to 96.
    /// - Spec: CountdownSlides. From age 96 it also slides toward its corner.
    fn run(&mut self, finished: FinishedFrames) -> CountdownExpiry {
        let expiry = match self.age.stage() {
            Stage::Waiting => CountdownExpiry::NoExpiry,
            Stage::InTheMiddle => self.time_left.tick(finished),
            Stage::Sliding => {
                self.place = self.place.slid();
                self.time_left.tick(finished)
            }
        };
        self.age = self.age.older();
        expiry
    }
}

/// The current countdown, part of the HUD's State.
///
/// No other input of the HUD, nor her energy or her protection from hits, replaces it; a step
/// outside play leaves it as it stands. Nothing in this spec clears a countdown once begun.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Countdown {
    /// Spec: EscapeCountdown. No escape countdown has begun.
    Dormant,
    /// A countdown has begun.
    Begun(BegunCountdown),
}

/// Whether the scene runs the countdown this frame: the scene's choice each frame of play,
/// which the ordinary HUD frame doesn't imply.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CountdownRuns {
    /// The scene runs the countdown this frame.
    MaintainCountdown,
    /// The scene only shows it: it stands still (spec: CountdownShown).
    PresentationOnly,
}

/// Spec: CountdownBegins. The time a countdown starts at: whole minutes 0 to 99 and seconds 0 to
/// 59.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StartTime(TimeLeft);

impl StartTime {
    /// The start at `minutes'seconds"00`, if it is one: minutes at most 99, seconds at most 59.
    #[must_use]
    pub fn new(minutes: u8, seconds: u8) -> Option<Self> {
        TimeLeft::new(minutes, seconds, 0).map(Self)
    }
}

/// Spec: CountdownTicks. The count of frames the game has finished, as the last frame left it.
///
/// A lag frame isn't one. It is 16 bits, wrapping, and never starts again, at a door or
/// anywhere, unlike the HUD's frame count. Only its remainder mod 128 is read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FinishedFrames(pub u16);

/// What the countdown reads in a frame of play (§5.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CountdownInputs {
    /// Whether the scene runs the countdown this frame, or only shows it.
    pub runs: CountdownRuns,
    /// The countdown starts this frame at this time, an event raised by the escape's scene; or
    /// `None`.
    pub start: Option<StartTime>,
    /// The count of finished frames.
    pub finished: FinishedFrames,
}

/// Whether the escape's time has run out this frame, read by the escape's scene.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CountdownExpiry {
    /// The escape's time hasn't run out this frame.
    NoExpiry,
    /// Spec: CountdownRunsOut. It has: this frame took its last hundredth, or it was already out.
    Expired,
}

/// Spec: CountdownShown. The countdown's picture the frame gives the room's presentation.
///
/// It is apart from the status bar's cells. Which figures draw it, and where each sits around
/// its place, are the importer's; the sprite encoder draws it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CountdownPicture {
    /// No countdown has begun: no picture.
    Absent,
    /// The countdown as the frame leaves it.
    Shown {
        /// Its place in whole screen pixels.
        place: Pair<u16>,
        /// The time left it shows.
        time: TimeLeft,
    },
}

/// The countdown's answer for a frame of play: one expiry answer and one picture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CountdownAnswer {
    /// Whether the escape's time has run out.
    pub expiry: CountdownExpiry,
    /// The countdown's picture.
    pub picture: CountdownPicture,
}

/// A frame of play the spec leaves out, which the HUD refuses before any effect. (A scene whose
/// choice isn't known is the other: `CountdownRuns` holds no value for it.)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnsupportedFrame {
    /// The countdown starts on a frame whose scene only shows it.
    StartWhileOnlyShown,
}

impl Countdown {
    /// The countdown's turn in a frame of play: whether the scene runs it or only shows it is
    /// decided here, once. A start comes before the frame's countdown and takes its place, so a
    /// countdown that begins doesn't run in the frame it begins. The expiry answer is the time's;
    /// the picture shows the countdown as the frame leaves it.
    ///
    /// - Spec: EscapeCountdown. Before a countdown begins, a running frame leaves it dormant.
    /// - Spec: CountdownBegins. A start, whatever the current countdown, begins it again.
    /// - Spec: CountdownShown. A frame that only shows it leaves it standing, its time, age and
    ///   place carried to the next frame.
    ///
    /// # Errors
    ///
    /// `UnsupportedFrame::StartWhileOnlyShown` for a start on a frame that only shows the
    /// countdown; nothing is changed.
    pub fn play(&mut self, inputs: CountdownInputs) -> Result<CountdownAnswer, UnsupportedFrame> {
        let expiry = match (inputs.runs, inputs.start) {
            (CountdownRuns::PresentationOnly, Some(_)) => {
                return Err(UnsupportedFrame::StartWhileOnlyShown);
            }
            (CountdownRuns::PresentationOnly, None) => CountdownExpiry::NoExpiry,
            (CountdownRuns::MaintainCountdown, Some(start)) => {
                *self = Self::Begun(BegunCountdown::begun_at(start));
                CountdownExpiry::NoExpiry
            }
            (CountdownRuns::MaintainCountdown, None) => match self {
                Self::Dormant => CountdownExpiry::NoExpiry,
                Self::Begun(countdown) => countdown.run(inputs.finished),
            },
        };
        Ok(CountdownAnswer {
            expiry,
            picture: self.picture(),
        })
    }

    /// Spec: CountdownShown. A begun countdown is shown at its place, in whole pixels, with its
    /// time left; a dormant one has no picture.
    const fn picture(self) -> CountdownPicture {
        match self {
            Self::Dormant => CountdownPicture::Absent,
            Self::Begun(countdown) => CountdownPicture::Shown {
                place: countdown.place.whole_pixels(),
                time: countdown.time_left,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Beat, FinishedFrames, RHYTHM, hundredths_taken};

    /// The spec's sums: each row takes 53 hundredths but the last, which takes 54; 213 in all.
    #[test]
    fn the_rhythm_takes_213_hundredths_over_128_frames() {
        let cells: Vec<u32> = RHYTHM.values().map(|&h| u32::from(h)).collect();
        assert_eq!(cells.len(), 128);
        let rows: Vec<u32> = cells.chunks(32).map(|row| row.iter().sum()).collect();
        assert_eq!(rows, [53, 53, 53, 54]);
        assert_eq!(cells.iter().sum::<u32>(), 213);
    }

    /// Every count of finished frames falls on a beat, its remainder mod 128, so the count's
    /// wrap from 65,535 to 0 is a beat like any other.
    #[test]
    fn every_count_has_a_beat_and_its_hundredths() {
        for count in 0..=u16::MAX {
            let beat = Beat::of(FinishedFrames(count));
            assert_eq!(beat, Beat(usize::from(count % 128)));
            assert_eq!(
                hundredths_taken(FinishedFrames(count)),
                u32::from(RHYTHM[beat])
            );
        }
    }
}
