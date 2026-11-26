//! Spec: `SlimeFalls`. The rising baby sheds droplets that burst against the floor.

use types::Pair;
use types::discovery::{DropletStage, Droplets, PieceId, RoomPoint, SlimeDroplet};

use crate::scene::{Animation, DropletContent};

/// Spec: `SlimeFalls`. The droplet cohort placed at the baby's whole-pixel anchor, before any
/// flight.
#[must_use]
pub fn shed(cohort: &[DropletContent], anchor: RoomPoint) -> Droplets {
    Droplets::gathered((0..=u8::MAX).zip(cohort).map(|(identity, _)| SlimeDroplet {
        identity: PieceId(identity),
        place: anchor,
        stage: DropletStage::Falling { steps: 0 },
    }))
}

/// Spec: `SlimeFalls`. One finished frame of every droplet: falling ones fly, splashes age.
pub fn fall(droplets: &mut Droplets, cohort: &[DropletContent], splash: &Animation, floor_y: i32) {
    droplets.update(|droplet| match droplet.stage {
        DropletStage::Falling { steps } => {
            let content = cohort.get(usize::from(droplet.identity.0))?;
            Some(flown(droplet, steps, content, floor_y))
        }
        DropletStage::Splashing { age } => {
            let age = age.saturating_add(1);
            splash.holds(age).then_some(SlimeDroplet {
                stage: DropletStage::Splashing { age },
                ..droplet
            })
        }
    });
}

/// Spec: `SlimeFalls`. A falling droplet one frame further: moved by its drift and its next
/// vertical displacement, or its last one past the track's end. Above the floor it keeps
/// falling and counts the step; at the floor it splashes where it stopped, at age zero.
fn flown(
    droplet: SlimeDroplet,
    steps: u16,
    content: &DropletContent,
    floor_y: i32,
) -> SlimeDroplet {
    let place = Pair {
        x: droplet.place.x.moved(content.drift),
        y: droplet.place.y.moved(content.track.at_or_last(steps)),
    };
    let stage = if place.y.whole() < floor_y {
        DropletStage::Falling {
            steps: steps.saturating_add(1),
        }
    } else {
        DropletStage::Splashing { age: 0 }
    };
    SlimeDroplet {
        place,
        stage,
        ..droplet
    }
}
