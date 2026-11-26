//! A shaft actor's turn and its later drawing visit are separate operations.

use serde::{Deserialize, Serialize};

use crate::picture::{
    Displacement, DrawView, FigureId, FigureRequest, PlacementSlot, PreScrollView, RoomPoint,
    ShaftPictureOrder,
};

const SHAKE_REFRESH: u16 = 2;
const SHAKE_PERIOD: u16 = 4;
const SHAKE_LEFT_PHASE: u16 = 2;
const OVERLAY_RIGHT: i32 = 256;
const OVERLAY_TOP: i16 = -8;
const OVERLAY_BOTTOM: i16 = 248;

/// B8's actual participation result, independent of drawing visibility.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Participation {
    /// This actor took a turn.
    Ran,
    /// This actor did not take a turn.
    Skipped,
}

/// Spec: `ActorShake`. Retained separately for each stable shaft actor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActorClock {
    /// Participating turn count modulo 65536.
    pub count: u16,
    /// Remaining drawing visits that will spend shake.
    pub shake: u16,
}

impl ActorClock {
    /// Spec: `ActorShake`. Advance at participation, before any drawing visit.
    pub fn advance(&mut self, participation: Participation) {
        if participation == Participation::Ran {
            self.count = self.count.wrapping_add(1);
        }
    }

    /// Spec: `ActorShake`. After drawing, apply the shake request to an actor
    /// that took a turn, even if its draw visit was skipped. With no request,
    /// keep the shake remaining after drawing.
    pub fn refresh(&mut self, participation: Participation) {
        if participation == Participation::Ran {
            self.shake = SHAKE_REFRESH;
        }
    }

    /// Spend shake before the picture window is tested, using the updated count.
    const fn spend_shake(&mut self) -> i16 {
        if self.shake == 0 {
            return 0;
        }
        self.shake = self.shake.saturating_sub(1);
        if self.count % SHAKE_PERIOD < SHAKE_LEFT_PHASE {
            1
        } else {
            -1
        }
    }
}

/// Geometry and placement of an actor for which B8 supplies a shown figure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShaftActor {
    /// The overlay checks its unprojected centre before drawing.
    Overlay {
        /// Whole-pixel room centre.
        centre: RoomPoint,
        /// Horizontal half-extent in whole pixels.
        radius: u16,
    },
    /// A mounted puff projects its figure from a stable room placement.
    Vent {
        /// Stable room placement slot.
        slot: PlacementSlot,
        /// Whole-pixel mount position.
        mount: RoomPoint,
        /// Projected displacement from the mount.
        offset: Displacement,
    },
}

/// Spec: `ShaftActorDraw`. A scheduled visit for a participating, shown actor.
/// Carry this to the draw stage with the corresponding actor's clock.
#[derive(Debug, PartialEq, Eq)]
pub struct ActorDrawVisit {
    actor: ShaftActor,
    figure: FigureId,
}

impl ActorDrawVisit {
    /// Spec: `ShaftActorDraw`. Vents always receive a visit on shown turns;
    /// the overlay must also pass its inclusive pre-scroll bounds.
    #[must_use]
    pub fn schedule(
        participation: Participation,
        shown: Option<(ShaftActor, FigureId)>,
        view: PreScrollView,
    ) -> Option<Self> {
        let (actor, figure) = shown?;
        if participation == Participation::Skipped {
            return None;
        }
        if let ShaftActor::Overlay { centre, radius } = actor {
            let relative = centre.relative_to(view.0);
            let radius = i32::from(radius);
            if !(radius.saturating_neg()..=OVERLAY_RIGHT.saturating_add(radius))
                .contains(&i32::from(relative.x))
                || !(OVERLAY_TOP..=OVERLAY_BOTTOM).contains(&relative.y)
            {
                return None;
            }
        }
        Some(Self { actor, figure })
    }

    /// Spec: `ActorShake`, `PictureWindow`. Resolve the visit against the draw
    /// view, spending shake even when the vent's projected origin is clipped.
    /// The overlay uses ordinary sprite placement without the puff window.
    #[must_use]
    pub fn draw(self, clock: &mut ActorClock, view: DrawView) -> Option<FigureRequest> {
        let shift = Displacement {
            dx: clock.spend_shake(),
            dy: 0,
        };
        let (origin, order) = match self.actor {
            ShaftActor::Overlay { centre, .. } => (
                centre.relative_to(view.0).shifted(shift),
                ShaftPictureOrder::Overlay,
            ),
            ShaftActor::Vent {
                slot,
                mount,
                offset,
            } => {
                let origin = mount.relative_to(view.0).shifted(offset).shifted(shift);
                if !origin.in_picture_window() {
                    return None;
                }
                (origin, ShaftPictureOrder::Vent(slot))
            }
        };
        Some(FigureRequest {
            figure: self.figure,
            origin,
            order,
        })
    }
}
