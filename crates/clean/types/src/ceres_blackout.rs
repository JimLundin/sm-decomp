//! The blackout in the Ceres shaft: the State the scene carries from the report that Samus
//! reached the elevator to the dispatch of the station explosion, saved and restored whole.
//!
//! The scene holds a work pass of the scheduler as either finished or prepared: a prepared pass
//! has run its room work and holds the picture that work drew until the display publishes it.

use serde::{Deserialize, Serialize};

/// Spec: `ElevatorStopsSamus`. The platform wait, 60 room work passes.
const WHOLE_WAIT: u8 = 60;
/// The brightest lit level, 15.
const BRIGHTEST: u8 = 15;

/// Spec: `PlatformWait`. Room work passes left of the wait on the platform: 1 to 60.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct WaitPasses(u8);

impl WaitPasses {
    /// Spec: `ElevatorStopsSamus`. The whole wait, 60 passes.
    pub const WHOLE: Self = Self(WHOLE_WAIT);

    /// The wait of this many passes, if it is one (1 to 60).
    #[must_use]
    pub const fn new(passes: u8) -> Option<Self> {
        if passes >= 1 && passes <= WHOLE_WAIT {
            Some(Self(passes))
        } else {
            None
        }
    }

    /// Its passes.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }

    /// One pass off the wait: the passes still to wait, or none when the wait ends.
    #[must_use]
    pub const fn counted_down(self) -> Option<Self> {
        Self::new(self.0.saturating_sub(1))
    }
}

impl TryFrom<u8> for WaitPasses {
    type Error = &'static str;

    fn try_from(passes: u8) -> Result<Self, Self::Error> {
        Self::new(passes).ok_or("a platform wait must be 1 to 60 passes")
    }
}

impl From<WaitPasses> for u8 {
    fn from(wait: WaitPasses) -> Self {
        wait.get()
    }
}

/// A lit level of the shaft's light: 1 (dimmest) to 15 (full).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct BlackoutLevel(u8);

impl BlackoutLevel {
    /// Full light, level 15.
    pub const FULL: Self = Self(BRIGHTEST);

    /// The level of this number, if it is a lit one (1 to 15).
    #[must_use]
    pub const fn new(level: u8) -> Option<Self> {
        if level >= 1 && level <= BRIGHTEST {
            Some(Self(level))
        } else {
            None
        }
    }

    /// Its number, 1 to 15.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }

    /// The lit level one below, or none below level 1.
    #[must_use]
    pub const fn lower(self) -> Option<Self> {
        Self::new(self.0.saturating_sub(1))
    }
}

impl TryFrom<u8> for BlackoutLevel {
    type Error = &'static str;

    fn try_from(level: u8) -> Result<Self, Self::Error> {
        Self::new(level).ok_or("a lit level must be 1 to 15")
    }
}

impl From<BlackoutLevel> for u8 {
    fn from(level: BlackoutLevel) -> Self {
        level.get()
    }
}

/// The shaft's light: lit at a level, or forced blank (nothing shown, read as level 0).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlackoutLight {
    /// The display shows nothing.
    ForcedBlank,
    /// The picture shows at this level.
    Lit(BlackoutLevel),
}

impl BlackoutLight {
    /// The picture at full light.
    pub const FULL: Self = Self::Lit(BlackoutLevel::FULL);
}

/// The escape timer's remaining time, in hundredths of a second: its digits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimerCentiseconds(pub u32);

/// Whether the escape timer is still active: drawn, and counted by the timer's owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimerActivity {
    /// Drawn each pass.
    Active,
    /// Retired at the end of the save: no longer drawn, its digits kept.
    Retired,
}

/// Spec: `ElevatorStopsSamus`, `DepartureHandoff`. The escape timer as the scene holds it: the
/// time it froze at and whether it is still active.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlackoutTimer {
    /// Its digits: the time it shows.
    pub time: TimerCentiseconds,
    /// Whether it is still active.
    pub activity: TimerActivity,
}

/// Spec: `DepartureHandoff`. The rotating shaft's stage: running until the save completes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShaftStage {
    /// The shaft rotates and runs its actors, platform and beacon.
    Rotating,
    /// Cleared at the end of the save.
    Cleared,
}

/// A sprite of the timer's drawing, by its identity in the drawing owner's content.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimerSprite(pub u16);

/// The picture a room work pass drew: the room writes the display carries, and the timer's
/// sprites among them.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreparedPicture {
    /// The timer's sprites, in drawing order.
    pub timer_sprites: Vec<TimerSprite>,
}

/// The picture the display last published: its light and the timer's sprites it shows.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishedPicture {
    /// The light it shows at.
    pub light: BlackoutLight,
    /// The timer's sprites it shows, in drawing order.
    pub timer_sprites: Vec<TimerSprite>,
}

/// Spec: `PlatformWait`. The scheduler's current work pass.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkPass {
    /// The last pass was published; the next room work begins a new one.
    Finished,
    /// Room work has run and its picture waits for the display's publication.
    Prepared(PreparedPicture),
}

/// Spec: `DepartureHandoff`. The destination the scene carries to the save and the explosion,
/// apart from the area and station she is in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlightRoute {
    /// The gunship's flight to Zebes.
    ZebesFlight,
}

/// Spec: `DepartureHandoff`. The scheduler's count of finished work passes, wrapping from 65,535
/// to 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinishedPasses(pub u16);

impl FinishedPasses {
    /// One more pass finished.
    #[must_use]
    pub const fn counted_up(self) -> Self {
        Self(self.0.wrapping_add(1))
    }
}

/// Where the blackout has got to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlackoutPhase {
    /// Ordinary play in the shaft, until she reaches the elevator.
    Shaft,
    /// On the platform, with the room work passes left of the wait.
    Hold(WaitPasses),
    /// The light dims one level each room work pass.
    Fade,
    /// Forced blank, the pass suspended until the end of a vertical interval.
    AwaitBlank,
    /// The save is being written, the route carried.
    Saving(FlightRoute),
    /// The station explosion is pending with its route, dispatched once the final room
    /// picture is published.
    Explosion(FlightRoute),
    /// The explosion has been dispatched and owns the scene.
    Dispatched(FlightRoute),
}

/// Everything the blackout remembers from one work opportunity to the next. A caller may build
/// it at any value its types allow, read it back, carry it, save it and restore it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlackoutState {
    /// Where the scene has got to, with the wait or the route it carries.
    pub phase: BlackoutPhase,
    /// The light the scene intends: what the next publication shows.
    pub light: BlackoutLight,
    /// What the display last published.
    pub published: PublishedPicture,
    /// The current work pass, with its prepared picture.
    pub pass: WorkPass,
    /// The escape timer, frozen once she stops.
    pub timer: BlackoutTimer,
    /// The shaft's stage.
    pub shaft: ShaftStage,
    /// The scheduler's count of finished work passes.
    pub finished: FinishedPasses,
}
