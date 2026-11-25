//! Ceres Ridley as she enters his room, as he wakes and as he flies: placed at his place in the
//! dark with the baby in his hold, waiting a fixed while once the door's passage is over, then
//! waking: his eye lights up, his body fades in, and tangible he roars, his tail at rest behind
//! him from his body's fade on and hitting her when she comes within its reach. Then he takes
//! off, rises, slows and hovers, drawn to his point, until he gives up or turns to an attack,
//! which later parts run. His tail in flight is a later piece's.
//!
//! The caller holds a [`CeresRidley`] between frames and steps it on the frame the room places
//! him ([`CeresRidley::place`]) and on each frame he runs ([`CeresRidley::run`]): whether a
//! door's passage is under way, the random number, her as his tail reads her ([`Her`]), how the
//! fight stands ([`Fight`]) and his content are all he reads. On a frame he doesn't run the
//! caller doesn't step him, and nothing of him or the baby changes.

use core::num::Wrapping;

use types::{Hit, Pair, Px};

use crate::baby::{Baby, BabyColours, BabyCry, BabyLook};
use crate::flight::{Bound, Fight, HoverRuns, Ruled, STILL, Speed, Steered, flown};
use crate::ridley_phase::{FightMusic, FlightPhase, PhaseRun, PhaseStep, RidleyPhase, RoarSound};
use crate::roar::RidleyLook;
use crate::rules::Rules;
use crate::station::{ShakeStart, Stage};
use crate::tail::{Her, TailCurl, place_at_rest};
use crate::takeoff::{LIFT_OFF, TakeOffFigure, TakeOffStart, hopped};
use crate::waking::{BodyColours, EyeColours, RidleyContent};

/// `RidleyPlaced`: his place, whatever the room's content says.
const PLACE: Pair<Px> = Pair {
    x: Wrapping(186),
    y: Wrapping(169),
};

/// His position's fractions as he is placed: none.
const NO_FRACTION: Pair<u8> = Pair { x: 0, y: 0 };

/// Ceres Ridley in his room: the state a caller holds from one frame to the next.
///
/// He is shown and never stops her motion: the framework draws his look at his position, in
/// colours that stay black until he wakes. Until he turns tangible (`RidleyPhase::tangible`),
/// her touch and her shots go through him too.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CeresRidley {
    /// Where he is, from his placing to the end of his flight.
    pub phase: RidleyPhase,
    /// The figure he is drawn with; none before his first run.
    pub look: Option<RidleyLook>,
    /// Where he is drawn and the baby held, in whole pixels.
    pub position: Pair<Px>,
    /// His position's fractions, 0 to 255 in 1/256 px.
    pub fraction: Pair<u8>,
    /// His speed, in 1/256 px a run, x rightward and y downward; 0 as he is placed.
    pub speed: Pair<Speed>,
    /// His take-off's figure, with its runs; none before he takes off (`TakesOff`).
    pub takeoff: Option<TakeOffFigure>,
    /// The runs he has hovered without reaching his point; 0 from his placing (`Hovers`).
    pub hover_runs: HoverRuns,
    /// The baby he holds.
    pub baby: Baby,
    /// How tightly his tail curls once it moves (`TailCurls`).
    pub curl: TailCurl,
}

/// The boss a placing hands the HUD as encountered, whose mini-map rule is the HUD's own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Boss {
    /// Ridley in Ceres.
    CeresRidley,
}

/// A request to audio that the music stop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MusicStop;

/// A request to the screen's shaking that any shake end.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShakeStop;

/// What a palette's goal colours become as he is placed. Listed alphabetically; the order means
/// nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GoalColours {
    /// Black, all but its first colour.
    BlackButFirst,
    /// What his room's content holds for it.
    RoomContent,
}

/// His room's colours as he is placed: the goal colours the passage's fade brings the screen to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoomColours {
    /// The baby's palette.
    pub baby: GoalColours,
    /// The `Own` palette of his room's door.
    pub door: GoalColours,
    /// The palette he is drawn in, the doors'.
    pub ridley: GoalColours,
}

