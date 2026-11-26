//! Ceres Ridley and the baby in his hold.
//!
//! In Ridley's room he runs first of the room's enemies, the baby in his hold, and the door of
//! his room after him. Fetching his room's contents places him: the HUD meets a boss, the
//! stage is calm and his room's colours are the fade's goal. The baby's colours go to the
//! screen as it glows, and his own as his eye and his body fade in. From his body's fade on his
//! tail rests behind him. At take-off it swings loose and can whip toward her. Once he is
//! tangible, the moved tip can hit her. Her energy and motion read the same hit.

use super::{Content, Fault, Rules, station};
use crate::production::{ridley, room_loading};
use boss_ceres_ridley::{CeresRidley, Fight, RidleyPhase, RidleyRun};
use world_doors::Palette;

impl station::State {
    /// Ridley's drawing as the frame leaves him, with the doors he draws; none outside his
    /// room. It is produced for drawing, which this loop doesn't compare.
    #[must_use]
    pub fn ridley_drawing(&self) -> Option<ridley::Drawing> {
        self.ridley
            .as_ref()
            .map(|ridley| ridley::drawing(ridley, &self.doors))
    }
}

/// One run of Ridley, the baby in his hold, his tail reading her in a frame of play:
/// the colour sets the run names go to the screen at once. His fight's music and his roar are
/// produced for audio along with his tail swish, the frame's hit for her energy and her motion,
/// and the shake he starts for the room's quake. In flight he reads her energy as the frame
/// starts it (samus-health) and his hits taken held at 0. A run that starts
/// in an attack is a later part's. Giving up starts his retreat on the same run.
pub(super) fn run(
    ridley: CeresRidley,
    running: station::Running<'_>,
    (random, finished): (types::RandomSequence, types::GameFrames),
    her_energy: u16,
    content: &Content,
    rules: &Rules,
    palette: &mut Palette,
) -> Result<RidleyRun, Fault> {
    match ridley.phase {
        RidleyPhase::Attacking(_) => return Err(Fault::RidleyAttacks),
        RidleyPhase::FakeRetreat => return Err(Fault::RidleyGivesUp),
        RidleyPhase::Getaway(types::ceres_getaway::GetawayState::WarningReady) => {
            return Err(Fault::RidleyWarning);
        }
        RidleyPhase::Getaway(_)
        | RidleyPhase::Awake
        | RidleyPhase::BodyFading { .. }
        | RidleyPhase::EyeFading { .. }
        | RidleyPhase::Holding
        | RidleyPhase::Retreating
        | RidleyPhase::CoastingAbove(_)
        | RidleyPhase::InFlight(_)
        | RidleyPhase::ReadyToRoar { .. }
        | RidleyPhase::Roaring { .. }
        | RidleyPhase::Waiting { .. } => {}
    }
    let (passage_under_way, her) = match running {
        station::Running::Play(her) => (false, her),
        station::Running::Passage => (true, None),
    };
    let run = ridley.run(
        boss_ceres_ridley::RidleyFrame {
            passage_under_way,
            finished,
        },
        random.number.0,
        her,
        Fight {
            her_energy,
            hits_taken: 0,
        },
        &content.ridley,
        &rules.ceres,
    );
    if matches!(run.ridley.phase, RidleyPhase::FakeRetreat) {
        return Err(Fault::RidleyGivesUp);
    }
    if let Some(colours) = run.colours {
        ridley::show_baby(colours, &content.rooms.actors.ceres_ridley, palette)
            .map_err(|ridley::ColourFault| Fault::BabyColours)?;
    }
    if let Some(colours) = run.eye_colours {
        ridley::show_eye(colours, palette);
    }
    if let Some(colours) = run.body_colours {
        ridley::show_body(colours, palette);
    }
    if let Some(backdrop) = run.backdrop {
        for wall in [backdrop.left_wall, backdrop.right_wall] {
            room_loading::door_goal(wall.colours, &content.rooms.actors, &mut palette.goal)
                .map_err(Fault::Load)?;
        }
        ridley::show_getaway(backdrop.colours, palette);
    }
    Ok(run)
}

/// The room work after Samus moves publishes the selected flyby picture and starts its shake.
pub(super) fn apply_getaway(
    effects: boss_ceres_ridley::GetawayEffects,
    quake: &mut world_quake::State,
    palette: &mut Palette,
) {
    if let Some(request) = effects.quake {
        world_quake::start_or_stop(
            quake,
            world_quake::ShakeOrder::Start {
                kind: request.kind,
                frames: request.duration,
            },
        );
    }
    if let boss_ceres_ridley::GetawayView::Flyby { body_colours, .. } = effects.view {
        ridley::show_flyby(&body_colours, palette);
    }
}

#[cfg(test)]
mod tests {
    use super::run;
    use crate::production::played::Fault;
    use crate::production::played::station::Running;
    use crate::production::played::tests::{content, rules};
    use boss_ceres_ridley::{
        Attack, BodyColours, CeresRidley, EyeColours, FadeSets, FlightPhase, Her, RidleyContent,
        RidleyPhase,
    };
    use core::num::{NonZeroU16, Wrapping};
    use types::{Colour, Hit, Pair, PushFrames, Px, RandomNumber, RandomSequence, Side};
    use world_doors::{COLOURS, Palette};

    /// His sprite rows' first colour on the screen: the second and the doors'.
    const SECOND: usize = (8 + 1) * 16;
    const HIS: usize = (8 + 7) * 16;

    fn colour(bits: u16) -> Colour {
        Colour::new(bits).unwrap()
    }

