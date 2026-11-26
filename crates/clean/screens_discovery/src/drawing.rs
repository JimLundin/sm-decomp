//! The encounter's pictures for E1 to draw, in precedence order: the shell in front of the
//! baby, the baby in front of the fragments, the fragments in front of the droplets and their
//! splashes, and within a cohort the first listed in front.

use types::Pair;
use types::discovery::{DiscoveryState, DropletStage, Hatchling, PieceId, Pulse};

use crate::scene::{DiscoveryScene, SpriteRef};
use crate::{egg, pulse};

/// What a picture shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Figure {
    /// The egg or its spent shell.
    Shell,
    /// The baby.
    Baby,
    /// A shell fragment.
    Fragment(PieceId),
    /// A droplet, falling or splashing.
    Droplet(PieceId),
}

/// A request to draw one picture at its whole-pixel anchor in the room.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Drawing {
    /// What it shows.
    pub figure: Figure,
    /// Its picture.
    pub sprite: SpriteRef,
    /// Its anchor, rounded toward the lower room pixel.
    pub anchor: Pair<i32>,
}

impl Drawing {
    /// Where it lands on the screen: its anchor less the room's whole-pixel scroll. E1 applies
    /// clipping and the picture's own tile priorities.
    #[must_use]
    pub const fn on_screen(&self, scroll: Pair<i32>) -> Pair<i32> {
        Pair {
            x: self.anchor.x.wrapping_sub(scroll.x),
            y: self.anchor.y.wrapping_sub(scroll.y),
        }
    }
}

/// The encounter's pictures, front first. A newly placed egg and baby have none until their
/// first finished frame, and the dormant encounter has none. Production places the group by the
/// state's `DiscoveryOrder`, and omits it while that is `Hidden`.
#[must_use]
pub fn drawings(state: &DiscoveryState, scene: &DiscoveryScene) -> Vec<Drawing> {
    let unseen = matches!(
        state.baby,
        Some(Hatchling {
            pulse: Pulse::Unseen,
            ..
        })
    );
    let shell = egg::picture(state.egg, &scene.shell)
        .filter(|_| !unseen)
        .map(|sprite| Drawing {
            figure: Figure::Shell,
            sprite,
            anchor: scene.home.whole(),
        });
    let baby = state.baby.and_then(|baby| {
        pulse::picture(baby.pulse, &scene.baby).map(|sprite| Drawing {
            figure: Figure::Baby,
            sprite,
            anchor: baby.place.whole(),
        })
    });
    let fragments = state.fragments.as_slice().iter().filter_map(|fragment| {
        let content = scene.fragments.get(usize::from(fragment.identity.0))?;
        Some(Drawing {
            figure: Figure::Fragment(fragment.identity),
            sprite: content.sprite,
            anchor: fragment.place.whole(),
        })
    });
    let droplets = state.droplets.as_slice().iter().filter_map(|droplet| {
        let sprite = match droplet.stage {
            DropletStage::Falling { .. } => {
                scene.droplets.get(usize::from(droplet.identity.0))?.sprite
            }
            DropletStage::Splashing { age } => scene.splash.picture_at(age)?,
        };
        Some(Drawing {
            figure: Figure::Droplet(droplet.identity),
            sprite,
            anchor: droplet.place.whole(),
        })
    });
    shell
        .into_iter()
        .chain(baby)
        .chain(fragments)
        .chain(droplets)
        .collect()
}
