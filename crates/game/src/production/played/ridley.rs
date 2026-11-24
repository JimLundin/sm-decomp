//! Ceres Ridley and the baby in his hold.
//!
//! In Ridley's room he runs first of the room's enemies, the baby in his hold, and the door of
//! his room after him. Fetching his room's contents places him: the HUD meets a boss, the
//! stage is calm and his room's colours are the fade's goal. The baby's colours go to the
//! screen as it glows, and his own as his eye and his body fade in.

use super::{Content, Fault, Rules, station};
use crate::production::ridley;
use boss_ceres_ridley::{CeresRidley, RidleyPhase, RidleyRun};
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

/// One run of Ridley, the baby in his hold: the colour sets the run names go to the screen at
/// once. His fight's music and his roar are produced, for audio.
///
/// Once he is tangible his tail's touch hurts her and knocks her back, which no owner here
/// plays, so the run that makes him tangible is refused; flying, a later piece runs him.
pub(super) fn run(
    ridley: CeresRidley,
    passage_under_way: bool,
    random: types::RandomSequence,
    content: &Content,
    rules: &Rules,
    palette: &mut Palette,
) -> Result<RidleyRun, Fault> {
    if ridley.phase == RidleyPhase::Flying {
        return Err(Fault::RidleyFlying);
    }
    let run = ridley.run(
        passage_under_way,
        random.number.0,
        &content.ridley,
        &rules.ceres,
    );
    if run.ridley.phase.tangible() {
        return Err(Fault::RidleyTangible);
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
    Ok(run)
}

#[cfg(test)]
mod tests {
    use super::run;
    use crate::production::played::Fault;
    use crate::production::played::tests::{content, rules};
    use boss_ceres_ridley::{
        BodyColours, CeresRidley, EyeColours, FadeSets, RidleyContent, RidleyPhase,
    };
    use core::num::NonZeroU16;
    use types::{Colour, RandomNumber, RandomSequence};
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

    fn ran(phase: RidleyPhase, palette: &mut Palette) -> Result<RidleyPhase, Fault> {
        let mut content = content();
        content.ridley = waking();
        let random = RandomSequence {
            number: RandomNumber(1),
        };
        run(ridley(phase), false, random, &content, &rules(), palette).map(|run| run.ridley.phase)
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

    #[test]
    fn the_run_he_turns_tangible_and_any_run_flying_are_refused_leaving_the_screen() {
        for (phase, fault) in [
            (
                RidleyPhase::BodyFading { sets: 1, runs: 1 },
                Fault::RidleyTangible,
            ),
            (RidleyPhase::Flying, Fault::RidleyFlying),
        ] {
            let mut palette = unlit();
            assert_eq!(ran(phase, &mut palette), Err(fault));
            assert_eq!(palette, unlit());
        }
    }
}