/// `RidleyPlaced`: the baby's palette and his door's own take his room's content; the palette he
/// is drawn in, the doors', turns black, all but its first colour, so only the baby is seen.
const ROOM_COLOURS: RoomColours = RoomColours {
    baby: GoalColours::RoomContent,
    door: GoalColours::RoomContent,
    ridley: GoalColours::BlackButFirst,
};

/// What the room's placing of him answers, and hands to the rest of the game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RidleyPlacing {
    /// Ridley as placed.
    pub ridley: CeresRidley,
    /// The boss encountered, for the HUD.
    pub boss: Boss,
    /// The music's stop, for audio.
    pub music: MusicStop,
    /// The shake's stop, for the screen's shaking.
    pub shake: ShakeStop,
    /// The station's stage, for progress: calm.
    pub stage: Stage,
    /// His room's colours, for the passage's fade.
    pub colours: RoomColours,
}

/// What one run of him answers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RidleyRun {
    /// Ridley as the run leaves him.
    pub ridley: CeresRidley,
    /// The baby's cry, for audio, if it starts to cry this run.
    pub cry: Option<BabyCry>,
    /// The colour set the screen takes in the baby's palette, if any.
    pub colours: Option<BabyColours>,
    /// The eye's colour set the screen takes, if any (`EyeFadesIn`).
    pub eye_colours: Option<EyeColours>,
    /// The body's colour set the screen takes, if any (`BodyFadesIn`).
    pub body_colours: Option<BodyColours>,
    /// The fight's music, for audio, on the run he turns tangible (`BodyFadesIn`).
    pub music: Option<FightMusic>,
    /// The roar's sound, for audio, on the run it is heard (`Roars`).
    pub roar: Option<RoarSound>,
    /// The frame's hit of his tail on her, its side and its push time, the one hit her motion
    /// reacts to and her energy reads, as it stands (`TailHits`).
    pub hit: Option<Hit>,
    /// The side of the room that stopped his move this run, for his attacks (`Flies`).
    pub bound: Option<Bound>,
    /// The shake he starts as he meets the room's left side fast, for the screen's shaking
    /// (`Flies`).
    pub shake: Option<ShakeStart>,
}

impl RidleyRun {
    /// A run that leaves him so, the baby run in his hold at his position, handing on nothing
    /// else.
    fn holding_the_baby(ridley: CeresRidley, random: u16) -> Self {
        let baby = ridley.baby.run(ridley.position, random);
        Self {
            ridley: CeresRidley {
                baby: baby.baby,
                ..ridley
            },
            cry: baby.cry,
            colours: baby.colours,
            eye_colours: None,
            body_colours: None,
            music: None,
            roar: None,
            hit: None,
            bound: None,
            shake: None,
        }
    }
}

/// A look drawn at a position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Drawn<L> {
    /// Which figure.
    pub look: L,
    /// Where, in whole pixels.
    pub position: Pair<Px>,
}

/// His drawing: what the framework draws of him and the baby. The door of his room, while it is
/// drawn with him, is the door's to report (`DrawnWithRidley`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RidleyDrawing {
    /// His look at his position; none before his first run.
    pub ridley: Option<Drawn<RidleyLook>>,
    /// The baby's look at its position; none before its first run.
    pub baby: Option<Drawn<BabyLook>>,
    /// Where his tail's tip and all its segments are while it rests; none before he moves and
    /// from his take-off on (`TailAtRest`).
    pub tail: Option<Pair<Px>>,
}

impl CeresRidley {
    /// He is shown: the framework draws his look at his position.
    pub const SHOWN: bool = true;
    /// He never stops her motion; whether her touch and her shots meet him is whether he is
    /// tangible (`RidleyPhase::tangible`).
    pub const SOLID: bool = false;

