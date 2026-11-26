//! Connect the shaft's burst requests and AF3 clips to the stationary effect pool.

use super::EscapeShaft;
use crate::production::played::Fault;
use crate::state::video::NmiBatch;
use boss_ceres_ridley::{RumbleBurst, RumbleParticle};
use canonical::{
    Canonical,
    model::{Model, Pinned},
};
use data::assets::{
    fingerprint::ContentFingerprint,
    room_effects::{Picture, ShaftBurst},
};
use data::spritemap::SpriteSize;
use enemy_interactions::{
    effect::{
        ActiveEffect, Burst, BurstKind, ClipFrame, EffectClip, EffectClips, EffectPhase,
        EffectSlots,
    },
    picture::{
        Displacement, DrawView, FigureId, FigureRequest, PreScrollView, RoomPoint,
        ShaftPictureOrder,
    },
};
use types::{Pair, Px};

/// Authenticated sprite pictures and their figure identities for the effect player.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BurstArt {
    art: ShaftBurst,
    smoke: [ClipFrame; 4],
    explosion: [ClipFrame; 6],
    fingerprint: ContentFingerprint,
}

impl BurstArt {
    /// Assign each imported picture a figure identity in its clip's order.
    #[must_use]
    pub fn new(art: ShaftBurst) -> Self {
        let mut next = 0_u16;
        let mut frame = |picture: &Picture| {
            let figure = FigureId(next);
            next = next.saturating_add(1);
            ClipFrame {
                figure,
                duration: picture.hold,
            }
        };
        let smoke = art.smoke.each_ref().map(&mut frame);
        let explosion = art.small_explosion.each_ref().map(frame);
        let fingerprint = fingerprint(&art);
        Self {
            art,
            smoke,
            explosion,
            fingerprint,
        }
    }

    fn clips(&self) -> EffectClips<'_> {
        let [smoke, smoke_rest @ ..] = &self.smoke;
        let [explosion, explosion_rest @ ..] = &self.explosion;
        enum_map::enum_map! {
            BurstKind::Smoke => EffectClip { first: *smoke, rest: smoke_rest },
            BurstKind::SmallExplosion => EffectClip { first: *explosion, rest: explosion_rest },
        }
    }

    fn admits(&self, effect: ActiveEffect) -> bool {
        let EffectPhase::Showing { frame, remaining } = effect.phase else {
            return true;
        };
        let frames = match effect.burst.kind {
            BurstKind::Smoke => self.smoke.as_slice(),
            BurstKind::SmallExplosion => self.explosion.as_slice(),
        };
        frames
            .get(frame)
            .is_some_and(|frame| remaining <= frame.duration)
    }

    /// Append one requested figure to the shared batch, or return none for a foreign figure.
    #[must_use]
    pub fn draw(&self, batch: &mut NmiBatch, figure: FigureRequest) -> Option<()> {
        if !matches!(figure.order, ShaftPictureOrder::Effect(_)) {
            return None;
        }
        let picture = self
            .art
            .smoke
            .iter()
            .chain(&self.art.small_explosion)
            .nth(usize::from(figure.figure.0))?;
        batch.draw_room_effect(
            &picture.sprites,
            Pair {
                x: i32::from(figure.origin.x),
                y: i32::from(figure.origin.y),
            },
        );
        Some(())
    }
}

fn fingerprint(art: &ShaftBurst) -> ContentFingerprint {
    let mut bytes = Vec::new();
    for picture in art.smoke.iter().chain(&art.small_explosion) {
        bytes.extend(picture.hold.get().to_le_bytes());
        bytes.extend(
            u64::try_from(picture.sprites.entries.len())
                .unwrap_or(u64::MAX)
                .to_le_bytes(),
        );
        for entry in &picture.sprites.entries {
            bytes.extend(entry.x.to_le_bytes());
            bytes.push(entry.y);
            bytes.push(u8::from(entry.size == SpriteSize::Large));
            bytes.extend(entry.attributes.tile.to_le_bytes());
            bytes.extend([
                entry.attributes.palette,
                entry.attributes.priority,
                u8::from(entry.attributes.flip_x),
                u8::from(entry.attributes.flip_y),
            ]);
        }
    }
    ContentFingerprint::of(&bytes)
}

struct EffectPoolPin;
impl Pinned for EffectPoolPin {
    type State = EffectSlots;
    const NAME: &'static str = "ceres-shaft-effect-slots";
    const SHAPE: u64 = 0xccae_9725_0e45_82dd;
}

/// The stationary effect pool and the clip content it was admitted against.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct ShaftEffects {
    slots: Model<EffectPoolPin>,
    art: Option<ContentFingerprint>,
}

impl Default for ShaftEffects {
    fn default() -> Self {
        Self {
            slots: Model(EffectSlots::default()),
            art: None,
        }
    }
}

