//! Carry C1's drawing clocks beside the actual B8 actor turns.

use boss_ceres_ridley::{MountedVent, OverlayTurn, PuffFigure, VentFacing, VentPhase};
use canonical::{
    Canonical,
    model::{Model, Pinned},
};
use enemy_interactions::actor::{ActorClock, ActorDrawVisit, Participation, ShaftActor};
use enemy_interactions::picture::{
    Displacement, DrawView, FigureId, FigureRequest, PlacementSlot, PreScrollView, RoomPoint,
};
use types::{Pair, Px};

pub(super) struct ActorClockPin;
impl Pinned for ActorClockPin {
    type State = ActorClock;
    const NAME: &'static str = "ceres-shaft-actor-clock";
    const SHAPE: u64 = 0xc46e_ad70_1ff6_9d41;
}

/// A mounted actor's clock and its stable slot in the room.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct VentClock {
    /// Slot assigned by the room population.
    pub slot: u16,
    /// Participation and shake count for this slot.
    pub(super) clock: Model<ActorClockPin>,
}

impl VentClock {
    /// The clock state paired with this mount's slot.
    #[must_use]
    pub const fn state(&self) -> ActorClock {
        self.clock.0
    }
}

#[derive(Debug, Default)]
pub(in crate::production::played) struct ActorPictures {
    pub(in crate::production::played) overlay: Option<FigureRequest>,
    pub(in crate::production::played) vents: Vec<(VentFacing, PuffFigure, FigureRequest)>,
}

pub(in crate::production::played) const fn room_point(point: Pair<Px>) -> RoomPoint {
    RoomPoint {
        x: point.x.0,
        y: point.y.0,
    }
}

impl ActorPictures {
    pub(in crate::production::played) fn resolve(
        clocks: &mut Clocks,
        overlay: OverlayTurn,
        radius: u16,
        vents: impl Iterator<Item = (MountedVent, Participation)>,
        views: (PreScrollView, DrawView),
        refresh: bool,
    ) -> Self {
        let (before, draw) = views;
        let shown = overlay.figure.map(|centre| {
            (
                ShaftActor::Overlay {
                    centre: room_point(centre),
                    radius,
                },
                FigureId(0),
            )
        });
        let picture = ActorDrawVisit::schedule(Participation::Ran, shown, before)
            .and_then(|visit| visit.draw(&mut clocks.overlay.0, draw));
        if refresh {
            clocks.overlay.0.refresh(Participation::Ran);
        }
        let vents = vents
            .zip(&mut clocks.vents)
            .filter_map(|((vent, turn), slot)| {
                let request = vent.vent.shown_puff_figure().and_then(|figure| {
                    let narrow = |value: i32| {
                        let [lo, hi, ..] = value.to_le_bytes();
                        i16::from_le_bytes([lo, hi])
                    };
                    let actor = ShaftActor::Vent {
                        slot: PlacementSlot(usize::from(slot.slot)),
                        mount: room_point(vent.vent.position),
                        offset: Displacement {
                            dx: narrow(vent.offset.0.x),
                            dy: narrow(vent.offset.0.y),
                        },
                    };
                    let visit = ActorDrawVisit::schedule(
                        turn,
                        Some((actor, FigureId(u16::from(figure.get())))),
                        before,
                    )?;
                    let request = visit.draw(&mut slot.clock.0, draw)?;
                    Some((vent.vent.facing, figure, request))
                });
                if refresh {
                    slot.clock.0.refresh(turn);
                }
                request
            })
            .collect();
        Self {
            overlay: picture,
            vents,
        }
    }
}

/// The overlay and mounted actor clocks, in placement order.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct Clocks {
    /// The overlay's participation and shake clock.
    pub(super) overlay: Model<ActorClockPin>,
    /// Mounted vent participation and shake clocks.
    pub vents: Vec<VentClock>,
}

impl Clocks {
    pub(in crate::production::played) fn placed(
        vents: &[usize],
    ) -> Result<Self, crate::production::played::Fault> {
        if vents
            .windows(2)
            .any(|pair| matches!(pair, [left, right] if left >= right))
        {
            return Err(crate::production::played::Fault::Escape);
        }
        let vents = vents
            .iter()
            .map(|&slot| {
                let slot =
                    u16::try_from(slot).map_err(|_| crate::production::played::Fault::Escape)?;
                Ok(VentClock {
                    slot,
                    clock: Model(ActorClock::default()),
                })
            })
            .collect::<Result<_, _>>()?;
        Ok(Self {
            overlay: Model(ActorClock::default()),
            vents,
        })
    }

    /// The overlay clock state.
    #[must_use]
    pub const fn overlay(&self) -> ActorClock {
        self.overlay.0
    }
}

// Every eligible B8 step consumes a distinct wait/hold or changes the phase.
// Even a hidden vent that cannot hit leaves this receipt.
pub(super) fn participation(before: VentPhase, after: VentPhase) -> Participation {
    if before == after {
        Participation::Skipped
    } else {
        Participation::Ran
    }
}

#[cfg(test)]
mod tests {
    use super::{ActorClockPin, participation};

    #[test]
    fn shaft_actor_clock_shape_stays_pinned() {
        assert_eq!(canonical::model::pin_holds::<ActorClockPin>(), Ok(()));
    }
    use boss_ceres_ridley::{
        FirstPuffWait, MountedVent, Puff, PuffFigure, Rules, ShaftView, Vent, VentFacing, VentPhase,
    };
    use core::num::{NonZeroU8, Wrapping};
    use enemy_interactions::actor::Participation;
    use types::Pair;

    #[test]
    fn each_legal_phase_change_receipts_exactly_one_eligible_vent_turn() {
        let origin = Pair {
            x: Wrapping(0),
            y: Wrapping(0),
        };
        let outside = Pair {
            x: Wrapping(300),
            y: Wrapping(0),
        };
        let view = ShaftView {
            matrix: [256, 0, 0, 256],
            pivot: origin,
        };
        let mut phases = Vec::new();
        for wait in (1..=32).map(|n| FirstPuffWait::new(n).unwrap()) {
            phases.extend([VentPhase::Placed(wait), VentPhase::AwaitingFirstPuff(wait)]);
        }
        for figure in (1..=7).map(|n| PuffFigure::new(n).unwrap()) {
            for hold in 1..=3 {
                phases.push(VentPhase::Puffing(Puff::new(figure, hold).unwrap()));
            }
        }
        phases.extend((1..=255).map(|n| VentPhase::Resting(NonZeroU8::new(n).unwrap())));
        for rest in (1..=255).map(|n| NonZeroU8::new(n).unwrap()) {
            let rules = Rules {
                steam_rest: rest,
                ..Rules::default()
            };
            for phase in &phases {
                let mut vent = MountedVent::new(Vent {
                    facing: VentFacing::Left,
                    position: origin,
                    phase: *phase,
                });
                assert_eq!(vent.run(outside, view, None, &rules), None);
                assert_eq!(
                    participation(*phase, vent.vent.phase),
                    Participation::Skipped
                );
                assert_eq!(vent.run(origin, view, None, &rules), None);
                assert_eq!(participation(*phase, vent.vent.phase), Participation::Ran);
            }
        }
    }
}