    /// `RidleyPlaced`: the room places him as she enters, unless he is beaten, when he isn't
    /// placed and none of it happens. Placed, he holds at his place with no look, the baby new;
    /// the HUD learns of a boss encountered, the music stops, the station stops shaking, the
    /// stage is calm and his room's colours are the fade's goal. He is placed before the door
    /// of his room, whose colours follow his (`TheDoorsColours`). `TailCurls`: his tail's curl
    /// starts at 16.
    #[must_use]
    pub const fn place(ridley_beaten: bool) -> Option<RidleyPlacing> {
        if ridley_beaten {
            return None;
        }
        Some(RidleyPlacing {
            ridley: Self {
                phase: RidleyPhase::Holding,
                look: None,
                position: PLACE,
                fraction: NO_FRACTION,
                speed: STILL,
                takeoff: None,
                hover_runs: HoverRuns::NONE,
                baby: Baby::NEW,
                curl: TailCurl::PLACED,
            },
            boss: Boss::CeresRidley,
            music: MusicStop,
            shake: ShakeStop,
            stage: Stage::Calm,
            colours: ROOM_COLOURS,
        })
    }

    /// One run of him, on a frame he runs, reading whether a door's passage is under way as he
    /// runs (the door passage), the frame's random number, as the doors read it, her as his
    /// tail reads her, if she is there to reach, how the fight stands and his content. The baby
    /// runs in his hold every run, whatever his phase (`BabyHeld`, `BabyFidgets`, `BabyCries`).
    ///
    /// Before his take-off he moves on by his phase (`waking`); his roar over, he takes off
    /// (`took_off`); in flight he flies (`flying`). Once he has turned to an attack or given up,
    /// a later part runs him: here only the baby runs.
    #[must_use]
    pub fn run(
        self,
        passage_under_way: bool,
        random: u16,
        her: Option<&Her>,
        fight: Fight,
        content: &RidleyContent,
        rules: &Rules,
    ) -> RidleyRun {
        match self.phase.run(passage_under_way, content, rules) {
            PhaseStep::Waking(phase) => self.waking(&phase, random, her, rules),
            PhaseStep::TakesOff => self.took_off(random),
            PhaseStep::Flies(phase) => self.flying(phase, fight, random),
            PhaseStep::HandedOver => RidleyRun::holding_the_baby(self, random),
        }
    }

    /// A run before his take-off: he moves on by his phase (`RidleyWaits`, `EyeFadesIn`,
    /// `BodyFadesIn`, `ReadyToRoar`, `Roars`) and takes its look. His position doesn't change.
    ///
    /// On a run that ends with his tail at rest (`TailAtRest`), his tail runs after the rest of
    /// him, so the run he turns tangible can hit: its curl changes with the random number
    /// (`TailCurls`), and if he is tangible its tip hits her within its reach (`TailHits`). A run
    /// that ends before he moves leaves the tail as it was.
    fn waking(self, phase: &PhaseRun, random: u16, her: Option<&Her>, rules: &Rules) -> RidleyRun {
        let tail_runs = phase.phase.tail_at_rest();
        let ridley = Self {
            phase: phase.phase,
            look: Some(phase.look),
            curl: if tail_runs {
                self.curl.curled(random)
            } else {
                self.curl
            },
            ..self
        };
        let run = RidleyRun::holding_the_baby(ridley, random);
        RidleyRun {
            eye_colours: phase.eye,
            body_colours: phase.body,
            music: phase.music,
            roar: phase.roar,
            hit: run.ridley.tail_hit(her, rules),
            ..run
        }
    }

    /// `TakesOff`: the run he takes off, his roar's wait over: `Flying`, the take-off's first
    /// figure at its first run and its look, his speed 0; then he moves (`Flies`) and the baby
    /// is held. His figure first steps on at the end of his next run.
    fn took_off(self, random: u16) -> RidleyRun {
        Self {
            look: Some(TakeOffFigure::START.look()),
            takeoff: Some(TakeOffFigure::START),
            ..self
        }
        .moved(
            &Steered {
                phase: RidleyPhase::InFlight(FlightPhase::Flying),
                speed: STILL,
                hover_runs: self.hover_runs,
            },
            random,
        )
    }

