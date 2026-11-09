//! What has happened in the world: the events and the bosses defeated, kept as sets. The
//! zebetites' count is a `Count` of `Progress`.

use clean_types::{BossKind, Event};

use crate::sets::flag_set;

flag_set! {
    /// Events that have happened. Once an event happens it has happened for the rest of the run;
    /// one that happens again changes nothing. Spec: EventsHappen.
    Events of Event {
        AcidReceded, AnimalsFreed, DraygonStatueGrey, GlassTunnelBroken, KraidStatueGrey,
        LavaQuakeOutrun, MetroidRoom1Empty, MetroidRoom2Empty, MetroidRoom3Empty,
        MetroidRoom4Empty, MotherBrainTankBroken, PhantoonStatueGrey, PlanetAwake,
        RidleyStatueGrey, ShaktoolPath, TimeBombSet, TourianOpen, TourianRoomsUnset,
    }
}

flag_set! {
    /// Kinds of boss: those defeated in an area, or those a room's version asks about. A boss is
    /// remembered by its kind in its area; one area's kind says nothing of another's.
    /// Spec: BossesDefeated.
    BossKinds of BossKind { MainBoss, MiniBoss, Torizo }
}
