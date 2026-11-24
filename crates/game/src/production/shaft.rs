//! The Ceres shaft as she arrives (boss-ceres-ridley part 2), in the played frame: the pad and
//! the cover, which hold her and lower her to the shaft's floor, and the lights the shaft's
//! overlay shows.
//!
//! The pad and the cover run on each frame of play and of a fade in, after her own frame; the
//! lights on each frame the overlay runs, which is each frame the room's enemies run.

use boss_ceres_ridley::{FadeSteps, Lights, LightsInputs, PadAndCover, Stage};
use data::assets::actors::BeaconColours;
use samus_movement::Samus;
use types::Parts as _;
use world_doors::{Doors, Palette, Passage, Phase};

/// The pad and the cover, gone: no pad holds her and no cover hides the floor.
pub const GONE: PadAndCover = PadAndCover {
    pad: None,
    cover: None,
};

/// Why the lights could not be shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LightsFault {
    /// A beacon colour of the content has more than 15 bits.
    Colour,
    /// The beacon's step names a colour set past the content's.
    Set,
    /// The doors' fade has taken more steps than the lights read.
    FadeSteps,
}

/// One run of the pad and the cover, after her own frame: her height becomes the pad's.
pub fn run_pad(
    pad_and_cover: PadAndCover,
    samus: &mut Samus,
    rules: &boss_ceres_ridley::Rules,
) -> PadAndCover {
    if pad_and_cover == GONE {
        return GONE;
    }
    let step = pad_and_cover.run(samus.position.y.pixel(), rules);
    samus.position.y = types::fixed(step.her_y, samus.position.y.sub());
    step.pad_and_cover
}

/// What the lights show on a frame the overlay runs, as the doors' passage stood as the frame
/// began.
///
/// # Errors
/// [`LightsFault::FadeSteps`] for a fade past the steps the lights read.
pub fn lights(
    finished_frames: u16,
    doors: Doors,
    stage: Stage,
    rules: &boss_ceres_ridley::Rules,
) -> Result<Lights, LightsFault> {
    let inputs = LightsInputs {
        finished_frames,
        fade_steps_taken: fade_steps_taken(doors).ok_or(LightsFault::FadeSteps)?,
        stage,
    };
    Ok(Lights::step(inputs, rules))
}

/// The fade's steps taken when the enemies run: darkening, the passage takes its step before
/// them; brightening, after them. Outside a door's fade, none.
fn fade_steps_taken(doors: Doors) -> Option<FadeSteps> {
    let steps = match doors {
        Doors::Passing(Passage {
            phase: Phase::Darkening(t),
            ..
        }) => t.0.checked_add(1)?,
        Doors::Passing(Passage {
            phase: Phase::Brightening(t),
            ..
        }) => t.0,
        Doors::Play | Doors::Passing(_) => 0,
    };
    FadeSteps::new(u8::try_from(steps).ok()?)
}

/// The beacon's step sets the screen colours its set names; with none, the screen keeps its
/// colours. The platform's look is the planes', which this loop doesn't draw.
///
/// # Errors
/// [`LightsFault::Set`] for a step past the content's colour sets, and [`LightsFault::Colour`]
/// for a content colour past 15 bits, each leaving the colours as they were.
pub fn show(
    lights: Lights,
    beacon: &BeaconColours,
    palette: &mut Palette,
) -> Result<(), LightsFault> {
    let Some(step) = lights.beacon else {
        return Ok(());
    };
    let set = beacon
        .sets
        .get(usize::from(step.place()))
        .ok_or(LightsFault::Set)?;
    let mut screen = palette.screen;
    let shown = screen
        .iter_mut()
        .skip(BeaconColours::FIRST)
        .take(BeaconColours::SHADES);
    for (colour, shade) in shown.zip(set) {
        *colour = types::Colour::new(shade.0).ok_or(LightsFault::Colour)?;
    }
    palette.screen = screen;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use world_doors::FadeTime;

    fn passing(phase: Phase) -> Doors {
        Doors::Passing(Passage {
            phase,
            arrival: None,
            slide_frames: 0,
        })
    }

    #[test]
    fn darkening_counts_the_step_taken_before_the_enemies_and_brightening_does_not() {
        let taken = |doors| fade_steps_taken(doors).map(FadeSteps::steps);
        assert_eq!(taken(passing(Phase::Darkening(FadeTime(3)))), Some(4));
        assert_eq!(taken(passing(Phase::Brightening(FadeTime(3)))), Some(3));
        assert_eq!(taken(passing(Phase::Quieting { asked: false })), Some(0));
        assert_eq!(taken(Doors::Play), Some(0));
        // Past the steps a fade takes, the lights refuse it.
        assert_eq!(
            taken(passing(Phase::Darkening(FadeTime(u16::from(
                FadeSteps::MOST
            ))))),
            None
        );
    }
}
