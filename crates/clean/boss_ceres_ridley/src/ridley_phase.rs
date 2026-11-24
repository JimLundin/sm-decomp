//! Where Ridley is from his placing to his take-off, and how each run moves him on: he holds
//! through the door's passage, waits in the dark and wakes; his eye lights up and his body fades
//! in; tangible, he readies his roar, roars, and takes off.

use core::num::NonZeroU16;

use crate::ridley_wait::RidleyWait;
use crate::roar::{RidleyLook, RoarFigure};
use crate::rules::Rules;
use crate::waking::{BodyColours, EyeColours, ReadyWait, RidleyContent, RoarWait};

/// Where he is, with what each phase carries. Listed alphabetically; the order means nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RidleyPhase {
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
        steps: NonZeroU16,
    },
    /// His roar over, he takes off: from this run on his next part runs him.
    Flying,
    /// Placed, the door's passage into his room still under way.
    Holding,
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

/// What one run of his phase leaves: the phase and his look, and what it hands on.
pub struct PhaseRun {
    /// The phase as the run leaves it.
    pub phase: RidleyPhase,
    /// His look; none from his take-off on, when his next part gives it.
    pub look: Option<RidleyLook>,
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
            look: Some(RidleyLook::Rest),
            eye: None,
            body: None,
            music: None,
            roar: None,
        }
    }

    /// `Roars`: his take-off: his figure and all he does from this run on are his next part's.
    const TAKEN_OFF: Self = Self {
        look: None,
        ..Self::at_rest(RidleyPhase::Flying)
    };
}

impl RidleyPhase {
    /// Whether her touch and her shots meet him: from the run he takes his body's last set on.
    /// Before, they go through him.
    #[must_use]
    pub const fn tangible(self) -> bool {
        matches!(
            self,
            Self::ReadyToRoar { .. } | Self::Roaring { .. } | Self::Flying
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
                | Self::Flying
        )
    }

    /// Whether he is roaring: `Roaring` with the roar's figure 2, 3 or 4.
    #[must_use]
    pub const fn roaring(self) -> bool {
        matches!(self, Self::Roaring { figure, .. } if figure.roaring())
    }

    /// One run of his phase, reading whether a door's passage is under way, his content and
    /// the rules.
    ///
    /// `RidleyWaits`: he holds while a door's passage is under way; once it is over he starts
    /// his wait, the first frame counted, and counts it down a run at a time; on the run he
    /// finds it at 0 he wakes. `ReadyToRoar`: tangible, he counts his wait down the same way,
    /// and on the run he finds it at 0 starts his roar, its first figure, with the roar's wait.
    pub(crate) fn run(
        self,
        passage_under_way: bool,
        content: &RidleyContent,
        rules: &Rules,
    ) -> PhaseRun {
        match self {
            Self::Holding if passage_under_way => PhaseRun::at_rest(Self::Holding),
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
            Self::Roaring { wait, figure } => roar_step(wait, figure),
            Self::Flying => PhaseRun::TAKEN_OFF,
        }
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
                steps: NonZeroU16::MIN.saturating_add(shown),
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
fn roar_step(wait: RoarWait, figure: RoarFigure) -> PhaseRun {
    wait.counted_down().map_or(PhaseRun::TAKEN_OFF, |wait| {
        let figure = figure.ran();
        PhaseRun {
            look: Some(figure.look()),
            roar: figure.heard().then_some(RoarSound),
            ..PhaseRun::at_rest(RidleyPhase::Roaring { wait, figure })
        }
    })
}
