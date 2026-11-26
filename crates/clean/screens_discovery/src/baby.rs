//! Spec: `BabyEmerges`, `BabyFollows`, `SlimeFalls`. The baby rises out of the shell, sheds its
//! droplets, hovers above Samus, then chases her, crying twice.

use types::Pair;
use types::discovery::{BabyMotion, DiscoveryOrder, Hatchling, RoomPoint, Slime, Speed};

use crate::rules::Rules;

/// What the baby asks of the encounter on a finished frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BabyCall {
    /// Nothing.
    Silent,
    /// Spec: `BabyFollows`. It cries.
    Cries,
    /// Spec: `SlimeFalls`. It cries and sheds its droplets at this whole-pixel anchor.
    Sheds(RoomPoint),
}

/// One finished frame of the baby's motion. `released` says the hatch reached its release
/// marker this frame; the baby's order in sprite priority is set while it follows.
pub fn turn(
    baby: &mut Hatchling,
    order: &mut DiscoveryOrder,
    released: bool,
    samus: Pair<i32>,
    rules: &Rules,
) -> BabyCall {
    match baby.motion {
        BabyMotion::Inside => {
            emerge(baby, released);
            BabyCall::Silent
        }
        BabyMotion::Rising { vertical, slime } => rise(baby, vertical, slime, samus.y, rules),
        BabyMotion::Hovering { left } => {
            hover(baby, left);
            BabyCall::Silent
        }
        BabyMotion::Following { velocity, cry } => follow(baby, order, velocity, cry, samus, rules),
    }
}

/// Spec: `BabyEmerges`. On the release marker the baby starts rising, still, with its droplets
/// still to shed. It first moves on the following finished frame.
const fn emerge(baby: &mut Hatchling, released: bool) {
    if released {
        baby.motion = BabyMotion::Rising {
            vertical: Speed::STILL,
            slime: Slime::Clinging,
        };
    }
}

/// Spec: `BabyEmerges`, `SlimeFalls`. One frame of the rise toward its place above Samus. Its
/// droplets shed first, if they cling and its whole-pixel y is the shedding height before the
/// move. A move that leaves its speed nonnegative starts the hover.
fn rise(
    baby: &mut Hatchling,
    vertical: Speed,
    slime: Slime,
    samus_y: i32,
    rules: &Rules,
) -> BabyCall {
    let sheds = slime == Slime::Clinging && baby.place.y.whole() == rules.shed_y;
    let call = if sheds {
        BabyCall::Sheds(baby.place.snapped())
    } else {
        BabyCall::Silent
    };
    let target = samus_y.saturating_sub(rules.rise_height);
    let (y, vertical) = rules.rise.toward(target, baby.place.y, vertical);
    baby.place.y = y;
    baby.motion = if vertical.negative() {
        BabyMotion::Rising {
            vertical,
            slime: if sheds { Slime::Shed } else { slime },
        }
    } else {
        BabyMotion::Hovering {
            left: rules.hover_frames,
        }
    };
    call
}

/// Spec: `BabyEmerges`. One frame less of the hover; on the frame it reaches zero the baby
/// starts following from the same place, still, with cry age zero.
const fn hover(baby: &mut Hatchling, left: u8) {
    let left = left.saturating_sub(1);
    baby.motion = if left == 0 {
        BabyMotion::Following {
            velocity: Pair {
                x: Speed::STILL,
                y: Speed::STILL,
            },
            cry: 0,
        }
    } else {
        BabyMotion::Hovering { left }
    };
}

/// Spec: `BabyFollows`. Chase just above Samus on each axis. A leftward horizontal speed puts
/// the encounter ahead of Samus, any other puts Samus ahead. The cry age counts up to the second cry, crying on reaching each cry age.
fn follow(
    baby: &mut Hatchling,
    order: &mut DiscoveryOrder,
    velocity: Pair<Speed>,
    cry: u8,
    samus: Pair<i32>,
    rules: &Rules,
) -> BabyCall {
    let goal_height = samus.y.saturating_sub(rules.follow_height);
    let (x, vx) = rules.follow.x.toward(samus.x, baby.place.x, velocity.x);
    let (y, vy) = rules.follow.y.toward(goal_height, baby.place.y, velocity.y);
    baby.place = Pair { x, y };
    *order = if vx.negative() {
        DiscoveryOrder::SceneFirst
    } else {
        DiscoveryOrder::SamusFirst
    };
    let cry_age = if cry < rules.last_cry {
        cry.saturating_add(1)
    } else {
        cry
    };
    baby.motion = BabyMotion::Following {
        velocity: Pair { x: vx, y: vy },
        cry: cry_age,
    };
    if cry_age != cry && (cry_age == rules.first_cry || cry_age == rules.last_cry) {
        BabyCall::Cries
    } else {
        BabyCall::Silent
    }
}