    /// A run of his flight, in this order: his phase's rule (`TakesOff`, `Rises`, `SlowsDown`,
    /// `Hovers`); his move by his speed (`Flies`); the baby, held at his position as his move
    /// leaves it (`BabyHeld`); then his take-off's figure steps on (`TakesOff`). A run on which
    /// he gives up is a later part's: only his phase changes, and the baby runs.
    fn flying(self, phase: FlightPhase, fight: Fight, random: u16) -> RidleyRun {
        match phase.rule(self.position, self.speed, self.hover_runs, fight, random) {
            Ruled::GivesUp(give_up) => RidleyRun::holding_the_baby(
                Self {
                    phase: RidleyPhase::GivingUp(give_up),
                    ..self
                },
                random,
            ),
            Ruled::FliesOn(steered) => {
                let run = self.moved(&steered, random);
                RidleyRun {
                    ridley: run.ridley.figure_stepped(),
                    ..run
                }
            }
        }
    }

    /// `Flies`, `BabyHeld`: he takes what his phase's rule leaves and moves by his speed, within
    /// the room's bounds; the baby is held at his position as the move leaves it.
    fn moved(self, steered: &Steered, random: u16) -> RidleyRun {
        let moved = flown(self.position, self.fraction, steered.speed);
        let ridley = Self {
            phase: steered.phase,
            hover_runs: steered.hover_runs,
            position: moved.position,
            fraction: moved.fraction,
            speed: moved.speed,
            ..self
        };
        RidleyRun {
            bound: moved.bound,
            shake: moved.shake,
            ..RidleyRun::holding_the_baby(ridley, random)
        }
    }

    /// `TakesOff`: at the end of every run after the one he takes off, his take-off's figure
    /// steps on and he takes its look; as figure 2 or 3 starts he hops, and as figure 4 starts
    /// he springs into the air: `Rising`, at the lift-off speed.
    fn figure_stepped(self) -> Self {
        let Some(figure) = self.takeoff.map(TakeOffFigure::ran) else {
            return self;
        };
        let stepped = Self {
            takeoff: Some(figure),
            look: Some(figure.look()),
            ..self
        };
        match figure.starts() {
            Some(TakeOffStart::Hop(hop)) => Self {
                position: hopped(self.position, hop),
                ..stepped
            },
            Some(TakeOffStart::LiftOff) => Self {
                phase: RidleyPhase::InFlight(FlightPhase::Rising),
                speed: LIFT_OFF,
                ..stepped
            },
            None => stepped,
        }
    }

    /// `TailAtRest`: where his tail's tip and all its segments hang while it rests, from his
    /// position; none before he moves and from his take-off on.
    #[must_use]
    pub fn tail(&self) -> Option<Pair<Px>> {
        self.phase
            .tail_at_rest()
            .then(|| place_at_rest(self.position))
    }

    /// `TailHits`: the frame's hit, as his run leaves him: his tail at rest and he tangible,
    /// her there and within its tip's reach.
    fn tail_hit(&self, her: Option<&Her>, rules: &Rules) -> Option<Hit> {
        let tip = self.tail().filter(|_| self.phase.tangible())?;
        her?.hit_by(tip, rules.tail_reach)
    }

    /// His drawing: his look at his position, the baby's look at its position and his tail's
    /// place while it rests.
    #[must_use]
    pub fn drawing(&self) -> RidleyDrawing {
        RidleyDrawing {
            ridley: self.look.map(|look| Drawn {
                look,
                position: self.position,
            }),
            baby: self
                .baby
                .routine
                .look()
                .zip(self.baby.position)
                .map(|(look, position)| Drawn { look, position }),
            tail: self.tail(),
        }
    }
}
