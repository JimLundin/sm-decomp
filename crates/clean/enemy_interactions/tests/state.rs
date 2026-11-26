//! Restored shaft state continues through the ordinary model operations.

use core::num;

use enemy_interactions::actor::{ActorClock, ActorDrawVisit, Participation, ShaftActor};
use enemy_interactions::effect::{
    ActiveEffect, Burst, BurstKind, ClipFrame, EffectClip, EffectPhase, EffectSlots,
};
use enemy_interactions::picture::{DrawView, FigureId, PreScrollView, RoomPoint, ScreenPoint};
use enum_map::enum_map;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct SavedPool {
    effects: EffectSlots,
}

#[test]
fn restored_clock_advances_and_spends_shake() -> Result<(), Box<dyn std::error::Error>> {
    let original = ActorClock { count: 1, shake: 2 };
    let saved = toml::to_string(&original)?;
    assert_eq!(saved, "count = 1\nshake = 2\n");
    let mut clock: ActorClock = toml::from_str(&saved)?;
    assert_eq!(clock, original);

    let centre = RoomPoint { x: 10, y: 20 };
    clock.advance(Participation::Ran);
    let picture = ActorDrawVisit::schedule(
        Participation::Ran,
        Some((ShaftActor::Overlay { centre, radius: 0 }, FigureId(1))),
        PreScrollView(centre),
    )
    .and_then(|visit| visit.draw(&mut clock, DrawView(centre)));
    assert_eq!(
        picture.map(|request| request.origin),
        Some(ScreenPoint { x: -1, y: 0 })
    );
    assert_eq!(clock, ActorClock { count: 2, shake: 1 });
    Ok(())
}

#[test]
fn restored_effect_continues_at_its_stored_frame() -> Result<(), Box<dyn std::error::Error>> {
    let centre = RoomPoint { x: 10, y: 20 };
    let view = PreScrollView(centre);
    let first = ClipFrame {
        figure: FigureId(1),
        duration: num::NonZeroU16::MIN,
    };
    let rest = [ClipFrame {
        figure: FigureId(2),
        duration: num::NonZeroU16::MIN,
    }];
    let clips = enum_map! {
        BurstKind::Smoke => EffectClip { first, rest: &[] },
        BurstKind::SmallExplosion => EffectClip { first, rest: &rest },
    };
    let mut original = EffectSlots::default();
    original.accept([Burst {
        kind: BurstKind::SmallExplosion,
        centre,
    }]);
    original.advance(view, &clips);
    original.advance(view, &clips);

    // TOML has no null value for empty array elements. Fill the remaining
    // slots with waiting smoke so the whole pool can round-trip in TOML.
    let smoke = Burst {
        kind: BurstKind::Smoke,
        centre,
    };
    original.accept(std::iter::repeat_n(smoke, original.0.len()));
    let saved_pool = SavedPool { effects: original };
    let saved = toml::to_string(&saved_pool)?;
    let SavedPool {
        effects: mut restored,
    } = toml::from_str(&saved)?;
    assert_eq!(restored, saved_pool.effects);
    assert!(matches!(
        restored.0.last(),
        Some(Some(ActiveEffect {
            phase: EffectPhase::Showing { frame: 1, .. },
            ..
        }))
    ));
    restored.advance(view, &clips);
    assert_eq!(restored.0.last(), Some(&None));
    assert!(matches!(
        restored.0.first(),
        Some(Some(ActiveEffect {
            phase: EffectPhase::Showing { frame: 0, .. },
            ..
        }))
    ));
    Ok(())
}
