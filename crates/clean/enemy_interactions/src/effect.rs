//! Stationary shaft bursts: allocation, single-play clips and figure requests.

use core::num;
use std::cmp::Reverse;

use enum_map::Enum;
use serde::{Deserialize, Serialize};

use crate::picture::{
    Displacement, DrawView, EffectSlot, FigureId, FigureRequest, PreScrollView, RoomPoint,
    ShaftPictureOrder,
};

const EFFECT_SLOT_COUNT: usize = 18;
const LIFETIME_MAX: i16 = 255;

/// Spec: `EffectSlots`. Only the shaft's two stationary burst kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum BurstKind {
    /// Stationary smoke.
    Smoke,
    /// Stationary explosion.
    SmallExplosion,
}

/// Spec: `EffectSlots`. Arrival from B8, with its fixed room centre.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Burst {
    /// Which clip to play.
    pub kind: BurstKind,
    /// Where it stays for its lifetime.
    pub centre: RoomPoint,
}

/// Spec: `EffectClip`. An AF3 figure and its positive duration in advancing turns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClipFrame {
    /// AF3's figure identity.
    pub figure: FigureId,
    /// Full time for this figure.
    pub duration: num::NonZeroU16,
}

/// Spec: `EffectClip`. Nonempty borrowed clip; the caller owns the content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EffectClip<'a> {
    /// Initial figure.
    pub first: ClipFrame,
    /// Remaining figures in playback order.
    pub rest: &'a [ClipFrame],
}

impl EffectClip<'_> {
    fn frame(self, index: usize) -> Option<ClipFrame> {
        index.checked_sub(1).map_or(Some(self.first), |tail_index| {
            self.rest.get(tail_index).copied()
        })
    }
}

/// AF3's clips by burst kind.
pub type EffectClips<'a> = enum_map::EnumMap<BurstKind, EffectClip<'a>>;

/// Spec: `EffectClip`. Retained playback state, without copying the art.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectPhase {
    /// Allocated, but not yet visited by the effect runner.
    Waiting,
    /// Showing a figure already selected by the effect runner.
    Showing {
        /// Zero-based index of the current figure within the clip.
        frame: usize,
        /// Positive time remaining for that figure.
        remaining: num::NonZeroU16,
    },
}

impl EffectPhase {
    fn advance(self, clip: EffectClip<'_>) -> Option<Self> {
        let frame = match self {
            Self::Waiting => 0,
            Self::Showing { frame, remaining } => {
                if let Some(remaining) = num::NonZeroU16::new(remaining.get().saturating_sub(1)) {
                    return Some(Self::Showing { frame, remaining });
                }
                frame.checked_add(1)?
            }
        };
        Some(Self::Showing {
            frame,
            remaining: clip.frame(frame)?.duration,
        })
    }
}

/// A retained stationary effect in one occupied slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveEffect {
    /// Kind and fixed centre.
    pub burst: Burst,
    /// Current playback state.
    pub phase: EffectPhase,
}

impl ActiveEffect {
    /// Spec: `EffectClip`. Cull against the pre-scroll view before playback.
    fn advance(mut self, view: PreScrollView, clip: EffectClip<'_>) -> Option<Self> {
        let origin = self.burst.centre.relative_to(view.0);
        if !(0..=LIFETIME_MAX).contains(&origin.x) || !(0..=LIFETIME_MAX).contains(&origin.y) {
            return None;
        }
        self.phase = self.phase.advance(clip)?;
        Some(self)
    }
}

/// Spec: `EffectSlots`. Stable places the caller can save and restore directly.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectSlots(pub [Option<ActiveEffect>; EFFECT_SLOT_COUNT]);

impl EffectSlots {
    /// Spec: `EffectSlots`. Accept arrivals in order into the highest empty
    /// places. Call before advancing; a full pool drops excess arrivals.
    pub fn accept(&mut self, bursts: impl IntoIterator<Item = Burst>) {
        let empty = self.0.iter_mut().rev().filter(|slot| slot.is_none());
        for (slot, burst) in empty.zip(bursts) {
            *slot = Some(ActiveEffect {
                burst,
                phase: EffectPhase::Waiting,
            });
        }
    }

    /// Spec: `EffectClip`. Advance all occupied slots, including new arrivals.
    /// The caller advances the pool only on a fresh turn with the effect runner enabled.
    pub fn advance(&mut self, view: PreScrollView, clips: &EffectClips<'_>) {
        for slot in &mut self.0 {
            *slot = slot.and_then(|effect| effect.advance(view, clips[effect.burst.kind]));
        }
    }

    /// Spec: `PictureWindow`, `ShaftPictureOrder`. Request visible figures from
    /// highest slot to lowest, without advancing playback or checking lifetime.
    pub fn pictures<'a>(
        &'a self,
        view: DrawView,
        shake: Displacement,
        clips: &'a EffectClips<'_>,
    ) -> impl Iterator<Item = FigureRequest> + 'a {
        self.0
            .iter()
            .enumerate()
            .rev()
            .filter_map(move |(slot, effect)| {
                let effect = (*effect)?;
                let EffectPhase::Showing { frame, .. } = effect.phase else {
                    return None;
                };
                let origin = effect.burst.centre.relative_to(view.0).shifted(shake);
                if !origin.in_picture_window() {
                    return None;
                }
                Some(FigureRequest {
                    figure: clips[effect.burst.kind].frame(frame)?.figure,
                    origin,
                    order: ShaftPictureOrder::Effect(Reverse(EffectSlot(slot))),
                })
            })
    }
}