impl ShaftEffects {
    /// All stable slots, as the clean model carries them.
    #[must_use]
    pub const fn slots(&self) -> &EffectSlots {
        &self.slots.0
    }

    fn empty(&self) -> bool {
        self.slots().0.iter().all(Option::is_none)
    }

    fn validate(&self, art: Option<&BurstArt>) -> Result<(), Fault> {
        match (self.art, art) {
            (None, _) if self.empty() => Ok(()),
            (Some(expected), Some(art))
                if expected == art.fingerprint
                    && self
                        .slots()
                        .0
                        .iter()
                        .flatten()
                        .all(|&effect| art.admits(effect)) =>
            {
                Ok(())
            }
            _ => Err(Fault::ShaftEffectContent),
        }
    }
}

impl EscapeShaft {
    pub(in crate::production::played) fn advance_effects(
        &mut self,
        arrival: Option<RumbleBurst>,
        view: Pair<Px>,
        art: Option<&BurstArt>,
    ) -> Result<(), Fault> {
        self.effects.validate(art)?;
        if arrival.is_none() && self.effects.empty() {
            return Ok(());
        }
        let art = art.ok_or(Fault::ShaftEffectContent)?;
        self.effects.art = Some(art.fingerprint);
        self.effects.slots.0.accept(arrival.map(|burst| Burst {
            kind: match burst.particle {
                RumbleParticle::Smoke => BurstKind::Smoke,
                RumbleParticle::SmallExplosion => BurstKind::SmallExplosion,
            },
            centre: room_point(burst.centre),
        }));
        self.effects
            .slots
            .0
            .advance(PreScrollView(room_point(view)), &art.clips());
        Ok(())
    }

    /// Request effect figures in draw order without changing the pool.
    ///
    /// # Errors
    /// Returns a fault when the pool or its admitted clip content is inconsistent.
    pub fn effect_pictures(
        &self,
        art: &BurstArt,
        view: Pair<Px>,
        shake: world_quake::Offset,
    ) -> Result<Vec<FigureRequest>, Fault> {
        self.effects.validate(Some(art))?;
        Ok(self
            .effects
            .slots()
            .pictures(
                DrawView(room_point(view)),
                Displacement {
                    dx: shake.x,
                    dy: shake.y,
                },
                &art.clips(),
            )
            .collect())
    }

    /// Restore the complete pool checkpoint against the clip art it was admitted with.
    ///
    /// # Errors
    /// Returns a fault for an invalid checkpoint or mismatched clip content.
    pub fn restore_effects(&mut self, bytes: &[u8], art: Option<&BurstArt>) -> Result<(), Fault> {
        let restored: ShaftEffects =
            canonical::from_bytes(bytes).map_err(|_| Fault::ShaftEffectContent)?;
        restored.validate(art)?;
        self.effects = restored;
        Ok(())
    }
}

const fn room_point(point: Pair<Px>) -> RoomPoint {
    RoomPoint {
        x: point.x.0,
        y: point.y.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pool_codec_retains_waiting_and_showing_slots_and_rejects_stale_frames() {
        use core::num::NonZeroU16;

        let picture = || Picture {
            hold: NonZeroU16::MIN,
            sprites: data::spritemap::Spritemap::default(),
        };
        let art = BurstArt::new(ShaftBurst {
            smoke: core::array::from_fn(|_| picture()),
            small_explosion: core::array::from_fn(|_| picture()),
        });
        let burst = Burst {
            kind: BurstKind::Smoke,
            centre: RoomPoint { x: 20, y: 30 },
        };
        let mut saved = ShaftEffects {
            art: Some(art.fingerprint),
            ..ShaftEffects::default()
        };
        saved.slots.0.0[17] = Some(ActiveEffect {
            burst,
            phase: EffectPhase::Waiting,
        });
        saved.slots.0.0[2] = Some(ActiveEffect {
            burst,
            phase: EffectPhase::Showing {
                frame: 3,
                remaining: NonZeroU16::MIN,
            },
        });
        let restored: ShaftEffects = canonical::from_bytes(&canonical::to_bytes(&saved)).unwrap();
        assert_eq!(restored, saved);
        assert_eq!(restored.validate(Some(&art)), Ok(()));

        for (frame, remaining) in [(4, 1), (3, 2)] {
            saved.slots.0.0[2] = Some(ActiveEffect {
                burst,
                phase: EffectPhase::Showing {
                    frame,
                    remaining: NonZeroU16::new(remaining).unwrap(),
                },
            });
            assert_eq!(saved.validate(Some(&art)), Err(Fault::ShaftEffectContent));
        }
    }

    #[test]
    fn slot_state_has_the_pinned_canonical_shape() {
        assert_eq!(canonical::model::pin_holds::<EffectPoolPin>(), Ok(()));
    }
}
