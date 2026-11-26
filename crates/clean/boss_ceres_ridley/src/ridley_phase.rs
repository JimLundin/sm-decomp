//! Where Ridley is from his placing to the end of his flight, and how each run before his
//! take-off moves him on: he holds through the door's passage, waits in the dark and wakes; his
//! eye lights up and his body fades in; tangible, he readies his roar, roars, and takes off. His
//! flight's phases are here too; the flight's own rules move him through them.

use core::num;

use types::GameFrames;
use types::ceres_getaway::{FlybyBeats, GetawayState};

use crate::ridley_wait::RidleyWait;
use crate::roar::{RidleyLook, RoarFigure};
use crate::rules::Rules;
use crate::waking::{BodyColours, EyeColours, ReadyWait, RemainingTurns, RidleyContent, RoarWait};

/// Frame timing as Ridley reads it: the current passage and the game clock, not his own run count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RidleyFrame {
    /// Whether the door's passage is still under way (`RidleyWaits`).
    pub passage_under_way: bool,
    /// The game clock at the start of this frame (`GetawayPicture`).
    pub finished: GameFrames,
}

/// His flight's phases, from his take-off to his hover. Listed alphabetically; the order means
/// nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlightPhase {
    /// He hovers, drifting to his point (`Hovers`).
    Hovering,
    /// He rises, faster and faster (`Rises`).
    Rising,
    /// Rising, he slows (`SlowsDown`).
    Slowing,
    /// His take-off: he crouches and hops (`TakesOff`).
    TakingOff,
}

/// The attack he turns to from his hover; from the next run on a later part runs him. Listed
/// alphabetically; the order means nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Attack {
    /// He breathes fireballs.
    Fireballs,
    /// He lunges.
    Lunge,
    /// He swoops.
    Swoop,
}

/// `FliesOutOfReach`: the first coast run leaves 63 more before the stopped run.
const AFTER_FIRST_COAST: u8 = 63;

/// The coast runs remaining, 0 through 63. Zero means the next run stops him.
pub type CoastRuns = RemainingTurns<AFTER_FIRST_COAST>;

/// Where he is, with what each phase carries. The declaration order does not determine transitions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RidleyPhase {
    /// He turns from his hover to an attack.
    Attacking(Attack),
    /// His waiting is over: his eye's fade, with no step shown yet.
    Awake,
    /// His body fading in from black, his tail and wings moving.
    BodyFading {
        /// How many of the body's colour sets it has shown, 0 to the sets' count less 1.
        sets: u16,
        /// The runs since the body last took a set, or since its fade began, 0 to `p` − 1.
        runs: u8,
    },
    /// His eye lighting up out of the dark.
    EyeFading {
        /// How many of the eye's steps it has shown, 1 to the steps' count.
        steps: num::NonZeroU16,
    },
    /// `FliesOutOfReach`: he pulls toward a spot above the room.
    Retreating,
    /// `FliesOutOfReach`: he coasts without pulling until the remaining runs are spent.
    CoastingAbove(CoastRuns),
    /// `GetawayPass`: from the prepared backdrop through the return to the room.
    Getaway(GetawayState),
    /// Her 100th hit ended his hover. A later part owns the fake retreat.
    FakeRetreat,
    /// Placed, the door's passage into his room still under way.
    Holding,
    /// His roar over, he flies: his take-off, his rise and his hover.
    InFlight(FlightPhase),
    /// Tangible, holding still a moment before he roars.
    ReadyToRoar {
        /// The runs left before the one he starts his roar on.
        wait: ReadyWait,
    },
    /// Roaring, then sitting until he takes off.
    Roaring {
        /// The runs left before the one he takes off on.
        wait: RoarWait,
        /// The roar's figure, with its runs.
        figure: RoarFigure,
    },
    /// Waiting in the dark, the passage over.
    Waiting {
        /// The runs left before the one he wakes on.
        wait: RidleyWait,
    },
}

/// A request to audio for the fight's music, on the run he turns tangible (`BodyFadesIn`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FightMusic;

/// His roar's sound, asked of audio with a queue limit of 6 on the run the roar's second figure
/// starts (`Roars`). What the queue does with a request it can't hold is audio's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoarSound;

