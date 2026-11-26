use core::num;

use enemy_interactions::actor::{ActorClock, ActorDrawVisit, Participation, ShaftActor};
use enemy_interactions::effect::{
    ActiveEffect, Burst, BurstKind, ClipFrame, EffectClip, EffectClips, EffectPhase, EffectSlots,
};
use enemy_interactions::picture::{
    Displacement, DrawView, FigureId, FigureRequest, PlacementSlot, PreScrollView, RoomPoint,
    ScreenPoint, ShaftPictureOrder,
};
use enum_map::enum_map;
use serde::Deserialize;

use crate::{Entry, Source};

type Picture = (u16, i16, i16);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Vector {
    id: String,
    source: Source,
    pub(crate) entries: Vec<Entry>,
    observation: Option<String>,
    actor: Option<Clock>,
    #[serde(default)]
    vents: Vec<Vent>,
    overlay_actor: Option<OverlayActor>,
    #[serde(default)]
    refresh: bool,
    lifetime_view: Option<[u16; 2]>,
    draw_view: Option<[u16; 2]>,
    effect_shake: Option<[i16; 2]>,
    #[serde(default)]
    clips: Clips,
    all_slots: Option<SavedEffect>,
    #[serde(default)]
    effects: Vec<OccupiedSlot>,
    #[serde(default)]
    bursts: Vec<Arrival>,
    overlay: Option<Picture>,
    samus: Option<Picture>,
    after: After,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct After {
    actor: Option<Clock>,
    effects: Option<Vec<OccupiedSlot>>,
    draw: Option<Vec<Picture>>,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
struct Clock {
    count: u16,
    shake: u16,
}

impl From<Clock> for ActorClock {
    fn from(clock: Clock) -> Self {
        Self {
            count: clock.count,
            shake: clock.shake,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Vent {
    slot: usize,
    ran: bool,
    figure: Option<u16>,
    mount: Option<[u16; 2]>,
    offset: Option<[i16; 2]>,
}

impl Vent {
    fn shown(&self) -> Option<(ShaftActor, FigureId)> {
        Some((
            ShaftActor::Vent {
                slot: PlacementSlot(self.slot),
                mount: point(self.mount?),
                offset: displacement(self.offset?),
            },
            FigureId(self.figure?),
        ))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OverlayActor {
    ran: bool,
    figure: u16,
    centre: [u16; 2],
    radius: u16,
}

#[derive(Clone, Copy, Deserialize)]
enum EffectKind {
    Smoke,
    SmallExplosion,
}

impl From<EffectKind> for BurstKind {
    fn from(kind: EffectKind) -> Self {
        match kind {
            EffectKind::Smoke => Self::Smoke,
            EffectKind::SmallExplosion => Self::SmallExplosion,
        }
    }
}

#[derive(Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
struct Arrival {
    kind: EffectKind,
    centre: [u16; 2],
}

impl From<Arrival> for Burst {
    fn from(arrival: Arrival) -> Self {
        Self {
            kind: arrival.kind.into(),
            centre: point(arrival.centre),
        }
    }
}

#[derive(Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedEffect {
    kind: EffectKind,
    centre: [u16; 2],
    frame: Option<usize>,
    remaining: Option<num::NonZeroU16>,
}

impl SavedEffect {
    fn effect(self) -> ActiveEffect {
        assert_eq!(self.frame.is_some(), self.remaining.is_some());
        let phase = self
            .frame
            .zip(self.remaining)
            .map_or(EffectPhase::Waiting, |(frame, remaining)| {
                EffectPhase::Showing { frame, remaining }
            });
        ActiveEffect {
            burst: Arrival {
                kind: self.kind,
                centre: self.centre,
            }
            .into(),
            phase,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OccupiedSlot {
    slot: usize,
    kind: EffectKind,
    centre: [u16; 2],
    frame: Option<usize>,
    remaining: Option<num::NonZeroU16>,
}

fn restore(all_slots: Option<SavedEffect>, occupied: &[OccupiedSlot]) -> EffectSlots {
    let mut pool = EffectSlots::default();
    pool.0.fill(all_slots.map(SavedEffect::effect));
    for entry in occupied {
        let saved = SavedEffect {
            kind: entry.kind,
            centre: entry.centre,
            frame: entry.frame,
            remaining: entry.remaining,
        };
        assert!(entry.slot < pool.0.len());
        if let Some(slot) = pool.0.get_mut(entry.slot) {
            *slot = Some(saved.effect());
        }
    }
    pool
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Clips {
    #[serde(rename = "Smoke", default)]
    smoke: Vec<(u16, num::NonZeroU16)>,
    #[serde(rename = "SmallExplosion", default)]
    small_explosion: Vec<(u16, num::NonZeroU16)>,
}

fn frames(clip: &[(u16, num::NonZeroU16)]) -> Vec<ClipFrame> {
    clip.iter()
        .map(|&(figure, duration)| ClipFrame {
            figure: FigureId(figure),
            duration,
        })
        .collect()
}

fn clip(frames: &[ClipFrame]) -> EffectClip<'_> {
    // If a case omits a kind, supply a placeholder clip to complete the art table.
    frames.split_first().map_or(
        EffectClip {
            first: ClipFrame {
                figure: FigureId(u16::MAX),
                duration: num::NonZeroU16::MIN,
            },
            rest: &[],
        },
        |(&first, rest)| EffectClip { first, rest },
    )
}

const fn point([x, y]: [u16; 2]) -> RoomPoint {
    RoomPoint { x, y }
}

const fn displacement([dx, dy]: [i16; 2]) -> Displacement {
    Displacement { dx, dy }
}

const fn picture(request: FigureRequest) -> Picture {
    (request.figure.0, request.origin.x, request.origin.y)
}

impl Vector {
    fn actor_turn(
        &self,
        ran: bool,
        shown: Option<(ShaftActor, FigureId)>,
    ) -> (ActorClock, Option<FigureRequest>) {
        let participation = if ran {
            Participation::Ran
        } else {
            Participation::Skipped
        };
        let mut clock = self
            .actor
            .map_or_else(ActorClock::default, ActorClock::from);
        clock.advance(participation);
        let view = PreScrollView(point(self.lifetime_view.unwrap_or_default()));
        let visit = ActorDrawVisit::schedule(participation, shown, view);
        let request = visit
            .zip(self.draw_view)
            .and_then(|(visit, view)| visit.draw(&mut clock, DrawView(point(view))));
        if self.refresh {
            clock.refresh(participation);
        }
        (clock, request)
    }
}

fn actor_requests(vector: &Vector) -> Vec<FigureRequest> {
    let overlay = vector.overlay_actor.iter().map(|actor| {
        let geometry = ShaftActor::Overlay {
            centre: point(actor.centre),
            radius: actor.radius,
        };
        vector.actor_turn(actor.ran, Some((geometry, FigureId(actor.figure))))
    });
    let vents = vector
        .vents
        .iter()
        .map(|vent| vector.actor_turn(vent.ran, vent.shown()));
    let mut turns = overlay.chain(vents).peekable();
    if let Some(expected) = vector.after.actor {
        let actual = turns
            .peek()
            .map_or_else(ActorClock::default, |(clock, _)| *clock);
        assert_eq!(actual, expected.into(), "{} actor", vector.id);
    }
    turns.filter_map(|(_, request)| request).collect()
}

fn check_picture(vector: &Vector, mut requests: Vec<FigureRequest>) {
    if let Some((figure, x, y)) = vector.overlay {
        requests.push(FigureRequest {
            figure: FigureId(figure),
            origin: ScreenPoint { x, y },
            order: ShaftPictureOrder::Overlay,
        });
    }
    requests.sort_by_key(|request| request.order);
    let first_vent =
        requests.partition_point(|request| !matches!(request.order, ShaftPictureOrder::Vent(_)));
    let (before_samus, vents) = requests.split_at(first_vent);
    let actual: Vec<_> = before_samus
        .iter()
        .copied()
        .map(picture)
        .chain(vector.samus)
        .chain(vents.iter().copied().map(picture))
        .collect();
    if let Some(expected) = &vector.after.draw {
        assert_eq!(&actual, expected, "{} draw", vector.id);
    } else {
        assert!(actual.is_empty(), "{} should not draw", vector.id);
    }
}

#[test]
fn overlay_visit_survives_scrolling_without_rechecking_the_picture_window() {
    let centre = point([u16::MAX, 0]);
    let shown = (ShaftActor::Overlay { centre, radius: 1 }, FigureId(7));
    let mut clock = ActorClock { count: 1, shake: 2 };
    clock.advance(Participation::Ran);
    let pre_scroll = PreScrollView(point([0, 0]));
    let visit = ActorDrawVisit::schedule(Participation::Ran, Some(shown), pre_scroll);
    assert_eq!(clock, ActorClock { count: 2, shake: 2 });

    let mut restored = clock;
    let request = visit.and_then(|visit| visit.draw(&mut restored, DrawView(point([399, 0]))));
    assert_eq!(
        request,
        Some(FigureRequest {
            figure: FigureId(7),
            origin: ScreenPoint { x: -401, y: 0 },
            order: ShaftPictureOrder::Overlay,
        })
    );
    assert_eq!(restored, ActorClock { count: 2, shake: 1 });
}

pub fn run(vector: &Vector) {
    assert!(matches!(vector.source, Source::Constructed));
    assert!(
        vector
            .observation
            .as_ref()
            .is_none_or(|note| !note.is_empty())
    );
    let smoke = frames(&vector.clips.smoke);
    let small = frames(&vector.clips.small_explosion);
    let clips: EffectClips<'_> = enum_map! {
        BurstKind::Smoke => clip(&smoke),
        BurstKind::SmallExplosion => clip(&small),
    };
    let mut pool = restore(vector.all_slots, &vector.effects);
    pool.accept(vector.bursts.iter().copied().map(Burst::from));
    if let Some(view) = vector.lifetime_view {
        pool.advance(PreScrollView(point(view)), &clips);
    }
    if let Some(expected) = &vector.after.effects {
        assert_eq!(pool, restore(None, expected), "{} effects", vector.id);
    }
    let mut requests = actor_requests(vector);
    if let Some(view) = vector.draw_view {
        requests.extend(pool.pictures(
            DrawView(point(view)),
            displacement(vector.effect_shake.unwrap_or_default()),
            &clips,
        ));
    }
    check_picture(vector, requests);
}
