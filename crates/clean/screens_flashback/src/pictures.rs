//! Spec: `FightPictures`. The pictures the encounter asks video to draw, foremost first, with
//! Mother Brain behind the rings and bursts. Origins are whole room coordinates.

use core::num::Wrapping;

use types::flashback::{
    BreathStep, BurstPlace, BurstStep, DrawRank, EncounterState, RingPath, RingStep,
};
use types::{Pair, Parts, Px};

/// Spec: `EncounterStarts`. Mother Brain's picture origin, (56, 111) px.
pub const BRAIN_ORIGIN: Pair<Px> = Pair {
    x: Wrapping(56),
    y: Wrapping(111),
};

/// An image palette of the scene's sprites.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImagePalette(pub u8);

/// Spec: `FightPictures`. Mother Brain and the rings use image palette 7, the one her flash
/// replaces.
pub const BOSS_PALETTE: ImagePalette = ImagePalette(7);
/// Spec: `FightPictures`. The bursts use image palette 5.
const BURST_PALETTE: ImagePalette = ImagePalette(5);

/// The number of an image of its clip, counting from 1; the art itself is the scene import's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClipImage(pub u8);

/// Whose picture it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Art {
    /// Mother Brain's breathing clip.
    MotherBrain,
    /// A ring's clip.
    Ring(RingPath),
    /// A burst's clip.
    Burst(BurstPlace),
}

/// A picture to draw: whose, where, which image of its clip and in which palette.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Drawing {
    /// Whose picture it is.
    pub art: Art,
    /// Its origin, whole room pixels.
    pub at: Pair<Px>,
    /// The image of its clip.
    pub image: ClipImage,
    /// Its image palette.
    pub palette: ImagePalette,
}

/// Spec: `BrainBreathes`. The breathing pictures 1, 2, 3, 2.
const fn breath_image(step: BreathStep) -> ClipImage {
    match step {
        BreathStep::First => ClipImage(1),
        BreathStep::Second | BreathStep::Fourth => ClipImage(2),
        BreathStep::Third => ClipImage(3),
    }
}

/// Spec: `RingsAppear`, `RingsSweep`. Forming images 1, 2, 3, then moving images 2, 1, 2, 3.
const fn ring_image(step: RingStep) -> ClipImage {
    match step {
        RingStep::FormingFirst | RingStep::MovingSecond => ClipImage(1),
        RingStep::FormingSecond | RingStep::MovingFirst | RingStep::MovingThird => ClipImage(2),
        RingStep::FormingThird | RingStep::MovingFourth => ClipImage(3),
    }
}

/// Spec: `BurstsPulse`. Images 1 to 6; none while waiting or blank.
const fn burst_image(step: BurstStep) -> Option<ClipImage> {
    match step {
        BurstStep::Waiting | BurstStep::Blank => None,
        BurstStep::First => Some(ClipImage(1)),
        BurstStep::Second => Some(ClipImage(2)),
        BurstStep::Third => Some(ClipImage(3)),
        BurstStep::Fourth => Some(ClipImage(4)),
        BurstStep::Fifth => Some(ClipImage(5)),
        BurstStep::Sixth => Some(ClipImage(6)),
    }
}

/// Spec: `FightPictures`. The encounter's visible pictures in draw order, foremost first: the
/// rings and bursts showing an image by rank, then Mother Brain behind them.
#[must_use]
pub fn pictures(state: &EncounterState) -> Vec<Drawing> {
    let rings = state.rings.iter().filter_map(|(path, ring)| {
        ring.map(|ring| {
            let drawing = Drawing {
                art: Art::Ring(path),
                at: Pair {
                    x: ring.position.x.pixel(),
                    y: ring.position.y.pixel(),
                },
                image: ring_image(ring.clip.step),
                palette: BOSS_PALETTE,
            };
            (ring.rank, drawing)
        })
    });
    let bursts = state.bursts.iter().filter_map(|(place, burst)| {
        let burst = (*burst)?;
        let drawing = Drawing {
            art: Art::Burst(place),
            at: burst.at,
            image: burst_image(burst.clip.step)?,
            palette: BURST_PALETTE,
        };
        Some((burst.rank, drawing))
    });
    let mut ranked: Vec<(DrawRank, Drawing)> = rings.chain(bursts).collect();
    ranked.sort_by_key(|&(rank, _)| rank);
    let brain = state.brain.map(|brain| Drawing {
        art: Art::MotherBrain,
        at: BRAIN_ORIGIN,
        image: breath_image(brain.breath.step),
        palette: BOSS_PALETTE,
    });
    ranked
        .into_iter()
        .map(|(_, drawing)| drawing)
        .chain(brain)
        .collect()
}
