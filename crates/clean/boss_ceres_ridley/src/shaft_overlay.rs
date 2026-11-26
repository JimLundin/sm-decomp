//! The escape overlay: placement, drawing, and the rumble that enables shaft rotation.

use core::num::Wrapping;

use types::ceres_shaft::{BurstPoint, BurstWait, RumbleTurns, ShaftOverlay, ShaftOverlayPhase};
use types::{Pair, Px};

use crate::lights::{Lights, LightsInputs};
use crate::rules::Rules;
use crate::station::{DoorColours, DoorPalette, Stage};

const SMOKE_LIMIT: u16 = 16_384;
const EXPLOSION_QUEUE_LIMIT: u8 = 6;

/// Spec: `OverlayAppears`. Upload the shaft overlay's content to the sprite plane.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OverlaySpriteUpload;

/// Spec: `OverlayAppears`. The initial overlay, sprite upload and door colours.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OverlayPlacement {
    /// State to retain for the actor's turns.
    pub overlay: ShaftOverlay,
    /// Overlay tile upload for the renderer, using the room's content.
    pub upload: OverlaySpriteUpload,
    /// Door colours for the passage fade.
    pub door_colours: DoorColours,
}

/// Spec: `OverlayAppears`. Place the overlay and issue its placement requests only during escape.
#[must_use]
pub const fn place_shaft_overlay(stage: Stage, anchor: Pair<Px>) -> Option<OverlayPlacement> {
    if !matches!(stage, Stage::Escape) {
        return None;
    }
    Some(OverlayPlacement {
        overlay: ShaftOverlay {
            anchor,
            phase: ShaftOverlayPhase::Opening,
        },
        upload: OverlaySpriteUpload,
        door_colours: DoorColours {
            set: stage.door_colours(),
            palette: DoorPalette::Doors,
        },
    })
}

/// Spec: `OverlayRumbles`. An effect request for the dust and explosion player.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RumbleParticle {
    /// Smoke from a draw below the smoke limit.
    Smoke,
    /// A small explosion from any other draw.
    SmallExplosion,
}

/// Spec: `OverlayRumbles`. A burst and its accompanying explosion sound.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RumbleBurst {
    /// What the effect player should spawn.
    pub particle: RumbleParticle,
    /// Effect centre in whole room pixels.
    pub centre: Pair<Px>,
    /// Queue limit for the explosion sound, even when the particle is smoke.
    pub sound_limit: u8,
}

/// Spec: `OverlayRumbles`, `ShaftTilts`. The rumble has ended and the shaft may turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StartShaftRotation;

/// Spec: `OverlayRumbles`. Current figure and any requests issued by the actor's turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OverlayTurn {
    /// Draw the overlay's single figure at this anchor, or omit it when hidden.
    pub figure: Option<Pair<Px>>,
    /// Lights updates, if granted a turn. The platform look only updates the low part of the
    /// affine room tilemap.
    pub lights: Option<Lights>,
    /// Particle and sound request, only on a burst turn.
    pub burst: Option<RumbleBurst>,
    /// Rotation starts on the shaft's subsequent turn, which can be in this same cut.
    pub start_rotation: Option<StartShaftRotation>,
}

/// Spec: `OverlayRumbles`. The overlay figure is visible throughout the rumble.
#[must_use]
pub const fn overlay_figure(overlay: &ShaftOverlay) -> Option<Pair<Px>> {
    match overlay.phase {
        ShaftOverlayPhase::Rumbling { .. } => Some(overlay.anchor),
        ShaftOverlayPhase::Opening | ShaftOverlayPhase::Hidden => None,
    }
}

/// Spec: `OverlayRumbles`. Call only on an overlay actor turn. The random source is invoked
/// once, and only when a burst is due. Without a turn the caller retains the state and figure.
pub fn turn_shaft_overlay(
    overlay: &mut ShaftOverlay,
    lights: LightsInputs,
    draw_random: impl FnOnce() -> u16,
    rules: &Rules,
) -> OverlayTurn {
    let mut out = OverlayTurn {
        figure: None,
        lights: None,
        burst: None,
        start_rotation: None,
    };
    overlay.phase = match overlay.phase {
        ShaftOverlayPhase::Opening => {
            out.lights = Some(Lights::step(lights, rules));
            ShaftOverlayPhase::Rumbling {
                remaining: RumbleTurns::MAX,
                wait: BurstWait::ZERO,
                point: BurstPoint::First,
            }
        }
        ShaftOverlayPhase::Rumbling {
            remaining: RumbleTurns::ZERO,
            ..
        } => {
            out.start_rotation = Some(StartShaftRotation);
            ShaftOverlayPhase::Hidden
        }
        ShaftOverlayPhase::Rumbling {
            remaining,
            wait: BurstWait::ZERO,
            point,
        } => {
            out.burst = Some(burst(overlay.anchor, point, draw_random()));
            ShaftOverlayPhase::Rumbling {
                remaining: remaining.count_down(),
                wait: BurstWait::MAX,
                point: next_point(point),
            }
        }
        ShaftOverlayPhase::Rumbling {
            remaining,
            wait,
            point,
        } => ShaftOverlayPhase::Rumbling {
            remaining: remaining.count_down(),
            wait: wait.count_down(),
            point,
        },
        ShaftOverlayPhase::Hidden => {
            out.lights = Some(Lights::step(lights, rules));
            ShaftOverlayPhase::Hidden
        }
    };
    out.figure = overlay_figure(overlay);
    out
}

fn burst(anchor: Pair<Px>, point: BurstPoint, random: u16) -> RumbleBurst {
    let (x, y) = burst_offset(point);
    RumbleBurst {
        particle: if random < SMOKE_LIMIT {
            RumbleParticle::Smoke
        } else {
            RumbleParticle::SmallExplosion
        },
        centre: Pair {
            x: anchor.x + Wrapping(x.cast_unsigned()),
            y: anchor.y + Wrapping(y.cast_unsigned()),
        },
        sound_limit: EXPLOSION_QUEUE_LIMIT,
    }
}

const fn burst_offset(point: BurstPoint) -> (i16, i16) {
    match point {
        BurstPoint::First => (2, 12),
        BurstPoint::Second => (-2, 22),
        BurstPoint::Third => (0, 4),
        BurstPoint::Fourth => (-4, -8),
    }
}

const fn next_point(point: BurstPoint) -> BurstPoint {
    match point {
        BurstPoint::First => BurstPoint::Second,
        BurstPoint::Second => BurstPoint::Third,
        BurstPoint::Third => BurstPoint::Fourth,
        BurstPoint::Fourth => BurstPoint::First,
    }
}
