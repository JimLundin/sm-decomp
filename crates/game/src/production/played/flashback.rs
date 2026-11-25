//! The Zebes encounter's scene setup, actor turn and picture publication.

use crate::intent::{SpriteDraw, on_screen};
use data::assets::flashback::Scene;
use screens_flashback::Art;
pub use screens_flashback::{
    Effects, HurtColours, IntroRequest, ReturnDissolve, RingKnockback, RoomPlay, Rules,
    ShotCandidate, ShotId, ShotKind,
};
use types::flashback::BurstPlace;
pub use types::flashback::{EncounterState, RoomBackgroundY};
use types::{Colour, Pair, Palette, Parity, Px, ROW_COLOURS, RowColours};

/// Stage the encounter during the flashback setup, before gameplay first takes a turn.
pub const fn start(state: &mut EncounterState, background_y: RoomBackgroundY) {
    screens_flashback::admit_start(state, background_y);
}

/// Step the encounter after Samus and weapons.
///
/// The scene dispatches this exactly once per work frame, including across intermediate cuts.
/// Apply the effects to the candidate owners before committing the whole turn.
pub fn turn(
    state: &mut EncounterState,
    rules: Rules,
    parity: Parity,
    dissolve: ReturnDissolve,
    play: &RoomPlay,
) -> Effects {
    screens_flashback::beat(state, rules, parity, dissolve, play)
}

/// Add the encounter's sprites after Samus and her shots. The screen origin follows the camera,
/// while the shake only moves the background.
#[must_use]
pub fn draw(state: &EncounterState, art: &Scene, camera: Pair<Px>) -> SpriteDraw {
    let mut draw = SpriteDraw::default();
    for picture in screens_flashback::pictures(state) {
        let clip = match picture.art {
            Art::MotherBrain => art.brain.as_slice(),
            Art::Ring(_) => art.ring.as_slice(),
            Art::Burst(BurstPlace::Near | BurstPlace::Low | BurstPlace::High) => {
                art.small_burst.as_slice()
            }
            Art::Burst(
                BurstPlace::Centre
                | BurstPlace::UpperRight
                | BurstPlace::LowerLeft
                | BurstPlace::UpperLeft
                | BurstPlace::LowerRight,
            ) => art.large_burst.as_slice(),
        };
        if let Some(map) = clip.get(usize::from(picture.image.0).wrapping_sub(1)) {
            let next = draw.pieces.len();
            draw.place(
                map,
                Pair {
                    x: on_screen(picture.at.x.0, camera.x.0),
                    y: on_screen(picture.at.y.0, camera.y.0),
                },
            );
            for piece in draw.pieces.iter_mut().skip(next) {
                piece.look.palette |= picture.palette.0;
            }
        }
    }
    draw
}

/// The row written by the flash, after the dissolve and before palette effects.
#[must_use]
pub const fn flash_colours(art: &Scene, flash: HurtColours) -> RowColours {
    match flash {
        HurtColours::White => RowColours([Colour::from_channels([31; 3]); ROW_COLOURS]),
        HurtColours::Normal => art.normal_boss,
    }
}

/// Put the encounter flash in the last sprite palette.
pub const fn publish_flash(palette: &mut Palette, row: RowColours) {
    if let Some(boss) = palette.0.as_chunks_mut::<ROW_COLOURS>().0.last_mut() {
        *boss = row.0;
    }
}

/// The encounter opportunities a scene can present at a video cut.
#[derive(Clone, Copy)]
pub enum Phase<'a> {
    /// Scene setup admits the start and inherits the background Y.
    Start(RoomBackgroundY),
    /// The scene reached the encounter's actor turn.
    Turn {
        /// Parity of the cinematic work frame after it has incremented.
        parity: Parity,
        /// The return dissolve as the current stage step left it.
        dissolve: ReturnDissolve,
        /// Room play after Samus and weapons have stepped.
        play: &'a RoomPlay,
    },
    /// The cut precedes the actor turn or finishes work after an earlier cut already took it.
    Held,
}

/// Opportunities selected at one video cut.
#[derive(Clone, Copy)]
pub struct Cut<'a> {
    /// Authorized setup or actor work.
    pub phase: Phase<'a>,
    /// Camera origin when this cut publishes the sprites, if it publishes them.
    pub drawing: Option<Pair<Px>>,
}

/// Newly produced encounter outputs at the scene's publication boundaries.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Publication {
    /// Requests of a new actor turn.
    pub effects: Option<Effects>,
    /// The flash row if a new actor turn wrote one.
    pub colours: Option<RowColours>,
    /// Ordered pieces at a drawing opportunity.
    pub draw: Option<SpriteDraw>,
    /// Room background Y published with the sprites.
    pub background_y: Option<RoomBackgroundY>,
}

pub(in crate::production) fn advance(
    state: &mut EncounterState,
    art: &Scene,
    rules: Rules,
    cut: Cut<'_>,
) -> Publication {
    let effects = match cut.phase {
        Phase::Start(y) => {
            start(state, y);
            None
        }
        Phase::Turn {
            parity,
            dissolve,
            play,
        } => Some(turn(state, rules, parity, dissolve, play)),
        Phase::Held => None,
    };
    Publication {
        effects,
        colours: effects
            .and_then(|effects| effects.palette)
            .map(|flash| flash_colours(art, flash)),
        draw: cut.drawing.map(|camera| draw(state, art, camera)),
        background_y: cut.drawing.map(|_| state.background_y),
    }
}
