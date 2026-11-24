//! Ceres Ridley as she enters his room and as he wakes: placed at his place in the dark with the
//! baby in his hold, waiting a fixed while once the door's passage is over, then waking: his eye
//! lights up, his body fades in, and tangible he roars before he takes off. His flight is a
//! later piece's.
//!
//! The caller holds a [`CeresRidley`] between frames and steps it on the frame the room places
//! him ([`CeresRidley::place`]) and on each frame he runs ([`CeresRidley::run`]): whether a
//! door's passage is under way, the random number and his content are all he reads. On a frame
//! he doesn't run the caller doesn't step him, and nothing of him or the baby changes.

use core::num::Wrapping;

use types::{Pair, Px};

use crate::baby::{Baby, BabyColours, BabyCry, BabyLook};
use crate::ridley_phase::{FightMusic, RidleyPhase, RoarSound};
use crate::roar::RidleyLook;
use crate::rules::Rules;
use crate::station::Stage;
use crate::waking::{BodyColours, EyeColours, RidleyContent};

/// `RidleyPlaced`: his place, whatever the room's content says.
const PLACE: Pair<Px> = Pair {
    x: Wrapping(186),
    y: Wrapping(169),
};

/// Ceres Ridley in his room: the state a caller holds from one frame to the next.
///
/// He is shown and never stops her motion: the framework draws his look at his position, in
/// colours that stay black until he wakes. Until he turns tangible (`RidleyPhase::tangible`),
/// her touch and her shots go through him too.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CeresRidley {
    /// Where he is, from his placing to his take-off.
    pub phase: RidleyPhase,
    /// The figure he is drawn with; none before his first run, and none from his take-off on,
    /// when his next part gives it.
    pub look: Option<RidleyLook>,
    /// Where he is drawn and the baby held, in whole pixels.
    pub position: Pair<Px>,
    /// The baby he holds.
    pub baby: Baby,
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
    /// of his room, whose colours follow his (`TheDoorsColours`).
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
                baby: Baby::NEW,
            },
            boss: Boss::CeresRidley,
            music: MusicStop,
            shake: ShakeStop,
            stage: Stage::Calm,
            colours: ROOM_COLOURS,
        })
    }

    /// One run of him, on a frame he runs, reading whether a door's passage is under way as he
    /// runs (the door passage), the frame's random number, as the doors read it, and his
    /// content. He moves on by his phase (`RidleyWaits`, `EyeFadesIn`, `BodyFadesIn`,
    /// `ReadyToRoar`, `Roars`) and takes its look, and the baby runs in his hold, whatever his
    /// phase (`BabyHeld`, `BabyFidgets`, `BabyCries`). His position doesn't change.
    #[must_use]
    pub fn run(
        self,
        passage_under_way: bool,
        random: u16,
        content: &RidleyContent,
        rules: &Rules,
    ) -> RidleyRun {
        let baby = self.baby.run(self.position, random);
        let phase = self.phase.run(passage_under_way, content, rules);
        RidleyRun {
            ridley: Self {
                phase: phase.phase,
                look: phase.look,
                baby: baby.baby,
                ..self
            },
            cry: baby.cry,
            colours: baby.colours,
            eye_colours: phase.eye,
            body_colours: phase.body,
            music: phase.music,
            roar: phase.roar,
        }
    }

    /// His drawing: his look at his position and the baby's look at its position.
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
        }
    }
}