impl RoarSound {
    /// `Roars`: the queue limit the roar is asked for with.
    pub const QUEUE_LIMIT: u8 = 6;
}

/// What one run of his phase before his take-off leaves: the phase and his look, and what it
/// hands on.
pub struct PhaseRun {
    /// The phase as the run leaves it.
    pub phase: RidleyPhase,
    /// His look.
    pub look: RidleyLook,
    /// The eye's colour set the screen takes, if any.
    pub eye: Option<EyeColours>,
    /// The body's colour set the screen takes, if any.
    pub body: Option<BodyColours>,
    /// The fight's music, on the run he turns tangible.
    pub music: Option<FightMusic>,
    /// The roar's sound, on the run it is heard.
    pub roar: Option<RoarSound>,
}

impl PhaseRun {
    /// A run into this phase with his look `Rest`, handing on nothing. `RidleyWaits`,
    /// `EyeFadesIn`: every run before his roar takes the look `Rest`.
    const fn at_rest(phase: RidleyPhase) -> Self {
        Self {
            phase,
            look: RidleyLook::Rest,
            eye: None,
            body: None,
            music: None,
            roar: None,
        }
    }
}

/// Which rules a run of him follows, by the phase it starts in.
pub enum PhaseStep {
    /// A run before his take-off, by his waking's rules, and what it leaves.
    Waking(PhaseRun),
    /// `Roars`: his roar's wait over, he takes off on this run (`TakesOff`).
    TakesOff,
    /// A run of his flight, by this phase's rule (`TakesOff`, `Rises`, `SlowsDown`, `Hovers`).
    Flies(FlightPhase),
    /// Start or continue his pull out of reach.
    PullsAway,
    /// Carry his momentum, or stop if the count is already zero.
    Coasts(CoastRuns),
    /// The flyby takes over from the airborne body.
    Flyby(GetawayState),
    /// An attack, the fake retreat, or the warning takes over.
    HandedOver,
}

impl RidleyPhase {
    /// `GetawayBackdrop`: the ordinary handoff starts both flyby beats at zero.
    pub const GETAWAY_START: Self = Self::Getaway(GetawayState::GetawayReady(FlybyBeats::START));

    /// Whether her touch and her shots meet him. They go through him before he takes his body's
    /// last colour set. He stays tangible through his flight, on the run he turns to an attack,
    /// and throughout the real retreat, including the last coast run when he stops.
    #[must_use]
    pub const fn tangible(self) -> bool {
        matches!(
            self,
            Self::ReadyToRoar { .. }
                | Self::Roaring { .. }
                | Self::InFlight(_)
                | Self::Attacking(_)
                | Self::Retreating
                | Self::CoastingAbove(_)
                | Self::Getaway(GetawayState::GetawayReady(_))
        )
    }

    /// Whether his tail and wings move, the baby drawn with them: from his body's fade on.
    #[must_use]
    pub const fn moving(self) -> bool {
        matches!(
            self,
            Self::BodyFading { .. }
                | Self::ReadyToRoar { .. }
                | Self::Roaring { .. }
                | Self::InFlight(_)
                | Self::Attacking(_)
                | Self::Retreating
                | Self::CoastingAbove(_)
                | Self::Getaway(GetawayState::GetawayReady(_))
                | Self::FakeRetreat
        )
    }

    /// `TailAtRest`: whether his tail hangs at rest: from his body's fade on, until he takes
    /// off, when his tail is his next part's.
    #[must_use]
    pub const fn tail_at_rest(self) -> bool {
        matches!(
            self,
            Self::BodyFading { .. } | Self::ReadyToRoar { .. } | Self::Roaring { .. }
        )
    }

    /// Whether he is roaring: `Roaring` with the roar's figure 2, 3 or 4.
    #[must_use]
    pub const fn roaring(self) -> bool {
        matches!(self, Self::Roaring { figure, .. } if figure.roaring())
    }