    /// Invented content: two eye steps and two body sets, each colour its own.
    fn waking() -> RidleyContent {
        RidleyContent {
            eye_steps: FadeSets::new(vec![
                EyeColours([colour(0x101), colour(0x102), colour(0x103)]),
                EyeColours([colour(0x201), colour(0x202), colour(0x203)]),
            ])
            .unwrap(),
            body_sets: FadeSets::new(vec![body(0x300), body(0x400)]).unwrap(),
            getaway: content().ridley.getaway,
            flyby: content().ridley.flyby,
        }
    }

    /// A body set whose colour `at` is `high` + `at`.
    fn body(high: u16) -> BodyColours {
        BodyColours(std::array::from_fn(|at| {
            colour(high | u16::try_from(at).unwrap())
        }))
    }

    fn ridley(phase: RidleyPhase) -> CeresRidley {
        CeresRidley {
            phase,
            ..CeresRidley::place(false).unwrap().ridley
        }
    }

    /// Her as his tail reads her: at `position`, a body of 8 by 16 px, unprotected.
    fn her(position: Pair<Px>) -> Her {
        Her {
            position,
            body: Pair {
                x: Wrapping(8),
                y: Wrapping(16),
            },
            protection: 0,
        }
    }

    /// Her far from his tail.
    fn far() -> Her {
        her(Pair {
            x: Wrapping(0),
            y: Wrapping(0),
        })
    }

    fn ran_by(
        phase: RidleyPhase,
        her: &Her,
        palette: &mut Palette,
    ) -> Result<(RidleyPhase, Option<Hit>), Fault> {
        let mut content = content();
        content.ridley = waking();
        let random = RandomSequence {
            number: RandomNumber(1),
        };
        run(
            ridley(phase),
            Running::Play(Some(her)),
            (random, types::GameFrames(0)),
            99,
            &content,
            &rules(),
            palette,
        )
        .map(|run| (run.ridley.phase, run.hit))
    }

    fn ran(phase: RidleyPhase, palette: &mut Palette) -> Result<RidleyPhase, Fault> {
        ran_by(phase, &far(), palette).map(|(phase, _)| phase)
    }

    fn unlit() -> Palette {
        Palette {
            screen: [colour(0x7fff); COLOURS],
            goal: [Colour::BLACK; COLOURS],
        }
    }

    #[test]
    fn the_eyes_step_shows_on_colours_12_to_14_of_his_palette() {
        let mut palette = unlit();
        let phase = RidleyPhase::EyeFading {
            steps: NonZeroU16::MIN,
        };
        assert!(matches!(
            ran(phase, &mut palette),
            Ok(RidleyPhase::EyeFading { steps }) if steps.get() == 2
        ));
        let mut shown = unlit().screen;
        shown[HIS + 12..HIS + 15].copy_from_slice(&[colour(0x201), colour(0x202), colour(0x203)]);
        assert_eq!(palette.screen, shown);
    }

    #[test]
    fn the_bodys_set_shows_on_colours_1_to_11_of_both_his_palettes() {
        let mut palette = unlit();
        let phase = RidleyPhase::BodyFading { sets: 0, runs: 1 };
        assert_eq!(
            ran(phase, &mut palette),
            Ok(RidleyPhase::BodyFading { sets: 1, runs: 0 })
        );
        let mut shown = unlit().screen;
        for row in [SECOND, HIS] {
            shown[row + 1..row + 12].copy_from_slice(&body(0x300).0);
        }
        assert_eq!(palette.screen, shown);
    }

    /// The run he turns tangible reports the frame's hit when she stands at his tail's tip, and
    /// none when she is out of its reach.
    #[test]
    fn the_run_he_turns_tangible_hits_her_at_his_tails_tip() {
        let turning = RidleyPhase::BodyFading { sets: 1, runs: 1 };
        let placed = ridley(turning).position;
        let tip = Pair {
            x: placed.x + Wrapping(32),
            y: placed.y + Wrapping(16),
        };
        let left_of_it = Pair {
            x: tip.x - Wrapping(1),
            ..tip
        };
        let (phase, hit) = ran_by(turning, &her(left_of_it), &mut unlit()).unwrap();
        assert!(phase.tangible());
        assert_eq!(
            hit,
            Some(Hit {
                side: Side::Left,
                push: PushFrames::new(5).unwrap(),
            })
        );
        assert_eq!(ran_by(turning, &far(), &mut unlit()).unwrap().1, None);
    }

    #[test]
    fn a_run_in_an_attack_or_giving_up_is_refused_leaving_the_screen() {
        let mut palette = unlit();
        assert_eq!(
            ran(RidleyPhase::Attacking(Attack::Swoop), &mut palette),
            Err(Fault::RidleyAttacks)
        );
        assert_eq!(
            ran(RidleyPhase::FakeRetreat, &mut palette),
            Err(Fault::RidleyGivesUp)
        );
        assert_eq!(
            ran(
                RidleyPhase::Getaway(types::ceres_getaway::GetawayState::WarningReady),
                &mut palette
            ),
            Err(Fault::RidleyWarning)
        );
        assert_eq!(palette, unlit());
    }

    /// In flight he runs, not refused: a run rising leaves him rising.
    #[test]
    fn his_flight_runs() {
        let phase = RidleyPhase::InFlight(FlightPhase::Rising);
        assert_eq!(
            ran(phase, &mut unlit()),
            Ok(RidleyPhase::InFlight(FlightPhase::Rising))
        );
    }
}