    /// Which rules a run starting in this phase follows; before his take-off, the run of his
    /// phase by them, reading whether a door's passage is under way, his content and the rules.
    ///
    /// `RidleyWaits`: he holds while a door's passage is under way; once it is over he starts
    /// his wait, the first frame counted, and counts it down a run at a time; on the run he
    /// finds it at 0 he wakes. `ReadyToRoar`: tangible, he counts his wait down the same way,
    /// and on the run he finds it at 0 starts his roar, its first figure, with the roar's wait.
    pub(crate) fn run(
        self,
        frame: RidleyFrame,
        content: &RidleyContent,
        rules: &Rules,
    ) -> PhaseStep {
        let waking = match self {
            Self::Holding if frame.passage_under_way => PhaseRun::at_rest(Self::Holding),
            Self::Holding => PhaseRun::at_rest(Self::Waiting {
                wait: rules.wake.first_count(),
            }),
            Self::Waiting { wait } => PhaseRun::at_rest(
                wait.counted_down()
                    .map_or(Self::Awake, |wait| Self::Waiting { wait }),
            ),
            Self::Awake => eye_step(0, content),
            Self::EyeFading { steps } => eye_step(steps.get(), content),
            Self::BodyFading { sets, runs } => body_step(sets, runs, content, rules),
            Self::ReadyToRoar { wait } => PhaseRun::at_rest(wait.counted_down().map_or(
                Self::Roaring {
                    wait: RoarWait::FULL,
                    figure: RoarFigure::START,
                },
                |wait| Self::ReadyToRoar { wait },
            )),
            Self::Roaring { wait, figure } => return roar_step(wait, figure),
            Self::InFlight(flight) => return PhaseStep::Flies(flight),
            Self::Retreating => return PhaseStep::PullsAway,
            Self::CoastingAbove(runs) => return PhaseStep::Coasts(runs),
            Self::Getaway(state) => return PhaseStep::Flyby(state),
            Self::Attacking(_) | Self::FakeRetreat => return PhaseStep::HandedOver,
        };
        PhaseStep::Waking(waking)
    }
}

/// `EyeFadesIn`: a run of his eye's fade with `shown` steps shown (0 as he wakes): while a step
/// is left it shows the next, its colour set on the screen, the same set as the last run's or
/// not; once every step is shown his body's fade begins, with no colours, and he moves.
fn eye_step(shown: u16, content: &RidleyContent) -> PhaseRun {
    content.eye_steps.get(shown).map_or(
        PhaseRun::at_rest(RidleyPhase::BodyFading { sets: 0, runs: 0 }),
        |colours| PhaseRun {
            eye: Some(colours),
            ..PhaseRun::at_rest(RidleyPhase::EyeFading {
                steps: num::NonZeroU16::MIN.saturating_add(shown),
            })
        },
    )
}

/// `BodyFadesIn`: a run of his body's fade, `sets` shown and `runs` since the last: every `p`
/// runs it shows the next set; with its last he turns tangible, ready with a wait of 4, and the
/// fight's music is asked for.
fn body_step(sets: u16, runs: u8, content: &RidleyContent, rules: &Rules) -> PhaseRun {
    let runs = runs.saturating_add(1);
    if runs < rules.pace.get() {
        return PhaseRun::at_rest(RidleyPhase::BodyFading { sets, runs });
    }
    let next = sets.saturating_add(1);
    if usize::from(next) < content.body_sets.count() {
        PhaseRun {
            body: content.body_sets.get(sets),
            ..PhaseRun::at_rest(RidleyPhase::BodyFading {
                sets: next,
                runs: 0,
            })
        }
    } else {
        PhaseRun {
            body: Some(content.body_sets.last()),
            music: Some(FightMusic),
            ..PhaseRun::at_rest(RidleyPhase::ReadyToRoar {
                wait: ReadyWait::FULL,
            })
        }
    }
}

/// `Roars`: a run of his roar: while its wait is over 0 it counts down and the figure moves on,
/// his look the figure's, the roar heard as its second figure starts; at 0 he takes off,
/// whatever the figure.
fn roar_step(wait: RoarWait, figure: RoarFigure) -> PhaseStep {
    wait.counted_down().map_or(PhaseStep::TakesOff, |wait| {
        let figure = figure.ran();
        PhaseStep::Waking(PhaseRun {
            look: figure.look(),
            roar: figure.heard().then_some(RoarSound),
            ..PhaseRun::at_rest(RidleyPhase::Roaring { wait, figure })
        })
    })
}
